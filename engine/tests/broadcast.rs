//! The beacon and the discovery listener at their limits, on loopback: a
//! beacon keeps the rate it was allowed, settings past their range are refused
//! or held, and a flood of peers or probes stays bounded and counted.

use std::time::{Duration, Instant};

use signal_lab_engine::broadcast::{start_beacon, EmitConfig, Payload, TargetMode};
use signal_lab_engine::discovery::{start_discovery, DiscoveryConfig};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::{Capture, Host, JobRegistry};

fn beacon_config(target: String, rate: f64, duration_s: f64) -> EmitConfig {
    EmitConfig {
        mode: TargetMode::List,
        target,
        port: 0,
        payload: Payload::Text { text: "BEACON".into() },
        bind: None,
        ttl: 1,
        multicast_loop: true,
        rate,
        count: 0,
        duration_s,
    }
}

/// The datagrams that reach `socket` until it has been quiet for 300 ms.
fn drain(socket: std::net::UdpSocket) -> u64 {
    socket.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
    let mut buf = [0u8; 256];
    let mut n = 0;
    while socket.recv_from(&mut buf).is_ok() {
        n += 1;
    }
    n
}

/// The engine allows 50 000 a second; a beacon asked for 10 000 to one target
/// for a second sends about 10 000, not what a millisecond timer allows.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_fast_beacon_keeps_its_rate() {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    let device = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let target = device.local_addr().unwrap().to_string();
    let job = start_beacon(host, JobRegistry::new(), beacon_config(target, 10_000.0, 1.0)).await.unwrap();
    let received = tokio::task::spawn_blocking(move || drain(device)).await.unwrap();
    let waiting = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || waiting.wait_for("job://ended", Duration::from_secs(3), |ended| ended["job_id"] == job.id)).await.unwrap();
    let last = recorder.payloads("broadcast://emit-stat").last().cloned().unwrap();
    eprintln!("beacon 10 000/s for 1 s: received {received}, reported {last}");
    assert!(ended.is_some());
    assert!((9_000..=10_100).contains(&received), "{received}");
    assert_eq!(last["packets"].as_u64().unwrap() + last["errors"].as_u64().unwrap() + last["local"].as_u64().unwrap_or(0), last["rounds"].as_u64().unwrap());
}

/// A rate so low its period is past what a clock holds is refused, not a panic
/// in a job that then never ends.
#[tokio::test]
async fn a_beacon_rate_past_any_clock_is_refused() {
    let host = Host::new(Recorder::new(), Capture::new());
    let jobs = JobRegistry::new();
    for rate in [1e-30, f64::MIN_POSITIVE] {
        let refused = start_beacon(host.clone(), jobs.clone(), beacon_config("127.0.0.1:9".into(), rate, 0.0)).await;
        assert!(refused.as_ref().is_err_and(|error| error.is("broadcast.rate_invalid")), "{rate}: {:?}", refused.map(|job| job.id));
    }
    assert!(jobs.list().is_empty());
}

fn listener(bind: String, respond: bool, delay_ms: u64) -> DiscoveryConfig {
    DiscoveryConfig {
        bind,
        groups: vec![],
        interface: None,
        reuse: false,
        respond,
        response: respond.then(|| Payload::Text { text: "HERE".into() }),
        respond_delay_ms: delay_ms,
        match_contains: None,
    }
}

/// Thousands of distinct senders: the table keeps a bounded number of them and
/// says how much it heard from the rest.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_crowd_of_peers_is_bounded_and_counted() {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    let jobs = JobRegistry::new();
    let port = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let job = start_discovery(host, jobs.clone(), listener(format!("127.0.0.1:{port}"), false, 0)).await.unwrap();
    tokio::task::spawn_blocking(move || {
        for _ in 0..2_000 {
            let peer = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
            peer.send_to(b"HELLO", ("127.0.0.1", port)).unwrap();
        }
    })
    .await
    .unwrap();
    let waiting = recorder.clone();
    let report = tokio::task::spawn_blocking(move || waiting.wait_for("broadcast://peers", Duration::from_secs(5), |report| report["packets"] == 2_000))
        .await
        .unwrap()
        .expect("all 2 000 counted");
    jobs.stop(job.id);
    let rows = report["peers"].as_array().unwrap();
    let in_rows: u64 = rows.iter().map(|peer| peer["packets"].as_u64().unwrap()).sum();
    let unlisted = report["unlisted"].as_u64().unwrap();
    eprintln!("2 000 peers: {} rows, {in_rows} in rows, {unlisted} unlisted", rows.len());
    assert!(rows.len() <= 512, "{} rows", rows.len());
    assert_eq!(in_rows + unlisted, 2_000, "every packet is in a row or counted as unlisted");
}

/// A flood of probes with a slow reply: what waits to be answered is bounded,
/// and what was not answered is counted.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_answering_listener_under_a_flood_stays_bounded() {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    let jobs = JobRegistry::new();
    let port = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let job = start_discovery(host, jobs.clone(), listener(format!("127.0.0.1:{port}"), true, 2_000)).await.unwrap();
    let prober = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let started = Instant::now();
    let mut probes = 0u64;
    while probes < 50_000 {
        if prober.send_to(b"PROBE", ("127.0.0.1", port)).is_ok() {
            probes += 1;
        }
    }
    eprintln!("50 000 probes in {:?}", started.elapsed());
    tokio::time::sleep(Duration::from_millis(3_000)).await;
    let last = recorder.payloads("broadcast://peers").last().cloned().unwrap();
    jobs.stop(job.id);
    let replies = drain(prober);
    eprintln!("answered {replies}; report {}", serde_json::json!({ "packets": last["packets"], "responses": last["responses"], "skipped": last["replies_skipped"] }));
    let heard = last["packets"].as_u64().unwrap();
    assert!(replies <= 1_100, "{replies} replies to one prober");
    assert_eq!(last["responses"].as_u64().unwrap() + last["replies_skipped"].as_u64().unwrap(), heard, "{last}");
}
