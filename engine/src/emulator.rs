//! Emulators: Signal Lab as the other side — the API, device or service the
//! system under test talks to. An emulator listens on one address and answers
//! by rules: HTTP routes with responses (in sequence, in turn, or a seeded mix
//! with faults), and for OSC, UDP and TCP "on this, reply that". Replies are
//! templates read with what arrived — `{{request.params.id}}`,
//! `{{request.args[0]}}`, `{{request.json.name}}` — in the language of
//! `template.rs`, so a field means here what it means in an experiment.
//!
//! An emulator is a job of its own (the Emulators screen, `signallab
//! emulate`), or part of a run as an *Emulator* node: then it opens before the
//! first step, like a wait's socket, and closes with the run. Every exchange
//! goes to the Inspector, is counted per rule, and the newest `RECENT` are
//! kept for whoever asks what arrived. The protocols live in
//! `emulator_http` and `emulator_net`; this module holds the document, its
//! checks, and what every protocol shares.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::host::Host;

use super::emulator_http;
use super::emulator_net;
use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Node, NodeKind};
use super::experiment_data as data;
use super::experiment_validate::check_bind;
use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::listen::{Inbox, Tap};
use super::matching::{self, ArgRule, CompareOp, Datagram, Matcher, OscMatcher, UdpMatcher, UdpMode};
use super::osc_codec::OscArg;
use super::signals::RawPayload;
use super::template::{self, Ref, Renderer, Rng, Scope, Segment};

/// Routes or rules in one emulator.
pub const MAX_RULES: usize = 64;
/// Responses of one route.
pub const MAX_RESPONSES: usize = 16;
/// Conditions of one route or wait.
pub const MAX_CONDITIONS: usize = 16;
pub const MAX_HEADERS: usize = 32;
/// The longest delay of one reply, its jitter apart.
pub const MAX_DELAY_MS: u64 = 60_000;
/// A body, reply or greeting as written, before its templates.
pub const MAX_TEXT: usize = 256 * 1024;
const MAX_NAME: usize = 120;
const MAX_PATH: usize = 512;
/// The most OSC argument conditions or reply arguments.
const MAX_ARGS: usize = 16;
const MAX_DATAGRAM: usize = 65_507;
/// How long a `timeout` fault holds a request before it closes the connection.
pub const HOLD: Duration = Duration::from_secs(120);
/// Exchanges kept for whoever asks what arrived (the screen, a script, an LLM).
pub const RECENT: usize = 500;
/// Exchanges per activity event; more in one interval are counted, not sent.
const BATCH: usize = 200;
const REPORT_EVERY: Duration = Duration::from_millis(200);
/// At most one exchange in this many milliseconds reaches the Inspector.
const FRAME_GAP_MS: u64 = 10;
/// Bytes of a request body an exchange keeps for whoever asks later.
const KEPT_BODY: usize = 4 * 1024;
/// Mixed into the seed, so picking a response and drawing a delay never draw
/// the numbers a template's generators do.
const PICK_STREAM: u64 = 0x7069_636b_0000_0001;
const DELAY_STREAM: u64 = 0x6465_6c61_7900_0001;

// ---- the document ---------------------------------------------------------------

/// One emulator: a name, where it listens, and how it answers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emulator {
    pub name: String,
    /// `IP:port`. Loopback unless a person chose to reach the network.
    pub bind: String,
    #[serde(flatten)]
    pub kind: EmulatorKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "protocol", rename_all = "snake_case")]
pub enum EmulatorKind {
    /// An HTTP/1.1 API: the first route a request matches answers it.
    Http {
        #[serde(default)]
        routes: Vec<Route>,
        /// What a request no route takes gets; `None`: 404.
        #[serde(default)]
        fallback: Option<Response>,
    },
    /// An OSC device: each message of a packet is answered by the first rule it matches.
    Osc {
        #[serde(default)]
        rules: Vec<OscRule>,
    },
    /// A UDP device: each datagram is answered by the first rule it matches.
    Udp {
        #[serde(default)]
        rules: Vec<UdpRule>,
    },
    /// A TCP device speaking lines (or chunks): each one is answered by the first rule it matches.
    Tcp {
        #[serde(default)]
        delimiter: Delimiter,
        /// Sent when a client connects, before anything else; may be empty.
        #[serde(default)]
        greeting: String,
        #[serde(default)]
        rules: Vec<TcpRule>,
    },
}

impl EmulatorKind {
    pub fn protocol(&self) -> &'static str {
        match self {
            EmulatorKind::Http { .. } => "http",
            EmulatorKind::Osc { .. } => "osc",
            EmulatorKind::Udp { .. } => "udp",
            EmulatorKind::Tcp { .. } => "tcp",
        }
    }

    /// Listens on a TCP port (HTTP, TCP) rather than a UDP one (OSC, UDP).
    pub fn over_tcp(&self) -> bool {
        matches!(self, EmulatorKind::Http { .. } | EmulatorKind::Tcp { .. })
    }

    pub fn rules(&self) -> usize {
        match self {
            EmulatorKind::Http { routes, .. } => routes.len(),
            EmulatorKind::Osc { rules } => rules.len(),
            EmulatorKind::Udp { rules } => rules.len(),
            EmulatorKind::Tcp { rules, .. } => rules.len(),
        }
    }
}

/// An HTTP route: which requests it takes, and what they get.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    /// A method, or `ANY`. A GET route answers HEAD as well.
    #[serde(default = "any_method")]
    pub method: String,
    /// `/users/:id` names a segment, a final `/*` takes the rest.
    pub path: String,
    /// Every condition must hold.
    #[serde(default)]
    pub when: Vec<Condition>,
    #[serde(default)]
    pub order: Order,
    pub responses: Vec<Response>,
}

/// Which response a route's next request gets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// One after another, the last one from then on: 500, 500, 200, 200…
    #[default]
    Sequence,
    /// One after another, then from the first again.
    Cycle,
    /// Drawn by weight from the seed: 80 % of one, 20 % of another.
    Random,
}

/// `<on> <name> <op> <value>` about a request; the value may use parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    pub on: ConditionOn,
    /// The header, the query parameter or the JSON path; unused for the body.
    #[serde(default)]
    pub name: String,
    pub op: CompareOp,
    #[serde(default)]
    pub value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOn {
    Header,
    Query,
    Body,
    Json,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    #[serde(default = "ok_status")]
    pub status: u16,
    /// Names may use parameters, values the request too.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: String,
    /// How long to wait before answering…
    #[serde(default)]
    pub delay_ms: u64,
    /// …and up to this much longer, drawn from the seed.
    #[serde(default)]
    pub jitter_ms: u64,
    #[serde(default)]
    pub fault: Fault,
    /// Its share when the route draws at random.
    #[serde(default = "one")]
    pub weight: u32,
}

/// Something other than an answer: the faults a client's error handling meets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fault {
    #[default]
    None,
    /// No answer: the request is held until the client gives up (at most `HOLD`).
    Timeout,
    /// The connection is closed without an answer (after the delay).
    Reset,
}

/// On an OSC message whose address and arguments match, reply with another.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OscRule {
    /// An OSC 1.0 address pattern; may use parameters.
    pub address: String,
    #[serde(default)]
    pub args: Vec<ArgRule>,
    /// `None`: take the message and answer nothing.
    #[serde(default)]
    pub reply: Option<OscOut>,
    /// Where the reply goes: empty for the sender.
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub jitter_ms: u64,
}

/// A reply message; its address and argument values are templates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OscOut {
    pub address: String,
    #[serde(default)]
    pub args: Vec<ArgOut>,
}

/// An argument of a reply: its type, and a template for its value
/// (`{{request.args[0]}}` echoes one as a number when the type is a number).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArgOut {
    #[serde(rename = "type")]
    pub kind: ArgType,
    #[serde(default)]
    pub value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgType {
    Int,
    Float,
    Str,
    Long,
    Double,
    Bool,
    /// Hex bytes.
    Blob,
    Nil,
}

/// On a datagram whose payload matches, reply with another.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UdpRule {
    #[serde(default)]
    pub mode: UdpMode,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub reply: Option<RawPayload>,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub jitter_ms: u64,
}

/// On a line (or chunk) that matches, reply on the connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TcpRule {
    #[serde(default)]
    pub mode: UdpMode,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub reply: Option<RawPayload>,
    /// Close the connection after the reply.
    #[serde(default)]
    pub close: bool,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub jitter_ms: u64,
}

/// What ends one message on a TCP connection; replies end with it too.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Delimiter {
    /// `\n` (a `\r` before it is dropped).
    #[default]
    Lf,
    Crlf,
    Cr,
    /// Every chunk that arrives is a message; replies are sent as written.
    None,
}

