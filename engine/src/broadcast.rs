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
use crate::host::Host;
use tokio::net::UdpSocket;

use super::error::{EngineError, EngineResult};
use super::inspect::{self, describe_payload, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::net;
use super::osc_codec::{encode_message, OscArg};
use super::transport::{self, Cause};

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
    pub fn encode(&self) -> EngineResult<Vec<u8>> {
        match self {
            Payload::Osc { address, args } => {
                if !address.starts_with('/') {
                    return Err(EngineError::new("node.osc_address").with("value", address));
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
fn parse_hex(s: &str) -> EngineResult<Vec<u8>> {
    let cleaned: String = s
        .replace("0x", "")
        .replace("0X", "")
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();
    if cleaned.is_empty() {
        return Err(EngineError::new("hex.empty"));
    }
    if !cleaned.len().is_multiple_of(2) {
        return Err(EngineError::new("hex.invalid").with("value", s.trim()));
    }
    // Only ASCII hex digits are left, so every pair parses.
    Ok((0..cleaned.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16).ok())
        .collect())
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

/// `192.168.1.0/24` → every usable host address on `port`.
fn sweep_hosts(cidr: &str, port: u16) -> EngineResult<Vec<SocketAddr>> {
    if port == 0 {
        return Err(EngineError::new("broadcast.sweep_port"));
    }
    let cidr = cidr.trim();
    let invalid = || EngineError::new("broadcast.cidr_invalid").with("value", cidr);
    let (base, prefix) = cidr.split_once('/').ok_or_else(invalid)?;
    let base: Ipv4Addr = base.trim().parse().map_err(|_| invalid())?;
    let prefix: u32 = match prefix.trim().parse() {
        Ok(prefix) if prefix <= 32 => prefix,
        _ => return Err(EngineError::new("broadcast.prefix_invalid").with("value", prefix.trim())),
    };
    let host_bits = 32 - prefix;
    let count = 1u64 << host_bits;
    if count > MAX_SWEEP_HOSTS as u64 + 2 {
        return Err(EngineError::new("broadcast.sweep_too_large")
            .with("prefix", prefix)
            .with("count", count)
            .with("max", MAX_SWEEP_HOSTS)
            .with("limit", 32 - MAX_SWEEP_HOSTS.ilog2()));
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
) -> EngineResult<Vec<SocketAddr>> {
    let target = target.trim();
    let required = || EngineError::new("broadcast.target_required");
    if target.is_empty() {
        return Err(required());
    }
    // `IP:port`, or `host:port` through DNS so "server.local:9000" works.
    match mode {
        TargetMode::List => {
            let mut out = Vec::new();
            for part in target.split([',', ';', '\n']) {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                out.push(transport::resolve(part).await?);
            }
            if out.is_empty() {
                return Err(required());
            }
            Ok(out)
        }
        TargetMode::Broadcast => {
            let addr = transport::resolve(target).await?;
            match addr.ip() {
                IpAddr::V4(ip) if is_broadcastish(ip) => Ok(vec![addr]),
                IpAddr::V4(_) => Err(EngineError::new("broadcast.not_broadcast").with("target", target)),
                IpAddr::V6(_) => Err(EngineError::new("broadcast.ipv6")),
            }
        }
        TargetMode::Multicast => {
            let addr = transport::resolve(target).await?;
            if !addr.ip().is_multicast() {
                return Err(EngineError::new("broadcast.not_multicast").with("target", target));
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
    /// Why the first failed packet failed, for callers that explain it.
    #[serde(skip)]
    pub first_error: Option<(Cause, String)>,
    /// The same, for the interface: the first failure as a code about its target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EngineError>,
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
async fn emit_socket(cfg: &EmitConfig, targets: &[SocketAddr]) -> EngineResult<UdpSocket> {
    let bind = cfg.bind.as_deref().map(str::trim).filter(|bind| !bind.is_empty()).unwrap_or("0.0.0.0:0");
    let address = super::osc::parse_bind(bind)?;
    let sock = UdpSocket::bind(address)
        .await
        .map_err(|e| net::bind_error(bind, e))?;
    let refused = |option: &str, e: std::io::Error| EngineError::new("socket.option_failed").with("option", option).because(e);

    let wants_broadcast = cfg.mode == TargetMode::Broadcast
        || targets.iter().any(|t| match t.ip() {
            IpAddr::V4(ip) => is_broadcastish(ip),
            IpAddr::V6(_) => false,
        });
    if wants_broadcast {
        sock.set_broadcast(true).map_err(|e| refused("SO_BROADCAST", e))?;
    }

    let ttl = cfg.ttl.clamp(1, 255);
    if cfg.mode == TargetMode::Multicast {
        sock.set_multicast_ttl_v4(ttl).map_err(|e| refused("IP_MULTICAST_TTL", e))?;
        sock.set_multicast_loop_v4(cfg.multicast_loop).map_err(|e| refused("IP_MULTICAST_LOOP", e))?;
    } else {
        let _ = sock.set_ttl(ttl);
    }
    Ok(sock)
}

/// Send one packet per target, once. Returns what actually went out.
pub async fn send_once(host: Host, cfg: EmitConfig) -> EngineResult<EmitResult> {
    let targets = resolve_targets(cfg.mode, &cfg.target, cfg.port).await?;
    let bytes = cfg.payload.encode()?;
    let sock = emit_socket(&cfg, &targets).await?;
    let local = sock.local_addr().map(|a| a.to_string()).unwrap_or_default();
    let summary = cfg.payload.summary();
    let capture = inspect::armed(&host);

    let mut packets = 0u64;
    let mut sent_bytes = 0u64;
    let mut errors = 0u64;
    let mut first_error = None;
    let mut error = None;
    for t in &targets {
        match sock.send_to(&bytes, t).await {
            Ok(n) => {
                packets += 1;
                sent_bytes += n as u64;
                if capture {
                    inspect::publish(
                        &host,
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
                if first_error.is_none() {
                    first_error = Some((transport::of_io(&e), format!("{t}: {e}")));
                    error = Some(transport::of_io(&e).error(&t.to_string()).because(&e));
                }
                if capture {
                    inspect::publish(
                        &host,
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
        first_error,
        error,
    })
}

fn proto_of(p: &Payload) -> &'static str {
    match p {
        Payload::Osc { .. } => "osc",
        _ => "udp",
    }
}

/// The mode as the interface writes it, for `job.beacon`'s parameters.
fn mode_name(m: TargetMode) -> &'static str {
    match m {
        TargetMode::List => "list",
        TargetMode::Broadcast => "broadcast",
        TargetMode::Multicast => "multicast",
        TargetMode::Sweep => "sweep",
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
    host: Host,
    jobs: JobRegistry,
    cfg: EmitConfig,
) -> EngineResult<JobInfo> {
    let targets = resolve_targets(cfg.mode, &cfg.target, cfg.port).await?;
    let payload = cfg.payload.encode()?;
    let rate = cfg.rate;
    if rate <= 0.0 {
        return Err(EngineError::new("broadcast.rate_invalid"));
    }
    let aggregate = rate * targets.len() as f64;
    if aggregate > MAX_AGGREGATE_PPS {
        return Err(EngineError::new("broadcast.rate_limit")
            .with("pps", format!("{aggregate:.0}"))
            .with("targets", targets.len())
            .with("max", MAX_AGGREGATE_PPS));
    }

    let sock = emit_socket(&cfg, &targets).await?;
    let local = sock.local_addr().map(|a| a.to_string()).unwrap_or_default();
    let summary = cfg.payload.summary();
    let proto = proto_of(&cfg.payload);
    let verdict = mode_label(cfg.mode);

    let id = jobs.next_id();
    let info = JobInfo::new(id, "beacon", format!("Beacon {} → {} ×{} @{}/s", verdict, cfg.target, targets.len(), rate))
        .with("mode", mode_name(cfg.mode))
        .with("target", &cfg.target)
        .with("targets", targets.len())
        .with("rate", rate);

    let host_cl = host.clone();
    let jobs_cl = jobs.clone();
    let handle = tokio::spawn(async move {
        let packets = Arc::new(AtomicU64::new(0));
        let bytes = Arc::new(AtomicU64::new(0));
        let errors = Arc::new(AtomicU64::new(0));
        let rounds = Arc::new(AtomicU64::new(0));
        let gate = Gate::new(50);
        // Stopping the beacon must stop its reporter; the guard aborts it when
        // this task's future is dropped on cancel.
        let mut guard = TaskGuard::new();

        let reporter = {
            let (host_r, p, b, e, r) = (
                host_cl.clone(),
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
                    host_r.emit(
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
        let mut error: Option<EngineError> = None;

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
            let capture = inspect::armed(&host_cl) && gate.allow();
            for t in &targets {
                match sock.send_to(&payload, t).await {
                    Ok(n) => {
                        packets.fetch_add(1, Ordering::Relaxed);
                        bytes.fetch_add(n as u64, Ordering::Relaxed);
                        if capture {
                            inspect::publish(
                                &host_cl,
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
                            error = Some(transport::of_io(&e).error(&t.to_string()).because(e));
                        }
                    }
                }
            }
            if error.is_some() {
                break;
            }
        }

        guard.watch(reporter.abort_handle());
        drop(guard);
        // Same reason as the storm reporter: a beacon that runs out its own
        // count or duration has to leave the real totals on screen, not the
        // last 250 ms tick.
        host_cl.emit(
            "broadcast://emit-stat",
            EmitStat {
                job_id: id,
                ts: now_ms(),
                rounds: rounds.load(Ordering::Relaxed),
                packets: packets.load(Ordering::Relaxed),
                bytes: bytes.load(Ordering::Relaxed),
                errors: errors.load(Ordering::Relaxed),
                pps: 0.0,
            },
        );
        host_cl.emit(
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
    host: Host,
    jobs: JobRegistry,
    cfg: DiscoveryConfig,
) -> EngineResult<JobInfo> {
    let bind_addr = super::osc::parse_bind(&cfg.bind)?;
    let sock = bind_udp(bind_addr, cfg.reuse).map_err(|e| {
        // Without SO_REUSEADDR a port the real service holds cannot be shared.
        if !cfg.reuse && transport::of_io(&e) == Cause::AddressInUse {
            EngineError::new("broadcast.port_shared").with("target", &cfg.bind).because(e)
        } else {
            net::bind_error(&cfg.bind, e)
        }
    })?;

    let iface: Ipv4Addr = match &cfg.interface {
        Some(s) if !s.trim().is_empty() => s
            .trim()
            .parse()
            .map_err(|_| EngineError::new("broadcast.interface_invalid").with("value", s.trim()))?,
        _ => Ipv4Addr::UNSPECIFIED,
    };
    let mut joined = Vec::new();
    for g in &cfg.groups {
        let g = g.trim();
        if g.is_empty() {
            continue;
        }
        let group = match g.parse::<Ipv4Addr>() {
            Ok(group) if group.is_multicast() => group,
            _ => return Err(EngineError::new("broadcast.not_multicast").with("target", g)),
        };
        sock.join_multicast_v4(group, iface)
            .map_err(|e| EngineError::new("broadcast.join_failed").with("group", g).because(e))?;
        joined.push(group.to_string());
    }

    let local = sock
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| cfg.bind.clone());
    let response = match (&cfg.respond, &cfg.response) {
        (true, Some(p)) => Some(p.encode()?),
        (true, None) => return Err(EngineError::new("broadcast.reply_missing")),
        _ => None,
    };
    let response_summary = cfg
        .response
        .as_ref()
        .map(|p| p.summary())
        .unwrap_or_default();

    let id = jobs.next_id();
    let label = if joined.is_empty() {
        format!("Discovery {local}")
    } else {
        format!("Discovery {local} + {}", joined.join(","))
    };
    let info = JobInfo::new(id, "discovery", label)
        .with("bind", &local)
        .with("groups", joined.join(", "))
        .with("joined", joined.len());

    let sock = Arc::new(sock);
    let peers: Arc<Mutex<HashMap<SocketAddr, Peer>>> = Arc::new(Mutex::new(HashMap::new()));
    let packets = Arc::new(AtomicU64::new(0));
    let bytes_total = Arc::new(AtomicU64::new(0));
    let responses = Arc::new(AtomicU64::new(0));
    let host_cl = host.clone();
    let jobs_cl = jobs.clone();

    let handle = tokio::spawn(async move {
        // Stopping discovery must stop its reporter; the guard aborts it when
        // this task's future is dropped on cancel.
        let mut guard = TaskGuard::new();
        let reporter = {
            let (host_r, peers_r, p, b, r) = (
                host_cl.clone(),
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
                    list.sort_by_key(|peer| std::cmp::Reverse(peer.last_ms));
                    host_r.emit(
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

                    if inspect::armed(&host_cl) && gate.allow() {
                        let mut frame = Frame::rx(proto, "discovery")
                            .job(id)
                            .local(&local)
                            .remote(from)
                            .payload(data)
                            .summary(summary.clone());
                        if let Some(d) = &detail {
                            frame = frame.detail(d.clone());
                        }
                        inspect::publish(&host_cl, frame);
                    }

                    if should_reply {
                        if let Some(reply) = response.clone() {
                            let (sock_r, host_r, resp_r, local_r, delay, rsum) = (
                                sock.clone(),
                                host_cl.clone(),
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
                                    if inspect::armed(&host_r) {
                                        inspect::publish(
                                            &host_r,
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
                // An auto-reply to a prober that has gone comes back as this
                // on Windows; the listener carries on.
                Err(e) if crate::net::udp_transient(&e) => continue,
                Err(e) => {
                    error = Some(EngineError::new("wait.receive_failed").with("target", &local).because(e));
                    break;
                }
            }
        }

        guard.watch(reporter.abort_handle());
        drop(guard);
        host_cl.emit(
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
        let odd = parse_hex(" abc ").unwrap_err();
        assert_eq!((odd.code.as_str(), odd.params["value"].as_str()), ("hex.invalid", "abc"));
        assert!(parse_hex("").unwrap_err().is("hex.empty"));
        assert!(parse_hex("zz").unwrap_err().is("hex.empty"), "nothing hex is left");
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
        let wide = sweep_hosts("10.0.0.0/8", 9000).unwrap_err();
        assert_eq!(wide.code, "broadcast.sweep_too_large");
        let params: Vec<(&str, &str)> = wide.params.iter().map(|(name, value)| (name.as_str(), value.as_str())).collect();
        assert_eq!(params, [("count", "16777216"), ("limit", "22"), ("max", "1024"), ("prefix", "8")]);
        assert_eq!(sweep_hosts("10.0.0.0/22", 9000).unwrap().len(), 1022, "the limit it names is accepted");
        assert!(sweep_hosts("10.0.0.0/24", 9000).is_ok());
    }

    #[test]
    fn sweep_says_which_part_of_the_block_is_wrong() {
        for (cidr, port, code, value) in [
            ("10.0.0.0/24", 0, "broadcast.sweep_port", None),
            ("10.0.0.0", 9000, "broadcast.cidr_invalid", Some("10.0.0.0")),
            (" 10.0.0/24 ", 9000, "broadcast.cidr_invalid", Some("10.0.0/24")),
            ("server.local/24", 9000, "broadcast.cidr_invalid", Some("server.local/24")),
            ("10.0.0.0/33", 9000, "broadcast.prefix_invalid", Some("33")),
            ("10.0.0.0/x", 9000, "broadcast.prefix_invalid", Some("x")),
        ] {
            let error = sweep_hosts(cidr, port).unwrap_err();
            assert_eq!((error.code.as_str(), error.params.get("value").map(String::as_str)), (code, value), "{cidr}");
        }
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

    /// Answering a prober that has already closed its port comes back to the
    /// listener as WSAECONNRESET on Windows; the listener must keep answering.
    #[tokio::test]
    async fn discovery_keeps_answering_after_a_prober_went_away() {
        let host = Host::new(crate::host::Recorder::new(), crate::inspect::Capture::new());
        let jobs = JobRegistry::new();
        let port = UdpSocket::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
        let cfg = DiscoveryConfig {
            bind: format!("127.0.0.1:{port}"),
            groups: vec![],
            interface: None,
            reuse: false,
            respond: true,
            response: Some(Payload::Text { text: "HERE".into() }),
            respond_delay_ms: 0,
            match_contains: None,
        };
        let job = start_discovery(host, jobs.clone(), cfg).await.unwrap();
        let listener: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

        let gone = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        gone.send_to(b"PROBE", listener).await.unwrap();
        drop(gone);
        tokio::time::sleep(Duration::from_millis(150)).await;

        let prober = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        prober.send_to(b"PROBE", listener).await.unwrap();
        let mut buf = vec![0u8; 64];
        let (n, _) = tokio::time::timeout(Duration::from_secs(2), prober.recv_from(&mut buf)).await.expect("the listener stopped answering").unwrap();
        assert_eq!(&buf[..n], b"HERE");
        assert!(jobs.list().iter().any(|info| info.id == job.id), "the listener is still running");
        jobs.stop(job.id);
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
        let refused = |mode, target: &'static str| async move { resolve_targets(mode, target, 0).await.err().map(EngineError::into_code) };
        // A plain host is not a broadcast address…
        assert_eq!(refused(TargetMode::Broadcast, "192.168.1.10:9000").await.as_deref(), Some("broadcast.not_broadcast"));
        assert_eq!(refused(TargetMode::Broadcast, "[ff02::1]:9000").await.as_deref(), Some("broadcast.ipv6"));
        // …and 239.x is a group, not a unicast host, so multicast accepts it.
        assert_eq!(refused(TargetMode::Multicast, "239.1.1.1:9000").await, None);
        assert_eq!(refused(TargetMode::Multicast, "192.168.1.10:9000").await.as_deref(), Some("broadcast.not_multicast"));
        assert_eq!(refused(TargetMode::List, "").await.as_deref(), Some("broadcast.target_required"));
        assert_eq!(refused(TargetMode::List, " , ;").await.as_deref(), Some("broadcast.target_required"));
        // A target without a port, or a name that does not resolve, says so.
        assert_eq!(refused(TargetMode::List, "127.0.0.1").await.as_deref(), Some("transport.target_invalid"));
        assert_eq!(refused(TargetMode::List, "no-such-host.invalid:9000").await.as_deref(), Some("transport.dns"));
    }

    #[tokio::test]
    async fn a_beacon_and_a_listener_refuse_settings_that_cannot_work() {
        let host = Host::new(crate::host::Recorder::new(), crate::inspect::Capture::new());
        let jobs = JobRegistry::new();
        let mut cfg = cfg_for(TargetMode::List, "127.0.0.1:9", Payload::Text { text: "x".into() });
        assert!(start_beacon(host.clone(), jobs.clone(), cfg.clone()).await.unwrap_err().is("broadcast.rate_invalid"));
        cfg.rate = 60_000.0;
        let limit = start_beacon(host.clone(), jobs.clone(), cfg.clone()).await.unwrap_err();
        assert_eq!((limit.code.as_str(), limit.params["pps"].as_str(), limit.params["targets"].as_str()), ("broadcast.rate_limit", "60000", "1"));
        cfg.rate = 1.0;
        cfg.bind = Some("0.0.0.0".into());
        assert!(start_beacon(host.clone(), jobs.clone(), cfg).await.unwrap_err().is("node.bind_invalid"));

        let listen = |bind: &str, groups: &[&str], interface: Option<&str>, respond: bool| DiscoveryConfig {
            bind: bind.into(),
            groups: groups.iter().map(|group| group.to_string()).collect(),
            interface: interface.map(String::from),
            reuse: true,
            respond,
            response: None,
            respond_delay_ms: 0,
            match_contains: None,
        };
        let refused = |cfg| {
            let (host, jobs) = (host.clone(), jobs.clone());
            async move { start_discovery(host, jobs, cfg).await.err().map(EngineError::into_code) }
        };
        assert_eq!(refused(listen("nonsense", &[], None, false)).await.as_deref(), Some("node.bind_invalid"));
        assert_eq!(refused(listen("127.0.0.1:0", &["10.0.0.1"], None, false)).await.as_deref(), Some("broadcast.not_multicast"));
        assert_eq!(refused(listen("127.0.0.1:0", &[], Some("eth0"), false)).await.as_deref(), Some("broadcast.interface_invalid"));
        assert_eq!(refused(listen("127.0.0.1:0", &[], None, true)).await.as_deref(), Some("broadcast.reply_missing"));
        assert!(jobs.list().is_empty(), "nothing was started");
    }

    #[tokio::test]
    async fn a_port_in_use_suggests_sharing_it_only_when_sharing_is_off() {
        let host = Host::new(crate::host::Recorder::new(), crate::inspect::Capture::new());
        let taken = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let bind = taken.local_addr().unwrap().to_string();
        let cfg = DiscoveryConfig {
            bind: bind.clone(),
            groups: vec![],
            interface: None,
            reuse: false,
            respond: false,
            response: None,
            respond_delay_ms: 0,
            match_contains: None,
        };
        let error = start_discovery(host, JobRegistry::new(), cfg).await.unwrap_err();
        assert_eq!((error.code.as_str(), error.params["target"].as_str()), ("broadcast.port_shared", bind.as_str()));
        assert!(error.detail.is_some(), "the system's wording is kept");
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
        let error = bad.encode().unwrap_err();
        assert_eq!((error.code.as_str(), error.params["value"].as_str()), ("node.osc_address", "probe"));

        let text = Payload::Text {
            text: "HELLO?".into(),
        };
        assert_eq!(text.encode().unwrap(), b"HELLO?".to_vec());
    }
}
