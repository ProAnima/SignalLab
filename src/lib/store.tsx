import {
  createContext, useCallback, useContext, useEffect, useRef, useState,
  type ReactNode,
} from "react";
import { api, on, EV, type JobInfo, type JobEnded, type HostInfo } from "./api";

export type LogLevel = "info" | "ok" | "warn" | "err";
export interface LogEntry {
  id: number;
  ts: number;
  level: LogLevel;
  tag: string;
  msg: string;
}

interface Store {
  host: HostInfo | null;
  jobs: JobInfo[];
  refreshJobs: () => void;
  stopJob: (id: number) => void;
  stopAll: () => void;
  log: LogEntry[];
  pushLog: (level: LogLevel, tag: string, msg: string) => void;
  clearLog: () => void;
}

const Ctx = createContext<Store | null>(null);

let logSeq = 0;

export function StoreProvider({ children }: { children: ReactNode }) {
  const [host, setHost] = useState<HostInfo | null>(null);
  const [jobs, setJobs] = useState<JobInfo[]>([]);
  const [log, setLog] = useState<LogEntry[]>([]);

  const pushLog = useCallback((level: LogLevel, tag: string, msg: string) => {
    setLog((prev) => {
      const next = [...prev, { id: ++logSeq, ts: Date.now(), level, tag, msg }];
      return next.length > 500 ? next.slice(next.length - 500) : next;
    });
  }, []);

  const refreshJobs = useCallback(() => {
    api.jobsList().then(setJobs).catch(() => {});
  }, []);

  const stopJob = useCallback((id: number) => {
    api.jobStop(id).then(() => {
      pushLog("warn", "jobs", `stopped job #${id}`);
      refreshJobs();
    });
  }, [pushLog, refreshJobs]);

  const stopAll = useCallback(() => {
    api.jobsStopAll().then(() => {
      pushLog("warn", "jobs", "stopped all jobs");
      refreshJobs();
    });
  }, [pushLog, refreshJobs]);

  const clearLog = useCallback(() => setLog([]), []);

  const started = useRef(false);
  useEffect(() => {
    if (started.current) return;
    started.current = true;

    api.hostInfo().then(setHost).catch(() => {});
    pushLog("info", "system", "Signal Lab ready");
    refreshJobs();

    const unlisteners: Promise<() => void>[] = [];
    unlisteners.push(
      on<JobEnded>(EV.jobEnded, (e) => {
        const p = e.payload;
        if (p.error) pushLog("err", p.kind, `job #${p.job_id} ended: ${p.error}`);
        else pushLog("ok", p.kind, `job #${p.job_id} finished`);
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