impl Delimiter {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Delimiter::Lf => b"\n",
            Delimiter::Crlf => b"\r\n",
            Delimiter::Cr => b"\r",
            Delimiter::None => b"",
        }
    }
}

fn any_method() -> String {
    "ANY".into()
}

fn ok_status() -> u16 {
    200
}

fn one() -> u32 {
    1
}

// ---- checks ---------------------------------------------------------------------

fn required(field: Field) -> EngineError {
    EngineError::new("node.required").in_field(field)
}

fn range(field: Field, min: u64, max: u64) -> EngineError {
    EngineError::new("node.range").with("min", min).with("max", max).in_field(field)
}

fn too_long(field: Field, max: usize) -> EngineError {
    EngineError::new("node.too_long").with("max", max).in_field(field)
}

/// A field fixed when the emulator starts: text and parameters only.
fn fixed(text: &str, field: Field, params: &BTreeMap<String, String>) -> EngineResult<String> {
    let parsed = template::parse(text).map_err(|error| error.in_field(field.clone()))?;
    for reference in parsed.refs() {
        let problem = match reference {
            Ref::Param(name) | Ref::Bare(name) if params.contains_key(&name) => continue,
            Ref::Param(name) => EngineError::new("param.unknown").with("name", name),
            Ref::Secret(_) => EngineError::new("emulator.secret_unsupported"),
            Ref::Bare(name) | Ref::Var(name) => EngineError::new("emulator.params_only").with("name", name),
        };
        return Err(problem.in_field(field));
    }
    // Generators (`{{uuid}}`) change from one request to the next; a pattern cannot.
    if !parsed.is_static(params) {
        return Err(EngineError::new("emulator.params_only").with("name", text).in_field(field));
    }
    data::static_text(text, params).ok_or_else(|| EngineError::new("emulator.params_only").with("name", text).in_field(field))
}

/// A field rendered for each reply: it may read what arrived (`request`) when
/// `request` is true, parameters and the generators — nothing else.
fn reply_template(text: &str, field: Field, params: &BTreeMap<String, String>, request: bool) -> EngineResult<()> {
    if text.len() > MAX_TEXT {
        return Err(too_long(field, MAX_TEXT));
    }
    let parsed = template::parse(text).map_err(|error| error.in_field(field.clone()))?;
    for reference in parsed.refs() {
        let problem = match reference {
            Ref::Bare(name) if request && name == "request" => continue,
            Ref::Param(name) | Ref::Bare(name) if params.contains_key(&name) => continue,
            Ref::Param(name) => EngineError::new("param.unknown").with("name", name),
            Ref::Secret(_) => EngineError::new("emulator.secret_unsupported"),
            Ref::Bare(name) | Ref::Var(name) => EngineError::new("emulator.name_unknown").with("name", name),
        };
        return Err(problem.in_field(field));
    }
    Ok(())
}

fn check_delay(delay_ms: u64, jitter_ms: u64) -> EngineResult<()> {
    if delay_ms > MAX_DELAY_MS {
        return Err(range(Field::new("delay_ms"), 0, MAX_DELAY_MS));
    }
    if jitter_ms > MAX_DELAY_MS {
        return Err(range(Field::new("jitter_ms"), 0, MAX_DELAY_MS));
    }
    Ok(())
}

/// Where a reply goes: empty for the sender, else an `IP:port`.
fn reply_target(to: &str, params: &BTreeMap<String, String>) -> EngineResult<Option<SocketAddr>> {
    if to.trim().is_empty() {
        return Ok(None);
    }
    let text = fixed(to, Field::new("to"), params)?;
    text.trim()
        .parse::<SocketAddr>()
        .map(Some)
        .map_err(|_| EngineError::new("node.target_invalid").with("value", text).in_field(Field::new("to")))
}

fn header_name_valid(name: &str) -> bool {
    hyper::header::HeaderName::from_bytes(name.trim().as_bytes()).is_ok()
}

/// `ANY` (or nothing) is every method.
fn method_of(method: &str) -> EngineResult<Option<String>> {
    let method = method.trim().to_ascii_uppercase();
    if method.is_empty() || method == "ANY" {
        return Ok(None);
    }
    hyper::Method::from_bytes(method.as_bytes())
        .map(|_| Some(method.clone()))
        .map_err(|_| EngineError::new("node.method_invalid").with("value", method).in_field(Field::new("method")))
}

/// An HTTP path pattern: literal segments, `:name` segments, and a final `*`.
#[derive(Clone, Debug, PartialEq)]
pub struct PathPattern {
    segments: Vec<PathSegment>,
    rest: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum PathSegment {
    Literal(String),
    Param(String),
}

/// Path segments without the leading slash; one trailing slash means nothing.
fn path_segments(path: &str) -> Vec<&str> {
    match path.strip_suffix('/') {
        Some("") | None if path.len() <= 1 => Vec::new(),
        Some(trimmed) => trimmed.split('/').skip(1).collect(),
        None => path.split('/').skip(1).collect(),
    }
}

impl PathPattern {
    pub fn parse(pattern: &str) -> EngineResult<PathPattern> {
        let field = || Field::new("path");
        if pattern.len() > MAX_PATH {
            return Err(too_long(field(), MAX_PATH));
        }
        if !pattern.starts_with('/') {
            return Err(EngineError::new("emulator.path_slash").with("value", pattern).in_field(field()));
        }
        let parts = path_segments(pattern);
        let mut segments = Vec::new();
        let mut rest = false;
        for (index, part) in parts.iter().enumerate() {
            if *part == "*" {
                if index + 1 != parts.len() {
                    return Err(EngineError::new("emulator.path_rest").with("value", pattern).in_field(field()));
                }
                rest = true;
            } else if let Some(name) = part.strip_prefix(':') {
                if !template::is_ident(name) {
                    return Err(EngineError::new("emulator.path_param").with("value", part).in_field(field()));
                }
                segments.push(PathSegment::Param(name.to_string()));
            } else if part.contains('*') {
                return Err(EngineError::new("emulator.path_rest").with("value", pattern).in_field(field()));
            } else {
                segments.push(PathSegment::Literal(part.to_string()));
            }
        }
        Ok(PathPattern { segments, rest })
    }

    /// The named segments of `path`, decoded, when it matches.
    pub fn matches(&self, path: &str) -> Option<Value> {
        let parts = path_segments(path);
        let fits = if self.rest { parts.len() >= self.segments.len() } else { parts.len() == self.segments.len() };
        if !fits {
            return None;
        }
        let mut params = serde_json::Map::new();
        for (segment, part) in self.segments.iter().zip(&parts) {
            match segment {
                PathSegment::Literal(literal) if literal == part => {}
                PathSegment::Literal(_) => return None,
                PathSegment::Param(name) => {
                    params.insert(name.clone(), Value::String(percent_decode(part, false)));
                }
            }
        }
        Some(Value::Object(params))
    }
}

/// `%41` → `A` (and `+` → space in a query); a malformed escape stays as written.
pub fn percent_decode(text: &str, plus: bool) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let hex = |at: usize| bytes.get(at).and_then(|digit| (*digit as char).to_digit(16));
        match byte {
            b'%' => match (hex(index + 1), hex(index + 2)) {
                (Some(high), Some(low)) => {
                    out.push((high * 16 + low) as u8);
                    index += 3;
                    continue;
                }
                _ => out.push(byte),
            },
            b'+' if plus => out.push(b' '),
            _ => out.push(byte),
        }
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A condition with its value resolved and its pattern or path compiled.
#[derive(Clone, Debug)]
pub struct Check {
    on: ConditionOn,
    name: String,
    path: Vec<Segment>,
    op: CompareOp,
    value: String,
    regex: Option<regex::Regex>,
}

impl Check {
    /// `condition` with its name and value already resolved; problems name `field`.
    pub fn new(condition: &Condition, field: Field) -> EngineResult<Check> {
        let name = condition.name.trim().to_string();
        let mut path = Vec::new();
        match condition.on {
            ConditionOn::Header if name.is_empty() => return Err(required(field)),
            ConditionOn::Header if !header_name_valid(&name) => {
                return Err(EngineError::new("node.header_invalid").with("value", &name).in_field(field));
            }
            ConditionOn::Query if name.is_empty() => return Err(required(field)),
            ConditionOn::Json => path = template::parse_json_path(if name.is_empty() { "$" } else { &name }).map_err(|error| error.in_field(field.clone()))?,
            _ => {}
        }
        let regex = match condition.op {
            CompareOp::Matches => Some(matching::compile_regex(&condition.value).map_err(|error| error.in_field(field))?),
            _ => None,
        };
        // Header names arrive in lower case; the request's map is keyed that way.
        let name = if condition.on == ConditionOn::Header { name.to_ascii_lowercase() } else { name };
        Ok(Check { on: condition.on, name, path, op: condition.op, value: condition.value.clone(), regex })
    }

