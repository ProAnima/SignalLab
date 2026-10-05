//! Load on an HTTP node: its request sent on a profile — a rate that holds,
//! ramps, steps or spikes over time, or arrives at random — `concurrency` at a
//! time, measured and judged by thresholds. The document's part (`Load`) and
//! its checks, the schedule, the measuring and the verdicts are here;
//! `experiment_flow` runs it as a step and reports it.

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{Interval, MissedTickBehavior};

use crate::host::Host;

use super::cookies::CookieJar;
use super::error::{EngineError, EngineResult, Field};
use super::experiment::RUN_LIMIT;
use super::http::{self, HttpRequest, HttpResponse, MAX_BURST_RATE, MIN_BURST_RATE};
use super::http_auth::DigestMemory;
use super::inspect::{self, Gate};
use super::latency::LatencyHistogram;
use super::template::Rng;

/// At most this many requests in flight at once, as in the burst.
pub const MAX_CONCURRENCY: u32 = 512;
/// The shortest profile, and the shortest step of one.
pub const MIN_DURATION_MS: u64 = 100;
/// The most steps a Steps profile climbs.
pub const MAX_STEPS: u32 = 100;
pub const MAX_THRESHOLDS: usize = 16;
/// How often a load reports how far it has got.
pub const REPORT_EVERY: Duration = Duration::from_secs(1);
/// The upper ends of the coarse latency histogram's bins, in milliseconds; the
/// last bin holds everything slower.
pub const LATENCY_BINS_MS: &[f64] = &[1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10_000.0];
/// Mixed into the seed, so Poisson arrivals never draw the numbers a node's templates do.
const ARRIVALS_STREAM: u64 = 0x6c6f_6164_0000_0001;

/// An HTTP node's load: how many requests when (`profile`), how many at once,
/// and what the step must measure to pass. In place of Repeat and Retry: a
/// failed request is counted, not tried again.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Load {
    pub profile: LoadProfile,
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    #[serde(default)]
    pub thresholds: Vec<Threshold>,
}

fn default_concurrency() -> u32 {
    32
}

/// Requests per second over time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum LoadProfile {
    /// `rate` throughout.
    Constant { rate: f64, duration_ms: u64 },
    /// From `from` to `to`, linearly.
    Ramp { from: f64, to: f64, duration_ms: u64 },
    /// `from`, then `from + step`, … — `steps` levels, each for `every_ms`.
    Steps { from: f64, step: f64, every_ms: u64, steps: u32 },
    /// `base`, `peak` from `at_ms` for `spike_ms`, then `base` again.
    Spike { base: f64, peak: f64, at_ms: u64, spike_ms: u64, duration_ms: u64 },
    /// Arrivals at random, `rate` on average, drawn from the run's seed.
    Poisson { rate: f64, duration_ms: u64 },
}

/// What a threshold reads of a load's measurements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    P50Ms,
    P90Ms,
    P95Ms,
    P99Ms,
    MeanMs,
    MaxMs,
    /// Failed requests, in % of those sent.
    ErrorRate,
    /// Requests per second achieved.
    Rps,
    /// Requests due while every worker was busy, skipped.
    Missed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdOp {
    Lt,
    Le,
    Gt,
    Ge,
}

/// The step passes only when `metric op value` holds for the load.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Threshold {
    pub metric: Metric,
    pub op: ThresholdOp,
    pub value: f64,
}

impl Metric {
    /// This metric of `metrics`.
    pub fn of(self, metrics: &LoadMetrics) -> f64 {
        match self {
            Metric::P50Ms => metrics.p50_ms,
            Metric::P90Ms => metrics.p90_ms,
            Metric::P95Ms => metrics.p95_ms,
            Metric::P99Ms => metrics.p99_ms,
            Metric::MeanMs => metrics.mean_ms,
            Metric::MaxMs => metrics.max_ms,
            Metric::ErrorRate => metrics.error_rate,
            Metric::Rps => metrics.rps,
            Metric::Missed => metrics.missed as f64,
        }
    }

    /// Its name in a document and in a message's params.
    pub fn code(self) -> &'static str {
        match self {
            Metric::P50Ms => "p50_ms",
            Metric::P90Ms => "p90_ms",
            Metric::P95Ms => "p95_ms",
            Metric::P99Ms => "p99_ms",
            Metric::MeanMs => "mean_ms",
            Metric::MaxMs => "max_ms",
            Metric::ErrorRate => "error_rate",
            Metric::Rps => "rps",
            Metric::Missed => "missed",
        }
    }
}

impl ThresholdOp {
    pub fn holds(self, actual: f64, value: f64) -> bool {
        match self {
            ThresholdOp::Lt => actual < value,
            ThresholdOp::Le => actual <= value,
            ThresholdOp::Gt => actual > value,
            ThresholdOp::Ge => actual >= value,
        }
    }

    /// As a person writes it: notation, not a sentence.
    pub fn symbol(self) -> &'static str {
        match self {
            ThresholdOp::Lt => "<",
            ThresholdOp::Le => "≤",
            ThresholdOp::Gt => ">",
            ThresholdOp::Ge => "≥",
        }
    }
}

// ---------------------------------------------------------------------------
// Checking a load
// ---------------------------------------------------------------------------

