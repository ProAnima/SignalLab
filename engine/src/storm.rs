//! Traffic / "storm" generator: a UDP or TCP load source aimed at a target you
//! own, for stress-testing servers and links. The payload is bounded (1 to
//! 65 507 bytes); the rate is packets or connections per second, 0 for as fast
//! as possible; the duration is in seconds, 0 until stopped. What went out is
//! reported live.
//!
//! The rate is a schedule, not a sleep per batch: unit *n* is due at
//! start + n / rate, and each wake sends what is due (a batch at most) and
//! sleeps until the next one, so the operating system's timer granularity
//! costs no packets. A TCP unit is a connection opened, written and closed,
//! one at a time.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UdpSocket};

use super::error::EngineResult;
use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::transport;

/// Units one wake sends at most; a schedule further behind skips what is older
/// than that, so a stall is not made up for with a burst above the rate.
const MAX_BATCH: u64 = 256;

const NANOS: u128 = 1_000_000_000;

/// The units due at `elapsed` into a run at `rate` per second: the first is due at once.
fn due_by(elapsed: Duration, rate: u64) -> u64 {
    (elapsed.as_nanos() * u128::from(rate) / NANOS) as u64 + 1
}

/// When unit `n` is due, from the start. Rounded up, so at that moment `due_by`
/// already counts it: a wake never finds nothing to send.
fn due_at(n: u64, rate: u64) -> Duration {
    Duration::from_nanos((u128::from(n) * NANOS).div_ceil(u128::from(rate)) as u64)
}

#[derive(Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Udp,
    Tcp,
}

#[derive(Clone, Deserialize)]
pub struct StormConfig {
    pub target: String,
    pub protocol: Protocol,
    /// Payload size in bytes.
    pub size: usize,
    /// Target packets/connections per second (0 = as fast as possible).
    pub rate: u64,
    /// Run duration in seconds (0 = until stopped).
    #[serde(default)]
    pub duration_s: f64,
}

#[derive(Clone, Serialize)]
struct StormStat {
    job_id: u64,
    ts: u64,
    packets: u64,
    bytes: u64,
    errors: u64,
    pps: f64,
    mbps: f64,
}

