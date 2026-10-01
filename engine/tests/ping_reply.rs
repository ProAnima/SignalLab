//! The bundled "OSC ping → reply" experiment, end to end through the command
//! table, with no window: a loopback OSC device answers `/ping <id>` with
//! `/pong <id>`, and the run is observed through the events a host receives.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::host::Recorder;
use signal_lab_engine::osc_codec::{decode_packet, encode_message};
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{experiment_files, paths, Capture, Host, Mode, Service};
use tokio::net::UdpSocket;

/// Run reports go to a scratch folder, never to the user's Documents.
fn scratch_data_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("signallab-ping-reply-{:016x}", rand_suffix()));
    std::fs::create_dir_all(&dir).unwrap();
    paths::set_data_dir(dir.clone());
    paths::data_dir()
}

fn rand_suffix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64 ^ std::process::id() as u64
}

/// A device that answers `/ping <args>` with `/pong <args>` to `reply_to`, or stays silent.
async fn device(reply_to: SocketAddr, answers: bool) -> SocketAddr {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let address = socket.local_addr().unwrap();
    tokio::spawn(async move {
        let mut buffer = [0u8; 2048];
        while let Ok((size, _)) = socket.recv_from(&mut buffer).await {
            let Ok(messages) = decode_packet(&buffer[..size]) else { continue };
            for message in messages.into_iter().filter(|message| message.address == "/ping") {
                if answers {
                    socket.send_to(&encode_message("/pong", &message.args), reply_to).await.unwrap();
                }
            }
        }
    });
    address
}

fn free_udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

/// The template, pointed at `device` and listening on `bind`.
fn template(device: SocketAddr, bind: SocketAddr) -> Experiment {
    let mut value: Value = serde_json::to_value(experiment_files::parse(include_str!("../../experiments/templates/osc-ping-reply.json")).unwrap()).unwrap();
    value["params"] = json!([{ "name": "device", "value": device.to_string() }]);
    let pong = value["nodes"].as_array_mut().unwrap().iter_mut().find(|node| node["id"] == "pong").unwrap();
    pong["bind"] = bind.to_string().into();
    pong["timeout_ms"] = 600.into();
    serde_json::from_value(value).unwrap()
}

async fn run(service: &Service, recorder: &Arc<Recorder>, doc: &Experiment) -> (Value, Vec<Value>) {
    let job = service.invoke("experiment_start", json!({ "document": doc })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let watcher = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || watcher.wait_for("experiment://ended", Duration::from_secs(15), |payload| payload["job_id"] == id))
        .await
        .unwrap()
        .expect("the run ended");
    let steps = recorder.payloads("experiment://step").into_iter().filter(|step| step["job_id"] == id).collect();
    (ended, steps)
}

fn step<'a>(steps: &'a [Value], node: &str, state: &str) -> Option<&'a Value> {
    steps.iter().find(|step| step["node_id"] == node && step["state"] == state)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_device_that_answers_is_matched_and_one_that_does_not_takes_the_timeout_path() {
    let data_dir = scratch_data_dir();
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    // An empty, read-only store: the template needs no secrets.
    let service = Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None)));

    // Answered: Matched, the reply carries the run id back, the log reads it.
    let bind: SocketAddr = format!("127.0.0.1:{}", free_udp_port()).parse().unwrap();
    let answering = device(bind, true).await;
    let (ended, steps) = run(&service, &recorder, &template(answering, bind)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let id = ended["job_id"].as_u64().unwrap();
    let matched = step(&steps, "pong", "passed").expect("the wait passed");
    assert_eq!(matched["message_key"], "exp.step.matched");
    assert_eq!(matched["vars"]["reply"]["address"], "/pong");
    assert_eq!(matched["vars"]["reply"]["args"], json!([id.to_string()]), "the device echoed this run's id");
    let answered = step(&steps, "answered", "passed").expect("the Matched path ran");
    assert!(answered["detail"].as_str().unwrap().starts_with(&format!("/pong from {answering}")), "{answered}");
    assert!(step(&steps, "silent", "running").is_none(), "the Timeout path did not run");
    let report = PathBuf::from(ended["report_path"].as_str().unwrap());
    assert!(report.starts_with(&data_dir) && report.exists(), "the report is written to the data folder");

    // Silent: the Timeout wire is followed and the run still passes.
    let bind: SocketAddr = format!("127.0.0.1:{}", free_udp_port()).parse().unwrap();
    let silent = device(bind, false).await;
    let (ended, steps) = run(&service, &recorder, &template(silent, bind)).await;
    assert_eq!(ended["error"], Value::Null, "{ended}");
    let timed_out = step(&steps, "pong", "passed").expect("the wait ended");
    assert_eq!(timed_out["message_key"], "exp.step.timedOut");
    assert!(step(&steps, "silent", "passed").is_some() && step(&steps, "answered", "running").is_none());

    // Nothing is left running.
    assert_eq!(service.invoke("jobs_list", Value::Null).await.unwrap(), json!([]));
    drop(service);
    std::fs::remove_dir_all(data_dir).unwrap();
}
