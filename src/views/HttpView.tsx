import { useEffect, useRef, useState } from "react";
import { api, on, EV, type HttpResponse, type JobInfo, type BurstProgress } from "../lib/api";
import { useStore } from "../lib/store";
import { useSeries } from "../lib/hooks";
import { fmtBytes, fmtNum, statusClass } from "../lib/format";
import { Scope } from "../components/Scope";

const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

export function HttpView() {
  const { pushLog, refreshJobs, stopJob, jobs } = useStore();

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
      if (r.error) pushLog("err", "http", `${method} ${url} — ${r.error}`);
      else pushLog(r.ok ? "ok" : "warn", "http", `${method} ${url} → ${r.status} in ${r.latency_ms.toFixed(0)}ms`);
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
      pushLog("ok", "http", `burst ${method} ${url} · ${concurrency} workers`);
      refreshJobs();
    } catch (e) {
      pushLog("err", "http", String(e));
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1>HTTP</h1>
        <p>Inspect a single request/response, then hammer the same endpoint with a concurrent load burst to measure throughput and latency.</p>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">Request</p>
          <div className="row" style={{ marginBottom: 12 }}>
            <select style={{ flex: "0 0 110px" }} value={method} onChange={(e) => setMethod(e.target.value)}>
              {METHODS.map((m) => <option key={m}>{m}</option>)}
            </select>
            <input value={url} onChange={(e) => setUrl(e.target.value)} placeholder="https://..." />
          </div>
          <div className="field">
            <label>Headers</label>
            {headers.map((h, i) => (
              <div className="row tight" key={i} style={{ marginBottom: 6 }}>
                <input placeholder="Name" value={h[0]} onChange={(e) => setHeaders(headers.map((x, j) => j === i ? [e.target.value, x[1]] : x))} />
                <input placeholder="Value" value={h[1]} onChange={(e) => setHeaders(headers.map((x, j) => j === i ? [x[0], e.target.value] : x))} />
                <button className="ghost sm" style={{ flex: "0 0 auto" }} onClick={() => setHeaders(headers.filter((_, j) => j !== i))}>✕</button>
              </div>
            ))}
            <button className="ghost sm" onClick={() => setHeaders([...headers, ["", ""]])}>+ header</button>
          </div>
          {method !== "GET" && method !== "HEAD" && (
            <div className="field">
              <label>Body</label>
              <textarea value={body} onChange={(e) => setBody(e.target.value)} placeholder='{ "key": "value" }' />
            </div>
          )}
          <div className="field">
            <label>Timeout (ms)</label>
            <input type="number" value={timeout} onChange={(e) => setTimeoutMs(+e.target.value)} />
          </div>
          <div className="btn-row">
            <button className="primary" onClick={send} disabled={busy}>{busy ? "Sending…" : "Send"}</button>
          </div>
        </div>

        <div className="panel" style={{ minHeight: 320 }}>
          <p className="section-label">Response</p>
          {!resp && <div style={{ color: "var(--text-faint)", padding: 24, textAlign: "center" }}>No response yet.</div>}
          {resp && (
            <>
              <div className="metrics" style={{ marginBottom: 14 }}>
                <div className="metric">
                  <div className="k">Status</div>
                  <div className={"v " + statusClass(resp.status)}>{resp.status || "ERR"} <small>{resp.status_text}</small></div>
                </div>
                <div className="metric"><div className="k">Latency</div><div className="v accent">{resp.latency_ms.toFixed(0)}<small>ms</small></div></div>
                <div className="metric"><div className="k">Size</div><div className="v">{fmtBytes(resp.body_bytes)}</div></div>
              </div>
              {resp.error && <div className="hint amber" style={{ marginBottom: 12 }}>{resp.error}</div>}
              <button className="ghost sm" onClick={() => setShowHeaders(!showHeaders)} style={{ marginBottom: 8 }}>
                {showHeaders ? "▾" : "▸"} {resp.headers.length} response headers
              </button>
              {showHeaders && (
                <div className="scroll-y" style={{ maxHeight: 140, marginBottom: 10, border: "1px solid var(--border)", borderRadius: 6 }}>
                  <table className="grid"><tbody>
                    {resp.headers.map(([k, v], i) => (
                      <tr key={i}><td style={{ color: "var(--cyan)", width: 200 }}>{k}</td><td>{v}</td></tr>
                    ))}
                  </tbody></table>
                </div>
              )}
              <textarea readOnly value={resp.body + (resp.truncated ? "\n… (truncated)" : "")} style={{ minHeight: 160, fontSize: 11 }} />
            </>
          )}
        </div>
      </div>

      <div className="panel" style={{ marginTop: 16 }}>
        <p className="section-label">Load burst</p>
        <div className="cols side">
          <div>
            <div className="row">
              <div className="field"><label>Concurrency</label><input type="number" value={concurrency} onChange={(e) => setConcurrency(+e.target.value)} /></div>
              <div className="field"><label>Total (0 = timed)</label><input type="number" value={total} onChange={(e) => setTotal(+e.target.value)} /></div>
              <div className="field"><label>Duration (s, 0 = by total)</label><input type="number" value={duration} onChange={(e) => setDuration(+e.target.value)} /></div>
            </div>
            <div className="metrics">
              <div className="metric"><div className="k">Sent</div><div className="v">{fmtNum(prog?.sent ?? 0)}</div></div>
              <div className="metric"><div className="k">OK</div><div className="v accent">{fmtNum(prog?.ok ?? 0)}</div></div>
              <div className="metric"><div className="k">Failed</div><div className="v red">{fmtNum(prog?.failed ?? 0)}</div></div>
              <div className="metric"><div className="k">RPS</div><div className="v accent">{fmtNum(prog?.rps ?? 0)}</div></div>
              <div className="metric"><div className="k">Avg</div><div className="v">{(prog?.avg_latency_ms ?? 0).toFixed(0)}<small>ms</small></div></div>
              <div className="metric"><div className="k">Min / Max</div><div className="v" style={{ fontSize: 14 }}>{(prog?.min_latency_ms ?? 0).toFixed(0)} / {(prog?.max_latency_ms ?? 0).toFixed(0)}<small>ms</small></div></div>
            </div>
            <div className="btn-row">
              <button className={burstJob ? "danger" : "primary"} onClick={toggleBurst}>{burstJob ? "Stop burst" : "Start burst"}</button>
            </div>
          </div>
          <div>
            <Scope data={rpsSeries} height={200} color="#5b9dff" min={0} />
            <div style={{ fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-faint)", marginTop: 6, textAlign: "right" }}>
              requests / second
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
