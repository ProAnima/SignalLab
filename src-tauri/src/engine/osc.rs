//! OSC monitor (listen + decode) and generator (waveform / pattern sender).

use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::net::UdpSocket;

use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::osc_codec::{
    arg_str, decode_packet, encode_message, summarize_messages, OscArg, OscMessage,
};

/// One decoded inbound OSC packet, forwarded to the UI on `osc://message`.
#[derive(Clone, Serialize)]
struct OscInbound {
    job_id: u64,
    ts: u64,
    from: String,
    bytes: usize,
    messages: Vec<OscMessage>,
    /// Decode error, if the packet was malformed.
    error: Option<String>,
}

#[derive(Clone, Serialize)]
struct JobEnded {
    job_id: u64,
    kind: String,
    error: Option<String>,
}

/// Every message in a packet, one per line, for the Inspector's detail pane.
fn detail_lines(messages: &[OscMessage]) -> String {
    messages
        .iter()
        .map(|m| {
            let args = m.args.iter().map(arg_str).collect::<Vec<_>>().join(" ");
            if args.is_empty() {
                m.address.clone()
            } else {
                format!("{} {}", m.address, args)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn emit_ended(app: &AppHandle, job_id: u64, kind: &str, error: Option<String>) {
    let _ = app.emit(
        "job://ended",
        JobEnded {
            job_id,
            kind: kind.to_string(),
            error,
        },
    );
}

// ---------------------------------------------------------------------------
// Monitor
// ---------------------------------------------------------------------------

pub async fn start_monitor(
    app: AppHandle,
    jobs: JobRegistry,
    bind: String,
) -> Result<JobInfo, String> {
    let addr: SocketAddr = bind
        .parse()
        .map_err(|e| format!("invalid bind address '{bind}': {e}"))?;
    let socket = UdpSocket::bind(addr)
        .await
        .map_err(|e| format!("bind {bind} failed: {e}"))?;
    let local = socket
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or(bind.clone());

    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "osc-monitor".into(),
        label: format!("OSC monitor {local}"),
        started_ms: now_ms(),
    };

    let jobs_cl = jobs.clone();
    let app_cl = app.clone();
    let local_cl = local.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let mut buf = vec![0u8; 65_536];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((n, from)) => {
                    let (messages, error) = match decode_packet(&buf[..n]) {
                        Ok(m) => (m, None),
                        Err(e) => (Vec::new(), Some(e)),
                    };

                    if inspect::armed(&app_cl) {
                        let mut frame = Frame::rx("osc", "osc-monitor")
                            .job(id)
                            .local(&local_cl)
                            .remote(from)
                            .payload(&buf[..n]);
                        frame = match &error {
                            Some(e) => frame
                                .summary(format!("malformed packet ({n} B)"))
                                .verdict(format!("decode error: {e}")),
                            None => frame
                                .summary(summarize_messages(&messages))
                                .detail(detail_lines(&messages)),
                        };
                        inspect::publish(&app_cl, frame);
                    }

                    let _ = app_cl.emit(
                        "osc://message",
                        OscInbound {
                            job_id: id,
                            ts: now_ms(),
                            from: from.to_string(),
                            bytes: n,
                            messages,
                            error,
                        },
                    );
                }
                Err(e) => {
                    emit_ended(&app_cl, id, "osc-monitor", Some(e.to_string()));
                    break;
                }
            }
        }
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

#[derive(Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Waveform {
    Sine,
    Triangle,
    Saw,
    Square,
    Random,
    Ramp,
    Constant,
}

#[derive(Clone, Deserialize)]
pub struct GenConfig {
    pub target: String,
    pub address: String,
    /// Packets per second.
    pub rate: f64,
    pub waveform: Waveform,
    /// Cycles per second of the waveform envelope.
    pub freq: f64,
    pub min: f64,
    pub max: f64,
    /// Send the value as int (`i`) instead of float (`f`).
    #[serde(default)]
    pub as_int: bool,
    /// Optional fixed run duration in seconds (0 = run until stopped).
    #[serde(default)]
    pub duration_s: f64,
}

#[derive(Clone, Serialize)]
struct GenTick {
    job_id: u64,
    ts: u64,
    value: f64,
    sent: u64,
}

pub async fn start_generator(
    app: AppHandle,
    jobs: JobRegistry,
    cfg: GenConfig,
) -> Result<JobInfo, String> {
    let target: SocketAddr = cfg
        .target
        .parse()
        .map_err(|e| format!("invalid target '{}': {e}", cfg.target))?;
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("socket create failed: {e}"))?;
    socket
        .connect(target)
        .await
        .map_err(|e| format!("connect {target} failed: {e}"))?;
    let socket = Arc::new(socket);

    let rate = cfg.rate.clamp(0.1, 5_000.0);
    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "osc-gen".into(),
        label: format!("OSC gen → {} {}", cfg.target, cfg.address),
        started_ms: now_ms(),
    };

    let app_cl = app.clone();
    let jobs_cl = jobs.clone();
    let gen_local = socket
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_default();
    let handle = tauri::async_runtime::spawn(async move {
        // The generator can run at thousands of pps; the Inspector only wants a
        // representative sample of that.
        let gate = Gate::new(100);
        let period = std::time::Duration::from_secs_f64(1.0 / rate);
        let mut ticker = tokio::time::interval(period);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let span = (cfg.max - cfg.min).max(0.0);
        let mid = (cfg.max + cfg.min) / 2.0;
        let amp = span / 2.0;
        let start = std::time::Instant::now();
        let mut sent: u64 = 0;
        let mut rng_state: u64 = 0x9E3779B97F4A7C15 ^ (id as u64).wrapping_mul(2654435761);

        loop {
            ticker.tick().await;
            let t = start.elapsed().as_secs_f64();
            if cfg.duration_s > 0.0 && t >= cfg.duration_s {
                emit_ended(&app_cl, id, "osc-gen", None);
                break;
            }
            let phase = (cfg.freq * t).fract();
            let unit = match cfg.waveform {
                Waveform::Sine => (2.0 * std::f64::consts::PI * phase).sin() * 0.5 + 0.5,
                Waveform::Triangle => 1.0 - (2.0 * phase - 1.0).abs(),
                Waveform::Saw => phase,
                Waveform::Ramp => phase,
                Waveform::Square => {
                    if phase < 0.5 {
                        1.0
                    } else {
                        0.0
                    }
                }
                Waveform::Random => {
                    // xorshift64 for a cheap deterministic-ish noise source
                    rng_state ^= rng_state << 13;
                    rng_state ^= rng_state >> 7;
                    rng_state ^= rng_state << 17;
                    (rng_state >> 11) as f64 / (1u64 << 53) as f64
                }
                Waveform::Constant => 1.0,
            };
            let value = match cfg.waveform {
                Waveform::Sine => mid + amp * (2.0 * (unit - 0.5)),
                _ => cfg.min + span * unit,
            };

            let arg = if cfg.as_int {
                OscArg::Int(value.round() as i32)
            } else {
                OscArg::Float(value as f32)
            };
            let packet = encode_message(&cfg.address, std::slice::from_ref(&arg));
            if let Err(e) = socket.send(&packet).await {
                emit_ended(&app_cl, id, "osc-gen", Some(e.to_string()));
                break;
            }
            sent += 1;

            if inspect::armed(&app_cl) && gate.allow() {
                inspect::publish(
                    &app_cl,
                    Frame::tx("osc", "osc-gen")
                        .job(id)
                        .local(&gen_local)
                        .remote(&cfg.target)
                        .payload(&packet)
                        .summary(format!("{} {}", cfg.address, arg_str(&arg)))
                        .verdict("sampled"),
                );
            }

            // Throttle UI telemetry to ~30 Hz regardless of send rate.
            if sent % ((rate / 30.0).max(1.0) as u64) == 0 {
                let _ = app_cl.emit(
                    "osc://gen-tick",
                    GenTick {
                        job_id: id,
                        ts: now_ms(),
                        value,
                        sent,
                    },
                );
            }
        }
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

/// Send a single OSC message immediately (fire-and-forget, no job).
pub async fn send_once(
    app: AppHandle,
    target: String,
    address: String,
    args: Vec<OscArg>,
) -> Result<usize, String> {
    let addr: SocketAddr = target
        .parse()
        .map_err(|e| format!("invalid target '{target}': {e}"))?;
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| format!("socket create failed: {e}"))?;
    let packet = encode_message(&address, &args);
    let sent = socket
        .send_to(&packet, addr)
        .await
        .map_err(|e| format!("send failed: {e}"))?;

    if inspect::armed(&app) {
        let arg_text = args.iter().map(arg_str).collect::<Vec<_>>().join(" ");
        inspect::publish(
            &app,
            Frame::tx("osc", "osc-send")
                .local(
                    socket
                        .local_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_default(),
                )
                .remote(addr)
                .payload(&packet)
                .summary(if arg_text.is_empty() {
                    address
                } else {
                    format!("{address} {arg_text}")
                }),
        );
    }
    Ok(sent)
}
