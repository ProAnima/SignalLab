import { useEffect, useState } from "react";
import { api, EV, type JobInfo, type OpenPort, type ScanProgress } from "../lib/api";
import { useStore } from "../lib/store";
import { useT, type TKey } from "../lib/i18n";
import { useJobStream, useRollingList } from "../lib/hooks";
import { fmtTime, fmtNum } from "../lib/format";

const PRESETS: { key: TKey; range: [number, number] }[] = [
  { key: "sc.preset.wellKnown", range: [1, 1024] },
  { key: "sc.preset.common", range: [1, 10000] },
  { key: "sc.preset.osc", range: [8000, 9100] },
  { key: "sc.preset.full", range: [1, 65535] },
];

export function ScanView() {
  const { pushLog, refreshJobs, stopJob, jobGone } = useStore();
  const t = useT();

  const [host, setHost] = useState("127.0.0.1");
  const [portStart, setPortStart] = useState(1);
  const [portEnd, setPortEnd] = useState(1024);
  const [concurrency, setConcurrency] = useState(400);
  const [timeout, setTimeoutMs] = useState(500);
  const [grabBanner, setGrabBanner] = useState(true);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [prog, setProg] = useState<ScanProgress | null>(null);
  const { items: open, push: pushOpen, clear: clearOpen } = useRollingList<OpenPort>(2000);

  useJobStream<OpenPort>(EV.scanOpen, job?.id ?? null, pushOpen);
  useJobStream<ScanProgress>(EV.scanProgress, job?.id ?? null, setProg);

  useEffect(() => {
    if (jobGone(job)) setJob(null);
  }, [jobGone, job]);

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
      pushLog("ok", "scan", "log.scanStarted", { host, from: portStart, to: portEnd });
      refreshJobs();
    } catch (e) {
      pushLog("err", "scan", String(e));
    }
  };

  const pct = prog ? Math.round((prog.done / prog.total) * 100) : 0;

  return (
    <div>
      <div className="view-head">
        <h1>{t("sc.title")}</h1>
        <p>{t("sc.blurb")}</p>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">{t("sc.target")}</p>
          <div className="field">
            <label>{t("sc.host")}</label>
            <input value={host} onChange={(e) => setHost(e.target.value)} disabled={!!job} />
          </div>
          <div className="row">
            <div className="field">
              <label>{t("sc.fromPort")}</label>
              <input type="number" value={portStart} onChange={(e) => setPortStart(+e.target.value)} disabled={!!job} />
            </div>
            <div className="field">
              <label>{t("sc.toPort")}</label>
              <input type="number" value={portEnd} onChange={(e) => setPortEnd(+e.target.value)} disabled={!!job} />
            </div>
          </div>
          <div className="field">
            <label>{t("sc.preset")}</label>
            {/* Reflects the current range instead of snapping back to the
                placeholder, which read as a bug. */}
            <select
              disabled={!!job}
              value={PRESETS.find((p) => p.range[0] === portStart && p.range[1] === portEnd)?.key ?? ""}
              onChange={(e) => {
                const p = PRESETS.find((x) => x.key === e.target.value);
                if (p) { setPortStart(p.range[0]); setPortEnd(p.range[1]); }
              }}
            >
              <option value="">{t("sc.choose")}</option>
              {PRESETS.map((p) => <option key={p.key} value={p.key}>{t(p.key)}</option>)}
            </select>
          </div>
          <div className="row">
            <div className="field">
              <label>{t("common.concurrency")}</label>
              <input type="number" value={concurrency} onChange={(e) => setConcurrency(+e.target.value)} disabled={!!job} />
            </div>
            <div className="field">
              <label>{t("common.timeoutMs")}</label>
              <input type="number" value={timeout} onChange={(e) => setTimeoutMs(+e.target.value)} disabled={!!job} />
            </div>
          </div>
          <div className="field">
            <label className="checkbox">
              <input type="checkbox" checked={grabBanner} onChange={(e) => setGrabBanner(e.target.checked)} disabled={!!job} />
              {t("sc.grabBanner")}
            </label>
          </div>
          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={start}>
              {job ? t("sc.stopScan") : t("sc.startScan")}
            </button>
          </div>
          {prog && (
            <div style={{ marginTop: 16 }}>
              <div style={{ height: 6, background: "var(--bg-input)", borderRadius: 6, overflow: "hidden", border: "1px solid var(--border)" }}>
                <div style={{ width: `${pct}%`, height: "100%", background: "var(--accent)", transition: "width 0.2s" }} />
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-dim)", marginTop: 6 }}>
                <span>{fmtNum(prog.done)} / {fmtNum(prog.total)} ({pct}%)</span>
                <span style={{ color: "var(--accent)" }}>{t("sc.open", { n: prog.open })}</span>
              </div>
            </div>
          )}
        </div>

        <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 360 }}>
          <div style={{ display: "flex", alignItems: "center", marginBottom: 10 }}>
            <p className="section-label" style={{ margin: 0 }}>{t("sc.openPorts")}</p>
            <div style={{ flex: 1 }} />
            <span className="tag-chip">{t("sc.found", { n: open.length })}</span>
          </div>
          <div className="scroll-y" style={{ flex: 1, border: "1px solid var(--border)", borderRadius: "var(--radius-sm)" }}>
            <table className="grid">
              <thead>
                <tr>
                  <th style={{ width: 80 }}>{t("sc.port")}</th>
                  <th style={{ width: 90 }}>{t("common.time")}</th>
                  <th>{t("sc.banner")}</th>
                </tr>
              </thead>
              <tbody>
                {open.slice().sort((a, b) => a.port - b.port).map((o, i) => (
                  <tr key={i}>
                    <td style={{ color: "var(--accent)", fontWeight: 700 }}>{o.port}</td>
                    <td style={{ color: "var(--text-faint)" }}>{fmtTime(o.ts)}</td>
                    <td style={{ color: "var(--text-dim)" }}>{o.banner ?? "—"}</td>
                  </tr>
                ))}
                {open.length === 0 && (
                  <tr><td colSpan={3} className="empty-state">{job ? t("sc.scanning") : t("sc.noOpen")}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  );
}
