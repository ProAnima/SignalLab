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
        fields: &[("request.method", "GET, POST, PUT, PATCH, DELETE, HEAD"), ("request.url", "http(s) URL, templated"), ("request.headers", "[[name, value], …], templated"), ("request.body", "text or null, templated"), ("request.timeout_ms", "milliseconds"),
                 ("request.auth", "optional: {scheme: basic|digest, username, password} or {scheme: bearer, token}, templated ({{secret.NAME}}); Digest answers the server's 401 challenge"),
                 ("load", "optional: send the request on a load profile, many at once, measured and judged by thresholds — see \"load\"")],
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
    Kind {
        kind: "emulator",
        fields: &[("emulator", "an emulator document (see \"emulators\"): it opens before the first step and answers until the run ends; the flow passes at once")],
        // Every field written out, as the engine keeps it, so a reader sees them all.
        example: || {
            let emulator: signal_lab_engine::emulator::Emulator = serde_json::from_value(json!({ "name": "Orders API", "bind": "127.0.0.1:18080", "protocol": "http", "routes": [
                { "method": "GET", "path": "/orders/:id", "order": "sequence", "responses": [{ "status": 503 }, { "status": 200, "body": "{\"id\":\"{{request.params.id}}\"}" }] }
            ] }))
            .expect("the example is an emulator");
            json!({ "emulator": emulator })
        },
    },
    Kind {
        kind: "wait_http",
        fields: &[
            ("bind", "IP:port: the run's HTTP emulator there, or a listener of the run's own that answers 204"),
            ("method", "a method or ANY"),
            ("path", "/hooks/:name, a final /* takes the rest; templated"),
            ("when", "[{on: header|query|body|json, name, op, value}], every one must hold; templated"),
            ("timeout_ms", "milliseconds"),
            ("variable", "the request as {{request.method}}, .path, .query, .headers, .body, .json, .params, .from, .ms"),
        ],
        example: || json!({ "bind": "127.0.0.1:18081", "method": "POST", "path": "/hooks/:name", "when": [{ "on": "json", "name": "$.event", "op": "eq", "value": "deploy" }], "timeout_ms": 5000, "variable": "request" }),
    },
    Kind {
        kind: "impairment",
        fields: &[
            ("listen", "IP:port the system under test sends to instead of the target; parameters only (opened before the first step)"),
            ("target", "IP:port it forwards to (replies come back the same way); parameters only"),
            ("profile", "the impairment from the first step on (see \"impairment profile\"); the flow passes at once"),
        ],
        // The profile written out whole, as the engine keeps it.
        example: || json!({ "listen": "127.0.0.1:9010", "target": "{{device}}", "profile": profile(json!({ "name": "lan", "latency_ms": 1, "jitter_ms": 1 })) }),
    },
    Kind {
        kind: "impairment_change",
        fields: &[
            ("relay", "the id of an impairment node of this document"),
            ("profile", "what it impairs with from now on; each phase is counted apart in the report"),
        ],
        example: || json!({ "relay": "relay", "profile": profile(json!({ "name": "offline", "offline": true })) }),
    },
    Kind {
        kind: "emulator_state",
        fields: &[
            ("emulator", "the id of an emulator node of this document"),
            ("down", "true takes it down until a later step brings it up (false)"),
            ("fault", "what HTTP meets while down: unavailable (503) | reset | timeout; TCP and MQTT drop connections, OSC and UDP answer nothing"),
        ],
        example: || json!({ "emulator": "api", "down": true, "fault": "unavailable" }),
    },
    Kind {
        kind: "ws_connect",
        fields: &[
            ("url", "ws:// or wss:// (templates: a token extracted earlier may be in it)"),
            ("headers", "[[name, value], …] sent with the upgrade"),
            ("protocols", "subprotocols to offer, in order of preference"),
            ("timeout_ms", "for the connection and the upgrade"),
        ],
        example: || json!({ "url": "ws://127.0.0.1:9001/chat", "headers": [["Authorization", "Bearer {{token}}"]], "protocols": ["chat.v1"], "timeout_ms": 5000 }),
    },
    Kind {
        kind: "ws_send",
        fields: &[("connection", "the id of a ws_connect node of this document"), ("text", "the message (templates)"), ("binary", "true: text is bytes written as hex, sent as a binary message")],
        example: || json!({ "connection": "socket", "text": "{\"type\":\"ping\",\"id\":\"{{uuid}}\"}", "binary": false }),
    },
    Kind {
        kind: "wait_ws",
        fields: &[
            ("connection", "the id of a ws_connect node; messages since it connected (or since the branch's last send) count"),
            ("mode", "any | contains | regex | hex"),
            ("pattern", "what the message must match"),
            ("timeout_ms", "milliseconds"),
            ("variable", "the message as {{reply.text}}, {{reply.json.field}} when it is JSON"),
        ],
        example: || json!({ "connection": "socket", "mode": "contains", "pattern": "pong", "timeout_ms": 3000, "variable": "reply" }),
    },
    Kind {
        kind: "ws_close",
        fields: &[("connection", "the id of a ws_connect node"), ("code", "1000, or 3000–4999 for an application's own"), ("reason", "at most 123 bytes")],
        example: || json!({ "connection": "socket", "code": 1000, "reason": "done" }),
    },
];

