//! The experiment document: nodes, edges, outputs and versions. Independent of
//! the editor and of execution — validation lives in `experiment_validate`,
//! execution in `experiment_run`.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::emulator::{Condition, Emulator};
use super::experiment_data::{CompareOp, ExtractFrom, Param, Profile};
use super::http::HttpRequest;
use super::matching::{ArgRule, UdpMode};
use super::osc_codec::OscArg;
use super::template::Rng;

pub const VERSION: u32 = 6;
/// Read and migrated on load (see `experiment_files::parse`): 1 had no
/// parameters, 2 had no profiles, 3 had no retries or expected replies, 4 had
/// no repeats or loops, 5 had no emulators or HTTP waits; serde defaults
/// supply what is missing. Each new version exists so that an older Signal
/// Lab refuses a newer file instead of silently dropping those settings.
pub const LEGACY_VERSIONS: &[u32] = &[1, 2, 3, 4, 5];
/// A run that takes longer than this is stopped.
pub const RUN_LIMIT: Duration = Duration::from_secs(300);
pub const MAX_NODES: usize = 64;
/// Every output name a document may use.
pub const PORTS: &[&str] = &["next", "yes", "no", "branch1", "branch2", "matched", "timeout", "body", "done", "limit"];
/// The most iterations one Loop runs.
pub const MAX_LOOP: u32 = 1000;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Experiment {
    pub version: u32,
    pub name: String,
    /// Default values; a profile overrides some of them.
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    /// The active profile; `None` runs with the defaults.
    #[serde(default)]
    pub profile: Option<String>,
    /// `None`: a fresh seed for every run, recorded in its report.
    #[serde(default)]
    pub seed: Option<u64>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub x: f64,
    pub y: f64,
    /// Try again when the step fails: actions and waits only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<Retry>,
    /// Send more than once: actions only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Repeat>,
    #[serde(flatten)]
    pub kind: NodeKind,
}

/// Retrying a failed action or wait: how often, and how long to pause.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Retry {
    /// Attempts in all, the first one included.
    pub attempts: u32,
    /// The pause before the second attempt.
    pub delay_ms: u64,
    #[serde(default)]
    pub backoff: Backoff,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backoff {
    /// The same pause every time.
    #[default]
    Fixed,
    /// The pause doubles after every failed attempt.
    Exponential,
}

/// The longest single pause between attempts.
pub const MAX_RETRY_PAUSE: Duration = Duration::from_secs(60);

impl Retry {
    /// The pause before `attempt` (2 for the first retry).
    pub fn pause_before(&self, attempt: u32) -> Duration {
        let base = Duration::from_millis(self.delay_ms);
        let pause = match self.backoff {
            Backoff::Fixed => base,
            Backoff::Exponential => base.saturating_mul(2u32.saturating_pow(attempt.saturating_sub(2))),
        };
        pause.min(MAX_RETRY_PAUSE)
    }
}

/// Sending an action more than once — a heartbeat, a poll, a steady stream —
/// without a loop in the graph. Each send is made like a single one (its
/// templates read afresh, retried by its Retry), and the step passes when
/// every send did.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Repeat {
    #[serde(default)]
    pub until: RepeatUntil,
    /// Sends in all, the first one included (`until: count`).
    #[serde(default = "default_repeat_count")]
    pub count: u32,
    /// How long to keep sending, from the first send (`until: duration`).
    #[serde(default = "default_repeat_duration")]
    pub duration_ms: u64,
    /// The pause between two sends.
    pub interval_ms: u64,
    /// Up to this much longer per pause, drawn from the run's seed.
    #[serde(default)]
    pub jitter_ms: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepeatUntil {
    /// A number of sends.
    #[default]
    Count,
    /// As many as fit in a time.
    Duration,
}

/// The most sends one repeating step makes, in either mode.
pub const MAX_REPEATS: u32 = 10_000;
/// Mixed into the seed, so jitter never draws the numbers a node's templates do.
const JITTER_STREAM: u64 = 0x6a69_7474_6572_0001;

impl Repeat {
    /// Whether another send is due after `sent` sends, `elapsed` since the
    /// first one began, with `pause` still to wait before it.
    pub fn more(&self, sent: u32, elapsed: Duration, pause: Duration) -> bool {
        match self.until {
            RepeatUntil::Count => sent < self.count.min(MAX_REPEATS),
            RepeatUntil::Duration => sent < MAX_REPEATS && elapsed + pause < Duration::from_millis(self.duration_ms),
        }
    }

    /// The pause before send `n` (2 for the second): the interval and its
    /// jitter, the same for the same seed every time.
    pub fn pause_before(&self, seed: u64, node_id: &str, n: u32) -> Duration {
        let jitter = if self.jitter_ms == 0 { 0 } else { Rng::for_node(seed ^ JITTER_STREAM, node_id, n as u64).below_u64(self.jitter_ms + 1) };
        Duration::from_millis(self.interval_ms + jitter)
    }
}