pub async fn start_storm(
    host: Host,
    jobs: JobRegistry,
    cfg: StormConfig,
) -> EngineResult<JobInfo> {
    let target = transport::resolve(&cfg.target).await?;
    let size = cfg.size.clamp(1, 65_507);

    let id = jobs.next_id();
    let protocol = match cfg.protocol {
        Protocol::Udp => "UDP",
        Protocol::Tcp => "TCP",
    };
    let info = JobInfo::new(id, "storm", format!("Storm {protocol} → {} @{}pps", cfg.target, cfg.rate))
        .with("protocol", protocol)
        .with("target", &cfg.target)
        .with("rate", cfg.rate);

    let host_cl = host.clone();
    let jobs_cl = jobs.clone();
    let handle = tokio::spawn(async move {
        let packets = Arc::new(AtomicU64::new(0));
        let bytes = Arc::new(AtomicU64::new(0));
        let errors = Arc::new(AtomicU64::new(0));
        let payload = vec![0x55u8; size];
        let start = Instant::now();
        let gate = Gate::new(1_000);
        // Stopping the storm must stop its reporter too; the guard aborts it
        // when this task's future is dropped on cancel.
        let mut guard = TaskGuard::new();

        // reporter
        let reporter = {
            let (host_r, packets_r, bytes_r, errors_r) =
                (host_cl.clone(), packets.clone(), bytes.clone(), errors.clone());
            tokio::spawn(async move {
                let mut prev_p = 0u64;
                let mut prev_b = 0u64;
                let mut prev = Instant::now();
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    let now = Instant::now();
                    let dt = now.duration_since(prev).as_secs_f64().max(1e-6);
                    let p = packets_r.load(Ordering::Relaxed);
                    let b = bytes_r.load(Ordering::Relaxed);
                    let pps = (p - prev_p) as f64 / dt;
                    let mbps = ((b - prev_b) as f64 * 8.0) / dt / 1_000_000.0;
                    prev_p = p;
                    prev_b = b;
                    prev = now;
                    host_r.emit(
                        "storm://stat",
                        StormStat {
                            job_id: id,
                            ts: now_ms(),
                            packets: p,
                            bytes: b,
                            errors: errors_r.load(Ordering::Relaxed),
                            pps,
                            mbps,
                        },
                    );
                }
            })
        };
        guard.watch(reporter.abort_handle());

        let udp = if cfg.protocol == Protocol::Udp {
            match UdpSocket::bind(transport::unspecified_for(&target)).await {
                Ok(s) => {
                    if s.connect(target).await.is_err() {
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                    Some(Arc::new(s))
                }
                Err(_) => {
                    errors.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        } else {
            None
        };

        // The next unit of the schedule (with a rate).
        let mut next: u64 = 0;
        'outer: loop {
            if cfg.duration_s > 0.0 && start.elapsed().as_secs_f64() >= cfg.duration_s {
                break;
            }
            let batch = if cfg.rate == 0 {
                MAX_BATCH
            } else {
                let due = due_by(start.elapsed(), cfg.rate);
                next = next.max(due.saturating_sub(MAX_BATCH));
                due - next
            };
            for _ in 0..batch {
                match cfg.protocol {
                    Protocol::Udp => {
                        if let Some(sock) = &udp {
                            match sock.send(&payload).await {
                                Ok(n) => {
                                    packets.fetch_add(1, Ordering::Relaxed);
                                    bytes.fetch_add(n as u64, Ordering::Relaxed);
                                    // Every packet is identical, so one sample a
                                    // second is all the timeline needs.
                                    if inspect::armed(&host_cl) && gate.allow() {
                                        inspect::publish(
                                            &host_cl,
                                            gate.mark(
                                                Frame::tx("udp", "storm")
                                                    .job(id)
                                                    .remote(target)
                                                    .payload(&payload)
                                                    .summary(format!("UDP flood packet ({n} B)"))
                                                    .verdict("sampled 1/s"),
                                            ),
                                        );
                                    }
                                }
                                Err(_) => {
                                    errors.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                    Protocol::Tcp => {
                        match tokio::time::timeout(
                            Duration::from_millis(500),
                            TcpStream::connect(target),
                        )
                        .await
                        {
                            Ok(Ok(mut s)) => {
                                if s.write_all(&payload).await.is_ok() {
                                    packets.fetch_add(1, Ordering::Relaxed);
                                    bytes.fetch_add(payload.len() as u64, Ordering::Relaxed);
                                }
                                let _ = s.shutdown().await;
                            }
                            _ => {
                                errors.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
                next += 1;
                if cfg.duration_s > 0.0 && start.elapsed().as_secs_f64() >= cfg.duration_s {
                    break 'outer;
                }
            }
            if cfg.rate == 0 {
                tokio::task::yield_now().await;
                continue;
            }
            // Until the next unit is due, or the end of the run if that comes first.
            let mut wake = start + due_at(next, cfg.rate);
            if cfg.duration_s > 0.0 {
                wake = wake.min(start + Duration::from_secs_f64(cfg.duration_s));
            }
            if wake <= Instant::now() {
                tokio::task::yield_now().await;
            } else {
                tokio::time::sleep_until(wake.into()).await;
            }
        }

        drop(guard);
        // The reporter only ticks every 250 ms, so a run that ends on its own
        // duration used to leave the panel showing a count from a quarter second
        // ago — and "how many actually went out" is the one number a load
        // generator owes you. The rate reads zero: nothing is being sent now.
        host_cl.emit(
            "storm://stat",
            StormStat {
                job_id: id,
                ts: now_ms(),
                packets: packets.load(Ordering::Relaxed),
                bytes: bytes.load(Ordering::Relaxed),
                errors: errors.load(Ordering::Relaxed),
                pps: 0.0,
                mbps: 0.0,
            },
        );
        host_cl.emit(
            "job://ended",
            serde_json::json!({ "job_id": id, "kind": "storm", "error": serde_json::Value::Null }),
        );
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Recorder;
    use crate::inspect::Capture;

    #[test]
    fn a_rate_is_a_schedule() {
        assert_eq!(due_by(Duration::ZERO, 50), 1, "the first at once");
        assert_eq!(due_by(Duration::from_millis(19), 50), 1);
        assert_eq!(due_by(Duration::from_millis(20), 50), 2);
        assert_eq!(due_by(Duration::from_millis(999), 250), 250, "250 a second, not 200");
        assert_eq!(due_by(Duration::from_secs(1), 7), 8);
        assert_eq!(due_at(50, 50), Duration::from_secs(1));
        assert_eq!(due_at(1, 250), Duration::from_millis(4));
        for rate in [1, 3, 7, 50, 333, 1000, 12_345, 250_000] {
            for n in [0, 1, 2, 99, 100_000] {
                assert!(due_by(due_at(n, rate), rate) > n, "{n} at {rate}/s is counted by the moment it is due");
            }
        }
    }

    /// Runs a UDP storm at `rate` for `seconds` against a loopback socket; what it says it sent.
    async fn storm(rate: u64, seconds: f64) -> u64 {
        let recorder = Recorder::new();
        let host = Host::new(recorder.clone(), Capture::new());
        let jobs = JobRegistry::new();
        let sink = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let cfg = StormConfig { target: format!("localhost:{}", sink.local_addr().unwrap().port()), protocol: Protocol::Udp, size: 16, rate, duration_s: seconds };
        start_storm(host, jobs, cfg).await.unwrap();
        let ended = recorder.clone();
        tokio::task::spawn_blocking(move || ended.wait_for("job://ended", Duration::from_secs(10), |_| true)).await.unwrap().expect("the storm ended on its own");
        let stats = recorder.payloads("storm://stat");
        assert!(stats.iter().all(|stat| stat["errors"] == 0), "{stats:?}");
        stats.iter().filter_map(|stat| stat["packets"].as_u64()).max().expect("a final count")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_storm_sends_the_rate_it_was_given() {
        let (slow, odd) = tokio::join!(storm(50, 1.0), storm(250, 1.0));
        // Loose enough for a loaded machine; the old pacing sent ~100 and 200.
        assert!((40..=60).contains(&slow), "50/s for a second sent {slow}");
        assert!((215..=280).contains(&odd), "250/s for a second sent {odd}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_slow_storm_ends_with_its_duration_not_with_its_next_packet() {
        let started = Instant::now();
        let sent = storm(1, 0.3).await;
        assert_eq!(sent, 1, "one a second for 0.3 s is the first only");
        assert!(started.elapsed() < Duration::from_millis(900), "ended after {:?}", started.elapsed());
    }
}
