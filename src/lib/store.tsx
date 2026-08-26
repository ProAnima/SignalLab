import {
  createContext, useCallback, useContext, useEffect, useState,
  type ReactNode,
} from "react";
import { api, on, EV, type JobInfo, type JobEnded, type HostInfo } from "./api";
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

interface Store {
  host: HostInfo | null;
  jobs: JobInfo[];
  refreshJobs: () => void;
  stopJob: (id: number) => void;
  stopAll: () => void;
  log: LogEntry[];
  pushLog: PushLog;
  clearLog: () => void;
}

const Ctx = createContext<Store | null>(null);

const LOG_CAPACITY = 500;
let logSeq = 0;

export function StoreProvider({ children }: { children: ReactNode }) {
  const [host, setHost] = useState<HostInfo | null>(null);
  const [jobs, setJobs] = useState<JobInfo[]>([]);
  const [log, setLog] = useState<LogEntry[]>([]);

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

  useEffect(() => {
    api.hostInfo().then(setHost).catch(() => {});
    pushLog("info", "system", "log.ready");
    refreshJobs();

    const unlisteners: Promise<() => void>[] = [];
    unlisteners.push(
      on<JobEnded>(EV.jobEnded, (e) => {
        const p = e.payload;
        if (p.error) pushLog("err", p.kind, "log.jobFailed", { id: p.job_id, error: p.error });
        else pushLog("ok", p.kind, "log.jobFinished", { id: p.job_id });
        refreshJobs();
      })
    );

    const poll = setInterval(refreshJobs, 2000);
    return () => {
      clearInterval(poll);
      unlisteners.forEach((u) => u.then((f) => f()));
    };
  }, [pushLog, refreshJobs]);

  const value: Store = {
    host, jobs, refreshJobs, stopJob, stopAll, log, pushLog, clearLog,
  };
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useStore(): Store {
  const s = useContext(Ctx);
  if (!s) throw new Error("useStore must be used within StoreProvider");
  return s;
}
