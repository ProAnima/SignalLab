import type { Experiment, ExperimentNode, ExperimentStep } from "../lib/api";
import type { Failure } from "../lib/errors";
import { nodeLabel, stepText } from "../lib/experimentText";
import type { Outcome, RunSetup } from "../lib/useExperimentRun";
import { fmtTime } from "../lib/format";
import { useT } from "../lib/i18n";
import { downloadUrl } from "../lib/platform";
import { Splitter } from "./Splitter";

/**
 * The run's pane under the canvas: how the run went (report, profile, seed),
 * one row per step, and the Inspector frames the waits matched.
 */
export function ExperimentTimeline({ doc, events, outcome, running, reportPath, lastRun, lastSeed, busy, open, onToggle, height, initialHeight, maxHeight, onHeight, describe, onEdit, onShowNode, onShowFrame }: {
  doc: Experiment;
  events: ExperimentStep[];
  outcome: Outcome;
  running: boolean;
  reportPath: string;
  lastRun: RunSetup | null;
  lastSeed: number | null;
  busy: boolean;
  open: boolean;
  onToggle: () => void;
  height: number;
  /** The height a double click on the handle restores. */
  initialHeight: number;
  maxHeight: () => number;
  onHeight: (height: number) => void;
  /** A failure in words, located by node label. */
  describe: (failure: Failure) => string;
  onEdit: (update: (current: Experiment) => Experiment) => void;
  onShowNode: (node: ExperimentNode) => void;
  onShowFrame?: (seq: number) => void;
}) {
  const t = useT();
  const label = (id: string) => nodeLabel(doc.nodes.find((node) => node.id === id)?.type ?? "end", t);
  const outcomeText = !outcome ? (running ? t("exp.running") : "")
    : outcome.kind === "failed" ? `${t("exp.failed")} · ${describe(outcome.error)}`
    : outcome.kind === "passed" ? t("exp.passed") : t("exp.stopped");
  const download = reportPath ? downloadUrl(reportPath) : null;

  return <div className="experiment-timeline">
    {open && <Splitter orientation="horizontal" label="layout.timeline" size={height} min={90} initial={initialHeight} max={maxHeight} onSize={onHeight} />}
    <div className="experiment-timeline-title">
      <button className="ghost sm" onClick={onToggle} aria-expanded={open}>{open ? "▾" : "▸"} {t("exp.timeline")}{!open && events.length > 0 && <span className="experiment-node-count">{events.length}</span>}</button>
      {reportPath && (download
        ? <a className="experiment-report download-link" href={download} download data-tip={reportPath}>{t("exp.reportSaved")} ↓</a>
        : <span className="experiment-report" data-tip={reportPath}>{t("exp.reportSaved")}</span>)}
      {lastRun && (lastRun.overridden || doc.profiles.length > 0) && <span className="experiment-run-profile">
        {lastRun.profile ? t("exp.runProfile", { name: lastRun.profile }) : t("exp.runDefaults")}{lastRun.overridden && ` · ${t("exp.overridden")}`}</span>}
      {doc.seed !== null
        ? <button className="ghost sm experiment-seed-chip pinned" data-tip={t("exp.unpinSeedHint")} onClick={() => onEdit((current) => ({ ...current, seed: null }))}>{t("exp.runSeed", { seed: doc.seed })} · {t("exp.unpinSeed")}</button>
        : lastSeed !== null && <button className="ghost sm experiment-seed-chip" data-tip={t("exp.pinSeedHint")} disabled={busy} onClick={() => onEdit((current) => ({ ...current, seed: lastSeed }))}>{t("exp.runSeed", { seed: lastSeed })} · {t("exp.pinSeed")}</button>}
      <strong className={outcome?.kind === "failed" ? "fail" : ""} data-tip={outcome?.kind === "failed" ? describe(outcome.error) : undefined}>{outcomeText}</strong>
    </div>
    {open && <div className="experiment-events">{events.length === 0 ? <span className="experiment-empty">{t("exp.noEvents")}</span> : events.map((event, index) =>
      <button key={index} onClick={() => { const node = doc.nodes.find((item) => item.id === event.node_id); if (node) onShowNode(node); }} className={event.state}>
        <time>{fmtTime(event.ts).slice(0, 8)}</time><b>{label(event.node_id)}</b><span data-tip={event.error?.detail}>{t(`exp.${event.state}`)}{stepText(event, t) && ` · ${stepText(event, t)}`}</span>
      </button>)}</div>}
    {open && onShowFrame && events.some((event) => event.frame !== undefined) && <div className="experiment-frame-links">{events.filter((event) => event.frame !== undefined).map((event) =>
      <button key={`${event.node_id}-${event.frame}`} className="ghost sm" data-tip={t("exp.openFrameHint")} onClick={() => onShowFrame(event.frame!)}>◫ {label(event.node_id)} · {t("exp.openFrame", { seq: event.frame! })}</button>)}</div>}
  </div>;
}
