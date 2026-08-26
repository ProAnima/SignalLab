//! HTTP client: single request inspector + concurrent load ("burst") runner.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<String>,
    /// Per-request timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_timeout() -> u64 {
    10_000
}

#[derive(Clone, Serialize)]
pub struct HttpResponse {
    pub ok: bool,
    pub status: u16,
    pub status_text: String,
    pub latency_ms: f64,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub body_bytes: usize,
    /// True if the body was truncated for display.
    pub truncated: bool,
    pub error: Option<String>,
}

const MAX_BODY_PREVIEW: usize = 256 * 1024;

fn build_client(timeout_ms: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(timeout_ms.max(1)))
        .danger_accept_invalid_certs(false)
        .user_agent("SignalLab/0.1")
        .build()
        .map_err(|e| e.to_string())
}

async fn execute(client: &reqwest::Client, req: &HttpRequest) -> HttpResponse {
    let method = reqwest::Method::from_bytes(req.method.to_uppercase().as_bytes())
        .unwrap_or(reqwest::Method::GET);
    let mut builder = client.request(method, &req.url);
    for (k, v) in &req.headers {
        if !k.trim().is_empty() {
            builder = builder.header(k, v);
        }
    }
    if let Some(b) = &req.body {
        if !b.is_empty() {
            builder = builder.body(b.clone());
        }
    }

    let start = Instant::now();
    match builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers: Vec<(String, String)> = resp
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let full = resp.bytes().await.unwrap_or_default();
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            let body_bytes = full.len();
            let truncated = body_bytes > MAX_BODY_PREVIEW;
            let slice = &full[..body_bytes.min(MAX_BODY_PREVIEW)];
            HttpResponse {
                ok: status.is_success(),
                status: status.as_u16(),
                status_text: status.canonical_reason().unwrap_or("").to_string(),
                latency_ms,
                headers,
                body: String::from_utf8_lossy(slice).into_owned(),
                body_bytes,
                truncated,
                error: None,
            }
        }
        Err(e) => HttpResponse {
            ok: false,
            status: 0,
            status_text: String::new(),
            latency_ms: start.elapsed().as_secs_f64() * 1000.0,
            headers: Vec::new(),
            body: String::new(),
            body_bytes: 0,
            truncated: false,
            error: Some(e.to_string()),
        },
    }
}

/// Response body kept in a capture frame's detail pane.
const FRAME_BODY_PREVIEW: usize = 2_000;

/// Render one request/response exchange as a capture frame.
fn exchange_frame(req: &HttpRequest, resp: &HttpResponse, job_id: Option<u64>) -> Frame {
    let summary = match &resp.error {
        Some(e) => format!("{} {} → {e}", req.method, req.url),
        None => format!(
            "{} {} → {} in {:.0}ms",
            req.method, req.url, resp.status, resp.latency_ms
        ),
    };

    let mut detail = String::new();
    for (k, v) in &resp.headers {
        detail.push_str(&format!("{k}: {v}\n"));
    }
    if !resp.body.is_empty() {
        detail.push('\n');
        detail.extend(resp.body.chars().take(FRAME_BODY_PREVIEW));
        if resp.body.len() > FRAME_BODY_PREVIEW {
            detail.push_str("\n… (body truncated)");
        }
    }

    let mut frame = Frame::tx("http", "http")
        .remote(&req.url)
        .size(resp.body_bytes)
        .summary(summary)
        .detail(detail)
        .verdict(match &resp.error {
            Some(_) => "failed".to_string(),
            None => format!("{} {}", resp.status, resp.status_text),
        });
    if let Some(id) = job_id {
        frame = frame.job(id);
    }
    frame
}

