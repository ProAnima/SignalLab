import { useEffect, useRef, useState } from "react";
import { api, type Comparison, type Experiment, type LoadMetric, type RunSummary } from "../lib/api";
import type { Failure } from "../lib/errors";
import { fmtTime } from "../lib/format";
import { useT, type TKey } from "../lib/i18n";
import { nodeLabel } from "../lib/experimentText";
import { OP_SYMBOL } from "../lib/load";
import { ErrorMessage } from "./ErrorMessage";
import { useMetricValue } from "./ExperimentLoad";

/** A run in the pickers: when it started, how it ended, its seed. */
function runLabel(run: RunSummary, t: ReturnType<typeof useT>) {
  const day = new Date(run.started_ms).toLocaleDateString();
  return `${day} ${fmtTime(run.started_ms).slice(0, 8)} · ${run.outcome === "passed" ? "✓" : "✕"} ${t(run.outcome === "passed" ? "exp.passed" : "exp.failed")} · ${t("exp.runSeed", { seed: run.seed })}`;
}

/**
 * Two runs of this experiment side by side, load step by load step: each
 * metric before and after, the change, a regression in red, and how each
 * threshold went in either run. The latest run against the one before it,
 * unless another is picked.
 */
export function ExperimentCompare({ doc, latest, onClose }: { doc: Experiment; latest: string; onClose: () => void }) {
  const t = useT();
  const value = useMetricValue();
  const dialog = useRef<HTMLDialogElement>(null);
  const [runs, setRuns] = useState<RunSummary[] | null>(null);
  const [a, setA] = useState("");
  const [b, setB] = useState("");
  const [comparison, setComparison] = useState<Comparison | null>(null);
  const [error, setError] = useState<Failure | null>(null);

  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);

  useEffect(() => {
    let live = true;
    api.experimentRuns(doc.name, 50).then((found) => {
      if (!live) return;
      setRuns(found);
      const after = found.findIndex((run) => run.name === latest);
      const at = after >= 0 ? after : 0;
      setB(found[at]?.name ?? "");
      setA(found[at + 1]?.name ?? "");
    }, (reason) => live && setError(reason ?? "?"));
    return () => { live = false; };
  }, [doc.name, latest]);

  useEffect(() => {
    if (!a || !b || a === b) { setComparison(null); return; }
    let live = true;
    setError(null);
    api.experimentCompare(a, b).then((found) => live && setComparison(found), (reason) => live && setError(reason ?? "?"));
    return () => { live = false; };
  }, [a, b]);

  const label = (id: string) => {
    const node = doc.nodes.find((item) => item.id === id);
    return node ? nodeLabel(node.type, t) : id;
  };
  const changeText = (metric: LoadMetric, change: number, percent: number | null) => {
    const sign = change > 0 ? "+" : change < 0 ? "−" : "±";
    return `${sign}${value(metric, Math.abs(change))}${percent === null ? "" : ` (${sign}${Math.abs(Math.round(percent))} %)`}`;
  };
  const picker = (key: TKey, picked: string, onPick: (name: string) => void) => <label>{t(key)}
    <select value={picked} onChange={(event) => onPick(event.target.value)}>
      {(runs ?? []).map((run) => <option key={run.name} value={run.name}>{runLabel(run, t)}</option>)}
    </select></label>;

  return <dialog ref={dialog} className="experiment-documents experiment-compare" role="dialog" aria-labelledby="experiment-compare-title"
    onCancel={(event) => { event.preventDefault(); onClose(); }} onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <div className="experiment-documents-content">
      <header><div><h2 id="experiment-compare-title" data-tip={t("exp.compareHint")}>{t("exp.compare")}</h2></div>
        <button className="ghost sm" aria-label={t("exp.close")} data-tip={t("common.closeHint")} onClick={onClose}>×</button>
      </header>
      {runs && runs.length < 2 ? <p className="experiment-empty">{t("exp.compareNone")}</p> : <div className="experiment-compare-pick">
        {picker("exp.compareBefore", a, setA)}
        {picker("exp.compareAfter", b, setB)}
      </div>}
      {error !== null && <ErrorMessage className="experiment-file-error" error={error} />}
      {comparison && comparison.steps.length === 0 && <p className="experiment-empty">{t("exp.compareNoLoad")}</p>}
      {comparison?.steps.map((step) => <section key={step.node} className="experiment-compare-step">
        <h3>{label(step.node)}{step.missing_in && <small> · {t(step.missing_in === "a" ? "exp.compareOnlyAfter" : "exp.compareOnlyBefore")}</small>}</h3>
        <table>
          <thead><tr><th>{t("exp.thresholdMetric")}</th><th>{t("exp.compareBefore")}</th><th>{t("exp.compareAfter")}</th><th>{t("exp.compareChange")}</th></tr></thead>
          <tbody>{step.metrics.map((row) => <tr key={row.metric} className={row.worse ? "worse" : ""}>
            <th>{t(`exp.metric.${row.metric}` as TKey)}</th>
            <td>{value(row.metric, row.a)}</td>
            <td>{value(row.metric, row.b)}</td>
            <td>{step.missing_in ? "—" : changeText(row.metric, row.change, row.percent)}</td>
          </tr>)}</tbody>
        </table>
        {(step.thresholds_a.length > 0 || step.thresholds_b.length > 0) && <ul className="experiment-verdicts">
          {(step.thresholds_b.length ? step.thresholds_b : step.thresholds_a).map((verdict, index) => {
            const before = step.thresholds_a[index];
            const after = step.thresholds_b[index];
            const mark = (held: boolean | undefined) => held === undefined ? "—" : held ? "✓" : "✕";
            return <li key={index} className={after && !after.held ? "broken" : "held"}>
              <span />
              <span>{t(`exp.metric.${verdict.metric}` as TKey)} {OP_SYMBOL[verdict.op]} {value(verdict.metric, verdict.value)}</span>
              <b aria-label={`${t("exp.compareBefore")} ${before ? t(before.held ? "exp.thresholdHeld" : "exp.thresholdBroken") : "—"}, ${t("exp.compareAfter")} ${after ? t(after.held ? "exp.thresholdHeld" : "exp.thresholdBroken") : "—"}`}>{mark(before?.held)} → {mark(after?.held)}</b>
            </li>;
          })}</ul>}
      </section>)}
    </div>
  </dialog>;
}
