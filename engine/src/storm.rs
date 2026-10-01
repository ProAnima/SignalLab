//! Traffic / "storm" generator: a controlled UDP or TCP load source aimed at a
//! target you own, for stress-testing servers and links. Rate, payload size and
//! duration are all bounded and reported live.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UdpSocket};

use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};

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
) -> Result<JobInfo, String> {
    let target: SocketAddr = cfg
        .target
        .parse()
        .map_err(|e| format!("invalid target '{}': {e}", cfg.target))?;
    let size = cfg.size.clamp(1, 65_507);

    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "storm".into(),
        label: format!(
            "Storm {} → {} @{}pps",
            match cfg.protocol {
                Protocol::Udp => "UDP",
                Protocol::Tcp => "TCP",
            },
            cfg.target,
            cfg.rate
        ),
        started_ms: now_ms(),
    };

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

        // Pace: send `per_tick` units every 10ms to approximate the target rate.
        let tick = Duration::from_millis(10);
        let per_tick = if cfg.rate == 0 {
            0
        } else {
            (cfg.rate / 100).max(1)
        };

        let udp = if cfg.protocol == Protocol::Udp {
            match UdpSocket::bind("0.0.0.0:0").await {
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

        'outer: loop {
            if cfg.duration_s > 0.0 && start.elapsed().as_secs_f64() >= cfg.duration_s {
                break;
            }
            let batch = if per_tick == 0 { 256 } else { per_tick };
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
                                            Frame::tx("udp", "storm")
                                                .job(id)
                                                .remote(target)
                                                .payload(&payload)
                                                .summary(format!("UDP flood packet ({n} B)"))
                                                .verdict("sampled 1/s"),
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
                if cfg.duration_s > 0.0 && start.elapsed().as_secs_f64() >= cfg.duration_s {
                    break 'outer;
                }
            }
            if per_tick == 0 {
                tokio::task::yield_now().await;
            } else {
                tokio::time::sleep(tick).await;
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
