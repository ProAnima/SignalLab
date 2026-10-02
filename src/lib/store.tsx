import {
  createContext, useCallback, useContext, useEffect, useRef, useState,
  type ReactNode,
} from "react";
import { localizeSeed, type LibraryState } from "./library";
import {
  api, on, EV,
  type AppInfo, type JobInfo, type JobEnded, type HostInfo, type MqttBatch, type Signal,
} from "./api";
import { watchConnection, type ConnectionState } from "./transport";
import { fireSignal } from "./signals";
import { ingestTopic, makeTopicRoot, type TopicNode } from "./topics";
import { describeError, messageParams, type Failure } from "./errors";
import { useI18n, type TextKey, type Translate } from "./i18n";
import { en } from "./locales/en";

export type LogLevel = "info" | "ok" | "warn" | "err";

/**
 * A console line stores its dictionary key and parameters rather than finished
 * text, so switching language re-renders the whole console in the new one. A
 * failure is stored as it came (a structured engine error or legacy text) and
 * described when the line is shown, through the same renderer as everywhere.
 */
export interface LogEntry {
  id: number;
  ts: number;
  level: LogLevel;
  tag: string;
  key: TextKey;
  params?: Record<string, string | number>;
  /** Shown as `{error}` in the message. */
  error?: Failure;
}

type PushLog = (
  level: LogLevel,
  tag: string,
  key: TextKey,
  params?: Record<string, string | number>
) => void;

/** Log a failure. `key` may place it (`{error}`) in a sentence; by default the line is the failure. */
type PushError = (
  tag: string,
  error: Failure,
  key?: TextKey,
  params?: Record<string, string | number>
) => void;

/** The text of a console line in the current language (an impairment preset's name too). */
export function logText(entry: LogEntry, t: Translate): string {
  const params = entry.error === undefined ? entry.params : { ...entry.params, error: describeError(entry.error, t).text };
  return t(entry.key, messageParams(params, t));
}

/** Where the library file stands right now, for the header line in the view. */
export type SaveState = "idle" | "saving" | "saved" | "error";

interface Store {
  host: HostInfo | null;
  /** Where the engine runs (desktop or server) and what it allows; null until known. */
  info: AppInfo | null;
  /** The engine's event connection: always "open" in the desktop app. */
  connection: ConnectionState;
  jobs: JobInfo[];
  refreshJobs: () => void;
  /**
   * Has the engine confirmed this job is gone?
   *
   * Absence from the polled list only means something if the list was asked for
   * after the job started. Otherwise a job created between two polls looks dead
   * the instant it exists — which is how a port scan used to wipe its own screen
   * before a single result could land in it.
   */
  jobGone: (job: JobInfo | null) => boolean;
  stopJob: (id: number) => void;
  stopAll: () => void;
  log: LogEntry[];
  pushLog: PushLog;
  pushError: PushError;
  clearLog: () => void;

  /** The signal library, in file order. */
  library: Signal[];
  /** Every folder of the library (see lib/library.ts). */
  folders: string[];
  /** Signals and folders at once, as a folder rename or move changes both. */
  setLibraryState: (next: LibraryState) => void;
  libraryPath: string;
  /** Set when the file on disk could not be read — the list is then empty. */
  libraryError: Failure | null;
  saveState: SaveState;
  /** Replace the library; the file follows on its own shortly after. */
  setLibrary: (next: Signal[]) => void;
  reloadLibrary: () => void;
  /** Send a signal and log the outcome. */
  fire: (signal: Signal) => Promise<void>;
  /** id -> when it last went out, this session only. */
  lastFired: Record<string, number>;

  /**
   * What the broker holds, as a live index. Kept here rather than in the MQTT
   * view: a retained value arrives once, on subscribe, so a tree that only
   * exists while its tab is open would lose the broker's state on a click.
   */
  mqttTopics: { current: TopicNode };
  /** Bumped whenever the index changes, since the index itself is mutable. */
  mqttVersion: number;
  /** Messages the engine had to shed while the UI was behind. */
  mqttDropped: number;
  clearMqttTopics: () => void;
}

const Ctx = createContext<Store | null>(null);

const LOG_CAPACITY = 500;
let logSeq = 0;
/** Long enough that typing a name is one write, short enough to feel saved. */
const SAVE_DEBOUNCE_MS = 700;

