//! A run followed to its end, as the server's `/api/run` and the command line
//! follow it: every step as it happens, then the result — the same run, events
//! and report the Run button gets.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_run::{Outcome, Progress, RunOptions};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

/// Run reports go to a scratch folder, never to the user's Documents.
fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-run-to-end-{}", std::process::id()));
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

/// Start → `middle` (wired on `port`) → End.
fn document(name: &str, middle: Value, port: &str) -> Experiment {
    let mut middle = middle;
    middle["id"] = "middle".into();
    middle["x"] = 200.into();
    middle["y"] = 80.into();
    serde_json::from_value(json!({
        "version": 9,
        "name": name,
        "params": [{ "name": "greeting", "value": "hello" }],
        "nodes": [
            { "id": "start", "type": "start", "x": 0, "y": 80 },
            middle,
            { "id": "end", "type": "end", "x": 400, "y": 80 },
        ],
        "edges": [
            { "from": "start", "to": "middle", "port": "next" },
            { "from": "middle", "to": "end", "port": port },
        ],
    }))
    .unwrap()
}

fn free_udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_followed_run_hands_over_every_step_then_the_result_and_is_the_same_run_the_app_sees() {
    let (service, recorder) = service();
    let doc = document("Greeting", json!({ "type": "log", "message": "{{greeting}} {{run.id}}" }), "next");
    let overrides = [("greeting".to_string(), "hi".to_string())].into();
    let mut handle = service.run(doc, RunOptions { overrides, seed: Some(42), limit: None }).await.unwrap();
    let id = handle.info.id;
    assert_eq!((handle.started.job_id, handle.started.seed, handle.started.overridden), (id, 42, true));
    assert_eq!(handle.info.kind, "experiment", "a job like any other");

    let mut steps = Vec::new();
    let result = loop {
        match handle.next().await.expect("the end comes before None") {
            Progress::Step(step) => steps.push(step),
            Progress::Ended(result) => break result,
        }
    };
    assert!(handle.next().await.is_none(), "nothing after the end");
    assert_eq!(result.outcome, Outcome::Passed);
    assert_eq!((result.job_id, result.seed, result.experiment.as_str()), (id, 42, "Greeting"));
    assert_eq!(result.params["greeting"], "hi", "the values the run used");
    assert!(result.error.is_none() && result.ended_ms >= result.started_ms);
    let order: Vec<(&str, &str)> = steps.iter().map(|step| (step.node_id.as_str(), step.state)).collect();
    assert_eq!(order, [("start", "running"), ("start", "passed"), ("middle", "running"), ("middle", "passed"), ("end", "running"), ("end", "passed")]);
    assert_eq!(steps[3].detail, format!("hi {id}"));
    assert_eq!(result.steps.len(), steps.len(), "the result holds the same steps");

    // The report is the one the app gets, and the app's events went out too.
    let report: Value = serde_json::from_slice(&std::fs::read(result.report_path.as_ref().unwrap()).unwrap()).unwrap();
    assert_eq!((report["outcome"].as_str(), report["seed"].as_u64(), report["ended_ms"].as_u64()), (Some("passed"), Some(42), Some(result.ended_ms)));
    let ended = recorder.payloads("experiment://ended").into_iter().find(|ended| ended["job_id"] == id).expect("experiment://ended");
    assert_eq!(ended["report_path"].as_str(), result.report_path.as_deref());
    assert_eq!(recorder.payloads("experiment://step").iter().filter(|step| step["job_id"] == id).count(), steps.len());
    assert!(service.jobs().list().is_empty(), "the job is gone");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_failure_a_time_limit_and_a_stop_each_end_the_run_their_own_way() {
    let (service, _) = service();

    // A wait that hears nothing and has no Timeout wire fails the run.
    let wait = json!({ "type": "wait_udp", "bind": format!("127.0.0.1:{}", free_udp_port()), "timeout_ms": 150 });
    let failed = service.run(document("Silent", wait, "matched"), RunOptions::default()).await.unwrap().finished().await;
    assert_eq!(failed.outcome, Outcome::Failed);
    let error = failed.error.as_ref().unwrap();
    assert_eq!((error.code.as_str(), error.node.as_deref()), ("wait.timeout", Some("middle")));
    assert!(failed.steps.iter().any(|step| step.node_id == "middle" && step.state == "failed"));
    let report: Value = serde_json::from_slice(&std::fs::read(failed.report_path.unwrap()).unwrap()).unwrap();
    assert_eq!(report["outcome"], "failed");

    // A run that outlasts its limit fails with run.timeout, naming the limit.
    let slow = document("Slow", json!({ "type": "delay", "ms": 5000 }), "next");
    let limited = service.run(slow.clone(), RunOptions { limit: Some(Duration::from_secs(1)), ..Default::default() }).await.unwrap().finished().await;
    assert_eq!(limited.outcome, Outcome::Failed);
    let error = limited.error.unwrap();
    assert_eq!((error.code.as_str(), error.params["seconds"].as_str()), ("run.timeout", "1"));
    for limit in [Duration::ZERO, Duration::from_secs(301)] {
        let refused = service.run(slow.clone(), RunOptions { limit: Some(limit), ..Default::default() }).await.err().unwrap();
        assert_eq!((refused.code.as_str(), refused.params["max"].as_str()), ("run.limit_range", "300"));
    }
    assert!(service.jobs().list().is_empty(), "a refused run never became a job");

    // Stopped from outside: what it got to, no report — as in the app.
    let mut handle = service.run(slow, RunOptions::default()).await.unwrap();
    while let Some(Progress::Step(step)) = handle.next().await {
        if step.node_id == "middle" && step.state == "running" {
            break;
        }
    }
    assert!(service.jobs().stop(handle.info.id));
    let stopped = tokio::time::timeout(Duration::from_secs(5), handle.finished()).await.expect("a stopped run ends at once");
    assert_eq!(stopped.outcome, Outcome::Stopped);
    assert!(stopped.error.is_none() && stopped.report_path.is_none());
    assert_eq!(stopped.steps.last().map(|step| (step.node_id.as_str(), step.state)), Some(("middle", "running")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn nobody_following_a_run_does_not_stop_it() {
    let (service, recorder) = service();
    let handle = service.run(document("Unwatched", json!({ "type": "delay", "ms": 200 }), "next"), RunOptions::default()).await.unwrap();
    let id = handle.info.id;
    drop(handle);
    let watcher = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || watcher.wait_for("experiment://ended", Duration::from_secs(10), |ended| ended["job_id"] == id))
        .await
        .unwrap()
        .expect("the run went on to its end");
    assert_eq!(ended["error"], Value::Null);
    assert!(PathBuf::from(ended["report_path"].as_str().unwrap()).exists(), "and saved its report");
}
