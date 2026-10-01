//! What one step does when it runs: send, check, extract, branch or wait.
//! `experiment_run` decides which step runs next and reports it; this module
//! decides what a step does with its branch's state.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use crate::host::Host;

use super::error::{EngineError, EngineResult, Field};
use super::experiment::NodeKind;
use super::experiment_actions as actions;
use super::experiment_data as data;
use super::http::HttpResponse;
use super::listen::{Inbox, Listener, Listeners, WaitOutcome};
use super::matching::{self, Datagram, Matcher, OscMatcher, UdpMatcher};
use super::subscribe::{self, Subscription, Subscriptions};

/// The state a branch carries from step to step. A Join merges the states of
/// its inputs.
#[derive(Clone, Default)]
pub struct BranchContext {
    pub last_status: Option<u16>,
    pub last_response: Option<HttpResponse>,
    pub vars: BTreeMap<String, Value>,
    /// When the latest network action on this branch started. A wait counts
    /// replies from then on — from the start of the run before any action.
    pub last_action: Option<Instant>,
}

impl BranchContext {
    /// Several branches' states as one, in the given (document) order: later
    /// responses and variables win, the earliest action counts for waits.
    pub fn merge<'a>(states: impl IntoIterator<Item = &'a BranchContext>) -> BranchContext {
        let mut merged = BranchContext::default();
        for state in states {
            if state.last_status.is_some() {
                merged.last_status = state.last_status;
            }
            if state.last_response.is_some() {
                merged.last_response = state.last_response.clone();
            }
            merged.vars.extend(state.vars.iter().map(|(name, value)| (name.clone(), value.clone())));
            merged.last_action = match (merged.last_action, state.last_action) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
        }
        merged
    }
}

/// What a step produced: the output to follow and what to report.
pub struct StepOutcome {
    pub port: &'static str,
    pub detail: String,
    pub message: Option<(&'static str, Value)>,
    pub written: Option<BTreeMap<String, Value>>,
    /// The Inspector frame of the message a wait matched.
    pub frame: Option<u64>,
}

impl StepOutcome {
    fn next(detail: impl Into<String>) -> Self {
        StepOutcome { port: "next", detail: detail.into(), message: None, written: None, frame: None }
    }

    fn said(mut self, key: &'static str, params: Value) -> Self {
        self.message = Some((key, params));
        self
    }
}

/// What a step may use besides its branch's state.
pub struct StepEnv<'a> {
    pub host: &'a Host,
    pub client_id: String,
    pub seed: u64,
    pub listeners: Listeners,
    /// The MQTT subscriptions of the run's *Wait for MQTT* steps.
    pub subscriptions: Subscriptions,
    /// The node's Timeout output is connected.
    pub has_timeout: bool,
    /// Inputs of the node (a Join reports how many it merged).
    pub inputs: usize,
    pub run_started: Instant,
}

/// Checks consume the latest response on the executed path, never another branch's.
fn check_response(kind: &NodeKind, response: Option<&HttpResponse>) -> EngineResult<StepOutcome> {
    let response = response.ok_or_else(|| EngineError::new("check.no_response"))?;
    match kind {
        NodeKind::AssertBody { contains } if response.body.contains(contains.as_str()) => {}
        NodeKind::AssertBody { .. } if response.truncated => {
            return Err(EngineError::new("check.body_truncated").with("bytes", response.body_bytes));
        }
        NodeKind::AssertBody { contains } => {
            return Err(EngineError::new("check.body_missing").with("text", matching::shorten(contains)));
        }
        NodeKind::AssertHeader { name, contains } => {
            let found = response.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name.trim()));
            match found {
                Some((_, value)) if value.contains(contains.as_str()) => {}
                Some((_, value)) => {
                    return Err(EngineError::new("check.header_mismatch")
                        .with("name", name.trim())
                        .with("text", matching::shorten(contains))
                        .with("actual", matching::shorten(value)));
                }
                None => return Err(EngineError::new("check.header_missing").with("name", name.trim())),
            }
        }
        NodeKind::AssertLatency { max_ms } if response.latency_ms <= *max_ms as f64 => {}
        NodeKind::AssertLatency { max_ms } => {
            return Err(EngineError::new("check.latency_failed").with("limit", max_ms).with("actual", format!("{:.0}", response.latency_ms)));
        }
        _ => return Err(EngineError::new("run.not_a_check")),
    }
    Ok(StepOutcome::next("Check passed").said("exp.step.checked", json!({})))
}

