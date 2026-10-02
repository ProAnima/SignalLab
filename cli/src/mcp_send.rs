//! The tools that put one message on the network — send_osc, send_udp,
//! send_http, send_mqtt — and `listen`, which shows what arrives on a port.

use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::error::EngineError;
use signal_lab_engine::http::HttpResponse;
use signal_lab_engine::osc_codec::{decode_packet, OscArg};
use signal_lab_engine::signals::RawPayload;

use crate::fail::{Exit, Failure};
use crate::mcp::{Answer, State};
use crate::params;
use crate::send::{datagram, hex_bytes, mqtt_publish, response_failure};

/// The longest response body or capture handed to the model.
pub(crate) const MAX_TEXT: usize = 16 * 1024;
/// Listening is bounded: a tool call should come back while the model waits.
const MAX_LISTEN: Duration = Duration::from_secs(60);

/// OSC arguments as JSON: numbers, strings, booleans, null, or {type, value}.
fn osc_args(given: &Value) -> Result<Vec<OscArg>, String> {
    let Some(items) = given.as_array() else {
        return if given.is_null() { Ok(Vec::new()) } else { Err("args is an array".into()) };
    };
    items
        .iter()
        .map(|item| match item {
            Value::Bool(flag) => Ok(OscArg::Bool(*flag)),
            Value::Null => Ok(OscArg::Nil),
            Value::String(text) => Ok(OscArg::Str(text.clone())),
            Value::Number(number) => match number.as_i64() {
                Some(int) => Ok(i32::try_from(int).map(OscArg::Int).unwrap_or(OscArg::Long(int))),
                None => Ok(OscArg::Float(number.as_f64().unwrap_or_default() as f32)),
            },
            Value::Object(_) => serde_json::from_value(item.clone()).map_err(|error| format!("{item} is not an OSC argument: {error}")),
            Value::Array(_) => Err(format!("{item} is not an OSC argument")),
        })
        .collect()
}

pub(crate) fn text_arg<'a>(arguments: &'a Value, name: &str) -> Result<&'a str, Answer> {
    arguments[name].as_str().filter(|text| !text.trim().is_empty()).ok_or_else(|| Answer::wrong(format!("{name} is required")))
}

pub(crate) async fn send_osc(state: &State, arguments: &Value) -> Answer {
    let (target, address) = match (text_arg(arguments, "target"), text_arg(arguments, "address")) {
        (Ok(target), Ok(address)) => (target, address),
        (Err(answer), _) | (_, Err(answer)) => return answer,
    };
    let args = match osc_args(&arguments["args"]) {
        Ok(args) => args,
        Err(text) => return Answer::wrong(text),
    };
    match state.engine.invoke("osc_send", json!({ "target": target, "address": address, "args": args })).await {
        Ok(bytes) => Answer::ok(
            state.ctx.texts.t("log.oscSent", &params! { "address" => address, "bytes" => bytes.as_u64().unwrap_or_default(), "target" => target }),
            json!({ "sent": true, "bytes": bytes }),
        ),
        Err(failure) => Answer::failed(state, &failure),
    }
}

pub(crate) async fn udp_send(state: &State, target: &str, payload: &RawPayload) -> Result<Value, Failure> {
    let result = state.engine.invoke("broadcast_send", datagram(target, payload)).await?;
    if result["errors"].as_u64().unwrap_or_default() > 0 {
        let error = serde_json::from_value::<EngineError>(result["error"].clone()).unwrap_or_else(|_| EngineError::new("transport.failed").with("target", target));
        return Err(Failure::sending(error));
    }
    Ok(result)
}

pub(crate) async fn send_udp(state: &State, arguments: &Value) -> Answer {
    let target = match text_arg(arguments, "target") {
        Ok(target) => target,
        Err(answer) => return answer,
    };
    let payload = match (arguments["text"].as_str(), arguments["hex"].as_str()) {
        (Some(text), None) => RawPayload::Text { text: text.into() },
        (None, Some(hex)) if hex_bytes(hex).is_some() => RawPayload::Hex { hex: hex.into() },
        (None, Some(hex)) => return Answer::wrong(format!("{hex} is not pairs of hex digits")),
        _ => return Answer::wrong("Give text or hex, one of them"),
    };
    match udp_send(state, target, &payload).await {
        Ok(result) => Answer::ok(state.ctx.texts.t("cli.sentUdp", &params! { "target" => target, "bytes" => result["bytes"].as_u64().unwrap_or_default() }), json!({ "sent": true, "result": result })),
        Err(failure) => Answer::failed(state, &failure),
    }
}

/// At most `MAX_TEXT` bytes of `text`, on a character boundary, said when cut.
pub(crate) fn clip(text: &str) -> String {
    if text.len() <= MAX_TEXT {
        return text.to_string();
    }
    let mut end = MAX_TEXT;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n… ({} more bytes)", &text[..end], text.len() - end)
}

