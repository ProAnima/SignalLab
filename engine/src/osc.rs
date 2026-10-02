//! OSC monitor (listen + decode) and generator (waveform / pattern sender).

use std::net::SocketAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::net::UdpSocket;

use super::error::{EngineError, EngineResult};
use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::net;
use super::osc_codec::{
    arg_str, decode_packet, encode_message, summarize_messages, OscArg, OscMessage,
};
use super::transport::{self, Cause};

/// One decoded inbound OSC packet, forwarded to the UI on `osc://message`.
#[derive(Clone, Serialize)]
struct OscInbound {
    job_id: u64,
    ts: u64,
    from: String,
    bytes: usize,
    messages: Vec<OscMessage>,
    /// `osc.packet_malformed`, the decoder's text as detail, if the packet was malformed.
    error: Option<EngineError>,
}

#[derive(Clone, Serialize)]
struct JobEnded {
    job_id: u64,
    kind: String,
    error: Option<EngineError>,
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

fn emit_ended(host: &Host, job_id: u64, kind: &str, error: Option<EngineError>) {
    host.emit(
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

/// A local address to listen on, `IP:port`.
pub(crate) fn parse_bind(bind: &str) -> EngineResult<SocketAddr> {
    bind.trim().parse().map_err(|_| EngineError::new("node.bind_invalid").with("value", bind))
}

/// A destination, `IP:port`.
pub(crate) fn parse_target(target: &str) -> EngineResult<SocketAddr> {
    target.trim().parse().map_err(|_| Cause::TargetInvalid.error(target))
}

pub async fn start_monitor(
    host: Host,
    jobs: JobRegistry,
    bind: String,
) -> EngineResult<JobInfo> {
    let addr = parse_bind(&bind)?;
    let socket = UdpSocket::bind(addr)
        .await
        .map_err(|e| net::bind_error(&bind, e))?;
    let local = socket
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or(bind.clone());

    let id = jobs.next_id();
    let info = JobInfo::new(id, "osc-monitor", format!("OSC monitor {local}")).with("bind", &local);

    let jobs_cl = jobs.clone();
    let host_cl = host.clone();
    let local_cl = local.clone();
    let handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 65_536];
        loop {
            match socket.recv_from(&mut buf).await {
                Err(e) if crate::net::udp_transient(&e) => continue,
                Ok((n, from)) => {
                    let (messages, error) = match decode_packet(&buf[..n]) {
                        Ok(m) => (m, None),
                        Err(e) => (Vec::new(), Some(e)),
                    };

                    if inspect::armed(&host_cl) {
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
                        inspect::publish(&host_cl, frame);
                    }

                    host_cl.emit(
                        "osc://message",
                        OscInbound {
                            job_id: id,
                            ts: now_ms(),
                            from: from.to_string(),
                            bytes: n,
                            messages,
                            error: error.map(|e| EngineError::new("osc.packet_malformed").with("bytes", n).because(e)),
                        },
                    );
                }
                Err(e) => {
                    let error = EngineError::new("wait.receive_failed").with("target", &local_cl).because(e);
                    emit_ended(&host_cl, id, "osc-monitor", Some(error));
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
    host: Host,
    jobs: JobRegistry,
    cfg: GenConfig,
) -> EngineResult<JobInfo> {
    let target = parse_target(&cfg.target)?;
    let local = if target.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let socket = UdpSocket::bind(local)
        .await
        .map_err(|e| net::bind_error(local, e))?;
    socket
        .connect(target)
        .await
        .map_err(|e| transport::of_io(&e).error(&cfg.target).because(e))?;
    let socket = Arc::new(socket);

    let rate = cfg.rate.clamp(0.1, 5_000.0);
    let id = jobs.next_id();
    let info = JobInfo::new(id, "osc-gen", format!("OSC gen → {} {}", cfg.target, cfg.address))
        .with("target", &cfg.target)
        .with("address", &cfg.address);

    let host_cl = host.clone();
    let jobs_cl = jobs.clone();
    let gen_local = socket
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_default();
    let handle = tokio::spawn(async move {
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
        let mut rng_state: u64 = 0x9E3779B97F4A7C15 ^ id.wrapping_mul(2654435761);

        loop {
            ticker.tick().await;
            let t = start.elapsed().as_secs_f64();
            if cfg.duration_s > 0.0 && t >= cfg.duration_s {
                emit_ended(&host_cl, id, "osc-gen", None);
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
                emit_ended(&host_cl, id, "osc-gen", Some(transport::of_io(&e).error(&cfg.target).because(e)));
                break;
            }
            sent += 1;

            if inspect::armed(&host_cl) && gate.allow() {
                inspect::publish(
                    &host_cl,
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
            if sent.is_multiple_of((rate / 30.0).max(1.0) as u64) {
                host_cl.emit(
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
    host: Host,
    target: String,
    address: String,
    args: Vec<OscArg>,
) -> EngineResult<usize> {
    let addr = parse_target(&target)?;
    send_to(&host, addr, address, &args).await.map_err(|e| transport::of_io(&e).error(&target).because(e))
}

/// The send itself, with the socket error intact so a caller can tell
/// "unreachable" from "not allowed" (experiments report it localized).
pub async fn send_to(
    host: &Host,
    addr: SocketAddr,
    address: String,
    args: &[OscArg],
) -> std::io::Result<usize> {
    let socket = UdpSocket::bind(if addr.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" }).await?;
    let packet = encode_message(&address, args);
    let sent = socket.send_to(&packet, addr).await?;

    if inspect::armed(host) {
        let arg_text = args.iter().map(arg_str).collect::<Vec<_>>().join(" ");
        inspect::publish(
            host,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Recorder;
    use crate::inspect::Capture;
    use std::time::Duration;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_malformed_packet_is_reported_with_a_code_and_the_decoder_s_text() {
        let recorder = Recorder::new();
        let host = Host::new(recorder.clone(), Capture::new());
        let jobs = JobRegistry::new();
        let job = start_monitor(host, jobs.clone(), "127.0.0.1:0".into()).await.unwrap();
        assert_eq!((job.kind.as_str(), job.params["bind"].starts_with("127.0.0.1:")), ("osc-monitor", true));
        let sender = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        sender.send_to(b"/x\0\0,i\0\0", job.params["bind"].as_str()).await.unwrap();
        let message = tokio::task::spawn_blocking(move || recorder.wait_for("osc://message", Duration::from_secs(2), |_| true)).await.unwrap().expect("the packet arrived");
        assert_eq!(message["error"]["code"], "osc.packet_malformed", "{message}");
        assert_eq!(message["error"]["params"]["bytes"], "8");
        assert!(message["error"]["detail"].as_str().is_some_and(|detail| !detail.is_empty()));
        jobs.stop(job.id);
    }

    #[tokio::test]
    async fn addresses_are_checked_before_anything_is_sent() {
        let host = Host::new(Recorder::new(), Capture::new());
        let jobs = JobRegistry::new();
        let bad = start_monitor(host.clone(), jobs.clone(), "9000".into()).await.unwrap_err();
        assert_eq!((bad.code.as_str(), bad.params["value"].as_str()), ("node.bind_invalid", "9000"));
        let target = send_once(host.clone(), "localhost:9000".into(), "/x".into(), vec![]).await.unwrap_err();
        assert_eq!((target.code.as_str(), target.params["target"].as_str()), ("transport.target_invalid", "localhost:9000"));
        let cfg = GenConfig { target: "nowhere".into(), address: "/x".into(), rate: 1.0, waveform: Waveform::Sine, freq: 1.0, min: 0.0, max: 1.0, as_int: false, duration_s: 0.0 };
        assert!(start_generator(host, jobs.clone(), cfg).await.unwrap_err().is("transport.target_invalid"));
        assert!(jobs.list().is_empty());
    }
}