fn range(field: Field, min: impl ToString, max: impl ToString) -> EngineError {
    EngineError::new("node.range").with("min", min).with("max", max).in_field(field)
}

/// A rate field: `MIN_BURST_RATE` … `MAX_BURST_RATE`, or from 0 where the profile may start or rest at nothing.
fn check_rate(rate: f64, field: Field, zero: bool) -> EngineResult<()> {
    let min = if zero { 0.0 } else { MIN_BURST_RATE };
    if !rate.is_finite() || rate < min || rate > MAX_BURST_RATE {
        return Err(range(field, min, MAX_BURST_RATE));
    }
    Ok(())
}

fn check_duration(ms: u64, field: Field) -> EngineResult<()> {
    let limit = RUN_LIMIT.as_millis() as u64;
    if !(MIN_DURATION_MS..=limit).contains(&ms) {
        return Err(range(field, MIN_DURATION_MS, limit));
    }
    Ok(())
}

/// A load that cannot run as written is refused before anything is sent: its
/// rates within the burst's bounds, all of it within a run, at least one request.
pub fn check(load: &Load) -> EngineResult<()> {
    if !(1..=MAX_CONCURRENCY).contains(&load.concurrency) {
        return Err(range(Field::new("load_concurrency"), 1, MAX_CONCURRENCY));
    }
    match &load.profile {
        LoadProfile::Constant { rate, duration_ms } | LoadProfile::Poisson { rate, duration_ms } => {
            check_rate(*rate, Field::new("load_rate"), false)?;
            check_duration(*duration_ms, Field::new("load_duration"))?;
        }
        LoadProfile::Ramp { from, to, duration_ms } => {
            check_rate(*from, Field::new("load_from"), true)?;
            check_rate(*to, Field::new("load_to"), true)?;
            check_duration(*duration_ms, Field::new("load_duration"))?;
        }
        LoadProfile::Steps { from, step, every_ms, steps } => {
            check_rate(*from, Field::new("load_from"), true)?;
            if !(1..=MAX_STEPS).contains(steps) {
                return Err(range(Field::new("load_steps"), 1, MAX_STEPS));
            }
            let last = from + step * (*steps as f64 - 1.0);
            if !step.is_finite() || !(0.0..=MAX_BURST_RATE).contains(&last) {
                return Err(EngineError::new("load.steps_out_of_range").with("max", MAX_BURST_RATE).in_field(Field::new("load_step")));
            }
            check_duration(*every_ms, Field::new("load_every"))?;
            if every_ms.saturating_mul(*steps as u64) > RUN_LIMIT.as_millis() as u64 {
                return Err(EngineError::new("load.too_long").with("seconds", RUN_LIMIT.as_secs()).in_field(Field::new("load_steps")));
            }
        }
        LoadProfile::Spike { base, peak, at_ms, spike_ms, duration_ms } => {
            check_rate(*base, Field::new("load_base"), true)?;
            check_rate(*peak, Field::new("load_peak"), false)?;
            check_duration(*duration_ms, Field::new("load_duration"))?;
            if *spike_ms == 0 || at_ms.saturating_add(*spike_ms) > *duration_ms {
                return Err(EngineError::new("load.spike_outside").in_field(Field::new("load_spike")));
            }
        }
    }
    if load.profile.expected() <= 0.0 {
        return Err(EngineError::new("load.nothing_planned").in_field(Field::new("load")));
    }
    if load.thresholds.len() > MAX_THRESHOLDS {
        return Err(EngineError::new("load.thresholds_too_many").with("max", MAX_THRESHOLDS).in_field(Field::new("thresholds")));
    }
    if let Some(index) = load.thresholds.iter().position(|threshold| !threshold.value.is_finite() || threshold.value < 0.0) {
        return Err(EngineError::new("load.threshold_value").in_field(Field::nth("threshold", index + 1)));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The schedule
// ---------------------------------------------------------------------------

/// A stretch of the profile where the rate goes straight from `r0` to `r1`
/// (per second) over `len` seconds, `before` requests due before it.
#[derive(Clone, Copy, Debug)]
struct Segment {
    start: f64,
    len: f64,
    r0: f64,
    r1: f64,
    before: f64,
}

impl Segment {
    fn count(&self) -> f64 {
        (self.r0 + self.r1) / 2.0 * self.len
    }

    /// Requests due in its first `tau` seconds: the rate's integral.
    fn count_in(&self, tau: f64) -> f64 {
        let tau = tau.clamp(0.0, self.len);
        self.r0 * tau + (self.r1 - self.r0) / (2.0 * self.len) * tau * tau
    }

    /// Where in it `count_in` reaches `m`: the quadratic solved in the form
    /// that stays exact when the rate is flat or starts at nothing.
    fn moment_of(&self, m: f64) -> f64 {
        if m <= 0.0 {
            return 0.0;
        }
        let a = (self.r1 - self.r0) / (2.0 * self.len);
        let root = (self.r0 * self.r0 + 4.0 * a * m).max(0.0).sqrt();
        let denominator = self.r0 + root;
        if denominator <= 0.0 { self.len } else { (2.0 * m / denominator).min(self.len) }
    }

    fn rate_at(&self, tau: f64) -> f64 {
        self.r0 + (self.r1 - self.r0) * (tau / self.len).clamp(0.0, 1.0)
    }
}

impl LoadProfile {
    pub fn duration(&self) -> Duration {
        Duration::from_millis(match self {
            LoadProfile::Constant { duration_ms, .. } | LoadProfile::Ramp { duration_ms, .. } | LoadProfile::Spike { duration_ms, .. } | LoadProfile::Poisson { duration_ms, .. } => *duration_ms,
            LoadProfile::Steps { every_ms, steps, .. } => every_ms.saturating_mul(*steps as u64),
        })
    }

    /// The rate over time as straight pieces; a Poisson profile's is its average.
    fn segments(&self) -> Vec<Segment> {
        let s = |ms: u64| ms as f64 / 1000.0;
        let pieces: Vec<(f64, f64, f64)> = match *self {
            LoadProfile::Constant { rate, duration_ms } | LoadProfile::Poisson { rate, duration_ms } => vec![(s(duration_ms), rate, rate)],
            LoadProfile::Ramp { from, to, duration_ms } => vec![(s(duration_ms), from, to)],
            LoadProfile::Steps { from, step, every_ms, steps } => (0..steps).map(|k| {
                let rate = from + step * k as f64;
                (s(every_ms), rate, rate)
            }).collect(),
            LoadProfile::Spike { base, peak, at_ms, spike_ms, duration_ms } => vec![
                (s(at_ms), base, base),
                (s(spike_ms), peak, peak),
                (s(duration_ms.saturating_sub(at_ms.saturating_add(spike_ms))), base, base),
            ],
        };
        let (mut start, mut before) = (0.0, 0.0);
        let mut segments = Vec::new();
        for (len, r0, r1) in pieces.into_iter().filter(|(len, _, _)| *len > 0.0) {
            let segment = Segment { start, len, r0, r1, before };
            start += len;
            before += segment.count();
            segments.push(segment);
        }
        segments
    }

    /// The rate the profile asks for `t` into it, per second.
    pub fn rate_at(&self, t: Duration) -> f64 {
        let t = t.as_secs_f64();
        self.segments().iter().find(|segment| t < segment.start + segment.len).map(|segment| segment.rate_at(t - segment.start)).unwrap_or(0.0)
    }

    /// The requests its rate adds up to, as a fraction.
    pub fn expected(&self) -> f64 {
        self.segments().iter().map(Segment::count).sum()
    }

    /// The requests it adds up to (a Poisson profile: on average).
    pub fn planned(&self) -> u64 {
        let total = self.expected();
        match self {
            LoadProfile::Poisson { .. } => total.round() as u64,
            // The n-th is due where the count reaches n, from 0: every n below the total.
            _ => (total - 1e-9).ceil().max(0.0) as u64,
        }
    }
}

enum Plan {
    /// The n-th request at the moment the profile's count reaches n.
    Fluid { segments: Vec<Segment>, planned: u64 },
    /// Exponential gaps, `rate` on average; `at` is the next arrival.
    Poisson { rng: Rng, rate: f64, at: f64 },
}

/// When each request of a load is due, from its start. Each moment is computed
/// from the profile, so a late wake-up never shifts the ones after it.
pub struct Schedule {
    end: f64,
    planned: u64,
    seconds: usize,
    n: u64,
    plan: Plan,
}

impl Schedule {
    /// A Poisson profile's arrivals are drawn per node from the run's seed: the
    /// same seed, the same moments.
    pub fn new(profile: &LoadProfile, seed: u64, node_id: &str) -> Self {
        let end = profile.duration().as_secs_f64();
        let planned = profile.planned();
        let plan = match *profile {
            LoadProfile::Poisson { rate, .. } => {
                let mut rng = Rng::for_node(seed ^ ARRIVALS_STREAM, node_id, 0);
                let at = gap(&mut rng, rate);
                Plan::Poisson { rng, rate, at }
            }
            _ => Plan::Fluid { segments: profile.segments(), planned },
        };
        Schedule { end, planned, seconds: (end.ceil() as usize).max(1), n: 0, plan }
    }

    pub fn planned(&self) -> u64 {
        self.planned
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.end)
    }

    /// Whole seconds the profile spans, the last one perhaps in part.
    pub fn seconds(&self) -> usize {
        self.seconds
    }

    /// Requests handed out or skipped so far.
    pub fn taken(&self) -> u64 {
        self.n
    }

    /// The next request's moment, or None when the profile is over.
    pub fn next(&self) -> Option<Duration> {
        match &self.plan {
            Plan::Fluid { segments, planned } => (self.n < *planned).then(|| Duration::from_secs_f64(moment(segments, self.n as f64).min(self.end))),
            Plan::Poisson { at, .. } => (*at < self.end).then(|| Duration::from_secs_f64(*at)),
        }
    }

    /// The next request is on its way.
    pub fn advance(&mut self) {
        self.n += 1;
        if let Plan::Poisson { rng, rate, at } = &mut self.plan {
            *at += gap(rng, *rate);
        }
    }

    /// Every request due before `t` that has not been handed out is skipped;
    /// returns how many.
    pub fn skip_before(&mut self, t: Duration) -> u64 {
        let t = t.as_secs_f64();
        match &mut self.plan {
            Plan::Fluid { segments, planned } => {
                let first = ((count_until(segments, t) - 1e-9).ceil().max(0.0) as u64).min(*planned);
                let skipped = first.saturating_sub(self.n);
                self.n += skipped;
                skipped
            }
            Plan::Poisson { rng, rate, at } => {
                let mut skipped = 0;
                while *at < t && *at < self.end {
                    *at += gap(rng, *rate);
                    skipped += 1;
                }
                self.n += skipped;
                skipped
            }
        }
    }
}

/// An exponential gap between two arrivals, `1 / rate` on average.
fn gap(rng: &mut Rng, rate: f64) -> f64 {
    -(1.0 - rng.unit()).ln() / rate
}

/// When the count reaches `n`: in the segment where it does.
fn moment(segments: &[Segment], n: f64) -> f64 {
    let index = segments.partition_point(|segment| segment.before + segment.count() <= n);
    match segments.get(index) {
        Some(segment) => segment.start + segment.moment_of(n - segment.before),
        // Rounding at the very end: the last moment there is.
        None => segments.last().map(|segment| segment.start + segment.len).unwrap_or(0.0),
    }
}

/// Requests due in the first `t` seconds.
fn count_until(segments: &[Segment], t: f64) -> f64 {
    let index = segments.partition_point(|segment| segment.start + segment.len <= t);
    match segments.get(index) {
        Some(segment) => segment.before + segment.count_in(t - segment.start),
        None => segments.iter().map(Segment::count).sum(),
    }
}

// ---------------------------------------------------------------------------
// Measuring
// ---------------------------------------------------------------------------

/// What a load measured: the step's event and the run report carry it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// Requests the profile adds up to (Poisson: on average).
    pub planned: u64,
    /// Answered or failed.
    pub sent: u64,
    /// Answered 2xx.
    pub ok: u64,
    /// Any other status, or no answer.
    pub failed: u64,
    /// Due while every worker was busy, and skipped.
    pub missed: u64,
    /// How long the profile ran.
    pub duration_ms: u64,
    /// Requests sent per second over `duration_ms`.
    pub rps: f64,
    /// `failed`, in % of `sent`.
    pub error_rate: f64,
    pub min_ms: f64,
    pub mean_ms: f64,
    pub max_ms: f64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    /// Body bytes received in all.
    pub received_bytes: u64,
    /// Requests by status (`"200"`) or, without one, by cause (`"timeout"`).
    pub statuses: BTreeMap<String, u64>,
    /// Each second of the profile: what was sent in it, what of that failed, how long it took.
    pub seconds: Vec<LoadSecond>,
    /// How many took up to each of `LATENCY_BINS_MS`, the last bin slower.
    pub histogram: Vec<LatencyBin>,
    /// Each threshold, in the document's order, read against the above.
    pub thresholds: Vec<Verdict>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LoadSecond {
    pub sent: u64,
    pub failed: u64,
    pub mean_ms: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LatencyBin {
    /// None: slower than the last bound.
    pub upto_ms: Option<f64>,
    pub count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    pub metric: Metric,
    pub op: ThresholdOp,
    pub value: f64,
    pub actual: f64,
    pub held: bool,
}

impl Verdict {
    /// The step's failure for a threshold that did not hold, the `index`-th.
    pub fn error(&self, index: usize) -> EngineError {
        EngineError::new("load.threshold")
            .with("metric", self.metric.code())
            .with("op", self.op.symbol())
            .with("value", shown(self.value))
            .with("actual", shown(self.actual))
            .in_field(Field::nth("threshold", index + 1))
    }
}

/// A number as a message shows it: two decimals at most, none when whole.
pub fn shown(value: f64) -> String {
    let text = format!("{:.2}", value);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Every threshold read against `metrics`, into `metrics.thresholds`.
pub fn judge(thresholds: &[Threshold], metrics: &mut LoadMetrics) {
    metrics.thresholds = thresholds
        .iter()
        .map(|threshold| {
            let actual = threshold.metric.of(metrics);
            Verdict { metric: threshold.metric, op: threshold.op, value: threshold.value, actual, held: threshold.op.holds(actual, threshold.value) }
        })
        .collect();
}

#[derive(Default)]
struct Second {
    sent: AtomicU64,
    failed: AtomicU64,
    answered: AtomicU64,
    sum_us: AtomicU64,
}

/// What every request of a load adds to, from many tasks at once.
struct Stats {
    sent: AtomicU64,
    ok: AtomicU64,
    failed: AtomicU64,
    missed: AtomicU64,
    received: AtomicU64,
    latency: LatencyHistogram,
    /// By status code; a code past 599 counts as 599.
    statuses: Vec<AtomicU64>,
    /// Failures without a status, by cause: rare next to statuses.
    causes: Mutex<BTreeMap<String, u64>>,
    seconds: Vec<Second>,
}

impl Stats {
    fn new(seconds: usize) -> Self {
        Stats {
            sent: AtomicU64::new(0),
            ok: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            missed: AtomicU64::new(0),
            received: AtomicU64::new(0),
            latency: LatencyHistogram::new(),
            statuses: (0..600).map(|_| AtomicU64::new(0)).collect(),
            causes: Mutex::new(BTreeMap::new()),
            seconds: (0..seconds).map(|_| Second::default()).collect(),
        }
    }

    /// The second of the profile a request due at `due` belongs to.
    fn second_of(&self, due: Duration) -> usize {
        (due.as_secs_f64() as usize).min(self.seconds.len() - 1)
    }

    fn record(&self, response: &HttpResponse, second: usize) {
        let micros = (response.latency_ms * 1000.0) as u64;
        self.latency.record(micros);
        let ok = response.error.is_none() && response.ok;
        if ok {
            self.ok.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
        self.received.fetch_add(response.body_bytes as u64, Ordering::Relaxed);
        match (&response.error, response.cause) {
            (Some(_), cause) => {
                let cause = cause.and_then(|cause| serde_json::to_value(cause).ok()).and_then(|value| value.as_str().map(str::to_string)).unwrap_or_else(|| "failed".into());
                *self.causes.lock().unwrap().entry(cause).or_insert(0) += 1;
            }
            (None, _) => {
                self.statuses[(response.status as usize).min(599)].fetch_add(1, Ordering::Relaxed);
            }
        }
        let slot = &self.seconds[second];
        slot.answered.fetch_add(1, Ordering::Relaxed);
        slot.sum_us.fetch_add(micros, Ordering::Relaxed);
        if !ok {
            slot.failed.fetch_add(1, Ordering::Relaxed);
        }
        // Last, so whoever reads `sent` sees the rest of this request too.
        self.sent.fetch_add(1, Ordering::Release);
    }

    fn metrics(&self, planned: u64, ran: Duration) -> LoadMetrics {
        let sent = self.sent.load(Ordering::Acquire);
        let failed = self.failed.load(Ordering::Relaxed);
        let percentiles = self.latency.percentiles();
        let mut statuses: BTreeMap<String, u64> = self
            .statuses
            .iter()
            .enumerate()
            .filter_map(|(status, count)| Some((status.to_string(), count.load(Ordering::Relaxed))).filter(|(_, count)| *count > 0))
            .collect();
        statuses.extend(self.causes.lock().unwrap().iter().map(|(cause, count)| (cause.clone(), *count)));
        let counts = self.latency.coarse(LATENCY_BINS_MS);
        LoadMetrics {
            planned,
            sent,
            ok: self.ok.load(Ordering::Relaxed),
            failed,
            missed: self.missed.load(Ordering::Relaxed),
            duration_ms: ran.as_millis() as u64,
            rps: sent as f64 / ran.as_secs_f64().max(1e-6),
            error_rate: if sent == 0 { 0.0 } else { failed as f64 * 100.0 / sent as f64 },
            min_ms: self.latency.min_ms(),
            mean_ms: self.latency.mean_ms(),
            max_ms: self.latency.max_ms(),
            p50_ms: percentiles.p50_ms,
            p90_ms: percentiles.p90_ms,
            p95_ms: percentiles.p95_ms,
            p99_ms: percentiles.p99_ms,
            received_bytes: self.received.load(Ordering::Relaxed),
            statuses,
            seconds: self
                .seconds
                .iter()
                .map(|second| {
                    let answered = second.answered.load(Ordering::Relaxed);
                    let mean_ms = if answered == 0 { 0.0 } else { second.sum_us.load(Ordering::Relaxed) as f64 / answered as f64 / 1000.0 };
                    LoadSecond { sent: second.sent.load(Ordering::Relaxed), failed: second.failed.load(Ordering::Relaxed), mean_ms }
                })
                .collect(),
            histogram: counts
                .into_iter()
                .enumerate()
                .map(|(index, count)| LatencyBin { upto_ms: LATENCY_BINS_MS.get(index).copied(), count })
                .collect(),
            thresholds: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Running a load
// ---------------------------------------------------------------------------

/// How far a load has got, reported every `REPORT_EVERY`.
#[derive(Clone, Debug)]
pub struct Progress {
    pub sent: u64,
    pub failed: u64,
    /// Over the last report's window.
    pub rps: f64,
    pub p95_ms: f64,
    pub elapsed: Duration,
}

/// What a load uses of its run.
pub struct LoadEnv<'a> {
    pub host: &'a Host,
    pub seed: u64,
    pub node_id: &'a str,
    /// The run's cookie jar, when the experiment keeps cookies.
    pub cookies: Option<Arc<CookieJar>>,
    /// Set when the run stops — another branch failed: no more requests.
    pub stop: &'a AtomicBool,
}

/// `future`, reporting at every tick meanwhile; None once the run stops.
async fn reporting<T>(future: impl Future<Output = T>, tick: &mut Interval, report: &mut impl FnMut(), stop: &AtomicBool) -> Option<T> {
    if stop.load(Ordering::Relaxed) {
        return None;
    }
    tokio::pin!(future);
    loop {
        tokio::select! {
            biased;
            out = &mut future => return Some(out),
            _ = tick.tick() => {
                if stop.load(Ordering::Relaxed) {
                    return None;
                }
                report();
            }
        }
    }
}

/// The load of one step: `request` (its templates already read) sent as the
/// profile says, `concurrency` at a time, with the run's cookie jar and one
/// Digest memory; the Inspector gets a sample. A request still waiting for a
/// worker past its moment is skipped and counted as missed, never sent late.
/// After the last answer the thresholds are read into the metrics; the step
/// fails on them, not here. Stopping the run (the task aborted) drops the
/// requests in flight with it.
pub async fn run(env: LoadEnv<'_>, request: HttpRequest, load: &Load, mut progress: impl FnMut(Progress)) -> EngineResult<LoadMetrics> {
    let client = Arc::new(http::build_client(&request, env.cookies.clone()).map_err(|error| error.in_field(Field::new("url")))?);
    let mut schedule = Schedule::new(&load.profile, env.seed, env.node_id);
    let stats = Arc::new(Stats::new(schedule.seconds()));
    let memory = Arc::new(DigestMemory::default());
    let request = Arc::new(request);
    // A load can push thousands of requests a second: the Inspector gets a sample.
    let gate = Arc::new(Gate::new(100));
    let permits = Arc::new(Semaphore::new(load.concurrency.clamp(1, MAX_CONCURRENCY) as usize));
    let mut flights = JoinSet::new();

    let start = Instant::now();
    let mut tick = tokio::time::interval_at((start + REPORT_EVERY).into(), REPORT_EVERY);
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut window = (0u64, start);
    let reported = stats.clone();
    let mut report = move || {
        let sent = reported.sent.load(Ordering::Acquire);
        let now = Instant::now();
        let rps = (sent - window.0) as f64 / now.duration_since(window.1).as_secs_f64().max(1e-6);
        window = (sent, now);
        progress(Progress { sent, failed: reported.failed.load(Ordering::Relaxed), rps, p95_ms: reported.latency.percentile_ms(0.95), elapsed: start.elapsed() });
    };

    while let Some(due) = schedule.next() {
        let at = start + due;
        if reporting(tokio::time::sleep_until(at.into()), &mut tick, &mut report, env.stop).await.is_none() {
            break;
        }
        let Some(Ok(permit)) = reporting(permits.clone().acquire_owned(), &mut tick, &mut report, env.stop).await else { break };
        if Instant::now().saturating_duration_since(at) > http::MISS_AFTER {
            // Every worker was busy past the margin: what was due meanwhile is
            // missed, and the next request is the first still within it.
            let mut skipped = schedule.skip_before(start.elapsed().saturating_sub(http::MISS_AFTER));
            if skipped == 0 {
                schedule.advance();
                skipped = 1;
            }
            stats.missed.fetch_add(skipped, Ordering::Relaxed);
            continue;
        }
        let second = stats.second_of(due);
        stats.seconds[second].sent.fetch_add(1, Ordering::Relaxed);
        schedule.advance();
        let (client, request, stats, gate, host, memory) = (client.clone(), request.clone(), stats.clone(), gate.clone(), env.host.clone(), memory.clone());
        flights.spawn(async move {
            let response = http::execute(&client, &request, &memory).await;
            if inspect::armed(&host) && gate.allow() {
                inspect::publish(&host, gate.mark(http::exchange_frame(&request, &response, None)));
            }
            stats.record(&response, second);
            drop(permit);
        });
        while flights.try_join_next().is_some() {}
    }
    let paced = start.elapsed();
    // The answers still to come; a stop drops them.
    while let Some(Some(_)) = reporting(flights.join_next(), &mut tick, &mut report, env.stop).await {}
    flights.abort_all();

    let ran = if env.stop.load(Ordering::Relaxed) { paced } else { paced.max(schedule.duration()) };
    let mut metrics = stats.metrics(schedule.planned(), ran);
    judge(&load.thresholds, &mut metrics);
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn due(schedule: &mut Schedule, n: u64) -> f64 {
        while schedule.taken() < n {
            schedule.advance();
        }
        schedule.next().unwrap().as_secs_f64()
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn all(profile: LoadProfile) -> Vec<f64> {
        let mut schedule = Schedule::new(&profile, 1, "load");
        let mut moments = Vec::new();
        while let Some(at) = schedule.next() {
            moments.push(at.as_secs_f64());
            schedule.advance();
        }
        moments
    }

    #[test]
    fn each_shape_adds_up_to_its_requests_and_spaces_them_by_its_rate() {
        let constant = LoadProfile::Constant { rate: 100.0, duration_ms: 1000 };
        assert_eq!(constant.planned(), 100);
        let mut schedule = Schedule::new(&constant, 1, "load");
        assert_eq!(schedule.next(), Some(Duration::ZERO));
        assert!(close(due(&mut schedule, 50), 0.5) && close(due(&mut schedule, 99), 0.99));
        schedule.advance();
        assert_eq!(schedule.next(), None, "a second at 100/s is 100 requests");

        // 0 → 100/s over 2 s: the count is 25 t², so the n-th is due at √(n / 25).
        let ramp = LoadProfile::Ramp { from: 0.0, to: 100.0, duration_ms: 2000 };
        assert_eq!(ramp.planned(), 100);
        let mut schedule = Schedule::new(&ramp, 1, "load");
        assert!(close(due(&mut schedule, 25), 1.0) && close(due(&mut schedule, 99), (99.0f64 / 25.0).sqrt()));
        assert!(close(ramp.rate_at(Duration::from_millis(500)), 25.0));

        let down = all(LoadProfile::Ramp { from: 100.0, to: 0.0, duration_ms: 2000 });
        assert_eq!(down.len(), 100);
        assert!(down.windows(2).all(|pair| pair[0] < pair[1]) && *down.last().unwrap() < 2.0);
        assert!(down[1] - down[0] < down[99] - down[98], "the gaps widen as the rate falls");

        // 10, 20, 30/s for a second each.
        let steps = LoadProfile::Steps { from: 10.0, step: 10.0, every_ms: 1000, steps: 3 };
        assert_eq!((steps.planned(), steps.duration()), (60, Duration::from_secs(3)));
        let mut schedule = Schedule::new(&steps, 1, "load");
        assert!(close(due(&mut schedule, 10), 1.0) && close(due(&mut schedule, 30), 2.0) && close(due(&mut schedule, 45), 2.5));

        // 10/s, 100/s from 1 s for half a second, 10/s again.
        let spike = LoadProfile::Spike { base: 10.0, peak: 100.0, at_ms: 1000, spike_ms: 500, duration_ms: 2000 };
        assert_eq!(spike.planned(), 65);
        let mut schedule = Schedule::new(&spike, 1, "load");
        assert!(close(due(&mut schedule, 10), 1.0) && close(due(&mut schedule, 60), 1.5) && close(due(&mut schedule, 64), 1.9));
        assert_eq!([0.5, 1.2, 1.7].map(|s| spike.rate_at(Duration::from_secs_f64(s))), [10.0, 100.0, 10.0]);

        // A rest at nothing in between: nothing due there.
        let rest = all(LoadProfile::Steps { from: 10.0, step: -10.0, every_ms: 1000, steps: 2 });
        assert_eq!(rest.len(), 10);
        assert!(rest.iter().all(|at| *at < 1.0));
    }

    #[test]
    fn what_falls_behind_is_skipped_up_to_the_moment_given() {
        let mut schedule = Schedule::new(&LoadProfile::Constant { rate: 100.0, duration_ms: 1000 }, 1, "load");
        schedule.advance();
        assert_eq!(schedule.skip_before(Duration::from_millis(500)), 49, "1…49 were due before 500 ms");
        assert!(close(schedule.next().unwrap().as_secs_f64(), 0.5));
        assert_eq!(schedule.skip_before(Duration::from_millis(300)), 0, "nothing again");
        assert_eq!(schedule.skip_before(Duration::from_secs(5)), 50, "never past the end");
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn poisson_arrivals_average_their_rate_and_repeat_for_the_same_seed() {
        let profile = LoadProfile::Poisson { rate: 200.0, duration_ms: 10_000 };
        let moments = all(profile.clone());
        assert!((1800..=2200).contains(&moments.len()), "{} arrivals, 2000 expected", moments.len());
        assert!(moments.windows(2).all(|pair| pair[0] <= pair[1]) && moments[0] > 0.0);
        let gaps: Vec<f64> = moments.windows(2).map(|pair| pair[1] - pair[0]).collect();
        assert!(gaps.iter().any(|gap| *gap > 0.015) && gaps.iter().any(|gap| *gap < 0.001), "random gaps, not even ones");
        assert_eq!(moments, all(profile.clone()), "the same seed, the same moments");
        let mut other = Schedule::new(&profile, 2, "load");
        assert_ne!(other.next().unwrap().as_secs_f64(), moments[0], "another seed, others");
        assert_eq!(profile.planned(), 2000);
        let skipped = other.skip_before(Duration::from_secs(1));
        assert!((150..=250).contains(&skipped), "{skipped}");
    }

    fn load(profile: Value) -> Load {
        serde_json::from_value(serde_json::json!({ "profile": profile })).unwrap()
    }

    #[test]
    fn a_load_that_cannot_run_as_written_is_refused() {
        let refused = |profile: Value| check(&load(profile)).unwrap_err();
        let field = |error: EngineError| (error.code.clone(), error.field.as_ref().map(|field| field.key.clone()).unwrap_or_default());
        assert!(check(&load(serde_json::json!({ "shape": "ramp", "from": 0, "to": 200, "duration_ms": 60_000 }))).is_ok());
        assert_eq!(load(serde_json::json!({ "shape": "constant", "rate": 5, "duration_ms": 1000 })).concurrency, 32);
        let cases = [
            (serde_json::json!({ "shape": "constant", "rate": 0.05, "duration_ms": 1000 }), "node.range", "load_rate"),
            (serde_json::json!({ "shape": "poisson", "rate": 100_001, "duration_ms": 1000 }), "node.range", "load_rate"),
            (serde_json::json!({ "shape": "constant", "rate": 10, "duration_ms": 300_001 }), "node.range", "load_duration"),
            (serde_json::json!({ "shape": "constant", "rate": 10, "duration_ms": 50 }), "node.range", "load_duration"),
            (serde_json::json!({ "shape": "ramp", "from": -1, "to": 10, "duration_ms": 1000 }), "node.range", "load_from"),
            (serde_json::json!({ "shape": "ramp", "from": 0, "to": 0, "duration_ms": 1000 }), "load.nothing_planned", "load"),
            (serde_json::json!({ "shape": "steps", "from": 10, "step": 10, "every_ms": 1000, "steps": 0 }), "node.range", "load_steps"),
            (serde_json::json!({ "shape": "steps", "from": 10, "step": -10, "every_ms": 1000, "steps": 3 }), "load.steps_out_of_range", "load_step"),
            (serde_json::json!({ "shape": "steps", "from": 10, "step": 10, "every_ms": 60_000, "steps": 6 }), "load.too_long", "load_steps"),
            (serde_json::json!({ "shape": "spike", "base": 10, "peak": 100, "at_ms": 800, "spike_ms": 500, "duration_ms": 1000 }), "load.spike_outside", "load_spike"),
            (serde_json::json!({ "shape": "spike", "base": 10, "peak": 0, "at_ms": 0, "spike_ms": 500, "duration_ms": 1000 }), "node.range", "load_peak"),
        ];
        for (profile, code, key) in cases {
            assert_eq!(field(refused(profile.clone())), (code.to_string(), key.to_string()), "{profile}");
        }
        let mut wide = load(serde_json::json!({ "shape": "constant", "rate": 10, "duration_ms": 1000 }));
        wide.concurrency = 513;
        assert_eq!(field(check(&wide).unwrap_err()), ("node.range".to_string(), "load_concurrency".to_string()));
        wide.concurrency = 1;
        wide.thresholds = vec![Threshold { metric: Metric::P95Ms, op: ThresholdOp::Lt, value: 300.0 }, Threshold { metric: Metric::Rps, op: ThresholdOp::Ge, value: f64::NAN }];
        let value = check(&wide).unwrap_err();
        assert_eq!((value.code.as_str(), value.field.as_ref().and_then(|field| field.index)), ("load.threshold_value", Some(2)));
    }

    #[test]
    fn thresholds_read_their_metric_and_name_it_as_a_document_does() {
        let mut metrics = LoadMetrics { p95_ms: 412.345, error_rate: 0.4, rps: 180.0, missed: 3, ..Default::default() };
        let thresholds: Vec<Threshold> = serde_json::from_value(serde_json::json!([
            { "metric": "p95_ms", "op": "lt", "value": 300 },
            { "metric": "error_rate", "op": "lt", "value": 1 },
            { "metric": "rps", "op": "ge", "value": 180 },
            { "metric": "missed", "op": "le", "value": 0 }
        ]))
        .unwrap();
        judge(&thresholds, &mut metrics);
        assert_eq!(metrics.thresholds.iter().map(|verdict| verdict.held).collect::<Vec<_>>(), [false, true, true, false]);
        let error = metrics.thresholds[0].error(0);
        assert_eq!((error.code.as_str(), error.params["metric"].as_str(), error.params["op"].as_str(), error.params["value"].as_str(), error.params["actual"].as_str()), ("load.threshold", "p95_ms", "<", "300", "412.35"));
        for metric in [Metric::P50Ms, Metric::P90Ms, Metric::P95Ms, Metric::P99Ms, Metric::MeanMs, Metric::MaxMs, Metric::ErrorRate, Metric::Rps, Metric::Missed] {
            assert_eq!(serde_json::to_value(metric).unwrap(), metric.code());
        }
        assert_eq!([shown(2.0), shown(0.5), shown(12.345), shown(0.0)], ["2", "0.5", "12.35", "0"]);
    }

    #[test]
    fn statuses_causes_seconds_and_the_histogram_add_up() {
        let stats = Stats::new(2);
        let response = |status: u16, ms: f64| HttpResponse {
            ok: (200..300).contains(&status),
            status,
            status_text: String::new(),
            latency_ms: ms,
            headers: Vec::new(),
            body: String::new(),
            body_bytes: 10,
            truncated: false,
            error: None,
            cause: None,
            digest: None,
        };
        stats.record(&response(200, 4.0), 0);
        stats.record(&response(200, 30.0), 1);
        stats.record(&response(503, 3.0), 1);
        stats.record(&HttpResponse { error: Some("timed out".into()), cause: Some(crate::transport::Cause::Timeout), status: 0, ..response(0, 4000.0) }, 1);
        let metrics = stats.metrics(4, Duration::from_secs(2));
        assert_eq!((metrics.sent, metrics.ok, metrics.failed, metrics.error_rate, metrics.rps, metrics.received_bytes), (4, 2, 2, 50.0, 2.0, 40));
        assert_eq!(metrics.statuses, BTreeMap::from([("200".to_string(), 2), ("503".to_string(), 1), ("timeout".to_string(), 1)]));
        assert_eq!(metrics.seconds.iter().map(|second| (second.failed, second.mean_ms.round() as u64)).collect::<Vec<_>>(), [(0, 4), (2, 1344)]);
        let bins: Vec<(Option<f64>, u64)> = metrics.histogram.iter().filter(|bin| bin.count > 0).map(|bin| (bin.upto_ms, bin.count)).collect();
        assert_eq!(bins, [(Some(5.0), 2), (Some(50.0), 1), (Some(5000.0), 1)]);
        assert_eq!(metrics.histogram.len(), LATENCY_BINS_MS.len() + 1);
    }
}