/// One line for the timeline about a matched reply.
fn reply_summary(reply: &Value) -> String {
    match (reply.get("address").and_then(Value::as_str), reply.get("topic").and_then(Value::as_str)) {
        (Some(address), _) => {
            let args: Vec<String> = reply["args"].as_array().into_iter().flatten().map(|arg| arg.to_string()).collect();
            if args.is_empty() { address.to_string() } else { format!("{address} {}", args.join(" ")) }
        }
        (None, Some(topic)) => format!("{topic} {}", matching::shorten(reply["text"].as_str().unwrap_or_default())),
        (None, None) => matching::shorten(reply["text"].as_str().unwrap_or_default()),
    }
}

/// The matcher for a resolved wait node.
fn matcher(kind: &NodeKind) -> EngineResult<Box<dyn Matcher>> {
    match kind {
        NodeKind::WaitOsc { address, args, .. } => Ok(Box::new(OscMatcher::new(address, args)?)),
        NodeKind::WaitUdp { mode, pattern, .. } | NodeKind::WaitMqtt { mode, pattern, .. } => Ok(Box::new(UdpMatcher::new(*mode, pattern)?)),
        _ => Err(EngineError::new("run.not_a_wait")),
    }
}

/// The socket this run opened for `bind`; `field` is where a wrong one is shown.
fn listener_for(listeners: &Listeners, bind: &str, field: &'static str) -> EngineResult<Arc<Listener>> {
    bind.trim()
        .parse::<SocketAddr>()
        .ok()
        .and_then(|address| listeners.get(&address).cloned())
        .ok_or_else(|| EngineError::new("wait.not_listening").with("target", bind).in_field(Field::new(field)))
}

/// What a wait listens to: one of the run's UDP sockets or MQTT subscriptions.
enum Source {
    Socket(Arc<Listener>),
    Broker(Arc<Subscription>),
}

impl Source {
    fn inbox(&self) -> &Inbox {
        match self {
            Source::Socket(listener) => listener.inbox(),
            Source::Broker(subscription) => subscription.inbox(),
        }
    }

    fn target(&self) -> String {
        match self {
            Source::Socket(listener) => listener.local().to_string(),
            Source::Broker(subscription) => subscription.broker().to_string(),
        }
    }
}

/// A matched reply: its value with the time it took since `since`, and how the
/// timeline says it.
struct Reply {
    value: Value,
    summary: String,
    from: String,
    ms: u64,
}

fn reply_of(datagram: &Datagram, mut value: Value, since: Instant) -> Reply {
    let ms = datagram.at.saturating_duration_since(since).as_millis() as u64;
    value["ms"] = ms.into();
    if let Some(topic) = &datagram.topic {
        value["topic"] = topic.clone().into();
    }
    Reply { summary: reply_summary(&value), from: datagram.from.to_string(), ms, value }
}

fn timed_out(source: &Source, timeout_ms: u64, unmatched: usize) -> EngineError {
    let error = EngineError::new("wait.timeout").with("ms", timeout_ms).with("unmatched", unmatched).with("target", source.target());
    // A full queue may have pushed the reply out; say so for diagnosis.
    match source.inbox().dropped() {
        0 => error,
        dropped => error.because(format!("listener queue full: {dropped} older datagrams dropped")),
    }
}