    /// Whether `request` (as templates read it) satisfies it. A comparison
    /// that cannot be made (text against a number) does not hold.
    pub fn holds(&self, request: &Value) -> bool {
        let text = |value: Option<&Value>| value.map(template::value_text).unwrap_or_default();
        let actual = match self.on {
            ConditionOn::Header => text(request["headers"].get(&self.name)),
            ConditionOn::Query => text(request["query"].get(&self.name)),
            ConditionOn::Body => text(request.get("body")),
            ConditionOn::Json => text(template::lookup(&request["json"], &self.path).filter(|value| !value.is_null())),
        };
        match &self.regex {
            Some(regex) => regex.is_match(&actual),
            None => matching::compare(&actual, self.op, &self.value).is_ok_and(|(holds, _)| holds),
        }
    }
}

/// Which HTTP requests something takes: a method, a path and conditions. A
/// route's and a *Wait for HTTP request*'s, so both read a request alike.
#[derive(Clone, Debug)]
pub struct RequestMatcher {
    method: Option<String>,
    path: PathPattern,
    when: Vec<Check>,
}

impl RequestMatcher {
    /// From resolved texts: `path` and the conditions' names and values hold no templates any more.
    pub fn new(method: &str, path: &str, when: &[Condition]) -> EngineResult<RequestMatcher> {
        if when.len() > MAX_CONDITIONS {
            return Err(EngineError::new("emulator.conditions_too_many").with("max", MAX_CONDITIONS).in_field(Field::new("conditions")));
        }
        let when = when.iter().enumerate().map(|(index, condition)| Check::new(condition, Field::nth("condition", index + 1))).collect::<EngineResult<_>>()?;
        Ok(RequestMatcher { method: method_of(method)?, path: PathPattern::parse(path.trim())?, when })
    }

    /// The path's named segments when `request` is one of these.
    pub fn accepts(&self, request: &Value) -> Option<Value> {
        let method = request["method"].as_str()?;
        if let Some(wanted) = &self.method {
            if wanted != method && !(wanted == "GET" && method == "HEAD") {
                return None;
            }
        }
        let params = self.path.matches(request["path"].as_str()?)?;
        self.when.iter().all(|check| check.holds(request)).then_some(params)
    }
}

/// A *Wait for HTTP request* matches what the run's HTTP listener received.
impl Matcher for RequestMatcher {
    fn matches(&self, datagram: &Datagram) -> Option<Value> {
        let request = datagram.request.as_ref()?;
        let params = self.accepts(request)?;
        let mut value = request.clone();
        value["params"] = params;
        Some(value)
    }
}

/// An emulator ready to serve: patterns compiled, fixed fields resolved.
pub(crate) struct Compiled {
    pub name: String,
    pub bind: SocketAddr,
    pub params: BTreeMap<String, String>,
    pub rules: Rules,
}

pub(crate) enum Rules {
    Http { routes: Vec<CompiledRoute>, fallback: Option<Response> },
    Osc(Vec<OscResponder>),
    Udp(Vec<UdpResponder>),
    Tcp { delimiter: Delimiter, greeting: String, rules: Vec<TcpResponder> },
}

pub(crate) struct CompiledRoute {
    pub matcher: RequestMatcher,
    pub order: Order,
    pub responses: Vec<Response>,
}

impl CompiledRoute {
    /// The response for the route's `count`th request (1-based) and its index.
    pub fn pick(&self, seed: u64, rule: usize, count: u64) -> (usize, &Response) {
        let index = pick(&self.responses, self.order, seed, rule, count);
        (index, &self.responses[index])
    }
}

/// Which of `responses` the `count`th request of rule `rule` gets.
pub fn pick(responses: &[Response], order: Order, seed: u64, rule: usize, count: u64) -> usize {
    let n = responses.len().max(1);
    let nth = count.saturating_sub(1) as usize;
    match order {
        Order::Sequence => nth.min(n - 1),
        Order::Cycle => nth % n,
        Order::Random => {
            let total: u64 = responses.iter().map(|response| response.weight as u64).sum();
            if total == 0 {
                return 0;
            }
            let mut draw = Rng::for_node(seed ^ PICK_STREAM, &format!("rule-{rule}"), count).below_u64(total);
            for (index, response) in responses.iter().enumerate() {
                if draw < response.weight as u64 {
                    return index;
                }
                draw -= response.weight as u64;
            }
            0
        }
    }
}

pub(crate) struct OscResponder {
    pub matcher: OscMatcher,
    pub reply: Option<OscOut>,
    pub to: Option<SocketAddr>,
    pub delay_ms: u64,
    pub jitter_ms: u64,
}

pub(crate) struct UdpResponder {
    pub matcher: UdpMatcher,
    pub reply: Option<RawPayload>,
    pub to: Option<SocketAddr>,
    pub delay_ms: u64,
    pub jitter_ms: u64,
}

pub(crate) struct TcpResponder {
    pub matcher: UdpMatcher,
    pub reply: Option<RawPayload>,
    pub close: bool,
    pub delay_ms: u64,
    pub jitter_ms: u64,
}

fn check_response(response: &Response, params: &BTreeMap<String, String>) -> EngineResult<()> {
    if !(100..=599).contains(&response.status) {
        return Err(range(Field::new("status"), 100, 599));
    }
    if response.headers.len() > MAX_HEADERS {
        return Err(EngineError::new("emulator.headers_too_many").with("max", MAX_HEADERS).in_field(Field::new("headers")));
    }
    for (index, (name, value)) in response.headers.iter().enumerate() {
        let name_field = Field::nth("header_name", index + 1);
        if name.trim().is_empty() {
            return Err(required(name_field));
        }
        let name = fixed(name, name_field.clone(), params)?;
        if !header_name_valid(&name) {
            return Err(EngineError::new("node.header_invalid").with("value", name).in_field(name_field));
        }
        let value_field = Field::nth("header_value", index + 1);
        reply_template(value, value_field.clone(), params, true)?;
        if let Some(text) = data::static_text(value, params) {
            if hyper::header::HeaderValue::from_str(&text).is_err() {
                return Err(EngineError::new("emulator.header_value_invalid").in_field(value_field));
            }
        }
    }
    reply_template(&response.body, Field::new("body"), params, true)?;
    check_delay(response.delay_ms, response.jitter_ms)
}

fn compile_route(route: &Route, params: &BTreeMap<String, String>) -> EngineResult<CompiledRoute> {
    if route.path.trim().is_empty() {
        return Err(required(Field::new("path")));
    }
    let path = fixed(&route.path, Field::new("path"), params)?;
    let when = route
        .when
        .iter()
        .enumerate()
        .map(|(index, condition)| {
            let field = Field::nth("condition", index + 1);
            Ok(Condition { name: fixed(&condition.name, field.clone(), params)?, value: fixed(&condition.value, field, params)?, ..condition.clone() })
        })
        .collect::<EngineResult<Vec<_>>>()?;
    let matcher = RequestMatcher::new(&route.method, &path, &when)?;
    if route.responses.is_empty() {
        return Err(EngineError::new("emulator.response_required").in_field(Field::new("responses")));
    }
    if route.responses.len() > MAX_RESPONSES {
        return Err(EngineError::new("emulator.responses_too_many").with("max", MAX_RESPONSES).in_field(Field::new("responses")));
    }
    for (index, response) in route.responses.iter().enumerate() {
        check_response(response, params).map_err(|error| error.with("response", index + 1))?;
    }
    if route.order == Order::Random && route.responses.iter().all(|response| response.weight == 0) {
        return Err(EngineError::new("emulator.weights_zero").in_field(Field::new("weight")));
    }
    Ok(CompiledRoute { matcher, order: route.order, responses: route.responses.clone() })
}

/// A reply argument's text as the type it says; empty text is that type's zero.
pub fn osc_arg(kind: ArgType, text: &str) -> EngineResult<OscArg> {
    let value = text.trim();
    let invalid = || EngineError::new("emulator.arg_value").with("value", text).with("type", serde_json::to_value(kind).ok().and_then(|kind| kind.as_str().map(str::to_string)).unwrap_or_default());
    fn parsed<T: std::str::FromStr + Default>(value: &str) -> Option<T> {
        if value.is_empty() { Some(T::default()) } else { value.parse().ok() }
    }
    Ok(match kind {
        ArgType::Int => OscArg::Int(parsed(value).ok_or_else(invalid)?),
        ArgType::Long => OscArg::Long(parsed(value).ok_or_else(invalid)?),
        ArgType::Float => OscArg::Float(parsed::<f32>(value).filter(|number| number.is_finite()).ok_or_else(invalid)?),
        ArgType::Double => OscArg::Double(parsed::<f64>(value).filter(|number| number.is_finite()).ok_or_else(invalid)?),
        ArgType::Str => OscArg::Str(text.to_string()),
        ArgType::Bool => match value.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => OscArg::Bool(true),
            "false" | "0" | "no" | "off" | "" => OscArg::Bool(false),
            _ => return Err(invalid()),
        },
        ArgType::Blob if value.is_empty() => OscArg::Blob(Vec::new()),
        ArgType::Blob => OscArg::Blob(matching::parse_hex(value).map_err(|_| invalid())?),
        ArgType::Nil => OscArg::Nil,
    })
}