pub(crate) async fn http(state: &State, request: Value, url: &str) -> Result<HttpResponse, Failure> {
    let value = state.engine.invoke("http_request", json!({ "request": request })).await?;
    let response: HttpResponse = serde_json::from_value(value).map_err(|error| Failure::sending(EngineError::new("command.reply_invalid").because(error)))?;
    match response_failure(&response, url) {
        Some(error) => Err(Failure { error, exit: Exit::Failed, remote: false }),
        None => Ok(response),
    }
}

pub(crate) async fn send_http(state: &State, arguments: &Value) -> Answer {
    let (method, url) = match (text_arg(arguments, "method"), text_arg(arguments, "url")) {
        (Ok(method), Ok(url)) => (method.to_ascii_uppercase(), url),
        (Err(answer), _) | (_, Err(answer)) => return answer,
    };
    let headers: Vec<(String, String)> = arguments["headers"].as_object().into_iter().flatten().map(|(name, value)| (name.clone(), value.as_str().map(str::to_string).unwrap_or_else(|| value.to_string()))).collect();
    let mut request = json!({ "method": method, "url": url, "headers": headers, "body": arguments["body"].as_str(), "timeout_ms": arguments["timeout_ms"].as_u64().unwrap_or(10_000) });
    if arguments["auth"].is_object() {
        request["auth"] = arguments["auth"].clone();
    }
    match http(state, request, url).await {
        Ok(response) => {
            let head: Vec<String> = response.headers.iter().map(|(name, value)| format!("{name}: {value}")).collect();
            let text = format!("HTTP {} {} · {} ms · {} bytes\n{}\n\n{}", response.status, response.status_text, response.latency_ms.round(), response.body_bytes, head.join("\n"), clip(&response.body));
            let mut data = json!({ "status": response.status, "status_text": response.status_text, "latency_ms": response.latency_ms, "headers": response.headers, "body": clip(&response.body), "body_bytes": response.body_bytes });
            if let Some(digest) = &response.digest {
                data["digest"] = json!(digest);
            }
            Answer::ok(text, data)
        }
        Err(failure) => Answer::failed(state, &failure),
    }
}

pub(crate) async fn send_mqtt(state: &State, arguments: &Value) -> Answer {
    let (broker, topic) = match (text_arg(arguments, "broker"), text_arg(arguments, "topic")) {
        (Ok(broker), Ok(topic)) => (broker, topic),
        (Err(answer), _) | (_, Err(answer)) => return answer,
    };
    let qos = arguments["qos"].as_u64().unwrap_or(0);
    if qos > 2 {
        return Answer::wrong("qos is 0, 1 or 2");
    }
    let args = match mqtt_publish(broker, topic, arguments["payload"].as_str().unwrap_or_default(), qos as u8, arguments["retain"].as_bool().unwrap_or(false)) {
        Ok(args) => args,
        Err(failure) => return Answer::failed(state, &failure),
    };
    match state.engine.invoke("mqtt_publish_once", args).await {
        Ok(summary) => Answer::ok(format!("Published {}", summary.as_str().unwrap_or_default()), json!({ "sent": true, "summary": summary })),
        Err(failure) => Answer::failed(state, &failure),
    }
}

