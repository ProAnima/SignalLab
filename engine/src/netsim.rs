//! Network impairment: a relay that sits between a client and a target and
//! impairs traffic in both directions. Over UDP — latency, jitter, loss,
//! bursts of loss, duplication, corruption, reordering, a bandwidth limit, or
//! nothing at all getting through; over TCP (`netsim_tcp`) — latency, jitter
//! and a bandwidth limit on each stream, connections reset or left half-open,
//! or nothing flowing. Point the client at the relay's listen port; it
//! forwards to the real target and impairs traffic according to its profile.
//!
//! The profile can change while the relay runs (`Relay::set`), without
//! rebinding: the Impairment screen applies an edit at once, and an
//! experiment's *Change impairment* step switches phases mid-run. Every
//! decision comes from a seeded stream per direction, so the same seed and
//! the same traffic give the same drops; what each phase did is counted
//! (`Phase`). A run's relays are opened in `netsim_run`.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::net::UdpSocket;
use tokio::sync::watch;
use tokio::task::JoinSet;

use super::error::{EngineError, EngineResult, Field};
use super::inspect::{self, describe_payload, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::net;
use super::osc::{parse_bind, parse_target};
use super::template::Rng;
use super::transport;

/// The longest latency or jitter a profile adds, milliseconds.
pub const MAX_DELAY_MS: f64 = 60_000.0;
/// The bandwidth limit's range, kilobits per second (0 is none).
pub const MIN_RATE_KBPS: f64 = 8.0;
pub const MAX_RATE_KBPS: f64 = 10_000_000.0;
/// The longest mean burst of losses, in packets.
pub const MAX_BURST_LENGTH: f64 = 1000.0;
/// A packet that would wait longer than this for a bandwidth-limited link is dropped.
const MAX_QUEUE: Duration = Duration::from_secs(1);
/// Packets on their way (delayed) at once; more are dropped as throttled.
const MAX_IN_FLIGHT: usize = 10_000;
/// A reordered packet is held back this long at least (or by the latency, if longer).
const REORDER_HOLD: Duration = Duration::from_millis(20);
/// Phases kept per relay; older ones are counted, not kept.
const MAX_PHASES: usize = 1000;
/// Mixed into the seed, so the relay's draws never repeat a template's.
const IMPAIR_STREAM: u64 = 0x696d_7061_6972_0001;

/// What a relay carries: datagrams, or TCP streams.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayProtocol {
    #[default]
    Udp,
    Tcp,
}

impl RelayProtocol {
    pub fn is_udp(&self) -> bool {
        *self == RelayProtocol::Udp
    }
}

fn is_zero_f64(value: &f64) -> bool {
    *value == 0.0
}

/// What a relay does to every packet, in both directions. A TCP relay reads
/// latency, jitter, the bandwidth limit, offline, `reset` and `stall`; loss,
/// bursts, duplication, corruption and reordering are a datagram's (a TCP
/// stream retransmits and orders itself), and `reset` and `stall` a stream's.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ImpairProfile {
    /// A name for the timeline and the report: a preset's key (`4g`), or a person's own.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
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
    /// Probability 0..1 a single bit is flipped (corruption).
    #[serde(default)]
    pub corrupt: f64,
    /// Probability 0..1 a packet is held back, so later ones overtake it.
    #[serde(default)]
    pub reorder: f64,
    /// A bandwidth limit, kilobits per second; 0 is none. Packets queue for
    /// the link and are dropped (throttled) past a second of queue.
    #[serde(default)]
    pub rate_kbps: f64,
    /// Bursts of loss (Gilbert–Elliott): the probability 0..1 a packet starts
    /// one, and how many packets one lasts on average.
    #[serde(default)]
    pub burst_start: f64,
    #[serde(default)]
    pub burst_length: f64,
    /// Nothing gets through.
    #[serde(default)]
    pub offline: bool,
    /// TCP: the probability 0..1 a chunk of a stream resets its connection
    /// instead of going through — both sides get a reset.
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub reset: f64,
    /// TCP: the probability 0..1 a chunk leaves its connection half-open —
    /// nothing more goes through either way, and neither side is told.
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub stall: f64,
}

fn probability(value: f64, field: &'static str) -> EngineResult<()> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(EngineError::new("node.range").with("min", 0).with("max", 1).in_field(Field::new(field)))
    }
}

fn within(value: f64, min: f64, max: f64, field: &'static str) -> EngineResult<()> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(EngineError::new("node.range").with("min", min).with("max", max).in_field(Field::new(field)))
    }
}

impl ImpairProfile {
    /// Every value in its range, named by its field.
    pub fn check(&self) -> EngineResult<()> {
        within(self.latency_ms, 0.0, MAX_DELAY_MS, "latency_ms")?;
        within(self.jitter_ms, 0.0, MAX_DELAY_MS, "jitter_ms")?;
        for (value, field) in [(self.loss, "loss"), (self.duplicate, "duplicate"), (self.corrupt, "corrupt"), (self.reorder, "reorder"), (self.burst_start, "burst_start"), (self.reset, "reset"), (self.stall, "stall")] {
            probability(value, field)?;
        }
        if self.rate_kbps != 0.0 {
            within(self.rate_kbps, MIN_RATE_KBPS, MAX_RATE_KBPS, "rate_kbps")?;
        }
        if self.burst_start > 0.0 {
            within(self.burst_length, 1.0, MAX_BURST_LENGTH, "burst_length")?;
        }
        if self.name.chars().count() > 60 {
            return Err(EngineError::new("node.too_long").with("max", 60).in_field(Field::new("profile_name")));
        }
        Ok(())
    }

