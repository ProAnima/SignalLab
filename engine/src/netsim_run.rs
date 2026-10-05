//! The impairment relays of a run: one per *Impairment* node, UDP or TCP,
//! opened before the first step like the run's emulators and listeners (a
//! taken port is an error at that node, before any traffic), impairing with
//! the node's profile until a *Change impairment* step switches it, and closed
//! with the run — a stopped or failed run leaves nothing impaired behind, and
//! a TCP relay's connections close with it. Each relay draws from the run's
//! seed, so the same seed and traffic drop the same packets. A target may be
//! a host name: it is looked up when the run opens its relays.

use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::host::Host;

use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Node, NodeKind};
use super::experiment_data as data;
use super::netsim::{ImpairmentSummary, Opened, Relay, RelayProtocol};
use super::transport;

/// Aborts a relay of the run when the run lets go of it.
struct Serving(tokio::task::AbortHandle);

impl Drop for Serving {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// The run's relays, by node id, in document order for the report.
#[derive(Default)]
pub struct RunRelays {
    pub relays: Vec<(String, Arc<Relay>)>,
    serving: Vec<Serving>,
}

impl RunRelays {
    /// The relay of Impairment node `id`.
    pub fn by_node(&self) -> HashMap<String, Arc<Relay>> {
        self.relays.iter().map(|(node, relay)| (node.clone(), relay.clone())).collect()
    }

    pub fn summaries(&self) -> Vec<ImpairmentSummary> {
        summaries(&self.relays)
    }

