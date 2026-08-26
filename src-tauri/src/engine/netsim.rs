//! Network impairment: a UDP relay that sits between a client and a target and
//! injects latency, jitter, packet loss, duplication and corruption in both
//! directions. Point your client at the relay's listen port; it forwards to the
//! real target and mangles traffic according to the profile.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::net::UdpSocket;

use super::inspect::{self, describe_payload, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};

#[derive(Clone, Deserialize)]
pub struct ImpairProfile {
    /// Base one-way delay added to every packet, milliseconds.
    #[serde(default)]
    pub latency_ms: f64,
    /// Extra uniform random delay 0..jitter_ms.
    #[serde(default)]
    pub jitter_ms: f64,
    /// Probability 0..1 a packet is dropped.
    #[serde(default)]
    pub loss: f64,
    /// Probability 0..1 a packet is duplicated.
    #[serde(default)]
    pub duplicate: f64,
    /// Probability 0..1 a single byte is flipped (corruption).
    #[serde(default)]
    pub corrupt: f64,
}

#[derive(Clone, Deserialize)]
pub struct ProxyConfig {
    /// Local address the relay listens on, e.g. "0.0.0.0:9010".
    pub listen: String,
    /// Real destination, e.g. "127.0.0.1:9000".
    pub target: String,
    pub profile: ImpairProfile,
}

#[derive(Default)]
struct Stats {
    forwarded: AtomicU64,
    dropped: AtomicU64,
    duplicated: AtomicU64,
    corrupted: AtomicU64,
    bytes: AtomicU64,
}

#[derive(Clone, Serialize)]
struct ProxyStat {
    job_id: u64,
    ts: u64,
    forwarded: u64,
    dropped: u64,
    duplicated: u64,
    corrupted: u64,
    bytes: u64,
}

/// Everything `schedule` needs to report a packet's fate to the Inspector.
#[derive(Clone)]
struct Tap {
    app: AppHandle,
    job_id: u64,
    /// Which leg of the relay: "client→target" or "target→client".
    leg: &'static str,
    /// "tx" for traffic heading to the target, "rx" for replies coming back —
    /// so the Inspector's direction filter means something for relayed packets.
    dir: &'static str,
    /// Fallback peer label when the socket is connected and `dest` is None.
    peer: String,
    /// Shared sampling budget — a relay under load must not flood the UI.
    gate: Arc<Gate>,
}

impl Tap {
    fn armed(&self) -> bool {
        inspect::armed(&self.app) && self.gate.allow()
    }

    fn record(&self, bytes: &[u8], dest: Option<SocketAddr>, verdict: String) {
        let (proto, summary, detail) = describe_payload(bytes);
        let peer = dest
            .map(|d| d.to_string())
            .unwrap_or_else(|| self.peer.clone());
        let mut frame = Frame::new(proto, self.dir, "netsim")
            .job(self.job_id)
            .local(self.leg)
            .remote(peer)
            .payload(bytes)
            .summary(summary)
            .verdict(verdict);
        if let Some(d) = detail {
            frame = frame.detail(d);
        }
        inspect::publish(&self.app, frame);
    }
}

/// Apply the impairment profile to one packet, sending it (possibly delayed,
/// duplicated, corrupted, or dropped) via `deliver`.
fn schedule(
    payload: Vec<u8>,
    profile: &ImpairProfile,
    stats: &Arc<Stats>,
    sock: Arc<UdpSocket>,
    dest: Option<SocketAddr>,
    tap: &Tap,
) {
    let (loss, dup, corrupt, latency, jitter) = (
        profile.loss,
        profile.duplicate,
        profile.corrupt,
        profile.latency_ms,
        profile.jitter_ms,
    );

    // Decide once per packet: a dropped packet is exactly the event a QA run
    // wants to see, so the sampling decision is made before the verdict.
    let capture = tap.armed();

    let mut rng = rand::thread_rng();
    if loss > 0.0 && rng.gen_bool(loss.clamp(0.0, 1.0)) {
        stats.dropped.fetch_add(1, Ordering::Relaxed);
        if capture {
            tap.record(&payload, dest, "dropped".to_string());
        }
        return;
    }

    let copies = if dup > 0.0 && rng.gen_bool(dup.clamp(0.0, 1.0)) {
        stats.duplicated.fetch_add(1, Ordering::Relaxed);
        2
    } else {
        1
    };

    for copy in 0..copies {
        let mut bytes = payload.clone();
        let mut corrupted = false;
        if corrupt > 0.0 && !bytes.is_empty() && rng.gen_bool(corrupt.clamp(0.0, 1.0)) {
            let idx = rng.gen_range(0..bytes.len());
            bytes[idx] ^= 1 << rng.gen_range(0..8);
            stats.corrupted.fetch_add(1, Ordering::Relaxed);
            corrupted = true;
        }
        let delay = latency + if jitter > 0.0 { rng.gen_range(0.0..jitter) } else { 0.0 };
        let sock = sock.clone();
        let stats = stats.clone();
        let tap = tap.clone();
        tokio::spawn(async move {
            if delay > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(delay / 1000.0)).await;
            }
            let res = match dest {
                Some(addr) => sock.send_to(&bytes, addr).await,
                None => sock.send(&bytes).await,
            };
            match res {
                Ok(n) => {
                    stats.forwarded.fetch_add(1, Ordering::Relaxed);
                    stats.bytes.fetch_add(n as u64, Ordering::Relaxed);
                    if capture {
                        let mut verdict = format!("forwarded +{delay:.0}ms");
                        if corrupted {
                            verdict.push_str(" · corrupted");
                        }
                        if copies > 1 {
                            verdict.push_str(&format!(" · copy {}/{copies}", copy + 1));
                        }
                        tap.record(&bytes, dest, verdict);
                    }
                }
                Err(e) => {
                    if capture {
                        tap.record(&bytes, dest, format!("send failed: {e}"));
                    }
                }
            }
        });
    }
}