    /// How the timeline and the report name it: its name, or what it does in
    /// protocol notation (`60 ms ±25 · loss 2% · 20000 kbps`).
    pub fn label(&self) -> String {
        if !self.name.trim().is_empty() {
            return self.name.trim().to_string();
        }
        if self.offline {
            return "offline".into();
        }
        let percent = |value: f64| format!("{}%", (value * 1000.0).round() / 10.0);
        let mut parts = Vec::new();
        if self.latency_ms > 0.0 || self.jitter_ms > 0.0 {
            parts.push(if self.jitter_ms > 0.0 { format!("{} ms ±{}", self.latency_ms, self.jitter_ms) } else { format!("{} ms", self.latency_ms) });
        }
        for (value, what) in [(self.loss, "loss"), (self.duplicate, "dup"), (self.corrupt, "corrupt"), (self.reorder, "reorder"), (self.reset, "reset"), (self.stall, "stall")] {
            if value > 0.0 {
                parts.push(format!("{what} {}", percent(value)));
            }
        }
        if self.burst_start > 0.0 {
            parts.push(format!("bursts {} × {}", percent(self.burst_start), self.burst_length));
        }
        if self.rate_kbps > 0.0 {
            parts.push(format!("{} kbps", self.rate_kbps));
        }
        if parts.is_empty() { "clean".into() } else { parts.join(" · ") }
    }

