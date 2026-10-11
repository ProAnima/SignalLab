import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, EV, type JobInfo, type OpenPort, type ScanProgress } from "../lib/api";
import { useStore } from "../lib/store";
import { useT, type TKey } from "../lib/i18n";
import { useFieldIds, useJobStream, useWindowedRows } from "../lib/hooks";
import { fmtTime, fmtNum } from "../lib/format";

const PRESETS: { key: TKey; range: [number, number] }[] = [
  { key: "sc.preset.wellKnown", range: [1, 1024] },
  { key: "sc.preset.common", range: [1, 10000] },
  { key: "sc.preset.osc", range: [8000, 9100] },
  { key: "sc.preset.full", range: [1, 65535] },
];

/** A TCP port: 1 to 65 535 (what is typed past either end stops there). */
const port = (value: number) => Math.min(65535, Math.max(1, Math.round(value) || 1));
/** Found ports reach the table at most this often: a loopback scan finds thousands a second. */
const DRAW_MS = 150;
/** One row of the found ports, fixed so the table can draw only what is in view. */
const ROW_HEIGHT = 28;

export function ScanView() {
  const { pushLog, pushError, refreshJobs, stopJob, jobGone } = useStore();
  const t = useT();
  const fid = useFieldIds();

  const [host, setHost] = useState("127.0.0.1");
  const [portStart, setPortStart] = useState(1);
  const [portEnd, setPortEnd] = useState(1024);
  const [concurrency, setConcurrency] = useState(400);
  const [timeout, setTimeoutMs] = useState(500);
  const [grabBanner, setGrabBanner] = useState(true);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [prog, setProg] = useState<ScanProgress | null>(null);
  // Every port found, once each — at most 65 535, so nothing is dropped; drawn
  // in batches, never one render per port.
  const found = useRef(new Map<number, OpenPort>());
  const [drawn, setDrawn] = useState(0);
  const timer = useRef<number | null>(null);
  const pushOpen = useCallback((hit: OpenPort) => {
    found.current.set(hit.port, hit);
    timer.current ??= window.setTimeout(() => { timer.current = null; setDrawn((n) => n + 1); }, DRAW_MS);
  }, []);
  const clearOpen = () => { found.current = new Map(); setDrawn((n) => n + 1); };
  useEffect(() => () => { if (timer.current !== null) window.clearTimeout(timer.current); }, []);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const open = useMemo(() => [...found.current.values()].sort((a, b) => a.port - b.port), [drawn]);
  const rows = useWindowedRows(ROW_HEIGHT, open.length);

  useJobStream<OpenPort>(EV.scanOpen, job?.id ?? null, pushOpen);
  // The last progress (every port tried) draws what is still waiting at once:
  // a finished scan shows all it found together with its 100 %.
  const onProgress = useCallback((report: ScanProgress) => {
    if (report.done >= report.total && timer.current !== null) {
      window.clearTimeout(timer.current);
      timer.current = null;
      setDrawn((n) => n + 1);
    }
    setProg(report);
  }, []);
  useJobStream<ScanProgress>(EV.scanProgress, job?.id ?? null, onProgress);

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
      pushError("scan", e);
    }
  };

  const pct = prog ? Math.round((prog.done / prog.total) * 100) : 0;

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("sc.blurb")}>{t("sc.title")}</h1>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">{t("sc.target")}</p>
          <div className="field">
            <label htmlFor={fid("host")} data-tip={t("sc.hostHint")}>{t("sc.host")}</label>
            <input id={fid("host")} value={host} onChange={(e) => setHost(e.target.value)} disabled={!!job} />
          </div>
          <div className="row">
            <div className="field">
              <label htmlFor={fid("from")}>{t("sc.fromPort")}</label>
              <input id={fid("from")} type="number" value={portStart} min={1} max={65535} onChange={(e) => setPortStart(port(+e.target.value))} disabled={!!job} />
            </div>
            <div className="field">
              <label htmlFor={fid("to")} data-tip={t("sc.rangeHint")}>{t("sc.toPort")}</label>
              <input id={fid("to")} type="number" value={portEnd} min={1} max={65535} onChange={(e) => setPortEnd(port(+e.target.value))} disabled={!!job} />
            </div>
          </div>
          <div className="field">
            <label htmlFor={fid("preset")}>{t("sc.preset")}</label>
            {/* Reflects the current range instead of snapping back to the
                placeholder, which read as a bug. */}
            <select id={fid("preset")}
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
              <label htmlFor={fid("concurrency")} data-tip={t("common.concurrencyHint")}>{t("common.concurrency")}</label>
              <input id={fid("concurrency")} type="number" value={concurrency} onChange={(e) => setConcurrency(+e.target.value)} disabled={!!job} />
            </div>
            <div className="field">
              <label htmlFor={fid("timeout")} data-tip={t("common.timeoutHint")}>{t("common.timeoutMs")}</label>
              <input id={fid("timeout")} type="number" value={timeout} onChange={(e) => setTimeoutMs(+e.target.value)} disabled={!!job} />
            </div>
          </div>
          <div className="field">
            <label className="checkbox" data-tip={t("sc.grabBannerHint")}>
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
                <span>
                  {prog.unchecked > 0 && <span style={{ color: "var(--amber)", marginInlineEnd: 10 }} data-tip={t("sc.uncheckedHint")} tabIndex={0}>{t("sc.unchecked", { n: prog.unchecked })}</span>}
                  <span style={{ color: "var(--accent)" }}>{t("sc.open", { n: prog.open })}</span>
                </span>
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
          <div ref={rows.ref} className="scroll-y" style={{ flex: 1, minHeight: 0, maxHeight: "70vh", border: "1px solid var(--border)", borderRadius: "var(--radius-sm)" }}>
            <table className="grid fixed-rows">
              <thead>
                <tr>
                  <th style={{ width: 80 }}>{t("sc.port")}</th>
                  <th style={{ width: 90 }}>{t("common.time")}</th>
                  <th>{t("sc.banner")}</th>
                </tr>
              </thead>
              <tbody>
                {rows.before > 0 && <tr aria-hidden="true" style={{ height: rows.before }}><td colSpan={3} /></tr>}
                {open.slice(rows.start, rows.end).map((o) => (
                  <tr key={o.port} style={{ height: ROW_HEIGHT }}>
                    <td style={{ color: "var(--accent)", fontWeight: 700 }}>{o.port}</td>
                    <td style={{ color: "var(--text-faint)" }}>{fmtTime(o.ts)}</td>
                    <td style={{ color: "var(--text-dim)" }}>{o.banner ?? "—"}</td>
                  </tr>
                ))}
                {rows.after > 0 && <tr aria-hidden="true" style={{ height: rows.after }}><td colSpan={3} /></tr>}
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