fn default_repeat_count() -> u32 {
    10
}

fn default_repeat_duration() -> u64 {
    10_000
}

/// An OSC reply an OSC message expects: the same matching as *Wait for OSC*.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OscReply {
    pub bind: String,
    pub address: String,
    #[serde(default)]
    pub args: Vec<ArgRule>,
    #[serde(default = "default_wait_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_reply_variable")]
    pub variable: String,
}

/// A UDP reply a datagram expects: the same matching as *Wait for UDP*.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UdpReply {
    pub bind: String,
    #[serde(default)]
    pub mode: UdpMode,
    #[serde(default)]
    pub pattern: String,
    #[serde(default = "default_wait_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_reply_variable")]
    pub variable: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeKind {
    Start,
    End,
    Fork,
    Join,
    Log {
        message: String,
    },
    Tcp {
        host: String,
        port: u16,
        payload: String,
        #[serde(default = "default_tcp_timeout")]
        timeout_ms: u64,
    },
    Delay {
        ms: u64,
    },
    Http {
        request: HttpRequest,
    },
    AssertStatus {
        status: u16,
    },
    AssertBody {
        contains: String,
    },
    AssertHeader {
        name: String,
        contains: String,
    },
    AssertLatency {
        max_ms: u64,
    },
    Mqtt {
        host: String,
        port: u16,
        topic: String,
        payload: String,
        qos: u8,
        retain: bool,
    },
    BranchStatus {
        status: u16,
    },
    Osc {
        target: String,
        address: String,
        #[serde(default)]
        args: Vec<OscArg>,
        /// Send from the socket on `reply.bind` and wait there for the answer.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reply: Option<OscReply>,
    },
    Udp {
        target: String,
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reply: Option<UdpReply>,
    },
    Extract {
        variable: String,
        from: ExtractFrom,
        #[serde(default)]
        expr: String,
    },
    AssertValue {
        value: String,
        op: CompareOp,
        #[serde(default)]
        expected: String,
    },
    BranchValue {
        value: String,
        op: CompareOp,
        #[serde(default)]
        expected: String,
    },
    /// Wait for an OSC message on `bind` whose address matches the pattern.
    WaitOsc {
        bind: String,
        address: String,
        #[serde(default)]
        args: Vec<ArgRule>,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
        #[serde(default = "default_reply_variable")]
        variable: String,
    },
    /// Wait for an MQTT message published to `topic` (a filter: `+`, `#`) at a
    /// broker, whose payload matches. Subscribed when the run starts.
    WaitMqtt {
        host: String,
        port: u16,
        topic: String,
        #[serde(default)]
        mode: UdpMode,
        #[serde(default)]
        pattern: String,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
        #[serde(default = "default_reply_variable")]
        variable: String,
    },
    /// Run the steps on Body — which lead back here — again and again: at
    /// most `max` times, and only until `until` holds when there is one. The
    /// condition is checked after each iteration, so the body always runs at
    /// least once and can set what it tests.
    Loop {
        max: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        until: Option<Until>,
    },
    /// Wait for a UDP datagram on `bind` whose payload matches.
    WaitUdp {
        bind: String,
        #[serde(default)]
        mode: UdpMode,
        #[serde(default)]
        pattern: String,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
        #[serde(default = "default_reply_variable")]
        variable: String,
    },
    /// An emulator serving for the whole run: opened before the first step,
    /// closed with the run. In the flow it passes at once.
    Emulator {
        emulator: Emulator,
    },
    /// Wait for an HTTP request on `bind` — to the run's emulator there, or to
    /// a listener of the run's own that answers 204 — whose method, path and
    /// conditions match.
    WaitHttp {
        bind: String,
        #[serde(default = "default_method")]
        method: String,
        #[serde(default = "default_path")]
        path: String,
        #[serde(default)]
        when: Vec<Condition>,
        #[serde(default = "default_wait_timeout")]
        timeout_ms: u64,
        #[serde(default = "default_request_variable")]
        variable: String,
    },
}

/// A Loop's exit condition: the comparison of *Check value*.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Until {
    pub value: String,
    pub op: CompareOp,
    #[serde(default)]
    pub expected: String,
}

/// The outputs of a node: the ones a runnable graph must connect, and the
/// ones it may.
pub struct Outputs {
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
}

