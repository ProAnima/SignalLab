//! Milestone 5: Signal Lab as the dependency. An emulator serves for a whole
//! run, and the run proves what reached it — through the command table,
//! against loopback, like the app and the command line run it.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_run::{Outcome, RunOptions, RunResult};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::osc_codec::{decode_packet, encode_message, OscArg};
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-emulators-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        paths::data_dir()
    })
    .clone()
}

fn service() -> Service {
    scratch_data_dir();
    let host = Host::new(Recorder::new(), Capture::new());
    Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None)))
}

fn tcp_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn node(id: &str, x: f64, kind: Value) -> Value {
    let mut value = kind;
    value["id"] = json!(id);
    value["x"] = json!(x);
    value["y"] = json!(0);
    value
}

fn document(name: &str, nodes: Vec<Value>, edges: Vec<(&str, &str, &str)>) -> Experiment {
    let edges: Vec<Value> = edges.into_iter().map(|(from, port, to)| json!({ "from": from, "to": to, "port": port })).collect();
    serde_json::from_value(json!({ "version": 6, "name": name, "params": [], "profiles": [], "profile": null, "seed": 11, "nodes": nodes, "edges": edges })).unwrap()
}

async fn run(service: &Service, doc: Experiment) -> RunResult {
    let handle = service.run(doc, RunOptions { limit: Some(Duration::from_secs(30)), ..Default::default() }).await.unwrap();
    handle.finished().await
}

fn failure(result: &RunResult) -> String {
    format!("{:?} {:#?}", result.error, result.steps.iter().filter(|step| step.state == "failed").collect::<Vec<_>>())
}

/// The roadmap's "done when": an app pointed at the mock retries after two 500
/// responses, and the experiment proves it — three requests were received, and
/// the third got 200. The app here is the experiment's own loop.
#[tokio::test]
async fn a_flaky_dependency_is_retried_and_the_run_proves_what_reached_it() {
    let service = service();
    let port = tcp_port();
    let doc = document(
        "Flaky dependency",
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("mock", 200.0, json!({ "type": "emulator", "emulator": {
                "name": "Flaky API", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
                "routes": [{ "method": "GET", "path": "/orders/:id", "responses": [{ "status": 500 }, { "status": 500 },
                    { "body": "{\"order\":\"{{request.params.id}}\",\"attempt\":{{counter}}}" }] }]
            } })),
            node("loop", 400.0, json!({ "type": "loop", "max": 5, "until": { "value": "{{status}}", "op": "eq", "expected": "200" } })),
            node("get", 600.0, json!({ "type": "http", "request": { "method": "GET", "url": format!("http://127.0.0.1:{port}/orders/42"), "headers": [], "body": null, "timeout_ms": 3000 } })),
            node("status", 800.0, json!({ "type": "extract", "variable": "status", "from": "status", "expr": "" })),
            node("body", 1000.0, json!({ "type": "extract", "variable": "body", "from": "body", "expr": "" })),
            node("third", 1200.0, json!({ "type": "assert_value", "value": "{{body}}", "op": "eq", "expected": "{\"order\":\"42\",\"attempt\":3}" })),
            node("end", 1400.0, json!({ "type": "end" })),
        ],
        vec![
            ("start", "next", "mock"),
            ("mock", "next", "loop"),
            ("loop", "body", "get"),
            ("get", "next", "status"),
            ("status", "next", "body"),
            ("body", "next", "loop"),
            ("loop", "done", "third"),
            ("third", "next", "end"),
        ],
    );
    let result = run(&service, doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let summary = &result.emulators[0];
    assert_eq!((summary.node.as_deref(), summary.name.as_str(), summary.protocol), (Some("mock"), "Flaky API", "http"));
    assert_eq!((summary.counts.total, summary.counts.hits.clone(), summary.counts.unmatched), (3, vec![3], 0), "three requests reached the route");
    let emulating = result.steps.iter().find(|step| step.node_id == "mock" && step.state == "passed").unwrap();
    assert_eq!(emulating.message_key, Some("exp.step.emulating"));
    // The run is over: so is the emulator.
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(reqwest::get(format!("http://127.0.0.1:{port}/orders/1")).await.is_err(), "the port closed with the run");
    let report: Value = serde_json::from_str(&std::fs::read_to_string(result.report_path.as_ref().unwrap()).unwrap()).unwrap();
    assert_eq!((report["version"].clone(), report["emulators"][0]["counts"]["hits"].clone()), (json!(3), json!([3])));
}

