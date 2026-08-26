//! Broadcast / multicast fan-out and device discovery.
//!
//! Two halves that mirror each other:
//!
//! * **Emitter** — sends a payload (OSC, text or raw hex) to many destinations
//!   at once: an explicit list, a broadcast address (`SO_BROADCAST`), a
//!   multicast group, or every host in a CIDR block ("sweep"). One-shot, or as
//!   a repeating beacon.
//! * **Discovery** — binds a port (optionally joining multicast groups with
//!   `SO_REUSEADDR` so it can share the port with the real service), tracks every
//!   peer that talks, and can auto-reply to simulate a device answering probes.
//!
//! Both halves feed the capture bus, so a beacon and the replies it triggers
//! land on the same Inspector timeline.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::net::UdpSocket;

use super::inspect::{self, describe_payload, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::osc_codec::{encode_message, OscArg};

/// Guard rail: a sweep never expands past this many hosts.
const MAX_SWEEP_HOSTS: usize = 1024;
/// Guard rail on aggregate send rate (targets × rounds/sec).
const MAX_AGGREGATE_PPS: f64 = 50_000.0;
const MAX_PEERS: usize = 512;

// ---------------------------------------------------------------------------
// Payload
// ---------------------------------------------------------------------------

/// What to put on the wire. `hex` lets you replay a captured frame verbatim or
/// hand-craft a foreign discovery protocol's magic bytes.
#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Payload {
    Osc {
        address: String,
        #[serde(default)]
        args: Vec<OscArg>,
    },
    Text {
        text: String,
    },
    Hex {
        hex: String,
    },
}

impl Payload {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        match self {
            Payload::Osc { address, args } => {
                if !address.starts_with('/') {
                    return Err(format!("OSC address must start with '/': '{address}'"));
                }
                Ok(encode_message(address, args))
            }
            Payload::Text { text } => Ok(text.as_bytes().to_vec()),
            Payload::Hex { hex } => parse_hex(hex),
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Payload::Osc { address, args } => {
                if args.is_empty() {
                    address.clone()
                } else {
                    format!(
                        "{address} {}",
                        args.iter()
                            .map(super::osc_codec::arg_str)
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                }
            }
            Payload::Text { text } => text.chars().take(80).collect(),
            Payload::Hex { hex } => format!("hex[{}]", hex.trim()),
        }
    }
}