export function StoreProvider({ children }: { children: ReactNode }) {
  const [host, setHost] = useState<HostInfo | null>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [connection, setConnection] = useState<ConnectionState>("connecting");
  const [jobs, setJobs] = useState<JobInfo[]>([]);
  // When the list currently in state was asked for, not when it arrived.
  const [jobsAt, setJobsAt] = useState(0);
  const jobsAtRef = useRef(0);
  const [log, setLog] = useState<LogEntry[]>([]);
  const [library, setSignals] = useState<Signal[]>([]);
  const [folders, setFolders] = useState<string[]>([]);
  // What the file must hold once the debounce runs: always the latest of both.
  const latestLibrary = useRef<LibraryState>({ signals: [], folders: [] });
  // The current language, for texts written into files (the starter set) — a ref,
  // so a language switch does not re-run what depends on it.
  const { t } = useI18n();
  const translate = useRef(t);
  translate.current = t;
  const [libraryPath, setLibraryPath] = useState("");
  const [libraryError, setLibraryError] = useState<Failure | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [lastFired, setLastFired] = useState<Record<string, number>>({});
  const saveTimer = useRef<number | null>(null);
  const pendingSave = useRef<LibraryState | null>(null);
  const mqttTopics = useRef<TopicNode>(makeTopicRoot());
  const [mqttVersion, setMqttVersion] = useState(0);
  const [mqttDropped, setMqttDropped] = useState(0);

  const append = useCallback((entry: Omit<LogEntry, "id" | "ts">) => {
    setLog((prev) => {
      const next = [...prev, { ...entry, id: ++logSeq, ts: Date.now() }];
      return next.length > LOG_CAPACITY ? next.slice(next.length - LOG_CAPACITY) : next;
    });
  }, []);

  const pushLog = useCallback<PushLog>((level, tag, key, params) => append({ level, tag, key, params }), [append]);

  const pushError = useCallback<PushError>(
    (tag, error, key = "log.error", params) => append({ level: "err", tag, key, params, error }),
    [append]
  );

  const refreshJobs = useCallback(() => {
    const asked = Date.now();
    api
      .jobsList()
      .then((list) => {
        // Two polls can overlap; the older answer must not win.
        if (asked < jobsAtRef.current) return;
        jobsAtRef.current = asked;
        setJobs(list);
        setJobsAt(asked);
      })
      .catch(() => {});
  }, []);

  const jobGone = useCallback(
    (job: JobInfo | null) =>
      !!job && jobsAt > job.started_ms && !jobs.some((j) => j.id === job.id),
    [jobs, jobsAt]
  );

  const stopJob = useCallback((id: number) => {
    api.jobStop(id).then(() => {
      pushLog("warn", "jobs", "log.jobStopped", { id });
      refreshJobs();
    });
  }, [pushLog, refreshJobs]);

  const stopAll = useCallback(() => {
    api.jobsStopAll().then(() => {
      pushLog("warn", "jobs", "log.allStopped");
      refreshJobs();
    });
  }, [pushLog, refreshJobs]);

  const clearLog = useCallback(() => setLog([]), []);

  const clearMqttTopics = useCallback(() => {
    mqttTopics.current = makeTopicRoot();
    setMqttDropped(0);
    setMqttVersion((v) => v + 1);
  }, []);

  // ---- signal library ----------------------------------------------------

  const loadLibrary = useCallback(() => {
    api.signalsLoad().then(
      (file) => {
        latestLibrary.current = { signals: file.library.signals, folders: file.library.folders ?? [] };
        if (file.seeded) {
          // The starter set, in the language of the person who opened it first.
          const text = (key: string) => key in en ? translate.current(key) : null;
          latestLibrary.current = { ...latestLibrary.current, signals: localizeSeed(file.library.signals, text) };
          api.signalsSave({ version: 2, ...latestLibrary.current }).catch((e) => pushError("signals", e));
        }
        setSignals(latestLibrary.current.signals);
        setFolders(latestLibrary.current.folders);
        setLibraryPath(file.path);
        setLibraryError(null);
        if (file.seeded) pushLog("ok", "signals", "log.librarySeeded", { path: file.path });
        else pushLog("info", "signals", "log.libraryLoaded", { n: file.library.signals.length, path: file.path });
      },
      (e) => {
        // A hand-edited file with a typo must say so and stay untouched, not be
        // silently replaced by the starter set.
        setLibraryError(e);
        pushError("signals", e);
      }
    );
  }, [pushLog, pushError]);

  const writeLibrary = useCallback((next: LibraryState) => {
    api.signalsSave({ version: 2, signals: next.signals, folders: next.folders }).then(
      () => setSaveState("saved"),
      (e) => {
        setSaveState("error");
        pushError("signals", e);
      }
    );
  }, [pushError]);

  /**
   * The file is ours alone, so there is nothing to merge and no reason to make
   * anyone remember to save — which matters on a show site, where the thing you
   * forget is the thing you needed. Debounced so typing a name is one write.
   */
  const setLibraryState = useCallback((next: LibraryState) => {
    latestLibrary.current = next;
    setSignals(next.signals);
    setFolders(next.folders);
    pendingSave.current = next;
    if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
    setSaveState("saving");
    saveTimer.current = window.setTimeout(() => {
      saveTimer.current = null;
      const pending = pendingSave.current;
      pendingSave.current = null;
      if (pending) writeLibrary(pending);
    }, SAVE_DEBOUNCE_MS);
  }, [writeLibrary]);
  const setLibrary = useCallback((next: Signal[]) => setLibraryState({ signals: next, folders: latestLibrary.current.folders }), [setLibraryState]);

  const fire = useCallback(async (signal: Signal) => {
    try {
      // An MQTT signal rides an open connection when there is one.
      const live = jobs.find((j) => j.kind === "mqtt")?.id ?? null;
      const fired = await fireSignal(signal, live);
      setLastFired((prev) => ({ ...prev, [signal.id]: Date.now() }));
      pushLog("ok", "signals", fired.key, { name: signal.name, ...fired.params });
    } catch (e) {
      pushError("signals", e, "log.signalFailed", { name: signal.name });
    }
  }, [pushLog, pushError, jobs]);

  useEffect(() => {
    api.hostInfo().then(setHost).catch(() => {});
    pushLog("info", "system", "log.ready");
    refreshJobs();
    loadLibrary();

    const unlisteners: Promise<() => void>[] = [];
    unlisteners.push(
      on<JobEnded>(EV.jobEnded, (e) => {
        const p = e.payload;
        if (p.error) pushError(p.kind, p.error, "log.jobFailed", { id: p.job_id });
        else pushLog("ok", p.kind, "log.jobFinished", { id: p.job_id });
        refreshJobs();
      })
    );

    unlisteners.push(
      on<MqttBatch>(EV.mqttMessages, (e) => {
        for (const m of e.payload.messages) ingestTopic(mqttTopics.current, m);
        if (e.payload.dropped > 0) setMqttDropped((d) => d + e.payload.dropped);
        setMqttVersion((v) => v + 1);
      })
    );

    // A page that fell behind is told how much it missed rather than nothing.
    unlisteners.push(
      on<{ skipped: number }>(EV.serverLagged, (e) => pushLog("warn", "server", "log.eventsLagged", { n: e.payload.skipped }))
    );

    api.appInfo().then(setInfo).catch(() => {});
    let wasOpen = false;
    const stopWatching = watchConnection((state) => {
      setConnection(state);
      if (state === "lost" && wasOpen) pushLog("warn", "server", "log.connectionLost");
      if (state === "open" && wasOpen) { pushLog("ok", "server", "log.connectionBack"); refreshJobs(); }
      if (state === "open") wasOpen = true;
    });

    const poll = setInterval(refreshJobs, 2000);
    return () => {
      clearInterval(poll);
      stopWatching();
      unlisteners.forEach((u) => u.then((f) => f()));
    };
  }, [pushLog, pushError, refreshJobs, loadLibrary]);

  // A debounced write must not be lost to teardown: cancel the timer and do
  // the write now instead of dropping it.
  useEffect(() => () => {
    if (saveTimer.current === null) return;
    window.clearTimeout(saveTimer.current);
    saveTimer.current = null;
    const pending = pendingSave.current;
    pendingSave.current = null;
    if (pending) writeLibrary(pending);
  }, [writeLibrary]);

  const value: Store = {
    host, info, connection, jobs, refreshJobs, jobGone, stopJob, stopAll, log, pushLog, pushError, clearLog,
    library, folders, setLibraryState, libraryPath, libraryError, saveState, setLibrary,
    reloadLibrary: loadLibrary, fire, lastFired,
    mqttTopics, mqttVersion, mqttDropped, clearMqttTopics,
  };
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useStore(): Store {
  const s = useContext(Ctx);
  if (!s) throw new Error("useStore must be used within StoreProvider");
  return s;
}
