//! Every kind of node in one experiment, run the ways a person, a script and a
//! pipeline run it — in this process and on a server — against loopback gear:
//! an HTTP API, a TCP sink, an OSC device that answers, a UDP device that
//! answers and an MQTT broker. Each node must pass, and the gear must have
//! seen the traffic.

mod common;

use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

use common::*;
use serde_json::{json, Value};

/// Every node type the engine has; the document below must use each one.
const KINDS: [&str; 23] = [
    "start", "end", "fork", "join", "log", "tcp", "delay", "http", "assert_status", "assert_body", "assert_header", "assert_latency",
    "mqtt", "branch_status", "osc", "udp", "extract", "assert_value", "branch_value", "wait_osc", "wait_mqtt", "loop", "wait_udp",
];

fn node(id: &str, x: i32, y: i32, body: Value) -> Value {
    let mut node = body;
    node["id"] = id.into();
    node["x"] = x.into();
    node["y"] = y.into();
    node
}

/// Start → log → delay → HTTP and its checks → extract → check → branch on the
/// value, then on the status → fork: OSC with its reply then a wait, UDP with
/// its reply then a wait → join → TCP → MQTT → wait for it → a loop → End.
fn everything(gear: &Gear) -> Value {
    let (mqtt_port, tcp_port) = (gear.mqtt.port(), gear.tcp.port());
    let nodes = vec![
        node("start", 0, 200, json!({ "type": "start" })),
        node("log", 200, 200, json!({ "type": "log", "message": "hello {{who}}" })),
        node("pause", 400, 200, json!({ "type": "delay", "ms": 10 })),
        node("get", 600, 200, json!({ "type": "http", "request": { "method": "GET", "url": "{{api}}/status", "headers": [["Accept", "application/json"]], "timeout_ms": 3000 } })),
        node("status", 800, 200, json!({ "type": "assert_status", "status": 200 })),
        node("body", 1000, 200, json!({ "type": "assert_body", "contains": "ready" })),
        node("header", 1200, 200, json!({ "type": "assert_header", "name": "X-Lab", "contains": "ready" })),
        node("latency", 1400, 200, json!({ "type": "assert_latency", "max_ms": 5000 })),
        node("take", 1600, 200, json!({ "type": "extract", "variable": "state", "from": "json", "expr": "$.state" })),
        node("check", 1800, 200, json!({ "type": "assert_value", "value": "{{state}}", "op": "eq", "expected": "ready" })),
        node("which", 2000, 200, json!({ "type": "branch_value", "value": "{{state}}", "op": "eq", "expected": "ready" })),
        node("odd_value", 2200, 400, json!({ "type": "log", "message": "not ready" })),
        node("code", 2200, 200, json!({ "type": "branch_status", "status": 200 })),
        node("odd_status", 2400, 400, json!({ "type": "log", "message": "not 200" })),
        node("split", 2400, 200, json!({ "type": "fork" })),
        node("ping", 2600, 100, json!({ "type": "osc", "target": gear.osc.to_string(), "address": "/ping", "args": [{ "type": "str", "value": "{{who}}" }],
            "reply": { "bind": "127.0.0.1:0", "address": "/pong", "args": [{ "index": 0, "op": "eq", "value": "1" }], "timeout_ms": 3000, "variable": "pong" } })),
        node("told", 2800, 100, json!({ "type": "wait_osc", "bind": format!("127.0.0.1:{}", gear.osc_notify), "address": "/status",
            "args": [{ "index": 0, "op": "eq", "value": "ready" }], "timeout_ms": 3000, "variable": "status" })),
        node("hi", 2600, 300, json!({ "type": "udp", "target": gear.udp.to_string(), "text": "hi {{who}}",
            "reply": { "bind": "127.0.0.1:0", "mode": "contains", "pattern": "ack hi", "timeout_ms": 3000, "variable": "ack" } })),
        node("heard", 2800, 300, json!({ "type": "wait_udp", "bind": format!("127.0.0.1:{}", gear.udp_notify), "mode": "contains", "pattern": "hello", "timeout_ms": 3000 })),
        node("both", 3000, 200, json!({ "type": "join" })),
        node("line", 3200, 200, json!({ "type": "tcp", "host": "127.0.0.1", "port": tcp_port, "payload": "line {{who}}\n", "timeout_ms": 3000 })),
        node("publish", 3400, 200, json!({ "type": "mqtt", "host": "127.0.0.1", "port": mqtt_port, "topic": "lab/ping", "payload": "ping {{who}}", "qos": 0, "retain": false })),
        node("echoed", 3600, 200, json!({ "type": "wait_mqtt", "host": "127.0.0.1", "port": mqtt_port, "topic": "lab/+", "mode": "contains", "pattern": "ping", "timeout_ms": 3000 })),
        node("again", 3800, 200, json!({ "type": "loop", "max": 3, "until": { "value": "{{state}}", "op": "eq", "expected": "ready" } })),
        node("beat", 3800, 400, json!({ "type": "delay", "ms": 5 })),
        node("end", 4000, 200, json!({ "type": "end" })),
    ];
    let wire = |from: &str, to: &str, port: &str| json!({ "from": from, "to": to, "port": port });
    let edges = vec![
        wire("start", "log", "next"), wire("log", "pause", "next"), wire("pause", "get", "next"), wire("get", "status", "next"),
        wire("status", "body", "next"), wire("body", "header", "next"), wire("header", "latency", "next"), wire("latency", "take", "next"),
        wire("take", "check", "next"), wire("check", "which", "next"),
        wire("which", "code", "yes"), wire("which", "odd_value", "no"), wire("odd_value", "end", "next"),
        wire("code", "split", "yes"), wire("code", "odd_status", "no"), wire("odd_status", "end", "next"),
        wire("split", "ping", "branch1"), wire("ping", "told", "next"), wire("told", "both", "matched"),
        wire("split", "hi", "branch2"), wire("hi", "heard", "next"), wire("heard", "both", "matched"),
        wire("both", "line", "next"), wire("line", "publish", "next"), wire("publish", "echoed", "next"), wire("echoed", "again", "matched"),
        wire("again", "beat", "body"), wire("beat", "again", "next"), wire("again", "end", "done"),
    ];
    json!({
        "version": 5,
        "name": "Every node",
        "params": [{ "name": "who", "value": "world" }, { "name": "api", "value": format!("http://{}", gear.http) }],
        "nodes": nodes,
        "edges": edges,
    })
}

