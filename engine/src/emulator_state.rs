//! What every protocol shares while it serves: the counters and the newest
//! exchanges of one emulator (`Emulation`), and the context a reply is
//! rendered, delayed and captured in (`Context`).

use std::collections::{BTreeMap, VecDeque};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::host::Host;

use super::emulator::{DownFault, Fault};
use super::emulator_rules::Compiled;
use super::error::{EngineError, EngineResult};
use super::inspect::{self, Frame, Gate};
use super::jobs::now_ms;
use super::listen::Inbox;
use super::template::{Renderer, Rng, Scope};

/// Exchanges kept for whoever asks what arrived (the screen, a script, an LLM).
pub const RECENT: usize = 500;
/// Exchanges per activity event; more in one interval are counted, not sent.
const BATCH: usize = 200;
/// At most one exchange in this many milliseconds reaches the Inspector.
const FRAME_GAP_MS: u64 = 10;
/// Bytes of a request body an exchange keeps for whoever asks later.
const KEPT_BODY: usize = 4 * 1024;
/// Mixed into the seed, so drawing a delay never draws the numbers a
/// template's generators or a route's pick do.
const DELAY_STREAM: u64 = 0x6465_6c61_7900_0001;

/// One request (message, line) and what became of it.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Exchange {
    pub seq: u64,
    pub ts: u64,
    pub from: String,
    /// What arrived, in protocol notation: `GET /users/7`, `/ping 1`, `POWER?`.
    pub request: String,
    /// The rule that answered (1-based); none when no rule took it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<usize>,
    /// What went back, in protocol notation; empty when nothing did.
    pub reply: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault: Option<Fault>,
    /// From arrival until the reply left, its delay included.
    pub ms: u64,
    /// Why no reply could be made or sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EngineError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<u64>,
    /// It arrived while the emulator was down (its `Outage`): no rule was asked.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub down: bool,
    /// What arrived as templates read it (`request`), a long body cut short.
    #[serde(skip_serializing_if = "Value::is_null")]
    pub data: Value,
}

/// Requests, how many no rule took, how many failed, how many met an outage,
/// and the hits per rule. `missed`: messages a broker could not hand to a
/// client too far behind (MQTT only; left out while none were).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Counts {
    pub total: u64,
    pub unmatched: u64,
    pub failed: u64,
    pub down: u64,
    #[serde(skip_serializing_if = "is_zero")]
    pub missed: u64,
    pub hits: Vec<u64>,
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

#[derive(Default)]
struct Fresh {
    exchanges: Vec<Exchange>,
    dropped: u64,
    changed: bool,
}

/// What one emulator has seen so far: counters and the newest exchanges.
pub struct Emulation {
    pub name: String,
    pub protocol: &'static str,
    pub local: SocketAddr,
    hits: Vec<AtomicU64>,
    total: AtomicU64,
    unmatched: AtomicU64,
    failed: AtomicU64,
    down: AtomicU64,
    missed: AtomicU64,
    /// Taken down by a run's *Emulator down/up* step or a person, until brought up.
    forced: Mutex<Option<DownFault>>,
    seq: AtomicU64,
    recent: Mutex<VecDeque<Exchange>>,
    fresh: Mutex<Fresh>,
}

impl Emulation {
    pub(crate) fn new(name: &str, protocol: &'static str, local: SocketAddr, rules: usize) -> Arc<Self> {
        Arc::new(Emulation {
            name: name.to_string(),
            protocol,
            local,
            hits: (0..rules).map(|_| AtomicU64::new(0)).collect(),
            total: AtomicU64::new(0),
            unmatched: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            down: AtomicU64::new(0),
            missed: AtomicU64::new(0),
            forced: Mutex::new(None),
            seq: AtomicU64::new(0),
            recent: Mutex::new(VecDeque::new()),
            fresh: Mutex::new(Fresh::default()),
        })
    }

    /// Count a hit of rule `index` (0-based): how many it has had, this one included.
    pub(crate) fn hit(&self, index: usize) -> u64 {
        self.hits.get(index).map(|hits| hits.fetch_add(1, Ordering::Relaxed) + 1).unwrap_or(1)
    }

    /// Down (`Some`, with what HTTP meets) or up again (`None`), whatever its outage says.
    pub fn force(&self, down: Option<DownFault>) {
        *self.forced.lock().unwrap() = down;
        self.fresh.lock().unwrap().changed = true;
    }

    /// Whether, and how, it was taken down.
    pub fn forced(&self) -> Option<DownFault> {
        *self.forced.lock().unwrap()
    }

