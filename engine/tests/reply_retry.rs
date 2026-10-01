//! Milestone 4.2: a send that waits for its reply in the same step, and
//! retries with pauses — end to end through the command table, against small
//! loopback devices.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::osc_codec::{decode_packet, encode_message};
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::net::UdpSocket;

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-reply-retry-{}", std::process::id()));
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

/// An OSC device: answers `/ping <args>` with `/pong <args>` to the sender's own
/// port, ignoring the first `ignore` pings. Counts every ping it receives.
async fn osc_device(ignore: usize) -> (SocketAddr, Arc<AtomicUsize>) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let address = socket.local_addr().unwrap();
    let pings = Arc::new(AtomicUsize::new(0));
    let counted = pings.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 2048];
        while let Ok((size, from)) = socket.recv_from(&mut buffer).await {
            let Ok(messages) = decode_packet(&buffer[..size]) else { continue };
            for message in messages.into_iter().filter(|message| message.address == "/ping") {
                if counted.fetch_add(1, Ordering::SeqCst) >= ignore {
                    socket.send_to(&encode_message("/pong", &message.args), from).await.unwrap();
                }
            }
        }
    });
    (address, pings)
}

fn free_udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn doc(nodes: Vec<Value>, chain: &[&str]) -> Value {
    let edges: Vec<Value> = chain.windows(2).map(|pair| json!({ "from": pair[0], "to": pair[1], "port": "next" })).collect();
    json!({ "version": 4, "name": "Reply and retry", "params": [], "profiles": [], "profile": null, "seed": null, "nodes": nodes, "edges": edges })
}

fn ends() -> [Value; 2] {
    [json!({ "id": "start", "type": "start", "x": 0, "y": 0 }), json!({ "id": "end", "type": "end", "x": 900, "y": 0 })]
}

/// `/ping {{run.id}}` that expects `/pong` carrying the same id back.
fn ping(device: SocketAddr, bind: &str, timeout_ms: u64, retry: Option<Value>) -> Value {
    let mut node = json!({
        "id": "ping", "type": "osc", "x": 200, "y": 0, "target": device.to_string(), "address": "/ping",
        "args": [{ "type": "str", "value": "{{run.id}}" }],
        "reply": { "bind": bind, "address": "/pong", "args": [{ "index": 0, "op": "eq", "value": "{{run.id}}" }], "timeout_ms": timeout_ms }
    });
    if let Some(retry) = retry {
        node["retry"] = retry;
    }
    node
}

async fn run(service: &Service, recorder: &Arc<Recorder>, document: Value) -> (Value, Vec<Value>) {
    let job = service.invoke("experiment_start", json!({ "document": document })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let watcher = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || watcher.wait_for("experiment://ended", Duration::from_secs(20), |payload| payload["job_id"] == id))
        .await
        .unwrap()
        .expect("the run ended");
    let steps = recorder.payloads("experiment://step").into_iter().filter(|step| step["job_id"] == id).collect();
    (ended, steps)
}

