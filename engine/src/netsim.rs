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
use crate::host::Host;
use tokio::net::UdpSocket;

use super::error::{EngineError, EngineResult};
use super::inspect::{self, describe_payload, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::net;
use super::osc::{parse_bind, parse_target};
use super::transport;

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
    host: Host,
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
        inspect::armed(&self.host) && self.gate.allow()
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
        inspect::publish(&self.host, frame);
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
    host: Host,
    jobs: JobRegistry,
    cfg: ProxyConfig,
) -> EngineResult<JobInfo> {
    let listen_addr = parse_bind(&cfg.listen)?;
    let target_addr = parse_target(&cfg.target)?;

    let downstream = Arc::new(
        UdpSocket::bind(listen_addr)
            .await
            .map_err(|e| net::bind_error(&cfg.listen, e))?,
    );
    let source = if target_addr.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let upstream = Arc::new(
        UdpSocket::bind(source)
            .await
            .map_err(|e| net::bind_error(source, e))?,
    );
    upstream
        .connect(target_addr)
        .await
        .map_err(|e| transport::of_io(&e).error(&cfg.target).because(e))?;
    // Where each leg receives, for the reason a leg stopped.
    let down_local = downstream.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| cfg.listen.clone());
    let up_local = upstream.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| source.to_string());

    let id = jobs.next_id();
    let info = JobInfo::new(id, "netsim", format!("Impair {} → {}", cfg.listen, cfg.target))
        .with("listen", &cfg.listen)
        .with("target", &cfg.target);

    let profile = cfg.profile.clone();
    let stats = Arc::new(Stats::default());
    let client_addr: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
    let host_cl = host.clone();
    let jobs_cl = jobs.clone();

    let handle = tokio::spawn(async move {
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
                host: host_cl.clone(),
                job_id: id,
                leg: "client→target",
                dir: "tx",
                peer: target_addr.to_string(),
                gate: gate.clone(),
            };
            let receiving = down_local.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                loop {
                    match down.recv_from(&mut buf).await {
                        Ok((n, from)) => {
                            *client_addr.lock().unwrap() = Some(from);
                            schedule(buf[..n].to_vec(), &profile, &stats, up.clone(), None, &tap);
                        }
                        // A reply to a client that has gone: the relay carries on.
                        Err(e) if crate::net::udp_transient(&e) => continue,
                        Err(e) => return EngineError::new("wait.receive_failed").with("target", &receiving).because(e),
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
                host: host_cl.clone(),
                job_id: id,
                leg: "target→client",
                dir: "rx",
                peer: target_addr.to_string(),
                gate: gate.clone(),
            };
            let receiving = up_local.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                loop {
                    match up.recv(&mut buf).await {
                        Ok(n) => {
                            let dest = *client_addr.lock().unwrap();
                            if let Some(addr) = dest {
                                schedule(buf[..n].to_vec(), &profile, &stats, down.clone(), Some(addr), &tap);
                            }
                        }
                        // The target is not listening (yet): the upstream socket is
                        // connected, so the OS says so here. It may come up later.
                        Err(e) if crate::net::udp_transient(&e) => continue,
                        Err(e) => return EngineError::new("wait.receive_failed").with("target", &receiving).because(e),
                    }
                }
            })
        };

        // stats reporter
        let reporter = {
            let (host_r, stats_r) = (host_cl.clone(), stats.clone());
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    host_r.emit(
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

        // The relay runs until the job is stopped, or until a leg cannot
        // receive any more. Both legs and the reporter are children of this
        // task; without the guard, stopping the job would abort only this
        // supervisor and leave the relay forwarding traffic and the reporter
        // emitting. The guard aborts all three when this task's future is
        // dropped on cancel, and when a leg has failed.
        let mut guard = TaskGuard::new();
        guard.watch(c2s.abort_handle());
        guard.watch(s2c.abort_handle());
        guard.watch(reporter.abort_handle());
        let (mut c2s, mut s2c) = (c2s, s2c);
        // A leg that panicked ends the relay the same way, its panic as detail.
        let error = tokio::select! {
            ended = &mut c2s => ended.unwrap_or_else(|e| EngineError::new("wait.receive_failed").with("target", &down_local).because(e)),
            ended = &mut s2c => ended.unwrap_or_else(|e| EngineError::new("wait.receive_failed").with("target", &up_local).because(e)),
        };
        drop(guard);
        host_cl.emit("job://ended", serde_json::json!({ "job_id": id, "kind": "netsim", "error": error }));
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

    fn calm() -> ImpairProfile {
        ImpairProfile { latency_ms: 0.0, jitter_ms: 0.0, loss: 0.0, duplicate: 0.0, corrupt: 0.0 }
    }

    async fn recv(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
        let mut buf = vec![0u8; 1024];
        let (n, from) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf)).await.expect("nothing arrived within 2 s").unwrap();
        (buf[..n].to_vec(), from)
    }

    /// The relay's target may come up after the relay: datagrams sent while it
    /// was not listening come back as an error on the connected upstream socket
    /// (refused here, WSAECONNRESET on Windows), which must not end either leg.
    #[tokio::test]
    async fn keeps_relaying_after_the_target_was_not_listening() {
        let host = Host::new(Recorder::new(), Capture::new());
        let jobs = JobRegistry::new();
        let target_port = UdpSocket::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
        let listen_port = UdpSocket::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
        let cfg = ProxyConfig { listen: format!("127.0.0.1:{listen_port}"), target: format!("127.0.0.1:{target_port}"), profile: calm() };
        let job = start_proxy(host, jobs.clone(), cfg).await.unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay: SocketAddr = format!("127.0.0.1:{listen_port}").parse().unwrap();

        // Nobody listens at the target: these are lost, and the OS says so.
        for _ in 0..3 {
            client.send_to(b"early", relay).await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let target = UdpSocket::bind(("127.0.0.1", target_port)).await.unwrap();
        client.send_to(b"ping", relay).await.unwrap();
        let (bytes, upstream) = recv(&target).await;
        assert_eq!(bytes, b"ping", "client → target still relays");
        target.send_to(b"pong", upstream).await.unwrap();
        let (bytes, from) = recv(&client).await;
        assert_eq!((bytes.as_slice(), from), (&b"pong"[..], relay), "target → client still relays");
        assert!(jobs.list().iter().any(|info| info.id == job.id), "the job is still running");
        jobs.stop(job.id);
    }

    #[tokio::test]
    async fn addresses_that_cannot_work_are_refused_with_a_code() {
        let host = Host::new(Recorder::new(), Capture::new());
        let jobs = JobRegistry::new();
        let start = |listen: &str, target: &str| {
            start_proxy(host.clone(), jobs.clone(), ProxyConfig { listen: listen.into(), target: target.into(), profile: calm() })
        };
        assert!(start("127.0.0.1", "127.0.0.1:9").await.unwrap_err().is("node.bind_invalid"));
        assert!(start("127.0.0.1:0", "localhost").await.unwrap_err().is("transport.target_invalid"));
        let taken = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let bind = taken.local_addr().unwrap().to_string();
        let error = start(&bind, "127.0.0.1:9").await.unwrap_err();
        assert_eq!((error.code.as_str(), error.params["target"].as_str()), ("transport.address_in_use", bind.as_str()));
        assert!(jobs.list().is_empty());
    }

    /// A reply to a client that has gone must not end the relay either.
    #[tokio::test]
    async fn keeps_relaying_after_a_client_went_away() {
        let host = Host::new(Recorder::new(), Capture::new());
        let jobs = JobRegistry::new();
        let target = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let listen_port = UdpSocket::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
        let cfg = ProxyConfig { listen: format!("127.0.0.1:{listen_port}"), target: target.local_addr().unwrap().to_string(), profile: calm() };
        let job = start_proxy(host, jobs.clone(), cfg).await.unwrap();
        let relay: SocketAddr = format!("127.0.0.1:{listen_port}").parse().unwrap();

        let gone = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        gone.send_to(b"hello", relay).await.unwrap();
        let (_, upstream) = recv(&target).await;
        drop(gone);
        target.send_to(b"to nobody", upstream).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(b"again", relay).await.unwrap();
        let (bytes, upstream) = recv(&target).await;
        assert_eq!(bytes, b"again");
        target.send_to(b"back", upstream).await.unwrap();
        assert_eq!(recv(&client).await.0, b"back");
        assert!(jobs.list().iter().any(|info| info.id == job.id));
        jobs.stop(job.id);
    }
}

