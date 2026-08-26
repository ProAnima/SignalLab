import { Fragment, useEffect, useMemo, useRef, useState } from "react";
import { api, on, EV, type CaptureStats, type Frame, type InspectBatch } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { fmtBytes, fmtNum, fmtTime } from "../lib/format";

/** Frames kept in the view. The engine's ring holds more for export. */
const VIEW_CAPACITY = 4000;
/** Rows actually rendered — filtering happens over the whole buffer first. */
const RENDER_LIMIT = 300;

/** A captured frame, plus a marker for frames that never made it to the UI. */
type Row = Frame & { gap?: number };

const PROTOS = ["osc", "udp", "tcp", "http"];

function verdictClass(v: string | null): string {
  if (!v) return "verdict-ok";
  if (/drop|fail|error/i.test(v)) return "verdict-drop";
  if (/corrupt|copy|sampled/i.test(v)) return "verdict-warn";
  return "verdict-ok";
}

export function InspectView() {
  const { pushLog } = useStore();
  const t = useT();

  const [rows, setRows] = useState<Row[]>([]);
  const [stats, setStats] = useState<CaptureStats | null>(null);
  const [paused, setPaused] = useState(false);
  const [selected, setSelected] = useState<number | null>(null);
  const [query, setQuery] = useState("");
  const [protoFilter, setProtoFilter] = useState<string[]>([]);
  const [dirFilter, setDirFilter] = useState<string[]>([]);
  const pausedRef = useRef(paused);
  pausedRef.current = paused;

  // Repopulate from the engine's ring so switching views doesn't lose history.
  useEffect(() => {
    api.inspectSnapshot(VIEW_CAPACITY).then(setRows).catch(() => {});
    api.inspectStats().then(setStats).catch(() => {});

    const un = on<InspectBatch>(EV.inspectBatch, (e) => {
      const b = e.payload;
      setStats(b.stats);
      if (pausedRef.current || b.frames.length === 0) return;
      setRows((prev) => {
        const incoming: Row[] = b.skipped_now > 0
          ? b.frames.map((f, i) => (i === 0 ? { ...f, gap: b.skipped_now } : f))
          : b.frames;
        const next = prev.concat(incoming);
        return next.length > VIEW_CAPACITY ? next.slice(next.length - VIEW_CAPACITY) : next;
      });
    });
    return () => { un.then((f) => f()); };
  }, []);

  const arm = async (enabled: boolean) => {
    try {
      setStats(await api.inspectSetEnabled(enabled));
      pushLog(enabled ? "ok" : "warn", "inspect", enabled ? "log.captureArmed" : "log.captureDisarmed");
    } catch (e) {
      pushLog("err", "inspect", String(e));
    }
  };

  const clear = async () => {
    setRows([]);
    setSelected(null);
    try { setStats(await api.inspectClear()); } catch { /* ignore */ }
  };

  const exportTo = async (format: "jsonl" | "txt") => {
    try {
      const path = await api.inspectExport(format);
      pushLog("ok", "inspect", "log.captureSaved", { path });
    } catch (e) {
      pushLog("err", "inspect", String(e));
    }
  };

  const toggle = (list: string[], set: (v: string[]) => void, key: string) =>
    set(list.includes(key) ? list.filter((k) => k !== key) : [...list, key]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return rows.filter((f) => {
      if (protoFilter.length && !protoFilter.includes(f.proto)) return false;
      if (dirFilter.length && !dirFilter.includes(f.dir)) return false;
      if (!q) return true;
      return (
        f.summary.toLowerCase().includes(q) ||
        f.remote.toLowerCase().includes(q) ||
        f.source.toLowerCase().includes(q) ||
        f.proto.includes(q) ||
        (f.verdict ?? "").toLowerCase().includes(q)
      );
    });
  }, [rows, query, protoFilter, dirFilter]);

  // Newest first, so live traffic appears at the top without scroll juggling.
  const visible = useMemo(
    () => filtered.slice(Math.max(0, filtered.length - RENDER_LIMIT)).reverse(),
    [filtered]
  );

  const picked = selected !== null ? rows.find((f) => f.seq === selected) ?? null : null;
  const armed = stats?.enabled ?? false;

  return (
    <div>
      <div className="view-head">
        <h1>{t("ins.title")}</h1>
        <p>{t("ins.blurb")}</p>
      </div>

      <div className="capture-bar">
        <span className={"rec-dot " + (armed ? (paused ? "paused" : "live") : "")} />
        <button className={armed ? "danger" : "primary"} onClick={() => arm(!armed)}>
          {armed ? t("ins.disarm") : t("ins.arm")}
        </button>
        <button className="ghost" onClick={() => setPaused(!paused)} disabled={!armed}>
          {paused ? t("ins.resume") : t("ins.pause")}
        </button>
        <button className="ghost" onClick={clear}>{t("common.clear")}</button>
        <button className="ghost sm" onClick={() => exportTo("jsonl")}>{t("ins.exportJsonl")}</button>
        <button className="ghost sm" onClick={() => exportTo("txt")}>{t("ins.exportTxt")}</button>
        <div style={{ flex: 1 }} />
        <span className="tag-chip">{t("ins.captured", { n: fmtNum(stats?.total ?? 0) })}</span>
        <span className="tag-chip">{fmtBytes(stats?.bytes ?? 0)}</span>
        <span className="tag-chip">
          {t("ins.buffer", { used: fmtNum(stats?.buffered ?? 0), cap: fmtNum(stats?.capacity ?? 0) })}
        </span>
        {!!stats?.skipped && (
          <span className="tag-chip" style={{ color: "var(--amber)" }} title={t("ins.notShownHint")}>
            {t("ins.notShown", { n: fmtNum(stats.skipped) })}
          </span>
        )}
      </div>

      <div className="capture-bar">
        <input
          style={{ flex: "1 1 240px", maxWidth: 320 }}
          placeholder={t("ins.filterPlaceholder")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <div className="chips" role="group" aria-label={t("common.protocol")}>
          {PROTOS.map((p) => (
            <button key={p} className={"chip" + (protoFilter.includes(p) ? " on" : "")}
              aria-pressed={protoFilter.includes(p)}
              onClick={() => toggle(protoFilter, setProtoFilter, p)}>{p}</button>
          ))}
        </div>
        <div className="chips" role="group" aria-label={t("ins.dir")}>
          {["tx", "rx"].map((d) => (
            <button key={d} className={"chip" + (dirFilter.includes(d) ? " on" : "")}
              aria-pressed={dirFilter.includes(d)}
              onClick={() => toggle(dirFilter, setDirFilter, d)}>{d}</button>
          ))}
        </div>
        {(query || protoFilter.length || dirFilter.length) && (
          <button className="ghost sm" onClick={() => { setQuery(""); setProtoFilter([]); setDirFilter([]); }}>
            {t("common.reset")}
          </button>
        )}
        <div style={{ flex: 1 }} />
        <span style={{ fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-faint)" }}>
          {t("ins.showing", { shown: fmtNum(visible.length), total: fmtNum(filtered.length) })}
        </span>
      </div>

      <div className="inspect-split">
        <div className="panel" style={{ padding: 0 }}>
          <div className="scroll-y" style={{ flex: 1, minHeight: 0 }}>
            <table className="grid">
              <thead>
                <tr>
                  <th style={{ width: 92 }}>{t("common.time")}</th>
                  <th style={{ width: 34 }}>{t("ins.dir")}</th>
                  <th style={{ width: 46 }}>{t("bc.proto")}</th>
                  <th style={{ width: 150 }}>{t("bc.peer")}</th>
                  <th style={{ width: 56 }}>{t("common.bytes")}</th>
                  <th>{t("ins.summary")}</th>
                  <th style={{ width: 130 }}>{t("ins.verdict")}</th>
                </tr>
              </thead>
              <tbody>
                {visible.map((f) => (
                  <Fragment key={f.seq}>
                    <tr
                      className={selected === f.seq ? "picked" : ""}
                      onClick={() => setSelected(f.seq)}
                      style={{ cursor: "pointer" }}
                    >
                      <td style={{ color: "var(--text-faint)" }}>{fmtTime(f.ts)}</td>
                      <td className={"dir-" + f.dir}>{f.dir === "tx" ? "→" : "←"}</td>
                      <td style={{ color: f.proto === "osc" ? "var(--accent)" : "var(--text-dim)" }}>{f.proto}</td>
                      <td>{f.remote}</td>
                      <td style={{ color: "var(--text-dim)" }}>{f.bytes}</td>
                      <td>{f.summary}</td>
                      <td className={verdictClass(f.verdict)}>{f.verdict ?? ""}</td>
                    </tr>
                    {/* Rows run newest-first, so the gap sits below the frame
                        that follows it in time. */}
                    {!!f.gap && (
                      <tr className="gap">
                        <td colSpan={7}>{t("ins.gap", { n: fmtNum(f.gap) })}</td>
                      </tr>
                    )}
                  </Fragment>
                ))}
                {visible.length === 0 && (
                  <tr>
                    <td colSpan={7} className="empty-state">
                      {!armed
                        ? t("ins.emptyDisarmed")
                        : rows.length === 0
                          ? t("ins.emptyWaiting")
                          : t("ins.emptyFiltered")}
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>

        <div className="panel scroll-y">
          <p className="section-label">{t("ins.detail")}</p>
          {!picked && <div className="empty-state">{t("ins.pickRow")}</div>}
          {picked && (
            <>
              <dl className="kv">
                <dt>{t("ins.seq")}</dt><dd>#{picked.seq}</dd>
                <dt>{t("common.time")}</dt><dd>{fmtTime(picked.ts)}</dd>
                <dt>{t("ins.direction")}</dt><dd>{picked.dir === "tx" ? t("ins.sentDir") : t("ins.receivedDir")}</dd>
                <dt>{t("common.protocol")}</dt><dd>{picked.proto}</dd>
                <dt>{t("ins.source")}</dt><dd>{picked.source}{picked.job_id !== null ? ` (${t("ins.job", { id: picked.job_id })})` : ""}</dd>
                <dt>{t("ins.local")}</dt><dd>{picked.local || "—"}</dd>
                <dt>{t("bc.peer")}</dt><dd>{picked.remote || "—"}</dd>
                <dt>{t("ins.size")}</dt><dd>{t("ins.sizeBytes", { n: picked.bytes })}</dd>
                {picked.verdict && <><dt>{t("ins.verdict")}</dt><dd className={verdictClass(picked.verdict)}>{picked.verdict}</dd></>}
              </dl>

              <p className="section-label" style={{ marginTop: 16 }}>{t("ins.decoded")}</p>
              <pre className="hex">{picked.detail ?? picked.summary}</pre>

              {picked.hex && (
                <>
                  <p className="section-label" style={{ marginTop: 16 }}>{t("ins.rawBytes")}</p>
                  <pre className="hex">{picked.hex}</pre>
                </>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}
