//! Values inside experiments: parameters, templated fields, extraction from a
//! response and value comparisons. The runner, validation and the editor's
//! preview commands share these functions, so a field means the same thing
//! everywhere. Specified in `docs/milestone-3-data.md`; errors are
//! `EngineError` codes with the node and field they are about.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Experiment, Node, NodeKind, Until};
use super::experiment_validate::LoopShape;
use super::http::HttpResponse;
use super::matching::{self, UdpMode};
pub use super::matching::{compare, CompareOp};
use super::osc_codec::OscArg;
use super::secrets::{self, SecretStore, MASK};
use super::template::{self, Ref, Renderer, Scope, RESERVED};

pub const MAX_PARAMS: usize = 64;
pub const MAX_PROFILES: usize = 32;
const MAX_PROFILE_NAME: usize = 64;
pub const MAX_PARAM_BYTES: usize = 64 * 1024;
/// Seeds must survive a round trip through JavaScript numbers.
pub const MAX_SEED: u64 = (1 << 53) - 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    #[serde(default)]
    pub value: String,
}

/// A named set of parameter values; parameters it does not mention keep their default.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub values: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractFrom {
    Json,
    Header,
    Status,
    Body,
    Regex,
}

/// The values a run uses: defaults, then the profile, then run overrides —
/// later wins. Runner, validation, preview and *Send now* all go through here.
pub fn effective_params(doc: &Experiment, profile: Option<&str>, overrides: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut values: BTreeMap<String, String> =
        doc.params.iter().map(|param| (param.name.clone(), param.value.clone())).collect();
    if let Some(profile) = profile.and_then(|name| doc.profiles.iter().find(|profile| profile.name == name)) {
        for (name, value) in &profile.values {
            if values.contains_key(name) {
                values.insert(name.clone(), value.clone());
            }
        }
    }
    for (name, value) in overrides {
        if values.contains_key(name) {
            values.insert(name.clone(), value.clone());
        }
    }
    values
}

/// Size limits only: a draft being typed must always save, so names and
/// references are checked before a run (`check_names`), not here.
pub fn check_sizes(params: &[Param], profiles: &[Profile], seed: Option<u64>) -> EngineResult<()> {
    if params.len() > MAX_PARAMS {
        return Err(EngineError::new("params.too_many").with("max", MAX_PARAMS));
    }
    if profiles.len() > MAX_PROFILES {
        return Err(EngineError::new("profiles.too_many").with("max", MAX_PROFILES));
    }
    let values = params.iter().map(|param| (param.name.as_str(), &param.value))
        .chain(profiles.iter().flat_map(|profile| profile.values.iter().map(|(name, value)| (name.as_str(), value))));
    for (name, value) in values {
        if value.len() > MAX_PARAM_BYTES {
            return Err(EngineError::new("param.too_large").with("name", name).with("max", MAX_PARAM_BYTES / 1024));
        }
    }
    if let Some(profile) = profiles.iter().find(|profile| profile.values.len() > MAX_PARAMS) {
        return Err(EngineError::new("profile.values_too_many").with("profile", &profile.name).with("max", MAX_PARAMS));
    }
    check_seed(seed)
}

pub fn check_seed(seed: Option<u64>) -> EngineResult<()> {
    match seed {
        Some(seed) if seed > MAX_SEED => Err(EngineError::new("seed.range").with("max", MAX_SEED)),
        _ => Ok(()),
    }
}

/// Names and references of parameters and profiles, checked before a run.
pub fn check_names(doc: &Experiment) -> EngineResult<()> {
    let mut seen = BTreeSet::new();
    for param in &doc.params {
        check_param_name(&param.name)?;
        if !seen.insert(param.name.as_str()) {
            return Err(EngineError::new("param.duplicate").with("name", &param.name));
        }
    }
    let mut profiles = BTreeSet::new();
    for profile in &doc.profiles {
        let name = profile.name.trim();
        if name.is_empty() || name.chars().count() > MAX_PROFILE_NAME {
            return Err(EngineError::new("profile.name_invalid").with("max", MAX_PROFILE_NAME));
        }
        if !profiles.insert(name) {
            return Err(EngineError::new("profile.duplicate").with("name", name));
        }
        if let Some(unknown) = profile.values.keys().find(|key| !seen.contains(key.as_str())) {
            return Err(EngineError::new("profile.unknown_param").with("profile", name).with("name", unknown));
        }
    }
    if let Some(active) = &doc.profile {
        if !doc.profiles.iter().any(|profile| &profile.name == active) {
            return Err(EngineError::new("profile.active_missing").with("name", active));
        }
    }
    Ok(())
}

fn check_param_name(name: &str) -> EngineResult<()> {
    if !template::is_ident(name) {
        return Err(EngineError::new("param.name_invalid").with("name", name));
    }
    if RESERVED.contains(&name) {
        return Err(EngineError::new("param.name_reserved").with("name", name));
    }
    Ok(())
}

