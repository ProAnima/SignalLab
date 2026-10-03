//! Whether a document can be saved (`validate_document`) and whether it can
//! run (`validate_with`, `validate_run`): its structure, its graph, its loops,
//! its names and values — and every node's fields (`experiment_fields`).
//! Every problem is an `EngineError` naming the node and field it is about,
//! so the editor can point at it.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;

use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Edge, Experiment, NodeKind, MAX_NODES, PORTS, VERSION};
use super::experiment_data as data;
use super::experiment_fields::{check_node, check_repeat, check_retry};

/// Structural checks for a draft. Incomplete graphs must remain saveable while
/// the editor is open; execution uses the stricter `validate_with`.
pub fn validate_document(doc: &Experiment) -> EngineResult<()> {
    if doc.version != VERSION {
        return Err(EngineError::new("doc.version_unsupported").with("version", doc.version).with("supported", VERSION));
    }
    data::check_sizes(&doc.params, &doc.profiles, doc.seed)?;
    if doc.nodes.is_empty() || doc.nodes.len() > MAX_NODES {
        return Err(EngineError::new("doc.node_count").with("max", MAX_NODES));
    }
    let mut ids = HashSet::new();
    for node in &doc.nodes {
        if node.id.trim().is_empty() {
            return Err(EngineError::new("doc.node_id_empty"));
        }
        if !ids.insert(node.id.as_str()) {
            return Err(EngineError::new("doc.node_id_duplicate").with("id", &node.id).at(&node.id));
        }
        if !node.x.is_finite() || !node.y.is_finite() || node.x < 0.0 || node.y < 0.0 {
            return Err(EngineError::new("doc.position_invalid").at(&node.id));
        }
    }
    let count = |wanted: fn(&NodeKind) -> bool| doc.nodes.iter().filter(|node| wanted(&node.kind)).count();
    if count(|kind| matches!(kind, NodeKind::Start)) != 1 || count(|kind| matches!(kind, NodeKind::End)) != 1 {
        return Err(EngineError::new("doc.start_end_count"));
    }
    // One output may feed several nodes (they run in parallel), but the same
    // wire twice is a mistake in the file.
    let mut wires = HashSet::new();
    for edge in &doc.edges {
        if !ids.contains(edge.from.as_str()) || !ids.contains(edge.to.as_str()) || edge.from == edge.to {
            let error = EngineError::new("doc.connection_invalid").with("from", &edge.from).with("to", &edge.to);
            return Err(if ids.contains(edge.from.as_str()) { error.at(&edge.from) } else { error });
        }
        if !PORTS.contains(&edge.port.as_str()) {
            return Err(EngineError::new("doc.port_invalid").with("port", &edge.port).at(&edge.from));
        }
        if !wires.insert((edge.from.as_str(), edge.port.as_str(), edge.to.as_str())) {
            return Err(EngineError::new("doc.connection_duplicate").with("from", &edge.from).with("to", &edge.to).at(&edge.from));
        }
    }
    Ok(())
}

/// The loops of a document: each Loop's body — the nodes reached from its
/// Body output that lead back to it — and the wires that come back. Those
/// wires are the only cycles a runnable document may have; everything else
/// sees the graph without them.
#[derive(Default)]
pub struct LoopShape<'a> {
    /// `(from, port, to)` of every wire from a body back to its Loop.
    pub back_edges: HashSet<(&'a str, &'a str, &'a str)>,
    pub bodies: HashMap<&'a str, HashSet<&'a str>>,
}

impl<'a> LoopShape<'a> {
    pub fn of(doc: &'a Experiment) -> Self {
        let mut shape = LoopShape::default();
        for node in doc.nodes.iter().filter(|node| matches!(node.kind, NodeKind::Loop { .. })) {
            let id = node.id.as_str();
            // Forward from Body, stopping at the loop; backward from the loop. The body is both.
            let mut forward = HashSet::new();
            let mut stack: Vec<&str> = doc.edges.iter().filter(|edge| edge.from == id && edge.port == "body").map(|edge| edge.to.as_str()).collect();
            while let Some(current) = stack.pop() {
                if current != id && forward.insert(current) {
                    stack.extend(doc.edges.iter().filter(|edge| edge.from == current).map(|edge| edge.to.as_str()));
                }
            }
            let mut back = HashSet::new();
            let mut stack: Vec<&str> = doc.edges.iter().filter(|edge| edge.to == id).map(|edge| edge.from.as_str()).collect();
            while let Some(current) = stack.pop() {
                if current != id && back.insert(current) {
                    stack.extend(doc.edges.iter().filter(|edge| edge.to == current).map(|edge| edge.from.as_str()));
                }
            }
            let body: HashSet<&str> = forward.intersection(&back).copied().collect();
            for edge in doc.edges.iter().filter(|edge| edge.to == id && body.contains(edge.from.as_str())) {
                shape.back_edges.insert((edge.from.as_str(), edge.port.as_str(), id));
            }
            shape.bodies.insert(id, body);
        }
        shape
    }