/// Accepts "de ad be ef", "deadbeef", "0xDE,0xAD" — anything non-hex is ignored.
fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = s
        .replace("0x", "")
        .replace("0X", "")
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    if cleaned.is_empty() {
        return Err("hex payload is empty".into());
    }
    if cleaned.len() % 2 != 0 {
        return Err(format!(
            "hex payload has an odd number of digits ({})",
            cleaned.len()
        ));
    }
    (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// Targets
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TargetMode {
    /// Comma/space separated `host:port` list.
    List,
    /// A single broadcast address, e.g. `255.255.255.255:9000` or `10.0.0.255:9000`.
    Broadcast,
    /// A multicast group, e.g. `239.1.1.1:9000`.
    Multicast,
    /// Every usable host in a CIDR block, e.g. `192.168.1.0/24` on `port`.
    Sweep,
}

fn is_broadcastish(ip: Ipv4Addr) -> bool {
    ip.is_broadcast() || ip.octets()[3] == 255
}

async fn resolve_one(spec: &str) -> Result<SocketAddr, String> {
    let spec = spec.trim();
    if let Ok(addr) = spec.parse::<SocketAddr>() {
        return Ok(addr);
    }
    // Fall back to DNS so "server.local:9000" works.
    let mut iter = tokio::net::lookup_host(spec)
        .await
        .map_err(|e| format!("cannot resolve '{spec}': {e}"))?;
    iter.next().ok_or_else(|| format!("no address for '{spec}'"))
}

/// `192.168.1.0/24` → every usable host address on `port`.
fn sweep_hosts(cidr: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    if port == 0 {
        return Err("sweep needs a port".into());
    }
    let (base, prefix) = cidr
        .trim()
        .split_once('/')
        .ok_or_else(|| format!("'{cidr}' is not CIDR notation (expected a.b.c.d/nn)"))?;
    let base: Ipv4Addr = base
        .parse()
        .map_err(|e| format!("invalid network address '{base}': {e}"))?;
    let prefix: u32 = prefix
        .parse()
        .map_err(|e| format!("invalid prefix '/{prefix}': {e}"))?;
    if prefix > 32 {
        return Err("prefix must be 0–32".into());
    }
    let host_bits = 32 - prefix;
    let count = 1u64 << host_bits;
    if count > MAX_SWEEP_HOSTS as u64 + 2 {
        return Err(format!(
            "/{prefix} expands to {count} addresses — narrow it to /{} or smaller",
            32 - (MAX_SWEEP_HOSTS as f64).log2().floor() as u32
        ));
    }

    let net = u32::from(base) & (!0u32).checked_shl(host_bits).unwrap_or(0);
    let mut out = Vec::new();
    // /31 and /32 have no network/broadcast address to skip.
    let (first, last) = if host_bits <= 1 {
        (0, count - 1)
    } else {
        (1, count - 2)
    };
    for i in first..=last {
        let ip = Ipv4Addr::from(net + i as u32);
        out.push(SocketAddr::new(IpAddr::V4(ip), port));
    }
    Ok(out)
}

async fn resolve_targets(
    mode: TargetMode,
    target: &str,
    port: u16,
) -> Result<Vec<SocketAddr>, String> {
    let target = target.trim();
    if target.is_empty() {
        return Err("target is required".into());
    }
    match mode {
        TargetMode::List => {
            let mut out = Vec::new();
            for part in target.split([',', ';', '\n']) {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                out.push(resolve_one(part).await?);
            }
            if out.is_empty() {
                return Err("target list is empty".into());
            }
            Ok(out)
        }
        TargetMode::Broadcast => {
            let addr = resolve_one(target).await?;
            match addr.ip() {
                IpAddr::V4(ip) if is_broadcastish(ip) => Ok(vec![addr]),
                IpAddr::V4(_) => Err(format!(
                    "'{target}' is not a broadcast address — use 255.255.255.255:port or x.x.x.255:port"
                )),
                IpAddr::V6(_) => Err("IPv6 has no broadcast — use multicast instead".into()),
            }
        }
        TargetMode::Multicast => {
            let addr = resolve_one(target).await?;
            if !addr.ip().is_multicast() {
                return Err(format!(
                    "'{target}' is not a multicast group (expected 224.0.0.0–239.255.255.255)"
                ));
            }
            Ok(vec![addr])
        }
        TargetMode::Sweep => sweep_hosts(target, port),
    }
}

// ---------------------------------------------------------------------------
// Emitter
// ---------------------------------------------------------------------------

#[derive(Clone, Deserialize)]
pub struct EmitConfig {
    pub mode: TargetMode,
    pub target: String,
    /// Port for `sweep` mode (the other modes carry it in `target`).
    #[serde(default)]
    pub port: u16,
    pub payload: Payload,
    /// Local bind, e.g. "0.0.0.0:0" — pin it to fix the source port or interface.
    #[serde(default)]
    pub bind: Option<String>,
    /// IP TTL / multicast hop limit.
    #[serde(default = "default_ttl")]
    pub ttl: u32,
    /// Loop multicast back to this host (so a local listener sees it).
    #[serde(default = "default_true")]
    pub multicast_loop: bool,
    /// Beacon rounds per second (one round = one packet per target).
    #[serde(default)]
    pub rate: f64,
    /// Stop after this many rounds (0 = unlimited).
    #[serde(default)]
    pub count: u64,
    /// Stop after this long (0 = until stopped).
    #[serde(default)]
    pub duration_s: f64,
}

fn default_ttl() -> u32 {
    1
}
fn default_true() -> bool {
    true
}

#[derive(Clone, Serialize)]
pub struct EmitResult {
    pub targets: usize,
    pub packets: u64,
    pub bytes: u64,
    pub errors: u64,
    /// First few resolved destinations, for the UI to confirm what it hit.
    pub resolved: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Serialize)]