/// Fire a single request and return the full response for the inspector.
pub async fn request_once(app: AppHandle, req: HttpRequest) -> Result<HttpResponse, String> {
    let client = build_client(req.timeout_ms)?;
    let resp = execute(&client, &req).await;
    if inspect::armed(&app) {
        inspect::publish(&app, exchange_frame(&req, &resp, None));
    }
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Burst / load runner
// ---------------------------------------------------------------------------

#[derive(Clone, Deserialize)]
pub struct BurstConfig {
    #[serde(flatten)]
    pub request: HttpRequest,
    /// Number of concurrent workers.
    pub concurrency: u32,
    /// Total requests to send (0 = run for duration_s).
    #[serde(default)]
    pub total: u64,
    /// Optional time limit in seconds (0 = until total reached / stopped).
    #[serde(default)]
    pub duration_s: f64,
}

#[derive(Clone, Serialize)]
struct BurstProgress {
    job_id: u64,
    ts: u64,
    sent: u64,
    ok: u64,
    failed: u64,
    /// Rolling requests-per-second over the last window.
    rps: f64,
    last_latency_ms: f64,
    min_latency_ms: f64,
    max_latency_ms: f64,
    avg_latency_ms: f64,
}

pub async fn start_burst(
    app: AppHandle,
    jobs: JobRegistry,
    cfg: BurstConfig,
) -> Result<JobInfo, String> {
    let client = build_client(cfg.request.timeout_ms)?;
    let concurrency = cfg.concurrency.clamp(1, 512);

    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "http-burst".into(),
        label: format!("HTTP burst {} {}", cfg.request.method, cfg.request.url),
        started_ms: now_ms(),
    };

    let app_cl = app.clone();
    let jobs_cl = jobs.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let sent = Arc::new(AtomicU64::new(0));
        let ok = Arc::new(AtomicU64::new(0));
        let failed = Arc::new(AtomicU64::new(0));
        let lat_sum = Arc::new(AtomicU64::new(0)); // microseconds
        let lat_min = Arc::new(AtomicU64::new(u64::MAX));
        let lat_max = Arc::new(AtomicU64::new(0));
        let last_lat = Arc::new(AtomicU64::new(0)); // microseconds
        let stop = Arc::new(AtomicBool::new(false));

        let request = Arc::new(cfg.request.clone());
        let client = Arc::new(client);
        let total = cfg.total;
        let claimed = Arc::new(AtomicU64::new(0));
        let start = Instant::now();

        // Reporter task: emits progress ~10 Hz.
        let reporter = {
            let (app_r, sent_r, ok_r, failed_r) =
                (app_cl.clone(), sent.clone(), ok.clone(), failed.clone());
            let (sum_r, min_r, max_r, last_r, stop_r) = (
                lat_sum.clone(),
                lat_min.clone(),
                lat_max.clone(),
                last_lat.clone(),
                stop.clone(),
            );
            tauri::async_runtime::spawn(async move {
                let mut prev_sent = 0u64;
                let mut prev = Instant::now();
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    let s = sent_r.load(Ordering::Relaxed);
                    let now = Instant::now();
                    let dt = now.duration_since(prev).as_secs_f64().max(1e-6);
                    let rps = (s - prev_sent) as f64 / dt;
                    prev_sent = s;
                    prev = now;
                    let done = ok_r.load(Ordering::Relaxed) + failed_r.load(Ordering::Relaxed);
                    let sum = sum_r.load(Ordering::Relaxed);
                    let avg = if done > 0 {
                        (sum as f64 / done as f64) / 1000.0
                    } else {
                        0.0
                    };
                    let minv = min_r.load(Ordering::Relaxed);
                    let _ = app_r.emit(
                        "http://burst-progress",
                        BurstProgress {
                            job_id: id,
                            ts: now_ms(),
                            sent: s,
                            ok: ok_r.load(Ordering::Relaxed),
                            failed: failed_r.load(Ordering::Relaxed),
                            rps,
                            last_latency_ms: last_r.load(Ordering::Relaxed) as f64 / 1000.0,
                            min_latency_ms: if minv == u64::MAX {
                                0.0
                            } else {
                                minv as f64 / 1000.0
                            },
                            max_latency_ms: max_r.load(Ordering::Relaxed) as f64 / 1000.0,
                            avg_latency_ms: avg,
                        },
                    );
                    if stop_r.load(Ordering::Relaxed) {
                        break;
                    }
                }
            })
        };

        // Worker pool. A burst can push thousands of requests per second, so the
        // pool shares one sampling gate into the Inspector.
        let gate = Arc::new(Gate::new(100));
        let mut workers = Vec::with_capacity(concurrency as usize);
        for _ in 0..concurrency {
            let client = client.clone();
            let request = request.clone();
            let claimed = claimed.clone();
            let gate = gate.clone();
            let app_w = app_cl.clone();
            let (sent, ok, failed) = (sent.clone(), ok.clone(), failed.clone());
            let (lat_sum, lat_min, lat_max, last_lat) = (
                lat_sum.clone(),
                lat_min.clone(),
                lat_max.clone(),
                last_lat.clone(),
            );
            let duration_s = cfg.duration_s;
            workers.push(tauri::async_runtime::spawn(async move {
                loop {
                    if duration_s > 0.0 && start.elapsed().as_secs_f64() >= duration_s {
                        break;
                    }
                    if total > 0 && claimed.fetch_add(1, Ordering::Relaxed) >= total {
                        break;
                    }
                    let resp = execute(&client, &request).await;
                    if inspect::armed(&app_w) && gate.allow() {
                        inspect::publish(&app_w, exchange_frame(&request, &resp, Some(id)));
                    }
                    sent.fetch_add(1, Ordering::Relaxed);
                    let micros = (resp.latency_ms * 1000.0) as u64;
                    last_lat.store(micros, Ordering::Relaxed);
                    lat_sum.fetch_add(micros, Ordering::Relaxed);
                    lat_min.fetch_min(micros, Ordering::Relaxed);
                    lat_max.fetch_max(micros, Ordering::Relaxed);
                    if resp.error.is_none() && resp.ok {
                        ok.fetch_add(1, Ordering::Relaxed);
                    } else {
                        failed.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }));
        }

        for w in workers {
            let _ = w.await;
        }
        stop.store(true, Ordering::Relaxed);
        let _ = reporter.await;

        let _ = app_cl.emit(
            "job://ended",
            serde_json::json!({
                "job_id": id,
                "kind": "http-burst",
                "error": serde_json::Value::Null,
            }),
        );
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}