    pub fn is_back(&self, edge: &Edge) -> bool {
        self.back_edges.contains(&(edge.from.as_str(), edge.port.as_str(), edge.to.as_str()))
    }
}

/// A body runs as one branch, one iteration after another: it leads only back
/// to its Loop, is entered only through Body, has no parallel work, and holds
/// no Start, End, Join or other Loop.
fn check_loops(doc: &Experiment, shape: &LoopShape) -> EngineResult<()> {
    let by_id: HashMap<&str, &NodeKind> = doc.nodes.iter().map(|node| (node.id.as_str(), &node.kind)).collect();
    for node in doc.nodes.iter().filter(|node| matches!(node.kind, NodeKind::Loop { .. })) {
        let id = node.id.as_str();
        let body = &shape.bodies[id];
        if !shape.back_edges.iter().any(|(_, _, to)| *to == id) {
            return Err(EngineError::new("loop.no_return").at(id));
        }
        for &member in body {
            if matches!(by_id[member], NodeKind::Start | NodeKind::End | NodeKind::Fork | NodeKind::Join | NodeKind::Loop { .. }) {
                return Err(EngineError::new("loop.body_unsupported").at(member));
            }
            let mut ports = HashSet::new();
            for edge in doc.edges.iter().filter(|edge| edge.from == member) {
                if !ports.insert(edge.port.as_str()) {
                    return Err(EngineError::new("loop.body_parallel").at(member));
                }
                if edge.to != id && !body.contains(edge.to.as_str()) {
                    return Err(EngineError::new("loop.body_leaves").at(member));
                }
            }
        }
        for edge in doc.edges.iter().filter(|edge| body.contains(edge.to.as_str())) {
            let through_body = edge.from == id && edge.port == "body";
            if !through_body && !body.contains(edge.from.as_str()) {
                return Err(EngineError::new("loop.body_entered").at(&edge.from));
            }
        }
    }
    Ok(())
}

fn visit<'a>(
    id: &'a str,
    outgoing: &HashMap<&'a str, Vec<&'a Edge>>,
    marks: &mut HashMap<&'a str, u8>,
    order: &mut Vec<String>,
) -> EngineResult<()> {
    match marks.get(id) {
        Some(1) => return Err(EngineError::new("graph.cycle").at(id)),
        Some(2) => return Ok(()),
        _ => {}
    }
    marks.insert(id, 1);
    if let Some(edges) = outgoing.get(id) {
        for edge in edges {
            visit(&edge.to, outgoing, marks, order)?;
        }
    }
    marks.insert(id, 2);
    order.push(id.into());
    Ok(())
}

/// A *Change impairment* names one of the document's Impairments, an
/// *Emulator down/up* one of its Emulators.
fn check_references(doc: &Experiment) -> EngineResult<()> {
    let is = |id: &str, wanted: fn(&NodeKind) -> bool| doc.nodes.iter().any(|node| node.id == id && wanted(&node.kind));
    for node in &doc.nodes {
        let missing = match &node.kind {
            NodeKind::ImpairmentChange { relay, .. } if !is(relay, |kind| matches!(kind, NodeKind::Impairment { .. })) => {
                EngineError::new("impair.relay_unknown").with("id", relay).in_field(Field::new("relay"))
            }
            NodeKind::EmulatorState { emulator, .. } if !is(emulator, |kind| matches!(kind, NodeKind::Emulator { .. })) => {
                EngineError::new("emulator.node_unknown").with("id", emulator).in_field(Field::new("emulator"))
            }
            kind if kind.connection().is_some_and(|connection| !is(connection, |kind| matches!(kind, NodeKind::WsConnect { .. }))) => {
                EngineError::new("ws.connection_unknown").with("id", kind.connection().unwrap_or_default()).in_field(Field::new("connection"))
            }
            _ => continue,
        };
        return Err(missing.at(&node.id));
    }
    Ok(())
}

/// Validate with the active profile's values (the runner uses `validate_run`).
#[cfg(test)]
pub fn validate(doc: &Experiment) -> EngineResult<Vec<String>> {
    validate_with(doc, &data::effective_params(doc, doc.profile.as_deref(), &BTreeMap::new()))
}

/// A profile other than the active one (or the defaults) that would not run.
#[derive(Clone, Serialize)]
pub struct ProfileIssue {
    /// `None`: the defaults, without a profile.
    pub profile: Option<String>,
    pub error: EngineError,
}

/// Problems the other profiles would have, so switching never breaks a working
/// experiment silently. Meaningful once the active profile validates.
pub fn profile_issues(doc: &Experiment) -> Vec<ProfileIssue> {
    let none = BTreeMap::new();
    std::iter::once(None)
        .chain(doc.profiles.iter().map(|profile| Some(profile.name.clone())))
        .filter(|candidate| candidate != &doc.profile)
        .filter_map(|candidate| {
            let params = data::effective_params(doc, candidate.as_deref(), &none);
            validate_with(doc, &params).err().map(|error| ProfileIssue { profile: candidate, error })
        })
        .collect()
}

