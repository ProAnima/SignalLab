//! TCP connect scanner: sweeps a port range on a host, reports open ports and a
//! best-effort service banner. Concurrency-bounded so it stays polite.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

use super::jobs::{now_ms, JobInfo, JobRegistry};

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
}

pub async fn start_scan(
    app: AppHandle,
    jobs: JobRegistry,
    cfg: ScanConfig,
) -> Result<JobInfo, String> {
    if cfg.host.trim().is_empty() {
        return Err("host is required".into());
    }
    let (start_port, end_port) = if cfg.port_start <= cfg.port_end {
        (cfg.port_start, cfg.port_end)
    } else {
        (cfg.port_end, cfg.port_start)
    };
    let total = (end_port as u64) - (start_port as u64) + 1;
    let concurrency = cfg.concurrency.clamp(1, 1024) as usize;
    let timeout = Duration::from_millis(cfg.timeout_ms.clamp(50, 10_000));

    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "scan".into(),
        label: format!("Scan {} :{}-{}", cfg.host, start_port, end_port),
        started_ms: now_ms(),
    };

    let app_cl = app.clone();
    let jobs_cl = jobs.clone();
    let host = cfg.host.clone();
    let grab = cfg.grab_banner;
    let handle = tauri::async_runtime::spawn(async move {
        let sem = Arc::new(Semaphore::new(concurrency));
        let done = Arc::new(AtomicU64::new(0));
        let open = Arc::new(AtomicU64::new(0));
        let mut tasks = Vec::new();

        for port in start_port..=end_port {
            let permit = match sem.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => break,
            };
            let (app_t, host_t, done_t, open_t) =
                (app_cl.clone(), host.clone(), done.clone(), open.clone());
            tasks.push(tokio::spawn(async move {
                let _permit = permit;
                let addr = format!("{host_t}:{port}");
                if let Ok(Ok(mut stream)) =
                    tokio::time::timeout(timeout, TcpStream::connect(&addr)).await
                {
                    let banner = if grab {
                        read_banner(&mut stream).await
                    } else {
                        None
                    };
                    open_t.fetch_add(1, Ordering::Relaxed);
                    let _ = app_t.emit(
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
                    let _ = app_t.emit(
                        "scan://progress",
                        ScanProgress {
                            job_id: id,
                            ts: now_ms(),
                            done: d,
                            total,
                            open: open_t.load(Ordering::Relaxed),
                        },
                    );
                }
            }));
        }

        for t in tasks {
            let _ = t.await;
        }
        let _ = app_cl.emit(
            "scan://progress",
            ScanProgress {
                job_id: id,
                ts: now_ms(),
                done: total,
                total,
                open: open.load(Ordering::Relaxed),
            },
        );
        let _ = app_cl.emit(
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
