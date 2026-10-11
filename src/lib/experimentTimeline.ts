/**
 * The run's steps as the editor keeps and shows them (pure): how a batch of
 * them is merged, and the timeline's rows — a step the engine reported after
 * a gap (`omitted`, engine/src/experiment_flow.rs `StepLog`) gets a row that
 * counts what was left out.
 */
import type { ExperimentStep } from "./api";

/** The engine's `STEPS_KEPT` and `STEPS_TAIL`: what a run reports one by one, from its start and from its end. */
export const STEPS_KEPT = 5_000;
export const STEPS_TAIL = 500;

/**
 * How often the steps that arrived are drawn. A Loop can report thousands a
 * second; each one a render that copies the list would freeze the editor.
 */
export const STEP_DRAW_MS = 150;

/** The variables a batch of steps wrote, later ones over earlier; null when none did. */
export function writtenBy(steps: ExperimentStep[]): Record<string, unknown> | null {
  let written: Record<string, unknown> | null = null;
  for (const step of steps) if (step.vars) written = { ...(written ?? {}), ...step.vars };
  return written;
}

/** A row: a step, or how many steps before the next one were left out. */
export type TimelineRow = { step: ExperimentStep; index: number } | { omitted: number; index: number };

/** The steps as rows: a step the engine reported after a gap gets the gap's row before it. */
export function timelineRows(events: ExperimentStep[]): TimelineRow[] {
  const rows: TimelineRow[] = [];
  events.forEach((step, index) => {
    if (step.omitted) rows.push({ omitted: step.omitted, index });
    rows.push({ step, index });
  });
  return rows;
}

/** Every step the run made: those reported and those counted in a gap. */
export function stepsMade(events: ExperimentStep[]): number {
  return events.reduce((sum, step) => sum + 1 + (step.omitted ?? 0), 0);
}