/// Validate a complete, bounded DAG with the given parameter values and return
/// its topological order. Every node connects exactly its required outputs
/// plus any optional ones; multiple inputs form a join.
pub fn validate_with(doc: &Experiment, params: &BTreeMap<String, String>) -> EngineResult<Vec<String>> {
    validate_document(doc)?;
    if doc.name.trim().is_empty() {
        return Err(EngineError::new("doc.name_required"));
    }
    data::check_names(doc)?;
    for node in &doc.nodes {
        check_node(&node.kind, params).map_err(|error| error.at(&node.id))?;
        if let Some(retry) = &node.retry {
            check_retry(&node.kind, retry).map_err(|error| error.at(&node.id))?;
        }
        if let Some(repeat) = &node.repeat {
            check_repeat(&node.kind, repeat).map_err(|error| error.at(&node.id))?;
        }
    }
    crate::emulator_run::check_run_binds(&doc.nodes)?;
    crate::netsim_run::check_run_binds(&doc.nodes, params)?;
    check_references(doc)?;
    // validate_document guarantees exactly one Start.
    let start = doc.nodes.iter().find(|node| matches!(node.kind, NodeKind::Start)).map(|node| node.id.clone()).unwrap_or_default();
    let by_id: HashMap<_, _> = doc.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let mut outgoing: HashMap<&str, Vec<&Edge>> = HashMap::new();
    for edge in &doc.edges {
        outgoing.entry(&edge.from).or_default().push(edge);
    }
    if doc.edges.iter().any(|edge| edge.to == start) {
        return Err(EngineError::new("graph.start_input").at(&start));
    }
    for node in &doc.nodes {
        let edges = outgoing.get(node.id.as_str()).map(Vec::as_slice).unwrap_or(&[]);
        let outputs = node.kind.outputs();
        if let Some(edge) = edges.iter().find(|edge| !outputs.required.contains(&edge.port.as_str()) && !outputs.optional.contains(&edge.port.as_str())) {
            return Err(EngineError::new("graph.port_unexpected").with("port", &edge.port).at(&node.id));
        }
        if !outputs.required.iter().all(|port| edges.iter().any(|edge| edge.port == *port)) {
            return Err(EngineError::new("graph.outputs_required").at(&node.id));
        }
    }
    // A loop's way back is not a cycle; ordering and the checks below see the
    // graph without it — and with the body before what follows Done, since a
    // Loop leaves only after its body ran.
    let shape = LoopShape::of(doc);
    check_loops(doc, &shape)?;
    let virtual_edges: Vec<Edge> = shape
        .back_edges
        .iter()
        .flat_map(|(from, _, loop_id)| {
            doc.edges
                .iter()
                .filter(move |edge| edge.from == *loop_id && edge.port != "body")
                .map(move |exit| Edge { from: from.to_string(), to: exit.to.clone(), port: "next".into() })
        })
        .collect();
    let mut ordered: HashMap<&str, Vec<&Edge>> = HashMap::new();
    for edge in doc.edges.iter().filter(|edge| !shape.is_back(edge)).chain(virtual_edges.iter()) {
        ordered.entry(&edge.from).or_default().push(edge);
    }
    let mut marks = HashMap::new();
    let mut order = Vec::new();
    visit(&start, &ordered, &mut marks, &mut order)?;
    if let Some(unreachable) = doc.nodes.iter().find(|node| !marks.contains_key(node.id.as_str())) {
        return Err(EngineError::new("graph.unreachable").at(&unreachable.id));
    }
    order.reverse();
    // A WebSocket send, wait or close comes after the connect it uses: on a
    // branch beside it, or before it, it would find no connection (or race one).
    let mut incoming: HashMap<&str, Vec<&str>> = HashMap::new();
    for edges in ordered.values() {
        for edge in edges {
            incoming.entry(edge.to.as_str()).or_default().push(edge.from.as_str());
        }
    }
    for node in &doc.nodes {
        let Some(connection) = node.kind.connection() else { continue };
        let mut before: HashSet<&str> = HashSet::new();
        let mut stack = vec![node.id.as_str()];
        while let Some(id) = stack.pop() {
            for from in incoming.get(id).into_iter().flatten() {
                if before.insert(from) {
                    stack.push(from);
                }
            }
        }
        if !before.contains(connection) {
            return Err(EngineError::new("ws.connection_after").with("id", connection).in_field(Field::new("connection")).at(&node.id));
        }
    }
    // `looped`: reached over a wire back from a Loop's body. A Loop's body runs
    // at least once, so Done and Limit follow only such an arrival — and a
    // request made in the body is there for the checks after the loop.
    let mut states = vec![(start.as_str(), false, false)];
    let mut seen = HashSet::new();
    while let Some((id, has_http, looped)) = states.pop() {
        if !seen.insert((id, has_http, looped)) {
            continue;
        }
        let node = by_id[id];
        if node.kind.reads_response() && !has_http {
            return Err(EngineError::new("graph.needs_http").at(id));
        }
        let has_http = has_http || matches!(node.kind, NodeKind::Http { .. });
        let entering = matches!(node.kind, NodeKind::Loop { .. }) && !looped;
        if let Some(edges) = outgoing.get(id) {
            for edge in edges.iter().filter(|edge| !entering || edge.port == "body") {
                states.push((&edge.to, has_http, shape.is_back(edge)));
            }
        }
    }
    data::check_values(doc, &order, &shape, params)?;
    Ok(order)
}

