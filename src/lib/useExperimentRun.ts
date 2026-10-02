import { useEffect, useRef, useState } from "react";
import { api, EV, on, type Experiment, type ExperimentEnded, type ExperimentStep, type JobInfo } from "./api";
import type { Failure } from "./errors";
import { useStore } from "./store";
import type { RunOptions } from "../components/ExperimentRunWith";

/** How the last run ended. A failure is kept as it came and described when shown. */
export type Outcome = { kind: "passed" | "failed" | "stopped"; error?: Failure } | null;
/** How the last run was set up, shown next to its result. */
export type RunSetup = { profile: string | null; overridden: boolean };

/**
 * The editor's run: started (validated and saved first), followed step by
 * step until it ends, or stopped. Also keeps the variable values seen so far —
 * in the last run and from Send now — which the preview and Send now use.
 */
export function useExperimentRun({ doc, save, setProblem, revealProblem, onLaunch }: {
  doc: Experiment | null;
  save: (doc: Experiment) => Promise<void>;
  /** Stable: the run's event listeners are registered once. */
  setProblem: (failure: Failure | null) => void;
  /** Say what stopped the run from starting and show the node it is about. */
  revealProblem: (failure: Failure) => void;
  /** The run is about to start; the editor makes room for its timeline. */
  onLaunch: () => void;
}) {
  const { refreshJobs } = useStore();
  const [events, setEvents] = useState<ExperimentStep[]>([]);
  const [job, setJob] = useState<JobInfo | null>(null);
  const [outcome, setOutcome] = useState<Outcome>(null);
  const [reportPath, setReportPath] = useState("");
  const [starting, setStarting] = useState(false);
  // Variable values seen in the last run and from Send now; the preview and Send now use them.
  const [knownVars, setKnownVars] = useState<Record<string, unknown>>({});
  const [lastSeed, setLastSeed] = useState<number | null>(null);
  const [lastRun, setLastRun] = useState<RunSetup | null>(null);
  const launchPending = useRef(false);
  // The job followed: null for none, -1 while it starts (its id is not known yet).
  const activeId = useRef<number | null>(null);
  const busy = !!job || starting;

  useEffect(() => {
    const steps = on<ExperimentStep>(EV.experimentStep, (event) => {
      if (activeId.current === null || (activeId.current !== -1 && event.payload.job_id !== activeId.current)) return;
      setEvents((prior) => [...prior, event.payload]);
      const written = event.payload.vars;
      if (written) setKnownVars((prior) => ({ ...prior, ...written }));
    });
    const ended = on<ExperimentEnded>(EV.experimentEnded, (event) => {
      if (activeId.current === null || (activeId.current !== -1 && event.payload.job_id !== activeId.current)) return;
      setOutcome(event.payload.error ? { kind: "failed", error: event.payload.error } : { kind: "passed" });
      setLastSeed(event.payload.seed ?? null);
      setLastRun({ profile: event.payload.profile ?? null, overridden: !!event.payload.overridden });
      setReportPath(event.payload.report_path ?? "");
      if (event.payload.report_error) setProblem(event.payload.report_error);
      setJob(null); activeId.current = null; refreshJobs();
    });
    return () => { steps.then((off) => off()); ended.then((off) => off()); };
  }, [refreshJobs, setProblem]);

  /** Run the document; Run with… passes another profile, overrides or a seed for this run only. */
  const run = async (options?: RunOptions) => {
    if (!doc || job || launchPending.current) return;
    launchPending.current = true; setStarting(true);
    try {
      const runDoc = options ? { ...doc, profile: options.profile } : doc;
      try { await api.experimentValidate(runDoc, options?.overrides); }
      catch (error) { revealProblem(error); activeId.current = null; return; }
      await save(doc);
      setProblem(null); setEvents([]); setOutcome(null); setReportPath(""); onLaunch();
      activeId.current = -1;
      const started = await api.experimentStart(runDoc, options?.overrides, options?.seed);
      if (activeId.current === null) { refreshJobs(); return; }
      if (activeId.current === -1) activeId.current = started.id;
      setJob(started); refreshJobs();
    // Opening a wait's port can still fail here, at the wait that needs it.
    } catch (error) { activeId.current = null; revealProblem(error); }
    finally { launchPending.current = false; setStarting(false); }
  };

  const stop = async () => {
    if (!job) return;
    try {
      await api.jobStop(job.id);
      activeId.current = null; setJob(null); setOutcome({ kind: "stopped" }); refreshJobs();
    } catch (error) { setProblem(error); }
  };

  /** Another document is open: nothing the last run saw applies to it. */
  const reset = () => { setEvents([]); setOutcome(null); setReportPath(""); setKnownVars({}); setLastSeed(null); setLastRun(null); };

  return { events, job, outcome, reportPath, starting, busy, knownVars, setKnownVars, lastSeed, lastRun, run, stop, reset };
}
