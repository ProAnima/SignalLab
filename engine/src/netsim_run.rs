//! The impairment relays of a run: one per *Impairment* node, opened before
//! the first step like the run's emulators and listeners (a taken port is an
//! error at that node, before any traffic), impairing with the node's profile
//! until a *Change impairment* step switches it, and closed with the run —
//! a stopped or failed run leaves nothing impaired behind. Each relay draws
//! from the run's seed, so the same seed and traffic drop the same packets.

use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::host::Host;

use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Node, NodeKind};
use super::experiment_data as data;
use super::netsim::{self, ImpairmentSummary, Relay};

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
pub fn addresses(listen: &str, target: &str, params: &BTreeMap<String, String>) -> EngineResult<(SocketAddr, SocketAddr)> {
    let fixed = |text: &str, field: &'static str| data::static_text(text, params).ok_or_else(|| EngineError::new("node.params_only").in_field(Field::new(field)));
    let (listen, target) = (fixed(listen, "listen")?, fixed(target, "target")?);
    let listen = match listen.trim().parse::<SocketAddr>() {
        Ok(address) if address.port() != 0 => address,
        _ => return Err(EngineError::new("node.bind_invalid").with("value", listen).in_field(Field::new("listen"))),
    };
    let target = match target.trim().parse::<SocketAddr>() {
        Ok(address) if address.port() != 0 => address,
        _ => return Err(EngineError::new("node.target_invalid").with("value", target).in_field(Field::new("target"))),
    };
    Ok((listen, target))
}

/// Open a run's relays, each impairing with its node's profile.
pub async fn arm_run(host: &Host, nodes: &[Node], params: &BTreeMap<String, String>, seed: u64, job: u64) -> EngineResult<RunRelays> {
    let mut armed = RunRelays::default();
    for node in nodes {
        let NodeKind::Impairment { listen, target, profile } = &node.kind else { continue };
        let at = |error: EngineError| error.at(&node.id);
        let (listen_address, target_address) = addresses(listen, target, params).map_err(at)?;
        let (downstream, upstream) = netsim::open(listen_address, &listen_address.to_string(), target_address, &target_address.to_string())
            .await
            .map_err(|error| at(error.in_field(Field::new("listen"))))?;
        let local = downstream.local_addr().unwrap_or(listen_address);
        let relay = Relay::new(local, target_address, profile.clone(), seed);
        let serving = tokio::spawn(relay.clone().serve(host.clone(), job, node.id.clone(), downstream, upstream));
        armed.serving.push(Serving(serving.abort_handle()));
        armed.relays.push((node.id.clone(), relay));
    }
    Ok(armed)
}

/// No two of a run's sockets on one UDP port: a relay's listen port is not
/// another relay's, an OSC or UDP emulator's, or a wait's.
pub fn check_run_binds(nodes: &[Node], params: &BTreeMap<String, String>) -> EngineResult<()> {
    let mut taken: HashMap<SocketAddr, &str> = HashMap::new();
    for node in nodes {
        let bind = match &node.kind {
            NodeKind::Emulator { emulator } if !emulator.kind.over_tcp() => emulator.bind.trim().parse().ok(),
            kind => kind.bind().and_then(|bind| bind.trim().parse::<SocketAddr>().ok()).filter(|address| address.port() != 0),
        };
        if let Some(bind) = bind {
            taken.entry(bind).or_insert(&node.id);
        }
    }
    let mut relays: HashMap<SocketAddr, &str> = HashMap::new();
    for node in nodes {
        let NodeKind::Impairment { listen, target, .. } = &node.kind else { continue };
        let Ok((listen, _)) = addresses(listen, target, params) else { continue };
        if let Some(other) = taken.get(&listen).or_else(|| relays.get(&listen)) {
            return Err(EngineError::new("impair.bind_taken").with("target", listen).with("other", *other).in_field(Field::new("listen")).at(&node.id));
        }
        relays.insert(listen, &node.id);
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

        let impairment = |listen: &str| json!({ "type": "impairment", "listen": listen, "target": "127.0.0.1:9000" });
        let wait = node("wait", json!({ "type": "wait_udp", "bind": "127.0.0.1:9010" }));
        let error = check_run_binds(&[wait, node("relay", impairment("127.0.0.1:9010"))], &params).unwrap_err();
        assert_eq!((error.code.as_str(), error.node.as_deref(), error.params["other"].as_str()), ("impair.bind_taken", Some("relay"), "wait"));
        let twice = check_run_binds(&[node("a", impairment("{{relay}}")), node("b", impairment("127.0.0.1:9010"))], &params).unwrap_err();
        assert_eq!(twice.node.as_deref(), Some("b"));
        assert!(check_run_binds(&[node("a", impairment("127.0.0.1:9010")), node("b", impairment("127.0.0.1:9011"))], &params).is_ok());
    }
}