/// The bundled template does what it says: answered on the third attempt.
#[tokio::test]
async fn the_flaky_api_template_is_answered_on_the_third_attempt() {
    let service = service();
    let port = tcp_port();
    let text = include_str!("../../experiments/templates/flaky-api.json").replace("127.0.0.1:18080", &format!("127.0.0.1:{port}"));
    let doc = signal_lab_engine::experiment_files::parse(&text).unwrap();
    let result = run(&service, doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let answered = result.steps.iter().find(|step| step.node_id == "answered" && step.state == "passed").unwrap();
    assert_eq!(answered.detail, format!("http://127.0.0.1:{port} answered on attempt 3"));
    assert_eq!(result.emulators[0].counts.hits, [3]);
}

/// A webhook: the run's own listener takes a request, a wait reads what it
/// carried, and the sender got 204.
#[tokio::test]
async fn a_wait_for_http_request_reads_what_arrived() {
    let service = service();
    let port = tcp_port();
    let bind = format!("127.0.0.1:{port}");
    let doc = document(
        "Webhook",
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("post", 200.0, json!({ "type": "http", "request": { "method": "POST", "url": format!("http://{bind}/hooks/deploy?env=stage"),
                "headers": [["X-Signature", "abc"]], "body": "{\"event\":\"deploy\",\"build\":17}", "timeout_ms": 3000 } })),
            node("hook", 400.0, json!({ "type": "wait_http", "bind": bind, "method": "POST", "path": "/hooks/:kind",
                "when": [{ "on": "header", "name": "x-signature", "op": "eq", "value": "abc" }, { "on": "json", "name": "$.build", "op": "ge", "value": "10" }],
                "timeout_ms": 3000 })),
            node("event", 600.0, json!({ "type": "assert_value", "value": "{{request.json.event}} {{request.params.kind}} {{request.query.env}}", "op": "eq", "expected": "deploy deploy stage" })),
            node("accepted", 800.0, json!({ "type": "assert_status", "status": 204 })),
            node("end", 1000.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "post"), ("post", "next", "hook"), ("hook", "matched", "event"), ("event", "next", "accepted"), ("accepted", "next", "end")],
    );
    let result = run(&service, doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let matched = result.steps.iter().find(|step| step.node_id == "hook" && step.state == "passed").unwrap();
    assert_eq!(matched.message_params["summary"], "POST /hooks/deploy");
    assert!(result.emulators.is_empty(), "a wait's listener is not an emulator of the report");

    // Nothing arrives: Timeout, naming the address it listened on.
    let quiet = document(
        "Quiet",
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("hook", 200.0, json!({ "type": "wait_http", "bind": format!("127.0.0.1:{}", tcp_port()), "timeout_ms": 150 })),
            node("end", 400.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "hook"), ("hook", "matched", "end")],
    );
    let result = run(&service, quiet).await;
    assert_eq!((result.outcome, result.error.as_ref().map(|error| error.code.as_str())), (Outcome::Failed, Some("wait.timeout")));
}