fn check_variable_name(name: &str) -> EngineResult<()> {
    if !template::is_ident(name) {
        return Err(EngineError::new("variable.name_invalid").with("name", name));
    }
    if RESERVED.contains(&name) {
        return Err(EngineError::new("variable.name_reserved").with("name", name));
    }
    Ok(())
}

/// Every text field of a node that may hold templates, with the field it is.
/// The single list behind validation, the preview and execution.
fn texts_mut(kind: &mut NodeKind) -> Vec<(Field, &mut String)> {
    let mut fields = Vec::new();
    match kind {
        NodeKind::Http { request } => {
            fields.push((Field::new("url"), &mut request.url));
            for (index, (name, value)) in request.headers.iter_mut().enumerate() {
                fields.push((Field::nth("header_name", index + 1), name));
                fields.push((Field::nth("header_value", index + 1), value));
            }
            if let Some(body) = &mut request.body {
                fields.push((Field::new("body"), body));
            }
        }
        NodeKind::Osc { target, address, args, reply } => {
            fields.push((Field::new("target"), target));
            fields.push((Field::new("address"), address));
            for (index, arg) in args.iter_mut().enumerate() {
                if let OscArg::Str(text) = arg {
                    fields.push((Field::nth("argument", index + 1), text));
                }
            }
            // As in Wait for OSC: the pattern and rule values; the bind is opened before the run.
            if let Some(reply) = reply {
                fields.push((Field::new("reply_address"), &mut reply.address));
                for (index, rule) in reply.args.iter_mut().enumerate() {
                    fields.push((Field::nth("rule_value", index + 1), &mut rule.value));
                }
            }
        }
        NodeKind::Udp { target, text, reply } => {
            fields.push((Field::new("target"), target));
            fields.push((Field::new("payload"), text));
            if let Some(reply) = reply.as_mut().filter(|reply| reply.mode != UdpMode::Any) {
                fields.push((Field::new("reply_pattern"), &mut reply.pattern));
            }
        }
        NodeKind::Tcp { host, payload, .. } => {
            fields.push((Field::new("host"), host));
            fields.push((Field::new("payload"), payload));
        }
        NodeKind::Mqtt { host, topic, payload, .. } => {
            fields.push((Field::new("broker"), host));
            fields.push((Field::new("topic"), topic));
            fields.push((Field::new("payload"), payload));
        }
        NodeKind::Log { message } => fields.push((Field::new("message"), message)),
        NodeKind::AssertBody { contains } => fields.push((Field::new("expected_text"), contains)),
        NodeKind::AssertHeader { name, contains } => {
            fields.push((Field::new("header_name"), name));
            fields.push((Field::new("expected_text"), contains));
        }
        NodeKind::AssertValue { value, expected, .. }
        | NodeKind::BranchValue { value, expected, .. }
        | NodeKind::Loop { until: Some(Until { value, expected, .. }), .. } => {
            fields.push((Field::new("value"), value));
            fields.push((Field::new("expected"), expected));
        }
        // The bind address is opened before the run, so it is never templated.
        NodeKind::WaitOsc { address, args, .. } => {
            fields.push((Field::new("address"), address));
            for (index, rule) in args.iter_mut().enumerate() {
                fields.push((Field::nth("rule_value", index + 1), &mut rule.value));
            }
        }
        NodeKind::WaitUdp { mode, pattern, .. } if *mode != UdpMode::Any => fields.push((Field::new("pattern"), pattern)),
        // Broker and topic may use parameters (validation allows nothing else): the run subscribes before its first step.
        NodeKind::WaitMqtt { host, topic, mode, pattern, .. } => {
            fields.push((Field::new("broker"), host));
            fields.push((Field::new("topic"), topic));
            if *mode != UdpMode::Any {
                fields.push((Field::new("pattern"), pattern));
            }
        }
        // As in Wait for OSC: the path pattern and the conditions, resolved when the wait runs.
        NodeKind::WaitHttp { path, when, .. } => {
            fields.push((Field::new("path"), path));
            for (index, condition) in when.iter_mut().enumerate() {
                fields.push((Field::nth("condition", index + 1), &mut condition.name));
                fields.push((Field::nth("condition", index + 1), &mut condition.value));
            }
        }
        // An emulator renders its replies itself, with what arrived (`emulator.rs`).
        _ => {}
    }
    fields
}

pub fn text_fields(kind: &NodeKind) -> Vec<(Field, String)> {
    let mut copy = kind.clone();
    texts_mut(&mut copy).into_iter().map(|(field, text)| (field, text.clone())).collect()
}

/// A copy of the node kind with every templated field resolved.
pub fn render_kind(kind: &NodeKind, renderer: &mut Renderer) -> EngineResult<NodeKind> {
    let mut rendered = kind.clone();
    for (field, text) in texts_mut(&mut rendered) {
        *text = renderer.render(text).map_err(|error| error.in_field(field))?;
    }
    Ok(rendered)
}

