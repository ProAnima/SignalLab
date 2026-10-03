import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { api } from "./api";
import { findUpdate, isDesktop, restartApp, updateChecksEnabled, type FoundUpdate } from "./platform";
import { flushAll } from "./flush";
import { usePersistentState } from "./hooks";
import { useStore } from "./store";

/**
 * Updates of the desktop app, from the project's published GitHub releases
 * only — as the studio's hub offers them (its channel, its rollout), or
 * straight from GitHub when the hub cannot be reached: looked for once a day
 * (and when asked in About), installed when a person says so — after every
 * pending write is made and every job stopped — then the app starts again as
 * the new version. The updater checks each download's signature against the
 * key built into the app before it runs anything. A browser never updates
 * itself: a server is updated by its image.
 */

export type UpdateState =
  | { phase: "idle" }
  | { phase: "checking" }
  | { phase: "current"; checkedAt: number }
  | { phase: "available" }
  | { phase: "downloading"; received: number; total?: number }
  | { phase: "installing" }
  | { phase: "failed"; during: "check" | "install"; detail: string };

/** A newer release: its version, when it was published, what is new. */
export interface Release {
  version: string;
  date?: string;
  notes?: string;
}

interface Updates {
  /** This app can update itself (the desktop app). */
  supported: boolean;
  state: UpdateState;
  /** The newer release found, until it is installed; kept when an install fails, to try again. */
  found: Release | null;
  /** Look once a day on its own. */
  auto: boolean;
  setAuto: (auto: boolean) => void;
  check: () => Promise<void>;
  install: () => Promise<void>;
}

const DAY_MS = 24 * 3600_000;
/** After start, once the app has settled. */
const FIRST_CHECK_MS = 20_000;
const LAST_CHECK = "signal-lab.updates.lastCheck";

function lastCheck(): number {
  try {
    return Number(localStorage.getItem(LAST_CHECK)) || 0;
  } catch {
    return 0;
  }
}

function rememberCheck(at: number) {
  try {
    localStorage.setItem(LAST_CHECK, String(at));
  } catch {
    // Then it looks again next start.
  }
}

/** A failure of the updater, as text for the details line. */
function detailOf(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return JSON.stringify(error);
}

const Ctx = createContext<Updates | null>(null);

export function UpdatesProvider({ children }: { children: ReactNode }) {
  const { jobs, pushLog, refreshJobs } = useStore();
  const [state, setState] = useState<UpdateState>({ phase: "idle" });
  const [release, setRelease] = useState<Release | null>(null);
  const [auto, setAuto] = usePersistentState("signal-lab.updates.auto", true);
  const [allowed, setAllowed] = useState(false);
  const found = useRef<FoundUpdate | null>(null);
  const busy = useRef(false);
  const jobCount = useRef(jobs.length);
  jobCount.current = jobs.length;

  useEffect(() => {
    if (isDesktop) void updateChecksEnabled().then(setAllowed);
  }, []);

  /** `quiet`: an automatic look, which shows only what it finds. */
  const look = useCallback(async (quiet: boolean) => {
    if (!isDesktop || busy.current) return;
    busy.current = true;
    if (!quiet) setState({ phase: "checking" });
    try {
      const update = await findUpdate();
      const at = Date.now();
      rememberCheck(at);
      found.current = update;
      setRelease(update ? { version: update.version, date: update.date, notes: update.notes } : null);
      if (update) {
        setState({ phase: "available" });
        if (quiet) pushLog("info", "update", "log.updateAvailable", { version: update.version });
      } else {
        setState((previous) => quiet && previous.phase === "idle" ? previous : { phase: "current", checkedAt: at });
      }
    } catch (error) {
      // An automatic look that fails (offline, the hub and GitHub down) waits for the next one in silence.
      if (!quiet) setState({ phase: "failed", during: "check", detail: detailOf(error) });
    } finally {
      busy.current = false;
    }
  }, [pushLog]);

  // Once a day while the app runs, and soon after start when the last look is a day old.
  useEffect(() => {
    if (!allowed || !auto) return;
    const due = () => Date.now() - lastCheck() >= DAY_MS;
    const first = window.setTimeout(() => { if (due()) void look(true); }, FIRST_CHECK_MS);
    const hourly = window.setInterval(() => { if (due()) void look(true); }, 3600_000);
    return () => { window.clearTimeout(first); window.clearInterval(hourly); };
  }, [allowed, auto, look]);

  const install = useCallback(async () => {
    const update = found.current;
    if (!update || busy.current) return;
    busy.current = true;
    try {
      // Nothing typed is lost and nothing keeps sending while the app goes away.
      await flushAll();
      if (jobCount.current > 0) {
        await api.jobsStopAll();
        pushLog("warn", "jobs", "log.allStopped");
        refreshJobs();
      }
      pushLog("info", "update", "log.updateInstalling", { version: update.version });
      setState({ phase: "downloading", received: 0 });
      let shown = 0;
      await update.install((received, total) => {
        const now = Date.now();
        if (now - shown < 100 && received !== total) return;
        shown = now;
        setState({ phase: "downloading", received, total });
      });
      setState({ phase: "installing" });
      // On Windows the installer has taken over and restarts the app itself.
      await restartApp();
    } catch (error) {
      setState({ phase: "failed", during: "install", detail: detailOf(error) });
    } finally {
      busy.current = false;
    }
  }, [pushLog, refreshJobs]);

  const value: Updates = { supported: isDesktop, state, found: release, auto, setAuto, check: () => look(false), install };
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useUpdates(): Updates {
  const updates = useContext(Ctx);
  if (!updates) throw new Error("useUpdates outside UpdatesProvider");
  return updates;
}
