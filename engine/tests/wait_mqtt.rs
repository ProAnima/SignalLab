//! Wait for MQTT end to end: an experiment publishes a ping through a broker
//! and waits for the device's answer on a subscription the run opened before
//! its first step. The broker here is a small fake that also plays the device.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-wait-mqtt-{}", std::process::id()));
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

// ---- a fake broker -----------------------------------------------------------

fn remaining(mut length: usize) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (length % 128) as u8;
        length /= 128;
        if length > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if length == 0 {
            return out;
        }
    }
}

fn publish_packet(topic: &str, payload: &[u8], retain: bool) -> Vec<u8> {
    let mut body = (topic.len() as u16).to_be_bytes().to_vec();
    body.extend_from_slice(topic.as_bytes());
    body.extend_from_slice(payload);
    let mut packet = vec![0x30 | u8::from(retain)];
    packet.extend(remaining(body.len()));
    packet.extend(body);
    packet
}

/// One MQTT packet: its first byte and body.
async fn read_packet(stream: &mut tokio::net::tcp::OwnedReadHalf) -> Option<(u8, Vec<u8>)> {
    let first = stream.read_u8().await.ok()?;
    let (mut length, mut shift) = (0usize, 0);
    loop {
        let byte = stream.read_u8().await.ok()?;
        length |= ((byte & 0x7f) as usize) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    let mut body = vec![0u8; length];
    stream.read_exact(&mut body).await.ok()?;
    Some((first, body))
}

fn filter_matches(filter: &str, topic: &str) -> bool {
    let (filter, topic): (Vec<&str>, Vec<&str>) = (filter.split('/').collect(), topic.split('/').collect());
    for (index, level) in filter.iter().enumerate() {
        match *level {
            "#" => return true,
            "+" if index < topic.len() => {}
            level if topic.get(index) == Some(&level) => {}
            _ => return false,
        }
    }
    filter.len() == topic.len()
}

/// Each subscriber's filter and the channel to its connection.
type Subscribers = Arc<Mutex<Vec<(String, mpsc::UnboundedSender<Vec<u8>>)>>>;

struct Broker {
    address: SocketAddr,
    /// Publishes it received from clients, by topic.
    received: Arc<Mutex<Vec<String>>>,
    subscribed: Arc<AtomicUsize>,
}

/// Answers CONNECT, SUBSCRIBE (refusing `refuse/#`, replaying a retained
/// `lab/pong` = `pong stale`) and PINGREQ, forwards publishes to subscribers —
/// and, as the device, answers a publish on `lab/ping` with `lab/pong` =
/// `pong <payload>` after `answer_after`, unless that is `None`.
async fn broker(answer_after: Option<Duration>) -> Broker {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let subscribers: Subscribers = Arc::default();
    let received: Arc<Mutex<Vec<String>>> = Arc::default();
    let subscribed = Arc::new(AtomicUsize::new(0));
    let (state, log, count) = (subscribers.clone(), received.clone(), subscribed.clone());
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let (mut reader, mut writer) = stream.into_split();
            let (to_client, mut outgoing) = mpsc::unbounded_channel::<Vec<u8>>();
            tokio::spawn(async move {
                while let Some(bytes) = outgoing.recv().await {
                    if writer.write_all(&bytes).await.is_err() {
                        break;
                    }
                }
            });
            let (state, log, count) = (state.clone(), log.clone(), count.clone());
            tokio::spawn(async move {
                while let Some((first, body)) = read_packet(&mut reader).await {
                    match first >> 4 {
                        1 => drop(to_client.send(vec![0x20, 0x02, 0x00, 0x00])),
                        8 => {
                            let id = [body[0], body[1]];
                            let length = u16::from_be_bytes([body[2], body[3]]) as usize;
                            let filter = String::from_utf8_lossy(&body[4..4 + length]).to_string();
                            let refused = filter.starts_with("refuse");
                            drop(to_client.send(vec![0x90, 0x03, id[0], id[1], if refused { 0x80 } else { 0x00 }]));
                            if !refused {
                                count.fetch_add(1, Ordering::SeqCst);
                                // What a real broker does for a retained value: replay it, flagged.
                                drop(to_client.send(publish_packet("lab/pong", b"pong stale", true)));
                                state.lock().unwrap().push((filter, to_client.clone()));
                            }
                        }
                        3 => {
                            let length = u16::from_be_bytes([body[0], body[1]]) as usize;
                            let topic = String::from_utf8_lossy(&body[2..2 + length]).to_string();
                            let payload = body[2 + length..].to_vec();
                            log.lock().unwrap().push(topic.clone());
                            let forward = |topic: &str, payload: &[u8]| {
                                for (filter, client) in state.lock().unwrap().iter() {
                                    if filter_matches(filter, topic) {
                                        drop(client.send(publish_packet(topic, payload, false)));
                                    }
                                }
                            };
                            forward(&topic, &payload);
                            if let (Some(delay), "lab/ping") = (answer_after, topic.as_str()) {
                                let state = state.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(delay).await;
                                    let answer = [b"pong ".as_slice(), &payload].concat();
                                    for (filter, client) in state.lock().unwrap().iter() {
                                        if filter_matches(filter, "lab/pong") {
                                            drop(client.send(publish_packet("lab/pong", &answer, false)));
                                        }
                                    }
                                });
                            }
                        }
                        12 => drop(to_client.send(vec![0xd0, 0x00])),
                        14 => break,
                        _ => {}
                    }
                }
            });
        }
    });
    Broker { address, received, subscribed }
}

