//! TCP connect scanner: sweeps a port range on a host, reports open ports and a
//! best-effort service banner. Concurrency-bounded so it stays polite.

use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use super::error::{EngineError, EngineResult};
use super::inspect::{self, Frame};
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::transport;

#[derive(Clone, Deserialize)]
pub struct ScanConfig {
    pub host: String,
    pub port_start: u16,
    pub port_end: u16,
    /// Max simultaneous connection attempts.
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    /// Per-port connect timeout, milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Try to read a service banner from open ports.
    #[serde(default)]
    pub grab_banner: bool,
}

fn default_concurrency() -> u32 {
    256
}
fn default_timeout() -> u64 {
    600
}

#[derive(Clone, Serialize)]
struct OpenPort {
    job_id: u64,
    ts: u64,
    port: u16,
    banner: Option<String>,
}

#[derive(Clone, Serialize)]
struct ScanProgress {
    job_id: u64,
    ts: u64,
    done: u64,
    total: u64,
    open: u64,
    /// Ports this machine could not try (it ran out of sockets), so they are
    /// neither open nor closed; always 0 unless something is wrong here.
    unchecked: u64,
}

/// What one connection attempt says about a port.
enum Probe {
    Open(TcpStream),
    /// Refused, reset or no answer in time: the port is not open.
    Closed,
    /// This machine could not make the attempt (out of local ports, sockets or
    /// buffers) — it says nothing about the port.
    Local,
}

/// Pauses before trying a port again after a local failure: sockets come back
/// as other attempts finish.
const LOCAL_RETRY_MS: [u64; 6] = [20, 50, 100, 250, 500, 1000];


async fn probe(addr: SocketAddr, timeout: Duration) -> Probe {
    match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
        Ok(Ok(stream)) => Probe::Open(stream),
        Ok(Err(error)) if transport::local_connect_failure(&error) => Probe::Local,
        _ => Probe::Closed,
    }
}

pub async fn start_scan(
    host: Host,
    jobs: JobRegistry,
    cfg: ScanConfig,
) -> EngineResult<JobInfo> {
    if cfg.host.trim().is_empty() {
        return Err(EngineError::new("scan.host_required"));
    }
    let (start_port, end_port) = if cfg.port_start <= cfg.port_end {
        (cfg.port_start, cfg.port_end)
    } else {
        (cfg.port_end, cfg.port_start)
    };
    let total = (end_port as u64) - (start_port as u64) + 1;
    // The name is looked up once, before the job: a name that does not resolve
    // is the scan's error, not 65 535 silent misses; an address is taken as it is.
    let ip: IpAddr = match cfg.host.trim().trim_start_matches('[').trim_end_matches(']').parse() {
        Ok(ip) => ip,
        Err(_) => transport::resolve(&format!("{}:{start_port}", cfg.host.trim())).await?.ip(),
    };
    let concurrency = cfg.concurrency.clamp(1, 1024) as usize;
    let timeout = Duration::from_millis(cfg.timeout_ms.clamp(50, 10_000));

    let id = jobs.next_id();
    let info = JobInfo::new(id, "scan", format!("Scan {} :{}-{}", cfg.host, start_port, end_port))
        .with("host", &cfg.host)
        .with("from", start_port)
        .with("to", end_port);

    let host_cl = host.clone();
    let jobs_cl = jobs.clone();
    let target = ip;
    let grab = cfg.grab_banner;
    let handle = tokio::spawn(async move {
        let sem = Arc::new(Semaphore::new(concurrency));
        let done = Arc::new(AtomicU64::new(0));
        let open = Arc::new(AtomicU64::new(0));
        let unchecked = Arc::new(AtomicU64::new(0));
        // A JoinSet aborts whatever is still running when it is dropped, so a
        // stopped scan stops probing instead of quietly finishing the range.
        let mut tasks: JoinSet<()> = JoinSet::new();

        for port in start_port..=end_port {
            let permit = match sem.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            let (host_t, target_t, done_t, open_t, unchecked_t) =
                (host_cl.clone(), target, done.clone(), open.clone(), unchecked.clone());
            tasks.spawn(async move {
                let _permit = permit;
                let addr = SocketAddr::new(target_t, port);
                let mut outcome = probe(addr, timeout).await;
                for pause in LOCAL_RETRY_MS {
                    if !matches!(outcome, Probe::Local) {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(pause)).await;
                    outcome = probe(addr, timeout).await;
                }
                if matches!(outcome, Probe::Local) {
                    unchecked_t.fetch_add(1, Ordering::Relaxed);
                }
                if let Probe::Open(mut stream) = outcome {
                    let banner = if grab {
                        read_banner(&mut stream).await
                    } else {
                        None
                    };
                    open_t.fetch_add(1, Ordering::Relaxed);
                    // Closed with a reset, not a goodbye: a closed connection would
                    // hold one of this machine's ports for minutes (TIME_WAIT), and a
                    // host with thousands open would use them all up, after which
                    // every later port would fail here and pass for closed.
                    let _ = stream.set_zero_linger();
                    drop(stream);

                    // Open ports are rare enough to put every one on the timeline.
                    if inspect::armed(&host_t) {
                        inspect::publish(
                            &host_t,
                            Frame::rx("tcp", "scan")
                                .job(id)
                                .remote(addr)
                                .summary(match &banner {
                                    Some(b) => format!("port {port} open — {b}"),
                                    None => format!("port {port} open"),
                                })
                                .verdict("open"),
                        );
                    }

                    host_t.emit(
                        "scan://open",
                        OpenPort {
                            job_id: id,
                            ts: now_ms(),
                            port,
                            banner,
                        },
                    );
                }
                let d = done_t.fetch_add(1, Ordering::Relaxed) + 1;
                // Emit progress roughly every 1% (and always on the last port).
                if d % (total / 100 + 1) == 0 || d == total {
                    host_t.emit(
                        "scan://progress",
                        ScanProgress {
                            job_id: id,
                            ts: now_ms(),
                            done: d,
                            total,
                            open: open_t.load(Ordering::Relaxed),
                            unchecked: unchecked_t.load(Ordering::Relaxed),
                        },
                    );
                }
            });
        }

        while tasks.join_next().await.is_some() {}
        host_cl.emit(
            "scan://progress",
            ScanProgress {
                job_id: id,
                ts: now_ms(),
                done: total,
                total,
                open: open.load(Ordering::Relaxed),
                unchecked: unchecked.load(Ordering::Relaxed),
            },
        );
        host_cl.emit(
            "job://ended",
            serde_json::json!({ "job_id": id, "kind": "scan", "error": serde_json::Value::Null }),
        );
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

async fn read_banner(stream: &mut TcpStream) -> Option<String> {
    let mut buf = [0u8; 256];
    match tokio::time::timeout(Duration::from_millis(400), stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let s = String::from_utf8_lossy(&buf[..n])
                .replace(['\r', '\n'], " ")
                .trim()
                .to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        }
        _ => None,
    }
}
