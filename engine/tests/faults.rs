//! Milestone 6: faults as nodes and phases. A run degrades the network and
//! its dependencies on a schedule of its own steps — through the command
//! table, against loopback — and always restores them: the same seed gives
//! the same drops, each phase is counted, and a stopped run leaves nothing
//! impaired behind.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_run::{Outcome, RunOptions, RunResult};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-faults-{}", std::process::id()));
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

fn udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn tcp_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn node(id: &str, x: f64, kind: Value) -> Value {
    let mut value = kind;
    value["id"] = json!(id);
    value["x"] = json!(x);
    value["y"] = json!(0);
    value
}

fn document(nodes: Vec<Value>, edges: Vec<(&str, &str, &str)>, seed: u64) -> Experiment {
    let edges: Vec<Value> = edges.into_iter().map(|(from, port, to)| json!({ "from": from, "to": to, "port": port })).collect();
    serde_json::from_value(json!({ "version": 8, "name": "Faults", "params": [], "profiles": [], "profile": null, "seed": seed, "nodes": nodes, "edges": edges })).unwrap()
}

async fn run(service: &Service, doc: Experiment) -> RunResult {
    let handle = service.run(doc, RunOptions { limit: Some(Duration::from_secs(30)), ..Default::default() }).await.unwrap();
    handle.finished().await
}

fn failure(result: &RunResult) -> String {
    format!("{:?} {:#?}", result.error, result.steps.iter().filter(|step| step.state == "failed").collect::<Vec<_>>())
}

/// A device on loopback that counts what reaches it, a relay in front of it,
/// and `count` datagrams sent through the relay.
fn lossy(device: u16, relay: u16, count: u32, seed: u64) -> Experiment {
    document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("device", 100.0, json!({ "type": "emulator", "emulator": { "name": "Device", "bind": format!("127.0.0.1:{device}"), "protocol": "udp", "rules": [{ "mode": "any" }] } })),
            node("relay", 200.0, json!({ "type": "impairment", "listen": format!("127.0.0.1:{relay}"), "target": format!("127.0.0.1:{device}"),
                "profile": { "name": "lossy", "loss": 0.3, "duplicate": 0.1 } })),
            node("send", 300.0, json!({ "type": "udp", "target": format!("127.0.0.1:{relay}"), "text": "beat {{counter}}",
                "repeat": { "until": "count", "count": count, "interval_ms": 10 } })),
            node("settle", 400.0, json!({ "type": "delay", "ms": 200 })),
            node("end", 500.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "device"), ("device", "next", "relay"), ("relay", "next", "send"), ("send", "next", "settle"), ("settle", "next", "end")],
        seed,
    )
}

/// The roadmap's "done when": the same seeded fault experiment gives the same
/// drop pattern twice, and another seed another.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_same_seed_drops_the_same_packets_run_after_run() {
    let service = service();
    let (device, relay) = (udp_port(), udp_port());
    let mut seen = Vec::new();
    for seed in [5, 5, 6] {
        let result = run(&service, lossy(device, relay, 60, seed)).await;
        assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
        let impairment = serde_json::to_value(&result.impairments[0]).unwrap();
        let received = serde_json::to_value(&result.emulators[0].counts).unwrap()["total"].clone();
        assert_eq!(impairment["counts"]["received"], 60, "every datagram reached the relay");
        assert_eq!(impairment["counts"]["forwarded"], received, "what it forwarded is what the device got");
        seen.push((impairment["counts"]["dropped"].clone(), impairment["counts"]["duplicated"].clone(), received));
    }
    assert_eq!(seen[0], seen[1], "the same seed, the same drops and copies");
    assert_ne!(seen[0], seen[2], "another seed, others");
    let dropped = seen[0].0.as_u64().unwrap();
    assert!((8..=30).contains(&dropped), "about 30 % of 60: {dropped}");
}