/// The source a wait node listens to.
fn source_of(env: &StepEnv<'_>, kind: &NodeKind) -> EngineResult<Source> {
    if let Some((host, port, topic)) = kind.subscription() {
        let key = subscribe::key(host, port, topic);
        return env.subscriptions.get(&key).cloned().map(Source::Broker).ok_or_else(|| {
            EngineError::new("wait.not_listening").with("target", format!("{} {}", key.0, key.1)).in_field(Field::new("topic"))
        });
    }
    let bind = kind.bind().ok_or_else(|| EngineError::new("run.not_a_wait"))?;
    listener_for(&env.listeners, bind, "bind").map(Source::Socket)
}

async fn wait(env: &StepEnv<'_>, kind: &NodeKind, context: &BranchContext) -> EngineResult<StepOutcome> {
    let (NodeKind::WaitOsc { timeout_ms, variable, .. } | NodeKind::WaitUdp { timeout_ms, variable, .. } | NodeKind::WaitMqtt { timeout_ms, variable, .. }) = kind else {
        return Err(EngineError::new("run.not_a_wait"));
    };
    let matcher = matcher(kind)?;
    let source = source_of(env, kind)?;
    let since = context.last_action.unwrap_or(env.run_started);
    match source.inbox().wait(matcher.as_ref(), since, Duration::from_millis(*timeout_ms)).await? {
        WaitOutcome::Matched(datagram, value) => {
            let Reply { value, summary, from, ms } = reply_of(&datagram, value, since);
            Ok(StepOutcome {
                port: "matched",
                detail: format!("{summary} ← {from} · {ms} ms"),
                message: Some(("exp.step.matched", json!({ "summary": summary, "from": from, "ms": ms }))),
                written: Some(BTreeMap::from([(variable.clone(), value)])),
                frame: datagram.frame,
            })
        }
        WaitOutcome::TimedOut { unmatched } if env.has_timeout => Ok(StepOutcome {
            port: "timeout",
            detail: format!("No match in {timeout_ms} ms · {unmatched} other"),
            message: Some(("exp.step.timedOut", json!({ "ms": timeout_ms, "unmatched": unmatched }))),
            written: None,
            frame: None,
        }),
        WaitOutcome::TimedOut { unmatched } => Err(timed_out(&source, *timeout_ms, unmatched)),
    }
}

/// An OSC message or datagram that expects an answer: sent from the socket the
/// run opened on the reply's bind (a device answering the sender is heard),
/// then the reply is awaited there. No reply in time fails the step, which a
/// retry can repeat.
async fn send_and_wait(env: &StepEnv<'_>, kind: &NodeKind, context: &mut BranchContext) -> EngineResult<StepOutcome> {
    let (bind, timeout_ms, variable, matcher): (&str, u64, &str, Box<dyn Matcher>) = match kind {
        NodeKind::Osc { reply: Some(reply), .. } => {
            let matcher = OscMatcher::new(&reply.address, &reply.args).map_err(|error| error.in_field(Field::new("reply_address")))?;
            (&reply.bind, reply.timeout_ms, &reply.variable, Box::new(matcher))
        }
        NodeKind::Udp { reply: Some(reply), .. } => {
            let matcher = UdpMatcher::new(reply.mode, &reply.pattern).map_err(|error| error.in_field(Field::new("reply_pattern")))?;
            (&reply.bind, reply.timeout_ms, &reply.variable, Box::new(matcher))
        }
        _ => return Err(EngineError::new("run.not_an_action")),
    };
    let listener = listener_for(&env.listeners, bind, "reply_bind")?;
    let since = Instant::now();
    context.last_action = Some(since);
    let sent = actions::send_via(env.host, &listener, kind).await?;
    match listener.wait(matcher.as_ref(), since, Duration::from_millis(timeout_ms)).await? {
        WaitOutcome::Matched(datagram, value) => {
            let Reply { value, summary, from, ms } = reply_of(&datagram, value, since);
            context.vars.insert(variable.to_string(), value.clone());
            Ok(StepOutcome {
                port: "next",
                detail: format!("{} · {summary} ← {from} · {ms} ms", sent.detail),
                message: Some(("exp.step.replied", json!({ "summary": summary, "from": from, "ms": ms }))),
                written: Some(BTreeMap::from([(variable.to_string(), value)])),
                frame: datagram.frame,
            })
        }
        WaitOutcome::TimedOut { unmatched } => Err(timed_out(&Source::Socket(listener), timeout_ms, unmatched).in_field(Field::new("reply_bind"))),
    }
}

