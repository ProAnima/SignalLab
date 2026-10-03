/**
 * The end-to-end tour, run inside the real interface — the desktop app's
 * webview or a browser on the server — by scripts/e2e.mjs. Every step works
 * the screens the way a person does: it finds controls by their visible
 * labels (from the English dictionary, so a renamed text does not break it),
 * types into fields, presses buttons, and reads the result off the screen.
 * The runner checks the far end of each send against its loopback fixtures.
 *
 * A step returns its checks; a step that cannot go on throws. The runner
 * calls them one by one (`start` + `poll`), screenshots each, and does the
 * host-side work between them.
 */
import { pageErrors } from "./page";
import { closeOverlays, type Check, type Expect, type StepArgs } from "./dsl";
import { cleanup, russian, shell } from "./steps/shell";
import { inspectArm, inspectCheck, inspectWhole } from "./steps/inspector";
import { broadcast, http, mqtt, netsimCheck, netsimStart, osc, oscStop, scan, storm } from "./steps/tools";
import { library, signals } from "./steps/signals";
import { emulatorMqtt, emulators } from "./steps/emulators";
import { experimentAuth, experimentEmulator, experimentExport, experimentFaults, experimentHttp, experimentLoad, experimentLoop, experimentOsc, experimentParallel, experimentRepeat, experimentSelection } from "./steps/experiment";
import { layout } from "./steps/layout";
import { experimentWs, websocket } from "./steps/websocket";
import { feedback } from "./steps/about";

export type { Check };

// ---- the steps (steps/*.ts, by screen; the runner's plan calls them by these names) ----

const STEPS: Record<string, (expect: Expect, args: StepArgs) => Promise<Record<string, unknown> | void>> = {
  shell, inspectArm, osc, signals, oscStop, mqtt, broadcast, netsimStart, netsimCheck, storm, inspectWhole, scan, http, websocket,
  library, emulators, emulatorMqtt, experimentHttp, experimentOsc, experimentRepeat, experimentLoop, experimentLoad, experimentParallel, experimentEmulator, experimentFaults, experimentAuth, experimentWs, experimentSelection, experimentExport, layout,
  inspectCheck, feedback, russian, cleanup,
};

// ---- the runner's side --------------------------------------------------------------

interface Run { state: "running" | "done" | "failed"; checks: Check[]; data?: Record<string, unknown>; error?: string; ms: number }
const runs: Run[] = [];

declare global { interface Window { __signalLabTour?: unknown } }

window.__signalLabTour = {
  steps: Object.keys(STEPS),
  start(name: string, args: StepArgs = {}): number {
    const step = STEPS[name];
    const run: Run = { state: "running", checks: [], ms: 0 };
    runs.push(run);
    const started = performance.now();
    const expect: Expect = (check, ok, detail) => run.checks.push({ name: check, ok, detail: ok ? undefined : detail });
    if (!step) { run.state = "failed"; run.error = `no step “${name}”`; return runs.length - 1; }
    step(expect, args).then((data) => { run.data = data ?? undefined; run.state = "done"; })
      .catch((error: Error) => { run.state = "failed"; run.error = error?.message ?? String(error); closeOverlays(); })
      .finally(() => { run.ms = Math.round(performance.now() - started); });
    return runs.length - 1;
  },
  poll(id: number): Run | null { return runs[id] ?? null; },
  errors(): string[] { return pageErrors.splice(0); },
};
