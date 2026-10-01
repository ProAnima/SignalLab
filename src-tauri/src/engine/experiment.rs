//! The experiment document: nodes, edges, outputs and versions. Independent of
//! the editor and of execution — validation lives in `experiment_validate`,
//! execution in `experiment_run`.

use serde::{Deserialize, Serialize};

use super::experiment_data::{CompareOp, ExtractFrom, Param, Profile};
use super::http::HttpRequest;
use super::matching::{ArgRule, UdpMode};
use super::osc_codec::OscArg;

pub const VERSION: u32 = 3;
/// Read and migrated on load (see `experiment_files::parse`): 1 had no
/// parameters, 2 had no profiles; serde defaults supply what is missing.
pub const LEGACY_VERSIONS: &[u32] = &[1, 2];
pub const MAX_NODES: usize = 64;
/// Every output name a document may use.
pub const PORTS: &[&str] = &["next", "yes", "no", "branch1", "branch2", "matched", "timeout"];

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
    #[serde(flatten)]
    pub kind: NodeKind,
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
    },
    Udp {
        target: String,
        text: String,
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
            NodeKind::WaitOsc { .. } | NodeKind::WaitUdp { .. } => &["matched"],
            _ => &["next"],
        };
        let optional: &'static [&'static str] = match self {
            // Without a Timeout wire, a timeout fails the step.
            NodeKind::WaitOsc { .. } | NodeKind::WaitUdp { .. } => &["timeout"],
            _ => &[],
        };
        Outputs { required, optional }
    }

    /// Sends traffic; *Send now* applies, and a later wait counts replies from here.
    pub fn is_action(&self) -> bool {
        matches!(self, NodeKind::Tcp { .. } | NodeKind::Http { .. } | NodeKind::Mqtt { .. } | NodeKind::Osc { .. } | NodeKind::Udp { .. })
    }

    pub fn is_wait(&self) -> bool {
        matches!(self, NodeKind::WaitOsc { .. } | NodeKind::WaitUdp { .. })
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

    /// The address a wait listens on.
    pub fn bind(&self) -> Option<&str> {
        match self {
            NodeKind::WaitOsc { bind, .. } | NodeKind::WaitUdp { bind, .. } => Some(bind),
            _ => None,
        }
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

pub fn starter() -> Experiment {
    serde_json::from_str(include_str!(
        "../../../experiments/templates/http-check.json"
    ))
    .expect("bundled HTTP template must be a valid experiment")
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