fn check_osc_out(out: &OscOut, params: &BTreeMap<String, String>) -> EngineResult<()> {
    let field = Field::new("reply_address");
    if out.address.trim().is_empty() {
        return Err(required(field));
    }
    reply_template(&out.address, field.clone(), params, true)?;
    if let Some(address) = data::static_text(&out.address, params).filter(|address| !address.starts_with('/')) {
        return Err(EngineError::new("node.osc_address").with("value", address).in_field(field));
    }
    if out.args.len() > MAX_ARGS {
        return Err(EngineError::new("emulator.args_too_many").with("max", MAX_ARGS).in_field(Field::new("reply_args")));
    }
    for (index, arg) in out.args.iter().enumerate() {
        let field = Field::nth("reply_arg", index + 1);
        reply_template(&arg.value, field.clone(), params, true)?;
        if let Some(text) = data::static_text(&arg.value, params) {
            osc_arg(arg.kind, &text).map_err(|error| error.in_field(field))?;
        }
    }
    Ok(())
}

fn check_raw(reply: &RawPayload, params: &BTreeMap<String, String>) -> EngineResult<()> {
    let field = Field::new("reply");
    match reply {
        RawPayload::Text { text } => {
            if text.len() > MAX_DATAGRAM {
                return Err(too_long(field, MAX_DATAGRAM));
            }
            reply_template(text, field, params, true)
        }
        RawPayload::Hex { hex } => {
            reply_template(hex, field.clone(), params, true)?;
            match data::static_text(hex, params) {
                Some(text) => matching::parse_hex(&text).map(|_| ()).map_err(|error| error.in_field(field)),
                None => Ok(()),
            }
        }
    }
}

fn payload_matcher(mode: UdpMode, pattern: &str, params: &BTreeMap<String, String>) -> EngineResult<UdpMatcher> {
    let field = Field::new("pattern");
    if mode != UdpMode::Any && pattern.is_empty() {
        return Err(required(field));
    }
    let pattern = if mode == UdpMode::Any { String::new() } else { fixed(pattern, field.clone(), params)? };
    UdpMatcher::new(mode, &pattern).map_err(|error| error.in_field(field))
}

fn compile_osc(rule: &OscRule, params: &BTreeMap<String, String>) -> EngineResult<OscResponder> {
    if rule.address.trim().is_empty() {
        return Err(required(Field::new("address")));
    }
    let address = fixed(&rule.address, Field::new("address"), params)?;
    if rule.args.len() > MAX_ARGS {
        return Err(EngineError::new("node.rules_too_many").with("max", MAX_ARGS).in_field(Field::new("rules")));
    }
    let mut conditions = Vec::new();
    for (index, arg) in rule.args.iter().enumerate() {
        let field = Field::nth("rule_value", index + 1);
        if arg.index > matching::MAX_ARG_INDEX {
            return Err(range(Field::nth("rule", index + 1), 0, matching::MAX_ARG_INDEX as u64));
        }
        conditions.push(ArgRule { value: fixed(&arg.value, field, params)?, ..arg.clone() });
    }
    let matcher = OscMatcher::new(&address, &conditions)?;
    if let Some(out) = &rule.reply {
        check_osc_out(out, params)?;
    }
    check_delay(rule.delay_ms, rule.jitter_ms)?;
    Ok(OscResponder { matcher, reply: rule.reply.clone(), to: reply_target(&rule.to, params)?, delay_ms: rule.delay_ms, jitter_ms: rule.jitter_ms })
}

fn compile_udp(rule: &UdpRule, params: &BTreeMap<String, String>) -> EngineResult<UdpResponder> {
    let matcher = payload_matcher(rule.mode, &rule.pattern, params)?;
    if let Some(reply) = &rule.reply {
        check_raw(reply, params)?;
    }
    check_delay(rule.delay_ms, rule.jitter_ms)?;
    Ok(UdpResponder { matcher, reply: rule.reply.clone(), to: reply_target(&rule.to, params)?, delay_ms: rule.delay_ms, jitter_ms: rule.jitter_ms })
}

fn compile_tcp(rule: &TcpRule, params: &BTreeMap<String, String>) -> EngineResult<TcpResponder> {
    let matcher = payload_matcher(rule.mode, &rule.pattern, params)?;
    if let Some(reply) = &rule.reply {
        check_raw(reply, params)?;
    }
    check_delay(rule.delay_ms, rule.jitter_ms)?;
    Ok(TcpResponder { matcher, reply: rule.reply.clone(), close: rule.close, delay_ms: rule.delay_ms, jitter_ms: rule.jitter_ms })
}

/// Every problem of an emulator before it starts, naming the rule (`rule`),
/// the response (`response`) and the field it is in. `params` are the values
/// its templates may read besides `request`: a run's parameters.
pub fn check(emulator: &Emulator, params: &BTreeMap<String, String>) -> EngineResult<()> {
    compile(emulator, params).map(|_| ())
}

pub(crate) fn compile(emulator: &Emulator, params: &BTreeMap<String, String>) -> EngineResult<Compiled> {
    let name = emulator.name.trim();
    if name.is_empty() {
        return Err(required(Field::new("name")));
    }
    if name.chars().count() > MAX_NAME {
        return Err(too_long(Field::new("name"), MAX_NAME));
    }
    let bind = check_bind(&emulator.bind)?;
    if emulator.kind.rules() > MAX_RULES {
        return Err(EngineError::new("emulator.rules_too_many").with("max", MAX_RULES).in_field(Field::new("rules")));
    }
    let at = |index: usize| move |error: EngineError| error.with("rule", index + 1);
    let rules = match &emulator.kind {
        EmulatorKind::Http { routes, fallback } => {
            let routes = routes.iter().enumerate().map(|(index, route)| compile_route(route, params).map_err(at(index))).collect::<EngineResult<_>>()?;
            if let Some(fallback) = fallback {
                check_response(fallback, params).map_err(|error| error.in_field(Field::new("fallback")))?;
            }
            Rules::Http { routes, fallback: fallback.clone() }
        }
        EmulatorKind::Osc { rules } => Rules::Osc(rules.iter().enumerate().map(|(index, rule)| compile_osc(rule, params).map_err(at(index))).collect::<EngineResult<_>>()?),
        EmulatorKind::Udp { rules } => Rules::Udp(rules.iter().enumerate().map(|(index, rule)| compile_udp(rule, params).map_err(at(index))).collect::<EngineResult<_>>()?),
        EmulatorKind::Tcp { delimiter, greeting, rules } => {
            // A greeting goes out before anything has arrived: no `request` yet.
            reply_template(greeting, Field::new("greeting"), params, false)?;
            let rules = rules.iter().enumerate().map(|(index, rule)| compile_tcp(rule, params).map_err(at(index))).collect::<EngineResult<_>>()?;
            Rules::Tcp { delimiter: *delimiter, greeting: greeting.clone(), rules }
        }
    };
    Ok(Compiled { name: name.to_string(), bind, params: params.clone(), rules })
}

// ---- what every protocol shares while it serves ---------------------------------

/// One request (message, line) and what became of it.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Exchange {
    pub seq: u64,
    pub ts: u64,
    pub from: String,
    /// What arrived, in protocol notation: `GET /users/7`, `/ping 1`, `POWER?`.
    pub request: String,
    /// The rule that answered (1-based); none when no rule took it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<usize>,
    /// What went back, in protocol notation; empty when nothing did.
    pub reply: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault: Option<Fault>,
    /// From arrival until the reply left, its delay included.
    pub ms: u64,
    /// Why no reply could be made or sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EngineError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<u64>,
    /// What arrived as templates read it (`request`), a long body cut short.
    #[serde(skip_serializing_if = "Value::is_null")]
    pub data: Value,
}

/// Requests, how many no rule took, how many failed, and the hits per rule.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Counts {
    pub total: u64,
    pub unmatched: u64,
    pub failed: u64,
    pub hits: Vec<u64>,
}

#[derive(Default)]
struct Fresh {
    exchanges: Vec<Exchange>,
    dropped: u64,
    changed: bool,
}

