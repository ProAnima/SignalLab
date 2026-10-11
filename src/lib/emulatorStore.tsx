import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { api, on, EV, type DownFault, type Emulator, type EmulatorActivity, type EmulatorCounts, type Exchange, type JobEnded, type JobInfo, type StoredEmulator } from "./api";
import { localizeEmulatorSeed } from "./emulators";
import { useStore, type SaveState } from "./store";
import { useI18n } from "./i18n";
import { isEngineError, type Failure } from "./errors";
import { en } from "./locales/en";
import { onFlush } from "./flush";

/** Exchanges a running emulator keeps on screen; the engine keeps its own. */
const SHOWN = 300;
/** Long enough that typing a name is one write, short enough to feel saved (as the signal library). */
const SAVE_DEBOUNCE_MS = 700;

/** What a running emulator has said: its counts and its newest exchanges, newest last. */
export interface Live {
  counts: EmulatorCounts;
  exchanges: Exchange[];
  /** Exchanges that happened but were not sent to this page. */
  dropped: number;
  /** Taken down by a person or a run, with what HTTP meets meanwhile. */
  forced?: DownFault;
}

interface EmulatorStore {
  emulators: StoredEmulator[];
  path: string;
  /**
   * The file on disk could not be read (the list is then empty), or changed
   * since it was read (another page, an editor). Until a reload reads it,
   * nothing writes it: an edit is refused with this failure.
   */
  error: Failure | null;
  saveState: SaveState;
  /** Replace the library; the file follows on its own shortly after. */
  setEmulators: (next: StoredEmulator[]) => void;
  reload: () => void;
  /** The job each library entry runs as, by its id (`JobInfo.params.source`). */
  running: Map<string, JobInfo>;
  /** Start one; the failure, if any, is thrown for the caller to show. */
  start: (stored: StoredEmulator, params?: Record<string, string>) => Promise<JobInfo>;
  /** What the job was started with, to tell when the document has changed since. */
  startedWith: (jobId: number) => Emulator | undefined;
}

const Ctx = createContext<EmulatorStore | null>(null);
/**
 * What each running emulator job has said, by job id: a context of its own,
 * since it changes five times a second for every emulator with traffic, and
 * only the Emulators screen shows it. A job's entry goes when the job ends.
 */
const LiveCtx = createContext<Record<number, Live> | null>(null);

