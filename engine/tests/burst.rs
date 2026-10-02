//! The HTTP burst against a loopback server: as fast as the workers go, on a
//! schedule of its own, and on a schedule the workers cannot keep — where the
//! requests they could not start in time are counted as missed, not sent late.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use signal_lab_engine::host::Recorder;
use signal_lab_engine::http::{start_burst, BurstConfig, HttpRequest};
use signal_lab_engine::{Capture, Host, JobRegistry};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Answers every request `200 ok` after `delay`, on keep-alive connections;
/// counts what it answered.
async fn server(delay: Duration) -> (String, Arc<AtomicU64>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let answered = Arc::new(AtomicU64::new(0));
    let counter = answered.clone();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else { return };
            let counter = counter.clone();
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let Ok(read) = stream.read(&mut chunk).await else { return };
                    if read == 0 {
                        return;
                    }
                    buffer.extend_from_slice(&chunk[..read]);
                    // GETs without a body: each blank line ends one request.
                    while let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                        buffer.drain(..end + 4);
                        tokio::time::sleep(delay).await;
                        if stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\nok").await.is_err() {
                            return;
                        }
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
    });
    (url, answered)
}

fn burst(url: &str, concurrency: u32, total: u64, duration_s: f64, rate: f64) -> BurstConfig {
    BurstConfig {
        request: HttpRequest { method: "GET".into(), url: url.into(), headers: Vec::new(), body: None, timeout_ms: 5000, auth: Default::default() },
        concurrency,
        total,
        duration_s,
        rate,
        cookies: false,
    }
}

/// Runs a burst to its end; its last report and how long it took.
async fn run(config: BurstConfig) -> (Value, Duration, Vec<Value>) {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    let jobs = JobRegistry::new();
    let began = Instant::now();
    let job = start_burst(host, jobs.clone(), config, None).await.unwrap();
    let waiting = recorder.clone();
    let last = tokio::task::spawn_blocking(move || waiting.wait_for("http://burst-progress", Duration::from_secs(20), |report| report["done"] == true))
        .await
        .unwrap()
        .expect("the burst ends with a last report");
    let took = began.elapsed();
    assert_eq!(last["job_id"], job.id);
    for _ in 0..50 {
        if jobs.list().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(jobs.list().is_empty(), "the job is no longer listed once over");
    (last, took, recorder.payloads("http://burst-progress"))
}

fn ms(report: &Value, name: &str) -> f64 {
    report[name].as_f64().unwrap_or_else(|| panic!("{name} in {report}"))
}

#[tokio::test(flavor = "multi_thread")]
async fn as_fast_as_the_workers_go_with_percentiles() {
    let (url, answered) = server(Duration::ZERO).await;
    let (last, _, reports) = run(burst(&url, 4, 40, 0.0, 0.0)).await;
    assert_eq!((last["sent"].as_u64(), last["ok"].as_u64(), last["failed"].as_u64(), last["missed"].as_u64()), (Some(40), Some(40), Some(0), Some(0)));
    assert_eq!(answered.load(Ordering::Relaxed), 40, "exactly the total, never one more");
    let (min, p50, p90, p95, p99, max) = (ms(&last, "min_latency_ms"), ms(&last, "p50_ms"), ms(&last, "p90_ms"), ms(&last, "p95_ms"), ms(&last, "p99_ms"), ms(&last, "max_latency_ms"));
    assert!(min > 0.0 && min <= p50 && p50 <= p90 && p90 <= p95 && p95 <= p99 && p99 <= max, "{last}");
    assert!(ms(&last, "rps") > 0.0, "the last report rates the whole burst: {last}");
    assert_eq!(reports.iter().filter(|report| report["done"] == true).count(), 1, "one last report");
    assert_eq!(reports.last(), Some(&last), "and nothing after it");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rate_spreads_the_requests_over_time() {
    let (url, answered) = server(Duration::ZERO).await;
    // 30 at 100/s: the last is due at 290 ms, however fast the answers are.
    let (last, took, _) = run(burst(&url, 4, 30, 0.0, 100.0)).await;
    assert_eq!((last["sent"].as_u64(), last["ok"].as_u64(), last["missed"].as_u64()), (Some(30), Some(30), Some(0)), "{last}");
    assert_eq!(answered.load(Ordering::Relaxed), 30);
    assert!(took >= Duration::from_millis(285) && took < Duration::from_secs(3), "{took:?}");
    let rps = ms(&last, "rps");
    assert!((60.0..=110.0).contains(&rps), "about the rate asked for: {rps}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rate_the_workers_cannot_keep_counts_what_they_missed() {
    // One worker, 100 ms an answer: at most ~10/s against 40/s asked for a second.
    let (url, answered) = server(Duration::from_millis(100)).await;
    let (last, took, _) = run(burst(&url, 1, 0, 1.0, 40.0)).await;
    let (sent, missed) = (last["sent"].as_u64().unwrap(), last["missed"].as_u64().unwrap());
    assert_eq!(sent + missed, 40, "every request due in the second is either sent or missed: {last}");
    assert!((8..=12).contains(&sent), "the worker sent what it could, on time: {last}");
    assert_eq!(answered.load(Ordering::Relaxed), sent);
    assert!(ms(&last, "p50_ms") >= 95.0, "{last}");
    assert!(took < Duration::from_millis(1500), "no catching up after the second: {took:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stopped_burst_sends_nothing_more() {
    let (url, answered) = server(Duration::ZERO).await;
    let recorder = Recorder::new();
    let jobs = JobRegistry::new();
    let job = start_burst(Host::new(recorder.clone(), Capture::new()), jobs.clone(), burst(&url, 2, 0, 0.0, 50.0), None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(jobs.stop(job.id));
    tokio::time::sleep(Duration::from_millis(100)).await;
    let after_stop = answered.load(Ordering::Relaxed);
    assert!(after_stop > 5, "it was sending: {after_stop}");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(answered.load(Ordering::Relaxed), after_stop, "a stop ends the pacer and what it started");
    assert!(jobs.list().is_empty());
}
