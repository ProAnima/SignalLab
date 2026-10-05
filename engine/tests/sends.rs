//! The protocol screens' sends through the command table, as the API and the
//! command line reach them: a host name is a destination (IPv4 preferred, so
//! `localhost` finds a receiver on 127.0.0.1), an OSC address starts with `/`,
//! a publish on a live MQTT connection has no wildcard, and a WebSocket message
//! fits the limit — each refused with the code the steps and the one-shot
//! paths already use.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{Capture, Host, Mode, Service};
use tokio::net::UdpSocket;

fn service() -> (Service, Arc<Recorder>) {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))), recorder)
}

async fn failure(service: &Service, command: &str, args: Value) -> Value {
    serde_json::to_value(service.invoke(command, args).await.unwrap_err()).unwrap()
}

async fn receiver() -> (UdpSocket, u16) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    (socket, port)
}

async fn arrives(socket: &UdpSocket) -> Vec<u8> {
    let mut buffer = [0u8; 2048];
    let (size, _) = tokio::time::timeout(Duration::from_secs(3), socket.recv_from(&mut buffer)).await.expect("a datagram arrived").unwrap();
    buffer[..size].to_vec()
}

#[tokio::test]
async fn osc_goes_to_a_host_name_and_needs_its_slash() {
    let (service, _) = service();
    let (device, port) = receiver().await;
    let sent = service.invoke("osc_send", json!({ "target": format!("localhost:{port}"), "address": "/cue/go", "args": [] })).await.unwrap();
    assert_eq!(arrives(&device).await.len() as u64, sent.as_u64().unwrap());

    let slashless = failure(&service, "osc_send", json!({ "target": format!("127.0.0.1:{port}"), "address": "cue/go", "args": [] })).await;
    assert_eq!((slashless["code"].as_str(), slashless["params"]["value"].as_str(), slashless["field"]["key"].as_str()), (Some("node.osc_address"), Some("cue/go"), Some("address")));
    let portless = failure(&service, "osc_send", json!({ "target": "localhost", "address": "/x", "args": [] })).await;
    assert_eq!(portless["code"], "transport.target_invalid");
    let unknown = failure(&service, "osc_send", json!({ "target": "no-such-host.invalid:9", "address": "/x", "args": [] })).await;
    assert_eq!(unknown["code"], "transport.dns");

    let generator = |address: &str| json!({ "config": { "target": format!("localhost:{port}"), "address": address, "rate": 20, "waveform": "saw", "freq": 1, "min": 0, "max": 1 } });
    assert_eq!(failure(&service, "osc_generator_start", generator("lfo")).await["code"], "node.osc_address");
    let job = service.invoke("osc_generator_start", generator("/lfo")).await.unwrap();
    assert!(arrives(&device).await.starts_with(b"/lfo\0"));
    service.invoke("job_stop", json!({ "id": job["id"] })).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_storm_and_a_datagram_go_to_a_host_name() {
    let (service, recorder) = service();
    let (device, port) = receiver().await;
    let job = service.invoke("storm_start", json!({ "config": { "target": format!("localhost:{port}"), "protocol": "udp", "size": 8, "rate": 20, "duration_s": 0.3 } })).await.unwrap();
    assert_eq!(job["params"]["target"], format!("localhost:{port}"), "the job names it as it was given");
    assert_eq!(arrives(&device).await, [0x55; 8]);
    let ended = recorder.clone();
    tokio::task::spawn_blocking(move || ended.wait_for("job://ended", Duration::from_secs(5), |_| true)).await.unwrap().expect("the storm ended by its duration");
    assert_eq!(failure(&service, "storm_start", json!({ "config": { "target": "no-such-host.invalid:9", "protocol": "udp", "size": 8, "rate": 1 } })).await["code"], "transport.dns");
    assert_eq!(failure(&service, "storm_start", json!({ "config": { "target": "localhost", "protocol": "udp", "size": 8, "rate": 1 } })).await["code"], "transport.target_invalid");

    let sent = service.invoke("broadcast_send", json!({ "config": { "mode": "list", "target": format!("localhost:{port}"), "port": 0, "payload": { "kind": "text", "text": "hi" } } })).await.unwrap();
    assert_eq!((sent["packets"].as_u64(), sent["errors"].as_u64()), (Some(1), Some(0)), "{sent}");
    while arrives(&device).await != b"hi" {}
}

#[tokio::test]
async fn a_live_mqtt_connection_refuses_a_publish_to_a_wildcard() {
    let (service, _) = service();
    for topic in ["zone/+/level", "zone/#"] {
        let refused = failure(&service, "mqtt_publish", json!({ "jobId": 1, "topic": topic, "payload": "1", "qos": 0, "retain": false })).await;
        assert_eq!((refused["code"].as_str(), refused["field"]["key"].as_str()), (Some("node.topic_wildcard"), Some("topic")), "{topic}");
    }
    let nothing = failure(&service, "mqtt_publish", json!({ "jobId": 1, "topic": " ", "payload": "1", "qos": 0, "retain": false })).await;
    assert_eq!(nothing["code"], "mqtt.topic_required");
    let unknown = failure(&service, "mqtt_publish", json!({ "jobId": 1, "topic": "zone/1", "payload": "1", "qos": 0, "retain": false })).await;
    assert_eq!(unknown["code"], "mqtt.not_connected", "a topic that is fine reaches the connection lookup");
    let filter = failure(&service, "mqtt_subscribe", json!({ "jobId": 1, "filters": [{ "filter": "zone/#", "qos": 0 }] })).await;
    assert_eq!(filter["code"], "mqtt.not_connected", "a subscription takes filters");
}