    /// What a relay of `protocol` reads of it: the other protocol's values are
    /// left out, so its label and its counts say only what it does.
    pub fn for_protocol(&self, protocol: RelayProtocol) -> ImpairProfile {
        match protocol {
            RelayProtocol::Udp => ImpairProfile { reset: 0.0, stall: 0.0, ..self.clone() },
            RelayProtocol::Tcp => ImpairProfile { loss: 0.0, duplicate: 0.0, corrupt: 0.0, reorder: 0.0, burst_start: 0.0, burst_length: 0.0, ..self.clone() },
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct ProxyConfig {
    /// Local address the relay listens on, e.g. "0.0.0.0:9010".
    pub listen: String,
    /// Real destination, e.g. "127.0.0.1:9000".
    pub target: String,
    pub profile: ImpairProfile,
    /// The draws' seed; `None`: a fresh one.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Datagrams (the default) or TCP streams.
    #[serde(default)]
    pub protocol: RelayProtocol,
}

/// What a relay did, in all or in one phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ImpairCounts {
    pub received: u64,
    pub forwarded: u64,
    pub dropped: u64,
    /// Dropped by the bandwidth limit (or too many packets on their way).
    pub throttled: u64,
    pub duplicated: u64,
    pub corrupted: u64,
    pub reordered: u64,
    pub bytes: u64,
    /// TCP: connections taken, connections reset, connections left half-open.
    #[serde(skip_serializing_if = "is_zero")]
    pub connections: u64,
    #[serde(skip_serializing_if = "is_zero")]
    pub reset: u64,
    #[serde(skip_serializing_if = "is_zero")]
    pub stalled: u64,
}

#[derive(Default)]
pub(crate) struct Stats {
    pub(crate) received: AtomicU64,
    pub(crate) forwarded: AtomicU64,
    pub(crate) dropped: AtomicU64,
    pub(crate) throttled: AtomicU64,
    duplicated: AtomicU64,
    corrupted: AtomicU64,
    reordered: AtomicU64,
    pub(crate) bytes: AtomicU64,
    pub(crate) connections: AtomicU64,
    pub(crate) reset: AtomicU64,
    pub(crate) stalled: AtomicU64,
}

impl Stats {
    fn counts(&self) -> ImpairCounts {
        let load = |counter: &AtomicU64| counter.load(Ordering::Relaxed);
        ImpairCounts {
            received: load(&self.received),
            forwarded: load(&self.forwarded),
            dropped: load(&self.dropped),
            throttled: load(&self.throttled),
            duplicated: load(&self.duplicated),
            corrupted: load(&self.corrupted),
            reordered: load(&self.reordered),
            bytes: load(&self.bytes),
            connections: load(&self.connections),
            reset: load(&self.reset),
            stalled: load(&self.stalled),
        }
    }
}

/// One stretch under one profile. A packet is counted in the phase that
/// decided it, even when its delayed copy goes out after the switch.
#[derive(Clone, Debug, Serialize)]
pub struct Phase {
    pub profile: String,
    /// From and until, milliseconds since the relay opened (`opened_ms`).
    pub from_ms: u64,
    pub to_ms: u64,
    pub counts: ImpairCounts,
}

/// The profile now, and the counters of the phase it began.
struct Current {
    profile: Arc<ImpairProfile>,
    stats: Arc<Stats>,
    since_ms: u64,
}

/// A phase that ended. Its counters stay live: a copy delayed past the
/// switch is counted in the phase that decided it, not in the next one.
struct ClosedPhase {
    profile: String,
    from_ms: u64,
    to_ms: u64,
    stats: Arc<Stats>,
}

struct PhaseLog {
    closed: Vec<ClosedPhase>,
    /// Phases no longer kept (more than `MAX_PHASES`).
    earlier: u64,
}

/// A relay's counters for a run's report: in all, and phase by phase.
#[derive(Clone, Debug, Serialize)]
pub struct ImpairmentSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    #[serde(skip_serializing_if = "RelayProtocol::is_udp")]
    pub protocol: RelayProtocol,
    pub listen: String,
    pub target: String,
    pub counts: ImpairCounts,
    pub phases: Vec<Phase>,
    #[serde(skip_serializing_if = "is_zero")]
    pub earlier_phases: u64,
    /// When the relay opened (ms since 1970): its phases' `from_ms`/`to_ms` count from then.
    pub opened_ms: u64,
    /// Why it stopped relaying before the run ended, if it did.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EngineError>,
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

/// A packet's fate, decided when it arrives.
#[derive(Debug, PartialEq)]
enum Fate {
    Dropped(&'static str),
    Throttled,
    Sent(Vec<Sending>),
}

#[derive(Debug, PartialEq)]
struct Sending {
    bytes: Vec<u8>,
    delay: Duration,
    corrupted: bool,
    reordered: bool,
}

/// One direction's state: its seed and name, how many packets it decided,
/// whether a burst of loss is on, when its bandwidth-limited link is free.
struct Leg {
    seed: u64,
    name: String,
    /// Packets decided so far. Each draws from a stream of its own, so how
    /// many draws one takes (a throttled one takes fewer) never moves the
    /// next one's: the same seed and traffic give the same fates.
    packets: u64,
    bursting: bool,
    link_free: Option<Instant>,
}

impl Leg {
    fn new(seed: u64, name: &str) -> Self {
        Leg { seed: seed ^ IMPAIR_STREAM, name: name.to_string(), packets: 0, bursting: false, link_free: None }
    }

    fn decide(&mut self, profile: &ImpairProfile, payload: &[u8], now: Instant) -> Fate {
        let mut rng = Rng::for_node(self.seed, &self.name, self.packets);
        self.packets += 1;
        if profile.offline {
            return Fate::Dropped("offline");
        }
        // Gilbert–Elliott: a packet starts a burst with `burst_start`; each packet in
        // one is lost and ends it with 1 / `burst_length`, so bursts last that long on average.
        if profile.burst_start > 0.0 {
            if !self.bursting && rng.unit() < profile.burst_start {
                self.bursting = true;
            }
            if self.bursting {
                if rng.unit() < 1.0 / profile.burst_length.max(1.0) {
                    self.bursting = false;
                }
                return Fate::Dropped("burst");
            }
        } else {
            self.bursting = false;
        }
        if profile.loss > 0.0 && rng.unit() < profile.loss {
            return Fate::Dropped("loss");
        }
        // The link serializes packets one after another at the rate; a long queue is a drop.
        let mut queued = Duration::ZERO;
        if profile.rate_kbps > 0.0 {
            let start = self.link_free.filter(|free| *free > now).unwrap_or(now);
            if start - now > MAX_QUEUE {
                return Fate::Throttled;
            }
            let departure = start + Duration::from_secs_f64(payload.len() as f64 * 8.0 / (profile.rate_kbps * 1000.0));
            self.link_free = Some(departure);
            queued = departure - now;
        } else {
            self.link_free = None;
        }
        let copies = if profile.duplicate > 0.0 && rng.unit() < profile.duplicate { 2 } else { 1 };
        let mut sent = Vec::with_capacity(copies);
        for _ in 0..copies {
            let mut bytes = payload.to_vec();
            let corrupted = profile.corrupt > 0.0 && !bytes.is_empty() && rng.unit() < profile.corrupt;
            if corrupted {
                let index = rng.below_u64(bytes.len() as u64) as usize;
                bytes[index] ^= 1 << rng.below_u64(8);
            }
            let jitter = if profile.jitter_ms > 0.0 { rng.unit() * profile.jitter_ms } else { 0.0 };
            let mut delay = queued + Duration::from_secs_f64((profile.latency_ms + jitter) / 1000.0);
            let reordered = profile.reorder > 0.0 && rng.unit() < profile.reorder;
            if reordered {
                delay += REORDER_HOLD.max(Duration::from_secs_f64(profile.latency_ms / 1000.0));
            }
            sent.push(Sending { bytes, delay, corrupted, reordered });
        }
        Fate::Sent(sent)
    }
}

/// Everything a packet's fate is reported with to the Inspector.
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
        let peer = dest.map(|d| d.to_string()).unwrap_or_else(|| self.peer.clone());
        let mut frame = Frame::new(proto, self.dir, "netsim").job(self.job_id).local(self.leg).remote(peer).payload(bytes).summary(summary).verdict(verdict);
        if let Some(d) = detail {
            frame = frame.detail(d);
        }
        inspect::publish(&self.host, frame);
    }
}

/// A relay: where it listens and forwards, the profile it impairs with now,
/// and what it has done.
pub struct Relay {
    pub listen: SocketAddr,
    pub target: SocketAddr,
    pub protocol: RelayProtocol,
    current: RwLock<Current>,
    /// Told of every profile change: a TCP stream held while offline resumes at once.
    changed: watch::Sender<u64>,
    /// In all: every phase's counters add up to these.
    pub(crate) stats: Stats,
    in_flight: AtomicUsize,
    phases: Mutex<PhaseLog>,
    opened: Instant,
    /// When it opened, wall clock: the phases' times are from then.
    opened_ms: u64,
    pub(crate) seed: u64,
    failed: Mutex<Option<EngineError>>,
}

impl Relay {
    pub(crate) fn new(listen: SocketAddr, target: SocketAddr, protocol: RelayProtocol, profile: ImpairProfile, seed: u64) -> Arc<Relay> {
        Arc::new(Relay {
            listen,
            target,
            protocol,
            current: RwLock::new(Current { profile: Arc::new(profile.for_protocol(protocol)), stats: Arc::default(), since_ms: 0 }),
            changed: watch::channel(0).0,
            stats: Stats::default(),
            in_flight: AtomicUsize::new(0),
            phases: Mutex::new(PhaseLog { closed: Vec::new(), earlier: 0 }),
            opened: Instant::now(),
            opened_ms: now_ms(),
            seed,
            failed: Mutex::new(None),
        })
    }

    pub fn profile(&self) -> Arc<ImpairProfile> {
        self.current.read().unwrap().profile.clone()
    }