/// The `ended` line of a `--json` run.
fn ended(stdout: &str) -> Value {
    stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|line| line["type"] == "ended")
        .unwrap_or_else(|| panic!("no ended line in:\n{stdout}"))
}

/// Every node that passed, and the ones that did not.
fn verdict(result: &Value, document: &Value) -> (BTreeSet<String>, Vec<String>) {
    let passed: BTreeSet<String> = result["steps"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|step| step["state"] == "passed")
        .map(|step| step["node_id"].as_str().unwrap().to_string())
        .collect();
    let expected = document["nodes"].as_array().unwrap().iter().map(|node| node["id"].as_str().unwrap().to_string()).filter(|id| !id.starts_with("odd_"));
    let missing = expected.filter(|id| !passed.contains(id)).collect();
    (passed, missing)
}

#[test]
fn the_document_uses_every_kind_of_node() {
    let gear = tokio::runtime::Runtime::new().unwrap().block_on(gear());
    let used: BTreeSet<String> = everything(&gear)["nodes"].as_array().unwrap().iter().map(|node| node["type"].as_str().unwrap().to_string()).collect();
    let kinds: BTreeSet<String> = KINDS.iter().map(|kind| kind.to_string()).collect();
    assert_eq!(used, kinds);
    // And KINDS is the engine's list: the node catalogue of the interface names each one.
    let catalogue = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/lib/locales/en.ts")).unwrap();
    for kind in KINDS {
        assert!(catalogue.contains(&format!("\"exp.node.{kind}\":")), "{kind} is a node the interface knows");
    }
    let engine = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/src/experiment.rs")).unwrap();
    let start = engine.find("pub enum NodeKind {").unwrap();
    let body = &engine[start..start + engine[start..].find("\n}\n").unwrap()];
    let variants = body.lines().filter(|line| line.starts_with("    ") && !line.starts_with("     ") && line.trim_start().starts_with(|c: char| c.is_ascii_uppercase())).count();
    assert_eq!(variants, KINDS.len(), "a new kind of node goes in this test too");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_node_passes_here_and_on_a_server() {
    let dir = folder("nodes");
    let gear = gear().await;
    let document = everything(&gear);
    let file = dir.join("every-node.json");
    std::fs::write(&file, serde_json::to_string_pretty(&document).unwrap()).unwrap();
    let file = file.display().to_string();

    let valid = signallab_async(vec!["validate".into(), file.clone()]).await;
    assert_eq!(code(&valid), 0, "{}{}", out(&valid), err(&valid));

    // In this process, with a JUnit report.
    let junit = dir.join("junit.xml");
    let here = signallab_async(vec!["--json".into(), "run".into(), file.clone(), "--param".into(), "who=lab".into(), "--junit".into(), junit.display().to_string()]).await;
    let result = ended(&out(&here));
    let (_, missing) = verdict(&result, &document);
    assert!(code(&here) == 0 && result["outcome"] == "passed" && missing.is_empty(), "here: missing {missing:?}, error {}\n{}", result["error"], out(&here));
    let xml = std::fs::read_to_string(&junit).unwrap();
    assert!(xml.contains("failures=\"0\" errors=\"0\" skipped=\"2\""), "only the two odd branches are not reached: {xml}");
    assert!(result["steps"].as_array().unwrap().iter().any(|step| step["node_id"] == "log" && step["detail"] == "hello lab"), "the parameter reached the template");

    let seen = &gear.seen;
    let before = (seen.http.load(Ordering::SeqCst), seen.osc.load(Ordering::SeqCst), seen.udp.load(Ordering::SeqCst), seen.tcp_bytes.load(Ordering::SeqCst));
    assert!(before.0 >= 1 && before.1 >= 1 && before.2 >= 1 && before.3 > 0, "the gear saw the run: {before:?}");
    assert!(seen.mqtt.lock().unwrap().iter().any(|topic| topic == "lab/ping"));

    // On a server: the same document, sent over the API, the steps streamed back.
    let running = start_server(&dir.join("server")).await;
    let token = dir.join("token.txt");
    std::fs::write(&token, TOKEN).unwrap();
    let remote = signallab_async(vec!["--json".into(), "run".into(), file, "--server".into(), running.url.clone(), "--token-file".into(), token.display().to_string()]).await;
    let result = ended(&out(&remote));
    let (_, missing) = verdict(&result, &document);
    assert!(code(&remote) == 0 && result["outcome"] == "passed" && missing.is_empty(), "server: missing {missing:?}, error {}\n{}", result["error"], out(&remote));
    assert!(seen.http.load(Ordering::SeqCst) > before.0, "the server's run reached the gear too");
    running.stop().await;
}
