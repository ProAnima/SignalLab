//! The HTTP side of emulators: an HTTP/1.1 server on hyper that answers each
//! request by the first route it matches — a response picked by the route's
//! order, rendered with what arrived, after its delay, or a fault instead.
//! In a run the same server is what *Wait for HTTP request* listens to: every
//! request also goes to the run's inbox, routes or not. While its outage has
//! it down, no route is asked: every request meets the outage's fault.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::Bytes;
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::Incoming;
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE, RETRY_AFTER};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, StatusCode};
use hyper_util::rt::{TokioIo, TokioTimer};
use serde_json::{json, Map, Value};
use tokio::net::TcpListener;
use tokio::task::JoinSet;

use super::emulator::{DownFault, Fault, Response, HOLD};
use super::emulator_match::percent_decode;
use super::emulator_rules::Rules;
use super::emulator_state::{kept, Context, Down, Exchange};
use super::error::{EngineError, EngineResult, Field};
use super::inspect::Frame;
use super::matching::{shorten, Datagram};

/// The largest request body taken; a larger one gets 413.
pub const MAX_BODY: usize = 1024 * 1024;
/// Bytes of a body templates and waits read (`request.body`); `json` only when it fit.
const READ_BODY: usize = 64 * 1024;
/// Connections served at once; more are closed as they arrive.
const MAX_CONNECTIONS: usize = 512;
/// A client must send its request's head within this.
const HEAD_TIMEOUT: Duration = Duration::from_secs(30);
/// Body shown in an Inspector frame.
const FRAME_BODY: usize = 2_000;
/// Consecutive failed accepts after which the listener is given up.
const ACCEPT_FAILURES: u32 = 50;

/// What a fault does instead of an answer: hyper closes the connection.
#[derive(Debug)]
struct Closed;

impl std::fmt::Display for Closed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("closed by the emulator")
    }
}

impl std::error::Error for Closed {}

type Answer = Result<hyper::Response<Full<Bytes>>, Closed>;

