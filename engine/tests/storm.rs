//! The storm against sinks of its own on loopback: what it says it sent went
//! out, what fails on this machine is never the target's error, and settings
//! past their range neither panic nor run away.

use std::time::Duration;

use signal_lab_engine::host::Recorder;
use signal_lab_engine::storm::{start_storm, Protocol, StormConfig};
use signal_lab_engine::{Capture, Host, JobRegistry};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

/// A TCP service that reads each connection to its end and closes it.
async fn tcp_sink() -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                while matches!(stream.read(&mut buf).await, Ok(n) if n > 0) {}
            });
        }
    });
    (addr, task)
}

/// Runs a storm to its end; its last report.
async fn storm(cfg: StormConfig, limit: Duration) -> serde_json::Value {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    let job = start_storm(host, JobRegistry::new(), cfg).await.unwrap();
    let waiting = recorder.clone();
    tokio::task::spawn_blocking(move || waiting.wait_for("job://ended", limit, |ended| ended["job_id"] == job.id))
        .await
        .unwrap()
        .expect("the storm ends");
    recorder.payloads("storm://stat").last().cloned().expect("a report")
}

/// A healthy sink never fails a TCP unit: whatever did not go out failed on
/// this machine (its ports in TIME_WAIT) and is counted as `local`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_healthy_tcp_sink_never_shows_errors() {
    let (sink, _task) = tcp_sink().await;
    let last = storm(StormConfig { target: sink.to_string(), protocol: Protocol::Tcp, size: 64, rate: 300, duration_s: 1.0 }, Duration::from_secs(10)).await;
    assert_eq!(last["errors"], 0, "{last}");
    let packets = last["packets"].as_u64().unwrap();
    let local = last["local"].as_u64().unwrap();
    assert!(packets + local >= 250, "{last}");
}

/// Nothing listens: every connection is the target's refusal, none is local.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_closed_tcp_port_is_the_targets_error() {
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    let last = storm(StormConfig { target: closed.to_string(), protocol: Protocol::Tcp, size: 8, rate: 0, duration_s: 1.0 }, Duration::from_secs(10)).await;
    assert_eq!((last["packets"].as_u64(), last["local"].as_u64()), (Some(0), Some(0)), "{last}");
    assert!(last["errors"].as_u64().unwrap() > 0, "{last}");
}

/// A duration past what a clock holds, the largest rate and size: no panic,
/// the job runs until stopped and stops at once.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn settings_past_their_range_neither_panic_nor_run_away() {
    let host = Host::new(Recorder::new(), Capture::new());
    let jobs = JobRegistry::new();
    let sink = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    for (rate, duration_s, size) in [(u64::MAX, 1e300, usize::MAX), (1, -5.0, 0), (u64::MAX, f64::MAX, 65_508)] {
        let cfg = StormConfig { target: sink.local_addr().unwrap().to_string(), protocol: Protocol::Udp, size, rate, duration_s };
        let job = start_storm(host.clone(), jobs.clone(), cfg).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(jobs.list().iter().any(|info| info.id == job.id), "rate {rate}, {duration_s} s: still running, not panicked");
        jobs.stop(job.id);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(!jobs.list().iter().any(|info| info.id == job.id), "stopped");
    }
}
