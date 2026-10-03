//! The fields of one node: present, in range, well formed. A field that uses
//! only parameters is checked as the text a run will send; the node's Retry
//! and Repeat settings are checked here too. Whether the graph can run is
//! `experiment_validate`.

use std::collections::BTreeMap;
use std::net::SocketAddr;

use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Node, NodeKind, Repeat, RepeatUntil, Retry, MAX_LOOP, MAX_REPEATS, RUN_LIMIT};
use super::experiment_data::{self as data, ExtractFrom};
use super::matching::{UdpMode, MAX_ARG_INDEX};


pub const MAX_TIMEOUT_MS: u64 = 120_000;
pub const MAX_DELAY_MS: u64 = 60_000;
const MAX_LOG_CHARS: usize = 10_000;
const MAX_DATAGRAM: usize = 65_507;
pub const MAX_RULES: usize = 16;

fn required(field: Field) -> EngineError {
    EngineError::new("node.required").in_field(field)
}

fn range(field: Field, min: u64, max: u64) -> EngineError {
    EngineError::new("node.range").with("min", min).with("max", max).in_field(field)
}

fn too_long(field: Field, max: usize) -> EngineError {
    EngineError::new("node.too_long").with("max", max).in_field(field)
}

fn check_timeout(timeout_ms: u64) -> EngineResult<()> {
    match timeout_ms {
        1..=MAX_TIMEOUT_MS => Ok(()),
        _ => Err(range(Field::new("timeout_ms"), 1, MAX_TIMEOUT_MS)),
    }
}

fn check_port(port: u16) -> EngineResult<()> {
    match port {
        0 => Err(range(Field::new("port"), 1, u16::MAX as u64)),
        _ => Ok(()),
    }
}

/// A listening address: `IP:port` with a port, since a reply must find it.
pub fn check_bind(bind: &str) -> EngineResult<SocketAddr> {
    if bind.trim().is_empty() {
        return Err(required(Field::new("bind")));
    }
    match bind.trim().parse::<SocketAddr>() {
        Ok(address) if address.port() != 0 => Ok(address),
        _ => Err(EngineError::new("node.bind_invalid").with("value", bind).in_field(Field::new("bind"))),
    }
}

/// Where a send that expects a reply listens. Port 0 is allowed: the message
/// goes out from that socket, so a device answering the sender finds it.
pub fn check_reply_bind(bind: &str) -> EngineResult<SocketAddr> {
    let field = Field::new("reply_bind");
    if bind.trim().is_empty() {
        return Err(required(field));
    }
    bind.trim().parse::<SocketAddr>().map_err(|_| EngineError::new("node.bind_invalid").with("value", bind).in_field(field))
}

/// The most attempts a retry may make, the first included.
pub const MAX_ATTEMPTS: u32 = 10;

pub(crate) fn check_retry(kind: &NodeKind, retry: &Retry) -> EngineResult<()> {
    if !kind.retries() {
        return Err(EngineError::new("node.retry_unsupported").in_field(Field::new("retry")));
    }
    if !(1..=MAX_ATTEMPTS).contains(&retry.attempts) {
        return Err(range(Field::new("attempts"), 1, MAX_ATTEMPTS as u64));
    }
    if retry.delay_ms > MAX_DELAY_MS {
        return Err(range(Field::new("retry_delay"), 0, MAX_DELAY_MS));
    }
    Ok(())
}

/// The shortest pause between two sends of a repeating step: 100 per second.
pub const MIN_REPEAT_INTERVAL_MS: u64 = 10;

/// Repeat sends again and again, so it is bounded twice: by its own numbers,
/// and by the run, which every repetition must fit in.
pub(crate) fn check_repeat(kind: &NodeKind, repeat: &Repeat) -> EngineResult<()> {
    if !kind.repeats() {
        return Err(EngineError::new("node.repeat_unsupported").in_field(Field::new("repeat")));
    }
    if !(MIN_REPEAT_INTERVAL_MS..=MAX_DELAY_MS).contains(&repeat.interval_ms) {
        return Err(range(Field::new("repeat_interval"), MIN_REPEAT_INTERVAL_MS, MAX_DELAY_MS));
    }
    if repeat.jitter_ms > MAX_DELAY_MS {
        return Err(range(Field::new("repeat_jitter"), 0, MAX_DELAY_MS));
    }
    let limit = RUN_LIMIT.as_millis() as u64;
    let longest = match repeat.until {
        RepeatUntil::Count => {
            if !(2..=MAX_REPEATS).contains(&repeat.count) {
                return Err(range(Field::new("repeat_count"), 2, MAX_REPEATS as u64));
            }
            (repeat.count as u64 - 1) * (repeat.interval_ms + repeat.jitter_ms)
        }
        RepeatUntil::Duration => {
            if !(1..=limit).contains(&repeat.duration_ms) {
                return Err(range(Field::new("repeat_duration"), 1, limit));
            }
            if repeat.duration_ms / repeat.interval_ms >= MAX_REPEATS as u64 {
                return Err(EngineError::new("node.repeat_too_many").with("max", MAX_REPEATS).in_field(Field::new("repeat_interval")));
            }
            repeat.duration_ms
        }
    };
    if longest > limit {
        return Err(EngineError::new("node.repeat_too_long").with("seconds", RUN_LIMIT.as_secs()).in_field(Field::new("repeat")));
    }
    Ok(())
}