/// Run one step whose templates are already resolved.
pub async fn execute(env: &StepEnv<'_>, kind: &NodeKind, context: &mut BranchContext) -> EngineResult<StepOutcome> {
    match kind {
        NodeKind::Start => Ok(StepOutcome::next("Started").said("exp.step.started", json!({ "seed": env.seed }))),
        NodeKind::End => Ok(StepOutcome::next("Complete").said("exp.step.complete", json!({}))),
        NodeKind::Fork => Ok(StepOutcome::next("Forked 2 branches").said("exp.step.forked", json!({}))),
        NodeKind::Join => Ok(StepOutcome::next("Synchronized branches").said("exp.step.joined", json!({ "count": env.inputs }))),
        NodeKind::Log { message } => Ok(StepOutcome::next(message.clone())),
        kind if kind.expects_reply() => send_and_wait(env, kind, context).await,
        kind if kind.is_action() => {
            // Before sending: a reply can arrive while the send is still being reported.
            context.last_action = Some(Instant::now());
            let outcome = actions::execute(env.host, env.client_id.clone(), kind).await?;
            if let Some(response) = outcome.response {
                context.last_status = Some(response.status);
                context.last_response = Some(response);
            }
            Ok(StepOutcome::next(outcome.detail))
        }
        NodeKind::Delay { ms } => {
            tokio::time::sleep(Duration::from_millis(*ms)).await;
            Ok(StepOutcome::next(format!("Waited {ms} ms")).said("exp.step.waited", json!({ "ms": ms })))
        }
        NodeKind::AssertBody { .. } | NodeKind::AssertHeader { .. } | NodeKind::AssertLatency { .. } => {
            check_response(kind, context.last_response.as_ref())
        }
        NodeKind::AssertStatus { status } => match context.last_status {
            Some(actual) if actual == *status => Ok(StepOutcome::next(format!("HTTP {actual} = {status}"))),
            Some(actual) => Err(EngineError::new("check.status_failed").with("expected", status).with("actual", actual)),
            None => Err(EngineError::new("check.no_response")),
        },
        NodeKind::BranchStatus { status } => {
            let actual = context.last_status.ok_or_else(|| EngineError::new("check.no_response"))?;
            let (port, key) = if actual == *status { ("yes", "exp.step.yes") } else { ("no", "exp.step.no") };
            Ok(StepOutcome { port, detail: format!("HTTP {actual} = {status} → {port}"), message: Some((key, json!({ "status": actual }))), written: None, frame: None })
        }
        NodeKind::Extract { variable, from, expr } => {
            let value = data::extract(*from, expr, context.last_response.as_ref()).map_err(|error| error.in_field(data::extract_field(*from)))?;
            let preview = data::value_preview(&value);
            context.vars.insert(variable.clone(), value.clone());
            Ok(StepOutcome {
                port: "next",
                detail: format!("{variable} = {preview}"),
                message: Some(("exp.step.extracted", json!({ "name": variable, "value": preview }))),
                written: Some(BTreeMap::from([(variable.clone(), value)])),
                frame: None,
            })
        }
        NodeKind::AssertValue { value, op, expected } => {
            let (holds, comparison) = data::compare(value, *op, expected).map_err(|error| error.in_field(Field::new("expected")))?;
            if !holds {
                return Err(comparison.failed());
            }
            Ok(StepOutcome::next(comparison.text()).said("exp.step.compared", comparison.params()))
        }
        NodeKind::BranchValue { value, op, expected } => {
            let (holds, comparison) = data::compare(value, *op, expected).map_err(|error| error.in_field(Field::new("expected")))?;
            let (port, key) = if holds { ("yes", "exp.step.valueYes") } else { ("no", "exp.step.valueNo") };
            Ok(StepOutcome { port, detail: comparison.text(), message: Some((key, comparison.params())), written: None, frame: None })
        }
        NodeKind::WaitOsc { .. } | NodeKind::WaitUdp { .. } | NodeKind::WaitMqtt { .. } => {
            let outcome = wait(env, kind, context).await?;
            if let Some(written) = &outcome.written {
                context.vars.extend(written.iter().map(|(name, value)| (name.clone(), value.clone())));
            }
            Ok(outcome)
        }
        _ => Err(EngineError::new("run.not_an_action")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn response() -> HttpResponse {
        HttpResponse {
            ok: true,
            status: 200,
            status_text: "OK".into(),
            latency_ms: 100.0,
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: "{\"ready\":true}".into(),
            body_bytes: 14,
            truncated: false,
            error: None,
            cause: None,
        }
    }

    fn code(result: EngineResult<StepOutcome>) -> String {
        result.err().unwrap().into_code()
    }

    #[test]
    fn response_checks_cover_boundaries_and_truncation() {
        let mut reply = response();
        let body = NodeKind::AssertBody { contains: "ready".into() };
        assert_eq!(code(check_response(&body, None)), "check.no_response");
        assert!(check_response(&body, Some(&reply)).is_ok());
        let missing = NodeKind::AssertBody { contains: "missing".into() };
        assert_eq!(code(check_response(&missing, Some(&reply))), "check.body_missing");
        reply.truncated = true;
        assert!(check_response(&body, Some(&reply)).is_ok());
        assert_eq!(code(check_response(&missing, Some(&reply))), "check.body_truncated");
        let header = |name: &str, contains: &str| NodeKind::AssertHeader { name: name.into(), contains: contains.into() };
        assert!(check_response(&header("CONTENT-type", "json"), Some(&reply)).is_ok());
        assert_eq!(code(check_response(&header("missing", ""), Some(&reply))), "check.header_missing");
        let mismatch = check_response(&header("content-type", "xml"), Some(&reply)).err().unwrap();
        assert_eq!((mismatch.code.as_str(), mismatch.params["actual"].as_str()), ("check.header_mismatch", "application/json"));
        assert!(check_response(&NodeKind::AssertLatency { max_ms: 100 }, Some(&reply)).is_ok());
        let slow = check_response(&NodeKind::AssertLatency { max_ms: 99 }, Some(&reply)).err().unwrap();
        assert_eq!((slow.params["limit"].as_str(), slow.params["actual"].as_str()), ("99", "100"));
    }

    #[test]
    fn merged_branches_keep_the_earliest_action() {
        let early = Instant::now();
        let late = early + Duration::from_millis(5);
        let a = BranchContext { last_action: Some(late), last_status: Some(200), ..Default::default() };
        let b = BranchContext { last_action: Some(early), vars: BTreeMap::from([("x".into(), json!(1))]), ..Default::default() };
        let none = BranchContext::default();
        let merged = BranchContext::merge([&a, &none, &b]);
        assert_eq!((merged.last_action, merged.last_status, merged.vars.len()), (Some(early), Some(200), 1));
        assert_eq!(BranchContext::merge([&none]).last_action, None);
    }

    fn wait_osc(bind: SocketAddr, timeout_ms: u64) -> NodeKind {
        serde_json::from_value(json!({
            "type": "wait_osc", "bind": bind.to_string(), "address": "/pong*", "timeout_ms": timeout_ms,
            "args": [{ "index": 0, "op": "eq", "value": "42" }]
        }))
        .unwrap()
    }

    async fn send_osc(to: SocketAddr, address: &str, args: &[crate::osc_codec::OscArg]) {
        let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        socket.send_to(&crate::osc_codec::encode_message(address, args), to).await.unwrap();
    }

    fn env<'a>(host: &'a Host, listeners: &Listeners, has_timeout: bool, started: Instant) -> StepEnv<'a> {
        StepEnv { host, client_id: "test".into(), seed: 0, listeners: listeners.clone(), subscriptions: Subscriptions::new(), has_timeout, inputs: 1, run_started: started }
    }

    /// `wait` as the old signature read, for the tests below.
    async fn wait(listeners: &Listeners, has_timeout: bool, started: Instant, kind: &NodeKind, context: &BranchContext) -> EngineResult<StepOutcome> {
        let host = Host::new(Arc::new(crate::host::NoEvents), crate::inspect::Capture::new());
        super::wait(&env(&host, listeners, has_timeout, started), kind, context).await
    }

    #[tokio::test]
    async fn a_wait_matches_the_reply_writes_it_and_follows_matched_or_timeout() {
        use crate::listen::{Listener, NoFrames};
        use crate::osc_codec::OscArg;
        let listener = Arc::new(Listener::arm("127.0.0.1:0".parse().unwrap(), Arc::new(NoFrames)).await.unwrap());
        let bind = listener.local();
        let listeners = Listeners::from([(bind, listener)]);
        let started = Instant::now();

        // A reply with the wrong argument, then the right one: only the second matches.
        send_osc(bind, "/pong", &[OscArg::Int(7)]).await;
        send_osc(bind, "/pongs", &[OscArg::Int(42), OscArg::Str("ready".into())]).await;
        let context = BranchContext::default();
        let outcome = wait(&listeners, false, started, &wait_osc(bind, 2000), &context).await.unwrap();
        assert_eq!(outcome.port, "matched");
        let reply = &outcome.written.as_ref().unwrap()["reply"];
        assert_eq!((reply["address"].as_str(), reply["args"].clone()), (Some("/pongs"), json!([42, "ready"])));
        assert!(reply["ms"].is_u64() && reply["from"].as_str().unwrap().starts_with("127.0.0.1:"));
        assert_eq!(outcome.message.unwrap().0, "exp.step.matched");

        // Nothing more arrives: Timeout when it is wired, an error naming the wait otherwise.
        let timed_out = wait(&listeners, true, started, &wait_osc(bind, 60), &context).await.unwrap();
        assert_eq!((timed_out.port, timed_out.written.is_none()), ("timeout", true));
        assert_eq!(timed_out.message.unwrap().1["unmatched"], 1, "the /pong 7 arrived and did not match");
        let failed = wait(&listeners, false, started, &wait_osc(bind, 60), &context).await.err().unwrap();
        assert_eq!((failed.code.as_str(), failed.params["ms"].as_str()), ("wait.timeout", "60"));

        // A reply from before the branch's last request does not count.
        send_osc(bind, "/pong", &[OscArg::Int(42)]).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        let after = BranchContext { last_action: Some(Instant::now()), ..Default::default() };
        assert_eq!(wait(&listeners, true, started, &wait_osc(bind, 60), &after).await.unwrap().port, "timeout");

        let elsewhere = wait(&Listeners::new(), true, started, &wait_osc(bind, 60), &after).await.err().unwrap();
        assert_eq!(elsewhere.code, "wait.not_listening");
    }

    #[test]
    fn reply_summaries_read_like_the_monitor() {
        assert_eq!(reply_summary(&json!({ "address": "/pong", "args": [42, "ok"] })), "/pong 42 \"ok\"");
        assert_eq!(reply_summary(&json!({ "address": "/pong", "args": [] })), "/pong");
        assert_eq!(reply_summary(&json!({ "text": "PONG" })), "PONG");
    }
}
