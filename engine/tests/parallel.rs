//! Parallel work from one output: several wires out of one port run their
//! nodes at the same time, each branch with its own copy of the variables, and
//! a Join waits for every wire that leads into it.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

/// Run reports go to one scratch folder for this test binary, never to Documents.
fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-parallel-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        paths::data_dir()
    })
    .clone()
}

fn service() -> (Service, Arc<Recorder>) {
    scratch_data_dir();
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))), recorder)
}

fn delay(id: &str, ms: u64) -> Value {
    json!({ "id": id, "type": "delay", "ms": ms, "x": 300, "y": 100 })
}

fn node(id: &str, kind: &str) -> Value {
    json!({ "id": id, "type": kind, "x": 100, "y": 100 })
}

fn doc(nodes: Vec<Value>, edges: &[(&str, &str, &str)]) -> Value {
    let edges: Vec<Value> = edges.iter().map(|(from, port, to)| json!({ "from": from, "to": to, "port": port })).collect();
    json!({ "version": 6, "name": "Parallel", "params": [], "profiles": [], "profile": null, "seed": null, "nodes": nodes, "edges": edges })
}

/// Run to the end; the ended event and this run's steps.
async fn run(service: &Service, recorder: &Arc<Recorder>, document: Value) -> (Value, Vec<Value>) {
    let job = service.invoke("experiment_start", json!({ "document": document })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let watcher = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || watcher.wait_for("experiment://ended", Duration::from_secs(15), |payload| payload["job_id"] == id))
        .await
        .unwrap()
        .expect("the run ended");
    let steps = recorder.payloads("experiment://step").into_iter().filter(|step| step["job_id"] == id).collect();
    (ended, steps)
}

fn at(steps: &[Value], node: &str, state: &str) -> Vec<u64> {
    steps.iter().filter(|step| step["node_id"] == node && step["state"] == state).map(|step| step["ts"].as_u64().unwrap()).collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_output_with_three_wires_runs_them_together_and_the_join_waits_for_all() {
    let (service, recorder) = service();
    let document = doc(
        vec![node("start", "start"), delay("a", 300), delay("b", 300), delay("c", 300), node("join", "join"), node("end", "end")],
        &[("start", "next", "a"), ("start", "next", "b"), ("start", "next", "c"), ("a", "next", "join"), ("b", "next", "join"), ("c", "next", "join"), ("join", "next", "end")],
    );
    let (ended, steps) = run(&service, &recorder, document).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let started: Vec<u64> = ["a", "b", "c"].iter().map(|id| at(&steps, id, "running")[0]).collect();
    let finished: Vec<u64> = ["a", "b", "c"].iter().map(|id| at(&steps, id, "passed")[0]).collect();
    assert!(started.iter().max() < finished.iter().min(), "all three were running at once: {started:?} {finished:?}");
    assert!(finished.iter().max().unwrap() - started.iter().min().unwrap() < 700, "three 300 ms delays took one delay's time, not three");
    assert_eq!(at(&steps, "join", "passed").len(), 1, "the Join passes once, after the last branch");
    assert!(at(&steps, "join", "running")[0] >= *finished.iter().max().unwrap());
    assert_eq!(at(&steps, "end", "passed").len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn branches_may_each_end_without_a_join_and_the_run_waits_for_both() {
    let (service, recorder) = service();
    let document = doc(
        vec![node("start", "start"), delay("short", 50), delay("long", 250), node("end", "end")],
        &[("start", "next", "short"), ("start", "next", "long"), ("short", "next", "end"), ("long", "next", "end")],
    );
    let (ended, steps) = run(&service, &recorder, document).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!((at(&steps, "short", "passed").len(), at(&steps, "long", "passed").len()), (1, 1), "neither branch was cut short");
    // End is reached twice but completes the run once, after the last branch.
    assert_eq!((at(&steps, "end", "running").len(), at(&steps, "end", "passed").len()), (1, 1));
    assert!(at(&steps, "end", "passed")[0] >= at(&steps, "long", "passed")[0], "complete only when the long branch is done");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_wire_straight_into_a_join_counts_as_an_arrival_and_fork_outputs_fan_out_too() {
    let (service, recorder) = service();
    // Start feeds the Join directly and through a delay: the Join waits for the delay.
    let direct = doc(
        vec![node("start", "start"), delay("slow", 150), node("join", "join"), node("end", "end")],
        &[("start", "next", "join"), ("start", "next", "slow"), ("slow", "next", "join"), ("join", "next", "end")],
    );
    let (ended, steps) = run(&service, &recorder, direct).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!(at(&steps, "join", "passed").len(), 1);
    assert!(at(&steps, "join", "running")[0] >= at(&steps, "slow", "passed")[0], "the Join waited for the slow wire");

    // A Parallel branch whose first output has two wires: three branches meet at the Join.
    let forked = doc(
        vec![node("start", "start"), node("fork", "fork"), delay("x", 100), delay("y", 100), delay("z", 100), node("join", "join"), node("end", "end")],
        &[("start", "next", "fork"), ("fork", "branch1", "x"), ("fork", "branch1", "y"), ("fork", "branch2", "z"), ("x", "next", "join"), ("y", "next", "join"), ("z", "next", "join"), ("join", "next", "end")],
    );
    let (ended, steps) = run(&service, &recorder, forked).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!(["x", "y", "z"].map(|id| at(&steps, id, "passed").len()), [1, 1, 1]);
    assert_eq!(at(&steps, "join", "passed").len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_same_wire_twice_is_refused_before_anything_runs() {
    let (service, _recorder) = service();
    let document = doc(vec![node("start", "start"), node("end", "end")], &[("start", "next", "end"), ("start", "next", "end")]);
    let error = service.invoke("experiment_start", json!({ "document": document })).await.unwrap_err();
    let error = serde_json::to_value(error).unwrap();
    assert_eq!((error["code"].as_str(), error["node"].as_str()), (Some("doc.connection_duplicate"), Some("start")), "{error}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_branch_that_fails_after_another_reached_end_fails_the_run_and_end_never_passes() {
    let (service, recorder) = service();
    // One branch is done at once; the other fails a check after a moment.
    let failing = json!({ "id": "check", "type": "assert_value", "value": "1", "op": "eq", "expected": "2", "x": 500, "y": 200 });
    let document = doc(
        vec![node("start", "start"), delay("wait", 150), failing, node("end", "end")],
        &[("start", "next", "end"), ("start", "next", "wait"), ("wait", "next", "check"), ("check", "next", "end")],
    );
    let (ended, steps) = run(&service, &recorder, document).await;
    assert_eq!(ended["error"]["code"], "check.value_failed", "{ended}");
    assert_eq!(ended["error"]["node"], "check");
    assert!(at(&steps, "end", "passed").is_empty(), "a failed run does not show End as passed");
}