struct EmitStat {
    job_id: u64,
    ts: u64,
    rounds: u64,
    packets: u64,
    bytes: u64,
    errors: u64,
    pps: f64,
}

/// Build the send socket and apply broadcast / multicast options.
async fn emit_socket(cfg: &EmitConfig, targets: &[SocketAddr]) -> Result<UdpSocket, String> {
    let bind = cfg.bind.clone().unwrap_or_else(|| "0.0.0.0:0".to_string());
    let sock = UdpSocket::bind(&bind)
        .await
        .map_err(|e| format!("bind {bind} failed: {e}"))?;

    let wants_broadcast = cfg.mode == TargetMode::Broadcast
        || targets.iter().any(|t| match t.ip() {
            IpAddr::V4(ip) => is_broadcastish(ip),
            IpAddr::V6(_) => false,
        });
    if wants_broadcast {
        sock.set_broadcast(true)
            .map_err(|e| format!("SO_BROADCAST failed: {e} (is the address really a broadcast address?)"))?;
    }

    let ttl = cfg.ttl.clamp(1, 255);
    if cfg.mode == TargetMode::Multicast {
        sock.set_multicast_ttl_v4(ttl)
            .map_err(|e| format!("multicast TTL failed: {e}"))?;
        sock.set_multicast_loop_v4(cfg.multicast_loop)
            .map_err(|e| format!("multicast loopback failed: {e}"))?;
    } else {
        let _ = sock.set_ttl(ttl);
    }
    Ok(sock)
}

/// Send one packet per target, once. Returns what actually went out.
pub async fn send_once(app: AppHandle, cfg: EmitConfig) -> Result<EmitResult, String> {
    let targets = resolve_targets(cfg.mode, &cfg.target, cfg.port).await?;
    let bytes = cfg.payload.encode()?;
    let sock = emit_socket(&cfg, &targets).await?;
    let local = sock.local_addr().map(|a| a.to_string()).unwrap_or_default();
    let summary = cfg.payload.summary();
    let capture = inspect::armed(&app);

    let mut packets = 0u64;
    let mut sent_bytes = 0u64;
    let mut errors = 0u64;
    for t in &targets {
        match sock.send_to(&bytes, t).await {
            Ok(n) => {
                packets += 1;
                sent_bytes += n as u64;
                if capture {
                    inspect::publish(
                        &app,
                        Frame::tx(proto_of(&cfg.payload), "broadcast")
                            .local(&local)
                            .remote(t)
                            .payload(&bytes)
                            .summary(summary.clone())
                            .verdict(mode_label(cfg.mode)),
                    );
                }
            }
            Err(e) => {
                errors += 1;
                if capture {
                    inspect::publish(
                        &app,
                        Frame::tx(proto_of(&cfg.payload), "broadcast")
                            .local(&local)
                            .remote(t)
                            .size(bytes.len())
                            .summary(summary.clone())
                            .verdict(format!("error: {e}")),
                    );
                }
            }
        }
    }

    Ok(EmitResult {
        targets: targets.len(),
        packets,
        bytes: sent_bytes,
        errors,
        resolved: targets.iter().take(8).map(|a| a.to_string()).collect(),
        summary,
    })
}

fn proto_of(p: &Payload) -> &'static str {
    match p {
        Payload::Osc { .. } => "osc",
        _ => "udp",
    }
}

fn mode_label(m: TargetMode) -> &'static str {
    match m {
        TargetMode::List => "fan-out",
        TargetMode::Broadcast => "broadcast",
        TargetMode::Multicast => "multicast",
        TargetMode::Sweep => "sweep",
    }
}

