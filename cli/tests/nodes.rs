//! Every kind of node in one experiment, run the ways a person, a script and a
//! pipeline run it — in this process and on a server — against loopback gear:
//! an HTTP API, a TCP sink, an OSC device that answers, a UDP device that
//! answers, an MQTT broker and a WebSocket service. Each node must pass, and the gear must have
//! seen the traffic.

mod common;

use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

use common::*;
use serde_json::{json, Value};

/// Every node type the engine has; the document below must use each one.
const KINDS: [&str; 32] = [
    "start", "end", "fork", "join", "log", "tcp", "delay", "http", "assert_status", "assert_body", "assert_header", "assert_latency",
    "mqtt", "branch_status", "osc", "udp", "extract", "assert_value", "branch_value", "wait_osc", "wait_mqtt", "loop", "wait_udp",
    "emulator", "wait_http", "impairment", "impairment_change", "emulator_state", "ws_connect", "ws_send", "wait_ws", "ws_close",
];

fn node(id: &str, x: i32, y: i32, body: Value) -> Value {
    let mut node = body;
    node["id"] = id.into();
    node["x"] = x.into();
    node["y"] = y.into();
    node
}

/// Start → an emulator for the run → log → delay → HTTP and its checks →
/// extract → check → branch on the value, then on the status → fork: OSC with
/// its reply then a wait, UDP with its reply then a wait → join → TCP → the API
/// under a short load, judged by thresholds → faults →
/// a WebSocket: connect, its greeting, a message and its echo, close → MQTT →
/// wait for it → a request to the emulator and a wait that sees it → a loop → End.
fn everything(gear: &Gear, mock_port: u16, relay_port: u16) -> Value {
    let (mqtt_port, tcp_port) = (gear.mqtt.port(), gear.tcp.port());
    let mock = format!("127.0.0.1:{mock_port}");
    let relay = format!("127.0.0.1:{relay_port}");
    let nodes = vec![
        node("start", 0, 200, json!({ "type": "start" })),
        node("mock", 0, 0, json!({ "type": "emulator", "emulator": { "name": "Hooks", "bind": mock, "protocol": "http",
            "routes": [{ "method": "POST", "path": "/hooks/:name", "responses": [{ "status": 202, "body": "{\"hook\":\"{{request.params.name}}\",\"for\":\"{{who}}\"}" }] }] } })),
        // The UDP device is reached through an impairment relay of the run's own.
        node("relay", 100, 0, json!({ "type": "impairment", "listen": relay, "target": gear.udp.to_string(), "profile": { "name": "lan" } })),
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
        node("hi", 2600, 300, json!({ "type": "udp", "target": relay, "text": "hi {{who}}",
            "reply": { "bind": "127.0.0.1:0", "mode": "contains", "pattern": "ack hi", "timeout_ms": 3000, "variable": "ack" } })),
        node("heard", 2800, 300, json!({ "type": "wait_udp", "bind": format!("127.0.0.1:{}", gear.udp_notify), "mode": "contains", "pattern": "hello", "timeout_ms": 3000 })),
        node("both", 3000, 200, json!({ "type": "join" })),
        node("line", 3200, 200, json!({ "type": "tcp", "host": "127.0.0.1", "port": tcp_port, "payload": "line {{who}}\n", "timeout_ms": 3000 })),
        node("pressure", 3225, 400, json!({ "type": "http", "request": { "method": "GET", "url": "{{api}}/status", "headers": [], "timeout_ms": 3000 },
            "load": { "profile": { "shape": "constant", "rate": 20, "duration_ms": 500 }, "concurrency": 4,
                      "thresholds": [{ "metric": "error_rate", "op": "lt", "value": 1 }, { "metric": "p95_ms", "op": "lt", "value": 2000 }] } })),
        node("rougher", 3250, 300, json!({ "type": "impairment_change", "relay": "relay", "profile": { "name": "4g", "latency_ms": 5, "jitter_ms": 2 } })),
        node("pull", 3300, 300, json!({ "type": "emulator_state", "emulator": "mock", "down": true, "fault": "unavailable" })),
        node("plug", 3350, 300, json!({ "type": "emulator_state", "emulator": "mock", "down": false })),
        node("socket", 3350, 500, json!({ "type": "ws_connect", "url": format!("ws://{}/chat", gear.ws), "headers": [["X-Who", "{{who}}"]], "protocols": ["lab.v1", "lab.v0"], "timeout_ms": 3000 })),
        node("greeted", 3350, 600, json!({ "type": "wait_ws", "connection": "socket", "mode": "contains", "pattern": "welcome", "timeout_ms": 3000 })),
        node("say", 3350, 700, json!({ "type": "ws_send", "connection": "socket", "text": "hi {{who}}" })),
        node("said", 3350, 800, json!({ "type": "wait_ws", "connection": "socket", "mode": "regex", "pattern": "^echo (hi .+)$", "timeout_ms": 3000, "variable": "echo" })),
        node("bye", 3350, 900, json!({ "type": "ws_close", "connection": "socket", "code": 1000, "reason": "done" })),
        node("publish", 3400, 200, json!({ "type": "mqtt", "host": "127.0.0.1", "port": mqtt_port, "topic": "lab/ping", "payload": "ping {{who}}", "qos": 0, "retain": false })),
        node("echoed", 3600, 200, json!({ "type": "wait_mqtt", "host": "127.0.0.1", "port": mqtt_port, "topic": "lab/+", "mode": "contains", "pattern": "ping", "timeout_ms": 3000 })),
        node("hook", 3800, 200, json!({ "type": "http", "request": { "method": "POST", "url": format!("http://{mock}/hooks/deploy"), "headers": [], "body": "{\"build\":7}", "timeout_ms": 3000 } })),
        node("hooked", 4000, 200, json!({ "type": "wait_http", "bind": mock, "method": "POST", "path": "/hooks/:name",
            "when": [{ "on": "json", "name": "$.build", "op": "eq", "value": "7" }], "timeout_ms": 3000, "variable": "hit" })),
        node("again", 4200, 200, json!({ "type": "loop", "max": 3, "until": { "value": "{{hit.params.name}}", "op": "eq", "expected": "deploy" } })),
        node("beat", 4200, 400, json!({ "type": "delay", "ms": 5 })),
        node("end", 4400, 200, json!({ "type": "end" })),
    ];
    let wire = |from: &str, to: &str, port: &str| json!({ "from": from, "to": to, "port": port });
    let edges = vec![
        wire("start", "mock", "next"), wire("mock", "relay", "next"), wire("relay", "log", "next"), wire("log", "pause", "next"), wire("pause", "get", "next"), wire("get", "status", "next"),
        wire("status", "body", "next"), wire("body", "header", "next"), wire("header", "latency", "next"), wire("latency", "take", "next"),
        wire("take", "check", "next"), wire("check", "which", "next"),
        wire("which", "code", "yes"), wire("which", "odd_value", "no"), wire("odd_value", "end", "next"),
        wire("code", "split", "yes"), wire("code", "odd_status", "no"), wire("odd_status", "end", "next"),
        wire("split", "ping", "branch1"), wire("ping", "told", "next"), wire("told", "both", "matched"),
        wire("split", "hi", "branch2"), wire("hi", "heard", "next"), wire("heard", "both", "matched"),
        wire("both", "line", "next"), wire("line", "pressure", "next"), wire("pressure", "rougher", "next"), wire("rougher", "pull", "next"), wire("pull", "plug", "next"), wire("plug", "socket", "next"),
        wire("socket", "greeted", "next"), wire("greeted", "say", "matched"), wire("say", "said", "next"), wire("said", "bye", "matched"), wire("bye", "publish", "next"), wire("publish", "echoed", "next"), wire("echoed", "hook", "matched"),
        wire("hook", "hooked", "next"), wire("hooked", "again", "matched"),
        wire("again", "beat", "body"), wire("beat", "again", "next"), wire("again", "end", "done"),
    ];
    json!({
        "version": 9,
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
    let used: BTreeSet<String> = everything(&gear, 1, 2)["nodes"].as_array().unwrap().iter().map(|node| node["type"].as_str().unwrap().to_string()).collect();
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
    let mock_port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let relay_port = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let document = everything(&gear, mock_port, relay_port);
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
    assert_eq!(result["emulators"][0]["counts"]["hits"], json!([1]), "the run's emulator took the request: {}", result["emulators"]);
    let phases: Vec<&str> = result["impairments"][0]["phases"].as_array().unwrap().iter().map(|phase| phase["profile"].as_str().unwrap()).collect();
    assert_eq!(phases, ["lan", "4g"], "the relay's phases: {}", result["impairments"]);
    assert!(result["impairments"][0]["counts"]["forwarded"].as_u64().unwrap() >= 2, "the datagram and its answer went through it");
    let hook = result["steps"].as_array().unwrap().iter().find(|step| step["node_id"] == "hook" && step["state"] == "passed").unwrap();
    assert!(hook["detail"].as_str().unwrap().starts_with("HTTP 202"), "its route answered: {hook}");
    let pressure = result["steps"].as_array().unwrap().iter().find(|step| step["node_id"] == "pressure" && step["state"] == "passed").unwrap();
    assert_eq!((pressure["load"]["sent"].as_u64(), pressure["load"]["failed"].as_u64()), (Some(10), Some(0)), "20/s for half a second: {pressure}");
    assert!(pressure["load"]["thresholds"].as_array().unwrap().iter().all(|verdict| verdict["held"] == true), "{pressure}");
    assert!(xml.contains("✓ "), "the JUnit report says how the thresholds went: {xml}");

    let seen = &gear.seen;
    let before = (seen.http.load(Ordering::SeqCst), seen.osc.load(Ordering::SeqCst), seen.udp.load(Ordering::SeqCst), seen.tcp_bytes.load(Ordering::SeqCst));
    assert!(before.0 >= 1 && before.1 >= 1 && before.2 >= 1 && before.3 > 0, "the gear saw the run: {before:?}");
    assert!(seen.mqtt.lock().unwrap().iter().any(|topic| topic == "lab/ping"));
    assert!(seen.ws.lock().unwrap().iter().any(|text| text == "hi lab"), "the WebSocket service got the message: {:?}", seen.ws.lock().unwrap());
    assert!(seen.ws_headers.lock().unwrap().iter().any(|header| header.eq_ignore_ascii_case("x-who: lab")), "and the upgrade's header");
    let said = result["steps"].as_array().unwrap().iter().find(|step| step["node_id"] == "said" && step["state"] == "passed").unwrap();
    assert_eq!(said["vars"]["echo"]["match"], "hi lab", "the regex group is the reply's match: {said}");
    let socket = result["steps"].as_array().unwrap().iter().find(|step| step["node_id"] == "socket" && step["state"] == "passed").unwrap();
    assert!(socket["detail"].as_str().unwrap().contains("(lab.v1)"), "the server chose the first subprotocol: {socket}");

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