/// An OSC device emulated in the run shares its port with the run's waits:
/// it answers the ping, and a wait sees that same ping arrive.
#[tokio::test]
async fn an_osc_emulator_answers_and_its_port_is_shared_with_the_waits() {
    let service = service();
    let device = format!("127.0.0.1:{}", udp_port());
    let doc = document(
        "Device",
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("device", 200.0, json!({ "type": "emulator", "emulator": { "name": "Desk", "bind": device, "protocol": "osc",
                "rules": [{ "address": "/ping", "reply": { "address": "/pong", "args": [{ "type": "int", "value": "{{request.args[0]}}" }] } }] } })),
            node("ping", 400.0, json!({ "type": "osc", "target": device, "address": "/ping", "args": [{ "type": "int", "value": 7 }],
                "reply": { "bind": "127.0.0.1:0", "address": "/pong", "args": [{ "index": 0, "op": "eq", "value": "7" }], "timeout_ms": 2000 } })),
            node("seen", 600.0, json!({ "type": "wait_osc", "bind": device, "address": "/ping", "timeout_ms": 2000, "variable": "sent" })),
            node("end", 800.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "device"), ("device", "next", "ping"), ("ping", "next", "seen"), ("seen", "matched", "end")],
    );
    let result = run(&service, doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let replied = result.steps.iter().find(|step| step.node_id == "ping" && step.state == "passed").unwrap();
    assert_eq!(replied.vars.as_ref().unwrap()["reply"]["args"], json!([7]), "the emulator echoed the argument");
    let seen = result.steps.iter().find(|step| step.node_id == "seen" && step.state == "passed").unwrap();
    assert_eq!(seen.vars.as_ref().unwrap()["sent"]["address"], "/ping", "the wait saw the ping the emulator answered");
    assert_eq!(result.emulators[0].counts.hits, [1]);

    // On its own, the same emulator answers any sender, from its own port.
    let job = service.invoke("emulator_start", json!({ "emulator": {
        "name": "Desk", "bind": device, "protocol": "osc",
        "rules": [{ "address": "/ping", "reply": { "address": "/pong", "args": [{ "type": "int", "value": "{{request.args[0]}}" }] } }] } })).await.unwrap();
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.send_to(&encode_message("/ping", &[OscArg::Int(9)]), &device).await.unwrap();
    let mut buffer = [0u8; 512];
    let (size, from) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buffer)).await.unwrap().unwrap();
    assert_eq!((decode_packet(&buffer[..size]).unwrap()[0].args.clone(), from.to_string()), (vec![OscArg::Int(9)], device.clone()));
    service.invoke("job_stop", json!({ "id": job["id"] })).await.unwrap();
}

/// Two emulators cannot share a port, and a port taken by another program is
/// an error at the node that wanted it, before any traffic.
#[tokio::test]
async fn ports_are_checked_before_the_first_step() {
    let service = service();
    let port = tcp_port();
    let emulator = |id: &str, bind: &str| node(id, 0.0, json!({ "type": "emulator", "emulator": { "name": id, "bind": bind, "protocol": "http" } }));
    let twice = document(
        "Twice",
        vec![node("start", 0.0, json!({ "type": "start" })), emulator("a", &format!("127.0.0.1:{port}")), emulator("b", &format!("127.0.0.1:{port}")), node("end", 0.0, json!({ "type": "end" }))],
        vec![("start", "next", "a"), ("a", "next", "b"), ("b", "next", "end")],
    );
    let error = service.run(twice, RunOptions::default()).await.err().unwrap();
    assert_eq!((error.code.as_str(), error.node.as_deref()), ("emulator.bind_taken", Some("b")));

    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let busy = document(
        "Busy",
        vec![node("start", 0.0, json!({ "type": "start" })), emulator("a", &taken.local_addr().unwrap().to_string()), node("end", 0.0, json!({ "type": "end" }))],
        vec![("start", "next", "a"), ("a", "next", "end")],
    );
    let error = service.run(busy, RunOptions::default()).await.err().unwrap();
    assert_eq!((error.code.as_str(), error.node.as_deref()), ("transport.address_in_use", Some("a")));
    assert!(service.jobs().list().is_empty(), "no job was started");
}