/// Repeat the emit on an interval as a stoppable job.
pub async fn start_beacon(
    app: AppHandle,
    jobs: JobRegistry,
    cfg: EmitConfig,
) -> Result<JobInfo, String> {
    let targets = resolve_targets(cfg.mode, &cfg.target, cfg.port).await?;
    let payload = cfg.payload.encode()?;
    let rate = cfg.rate;
    if rate <= 0.0 {
        return Err("beacon rate must be greater than 0".into());
    }
    let aggregate = rate * targets.len() as f64;
    if aggregate > MAX_AGGREGATE_PPS {
        return Err(format!(
            "{:.0} packets/s across {} targets exceeds the {:.0} pps guard rail — lower the rate or narrow the target",
            aggregate,
            targets.len(),
            MAX_AGGREGATE_PPS
        ));
    }

    let sock = emit_socket(&cfg, &targets).await?;
    let local = sock.local_addr().map(|a| a.to_string()).unwrap_or_default();
    let summary = cfg.payload.summary();
    let proto = proto_of(&cfg.payload);
    let verdict = mode_label(cfg.mode);

    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "beacon".into(),
        label: format!(
            "Beacon {} → {} ×{} @{}/s",
            verdict,
            cfg.target,
            targets.len(),
            rate
        ),
        started_ms: now_ms(),
    };

    let app_cl = app.clone();
    let jobs_cl = jobs.clone();
    let handle = tauri::async_runtime::spawn(async move {
        let packets = Arc::new(AtomicU64::new(0));
        let bytes = Arc::new(AtomicU64::new(0));
        let errors = Arc::new(AtomicU64::new(0));
        let rounds = Arc::new(AtomicU64::new(0));
        let gate = Gate::new(50);

        let reporter = {
            let (app_r, p, b, e, r) = (
                app_cl.clone(),
                packets.clone(),
                bytes.clone(),
                errors.clone(),
                rounds.clone(),
            );
            tokio::spawn(async move {
                let mut prev = 0u64;
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    let now = p.load(Ordering::Relaxed);
                    let pps = (now - prev) as f64 * 4.0;
                    prev = now;
                    let _ = app_r.emit(
                        "broadcast://emit-stat",
                        EmitStat {
                            job_id: id,
                            ts: now_ms(),
                            rounds: r.load(Ordering::Relaxed),
                            packets: now,
                            bytes: b.load(Ordering::Relaxed),
                            errors: e.load(Ordering::Relaxed),
                            pps,
                        },
                    );
                }
            })
        };

        let mut ticker = tokio::time::interval(Duration::from_secs_f64(1.0 / rate));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let start = std::time::Instant::now();
        let mut error: Option<String> = None;

        loop {
            ticker.tick().await;
            if cfg.duration_s > 0.0 && start.elapsed().as_secs_f64() >= cfg.duration_s {
                break;
            }
            // Check before counting, so the reported round total matches what
            // actually went out.
            if cfg.count > 0 && rounds.load(Ordering::Relaxed) >= cfg.count {
                break;
            }
            rounds.fetch_add(1, Ordering::Relaxed);
            // Sample the capture: a fast beacon must not flood the Inspector.
            let capture = inspect::armed(&app_cl) && gate.allow();
            for t in &targets {
                match sock.send_to(&payload, t).await {
                    Ok(n) => {
                        packets.fetch_add(1, Ordering::Relaxed);
                        bytes.fetch_add(n as u64, Ordering::Relaxed);
                        if capture {
                            inspect::publish(
                                &app_cl,
                                Frame::tx(proto, "beacon")
                                    .job(id)
                                    .local(&local)
                                    .remote(t)
                                    .payload(&payload)
                                    .summary(summary.clone())
                                    .verdict(verdict),
                            );
                        }
                    }
                    Err(e) => {
                        errors.fetch_add(1, Ordering::Relaxed);
                        // A persistent send error (no route, blocked broadcast)
                        // would otherwise spin silently.
                        if errors.load(Ordering::Relaxed) > 32 && packets.load(Ordering::Relaxed) == 0
                        {
                            error = Some(format!("send to {t} keeps failing: {e}"));
                        }
                    }
                }
            }
            if error.is_some() {
                break;
            }
        }

        reporter.abort();
        let _ = app_cl.emit(
            "job://ended",
            serde_json::json!({ "job_id": id, "kind": "beacon", "error": error }),
        );
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

// ---------------------------------------------------------------------------
// Discovery listener
// ---------------------------------------------------------------------------

#[derive(Clone, Deserialize)]
pub struct DiscoveryConfig {
    /// e.g. "0.0.0.0:9000".
    pub bind: String,
    /// Multicast groups to join, e.g. ["239.1.1.1"].
    #[serde(default)]
    pub groups: Vec<String>,
    /// Local interface for the multicast join (default: the OS picks).
    #[serde(default)]
    pub interface: Option<String>,
    /// SO_REUSEADDR — share the port with the real service that also listens.
    #[serde(default = "default_true")]
    pub reuse: bool,
    /// Answer probes, simulating a device that responds to discovery.
    #[serde(default)]
    pub respond: bool,
    #[serde(default)]
    pub response: Option<Payload>,
    /// Delay before answering, to model a slow device.
    #[serde(default)]
    pub respond_delay_ms: u64,
    /// Only answer packets whose decoded text contains this needle.
    #[serde(default)]
    pub match_contains: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Peer {
    pub addr: String,
    pub proto: String,
    pub packets: u64,
    pub bytes: u64,
    pub first_ms: u64,
    pub last_ms: u64,
    pub last_summary: String,
    pub responded: u64,
}

#[derive(Clone, Serialize)]
struct PeerReport {
    job_id: u64,
    ts: u64,
    peers: Vec<Peer>,
    packets: u64,
    bytes: u64,
    responses: u64,
}

/// Bind a UDP socket, optionally with SO_REUSEADDR so the port can be shared.
fn bind_udp(addr: SocketAddr, reuse: bool) -> std::io::Result<UdpSocket> {
    use socket2::{Domain, Protocol, Socket, Type};

    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };
    let sock = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;
    if reuse {
        sock.set_reuse_address(true)?;
    }
    sock.bind(&addr.into())?;
    sock.set_nonblocking(true)?;
    UdpSocket::from_std(std::net::UdpSocket::from(sock))
}

pub async fn start_discovery(
    app: AppHandle,
    jobs: JobRegistry,
    cfg: DiscoveryConfig,
) -> Result<JobInfo, String> {
    let bind_addr: SocketAddr = cfg
        .bind
        .parse()
        .map_err(|e| format!("invalid bind '{}': {e}", cfg.bind))?;
    let sock = bind_udp(bind_addr, cfg.reuse).map_err(|e| {
        format!(
            "bind {} failed: {e}{}",
            cfg.bind,
            if cfg.reuse {
                ""
            } else {
                " — enable address reuse to share the port"
            }
        )
    })?;

    let iface: Ipv4Addr = match &cfg.interface {
        Some(s) if !s.trim().is_empty() => s
            .trim()
            .parse()
            .map_err(|e| format!("invalid interface '{s}': {e}"))?,
        _ => Ipv4Addr::UNSPECIFIED,
    };
    let mut joined = Vec::new();
    for g in &cfg.groups {
        let g = g.trim();
        if g.is_empty() {
            continue;
        }
        let group: Ipv4Addr = g
            .parse()
            .map_err(|e| format!("invalid multicast group '{g}': {e}"))?;
        if !group.is_multicast() {
            return Err(format!("'{g}' is not a multicast group"));
        }
        sock.join_multicast_v4(group, iface)
            .map_err(|e| format!("join {g} failed: {e}"))?;
        joined.push(group.to_string());
    }

    let local = sock
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| cfg.bind.clone());
    let response = match (&cfg.respond, &cfg.response) {
        (true, Some(p)) => Some(p.encode()?),
        (true, None) => return Err("auto-reply is on but no response payload is set".into()),
        _ => None,
    };
    let response_summary = cfg
        .response
        .as_ref()
        .map(|p| p.summary())
        .unwrap_or_default();

    let id = jobs.next_id();
    let info = JobInfo {
        id,
        kind: "discovery".into(),
        label: if joined.is_empty() {
            format!("Discovery {local}")
        } else {
            format!("Discovery {local} + {}", joined.join(","))
        },
        started_ms: now_ms(),
    };

    let sock = Arc::new(sock);
    let peers: Arc<Mutex<HashMap<SocketAddr, Peer>>> = Arc::new(Mutex::new(HashMap::new()));
    let packets = Arc::new(AtomicU64::new(0));
    let bytes_total = Arc::new(AtomicU64::new(0));
    let responses = Arc::new(AtomicU64::new(0));
    let app_cl = app.clone();
    let jobs_cl = jobs.clone();

    let handle = tauri::async_runtime::spawn(async move {
        let reporter = {
            let (app_r, peers_r, p, b, r) = (
                app_cl.clone(),
                peers.clone(),
                packets.clone(),
                bytes_total.clone(),
                responses.clone(),
            );
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(400)).await;
                    let mut list: Vec<Peer> =
                        peers_r.lock().unwrap().values().cloned().collect();
                    list.sort_by(|a, b| b.last_ms.cmp(&a.last_ms));
                    let _ = app_r.emit(
                        "broadcast://peers",
                        PeerReport {
                            job_id: id,
                            ts: now_ms(),
                            peers: list,
                            packets: p.load(Ordering::Relaxed),
                            bytes: b.load(Ordering::Relaxed),
                            responses: r.load(Ordering::Relaxed),
                        },
                    );
                }
            })
        };

        let gate = Gate::new(40);
        let mut buf = vec![0u8; 65_536];
        // The receive loop only ever exits through the error arm below; a stop
        // from the Jobs panel aborts the task instead.
        let error;
        loop {
            match sock.recv_from(&mut buf).await {
                Ok((n, from)) => {
                    let data = &buf[..n];
                    let (proto, summary, detail) = describe_payload(data);
                    packets.fetch_add(1, Ordering::Relaxed);
                    bytes_total.fetch_add(n as u64, Ordering::Relaxed);

                    // Never answer an echo of our own reply: two Signal Labs
                    // pointed at each other would otherwise ping-pong forever.
                    let is_own_reply = response.as_deref() == Some(data);
                    let should_reply = response.is_some()
                        && !is_own_reply
                        && match &cfg.match_contains {
                            Some(needle) if !needle.trim().is_empty() => {
                                summary.contains(needle.trim())
                                    || detail
                                        .as_deref()
                                        .map(|d| d.contains(needle.trim()))
                                        .unwrap_or(false)
                            }
                            _ => true,
                        };

                    {
                        let mut map = peers.lock().unwrap();
                        if map.len() >= MAX_PEERS && !map.contains_key(&from) {
                            // Table full — keep counting traffic, stop adding rows.
                        } else {
                            let now = now_ms();
                            let entry = map.entry(from).or_insert_with(|| Peer {
                                addr: from.to_string(),
                                proto: proto.to_string(),
                                packets: 0,
                                bytes: 0,
                                first_ms: now,
                                last_ms: now,
                                last_summary: String::new(),
                                responded: 0,
                            });
                            entry.packets += 1;
                            entry.bytes += n as u64;
                            entry.last_ms = now;
                            entry.proto = proto.to_string();
                            entry.last_summary = summary.clone();
                            if should_reply {
                                entry.responded += 1;
                            }
                        }
                    }

                    if inspect::armed(&app_cl) && gate.allow() {
                        let mut frame = Frame::rx(proto, "discovery")
                            .job(id)
                            .local(&local)
                            .remote(from)
                            .payload(data)
                            .summary(summary.clone());
                        if let Some(d) = &detail {
                            frame = frame.detail(d.clone());
                        }
                        inspect::publish(&app_cl, frame);
                    }

                    if should_reply {
                        if let Some(reply) = response.clone() {
                            let (sock_r, app_r, resp_r, local_r, delay, rsum) = (
                                sock.clone(),
                                app_cl.clone(),
                                responses.clone(),
                                local.clone(),
                                cfg.respond_delay_ms,
                                response_summary.clone(),
                            );
                            tokio::spawn(async move {
                                if delay > 0 {
                                    tokio::time::sleep(Duration::from_millis(delay)).await;
                                }
                                if sock_r.send_to(&reply, from).await.is_ok() {
                                    resp_r.fetch_add(1, Ordering::Relaxed);
                                    if inspect::armed(&app_r) {
                                        inspect::publish(
                                            &app_r,
                                            Frame::tx("udp", "discovery")
                                                .job(id)
                                                .local(&local_r)
                                                .remote(from)
                                                .payload(&reply)
                                                .summary(rsum)
                                                .verdict("auto-reply"),
                                        );
                                    }
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    error = Some(e.to_string());
                    break;
                }
            }
        }

        reporter.abort();
        let _ = app_cl.emit(
            "job://ended",
            serde_json::json!({ "job_id": id, "kind": "discovery", "error": error }),
        );
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_in_several_notations() {
        assert_eq!(parse_hex("de ad be ef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(parse_hex("DEADBEEF").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(parse_hex("0xDE,0xAD").unwrap(), vec![0xde, 0xad]);
        assert!(parse_hex("abc").is_err());
        assert!(parse_hex("").is_err());
    }

    #[test]
    fn sweep_skips_network_and_broadcast() {
        let hosts = sweep_hosts("192.168.1.0/29", 9000).unwrap();
        assert_eq!(hosts.len(), 6);
        assert_eq!(hosts[0].to_string(), "192.168.1.1:9000");
        assert_eq!(hosts[5].to_string(), "192.168.1.6:9000");
    }

    #[test]
    fn sweep_honours_the_host_cap() {
        assert!(sweep_hosts("10.0.0.0/8", 9000).is_err());
        assert!(sweep_hosts("10.0.0.0/24", 9000).is_ok());
    }

    #[test]
    fn sweep_normalizes_a_non_network_base() {
        let hosts = sweep_hosts("192.168.1.77/30", 9000).unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].to_string(), "192.168.1.77:9000");
    }

    #[test]
    fn broadcast_addresses_are_recognized() {
        assert!(is_broadcastish("255.255.255.255".parse().unwrap()));
        assert!(is_broadcastish("192.168.1.255".parse().unwrap()));
        assert!(!is_broadcastish("192.168.1.10".parse().unwrap()));
    }

    #[test]
    fn describes_osc_and_opaque_payloads() {
        let osc = encode_message("/ping", &[OscArg::Int(1)]);
        let (proto, summary, detail) = describe_payload(&osc);
        assert_eq!(proto, "osc");
        assert!(summary.starts_with("/ping"));
        assert!(detail.is_some());

        let (proto, summary, _) = describe_payload(b"HELLO-PROBE");
        assert_eq!(proto, "udp");
        assert_eq!(summary, "HELLO-PROBE");
    }

    /// A config with the knobs a test doesn't care about set to their defaults.
    fn cfg_for(mode: TargetMode, target: &str, payload: Payload) -> EmitConfig {
        EmitConfig {
            mode,
            target: target.to_string(),
            port: 0,
            payload,
            bind: None,
            ttl: 1,
            multicast_loop: true,
            rate: 0.0,
            count: 0,
            duration_s: 0.0,
        }
    }

    #[tokio::test]
    async fn emits_to_a_resolved_target_and_the_listener_decodes_it() {
        let rx = bind_udp("127.0.0.1:0".parse().unwrap(), true).unwrap();
        let port = rx.local_addr().unwrap().port();

        let cfg = cfg_for(
            TargetMode::List,
            &format!("127.0.0.1:{port}"),
            Payload::Osc {
                address: "/hello/discover".into(),
                args: vec![OscArg::Str("who".into())],
            },
        );
        let targets = resolve_targets(cfg.mode, &cfg.target, cfg.port).await.unwrap();
        assert_eq!(targets.len(), 1);

        let sock = emit_socket(&cfg, &targets).await.unwrap();
        let bytes = cfg.payload.encode().unwrap();
        sock.send_to(&bytes, targets[0]).await.unwrap();

        let mut buf = vec![0u8; 2048];
        let (n, _) = tokio::time::timeout(Duration::from_secs(2), rx.recv_from(&mut buf))
            .await
            .expect("no packet arrived within 2s")
            .unwrap();

        let (proto, summary, _) = describe_payload(&buf[..n]);
        assert_eq!(proto, "osc");
        assert_eq!(summary, "/hello/discover \"who\"");
    }

    #[tokio::test]
    async fn broadcast_mode_turns_on_so_broadcast() {
        let cfg = cfg_for(
            TargetMode::Broadcast,
            "255.255.255.255:9999",
            Payload::Text { text: "probe".into() },
        );
        let targets = resolve_targets(cfg.mode, &cfg.target, cfg.port).await.unwrap();
        let sock = emit_socket(&cfg, &targets).await.unwrap();
        assert!(sock.broadcast().unwrap());
    }

    #[tokio::test]
    async fn modes_reject_addresses_of_the_wrong_shape() {
        // A plain host is not a broadcast address…
        assert!(resolve_targets(TargetMode::Broadcast, "192.168.1.10:9000", 0)
            .await
            .is_err());
        // …and 239.x is a group, not a unicast host, so multicast accepts it.
        assert!(resolve_targets(TargetMode::Multicast, "239.1.1.1:9000", 0)
            .await
            .is_ok());
        assert!(resolve_targets(TargetMode::Multicast, "192.168.1.10:9000", 0)
            .await
            .is_err());
        assert!(resolve_targets(TargetMode::List, "", 0).await.is_err());
    }

    /// SO_REUSEADDR semantics differ per platform: on Windows it lets the
    /// discovery listener share a port with the real service, which is the whole
    /// point of the `reuse` flag. On Linux a unicast bind still refuses.
    #[cfg(windows)]
    #[tokio::test]
    async fn reuse_address_lets_two_sockets_share_a_port() {
        let a = bind_udp("0.0.0.0:0".parse().unwrap(), true).unwrap();
        let port = a.local_addr().unwrap().port();
        let b = bind_udp(format!("0.0.0.0:{port}").parse().unwrap(), true);
        assert!(b.is_ok(), "second bind failed: {:?}", b.err());

        let c = bind_udp(format!("0.0.0.0:{port}").parse().unwrap(), false);
        assert!(c.is_err(), "bind without reuse should have been refused");
    }

    #[test]
    fn payload_encodes_each_kind() {
        let osc = Payload::Osc {
            address: "/probe".into(),
            args: vec![OscArg::Str("who".into())],
        };
        assert!(!osc.encode().unwrap().is_empty());
        assert_eq!(osc.summary(), "/probe \"who\"");

        let bad = Payload::Osc {
            address: "probe".into(),
            args: vec![],
        };
        assert!(bad.encode().is_err());

        let text = Payload::Text {
            text: "HELLO?".into(),
        };
        assert_eq!(text.encode().unwrap(), b"HELLO?".to_vec());
    }
}