    /// The relays' tasks; dropping them closes the relays.
    pub fn take_serving(&mut self) -> RunServing {
        RunServing { _tasks: std::mem::take(&mut self.serving) }
    }
}

/// What each relay did, phase by phase, in document order.
pub fn summaries(relays: &[(String, Arc<Relay>)]) -> Vec<ImpairmentSummary> {
    relays.iter().map(|(node, relay)| relay.summary(Some(node))).collect()
}

/// The tasks a run's relays serve with; dropping it stops them.
#[derive(Default)]
pub struct RunServing {
    _tasks: Vec<Serving>,
}

/// An Impairment's listen and target with the run's parameters filled in:
/// both are opened before the first step, so nothing else may be in them.
/// The listen address is a bind, `IP:port`; the target is `IP:port` or
/// `name:port`, looked up when the run opens the relay (`arm_run`).
pub fn addresses(listen: &str, target: &str, params: &BTreeMap<String, String>) -> EngineResult<(SocketAddr, String)> {
    let fixed = |text: &str, field: &'static str| data::static_text(text, params).ok_or_else(|| EngineError::new("node.params_only").in_field(Field::new(field)));
    let (listen, target) = (fixed(listen, "listen")?, fixed(target, "target")?);
    let listen = match listen.trim().parse::<SocketAddr>() {
        Ok(address) if address.port() != 0 => address,
        _ => return Err(EngineError::new("node.bind_invalid").with("value", listen).in_field(Field::new("listen"))),
    };
    if !target_spec(target.trim()) {
        return Err(EngineError::new("node.target_invalid").with("value", target).in_field(Field::new("target")));
    }
    Ok((listen, target.trim().to_string()))
}

/// `IP:port` or `name:port`, the port not 0 — what `transport::resolve` can look up.
fn target_spec(text: &str) -> bool {
    if let Ok(address) = text.parse::<SocketAddr>() {
        return address.port() != 0;
    }
    text.rsplit_once(':').is_some_and(|(host, port)| {
        !host.is_empty() && host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_')) && port.parse::<u16>().is_ok_and(|port| port != 0)
    })
}

/// Open a run's relays, each impairing with its node's profile. Every target
/// is looked up first: a name may lead one relay into another, which only
/// the addresses show, so loops are checked again before anything opens.
pub async fn arm_run(host: &Host, nodes: &[Node], params: &BTreeMap<String, String>, seed: u64, job: u64) -> EngineResult<RunRelays> {
    let mut planned = Vec::new();
    for node in nodes {
        let NodeKind::Impairment { listen, target, profile, protocol } = &node.kind else { continue };
        let at = |error: EngineError| error.at(&node.id);
        let (listen_address, target_text) = addresses(listen, target, params).map_err(at)?;
        let target_address = transport::resolve(&target_text).await.map_err(|error| at(error.in_field(Field::new("target"))))?;
        planned.push((node, listen_address, target_text, target_address, profile, *protocol));
    }
    check_loops(&planned.iter().map(|(node, listen, text, target, _, protocol)| (*protocol, *listen, Some(*target), text.as_str(), node.id.as_str())).collect::<Vec<_>>())?;
    let mut armed = RunRelays::default();
    for (node, listen_address, target_text, target_address, profile, protocol) in planned {
        let at = |error: EngineError| error.at(&node.id);
        let opened = Opened::open(protocol, listen_address, &listen_address.to_string(), target_address, &target_text)
            .await
            .map_err(|error| at(error.in_field(Field::new("listen"))))?;
        let local = opened.local().unwrap_or(listen_address);
        let relay = Relay::new(local, target_address, protocol, profile.clone(), seed);
        // A relay that stops relaying keeps why: its steps fail with it and the report says so.
        let serving = tokio::spawn({
            let (relay, host, id) = (relay.clone(), host.clone(), node.id.clone());
            async move {
                let error = relay.clone().run(host, job, id, opened).await;
                relay.fail(error);
            }
        });
        armed.serving.push(Serving(serving.abort_handle()));
        armed.relays.push((node.id.clone(), relay));
    }
    Ok(armed)
}

/// Two addresses one would take from the other: the same port on the same
/// IP, or on any IP when either is unspecified (`0.0.0.0` receives what
/// `127.0.0.1` would, and the OS may let both bind).
fn clash(a: SocketAddr, b: SocketAddr) -> bool {
    a.port() == b.port() && (a.ip() == b.ip() || a.ip().is_unspecified() || b.ip().is_unspecified())
}

/// No two of a run's sockets on one port of one protocol: a UDP relay's
/// listen port is not another UDP relay's, an OSC or UDP emulator's, or a
/// wait's; a TCP relay's is not another TCP relay's, an HTTP, TCP or MQTT
/// emulator's, or a *Wait for HTTP request*'s. And no relay forwards into
/// itself, directly or through other relays: its traffic would circle on
/// loopback.
pub fn check_run_binds(nodes: &[Node], params: &BTreeMap<String, String>) -> EngineResult<()> {
    let mut taken: Vec<(RelayProtocol, SocketAddr, &str)> = Vec::new();
    for node in nodes {
        let (protocol, bind) = match &node.kind {
            NodeKind::Emulator { emulator } => (if emulator.kind.over_tcp() { RelayProtocol::Tcp } else { RelayProtocol::Udp }, emulator.bind.trim().parse().ok()),
            NodeKind::WaitHttp { bind, .. } => (RelayProtocol::Tcp, bind.trim().parse().ok()),
            kind => (RelayProtocol::Udp, kind.bind().and_then(|bind| bind.trim().parse::<SocketAddr>().ok()).filter(|address| address.port() != 0)),
        };
        if let Some(bind) = bind {
            taken.push((protocol, bind, &node.id));
        }
    }
    let mut relays: Vec<(RelayProtocol, SocketAddr, String, &str)> = Vec::new();
    for node in nodes {
        let NodeKind::Impairment { listen, target, protocol, .. } = &node.kind else { continue };
        let Ok((listen, target)) = addresses(listen, target, params) else { continue };
        let other = taken
            .iter()
            .map(|(protocol, bind, id)| (*protocol, *bind, *id))
            .chain(relays.iter().map(|(protocol, bind, _, id)| (*protocol, *bind, *id)))
            .find(|(other, bind, _)| other == protocol && clash(*bind, listen));
        if let Some((_, _, other)) = other {
            return Err(EngineError::new("impair.bind_taken").with("target", listen).with("other", other).in_field(Field::new("listen")).at(&node.id));
        }
        relays.push((*protocol, listen, target, &node.id));
    }
    // A target that is a name is followed once it is looked up (`arm_run`).
    check_loops(&relays.iter().map(|(protocol, listen, target, id)| (*protocol, *listen, target.parse::<SocketAddr>().ok(), target.as_str(), *id)).collect::<Vec<_>>())
}

/// Follow each relay's target through the relays of its protocol it reaches;
/// back at the start is a loop. A relay is (protocol, listen, target if known,
/// the target as written, node id).
fn check_loops(relays: &[(RelayProtocol, SocketAddr, Option<SocketAddr>, &str, &str)]) -> EngineResult<()> {
    for (start, (protocol, _, target, shown, id)) in relays.iter().enumerate() {
        let Some(mut next) = *target else { continue };
        for _ in 0..relays.len() {
            let Some(hop) = relays.iter().position(|(other, listen, ..)| other == protocol && clash(*listen, next)) else { break };
            if hop == start {
                return Err(EngineError::new("impair.loop").with("target", shown).in_field(Field::new("target")).at(id));
            }
            match relays[hop].2 {
                Some(after) => next = after,
                None => break,
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn node(id: &str, kind: serde_json::Value) -> Node {
        let mut value = kind;
        value["id"] = json!(id);
        value["x"] = json!(0);
        value["y"] = json!(0);
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn addresses_take_parameters_and_ports_are_not_shared() {
        let params = BTreeMap::from([("relay".to_string(), "127.0.0.1:9010".to_string())]);
        assert_eq!(addresses("{{relay}}", "127.0.0.1:9000", &params).unwrap().0.port(), 9010);
        assert_eq!(addresses("{{vars.x}}", "127.0.0.1:9000", &params).unwrap_err().code, "node.params_only");
        assert_eq!(addresses("127.0.0.1:0", "127.0.0.1:9000", &params).unwrap_err().code, "node.bind_invalid");
        assert_eq!(addresses("127.0.0.1:9010", "nowhere", &params).unwrap_err().field.clone().map(|field| field.key), Some("target".into()));
        // A target may be a name (looked up when the run opens it); the listen address is a bind.
        assert_eq!(addresses("127.0.0.1:9010", "device.local:9000", &params).unwrap().1, "device.local:9000");
        for bad in ["device.local", "device.local:0", "dev ice:9000", ":9000", "device:99999", "127.0.0.1:0"] {
            let error = addresses("127.0.0.1:9010", bad, &params).unwrap_err();
            assert_eq!((error.code.as_str(), error.field.clone().map(|field| field.key)), ("node.target_invalid", Some("target".into())), "{bad}");
        }
        assert_eq!(addresses("localhost:9010", "127.0.0.1:9000", &params).unwrap_err().code, "node.bind_invalid");

        let impairment = |listen: &str| json!({ "type": "impairment", "listen": listen, "target": "127.0.0.1:9000" });
        let wait = node("wait", json!({ "type": "wait_udp", "bind": "127.0.0.1:9010" }));
        let error = check_run_binds(&[wait, node("relay", impairment("127.0.0.1:9010"))], &params).unwrap_err();
        assert_eq!((error.code.as_str(), error.node.as_deref(), error.params["other"].as_str()), ("impair.bind_taken", Some("relay"), "wait"));
        let twice = check_run_binds(&[node("a", impairment("{{relay}}")), node("b", impairment("127.0.0.1:9010"))], &params).unwrap_err();
        assert_eq!(twice.node.as_deref(), Some("b"));
        assert!(check_run_binds(&[node("a", impairment("127.0.0.1:9010")), node("b", impairment("127.0.0.1:9011"))], &params).is_ok());

        // 0.0.0.0 takes what 127.0.0.1 would on the same port, either way round.
        let wide = check_run_binds(&[node("wait", json!({ "type": "wait_udp", "bind": "127.0.0.1:9010" })), node("relay", impairment("0.0.0.0:9010"))], &params).unwrap_err();
        assert_eq!((wide.code.as_str(), wide.params["other"].as_str()), ("impair.bind_taken", "wait"));
        let narrow = check_run_binds(&[node("wait", json!({ "type": "wait_udp", "bind": "0.0.0.0:9010" })), node("relay", impairment("127.0.0.1:9010"))], &params).unwrap_err();
        assert_eq!(narrow.code, "impair.bind_taken");
    }

    #[test]
    fn a_relay_never_forwards_into_itself() {
        let params = BTreeMap::new();
        let relay = |id: &str, listen: &str, target: &str| node(id, json!({ "type": "impairment", "listen": listen, "target": target }));
        let own = check_run_binds(&[relay("a", "127.0.0.1:9010", "127.0.0.1:9010")], &params).unwrap_err();
        assert_eq!((own.code.as_str(), own.node.as_deref(), own.field.clone().map(|field| field.key)), ("impair.loop", Some("a"), Some("target".into())));
        let round = check_run_binds(&[relay("a", "127.0.0.1:9010", "127.0.0.1:9011"), relay("b", "127.0.0.1:9011", "0.0.0.0:9010")], &params).unwrap_err();
        assert_eq!(round.code, "impair.loop", "a → b → a");
        // Two relays in a row in front of a device are fine.
        assert!(check_run_binds(&[relay("a", "127.0.0.1:9010", "127.0.0.1:9011"), relay("b", "127.0.0.1:9011", "127.0.0.1:9000")], &params).is_ok());
    }

    /// A target by name is looked up when the run opens its relays: a name
    /// that leads a relay back into itself is a loop then, one that does not
    /// resolve is the network's answer at that node.
    #[tokio::test]
    async fn a_target_by_name_is_looked_up_when_the_run_opens_it() {
        let host = Host::new(crate::host::Recorder::new(), crate::inspect::Capture::new());
        let params = BTreeMap::new();
        let port = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let target = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let target_port = target.local_addr().unwrap().port();
        let relay = |listen: String, target: String| node("relay", json!({ "type": "impairment", "listen": listen, "target": target }));
        let armed = arm_run(&host, &[relay(format!("127.0.0.1:{port}"), format!("localhost:{target_port}"))], &params, 1, 0).await.unwrap();
        let (_, opened) = &armed.relays[0];
        assert!(opened.target.ip().is_loopback() && opened.target.port() == target_port, "localhost is this machine: {}", opened.target);
        drop(armed);

        // Validation cannot follow a name; opening the run can.
        let own = [relay(format!("127.0.0.1:{port}"), format!("localhost:{port}"))];
        assert!(check_run_binds(&own, &params).is_ok());
        let error = arm_run(&host, &own, &params, 1, 0).await.map(|_| ()).unwrap_err();
        assert_eq!((error.code.as_str(), error.node.as_deref(), error.params["target"].as_str()), ("impair.loop", Some("relay"), format!("localhost:{port}").as_str()));

        // `.invalid` never resolves (RFC 6761).
        let error = arm_run(&host, &[relay(format!("127.0.0.1:{port}"), "no-such-host.invalid:9000".into())], &params, 1, 0).await.map(|_| ()).unwrap_err();
        assert_eq!((error.code.as_str(), error.node.as_deref(), error.field.clone().map(|field| field.key)), ("transport.dns", Some("relay"), Some("target".into())));
    }

    #[test]
    fn a_tcp_relay_shares_no_tcp_port_and_may_share_a_udp_one() {
        let params = BTreeMap::new();
        let tcp = |id: &str, listen: &str, target: &str| node(id, json!({ "type": "impairment", "listen": listen, "target": target, "protocol": "tcp" }));
        let udp = |id: &str, listen: &str| node(id, json!({ "type": "impairment", "listen": listen, "target": "127.0.0.1:9000" }));
        assert!(check_run_binds(&[tcp("a", "127.0.0.1:9010", "127.0.0.1:9000"), udp("b", "127.0.0.1:9010")], &params).is_ok(), "one port, two protocols");
        let api = node("api", json!({ "type": "emulator", "emulator": { "name": "API", "bind": "127.0.0.1:18080", "protocol": "http", "routes": [] } }));
        let taken = check_run_binds(&[api, tcp("relay", "127.0.0.1:18080", "127.0.0.1:9000")], &params).unwrap_err();
        assert_eq!((taken.code.as_str(), taken.params["other"].as_str()), ("impair.bind_taken", "api"));
        let wait = node("hook", json!({ "type": "wait_http", "bind": "127.0.0.1:18081" }));
        assert_eq!(check_run_binds(&[wait, tcp("relay", "0.0.0.0:18081", "127.0.0.1:9000")], &params).unwrap_err().params["other"], "hook");
        let round = check_run_binds(&[tcp("a", "127.0.0.1:9010", "127.0.0.1:9011"), tcp("b", "127.0.0.1:9011", "127.0.0.1:9010")], &params).unwrap_err();
        assert_eq!(round.code, "impair.loop");
        assert!(check_run_binds(&[tcp("a", "127.0.0.1:9010", "127.0.0.1:9011"), udp("b", "127.0.0.1:9011")], &params).is_ok(), "a UDP relay is not on a TCP relay's way");
    }
}