/// Phases from a branch of their own, next to the traffic: clean, then
/// offline, then clean again — each counted apart, and on the timeline.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_schedule_branch_switches_phases_and_each_is_counted() {
    let service = service();
    let (device, relay) = (udp_port(), udp_port());
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("device", 100.0, json!({ "type": "emulator", "emulator": { "name": "Device", "bind": format!("127.0.0.1:{device}"), "protocol": "udp", "rules": [{ "mode": "any" }] } })),
            node("relay", 200.0, json!({ "type": "impairment", "listen": format!("127.0.0.1:{relay}"), "target": format!("127.0.0.1:{device}"), "profile": { "name": "lan" } })),
            node("fork", 300.0, json!({ "type": "fork" })),
            node("send", 400.0, json!({ "type": "udp", "target": format!("127.0.0.1:{relay}"), "text": "beat", "repeat": { "until": "duration", "duration_ms": 1500, "interval_ms": 20 } })),
            node("wait1", 400.0, json!({ "type": "delay", "ms": 400 })),
            node("offline", 500.0, json!({ "type": "impairment_change", "relay": "relay", "profile": { "name": "offline", "offline": true } })),
            node("wait2", 600.0, json!({ "type": "delay", "ms": 400 })),
            node("back", 700.0, json!({ "type": "impairment_change", "relay": "relay", "profile": { "name": "lan" } })),
            node("join", 800.0, json!({ "type": "join" })),
            node("settle", 850.0, json!({ "type": "delay", "ms": 100 })),
            node("end", 900.0, json!({ "type": "end" })),
        ],
        vec![
            ("start", "next", "device"),
            ("device", "next", "relay"),
            ("relay", "next", "fork"),
            ("fork", "branch1", "send"),
            ("fork", "branch2", "wait1"),
            ("wait1", "next", "offline"),
            ("offline", "next", "wait2"),
            ("wait2", "next", "back"),
            ("send", "next", "join"),
            ("back", "next", "join"),
            ("join", "next", "settle"),
            ("settle", "next", "end"),
        ],
        1,
    );
    let result = run(&service, doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let summary = serde_json::to_value(&result.impairments[0]).unwrap();
    let phases: Vec<(&str, u64, u64)> = summary["phases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|phase| (phase["profile"].as_str().unwrap(), phase["counts"]["forwarded"].as_u64().unwrap(), phase["counts"]["dropped"].as_u64().unwrap()))
        .collect();
    assert_eq!(phases.iter().map(|phase| phase.0).collect::<Vec<_>>(), ["lan", "offline", "lan"], "{phases:?}");
    assert!(phases[0].1 > 5 && phases[0].2 == 0, "clean at first: {phases:?}");
    assert!(phases[1].1 <= 1 && phases[1].2 > 5, "nothing through while offline (one still on its way, at most): {phases:?}");
    assert!(phases[2].1 > 5 && phases[2].2 == 0, "and clean again: {phases:?}");
    let switched: Vec<(&str, &str)> = result
        .steps
        .iter()
        .filter(|step| step.message_key == Some("exp.step.impaired"))
        .map(|step| (step.node_id.as_str(), step.message_params["profile"].as_str().unwrap()))
        .collect();
    assert_eq!(switched, [("offline", "offline"), ("back", "lan")], "every applied fault is a timeline step");
    let counts = serde_json::to_value(&result.emulators[0].counts).unwrap();
    assert_eq!(counts["total"], summary["counts"]["forwarded"], "the device got what was let through");
}

/// Stop in the middle: the relay closes with the run, nothing stays impaired.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_stopped_run_leaves_nothing_impaired() {
    let service = service();
    let (device, relay) = (udp_port(), udp_port());
    let mut doc = lossy(device, relay, 10_000, 1);
    doc.nodes.retain(|node| node.id != "device");
    doc.edges.retain(|edge| edge.from != "device" && edge.to != "device");
    doc.edges.push(serde_json::from_value(json!({ "from": "start", "to": "relay", "port": "next" })).unwrap());
    let handle = service.run(doc, RunOptions::default()).await.unwrap();
    let job = handle.started.job_id;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(std::net::UdpSocket::bind(("127.0.0.1", relay)).is_err(), "the relay listens while the run lasts");
    assert_eq!(service.invoke("job_stop", json!({ "id": job })).await.unwrap(), json!(true));
    let result = handle.finished().await;
    assert_eq!(result.outcome, Outcome::Stopped);
    assert!(result.impairments[0].counts.received > 0, "a stopped run reports what its relay did");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(std::net::UdpSocket::bind(("127.0.0.1", relay)).is_ok(), "and its port is free again");
}

/// A dependency taken down and brought back by the run's own steps: the
/// client meets 503 meanwhile, then 200.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_dependency_goes_down_and_comes_back_on_cue() {
    let service = service();
    let port = tcp_port();
    let url = format!("http://127.0.0.1:{port}/health");
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("api", 100.0, json!({ "type": "emulator", "emulator": { "name": "API", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
                "routes": [{ "path": "/health", "responses": [{ "body": "ok" }] }] } })),
            node("down", 200.0, json!({ "type": "emulator_state", "emulator": "api", "down": true })),
            node("get1", 300.0, json!({ "type": "http", "request": { "method": "GET", "url": url, "headers": [], "body": null, "timeout_ms": 2000 } })),
            node("is503", 400.0, json!({ "type": "assert_status", "status": 503 })),
            node("up", 500.0, json!({ "type": "emulator_state", "emulator": "api", "down": false })),
            node("get2", 600.0, json!({ "type": "http", "request": { "method": "GET", "url": url, "headers": [], "body": null, "timeout_ms": 2000 } })),
            node("is200", 700.0, json!({ "type": "assert_status", "status": 200 })),
            node("end", 800.0, json!({ "type": "end" })),
        ],
        vec![
            ("start", "next", "api"),
            ("api", "next", "down"),
            ("down", "next", "get1"),
            ("get1", "next", "is503"),
            ("is503", "next", "up"),
            ("up", "next", "get2"),
            ("get2", "next", "is200"),
            ("is200", "next", "end"),
        ],
        1,
    );
    let result = run(&service, doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let keys: Vec<&str> = result.steps.iter().filter(|step| step.state == "passed").filter_map(|step| step.message_key).filter(|key| key.starts_with("exp.step.emulat")).collect();
    assert_eq!(keys, ["exp.step.emulating", "exp.step.emulatorDown", "exp.step.emulatorUp"]);
    let counts = serde_json::to_value(&result.emulators[0].counts).unwrap();
    assert_eq!((counts["total"].clone(), counts["down"].clone(), counts["hits"].clone()), (json!(2), json!(1), json!([1])));
}

