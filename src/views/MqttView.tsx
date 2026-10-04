import { useEffect, useMemo, useRef, useState } from "react";
import {
  api, on, EV,
  type ExperimentNode, type JobInfo, type MqttAck, type MqttConfig, type MqttGrant, type MqttStateEvent, type Signal,
} from "../lib/api";
import { SaveSignal, saveShortcut } from "../components/SaveSignal";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { fmtNum, fmtTime } from "../lib/format";
import { signalFromMqttTopic, splitBroker } from "../lib/signals";
import { waitForMqttMessage } from "../lib/experimentGraph";
import { flattenTopics, sortedChildren, type TopicNode } from "../lib/topics";

const QOS = [0, 1, 2];

function defaultClientId(): string {
  return `signal-lab-${Math.random().toString(16).slice(2, 8)}`;
}

/** `load`: a stored MQTT signal to publish from here; `onShowSignal` opens the one the publisher is saved as. */
export function MqttView({ onWaitFor, load, onShowSignal }: {
  onWaitFor?: (node: ExperimentNode) => void;
  load?: { signal: Signal; at: number } | null;
  onShowSignal?: (id: string) => void;
} = {}) {
  const {
    pushLog, pushError, jobGone, stopJob, refreshJobs, library, setLibrary,
    mqttTopics, mqttVersion, mqttDropped, clearMqttTopics,
  } = useStore();
  const t = useT();

  const [cfg, setCfg] = useState({
    host: "127.0.0.1",
    port: 1883,
    client_id: defaultClientId(),
    username: "",
    password: "",
    keep_alive_s: 60,
    clean_session: true,
  });
  const [willOn, setWillOn] = useState(false);
  const [will, setWill] = useState({ topic: "", payload: "off", qos: 2, retain: true });
  const [scanFilter, setScanFilter] = useState("#");
  const [scanQos, setScanQos] = useState(0);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [grants, setGrants] = useState<MqttGrant[]>([]);
  const jobRef = useRef<number | null>(null);
  jobRef.current = job?.id ?? null;

  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [picked, setPicked] = useState<string | null>(null);
  const [query, setQuery] = useState("");

  const [pub, setPub] = useState({ topic: "", payload: "", qos: 0, retain: false });
  const [signalId, setSignalId] = useState<string | null>(null);
  const [confirmClear, setConfirmClear] = useState<string | null>(null);
  const [sub, setSub] = useState({ filter: "", qos: 0 });

  // A stored signal opened here: its message, and its broker while not connected elsewhere.
  useEffect(() => {
    if (load?.signal.body.transport !== "mqtt") return;
    const { broker, topic, payload, qos, retain } = load.signal.body;
    setPub({ topic, payload, qos, retain });
    if (!job) { const { host, port } = splitBroker(broker); setCfg((prior) => ({ ...prior, host, port })); }
    setSignalId(load.signal.id);
  }, [load]); // eslint-disable-line react-hooks/exhaustive-deps

  // Messages are ingested by the store, so the tree survives leaving this tab.
  useEffect(() => {
    const uns = [
      on<MqttStateEvent>(EV.mqttState, (e) => {
        const s = e.payload;
        if (jobRef.current !== null && s.job_id !== jobRef.current) return;
        if (s.state === "subscribed") {
          setGrants((prev) => {
            const next = prev.filter((g) => !s.grants.some((n) => n.filter === g.filter));
            return [...next, ...s.grants];
          });
          for (const g of s.grants) {
            if (g.accepted) pushLog("ok", "mqtt", "log.mqttSubscribed", { filter: g.filter, qos: g.qos });
            else pushLog("err", "mqtt", "log.mqttSubRefused", { filter: g.filter });
          }
        }
        if (s.state === "closed") {
          setJob(null);
          setGrants([]);
          if (s.error) pushError("mqtt", s.error, "log.mqttClosed", { broker: s.broker });
        }
      }),
      on<MqttAck>(EV.mqttAck, (e) => {
        const a = e.payload;
        if (jobRef.current !== null && a.job_id !== jobRef.current) return;
        if (a.kind === "published" && a.topic) {
          pushLog("ok", "mqtt", "log.mqttPublished", { topic: a.topic });
        }
      }),
    ];
    return () => { uns.forEach((u) => u.then((f) => f())); };
  }, [pushLog]);

  // Reflect a stop from the console strip.
  useEffect(() => {
    if (jobGone(job)) { setJob(null); setGrants([]); }
  }, [jobGone, job]);

  const connect = async () => {
    if (job) { stopJob(job.id); return; }
    const config: MqttConfig = {
      ...cfg,
      will: willOn && will.topic.trim() ? will : null,
      subscribe: scanFilter.trim() ? [{ filter: scanFilter.trim(), qos: scanQos }] : [],
    };
    try {
      const started = await api.mqttConnect(config);
      setJob(started);
      pushLog("ok", "mqtt", "log.mqttConnected", { broker: `${cfg.host}:${cfg.port}`, id: cfg.client_id });
      refreshJobs();
    } catch (e) {
      pushError("mqtt", e);
    }
  };

  const publish = async (topic: string, payload: string, qos: number, retain: boolean) => {
    if (!job) { pushLog("err", "mqtt", "log.mqttNotConnected"); return; }
    if (!topic.trim()) { pushLog("err", "mqtt", "log.mqttNoTopic"); return; }
    try {
      await api.mqttPublish(job.id, topic, payload, qos, retain);
      // QoS 0 gets no acknowledgement, so this is the only line it will produce.
      if (qos === 0) pushLog("ok", "mqtt", "log.mqttPublished", { topic });
    } catch (e) {
      pushError("mqtt", e);
    }
  };

  const subscribe = async () => {
    if (!job || !sub.filter.trim()) return;
    try {
      await api.mqttSubscribe(job.id, [{ filter: sub.filter.trim(), qos: sub.qos }]);
    } catch (e) {
      pushError("mqtt", e);
    }
  };

  const unsubscribe = async (filter: string) => {
    if (!job) return;
    try {
      await api.mqttUnsubscribe(job.id, [filter]);
      setGrants((prev) => prev.filter((g) => g.filter !== filter));
      pushLog("warn", "mqtt", "log.mqttUnsubscribed", { filter });
    } catch (e) {
      pushError("mqtt", e);
    }
  };

  const clearTree = () => {
    clearMqttTopics();
    setPicked(null);
  };

  const all = useMemo(
    () => flattenTopics(mqttTopics.current),
    [mqttVersion, mqttTopics] // eslint-disable-line react-hooks/exhaustive-deps
  );
  const found = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return null;
    return all.filter((n) => n.path.toLowerCase().includes(q)
      || (n.last?.payload ?? "").toLowerCase().includes(q));
  }, [all, query]);

  const pickedNode = picked ? all.find((n) => n.path === picked) ?? null : null;
  const retainedCount = all.filter((n) => n.last?.retain).length;

  const toggle = (path: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path); else next.add(path);
      return next;
    });
  };

  const saveTopicAsSignal = (node: TopicNode) => {
    const signal = signalFromMqttTopic(
      `${cfg.host}:${cfg.port}`,
      node.path,
      node.last?.payload ?? "",
      node.last?.qos ?? 0,
      node.last?.retain ?? false,
      library,
      t,
    );
    setLibrary([...library, signal]);
    pushLog("ok", "mqtt", "log.mqttSaved", { topic: node.path, name: signal.name });
  };

  const row = (node: TopicNode, depth: number): React.ReactNode => {
    const kids = sortedChildren(node);
    const open = expanded.has(node.path);
    return (
      <div key={node.path}>
        <button
          className={"topic-row" + (picked === node.path ? " active" : "")}
          style={{ paddingInlineStart: 8 + depth * 14 }}
          onClick={() => { setPicked(node.last ? node.path : picked); if (kids.length) toggle(node.path); }}
        >
          <span className="topic-twist">{kids.length ? (open ? "▾" : "▸") : "·"}</span>
          <span className="topic-name">{node.name}</span>
          {node.last?.retain && <span className="sig-badge">R</span>}
          {node.last && <span className="topic-value">{node.last.payload || "—"}</span>}
          {node.count > 1 && <span className="topic-count">{fmtNum(node.count)}</span>}
        </button>
        {open && kids.map((k) => row(k, depth + 1))}
      </div>
    );
  };

  const roots = sortedChildren(mqttTopics.current);

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("mq.blurb")}>{t("mq.title")}</h1>
      </div>

      <div className="cols side">
        {/* ---- connection ---- */}
        <div className="panel">
          <p className="section-label">{t("mq.connection")}</p>
          <div className="row">
            <div className="field">
              <label htmlFor="mq-host">{t("mq.host")}</label>
              <input id="mq-host" value={cfg.host} disabled={!!job}
                onChange={(e) => setCfg({ ...cfg, host: e.target.value })} />
            </div>
            <div className="field" style={{ flex: "0 0 92px" }}>
              <label htmlFor="mq-port">{t("common.port")}</label>
              <input id="mq-port" type="number" value={cfg.port} disabled={!!job}
                onChange={(e) => setCfg({ ...cfg, port: +e.target.value })} />
            </div>
          </div>
          <div className="field">
            <label htmlFor="mq-id" data-tip={t("mq.clientIdHint")}>{t("mq.clientId")}</label>
            <input id="mq-id" value={cfg.client_id} disabled={!!job}
              onChange={(e) => setCfg({ ...cfg, client_id: e.target.value })} />
          </div>
          <div className="row">
            <div className="field">
              <label htmlFor="mq-user" data-tip={t("mq.credentialsHint")}>{t("mq.username")}</label>
              <input id="mq-user" value={cfg.username} disabled={!!job}
                onChange={(e) => setCfg({ ...cfg, username: e.target.value })} />
            </div>
            <div className="field">
              <label htmlFor="mq-pass" data-tip={t("mq.credentialsHint")}>{t("mq.password")}</label>
              <input id="mq-pass" type="password" value={cfg.password} disabled={!!job}
                onChange={(e) => setCfg({ ...cfg, password: e.target.value })} />
            </div>
          </div>
          <div className="row">
            <div className="field" style={{ flex: "0 0 104px" }}>
              <label htmlFor="mq-keep" data-tip={t("mq.keepAliveHint")}>{t("mq.keepAlive")}</label>
              <input id="mq-keep" type="number" value={cfg.keep_alive_s} disabled={!!job}
                onChange={(e) => setCfg({ ...cfg, keep_alive_s: +e.target.value })} />
            </div>
            <div className="field check">
              <label className="checkbox" data-tip={t("mq.cleanSessionHint")}>
                <input type="checkbox" checked={cfg.clean_session} disabled={!!job}
                  onChange={(e) => setCfg({ ...cfg, clean_session: e.target.checked })} />
                {t("mq.cleanSession")}
              </label>
            </div>
          </div>

          <div className="row">
            <div className="field">
              <label htmlFor="mq-scan" data-tip={t("mq.scanHint")}>{t("mq.scanFilter")}</label>
              <input id="mq-scan" value={scanFilter} disabled={!!job}
                onChange={(e) => setScanFilter(e.target.value)} />
            </div>
            <div className="field" style={{ flex: "0 0 88px" }}>
              <label htmlFor="mq-scan-qos" data-tip={t("mq.qosHint")}>{t("mq.qos")}</label>
              <select id="mq-scan-qos" value={scanQos} disabled={!!job}
                onChange={(e) => setScanQos(+e.target.value)}>
                {QOS.map((q) => <option key={q} value={q}>{q}</option>)}
              </select>
            </div>
          </div>

          <div className="field">
            {/* The concept has a name nobody outside MQTT knows, so the label
                says what it does and the tooltip how presence is built on it. */}
            <label className="checkbox" data-tip={t("mq.willHint")}>
              <input type="checkbox" checked={willOn} disabled={!!job}
                onChange={(e) => setWillOn(e.target.checked)} />
              {t("mq.willEnable")}
            </label>
          </div>
          {willOn && (
            <div className="row">
              <div className="field">
                <label htmlFor="mq-will-topic">{t("mq.willTopic")}</label>
                <input id="mq-will-topic" value={will.topic} disabled={!!job}
                  onChange={(e) => setWill({ ...will, topic: e.target.value })} />
              </div>
              <div className="field" style={{ flex: "0 0 104px" }}>
                <label htmlFor="mq-will-payload">{t("mq.willPayload")}</label>
                <input id="mq-will-payload" value={will.payload} disabled={!!job}
                  onChange={(e) => setWill({ ...will, payload: e.target.value })} />
              </div>
            </div>
          )}

          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={connect}>
              {job ? t("mq.disconnect") : t("mq.connect")}
            </button>
          </div>

          {grants.length > 0 && (
            <>
              <p className="section-label" style={{ marginTop: 18 }}>{t("mq.subscriptions")}</p>
              {grants.map((g) => (
                <div className="row tight" key={g.filter} style={{ marginBottom: 5, alignItems: "center" }}>
                  <span className="sig-badge">{g.accepted ? `qos${g.qos}` : t("mq.refused")}</span>
                  <span style={{ fontFamily: "var(--mono)", fontSize: 11, overflow: "hidden", textOverflow: "ellipsis" }}>
                    {g.filter}
                  </span>
                  <button className="ghost sm" style={{ flex: "0 0 auto" }} onClick={() => unsubscribe(g.filter)}>
                    {t("mq.unsubscribe")}
                  </button>
                </div>
              ))}
            </>
          )}

          <div className="row" style={{ marginTop: 12 }}>
            <div className="field" style={{ margin: 0 }}>
              <label htmlFor="mq-sub" data-tip={t("mq.filterHint")}>{t("mq.addSubscription")}</label>
              <input id="mq-sub" value={sub.filter} disabled={!job} placeholder="sensors/+/state/#"
                onChange={(e) => setSub({ ...sub, filter: e.target.value })}
                onKeyDown={(e) => { if (e.key === "Enter") void subscribe(); }} />
            </div>
            <div className="field" style={{ flex: "0 0 88px", margin: 0 }}>
              <label htmlFor="mq-sub-qos" data-tip={t("mq.qosHint")}>{t("mq.qos")}</label>
              <select id="mq-sub-qos" value={sub.qos} disabled={!job}
                onChange={(e) => setSub({ ...sub, qos: +e.target.value })}>
                {QOS.map((q) => <option key={q} value={q}>{q}</option>)}
              </select>
            </div>
            <button className="ghost" style={{ flex: "0 0 auto", alignSelf: "flex-end" }}
              disabled={!job} onClick={subscribe}>
              {t("mq.subscribe")}
            </button>
          </div>
        </div>

        {/* ---- topics ---- */}
        <div>
          <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 360 }}>
            <div className="row" style={{ alignItems: "flex-end", marginBottom: 10 }}>
              <div className="field" style={{ margin: 0 }}>
                <label htmlFor="mq-find" data-tip={t("mq.findHint")}>{t("mq.topics")}</label>
                <input id="mq-find" value={query} placeholder={t("mq.find")}
                  onChange={(e) => setQuery(e.target.value)} />
              </div>
              <button className="ghost" style={{ flex: "0 0 auto" }} onClick={clearTree}>
                {t("common.clear")}
              </button>
            </div>

            <div className="mq-stats">
              <span>{t("mq.topicCount", { n: all.length })}</span>
              <span>{t("mq.retainedCount", { n: fmtNum(retainedCount) })}</span>
              {mqttDropped > 0 && <span className="warn">{t("mq.dropped", { n: fmtNum(mqttDropped) })}</span>}
              <span className="spacer" />
              {job && <span className="ok">{t("mq.live", { broker: `${cfg.host}:${cfg.port}` })}</span>}
            </div>

            <div className="topic-tree scroll-y">
              {found
                ? found.map((n) => (
                    <button
                      key={n.path}
                      className={"topic-row" + (picked === n.path ? " active" : "")}
                      onClick={() => setPicked(n.path)}
                    >
                      <span className="topic-twist">·</span>
                      <span className="topic-name">{n.path}</span>
                      {n.last?.retain && <span className="sig-badge">R</span>}
                      <span className="topic-value">{n.last?.payload || "—"}</span>
                    </button>
                  ))
                : roots.map((r) => row(r, 0))}
              {all.length === 0 && (
                <div className="empty-state">{job ? t("mq.waiting") : t("mq.notConnected")}</div>
              )}
              {found?.length === 0 && <div className="empty-state">{t("mq.noMatch")}</div>}
            </div>
          </div>

          {/* ---- the picked topic ---- */}
          {pickedNode?.last && (
            <div className="panel" style={{ marginTop: 16 }}>
              <p className="section-label">{pickedNode.path}</p>
              <dl className="kv">
                <dt>{t("mq.value")}</dt>
                <dd style={{ whiteSpace: "pre-wrap" }}>{pickedNode.last.payload || t("mq.empty")}</dd>
                <dt>{t("mq.qos")}</dt><dd>{pickedNode.last.qos}</dd>
                <dt>{t("mq.retain")}</dt><dd>{pickedNode.last.retain ? t("mq.yes") : t("mq.no")}</dd>
                <dt>{t("common.bytes")}</dt><dd>{fmtNum(pickedNode.last.bytes)}</dd>
                <dt>{t("mq.messages")}</dt><dd>{fmtNum(pickedNode.count)}</dd>
                <dt>{t("mq.lastAt")}</dt><dd>{fmtTime(pickedNode.last.ts)}</dd>
              </dl>
              <div className="btn-row">
                <button className="ghost sm"
                  onClick={() => setPub({
                    topic: pickedNode.path,
                    payload: pickedNode.last?.payload ?? "",
                    qos: pickedNode.last?.qos ?? 0,
                    retain: pickedNode.last?.retain ?? false,
                  })}>
                  {t("mq.editHere")}
                </button>
                {onWaitFor && <button className="ghost sm" data-tip={t("mq.waitForThisHint")}
                  onClick={() => onWaitFor(waitForMqttMessage(cfg.host, cfg.port, pickedNode.path))}>⇠ {t("mq.waitForThis")}</button>}
                {/* Two steps: this writes to the broker for every client at once. */}
                <button
                  className="danger sm"
                  disabled={!job || !pickedNode.last.retain}
                  data-tip={pickedNode.last.retain ? t("mq.clearHint") : t("mq.nothingRetained")}
                  onClick={() => {
                    if (confirmClear === pickedNode.path) {
                      void publish(pickedNode.path, "", 1, true);
                      setConfirmClear(null);
                    } else {
                      setConfirmClear(pickedNode.path);
                    }
                  }}
                >
                  {confirmClear === pickedNode.path ? t("mq.clearConfirm") : t("mq.clearRetained")}
                </button>
                <button className="ghost sm" onClick={() => saveTopicAsSignal(pickedNode)}>
                  {t("sig.fromFrame")}
                </button>
              </div>
            </div>
          )}

          {/* ---- publish ---- */}
          <div className="panel" style={{ marginTop: 16 }} onKeyDown={saveShortcut}>
            <p className="section-label">{t("mq.publish")}</p>
            <div className="row">
              <div className="field">
                <label htmlFor="mq-pub-topic">{t("mq.topic")}</label>
                <input id="mq-pub-topic" value={pub.topic}
                  onChange={(e) => setPub({ ...pub, topic: e.target.value })} />
              </div>
              <div className="field" style={{ flex: "0 0 88px" }}>
                <label htmlFor="mq-pub-qos" data-tip={t("mq.qosHint")}>{t("mq.qos")}</label>
                <select id="mq-pub-qos" value={pub.qos}
                  onChange={(e) => setPub({ ...pub, qos: +e.target.value })}>
                  {QOS.map((q) => <option key={q} value={q}>{q}</option>)}
                </select>
              </div>
              <div className="field check" style={{ flex: "0 0 108px" }}>
                <label className="checkbox" data-tip={t("mq.emptyClears")}>
                  <input type="checkbox" checked={pub.retain}
                    onChange={(e) => setPub({ ...pub, retain: e.target.checked })} />
                  {t("mq.retain")}
                </label>
              </div>
            </div>
            <div className="field">
              <label htmlFor="mq-pub-payload">{t("sig.payload")}</label>
              <textarea id="mq-pub-payload" value={pub.payload}
                onChange={(e) => setPub({ ...pub, payload: e.target.value })} />
            </div>
            <div className="btn-row">
              <button className="primary" disabled={!job}
                onClick={() => void publish(pub.topic, pub.payload, pub.qos, pub.retain)}>
                {t("mq.publishBtn")}
              </button>
            </div>
            <SaveSignal body={{ transport: "mqtt", broker: `${cfg.host}:${cfg.port}`, topic: pub.topic, payload: pub.payload, qos: pub.qos, retain: pub.retain }}
              signalId={signalId} onSignalId={setSignalId} suggestName={pub.topic || t("mq.publish")} onShow={onShowSignal} />
          </div>
        </div>
      </div>
    </div>
  );
}