/// What one emulator has seen so far: counters and the newest exchanges.
pub struct Emulation {
    pub name: String,
    pub protocol: &'static str,
    pub local: SocketAddr,
    hits: Vec<AtomicU64>,
    total: AtomicU64,
    unmatched: AtomicU64,
    failed: AtomicU64,
    seq: AtomicU64,
    recent: Mutex<VecDeque<Exchange>>,
    fresh: Mutex<Fresh>,
}

impl Emulation {
    pub(crate) fn new(name: &str, protocol: &'static str, local: SocketAddr, rules: usize) -> Arc<Self> {
        Arc::new(Emulation {
            name: name.to_string(),
            protocol,
            local,
            hits: (0..rules).map(|_| AtomicU64::new(0)).collect(),
            total: AtomicU64::new(0),
            unmatched: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            seq: AtomicU64::new(0),
            recent: Mutex::new(VecDeque::new()),
            fresh: Mutex::new(Fresh::default()),
        })
    }

    /// Count a hit of rule `index` (0-based): how many it has had, this one included.
    pub(crate) fn hit(&self, index: usize) -> u64 {
        self.hits.get(index).map(|hits| hits.fetch_add(1, Ordering::Relaxed) + 1).unwrap_or(1)
    }

    /// Requests no rule took so far, plus this one: the fallback's `{{counter}}`.
    pub(crate) fn next_unmatched(&self) -> u64 {
        self.unmatched.load(Ordering::Relaxed) + 1
    }

    pub(crate) fn record(&self, mut exchange: Exchange) {
        exchange.seq = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
        if exchange.ts == 0 {
            exchange.ts = now_ms();
        }
        self.total.fetch_add(1, Ordering::Relaxed);
        if exchange.rule.is_none() {
            self.unmatched.fetch_add(1, Ordering::Relaxed);
        }
        if exchange.error.is_some() {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
        {
            let mut recent = self.recent.lock().unwrap();
            if recent.len() == RECENT {
                recent.pop_front();
            }
            recent.push_back(exchange.clone());
        }
        let mut fresh = self.fresh.lock().unwrap();
        fresh.changed = true;
        if fresh.exchanges.len() >= BATCH {
            fresh.dropped += 1;
        } else {
            // What arrived stays with the command that asks for it; events stay small.
            fresh.exchanges.push(Exchange { data: Value::Null, ..exchange });
        }
    }

    pub fn counts(&self) -> Counts {
        Counts {
            total: self.total.load(Ordering::Relaxed),
            unmatched: self.unmatched.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            hits: self.hits.iter().map(|hits| hits.load(Ordering::Relaxed)).collect(),
        }
    }

    /// Kept exchanges after `after` (a `seq`), oldest first, at most `limit`.
    pub fn exchanges(&self, after: u64, limit: usize) -> Vec<Exchange> {
        let recent = self.recent.lock().unwrap();
        recent.iter().filter(|exchange| exchange.seq > after).take(limit).cloned().collect()
    }

    fn take_fresh(&self) -> Option<(Vec<Exchange>, u64)> {
        let mut fresh = self.fresh.lock().unwrap();
        if !fresh.changed {
            return None;
        }
        let taken = std::mem::take(&mut *fresh);
        Some((taken.exchanges, taken.dropped))
    }

    pub fn summary(&self, node: Option<&str>) -> EmulatorSummary {
        EmulatorSummary { node: node.map(str::to_string), name: self.name.clone(), protocol: self.protocol, local: self.local.to_string(), counts: self.counts() }
    }
}

/// An emulator's counters in a run's report.
#[derive(Clone, Debug, Serialize)]
pub struct EmulatorSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    pub name: String,
    pub protocol: &'static str,
    pub local: String,
    pub counts: Counts,
}

/// A request body as an exchange keeps it: the start of a long one, and no JSON then.
pub(crate) fn kept(request: &Value) -> Value {
    let mut value = request.clone();
    if let Some(body) = request.get("body").and_then(Value::as_str) {
        if body.len() > KEPT_BODY {
            let mut end = KEPT_BODY;
            while !body.is_char_boundary(end) {
                end -= 1;
            }
            value["body"] = Value::String(format!("{}…", &body[..end]));
            value["json"] = Value::Null;
        }
    }
    value
}

/// Everything a serving emulator reads: its rules, its counters, where it reports.
pub(crate) struct Context {
    pub host: Host,
    pub compiled: Compiled,
    pub emulation: Arc<Emulation>,
    pub seed: u64,
    /// The job its frames belong to: the emulator's own, or the run's.
    pub job: Option<u64>,
    /// What a run's *Wait for HTTP request* steps read (HTTP only).
    pub inbox: Option<Arc<Inbox>>,
    gate: Gate,
}

impl Context {
    pub(crate) fn new(host: Host, compiled: Compiled, emulation: Arc<Emulation>, seed: u64, job: Option<u64>, inbox: Option<Arc<Inbox>>) -> Arc<Self> {
        Arc::new(Context { host, compiled, emulation, seed, job, inbox, gate: Gate::new(FRAME_GAP_MS) })
    }

    /// Render the fields of rule `rule`'s reply (1-based; 0 for a fallback or
    /// a greeting) at its `count`th hit, with one renderer, so the generators
    /// of one reply draw one stream.
    pub(crate) fn render<T>(&self, rule: usize, count: u64, request: &Value, work: impl FnOnce(&mut Renderer<'_>) -> EngineResult<T>) -> EngineResult<T> {
        let vars = BTreeMap::from([("request".to_string(), request.clone())]);
        let secrets = BTreeMap::new();
        let node = format!("rule-{rule}");
        let scope = Scope { params: &self.compiled.params, vars: &vars, secrets: &secrets, run_id: self.job.unwrap_or(0), seed: self.seed, node_id: &node, count, now_ms: now_ms() };
        let mut renderer = Renderer::new(scope);
        work(&mut renderer).map_err(|error| if rule > 0 { error.with("rule", rule) } else { error })
    }

    /// The delay before rule `rule`'s reply: its own plus seeded jitter.
    pub(crate) fn delay(&self, rule: usize, count: u64, delay_ms: u64, jitter_ms: u64) -> Duration {
        let jitter = if jitter_ms == 0 { 0 } else { Rng::for_node(self.seed ^ DELAY_STREAM, &format!("rule-{rule}"), count).below_u64(jitter_ms + 1) };
        Duration::from_millis(delay_ms + jitter)
    }

    /// Whether this exchange goes to the Inspector — decided once, so its
    /// request and its reply are both there or neither.
    pub(crate) fn capturing(&self) -> bool {
        inspect::armed(&self.host) && self.gate.allow()
    }

    pub(crate) fn publish(&self, frame: Frame) -> Option<u64> {
        let frame = frame.local(self.emulation.local);
        let frame = match self.job {
            Some(id) => frame.job(id),
            None => frame,
        };
        inspect::publish(&self.host, frame)
    }
}

// ---- a job of its own -----------------------------------------------------------

/// The sockets of running emulators, by job, for whoever asks what arrived.
#[derive(Clone, Default)]
pub struct EmulatorHub {
    running: Arc<Mutex<HashMap<u64, Weak<Emulation>>>>,
}

impl EmulatorHub {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&self, id: u64, emulation: &Arc<Emulation>) {
        let mut running = self.running.lock().unwrap();
        running.retain(|_, emulation| emulation.strong_count() > 0);
        running.insert(id, Arc::downgrade(emulation));
    }

    /// The emulator job `id`, while it runs.
    pub fn get(&self, id: u64) -> EngineResult<Arc<Emulation>> {
        self.running.lock().unwrap().get(&id).and_then(Weak::upgrade).ok_or_else(|| EngineError::new("emulator.not_running").with("id", id))
    }
}

/// What `emulator_exchanges` answers: where it listens, its counters, and what arrived.
#[derive(Serialize)]
pub struct Snapshot {
    pub job_id: u64,
    pub name: String,
    pub protocol: &'static str,
    pub local: String,
    pub counts: Counts,
    pub exchanges: Vec<Exchange>,
}

impl EmulatorHub {
    pub fn snapshot(&self, id: u64, after: u64, limit: usize) -> EngineResult<Snapshot> {
        let emulation = self.get(id)?;
        Ok(Snapshot {
            job_id: id,
            name: emulation.name.clone(),
            protocol: emulation.protocol,
            local: emulation.local.to_string(),
            counts: emulation.counts(),
            exchanges: emulation.exchanges(after, limit.clamp(1, RECENT)),
        })
    }
}

/// How a job of its own starts besides its document.
#[derive(Clone, Debug, Default)]
pub struct StartOptions {
    /// Values its templates read as parameters (an experiment's, for its node's *Start now*).
    pub params: BTreeMap<String, String>,
    /// `None`: a fresh seed.
    pub seed: Option<u64>,
    /// The library entry it was started from, for the screen that lists them.
    pub source: Option<String>,
}