/// The variable a node writes, and the output it is set on.
pub fn written_var(kind: &NodeKind) -> Option<(&str, &'static str)> {
    match kind {
        NodeKind::Extract { variable, .. } => Some((variable, "next")),
        // A timeout has no reply, so the variable exists on Matched only.
        NodeKind::WaitOsc { variable, .. } | NodeKind::WaitUdp { variable, .. } | NodeKind::WaitMqtt { variable, .. } | NodeKind::WaitHttp { variable, .. } => Some((variable, "matched")),
        // A send that expects a reply passes only with one.
        NodeKind::Osc { reply: Some(reply), .. } => Some((&reply.variable, "next")),
        NodeKind::Udp { reply: Some(reply), .. } => Some((&reply.variable, "next")),
        _ => None,
    }
}

/// The text of a field when it depends on parameters only — the value a run
/// will use — so it can be checked like a literal before the run.
pub fn static_text(text: &str, params: &BTreeMap<String, String>) -> Option<String> {
    let parsed = template::parse(text).ok()?;
    if !parsed.is_static(params) {
        return None;
    }
    let (vars, secrets) = (BTreeMap::new(), BTreeMap::new());
    let scope = Scope { params, vars: &vars, secrets: &secrets, run_id: 0, seed: 0, node_id: "", count: 1, now_ms: 0 };
    Renderer::new(scope).render(text).ok()
}

/// Read one value out of the latest response on the executed path.
pub fn extract(from: ExtractFrom, expr: &str, response: Option<&HttpResponse>) -> EngineResult<Value> {
    let response = response.ok_or_else(|| EngineError::new("check.no_response"))?;
    let truncated = || EngineError::new("extract.truncated").with("bytes", response.body_bytes);
    match from {
        ExtractFrom::Status => Ok(response.status.into()),
        ExtractFrom::Header => response
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(expr.trim()))
            .map(|(_, value)| Value::String(value.clone()))
            .ok_or_else(|| EngineError::new("extract.header_missing").with("name", expr.trim())),
        ExtractFrom::Body if response.truncated => Err(truncated()),
        ExtractFrom::Body => Ok(Value::String(response.body.clone())),
        ExtractFrom::Json => {
            if response.truncated {
                return Err(truncated());
            }
            let segments = template::parse_json_path(expr)?;
            let body: Value = serde_json::from_str(&response.body)
                .map_err(|error| EngineError::new("extract.not_json").because(error))?;
            template::lookup(&body, &segments)
                .cloned()
                .ok_or_else(|| EngineError::new("extract.json_missing").with("path", template::path_text(&segments)))
        }
        ExtractFrom::Regex => {
            let pattern = matching::compile_regex(expr)?;
            match pattern.captures(&response.body) {
                Some(found) => Ok(Value::String(
                    found.get(1).or_else(|| found.get(0)).map(|m| m.as_str()).unwrap_or("").to_string(),
                )),
                None if response.truncated => Err(truncated()),
                None => Err(EngineError::new("extract.no_match").with("pattern", expr)),
            }
        }
    }
}

/// Short text for a timeline message about a value.
pub fn value_preview(value: &Value) -> String {
    matching::shorten(&template::value_text(value))
}

/// The field a node's extraction expression is shown as.
pub fn extract_field(from: ExtractFrom) -> Field {
    match from {
        ExtractFrom::Header => Field::new("header_name"),
        ExtractFrom::Regex => Field::new("pattern"),
        _ => Field::new("json_path"),
    }
}

