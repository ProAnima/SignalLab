import { useEffect, useMemo, useRef, useState } from "react";
import { api, EV, type CookieInfo, type HttpAuth, type HttpResponse, type JobInfo, type BurstProgress, type Signal, type SignalBody } from "../lib/api";
import { HttpAuthFields } from "../components/HttpAuthFields";
import { SaveSignal, saveShortcut } from "../components/SaveSignal";
import { useStore } from "../lib/store";
import { describeError, responseFailure } from "../lib/errors";
import { ErrorMessage } from "../components/ErrorMessage";
import { useT } from "../lib/i18n";
import { useFieldIds, useJobStream, usePersistentState, useSeries } from "../lib/hooks";
import { fmtBytes, fmtNum, fmtRate, fmtTime, latencyParts, prettyJson, statusClass } from "../lib/format";
import { Scope } from "../components/Scope";
import { MockThis } from "../components/MockThis";
import { HTTP_COOKIES_KEY } from "../lib/signals";

const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

const isHeaders = (value: unknown) => Array.isArray(value)
  && value.every((pair) => Array.isArray(pair) && pair.length === 2 && pair.every((part) => typeof part === "string"));

/** "GET /api/login" for a new signal's name. */
function requestName(method: string, url: string): string {
  try { const parsed = new URL(url); return `${method} ${parsed.pathname}${parsed.search}`; } catch { return `${method} ${url}`; }
}

/**
 * `load`: a stored HTTP signal to edit and send here (from Signals); `at`
 * makes the same one opened twice a new request. `onShowSignal` opens the
 * one this request is saved as.
 */