/// A port taken a moment ago by an emulator or a run that just stopped is
/// released asynchronously: starting again right away retries briefly.
const RELEASE_RETRIES: u32 = 10;
const RELEASE_PAUSE: Duration = Duration::from_millis(20);

/// The socket an emulator listens on: bound before anything else, so a taken
/// port is an error on the Start button.
pub(crate) enum Socket {
    Tcp(tokio::net::TcpListener),
    Udp(Arc<tokio::net::UdpSocket>),
}

impl Socket {
    pub(crate) async fn open(bind: SocketAddr, over_tcp: bool) -> EngineResult<Socket> {
        let mut attempt = 0;
        loop {
            let bound = if over_tcp {
                tokio::net::TcpListener::bind(bind).await.map(Socket::Tcp)
            } else {
                tokio::net::UdpSocket::bind(bind).await.map(|socket| Socket::Udp(Arc::new(socket)))
            };
            match bound {
                Err(error) if error.kind() == std::io::ErrorKind::AddrInUse && attempt < RELEASE_RETRIES => {
                    attempt += 1;
                    tokio::time::sleep(RELEASE_PAUSE).await;
                }
                Err(error) => return Err(crate::net::bind_error(&bind.to_string(), error).in_field(Field::new("bind"))),
                Ok(socket) => return Ok(socket),
            }
        }
    }

    pub(crate) fn local(&self, bind: SocketAddr) -> SocketAddr {
        match self {
            Socket::Tcp(listener) => listener.local_addr().unwrap_or(bind),
            Socket::Udp(socket) => socket.local_addr().unwrap_or(bind),
        }
    }

    /// Serve until the socket fails; the error says why.
    pub(crate) async fn serve(self, context: Arc<Context>) -> EngineError {
        match (self, &context.compiled.rules) {
            (Socket::Tcp(listener), Rules::Http { .. }) => emulator_http::serve(listener, context).await,
            (Socket::Tcp(listener), Rules::Tcp { .. }) => emulator_net::serve_tcp(listener, context).await,
            (Socket::Udp(socket), _) => emulator_net::serve_datagrams(socket, context).await,
            (Socket::Tcp(_), _) => EngineError::new("emulator.failed"),
        }
    }
}

#[derive(Serialize)]
struct Activity {
    job_id: u64,
    ts: u64,
    counts: Counts,
    exchanges: Vec<Exchange>,
    /// Exchanges that happened but were not sent in this event.
    dropped: u64,
}

fn flush(host: &Host, id: u64, emulation: &Emulation) {
    if let Some((exchanges, dropped)) = emulation.take_fresh() {
        host.emit("emulator://activity", Activity { job_id: id, ts: now_ms(), counts: emulation.counts(), exchanges, dropped });
    }
}

/// Start an emulator as a job: its socket is open when this returns, and
/// `emulator://activity` reports what it answers until it is stopped.
pub async fn start(host: Host, jobs: JobRegistry, hub: EmulatorHub, emulator: Emulator, options: StartOptions) -> EngineResult<JobInfo> {
    data::check_seed(options.seed)?;
    let compiled = compile(&emulator, &options.params)?;
    let socket = Socket::open(compiled.bind, emulator.kind.over_tcp()).await?;
    let local = socket.local(compiled.bind);
    let protocol = emulator.kind.protocol();
    let emulation = Emulation::new(&compiled.name, protocol, local, emulator.kind.rules());
    let id = jobs.next_id();
    let name = compiled.name.clone();
    let mut info = JobInfo::new(id, "emulator", format!("Emulator {name} ({protocol}) on {local}")).with("name", &name).with("protocol", protocol).with("local", local);
    if let Some(source) = &options.source {
        info = info.with("source", source);
    }
    let seed = options.seed.unwrap_or_else(|| rand::random::<u64>() & data::MAX_SEED);
    let context = Context::new(host.clone(), compiled, emulation.clone(), seed, Some(id), None);
    hub.insert(id, &emulation);
    let jobs_cl = jobs.clone();
    let task = tokio::spawn(async move {
        // Stopping the job drops this future and, with the guard, the server
        // and the reporter it spawned.
        let mut guard = TaskGuard::new();
        let serving = tokio::spawn(socket.serve(context));
        guard.watch(serving.abort_handle());
        let reporter = {
            let (host, emulation) = (host.clone(), emulation.clone());
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(REPORT_EVERY).await;
                    flush(&host, id, &emulation);
                }
            })
        };
        guard.watch(reporter.abort_handle());
        let error = serving.await.unwrap_or_else(|panic| EngineError::new("emulator.failed").because(panic));
        drop(guard);
        flush(&host, id, &emulation);
        host.emit("job://ended", json!({ "job_id": id, "kind": "emulator", "error": error }));
        jobs_cl.finish(id);
    });
    jobs.insert(info.clone(), task);
    Ok(info)
}

// ---- inside a run ---------------------------------------------------------------

/// Aborts a task of a run's emulators when the run lets go of it.
struct Serving(tokio::task::AbortHandle);

impl Drop for Serving {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// An HTTP listener of a run: what its *Wait for HTTP request* steps read.
pub struct HttpListener {
    pub local: SocketAddr,
    pub inbox: Arc<Inbox>,
}

/// The run's HTTP listeners, by bind.
pub type HttpListeners = HashMap<SocketAddr, Arc<HttpListener>>;

/// The emulators of a run and the HTTP listeners of its waits, open before
/// its first step. Dropping it closes the TCP ports; the OSC and UDP
/// emulators answer through the run's listeners (`taps`), which close with them.
#[derive(Default)]
pub struct RunEmulators {
    pub http: HttpListeners,
    /// Responders for the run's UDP sockets, by bind; `arm_listeners` gives them their socket.
    pub taps: HashMap<SocketAddr, Arc<dyn Tap>>,
    /// Each Emulator node's counters, for the report.
    pub reports: Vec<(String, Arc<Emulation>)>,
    serving: Vec<Serving>,
}

impl RunEmulators {
    pub fn summaries(&self) -> Vec<EmulatorSummary> {
        self.reports.iter().map(|(node, emulation)| emulation.summary(Some(node))).collect()
    }

