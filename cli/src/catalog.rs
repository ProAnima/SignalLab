//! What an experiment is made of, for someone writing one without the editor —
//! an LLM through `signallab mcp` (`describe_nodes`), or a person reading
//! `signallab nodes`: every kind of node with its fields and an example, its
//! outputs as the engine computes them, how a document is put together, and
//! the template language. The labels and descriptions are the interface's own.

use serde_json::{json, Value};
use signal_lab_engine::experiment::Node;

use crate::i18n::Texts;

/// One kind of node: its fields (name → what it is) and an example the engine accepts.
struct Kind {
    kind: &'static str,
    fields: &'static [(&'static str, &'static str)],
    example: fn() -> Value,
}

const KINDS: &[Kind] = &[
    Kind { kind: "start", fields: &[], example: || json!({}) },
    Kind { kind: "end", fields: &[], example: || json!({}) },
    Kind { kind: "fork", fields: &[], example: || json!({}) },
    Kind { kind: "join", fields: &[], example: || json!({}) },
    Kind { kind: "log", fields: &[("message", "text, templated")], example: || json!({ "message": "device {{device}} ready" }) },
    Kind { kind: "delay", fields: &[("ms", "milliseconds")], example: || json!({ "ms": 500 }) },
    Kind {
        kind: "http",
        fields: &[("request.method", "GET, POST, PUT, PATCH, DELETE, HEAD"), ("request.url", "http(s) URL, templated"), ("request.headers", "[[name, value], …], templated"), ("request.body", "text or null, templated"), ("request.timeout_ms", "milliseconds")],
        example: || json!({ "request": { "method": "POST", "url": "{{api}}/cue", "headers": [["Content-Type", "application/json"]], "body": "{\"cue\": 1}", "timeout_ms": 5000 } }),
    },
    Kind { kind: "assert_status", fields: &[("status", "the HTTP status the last response must have")], example: || json!({ "status": 200 }) },
    Kind { kind: "assert_body", fields: &[("contains", "text the last response body must contain, templated")], example: || json!({ "contains": "ready" }) },
    Kind { kind: "assert_header", fields: &[("name", "header name"), ("contains", "text its value must contain")], example: || json!({ "name": "Content-Type", "contains": "json" }) },
    Kind { kind: "assert_latency", fields: &[("max_ms", "the last response's time limit")], example: || json!({ "max_ms": 250 }) },
    Kind { kind: "branch_status", fields: &[("status", "yes when the last response has it, else no")], example: || json!({ "status": 200 }) },
    Kind {
        kind: "extract",
        fields: &[("variable", "name to store the value under ({{name}} later)"), ("from", "json | header | status | body | regex — of the last HTTP response"), ("expr", "$.path for json, header name, or a regex with one group")],
        example: || json!({ "variable": "token", "from": "json", "expr": "$.data.token" }),
    },
    Kind {
        kind: "assert_value",
        fields: &[("value", "templated, e.g. {{token}}"), ("op", "eq ne lt le gt ge contains matches empty not_empty"), ("expected", "templated")],
        example: || json!({ "value": "{{state}}", "op": "eq", "expected": "ready" }),
    },
    Kind {
        kind: "branch_value",
        fields: &[("value", "templated"), ("op", "as assert_value"), ("expected", "templated")],
        example: || json!({ "value": "{{reply.args[0]}}", "op": "eq", "expected": "ok" }),
    },
    Kind {
        kind: "osc",
        fields: &[
            ("target", "host:port, templated"),
            ("address", "/osc/address, templated"),
            ("args", "[{type: int|float|str|long|double|bool|blob|nil, value}]; str values templated"),
            ("reply", "optional: wait for the answer on this node — {bind: \"0.0.0.0:0\", address pattern, args rules, timeout_ms, variable}"),
        ],
        example: || json!({ "target": "{{device}}", "address": "/fader/1", "args": [{ "type": "float", "value": 0.75 }] }),
    },
    Kind {
        kind: "udp",
        fields: &[("target", "host:port (several: comma-separated), templated"), ("text", "payload, templated"), ("reply", "optional: {bind, mode: any|contains|regex|hex, pattern, timeout_ms, variable}")],
        example: || json!({ "target": "{{device}}", "text": "PING {{run.id}}", "reply": { "bind": "0.0.0.0:0", "mode": "contains", "pattern": "PONG", "timeout_ms": 1000, "variable": "pong" } }),
    },
    Kind {
        kind: "tcp",
        fields: &[("host", "host name or IP, templated"), ("port", "number"), ("payload", "text sent once connected, templated"), ("timeout_ms", "milliseconds")],
        example: || json!({ "host": "127.0.0.1", "port": 5000, "payload": "GO\r\n", "timeout_ms": 2000 }),
    },
    Kind {
        kind: "mqtt",
        fields: &[("host", "broker, templated"), ("port", "number, usually 1883"), ("topic", "templated"), ("payload", "templated"), ("qos", "0, 1 or 2"), ("retain", "true to keep it as the topic's value")],
        example: || json!({ "host": "{{broker}}", "port": 1883, "topic": "lab/light/1/set", "payload": "on", "qos": 1, "retain": false }),
    },
    Kind {
        kind: "wait_osc",
        fields: &[("bind", "IP:port to listen on (opened when the run starts)"), ("address", "OSC address pattern: * ? [a-z] {a,b}"), ("args", "[{index, op, value}] rules"), ("timeout_ms", "milliseconds"), ("variable", "the message as {{reply.address}}, {{reply.args[0]}}, {{reply.from}}")],
        example: || json!({ "bind": "0.0.0.0:9001", "address": "/status", "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 5000, "variable": "reply" }),
    },
    Kind {
        kind: "wait_udp",
        fields: &[("bind", "IP:port to listen on"), ("mode", "any | contains | regex | hex"), ("pattern", "what the payload must match"), ("timeout_ms", "milliseconds"), ("variable", "the datagram as {{reply.text}}, {{reply.from}}")],
        example: || json!({ "bind": "0.0.0.0:9002", "mode": "contains", "pattern": "READY", "timeout_ms": 5000, "variable": "reply" }),
    },
    Kind {
        kind: "wait_mqtt",
        fields: &[("host", "broker (a parameter at most: subscribed before the first step)"), ("port", "number"), ("topic", "filter: + one level, # the rest"), ("mode", "any | contains | regex | hex"), ("pattern", "what the payload must match"), ("timeout_ms", "milliseconds"), ("variable", "the message as {{reply.topic}}, {{reply.text}}")],
        example: || json!({ "host": "{{broker}}", "port": 1883, "topic": "lab/+/state", "mode": "contains", "pattern": "on", "timeout_ms": 5000, "variable": "reply" }),
    },
    Kind {
        kind: "loop",
        fields: &[("max", "iterations at most"), ("until", "optional exit condition {value, op, expected}, read after each iteration")],
        example: || json!({ "max": 10, "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }),
    },
];

/// Every kind of node the catalogue describes (a test keeps it equal to the engine's).
#[cfg(test)]
pub fn kinds() -> Vec<&'static str> {
    KINDS.iter().map(|kind| kind.kind).collect()
}

/// The example of `kind` as a whole node, as a document holds it.
pub fn example_node(kind: &str) -> Option<Value> {
    let found = KINDS.iter().find(|candidate| candidate.kind == kind)?;
    let mut node = (found.example)();
    node["type"] = kind.into();
    node["id"] = kind.into();
    node["x"] = 0.into();
    node["y"] = 0.into();
    Some(node)
}

/// The outputs of a node, as the engine computes them: required, then optional.
fn outputs(example: &Value) -> (Vec<&'static str>, Vec<&'static str>) {
    match serde_json::from_value::<Node>(example.clone()) {
        Ok(node) => {
            let outputs = node.kind.outputs();
            (outputs.required.to_vec(), outputs.optional.to_vec())
        }
        Err(_) => (Vec::new(), Vec::new()),
    }
}

/// The whole catalogue, in the reader's language where the interface has words.
pub fn describe(texts: &Texts) -> Value {
    let nodes: Vec<Value> = KINDS
        .iter()
        .map(|kind| {
            let example = example_node(kind.kind).unwrap_or_default();
            let (required, optional) = outputs(&example);
            let fields: serde_json::Map<String, Value> = kind.fields.iter().map(|(name, what)| (name.to_string(), Value::from(*what))).collect();
            json!({
                "type": kind.kind,
                "label": texts.plain(&format!("exp.node.{}", kind.kind)),
                "description": texts.plain(&format!("exp.description.{}", kind.kind)),
                "fields": fields,
                "outputs": required,
                "optional_outputs": optional,
                "example": example,
            })
        })
        .collect();
    json!({
        "document": {
            "shape": "{ version: 5, name, params: [{name, value}], profiles: [{name, values: {param: value}}], nodes: [...], edges: [{from, to, port}] }",
            "rules": [
                "Exactly one start and at least one end; every required output of a node is wired; ids are unique.",
                "A node is {id, type, x, y, ...its fields}; x and y place it on the canvas (any numbers).",
                "An edge leaves a node by one of its outputs (port, default \"next\") and enters another node.",
                "Several edges out of one output run in parallel; a join waits for every edge into it.",
                "A loop's body (from its body output) must lead back to the loop; the loop continues by done, or limit when the iterations ran out.",
                "Waits listen from the start of the run, so a fast answer is not missed; an action followed by a wait is how a reply is checked.",
                "retry: {attempts, delay_ms, backoff: fixed|exponential} on an action or wait tries it again; repeat: {until: count|duration, count, duration_ms, interval_ms, jitter_ms} on an action sends it again.",
            ],
        },
        "templates": {
            "syntax": "{{name}} in text fields; \\{{ is a literal {{",
            "values": [
                "{{param}} — a parameter of the document (or a run's override)",
                "{{variable}} — what an extract or a wait stored; JSON paths go further: {{reply.args[0]}}, {{token.data.id}}",
                "{{secret.NAME}} — a secret, never shown in results",
                "{{run.id}}, {{node.id}}, {{now}} (ms), {{now.iso}}, {{uuid}}, {{counter}} (the send number of a repeat)",
                "{{random_int(1, 100)}} — from the run's seed, so a seed repeats a run",
            ],
        },
        "nodes": nodes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Lang;

    #[test]
    fn every_example_is_a_node_the_engine_accepts_with_its_outputs() {
        for kind in kinds() {
            let example = example_node(kind).unwrap();
            let node: Node = serde_json::from_value(example.clone()).unwrap_or_else(|error| panic!("{kind}: {error}"));
            let back = serde_json::to_value(&node).unwrap();
            for (field, value) in example.as_object().unwrap() {
                if !matches!(field.as_str(), "x" | "y") {
                    assert_eq!(back.get(field), Some(value), "{kind}.{field} is a field the engine keeps");
                }
            }
        }
        let catalogue = describe(&Texts::new(Lang::En));
        let fork = catalogue["nodes"].as_array().unwrap().iter().find(|node| node["type"] == "fork").unwrap();
        assert_eq!(fork["outputs"], json!(["branch1", "branch2"]));
        let wait = catalogue["nodes"].as_array().unwrap().iter().find(|node| node["type"] == "wait_osc").unwrap();
        assert_eq!((wait["outputs"].clone(), wait["optional_outputs"].clone()), (json!(["matched"]), json!(["timeout"])));
        assert_eq!(wait["label"], "Wait for OSC");
    }

    #[test]
    fn the_catalogue_names_every_kind_of_node_the_engine_has() {
        let engine = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/src/experiment.rs")).unwrap().replace("\r\n", "\n");
        let start = engine.find("pub enum NodeKind {").unwrap();
        let body = &engine[start..start + engine[start..].find("\n}\n").unwrap()];
        let variants: Vec<String> = body
            .lines()
            .filter(|line| line.starts_with("    ") && !line.starts_with("     "))
            .filter_map(|line| line.trim().split([' ', ',', '{']).next().filter(|name| name.starts_with(|c: char| c.is_ascii_uppercase())).map(str::to_string))
            .collect();
        // CamelCase variant → the snake_case type a document writes.
        let snake = |name: &str| name.chars().enumerate().fold(String::new(), |mut out, (at, char)| {
            if char.is_ascii_uppercase() && at > 0 {
                out.push('_');
            }
            out.push(char.to_ascii_lowercase());
            out
        });
        let mut engine_kinds: Vec<String> = variants.iter().map(|name| snake(name)).collect();
        let mut described: Vec<String> = kinds().iter().map(|kind| kind.to_string()).collect();
        engine_kinds.sort();
        described.sort();
        assert_eq!(described, engine_kinds, "a new kind of node needs its entry in cli/src/catalog.rs");
    }
}
