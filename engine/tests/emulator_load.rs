//! An emulator hammered on loopback: every request it answers is counted once,
//! its activity events stay few and bounded and add up to its counters, and a
//! connection past its limit is counted as refused — never closed unsaid.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::emulator::Emulator;
use signal_lab_engine::emulator_job::{self, EmulatorHub, StartOptions};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::{Capture, Host, JobRegistry};

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

async fn start(document: Value) -> (Arc<Recorder>, JobRegistry, EmulatorHub, u64, String) {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    host.capture().set_enabled(true);
    let (jobs, hub) = (JobRegistry::new(), EmulatorHub::new());
    let emulator: Emulator = serde_json::from_value(document).unwrap();
    let info = emulator_job::start(host, jobs.clone(), hub.clone(), emulator, StartOptions { seed: Some(1), ..Default::default() }).await.unwrap();
    let local = info.params["local"].clone();
    (recorder, jobs, hub, info.id, local)
}

/// Requests over kept-alive connections for two seconds, as a load tool sends
/// them (Windows allows a few hundred new connections a second; kept-alive ones
/// go much faster). Every answer is counted, the Inspector's frames are gated
/// with what they did not show said, and the screen hears at most five events
/// a second, each of at most 200 exchanges, that add up to the total.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_hammered_emulator_counts_every_request_and_reports_in_bounded_batches() {
    let port = free_port();
    let (recorder, jobs, hub, id, local) = start(json!({
        "name": "Hammered", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
        "routes": [{ "method": "GET", "path": "/ping", "responses": [{ "body": "pong {{counter}}" }] }]
    }))
    .await;
    let url = format!("http://{local}/ping");
    let answered = Arc::new(AtomicU64::new(0));
    let until = Instant::now() + Duration::from_secs(2);
    let workers: Vec<_> = (0..32)
        .map(|_| {
            let (url, answered) = (url.clone(), answered.clone());
            tokio::spawn(async move {
                let client = reqwest::Client::builder().pool_max_idle_per_host(1).build().unwrap();
                while Instant::now() < until {
                    let response = client.get(&url).send().await.expect("loopback answers");
                    assert_eq!(response.status(), 200);
                    response.bytes().await.unwrap();
                    answered.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    for worker in workers {
        worker.await.unwrap();
    }
    let sent = answered.load(Ordering::Relaxed);
    eprintln!("{sent} requests in 2 s: {} a second", sent / 2);
    // The last activity is reported within one interval.
    tokio::time::sleep(Duration::from_millis(500)).await;

    let counts = hub.snapshot(id, 0, 1).unwrap().counts;
    assert_eq!((counts.total, counts.hits.clone(), counts.unmatched, counts.failed), (sent, vec![sent], 0, 0), "each request once, on its rule");
    let events = recorder.payloads("emulator://activity");
    let carried: u64 = events.iter().map(|event| event["exchanges"].as_array().unwrap().len() as u64).sum();
    let dropped: u64 = events.iter().map(|event| event["dropped"].as_u64().unwrap()).sum();
    assert_eq!(carried + dropped, sent, "what the events carried and what they said they could not");
    assert!(events.iter().all(|event| event["exchanges"].as_array().unwrap().len() <= 200), "an event is bounded");
    assert!(events.len() <= 16, "about five a second, not one a request: {}", events.len());
    assert!(events.iter().all(|event| event["exchanges"].as_array().unwrap().iter().all(|exchange| exchange.get("data").is_none())), "bodies stay with the command");
    assert!(hub.snapshot(id, 0, 10_000).unwrap().exchanges.len() <= 500, "the emulator keeps its newest 500");
    jobs.stop(id);
}

/// A client past the connection limit is closed at once — and counted, so a
/// load test against the emulator does not see resets nobody explains.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connections_past_the_limit_are_counted_as_refused() {
    let port = free_port();
    let (_recorder, jobs, hub, id, local) = start(json!({
        "name": "Full", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
        "routes": [{ "path": "/", "responses": [{ "body": "ok" }] }]
    }))
    .await;
    // Connections that say nothing hold their place until the head timeout (30 s).
    let mut held = Vec::new();
    for _ in 0..signal_lab_engine::emulator_http::MAX_CONNECTIONS {
        held.push(tokio::net::TcpStream::connect(&local).await.unwrap());
    }
    // Accepted in order: wait until the server has taken them all.
    let started = Instant::now();
    let refused = loop {
        let extra = tokio::net::TcpStream::connect(&local).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let counts = hub.snapshot(id, 0, 1).unwrap().counts;
        drop(extra);
        if counts.refused > 0 || started.elapsed() > Duration::from_secs(10) {
            break counts.refused;
        }
    };
    assert!(refused >= 1, "a connection past the limit is counted");
    assert_eq!(hub.snapshot(id, 0, 1).unwrap().counts.total, 0, "and is not a request");
    let counts = serde_json::to_value(hub.snapshot(id, 0, 1).unwrap().counts).unwrap();
    assert!(counts["refused"].as_u64().unwrap() >= 1, "the counters carry it: {counts}");
    drop(held);
    jobs.stop(id);
}

#[test]
fn refused_is_left_out_while_none_were() {
    let counts = signal_lab_engine::emulator_state::Counts::default();
    let value = serde_json::to_value(counts).unwrap();
    assert!(value.get("refused").is_none() && value.get("missed").is_none(), "{value}");
}