pub(crate) async fn send_ws(state: &State, arguments: &Value) -> Answer {
    let url = match text_arg(arguments, "url") {
        Ok(url) => url,
        Err(answer) => return answer,
    };
    let headers: Vec<(String, String)> = arguments["headers"].as_object().into_iter().flatten().map(|(name, value)| (name.clone(), value.as_str().map(str::to_string).unwrap_or_else(|| value.to_string()))).collect();
    let protocols: Vec<&str> = arguments["protocols"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    let message = match (arguments["text"].as_str(), arguments["hex"].as_str()) {
        (Some(_), Some(_)) => return Answer::wrong("give text or hex, not both"),
        (Some(text), None) => Some(json!({ "text": text })),
        (None, Some(hex)) => Some(json!({ "hex": hex })),
        (None, None) => None,
    };
    let timeout = arguments["timeout_ms"].as_u64().unwrap_or(2_000);
    let expect = match (arguments["expect"].as_str(), arguments["expect_regex"].as_str(), arguments["wait"].as_bool().unwrap_or(false)) {
        (Some(text), _, _) => Some(json!({ "mode": "contains", "pattern": text, "timeout_ms": timeout })),
        (None, Some(regex), _) => Some(json!({ "mode": "regex", "pattern": regex, "timeout_ms": timeout })),
        (None, None, true) => Some(json!({ "mode": "any", "pattern": "", "timeout_ms": timeout })),
        _ => None,
    };
    let config = json!({ "url": url, "headers": headers, "protocols": protocols });
    match state.engine.invoke("ws_exchange", json!({ "config": config, "message": message, "expect": expect })).await {
        Ok(exchange) => {
            let handshake = &exchange["handshake"];
            let mut text = format!("Connected to {} in {} ms", handshake["url"].as_str().unwrap_or_default(), handshake["ms"]);
            if let Some(protocol) = handshake["protocol"].as_str() {
                text.push_str(&format!(", subprotocol {protocol}"));
            }
            if let Some(bytes) = exchange["sent"].as_u64() {
                text.push_str(&format!(" · sent {bytes} bytes"));
            }
            if let Some(reply) = exchange.get("reply").filter(|reply| !reply.is_null()) {
                let shown = if reply["kind"] == "binary" { reply["hex"].as_str() } else { reply["text"].as_str() };
                text.push_str(&format!("\nAnswer after {} ms:\n{}", reply["ms"], clip(shown.unwrap_or_default())));
            }
            Answer::ok(text, exchange)
        }
        Err(failure) => Answer::failed(state, &failure),
    }
}

pub(crate) async fn listen(state: &State, arguments: &Value) -> Answer {
    if state.engine.remote() {
        return Answer::wrong("listen runs on this machine; on a server, an experiment with a wait node listens there (run_experiment)");
    }
    let bind = match text_arg(arguments, "bind") {
        Ok(bind) => bind,
        Err(answer) => return answer,
    };
    let osc = arguments["protocol"].as_str().unwrap_or("osc") == "osc";
    let seconds = Duration::from_secs_f64(arguments["seconds"].as_f64().unwrap_or(5.0).clamp(0.1, MAX_LISTEN.as_secs_f64()));
    let max = arguments["max"].as_u64().unwrap_or(100).clamp(1, 1000) as usize;
    let address: std::net::SocketAddr = match bind.parse() {
        Ok(address) => address,
        Err(_) => return Answer::failed(state, &Failure::invalid(EngineError::new("node.bind_invalid").with("value", bind))),
    };
    let socket = match tokio::net::UdpSocket::bind(address).await {
        Ok(socket) => socket,
        Err(error) => return Answer::failed(state, &Failure::environment(signal_lab_engine::net::bind_error(bind, error))),
    };
    let started = Instant::now();
    let mut received = Vec::new();
    let mut buffer = vec![0u8; 65_536];
    while received.len() < max {
        let left = seconds.saturating_sub(started.elapsed());
        if left.is_zero() {
            break;
        }
        let (n, from) = match tokio::time::timeout(left, socket.recv_from(&mut buffer)).await {
            Ok(Ok(got)) => got,
            // Windows tells of an earlier send's "port unreachable" on the next receive: carry on.
            Ok(Err(error)) if signal_lab_engine::net::udp_transient(&error) => continue,
            Ok(Err(_)) | Err(_) => break,
        };
        let bytes = &buffer[..n];
        let ms = started.elapsed().as_millis() as u64;
        let entry = match osc.then(|| decode_packet(bytes)).and_then(Result::ok) {
            Some(messages) => json!({ "ms": ms, "from": from.to_string(), "osc": messages.iter().map(|message| json!({ "address": message.address, "args": message.args })).collect::<Vec<_>>() }),
            None => json!({ "ms": ms, "from": from.to_string(), "bytes": n, "text": String::from_utf8_lossy(bytes), "hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" ") }),
        };
        received.push(entry);
    }
    let mut text = format!("{} on {bind} in {:.1} s", match received.len() { 0 => "Nothing arrived".to_string(), 1 => "1 datagram".to_string(), n => format!("{n} datagrams") }, started.elapsed().as_secs_f64());
    if received.is_empty() && address.ip().is_unspecified() {
        text.push_str(". If the sender is another machine, check that the firewall lets UDP in on this port (signallab doctor).");
    }
    for entry in &received {
        let what = match entry.get("osc") {
            Some(messages) => messages.as_array().into_iter().flatten().map(|message| format!("{} {}", message["address"].as_str().unwrap_or_default(), message["args"])).collect::<Vec<_>>().join("; "),
            None => format!("{:?}", entry["text"].as_str().unwrap_or_default()),
        };
        text.push_str(&format!("\n{:>6} ms  {}  {what}", entry["ms"], entry["from"].as_str().unwrap_or_default()));
    }
    Answer::ok(clip(&text), json!({ "received": received }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc_arguments_from_json() {
        let args = osc_args(&json!([1, 2.5, "x", true, null, 5_000_000_000_i64, { "type": "double", "value": 0.125 }])).unwrap();
        assert_eq!(
            serde_json::to_value(args).unwrap(),
            json!([
                { "type": "int", "value": 1 }, { "type": "float", "value": 2.5 }, { "type": "str", "value": "x" }, { "type": "bool", "value": true },
                { "type": "nil" }, { "type": "long", "value": 5_000_000_000_i64 }, { "type": "double", "value": 0.125 }
            ])
        );
        assert!(osc_args(&json!([[1]])).is_err() && osc_args(&json!("x")).is_err());
        assert!(osc_args(&Value::Null).unwrap().is_empty());
    }

    #[test]
    fn long_text_is_cut_on_a_character_and_says_so() {
        assert_eq!(clip(&"é".repeat(MAX_TEXT)).lines().last().unwrap(), format!("… ({} more bytes)", MAX_TEXT));
    }
}
