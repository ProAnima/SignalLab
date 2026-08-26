import { useEffect, useRef, useState } from "react";
import { api, on, EV, type HttpResponse, type JobInfo, type BurstProgress } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { useSeries } from "../lib/hooks";
import { fmtBytes, fmtNum, statusClass } from "../lib/format";
import { Scope } from "../components/Scope";

const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

export function HttpView() {
  const { pushLog, refreshJobs, stopJob, jobs } = useStore();
  const t = useT();

  const [method, setMethod] = useState("GET");
  const [url, setUrl] = useState("https://httpbin.org/get");
  const [headers, setHeaders] = useState<[string, string][]>([["Accept", "application/json"]]);
  const [body, setBody] = useState("");
  const [timeout, setTimeoutMs] = useState(10000);
  const [busy, setBusy] = useState(false);
  const [resp, setResp] = useState<HttpResponse | null>(null);
  const [showHeaders, setShowHeaders] = useState(false);

  const buildReq = () => ({
    method, url,
    headers: headers.filter(([k]) => k.trim()),
    body: body.length ? body : null,
    timeout_ms: timeout,
  });

  const send = async () => {
    setBusy(true);
    try {
      const r = await api.httpRequest(buildReq());
      setResp(r);
      if (r.error) {
        pushLog("err", "http", "log.httpFailed", { method, url, error: r.error });
      } else {
        pushLog(r.ok ? "ok" : "warn", "http", "log.httpDone", {
          method, url, status: r.status, ms: r.latency_ms.toFixed(0),
        });
      }
    } catch (e) {
      pushLog("err", "http", String(e));
    } finally {
      setBusy(false);
    }
  };

  // ---- burst ----
  const [concurrency, setConcurrency] = useState(20);
  const [total, setTotal] = useState(500);
  const [duration, setDuration] = useState(0);
  const [burstJob, setBurstJob] = useState<JobInfo | null>(null);
  const [prog, setProg] = useState<BurstProgress | null>(null);
  const burstRef = useRef<number | null>(null);
  burstRef.current = burstJob?.id ?? null;
  const { data: rpsSeries, push: pushRps, clear: clearRps } = useSeries(240);

  useEffect(() => {
    const un = on<BurstProgress>(EV.burstProgress, (e) => {
      if (burstRef.current !== null && e.payload.job_id === burstRef.current) {
        setProg(e.payload);
        pushRps(e.payload.rps);
      }
    });
    return () => { un.then((f) => f()); };
  }, [pushRps]);

  useEffect(() => {
    if (burstJob && !jobs.find((j) => j.id === burstJob.id)) setBurstJob(null);
  }, [jobs, burstJob]);

  const toggleBurst = async () => {
    if (burstJob) { stopJob(burstJob.id); setBurstJob(null); return; }
    try {
      clearRps();
      setProg(null);
      const job = await api.httpBurstStart({ ...buildReq(), concurrency, total, duration_s: duration });
      setBurstJob(job);
      pushLog("ok", "http", "log.httpBurst", { method, url, workers: concurrency });
      refreshJobs();
    } catch (e) {
      pushLog("err", "http", String(e));
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1>{t("http.title")}</h1>
        <p>{t("http.blurb")}</p>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">{t("http.request")}</p>
          <div className="row" style={{ marginBottom: 12 }}>
            <select style={{ flex: "0 0 110px" }} value={method} onChange={(e) => setMethod(e.target.value)}>
              {METHODS.map((m) => <option key={m}>{m}</option>)}
            </select>
            <input
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter" && !busy) { e.preventDefault(); void send(); } }}
              placeholder="https://..."
            />
          </div>
          <div className="field">
            <label>{t("http.headers")}</label>
            {headers.map((h, i) => (
              <div className="row tight" key={i} style={{ marginBottom: 6 }}>
                <input
                  placeholder={t("http.headerName")}
                  value={h[0]}
                  onChange={(e) => setHeaders(headers.map((x, j) => j === i ? [e.target.value, x[1]] : x))}
                />
                <input
                  placeholder={t("http.headerValue")}
                  value={h[1]}
                  onChange={(e) => setHeaders(headers.map((x, j) => j === i ? [x[0], e.target.value] : x))}
                />
                <button className="ghost sm" style={{ flex: "0 0 auto" }} onClick={() => setHeaders(headers.filter((_, j) => j !== i))}>✕</button>
              </div>
            ))}
            <button className="ghost sm" onClick={() => setHeaders([...headers, ["", ""]])}>
              {t("http.addHeader")}
            </button>
          </div>
          {method !== "GET" && method !== "HEAD" && (
            <div className="field">
              <label>{t("http.body")}</label>
              <textarea value={body} onChange={(e) => setBody(e.target.value)} placeholder='{ "key": "value" }' />
            </div>
          )}
          <div className="field">
            <label>{t("common.timeoutMs")}</label>
            <input type="number" value={timeout} onChange={(e) => setTimeoutMs(+e.target.value)} />
          </div>
          <div className="btn-row">
            <button className="primary" onClick={send} disabled={busy}>
              {busy ? t("common.sending") : t("common.send")}
            </button>
          </div>
        </div>

        <div className="panel" style={{ minHeight: 320 }}>
          <p className="section-label">{t("http.response")}</p>
          {!resp && <div className="empty-state">{t("http.noResponse")}</div>}
          {resp && (
            <>
              <div className="metrics" style={{ marginBottom: 14 }}>
                <div className="metric">
                  <div className="k">{t("http.status")}</div>
                  <div className={"v " + statusClass(resp.status)}>
                    {resp.status || "ERR"} <small>{resp.status_text}</small>
                  </div>
                </div>
                <div className="metric">
                  <div className="k">{t("http.latency")}</div>
                  <div className="v accent">{resp.latency_ms.toFixed(0)}<small>ms</small></div>
                </div>
                <div className="metric">
                  <div className="k">{t("http.size")}</div>
                  <div className="v">{fmtBytes(resp.body_bytes)}</div>
                </div>
              </div>
              {resp.error && <div className="hint amber" style={{ marginBottom: 12 }}>{resp.error}</div>}
              <button className="ghost sm" onClick={() => setShowHeaders(!showHeaders)} style={{ marginBottom: 8 }}>
                {showHeaders ? "▾" : "▸"} {t("http.responseHeaders", { n: resp.headers.length })}
              </button>
              {showHeaders && (
                <div className="scroll-y" style={{ maxHeight: 140, marginBottom: 10, border: "1px solid var(--border)", borderRadius: "var(--radius-sm)" }}>
                  <table className="grid"><tbody>
                    {resp.headers.map(([k, v], i) => (
                      <tr key={i}><td style={{ color: "var(--cyan)", width: 200 }}>{k}</td><td>{v}</td></tr>
                    ))}
                  </tbody></table>
                </div>
              )}
              <textarea
                readOnly
                value={resp.body + (resp.truncated ? "\n" + t("http.truncated") : "")}
                style={{ minHeight: 160, fontSize: 11 }}
              />
            </>
          )}
        </div>
      </div>

      <div className="panel" style={{ marginTop: 16 }}>
        <p className="section-label">{t("http.burst")}</p>
        <div className="cols side">
          <div>
            <div className="row">
              <div className="field">
                <label>{t("common.concurrency")}</label>
                <input type="number" value={concurrency} onChange={(e) => setConcurrency(+e.target.value)} />
              </div>
              <div className="field">
                <label>{t("http.total")}</label>
                <input type="number" value={total} onChange={(e) => setTotal(+e.target.value)} />
              </div>
              <div className="field">
                <label>{t("http.duration")}</label>
                <input type="number" value={duration} onChange={(e) => setDuration(+e.target.value)} />
              </div>
            </div>
            <div className="metrics">
              <div className="metric"><div className="k">{t("http.sent")}</div><div className="v">{fmtNum(prog?.sent ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("http.ok")}</div><div className="v accent">{fmtNum(prog?.ok ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("http.failed")}</div><div className="v red">{fmtNum(prog?.failed ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("http.rps")}</div><div className="v accent">{fmtNum(prog?.rps ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("http.avg")}</div><div className="v">{(prog?.avg_latency_ms ?? 0).toFixed(0)}<small>ms</small></div></div>
              <div className="metric">
                <div className="k">{t("http.minMax")}</div>
                <div className="v" style={{ fontSize: 15 }}>
                  {(prog?.min_latency_ms ?? 0).toFixed(0)} / {(prog?.max_latency_ms ?? 0).toFixed(0)}<small>ms</small>
                </div>
              </div>
            </div>
            <div className="btn-row">
              <button className={burstJob ? "danger" : "primary"} onClick={toggleBurst}>
                {burstJob ? t("http.stopBurst") : t("http.startBurst")}
              </button>
            </div>
          </div>
          <div>
            <Scope data={rpsSeries} height={200} color="#6aa9ff" min={0} />
            <div style={{ fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-faint)", marginTop: 8, textAlign: "right" }}>
              {t("http.rpsCaption")}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