/// Validate what a run would use: the active profile plus Run with… values.
/// Returns the order and the effective values.
pub fn validate_run(doc: &Experiment, overrides: &BTreeMap<String, String>) -> EngineResult<(Vec<String>, BTreeMap<String, String>)> {
    if let Some(unknown) = overrides.keys().find(|name| !doc.params.iter().any(|param| &param.name == *name)) {
        return Err(EngineError::new("run.override_unknown").with("name", unknown));
    }
    let params = data::effective_params(doc, doc.profile.as_deref(), overrides);
    Ok((validate_with(doc, &params)?, params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experiment::{starter, Node};
    use serde_json::json;

    /// The code, node and field of a failed validation.
    fn problem(result: EngineResult<Vec<String>>) -> (String, Option<String>, Option<String>) {
        let error = result.unwrap_err();
        (error.code.clone(), error.node.clone(), error.field.as_ref().map(|field| field.key.clone()))
    }

    fn code(result: EngineResult<Vec<String>>) -> String {
        result.unwrap_err().into_code()
    }

    #[test]
    fn new_checks_require_http_and_round_trip() {
        for kind in [
            NodeKind::AssertBody { contains: "ready".into() },
            NodeKind::AssertHeader { name: "content-type".into(), contains: "json".into() },
            NodeKind::AssertLatency { max_ms: 100 },
        ] {
            let mut doc = starter();
            doc.nodes[2].kind = kind;
            let encoded = serde_json::to_string(&doc).unwrap();
            assert!(validate(&serde_json::from_str(&encoded).unwrap()).is_ok());
            doc.nodes[1].kind = NodeKind::Delay { ms: 1 };
            assert_eq!(problem(validate(&doc)), ("graph.needs_http".into(), Some("check".into()), None));
        }
    }

    #[test]
    fn mqtt_and_check_settings_are_validated_before_running() {
        let mut doc = starter();
        for (topic, expected) in [("lab/test", None), ("lab/+", Some("node.topic_wildcard")), ("", Some("node.required")), ("lab/#", Some("node.topic_wildcard"))] {
            doc.nodes[2].kind = NodeKind::Mqtt {
                host: "127.0.0.1".into(),
                port: 1883,
                topic: topic.into(),
                payload: "hello".into(),
                qos: 1,
                retain: false,
            };
            assert_eq!(validate(&doc).err().map(EngineError::into_code), expected.map(String::from), "{topic}");
        }
        for (kind, expected, field) in [
            (NodeKind::AssertBody { contains: "".into() }, "node.required", "expected_text"),
            (NodeKind::AssertHeader { name: "bad name".into(), contains: "".into() }, "node.header_invalid", "header_name"),
            (NodeKind::AssertLatency { max_ms: 0 }, "node.range", "max_ms"),
            (NodeKind::AssertLatency { max_ms: 120_001 }, "node.range", "max_ms"),
        ] {
            doc.nodes[2].kind = kind;
            assert_eq!(problem(validate(&doc)), (expected.into(), Some("check".into()), Some(field.into())));
        }
    }

    #[test]
    fn starter_is_valid_and_round_trips() {
        let doc: Experiment = serde_json::from_str(&serde_json::to_string(&starter()).unwrap()).unwrap();
        assert_eq!(validate(&doc).unwrap(), ["start", "request", "check", "end"]);
    }

    #[test]
    fn rejects_cycle_and_disconnected_nodes() {
        let mut doc = starter();
        doc.edges[2].to = "request".into();
        assert_eq!(problem(validate(&doc)), ("graph.cycle".into(), Some("request".into()), None));
        let mut doc = starter();
        doc.edges.pop();
        assert_eq!(problem(validate(&doc)), ("graph.outputs_required".into(), Some("check".into()), None));
        let mut doc = starter();
        doc.nodes.push(Node { id: "orphan".into(), x: 0.0, y: 0.0, retry: None, repeat: None, kind: NodeKind::Log { message: "x".into() } });
        doc.edges.push(Edge { from: "orphan".into(), to: "end".into(), port: "next".into() });
        assert_eq!(problem(validate(&doc)), ("graph.unreachable".into(), Some("orphan".into()), None));
    }

    #[test]
    fn status_check_needs_previous_request() {
        let mut doc = starter();
        doc.nodes[1].kind = NodeKind::Delay { ms: 50 };
        assert_eq!(code(validate(&doc)), "graph.needs_http");
    }

    #[test]
    fn branch_paths_join_and_incomplete_drafts_save() {
        let mut doc = starter();
        doc.nodes[2].kind = NodeKind::BranchStatus { status: 200 };
        doc.nodes.push(Node { id: "good".into(), x: 540.0, y: 30.0, retry: None, repeat: None, kind: NodeKind::Delay { ms: 10 } });
        doc.edges.pop();
        doc.edges.extend([
            Edge { from: "check".into(), to: "good".into(), port: "yes".into() },
            Edge { from: "check".into(), to: "end".into(), port: "no".into() },
            Edge { from: "good".into(), to: "end".into(), port: "next".into() },
        ]);
        assert_eq!(validate(&doc).unwrap().len(), 5);
        doc.edges.retain(|edge| edge.port != "no");
        assert!(validate_document(&doc).is_ok());
        assert_eq!(problem(validate(&doc)), ("graph.outputs_required".into(), Some("check".into()), None));
        doc.edges.push(Edge { from: "check".into(), to: "end".into(), port: "next".into() });
        assert_eq!(code(validate(&doc)), "graph.port_unexpected");
    }

    #[test]
    fn branch_rejects_cycle() {
        let mut doc = starter();
        doc.nodes[2].kind = NodeKind::BranchStatus { status: 200 };
        doc.edges.pop();
        doc.edges.extend([
            Edge { from: "check".into(), to: "request".into(), port: "yes".into() },
            Edge { from: "check".into(), to: "end".into(), port: "no".into() },
        ]);
        assert_eq!(code(validate(&doc)), "graph.cycle");
    }

    #[test]
    fn document_structure_errors_say_which_node() {
        let mut doc = starter();
        doc.version = 999;
        assert_eq!(validate_document(&doc).unwrap_err().params["version"], "999");
        let mut doc = starter();
        doc.nodes[1].id = doc.nodes[0].id.clone();
        let duplicate = validate_document(&doc).unwrap_err();
        assert_eq!((duplicate.code.as_str(), duplicate.node.as_deref()), ("doc.node_id_duplicate", Some("start")));
        let mut doc = starter();
        doc.nodes[2].x = -1.0;
        assert_eq!(validate_document(&doc).unwrap_err().node.as_deref(), Some("check"));
        let mut doc = starter();
        doc.edges[0].port = "sideways".into();
        assert!(validate_document(&doc).unwrap_err().is("doc.port_invalid"));
        let mut doc = starter();
        doc.edges.push(doc.edges[0].clone());
        assert!(validate_document(&doc).unwrap_err().is("doc.connection_duplicate"));
        // One output feeding two nodes is parallel work, not a mistake.
        let mut doc = starter();
        doc.edges[1].from = doc.edges[0].from.clone();
        assert!(validate_document(&doc).is_ok());
        let mut doc = starter();
        doc.edges[0].to = "absent".into();
        assert!(validate_document(&doc).unwrap_err().is("doc.connection_invalid"));
        let mut doc = starter();
        doc.name = " ".into();
        assert_eq!(code(validate(&doc)), "doc.name_required");
    }

    #[test]
    fn fork_and_join_parallel_branches_validate_cleanly() {
        let mut doc = starter();
        doc.nodes[1].kind = NodeKind::Fork;
        doc.nodes[1].id = "fork".into();
        doc.nodes[2].kind = NodeKind::Join;
        doc.nodes[2].id = "join".into();
        doc.nodes.push(Node { id: "branch_a".into(), x: 240.0, y: 20.0, retry: None, repeat: None, kind: NodeKind::Delay { ms: 100 } });
        doc.nodes.push(Node { id: "branch_b".into(), x: 240.0, y: 120.0, retry: None, repeat: None, kind: NodeKind::Log { message: "Parallel test".into() } });
        doc.edges = vec![
            Edge { from: "start".into(), to: "fork".into(), port: "next".into() },
            Edge { from: "fork".into(), to: "branch_a".into(), port: "branch1".into() },
            Edge { from: "fork".into(), to: "branch_b".into(), port: "branch2".into() },
            Edge { from: "branch_a".into(), to: "join".into(), port: "next".into() },
            Edge { from: "branch_b".into(), to: "join".into(), port: "next".into() },
            Edge { from: "join".into(), to: "end".into(), port: "next".into() },
        ];
        let validated = validate(&doc);
        assert!(validated.is_ok(), "{:?}", validated.err());
        assert_eq!(validated.unwrap().len(), 6);
        doc.edges.retain(|e| e.port != "branch2");
        assert_eq!(problem(validate(&doc)).1.as_deref(), Some("fork"));
    }

    /// Start → login (HTTP) → the nodes given → End, wired in order.
    fn flow(params: serde_json::Value, middle: serde_json::Value, extra_edges: serde_json::Value) -> Experiment {
        let mut nodes = vec![
            json!({ "id": "start", "type": "start", "x": 0, "y": 0 }),
            json!({ "id": "login", "type": "http", "x": 0, "y": 0,
                "request": { "method": "POST", "url": "{{api}}/login", "headers": [], "body": null, "timeout_ms": 1000 } }),
        ];
        nodes.extend(middle.as_array().unwrap().iter().cloned());
        nodes.push(json!({ "id": "end", "type": "end", "x": 0, "y": 0 }));
        let edges = if extra_edges.as_array().unwrap().is_empty() {
            let ids: Vec<String> = nodes.iter().map(|node| node["id"].as_str().unwrap().to_string()).collect();
            ids.windows(2).map(|pair| json!({ "from": pair[0], "to": pair[1] })).collect()
        } else {
            extra_edges.as_array().unwrap().clone()
        };
        serde_json::from_value(json!({ "version": VERSION, "name": "data", "params": params, "seed": null, "nodes": nodes, "edges": edges }))
            .unwrap()
    }

    fn api() -> serde_json::Value {
        json!([{ "name": "api", "value": "http://127.0.0.1:8080" }])
    }

    fn node(id: &str, kind: serde_json::Value) -> serde_json::Value {
        let mut value = kind;
        value["id"] = id.into();
        value["x"] = 0.into();
        value["y"] = 0.into();
        value
    }

    #[test]
    fn variables_must_be_set_on_every_path_before_use() {
        let extract = node("take", json!({ "type": "extract", "variable": "token", "from": "json", "expr": "$.token" }));
        let use_it = node("use", json!({ "type": "log", "message": "token {{token}}" }));
        assert!(validate(&flow(api(), json!([extract.clone(), use_it.clone()]), json!([]))).is_ok());
        let early = validate(&flow(api(), json!([use_it.clone(), extract.clone()]), json!([]))).unwrap_err();
        assert_eq!((early.code.as_str(), early.node.as_deref(), early.params["name"].as_str()), ("name.not_on_every_path", Some("use"), "token"));
        assert_eq!(early.field, Some(Field::new("message")));
        let unknown = validate(&flow(api(), json!([node("use", json!({ "type": "log", "message": "{{nope}}" }))]), json!([]))).unwrap_err();
        assert_eq!((unknown.code.as_str(), unknown.params["name"].as_str()), ("name.unknown", "nope"));
        let shadow = json!([{ "name": "api", "value": "http://127.0.0.1" }, { "name": "token", "value": "x" }]);
        assert_eq!(problem(validate(&flow(shadow, json!([extract.clone()]), json!([])))), ("variable.shadows_param".into(), Some("take".into()), Some("variable".into())));

        // Set on one branch of a status branch only: not available after the merge.
        let branch = node("branch", json!({ "type": "branch_status", "status": 200 }));
        let wait = node("wait", json!({ "type": "delay", "ms": 1 }));
        let edges = json!([
            { "from": "start", "to": "login" }, { "from": "login", "to": "branch" },
            { "from": "branch", "to": "take", "port": "yes" }, { "from": "branch", "to": "wait", "port": "no" },
            { "from": "take", "to": "use" }, { "from": "wait", "to": "use" }, { "from": "use", "to": "end" }
        ]);
        assert_eq!(code(validate(&flow(api(), json!([branch, extract.clone(), wait.clone(), use_it.clone()]), edges))), "name.not_on_every_path");

        // Parallel branches all run, so a Join makes a variable from either branch available.
        let fork = node("fork", json!({ "type": "fork" }));
        let join = node("join", json!({ "type": "join" }));
        let edges = json!([
            { "from": "start", "to": "login" }, { "from": "login", "to": "fork" },
            { "from": "fork", "to": "take", "port": "branch1" }, { "from": "fork", "to": "wait", "port": "branch2" },
            { "from": "take", "to": "join" }, { "from": "wait", "to": "join" },
            { "from": "join", "to": "use" }, { "from": "use", "to": "end" }
        ]);
        let joined = validate(&flow(api(), json!([fork, extract, wait, join, use_it]), edges));
        assert!(joined.is_ok(), "{:?}", joined.err());
    }

    #[test]
    fn parameter_only_fields_are_checked_like_literals() {
        assert!(validate(&flow(api(), json!([]), json!([]))).is_ok());
        let ftp = json!([{ "name": "api", "value": "ftp://127.0.0.1" }]);
        assert_eq!(problem(validate(&flow(ftp, json!([]), json!([])))), ("node.url_invalid".into(), Some("login".into()), Some("url".into())));
        assert_eq!(problem(validate(&flow(json!([]), json!([]), json!([])))), ("name.unknown".into(), Some("login".into()), Some("url".into())));
        let osc = |target: &str| node("cue", json!({ "type": "osc", "target": target, "address": "/cue", "args": [] }));
        let host = json!([{ "name": "api", "value": "http://127.0.0.1" }, { "name": "host", "value": "127.0.0.1" }]);
        assert!(validate(&flow(host.clone(), json!([osc("{{host}}:9000")]), json!([]))).is_ok());
        assert_eq!(problem(validate(&flow(host.clone(), json!([osc("{{host}}")]), json!([])))), ("node.target_invalid".into(), Some("cue".into()), Some("target".into())));
        let syntax = validate(&flow(host, json!([osc("{{host:9000")]), json!([]))).unwrap_err();
        assert_eq!((syntax.code.as_str(), syntax.node.as_deref(), syntax.params["position"].as_str()), ("template.unclosed", Some("cue"), "1"));
        let bad_path = node("take", json!({ "type": "extract", "variable": "t", "from": "json", "expr": "$.a[" }));
        assert_eq!(problem(validate(&flow(api(), json!([bad_path]), json!([])))), ("json_path.invalid".into(), Some("take".into()), Some("json_path".into())));
        let bad_name = node("take", json!({ "type": "extract", "variable": "run", "from": "status" }));
        assert_eq!(code(validate(&flow(api(), json!([bad_name]), json!([])))), "variable.name_reserved");
        let header = flow(api(), json!([]), json!([]));
        let mut header = serde_json::to_value(header).unwrap();
        header["nodes"][1]["request"]["headers"] = json!([["X-Ok", "1"], ["bad name", "2"]]);
        let header: Experiment = serde_json::from_value(header).unwrap();
        let invalid = validate(&header).unwrap_err();
        assert_eq!((invalid.code.as_str(), invalid.field.clone()), ("node.header_invalid", Some(Field::nth("header_name", 2))));
    }

    #[test]
    fn other_profiles_are_checked_without_blocking_the_active_one() {
        let mut doc = flow(api(), json!([]), json!([]));
        doc.profiles = serde_json::from_value(json!([
            { "name": "Stage", "values": { "api": "http://10.0.0.20:8080" } },
            { "name": "Broken", "values": { "api": "ftp://10.0.0.21" } }
        ]))
        .unwrap();
        doc.profile = Some("Stage".into());
        assert!(validate(&doc).is_ok());
        let issues = profile_issues(&doc);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].profile.as_deref(), Some("Broken"));
        assert_eq!(issues[0].error.code, "node.url_invalid");
        doc.profile = Some("Broken".into());
        assert_eq!(code(validate(&doc)), "node.url_invalid");
        // The defaults count as a choice too.
        doc.params[0].value = "not a url".into();
        doc.profile = Some("Stage".into());
        assert!(profile_issues(&doc).iter().any(|issue| issue.profile.is_none()));
        // Run with… values take part in validation like any other value.
        doc.profile = Some("Broken".into());
        assert!(validate_run(&doc, &BTreeMap::from([("api".to_string(), "http://ok".to_string())])).is_ok());
        let ghost = validate_run(&doc, &BTreeMap::from([("ghost".to_string(), "x".to_string())])).err().unwrap();
        assert_eq!((ghost.code.as_str(), ghost.params["name"].as_str()), ("run.override_unknown", "ghost"));
    }

    #[test]
    fn value_nodes_need_a_response_and_branches_need_both_outputs() {
        let mut doc = flow(api(), json!([
            node("take", json!({ "type": "extract", "variable": "code", "from": "status" })),
            node("check", json!({ "type": "assert_value", "value": "{{code}}", "op": "lt", "expected": "300" })),
        ]), json!([]));
        assert!(validate(&doc).is_ok());
        doc.nodes[1].kind = NodeKind::Delay { ms: 1 };
        assert_eq!(code(validate(&doc)), "graph.needs_http");
        let branch = flow(api(), json!([
            node("pick", json!({ "type": "branch_value", "value": "{{api}}", "op": "contains", "expected": "127." })),
        ]), json!([
            { "from": "start", "to": "login" }, { "from": "login", "to": "pick" }, { "from": "pick", "to": "end", "port": "yes" }
        ]));
        assert_eq!(problem(validate(&branch)), ("graph.outputs_required".into(), Some("pick".into()), None));
    }

    /// A Loop's body runs at least once: a request it makes is there after Done.
    #[test]
    fn a_request_in_a_loop_body_counts_for_the_checks_after_the_loop() {
        let looped = |inside: serde_json::Value| -> Experiment {
            serde_json::from_value(json!({
                "version": VERSION, "name": "poll", "params": [], "profiles": [], "profile": null, "seed": null,
                "nodes": [
                    { "id": "start", "type": "start", "x": 0, "y": 0 },
                    node("loop", json!({ "type": "loop", "max": 3 })),
                    node("inside", inside),
                    node("status", json!({ "type": "assert_status", "status": 200 })),
                    { "id": "end", "type": "end", "x": 0, "y": 0 }
                ],
                "edges": [
                    { "from": "start", "to": "loop" }, { "from": "loop", "to": "inside", "port": "body" }, { "from": "inside", "to": "loop" },
                    { "from": "loop", "to": "status", "port": "done" }, { "from": "status", "to": "end" }
                ]
            }))
            .unwrap()
        };
        let request = json!({ "type": "http", "request": { "method": "GET", "url": "http://127.0.0.1:8080/", "headers": [], "body": null, "timeout_ms": 1000 } });
        assert!(validate(&looped(request)).is_ok());
        assert_eq!(problem(validate(&looped(json!({ "type": "delay", "ms": 1 })))), ("graph.needs_http".into(), Some("status".into()), None));
    }

    #[test]
    fn tcp_and_log_nodes_validate_bounds() {
        let mut doc = starter();
        doc.nodes[1].kind = NodeKind::Tcp { host: "".into(), port: 80, payload: "hi".into(), timeout_ms: 1000 };
        assert_eq!(problem(validate(&doc)), ("node.required".into(), Some("request".into()), Some("host".into())));
        doc.nodes[1].kind = NodeKind::Tcp { host: "127.0.0.1".into(), port: 0, payload: "hi".into(), timeout_ms: 1000 };
        assert_eq!(problem(validate(&doc)).2.as_deref(), Some("port"));
        doc.nodes[1].kind = NodeKind::Tcp { host: "127.0.0.1".into(), port: 80, payload: "hi".into(), timeout_ms: 0 };
        let timeout = validate(&doc).unwrap_err();
        assert_eq!((timeout.params["min"].as_str(), timeout.params["max"].as_str()), ("1", "120000"));
        doc.nodes[1].kind = NodeKind::Log { message: "a".repeat(10_001) };
        assert_eq!(problem(validate(&doc)), ("node.too_long".into(), Some("request".into()), Some("message".into())));
    }

    /// Start → ping (OSC) → wait → `after` on Matched → End, plus `extra` nodes and edges.
    fn waiting(wait: serde_json::Value, after: &str, timeout_edge: bool) -> Experiment {
        let mut edges = vec![
            json!({ "from": "start", "to": "ping" }),
            json!({ "from": "ping", "to": "wait" }),
            json!({ "from": "wait", "to": "after", "port": "matched" }),
            json!({ "from": "after", "to": "end" }),
        ];
        let mut nodes = vec![
            json!({ "id": "start", "type": "start", "x": 0, "y": 0 }),
            node("ping", json!({ "type": "osc", "target": "127.0.0.1:9000", "address": "/ping", "args": [] })),
            node("wait", wait),
            node("after", json!({ "type": "log", "message": after })),
            json!({ "id": "end", "type": "end", "x": 0, "y": 0 }),
        ];
        if timeout_edge {
            nodes.push(node("late", json!({ "type": "log", "message": "late {{reply.address}}" })));
            edges.push(json!({ "from": "wait", "to": "late", "port": "timeout" }));
            edges.push(json!({ "from": "late", "to": "end" }));
        }
        serde_json::from_value(json!({ "version": VERSION, "name": "wait", "nodes": nodes, "edges": edges })).unwrap()
    }

    fn wait_osc(extra: serde_json::Value) -> serde_json::Value {
        let mut base = json!({ "type": "wait_osc", "bind": "127.0.0.1:9001", "address": "/pong*", "timeout_ms": 500 });
        for (key, value) in extra.as_object().unwrap() {
            base[key] = value.clone();
        }
        base
    }

    #[test]
    fn waits_validate_their_fields_and_set_the_reply_on_matched_only() {
        assert_eq!(validate(&waiting(wait_osc(json!({})), "got {{reply.args[0]}}", false)).unwrap().len(), 5);
        let timeout_path = waiting(wait_osc(json!({})), "{{reply.address}}", true);
        assert_eq!(problem(validate(&timeout_path)), ("name.not_on_every_path".into(), Some("late".into()), Some("message".into())));
        let mut fine = timeout_path.clone();
        if let NodeKind::Log { message } = &mut fine.nodes[5].kind {
            *message = "late".into();
        }
        assert!(validate(&fine).is_ok(), "{:?}", validate(&fine).err());

        for (extra, expected, field) in [
            (json!({ "bind": "" }), "node.required", "bind"),
            (json!({ "bind": "127.0.0.1" }), "node.bind_invalid", "bind"),
            (json!({ "bind": "127.0.0.1:0" }), "node.bind_invalid", "bind"),
            (json!({ "address": "pong" }), "osc.pattern_slash", "address"),
            (json!({ "address": "/cue/[1-" }), "osc.pattern_unclosed", "address"),
            (json!({ "timeout_ms": 0 }), "node.range", "timeout_ms"),
            (json!({ "variable": "run" }), "variable.name_reserved", "variable"),
            (json!({ "args": [{ "index": 64, "op": "eq", "value": "1" }] }), "node.range", "rule"),
            (json!({ "args": [{ "index": 0, "op": "matches", "value": "(" }] }), "regex.invalid", "rule_value"),
        ] {
            assert_eq!(problem(validate(&waiting(wait_osc(extra.clone()), "x", false))), (expected.into(), Some("wait".into()), Some(field.into())), "{extra}");
        }
        for (mode, pattern, expected) in [("hex", "zz", "hex.invalid"), ("regex", "(", "regex.invalid"), ("contains", "", "node.required")] {
            let wait = json!({ "type": "wait_udp", "bind": "127.0.0.1:9001", "mode": mode, "pattern": pattern });
            assert_eq!(problem(validate(&waiting(wait, "x", false))), (expected.into(), Some("wait".into()), Some("pattern".into())), "{mode}");
        }
        let any = json!({ "type": "wait_udp", "bind": "127.0.0.1:9001", "mode": "any" });
        assert!(validate(&waiting(any, "{{reply.text}}", false)).is_ok());

        let mut unwired = waiting(wait_osc(json!({})), "x", false);
        unwired.edges[2].port = "next".into();
        assert_eq!(problem(validate(&unwired)), ("graph.port_unexpected".into(), Some("wait".into()), None));
    }
}