/// Serve until accepting fails for good; the error says why.
pub(crate) async fn serve(listener: TcpListener, context: Arc<Context>) -> EngineError {
    let local = context.emulation.local;
    let active = Arc::new(AtomicUsize::new(0));
    // Owned here: when the server stops, every connection it serves stops with it.
    let mut connections = JoinSet::new();
    let mut failures = 0;
    loop {
        while connections.try_join_next().is_some() {}
        let (stream, peer) = match listener.accept().await {
            Ok(accepted) => accepted,
            // A client that gave up during the handshake, a moment without file handles: carry on.
            Err(error) => {
                failures += 1;
                if failures >= ACCEPT_FAILURES {
                    return EngineError::new("emulator.accept_failed").with("target", local).because(error);
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
                continue;
            }
        };
        failures = 0;
        if active.load(Ordering::Relaxed) >= MAX_CONNECTIONS {
            drop(stream);
            continue;
        }
        active.fetch_add(1, Ordering::Relaxed);
        let (context, active) = (context.clone(), active.clone());
        connections.spawn(async move {
            let service = service_fn(move |request| {
                let context = context.clone();
                async move { answer(&context, peer, request).await }
            });
            let _ = http1::Builder::new()
                .timer(TokioTimer::new())
                .header_read_timeout(HEAD_TIMEOUT)
                .serve_connection(TokioIo::new(stream), service)
                .await;
            active.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

/// `?a=1&b=x%20y` as an object; the first value of a repeated name.
fn query_of(query: &str) -> Value {
    let mut values = Map::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let name = percent_decode(name, true);
        values.entry(name).or_insert_with(|| Value::String(percent_decode(value, true)));
    }
    Value::Object(values)
}

/// The request as templates and waits read it.
fn request_value(parts: &hyper::http::request::Parts, body: &[u8], peer: SocketAddr) -> Value {
    let mut headers = Map::new();
    for (name, value) in &parts.headers {
        let text = String::from_utf8_lossy(value.as_bytes()).into_owned();
        match headers.get_mut(name.as_str()) {
            Some(Value::String(joined)) => {
                joined.push_str(", ");
                joined.push_str(&text);
            }
            _ => {
                headers.insert(name.as_str().to_string(), Value::String(text));
            }
        }
    }
    let read = &body[..body.len().min(READ_BODY)];
    let truncated = body.len() > READ_BODY;
    let json = if truncated || read.iter().all(u8::is_ascii_whitespace) { Value::Null } else { serde_json::from_slice(read).unwrap_or(Value::Null) };
    let mut value = json!({
        "method": parts.method.as_str(),
        "path": parts.uri.path(),
        "query": query_of(parts.uri.query().unwrap_or("")),
        "headers": headers,
        "body": String::from_utf8_lossy(read),
        "json": json,
        "params": {},
        "from": peer.to_string(),
    });
    if truncated {
        value["truncated"] = Value::Bool(true);
    }
    value
}

/// The request line: `GET /users/7?x=1`.
fn request_line(parts: &hyper::http::request::Parts) -> String {
    let target = parts.uri.path_and_query().map(|target| target.as_str()).unwrap_or("/");
    format!("{} {target}", parts.method)
}

fn preview(text: &str) -> String {
    let short: String = text.chars().take(FRAME_BODY).collect();
    if short.len() < text.len() { format!("{short}\n…") } else { short }
}

/// The request and the reply, as the Inspector's detail pane shows them.
fn detail(line: &str, request: &Value, reply: &str) -> String {
    let mut text = format!("{line}\n");
    if let Some(headers) = request["headers"].as_object() {
        for (name, value) in headers {
            text.push_str(&format!("{name}: {}\n", value.as_str().unwrap_or_default()));
        }
    }
    let body = request["body"].as_str().unwrap_or_default();
    if !body.is_empty() {
        text.push('\n');
        text.push_str(&preview(body));
        text.push('\n');
    }
    if !reply.is_empty() {
        text.push('\n');
        text.push_str(reply);
    }
    text
}

/// A plain answer for a request no rule could take a part in.
fn plain(status: StatusCode, body: Value) -> hyper::Response<Full<Bytes>> {
    let mut response = hyper::Response::new(Full::new(Bytes::from(body.to_string())));
    *response.status_mut() = status;
    response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

/// What a request no route takes gets without a fallback.
fn no_route() -> Response {
    Response { status: 404, headers: Vec::new(), body: r#"{"error":"no_route"}"#.into(), delay_ms: 0, jitter_ms: 0, fault: Fault::None, weight: 1 }
}

/// The content type a response gets when its headers give none: JSON when
/// the body parses, else text.
fn content_kind(headers: &[(String, String)], body: &str) -> Option<&'static str> {
    let typed = headers.iter().any(|(name, _)| name.trim().eq_ignore_ascii_case("content-type"));
    if typed || body.is_empty() {
        None
    } else if serde_json::from_str::<Value>(body).is_ok() {
        Some("application/json")
    } else {
        Some("text/plain; charset=utf-8")
    }
}

/// The first half of a body, on a character boundary.
fn half(body: &str) -> String {
    let mut end = body.len() / 2;
    while !body.is_char_boundary(end) {
        end -= 1;
    }
    body[..end].to_string()
}

/// A malformed answer's body: its first half, and an unclosed `{` when that
/// half still parses (a number) or nothing is left — JSON that never reads.
fn malformed(body: &str) -> String {
    let mut cut = half(body);
    if cut.trim().is_empty() || serde_json::from_str::<Value>(&cut).is_ok() {
        cut.push('{');
    }
    cut
}

/// A rendered response as hyper sends it, with `kind` as its content type when its headers give none.
fn build(status: u16, headers: &[(String, String)], body: String, kind: Option<&'static str>) -> EngineResult<hyper::Response<Full<Bytes>>> {
    let mut response = hyper::Response::new(Full::new(Bytes::from(body)));
    *response.status_mut() = StatusCode::from_u16(status).map_err(|_| EngineError::new("node.range").with("min", 100).with("max", 599).in_field(Field::new("status")))?;
    for (index, (name, value)) in headers.iter().enumerate() {
        let name = HeaderName::from_bytes(name.trim().as_bytes())
            .map_err(|_| EngineError::new("node.header_invalid").with("value", name).in_field(Field::nth("header_name", index + 1)))?;
        let value = HeaderValue::from_str(value).map_err(|_| EngineError::new("emulator.header_value_invalid").in_field(Field::nth("header_value", index + 1)))?;
        response.headers_mut().append(name, value);
    }
    if let Some(kind) = kind {
        response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(kind));
    }
    Ok(response)
}

/// How a sent response reads in protocol notation: `200 OK · 37 B`.
fn status_line(response: &hyper::Response<Full<Bytes>>, size: usize) -> String {
    let status = response.status();
    format!("{} {} · {size} B", status.as_u16(), status.canonical_reason().unwrap_or_default())
}

/// The reply as the detail pane shows it.
fn reply_text(response: &hyper::Response<Full<Bytes>>, body: &str) -> String {
    let status = response.status();
    let mut text = format!("HTTP/1.1 {} {}\n", status.as_u16(), status.canonical_reason().unwrap_or_default());
    for (name, value) in response.headers() {
        text.push_str(&format!("{name}: {}\n", String::from_utf8_lossy(value.as_bytes())));
    }
    if !body.is_empty() {
        text.push('\n');
        text.push_str(&preview(body));
    }
    text
}

/// A run's *Wait for HTTP request* steps hear every request, answered or not.
fn heard(context: &Context, peer: SocketAddr, started: Instant, body: &[u8], frame: Option<u64>, request: &Value) {
    if let Some(inbox) = &context.inbox {
        inbox.push(Datagram { bytes: body[..body.len().min(READ_BODY)].to_vec(), from: peer, at: started, topic: None, frame, request: Some(request.clone()) });
    }
}

/// A request that arrived while the emulator is down: what it meets then, no route.
async fn down(context: &Context, peer: SocketAddr, started: Instant, line: &str, body: &[u8], request: &Value, state: Down) -> Answer {
    let Down { fault, back } = state;
    let verdict = match fault {
        DownFault::Unavailable => "down → 503",
        DownFault::Reset => "down → reset",
        DownFault::Timeout => "down → timeout",
    };
    let frame = if context.capturing() {
        context.publish(Frame::rx("http", "emulator").remote(peer).payload(body).summary(format!("{line} {verdict}")).detail(detail(line, request, "")).verdict(verdict))
    } else {
        None
    };
    heard(context, peer, started, body, frame, request);
    let mut exchange = Exchange { from: peer.to_string(), request: shorten(line), frame, down: true, data: kept(request), ..Default::default() };
    match fault {
        DownFault::Unavailable => {
            let body = json!({ "error": "unavailable" });
            let size = body.to_string().len();
            let mut response = plain(StatusCode::SERVICE_UNAVAILABLE, body);
            // Whole seconds, at least one: when a client that honours it may try again —
            // known for an outage's schedule, not for one a step or a person caused.
            if let Some(back) = back {
                response.headers_mut().insert(RETRY_AFTER, HeaderValue::from(back.as_secs() + u64::from(back.subsec_nanos() > 0)));
            }
            exchange.status = Some(503);
            exchange.reply = status_line(&response, size);
            exchange.ms = started.elapsed().as_millis() as u64;
            context.emulation.record(exchange);
            Ok(response)
        }
        DownFault::Reset => {
            exchange.fault = Some(Fault::Reset);
            context.emulation.record(exchange);
            Err(Closed)
        }
        DownFault::Timeout => {
            exchange.fault = Some(Fault::Timeout);
            context.emulation.record(exchange);
            tokio::time::sleep(HOLD).await;
            Err(Closed)
        }
    }
}

async fn answer(context: &Context, peer: SocketAddr, request: Request<Incoming>) -> Answer {
    let started = Instant::now();
    let (parts, body) = request.into_parts();
    let line = request_line(&parts);
    let emulation = &context.emulation;
    let body = match Limited::new(body, MAX_BODY).collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(error) => {
            let too_large = error.downcast_ref::<http_body_util::LengthLimitError>().is_some();
            let (status, failure) = if too_large {
                (StatusCode::PAYLOAD_TOO_LARGE, EngineError::new("emulator.body_too_large").with("max", MAX_BODY / 1024))
            } else {
                (StatusCode::BAD_REQUEST, EngineError::new("emulator.body_failed").because(error))
            };
            emulation.record(Exchange { from: peer.to_string(), request: line, status: Some(status.as_u16()), error: Some(failure), ..Default::default() });
            return Ok(plain(status, json!({ "error": if too_large { "body_too_large" } else { "body_failed" } })));
        }
    };
    let mut request = request_value(&parts, &body, peer);
    if let Some(state) = context.down() {
        return down(context, peer, started, &line, &body, &request, state).await;
    }
    let Rules::Http { routes, fallback } = &context.compiled.rules else { return Err(Closed) };
    let found = routes.iter().enumerate().find_map(|(index, route)| route.matcher.accepts(&request).map(|params| (index, params)));
    let no_route = no_route();
    let (rule, count, picked, response) = match found {
        Some((index, params)) => {
            request["params"] = params;
            let count = emulation.hit(index);
            let (picked, response) = routes[index].pick(context.seed, index + 1, count);
            (Some(index + 1), count, Some(picked + 1), response)
        }
        None => (None, emulation.next_unmatched(), None, fallback.as_ref().unwrap_or(&no_route)),
    };
    let rendered = context.render(rule.unwrap_or(0), count, &request, |renderer| {
        let mut headers = Vec::with_capacity(response.headers.len());
        for (index, (name, value)) in response.headers.iter().enumerate() {
            let name = renderer.render(name).map_err(|error| error.in_field(Field::nth("header_name", index + 1)))?;
            let value = renderer.render(value).map_err(|error| error.in_field(Field::nth("header_value", index + 1)))?;
            headers.push((name, value));
        }
        let body = renderer.render(&response.body).map_err(|error| error.in_field(Field::new("body")))?;
        Ok((headers, body))
    });
    let built = rendered.and_then(|(headers, body)| {
        let kind = content_kind(&headers, &body);
        let body = if response.fault == Fault::Malformed { malformed(&body) } else { body };
        build(response.status, &headers, body.clone(), kind).map(|built| (built, body))
    });
    let built = built.map_err(|error| match picked {
        Some(picked) => error.with("response", picked),
        None => error,
    });

    // What happens, decided: the Inspector and the run's waits learn of the request now.
    let shown = |text: &str| match rule {
        Some(rule) => format!("#{rule} → {text}"),
        None => format!("— → {text}"),
    };
    let (verdict, reply) = match (&built, response.fault) {
        (Err(_), _) => (shown("failed"), String::new()),
        (Ok(_), Fault::Timeout) => (shown("timeout"), String::new()),
        (Ok(_), Fault::Reset) => (shown("reset"), String::new()),
        (Ok((built, body)), Fault::Malformed) => (shown(&format!("{} · cut", status_line(built, body.len()))), reply_text(built, body)),
        (Ok((built, body)), Fault::None) => (shown(&status_line(built, body.len())), reply_text(built, body)),
    };
    let frame = if context.capturing() {
        let frame = Frame::rx("http", "emulator").remote(peer).payload(&body).summary(format!("{line} {verdict}")).detail(detail(&line, &request, &reply)).verdict(verdict);
        context.publish(frame)
    } else {
        None
    };
    heard(context, peer, started, &body, frame, &request);
    let mut exchange = Exchange { from: peer.to_string(), request: shorten(&line), rule, frame, data: kept(&request), ..Default::default() };
    let delay = context.delay(rule.unwrap_or(0), count, response.delay_ms, response.jitter_ms);
    match built {
        Err(error) => {
            exchange.status = Some(500);
            let body = json!({ "error": error.code, "params": error.params });
            exchange.error = Some(error);
            emulation.record(exchange);
            Ok(plain(StatusCode::INTERNAL_SERVER_ERROR, body))
        }
        Ok(_) if response.fault == Fault::Timeout => {
            // Held, not answered: the client's own timeout is what is being tested.
            exchange.fault = Some(Fault::Timeout);
            emulation.record(exchange);
            tokio::time::sleep(HOLD).await;
            Err(Closed)
        }
        Ok(_) if response.fault == Fault::Reset => {
            tokio::time::sleep(delay).await;
            exchange.fault = Some(Fault::Reset);
            exchange.ms = started.elapsed().as_millis() as u64;
            emulation.record(exchange);
            Err(Closed)
        }
        Ok((built, body)) => {
            tokio::time::sleep(delay).await;
            exchange.status = Some(built.status().as_u16());
            exchange.reply = status_line(&built, body.len());
            if response.fault == Fault::Malformed {
                exchange.fault = Some(Fault::Malformed);
            }
            exchange.ms = started.elapsed().as_millis() as u64;
            emulation.record(exchange);
            Ok(built)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::Emulator;
    use crate::emulator_job::{self, EmulatorHub, StartOptions};
    use crate::host::{Host, Recorder};
    use crate::inspect::Capture;
    use crate::jobs::JobRegistry;

    fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
    }

    async fn started(document: Value) -> (JobRegistry, EmulatorHub, u64, String) {
        let host = Host::new(Recorder::new(), Capture::new());
        let (jobs, hub) = (JobRegistry::new(), EmulatorHub::new());
        let emulator: Emulator = serde_json::from_value(document).unwrap();
        let options = StartOptions { seed: Some(7), ..Default::default() };
        let info = emulator_job::start(host, jobs.clone(), hub.clone(), emulator, options).await.unwrap();
        let local = info.params["local"].clone();
        (jobs, hub, info.id, format!("http://{local}"))
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap()
    }

    #[tokio::test]
    async fn routes_answer_with_rendered_responses_in_sequence() {
        let port = free_port();
        let (jobs, hub, id, base) = started(json!({
            "name": "API", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
            "routes": [
                { "method": "GET", "path": "/users/:id", "responses": [{ "headers": [["X-User", "{{request.params.id}}"]], "body": "{\"id\":\"{{request.params.id}}\",\"page\":\"{{request.query.page}}\"}" }] },
                { "method": "POST", "path": "/echo", "responses": [{ "status": 201, "body": "{{request.json.name}} #{{counter}}" }] },
                { "path": "/flaky", "responses": [{ "status": 503 }, { "status": 503 }, { "body": "ok after {{counter}}" }] }
            ]
        }))
        .await;
        let client = client();
        let user = client.get(format!("{base}/users/7?page=2")).send().await.unwrap();
        assert_eq!((user.status().as_u16(), user.headers()["x-user"].to_str().unwrap(), user.headers()["content-type"].to_str().unwrap()), (200, "7", "application/json"));
        assert_eq!(user.text().await.unwrap(), r#"{"id":"7","page":"2"}"#);
        let echo = client.post(format!("{base}/echo")).body(r#"{"name":"Ada"}"#).send().await.unwrap();
        assert_eq!((echo.status().as_u16(), echo.text().await.unwrap()), (201, "Ada #1".to_string()));
        let statuses: Vec<u16> = futures_statuses(&client, &format!("{base}/flaky"), 4).await;
        assert_eq!(statuses, [503, 503, 200, 200]);
        let missing = client.get(format!("{base}/nowhere")).send().await.unwrap();
        assert_eq!((missing.status().as_u16(), missing.text().await.unwrap()), (404, r#"{"error":"no_route"}"#.to_string()));
        // A template that cannot be filled answers 500 and says why, and is counted as failed.
        let broken = client.post(format!("{base}/echo")).body("not json").send().await.unwrap();
        assert_eq!(broken.status().as_u16(), 500);
        assert_eq!(broken.json::<Value>().await.unwrap()["error"], "template.no_field");

        let snapshot = hub.snapshot(id, 0, 100).unwrap();
        assert_eq!(snapshot.counts.hits, [1, 2, 4]);
        assert_eq!((snapshot.counts.total, snapshot.counts.unmatched, snapshot.counts.failed), (8, 1, 1));
        let first = &snapshot.exchanges[0];
        assert_eq!((first.request.as_str(), first.rule, first.status, first.reply.as_str()), ("GET /users/7?page=2", Some(1), Some(200), "200 OK · 21 B"));
        assert_eq!(first.data["params"]["id"], "7");
        let failed = snapshot.exchanges.iter().find(|exchange| exchange.error.is_some()).unwrap();
        assert_eq!((failed.error.as_ref().unwrap().params["rule"].as_str(), failed.error.as_ref().unwrap().field.clone()), ("2", Some(Field::new("body"))));
        jobs.stop(id);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(client.get(format!("{base}/users/1")).send().await.is_err(), "stopped means the port is closed");
        assert_eq!(hub.get(id).err().unwrap().code, "emulator.not_running");
    }

    async fn futures_statuses(client: &reqwest::Client, url: &str, n: usize) -> Vec<u16> {
        let mut statuses = Vec::new();
        for _ in 0..n {
            statuses.push(client.get(url).send().await.unwrap().status().as_u16());
        }
        statuses
    }

    #[tokio::test]
    async fn faults_hold_or_close_and_delays_are_kept() {
        let port = free_port();
        let (jobs, hub, id, base) = started(json!({
            "name": "Faults", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
            "routes": [
                { "path": "/slow", "responses": [{ "delay_ms": 300, "body": "late" }] },
                { "path": "/hang", "responses": [{ "fault": "timeout" }] },
                { "path": "/reset", "responses": [{ "fault": "reset" }] }
            ],
            "fallback": { "status": 418, "body": "teapot {{request.method}}" }
        }))
        .await;
        let client = reqwest::Client::builder().timeout(Duration::from_millis(800)).build().unwrap();
        let begun = Instant::now();
        assert_eq!(client.get(format!("{base}/slow")).send().await.unwrap().text().await.unwrap(), "late");
        assert!(begun.elapsed() >= Duration::from_millis(300), "the delay is kept");
        let hang = client.get(format!("{base}/hang")).send().await.unwrap_err();
        assert!(hang.is_timeout(), "{hang}");
        let reset = client.get(format!("{base}/reset")).send().await.unwrap_err();
        assert!(!reset.is_timeout(), "closed, not held: {reset}");
        let teapot = client.delete(format!("{base}/x")).send().await.unwrap();
        assert_eq!((teapot.status().as_u16(), teapot.text().await.unwrap()), (418, "teapot DELETE".to_string()));
        let exchanges = hub.snapshot(id, 0, 10).unwrap().exchanges;
        let faults: Vec<Option<Fault>> = exchanges.iter().map(|exchange| exchange.fault).collect();
        assert_eq!(faults, [None, Some(Fault::Timeout), Some(Fault::Reset), None]);
        assert!(exchanges[0].ms >= 300);
        jobs.stop(id);
    }

    #[test]
    fn requests_read_as_templates_expect() {
        let request = Request::builder().method("PUT").uri("/a/b?x=1&x=2&y=a%20b").header("X-One", "1").header("x-one", "2").body(()).unwrap();
        let (parts, ()) = request.into_parts();
        let value = request_value(&parts, br#"{"k":[1,2]}"#, "127.0.0.1:9".parse().unwrap());
        assert_eq!(value["query"], json!({ "x": "1", "y": "a b" }));
        assert_eq!(value["headers"]["x-one"], "1, 2");
        assert_eq!((value["json"]["k"][1].clone(), value["method"].clone(), value["path"].clone()), (json!(2), json!("PUT"), json!("/a/b")));
        assert_eq!(request_line(&parts), "PUT /a/b?x=1&x=2&y=a%20b");
        let long = vec![b'x'; READ_BODY + 1];
        let cut = request_value(&parts, &long, "127.0.0.1:9".parse().unwrap());
        assert_eq!((cut["body"].as_str().unwrap().len(), cut["truncated"].clone(), cut["json"].clone()), (READ_BODY, json!(true), Value::Null));
        let headers = [("X-A".to_string(), "1".to_string())];
        let built = build(200, &headers, "plain".into(), content_kind(&headers, "plain")).unwrap();
        assert_eq!(built.headers()["content-type"], "text/plain; charset=utf-8");
        assert_eq!(content_kind(&[], "{\"a\":1}"), Some("application/json"));
        assert_eq!(content_kind(&[("content-type".into(), "x/y".into())], "{}"), None, "the route's own type is kept");
        assert!(build(200, &[("X-A".into(), "bad\nvalue".into())], String::new(), None).unwrap_err().is("emulator.header_value_invalid"));
        assert_eq!((half("{\"id\":7}"), half("ééé"), half("")), ("{\"id".to_string(), "é".to_string(), String::new()));
        assert_eq!((malformed("12345678"), malformed(""), malformed("{\"id\":7}")), ("1234{".to_string(), "{".to_string(), "{\"id".to_string()), "never JSON that parses");
    }

    #[tokio::test]
    async fn a_malformed_body_stops_halfway_and_says_it_is_json() {
        let port = free_port();
        let (jobs, hub, id, base) = started(json!({
            "name": "Broken", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
            "routes": [{ "path": "/order", "responses": [{ "fault": "malformed", "body": "{\"order\":\"A-17\",\"total\":12}" }] }]
        }))
        .await;
        let response = client().get(format!("{base}/order")).send().await.unwrap();
        assert_eq!((response.status().as_u16(), response.headers()["content-type"].to_str().unwrap()), (200, "application/json"));
        let text = response.text().await.unwrap();
        assert_eq!(text, "{\"order\":\"A-1");
        assert!(serde_json::from_str::<Value>(&text).is_err(), "a JSON client fails to read it");
        let exchange = &hub.snapshot(id, 0, 10).unwrap().exchanges[0];
        assert_eq!((exchange.fault, exchange.reply.as_str()), (Some(Fault::Malformed), "200 OK · 13 B"));
        jobs.stop(id);
    }

    #[tokio::test]
    async fn an_outage_answers_503_until_it_is_back_then_routes_again() {
        let port = free_port();
        let (jobs, hub, id, base) = started(json!({
            "name": "Flapping", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
            "routes": [{ "path": "/health", "responses": [{ "body": "ok" }] }],
            "outage": { "up_ms": 300, "down_ms": 400 }
        }))
        .await;
        let client = client();
        assert_eq!(client.get(format!("{base}/health")).send().await.unwrap().status().as_u16(), 200, "up first");
        tokio::time::sleep(Duration::from_millis(400)).await;
        let down = client.get(format!("{base}/health")).send().await.unwrap();
        assert_eq!((down.status().as_u16(), down.headers()["retry-after"].to_str().unwrap()), (503, "1"));
        assert_eq!(down.text().await.unwrap(), r#"{"error":"unavailable"}"#);
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(client.get(format!("{base}/health")).send().await.unwrap().status().as_u16(), 200, "back up");
        let counts = hub.snapshot(id, 0, 10).unwrap().counts;
        assert_eq!((counts.total, counts.down, counts.unmatched, counts.hits.clone()), (3, 1, 0, vec![2]));
        jobs.stop(id);

        // Closing or holding instead.
        let port = free_port();
        let (jobs, _hub, id, base) = started(json!({
            "name": "Gone", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
            "outage": { "up_ms": 10, "down_ms": 3_600_000, "fault": "reset" }
        }))
        .await;
        tokio::time::sleep(Duration::from_millis(30)).await;
        let reset = client.get(format!("{base}/x")).send().await.unwrap_err();
        assert!(!reset.is_timeout(), "closed: {reset}");
        jobs.stop(id);
    }
}
