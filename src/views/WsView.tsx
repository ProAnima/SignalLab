import { useEffect, useMemo, useRef, useState } from "react";
import { api, EV, type JobInfo, type WsBatch, type WsClosed, type WsHandshake, type WsMessage, type WsStateEvent } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { useFieldIds, useJobStream, usePersistentState } from "../lib/hooks";
import { fmtBytes, fmtNum, fmtTime } from "../lib/format";
import { asJson, protocolsOf, protocolsText } from "../lib/websocket";

/** Messages the screen keeps; older ones go (the Inspector has them while capture is armed). */
const KEPT = 2000;
/** Characters a row of the list shows; the whole message is in its detail. */
const ROW_TEXT = 300;

type Shown = WsMessage & { n: number };


/**
 * A WebSocket client: connect with headers and subprotocols, send text or
 * bytes, read what comes back — as it comes, newest last. One connection, a
 * job like any other, so the console strip can stop it.
 */
export function WsView() {
  const { pushLog, pushError, jobGone, refreshJobs } = useStore();
  const t = useT();
  const fid = useFieldIds();

  const [url, setUrl] = usePersistentState("signal-lab.ws.url", "ws://127.0.0.1:9001/");
  const [headers, setHeaders] = useState<[string, string][]>([]);
  const [protocols, setProtocols] = useState("");
  const [job, setJob] = useState<JobInfo | null>(null);
  const [handshake, setHandshake] = useState<WsHandshake | null>(null);
  const [closed, setClosed] = useState<WsClosed | null>(null);
  const [connecting, setConnecting] = useState(false);

  const [binary, setBinary] = useState(false);
  const [draft, setDraft] = usePersistentState("signal-lab.ws.draft", '{"type":"ping"}');
  const [messages, setMessages] = useState<Shown[]>([]);
  const [dropped, setDropped] = useState(0);
  const [picked, setPicked] = useState<number | null>(null);
  const counter = useRef(0);
  const list = useRef<HTMLDivElement>(null);
  // Whether the list is scrolled to its end, as the person left it: then it follows new messages.
  const pinned = useRef(true);

  useJobStream<WsStateEvent>(EV.wsState, job?.id ?? null, (event) => {
    setHandshake(event.handshake);
    if (event.state !== "closed" || !event.closed) return;
    const how = event.closed;
    setClosed(how);
    setJob(null);
    if (how.by === "lost" && how.error) pushError("ws", how.error, "log.wsLost", { url: event.handshake.url });
    else if (how.by === "server") pushLog("warn", "ws", "log.wsClosedByServer", { url: event.handshake.url, code: how.code });
    else pushLog("ok", "ws", "log.wsClosed", { url: event.handshake.url, code: how.code });
  });

  useJobStream<WsBatch>(EV.wsMessages, job?.id ?? null, (batch) => {
    const added = batch.messages.map((message) => ({ ...message, n: ++counter.current }));
    setMessages((prior) => {
      const next = prior.concat(added);
      return next.length > KEPT ? next.slice(next.length - KEPT) : next;
    });
    if (batch.dropped) setDropped((prior) => prior + batch.dropped);
  });

  // Newest last, followed while the list was scrolled to its end before they came.
  useEffect(() => {
    const element = list.current;
    if (element && pinned.current) element.scrollTop = element.scrollHeight;
  }, [messages]);

  // Stopped from the console strip.
  useEffect(() => {
    if (jobGone(job)) setJob(null);
  }, [jobGone, job]);

  const connect = async () => {
    if (connecting) return;
    if (job) {
      try { await api.wsClose(job.id); } catch (e) { pushError("ws", e); }
      return;
    }
    setConnecting(true);
    setClosed(null);
    try {
      const started = await api.wsConnect({ url: url.trim(), headers: headers.filter(([name]) => name.trim()), protocols: protocolsOf(protocols), timeout_ms: 10_000 });
      setJob(started);
      pushLog("ok", "ws", "log.wsConnected", { url: url.trim() });
      refreshJobs();
    } catch (e) {
      pushError("ws", e);
    } finally {
      setConnecting(false);
    }
  };

  const send = async () => {
    if (!job) { pushLog("err", "ws", "log.wsNotConnected"); return; }
    try {
      await api.wsSend(job.id, binary ? { hex: draft } : { text: draft });
    } catch (e) {
      pushError("ws", e);
    }
  };

  const counts = useMemo(() => ({
    rx: messages.filter((message) => message.dir === "rx").length,
    tx: messages.filter((message) => message.dir === "tx").length,
  }), [messages]);
  const chosen = messages.find((message) => message.n === picked) ?? null;
  const chosenJson = chosen && chosen.kind === "text" ? asJson(chosen.text) : null;

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("ws.blurb")}>{t("ws.title")}</h1>
      </div>

      <div className="cols side">
        <div>
          <div className="panel">
            <p className="section-label">{t("ws.connection")}</p>
            <div className="field">
              <label htmlFor={fid("url")} data-tip={t("ws.urlHint")}>URL</label>
              <input id={fid("url")} value={url} spellCheck={false} disabled={!!job} placeholder="ws://127.0.0.1:9001/"
                onChange={(e) => setUrl(e.target.value)} onKeyDown={(e) => { if (e.key === "Enter" && !job) void connect(); }} />
            </div>
            <div className="field">
              <label htmlFor={fid("protocols")} data-tip={t("exp.wsProtocolsHint")}>{t("exp.wsProtocols")}</label>
              <input id={fid("protocols")} value={protocols} spellCheck={false} disabled={!!job} placeholder="graphql-transport-ws"
                onChange={(e) => setProtocols(e.target.value)} onBlur={() => setProtocols(protocolsText(protocolsOf(protocols)))} />
            </div>
            <p className="section-label">{t("http.headers")}</p>
            {headers.map(([name, value], index) => (
              <div className="row tight" key={index} style={{ marginBottom: 6 }}>
                <input aria-label={t("http.headerName")} placeholder={t("http.headerName")} value={name} disabled={!!job}
                  onChange={(e) => setHeaders(headers.map((pair, i) => i === index ? [e.target.value, pair[1]] : pair))} />
                <input aria-label={t("http.headerValue")} placeholder={t("http.headerValue")} value={value} disabled={!!job}
                  onChange={(e) => setHeaders(headers.map((pair, i) => i === index ? [pair[0], e.target.value] : pair))} />
                <button className="ghost sm" style={{ flex: "0 0 auto" }} aria-label={t("exp.removeHeader")} data-tip={t("exp.removeHeader")} disabled={!!job}
                  onClick={() => setHeaders(headers.filter((_, i) => i !== index))}>×</button>
              </div>
            ))}
            <button className="ghost sm" disabled={!!job} onClick={() => setHeaders([...headers, ["", ""]])}>{t("http.addHeader")}</button>
            <div className="btn-row">
              <button className={job ? "danger" : "primary"} disabled={connecting} onClick={connect}>
                {connecting ? t("ws.connecting") : job ? t("ws.disconnect") : t("ws.connect")}
              </button>
            </div>
            {handshake && (job || closed) && (
              <dl className="kv" style={{ marginTop: 12 }}>
                <dt>{t("ws.state")}</dt>
                <dd className={job ? "ok" : undefined}>{job ? t("ws.open") : closed?.by === "server" ? t("ws.closedByServer", { code: closed.code })
                  : closed?.by === "lost" ? t("ws.lost") : t("ws.closedByYou", { code: closed?.code ?? 1000 })}</dd>
                {closed?.reason && <><dt>{t("field.reason")}</dt><dd>{closed.reason}</dd></>}
                <dt>{t("ws.subprotocol")}</dt><dd>{handshake.protocol ?? "—"}</dd>
                <dt>{t("ws.peer")}</dt><dd>{handshake.peer}</dd>
                <dt>{t("ws.upgradeTime")}</dt><dd>{fmtNum(handshake.ms)} {t("unit.ms")}</dd>
              </dl>
            )}
          </div>

          <div className="panel" style={{ marginTop: 16 }}>
            <p className="section-label">{t("ws.message")}</p>
            <div className="chips" role="group" aria-label={t("exp.wsFormat")} style={{ marginBottom: 8 }}>
              <button className={"chip" + (!binary ? " on" : "")} aria-pressed={!binary} onClick={() => setBinary(false)}>{t("exp.wsText")}</button>
              <button className={"chip" + (binary ? " on" : "")} aria-pressed={binary} data-tip={t("exp.wsFormatHint")} onClick={() => setBinary(true)}>{t("exp.wsBinary")}</button>
            </div>
            <textarea aria-label={t("ws.message")} rows={5} spellCheck={false} value={draft} placeholder={binary ? "de ad be ef" : '{"type":"ping"}'}
              onChange={(e) => setDraft(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); void send(); } }} />
            <div className="btn-row">
              <button className="primary" disabled={!job} data-tip={t("ws.sendHint")} onClick={send}>{t("common.send")}</button>
              {!binary && asJson(draft) !== null && <button className="ghost" onClick={() => setDraft(asJson(draft) ?? draft)}>{t("http.formatJson")}</button>}
            </div>
          </div>
        </div>

        <div>
          <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 360 }}>
            <div className="row" style={{ alignItems: "center", marginBottom: 8 }}>
              <p className="section-label" style={{ margin: 0 }}>{t("ws.messages")}</p>
              <span className="spacer" style={{ flex: 1 }} />
              <button className="ghost sm" style={{ flex: "0 0 auto" }} disabled={!messages.length}
                onClick={() => { setMessages([]); setDropped(0); setPicked(null); }}>{t("common.clear")}</button>
            </div>
            <div className="mq-stats">
              <span>{t("ws.received", { n: counts.rx })}</span>
              <span>{t("ws.sent", { n: counts.tx })}</span>
              {dropped > 0 && <span className="warn">{t("ws.dropped", { n: dropped })}</span>}
            </div>
            <div className="ws-list scroll-y" ref={list}
              onScroll={(e) => { const element = e.currentTarget; pinned.current = element.scrollHeight - element.scrollTop - element.clientHeight < 24; }}>
              {messages.map((message) => (
                <button key={message.n} className={"ws-row " + message.dir + (picked === message.n ? " active" : "")} onClick={() => setPicked(message.n)}>
                  <span className="ws-time">{fmtTime(message.ts)}</span>
                  <span className="ws-dir" aria-label={t(message.dir === "rx" ? "ws.in" : "ws.out")}>{message.dir === "rx" ? "↓" : "↑"}</span>
                  <span className="ws-text">{(message.kind === "binary" ? message.hex ?? "" : message.text).slice(0, ROW_TEXT)}</span>
                  <span className="ws-size">{message.kind === "binary" ? `${t("ws.binary")} · ` : ""}{fmtBytes(message.bytes)}</span>
                </button>
              ))}
              {messages.length === 0 && <div className="empty-state">{job ? t("ws.waiting") : t("ws.notConnected")}</div>}
            </div>
          </div>

          {chosen && (
            <div className="panel" style={{ marginTop: 16 }}>
              <p className="section-label">{t(chosen.dir === "rx" ? "ws.in" : "ws.out")} · {fmtTime(chosen.ts)}</p>
              <pre className="ws-detail">{chosen.kind === "binary" ? chosen.hex : chosenJson ?? chosen.text}</pre>
              {chosen.truncated && <p className="experiment-empty">{t("ws.truncated", { size: fmtBytes(chosen.bytes) })}</p>}
              {/* A cut message would be sent cut: only a whole one is offered as a draft. */}
              {!chosen.truncated && <div className="btn-row">
                <button className="ghost sm" onClick={() => { setBinary(chosen.kind === "binary"); setDraft(chosen.kind === "binary" ? chosen.hex ?? "" : chosen.text); }}>{t("ws.editHere")}</button>
              </div>}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