fn of<'a>(steps: &'a [Value], node: &str, state: &str) -> Vec<&'a Value> {
    steps.iter().filter(|step| step["node_id"] == node && step["state"] == state).collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_send_waits_for_its_reply_even_when_the_device_answers_the_senders_port() {
    let (service, recorder) = service();
    let (device, pings) = osc_device(0).await;
    let [start, end] = ends();
    // Port 0: any free port; the message goes out from it, so the answer comes back to it.
    let log = json!({ "id": "log", "type": "log", "x": 400, "y": 0, "message": "got {{reply.args[0]}} from {{reply.from}}" });
    let (ended, steps) = run(&service, &recorder, doc(vec![start, ping(device, "127.0.0.1:0", 1000, None), log, end], &["start", "ping", "log", "end"])).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let id = ended["job_id"].as_u64().unwrap();
    let replied = of(&steps, "ping", "passed")[0];
    assert_eq!(replied["message_key"], "exp.step.replied");
    assert_eq!(replied["vars"]["reply"]["args"], json!([id.to_string()]), "the reply carried this run's id");
    assert_eq!(of(&steps, "log", "passed")[0]["detail"], format!("got {id} from {device}"));
    assert_eq!(pings.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_send_is_retried_until_the_device_answers_and_every_failed_attempt_is_reported() {
    let (service, recorder) = service();
    let (device, pings) = osc_device(2).await;
    let [start, end] = ends();
    let retry = json!({ "attempts": 3, "delay_ms": 100, "backoff": "exponential" });
    let started = Instant::now();
    let (ended, steps) = run(&service, &recorder, doc(vec![start, ping(device, "127.0.0.1:0", 200, Some(retry)), end], &["start", "ping", "end"])).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let retries = of(&steps, "ping", "retry");
    assert_eq!(retries.len(), 2, "two attempts failed before the third was answered");
    assert_eq!(retries.iter().map(|step| step["message_params"]["attempt"].as_u64().unwrap()).collect::<Vec<_>>(), [1, 2]);
    assert_eq!(retries.iter().map(|step| step["message_params"]["ms"].as_u64().unwrap()).collect::<Vec<_>>(), [100, 200], "the pause doubles");
    assert_eq!(retries[0]["error"]["code"], "wait.timeout", "each retry says why the attempt failed");
    assert_eq!(of(&steps, "ping", "passed").len(), 1);
    assert_eq!(pings.load(Ordering::SeqCst), 3, "sent again on every attempt");
    assert!(started.elapsed() >= Duration::from_millis(2 * 200 + 100 + 200), "two timeouts, two pauses, then the answer");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn when_the_attempts_run_out_the_step_fails_with_the_last_reason() {
    let (service, recorder) = service();
    let (device, pings) = osc_device(usize::MAX).await;
    let [start, end] = ends();
    let retry = json!({ "attempts": 2, "delay_ms": 50 });
    let (ended, steps) = run(&service, &recorder, doc(vec![start, ping(device, "127.0.0.1:0", 150, Some(retry)), end], &["start", "ping", "end"])).await;
    assert_eq!((ended["error"]["code"].as_str(), ended["error"]["node"].as_str()), (Some("wait.timeout"), Some("ping")), "{ended}");
    assert_eq!(ended["error"]["field"]["key"], "reply_bind");
    assert_eq!((of(&steps, "ping", "retry").len(), of(&steps, "ping", "failed").len()), (1, 1));
    assert_eq!(pings.load(Ordering::SeqCst), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stop_during_a_pause_ends_the_retries() {
    let (service, recorder) = service();
    let (device, pings) = osc_device(usize::MAX).await;
    let [start, end] = ends();
    let retry = json!({ "attempts": 5, "delay_ms": 5000 });
    let job = service.invoke("experiment_start", json!({ "document": doc(vec![start, ping(device, "127.0.0.1:0", 100, Some(retry)), end], &["start", "ping", "end"]) })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let watcher = recorder.clone();
    tokio::task::spawn_blocking(move || watcher.wait_for("experiment://step", Duration::from_secs(5), |step| step["job_id"] == id && step["state"] == "retry"))
        .await
        .unwrap()
        .expect("the first attempt failed and a pause began");
    assert_eq!(service.invoke("job_stop", json!({ "id": id })).await.unwrap(), json!(true));
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(pings.load(Ordering::SeqCst), 1, "nothing was sent after Stop");
    assert_eq!(service.invoke("jobs_list", Value::Null).await.unwrap(), json!([]));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_udp_reply_to_a_fixed_port_and_a_retried_wait() {
    let (service, recorder) = service();
    // A device that answers on a port of its own choosing, not the sender's.
    let reply_port = free_udp_port();
    let device = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let device_address = device.local_addr().unwrap();
    tokio::spawn(async move {
        let mut buffer = [0u8; 256];
        while let Ok((size, _)) = device.recv_from(&mut buffer).await {
            let text = String::from_utf8_lossy(&buffer[..size]).to_string();
            device.send_to(format!("ACK {text}").as_bytes(), ("127.0.0.1", reply_port)).await.unwrap();
        }
    });
    let [start, end] = ends();
    let send = json!({
        "id": "send", "type": "udp", "x": 200, "y": 0, "target": device_address.to_string(), "text": "LIGHTS ON",
        "reply": { "bind": format!("127.0.0.1:{reply_port}"), "mode": "regex", "pattern": "^ACK (.+)$", "timeout_ms": 1000, "variable": "ack" }
    });
    let (ended, steps) = run(&service, &recorder, doc(vec![start.clone(), send, end.clone()], &["start", "send", "end"])).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!(of(&steps, "send", "passed")[0]["vars"]["ack"]["text"], "ACK LIGHTS ON");

    // A Wait without a Timeout wire, retried: nothing in the first window, a datagram in the second.
    let bind = format!("127.0.0.1:{}", free_udp_port());
    let wait = json!({ "id": "wait", "type": "wait_udp", "x": 200, "y": 0, "bind": bind, "mode": "contains", "pattern": "hello", "timeout_ms": 200, "retry": { "attempts": 2, "delay_ms": 50 } });
    let edges = json!([{ "from": "start", "to": "wait", "port": "next" }, { "from": "wait", "to": "end", "port": "matched" }]);
    let mut document = doc(vec![start, wait, end], &[]);
    document["edges"] = edges;
    let target: SocketAddr = bind.parse().unwrap();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(320)).await;
        UdpSocket::bind("127.0.0.1:0").await.unwrap().send_to(b"hello again", target).await.unwrap();
    });
    let (ended, steps) = run(&service, &recorder, document).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!((of(&steps, "wait", "retry").len(), of(&steps, "wait", "passed").len()), (1, 1));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn retry_and_reply_settings_are_checked_before_anything_is_sent() {
    let (service, _recorder) = service();
    let [start, end] = ends();
    let service = &service;
    let check = |document: Value| async move {
        let error = service.invoke("experiment_validate", json!({ "document": document, "overrides": {} })).await;
        error.map(|value| value.to_string()).unwrap_or_else(|failure| serde_json::to_value(failure).unwrap().to_string())
    };
    let delay = json!({ "id": "pause", "type": "delay", "x": 200, "y": 0, "ms": 10, "retry": { "attempts": 3, "delay_ms": 10 } });
    let report = check(doc(vec![start.clone(), delay, end.clone()], &["start", "pause", "end"])).await;
    assert!(report.contains("node.retry_unsupported") && report.contains("\"pause\""), "{report}");
    let device: SocketAddr = "127.0.0.1:9".parse().unwrap();
    let too_many = check(doc(vec![start.clone(), ping(device, "127.0.0.1:0", 100, Some(json!({ "attempts": 11, "delay_ms": 0 }))), end.clone()], &["start", "ping", "end"])).await;
    assert!(too_many.contains("node.range") && too_many.contains("attempts"), "{too_many}");
    let mut bad_bind = ping(device, "not an address", 100, None);
    bad_bind["reply"]["bind"] = "nowhere".into();
    let bind = check(doc(vec![start, bad_bind, end], &["start", "ping", "end"])).await;
    assert!(bind.contains("node.bind_invalid") && bind.contains("reply_bind"), "{bind}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn with_capture_armed_a_matched_reply_points_at_its_inspector_frame() {
    let (service, recorder) = service();
    service.invoke("inspect_set_enabled", json!({ "enabled": true })).await.unwrap();
    let (device, _pings) = osc_device(0).await;
    let [start, end] = ends();
    let (ended, steps) = run(&service, &recorder, doc(vec![start, ping(device, "127.0.0.1:0", 1000, None), end], &["start", "ping", "end"])).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let frame = of(&steps, "ping", "passed")[0]["frame"].as_u64().expect("the step names the frame of its reply");
    let snapshot = service.invoke("inspect_snapshot", json!({ "limit": 100 })).await.unwrap();
    let frames = snapshot.as_array().cloned().or_else(|| snapshot["frames"].as_array().cloned()).unwrap();
    let found = frames.iter().find(|item| item["seq"] == frame).expect("that frame is in the Inspector");
    assert_eq!((found["dir"].as_str(), found["proto"].as_str()), (Some("rx"), Some("osc")));
    assert!(found["summary"].as_str().unwrap().starts_with("/pong"), "{found}");
    service.invoke("inspect_set_enabled", json!({ "enabled": false })).await.unwrap();
}

