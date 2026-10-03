import type { ExperimentNode, Load, LoadMetric, LoadMetrics, LoadProfile, LoadShape, Threshold, ThresholdOp } from "../lib/api";
import { fmtNum, fmtRate, latencyParts } from "../lib/format";
import { useT, type TKey } from "../lib/i18n";
import { DEFAULT_LOAD, durationMs, LOAD_METRICS, LOAD_SHAPES, MAX_CONCURRENCY, MAX_THRESHOLDS, METRIC_UNIT, OP_SYMBOL, peakRate, planned, ratePoints, THRESHOLD_OPS, withShape } from "../lib/load";

type Patch = (change: Partial<ExperimentNode>) => void;

/** A metric's unit as the language writes it. */
function useUnit() {
  const t = useT();
  return (metric: LoadMetric) => {
    const unit = METRIC_UNIT[metric];
    return unit === "ms" ? t("unit.ms") : unit === "%" ? "%" : unit === "/s" ? t("unit.perSecond") : "";
  };
}

/** A percentage as precise as it is small: 0.04 %, 2.7 %, 35 %. */
const percent = (value: number) => fmtNum(value, Number.isInteger(value) ? 0 : value < 1 ? 2 : value < 10 ? 1 : 0);

/** A metric's value as the language writes it, with its unit: `212 ms`, `1.20 s`, `0.4 %`, `180/s`, `3`. */
export function useMetricValue() {
  const t = useT();
  return (metric: LoadMetric, value: number) => {
    switch (METRIC_UNIT[metric]) {
      case "ms": { const [shown, unit] = latencyParts(value); return `${shown} ${t(unit)}`; }
      case "%": return `${percent(value)} %`;
      case "/s": return `${fmtRate(Math.round(value * 10) / 10)}${t("unit.perSecond")}`;
      default: return fmtNum(value);
    }
  };
}

/**
 * Load on an HTTP node: the request sent on a profile, many at once, measured
 * and judged by thresholds. Turned on, it replaces Repeat and Retry.
 */
export function LoadFields({ load, patch }: { load: Load | undefined; patch: Patch }) {
  const t = useT();
  const change = (next: Partial<Load>) => load && patch({ load: { ...load, ...next } });
  const profile = (next: Partial<LoadProfile>) => load && change({ profile: { ...load.profile, ...next } as LoadProfile });
  const number = (value: string) => Number(value);
  const field = (label: TKey, value: number, onValue: (value: number) => void, attrs: { min?: number; max?: number; step?: number; tip?: TKey } = {}) =>
    <label data-tip={attrs.tip && t(attrs.tip)}>{t(label)}<input type="number" value={value} min={attrs.min} max={attrs.max} step={attrs.step} onChange={(event) => onValue(number(event.target.value))} /></label>;
  const p = load?.profile;
  return <div className="experiment-option">
    <label className="checkbox experiment-check" data-tip={t("exp.loadHint")}>
      <input type="checkbox" checked={!!load} onChange={(event) => patch(event.target.checked
        ? { load: structuredClone(DEFAULT_LOAD), repeat: undefined, retry: undefined }
        : { load: undefined })} />{t("exp.loadOn")}
    </label>
    {load && p && <div className="experiment-option-fields experiment-load-fields">
      <label data-tip={t("exp.loadShapeHint")}>{t("exp.loadShape")}<select value={p.shape} onChange={(event) => change({ profile: withShape(p, event.target.value as LoadShape) })}>
        {LOAD_SHAPES.map((shape) => <option key={shape} value={shape}>{t(`exp.loadShape.${shape}` as TKey)}</option>)}</select></label>
      {field("exp.loadConcurrency", load.concurrency, (concurrency) => change({ concurrency }), { min: 1, max: MAX_CONCURRENCY, tip: "exp.loadConcurrencyHint" })}
      {(p.shape === "constant" || p.shape === "poisson") && <>
        {field("exp.loadRate", p.rate, (rate) => profile({ rate }), { min: 0.1, step: 1 })}
        {field("exp.loadDuration", p.duration_ms, (duration_ms) => profile({ duration_ms }), { min: 100, step: 1000 })}
      </>}
      {p.shape === "ramp" && <>
        {field("exp.loadFrom", p.from, (from) => profile({ from }), { min: 0, step: 1 })}
        {field("exp.loadTo", p.to, (to) => profile({ to }), { min: 0, step: 1 })}
        {field("exp.loadDuration", p.duration_ms, (duration_ms) => profile({ duration_ms }), { min: 100, step: 1000 })}
      </>}
      {p.shape === "steps" && <>
        {field("exp.loadFrom", p.from, (from) => profile({ from }), { min: 0, step: 1 })}
        {field("exp.loadStepBy", p.step, (step) => profile({ step }), { step: 1 })}
        {field("exp.loadEvery", p.every_ms, (every_ms) => profile({ every_ms }), { min: 100, step: 1000 })}
        {field("exp.loadSteps", p.steps, (steps) => profile({ steps }), { min: 1, max: 100 })}
      </>}
      {p.shape === "spike" && <>
        {field("exp.loadBase", p.base, (base) => profile({ base }), { min: 0, step: 1 })}
        {field("exp.loadPeak", p.peak, (peak) => profile({ peak }), { min: 0.1, step: 1 })}
        {field("exp.loadAt", p.at_ms, (at_ms) => profile({ at_ms }), { min: 0, step: 1000 })}
        {field("exp.loadSpikeFor", p.spike_ms, (spike_ms) => profile({ spike_ms }), { min: 1, step: 1000 })}
        {field("exp.loadDuration", p.duration_ms, (duration_ms) => profile({ duration_ms }), { min: 100, step: 1000 })}
      </>}
    </div>}
    {load && p && <LoadChart profile={p} />}
    {load && <Thresholds thresholds={load.thresholds} onChange={(thresholds) => change({ thresholds })} />}
  </div>;
}

