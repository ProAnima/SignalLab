//! Milestone 4.3: an action sent again and again (Repeat) — end to end through
//! the command table, against loopback sockets.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::osc_codec::{decode_packet, encode_message, OscArg};
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::net::UdpSocket;

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-repeat-loop-{}", std::process::id()));
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

/// A UDP port that keeps every OSC message it receives, with when it came.
async fn osc_sink() -> (SocketAddr, Arc<Mutex<Vec<(Instant, String, Vec<OscArg>)>>>) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let address = socket.local_addr().unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let kept = received.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 2048];
        while let Ok((size, _)) = socket.recv_from(&mut buffer).await {
            let Ok(messages) = decode_packet(&buffer[..size]) else { continue };
            let mut kept = kept.lock().unwrap();
            for message in messages {
                kept.push((Instant::now(), message.address, message.args));
            }
        }
    });
    (address, received)
}

/// An OSC device answering every `/ping <args>` with `/pong <args>` to the sender's port.
async fn osc_device() -> (SocketAddr, Arc<AtomicUsize>) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let address = socket.local_addr().unwrap();
    let pings = Arc::new(AtomicUsize::new(0));
    let counted = pings.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 2048];
        while let Ok((size, from)) = socket.recv_from(&mut buffer).await {
            let Ok(messages) = decode_packet(&buffer[..size]) else { continue };
            for message in messages.into_iter().filter(|message| message.address == "/ping") {
                counted.fetch_add(1, Ordering::SeqCst);
                socket.send_to(&encode_message("/pong", &message.args), from).await.unwrap();
            }
        }
    });
    (address, pings)
}

fn doc(nodes: Vec<Value>, chain: &[&str]) -> Value {
    let edges: Vec<Value> = chain.windows(2).map(|pair| json!({ "from": pair[0], "to": pair[1], "port": "next" })).collect();
    json!({ "version": 9, "name": "Repeat", "params": [], "profiles": [], "profile": null, "seed": 7, "nodes": nodes, "edges": edges })
}

fn ends() -> [Value; 2] {
    [json!({ "id": "start", "type": "start", "x": 0, "y": 0 }), json!({ "id": "end", "type": "end", "x": 900, "y": 0 })]
}

/// `/beat {{counter}}` to `target`, repeated as `repeat` says.
fn beat(target: SocketAddr, repeat: Value) -> Value {
    json!({
        "id": "beat", "type": "osc", "x": 200, "y": 0, "target": target.to_string(), "address": "/beat",
        "args": [{ "type": "str", "value": "{{counter}}" }], "repeat": repeat
    })
}

async fn run(service: &Service, recorder: &Arc<Recorder>, document: Value) -> (Value, Vec<Value>) {
    let job = service.invoke("experiment_start", json!({ "document": document })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let watcher = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || watcher.wait_for("experiment://ended", Duration::from_secs(30), |payload| payload["job_id"] == id))
        .await
        .unwrap()
        .expect("the run ended");
    let steps = recorder.payloads("experiment://step").into_iter().filter(|step| step["job_id"] == id).collect();
    (ended, steps)
}

fn of<'a>(steps: &'a [Value], node: &str, state: &str) -> Vec<&'a Value> {
    steps.iter().filter(|step| step["node_id"] == node && step["state"] == state).collect()
}