    /// Messages a broker could not deliver to a slow client.
    pub(crate) fn miss(&self, count: u64) {
        if count > 0 {
            self.missed.fetch_add(count, Ordering::Relaxed);
            self.fresh.lock().unwrap().changed = true;
        }
    }

    /// Requests no rule took so far, plus this one: the fallback's `{{counter}}`.
    pub(crate) fn next_unmatched(&self) -> u64 {
        self.unmatched.load(Ordering::Relaxed) + 1
    }

    pub(crate) fn record(&self, mut exchange: Exchange) {
        exchange.seq = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
        if exchange.ts == 0 {
            exchange.ts = now_ms();
        }
        self.total.fetch_add(1, Ordering::Relaxed);
        if exchange.down {
            self.down.fetch_add(1, Ordering::Relaxed);
        } else if exchange.rule.is_none() {
            self.unmatched.fetch_add(1, Ordering::Relaxed);
        }
        if exchange.error.is_some() {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
        {
            let mut recent = self.recent.lock().unwrap();
            if recent.len() == RECENT {
                recent.pop_front();
            }
            recent.push_back(exchange.clone());
        }
        let mut fresh = self.fresh.lock().unwrap();
        fresh.changed = true;
        if fresh.exchanges.len() >= BATCH {
            fresh.dropped += 1;
        } else {
            // What arrived stays with the command that asks for it; events stay small.
            fresh.exchanges.push(Exchange { data: Value::Null, ..exchange });
        }
    }

    pub fn counts(&self) -> Counts {
        Counts {
            total: self.total.load(Ordering::Relaxed),
            unmatched: self.unmatched.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            down: self.down.load(Ordering::Relaxed),
            missed: self.missed.load(Ordering::Relaxed),
            hits: self.hits.iter().map(|hits| hits.load(Ordering::Relaxed)).collect(),
        }
    }

    /// Kept exchanges after `after` (a `seq`), oldest first, at most `limit`.
    pub fn exchanges(&self, after: u64, limit: usize) -> Vec<Exchange> {
        let recent = self.recent.lock().unwrap();
        recent.iter().filter(|exchange| exchange.seq > after).take(limit).cloned().collect()
    }

    pub(crate) fn take_fresh(&self) -> Option<(Vec<Exchange>, u64)> {
        let mut fresh = self.fresh.lock().unwrap();
        if !fresh.changed {
            return None;
        }
        let taken = std::mem::take(&mut *fresh);
        Some((taken.exchanges, taken.dropped))
    }

    pub fn summary(&self, node: Option<&str>) -> EmulatorSummary {
        EmulatorSummary { node: node.map(str::to_string), name: self.name.clone(), protocol: self.protocol, local: self.local.to_string(), counts: self.counts() }
    }
}

/// An emulator's counters in a run's report.
#[derive(Clone, Debug, Serialize)]
pub struct EmulatorSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    pub name: String,
    pub protocol: &'static str,
    pub local: String,
    pub counts: Counts,
}

/// A request body as an exchange keeps it: the start of a long one, and no JSON then.
pub(crate) fn kept(request: &Value) -> Value {
    let mut value = request.clone();
    // An HTTP request's body, an MQTT message's payload.
    for key in ["body", "payload"] {
        if let Some(body) = request.get(key).and_then(Value::as_str) {
            if body.len() > KEPT_BODY {
                let mut end = KEPT_BODY;
                while !body.is_char_boundary(end) {
                    end -= 1;
                }
                value[key] = Value::String(format!("{}…", &body[..end]));
                value["json"] = Value::Null;
            }
        }
    }
    value
}

/// An emulator that is down: what an HTTP request meets, and when it is back
/// (`None`: when someone brings it up).
pub(crate) struct Down {
    pub fault: DownFault,
    pub back: Option<Duration>,
}

/// Everything a serving emulator reads: its rules, its counters, where it reports.
pub(crate) struct Context {
    pub host: Host,
    pub compiled: Compiled,
    pub emulation: Arc<Emulation>,
    pub seed: u64,
    /// The job its frames belong to: the emulator's own, or the run's.
    pub job: Option<u64>,
    /// What a run's *Wait for HTTP request* steps read (HTTP only).
    pub inbox: Option<Arc<Inbox>>,
    gate: Gate,
    /// When it started serving: an outage's schedule counts from here.
    started: Instant,
}