/// The commands the screen and the command line use: start, what arrived, stop.
#[tokio::test]
async fn an_emulator_job_is_started_followed_and_stopped_through_the_commands() {
    let service = service();
    let port = tcp_port();
    let emulator = json!({ "name": "API", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
        "routes": [{ "path": "/items/:id", "responses": [{ "body": "item {{request.params.id}} for {{who}}" }] }] });
    let problem = service.invoke("emulator_check", json!({ "emulator": emulator })).await.unwrap_err();
    assert_eq!(serde_json::to_value(&problem).unwrap()["code"], "emulator.name_unknown", "{{who}} is no parameter yet");
    assert!(service.invoke("emulator_check", json!({ "emulator": emulator, "params": { "who": "Ada" } })).await.is_ok());
    let job = service.invoke("emulator_start", json!({ "emulator": emulator, "params": { "who": "Ada" }, "source": "lib-1" })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    assert_eq!((job["kind"].as_str(), job["params"]["local"].as_str(), job["params"]["source"].as_str()), (Some("emulator"), Some(format!("127.0.0.1:{port}").as_str()), Some("lib-1")));
    let body = reqwest::get(format!("http://127.0.0.1:{port}/items/5")).await.unwrap().text().await.unwrap();
    assert_eq!(body, "item 5 for Ada");
    let seen = service.invoke("emulator_exchanges", json!({ "jobId": id })).await.unwrap();
    assert_eq!((seen["counts"]["hits"].clone(), seen["exchanges"][0]["request"].clone()), (json!([1]), json!("GET /items/5")));
    let after = seen["exchanges"][0]["seq"].as_u64().unwrap();
    assert_eq!(service.invoke("emulator_exchanges", json!({ "jobId": id, "after": after })).await.unwrap()["exchanges"], json!([]));
    assert_eq!(service.invoke("job_stop", json!({ "id": id })).await.unwrap(), json!(true));
    tokio::time::sleep(Duration::from_millis(50)).await;
    let gone = service.invoke("emulator_exchanges", json!({ "jobId": id })).await.unwrap_err();
    assert_eq!(serde_json::to_value(&gone).unwrap()["code"], "emulator.not_running");
}

/// MQTT gear tested against a broker of the run's own: the experiment tells a
/// lamp to switch on, the emulated device reports its new state, and a wait
/// subscribed before the first step reads it — no broker to install.
#[tokio::test]
async fn an_mqtt_device_of_the_run_answers_a_command_and_the_wait_reads_it() {
    let service = service();
    let port = tcp_port();
    let doc = document(
        "MQTT device",
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("broker", 200.0, json!({ "type": "emulator", "emulator": {
                "name": "Lamp", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt",
                "rules": [{ "topic": "lab/+/set", "mode": "regex", "pattern": "^(ON|OFF)$",
                            "reply": { "topic": "lab/{{request.levels[1]}}/state", "payload": "{{request.match}} by {{request.client}}", "delay_ms": 50 } }]
            } })),
            node("set", 400.0, json!({ "type": "mqtt", "host": "127.0.0.1", "port": port, "topic": "lab/lamp/set", "payload": "ON", "qos": 1, "retain": false })),
            node("state", 600.0, json!({ "type": "wait_mqtt", "host": "127.0.0.1", "port": port, "topic": "lab/lamp/state", "mode": "contains", "pattern": "ON", "timeout_ms": 3000 })),
            node("end", 800.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "broker"), ("broker", "next", "set"), ("set", "next", "state"), ("state", "matched", "end")],
    );
    let result = run(&service, doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let state = result.steps.iter().find(|step| step.node_id == "state" && step.state == "passed").expect("the wait passed");
    let reply = &state.vars.as_ref().unwrap_or_else(|| panic!("{state:?}"))["reply"];
    assert!(reply["text"].as_str().is_some_and(|text| text.starts_with("ON by lab-")), "the run's own client id: {reply}");
    let counts = serde_json::to_value(&result.emulators[0].counts).unwrap();
    assert_eq!((counts["total"].clone(), counts["hits"].clone(), counts["down"].clone()), (json!(1), json!([1]), json!(0)));
}
