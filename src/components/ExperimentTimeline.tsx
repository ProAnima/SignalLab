import { useMemo, type ReactNode } from "react";
import type { Experiment, ExperimentNode, ExperimentStep } from "../lib/api";
import type { Failure } from "../lib/errors";
import { nodeLabel, stepText } from "../lib/experimentText";
import type { Outcome, RunSetup } from "../lib/useExperimentRun";
import { fmtNum, fmtTime } from "../lib/format";
import { useWindowedRows } from "../lib/hooks";
import { useT } from "../lib/i18n";
import { downloadUrl } from "../lib/platform";
import { STEPS_KEPT, STEPS_TAIL, stepsMade, timelineRows, type TimelineRow } from "../lib/experimentTimeline";
import { Splitter } from "./Splitter";

/** One row of the timeline, in pixels (`.experiment-events` rows are this high). */
export const TIMELINE_ROW = 25;
/** Frame buttons under the rows: the latest ones, the rest counted. */
const FRAME_LINKS = 20;

/**
 * The run's pane under the canvas: how the run went (report, profile, seed),
 * one row per step, and the Inspector frames the waits matched.
 */
export function ExperimentTimeline({ doc, events, outcome, running, reportPath, lastRun, lastSeed, busy, open, onToggle, height, initialHeight, maxHeight, onHeight, describe, onEdit, onShowNode, onShowFrame, onCompare }: {
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
  /** Opens the run beside an earlier one. */
  onCompare: () => void;
}) {
  const t = useT();
  const label = (id: string) => nodeLabel(doc.nodes.find((node) => node.id === id)?.type ?? "end", t);
  const outcomeText = !outcome ? (running ? t("exp.running") : "")
    : outcome.kind === "failed" ? `${t("exp.failed")} · ${describe(outcome.error)}`
    : outcome.kind === "passed" ? t("exp.passed") : t("exp.stopped");
  const download = reportPath ? downloadUrl(reportPath) : null;
  const rows = useMemo(() => timelineRows(events), [events]);
  const made = useMemo(() => stepsMade(events), [events]);
  const frames = useMemo(() => events.filter((event) => event.frame !== undefined), [events]);
  const shownFrames = frames.slice(-FRAME_LINKS);

  return <div className="experiment-timeline">
    {open && <Splitter orientation="horizontal" label="layout.timeline" size={height} min={90} initial={initialHeight} max={maxHeight} onSize={onHeight} />}
    <div className="experiment-timeline-title">
      <button className="ghost sm" onClick={onToggle} aria-expanded={open}>{open ? "▾" : "▸"} {t("exp.timeline")}{!open && events.length > 0 && <span className="experiment-node-count">{fmtNum(made)}</span>}</button>
      {reportPath && (download
        ? <a className="experiment-report download-link" href={download} download data-tip={reportPath}>{t("exp.reportSaved")} ↓</a>
        : <span className="experiment-report" data-tip={reportPath}>{t("exp.reportSaved")}</span>)}
      {reportPath && <button className="ghost sm experiment-compare-open" data-tip={t("exp.compareHint")} disabled={running} onClick={onCompare}>{t("exp.compare")}</button>}
      {lastRun && (lastRun.overridden || doc.profiles.length > 0) && <span className="experiment-run-profile">
        {lastRun.profile ? t("exp.runProfile", { name: lastRun.profile }) : t("exp.runDefaults")}{lastRun.overridden && ` · ${t("exp.overridden")}`}</span>}
      {doc.seed !== null
        ? <button className="ghost sm experiment-seed-chip pinned" data-tip={t("exp.unpinSeedHint")} onClick={() => onEdit((current) => ({ ...current, seed: null }))}>{t("exp.runSeed", { seed: doc.seed })} · {t("exp.unpinSeed")}</button>
        : lastSeed !== null && <button className="ghost sm experiment-seed-chip" data-tip={t("exp.pinSeedHint")} disabled={busy} onClick={() => onEdit((current) => ({ ...current, seed: lastSeed }))}>{t("exp.runSeed", { seed: lastSeed })} · {t("exp.pinSeed")}</button>}
      <strong className={outcome?.kind === "failed" ? "fail" : ""} data-tip={outcome?.kind === "failed" ? describe(outcome.error) : undefined}>{outcomeText}</strong>
    </div>
    {open && (events.length === 0
      ? <div className="experiment-events"><span className="experiment-empty">{t("exp.noEvents")}</span></div>
      : <TimelineRows rows={rows} render={(row) => "omitted" in row
        ? <div key={`gap-${row.index}`} className="experiment-omitted" data-tip={t("exp.stepsOmittedHint", { kept: fmtNum(STEPS_KEPT), tail: fmtNum(STEPS_TAIL) })}>⋯ {t("exp.stepsOmitted", { n: row.omitted })}</div>
        : (() => { const event = row.step; const text = `${t(`exp.${event.state}`)}${stepText(event, t) && ` · ${stepText(event, t)}`}`;
          return <button key={row.index} onClick={() => { const node = doc.nodes.find((item) => item.id === event.node_id); if (node) onShowNode(node); }} className={event.state}>
            <time>{fmtTime(event.ts).slice(0, 8)}</time><b>{label(event.node_id)}</b><span data-tip={event.error?.detail ? `${text}\n${event.error.detail}` : text}>{text}</span>
          </button>; })()} />)}
    {open && onShowFrame && frames.length > 0 && <div className="experiment-frame-links">
      {frames.length > shownFrames.length && <span className="experiment-node-count">+{fmtNum(frames.length - shownFrames.length)}</span>}
      {shownFrames.map((event) =>
      <button key={`${event.node_id}-${event.frame}`} className="ghost sm" data-tip={t("exp.openFrameHint")} onClick={() => onShowFrame(event.frame!)}>◫ {label(event.node_id)} · {t("exp.openFrame", { seq: event.frame! })}</button>)}</div>}
  </div>;
}

/**
 * The rows that are in view, the rest stood in for by padding: a run reports
 * up to some 6 000 steps, and every one a button would make each draw slow.
 * Mounted with the list, so it measures the box it scrolls.
 */
function TimelineRows({ rows, render }: { rows: TimelineRow[]; render: (row: TimelineRow) => ReactNode }) {
  const { ref, start, end, before, after } = useWindowedRows(TIMELINE_ROW, rows.length);
  return <div className="experiment-events" ref={ref}>
    {before > 0 && <div aria-hidden="true" style={{ height: before, flexShrink: 0 }} />}
    {rows.slice(start, end).map(render)}
    {after > 0 && <div aria-hidden="true" style={{ height: after, flexShrink: 0 }} />}
  </div>;
}