impl NodeKind {
    pub fn outputs(&self) -> Outputs {
        let required: &'static [&'static str] = match self {
            NodeKind::End => &[],
            NodeKind::BranchStatus { .. } | NodeKind::BranchValue { .. } => &["yes", "no"],
            NodeKind::Fork => &["branch1", "branch2"],
            kind if kind.is_wait() => &["matched"],
            NodeKind::Loop { .. } => &["body", "done"],
            _ => &["next"],
        };
        let optional: &'static [&'static str] = match self {
            // Without a Timeout wire, a timeout fails the step.
            kind if kind.is_wait() => &["timeout"],
            // Without a Limit wire, running out of iterations before the exit condition fails the step.
            NodeKind::Loop { .. } => &["limit"],
            _ => &[],
        };
        Outputs { required, optional }
    }

    /// Sends traffic; *Send now* applies, and a later wait counts replies from here.
    pub fn is_action(&self) -> bool {
        matches!(self, NodeKind::Tcp { .. } | NodeKind::Http { .. } | NodeKind::Mqtt { .. } | NodeKind::Osc { .. } | NodeKind::Udp { .. })
    }

    pub fn is_wait(&self) -> bool {
        matches!(self, NodeKind::WaitOsc { .. } | NodeKind::WaitUdp { .. } | NodeKind::WaitMqtt { .. } | NodeKind::WaitHttp { .. })
    }

    /// The broker and topic filter a *Wait for MQTT* subscribes to.
    pub fn subscription(&self) -> Option<(&str, u16, &str)> {
        match self {
            NodeKind::WaitMqtt { host, port, topic, .. } => Some((host, *port, topic)),
            _ => None,
        }
    }

    /// Reads the latest HTTP response, so an earlier request is required.
    pub fn reads_response(&self) -> bool {
        matches!(
            self,
            NodeKind::AssertStatus { .. }
                | NodeKind::BranchStatus { .. }
                | NodeKind::AssertBody { .. }
                | NodeKind::AssertHeader { .. }
                | NodeKind::AssertLatency { .. }
                | NodeKind::Extract { .. }
        )
    }

    /// The UDP address a wait — or an action expecting a reply — listens on.
    /// (*Wait for HTTP request* listens on TCP: `emulator::arm_run` opens it.)
    pub fn bind(&self) -> Option<&str> {
        match self {
            NodeKind::WaitOsc { bind, .. } | NodeKind::WaitUdp { bind, .. } => Some(bind),
            NodeKind::Osc { reply: Some(reply), .. } => Some(&reply.bind),
            NodeKind::Udp { reply: Some(reply), .. } => Some(&reply.bind),
            _ => None,
        }
    }

    /// Sends and then waits for an answer in the same step.
    pub fn expects_reply(&self) -> bool {
        matches!(self, NodeKind::Osc { reply: Some(_), .. } | NodeKind::Udp { reply: Some(_), .. })
    }

    /// Can be retried: it sends or listens, so a second attempt may succeed.
    pub fn retries(&self) -> bool {
        self.is_action() || self.is_wait()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    #[serde(default = "default_port")]
    pub port: String,
}

fn default_port() -> String {
    "next".into()
}

fn default_tcp_timeout() -> u64 {
    4000
}

fn default_wait_timeout() -> u64 {
    2000
}

fn default_reply_variable() -> String {
    "reply".into()
}

fn default_request_variable() -> String {
    "request".into()
}

fn default_method() -> String {
    "ANY".into()
}

fn default_path() -> String {
    "/*".into()
}

