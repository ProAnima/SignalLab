import { useEffect, useId, useState } from "react";
import { api, EV, type JobInfo, type ProxyStat } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { useFieldIds, useJobStream } from "../lib/hooks";
import { fmtBytes, fmtNum } from "../lib/format";

function Slider({ label, tip, value, onChange, min, max, step, unit }: {
  label: string; tip: string; value: number; onChange: (v: number) => void;
  min: number; max: number; step: number; unit: string;
}) {
  const id = useId();
  return (
    <div className="field">
      <label htmlFor={id} data-tip={tip} style={{ display: "flex", justifyContent: "space-between" }}>
        <span>{label}</span>
        <span style={{ fontFamily: "var(--mono)", color: "var(--accent)" }}>{value}{unit}</span>
      </label>
      {/* aria-valuetext: a screen reader says the value with its unit, as the label shows it. */}
      <input id={id} type="range" min={min} max={max} step={step} value={value} aria-valuetext={`${value}${unit}`} onChange={(e) => onChange(+e.target.value)} />
    </div>
  );
}

export function NetsimView() {
  const { pushLog, pushError, refreshJobs, stopJob, jobGone } = useStore();
  const t = useT();
  const fid = useFieldIds();

  const [listen, setListen] = useState("0.0.0.0:9010");
  const [target, setTarget] = useState("127.0.0.1:9000");
  const [latency, setLatency] = useState(40);
  const [jitter, setJitter] = useState(15);
  const [loss, setLoss] = useState(2);
  const [duplicate, setDuplicate] = useState(0);
  const [corrupt, setCorrupt] = useState(0);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [stat, setStat] = useState<ProxyStat | null>(null);
  useJobStream<ProxyStat>(EV.netsimStat, job?.id ?? null, setStat);

  useEffect(() => {
    if (jobGone(job)) setJob(null);
  }, [jobGone, job]);

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
      pushLog("ok", "netsim", "log.netsimStarted", { listen, target, latency, jitter, loss });
      refreshJobs();
    } catch (e) {
      pushError("netsim", e);
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("ns.blurb")}>{t("ns.title")}</h1>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">{t("ns.relay")}</p>
          <div className="field">
            <label htmlFor={fid("listen")} data-tip={t("ns.howToBody", { listen: listen.replace("0.0.0.0", "127.0.0.1"), target })}>{t("ns.listen")}</label>
            <input id={fid("listen")} value={listen} onChange={(e) => setListen(e.target.value)} disabled={!!job} />
          </div>
          <div className="field">
            <label htmlFor={fid("target")} data-tip={t("ns.targetHint")}>{t("ns.target")}</label>
            <input id={fid("target")} value={target} onChange={(e) => setTarget(e.target.value)} disabled={!!job} />
          </div>

          <p className="section-label" style={{ marginTop: 20 }}>{t("ns.profile")}</p>
          <Slider label={t("ns.latency")} tip={t("ns.latencyHint")} value={latency} onChange={setLatency} min={0} max={1000} step={5} unit={` ${t("unit.ms")}`} />
          <Slider label={t("ns.jitter")} tip={t("ns.jitterHint")} value={jitter} onChange={setJitter} min={0} max={500} step={5} unit={` ${t("unit.ms")}`} />
          <Slider label={t("ns.loss")} tip={t("ns.lossHint")} value={loss} onChange={setLoss} min={0} max={100} step={1} unit="%" />
          <Slider label={t("ns.duplicate")} tip={t("ns.duplicateHint")} value={duplicate} onChange={setDuplicate} min={0} max={100} step={1} unit="%" />
          <Slider label={t("ns.corrupt")} tip={t("ns.corruptHint")} value={corrupt} onChange={setCorrupt} min={0} max={100} step={1} unit="%" />

          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={toggle}>
              {job ? t("ns.stopRelay") : t("ns.startRelay")}
            </button>
          </div>
        </div>

        <div className="panel">
          <p className="section-label">{t("ns.live")}</p>
          <div className="metrics">
            <div className="metric"><div className="k">{t("ns.forwarded")}</div><div className="v accent">{fmtNum(stat?.forwarded ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.dropped")}</div><div className="v red">{fmtNum(stat?.dropped ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.duplicated")}</div><div className="v amber">{fmtNum(stat?.duplicated ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.corrupted")}</div><div className="v amber">{fmtNum(stat?.corrupted ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("common.volume")}</div><div className="v">{fmtBytes(stat?.bytes ?? 0)}</div></div>
          </div>

        </div>
      </div>
    </div>
  );
}
