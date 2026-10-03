/**
 * Load on an HTTP node (engine/src/load.rs): the profiles, what they add up
 * to, the points of their rate over time, their notation. Pure — the editor's
 * chart and badge, and the tests, read it; the engine schedules the requests.
 */
import type { Load, LoadMetric, LoadMetrics, LoadProfile, LoadShape, ThresholdOp } from "./api";

export const LOAD_SHAPES: LoadShape[] = ["constant", "ramp", "steps", "spike", "poisson"];
export const LOAD_METRICS: LoadMetric[] = ["p95_ms", "p99_ms", "p90_ms", "p50_ms", "mean_ms", "max_ms", "error_rate", "rps", "missed"];
export const THRESHOLD_OPS: ThresholdOp[] = ["lt", "le", "gt", "ge"];
export const OP_SYMBOL: Record<ThresholdOp, string> = { lt: "<", le: "≤", gt: ">", ge: "≥" };
/** What a metric is read in: milliseconds, a percentage, per second, or a count. */
export const METRIC_UNIT: Record<LoadMetric, "ms" | "%" | "/s" | ""> = {
  p50_ms: "ms", p90_ms: "ms", p95_ms: "ms", p99_ms: "ms", mean_ms: "ms", max_ms: "ms", error_rate: "%", rps: "/s", missed: "",
};
/** The engine's bounds (`load::check`, the burst's rates). */
export const MIN_RATE = 0.1;
export const MAX_RATE = 100_000;
export const MAX_CONCURRENCY = 512;
export const MAX_STEPS = 100;
export const MIN_DURATION_MS = 100;
export const MAX_DURATION_MS = 300_000;
export const MAX_THRESHOLDS = 16;

/** Load as an HTTP node first gets it: a 30-second ramp to 100/s, judged by p95 and errors. */
export const DEFAULT_LOAD: Load = {
  profile: { shape: "ramp", from: 0, to: 100, duration_ms: 30_000 },
  concurrency: 32,
  thresholds: [{ metric: "p95_ms", op: "lt", value: 500 }, { metric: "error_rate", op: "lt", value: 1 }],
};

/** How long a profile runs, in milliseconds. */
export function durationMs(profile: LoadProfile): number {
  return profile.shape === "steps" ? profile.every_ms * profile.steps : profile.duration_ms;
}

/** Straight pieces of the rate over time: from `start` (s) for `len` s, `r0` → `r1` per second. */
interface Piece { start: number; len: number; r0: number; r1: number }

function pieces(profile: LoadProfile): Piece[] {
  const s = (ms: number) => ms / 1000;
  const raw: [number, number, number][] = (() => {
    switch (profile.shape) {
      case "constant": case "poisson": return [[s(profile.duration_ms), profile.rate, profile.rate]];
      case "ramp": return [[s(profile.duration_ms), profile.from, profile.to]];
      case "steps": return Array.from({ length: Math.max(0, profile.steps) }, (_, k): [number, number, number] => {
        const rate = profile.from + profile.step * k;
        return [s(profile.every_ms), rate, rate];
      });
      case "spike": return [
        [s(profile.at_ms), profile.base, profile.base],
        [s(profile.spike_ms), profile.peak, profile.peak],
        [s(Math.max(0, profile.duration_ms - profile.at_ms - profile.spike_ms)), profile.base, profile.base],
      ];
    }
  })();
  let start = 0;
  return raw.filter(([len]) => len > 0).map(([len, r0, r1]) => { const piece = { start, len, r0, r1 }; start += len; return piece; });
}

/** The requests its rate adds up to, as a fraction. */
export function expected(profile: LoadProfile): number {
  return pieces(profile).reduce((sum, piece) => sum + (piece.r0 + piece.r1) / 2 * piece.len, 0);
}

/** The requests a profile sends (a Poisson one: on average) — as the engine counts them. */
export function planned(profile: LoadProfile): number {
  const total = expected(profile);
  return profile.shape === "poisson" ? Math.round(total) : Math.max(0, Math.ceil(total - 1e-9));
}

/** The rate `t` seconds in, per second. */
export function rateAt(profile: LoadProfile, t: number): number {
  const piece = pieces(profile).find((item) => t < item.start + item.len);
  return piece ? piece.r0 + (piece.r1 - piece.r0) * Math.min(1, Math.max(0, (t - piece.start) / piece.len)) : 0;
}

/** The rate's corners over time, `[seconds, per second]`: a line through them is the profile. */
export function ratePoints(profile: LoadProfile): [number, number][] {
  return pieces(profile).flatMap((piece): [number, number][] => [[piece.start, piece.r0], [piece.start + piece.len, piece.r1]]);
}

/** The highest rate it asks for. */
export function peakRate(profile: LoadProfile): number {
  return Math.max(0, ...ratePoints(profile).map(([, rate]) => rate));
}

/** A rate or a time as notation: at most one decimal. */
export function short(value: number): string {
  return String(Math.round(value * 10) / 10);
}

/** The node's badge: the highest rate, `~` before a random one's average. */
export function loadBadge(profile: LoadProfile): string {
  return `${profile.shape === "poisson" ? "~" : ""}${short(peakRate(profile))}/s`;
}

/** The rates a profile goes through: `10→200/s`, `40/s`, `~40/s`, `10↑100/s`. */
export function loadNotation(profile: LoadProfile): string {
  switch (profile.shape) {
    case "constant": return `${short(profile.rate)}/s`;
    case "poisson": return `~${short(profile.rate)}/s`;
    case "ramp": return `${short(profile.from)}→${short(profile.to)}/s`;
    case "steps": return `${short(profile.from)}→${short(profile.from + profile.step * (profile.steps - 1))}/s`;
    case "spike": return `${short(profile.base)}↑${short(profile.peak)}/s`;
  }
}

const round = (value: number) => Math.max(MIN_RATE, Math.round(value * 10) / 10);

/**
 * The profile in another shape, keeping what carries over: how long it runs
 * and the rate it reaches.
 */
export function withShape(profile: LoadProfile, shape: LoadShape): LoadProfile {
  if (profile.shape === shape) return profile;
  const duration = durationMs(profile);
  const top = round(peakRate(profile) || 10);
  switch (shape) {
    case "constant": case "poisson": return { shape, rate: top, duration_ms: duration };
    case "ramp": return { shape, from: 0, to: top, duration_ms: duration };
    case "steps": {
      const steps = 5;
      return { shape, from: round(top / steps), step: round(top / steps), every_ms: Math.max(MIN_DURATION_MS, Math.round(duration / steps)), steps };
    }
    case "spike": return { shape, base: round(top / 5), peak: top, at_ms: Math.round(duration * 0.4), spike_ms: Math.max(1, Math.round(duration * 0.2)), duration_ms: duration };
  }
}

/** A metric of what a load measured. */
export function metricOf(metrics: LoadMetrics, metric: LoadMetric): number {
  return metric === "missed" ? metrics.missed : metrics[metric];
}
