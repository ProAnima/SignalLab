//! Emulators: Signal Lab as the other side — the API, device or service the
//! system under test talks to. An emulator listens on one address and answers
//! by rules: HTTP routes with responses (in sequence, in turn, or a seeded mix
//! with faults), for OSC, UDP and TCP "on this, reply that", and an MQTT
//! broker that routes like any broker and answers what is published to it by
//! rules. It may also go down now and then (`Outage`). Replies are
//! templates read with what arrived — `{{request.params.id}}`,
//! `{{request.args[0]}}`, `{{request.json.name}}` — in the language of
//! `template.rs`, so a field means here what it means in an experiment.
//!
//! An emulator is a job of its own (the Emulators screen, `signallab
//! emulate`), or part of a run as an *Emulator* node: then it opens before the
//! first step, like a wait's socket, and closes with the run. Every exchange
//! goes to the Inspector, is counted per rule, and the newest `RECENT` are
//! kept for whoever asks what arrived.
//!
//! This module is the document. The rest of the family: `emulator_match`
//! (which requests a route or a wait takes), `emulator_rules` (a document
//! checked and compiled), `emulator_state` (what every protocol shares while
//! it serves), `emulator_http`, `emulator_net` and `emulator_mqtt` (the protocols),
//! `emulator_job` (one as a job of its own), `emulator_run` (the ones of a
//! run) and `emulator_files` (the library).

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::emulator_rules;
use super::error::EngineResult;
use super::matching::{ArgRule, CompareOp, UdpMode};
use super::signals::RawPayload;

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
pub(crate) const MAX_NAME: usize = 120;
pub(crate) const MAX_PATH: usize = 512;
/// The most OSC argument conditions or reply arguments.
pub(crate) const MAX_ARGS: usize = 16;
pub(crate) const MAX_DATAGRAM: usize = 65_507;
/// The shortest and longest stretch an outage is up or down.
pub(crate) const MIN_OUTAGE_MS: u64 = 10;
pub(crate) const MAX_OUTAGE_MS: u64 = 3_600_000;
/// How long a `timeout` fault holds a request before it closes the connection.
pub const HOLD: Duration = Duration::from_secs(120);

// ---- the document ---------------------------------------------------------------

/// One emulator: a name, where it listens, and how it answers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Emulator {
    pub name: String,
    /// `IP:port`. Loopback unless a person chose to reach the network.
    pub bind: String,
    #[serde(flatten)]
    pub kind: EmulatorKind,
    /// Down now and then — a dependency that flaps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outage: Option<Outage>,
}

/// Answering for `up_ms`, then down for `down_ms`, and so on from the start.
/// While down, HTTP meets `fault`, a TCP device and an MQTT broker drop their
/// connections and refuse new ones, and OSC and UDP devices answer nothing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Outage {
    pub up_ms: u64,
    pub down_ms: u64,
    #[serde(default)]
    pub fault: DownFault,
}

/// What an HTTP request meets while its emulator is down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownFault {
    /// 503 with `Retry-After` set to when it is back.
    #[default]
    Unavailable,
    /// The connection is closed without an answer.
    Reset,
    /// No answer: held until the client gives up (at most `HOLD`).
    Timeout,
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
    /// An MQTT 3.1.1 broker: what a client publishes goes to every client
    /// subscribed to it, as on any broker, and is answered by the first rule
    /// it matches.
    Mqtt {
        /// When not empty, a client must connect with this user name and `password`.
        #[serde(default)]
        username: String,
        #[serde(default)]
        password: String,
        /// Retained from the start, as if published with retain before anyone connected.
        #[serde(default)]
        retained: Vec<MqttRetained>,
        #[serde(default)]
        rules: Vec<MqttRule>,
    },
}

impl EmulatorKind {
    pub fn protocol(&self) -> &'static str {
        match self {
            EmulatorKind::Http { .. } => "http",
            EmulatorKind::Osc { .. } => "osc",
            EmulatorKind::Udp { .. } => "udp",
            EmulatorKind::Tcp { .. } => "tcp",
            EmulatorKind::Mqtt { .. } => "mqtt",
        }
    }

    /// Listens on a TCP port (HTTP, TCP, MQTT) rather than a UDP one (OSC, UDP).
    pub fn over_tcp(&self) -> bool {
        matches!(self, EmulatorKind::Http { .. } | EmulatorKind::Tcp { .. } | EmulatorKind::Mqtt { .. })
    }

    pub fn rules(&self) -> usize {
        match self {
            EmulatorKind::Http { routes, .. } => routes.len(),
            EmulatorKind::Osc { rules } => rules.len(),
            EmulatorKind::Udp { rules } => rules.len(),
            EmulatorKind::Tcp { rules, .. } => rules.len(),
            EmulatorKind::Mqtt { rules, .. } => rules.len(),
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
    /// An answer whose body stops halfway: well-formed HTTP, but JSON that
    /// does not parse (the content type still says it is JSON).
    Malformed,
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

/// On a message published to a topic the filter matches, with a payload that
/// matches, publish another — a device reporting what it did.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MqttRule {
    /// A topic filter (`+` one level, `#` the rest); may use parameters.
    pub topic: String,
    #[serde(default)]
    pub mode: UdpMode,
    #[serde(default)]
    pub pattern: String,
    /// `None`: take the message and publish nothing.
    #[serde(default)]
    pub reply: Option<MqttOut>,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub jitter_ms: u64,
}

/// A message the emulator publishes, to every client subscribed to it; its
/// topic and payload are templates (`{{request.levels[1]}}`, `{{request.payload}}`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MqttOut {
    pub topic: String,
    #[serde(default)]
    pub payload: String,
    #[serde(default)]
    pub qos: u8,
    #[serde(default)]
    pub retain: bool,
}

/// A retained message the broker holds from the start; parameters only.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MqttRetained {
    pub topic: String,
    #[serde(default)]
    pub payload: String,
    #[serde(default)]
    pub qos: u8,
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

/// Every problem of an emulator before it starts, naming the rule (`rule`),
/// the response (`response`) and the field it is in. `params` are the values
/// its templates may read besides `request`: a run's parameters.
pub fn check(emulator: &Emulator, params: &BTreeMap<String, String>) -> EngineResult<()> {
    emulator_rules::compile(emulator, params).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc(value: serde_json::Value) -> Emulator {
        serde_json::from_value(value).unwrap()
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
}