/** The profile's rate over time, and the requests it adds up to. */
function LoadChart({ profile }: { profile: LoadProfile }) {
  const t = useT();
  const points = ratePoints(profile);
  const seconds = durationMs(profile) / 1000;
  const top = peakRate(profile) || 1;
  const [w, h] = [200, 48];
  const x = (s: number) => seconds > 0 ? (s / seconds) * w : 0;
  const y = (rate: number) => h - (rate / top) * (h - 4);
  const line = points.map(([s, rate]) => `${x(s).toFixed(1)},${y(rate).toFixed(1)}`).join(" ");
  const label = t("exp.loadPlanned", { approx: String(profile.shape === "poisson"), n: planned(profile), s: fmtNum(seconds, Number.isInteger(seconds) ? 0 : 1) });
  return <figure className="experiment-load-chart">
    <svg viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" role="img" aria-label={`${t("exp.loadChart")}: ${label}`}>
      {points.length > 0 && <polygon className="area" points={`0,${h} ${line} ${w},${h}`} />}
      {points.length > 0 && <polyline className={profile.shape === "poisson" ? "line random" : "line"} points={line} />}
    </svg>
    <figcaption><span>{fmtRate(Math.round(top * 10) / 10)}{t("unit.perSecond")}</span><span>{label}</span></figcaption>
  </figure>;
}

/** Rows of `metric op value`; the step fails on the first that does not hold. */
function Thresholds({ thresholds, onChange }: { thresholds: Threshold[]; onChange: (thresholds: Threshold[]) => void }) {
  const t = useT();
  const unit = useUnit();
  const set = (index: number, next: Partial<Threshold>) => onChange(thresholds.map((threshold, at) => at === index ? { ...threshold, ...next } : threshold));
  return <div className="experiment-thresholds">
    <p className="section-label" data-tip={t("exp.thresholdsHint")}>{t("exp.thresholds")}</p>
    {thresholds.map((threshold, index) => <div className="experiment-threshold" key={index}>
      <select aria-label={t("exp.thresholdMetric")} value={threshold.metric} onChange={(event) => set(index, { metric: event.target.value as LoadMetric })}>
        {LOAD_METRICS.map((metric) => <option key={metric} value={metric}>{t(`exp.metric.${metric}` as TKey)}</option>)}</select>
      <select aria-label={t("exp.thresholdOp")} value={threshold.op} onChange={(event) => set(index, { op: event.target.value as ThresholdOp })}>
        {THRESHOLD_OPS.map((op) => <option key={op} value={op}>{OP_SYMBOL[op]}</option>)}</select>
      <input type="number" aria-label={t("exp.thresholdValue")} min={0} value={threshold.value} onChange={(event) => set(index, { value: Number(event.target.value) })} />
      <span className="experiment-threshold-unit">{unit(threshold.metric)}</span>
      <button className="ghost sm" aria-label={t("exp.thresholdRemove")} data-tip={t("exp.thresholdRemove")} onClick={() => onChange(thresholds.filter((_, at) => at !== index))}>×</button>
    </div>)}
    {thresholds.length < MAX_THRESHOLDS && <button className="ghost sm" onClick={() => onChange([...thresholds, { metric: "p95_ms", op: "lt", value: 500 }])}>＋ {t("exp.thresholdAdd")}</button>}
  </div>;
}