/// An impairment profile with every field, as a document holds it after the engine wrote it.
fn profile(value: Value) -> Value {
    let profile: signal_lab_engine::netsim::ImpairProfile = serde_json::from_value(value).expect("the example is a profile");
    serde_json::to_value(profile).unwrap_or_default()
}

/// What an emulator document holds, for a node's `emulator` and for `signallab emulate`.
fn emulators() -> Value {
    json!({
        "shape": "{ name, bind: \"IP:port\", protocol: http|osc|udp|tcp|mqtt, ...the protocol's fields, outage }",
        "rules": [
            "The first route or rule that matches answers; what none takes is counted (HTTP: 404, or fallback).",
            "Replies are templates read with what arrived as {{request…}}, and parameters; matching patterns may use parameters only.",
            "Bind 127.0.0.1 to answer this computer only, 0.0.0.0 to answer the network as well.",
            "Every exchange is counted per rule; a run's report has the counts of each Emulator node.",
            "outage: {up_ms, down_ms, fault: unavailable|reset|timeout} — answering for up_ms, then down for down_ms, from the start on: HTTP meets fault (503 with Retry-After by default), a TCP device and an MQTT broker drop and refuse connections, OSC and UDP answer nothing; counts.down says how many met it.",
        ],
        "protocols": {
            "http": {
                "fields": {
                    "routes": "[{method: ANY|GET|…, path: \"/users/:id\" (a final /* takes the rest), when: [{on: header|query|body|json, name, op, value}], order: sequence|cycle|random, responses: [...]}]",
                    "responses": "[{status, headers: [[name, value]], body, delay_ms, jitter_ms, fault: none|timeout|reset|malformed (the body stops halfway, still typed as JSON), weight}]: sequence answers them in turn and then the last (500, 500, 200 for retries); cycle starts over; random draws by weight from the seed",
                    "fallback": "a response for requests no route takes, or null for 404",
                },
                "request": "{{request.method}}, .path, .query.NAME, .headers.NAME (lower case), .body, .json.PATH, .params.NAME, .from",
            },
            "osc": {
                "fields": {
                    "rules": "[{address: pattern, args: [{index, op, value}], reply: {address, args: [{type: int|float|str|long|double|bool|blob|nil, value: template}]} or null, to: \"\" (the sender) or IP:port, delay_ms, jitter_ms}]",
                },
                "request": "{{request.address}}, {{request.args[0]}}, {{request.from}}",
            },
            "udp": {
                "fields": { "rules": "[{mode: any|contains|regex|hex, pattern, reply: {kind: text, text} | {kind: hex, hex} or null, to, delay_ms, jitter_ms}]" },
                "request": "{{request.text}}, {{request.match}} (a regex's first group), {{request.hex}}, {{request.bytes}}, {{request.from}}",
            },
            "tcp": {
                "fields": {
                    "delimiter": "lf | crlf | cr | none: splits what arrives into messages and ends every reply",
                    "greeting": "text sent as a client connects",
                    "rules": "[{mode, pattern, reply, close: true to hang up after the reply, delay_ms, jitter_ms}]",
                },
                "request": "as udp",
            },
            "mqtt": {
                "fields": {
                    "username": "when set, a client must connect with it and password (parameters only); else anyone may",
                    "password": "the password that goes with username",
                    "retained": "[{topic, payload, qos}] held from the start, as if published with retain",
                    "rules": "[{topic: filter (+, #), mode: any|contains|regex|hex, pattern, reply: {topic, payload, qos, retain} or null, delay_ms, jitter_ms}]: every message is routed to its subscribers as on any broker, then answered by the first rule it matches",
                },
                "request": "{{request.topic}}, {{request.levels[1]}}, {{request.payload}}, {{request.json.PATH}}, {{request.match}}, {{request.qos}}, {{request.retain}}, {{request.client}}, {{request.from}}",
                "notes": "MQTT 3.1.1, plain TCP, QoS 0/1/2, retained messages, last wills, clean sessions; a second connection with a client id takes over from the first",
            },
        },
        "templates": "{{counter}} is the rule's hit number; {{uuid}}, {{now}}, {{now.iso}}, {{random_int(a, b)}} as in experiments",
        "impairment profile": {
            "fields": "{name (a label: lan, wifi, 4g, satellite, intermittent, offline, or your own), latency_ms, jitter_ms, loss, duplicate, corrupt, reorder (probabilities 0..1), rate_kbps (0: no limit; packets queue, past a second they are dropped as throttled), burst_start (0..1) and burst_length (packets, mean): bursts of loss, offline (nothing gets through)}",
            "seed": "every decision draws from the run's seed: the same seed and the same traffic drop the same packets",
            "report": "impairments: per impairment node, counts in all and a phase per profile it had (from_ms, to_ms, received, forwarded, dropped, throttled, duplicated, corrupted, reordered)",
        },
        "example": (KINDS.iter().find(|kind| kind.kind == "emulator").map(|kind| (kind.example)()).unwrap_or_default())["emulator"].clone(),
    })
}

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
    let version = signal_lab_engine::experiment::VERSION;
    json!({
        "document": {
            "shape": format!("{{ version: {version}, name, params: [{{name, value}}], profiles: [{{name, values: {{param: value}}}}], nodes: [...], edges: [{{from, to, port}}] }}"),
            "rules": [
                "Exactly one start and at least one end; every required output of a node is wired; ids are unique.",
                "A node is {id, type, x, y, ...its fields}; x and y place it on the canvas (any numbers).",
                "An edge leaves a node by one of its outputs (port, default \"next\") and enters another node.",
                "Several edges out of one output run in parallel; a join waits for every edge into it.",
                "A loop's body (from its body output) must lead back to the loop; the loop continues by done, or limit when the iterations ran out.",
                "Waits listen from the start of the run, so a fast answer is not missed; an action followed by a wait is how a reply is checked.",
                "An emulator node plays a dependency for the whole run; wait_http then checks what the system under test sent it.",
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
        "load": load(),
        "emulators": emulators(),
    })
}

/// Load on an HTTP node: the profiles, the thresholds, what a step reports.
fn load() -> Value {
    let example = json!({
        "profile": { "shape": "ramp", "from": 10, "to": 200, "duration_ms": 60000 },
        "concurrency": 64,
        "thresholds": [{ "metric": "p95_ms", "op": "lt", "value": 300 }, { "metric": "error_rate", "op": "lt", "value": 1 }],
    });
    debug_assert!(serde_json::from_value::<signal_lab_engine::load::Load>(example.clone()).is_ok());
    json!({
        "shape": "\"load\": {profile, concurrency (1–512, default 32), thresholds: [{metric, op, value}]} on an http node",
        "profiles": {
            "constant": "{shape, rate, duration_ms}: rate requests/s throughout",
            "ramp": "{shape, from, to, duration_ms}: from → to linearly",
            "steps": "{shape, from, step, every_ms, steps}: from, from + step, … each for every_ms",
            "spike": "{shape, base, peak, at_ms, spike_ms, duration_ms}: base, peak from at_ms for spike_ms, base again",
            "poisson": "{shape, rate, duration_ms}: arrivals at random, rate on average, from the run's seed",
        },
        "metrics": ["p50_ms", "p90_ms", "p95_ms", "p99_ms", "mean_ms", "max_ms", "error_rate (% failed)", "rps (achieved)", "missed"],
        "ops": ["lt", "le", "gt", "ge"],
        "rules": [
            "HTTP nodes only, without repeat or retry: a failed request (no answer, or not 2xx) is counted, not tried again.",
            "Rates are 0.1–100000 requests/s (ramp, steps and spike may start or rest at 0); the whole profile runs within a run's 300 s.",
            "Templates are read once, as the step starts; the run's cookies and Digest answers are shared by every request.",
            "A request still waiting for a free worker 50 ms past its moment is skipped and counted as missed, never sent late.",
            "The step fails on the first threshold that does not hold (error load.threshold, with metric, op, value, actual); without thresholds it passes and measures.",
            "A load leaves no response for assert_status, extract or the like: judge it with thresholds.",
            "The step's last event carries load: {planned, sent, ok, failed, missed, rps, error_rate, p50_ms…p99_ms, statuses, seconds, histogram, thresholds}; list_runs and compare_runs read them back.",
        ],
        "example": example,
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
        // The emulator a node's example carries is one the engine would start.
        let example: signal_lab_engine::emulator::Emulator = serde_json::from_value(catalogue["emulators"]["example"].clone()).unwrap();
        signal_lab_engine::emulator::check(&example, &Default::default()).unwrap();
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