    /// Everything that keeps serving; the rest (`http`, `taps`, `reports`) is
    /// read while the run goes on.
    pub fn take_serving(&mut self) -> RunServing {
        RunServing { _tasks: std::mem::take(&mut self.serving) }
    }
}

/// The tasks a run's emulators serve with; dropping it stops them.
#[derive(Default)]
pub struct RunServing {
    _tasks: Vec<Serving>,
}

/// What a wait-only HTTP listener answers: accepted, nothing to say.
fn accepted() -> Response {
    Response { status: 204, headers: Vec::new(), body: String::new(), delay_ms: 0, jitter_ms: 0, fault: Fault::None, weight: 1 }
}

/// Open a run's emulators and the HTTP listeners its waits need. A port that
/// cannot be opened is an error at the node that wanted it, before any traffic.
pub async fn arm_run(host: &Host, nodes: &[Node], params: &BTreeMap<String, String>, seed: u64, job: u64) -> EngineResult<RunEmulators> {
    let mut armed = RunEmulators::default();
    // HTTP: a bind with an emulator's routes, a wait, or both — and the node that asked for it first.
    type Wanted = (String, Option<(Compiled, Arc<Emulation>)>);
    let mut http: BTreeMap<SocketAddr, Wanted> = BTreeMap::new();
    for node in nodes {
        let at = |error: EngineError| error.at(&node.id);
        match &node.kind {
            NodeKind::Emulator { emulator } => {
                let compiled = compile(emulator, params).map_err(at)?;
                let emulation = Emulation::new(&compiled.name, emulator.kind.protocol(), compiled.bind, emulator.kind.rules());
                armed.reports.push((node.id.clone(), emulation.clone()));
                let bind = compiled.bind;
                match emulator.kind {
                    EmulatorKind::Http { .. } => {
                        http.insert(bind, (node.id.clone(), Some((compiled, emulation))));
                    }
                    EmulatorKind::Tcp { .. } => {
                        let Socket::Tcp(listener) = Socket::open(bind, true).await.map_err(at)? else { unreachable!("a TCP bind opens a TCP listener") };
                        let context = Context::new(host.clone(), compiled, emulation, seed, Some(job), None);
                        armed.serving.push(Serving(tokio::spawn(async move { emulator_net::serve_tcp(listener, context).await; }).abort_handle()));
                    }
                    EmulatorKind::Osc { .. } | EmulatorKind::Udp { .. } => {
                        let context = Context::new(host.clone(), compiled, emulation, seed, Some(job), None);
                        armed.taps.insert(bind, emulator_net::Responder::new(context));
                    }
                }
            }
            NodeKind::WaitHttp { bind, .. } => {
                let address = check_bind(bind).map_err(at)?;
                http.entry(address).or_insert_with(|| (node.id.clone(), None));
            }
            _ => {}
        }
    }
    for (bind, (node, emulator)) in http {
        let at = |error: EngineError| error.at(&node);
        let Socket::Tcp(listener) = Socket::open(bind, true).await.map_err(at)? else { unreachable!("an HTTP bind opens a TCP listener") };
        let local = listener.local_addr().unwrap_or(bind);
        let (compiled, emulation) = emulator.unwrap_or_else(|| {
            let compiled = Compiled { name: String::new(), bind, params: params.clone(), rules: Rules::Http { routes: Vec::new(), fallback: Some(accepted()) } };
            (compiled, Emulation::new("", "http", local, 0))
        });
        let inbox = Arc::new(Inbox::default());
        let context = Context::new(host.clone(), compiled, emulation, seed, Some(job), Some(inbox.clone()));
        armed.serving.push(Serving(tokio::spawn(async move { emulator_http::serve(listener, context).await; }).abort_handle()));
        armed.http.insert(bind, Arc::new(HttpListener { local, inbox }));
    }
    Ok(armed)
}

/// Emulators of one run cannot share a port of one transport, and a TCP
/// emulator's port cannot be a wait's HTTP listener; an OSC or UDP
/// emulator's socket is shared with the waits on it, an HTTP emulator's with
/// its *Wait for HTTP request* steps.
pub fn check_run_binds(nodes: &[Node]) -> EngineResult<()> {
    let mut tcp: HashMap<SocketAddr, (&str, bool)> = HashMap::new();
    let mut udp: HashMap<SocketAddr, &str> = HashMap::new();
    let taken = |node: &str, other: &str, bind: &SocketAddr| EngineError::new("emulator.bind_taken").with("target", bind).with("other", other).in_field(Field::new("bind")).at(node);
    for node in nodes {
        let NodeKind::Emulator { emulator } = &node.kind else { continue };
        let Ok(bind) = emulator.bind.trim().parse::<SocketAddr>() else { continue };
        if emulator.kind.over_tcp() {
            let http = matches!(emulator.kind, EmulatorKind::Http { .. });
            if let Some((other, _)) = tcp.insert(bind, (&node.id, http)) {
                return Err(taken(&node.id, other, &bind));
            }
        } else if let Some(other) = udp.insert(bind, &node.id) {
            return Err(taken(&node.id, other, &bind));
        }
    }
    for node in nodes {
        let NodeKind::WaitHttp { bind, .. } = &node.kind else { continue };
        let Ok(bind) = bind.trim().parse::<SocketAddr>() else { continue };
        if let Some((other, false)) = tcp.get(&bind) {
            return Err(taken(&node.id, other, &bind));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(value: Value) -> Emulator {
        serde_json::from_value(value).unwrap()
    }

    fn code(result: EngineResult<()>) -> (String, BTreeMap<String, String>, Option<Field>) {
        let error = result.unwrap_err();
        (error.code.clone(), error.params.clone(), error.field.clone())
    }

    #[test]
    fn a_document_takes_defaults_and_names_its_protocol() {
        let api = doc(json!({ "name": "API", "bind": "127.0.0.1:8080", "protocol": "http",
            "routes": [{ "path": "/health", "responses": [{ "body": "ok" }] }] }));
        let EmulatorKind::Http { routes, fallback } = &api.kind else { panic!() };
        assert_eq!((routes[0].method.as_str(), routes[0].order, routes[0].responses[0].status, routes[0].responses[0].weight), ("ANY", Order::Sequence, 200, 1));
        assert!(fallback.is_none() && api.kind.over_tcp() && api.kind.protocol() == "http");
        let round: Emulator = serde_json::from_value(serde_json::to_value(&api).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(&round).unwrap(), serde_json::to_value(&api).unwrap());
        let tcp = doc(json!({ "name": "Projector", "bind": "127.0.0.1:4352", "protocol": "tcp" }));
        assert!(matches!(tcp.kind, EmulatorKind::Tcp { delimiter: Delimiter::Lf, .. }));
        assert!(check(&tcp, &BTreeMap::new()).is_ok(), "no rules is a device that only listens");
    }

    #[test]
    fn path_patterns_name_segments_and_take_the_rest() {
        let pattern = PathPattern::parse("/users/:id/items").unwrap();
        assert_eq!(pattern.matches("/users/7/items"), Some(json!({ "id": "7" })));
        assert_eq!(pattern.matches("/users/7/items/"), Some(json!({ "id": "7" })), "a trailing slash means nothing");
        assert_eq!(pattern.matches("/users/john%20doe/items"), Some(json!({ "id": "john doe" })));
        assert_eq!(pattern.matches("/users/7"), None);
        assert_eq!(pattern.matches("/Users/7/items"), None, "literal segments are exact");
        let rest = PathPattern::parse("/files/*").unwrap();
        assert!(rest.matches("/files").is_some() && rest.matches("/files/a/b/c").is_some() && rest.matches("/other").is_none());
        let root = PathPattern::parse("/").unwrap();
        assert!(root.matches("/").is_some() && root.matches("/x").is_none());
        for (bad, code) in [("users", "emulator.path_slash"), ("/a/*/b", "emulator.path_rest"), ("/a*", "emulator.path_rest"), ("/:1x", "emulator.path_param")] {
            assert_eq!(PathPattern::parse(bad).unwrap_err().code, code, "{bad}");
        }
        assert_eq!(percent_decode("a+b%2Fc%zz", true), "a b/c%zz");
    }

    fn request(method: &str, path: &str) -> Value {
        json!({ "method": method, "path": path, "query": { "page": "2" }, "headers": { "x-key": "abc", "content-type": "application/json" },
                "body": "{\"name\":\"Ada\",\"age\":36}", "json": { "name": "Ada", "age": 36 }, "params": {}, "from": "127.0.0.1:5000" })
    }

    #[test]
    fn requests_match_by_method_path_and_conditions() {
        let condition = |on: &str, name: &str, op: &str, value: &str| serde_json::from_value::<Condition>(json!({ "on": on, "name": name, "op": op, "value": value })).unwrap();
        let matcher = RequestMatcher::new("post", "/users/:id", &[condition("header", "X-Key", "eq", "abc"), condition("json", "$.age", "ge", "18"), condition("query", "page", "eq", "2")]).unwrap();
        assert_eq!(matcher.accepts(&request("POST", "/users/9")), Some(json!({ "id": "9" })));
        assert!(matcher.accepts(&request("GET", "/users/9")).is_none(), "another method");
        let mut young = request("POST", "/users/9");
        young["json"]["age"] = json!(12);
        assert!(matcher.accepts(&young).is_none(), "a condition that does not hold");
        let body = RequestMatcher::new("ANY", "/*", &[condition("body", "", "contains", "Ada")]).unwrap();
        assert!(body.accepts(&request("DELETE", "/x")).is_some());
        let get = RequestMatcher::new("GET", "/users/:id", &[]).unwrap();
        assert!(get.accepts(&request("HEAD", "/users/1")).is_some(), "a GET route answers HEAD");
        let missing = RequestMatcher::new("ANY", "/", &[condition("header", "x-absent", "empty", "")]).unwrap();
        assert!(missing.accepts(&request("GET", "/")).is_some(), "an absent header is empty");
        assert_eq!(RequestMatcher::new("ANY", "/", &[condition("json", "$.[", "eq", "")]).unwrap_err().code, "json_path.invalid");
        assert_eq!(RequestMatcher::new("BAD METHOD", "/", &[]).unwrap_err().code, "node.method_invalid");
    }

    fn responses(statuses: &[(u16, u32)]) -> Vec<Response> {
        statuses.iter().map(|(status, weight)| Response { status: *status, weight: *weight, ..accepted() }).collect()
    }

    #[test]
    fn sequences_repeat_their_last_cycles_wrap_and_draws_follow_the_seed() {
        let three = responses(&[(500, 1), (500, 1), (200, 1)]);
        assert_eq!((1..=5).map(|count| three[pick(&three, Order::Sequence, 1, 1, count)].status).collect::<Vec<_>>(), [500, 500, 200, 200, 200]);
        assert_eq!((1..=5).map(|count| pick(&three, Order::Cycle, 1, 1, count)).collect::<Vec<_>>(), [0, 1, 2, 0, 1]);
        let mix = responses(&[(200, 80), (429, 10), (500, 10)]);
        let draws: Vec<usize> = (1..=2000).map(|count| pick(&mix, Order::Random, 42, 1, count)).collect();
        let ok = draws.iter().filter(|index| **index == 0).count();
        assert!((1450..=1750).contains(&ok), "about 80 % of 2000: {ok}");
        assert!(draws.contains(&1) && draws.contains(&2));
        assert_eq!(draws, (1..=2000).map(|count| pick(&mix, Order::Random, 42, 1, count)).collect::<Vec<_>>(), "the same seed, the same draws");
        assert_ne!(draws, (1..=2000).map(|count| pick(&mix, Order::Random, 43, 1, count)).collect::<Vec<_>>());
        let never = responses(&[(200, 0), (503, 1)]);
        assert!((1..=50).all(|count| pick(&never, Order::Random, 7, 1, count) == 1), "weight 0 is never drawn");
    }

    #[test]
    fn problems_name_the_rule_the_response_and_the_field() {
        let params = BTreeMap::from([("port".to_string(), "9".to_string())]);
        let http = |route: Value| doc(json!({ "name": "API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [{ "path": "/ok", "responses": [{}] }, route] }));
        let (code_, params_, field) = code(check(&http(json!({ "path": "/x", "responses": [{}, { "body": "{{vars.token}}" }] })), &params));
        assert_eq!((code_.as_str(), params_["rule"].as_str(), params_["response"].as_str(), field), ("emulator.name_unknown", "2", "2", Some(Field::new("body"))));
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "body": "{{secret.KEY}}" }] })), &params)).0, "emulator.secret_unsupported");
        assert_eq!(code(check(&http(json!({ "path": "/{{request.path}}", "responses": [{}] })), &params)).0, "emulator.params_only");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [] })), &params)).0, "emulator.response_required");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "status": 99 }] })), &params)).0, "node.range");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "headers": [["bad name", "x"]] }] })), &params)).2, Some(Field::nth("header_name", 1)));
        assert_eq!(code(check(&http(json!({ "path": "/x", "order": "random", "responses": [{ "weight": 0 }] })), &params)).0, "emulator.weights_zero");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "delay_ms": 60001 }] })), &params)).2, Some(Field::new("delay_ms")));
        // Parameters are welcome everywhere, and `request` in what is rendered per request.
        let fine = http(json!({ "path": "/items/:id", "when": [{ "on": "header", "name": "x-port", "op": "eq", "value": "{{params.port}}" }],
            "responses": [{ "headers": [["X-Id", "{{request.params.id}}"]], "body": "{{request.json.name}} {{port}} {{uuid}} {{counter}}" }] }));
        assert!(check(&fine, &params).is_ok());

        let osc = |rule: Value| doc(json!({ "name": "Device", "bind": "127.0.0.1:9100", "protocol": "osc", "rules": [rule] }));
        let reply = osc(json!({ "address": "/ping", "reply": { "address": "/pong", "args": [{ "type": "int", "value": "seven" }] } }));
        let (code_, params_, field) = code(check(&reply, &params));
        assert_eq!((code_.as_str(), params_["rule"].as_str(), field), ("emulator.arg_value", "1", Some(Field::nth("reply_arg", 1))));
        assert!(check(&osc(json!({ "address": "/ping", "reply": { "address": "/pong", "args": [{ "type": "int", "value": "{{request.args[0]}}" }] } })), &params).is_ok());
        assert_eq!(code(check(&osc(json!({ "address": "ping" })), &params)).0, "osc.pattern_slash");
        assert_eq!(code(check(&osc(json!({ "address": "/ping", "to": "nowhere" })), &params)).0, "node.target_invalid");
        let udp = doc(json!({ "name": "Echo", "bind": "127.0.0.1:7100", "protocol": "udp", "rules": [{ "mode": "hex", "pattern": "zz" }] }));
        assert_eq!(code(check(&udp, &params)).0, "hex.invalid");
        let greeting = doc(json!({ "name": "Device", "bind": "127.0.0.1:7200", "protocol": "tcp", "greeting": "{{request.from}}" }));
        assert_eq!(code(check(&greeting, &params)).0, "emulator.name_unknown", "nothing has arrived when the greeting goes out");
        assert_eq!(code(check(&doc(json!({ "name": " ", "bind": "127.0.0.1:1", "protocol": "udp" })), &params)).2, Some(Field::new("name")));
        assert_eq!(code(check(&doc(json!({ "name": "x", "bind": "127.0.0.1:0", "protocol": "udp" })), &params)).0, "node.bind_invalid");
    }

    #[test]
    fn reply_arguments_take_their_type_from_the_text() {
        assert_eq!(osc_arg(ArgType::Int, " 42 ").unwrap(), OscArg::Int(42));
        assert_eq!(osc_arg(ArgType::Float, "0.5").unwrap(), OscArg::Float(0.5));
        assert_eq!(osc_arg(ArgType::Double, "").unwrap(), OscArg::Double(0.0));
        assert_eq!(osc_arg(ArgType::Bool, "on").unwrap(), OscArg::Bool(true));
        assert_eq!(osc_arg(ArgType::Blob, "de ad").unwrap(), OscArg::Blob(vec![0xde, 0xad]));
        assert_eq!(osc_arg(ArgType::Str, " a ").unwrap(), OscArg::Str(" a ".into()));
        assert_eq!(osc_arg(ArgType::Nil, "ignored").unwrap(), OscArg::Nil);
        let error = osc_arg(ArgType::Int, "1.5").unwrap_err();
        assert_eq!((error.code.as_str(), error.params["type"].as_str()), ("emulator.arg_value", "int"));
        assert!(osc_arg(ArgType::Float, "inf").is_err() && osc_arg(ArgType::Bool, "maybe").is_err());
    }

    #[test]
    fn the_newest_exchanges_are_kept_and_events_carry_no_bodies() {
        let emulation = Emulation::new("API", "http", "127.0.0.1:8080".parse().unwrap(), 2);
        assert_eq!((emulation.hit(1), emulation.hit(1), emulation.hit(0)), (1, 2, 1));
        for index in 0..RECENT + 3 {
            let rule = if index % 2 == 0 { Some(1) } else { None };
            emulation.record(Exchange { request: format!("GET /{index}"), rule, data: json!({ "body": "x" }), ..Default::default() });
        }
        let counts = emulation.counts();
        assert_eq!((counts.total, counts.unmatched, counts.hits.clone()), ((RECENT + 3) as u64, ((RECENT + 3) / 2) as u64, vec![1, 2]));
        let kept = emulation.exchanges(0, RECENT);
        assert_eq!((kept.len(), kept[0].seq), (RECENT, 4), "the oldest three are gone");
        assert_eq!(emulation.exchanges(RECENT as u64, 10).len(), 3);
        let (fresh, dropped) = emulation.take_fresh().unwrap();
        assert_eq!((fresh.len(), dropped), (BATCH, (RECENT + 3 - BATCH) as u64));
        assert!(fresh.iter().all(|exchange| exchange.data.is_null()) && !kept[0].data.is_null());
        assert!(emulation.take_fresh().is_none(), "nothing new since");
        let long = kept_body_case();
        assert!(long["body"].as_str().unwrap().len() < KEPT_BODY + 8 && long["json"].is_null());
    }

    fn kept_body_case() -> Value {
        kept(&json!({ "body": "é".repeat(KEPT_BODY), "json": { "a": 1 } }))
    }

    fn node(id: &str, kind: Value) -> Node {
        let mut value = kind;
        value["id"] = json!(id);
        value["x"] = json!(0);
        value["y"] = json!(0);
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn a_run_shares_a_port_only_where_that_makes_sense() {
        let emulator = |protocol: &str, bind: &str| json!({ "type": "emulator", "emulator": { "name": "E", "bind": bind, "protocol": protocol } });
        let wait = |bind: &str| json!({ "type": "wait_http", "bind": bind });
        assert!(check_run_binds(&[node("a", emulator("http", "127.0.0.1:8080")), node("w", wait("127.0.0.1:8080"))]).is_ok());
        assert!(check_run_binds(&[node("a", emulator("http", "127.0.0.1:8080")), node("b", emulator("osc", "127.0.0.1:8080"))]).is_ok(), "TCP and UDP ports are apart");
        let twice = check_run_binds(&[node("a", emulator("udp", "127.0.0.1:9000")), node("b", emulator("osc", "127.0.0.1:9000"))]).unwrap_err();
        assert_eq!((twice.code.as_str(), twice.node.as_deref(), twice.params["other"].as_str()), ("emulator.bind_taken", Some("b"), "a"));
        let tcp = check_run_binds(&[node("a", emulator("tcp", "127.0.0.1:8080")), node("w", wait("127.0.0.1:8080"))]).unwrap_err();
        assert_eq!(tcp.node.as_deref(), Some("w"));
    }
}