    /// The profile now and the counters of its phase, read together: what a
    /// packet or a chunk is decided by, and where it is counted.
    pub(crate) fn now(&self) -> (Arc<ImpairProfile>, Arc<Stats>) {
        let current = self.current.read().unwrap();
        (current.profile.clone(), current.stats.clone())
    }

    /// Count one event in all and in the phase that decided it.
    pub(crate) fn count(&self, phase: &Stats, field: fn(&Stats) -> &AtomicU64, by: u64) {
        field(&self.stats).fetch_add(by, Ordering::Relaxed);
        field(phase).fetch_add(by, Ordering::Relaxed);
    }

    /// Returns once the profile is not offline: at once, or when a change brings it back.
    pub(crate) async fn until_online(&self) {
        let mut changes = self.changed.subscribe();
        while self.profile().offline {
            if changes.changed().await.is_err() {
                return;
            }
        }
    }

    /// Impair with `profile` from now on: the phase so far is closed.
    pub fn set(&self, profile: ImpairProfile) {
        let mut log = self.phases.lock().unwrap();
        let mut current = self.current.write().unwrap();
        let now_ms = self.opened.elapsed().as_millis() as u64;
        let closed = ClosedPhase { profile: current.profile.label(), from_ms: current.since_ms, to_ms: now_ms, stats: current.stats.clone() };
        if log.closed.len() == MAX_PHASES {
            log.closed.remove(0);
            log.earlier += 1;
        }
        log.closed.push(closed);
        *current = Current { profile: Arc::new(profile.for_protocol(self.protocol)), stats: Arc::default(), since_ms: now_ms };
        drop(current);
        self.changed.send_modify(|generation| *generation += 1);
    }

    /// It stopped relaying: why, kept for the steps that use it and the report.
    pub(crate) fn fail(&self, error: EngineError) {
        *self.failed.lock().unwrap() = Some(error);
    }

    /// Why it stopped relaying, if it has.
    pub fn failed(&self) -> Option<EngineError> {
        self.failed.lock().unwrap().clone()
    }

    pub fn counts(&self) -> ImpairCounts {
        self.stats.counts()
    }

    /// What it did so far, the current phase up to now included.
    pub fn summary(&self, node: Option<&str>) -> ImpairmentSummary {
        let log = self.phases.lock().unwrap();
        let current = self.current.read().unwrap();
        let mut phases: Vec<Phase> = log.closed.iter().map(|phase| Phase { profile: phase.profile.clone(), from_ms: phase.from_ms, to_ms: phase.to_ms, counts: phase.stats.counts() }).collect();
        phases.push(Phase { profile: current.profile.label(), from_ms: current.since_ms, to_ms: self.opened.elapsed().as_millis() as u64, counts: current.stats.counts() });
        ImpairmentSummary {
            node: node.map(str::to_string),
            protocol: self.protocol,
            listen: self.listen.to_string(),
            target: self.target.to_string(),
            counts: self.stats.counts(),
            phases,
            earlier_phases: log.earlier,
            opened_ms: self.opened_ms,
            error: self.failed(),
        }
    }

    /// One packet: decided now, sent (possibly later, more than once) through
    /// `socket` by a task of `flights` — the relay's, so stopping the relay
    /// stops what it still had on its way.
    #[allow(clippy::too_many_arguments)]
    fn take(self: &Arc<Self>, leg: &mut Leg, payload: &[u8], socket: &Arc<UdpSocket>, dest: Option<SocketAddr>, tap: &Tap, flights: &Mutex<JoinSet<()>>) {
        // The profile and the phase it counts in, read together: a packet is
        // counted where it was decided, whatever a switch does meanwhile.
        let (profile, phase) = {
            let current = self.current.read().unwrap();
            (current.profile.clone(), current.stats.clone())
        };
        let count = |field: fn(&Stats) -> &AtomicU64| {
            field(&self.stats).fetch_add(1, Ordering::Relaxed);
            field(&phase).fetch_add(1, Ordering::Relaxed);
        };
        count(|stats| &stats.received);
        // Decided once per packet: a dropped packet is exactly what a QA run wants to see.
        let capture = tap.armed();
        let copies = match leg.decide(&profile, payload, Instant::now()) {
            Fate::Dropped(why) => {
                count(|stats| &stats.dropped);
                if capture {
                    tap.record(payload, dest, format!("dropped ({why})"));
                }
                return;
            }
            Fate::Throttled => {
                count(|stats| &stats.throttled);
                if capture {
                    tap.record(payload, dest, "throttled".into());
                }
                return;
            }
            Fate::Sent(copies) => copies,
        };
        if self.in_flight.load(Ordering::Relaxed) + copies.len() > MAX_IN_FLIGHT {
            count(|stats| &stats.throttled);
            if capture {
                tap.record(payload, dest, format!("throttled ({MAX_IN_FLIGHT} on their way)"));
            }
            return;
        }
        let total = copies.len();
        if total > 1 {
            count(|stats| &stats.duplicated);
        }
        let mut flights = flights.lock().unwrap();
        // Finished sends go; the set holds only what is still on its way.
        while flights.try_join_next().is_some() {}
        for (index, copy) in copies.into_iter().enumerate() {
            if copy.corrupted {
                count(|stats| &stats.corrupted);
            }
            if copy.reordered {
                count(|stats| &stats.reordered);
            }
            self.in_flight.fetch_add(1, Ordering::Relaxed);
            // Due from the decision, not from when the task first runs: no drift under load.
            let due = tokio::time::Instant::now() + copy.delay;
            let (relay, socket, tap, phase) = (self.clone(), socket.clone(), tap.clone(), phase.clone());
            flights.spawn(async move {
                if !copy.delay.is_zero() {
                    tokio::time::sleep_until(due).await;
                }
                let sent = match dest {
                    Some(address) => socket.send_to(&copy.bytes, address).await,
                    None => socket.send(&copy.bytes).await,
                };
                relay.in_flight.fetch_sub(1, Ordering::Relaxed);
                match sent {
                    Ok(size) => {
                        for stats in [&relay.stats, &*phase] {
                            stats.forwarded.fetch_add(1, Ordering::Relaxed);
                            stats.bytes.fetch_add(size as u64, Ordering::Relaxed);
                        }
                        if capture {
                            let mut verdict = format!("forwarded +{:.0}ms", copy.delay.as_secs_f64() * 1000.0);
                            if copy.corrupted {
                                verdict.push_str(" · corrupted");
                            }
                            if copy.reordered {
                                verdict.push_str(" · reordered");
                            }
                            if total > 1 {
                                verdict.push_str(&format!(" · copy {}/{total}", index + 1));
                            }
                            tap.record(&copy.bytes, dest, verdict);
                        }
                    }
                    Err(error) => {
                        if capture {
                            tap.record(&copy.bytes, dest, format!("send failed: {error}"));
                        }
                    }
                }
            });
        }
    }