/// Literal patterns and paths a node carries, checked before a run.
fn check_literals(node: &Node, params: &BTreeMap<String, String>) -> EngineResult<()> {
    match &node.kind {
        NodeKind::Extract { from: ExtractFrom::Json, expr, .. } => {
            template::parse_json_path(expr).map(|_| ()).map_err(|error| error.in_field(Field::new("json_path")))
        }
        NodeKind::Extract { from: ExtractFrom::Header, expr, .. }
            if reqwest::header::HeaderName::from_bytes(expr.trim().as_bytes()).is_err() =>
        {
            Err(EngineError::new("node.header_invalid").with("value", expr).in_field(Field::new("header_name")))
        }
        NodeKind::Extract { from: ExtractFrom::Regex, expr, .. } => {
            matching::compile_regex(expr).map(|_| ()).map_err(|error| error.in_field(Field::new("pattern")))
        }
        NodeKind::AssertValue { op: CompareOp::Matches, expected, .. }
        | NodeKind::BranchValue { op: CompareOp::Matches, expected, .. }
        | NodeKind::Loop { until: Some(Until { op: CompareOp::Matches, expected, .. }), .. } => match static_text(expected, params) {
            Some(pattern) => matching::compile_regex(&pattern).map(|_| ()).map_err(|error| error.in_field(Field::new("expected"))),
            None => Ok(()),
        },
        NodeKind::WaitOsc { address, args, .. } => check_osc_reply(address, args, Field::new("address"), params),
        NodeKind::Osc { reply: Some(reply), .. } => check_osc_reply(&reply.address, &reply.args, Field::new("reply_address"), params),
        NodeKind::WaitUdp { mode, pattern, .. } | NodeKind::WaitMqtt { mode, pattern, .. } => check_udp_pattern(*mode, pattern, Field::new("pattern"), params),
        NodeKind::Udp { reply: Some(reply), .. } => check_udp_pattern(reply.mode, &reply.pattern, Field::new("reply_pattern"), params),
        // What is literal already is checked as the wait will read it.
        NodeKind::WaitHttp { method, path, when, .. } => {
            use crate::emulator::Condition;
            use crate::emulator_match::{Check, RequestMatcher};
            RequestMatcher::new(method, &static_text(path, params).unwrap_or_else(|| "/".into()), &[])?;
            for (index, condition) in when.iter().enumerate() {
                if let (Some(name), Some(value)) = (static_text(&condition.name, params), static_text(&condition.value, params)) {
                    Check::new(&Condition { name, value, ..condition.clone() }, Field::nth("condition", index + 1))?;
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// A literal OSC address pattern and literal regex rule values compile.
fn check_osc_reply(address: &str, args: &[matching::ArgRule], field: Field, params: &BTreeMap<String, String>) -> EngineResult<()> {
    if let Some(pattern) = static_text(address, params) {
        matching::OscPattern::parse(&pattern).map_err(|error| error.in_field(field))?;
    }
    for (index, rule) in args.iter().enumerate() {
        if rule.op == CompareOp::Matches {
            if let Some(pattern) = static_text(&rule.value, params) {
                matching::compile_regex(&pattern).map_err(|error| error.in_field(Field::nth("rule_value", index + 1)))?;
            }
        }
    }
    Ok(())
}

/// A literal regex compiles and literal hex is hex.
fn check_udp_pattern(mode: UdpMode, pattern: &str, field: Field, params: &BTreeMap<String, String>) -> EngineResult<()> {
    match (mode, static_text(pattern, params)) {
        (UdpMode::Regex, Some(text)) => matching::compile_regex(&text).map(|_| ()),
        (UdpMode::Hex, Some(text)) => matching::parse_hex(&text).map(|_| ()),
        _ => Ok(()),
    }
    .map_err(|error| error.in_field(field))
}

/// Template checks before a run (see "Static checks" in the design): syntax,
/// literal regular expressions and JSON paths, variable names, and that every
/// name is a parameter or a variable set on every path before its use.
/// `order` is the topological order validation computed, `loops` its loops: a
/// Loop's exit condition, and what follows Done or Limit, also know what every
/// iteration of the body set.
pub fn check_values(doc: &Experiment, order: &[String], loops: &LoopShape, params: &BTreeMap<String, String>) -> EngineResult<()> {
    let by_id: HashMap<&str, &Node> = doc.nodes.iter().map(|node| (node.id.as_str(), node)).collect();
    let mut producers = BTreeSet::new();
    for node in &doc.nodes {
        let at = |error: EngineError| error.at(&node.id);
        for (field, text) in text_fields(&node.kind) {
            template::parse(&text).map_err(|error| at(error.in_field(field)))?;
        }
        if let Some((variable, _)) = written_var(&node.kind) {
            check_variable_name(variable).map_err(|error| at(error.in_field(Field::new("variable"))))?;
            if params.contains_key(variable) {
                return Err(at(EngineError::new("variable.shadows_param").with("name", variable).in_field(Field::new("variable"))));
            }
            producers.insert(variable.to_string());
        }
        check_literals(node, params).map_err(at)?;
    }

    // Variables known on each wire: what was known after the source node, plus
    // what the source writes on that output.
    let mut incoming: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    for edge in doc.edges.iter().filter(|edge| !loops.is_back(edge)) {
        incoming.entry(edge.to.as_str()).or_default().push((edge.from.as_str(), edge.port.as_str()));
    }
    let mut known_after: HashMap<&str, BTreeSet<String>> = HashMap::new();
    // What is known on a wire out of `from`: what was known after it, what it
    // writes on that output — and, out of a Loop's Done or Limit, what every
    // iteration of its body set (the order puts the body first).
    let wire_known = |known_after: &HashMap<&str, BTreeSet<String>>, from: &str, port: &str| -> Option<BTreeSet<String>> {
        let mut set = known_after.get(from)?.clone();
        if let Some((variable, on)) = written_var(&by_id[from].kind) {
            if on == port {
                set.insert(variable.to_string());
            }
        }
        if matches!(by_id[from].kind, NodeKind::Loop { .. }) && port != "body" {
            set.extend(body_known(loops, known_after, &by_id, from));
        }
        Some(set)
    };
    for id in order {
        let node = by_id[id.as_str()];
        let inputs: Vec<BTreeSet<String>> = incoming
            .get(id.as_str())
            .map(|list| list.iter().filter_map(|(from, port)| wire_known(&known_after, from, port)).collect())
            .unwrap_or_default();
        // Every branch into a Join has run; any other merge took only one path.
        let known: BTreeSet<String> = match (&node.kind, inputs.split_first()) {
            (_, None) => BTreeSet::new(),
            (NodeKind::Join, Some(_)) => inputs.iter().flat_map(|set| set.iter().cloned()).collect(),
            (_, Some((first, rest))) => {
                first.iter().filter(|name| rest.iter().all(|set| set.contains(*name))).cloned().collect()
            }
        };
        // A Loop's exit condition is read when the body comes back: checked below.
        if !matches!(node.kind, NodeKind::Loop { .. }) {
            check_refs(&node.kind, &known, &producers, params).map_err(|error| error.at(id))?;
        }
        known_after.insert(id.as_str(), known);
    }
    for node in doc.nodes.iter().filter(|node| matches!(node.kind, NodeKind::Loop { .. })) {
        let Some(entry) = known_after.get(node.id.as_str()) else { continue };
        let mut known = entry.clone();
        known.extend(body_known(loops, &known_after, &by_id, &node.id));
        check_refs(&node.kind, &known, &producers, params).map_err(|error| error.at(&node.id))?;
    }
    Ok(())
}

/// Every name a node's templates use is a parameter, a known variable or a valid secret.
fn check_refs(kind: &NodeKind, known: &BTreeSet<String>, producers: &BTreeSet<String>, params: &BTreeMap<String, String>) -> EngineResult<()> {
    for (field, text) in text_fields(kind) {
        let Ok(parsed) = template::parse(&text) else { continue };
        for reference in parsed.refs() {
            let problem = match &reference {
                Ref::Param(name) if !params.contains_key(name) => Some(EngineError::new("param.unknown").with("name", name)),
                Ref::Bare(name) if params.contains_key(name) => None,
                Ref::Var(name) | Ref::Bare(name) if !known.contains(name) => Some(if producers.contains(name) {
                    EngineError::new("name.not_on_every_path").with("name", name)
                } else {
                    EngineError::new("name.unknown").with("name", name)
                }),
                Ref::Secret(name) => secrets::check_name(name).err(),
                _ => None,
            };
            if let Some(problem) = problem {
                return Err(problem.in_field(field));
            }
        }
    }
    Ok(())
}

/// Variables every iteration of a Loop's body has set when it comes back: on
/// each wire back, what was known after its source and what that output writes.
fn body_known(loops: &LoopShape, known_after: &HashMap<&str, BTreeSet<String>>, by_id: &HashMap<&str, &Node>, loop_id: &str) -> BTreeSet<String> {
    let mut sets = loops.back_edges.iter().filter(|(_, _, to)| *to == loop_id).filter_map(|(from, port, _)| {
        let mut set = known_after.get(from)?.clone();
        if let Some((variable, on)) = written_var(&by_id[from].kind) {
            if on == *port {
                set.insert(variable.to_string());
            }
        }
        Some(set)
    });
    let Some(first) = sets.next() else { return BTreeSet::new() };
    sets.fold(first, |all, set| all.intersection(&set).cloned().collect())
}

/// Secret names a node kind reads, in field order.
pub fn node_secrets(kind: &NodeKind) -> Vec<String> {
    let mut names = Vec::new();
    for (_, text) in text_fields(kind) {
        let Ok(parsed) = template::parse(&text) else { continue };
        for reference in parsed.refs() {
            if let Ref::Secret(name) = reference {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }
    names
}

/// Load the secrets `nodes` read. A missing one is reported at the first node
/// and field that use it, like any other validation problem.
pub fn load_secrets<'a>(nodes: impl IntoIterator<Item = &'a Node>, store: &dyn SecretStore) -> EngineResult<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for node in nodes {
        for (field, text) in text_fields(&node.kind) {
            let Ok(parsed) = template::parse(&text) else { continue };
            for reference in parsed.refs() {
                let Ref::Secret(name) = reference else { continue };
                if values.contains_key(&name) {
                    continue;
                }
                let loaded = secrets::load(store, std::slice::from_ref(&name)).map_err(|error| error.in_field(field.clone()).at(&node.id))?;
                values.extend(loaded);
            }
        }
    }
    Ok(values)
}

/// A response as it may leave the engine: secret values masked.
pub fn mask_response(response: &HttpResponse, values: &[String]) -> HttpResponse {
    if values.is_empty() {
        return response.clone();
    }
    HttpResponse {
        headers: response.headers.iter().map(|(name, value)| (name.clone(), secrets::mask(value, values))).collect(),
        body: secrets::mask(&response.body, values),
        error: response.error.as_ref().map(|error| secrets::mask(error, values)),
        ..response.clone()
    }
}

pub fn find_node<'a>(doc: &'a Experiment, node_id: &str) -> EngineResult<&'a Node> {
    doc.nodes.iter().find(|node| node.id == node_id).ok_or_else(|| EngineError::new("node.not_found").with("id", node_id))
}

/// A node as it would be sent now: templates resolved with the active
/// profile's values and the variable values the editor knows. Names without a
/// value stay as written and are returned, so the preview can point at them.
/// `stored` lists the secrets that exist; the preview shows them only as masks.
pub fn resolve_node(doc: &Experiment, node_id: &str, vars: &BTreeMap<String, Value>, stored: &[String]) -> EngineResult<(Node, Vec<String>)> {
    let node = find_node(doc, node_id)?;
    let params = effective_params(doc, doc.profile.as_deref(), &BTreeMap::new());
    let masks: BTreeMap<String, String> = stored.iter().map(|name| (name.clone(), MASK.to_string())).collect();
    let scope = Scope {
        params: &params,
        vars,
        secrets: &masks,
        run_id: 0,
        seed: doc.seed.unwrap_or(0),
        node_id,
        count: 1,
        now_ms: super::jobs::now_ms(),
    };
    let mut renderer = Renderer::lenient(scope);
    let kind = render_kind(&node.kind, &mut renderer).map_err(|error| error.at(node_id))?;
    Ok((Node { kind, ..node.clone() }, renderer.missing))
}

/// What the Extract nodes right after `node_id` would set from `response`:
/// *Send now* on a request makes its values known without a full run. The walk
/// passes checks and other data steps and stops at the next network action.
pub fn extract_after(doc: &Experiment, node_id: &str, response: &HttpResponse) -> BTreeMap<String, Value> {
    let mut values = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut stack: Vec<&str> = doc.edges.iter().filter(|edge| edge.from == node_id).map(|edge| edge.to.as_str()).collect();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        let Some(node) = doc.nodes.iter().find(|node| node.id == id) else { continue };
        match &node.kind {
            NodeKind::Extract { variable, from, expr } => {
                if let Ok(value) = extract(*from, expr, Some(response)) {
                    values.insert(variable.clone(), value);
                }
            }
            NodeKind::AssertStatus { .. }
            | NodeKind::AssertBody { .. }
            | NodeKind::AssertHeader { .. }
            | NodeKind::AssertLatency { .. }
            | NodeKind::AssertValue { .. }
            | NodeKind::BranchStatus { .. }
            | NodeKind::BranchValue { .. }
            | NodeKind::Log { .. }
            | NodeKind::Delay { .. } => {}
            _ => continue,
        }
        stack.extend(doc.edges.iter().filter(|edge| edge.from == id).map(|edge| edge.to.as_str()));
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn response(body: &str) -> HttpResponse {
        HttpResponse {
            ok: true,
            status: 201,
            status_text: "Created".into(),
            latency_ms: 12.0,
            headers: vec![("X-Session".into(), "s-1".into())],
            body: body.into(),
            body_bytes: body.len(),
            truncated: false,
            error: None,
            cause: None,
        }
    }

    fn code<T: std::fmt::Debug>(result: EngineResult<T>) -> String {
        result.unwrap_err().into_code()
    }

    #[test]
    fn extraction_reads_each_source_and_says_what_is_missing() {
        let reply = response(r#"{"token":"abc","items":[{"id":7}]}"#);
        assert_eq!(extract(ExtractFrom::Json, "$.token", Some(&reply)).unwrap(), json!("abc"));
        assert_eq!(extract(ExtractFrom::Json, "items[0]", Some(&reply)).unwrap(), json!({ "id": 7 }));
        let missing = extract(ExtractFrom::Json, "$.nope", Some(&reply)).unwrap_err();
        assert_eq!((missing.code.as_str(), missing.params["path"].as_str()), ("extract.json_missing", "$.nope"));
        assert_eq!(extract(ExtractFrom::Header, "x-session", Some(&reply)).unwrap(), json!("s-1"));
        assert_eq!(code(extract(ExtractFrom::Header, "x-other", Some(&reply))), "extract.header_missing");
        assert_eq!(extract(ExtractFrom::Status, "", Some(&reply)).unwrap(), json!(201));
        assert_eq!(extract(ExtractFrom::Regex, r#""token":"(\w+)""#, Some(&reply)).unwrap(), json!("abc"));
        assert_eq!(extract(ExtractFrom::Regex, r"\d+", Some(&reply)).unwrap(), json!("7"));
        assert_eq!(code(extract(ExtractFrom::Regex, "zzz", Some(&reply))), "extract.no_match");
        let not_json = extract(ExtractFrom::Json, "$", Some(&response("<html>"))).unwrap_err();
        assert!(not_json.is("extract.not_json") && not_json.detail.is_some(), "the parser's reason is kept");
        assert_eq!(code(extract(ExtractFrom::Status, "", None)), "check.no_response");
        assert_eq!(code(extract(ExtractFrom::Regex, "(", Some(&reply))), "regex.invalid");
        let mut cut = response(r#"{"token":"#);
        cut.truncated = true;
        for from in [ExtractFrom::Json, ExtractFrom::Body] {
            assert_eq!(code(extract(from, "$.token", Some(&cut))), "extract.truncated");
        }
        assert_eq!(code(extract(ExtractFrom::Regex, "zzz", Some(&cut))), "extract.truncated");
    }

    #[test]
    fn comparisons_are_numeric_when_they_can_be_and_explicit_when_they_cannot() {
        assert!(compare("200", CompareOp::Eq, "200.0").unwrap().0);
        assert!(!compare("abc", CompareOp::Eq, "ABC").unwrap().0);
        assert!(compare("abc", CompareOp::Ne, "ABC").unwrap().0);
        assert!(compare("9", CompareOp::Lt, "10").unwrap().0, "numeric, not text order");
        assert!(compare("10", CompareOp::Ge, "10").unwrap().0);
        assert_eq!(code(compare("abc", CompareOp::Gt, "1")), "compare.not_numbers");
        assert!(compare("hello world", CompareOp::Contains, "lo w").unwrap().0);
        assert!(compare("ID-42", CompareOp::Matches, r"^ID-\d+$").unwrap().0);
        assert_eq!(code(compare("x", CompareOp::Matches, "(")), "regex.invalid");
        assert!(compare("  ", CompareOp::Empty, "ignored").unwrap().0);
        assert!(compare("t", CompareOp::NotEmpty, "").unwrap().0);
        assert_eq!(compare("401", CompareOp::Eq, "200").unwrap().1.text(), "401 = 200");
        let (_, contains) = compare("a", CompareOp::Contains, "b").unwrap();
        assert_eq!(contains.text(), "“a” contains “b”");
        assert_eq!(contains.params(), json!({ "value": "“a”", "op": "contains", "expected": "“b”" }));
        let failed = compare("", CompareOp::NotEmpty, "x").unwrap().1.failed();
        assert_eq!((failed.code.as_str(), failed.params["op"].as_str(), failed.params["expected"].as_str()), ("check.value_failed", "not_empty", ""));
    }

    fn doc_with(params: &[(&str, &str)], profiles: serde_json::Value, profile: Option<&str>) -> Experiment {
        serde_json::from_value(json!({
            "version": 6, "name": "p",
            "params": params.iter().map(|(name, value)| json!({ "name": name, "value": value })).collect::<Vec<_>>(),
            "profiles": profiles, "profile": profile, "seed": null,
            "nodes": [{ "id": "start", "type": "start", "x": 0, "y": 0 }, { "id": "end", "type": "end", "x": 0, "y": 0 }],
            "edges": [{ "from": "start", "to": "end" }]
        }))
        .unwrap()
    }

    #[test]
    fn parameters_follow_the_naming_rules_before_a_run_but_drafts_still_save() {
        let named = |names: &[&str]| doc_with(&names.iter().map(|name| (*name, "")).collect::<Vec<_>>(), json!([]), None);
        assert!(check_names(&named(&["api", "_host2"])).is_ok());
        for (names, expected) in [
            (&["api", "api"][..], "param.duplicate"),
            (&["2x"][..], "param.name_invalid"),
            (&["uuid"][..], "param.name_reserved"),
            (&[""][..], "param.name_invalid"),
        ] {
            let doc = named(names);
            assert_eq!(code(check_names(&doc)), expected, "{names:?}");
            assert!(check_sizes(&doc.params, &doc.profiles, doc.seed).is_ok(), "a draft with {names:?} must still save");
        }
        assert_eq!(code(check_sizes(&[], &[], Some(MAX_SEED + 1))), "seed.range");
        let big = vec![Param { name: "big".into(), value: "x".repeat(MAX_PARAM_BYTES + 1) }];
        assert_eq!(check_sizes(&big, &[], None).unwrap_err().params["name"], "big");
    }

    #[test]
    fn profiles_override_defaults_and_runs_override_profiles() {
        let doc = doc_with(
            &[("api", "http://local"), ("device", "127.0.0.1:9000")],
            json!([{ "name": "Stage", "values": { "api": "http://stage" } }, { "name": "Venue", "values": { "api": "http://venue", "device": "10.0.0.5:9000" } }]),
            Some("Stage"),
        );
        let none = BTreeMap::new();
        assert_eq!(effective_params(&doc, None, &none)["api"], "http://local");
        let stage = effective_params(&doc, Some("Stage"), &none);
        assert_eq!((stage["api"].as_str(), stage["device"].as_str()), ("http://stage", "127.0.0.1:9000"));
        let overrides = BTreeMap::from([("device".to_string(), "10.9.9.9:9000".to_string()), ("ghost".to_string(), "x".to_string())]);
        let run = effective_params(&doc, Some("Venue"), &overrides);
        assert_eq!((run["api"].as_str(), run["device"].as_str()), ("http://venue", "10.9.9.9:9000"));
        assert!(!run.contains_key("ghost"), "an override cannot invent a parameter");
        assert!(check_names(&doc).is_ok());

        let unknown_key = doc_with(&[("api", "")], json!([{ "name": "Stage", "values": { "gone": "x" } }]), None);
        let unknown = check_names(&unknown_key).unwrap_err();
        assert_eq!((unknown.code.as_str(), unknown.params["profile"].as_str(), unknown.params["name"].as_str()), ("profile.unknown_param", "Stage", "gone"));
        assert_eq!(code(check_names(&doc_with(&[("api", "")], json!([]), Some("Stage")))), "profile.active_missing");
        assert_eq!(code(check_names(&doc_with(&[("api", "")], json!([{ "name": "A" }, { "name": " A " }]), None))), "profile.duplicate");
        assert_eq!(code(check_names(&doc_with(&[("api", "")], json!([{ "name": "  " }]), None))), "profile.name_invalid");
    }

    #[test]
    fn secrets_must_be_stored_before_a_run_and_never_leave_in_responses() {
        let doc: Experiment = serde_json::from_value(json!({
            "version": 6, "name": "s", "params": [], "profiles": [], "profile": null, "seed": null,
            "nodes": [
                { "id": "start", "type": "start", "x": 0, "y": 0 },
                { "id": "login", "type": "http", "x": 0, "y": 0, "request": { "method": "GET", "url": "http://127.0.0.1/",
                  "headers": [["Authorization", "Bearer {{secret.API_TOKEN}}"]], "body": "{{secret.API_TOKEN}}", "timeout_ms": 1000 } },
                { "id": "end", "type": "end", "x": 0, "y": 0 }
            ],
            "edges": [{ "from": "start", "to": "login" }, { "from": "login", "to": "end" }]
        }))
        .unwrap();
        assert_eq!(node_secrets(&doc.nodes[1].kind), ["API_TOKEN"]);
        let store = secrets::MemoryStore::default();
        let missing = load_secrets(&doc.nodes, &store).unwrap_err();
        assert_eq!(missing.code, "secret.missing");
        assert_eq!((missing.node.as_deref(), missing.field.clone()), (Some("login"), Some(Field::nth("header_value", 1))));
        secrets::set(&store, "API_TOKEN", "t0k3n").unwrap();
        assert_eq!(load_secrets(&doc.nodes, &store).unwrap()["API_TOKEN"], "t0k3n");

        // The preview sees a mask, never the value.
        let (node, unresolved) = resolve_node(&doc, "login", &BTreeMap::new(), &["API_TOKEN".to_string()]).unwrap();
        assert!(unresolved.is_empty());
        let NodeKind::Http { request } = node.kind else { panic!() };
        assert_eq!(request.headers[0].1, format!("Bearer {MASK}"));
        let (_, unresolved) = resolve_node(&doc, "login", &BTreeMap::new(), &[]).unwrap();
        assert_eq!(unresolved, ["secret.API_TOKEN"]);
        assert_eq!(code(resolve_node(&doc, "ghost", &BTreeMap::new(), &[])), "node.not_found");

        let mut reply = response(r#"{"echo":"t0k3n"}"#);
        reply.headers.push(("X-Echo".into(), "Bearer t0k3n".into()));
        let masked = mask_response(&reply, &["t0k3n".to_string()]);
        assert_eq!(masked.body, format!(r#"{{"echo":"{MASK}"}}"#));
        assert_eq!(masked.headers[1].1, format!("Bearer {MASK}"));
        assert_eq!(masked.status, 201);
    }

    #[test]
    fn rendering_touches_only_text_fields_and_names_the_field_that_failed() {
        let kind: NodeKind = serde_json::from_value(json!({
            "type": "osc", "target": "{{host}}:9000", "address": "/cue/{{n}}",
            "args": [{ "type": "str", "value": "{{n}}" }, { "type": "int", "value": 5 }, { "type": "str", "value": "{{gone}}" }]
        }))
        .unwrap();
        let params = BTreeMap::from([("host".to_string(), "127.0.0.1".to_string())]);
        let vars = BTreeMap::from([("n".to_string(), json!(3))]);
        let secrets = BTreeMap::new();
        let scope = || Scope { params: &params, vars: &vars, secrets: &secrets, run_id: 1, seed: 1, node_id: "osc", count: 1, now_ms: 0 };
        let failed = render_kind(&kind, &mut Renderer::new(scope())).unwrap_err();
        assert_eq!((failed.code.as_str(), failed.field.clone()), ("name.no_value", Some(Field::nth("argument", 3))));
        let NodeKind::Osc { target, address, mut args, .. } = kind else { panic!() };
        args.pop();
        let kind = NodeKind::Osc { target, address, args, reply: None };
        let rendered = render_kind(&kind, &mut Renderer::new(scope())).unwrap();
        assert_eq!(
            serde_json::to_value(rendered).unwrap(),
            json!({ "type": "osc", "target": "127.0.0.1:9000", "address": "/cue/3",
                    "args": [{ "type": "str", "value": "3" }, { "type": "int", "value": 5 }] })
        );
        assert_eq!(static_text("{{host}}:9000", &params).as_deref(), Some("127.0.0.1:9000"));
        assert_eq!(static_text("{{n}}", &params), None);
    }
}