/// What cannot work is refused before any traffic, naming the node and field.
#[tokio::test]
async fn faults_that_cannot_work_are_refused_before_the_run() {
    let service = service();
    let (device, relay) = (udp_port(), udp_port());
    let refused = |doc: Experiment| async { service.run(doc, RunOptions::default()).await.err().unwrap() };
    let mut unknown = lossy(device, relay, 2, 1);
    unknown.nodes.push(serde_json::from_value(node("change", 600.0, json!({ "type": "impairment_change", "relay": "nowhere" }))).unwrap());
    unknown.edges.retain(|edge| edge.from != "settle");
    unknown.edges.push(serde_json::from_value(json!({ "from": "settle", "to": "change", "port": "next" })).unwrap());
    unknown.edges.push(serde_json::from_value(json!({ "from": "change", "to": "end", "port": "next" })).unwrap());
    let error = refused(unknown).await;
    assert_eq!((error.code.as_str(), error.node.as_deref(), error.field.as_ref().map(|field| field.key.as_str())), ("impair.relay_unknown", Some("change"), Some("relay")));

    let mut not_an_emulator = lossy(device, relay, 2, 1);
    not_an_emulator.nodes.push(serde_json::from_value(node("flip", 600.0, json!({ "type": "emulator_state", "emulator": "relay", "down": true }))).unwrap());
    not_an_emulator.edges.retain(|edge| edge.from != "settle");
    not_an_emulator.edges.push(serde_json::from_value(json!({ "from": "settle", "to": "flip", "port": "next" })).unwrap());
    not_an_emulator.edges.push(serde_json::from_value(json!({ "from": "flip", "to": "end", "port": "next" })).unwrap());
    assert_eq!(refused(not_an_emulator).await.code, "emulator.node_unknown");

    let mut wild = lossy(device, relay, 2, 1);
    wild.nodes[2].kind = serde_json::from_value(json!({ "type": "impairment", "listen": format!("127.0.0.1:{relay}"), "target": format!("127.0.0.1:{device}"), "profile": { "loss": 2 } })).unwrap();
    let error = refused(wild).await;
    assert_eq!((error.code.as_str(), error.field.as_ref().map(|field| field.key.as_str())), ("node.range", Some("loss")));

    let taken = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let busy = lossy(device, taken.local_addr().unwrap().port(), 2, 1);
    let error = refused(busy).await;
    assert_eq!((error.code.as_str(), error.node.as_deref()), ("transport.address_in_use", Some("relay")));
    assert!(service.jobs().list().is_empty(), "nothing was started");
}

/// The bundled templates do what their names say, end to end.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_fault_templates_run_as_they_are() {
    let service = service();
    let phases: Experiment = serde_json::from_str(include_str!("../../experiments/templates/fault-phases.json")).unwrap();
    let result = run(&service, phases).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let profiles: Vec<&str> = result.impairments[0].phases.iter().map(|phase| phase.profile.as_str()).collect();
    assert_eq!(profiles, ["lan", "wifi", "offline", "lan"]);
    let offline = &result.impairments[0].phases[2].counts;
    // What arrives while offline is dropped; a datagram still on its way from the phase before may land in it.
    assert!(offline.dropped == offline.received && offline.received > 20 && offline.forwarded <= 2, "{offline:?}");

    let outage: Experiment = serde_json::from_str(include_str!("../../experiments/templates/dependency-outage.json")).unwrap();
    let result = run(&service, outage).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    let counts = serde_json::to_value(&result.emulators[0].counts).unwrap();
    assert!(counts["down"].as_u64().unwrap() >= 3, "the client met the outage: {counts}");
    let answered = result.steps.iter().find(|step| step.node_id == "answered" && step.state == "passed").unwrap();
    assert!(answered.detail.ends_with("HTTP 200"), "{}", answered.detail);
}
