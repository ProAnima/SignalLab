//! Device discovery: the listening half of broadcast. Binds a port
//! (optionally joining multicast groups with `SO_REUSEADDR`, so it can share
//! the port with the real service), tracks every peer that talks, and can
//! auto-reply to simulate a device answering probes. It feeds the capture bus
//! like the emitter (`broadcast.rs`), so a beacon and the replies it triggers
//! land on the same Inspector timeline.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::net::UdpSocket;

use super::broadcast::Payload;
use super::error::{EngineError, EngineResult};
use super::inspect::{self, describe_payload, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::net;
use super::transport::{self, Cause};

const MAX_PEERS: usize = 512;

fn default_true() -> bool {
    true
}

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
pub(crate) fn bind_udp(addr: SocketAddr, reuse: bool) -> std::io::Result<UdpSocket> {
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
    async fn a_listener_refuses_settings_that_cannot_work() {
        let host = Host::new(crate::host::Recorder::new(), crate::inspect::Capture::new());
        let jobs = JobRegistry::new();
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
}
