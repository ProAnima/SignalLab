import { useEffect, useRef, useState } from "react";
import {
  api, on, EV,
  type EmitConfig, type EmitResult, type EmitStat, type JobInfo,
  type Payload, type Peer, type PeerReport, type TargetMode,
} from "../lib/api";
import { useStore } from "../lib/store";
import { useT, type TKey, type Translate } from "../lib/i18n";
import { OscArgsEditor, toOscArg, type ArgRow } from "../components/OscArgs";
import { fmtBytes, fmtNum } from "../lib/format";

// ---------------------------------------------------------------------------
// Payload editing (shared by the emitter and the discovery auto-reply)
// ---------------------------------------------------------------------------

type PayloadKind = "osc" | "text" | "hex";

interface PayloadState {
  kind: PayloadKind;
  address: string;
  args: ArgRow[];
  text: string;
  hex: string;
}

function toPayload(p: PayloadState): Payload {
  switch (p.kind) {
    case "osc": return { kind: "osc", address: p.address, args: p.args.map(toOscArg) };
    case "text": return { kind: "text", text: p.text };
    default: return { kind: "hex", hex: p.hex };
  }
}

function PayloadEditor({
  value, onChange, disabled, t,
}: {
  value: PayloadState;
  onChange: (next: PayloadState) => void;
  disabled?: boolean;
  t: Translate;
}) {
  return (
    <>
      <div className="field">
        <label>{t("bc.payload")}</label>
        <div className="chips" role="radiogroup" aria-label={t("bc.payload")}>
          {(["osc", "text", "hex"] as PayloadKind[]).map((k) => (
            <button
              key={k}
              className={"chip" + (value.kind === k ? " on" : "")}
              role="radio"
              aria-checked={value.kind === k}
              disabled={disabled}
              onClick={() => onChange({ ...value, kind: k })}
            >
              {t(`bc.payload.${k}` as TKey)}
            </button>
          ))}
        </div>
      </div>

      {value.kind === "osc" && (
        <>
          <div className="field">
            <label>{t("common.address")}</label>
            <input
              value={value.address}
              disabled={disabled}
              onChange={(e) => onChange({ ...value, address: e.target.value })}
            />
          </div>
          <div className="field">
            <label>{t("common.arguments")}</label>
            <OscArgsEditor
              args={value.args}
              disabled={disabled}
              onChange={(args) => onChange({ ...value, args })}
            />
          </div>
        </>
      )}

      {value.kind === "text" && (
        <div className="field">
          <label>{t("bc.textHint")}</label>
          <textarea
            value={value.text}
            disabled={disabled}
            style={{ minHeight: 56 }}
            onChange={(e) => onChange({ ...value, text: e.target.value })}
          />
        </div>
      )}

      {value.kind === "hex" && (
        <div className="field">
          <label>{t("bc.hexHint")}</label>
          <textarea
            value={value.hex}
            disabled={disabled}
            style={{ minHeight: 56 }}
            placeholder="de ad be ef"
            onChange={(e) => onChange({ ...value, hex: e.target.value })}
          />
        </div>
      )}
    </>
  );
}

// ---------------------------------------------------------------------------

const MODES: TargetMode[] = ["list", "broadcast", "multicast", "sweep"];

const DEFAULT_TARGET: Record<TargetMode, string> = {
  list: "127.0.0.1:9000, 127.0.0.1:9001",
  broadcast: "255.255.255.255:9000",
  multicast: "239.1.1.1:9000",
  sweep: "192.168.1.0/24",
};

/** "192.168.1.42" → the form the given mode wants. */
function fromLocalIp(ip: string, mode: TargetMode): string | null {
  const parts = ip.split(".");
  if (parts.length !== 4) return null;
  const net = parts.slice(0, 3).join(".");
  if (mode === "broadcast") return `${net}.255:9000`;
  if (mode === "sweep") return `${net}.0/24`;
  if (mode === "list") return `${net}.10:9000, ${net}.11:9000`;
  return null;
}

