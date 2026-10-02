//! The emulators of a run and the HTTP listeners of its waits: opened before
//! the first step (a taken port is an error at the node that wanted it,
//! before any traffic), closed with the run.

use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::host::Host;

use super::emulator::{EmulatorKind, Fault, Response};
use super::emulator_http;
use super::emulator_job::Socket;
use super::emulator_net;
use super::emulator_rules::{compile, Compiled, Rules};
use super::emulator_state::{Context, Emulation, EmulatorSummary};
use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Node, NodeKind};
use super::experiment_fields::check_bind;
use super::listen::{Inbox, Tap};

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
                    EmulatorKind::Tcp { .. } | EmulatorKind::Mqtt { .. } => {
                        let socket = Socket::open(bind, true).await.map_err(at)?;
                        let context = Context::new(host.clone(), compiled, emulation, seed, Some(job), None);
                        armed.serving.push(Serving(tokio::spawn(async move { socket.serve(context).await; }).abort_handle()));
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
            let compiled = Compiled { name: String::new(), bind, params: params.clone(), rules: Rules::Http { routes: Vec::new(), fallback: Some(accepted()) }, outage: None };
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
    use serde_json::{json, Value};

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
