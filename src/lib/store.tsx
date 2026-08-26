import {
  createContext, useCallback, useContext, useEffect, useRef, useState,
  type ReactNode,
} from "react";
import {
  api, on, EV,
  type JobInfo, type JobEnded, type HostInfo, type MqttBatch, type Signal,
} from "./api";
import { fireSignal } from "./signals";
import { ingestTopic, makeTopicRoot, type TopicNode } from "./topics";
import type { TextKey } from "./i18n";

export type LogLevel = "info" | "ok" | "warn" | "err";

/**
 * A console line stores its dictionary key and parameters rather than finished
 * text, so switching language re-renders the whole console in the new one. Raw
 * engine errors are passed as free text and fall through `t` untouched.
 */
export interface LogEntry {
  id: number;
  ts: number;
  level: LogLevel;
  tag: string;
  key: TextKey;
  params?: Record<string, string | number>;
}

type PushLog = (
  level: LogLevel,
  tag: string,
  key: TextKey,
  params?: Record<string, string | number>
) => void;

/** Where the library file stands right now, for the header line in the view. */
export type SaveState = "idle" | "saving" | "saved" | "error";

interface Store {
  host: HostInfo | null;
  jobs: JobInfo[];
  refreshJobs: () => void;
  stopJob: (id: number) => void;
  stopAll: () => void;
  log: LogEntry[];
  pushLog: PushLog;
  clearLog: () => void;

  /** The signal library, in file order. */
  library: Signal[];
  libraryPath: string;
  /** Set when the file on disk could not be read — the list is then empty. */
  libraryError: string | null;
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
  const [jobs, setJobs] = useState<JobInfo[]>([]);
  const [log, setLog] = useState<LogEntry[]>([]);
  const [library, setLibraryState] = useState<Signal[]>([]);
  const [libraryPath, setLibraryPath] = useState("");
  const [libraryError, setLibraryError] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [lastFired, setLastFired] = useState<Record<string, number>>({});
  const saveTimer = useRef<number | null>(null);
  const pendingSave = useRef<Signal[] | null>(null);
  const mqttTopics = useRef<TopicNode>(makeTopicRoot());
  const [mqttVersion, setMqttVersion] = useState(0);
  const [mqttDropped, setMqttDropped] = useState(0);

  const pushLog = useCallback<PushLog>((level, tag, key, params) => {
    setLog((prev) => {
      const next = [...prev, { id: ++logSeq, ts: Date.now(), level, tag, key, params }];
      return next.length > LOG_CAPACITY ? next.slice(next.length - LOG_CAPACITY) : next;
    });
  }, []);

  const refreshJobs = useCallback(() => {
    api.jobsList().then(setJobs).catch(() => {});
  }, []);

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
        setLibraryState(file.library.signals);
        setLibraryPath(file.path);
        setLibraryError(null);
        if (file.seeded) pushLog("ok", "signals", "log.librarySeeded", { path: file.path });
        else pushLog("info", "signals", "log.libraryLoaded", { n: file.library.signals.length, path: file.path });
      },
      (e) => {
        // A hand-edited file with a typo must say so and stay untouched, not be
        // silently replaced by the starter set.
        setLibraryError(String(e));
        pushLog("err", "signals", String(e));
      }
    );
  }, [pushLog]);

  const writeLibrary = useCallback((next: Signal[]) => {
    api.signalsSave({ version: 1, signals: next }).then(
      () => setSaveState("saved"),
      (e) => {
        setSaveState("error");
        pushLog("err", "signals", String(e));
      }
    );
  }, [pushLog]);

  /**
   * The file is ours alone, so there is nothing to merge and no reason to make
   * anyone remember to save — which matters on a show site, where the thing you
   * forget is the thing you needed. Debounced so typing a name is one write.
   */
  const setLibrary = useCallback((next: Signal[]) => {
    setLibraryState(next);
    pendingSave.current = next;
    if (saveTimer.current !== null) window.clearTimeout(saveTimer.current);
    setSaveState("saving");
    saveTimer.current = window.setTimeout(() => {
      saveTimer.current = null;
      pendingSave.current = null;
      writeLibrary(next);
    }, SAVE_DEBOUNCE_MS);
  }, [writeLibrary]);

  const fire = useCallback(async (signal: Signal) => {
    try {
      // An MQTT signal rides an open connection when there is one.
      const live = jobs.find((j) => j.kind === "mqtt")?.id ?? null;
      const detail = await fireSignal(signal, live);
      setLastFired((prev) => ({ ...prev, [signal.id]: Date.now() }));
      pushLog("ok", "signals", "log.signalFired", { name: signal.name, detail });
    } catch (e) {
      pushLog("err", "signals", "log.signalFailed", { name: signal.name, error: String(e) });
    }
  }, [pushLog, jobs]);

  useEffect(() => {
    api.hostInfo().then(setHost).catch(() => {});
    pushLog("info", "system", "log.ready");
    refreshJobs();
    loadLibrary();

    const unlisteners: Promise<() => void>[] = [];
    unlisteners.push(
      on<JobEnded>(EV.jobEnded, (e) => {
        const p = e.payload;
        if (p.error) pushLog("err", p.kind, "log.jobFailed", { id: p.job_id, error: p.error });
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

    const poll = setInterval(refreshJobs, 2000);
    return () => {
      clearInterval(poll);
      unlisteners.forEach((u) => u.then((f) => f()));
    };
  }, [pushLog, refreshJobs, loadLibrary]);

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
    host, jobs, refreshJobs, stopJob, stopAll, log, pushLog, clearLog,
    library, libraryPath, libraryError, saveState, setLibrary,
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
