//! Milestone 7: an HTTP node under load against a loopback server — a ramp
//! that fails its p95 threshold for the right reason, a load that holds and
//! reports its progress, failures counted by status and cause, workers that
//! cannot keep up missing requests rather than sending them late, Stop, and
//! what validation refuses.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_run::{Outcome, RunEvent, RunOptions, RunResult};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::load::LoadMetrics;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::service::Failure;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-load-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        paths::data_dir()
    })
    .clone()
}

fn service() -> Service {
    scratch_data_dir();
    Service::new(Host::new(Recorder::new(), Capture::new()), Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None)))
}

/// What the server does with its n-th request (from 1): how long it waits, which status it answers.
type Behaviour = fn(u64) -> (Duration, u16);

/// An HTTP/1.1 server on loopback, keep-alive, GETs only: each request answered
/// as `behaviour` says for its number. Keeps every request line it got.
async fn server(behaviour: Behaviour) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let lines = Arc::new(Mutex::new(Vec::new()));
    let (kept, count) = (lines.clone(), Arc::new(AtomicU64::new(0)));
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else { return };
            let (kept, count) = (kept.clone(), count.clone());
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let Ok(read) = stream.read(&mut chunk).await else { return };
                    if read == 0 {
                        return;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                    while let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                        let head: Vec<u8> = buffer.drain(..end + 4).collect();
                        let line = String::from_utf8_lossy(&head).lines().next().unwrap_or_default().to_string();
                        kept.lock().unwrap().push(line);
                        let (delay, status) = behaviour(count.fetch_add(1, Ordering::SeqCst) + 1);
                        tokio::time::sleep(delay).await;
                        let answer = format!("HTTP/1.1 {status} X\r\ncontent-length: 2\r\n\r\nok");
                        if stream.write_all(answer.as_bytes()).await.is_err() {
                            return;
                        }
                    }
                }
            });
        }
    });
    (url, lines)
}

fn document(nodes: Vec<Value>) -> Experiment {
    let ids: Vec<&str> = nodes.iter().map(|node| node["id"].as_str().unwrap()).collect();
    let edges: Vec<Value> = ids.windows(2).map(|pair| json!({ "from": pair[0], "to": pair[1], "port": "next" })).collect();
    serde_json::from_value(json!({ "version": 9, "name": "Load", "params": [], "profiles": [], "profile": null, "seed": 5, "nodes": nodes, "edges": edges })).unwrap()
}

fn ends() -> [Value; 2] {
    [json!({ "id": "start", "type": "start", "x": 0, "y": 0 }), json!({ "id": "end", "type": "end", "x": 900, "y": 0 })]
}

fn http(id: &str, url: &str, load: Value) -> Value {
    json!({ "id": id, "type": "http", "x": 200, "y": 0, "request": { "method": "GET", "url": url, "timeout_ms": 5000 }, "load": load })
}

async fn run(service: &Service, doc: Experiment) -> RunResult {
    let handle = service.run(doc, RunOptions::default()).await.unwrap();
    tokio::time::timeout(Duration::from_secs(60), handle.finished()).await.expect("the run ends")
}

fn steps<'a>(result: &'a RunResult, node: &str, state: &str) -> Vec<&'a RunEvent> {
    result.steps.iter().filter(|step| step.node_id == node && step.state == state).collect()
}