export function EmulatorProvider({ children }: { children: ReactNode }) {
  const { jobs, refreshJobs, pushLog, pushError } = useStore();
  const { t } = useI18n();
  const translate = useRef(t);
  translate.current = t;
  const [emulators, setList] = useState<StoredEmulator[]>([]);
  const [path, setPath] = useState("");
  const [error, setError] = useState<Failure | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [live, setLive] = useState<Record<number, Live>>({});
  const pending = useRef<StoredEmulator[] | null>(null);
  const timer = useRef<number | null>(null);
  const started = useRef(new Map<number, Emulator>());
  // What the file held when this page last read or wrote it: a save names it (as the signal library's).
  const stamp = useRef("");
  const writes = useRef<Promise<unknown>>(Promise.resolve());
  const blocked = useRef<Failure | null>(null);
  const block = useCallback((failure: Failure | null) => { blocked.current = failure; setError(failure); }, []);

  const write = useCallback((next: StoredEmulator[]) => {
    // One write at a time: each names the stamp the one before it left.
    const written = writes.current.catch(() => {}).then(() =>
      api.emulatorsSave({ version: 1, emulators: next }, stamp.current).then((saved) => { stamp.current = saved.stamp; }));
    writes.current = written;
    return written.then(
      () => setSaveState("saved"),
      (e) => {
        setSaveState("error");
        // Changed by someone else, or broken, since it was read: left alone until Reload.
        if (isEngineError(e) && (e.code === "file.changed" || e.code === "emulators.json_invalid")) block(e);
        pushError("emulators", e);
      },
    );
  }, [pushError, block]);

  const load = useCallback(() => {
    // What the file holds wins over an edit not written yet: that is what reloading asks for.
    if (timer.current !== null) window.clearTimeout(timer.current);
    timer.current = null;
    pending.current = null;
    api.emulatorsLoad().then(
      (file) => {
        stamp.current = file.stamp;
        let list = file.library.emulators;
        if (file.seeded) {
          // The starter set, in the language of the person who opened it first.
          const text = (key: string) => key in en ? translate.current(key) : null;
          list = localizeEmulatorSeed(list, text);
          write(list);
          pushLog("ok", "emulators", "log.emulatorsSeeded", { path: file.path });
        } else {
          pushLog("info", "emulators", "log.emulatorsLoaded", { n: list.length, path: file.path });
        }
        setList(list);
        setPath(file.path);
        block(null);
      },
      // A hand-edited file with a typo must say so and stay untouched.
      (e) => { block(e); pushError("emulators", e); },
    );
  }, [pushLog, pushError, write, block]);

  const setEmulators = useCallback((next: StoredEmulator[]) => {
    if (blocked.current !== null) {
      pushError("emulators", blocked.current);
      return;
    }
    setList(next);
    pending.current = next;
    setSaveState("saving");
    if (timer.current !== null) window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      timer.current = null;
      const list = pending.current;
      pending.current = null;
      if (list) write(list);
    }, SAVE_DEBOUNCE_MS);
  }, [write, pushError]);

  // Before an update restarts the app: the write waiting for its debounce, now.
  useEffect(() => onFlush(() => {
    if (timer.current === null) return;
    window.clearTimeout(timer.current);
    timer.current = null;
    const list = pending.current;
    pending.current = null;
    return list ? write(list) : undefined;
  }), [write]);

  // A write still waiting for its debounce is made now, not lost to teardown.
  useEffect(() => () => {
    if (timer.current === null) return;
    window.clearTimeout(timer.current);
    if (pending.current) write(pending.current);
  }, [write]);

  useEffect(() => { load(); }, [load]);

  const emulatorJobs = useMemo(() => jobs.filter((job) => job.kind === "emulator"), [jobs]);
  const running = useMemo(() => {
    const map = new Map<string, JobInfo>();
    for (const job of emulatorJobs) if (job.params?.source) map.set(job.params.source, job);
    return map;
  }, [emulatorJobs]);

  // What arrives, as it arrives; a job first seen here (another page started
  // it, or this screen came back) is filled from what the engine kept.
  useEffect(() => {
    const off = on<EmulatorActivity>(EV.emulatorActivity, (event) => {
      const activity = event.payload;
      setLive((prior) => {
        const before = prior[activity.job_id] ?? { counts: activity.counts, exchanges: [], dropped: 0 };
        const known = new Set(before.exchanges.map((exchange) => exchange.seq));
        const exchanges = [...before.exchanges, ...activity.exchanges.filter((exchange) => !known.has(exchange.seq))].slice(-SHOWN);
        return { ...prior, [activity.job_id]: { counts: activity.counts, exchanges, dropped: before.dropped + activity.dropped, forced: activity.forced } };
      });
    });
    // An emulator stopped is not shown any more: what it said goes with it, so
    // starting and stopping all afternoon does not keep every run's exchanges.
    const ended = on<JobEnded>(EV.jobEnded, (event) => {
      if (event.payload.kind !== "emulator") return;
      const id = event.payload.job_id;
      started.current.delete(id);
      setLive((prior) => {
        if (!(id in prior)) return prior;
        const { [id]: _gone, ...rest } = prior;
        return rest;
      });
    });
    return () => { off.then((stop) => stop()); ended.then((stop) => stop()); };
  }, []);
  // Asked once per job: activity keeps arriving while the answer is on its way.
  const asked = useRef(new Set<number>());
  useEffect(() => {
    for (const job of emulatorJobs) {
      if (live[job.id] || asked.current.has(job.id)) continue;
      asked.current.add(job.id);
      api.emulatorExchanges(job.id).then(
        (snapshot) => setLive((prior) => prior[job.id] ? prior : { ...prior, [job.id]: { counts: snapshot.counts, exchanges: snapshot.exchanges.slice(-SHOWN), dropped: 0, forced: snapshot.forced } }),
        () => {},
      );
    }
  }, [emulatorJobs, live]);

  const start = useCallback(async (stored: StoredEmulator, params?: Record<string, string>) => {
    const job = await api.emulatorStart(stored.emulator, stored.id, params);
    started.current.set(job.id, stored.emulator);
    pushLog("ok", "emulators", "log.emulatorStarted", { name: stored.emulator.name, local: job.params?.local ?? stored.emulator.bind });
    refreshJobs();
    return job;
  }, [pushLog, refreshJobs]);

  const startedWith = useCallback((jobId: number) => started.current.get(jobId), []);

  const value = useMemo<EmulatorStore>(() => ({ emulators, path, error, saveState, setEmulators, reload: load, running, start, startedWith }),
    [emulators, path, error, saveState, setEmulators, load, running, start, startedWith]);
  return <Ctx.Provider value={value}><LiveCtx.Provider value={live}>{children}</LiveCtx.Provider></Ctx.Provider>;
}

/** What each running emulator job has said, by job id. */
export function useEmulatorLive(): Record<number, Live> {
  const live = useContext(LiveCtx);
  if (!live) throw new Error("useEmulatorLive must be used within EmulatorProvider");
  return live;
}

export function useEmulators(): EmulatorStore {
  const store = useContext(Ctx);
  if (!store) throw new Error("useEmulators must be used within EmulatorProvider");
  return store;
}