export function BroadcastView() {
  const { pushLog, refreshJobs, stopJob, jobs, host } = useStore();
  const t = useT();

  // ---- emitter ----
  const [mode, setMode] = useState<TargetMode>("broadcast");
  const [targets, setTargets] = useState<Record<TargetMode, string>>(DEFAULT_TARGET);
  const [port, setPort] = useState(9000);
  const [bind, setBind] = useState("0.0.0.0:0");
  const [ttl, setTtl] = useState(1);
  const [mcastLoop, setMcastLoop] = useState(true);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [payload, setPayload] = useState<PayloadState>({
    kind: "osc",
    address: "/hello/discover",
    args: [{ type: "str", value: "who-is-there" }],
    text: "HELLO-PROBE",
    hex: "48 45 4c 4c 4f",
  });

  const [rate, setRate] = useState(2);
  const [count, setCount] = useState(0);
  const [duration, setDuration] = useState(0);
  const [result, setResult] = useState<EmitResult | null>(null);
  const [busy, setBusy] = useState(false);

  const [beaconJob, setBeaconJob] = useState<JobInfo | null>(null);
  const [emitStat, setEmitStat] = useState<EmitStat | null>(null);
  const beaconRef = useRef<number | null>(null);
  beaconRef.current = beaconJob?.id ?? null;

  const target = targets[mode];
  const setTarget = (v: string) => setTargets({ ...targets, [mode]: v });

  const emitConfig = (): EmitConfig => ({
    mode,
    target,
    port,
    payload: toPayload(payload),
    bind: bind.trim() || null,
    ttl,
    multicast_loop: mcastLoop,
    rate,
    count,
    duration_s: duration,
  });

  const sendOnce = async () => {
    setBusy(true);
    try {
      const r = await api.broadcastSend(emitConfig());
      setResult(r);
      if (r.errors > 0) {
        pushLog("warn", "broadcast", "log.emitErrors", {
          sent: r.packets, targets: r.targets, summary: r.summary, errors: r.errors,
        });
      } else {
        pushLog("ok", "broadcast", "log.emitSent", {
          sent: r.packets, targets: r.targets, summary: r.summary,
        });
      }
    } catch (e) {
      pushLog("err", "broadcast", String(e));
    } finally {
      setBusy(false);
    }
  };

  const toggleBeacon = async () => {
    if (beaconJob) {
      stopJob(beaconJob.id);
      setBeaconJob(null);
      return;
    }
    try {
      setEmitStat(null);
      const job = await api.broadcastBeaconStart(emitConfig());
      setBeaconJob(job);
      pushLog("ok", "broadcast", "log.beaconStarted", { mode: t(`bc.mode.${mode}` as TKey), target, rate });
      refreshJobs();
    } catch (e) {
      pushLog("err", "broadcast", String(e));
    }
  };

  useEffect(() => {
    const un = on<EmitStat>(EV.emitStat, (e) => {
      if (beaconRef.current !== null && e.payload.job_id === beaconRef.current) setEmitStat(e.payload);
    });
    return () => { un.then((f) => f()); };
  }, []);

  useEffect(() => {
    if (beaconJob && !jobs.find((j) => j.id === beaconJob.id)) setBeaconJob(null);
  }, [jobs, beaconJob]);

  // ---- discovery ----
  const [dBind, setDBind] = useState("0.0.0.0:9000");
  const [groups, setGroups] = useState("239.1.1.1");
  const [iface, setIface] = useState("");
  const [reuse, setReuse] = useState(true);
  const [respond, setRespond] = useState(false);
  const [respondDelay, setRespondDelay] = useState(0);
  const [matchContains, setMatchContains] = useState("");
  const [response, setResponse] = useState<PayloadState>({
    kind: "osc",
    address: "/hello/here",
    args: [{ type: "str", value: "signal-lab" }],
    text: "HELLO-ACK",
    hex: "41 43 4b",
  });

  const [discJob, setDiscJob] = useState<JobInfo | null>(null);
  const [report, setReport] = useState<PeerReport | null>(null);
  const discRef = useRef<number | null>(null);
  discRef.current = discJob?.id ?? null;

  useEffect(() => {
    const un = on<PeerReport>(EV.peers, (e) => {
      if (discRef.current !== null && e.payload.job_id === discRef.current) setReport(e.payload);
    });
    return () => { un.then((f) => f()); };
  }, []);

  useEffect(() => {
    if (discJob && !jobs.find((j) => j.id === discJob.id)) setDiscJob(null);
  }, [jobs, discJob]);

  const toggleDiscovery = async () => {
    if (discJob) {
      stopJob(discJob.id);
      setDiscJob(null);
      return;
    }
    try {
      setReport(null);
      const groupList = groups.split(",").map((g) => g.trim()).filter(Boolean);
      const job = await api.discoveryStart({
        bind: dBind,
        groups: groupList,
        interface: iface.trim() || null,
        reuse,
        respond,
        response: respond ? toPayload(response) : null,
        respond_delay_ms: respondDelay,
        match_contains: matchContains.trim() || null,
      });
      setDiscJob(job);
      if (groupList.length) {
        pushLog("ok", "discovery", "log.discoveryStartedGroups", { bind: dBind, groups: groupList.join(", ") });
      } else {
        pushLog("ok", "discovery", "log.discoveryStarted", { bind: dBind });
      }
      refreshJobs();
    } catch (e) {
      pushLog("err", "discovery", String(e));
    }
  };

  const peers: Peer[] = report?.peers ?? [];
  const now = Date.now();
  const suggestion = host ? fromLocalIp(host.local_ip, mode) : null;

  return (
    <div>
      <div className="view-head">
        <h1>{t("bc.title")}</h1>
        <p>{t("bc.blurb")}</p>
      </div>

      <div className="cols side">
        {/* ---- emitter ---- */}
        <div className="panel">
          <p className="section-label">{t("bc.emitter")}</p>

          <div className="field">
            <label>{t("bc.mode")}</label>
            <div className="chips" role="radiogroup" aria-label={t("bc.mode")}>
              {MODES.map((m) => (
                <button
                  key={m}
                  className={"chip" + (mode === m ? " on" : "")}
                  role="radio"
                  aria-checked={mode === m}
                  onClick={() => setMode(m)}
                >
                  {t(`bc.mode.${m}` as TKey)}
                </button>
              ))}
            </div>
          </div>

          <div className="field">
            {/* The shortcut is a sibling of the label, not a child: a click
                target inside a <label> also activates the labelled input. */}
            <div className="label-row">
              <label htmlFor="bc-target">
                {mode === "sweep" ? t("bc.targetCidr") : mode === "list" ? t("bc.targetList") : t("bc.targetAddress")}
              </label>
              {suggestion && (
                <button
                  className="link-btn"
                  onClick={() => setTarget(suggestion)}
                  title={t("bc.useSubnetHint", { ip: host?.local_ip ?? "" })}
                >
                  {t("bc.useSubnet")}
                </button>
              )}
            </div>
            {mode === "list" ? (
              <textarea id="bc-target" value={target} style={{ minHeight: 56 }} onChange={(e) => setTarget(e.target.value)} />
            ) : (
              <input
                id="bc-target"
                value={target}
                onChange={(e) => setTarget(e.target.value)}
                onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); sendOnce(); } }}
              />
            )}
          </div>

          {mode === "sweep" && (
            <div className="field">
              <label>{t("bc.port")}</label>
              <input type="number" value={port} onChange={(e) => setPort(+e.target.value)} />
            </div>
          )}

          <PayloadEditor value={payload} onChange={setPayload} t={t} />

          <button className="ghost sm" style={{ marginTop: 4 }} onClick={() => setShowAdvanced(!showAdvanced)}>
            {showAdvanced ? "▾" : "▸"} {t("bc.socketOptions")}
          </button>
          {showAdvanced && (
            <div style={{ marginTop: 10 }}>
              <div className="row">
                <div className="field">
                  <label>{t("bc.bindSource")}</label>
                  <input value={bind} onChange={(e) => setBind(e.target.value)} placeholder="0.0.0.0:0" />
                </div>
                <div className="field">
                  <label>{t("bc.ttl")}</label>
                  <input type="number" value={ttl} onChange={(e) => setTtl(+e.target.value)} />
                </div>
              </div>
              {mode === "multicast" && (
                <div className="field">
                  <label className="checkbox">
                    <input type="checkbox" checked={mcastLoop} onChange={(e) => setMcastLoop(e.target.checked)} />
                    {t("bc.mcastLoop")}
                  </label>
                </div>
              )}
            </div>
          )}

          <div className="btn-row">
            <button className="primary" onClick={sendOnce} disabled={busy || !!beaconJob}>
              {busy ? t("common.sending") : t("bc.sendOnce")}
            </button>
          </div>

          <p className="section-label" style={{ marginTop: 20 }}>{t("bc.beacon")}</p>
          <div className="row">
            <div className="field">
              <label>{t("bc.beaconRate")}</label>
              <input type="number" step="0.5" value={rate} disabled={!!beaconJob} onChange={(e) => setRate(+e.target.value)} />
            </div>
            <div className="field">
              <label>{t("bc.beaconRounds")}</label>
              <input type="number" value={count} disabled={!!beaconJob} onChange={(e) => setCount(+e.target.value)} />
            </div>
            <div className="field">
              <label>{t("bc.beaconSeconds")}</label>
              <input type="number" value={duration} disabled={!!beaconJob} onChange={(e) => setDuration(+e.target.value)} />
            </div>
          </div>
          <div className="btn-row">
            <button className={beaconJob ? "danger" : ""} onClick={toggleBeacon}>
              {beaconJob ? t("bc.stopBeacon") : t("bc.startBeacon")}
            </button>
          </div>

          <div className="hint info">{t(`bc.blurb.${mode}` as TKey)}</div>
        </div>

        {/* ---- right column ---- */}
        <div>
          <div className="panel" style={{ marginBottom: 16 }}>
            <p className="section-label">{t("bc.lastEmit")}</p>
            {!result && !emitStat && <div className="empty-state">{t("bc.nothingSent")}</div>}
            {(result || emitStat) && (
              <div className="metrics">
                <div className="metric">
                  <div className="k">{t("bc.targets")}</div>
                  <div className="v accent">{fmtNum(result?.targets ?? 0)}</div>
                </div>
                <div className="metric">
                  <div className="k">{t("common.packets")}</div>
                  <div className="v">{fmtNum(emitStat?.packets ?? result?.packets ?? 0)}</div>
                </div>
                <div className="metric">
                  <div className="k">{t("common.volume")}</div>
                  <div className="v">{fmtBytes(emitStat?.bytes ?? result?.bytes ?? 0)}</div>
                </div>
                <div className="metric">
                  <div className="k">{t("common.errors")}</div>
                  <div className="v red">{fmtNum(emitStat?.errors ?? result?.errors ?? 0)}</div>
                </div>
                {emitStat && (
                  <>
                    <div className="metric">
                      <div className="k">{t("bc.rounds")}</div>
                      <div className="v">{fmtNum(emitStat.rounds)}</div>
                    </div>
                    <div className="metric">
                      <div className="k">{t("bc.pps")}</div>
                      <div className="v accent">{fmtNum(emitStat.pps)}</div>
                    </div>
                  </>
                )}
              </div>
            )}
            {result && result.resolved.length > 0 && (
              <div style={{ marginTop: 12, fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-faint)" }}>
                → {result.resolved.join("  ")}
                {result.targets > result.resolved.length && ` ${t("bc.andMore", { n: result.targets - result.resolved.length })}`}
              </div>
            )}
          </div>

          <div className="panel">
            <p className="section-label">{t("bc.discovery")}</p>
            <div className="row">
              <div className="field">
                <label>{t("common.bind")}</label>
                <input value={dBind} disabled={!!discJob} onChange={(e) => setDBind(e.target.value)} />
              </div>
              <div className="field">
                <label>{t("bc.joinGroups")}</label>
                <input value={groups} disabled={!!discJob} placeholder="239.1.1.1, 224.0.0.251" onChange={(e) => setGroups(e.target.value)} />
              </div>
            </div>
            <div className="row">
              <div className="field">
                <label>{t("bc.interface")}</label>
                <input value={iface} disabled={!!discJob} placeholder={host?.local_ip ?? "0.0.0.0"} onChange={(e) => setIface(e.target.value)} />
              </div>
              <div className="field check">
                <label className="checkbox">
                  <input type="checkbox" checked={reuse} disabled={!!discJob} onChange={(e) => setReuse(e.target.checked)} />
                  {t("bc.reuse")}
                </label>
              </div>
            </div>

            <div className="field">
              <label className="checkbox">
                <input type="checkbox" checked={respond} disabled={!!discJob} onChange={(e) => setRespond(e.target.checked)} />
                {t("bc.respond")}
              </label>
            </div>

            {respond && (
              <div style={{ borderLeft: "2px solid var(--border-strong)", paddingLeft: 12, marginBottom: 12 }}>
                <PayloadEditor value={response} onChange={setResponse} disabled={!!discJob} t={t} />
                <div className="row">
                  <div className="field">
                    <label>{t("bc.replyDelay")}</label>
                    <input type="number" value={respondDelay} disabled={!!discJob} onChange={(e) => setRespondDelay(+e.target.value)} />
                  </div>
                  <div className="field">
                    <label>{t("bc.matchContains")}</label>
                    <input value={matchContains} disabled={!!discJob} placeholder={t("common.anything")} onChange={(e) => setMatchContains(e.target.value)} />
                  </div>
                </div>
              </div>
            )}

            <div className="btn-row">
              <button className={discJob ? "danger" : "primary"} onClick={toggleDiscovery}>
                {discJob ? t("bc.stopListen") : t("bc.startListen")}
              </button>
            </div>

            <div style={{ display: "flex", alignItems: "center", gap: 10, margin: "16px 0 8px", flexWrap: "wrap" }}>
              <p className="section-label" style={{ margin: 0 }}>{t("bc.peers")}</p>
              <span className="tag-chip">{t("bc.peersSeen", { n: peers.length })}</span>
              {report && <span className="tag-chip">{t("bc.peersPackets", { n: fmtNum(report.packets) })}</span>}
              {report && report.responses > 0 && <span className="tag-chip">{t("bc.peersReplies", { n: fmtNum(report.responses) })}</span>}
            </div>
            <div className="scroll-y" style={{ maxHeight: 260, border: "1px solid var(--border)", borderRadius: "var(--radius-sm)" }}>
              <table className="grid">
                <thead>
                  <tr>
                    <th style={{ width: 160 }}>{t("bc.peer")}</th>
                    <th style={{ width: 54 }}>{t("bc.proto")}</th>
                    <th style={{ width: 64 }}>{t("common.packets")}</th>
                    <th style={{ width: 60 }}>{t("bc.age")}</th>
                    <th>{t("bc.lastMessage")}</th>
                  </tr>
                </thead>
                <tbody>
                  {peers.map((p) => {
                    const age = Math.max(0, now - p.last_ms);
                    return (
                      <tr key={p.addr}>
                        <td>
                          <span className={"peer-dot " + (age < 3000 ? "fresh" : "stale")} />
                          {p.addr}
                        </td>
                        <td style={{ color: p.proto === "osc" ? "var(--accent)" : "var(--text-dim)" }}>{p.proto}</td>
                        <td>{fmtNum(p.packets)}</td>
                        <td style={{ color: "var(--text-faint)" }}>{(age / 1000).toFixed(1)}s</td>
                        <td style={{ color: "var(--text-dim)" }}>{p.last_summary}</td>
                      </tr>
                    );
                  })}
                  {peers.length === 0 && (
                    <tr>
                      <td colSpan={5} className="empty-state">
                        {discJob ? t("bc.listeningEmpty") : t("bc.notListening")}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>

            <div className="hint info">{t("bc.firewallHint")}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