// ---- experiments ------------------------------------------------------------------

fn doc(broker: SocketAddr, filter: &str, timeout_wired: bool, wait_timeout_ms: u64) -> Value {
    let port = broker.port();
    let mut edges = vec![
        json!({ "from": "start", "to": "ping", "port": "next" }),
        json!({ "from": "ping", "to": "pong", "port": "next" }),
        json!({ "from": "pong", "to": "end", "port": "matched" }),
    ];
    if timeout_wired {
        edges.push(json!({ "from": "pong", "to": "silent", "port": "timeout" }));
        edges.push(json!({ "from": "silent", "to": "end", "port": "next" }));
    }
    let mut document = json!({
        "version": 8, "name": "MQTT ping", "seed": null, "profile": null, "profiles": [],
        "params": [{ "name": "broker", "value": "127.0.0.1" }],
        "nodes": [
            { "id": "start", "type": "start", "x": 0, "y": 0 },
            { "id": "ping", "type": "mqtt", "x": 200, "y": 0, "host": "{{broker}}", "port": port, "topic": "lab/ping", "payload": "{{run.id}}", "qos": 0, "retain": false },
            { "id": "pong", "type": "wait_mqtt", "x": 400, "y": 0, "host": "{{broker}}", "port": port, "topic": filter, "mode": "contains", "pattern": "pong {{run.id}}", "timeout_ms": wait_timeout_ms },
            { "id": "end", "type": "end", "x": 800, "y": 0 }
        ],
        "edges": edges
    });
    if timeout_wired {
        document["nodes"].as_array_mut().unwrap().push(json!({ "id": "silent", "type": "log", "x": 600, "y": 100, "message": "no answer" }));
    }
    document
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

fn passed<'a>(steps: &'a [Value], node: &str) -> Option<&'a Value> {
    steps.iter().find(|step| step["node_id"] == node && step["state"] == "passed")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_reply_published_after_the_ping_is_matched_and_a_retained_value_is_not() {
    let (service, recorder) = service();
    let broker = broker(Some(Duration::from_millis(150))).await;
    let (ended, steps) = run(&service, &recorder, doc(broker.address, "lab/+", false, 2000)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let id = ended["job_id"].as_u64().unwrap();
    let pong = passed(&steps, "pong").expect("the wait passed");
    assert_eq!(pong["message_key"], "exp.step.matched");
    let reply = &pong["vars"]["reply"];
    assert_eq!((reply["topic"].as_str(), reply["text"].clone()), (Some("lab/pong"), json!(format!("pong {id}"))), "not the retained `pong stale`");
    assert_eq!(broker.subscribed.load(Ordering::SeqCst), 1, "one subscription, opened for the run");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn silence_follows_timeout_when_it_is_wired() {
    let (service, recorder) = service();
    let broker = broker(None).await;
    let (ended, steps) = run(&service, &recorder, doc(broker.address, "lab/pong", true, 200)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    assert_eq!(passed(&steps, "pong").unwrap()["message_key"], "exp.step.timedOut");
    assert!(passed(&steps, "silent").is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_refused_subscription_or_an_absent_broker_stops_the_run_before_any_traffic() {
    let (service, _recorder) = service();
    let broker = broker(Some(Duration::from_millis(10))).await;
    let failure = |result: Result<Value, signal_lab_engine::Failure>| serde_json::to_value(result.unwrap_err()).unwrap();
    let refused = failure(service.invoke("experiment_start", json!({ "document": doc(broker.address, "refuse/#", false, 500) })).await);
    assert_eq!((refused["code"].as_str(), refused["node"].as_str(), refused["field"]["key"].as_str()), (Some("mqtt.subscribe_refused"), Some("pong"), Some("topic")), "{refused}");
    assert!(broker.received.lock().unwrap().is_empty(), "the ping was never published");

    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    let absent = failure(service.invoke("experiment_start", json!({ "document": doc(closed, "lab/pong", false, 500) })).await);
    assert_eq!((absent["code"].as_str(), absent["node"].as_str(), absent["field"]["key"].as_str()), (Some("transport.refused"), Some("pong"), Some("broker")), "{absent}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn broker_and_topic_take_parameters_only_and_filters_follow_the_wildcard_rules() {
    let (service, _recorder) = service();
    let address: SocketAddr = "127.0.0.1:1883".parse().unwrap();
    let check = |document: Value| {
        let service = &service;
        async move {
            let result = service.invoke("experiment_validate", json!({ "document": document, "overrides": {} })).await;
            result.map(|value| value.to_string()).unwrap_or_else(|failure| serde_json::to_value(failure).unwrap().to_string())
        }
    };
    let mut generated = doc(address, "lab/pong", false, 500);
    generated["nodes"][2]["topic"] = "lab/{{uuid}}".into();
    let report = check(generated).await;
    assert!(report.contains("node.params_only") && report.contains("topic"), "{report}");
    let report = check(doc(address, "lab/#/more", false, 500)).await;
    assert!(report.contains("node.topic_filter"), "{report}");
    assert!(!check(doc(address, "lab/+/state/#", false, 500)).await.contains("code"), "a good filter with a parameter broker validates");
}
