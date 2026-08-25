import { useEffect, useRef, useState } from "react";
import { api, on, EV, type JobInfo, type StormStat } from "../lib/api";
import { useStore } from "../lib/store";
import { useSeries } from "../lib/hooks";
import { fmtBytes, fmtNum } from "../lib/format";
import { Scope } from "../components/Scope";

export function StormView() {
  const { pushLog, refreshJobs, stopJob, jobs } = useStore();

  const [target, setTarget] = useState("127.0.0.1:9000");
  const [protocol, setProtocol] = useState<"udp" | "tcp">("udp");
  const [size, setSize] = useState(512);
  const [rate, setRate] = useState(1000);
  const [duration, setDuration] = useState(10);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [stat, setStat] = useState<StormStat | null>(null);
  const jobRef = useRef<number | null>(null);
  jobRef.current = job?.id ?? null;
  const { data: ppsSeries, push: pushPps, clear: clearPps } = useSeries(240);

  useEffect(() => {
    const un = on<StormStat>(EV.stormStat, (e) => {
      if (jobRef.current !== null && e.payload.job_id === jobRef.current) {
        setStat(e.payload);
        pushPps(e.payload.pps);
      }
    });
    return () => { un.then((f) => f()); };
  }, [pushPps]);

  useEffect(() => {
    if (job && !jobs.find((j) => j.id === job.id)) setJob(null);
  }, [jobs, job]);

  const toggle = async () => {
    if (job) { stopJob(job.id); setJob(null); return; }
    try {
      clearPps();
      setStat(null);
      const j = await api.stormStart({ target, protocol, size, rate, duration_s: duration });
      setJob(j);
      pushLog("warn", "storm", `${protocol.toUpperCase()} storm → ${target} @ ${rate}pps × ${size}B`);
      refreshJobs();
    } catch (e) {
      pushLog("err", "storm", String(e));
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1>Network storm</h1>
        <p>A controlled UDP/TCP load source for stress-testing your own servers and links. Set the packet rate, payload size and duration; throughput is metered live.</p>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">Generator</p>
          <div className="field"><label>Target host:port</label><input value={target} onChange={(e) => setTarget(e.target.value)} disabled={!!job} /></div>
          <div className="row">
            <div className="field">
              <label>Protocol</label>
              <select value={protocol} onChange={(e) => setProtocol(e.target.value as "udp" | "tcp")} disabled={!!job}>
                <option value="udp">UDP flood</option>
                <option value="tcp">TCP connect flood</option>
              </select>
            </div>
            <div className="field"><label>Payload (bytes)</label><input type="number" value={size} onChange={(e) => setSize(+e.target.value)} disabled={!!job} /></div>
          </div>
          <div className="row">
            <div className="field"><label>Rate (pps, 0 = max)</label><input type="number" value={rate} onChange={(e) => setRate(+e.target.value)} disabled={!!job} /></div>
            <div className="field"><label>Duration (s, 0 = manual)</label><input type="number" value={duration} onChange={(e) => setDuration(+e.target.value)} disabled={!!job} /></div>
          </div>
          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={toggle}>{job ? "Stop storm" : "Launch storm"}</button>
          </div>
          <div className="hint amber">
            ⚠ Only target hosts and networks you own or are authorized to test. High packet rates can saturate links and trip intrusion detection.
          </div>
        </div>

        <div className="panel">
          <p className="section-label">Live throughput</p>
          <div className="metrics" style={{ marginBottom: 14 }}>
            <div className="metric"><div className="k">Packets</div><div className="v accent">{fmtNum(stat?.packets ?? 0)}</div></div>
            <div className="metric"><div className="k">PPS</div><div className="v accent">{fmtNum(stat?.pps ?? 0)}</div></div>
            <div className="metric"><div className="k">Rate</div><div className="v amber">{(stat?.mbps ?? 0).toFixed(2)}<small>Mbps</small></div></div>
            <div className="metric"><div className="k">Volume</div><div className="v">{fmtBytes(stat?.bytes ?? 0)}</div></div>
            <div className="metric"><div className="k">Errors</div><div className="v red">{fmtNum(stat?.errors ?? 0)}</div></div>
          </div>
          <Scope data={ppsSeries} height={220} color="#ffb454" min={0} />
          <div style={{ fontFamily: "var(--mono)", fontSize: 11, color: "var(--text-faint)", marginTop: 6, textAlign: "right" }}>
            packets / second
          </div>
        </div>
      </div>
    </div>
  );
}