/// Load sends an HTTP request on a profile, in place of Repeat and Retry: a
/// failed request is counted, not tried again.
pub(crate) fn check_load(node: &Node, load: &crate::load::Load) -> EngineResult<()> {
    if !node.kind.loads() {
        return Err(EngineError::new("node.load_unsupported").in_field(Field::new("load")));
    }
    if node.repeat.is_some() || node.retry.is_some() {
        return Err(EngineError::new("node.load_alone").in_field(Field::new("load")));
    }
    crate::load::check(load)
}

fn check_rules(count: usize, indexes: impl Iterator<Item = usize>) -> EngineResult<()> {
    if count > MAX_RULES {
        return Err(EngineError::new("node.rules_too_many").with("max", MAX_RULES).in_field(Field::new("rules")));
    }
    if let Some(index) = indexes.into_iter().position(|index| index > MAX_ARG_INDEX) {
        return Err(range(Field::nth("rule", index + 1), 0, MAX_ARG_INDEX as u64));
    }
    Ok(())
}

fn header_valid(name: &str) -> bool {
    reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_ok()
}

/// The fields of one node. A field that uses only parameters is checked as
/// the text a run will send.
pub(crate) fn check_node(kind: &NodeKind, params: &BTreeMap<String, String>) -> EngineResult<()> {
    let fixed = |text: &str| data::static_text(text, params);
    match kind {
        NodeKind::Start | NodeKind::End | NodeKind::Fork | NodeKind::Join => {}
        NodeKind::Log { message } if message.chars().count() > MAX_LOG_CHARS => {
            return Err(too_long(Field::new("message"), MAX_LOG_CHARS));
        }
        NodeKind::Log { .. } => {}
        NodeKind::Tcp { host, port, timeout_ms, payload: _ } => {
            if host.trim().is_empty() {
                return Err(required(Field::new("host")));
            }
            check_port(*port)?;
            check_timeout(*timeout_ms)?;
        }
        NodeKind::Delay { ms } if *ms > MAX_DELAY_MS => return Err(range(Field::new("ms"), 0, MAX_DELAY_MS)),
        NodeKind::Delay { .. } => {}
        NodeKind::Http { request } => {
            if request.url.trim().is_empty() {
                return Err(required(Field::new("url")));
            }
            if let Some(url) = fixed(&request.url) {
                let scheme = reqwest::Url::parse(url.trim()).map(|url| url.scheme().to_string());
                if !matches!(scheme.as_deref(), Ok("http" | "https")) {
                    return Err(EngineError::new("node.url_invalid").with("value", url).in_field(Field::new("url")));
                }
            }
            if reqwest::Method::from_bytes(request.method.as_bytes()).is_err() {
                return Err(EngineError::new("node.method_invalid").with("value", &request.method).in_field(Field::new("method")));
            }
            check_timeout(request.timeout_ms)?;
            match &request.auth {
                crate::http_auth::Auth::Basic { username, .. } | crate::http_auth::Auth::Digest { username, .. } if username.trim().is_empty() => {
                    return Err(required(Field::new("username")));
                }
                crate::http_auth::Auth::Bearer { token } if token.trim().is_empty() => return Err(required(Field::new("token"))),
                _ => {}
            }
            for (index, (name, _)) in request.headers.iter().enumerate() {
                if let Some(name) = fixed(name).filter(|name| !name.trim().is_empty()) {
                    if !header_valid(name.trim()) {
                        let field = Field::nth("header_name", index + 1);
                        return Err(EngineError::new("node.header_invalid").with("value", name).in_field(field));
                    }
                }
            }
        }
        NodeKind::AssertStatus { status } | NodeKind::BranchStatus { status } if !(100..=599).contains(status) => {
            return Err(range(Field::new("status"), 100, 599));
        }
        NodeKind::AssertStatus { .. } | NodeKind::BranchStatus { .. } => {}
        NodeKind::AssertBody { contains } if contains.is_empty() => return Err(required(Field::new("expected_text"))),
        NodeKind::AssertBody { .. } => {}
        NodeKind::AssertHeader { name, .. } => {
            if name.trim().is_empty() {
                return Err(required(Field::new("header_name")));
            }
            if let Some(name) = fixed(name).filter(|name| !header_valid(name.trim())) {
                return Err(EngineError::new("node.header_invalid").with("value", name).in_field(Field::new("header_name")));
            }
        }
        NodeKind::AssertLatency { max_ms } if !(1..=MAX_TIMEOUT_MS).contains(max_ms) => {
            return Err(range(Field::new("max_ms"), 1, MAX_TIMEOUT_MS));
        }
        NodeKind::AssertLatency { .. } => {}
        NodeKind::Mqtt { host, port, topic, qos, .. } => {
            if host.trim().is_empty() {
                return Err(required(Field::new("broker")));
            }
            check_port(*port)?;
            if topic.is_empty() {
                return Err(required(Field::new("topic")));
            }
            if fixed(topic).is_some_and(|topic| topic.contains(['#', '+', '\0'])) {
                return Err(EngineError::new("node.topic_wildcard").in_field(Field::new("topic")));
            }
            if topic.len() > u16::MAX as usize {
                return Err(too_long(Field::new("topic"), u16::MAX as usize));
            }
            if *qos > 2 {
                return Err(range(Field::new("qos"), 0, 2));
            }
        }
        NodeKind::Osc { target, address, reply, .. } => {
            if target.trim().is_empty() {
                return Err(required(Field::new("target")));
            }
            if let Some(reply) = reply {
                check_reply_bind(&reply.bind)?;
                if reply.address.trim().is_empty() {
                    return Err(required(Field::new("reply_address")));
                }
                check_rules(reply.args.len(), reply.args.iter().map(|rule| rule.index))?;
                check_timeout(reply.timeout_ms)?;
            }
            if address.trim().is_empty() {
                return Err(required(Field::new("address")));
            }
            if let Some(address) = fixed(address).filter(|address| !address.starts_with('/')) {
                return Err(EngineError::new("node.osc_address").with("value", address).in_field(Field::new("address")));
            }
            if let Some(target) = fixed(target).filter(|target| target.trim().parse::<SocketAddr>().is_err()) {
                return Err(EngineError::new("node.target_invalid").with("value", target).in_field(Field::new("target")));
            }
        }
        NodeKind::Udp { target, text, reply } => {
            if target.trim().is_empty() {
                return Err(required(Field::new("target")));
            }
            if text.len() > MAX_DATAGRAM {
                return Err(too_long(Field::new("payload"), MAX_DATAGRAM));
            }
            if let Some(reply) = reply {
                check_reply_bind(&reply.bind)?;
                if reply.mode != UdpMode::Any && reply.pattern.is_empty() {
                    return Err(required(Field::new("reply_pattern")));
                }
                check_timeout(reply.timeout_ms)?;
            }
        }
        NodeKind::Extract { from, expr, .. } => {
            if expr.trim().is_empty() && !matches!(from, ExtractFrom::Status | ExtractFrom::Body) {
                return Err(required(data::extract_field(*from)));
            }
        }
        NodeKind::AssertValue { .. } | NodeKind::BranchValue { .. } => {}
        NodeKind::Loop { max, .. } => {
            if !(1..=MAX_LOOP).contains(max) {
                return Err(range(Field::new("loop_max"), 1, MAX_LOOP as u64));
            }
        }
        NodeKind::WaitOsc { bind, address, args, timeout_ms, .. } => {
            check_bind(bind)?;
            if address.trim().is_empty() {
                return Err(required(Field::new("address")));
            }
            check_rules(args.len(), args.iter().map(|rule| rule.index))?;
            check_timeout(*timeout_ms)?;
        }
        NodeKind::WaitMqtt { host, port, topic, mode, pattern, timeout_ms, .. } => {
            if host.trim().is_empty() {
                return Err(required(Field::new("broker")));
            }
            check_port(*port)?;
            if topic.is_empty() {
                return Err(required(Field::new("topic")));
            }
            // Subscribed before the first step: only parameters are known then.
            for (field, text) in [("broker", host), ("topic", topic)] {
                if fixed(text).is_none() {
                    return Err(EngineError::new("node.params_only").in_field(Field::new(field)));
                }
            }
            if fixed(topic).is_some_and(|topic| !crate::subscribe::filter_valid(&topic)) {
                return Err(EngineError::new("node.topic_filter").with("value", topic).in_field(Field::new("topic")));
            }
            if *mode != UdpMode::Any && pattern.is_empty() {
                return Err(required(Field::new("pattern")));
            }
            check_timeout(*timeout_ms)?;
        }
        NodeKind::WaitUdp { bind, mode, pattern, timeout_ms, .. } => {
            check_bind(bind)?;
            if *mode != UdpMode::Any && pattern.is_empty() {
                return Err(required(Field::new("pattern")));
            }
            check_timeout(*timeout_ms)?;
        }
        // Its own rules, with the run's parameters; its port is opened before the first step.
        NodeKind::Emulator { emulator } => crate::emulator::check(emulator, params)?,
        NodeKind::WaitHttp { bind, path, when, timeout_ms, .. } => {
            check_bind(bind)?;
            if path.trim().is_empty() {
                return Err(required(Field::new("path")));
            }
            if when.len() > crate::emulator::MAX_CONDITIONS {
                return Err(EngineError::new("emulator.conditions_too_many").with("max", crate::emulator::MAX_CONDITIONS).in_field(Field::new("conditions")));
            }
            check_timeout(*timeout_ms)?;
        }
        // Opened before the first step: its addresses may use parameters only.
        NodeKind::Impairment { listen, target, profile, .. } => {
            for (field, text) in [("listen", listen), ("target", target)] {
                if text.trim().is_empty() {
                    return Err(required(Field::new(field)));
                }
            }
            crate::netsim_run::addresses(listen, target, params)?;
            profile.check()?;
        }
        NodeKind::ImpairmentChange { relay, profile } => {
            if relay.trim().is_empty() {
                return Err(required(Field::new("relay")));
            }
            profile.check()?;
        }
        NodeKind::EmulatorState { emulator, .. } => {
            if emulator.trim().is_empty() {
                return Err(required(Field::new("emulator")));
            }
        }
        NodeKind::WsConnect { url, headers, protocols, timeout_ms } => {
            if url.trim().is_empty() {
                return Err(required(Field::new("url")));
            }
            if let Some(url) = fixed(url) {
                crate::ws::check_url(url.trim())?;
            }
            for (index, (name, _)) in headers.iter().enumerate() {
                if let Some(name) = fixed(name).filter(|name| !name.trim().is_empty()) {
                    if !header_valid(name.trim()) {
                        let field = Field::nth("header_name", index + 1);
                        return Err(EngineError::new("node.header_invalid").with("value", name).in_field(field));
                    }
                }
            }
            if let Some(protocol) = protocols.iter().find(|protocol| !crate::ws::protocol_valid(protocol)) {
                return Err(EngineError::new("ws.protocol_invalid").with("value", protocol).in_field(Field::new("protocols")));
            }
            check_timeout(*timeout_ms)?;
        }
        NodeKind::WsSend { connection, text, binary } => {
            if connection.trim().is_empty() {
                return Err(required(Field::new("connection")));
            }
            if *binary {
                if let Some(text) = fixed(text) {
                    crate::matching::parse_hex(&text).map_err(|error| error.in_field(Field::new("payload")))?;
                }
            }
            if text.len() > crate::ws::MAX_MESSAGE {
                return Err(too_long(Field::new("payload"), crate::ws::MAX_MESSAGE));
            }
        }
        NodeKind::WaitWs { connection, mode, pattern, timeout_ms, .. } => {
            if connection.trim().is_empty() {
                return Err(required(Field::new("connection")));
            }
            if *mode != UdpMode::Any && pattern.is_empty() {
                return Err(required(Field::new("pattern")));
            }
            check_timeout(*timeout_ms)?;
        }
        NodeKind::WsClose { connection, code, reason } => {
            if connection.trim().is_empty() {
                return Err(required(Field::new("connection")));
            }
            crate::ws::check_close_code(*code)?;
            // A close frame carries at most 125 bytes: the code and the reason.
            if reason.len() > crate::ws::MAX_CLOSE_REASON {
                return Err(too_long(Field::new("reason"), crate::ws::MAX_CLOSE_REASON));
            }
        }
    }
    Ok(())
}