export function HttpView({ onToExperiment, load, onShowSignal, onShowEmulator }: {
  onToExperiment?: (body: SignalBody) => void;
  load?: { signal: Signal; at: number } | null;
  onShowSignal?: (id: string) => void;
  /** Open the emulator Mock this put a route into. */
  onShowEmulator?: (id: string) => void;
}) {
  const { pushLog, pushError, refreshJobs, stopJob, jobGone, library, info } = useStore();
  const t = useT();
  const fid = useFieldIds();

  // The request is kept across screens and restarts; the response is not.
  const [method, setMethod] = usePersistentState("signal-lab.http.method", "GET", (value) => METHODS.includes(value as string));
  // Loopback, like every default target: a host we do not own is never the default.
  const [url, setUrl] = usePersistentState("signal-lab.http.url", "http://127.0.0.1:8080/");
  const [headers, setHeaders] = usePersistentState<[string, string][]>("signal-lab.http.headers", [["Accept", "application/json"]], isHeaders);
  const [body, setBody] = usePersistentState("signal-lab.http.body", "");
  const [timeout, setTimeoutMs] = usePersistentState("signal-lab.http.timeout", 10000);
  // Credentials stay in memory only: a password is never written to the browser's storage.
  const [auth, setAuth] = useState<HttpAuth>({ scheme: "none" });
  const [keepCookies, setKeepCookies] = usePersistentState(HTTP_COOKIES_KEY, true);
  const [cookies, setCookies] = useState<CookieInfo[]>([]);
  const refreshCookies = () => { api.httpCookies().then(setCookies).catch(() => {}); };
  useEffect(refreshCookies, []);
  // The library signal this request is saved as, if any.
  const [signalId, setSignalId] = usePersistentState<string | null>("signal-lab.http.signal", null, (value) => value === null || typeof value === "string");

  // Credentials are not kept in the browser, so a request bound to a signal takes
  // them back from it when the library arrives — else Save would write them away.
  const authRestored = useRef(false);
  useEffect(() => {
    if (authRestored.current || !signalId) return;
    const bound = library.find((signal) => signal.id === signalId);
    if (!bound) return;
    authRestored.current = true;
    const saved = bound.body.transport === "http" ? bound.body.request.auth : undefined;
    if (saved && saved.scheme !== "none") setAuth((current) => current.scheme === "none" ? saved : current);
  }, [library, signalId]);

  useEffect(() => {
    if (load?.signal.body.transport !== "http") return;
    authRestored.current = true;
    const request = load.signal.body.request;
    setMethod(request.method); setUrl(request.url); setHeaders(request.headers.map(([name, value]) => [name, value]));
    setBody(request.body ?? ""); setTimeoutMs(request.timeout_ms); setAuth(request.auth ?? { scheme: "none" }); setSignalId(load.signal.id);
  }, [load]); // eslint-disable-line react-hooks/exhaustive-deps
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
    ...(auth.scheme === "none" ? {} : { auth }),
  });

  const send = async () => {
    if (busy) return;
    setBusy(true);
    try {
      const r = await api.httpRequest(buildReq(), keepCookies);
      setResp(r);
      if (keepCookies) refreshCookies();
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
  const onRequestKey = (e: React.KeyboardEvent<HTMLElement>) => {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); void send(); }
    saveShortcut(e);
  };
  const onEnterSend = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.ctrlKey && !e.metaKey) { e.preventDefault(); void send(); }
  };

  // ---- burst ----
  const [concurrency, setConcurrency] = useState(20);
  const [total, setTotal] = useState(500);
  const [duration, setDuration] = useState(0);
  const [rate, setRate] = useState(0);
  const [burstJob, setBurstJob] = useState<JobInfo | null>(null);
  // The rate the shown numbers were asked for: editing the field does not relabel them.
  const [burstRate, setBurstRate] = useState(0);
  const [prog, setProg] = useState<BurstProgress | null>(null);
  const { data: rpsSeries, push: pushRps, clear: clearRps } = useSeries(240);
  const latency = (ms: number | undefined) => {
    const [value, unit] = latencyParts(ms ?? 0);
    return <>{value}<small>{t(unit)}</small></>;
  };

  useJobStream<BurstProgress>(EV.burstProgress, burstJob?.id ?? null, (p) => {
    setProg(p);
    // The last report rates the whole burst; the chart keeps the windows.
    if (!p.done) pushRps(p.rps);
    // What the burst's answers set is in the jar now.
    else refreshCookies();
  });

  useEffect(() => {
    if (jobGone(burstJob)) setBurstJob(null);
  }, [jobGone, burstJob]);

  const toggleBurst = async () => {
    if (burstJob) { stopJob(burstJob.id); setBurstJob(null); return; }
    try {
      clearRps();
      setProg(null);
      const job = await api.httpBurstStart({ ...buildReq(), concurrency, total, duration_s: duration, rate, cookies: keepCookies });
      setBurstJob(job);
      setBurstRate(rate);
      if (rate > 0) pushLog("ok", "http", "log.httpBurstPaced", { method, url, workers: concurrency, rate });
      else pushLog("ok", "http", "log.httpBurst", { method, url, workers: concurrency });
      refreshJobs();
    } catch (e) {
      pushError("http", e);
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("http.blurb")}>{t("http.title")}</h1>
      </div>

      <div className="cols side">
        <div className="panel" onKeyDown={onRequestKey}>
          <p className="section-label">{t("http.request")}</p>
          <div className="row" style={{ marginBottom: 12 }}>
            <select style={{ flex: "0 0 110px" }} aria-label={t("exp.method")} value={method} onChange={(e) => setMethod(e.target.value)}>
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
                <button className="ghost sm" style={{ flex: "0 0 auto" }} aria-label={t("exp.removeHeader")} data-tip={t("exp.removeHeader")} onClick={() => setHeaders(headers.filter((_, j) => j !== i))}>✕</button>
              </div>
            ))}
            <button className="ghost sm" onClick={() => setHeaders([...headers, ["", ""]])}>
              {t("http.addHeader")}
            </button>
          </div>
          <HttpAuthFields auth={auth} onChange={setAuth} onKeyDown={onEnterSend} />
          <div className="field">
            <label className="checkbox" data-tip={t("http.keepCookiesHint")}>
              <input type="checkbox" checked={keepCookies} onChange={(e) => setKeepCookies(e.target.checked)} />
              {t("http.keepCookies")}
            </label>
          </div>
          {method !== "GET" && method !== "HEAD" && (
            <div className="field">
              <label htmlFor={fid("body")} data-tip={t("http.bodyHint")}>{t("http.body")}</label>
              <textarea id={fid("body")} value={body} onChange={(e) => setBody(e.target.value)} placeholder='{ "key": "value" }' />
            </div>
          )}
          <div className="field">
            <label htmlFor={fid("timeout")} data-tip={t("common.timeoutHint")}>{t("common.timeoutMs")}</label>
            <input id={fid("timeout")} type="number" value={timeout} onChange={(e) => setTimeoutMs(+e.target.value)} onKeyDown={onEnterSend} />
          </div>
          <div className="btn-row">
            <button className="primary" onClick={send} disabled={busy} data-tip={`${t("common.send")} · Enter / Ctrl+Enter`}>
              {busy ? t("common.sending") : t("common.send")}
            </button>
            {onToExperiment && (
              <button className="ghost" data-tip={t("common.toExperimentHint")}
                onClick={() => onToExperiment({ transport: "http", request: buildReq() })}>
                {t("common.toExperiment")}
              </button>
            )}
          </div>
          <SaveSignal body={{ transport: "http", request: buildReq() }} signalId={signalId} onSignalId={setSignalId}
            suggestName={requestName(method, url)} onShow={onShowSignal} />
          {/* The response panel can sit below the fold on a small window; the verdict never does. */}
          {resp && !busy && (
            <p className={"send-result " + (resp.error ? "err" : resp.ok ? "ok" : "warn")} role="status">
              <span>{resp.error ? `✕ ${describeError(responseFailure(resp, sentUrl), t).text}` : `${resp.status} ${resp.status_text} · ${fmtNum(resp.latency_ms)} ${t("unit.ms")} · ${fmtBytes(resp.body_bytes)}`}</span>
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
                  <div className="v accent">{fmtNum(resp.latency_ms)}<small>{t("unit.ms")}</small></div>
                </div>
                <div className="metric">
                  <div className="k">{t("http.size")}</div>
                  <div className="v">{fmtBytes(resp.body_bytes)}</div>
                </div>
              </div>
              {resp.error && <ErrorMessage className="hint amber" error={responseFailure(resp, sentUrl)} />}
              {resp.digest?.error && <ErrorMessage className="hint amber" error={resp.digest.error} />}
              {resp.digest?.challenged && <p className="http-auth-state"><span className="sig-badge" data-tip={t("http.digestAnsweredHint")}>{t("http.digestAnswered")}</span></p>}
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
              {!resp.error && <span style={{ marginLeft: 6 }}><MockThis method={method} url={sentUrl} response={resp} onShow={onShowEmulator} /></span>}
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
        <div className="row" style={{ alignItems: "center", marginBottom: 8 }}>
          <p className="section-label" style={{ margin: 0 }} data-tip={t(info?.mode === "server" ? "http.cookiesHintServer" : "http.cookiesHint")}>{t("http.cookies", { n: cookies.length })}</p>
          <span style={{ flex: 1 }} />
          <button className="ghost sm" style={{ flex: "0 0 auto" }} disabled={!cookies.length}
            onClick={() => { api.httpCookiesClear().then(refreshCookies).catch((e) => pushError("http", e)); }}>{t("common.clear")}</button>
        </div>
        {cookies.length === 0 ? <div className="empty-state" style={{ padding: 14 }}>{t("http.noCookies")}</div> : (
          <div className="scroll-y" style={{ maxHeight: 180 }}>
            <table className="grid http-cookies"><thead><tr>
              <th>{t("http.cookieName")}</th><th>{t("http.cookieValue")}</th><th>{t("http.cookieWhere")}</th><th>{t("http.cookieExpires")}</th><th>{t("http.cookieFlags")}</th>
            </tr></thead><tbody>
              {cookies.map((cookie) => (
                <tr key={`${cookie.domain}${cookie.path}${cookie.name}`}>
                  <td>{cookie.name}</td>
                  <td className="http-cookie-value">{cookie.value}</td>
                  <td>{cookie.host_only ? cookie.domain : `.${cookie.domain}`}{cookie.path}</td>
                  <td>{cookie.expires === null ? t("http.cookieSession") : new Date(cookie.expires * 1000).toLocaleString()}</td>
                  <td>{[cookie.secure && "Secure", cookie.http_only && "HttpOnly", cookie.same_site && `SameSite=${cookie.same_site}`].filter(Boolean).join(" · ") || "—"}</td>
                </tr>
              ))}
            </tbody></table>
          </div>
        )}
      </div>

      <div className="panel" style={{ marginTop: 16 }}>
        <p className="section-label">{t("http.burst")}</p>
        <div className="cols side">
          <div>
            <div className="row">
              <div className="field">
                <label htmlFor={fid("burst-concurrency")} data-tip={t("common.concurrencyHint")}>{t("common.concurrency")}</label>
                <input id={fid("burst-concurrency")} type="number" value={concurrency} onChange={(e) => setConcurrency(+e.target.value)} />
              </div>
              <div className="field">
                <label htmlFor={fid("burst-total")} data-tip={t("http.totalHint")}>{t("http.total")}</label>
                <input id={fid("burst-total")} type="number" value={total} onChange={(e) => setTotal(+e.target.value)} />
              </div>
              <div className="field">
                <label htmlFor={fid("burst-duration")} data-tip={t("http.durationHint")}>{t("http.duration")}</label>
                <input id={fid("burst-duration")} type="number" value={duration} onChange={(e) => setDuration(+e.target.value)} />
              </div>
              <div className="field">
                <label htmlFor={fid("burst-rate")} data-tip={t("http.rateHint")}>{t("http.rate")}</label>
                <input id={fid("burst-rate")} type="number" min={0} step="any" value={rate} onChange={(e) => setRate(+e.target.value)} />
              </div>
            </div>
            <div className="metrics">
              <div className="metric"><div className="k">{t("http.sent")}</div><div className="v">{fmtNum(prog?.sent ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("http.ok")}</div><div className="v accent">{fmtNum(prog?.ok ?? 0)}</div></div>
              <div className="metric"><div className="k">{t("http.failed")}</div><div className="v red">{fmtNum(prog?.failed ?? 0)}</div></div>
              {burstRate > 0 && (
                <div className="metric" data-tip={t("http.missedHint")}>
                  <div className="k">{t("http.missed")}</div>
                  <div className={"v" + ((prog?.missed ?? 0) > 0 ? " amber" : "")}>{fmtNum(prog?.missed ?? 0)}</div>
                </div>
              )}
              <div className="metric" data-tip={t(prog?.done ? "http.rpsWholeHint" : "http.rpsCaption")}>
                <div className="k">{burstRate > 0 ? t("http.rpsOf", { rate: fmtRate(burstRate) }) : t("http.rps")}</div>
                <div className="v accent">{fmtNum(prog?.rps ?? 0)}</div>
              </div>
            </div>
            <div className="btn-row">
              <button className={burstJob ? "danger" : "primary"} onClick={toggleBurst}>
                {burstJob ? t("http.stopBurst") : t("http.startBurst")}
              </button>
            </div>
          </div>
          <div data-tip={t("http.rpsCaption")}>
            <Scope data={rpsSeries} height={200} color="#6aa9ff" min={0} />
          </div>
        </div>
        {/* Latency across the panel: the distribution read left to right, never wrapped inside a card. */}
        <div className="metrics latency">
          {(["p50", "p90", "p95", "p99"] as const).map((p) => (
            <div className="metric" key={p} data-tip={t("http.percentileHint", { p: p.slice(1) })}>
              <div className="k">{t(`http.${p}`)}</div>
              <div className="v">{latency(prog?.[`${p}_ms`])}</div>
            </div>
          ))}
          <div className="metric"><div className="k">{t("http.avg")}</div><div className="v">{latency(prog?.avg_latency_ms)}</div></div>
          <div className="metric"><div className="k">{t("http.min")}</div><div className="v">{latency(prog?.min_latency_ms)}</div></div>
          <div className="metric"><div className="k">{t("http.max")}</div><div className="v">{latency(prog?.max_latency_ms)}</div></div>
        </div>
      </div>
    </div>
  );
}
