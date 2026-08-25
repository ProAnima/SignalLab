import { useEffect, useRef, useState } from "react";
import { api, on, EV, type JobInfo, type OscArg, type OscInbound, type GenTick, type Waveform } from "../lib/api";
import { useStore } from "../lib/store";
import { useRollingList, useSeries } from "../lib/hooks";
import { fmtTime } from "../lib/format";
import { Scope } from "../components/Scope";

type ArgRow = { type: OscArg["type"]; value: string };

function toOscArg(row: ArgRow): OscArg {
  switch (row.type) {
    case "int": return { type: "int", value: parseInt(row.value) || 0 };
    case "long": return { type: "long", value: parseInt(row.value) || 0 };
    case "float": return { type: "float", value: parseFloat(row.value) || 0 };
    case "double": return { type: "double", value: parseFloat(row.value) || 0 };
    case "bool": return { type: "bool", value: row.value === "true" };
    case "str": return { type: "str", value: row.value };
    default: return { type: "nil" };
  }
}

function fmtArg(a: OscArg): string {
  switch (a.type) {
    case "nil": return "nil";
    case "blob": return `blob[${a.value.length}]`;
    case "bool": return a.value ? "true" : "false";
    default: return String(a.value);
  }
}

interface FlatMsg { ts: number; from: string; address: string; args: OscArg[]; error?: string | null; }

