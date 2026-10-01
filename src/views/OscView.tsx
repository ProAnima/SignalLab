import { useEffect, useState } from "react";
import { api, EV, type ExperimentNode, type JobInfo, type OscArg, type OscInbound, type GenTick, type SignalBody, type Waveform } from "../lib/api";
import { waitForOscMessage } from "../lib/experimentGraph";
import { useStore } from "../lib/store";
import { describeError, type Failure } from "../lib/errors";
import { useT, type TKey } from "../lib/i18n";
import { useJobStream, usePersistentState, useRollingList, useSeries } from "../lib/hooks";
import { fmtTime } from "../lib/format";
import { Scope } from "../components/Scope";
import { OscArgsEditor, fmtArg, isArgRows, toOscArg, type ArgRow } from "../components/OscArgs";

interface FlatMsg { ts: number; from: string; address: string; args: OscArg[]; error?: string | null; }

const WAVEFORMS: Waveform[] = ["sine", "triangle", "saw", "square", "ramp", "random", "constant"];

export function OscView({ onToExperiment, onWaitFor }: { onToExperiment?: (body: SignalBody) => void; onWaitFor?: (node: ExperimentNode) => void }) {
  const { pushLog, pushError, refreshJobs, stopJob, jobGone } = useStore();
  const t = useT();

  // ---- sender ----
  // Kept across screens and restarts: the message you were sending is the one you want next.
  const [target, setTarget] = usePersistentState("signal-lab.osc.target", "127.0.0.1:9000");
  const [address, setAddress] = usePersistentState("signal-lab.osc.address", "/hello/avatar/1");
  const [args, setArgs] = usePersistentState<ArgRow[]>("signal-lab.osc.args", [{ type: "float", value: "1.0" }], isArgRows);
  const [last, setLast] = useState<{ ok: boolean; text: string; error?: Failure; ts: number; n: number } | null>(null);

  const send = async () => {
    try {
      const oscArgs = args.map(toOscArg);
      const n = await api.oscSend(target, address, oscArgs);
      pushLog("ok", "osc", "log.oscSent", { address, bytes: n, target });
      const text = t("osc.sentResult", { address, bytes: n, target });
      // Repeats of the same result count up, so a re-send visibly did something.
      setLast((prior) => ({ ok: true, text, ts: Date.now(), n: prior?.ok && prior.text === text ? prior.n + 1 : 1 }));
    } catch (e) {
      pushError("osc", e);
      setLast({ ok: false, text: "", error: e, ts: Date.now(), n: 1 });
    }
  };

  // Enter anywhere in the sender fires it — this is a tool you re-trigger a lot.
  const onEnterSend = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") { e.preventDefault(); void send(); }
  };

  // ---- monitor ----
  const [bind, setBind] = useState("0.0.0.0:9000");
  const [monJob, setMonJob] = useState<JobInfo | null>(null);
  const { items: msgs, push: pushMsg, clear: clearMsgs } = useRollingList<FlatMsg>(300);

  useJobStream<OscInbound>(EV.oscMessage, monJob?.id ?? null, (p) => {
    if (p.error) {
      pushMsg({ ts: p.ts, from: p.from, address: "", args: [], error: p.error });
      return;
    }
    for (const m of p.messages) {
      pushMsg({ ts: p.ts, from: p.from, address: m.address, args: m.args });
    }
  });

  const toggleMonitor = async () => {
    if (monJob) {
      stopJob(monJob.id);
      setMonJob(null);
      return;
    }
    try {
      const job = await api.oscMonitorStart(bind);
      setMonJob(job);
      pushLog("ok", "osc", "log.oscListening", { bind });
      refreshJobs();
    } catch (e) {
      pushError("osc", e);
    }
  };

  // Reflect external stop (from the jobs bar) back into local state.
  useEffect(() => {
    if (jobGone(monJob)) setMonJob(null);
  }, [jobGone, monJob]);

  // ---- generator ----
  const [gen, setGen] = useState({
    target: "127.0.0.1:9000",
    address: "/hello/lfo",
    waveform: "sine" as Waveform,
    freq: 1,
    rate: 60,
    min: 0,
    max: 1,
    as_int: false,
  });
  const [genJob, setGenJob] = useState<JobInfo | null>(null);
  const { data: wave, push: pushWave, clear: clearWave } = useSeries(300);

  useJobStream<GenTick>(EV.oscGenTick, genJob?.id ?? null, (t) => pushWave(t.value));

  useEffect(() => {
    if (jobGone(genJob)) setGenJob(null);
  }, [jobGone, genJob]);

  const toggleGen = async () => {
    if (genJob) {
      stopJob(genJob.id);
      setGenJob(null);
      return;
    }
    try {
      clearWave();
      const job = await api.oscGeneratorStart({ ...gen, duration_s: 0 });
      setGenJob(job);
      pushLog("ok", "osc", "log.oscGenerator", {
        target: gen.target,
        address: gen.address,
        waveform: t(`wave.${gen.waveform}` as TKey),
      });
      refreshJobs();
    } catch (e) {
      pushError("osc", e);
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("osc.blurb")}>{t("osc.title")}</h1>
      </div>

      <div className="cols side">
        {/* Sender */}
        <div className="panel">
          <p className="section-label">{t("osc.sender")}</p>
          <div className="field">
            <label>{t("common.target")}</label>
            <input value={target} onChange={(e) => setTarget(e.target.value)} onKeyDown={onEnterSend} />
          </div>
          <div className="field">
            <label>{t("common.address")}</label>
            <input value={address} onChange={(e) => setAddress(e.target.value)} onKeyDown={onEnterSend} />
          </div>
          <div className="field">
            <label>{t("common.arguments")}</label>
            <OscArgsEditor args={args} onChange={setArgs} onSubmit={send} />
          </div>
          <div className="btn-row">
            <button className="primary" onClick={send} data-tip={`${t("common.send")} · Enter`}>{t("common.send")}</button>
            {onToExperiment && (
              <button
                className="ghost"
                data-tip={t("common.toExperimentHint")}
                onClick={() => onToExperiment({ transport: "osc", target, address, args: args.map(toOscArg) })}
              >
                {t("common.toExperiment")}
              </button>
            )}
          </div>
          {last && (
            <p className={"send-result " + (last.ok ? "ok" : "err")} role="status">
              <span>{last.ok ? "✓" : "✕"} {last.error !== undefined ? describeError(last.error, t).text : last.text}</span>
              {last.n > 1 && <b>×{last.n}</b>}
              <time>{fmtTime(last.ts).slice(0, 8)}</time>
            </p>
          )}
        </div>

        {/* Monitor */}
        <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 320 }}>
          <p className="section-label">{t("osc.monitor")}</p>
          <div className="row" style={{ alignItems: "flex-end", marginBottom: 12 }}>
            <div className="field" style={{ margin: 0 }}>
              <label>{t("common.bind")}</label>
              <input value={bind} onChange={(e) => setBind(e.target.value)} disabled={!!monJob} />
            </div>
            <button className={monJob ? "danger" : "primary"} style={{ flex: "0 0 auto" }} onClick={toggleMonitor}>
              {monJob ? t("common.stop") : t("osc.listen")}
            </button>
            <button className="ghost" style={{ flex: "0 0 auto" }} onClick={clearMsgs}>
              {t("common.clear")}
            </button>
          </div>
          <div className="scroll-y" style={{ flex: 1, border: "1px solid var(--border)", borderRadius: "var(--radius-sm)" }}>
            <table className="grid">
              <thead>
                <tr>
                  <th style={{ width: 96 }}>{t("common.time")}</th>
                  <th style={{ width: 130 }}>{t("osc.from")}</th>
                  <th>{t("osc.address")}</th>
                  <th>{t("osc.args")}</th>
                  {onWaitFor && <th style={{ width: 40 }} aria-label={t("osc.waitForThis")} />}
                </tr>
              </thead>
              <tbody>
                {msgs.slice().reverse().map((m, i) => (
                  <tr key={i}>
                    <td style={{ color: "var(--text-faint)" }}>{fmtTime(m.ts)}</td>
                    <td>{m.from}</td>
                    <td style={{ color: m.error ? "var(--red)" : "var(--accent)" }}>
                      {m.error ? t("osc.decodeError") : m.address}
                    </td>
                    <td>{m.error ?? m.args.map(fmtArg).join("  ")}</td>
                    {onWaitFor && <td>{!m.error && <button className="ghost xs" aria-label={t("osc.waitForThis")}
                      data-tip={t("osc.waitForThisHint", { bind })} onClick={() => onWaitFor(waitForOscMessage(bind, m.address, m.args))}>⇠</button>}</td>}
                  </tr>
                ))}
                {msgs.length === 0 && (
                  <tr><td colSpan={onWaitFor ? 5 : 4} className="empty-state">{t("osc.noPackets")}</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Generator */}
      <div className="panel" style={{ marginTop: 16 }}>
        <p className="section-label">{t("osc.generator")}</p>
        <div className="cols side">
          <div>
            <div className="row">
              <div className="field">
                <label>{t("common.target")}</label>
                <input value={gen.target} onChange={(e) => setGen({ ...gen, target: e.target.value })} />
              </div>
              <div className="field">
                <label>{t("osc.address")}</label>
                <input value={gen.address} onChange={(e) => setGen({ ...gen, address: e.target.value })} />
              </div>
            </div>
            <div className="row">
              <div className="field">
                <label>{t("osc.waveform")}</label>
                <select value={gen.waveform} onChange={(e) => setGen({ ...gen, waveform: e.target.value as Waveform })}>
                  {WAVEFORMS.map((w) => (
                    <option key={w} value={w}>{t(`wave.${w}` as TKey)}</option>
                  ))}
                </select>
              </div>
              <div className="field">
                <label>{t("osc.freq")}</label>
                <input type="number" value={gen.freq} onChange={(e) => setGen({ ...gen, freq: +e.target.value })} />
              </div>
              <div className="field">
                <label>{t("osc.rate")}</label>
                <input type="number" value={gen.rate} onChange={(e) => setGen({ ...gen, rate: +e.target.value })} />
              </div>
            </div>
            <div className="row">
              <div className="field">
                <label>{t("osc.min")}</label>
                <input type="number" value={gen.min} onChange={(e) => setGen({ ...gen, min: +e.target.value })} />
              </div>
              <div className="field">
                <label>{t("osc.max")}</label>
                <input type="number" value={gen.max} onChange={(e) => setGen({ ...gen, max: +e.target.value })} />
              </div>
              <div className="field check">
                <label className="checkbox">
                  <input type="checkbox" checked={gen.as_int} onChange={(e) => setGen({ ...gen, as_int: e.target.checked })} />
                  {t("osc.asInt")}
                </label>
              </div>
            </div>
            <div className="btn-row">
              <button className={genJob ? "danger" : "primary"} onClick={toggleGen}>
                {genJob ? t("osc.stopGen") : t("osc.startGen")}
              </button>
            </div>
          </div>
          <div>
            <Scope data={wave} height={200} color="#3ee6b0" />
            <div style={{ display: "flex", justifyContent: "space-between", fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-faint)", marginTop: 8 }}>
              <span>{t(`wave.${gen.waveform}` as TKey)} · {gen.freq} Hz · {gen.rate} pps</span>
              <span>{wave.length ? wave[wave.length - 1].toFixed(3) : "—"}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
