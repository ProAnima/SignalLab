import { useEffect, useMemo, useState } from "react";
import { api, EV, type HttpResponse, type JobInfo, type BurstProgress, type SignalBody } from "../lib/api";
import { useStore } from "../lib/store";
import { describeError, responseFailure } from "../lib/errors";
import { ErrorMessage } from "../components/ErrorMessage";
import { useT } from "../lib/i18n";
import { useJobStream, usePersistentState, useSeries } from "../lib/hooks";
import { fmtBytes, fmtNum, fmtTime, prettyJson, statusClass } from "../lib/format";
import { Scope } from "../components/Scope";

const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

const isHeaders = (value: unknown) => Array.isArray(value)
  && value.every((pair) => Array.isArray(pair) && pair.length === 2 && pair.every((part) => typeof part === "string"));

export function HttpView({ onToExperiment }: { onToExperiment?: (body: SignalBody) => void }) {
  const { pushLog, pushError, refreshJobs, stopJob, jobGone } = useStore();
  const t = useT();

  // The request is kept across screens and restarts; the response is not.
  const [method, setMethod] = usePersistentState("signal-lab.http.method", "GET", (value) => METHODS.includes(value as string));
  const [url, setUrl] = usePersistentState("signal-lab.http.url", "https://httpbin.org/get");
  const [headers, setHeaders] = usePersistentState<[string, string][]>("signal-lab.http.headers", [["Accept", "application/json"]], isHeaders);
  const [body, setBody] = usePersistentState("signal-lab.http.body", "");
  const [timeout, setTimeoutMs] = usePersistentState("signal-lab.http.timeout", 10000);
  const [busy, setBusy] = useState(false);
  const [resp, setResp] = useState<HttpResponse | null>(null);
  const [sentAt, setSentAt] = useState(0);
  // The URL the shown response is for; the field may have changed since.
  const [sentUrl, setSentUrl] = useState("");
  const [showHeaders, setShowHeaders] = useState(false);
  const [raw, setRaw] = useState(false);
  const pretty = useMemo(() => (resp ? prettyJson(resp.body) : null), [resp]);

  const buildReq = () => ({
    method, url,
    headers: headers.filter(([k]) => k.trim()),
    body: body.length ? body : null,
    timeout_ms: timeout,
  });

  const send = async () => {
    if (busy) return;
    setBusy(true);
    try {
      const r = await api.httpRequest(buildReq());
      setResp(r);
      setSentAt(Date.now());
      setSentUrl(url);
      if (r.error) {
        pushError("http", responseFailure(r, url), "log.httpFailed", { method, url });
      } else {
        pushLog(r.ok ? "ok" : "warn", "http", "log.httpDone", {
          method, url, status: r.status, ms: r.latency_ms.toFixed(0),
        });
      }
    } catch (e) {
      pushError("http", e);
      setResp({ ok: false, status: 0, status_text: "", latency_ms: 0, headers: [], body: "", body_bytes: 0, truncated: false, error: describeError(e, t).text });
      setSentAt(Date.now());
      setSentUrl(url);
    } finally {
      setBusy(false);
    }
  };

  // Ctrl+Enter sends from any field of the request, the body included.
  const onRequestKey = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); void send(); }
  };
  const onEnterSend = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.ctrlKey && !e.metaKey) { e.preventDefault(); void send(); }
  };

  // ---- burst ----
  const [concurrency, setConcurrency] = useState(20);
  const [total, setTotal] = useState(500);
  const [duration, setDuration] = useState(0);
  const [burstJob, setBurstJob] = useState<JobInfo | null>(null);
  const [prog, setProg] = useState<BurstProgress | null>(null);
  const { data: rpsSeries, push: pushRps, clear: clearRps } = useSeries(240);

  useJobStream<BurstProgress>(EV.burstProgress, burstJob?.id ?? null, (p) => {
    setProg(p);
    pushRps(p.rps);
  });

  useEffect(() => {
    if (jobGone(burstJob)) setBurstJob(null);
  }, [jobGone, burstJob]);

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
      pushError("http", e);
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1>{t("http.title")}</h1>
        <p>{t("http.blurb")}</p>
      </div>

      <div className="cols side">
        <div className="panel" onKeyDown={onRequestKey}>
          <p className="section-label">{t("http.request")}</p>
          <div className="row" style={{ marginBottom: 12 }}>
            <select style={{ flex: "0 0 110px" }} value={method} onChange={(e) => setMethod(e.target.value)}>
              {METHODS.map((m) => <option key={m}>{m}</option>)}
            </select>
            <input
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              onKeyDown={onEnterSend}
              placeholder="https://..."
              aria-label="URL"
            />
          </div>
          <div className="field">
            <label>{t("http.headers")}</label>
            {headers.map((h, i) => (
              <div className="row tight" key={i} style={{ marginBottom: 6 }}>
                <input
                  placeholder={t("http.headerName")}
                  aria-label={t("http.headerName")}
                  onKeyDown={onEnterSend}
                  value={h[0]}
                  onChange={(e) => setHeaders(headers.map((x, j) => j === i ? [e.target.value, x[1]] : x))}
                />
                <input
                  placeholder={t("http.headerValue")}
                  aria-label={t("http.headerValue")}
                  onKeyDown={onEnterSend}
                  value={h[1]}
                  onChange={(e) => setHeaders(headers.map((x, j) => j === i ? [x[0], e.target.value] : x))}
                />
                <button className="ghost sm" style={{ flex: "0 0 auto" }} aria-label={t("exp.removeHeader")} onClick={() => setHeaders(headers.filter((_, j) => j !== i))}>✕</button>
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
            <input type="number" value={timeout} onChange={(e) => setTimeoutMs(+e.target.value)} onKeyDown={onEnterSend} />
          </div>
          <div className="btn-row">
            <button className="primary" onClick={send} disabled={busy} title={`${t("common.send")} · Enter / Ctrl+Enter`}>
              {busy ? t("common.sending") : t("common.send")}
            </button>
            {onToExperiment && (
              <button className="ghost" title={t("common.toExperimentHint")}
                onClick={() => onToExperiment({ transport: "http", request: buildReq() })}>
                {t("common.toExperiment")}
              </button>
            )}
          </div>
          {/* The response panel can sit below the fold on a small window; the verdict never does. */}
          {resp && !busy && (
            <p className={"send-result " + (resp.error ? "err" : resp.ok ? "ok" : "warn")} role="status">
              <span>{resp.error ? `✕ ${describeError(responseFailure(resp, sentUrl), t).text}` : `${resp.status} ${resp.status_text} · ${resp.latency_ms.toFixed(0)} ms · ${fmtBytes(resp.body_bytes)}`}</span>
              <time>{fmtTime(sentAt).slice(0, 8)}</time>
            </p>
          )}
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
              {resp.error && <ErrorMessage className="hint amber" error={responseFailure(resp, sentUrl)} />}
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
              {pretty && (
                <button className="ghost sm" style={{ marginBottom: 8, marginLeft: 6 }} aria-pressed={!raw} onClick={() => setRaw(!raw)}>
                  {raw ? t("http.formatJson") : t("http.rawBody")}
                </button>
              )}
              <textarea
                readOnly
                aria-label={t("http.response")}
                value={(pretty && !raw ? pretty : resp.body) + (resp.truncated ? "\n" + t("http.truncated") : "")}
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