/// What the node's last event measured.
fn measured<'a>(result: &'a RunResult, node: &str) -> &'a LoadMetrics {
    result.steps.iter().rev().find(|step| step.node_id == node && step.load.is_some()).and_then(|step| step.load.as_deref()).expect("the load's numbers on its last event")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_ramp_fails_its_p95_threshold_for_the_right_reason() {
    let service = service();
    // One request in five is slow: the p95 is the slow ones', the errors none.
    let (url, lines) = server(|n| if n % 5 == 0 { (Duration::from_millis(150), 200) } else { (Duration::from_millis(1), 200) }).await;
    let [start, end] = ends();
    let load = json!({
        "profile": { "shape": "ramp", "from": 20, "to": 100, "duration_ms": 1500 },
        "concurrency": 64,
        "thresholds": [{ "metric": "error_rate", "op": "lt", "value": 1 }, { "metric": "p95_ms", "op": "lt", "value": 100 }]
    });
    let began = Instant::now();
    let result = run(&service, document(vec![start, http("load", &format!("{url}/ramp"), load), end])).await;
    assert_eq!(result.outcome, Outcome::Failed);
    let error = result.error.as_ref().unwrap();
    assert_eq!((error.code.as_str(), error.node.as_deref(), error.params["metric"].as_str(), error.params["op"].as_str(), error.params["value"].as_str()), ("load.threshold", Some("load"), "p95_ms", "<", "100"));
    assert_eq!(error.field.as_ref().map(|field| (field.key.as_str(), field.index)), Some(("threshold", Some(2))), "the second threshold");
    let actual: f64 = error.params["actual"].parse().unwrap();
    assert!(actual >= 140.0, "p95 is a slow request's: {actual}");

    let failed = steps(&result, "load", "failed");
    assert_eq!(failed.len(), 1);
    let metrics = failed[0].load.as_deref().expect("the failed step carries what the load measured");
    assert_eq!((metrics.planned, metrics.sent, metrics.ok, metrics.failed, metrics.missed), (90, 90, 90, 0, 0), "20 → 100/s over 1.5 s is 90 requests");
    assert_eq!(lines.lock().unwrap().len(), 90);
    assert_eq!(metrics.statuses, BTreeMap::from([("200".to_string(), 90)]));
    assert_eq!(metrics.thresholds.iter().map(|verdict| (verdict.held, verdict.actual == 0.0)).collect::<Vec<_>>(), [(true, true), (false, false)]);
    assert!((50.0..=70.0).contains(&metrics.rps), "90 in 1.5 s: {}", metrics.rps);
    assert_eq!(metrics.seconds.iter().map(|second| second.sent).sum::<u64>(), 90);
    assert!(metrics.seconds[1].sent as f64 / 0.5 > metrics.seconds[0].sent as f64 * 1.5, "the rate climbs: {:?}", metrics.seconds);
    let slow: u64 = metrics.histogram.iter().filter(|bin| bin.upto_ms.is_none_or(|upto| upto > 100.0)).map(|bin| bin.count).sum();
    assert_eq!(slow, 18, "every fifth of 90 took over 100 ms");
    assert!(began.elapsed() < Duration::from_secs(5), "{:?}", began.elapsed());

    let report: Value = serde_json::from_str(&std::fs::read_to_string(result.report_path.as_deref().unwrap()).unwrap()).unwrap();
    assert_eq!(report["version"], 5);
    let reported = report["steps"].as_array().unwrap().iter().find(|step| step["node_id"] == "load" && step["state"] == "failed").unwrap();
    assert_eq!((reported["load"]["planned"].as_u64(), reported["load"]["thresholds"][1]["metric"].as_str()), (Some(90), Some("p95_ms")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_load_that_holds_passes_with_its_numbers_and_says_how_far_it_has_got() {
    let service = service();
    let (url, lines) = server(|_| (Duration::from_millis(1), 200)).await;
    let [start, end] = ends();
    let load = json!({
        "profile": { "shape": "constant", "rate": 40, "duration_ms": 2200 },
        "concurrency": 8,
        "thresholds": [{ "metric": "p95_ms", "op": "lt", "value": 500 }, { "metric": "error_rate", "op": "lt", "value": 1 }, { "metric": "rps", "op": "ge", "value": 30 }]
    });
    let after = json!({ "id": "after", "type": "log", "x": 400, "y": 0, "message": "after the load" });
    let result = run(&service, document(vec![start, http("load", &format!("{url}/load?n={{{{counter}}}}"), load), after, end])).await;
    assert_eq!(result.outcome, Outcome::Passed, "{:?}", result.error);

    let passed = steps(&result, "load", "passed");
    assert_eq!(passed.len(), 1);
    assert_eq!((passed[0].message_key, passed[0].message_params["sent"].as_u64()), (Some("exp.step.loaded"), Some(88)));
    let metrics = measured(&result, "load");
    assert_eq!((metrics.planned, metrics.sent, metrics.failed, metrics.missed, metrics.error_rate), (88, 88, 0, 0, 0.0));
    assert!(metrics.thresholds.iter().all(|verdict| verdict.held), "{:?}", metrics.thresholds);
    assert_eq!(metrics.seconds.len(), 3, "2.2 s: two whole seconds and a part");
    assert_eq!(metrics.received_bytes, 88 * 2);

    let progress = steps(&result, "load", "load");
    assert!(progress.len() >= 2, "a report every second: {}", progress.len());
    let sent: Vec<u64> = progress.iter().map(|step| step.message_params["sent"].as_u64().unwrap()).collect();
    assert!(sent.windows(2).all(|pair| pair[0] <= pair[1]) && sent[0] > 0, "{sent:?}");
    assert_eq!(progress[0].message_key, Some("exp.step.loading"));

    let lines = lines.lock().unwrap();
    assert_eq!(lines.len(), 88);
    assert!(lines.iter().all(|line| line == "GET /load?n=1 HTTP/1.1"), "the templates are read once, as the step starts: {:?}", &lines[..3]);
    assert_eq!(steps(&result, "after", "passed").len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn failures_are_counted_by_status_and_by_cause() {
    let service = service();
    let (url, _) = server(|n| (Duration::from_millis(1), if n % 2 == 0 { 503 } else { 200 })).await;
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    let [start, end] = ends();
    let profile = json!({ "shape": "constant", "rate": 20, "duration_ms": 500 });
    let flaky = http("flaky", &format!("{url}/flaky"), json!({ "profile": profile, "concurrency": 16 }));
    let nobody = http("nobody", &format!("http://{closed}/"), json!({ "profile": profile, "concurrency": 16, "thresholds": [{ "metric": "error_rate", "op": "lt", "value": 1 }] }));
    let result = run(&service, document(vec![start, flaky, nobody, end])).await;

    let flaky = measured(&result, "flaky");
    assert_eq!(steps(&result, "flaky", "passed").len(), 1, "without thresholds a load measures and passes");
    assert_eq!((flaky.sent, flaky.ok, flaky.failed, flaky.error_rate), (10, 5, 5, 50.0));
    assert_eq!(flaky.statuses, BTreeMap::from([("200".to_string(), 5), ("503".to_string(), 5)]));

    let error = result.error.as_ref().unwrap();
    assert_eq!((error.code.as_str(), error.node.as_deref(), error.params["metric"].as_str(), error.params["actual"].as_str()), ("load.threshold", Some("nobody"), "error_rate", "100"));
    let nobody = measured(&result, "nobody");
    assert_eq!((nobody.sent, nobody.failed), (10, 10));
    assert_eq!(nobody.statuses, BTreeMap::from([("refused".to_string(), 10)]), "no status: the cause");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn workers_that_cannot_keep_up_miss_requests_instead_of_sending_them_late() {
    let service = service();
    let (url, lines) = server(|_| (Duration::from_millis(100), 200)).await;
    let [start, end] = ends();
    // One worker, 100 ms an answer: about ten a second, of fifty due.
    let load = json!({
        "profile": { "shape": "constant", "rate": 50, "duration_ms": 1000 },
        "concurrency": 1,
        "thresholds": [{ "metric": "missed", "op": "le", "value": 0 }]
    });
    let result = run(&service, document(vec![start, http("load", &format!("{url}/slow"), load), end])).await;
    let metrics = measured(&result, "load");
    assert_eq!(metrics.sent + metrics.missed, 50, "each request is either sent or missed: {metrics:?}");
    assert!(metrics.missed >= 30 && metrics.sent >= 5, "{} sent, {} missed", metrics.sent, metrics.missed);
    assert_eq!(lines.lock().unwrap().len() as u64, metrics.sent);
    let error = result.error.as_ref().unwrap();
    assert_eq!((error.code.as_str(), error.params["metric"].as_str(), error.params["op"].as_str()), ("load.threshold", "missed", "≤"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stop_ends_a_load_at_once() {
    let service = service();
    let (url, lines) = server(|_| (Duration::from_millis(1), 200)).await;
    let [start, end] = ends();
    let load = json!({ "profile": { "shape": "constant", "rate": 100, "duration_ms": 20_000 }, "concurrency": 8 });
    let handle = service.run(document(vec![start, http("load", &format!("{url}/long"), load), end]), RunOptions::default()).await.unwrap();
    let job = handle.started.job_id;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let stopped = Instant::now();
    assert_eq!(service.invoke("job_stop", json!({ "id": job })).await.unwrap(), json!(true));
    let result = tokio::time::timeout(Duration::from_secs(5), handle.finished()).await.expect("the run ends");
    assert_eq!(result.outcome, Outcome::Stopped);
    assert!(stopped.elapsed() < Duration::from_secs(2), "{:?}", stopped.elapsed());
    let sent = lines.lock().unwrap().len();
    assert!((20..=80).contains(&sent), "half a second at 100/s: {sent}");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(lines.lock().unwrap().len(), sent, "nothing is sent after Stop");
}

#[tokio::test]
async fn a_load_is_for_an_http_request_alone_and_leaves_no_response_to_check() {
    let service = service();
    let code = |result: signal_lab_engine::service::Reply| {
        let Err(Failure::Engine(error)) = result else { panic!("refused") };
        (error.code.clone(), error.node.clone().unwrap_or_default(), error.field.as_ref().map(|field| field.key.clone()).unwrap_or_default())
    };
    let constant = json!({ "profile": { "shape": "constant", "rate": 10, "duration_ms": 1000 } });
    let validate = |nodes: Vec<Value>| service.invoke("experiment_validate", json!({ "document": serde_json::to_value(document(nodes)).unwrap() }));
    let [start, end] = ends();

    let udp = json!({ "id": "udp", "type": "udp", "x": 200, "y": 0, "target": "127.0.0.1:9", "text": "x", "load": constant });
    assert_eq!(code(validate(vec![start.clone(), udp, end.clone()]).await), ("node.load_unsupported".into(), "udp".into(), "load".into()));

    let mut repeated = http("api", "http://127.0.0.1:9/", constant.clone());
    repeated["repeat"] = json!({ "count": 3, "interval_ms": 100 });
    assert_eq!(code(validate(vec![start.clone(), repeated, end.clone()]).await), ("node.load_alone".into(), "api".into(), "load".into()));

    let too_fast = http("api", "http://127.0.0.1:9/", json!({ "profile": { "shape": "constant", "rate": 200_000, "duration_ms": 1000 } }));
    assert_eq!(code(validate(vec![start.clone(), too_fast, end.clone()]).await), ("node.range".into(), "api".into(), "load_rate".into()));

    let check = json!({ "id": "check", "type": "assert_status", "x": 400, "y": 0, "status": 200 });
    let loaded = http("api", "http://127.0.0.1:9/", constant.clone());
    assert_eq!(code(validate(vec![start.clone(), loaded.clone(), check, end.clone()]).await).0, "graph.needs_http", "a load is measured, not checked");
    assert!(validate(vec![start, loaded, end]).await.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_runs_of_an_experiment_compare_and_the_slower_one_shows_its_regression() {
    let service = service();
    let (fast, _) = server(|_| (Duration::from_millis(1), 200)).await;
    let (slow, _) = server(|n| if n % 4 == 0 { (Duration::from_millis(120), 200) } else { (Duration::from_millis(1), 200) }).await;
    let compared = |url: &str| {
        let [start, end] = ends();
        let load = json!({ "profile": { "shape": "constant", "rate": 40, "duration_ms": 500 }, "concurrency": 16, "thresholds": [{ "metric": "p95_ms", "op": "lt", "value": 60 }] });
        let mut doc = document(vec![start, http("api", &format!("{url}/api"), load), end]);
        doc.name = "Compared".into();
        doc
    };
    let before = run(&service, compared(&fast)).await;
    let after = run(&service, compared(&slow)).await;
    assert_eq!((before.outcome, after.outcome), (Outcome::Passed, Outcome::Failed));
    let name = |result: &RunResult| PathBuf::from(result.report_path.as_deref().unwrap()).file_name().unwrap().to_string_lossy().to_string();

    let runs = service.invoke("experiment_runs", json!({ "name": "Compared", "limit": 10 })).await.unwrap();
    let listed: Vec<&str> = runs.as_array().unwrap().iter().map(|run| run["name"].as_str().unwrap()).collect();
    assert_eq!(listed, [name(&after), name(&before)], "this experiment's runs, newest first");
    assert_eq!((runs[0]["loads"][0]["node"].as_str(), runs[0]["loads"][0]["held"].as_bool(), runs[1]["loads"][0]["held"].as_bool()), (Some("api"), Some(false), Some(true)));

    let comparison = service.invoke("experiment_compare", json!({ "a": name(&before), "b": name(&after) })).await.unwrap();
    let step = &comparison["steps"][0];
    let p95 = step["metrics"].as_array().unwrap().iter().find(|row| row["metric"] == "p95_ms").unwrap();
    assert!(p95["b"].as_f64().unwrap() > 100.0 && p95["worse"] == true, "{p95}");
    assert_eq!((step["thresholds_a"][0]["held"].as_bool(), step["thresholds_b"][0]["held"].as_bool()), (Some(true), Some(false)));
    assert_eq!(step["sent"], json!([20, 20]));

    for (name, code) in [("../experiment.json", "runs.name_invalid"), ("run-1-404.json", "runs.not_found")] {
        let Err(Failure::Engine(error)) = service.invoke("experiment_compare", json!({ "a": name, "b": "run-1-1.json" })).await else { panic!("{name} refused") };
        assert_eq!(error.code, code, "{name}: nothing outside the runs folder, nothing that is not there");
    }
}