fn texts(received: &Mutex<Vec<(Instant, String, Vec<OscArg>)>>) -> Vec<String> {
    received
        .lock()
        .unwrap()
        .iter()
        .map(|(_, _, args)| match args.first() {
            Some(OscArg::Str(text)) => text.clone(),
            other => format!("{other:?}"),
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_repeated_send_goes_out_the_given_number_of_times_each_with_its_own_number() {
    let (service, recorder) = service();
    let (sink, received) = osc_sink().await;
    let [start, end] = ends();
    let started = Instant::now();
    let (ended, steps) = run(&service, &recorder, doc(vec![start, beat(sink, json!({ "until": "count", "count": 5, "interval_ms": 60 })), end], &["start", "beat", "end"])).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(texts(&received), ["1", "2", "3", "4", "5"], "five sends, {{counter}} read afresh for each");
    assert!(started.elapsed() >= Duration::from_millis(4 * 60), "four pauses between five sends");
    let passed = of(&steps, "beat", "passed");
    assert_eq!(passed.len(), 1, "the step passes once, after the last send");
    assert_eq!(passed[0]["message_key"], "exp.step.repeated");
    assert_eq!(passed[0]["message_params"]["n"], 5);
    assert_eq!(of(&steps, "end", "passed").len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_send_repeated_for_a_time_stops_when_the_time_is_up_and_reports_its_progress() {
    let (service, recorder) = service();
    let (sink, received) = osc_sink().await;
    let [start, end] = ends();
    let repeat = json!({ "until": "duration", "duration_ms": 1300, "interval_ms": 100 });
    let started = Instant::now();
    let (ended, steps) = run(&service, &recorder, doc(vec![start, beat(sink, repeat), end], &["start", "beat", "end"])).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let elapsed = started.elapsed();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let sent = received.lock().unwrap().len();
    // Sends at 0, 100 … 1200 ms: 13 on time, fewer when the timer runs late — never one past the end.
    assert!((10..=13).contains(&sent), "{sent} sends in 1.3 s");
    assert!(elapsed < Duration::from_millis(1300 + 500), "no send after the time is up ({elapsed:?})");
    let progress = of(&steps, "beat", "repeating");
    assert!(!progress.is_empty(), "a step that repeats for over a second reports how far it is");
    assert_eq!(progress[0]["message_key"], "exp.step.repeatingFor");
    assert_eq!(of(&steps, "beat", "passed")[0]["message_params"]["n"], sent);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stop_ends_a_repeat_in_its_pause_and_nothing_is_sent_after() {
    let (service, recorder) = service();
    let (sink, received) = osc_sink().await;
    let [start, end] = ends();
    let document = doc(vec![start, beat(sink, json!({ "until": "count", "count": 1000, "interval_ms": 50 })), end], &["start", "beat", "end"]);
    let job = service.invoke("experiment_start", json!({ "document": document })).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    service.invoke("job_stop", json!({ "id": job["id"] })).await.unwrap();
    let stopped_at = received.lock().unwrap().len();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let after = received.lock().unwrap().len();
    assert!((2..1000).contains(&stopped_at), "it was sending ({stopped_at})");
    assert!(after <= stopped_at + 1, "nothing more after Stop ({stopped_at} → {after})");
    assert!(recorder.payloads("experiment://step").iter().all(|step| step["state"] != "failed"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_repetition_of_a_send_that_expects_a_reply_waits_for_its_own() {
    let (service, recorder) = service();
    let (device, pings) = osc_device().await;
    let [start, end] = ends();
    let poll = json!({
        "id": "poll", "type": "osc", "x": 200, "y": 0, "target": device.to_string(), "address": "/ping",
        "args": [{ "type": "str", "value": "n{{counter}}" }],
        "reply": { "bind": "127.0.0.1:0", "address": "/pong", "args": [{ "index": 0, "op": "eq", "value": "n{{counter}}" }], "timeout_ms": 1000 },
        "repeat": { "until": "count", "count": 3, "interval_ms": 20 }
    });
    let (ended, steps) = run(&service, &recorder, doc(vec![start, poll, end], &["start", "poll", "end"])).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!(pings.load(Ordering::SeqCst), 3);
    let passed = of(&steps, "poll", "passed");
    assert_eq!(passed[0]["vars"]["reply"]["args"], json!(["n3"]), "the variable holds the last reply");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_repeat_where_one_does_not_belong_is_refused_before_the_run() {
    let (service, _) = service();
    let [start, end] = ends();
    let wait = json!({ "id": "wait", "type": "wait_osc", "x": 200, "y": 0, "bind": "127.0.0.1:9901", "address": "/x", "repeat": { "until": "count", "count": 3, "interval_ms": 100 } });
    let refused = service.invoke("experiment_validate", json!({ "document": doc(vec![start.clone(), wait, end.clone()], &["start", "wait", "end"]) })).await.unwrap_err();
    let refused = serde_json::to_value(refused).unwrap();
    assert_eq!((refused["code"].as_str(), refused["node"].as_str()), (Some("node.repeat_unsupported"), Some("wait")), "{refused}");

    let sink: SocketAddr = "127.0.0.1:9".parse().unwrap();
    let cases = [
        (json!({ "until": "count", "count": 1, "interval_ms": 100 }), "field.repeat_count"),
        (json!({ "until": "count", "count": 5, "interval_ms": 5 }), "field.repeat_interval"),
        (json!({ "until": "count", "count": 100, "interval_ms": 5000 }), "node.repeat_too_long"),
        (json!({ "until": "duration", "duration_ms": 300_000, "interval_ms": 10 }), "node.repeat_too_many"),
    ];
    for (repeat, expected) in cases {
        let document = doc(vec![start.clone(), beat(sink, repeat.clone()), end.clone()], &["start", "beat", "end"]);
        let refused = serde_json::to_value(service.invoke("experiment_validate", json!({ "document": document })).await.unwrap_err()).unwrap();
        let got = match expected.strip_prefix("field.") {
            Some(field) => refused["code"] == "node.range" && refused["field"]["key"] == field,
            None => refused["code"] == expected,
        };
        assert!(got, "{repeat} → {refused}");
    }
}

// ---- Loop --------------------------------------------------------------------

/// A device answering `/status` to the sender's port: "busy" `busy` times, then "ready".
async fn status_device(busy: usize) -> (SocketAddr, Arc<AtomicUsize>) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let address = socket.local_addr().unwrap();
    let polls = Arc::new(AtomicUsize::new(0));
    let counted = polls.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 2048];
        while let Ok((size, from)) = socket.recv_from(&mut buffer).await {
            let Ok(messages) = decode_packet(&buffer[..size]) else { continue };
            for _ in messages.into_iter().filter(|message| message.address == "/status") {
                let state = if counted.fetch_add(1, Ordering::SeqCst) < busy { "busy" } else { "ready" };
                socket.send_to(&encode_message("/status", &[OscArg::Str(state.into())]), from).await.unwrap();
            }
        }
    });
    (address, polls)
}

fn graph(nodes: Vec<Value>, edges: &[(&str, &str, &str)]) -> Value {
    let edges: Vec<Value> = edges.iter().map(|(from, port, to)| json!({ "from": from, "to": to, "port": port })).collect();
    json!({ "version": 9, "name": "Loop", "params": [], "profiles": [], "profile": null, "seed": 7, "nodes": nodes, "edges": edges })
}

/// Start → Loop (≤ max, until `{{status.args[0]}} = ready`) → body: poll the device → back;
/// Done → "ready" log; Limit → "gave up" log (when `limit`) → End.
fn poll_until_ready(device: SocketAddr, max: u32, limit: bool) -> Value {
    let [start, end] = ends();
    let mut nodes = vec![
        start,
        json!({ "id": "loop", "type": "loop", "x": 200, "y": 0, "max": max, "until": { "value": "{{status.args[0]}}", "op": "eq", "expected": "ready" } }),
        json!({ "id": "poll", "type": "osc", "x": 400, "y": 0, "target": device.to_string(), "address": "/status", "args": [{ "type": "str", "value": "#{{counter}}" }],
                "reply": { "bind": "127.0.0.1:0", "address": "/status", "timeout_ms": 1000, "variable": "status" } }),
        json!({ "id": "ready", "type": "log", "x": 600, "y": 0, "message": "{{status.args[0]}}" }),
        end,
    ];
    let mut edges = vec![("start", "next", "loop"), ("loop", "body", "poll"), ("poll", "next", "loop"), ("loop", "done", "ready"), ("ready", "next", "end")];
    if limit {
        nodes.push(json!({ "id": "gave_up", "type": "log", "x": 600, "y": 120, "message": "still {{status.args[0]}}" }));
        edges.extend([("loop", "limit", "gave_up"), ("gave_up", "next", "end")]);
    }
    graph(nodes, &edges)
}

/// The beat node without its repeat: a loop sends it once per iteration.
fn single_beat(sink: SocketAddr) -> Value {
    let mut node = beat(sink, Value::Null);
    node.as_object_mut().unwrap().remove("repeat");
    node
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_loop_polls_until_the_device_is_ready_and_what_follows_reads_the_last_answer() {
    let (service, recorder) = service();
    let (device, polls) = status_device(2).await;
    let (ended, steps) = run(&service, &recorder, poll_until_ready(device, 5, false)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!(polls.load(Ordering::SeqCst), 3, "busy, busy, ready");
    let loops = of(&steps, "loop", "passed");
    let keys: Vec<&str> = loops.iter().map(|step| step["message_key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["exp.step.loopIteration", "exp.step.loopIteration", "exp.step.loopIteration", "exp.step.loopDone"]);
    let iterations: Vec<u64> = loops[..3].iter().map(|step| step["message_params"]["n"].as_u64().unwrap()).collect();
    assert_eq!(iterations, [1, 2, 3]);
    assert_eq!(loops[3]["message_params"]["n"], 3);
    assert_eq!(of(&steps, "ready", "passed")[0]["detail"], "ready", "Done knows what the body set");
    let answers: Vec<&str> = of(&steps, "poll", "passed").iter().map(|step| step["vars"]["status"]["args"][0].as_str().unwrap()).collect();
    assert_eq!(answers, ["busy", "busy", "ready"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_loop_whose_condition_never_holds_leaves_through_limit_or_fails_without_it() {
    let (service, recorder) = service();
    let (device, polls) = status_device(usize::MAX).await;
    let (ended, steps) = run(&service, &recorder, poll_until_ready(device, 3, true)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!(polls.load(Ordering::SeqCst), 3, "the body ran max times");
    assert_eq!(of(&steps, "loop", "passed").last().unwrap()["message_key"], "exp.step.loopLimit");
    assert_eq!(of(&steps, "gave_up", "passed")[0]["detail"], "still busy");
    assert!(of(&steps, "ready", "passed").is_empty());

    let (device, _) = status_device(usize::MAX).await;
    let (ended, _) = run(&service, &recorder, poll_until_ready(device, 2, false)).await;
    assert_eq!((ended["error"]["code"].as_str(), ended["error"]["node"].as_str()), (Some("loop.limit"), Some("loop")), "{ended}");
    assert_eq!(ended["error"]["params"]["max"], "2");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_loop_without_a_condition_runs_its_body_max_times_and_counter_numbers_the_iterations() {
    let (service, recorder) = service();
    let (sink, received) = osc_sink().await;
    let [start, end] = ends();
    let document = graph(
        vec![start, json!({ "id": "loop", "type": "loop", "x": 200, "y": 0, "max": 4 }), single_beat(sink), end],
        &[("start", "next", "loop"), ("loop", "body", "beat"), ("beat", "next", "loop"), ("loop", "done", "end")],
    );
    let (ended, steps) = run(&service, &recorder, document).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(texts(&received), ["1", "2", "3", "4"]);
    assert_eq!(of(&steps, "loop", "passed").last().unwrap()["message_key"], "exp.step.loopFinished");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stop_ends_a_loop_between_iterations() {
    let (service, _) = service();
    let (sink, received) = osc_sink().await;
    let [start, end] = ends();
    let document = graph(
        vec![start, json!({ "id": "loop", "type": "loop", "x": 200, "y": 0, "max": 1000 }), single_beat(sink),
             json!({ "id": "pause", "type": "delay", "x": 400, "y": 120, "ms": 40 }), end],
        &[("start", "next", "loop"), ("loop", "body", "beat"), ("beat", "next", "pause"), ("pause", "next", "loop"), ("loop", "done", "end")],
    );
    let job = service.invoke("experiment_start", json!({ "document": document })).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    service.invoke("job_stop", json!({ "id": job["id"] })).await.unwrap();
    let stopped_at = received.lock().unwrap().len();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(stopped_at >= 2, "it was looping ({stopped_at})");
    assert!(received.lock().unwrap().len() <= stopped_at + 1, "no iteration after Stop");
}

/// The code and node of a refused document.
async fn refusal(service: &Service, document: Value) -> (String, String) {
    let refused = serde_json::to_value(service.invoke("experiment_validate", json!({ "document": document })).await.unwrap_err()).unwrap();
    (refused["code"].as_str().unwrap_or_default().to_string(), refused["node"].as_str().unwrap_or_default().to_string())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_loop_body_runs_as_one_branch_that_comes_back_and_nothing_else_may_cycle() {
    let (service, _) = service();
    let [start, end] = ends();
    let node = |id: &str, kind: &str| match kind {
        "branch_value" => json!({ "id": id, "type": kind, "x": 100, "y": 100, "value": "1", "op": "eq", "expected": "1" }),
        "fork" => json!({ "id": id, "type": kind, "x": 100, "y": 100 }),
        _ => json!({ "id": id, "type": "log", "x": 100, "y": 100, "message": id }),
    };
    let lp = json!({ "id": "loop", "type": "loop", "x": 200, "y": 0, "max": 3 });
    let base = |extra: Vec<Value>, edges: &[(&str, &str, &str)]| {
        let mut nodes = vec![start.clone(), lp.clone(), end.clone()];
        nodes.extend(extra);
        graph(nodes, edges)
    };
    let fine = base(vec![node("a", "log")], &[("start", "next", "loop"), ("loop", "body", "a"), ("a", "next", "loop"), ("loop", "done", "end")]);
    assert!(service.invoke("experiment_validate", json!({ "document": fine })).await.is_ok(), "a body that comes back is a loop, not a cycle");

    let cases: Vec<(Value, (&str, &str))> = vec![
        (base(vec![node("a", "log")], &[("start", "next", "loop"), ("loop", "body", "a"), ("a", "next", "end"), ("loop", "done", "end")]), ("loop.no_return", "loop")),
        (base(vec![node("a", "branch_value"), node("b", "log")], &[("start", "next", "loop"), ("loop", "body", "a"), ("a", "yes", "loop"), ("a", "no", "b"), ("b", "next", "end"), ("loop", "done", "end")]), ("loop.body_leaves", "a")),
        (base(vec![node("a", "log"), node("b", "log")], &[("start", "next", "b"), ("b", "next", "loop"), ("loop", "body", "a"), ("a", "next", "loop"), ("loop", "done", "a")]), ("loop.body_entered", "loop")),
        (base(vec![node("a", "log"), node("b", "log")], &[("start", "next", "loop"), ("loop", "body", "a"), ("a", "next", "loop"), ("a", "next", "b"), ("b", "next", "loop"), ("loop", "done", "end")]), ("loop.body_parallel", "a")),
        (base(vec![node("f", "fork"), node("a", "log"), node("b", "log")], &[("start", "next", "loop"), ("loop", "body", "f"), ("f", "branch1", "a"), ("f", "branch2", "b"), ("a", "next", "loop"), ("b", "next", "loop"), ("loop", "done", "end")]), ("loop.body_unsupported", "f")),
        (graph(vec![start.clone(), end.clone(), node("a", "log"), node("b", "branch_value")], &[("start", "next", "a"), ("a", "next", "b"), ("b", "yes", "a"), ("b", "no", "end")]), ("graph.cycle", "a")),
    ];
    for (document, (code, at)) in cases {
        let (got_code, got_node) = refusal(&service, document.clone()).await;
        assert_eq!((got_code.as_str(), got_node.as_str()), (code, at), "{}", document["edges"]);
    }

    // The exit condition and what follows Done may read what the body sets; nothing else may.
    let set = json!({ "id": "set", "type": "wait_osc", "x": 0, "y": 0, "bind": "127.0.0.1:9902", "address": "/x", "variable": "got" });
    let with = |until: &str, after: &str| {
        let mut lp = lp.clone();
        lp["until"] = json!({ "value": until, "op": "not_empty" });
        graph(
            vec![start.clone(), lp, set.clone(), json!({ "id": "after", "type": "log", "x": 0, "y": 0, "message": after }), end.clone()],
            &[("start", "next", "loop"), ("loop", "body", "set"), ("set", "matched", "loop"), ("loop", "done", "after"), ("loop", "limit", "after"), ("after", "next", "end")],
        )
    };
    assert!(service.invoke("experiment_validate", json!({ "document": with("{{got.address}}", "{{got.from}}") })).await.is_ok());
    assert_eq!(refusal(&service, with("{{nothing}}", "x")).await, ("name.unknown".to_string(), "loop".to_string()));
    assert_eq!(refusal(&service, with("{{got}}", "{{missing}}")).await, ("name.unknown".to_string(), "after".to_string()));
}