impl Context {
    pub(crate) fn new(host: Host, compiled: Compiled, emulation: Arc<Emulation>, seed: u64, job: Option<u64>, inbox: Option<Arc<Inbox>>) -> Arc<Self> {
        Arc::new(Context { host, compiled, emulation, seed, job, inbox, gate: Gate::new(FRAME_GAP_MS), started: Instant::now() })
    }

    /// Whether it is down now — taken down, or in its outage's down stretch —
    /// what HTTP meets meanwhile, and when it is back if that is known.
    pub(crate) fn down(&self) -> Option<Down> {
        if let Some(fault) = self.emulation.forced() {
            return Some(Down { fault, back: None });
        }
        let outage = self.compiled.outage.as_ref()?;
        let period = outage.up_ms + outage.down_ms;
        let into = (self.started.elapsed().as_millis() % u128::from(period)) as u64;
        (into >= outage.up_ms).then(|| Down { fault: outage.fault, back: Some(Duration::from_millis(period - into)) })
    }

    /// Render the fields of rule `rule`'s reply (1-based; 0 for a fallback or
    /// a greeting) at its `count`th hit, with one renderer, so the generators
    /// of one reply draw one stream.
    pub(crate) fn render<T>(&self, rule: usize, count: u64, request: &Value, work: impl FnOnce(&mut Renderer<'_>) -> EngineResult<T>) -> EngineResult<T> {
        let vars = BTreeMap::from([("request".to_string(), request.clone())]);
        let secrets = BTreeMap::new();
        let node = format!("rule-{rule}");
        let scope = Scope { params: &self.compiled.params, vars: &vars, secrets: &secrets, run_id: self.job.unwrap_or(0), seed: self.seed, node_id: &node, count, now_ms: now_ms() };
        let mut renderer = Renderer::new(scope);
        work(&mut renderer).map_err(|error| if rule > 0 { error.with("rule", rule) } else { error })
    }

    /// The delay before rule `rule`'s reply: its own plus seeded jitter.
    pub(crate) fn delay(&self, rule: usize, count: u64, delay_ms: u64, jitter_ms: u64) -> Duration {
        let jitter = if jitter_ms == 0 { 0 } else { Rng::for_node(self.seed ^ DELAY_STREAM, &format!("rule-{rule}"), count).below_u64(jitter_ms + 1) };
        Duration::from_millis(delay_ms + jitter)
    }

    /// Whether this exchange goes to the Inspector — decided once, so its
    /// request and its reply are both there or neither.
    pub(crate) fn capturing(&self) -> bool {
        inspect::armed(&self.host) && self.gate.allow()
    }

    pub(crate) fn publish(&self, frame: Frame) -> Option<u64> {
        let frame = frame.local(self.emulation.local);
        let frame = match self.job {
            Some(id) => frame.job(id),
            None => frame,
        };
        inspect::publish(&self.host, frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_newest_exchanges_are_kept_and_events_carry_no_bodies() {
        let emulation = Emulation::new("API", "http", "127.0.0.1:8080".parse().unwrap(), 2);
        assert_eq!((emulation.hit(1), emulation.hit(1), emulation.hit(0)), (1, 2, 1));
        for index in 0..RECENT + 3 {
            let rule = if index % 2 == 0 { Some(1) } else { None };
            emulation.record(Exchange { request: format!("GET /{index}"), rule, data: json!({ "body": "x" }), ..Default::default() });
        }
        let counts = emulation.counts();
        assert_eq!((counts.total, counts.unmatched, counts.hits.clone()), ((RECENT + 3) as u64, ((RECENT + 3) / 2) as u64, vec![1, 2]));
        let kept = emulation.exchanges(0, RECENT);
        assert_eq!((kept.len(), kept[0].seq), (RECENT, 4), "the oldest three are gone");
        assert_eq!(emulation.exchanges(RECENT as u64, 10).len(), 3);
        let (fresh, dropped) = emulation.take_fresh().unwrap();
        assert_eq!((fresh.len(), dropped), (BATCH, (RECENT + 3 - BATCH) as u64));
        assert!(fresh.iter().all(|exchange| exchange.data.is_null()) && !kept[0].data.is_null());
        assert!(emulation.take_fresh().is_none(), "nothing new since");
        let long = kept_body_case();
        assert!(long["body"].as_str().unwrap().len() < KEPT_BODY + 8 && long["json"].is_null());
    }

    fn kept_body_case() -> Value {
        kept(&json!({ "body": "é".repeat(KEPT_BODY), "json": { "a": 1 } }))
    }
}
