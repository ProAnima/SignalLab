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

use super::jobs::{now_ms, JobInfo, JobRegistry};

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

/// Apply the impairment profile to one packet, sending it (possibly delayed,
/// duplicated, corrupted, or dropped) via `deliver`.
fn schedule(
    payload: Vec<u8>,
    profile: &ImpairProfile,
    stats: &Arc<Stats>,
    sock: Arc<UdpSocket>,
    dest: Option<SocketAddr>,
) {
    let (loss, dup, corrupt, latency, jitter) = (
        profile.loss,
        profile.duplicate,
        profile.corrupt,
        profile.latency_ms,
        profile.jitter_ms,
    );

    let mut rng = rand::thread_rng();
    if loss > 0.0 && rng.gen_bool(loss.clamp(0.0, 1.0)) {
        stats.dropped.fetch_add(1, Ordering::Relaxed);
        return;
    }

    let copies = if dup > 0.0 && rng.gen_bool(dup.clamp(0.0, 1.0)) {
        stats.duplicated.fetch_add(1, Ordering::Relaxed);
        2
    } else {
        1
    };

    for _ in 0..copies {
        let mut bytes = payload.clone();
        if corrupt > 0.0 && !bytes.is_empty() && rng.gen_bool(corrupt.clamp(0.0, 1.0)) {
            let idx = rng.gen_range(0..bytes.len());
            bytes[idx] ^= 1 << rng.gen_range(0..8);
            stats.corrupted.fetch_add(1, Ordering::Relaxed);
        }
        let delay = latency + if jitter > 0.0 { rng.gen_range(0.0..jitter) } else { 0.0 };
        let sock = sock.clone();
        let stats = stats.clone();
        tokio::spawn(async move {
            if delay > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(delay / 1000.0)).await;
            }
            let res = match dest {
                Some(addr) => sock.send_to(&bytes, addr).await,
                None => sock.send(&bytes).await,
            };
            if let Ok(n) = res {
                stats.forwarded.fetch_add(1, Ordering::Relaxed);
                stats.bytes.fetch_add(n as u64, Ordering::Relaxed);
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
        // client -> target
        let c2s = {
            let (down, up, profile, stats, client_addr) = (
                downstream.clone(),
                upstream.clone(),
                profile.clone(),
                stats.clone(),
                client_addr.clone(),
            );
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                loop {
                    match down.recv_from(&mut buf).await {
                        Ok((n, from)) => {
                            *client_addr.lock().unwrap() = Some(from);
                            schedule(buf[..n].to_vec(), &profile, &stats, up.clone(), None);
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

        // Run until the job is aborted; abort propagates by dropping the tasks.
        let _ = tokio::join!(c2s, s2c);
        reporter.abort();
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}