    /// Relay until a leg cannot receive any more; the error says why. Both
    /// legs are children of the future: dropping it stops the relay.
    pub(crate) async fn serve(self: Arc<Self>, host: Host, job_id: u64, name: String, downstream: Arc<UdpSocket>, upstream: Arc<UdpSocket>) -> EngineError {
        let down_local = downstream.local_addr().map(|a| a.to_string()).unwrap_or_else(|_| self.listen.to_string());
        let up_local = upstream.local_addr().map(|a| a.to_string()).unwrap_or_default();
        let client: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
        // One sampling budget shared by both legs, so a loaded relay reports a
        // representative slice rather than swamping the Inspector.
        let gate = Arc::new(Gate::new(25));
        let tap = |leg: &'static str, dir: &'static str| Tap { host: host.clone(), job_id, leg, dir, peer: self.target.to_string(), gate: gate.clone() };
        // What is on its way, delayed: owned here, so it ends with the relay —
        // no packet goes out after a stop, and no socket outlives it.
        let flights: Arc<Mutex<JoinSet<()>>> = Arc::new(Mutex::new(JoinSet::new()));

        let c2s = {
            let (relay, down, up, client, tap, receiving, flights) = (self.clone(), downstream.clone(), upstream.clone(), client.clone(), tap("client→target", "tx"), down_local.clone(), flights.clone());
            let mut leg = Leg::new(self.seed, &format!("{name}:client"));
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                loop {
                    match down.recv_from(&mut buf).await {
                        Ok((n, from)) => {
                            *client.lock().unwrap() = Some(from);
                            relay.take(&mut leg, &buf[..n], &up, None, &tap, &flights);
                        }
                        // A reply to a client that has gone: the relay carries on.
                        Err(e) if crate::net::udp_transient(&e) => continue,
                        Err(e) => return EngineError::new("wait.receive_failed").with("target", &receiving).because(e),
                    }
                }
            })
        };
        let s2c = {
            let (relay, down, up, client, tap, receiving, flights) = (self.clone(), downstream.clone(), upstream.clone(), client.clone(), tap("target→client", "rx"), up_local.clone(), flights.clone());
            let mut leg = Leg::new(self.seed, &format!("{name}:target"));
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                loop {
                    match up.recv(&mut buf).await {
                        Ok(n) => {
                            let dest = *client.lock().unwrap();
                            if let Some(address) = dest {
                                relay.take(&mut leg, &buf[..n], &down, Some(address), &tap, &flights);
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
        let mut guard = TaskGuard::new();
        guard.watch(c2s.abort_handle());
        guard.watch(s2c.abort_handle());
        let (mut c2s, mut s2c) = (c2s, s2c);
        // A leg that panicked ends the relay the same way, its panic as detail.
        tokio::select! {
            ended = &mut c2s => ended.unwrap_or_else(|e| EngineError::new("wait.receive_failed").with("target", &down_local).because(e)),
            ended = &mut s2c => ended.unwrap_or_else(|e| EngineError::new("wait.receive_failed").with("target", &up_local).because(e)),
        }
    }
}

/// Bind the relay's two sockets: the one clients send to, and one connected to the target.
pub(crate) async fn open(listen: SocketAddr, listen_text: &str, target: SocketAddr, target_text: &str) -> EngineResult<(Arc<UdpSocket>, Arc<UdpSocket>)> {
    let downstream = UdpSocket::bind(listen).await.map_err(|e| net::bind_error(listen_text, e))?;
    let source = if target.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
    let upstream = UdpSocket::bind(source).await.map_err(|e| net::bind_error(source, e))?;
    upstream.connect(target).await.map_err(|e| transport::of_io(&e).error(target_text).because(e))?;
    Ok((Arc::new(downstream), Arc::new(upstream)))
}

/// The relays running as jobs of their own, for whoever changes their profile.
#[derive(Clone, Default)]
pub struct RelayHub {
    running: Arc<Mutex<HashMap<u64, Weak<Relay>>>>,
}

impl RelayHub {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&self, id: u64, relay: &Arc<Relay>) {
        let mut running = self.running.lock().unwrap();
        running.retain(|_, relay| relay.strong_count() > 0);
        running.insert(id, Arc::downgrade(relay));
    }

    /// Impair relay job `id` with `profile` from now on.
    pub fn set(&self, id: u64, profile: ImpairProfile) -> EngineResult<()> {
        profile.check()?;
        let relay = self.running.lock().unwrap().get(&id).and_then(Weak::upgrade).ok_or_else(|| EngineError::new("netsim.not_running").with("id", id))?;
        relay.set(profile);
        Ok(())
    }
}

#[derive(Clone, Serialize)]
struct ProxyStat {
    job_id: u64,
    ts: u64,
    #[serde(flatten)]
    counts: ImpairCounts,
    profile: String,
}

/// What a relay serves with: its two UDP sockets, or the TCP listener clients connect to.
pub(crate) enum Opened {
    Udp(Arc<UdpSocket>, Arc<UdpSocket>),
    Tcp(tokio::net::TcpListener),
}

impl Opened {
    /// Bind what a relay of `protocol` listens with (and, over UDP, the socket to the target).
    pub(crate) async fn open(protocol: RelayProtocol, listen: SocketAddr, listen_text: &str, target: SocketAddr, target_text: &str) -> EngineResult<Opened> {
        Ok(match protocol {
            RelayProtocol::Udp => {
                let (downstream, upstream) = open(listen, listen_text, target, target_text).await?;
                Opened::Udp(downstream, upstream)
            }
            RelayProtocol::Tcp => Opened::Tcp(super::netsim_tcp::open(listen, listen_text).await?),
        })
    }

    pub(crate) fn local(&self) -> Option<SocketAddr> {
        match self {
            Opened::Udp(downstream, _) => downstream.local_addr().ok(),
            Opened::Tcp(listener) => listener.local_addr().ok(),
        }
    }
}

impl Relay {
    /// Relay until it cannot any more; the error says why.
    pub(crate) async fn run(self: Arc<Self>, host: Host, job_id: u64, name: String, opened: Opened) -> EngineError {
        match opened {
            Opened::Udp(downstream, upstream) => self.serve(host, job_id, name, downstream, upstream).await,
            Opened::Tcp(listener) => self.serve_tcp(host, job_id, name, listener).await,
        }
    }
}

pub async fn start_proxy(host: Host, jobs: JobRegistry, hub: RelayHub, cfg: ProxyConfig) -> EngineResult<JobInfo> {
    cfg.profile.check()?;
    let listen = parse_bind(&cfg.listen)?;
    let target = parse_target(&cfg.target)?;
    let opened = Opened::open(cfg.protocol, listen, &cfg.listen, target, &cfg.target).await?;
    let local = opened.local().unwrap_or(listen);

    let id = jobs.next_id();
    let mut info = JobInfo::new(id, "netsim", format!("Impair {} → {}", cfg.listen, cfg.target)).with("listen", &cfg.listen).with("target", &cfg.target);
    if cfg.protocol == RelayProtocol::Tcp {
        info = info.with("protocol", "tcp");
    }
    let seed = cfg.seed.unwrap_or_else(rand::random);
    let relay = Relay::new(local, target, cfg.protocol, cfg.profile, seed);
    hub.insert(id, &relay);
    let jobs_cl = jobs.clone();

    let handle = tokio::spawn(async move {
        // The relay and its reporter are children of this task: stopping the
        // job drops this future, and the guard takes both down with it.
        let mut guard = TaskGuard::new();
        let reporter = {
            let (host, relay) = (host.clone(), relay.clone());
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    host.emit("netsim://stat", ProxyStat { job_id: id, ts: now_ms(), counts: relay.counts(), profile: relay.profile().label() });
                }
            })
        };
        guard.watch(reporter.abort_handle());
        // One name for every screen relay: the same seed gives the same drops in any job.
        let error = relay.clone().run(host.clone(), id, "relay".into(), opened).await;
        drop(guard);
        host.emit("job://ended", serde_json::json!({ "job_id": id, "kind": "netsim", "error": error }));
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
        ImpairProfile::default()
    }

    async fn recv(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
        let mut buf = vec![0u8; 1024];
        let (n, from) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf)).await.expect("nothing arrived within 2 s").unwrap();
        (buf[..n].to_vec(), from)
    }

    fn start_relay(host: Host, jobs: JobRegistry, listen: String, target: String, profile: ImpairProfile) -> impl std::future::Future<Output = EngineResult<JobInfo>> {
        start_proxy(host, jobs, RelayHub::new(), ProxyConfig { listen, target, profile, seed: Some(1), protocol: RelayProtocol::Udp })
    }

    /// A throttled packet takes fewer draws than a sent one; the packets after
    /// it must not notice: packet n meets the same loss and copies whatever the
    /// link's timing did to the ones before it.
    #[test]
    fn a_rate_limit_does_not_move_the_seeded_draws() {
        let profile = ImpairProfile { loss: 0.3, duplicate: 0.3, rate_kbps: 8.0, ..calm() };
        let start = Instant::now();
        let fate = |fate: Fate| match fate {
            Fate::Dropped(why) => why.to_string(),
            Fate::Throttled => "throttled".into(),
            Fate::Sent(copies) => format!("sent {}", copies.len()),
        };
        // All at once: the 8 kbps link queues past a second and throttles.
        let mut rushed = Leg::new(9, "leg");
        let crowded: Vec<String> = (0..60).map(|_| fate(rushed.decide(&profile, &[0; 100], start))).collect();
        // Ten seconds apart: the link is free for each one.
        let mut calm_leg = Leg::new(9, "leg");
        let spaced: Vec<String> = (0..60).map(|n| fate(calm_leg.decide(&profile, &[0; 100], start + Duration::from_secs(10 * n)))).collect();
        assert!(crowded.iter().any(|fate| fate == "throttled") && !spaced.iter().any(|fate| fate == "throttled"));
        for (n, (a, b)) in crowded.iter().zip(&spaced).enumerate() {
            if a != "throttled" {
                assert_eq!(a, b, "packet {n} met another fate because earlier ones were throttled");
            }
        }
    }

    /// Stopping a relay stops what it still had on its way: its port is free
    /// at once, and nothing reaches the target after the stop.
    #[tokio::test]
    async fn a_stop_takes_the_delayed_packets_and_the_port_with_it() {
        let host = Host::new(Recorder::new(), Capture::new());
        let jobs = JobRegistry::new();
        let target = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let listen_port = UdpSocket::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
        let slow = ImpairProfile { latency_ms: 800.0, ..calm() };
        let job = start_relay(host, jobs.clone(), format!("127.0.0.1:{listen_port}"), target.local_addr().unwrap().to_string(), slow).await.unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        for _ in 0..5 {
            client.send_to(b"late", ("127.0.0.1", listen_port)).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(jobs.stop(job.id));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(UdpSocket::bind(("127.0.0.1", listen_port)).await.is_ok(), "the listen port is free right after the stop");
        let mut buf = [0u8; 64];
        let late = tokio::time::timeout(Duration::from_millis(1200), target.recv_from(&mut buf)).await;
        assert!(late.is_err(), "nothing delayed went out after the stop");
    }

    /// A copy delayed past a switch is counted where it was decided.
    #[tokio::test]
    async fn a_delayed_packet_counts_in_the_phase_that_decided_it() {
        let target = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (downstream, upstream) = open("127.0.0.1:0".parse().unwrap(), "127.0.0.1:0", target.local_addr().unwrap(), "target").await.unwrap();
        let listen = downstream.local_addr().unwrap();
        let relay = Relay::new(listen, target.local_addr().unwrap(), RelayProtocol::Udp, ImpairProfile { name: "slow".into(), latency_ms: 300.0, ..calm() }, 1);
        let serving = tokio::spawn(relay.clone().serve(Host::new(Recorder::new(), Capture::new()), 0, "test".into(), downstream, upstream));
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(b"one", listen).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        relay.set(ImpairProfile { name: "offline".into(), offline: true, ..calm() });
        recv(&target).await;
        let summary = relay.summary(None);
        let phases: Vec<(&str, u64, u64)> = summary.phases.iter().map(|phase| (phase.profile.as_str(), phase.counts.received, phase.counts.forwarded)).collect();
        assert_eq!(phases, [("slow", 1, 1), ("offline", 0, 0)], "it went out during offline, and counts for slow");
        assert!(summary.opened_ms > 0 && summary.error.is_none());
        serving.abort();
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
        let job = start_relay(host, jobs.clone(), format!("127.0.0.1:{listen_port}"), format!("127.0.0.1:{target_port}"), calm()).await.unwrap();
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
    async fn addresses_and_profiles_that_cannot_work_are_refused_with_a_code() {
        let host = Host::new(Recorder::new(), Capture::new());
        let jobs = JobRegistry::new();
        let start = |listen: &str, target: &str, profile: ImpairProfile| start_relay(host.clone(), jobs.clone(), listen.into(), target.into(), profile);
        assert!(start("127.0.0.1", "127.0.0.1:9", calm()).await.unwrap_err().is("node.bind_invalid"));
        assert!(start("127.0.0.1:0", "localhost", calm()).await.unwrap_err().is("transport.target_invalid"));
        let taken = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let bind = taken.local_addr().unwrap().to_string();
        let error = start(&bind, "127.0.0.1:9", calm()).await.unwrap_err();
        assert_eq!((error.code.as_str(), error.params["target"].as_str()), ("transport.address_in_use", bind.as_str()));
        let lossy = start("127.0.0.1:0", "127.0.0.1:9", ImpairProfile { loss: 1.5, ..calm() }).await.unwrap_err();
        assert_eq!((lossy.code.as_str(), lossy.field.clone().map(|field| field.key)), ("node.range", Some("loss".to_string())));
        assert!(jobs.list().is_empty());
    }

    /// A reply to a client that has gone must not end the relay either.
    #[tokio::test]
    async fn keeps_relaying_after_a_client_went_away() {
        let host = Host::new(Recorder::new(), Capture::new());
        let jobs = JobRegistry::new();
        let target = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let listen_port = UdpSocket::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
        let job = start_relay(host, jobs.clone(), format!("127.0.0.1:{listen_port}"), target.local_addr().unwrap().to_string(), calm()).await.unwrap();
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

    fn fates(seed: u64, profile: &ImpairProfile, n: usize) -> Vec<String> {
        let mut leg = Leg::new(seed, "test");
        let now = Instant::now();
        (0..n)
            .map(|index| match leg.decide(profile, &[index as u8; 100], now) {
                Fate::Dropped(why) => why.to_string(),
                Fate::Throttled => "throttled".into(),
                Fate::Sent(copies) => format!("sent {}", copies.len()),
            })
            .collect()
    }

    #[test]
    fn the_same_seed_drops_the_same_packets() {
        let profile = ImpairProfile { loss: 0.2, duplicate: 0.1, burst_start: 0.05, burst_length: 4.0, ..calm() };
        let first = fates(7, &profile, 2000);
        assert_eq!(first, fates(7, &profile, 2000), "the same seed, the same fates");
        assert_ne!(first, fates(8, &profile, 2000), "another seed, others");
        let lost = first.iter().filter(|fate| *fate == "loss").count();
        assert!((250..=450).contains(&lost), "about 20 % of what bursts leave: {lost}");
        let bursts = first.iter().filter(|fate| *fate == "burst").count();
        assert!(bursts > 100, "bursts of four, started by 5 %: {bursts}");
        assert!(first.windows(3).any(|three| three.iter().all(|fate| fate == "burst")), "losses come in runs");
        assert!(first.iter().any(|fate| fate == "sent 2"), "and some are doubled");
        assert!(fates(1, &ImpairProfile { offline: true, ..calm() }, 50).iter().all(|fate| fate == "offline"));
    }

    #[test]
    fn a_bandwidth_limit_queues_packets_and_drops_past_a_second_of_queue() {
        // 100 bytes at 8 kbps take 100 ms each: the tenth waits 900 ms, the twelfth would wait 1.1 s.
        let profile = ImpairProfile { rate_kbps: 8.0, ..calm() };
        let mut leg = Leg::new(1, "test");
        let now = Instant::now();
        let delays: Vec<Option<u128>> = (0..12)
            .map(|_| match leg.decide(&profile, &[0; 100], now) {
                Fate::Sent(copies) => Some(copies[0].delay.as_millis()),
                _ => None,
            })
            .collect();
        assert_eq!(&delays[..3], [Some(100), Some(200), Some(300)]);
        assert_eq!(delays[9], Some(1000));
        assert_eq!(delays[11], None, "throttled");
        let mut leg = Leg::new(1, "test");
        let Fate::Sent(copies) = leg.decide(&ImpairProfile { latency_ms: 50.0, reorder: 1.0, ..calm() }, &[0; 10], now) else { panic!() };
        assert_eq!((copies[0].delay.as_millis(), copies[0].reordered), (100, true), "held back by the latency again");
    }

    #[test]
    fn profiles_are_checked_and_say_what_they_do() {
        assert!(calm().check().is_ok());
        let field = |profile: ImpairProfile| profile.check().unwrap_err().field.clone().map(|field| field.key);
        assert_eq!(field(ImpairProfile { latency_ms: -1.0, ..calm() }), Some("latency_ms".into()));
        assert_eq!(field(ImpairProfile { reorder: f64::NAN, ..calm() }), Some("reorder".into()));
        assert_eq!(field(ImpairProfile { rate_kbps: 2.0, ..calm() }), Some("rate_kbps".into()));
        assert_eq!(field(ImpairProfile { burst_start: 0.1, burst_length: 0.0, ..calm() }), Some("burst_length".into()));
        assert_eq!(calm().label(), "clean");
        assert_eq!(ImpairProfile { name: "4g".into(), loss: 0.5, ..calm() }.label(), "4g");
        assert_eq!(ImpairProfile { latency_ms: 60.0, jitter_ms: 25.0, loss: 0.02, rate_kbps: 20000.0, ..calm() }.label(), "60 ms ±25 · loss 2% · 20000 kbps");
        assert_eq!(ImpairProfile { offline: true, ..calm() }.label(), "offline");
        assert_eq!(field(ImpairProfile { reset: 1.5, ..calm() }), Some("reset".into()));
        // Each protocol reads its own values: a 4G preset on a TCP relay keeps its delay and its rate.
        let preset = ImpairProfile { latency_ms: 60.0, jitter_ms: 25.0, loss: 0.005, reorder: 0.005, rate_kbps: 20000.0, reset: 0.01, ..calm() };
        assert_eq!(preset.for_protocol(RelayProtocol::Tcp).label(), "60 ms ±25 · reset 1% · 20000 kbps");
        assert_eq!(preset.for_protocol(RelayProtocol::Udp).label(), "60 ms ±25 · loss 0.5% · reorder 0.5% · 20000 kbps");
        // Absent in a document: zero, and nothing written back.
        let profile: ImpairProfile = serde_json::from_value(serde_json::json!({ "latency_ms": 5 })).unwrap();
        let written = serde_json::to_value(&profile).unwrap();
        assert!(profile.reset == 0.0 && written.get("reset").is_none() && written.get("stall").is_none());
    }

    #[tokio::test]
    async fn a_running_relay_changes_its_profile_and_counts_each_phase() {
        let host = Host::new(Recorder::new(), Capture::new());
        let (jobs, hub) = (JobRegistry::new(), RelayHub::new());
        let target = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let listen_port = UdpSocket::bind("127.0.0.1:0").await.unwrap().local_addr().unwrap().port();
        let cfg = ProxyConfig { listen: format!("127.0.0.1:{listen_port}"), target: target.local_addr().unwrap().to_string(), profile: ImpairProfile { name: "lan".into(), ..calm() }, seed: Some(3), protocol: RelayProtocol::Udp };
        let job = start_proxy(host, jobs.clone(), hub.clone(), cfg).await.unwrap();
        let relay: SocketAddr = format!("127.0.0.1:{listen_port}").parse().unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(b"one", relay).await.unwrap();
        assert_eq!(recv(&target).await.0, b"one");
        hub.set(job.id, ImpairProfile { name: "offline".into(), offline: true, ..calm() }).unwrap();
        for _ in 0..3 {
            client.send_to(b"lost", relay).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        let running = hub.running.lock().unwrap().get(&job.id).and_then(Weak::upgrade).unwrap();
        let summary = running.summary(None);
        assert_eq!((summary.counts.received, summary.counts.forwarded, summary.counts.dropped), (4, 1, 3));
        let phases: Vec<(&str, u64, u64)> = summary.phases.iter().map(|phase| (phase.profile.as_str(), phase.counts.forwarded, phase.counts.dropped)).collect();
        assert_eq!(phases, [("lan", 1, 0), ("offline", 0, 3)]);
        assert!(hub.set(job.id, ImpairProfile { loss: 2.0, ..calm() }).unwrap_err().is("node.range"));
        jobs.stop(job.id);
        tokio::time::sleep(Duration::from_millis(50)).await;
        drop(running);
        assert!(hub.set(job.id, calm()).unwrap_err().is("netsim.not_running"));
        assert!(UdpSocket::bind(relay).await.is_ok(), "stopped means the port is free again");
    }
}
