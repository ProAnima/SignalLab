import { useEffect, useState } from "react";
import { api, EV, type JobInfo, type StormStat } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { useFieldIds, useJobStream, useSeries } from "../lib/hooks";
import { fmtBytes, fmtNum } from "../lib/format";
import { Scope } from "../components/Scope";

export function StormView() {
  const { pushLog, pushError, refreshJobs, stopJob, jobGone } = useStore();
  const t = useT();
  const fid = useFieldIds();

  const [target, setTarget] = useState("127.0.0.1:9000");
  const [protocol, setProtocol] = useState<"udp" | "tcp">("udp");
  const [size, setSize] = useState(512);
  const [rate, setRate] = useState(1000);
  const [duration, setDuration] = useState(10);

  const [job, setJob] = useState<JobInfo | null>(null);
  const [stat, setStat] = useState<StormStat | null>(null);
  const { data: ppsSeries, push: pushPps, clear: clearPps } = useSeries(240);

  useJobStream<StormStat>(EV.stormStat, job?.id ?? null, (s) => {
    setStat(s);
    pushPps(s.pps);
  });

  useEffect(() => {
    if (jobGone(job)) setJob(null);
  }, [jobGone, job]);

  const toggle = async () => {
    if (job) { stopJob(job.id); setJob(null); return; }
    try {
      clearPps();
      setStat(null);
      const j = await api.stormStart({ target, protocol, size, rate, duration_s: duration });
      setJob(j);
      pushLog("warn", "storm", "log.stormStarted", {
        protocol: protocol.toUpperCase(), target, rate, size,
      });
      refreshJobs();
    } catch (e) {
      pushError("storm", e);
    }
  };

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("st.blurb")}>{t("st.title")}</h1>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">{t("st.generator")}</p>
          <div className="field">
            <label htmlFor={fid("target")} data-tip={t("st.warning")}>{t("common.target")}</label>
            <input id={fid("target")} value={target} onChange={(e) => setTarget(e.target.value)} disabled={!!job} />
          </div>
          <div className="row">
            <div className="field">
              <label htmlFor={fid("protocol")} data-tip={t("st.protocolHint")}>{t("common.protocol")}</label>
              <select id={fid("protocol")} value={protocol} onChange={(e) => setProtocol(e.target.value as "udp" | "tcp")} disabled={!!job}>
                <option value="udp">{t("st.udp")}</option>
                <option value="tcp">{t("st.tcp")}</option>
              </select>
            </div>
            <div className="field">
              <label htmlFor={fid("size")} data-tip={t("st.payloadSizeHint")}>{t("st.payloadSize")}</label>
              <input id={fid("size")} type="number" value={size} onChange={(e) => setSize(+e.target.value)} disabled={!!job} />
            </div>
          </div>
          <div className="row">
            <div className="field">
              <label htmlFor={fid("rate")} data-tip={t("st.rateHint")}>{t("st.rate")}</label>
              <input id={fid("rate")} type="number" value={rate} onChange={(e) => setRate(+e.target.value)} disabled={!!job} />
            </div>
            <div className="field">
              <label htmlFor={fid("duration")} data-tip={t("st.durationHint")}>{t("st.duration")}</label>
              <input id={fid("duration")} type="number" value={duration} onChange={(e) => setDuration(+e.target.value)} disabled={!!job} />
            </div>
          </div>
          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={toggle} data-tip={job ? undefined : t("st.warning")}>
              {job ? t("st.stop") : t("st.launch")}
            </button>
          </div>
        </div>

        <div className="panel">
          <p className="section-label">{t("st.throughput")}</p>
          <div className="metrics" style={{ marginBottom: 14 }}>
            <div className="metric"><div className="k">{t("common.packets")}</div><div className="v accent">{fmtNum(stat?.packets ?? 0)}</div></div>
            <div className="metric" data-tip={t("st.ppsCaption")}><div className="k">{t("st.pps")}</div><div className="v accent">{fmtNum(stat?.pps ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("st.rateLabel")}</div><div className="v amber">{fmtNum(stat?.mbps ?? 0, 2)}<small>{t("unit.mbps")}</small></div></div>
            <div className="metric"><div className="k">{t("common.volume")}</div><div className="v">{fmtBytes(stat?.bytes ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("common.errors")}</div><div className="v red">{fmtNum(stat?.errors ?? 0)}</div></div>
          </div>
          <div data-tip={t("st.ppsCaption")}>
            <Scope data={ppsSeries} height={220} color="#ffc069" min={0} />
          </div>
        </div>
      </div>
    </div>
  );
}
