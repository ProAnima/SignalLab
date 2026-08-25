import { useEffect, useRef, useState } from "react";
import { api, on, EV, type JobInfo, type ProxyStat } from "../lib/api";
import { useStore } from "../lib/store";
import { fmtBytes, fmtNum } from "../lib/format";

function Slider({ label, value, onChange, min, max, step, unit }: {
  label: string; value: number; onChange: (v: number) => void;
  min: number; max: number; step: number; unit: string;
}) {
  return (
    <div className="field">
      <label style={{ display: "flex", justifyContent: "space-between" }}>
        <span>{label}</span>
        <span style={{ fontFamily: "var(--mono)", color: "var(--accent)" }}>{value}{unit}</span>
      </label>
      <input type="range" min={min} max={max} step={step} value={value} onChange={(e) => onChange(+e.target.value)} />
    </div>
  );
}

export function NetsimView() {
  const { pushLog, refreshJobs, stopJob, jobs } = useStore();

  const [listen, setListen] = useState("0.0.0.0:9010");
  const [target, setTarget] = useState("127.0.0.1:9000");
  const [latency, setLatency] = useState(40);
  const [jitter, setJitter] = useState(15);
  const [loss, setLoss] = useState(2);
  const [duplicate, setDuplicate] = useState(0);
  const [corrupt, setCorrupt] = useState(0);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [stat, setStat] = useState<ProxyStat | null>(null);
  const jobRef = useRef<number | null>(null);
  jobRef.current = job?.id ?? null;

  useEffect(() => {
    const un = on<ProxyStat>(EV.netsimStat, (e) => {
      if (jobRef.current !== null && e.payload.job_id === jobRef.current) setStat(e.payload);
    });
    return () => { un.then((f) => f()); };
  }, []);

  useEffect(() => {
    if (job && !jobs.find((j) => j.id === job.id)) setJob(null);
  }, [jobs, job]);

  const toggle = async () => {
    if (job) { stopJob(job.id); setJob(null); return; }
    try {
      setStat(null);
      const j = await api.netsimStart({
        listen, target,
        profile: {
          latency_ms: latency, jitter_ms: jitter,
          loss: loss / 100, duplicate: duplicate / 100, corrupt: corrupt / 100,
        },
      });
      setJob(j);
      pushLog("ok", "netsim", `impairment ${listen} → ${target} (${latency}±${jitter}ms, ${loss}% loss)`);
      refreshJobs();
    } catch (e) {
      pushLog("err", "netsim", String(e));
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1>Network impairment</h1>
        <p>A UDP relay that degrades traffic between a client and a target. Point your client at the listen port; packets are forwarded to the real target with the delay, loss and corruption you dial in.</p>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">Relay</p>
          <div className="field"><label>Listen (client connects here)</label><input value={listen} onChange={(e) => setListen(e.target.value)} disabled={!!job} /></div>
          <div className="field"><label>Target (real destination)</label><input value={target} onChange={(e) => setTarget(e.target.value)} disabled={!!job} /></div>

          <p className="section-label" style={{ marginTop: 18 }}>Impairment profile</p>
          <Slider label="Latency" value={latency} onChange={setLatency} min={0} max={1000} step={5} unit="ms" />
          <Slider label="Jitter" value={jitter} onChange={setJitter} min={0} max={500} step={5} unit="ms" />
          <Slider label="Packet loss" value={loss} onChange={setLoss} min={0} max={100} step={1} unit="%" />
          <Slider label="Duplication" value={duplicate} onChange={setDuplicate} min={0} max={100} step={1} unit="%" />
          <Slider label="Corruption" value={corrupt} onChange={setCorrupt} min={0} max={100} step={1} unit="%" />

          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={toggle}>{job ? "Stop relay" : "Start relay"}</button>
          </div>
        </div>

        <div className="panel">
          <p className="section-label">Live</p>
          <div className="metrics">
            <div className="metric"><div className="k">Forwarded</div><div className="v accent">{fmtNum(stat?.forwarded ?? 0)}</div></div>
            <div className="metric"><div className="k">Dropped</div><div className="v red">{fmtNum(stat?.dropped ?? 0)}</div></div>
            <div className="metric"><div className="k">Duplicated</div><div className="v amber">{fmtNum(stat?.duplicated ?? 0)}</div></div>
            <div className="metric"><div className="k">Corrupted</div><div className="v amber">{fmtNum(stat?.corrupted ?? 0)}</div></div>
            <div className="metric"><div className="k">Volume</div><div className="v">{fmtBytes(stat?.bytes ?? 0)}</div></div>
          </div>

          <div className="hint" style={{ marginTop: 18 }}>
            <b>How to wire it up</b><br />
            Set your client's OSC/UDP target to <code>{listen.replace("0.0.0.0", "127.0.0.1")}</code> instead of the real server.
            The relay forwards to <code>{target}</code>, applying the profile above in both directions.
            Impairment is symmetric and applies per-packet.
          </div>
        </div>
      </div>
    </div>
  );
}