export function OscView() {
  const { pushLog, refreshJobs, stopJob, jobs } = useStore();

  // ---- sender ----
  const [target, setTarget] = useState("127.0.0.1:9000");
  const [address, setAddress] = useState("/hello/avatar/1");
  const [args, setArgs] = useState<ArgRow[]>([{ type: "float", value: "1.0" }]);

  const send = async () => {
    try {
      const oscArgs = args.map(toOscArg);
      const n = await api.oscSend(target, address, oscArgs);
      pushLog("ok", "osc", `sent ${address} (${n} bytes) → ${target}`);
    } catch (e) {
      pushLog("err", "osc", String(e));
    }
  };

  // ---- monitor ----
  const [bind, setBind] = useState("0.0.0.0:9000");
  const [monJob, setMonJob] = useState<JobInfo | null>(null);
  const { items: msgs, push: pushMsg, clear: clearMsgs } = useRollingList<FlatMsg>(300);
  const monRef = useRef<number | null>(null);
  monRef.current = monJob?.id ?? null;

  useEffect(() => {
    const un = on<OscInbound>(EV.oscMessage, (e) => {
      const p = e.payload;
      if (monRef.current !== null && p.job_id !== monRef.current) return;
      if (p.error) {
        pushMsg({ ts: p.ts, from: p.from, address: "(decode error)", args: [], error: p.error });
        return;
      }
      for (const m of p.messages) {
        pushMsg({ ts: p.ts, from: p.from, address: m.address, args: m.args });
      }
    });
    return () => { un.then((f) => f()); };
  }, [pushMsg]);

  const toggleMonitor = async () => {
    if (monJob) {
      stopJob(monJob.id);
      setMonJob(null);
      return;
    }
    try {
      const job = await api.oscMonitorStart(bind);
      setMonJob(job);
      pushLog("ok", "osc", `monitor listening on ${bind}`);
      refreshJobs();
    } catch (e) {
      pushLog("err", "osc", String(e));
    }
  };

  // Reflect external stop (from jobs bar) back into local state.
  useEffect(() => {
    if (monJob && !jobs.find((j) => j.id === monJob.id)) setMonJob(null);
  }, [jobs, monJob]);

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
  const genRef = useRef<number | null>(null);
  genRef.current = genJob?.id ?? null;
  const { data: wave, push: pushWave, clear: clearWave } = useSeries(300);

  useEffect(() => {
    const un = on<GenTick>(EV.oscGenTick, (e) => {
      if (genRef.current !== null && e.payload.job_id === genRef.current) pushWave(e.payload.value);
    });
    return () => { un.then((f) => f()); };
  }, [pushWave]);

  useEffect(() => {
    if (genJob && !jobs.find((j) => j.id === genJob.id)) setGenJob(null);
  }, [jobs, genJob]);

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
      pushLog("ok", "osc", `generator → ${gen.target} ${gen.address} (${gen.waveform})`);
      refreshJobs();
    } catch (e) {
      pushLog("err", "osc", String(e));
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1>OSC</h1>
        <p>Send Open Sound Control messages, monitor an incoming port, and drive continuous waveforms into any OSC endpoint.</p>
      </div>

      <div className="cols side">
        {/* Sender */}
        <div className="panel">
          <p className="section-label">Sender</p>
          <div className="field">
            <label>Target host:port</label>
            <input value={target} onChange={(e) => setTarget(e.target.value)} />
          </div>
          <div className="field">
            <label>OSC address</label>
            <input value={address} onChange={(e) => setAddress(e.target.value)} />
          </div>
          <div className="field">
            <label>Arguments</label>
            {args.map((a, i) => (
              <div className="row tight" key={i} style={{ marginBottom: 6 }}>
                <select
                  style={{ flex: "0 0 92px" }}
                  value={a.type}
                  onChange={(e) => setArgs(args.map((x, j) => j === i ? { ...x, type: e.target.value as OscArg["type"] } : x))}
                >
                  <option value="int">int</option>
                  <option value="float">float</option>
                  <option value="str">string</option>
                  <option value="bool">bool</option>
                  <option value="long">long</option>
                  <option value="double">double</option>
                </select>
                {a.type === "bool" ? (
                  <select value={a.value} onChange={(e) => setArgs(args.map((x, j) => j === i ? { ...x, value: e.target.value } : x))}>
                    <option value="true">true</option>
                    <option value="false">false</option>
                  </select>
                ) : (
                  <input value={a.value} onChange={(e) => setArgs(args.map((x, j) => j === i ? { ...x, value: e.target.value } : x))} />
                )}
                <button className="ghost sm" style={{ flex: "0 0 auto" }} onClick={() => setArgs(args.filter((_, j) => j !== i))}>✕</button>
              </div>
            ))}
            <button className="ghost sm" onClick={() => setArgs([...args, { type: "float", value: "0" }])}>+ argument</button>
          </div>
          <div className="btn-row">
            <button className="primary" onClick={send}>Send</button>
          </div>
        </div>

        {/* Monitor */}
        <div className="panel" style={{ display: "flex", flexDirection: "column", minHeight: 320 }}>
          <p className="section-label">Monitor</p>
          <div className="row" style={{ alignItems: "flex-end", marginBottom: 12 }}>
            <div className="field" style={{ margin: 0 }}>
              <label>Bind address</label>
              <input value={bind} onChange={(e) => setBind(e.target.value)} disabled={!!monJob} />
            </div>
            <button className={monJob ? "danger" : "primary"} style={{ flex: "0 0 auto" }} onClick={toggleMonitor}>
              {monJob ? "Stop" : "Listen"}
            </button>
            <button className="ghost" style={{ flex: "0 0 auto" }} onClick={clearMsgs}>Clear</button>
          </div>
          <div className="scroll-y" style={{ flex: 1, border: "1px solid var(--border)", borderRadius: 6 }}>
            <table className="grid">
              <thead>
                <tr><th style={{ width: 96 }}>Time</th><th style={{ width: 130 }}>From</th><th>Address</th><th>Args</th></tr>
              </thead>
              <tbody>
                {msgs.slice().reverse().map((m, i) => (
                  <tr key={i}>
                    <td style={{ color: "var(--text-faint)" }}>{fmtTime(m.ts)}</td>
                    <td>{m.from}</td>
                    <td style={{ color: m.error ? "var(--red)" : "var(--accent)" }}>{m.address}</td>
                    <td>{m.error ?? m.args.map(fmtArg).join("  ")}</td>
                  </tr>
                ))}
                {msgs.length === 0 && (
                  <tr><td colSpan={4} style={{ color: "var(--text-faint)", textAlign: "center", padding: 20 }}>No packets yet — start listening.</td></tr>
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>

      {/* Generator */}
      <div className="panel" style={{ marginTop: 16 }}>
        <p className="section-label">Signal generator</p>
        <div className="cols side">
          <div>
            <div className="row">
              <div className="field"><label>Target</label><input value={gen.target} onChange={(e) => setGen({ ...gen, target: e.target.value })} /></div>
              <div className="field"><label>Address</label><input value={gen.address} onChange={(e) => setGen({ ...gen, address: e.target.value })} /></div>
            </div>
            <div className="row">
              <div className="field">
                <label>Waveform</label>
                <select value={gen.waveform} onChange={(e) => setGen({ ...gen, waveform: e.target.value as Waveform })}>
                  {["sine", "triangle", "saw", "square", "ramp", "random", "constant"].map((w) => <option key={w} value={w}>{w}</option>)}
                </select>
              </div>
              <div className="field"><label>Freq (Hz)</label><input type="number" value={gen.freq} onChange={(e) => setGen({ ...gen, freq: +e.target.value })} /></div>
              <div className="field"><label>Rate (pps)</label><input type="number" value={gen.rate} onChange={(e) => setGen({ ...gen, rate: +e.target.value })} /></div>
            </div>
            <div className="row">
              <div className="field"><label>Min</label><input type="number" value={gen.min} onChange={(e) => setGen({ ...gen, min: +e.target.value })} /></div>
              <div className="field"><label>Max</label><input type="number" value={gen.max} onChange={(e) => setGen({ ...gen, max: +e.target.value })} /></div>
              <div className="field" style={{ display: "flex", alignItems: "flex-end" }}>
                <label className="checkbox"><input type="checkbox" checked={gen.as_int} onChange={(e) => setGen({ ...gen, as_int: e.target.checked })} /> send as int</label>
              </div>
            </div>
            <div className="btn-row">
              <button className={genJob ? "danger" : "primary"} onClick={toggleGen}>{genJob ? "Stop generator" : "Start generator"}</button>
            </div>
          </div>
          <div>
            <Scope data={wave} height={200} color="#38e0b0" />
            <div style={{ display: "flex", justifyContent: "space-between", fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-faint)", marginTop: 6 }}>
              <span>{gen.waveform} · {gen.freq} Hz · {gen.rate} pps</span>
              <span>{wave.length ? wave[wave.length - 1].toFixed(3) : "—"}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