pub async fn start_proxy(
    app: AppHandle,
    jobs: JobRegistry,
    cfg: ProxyConfig,
) -> Result<JobInfo, String> {
    let listen_addr: SocketAddr = cfg
        .listen
        .parse()
        .map_err(|e| format!("invalid listen '{}': {e}", cfg.listen))?;
    let target_addr: SocketAddr = cfg
        .target
        .parse()
        .map_err(|e| format!("invalid target '{}': {e}", cfg.target))?;

    let downstream = Arc::new(
        UdpSocket::bind(listen_addr)
            .await
            .map_err(|e| format!("bind {} failed: {e}", cfg.listen))?,
    );
    let upstream = Arc::new(
        UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("upstream socket failed: {e}"))?,
    );
    upstream
        .connect(target_addr)
        .await
        .map_err(|e| format!("connect {target_addr} failed: {e}"))?;

    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "netsim".into(),
        label: format!("Impair {} → {}", cfg.listen, cfg.target),
        started_ms: now_ms(),
    };

    let profile = cfg.profile.clone();
    let stats = Arc::new(Stats::default());
    let client_addr: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
    let app_cl = app.clone();
    let jobs_cl = jobs.clone();

    let handle = tauri::async_runtime::spawn(async move {
        // One sampling budget shared by both legs, so a loaded relay reports a
        // representative slice rather than swamping the Inspector.
        let gate = Arc::new(Gate::new(25));

        // client -> target
        let c2s = {
            let (down, up, profile, stats, client_addr) = (
                downstream.clone(),
                upstream.clone(),
                profile.clone(),
                stats.clone(),
                client_addr.clone(),
            );
            let tap = Tap {
                app: app_cl.clone(),
                job_id: id,
                leg: "client→target",
                dir: "tx",
                peer: target_addr.to_string(),
                gate: gate.clone(),
            };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                loop {
                    match down.recv_from(&mut buf).await {
                        Ok((n, from)) => {
                            *client_addr.lock().unwrap() = Some(from);
                            schedule(buf[..n].to_vec(), &profile, &stats, up.clone(), None, &tap);
                        }
                        Err(_) => break,
                    }
                }
            })
        };

        // target -> client
        let s2c = {
            let (down, up, profile, stats, client_addr) = (
                downstream.clone(),
                upstream.clone(),
                profile.clone(),
                stats.clone(),
                client_addr.clone(),
            );
            let tap = Tap {
                app: app_cl.clone(),
                job_id: id,
                leg: "target→client",
                dir: "rx",
                peer: target_addr.to_string(),
                gate: gate.clone(),
            };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                loop {
                    match up.recv(&mut buf).await {
                        Ok(n) => {
                            let dest = *client_addr.lock().unwrap();
                            if let Some(addr) = dest {
                                schedule(
                                    buf[..n].to_vec(),
                                    &profile,
                                    &stats,
                                    down.clone(),
                                    Some(addr),
                                    &tap,
                                );
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
        };

        // stats reporter
        let reporter = {
            let (app_r, stats_r) = (app_cl.clone(), stats.clone());
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    let _ = app_r.emit(
                        "netsim://stat",
                        ProxyStat {
                            job_id: id,
                            ts: now_ms(),
                            forwarded: stats_r.forwarded.load(Ordering::Relaxed),
                            dropped: stats_r.dropped.load(Ordering::Relaxed),
                            duplicated: stats_r.duplicated.load(Ordering::Relaxed),
                            corrupted: stats_r.corrupted.load(Ordering::Relaxed),
                            bytes: stats_r.bytes.load(Ordering::Relaxed),
                        },
                    );
                }
            })
        };

        // The relay runs until the job is stopped. Both legs and the reporter
        // are children of this task; without the guard, stopping the job would
        // abort only this supervisor and leave the relay forwarding traffic and
        // the reporter emitting. The guard aborts all three when this task's
        // future is dropped on cancel.
        let mut guard = TaskGuard::new();
        guard.watch(c2s.abort_handle());
        guard.watch(s2c.abort_handle());
        guard.watch(reporter.abort_handle());
        std::future::pending::<()>().await;
        drop(guard);
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}