pub fn starter() -> Experiment {
    serde_json::from_str(include_str!(
        "../../experiments/templates/http-check.json"
    ))
    .expect("bundled HTTP template must be a valid experiment")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_counts_or_times_its_sends_and_jitters_the_same_way_for_the_same_seed() {
        let count = Repeat { until: RepeatUntil::Count, count: 3, duration_ms: 0, interval_ms: 100, jitter_ms: 0 };
        let pause = Duration::from_millis(100);
        assert_eq!([1, 2, 3].map(|sent| count.more(sent, Duration::ZERO, pause)), [true, true, false]);
        let timed = Repeat { until: RepeatUntil::Duration, count: 0, duration_ms: 1000, ..count.clone() };
        assert!(timed.more(5, Duration::from_millis(850), pause), "the next send at 950 ms is in time");
        assert!(!timed.more(5, Duration::from_millis(900), pause), "one at 1000 ms is not");
        assert!(!timed.more(MAX_REPEATS, Duration::ZERO, pause), "never more than MAX_REPEATS");

        assert_eq!(count.pause_before(1, "beat", 2), pause, "no jitter, the interval exactly");
        let jittery = Repeat { jitter_ms: 50, ..count };
        let pauses: Vec<Duration> = (2..40).map(|n| jittery.pause_before(42, "beat", n)).collect();
        assert!(pauses.iter().all(|pause| (100..=150).contains(&pause.as_millis())), "{pauses:?}");
        assert!(pauses.windows(2).any(|pair| pair[0] != pair[1]), "the jitter varies");
        assert_eq!(pauses, (2..40).map(|n| jittery.pause_before(42, "beat", n)).collect::<Vec<_>>(), "the same seed, the same pauses");
        assert_ne!(pauses, (2..40).map(|n| jittery.pause_before(43, "beat", n)).collect::<Vec<_>>(), "another seed, others");

        // Absent in a document: no repeat, and nothing written back; partial: the defaults.
        let node: Node = serde_json::from_value(serde_json::json!({ "id": "a", "x": 0, "y": 0, "type": "udp", "target": "127.0.0.1:9", "text": "x" })).unwrap();
        assert!(node.repeat.is_none() && serde_json::to_value(&node).unwrap().get("repeat").is_none());
        let node: Node = serde_json::from_value(serde_json::json!({ "id": "a", "x": 0, "y": 0, "type": "udp", "target": "127.0.0.1:9", "text": "x", "repeat": { "interval_ms": 250 } })).unwrap();
        assert_eq!(node.repeat, Some(Repeat { until: RepeatUntil::Count, count: 10, duration_ms: 10_000, interval_ms: 250, jitter_ms: 0 }));
    }

    #[test]
    fn wait_nodes_take_defaults_and_round_trip() {
        let kind: NodeKind = serde_json::from_value(serde_json::json!({
            "type": "wait_osc", "bind": "127.0.0.1:9001", "address": "/pong"
        }))
        .unwrap();
        let NodeKind::WaitOsc { timeout_ms, variable, args, .. } = &kind else { panic!() };
        assert_eq!((*timeout_ms, variable.as_str(), args.len()), (2000, "reply", 0));
        assert_eq!(kind.outputs().required, ["matched"]);
        assert_eq!(kind.outputs().optional, ["timeout"]);
        let udp: NodeKind = serde_json::from_value(serde_json::json!({ "type": "wait_udp", "bind": "0.0.0.0:7000" })).unwrap();
        assert_eq!(serde_json::to_value(&udp).unwrap()["mode"], "any");
        assert!(udp.is_wait() && !udp.is_action() && udp.bind() == Some("0.0.0.0:7000"));
        for port in kind.outputs().required.iter().chain(kind.outputs().optional) {
            assert!(PORTS.contains(port));
        }
    }

    #[test]
    fn retries_pause_fixed_or_doubling_and_never_longer_than_a_minute() {
        let fixed = Retry { attempts: 4, delay_ms: 200, backoff: Backoff::Fixed };
        assert_eq!([2, 3, 4].map(|attempt| fixed.pause_before(attempt).as_millis()), [200, 200, 200]);
        let doubling = Retry { backoff: Backoff::Exponential, ..fixed.clone() };
        assert_eq!([2, 3, 4].map(|attempt| doubling.pause_before(attempt).as_millis()), [200, 400, 800]);
        let long = Retry { attempts: 10, delay_ms: 40_000, backoff: Backoff::Exponential };
        assert_eq!(long.pause_before(10), MAX_RETRY_PAUSE);
        // Absent in a document: no retry, and nothing written back.
        let node: Node = serde_json::from_value(serde_json::json!({ "id": "a", "x": 0, "y": 0, "type": "delay", "ms": 1 })).unwrap();
        assert!(node.retry.is_none() && serde_json::to_value(&node).unwrap().get("retry").is_none());
        let node: Node = serde_json::from_value(serde_json::json!({ "id": "a", "x": 0, "y": 0, "type": "udp", "target": "127.0.0.1:9", "text": "x", "retry": { "attempts": 3, "delay_ms": 100 } })).unwrap();
        assert_eq!(node.retry, Some(Retry { attempts: 3, delay_ms: 100, backoff: Backoff::Fixed }));
    }

    #[test]
    fn an_expected_reply_listens_like_a_wait_and_stays_out_of_the_file_when_absent() {
        let plain: NodeKind = serde_json::from_value(serde_json::json!({ "type": "osc", "target": "127.0.0.1:9000", "address": "/ping" })).unwrap();
        assert!(!plain.expects_reply() && plain.bind().is_none());
        assert!(serde_json::to_value(&plain).unwrap().get("reply").is_none());
        let asking: NodeKind = serde_json::from_value(serde_json::json!({
            "type": "osc", "target": "127.0.0.1:9000", "address": "/ping", "reply": { "bind": "0.0.0.0:9001", "address": "/pong" }
        }))
        .unwrap();
        let NodeKind::Osc { reply: Some(reply), .. } = &asking else { panic!() };
        assert_eq!((reply.timeout_ms, reply.variable.as_str()), (2000, "reply"));
        assert!(asking.expects_reply() && asking.is_action() && asking.retries() && asking.bind() == Some("0.0.0.0:9001"));
        assert_eq!(asking.outputs().required, ["next"], "a missing reply fails the step; Retry or a Wait node handle it");
    }
}
