//! A run at its largest: a Loop of `MAX_LOOP` iterations over a body that fills
//! the document, and layers of steps each wired to the next layer's — what the
//! timeline, a follower and the report get, and that neither grows without bound.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::experiment_flow::{MAX_BRANCHES, STEPS_KEPT, STEPS_TAIL};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-run-volume-{}", std::process::id()));
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

fn document(name: &str, nodes: Vec<Value>, edges: Vec<Value>) -> Value {
    json!({ "version": 9, "name": name, "params": [], "profiles": [], "profile": null, "seed": 7, "nodes": nodes, "edges": edges })
}

/// Start → Loop (max 1000) → `body` Log nodes → back to the Loop; Done → End.
fn big_loop(body: usize) -> Value {
    let mut nodes = vec![
        json!({ "id": "start", "type": "start", "x": 0, "y": 0 }),
        json!({ "id": "loop", "type": "loop", "x": 100, "y": 0, "max": 1000 }),
        json!({ "id": "end", "type": "end", "x": 900, "y": 0 }),
    ];
    let mut edges = vec![json!({ "from": "start", "to": "loop", "port": "next" }), json!({ "from": "loop", "to": "end", "port": "done" })];
    let mut previous = ("loop".to_string(), "body");
    for index in 0..body {
        let id = format!("log{index}");
        nodes.push(json!({ "id": id, "type": "log", "x": 200 + index * 10, "y": 100, "message": "step {{counter}}" }));
        edges.push(json!({ "from": previous.0, "to": id, "port": previous.1 }));
        previous = (id, "next");
    }
    edges.push(json!({ "from": previous.0, "to": "loop", "port": previous.1 }));
    document("Volume", nodes, edges)
}

/// Start → two Log nodes per layer, each wired to both of the next layer's → End:
/// every layer doubles the arrivals.
fn ladder(layers: usize) -> Value {
    let mut nodes = vec![json!({ "id": "start", "type": "start", "x": 0, "y": 0 }), json!({ "id": "end", "type": "end", "x": 900, "y": 0 })];
    let mut edges = Vec::new();
    let mut previous = vec!["start".to_string()];
    for layer in 0..layers {
        let ids: Vec<String> = (0..2).map(|side| format!("l{layer}s{side}")).collect();
        for id in &ids {
            nodes.push(json!({ "id": id, "type": "log", "x": 100 + layer * 10, "y": 100, "message": "x" }));
            for from in &previous {
                edges.push(json!({ "from": from, "to": id, "port": "next" }));
            }
        }
        previous = ids;
    }
    for from in &previous {
        edges.push(json!({ "from": from, "to": "end", "port": "next" }));
    }
    document("Ladder", nodes, edges)
}

async fn run(service: &Service, recorder: &Arc<Recorder>, document: Value) -> (Value, Vec<Value>) {
    let job = service.invoke("experiment_start", json!({ "document": document })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let watcher = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || watcher.wait_for("experiment://ended", Duration::from_secs(120), |payload| payload["job_id"] == id))
        .await
        .unwrap()
        .expect("the run ended");
    let steps = recorder.payloads("experiment://step").into_iter().filter(|step| step["job_id"] == id).collect();
    (ended, steps)
}

/// 124 006 steps (Start 2, the Loop 2 × 1001, 61 Logs 2 × 1000, End 2): before,
/// every one went to the timeline and the report (23.7 MB). Now the first
/// `STEPS_KEPT`, one a second and the last `STEPS_TAIL` — the gaps counted.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_largest_loop_reports_its_start_a_sample_and_its_end_with_the_gaps_counted() {
    let (service, recorder) = service();
    let started = Instant::now();
    let (ended, steps) = run(&service, &recorder, big_loop(61)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let ran = started.elapsed();
    let omitted: u64 = steps.iter().filter_map(|step| step["omitted"].as_u64()).sum();
    assert_eq!(steps.len() as u64 + omitted, 124_006, "every step is reported or counted");
    let sampled = ran.as_secs() as usize + 2;
    assert!(steps.len() <= STEPS_KEPT + sampled + STEPS_TAIL, "{} steps reported in {ran:?}", steps.len());
    assert!(steps[..STEPS_KEPT].iter().all(|step| step["omitted"].is_null()), "the first ones whole");
    let last = steps.last().unwrap();
    assert_eq!((last["node_id"].as_str(), last["message_key"].as_str()), (Some("end"), Some("exp.step.complete")), "the end is there");
    let finished = steps.iter().rev().find(|step| step["node_id"] == "loop" && step["state"] == "passed").unwrap();
    assert_eq!(finished["message_key"], "exp.step.loopFinished");

    // The report holds what the timeline got.
    let report: Value = serde_json::from_slice(&std::fs::read(ended["report_path"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(report["version"], 6);
    let kept = report["steps"].as_array().unwrap();
    assert_eq!(kept.len(), steps.len());
    assert_eq!(kept.iter().filter_map(|step| step["omitted"].as_u64()).sum::<u64>(), omitted);
    let bytes = std::fs::metadata(ended["report_path"].as_str().unwrap()).unwrap().len();
    assert!(bytes < 4 * 1024 * 1024, "{bytes} bytes");
}

/// 31 layers (64 nodes) would start 2³² branches; a run stops at
/// `MAX_BRANCHES` and says why, at once. Ten layers stay well below it.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn steps_wired_to_each_other_layer_after_layer_stop_at_the_branch_limit() {
    let (service, recorder) = service();
    let (ended, steps) = run(&service, &recorder, ladder(8)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert!(steps.iter().any(|step| step["message_key"] == "exp.step.complete"));

    let started = Instant::now();
    let (ended, steps) = run(&service, &recorder, ladder(31)).await;
    assert_eq!(ended["error"]["code"], "run.too_many_branches", "{ended}");
    assert_eq!(ended["error"]["params"]["max"], MAX_BRANCHES.to_string());
    assert!(started.elapsed() < Duration::from_secs(10), "{:?}", started.elapsed());
    assert!(steps.len() < 4 * MAX_BRANCHES + 1000, "{} steps", steps.len());
}
