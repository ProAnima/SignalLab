import { useEffect, useRef, useState } from "react";
import { api, on, EV, type JobInfo, type OpenPort, type ScanProgress } from "../lib/api";
import { useStore } from "../lib/store";
import { useRollingList } from "../lib/hooks";
import { fmtTime, fmtNum } from "../lib/format";

const PRESETS: Record<string, [number, number]> = {
  "Well-known (1–1024)": [1, 1024],
  "Common (1–10000)": [1, 10000],
  "OSC range (8000–9100)": [8000, 9100],
  "Full (1–65535)": [1, 65535],
};

export function ScanView() {
  const { pushLog, refreshJobs, stopJob, jobs } = useStore();

  const [host, setHost] = useState("127.0.0.1");
  const [portStart, setPortStart] = useState(1);
  const [portEnd, setPortEnd] = useState(1024);
  const [concurrency, setConcurrency] = useState(400);
  const [timeout, setTimeoutMs] = useState(500);
  const [grabBanner, setGrabBanner] = useState(true);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [prog, setProg] = useState<ScanProgress | null>(null);
  const jobRef = useRef<number | null>(null);
  jobRef.current = job?.id ?? null;
  const { items: open, push: pushOpen, clear: clearOpen } = useRollingList<OpenPort>(2000);

  useEffect(() => {
    const unOpen = on<OpenPort>(EV.scanOpen, (e) => {
      if (jobRef.current !== null && e.payload.job_id === jobRef.current) pushOpen(e.payload);
    });
    const unProg = on<ScanProgress>(EV.scanProgress, (e) => {
      if (jobRef.current !== null && e.payload.job_id === jobRef.current) setProg(e.payload);
    });
    return () => { unOpen.then((f) => f()); unProg.then((f) => f()); };
  }, [pushOpen]);

  useEffect(() => {
    if (job && !jobs.find((j) => j.id === job.id)) setJob(null);
  }, [jobs, job]);

  const start = async () => {
    if (job) { stopJob(job.id); setJob(null); return; }
    try {
      clearOpen();
      setProg(null);
      const j = await api.scanStart({
        host, port_start: portStart, port_end: portEnd,
        concurrency, timeout_ms: timeout, grab_banner: grabBanner,
      });
      setJob(j);
      pushLog("ok", "scan", `scanning ${host} :${portStart}-${portEnd}`);
      refreshJobs();
    } catch (e) {
      pushLog("err", "scan", String(e));
    }
  };

  const pct = prog ? Math.round((prog.done / prog.total) * 100) : 0;

  return (
    <div>
      <div className="view-head">
        <h1>Port scanner</h1>
        <p>Concurrent TCP connect scan across a port range, with best-effort service banners. Use it to confirm which services are actually listening on a host.</p>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">Target</p>
          <div className="field"><label>Host / IP</label><input value={host} onChange={(e) => setHost(e.target.value)} disabled={!!job} /></div>
          <div className="row">
            <div className="field"><label>From port</label><input type="number" value={portStart} onChange={(e) => setPortStart(+e.target.value)} disabled={!!job} /></div>
            <div className="field"><label>To port</label><input type="number" value={portEnd} onChange={(e) => setPortEnd(+e.target.value)} disabled={!!job} /></div>
          </div>
          <div className="field">
            <label>Preset</label>
            <select disabled={!!job} onChange={(e) => { const p = PRESETS[e.target.value]; if (p) { setPortStart(p[0]); setPortEnd(p[1]); } }}>
              <option>— choose —</option>
              {Object.keys(PRESETS).map((k) => <option key={k}>{k}</option>)}
            </select>
          </div>
          <div className="row">
            <div className="field"><label>Concurrency</label><input type="number" value={concurrency} onChange={(e) => setConcurrency(+e.target.value)} disabled={!!job} /></div>
            <div className="field"><label>Timeout (ms)</label><input type="number" value={timeout} onChange={(e) => setTimeoutMs(+e.target.value)} disabled={!!job} /></div>
          </div>
          <div className="field">
            <label className="checkbox"><input type="checkbox" checked={grabBanner} onChange={(e) => setGrabBanner(e.target.checked)} disabled={!!job} /> grab service banners</label>
          </div>
          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={start}>{job ? "Stop scan" : "Start scan"}</button>
          </div>
          {prog && (
            <div style={{ marginTop: 16 }}>
              <div style={{ height: 6, background: "var(--bg-input)", borderRadius: 6, overflow: "hidden", border: "1px solid var(--border)" }}>
                <div style={{ width: `${pct}%`, height: "100%", background: "var(--accent)", transition: "width 0.2s" }} />
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-dim)", marginTop: 6 }}>
                <span>{fmtNum(prog.done)} / {fmtNum(prog.total)} ({pct}%)</span>
                <span style={{ color: "var(--accent)" }}>{prog.open} open</span>
              </div>
            </div>
          )}
        </div>

        <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 360 }}>
          <div style={{ display: "flex", alignItems: "center", marginBottom: 10 }}>
            <p className="section-label" style={{ margin: 0 }}>Open ports</p>
            <div style={{ flex: 1 }} />
            <span className="tag-chip">{open.length} found</span>
          </div>
          <div className="scroll-y" style={{ flex: 1, border: "1px solid var(--border)", borderRadius: 6 }}>
            <table className="grid">
              <thead><tr><th style={{ width: 80 }}>Port</th><th style={{ width: 90 }}>Time</th><th>Banner</th></tr></thead>
              <tbody>
                {open.slice().sort((a, b) => a.port - b.port).map((o, i) => (
                  <tr key={i}>
                    <td style={{ color: "var(--accent)", fontWeight: 700 }}>{o.port}</td>
                    <td style={{ color: "var(--text-faint)" }}>{fmtTime(o.ts)}</td>
                    <td style={{ color: "var(--text-dim)" }}>{o.banner ?? "—"}</td>
                  </tr>
                ))}
                {open.length === 0 && (
                  <tr><td colSpan={3} style={{ color: "var(--text-faint)", textAlign: "center", padding: 24 }}>{job ? "Scanning…" : "No open ports yet."}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}