/** What the last run's load on this node measured. */
export function LoadResult({ metrics }: { metrics: LoadMetrics }) {
  const t = useT();
  const value = useMetricValue();
  const metric = (key: TKey, value: string, className = "", more?: string) => <div className="metric"><div className="k">{t(key)}</div><div className={`v ${className}`}>{value}{more && <small>{more}</small>}</div></div>;
  const latency = (key: TKey, ms: number) => { const [shown, unit] = latencyParts(ms); return metric(key, shown, "", t(unit)); };
  const statuses = Object.entries(metrics.statuses);
  return <div className="experiment-load-result">
    <p className="section-label">{t("exp.loadResult")}</p>
    {metrics.thresholds.length > 0 && <ul className="experiment-verdicts">{metrics.thresholds.map((verdict, index) =>
      <li key={index} className={verdict.held ? "held" : "broken"}>
        <span role="img" aria-label={t(verdict.held ? "exp.thresholdHeld" : "exp.thresholdBroken")}>{verdict.held ? "✓" : "✕"}</span>
        <span>{t(`exp.metric.${verdict.metric}` as TKey)} {OP_SYMBOL[verdict.op]} {value(verdict.metric, verdict.value)}</span>
        <b>{value(verdict.metric, verdict.actual)}</b>
      </li>)}</ul>}
    <div className="metrics experiment-load-metrics">
      {metric("http.sent", fmtNum(metrics.sent))}
      {metric("exp.loadRps", fmtRate(Math.round(metrics.rps * 10) / 10), "accent")}
      {metric("exp.loadErrors", fmtNum(metrics.failed), metrics.failed ? "red" : "", `${percent(metrics.error_rate)} %`)}
      {metric("http.missed", fmtNum(metrics.missed), metrics.missed ? "amber" : "")}
    </div>
    <div className="metrics latency experiment-load-metrics">
      {latency("http.p50", metrics.p50_ms)}
      {latency("http.p90", metrics.p90_ms)}
      {latency("http.p95", metrics.p95_ms)}
      {latency("http.p99", metrics.p99_ms)}
      {latency("http.avg", metrics.mean_ms)}
      {latency("http.max", metrics.max_ms)}
    </div>
    {metrics.seconds.length > 0 && <PerSecond metrics={metrics} />}
    {metrics.histogram.some((bin) => bin.count > 0) && <Histogram metrics={metrics} />}
    {statuses.length > 0 && <div className="experiment-load-statuses" aria-label={t("exp.loadStatuses")}>
      {statuses.map(([status, count]) => <span key={status} className={/^2/.test(status) ? "ok" : "fail"}>{status} × {fmtNum(count)}</span>)}</div>}
  </div>;
}

/** Each second: what was sent (failures in red) and how long it took on average. */
function PerSecond({ metrics }: { metrics: LoadMetrics }) {
  const t = useT();
  const [w, h] = [200, 48];
  const top = Math.max(1, ...metrics.seconds.map((second) => second.sent));
  const slowest = Math.max(1, ...metrics.seconds.map((second) => second.mean_ms));
  const [slowestShown, slowestUnit] = latencyParts(slowest);
  const bar = w / metrics.seconds.length;
  const line = metrics.seconds.map((second, index) => `${((index + 0.5) * bar).toFixed(1)},${(h - (second.mean_ms / slowest) * (h - 4)).toFixed(1)}`).join(" ");
  return <figure className="experiment-load-chart">
    <svg viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" role="img" aria-label={t("exp.loadPerSecond")}>
      {metrics.seconds.map((second, index) => <g key={index}>
        <rect className="sent" x={index * bar + 0.5} width={Math.max(0.5, bar - 1)} y={h - (second.sent / top) * (h - 4)} height={(second.sent / top) * (h - 4)} />
        {second.failed > 0 && <rect className="failed" x={index * bar + 0.5} width={Math.max(0.5, bar - 1)} y={h - (second.failed / top) * (h - 4)} height={(second.failed / top) * (h - 4)} />}
      </g>)}
      <polyline className="latency" points={line} />
    </svg>
    <figcaption><span>{t("exp.loadPerSecond")}</span><span>{fmtNum(top)}{t("unit.perSecond")} · {slowestShown} {t(slowestUnit)}</span></figcaption>
  </figure>;
}

/** How many took how long, in coarse bins. */
function Histogram({ metrics }: { metrics: LoadMetrics }) {
  const t = useT();
  const bins = metrics.histogram;
  const first = bins.findIndex((bin) => bin.count > 0);
  const last = bins.length - 1 - [...bins].reverse().findIndex((bin) => bin.count > 0);
  const shown = bins.slice(first, last + 1);
  const top = Math.max(1, ...shown.map((bin) => bin.count));
  // The bars' labels are their upper bounds (the tooltip says ≤); the last bin, anything slower.
  const bound = (upto: number | null) => upto === null ? `>${fmtNum(bins[bins.length - 2]?.upto_ms ?? 0)}` : fmtNum(upto);
  return <figure className="experiment-load-histogram" aria-label={t("exp.loadLatencies")}>
    {shown.map((bin) => <div key={String(bin.upto_ms)} className="experiment-load-bin" data-tip={`${bin.upto_ms === null ? "" : "≤"}${bound(bin.upto_ms)} ${t("unit.ms")}: ${fmtNum(bin.count)}`}>
      <span className="bar" style={{ height: `${Math.max(2, (bin.count / top) * 100)}%` }} />
      <small>{bound(bin.upto_ms)}</small>
    </div>)}
    <figcaption>{t("exp.loadLatencies")}, {t("unit.ms")}</figcaption>
  </figure>;
}
