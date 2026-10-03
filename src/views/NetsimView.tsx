import { useEffect, useRef, useState } from "react";
import { api, EV, type ImpairProfile, type JobInfo, type ProxyStat, type RelayProtocol } from "../lib/api";
import { useStore } from "../lib/store";
import { useT, type TKey } from "../lib/i18n";
import { useFieldIds, useJobStream } from "../lib/hooks";
import { fmtBytes, fmtNum } from "../lib/format";
import { forProtocol, fullProfile, impairNotation, IMPAIR_PRESETS, presetOf, type ImpairPreset } from "../lib/impairments";
import { ImpairProfileFields } from "../components/ImpairProfileFields";

/** How long after the last change a running relay is told the new profile. */
const APPLY_AFTER_MS = 250;

export function NetsimView() {
  const { pushLog, pushError, refreshJobs, stopJob, jobGone } = useStore();
  const t = useT();
  const fid = useFieldIds();

  const [protocol, setProtocol] = useState<RelayProtocol>("udp");
  const [listen, setListen] = useState("0.0.0.0:9010");
  const [target, setTarget] = useState("127.0.0.1:9000");
  const [profile, setProfile] = useState<ImpairProfile>(() => fullProfile({ latency_ms: 40, jitter_ms: 15, loss: 0.02, duplicate: 0, corrupt: 0 }));

  const [job, setJob] = useState<JobInfo | null>(null);
  const [stat, setStat] = useState<ProxyStat | null>(null);
  useJobStream<ProxyStat>(EV.netsimStat, job?.id ?? null, setStat);

  useEffect(() => {
    if (jobGone(job)) setJob(null);
  }, [jobGone, job]);

  // A running relay takes an edit at once, without dropping its port: the
  // profile it was started with is not sent again.
  const applied = useRef<ImpairProfile | null>(null);
  useEffect(() => {
    if (!job || applied.current === profile) return;
    const timer = window.setTimeout(() => {
      applied.current = profile;
      api.netsimSetProfile(job.id, profile).then(
        // The preset's key, worded when the line is shown: the console re-renders in another language.
        () => pushLog("info", "netsim", "log.netsimProfile", { profile: presetOf(profile, job.params?.protocol === "tcp" ? "tcp" : "udp") ?? impairNotation(forProtocol(profile, job.params?.protocol === "tcp" ? "tcp" : "udp")) }),
        (error) => pushError("netsim", error),
      );
    }, APPLY_AFTER_MS);
    return () => window.clearTimeout(timer);
  }, [job, profile, pushLog, pushError]);

  const toggle = async () => {
    if (job) { stopJob(job.id); setJob(null); return; }
    try {
      setStat(null);
      const started = await api.netsimStart({ listen, target, profile, protocol });
      applied.current = profile;
      setJob(started);
      const full = fullProfile(profile);
      pushLog("ok", "netsim", "log.netsimStarted", { listen, target, latency: full.latency_ms, jitter: full.jitter_ms, loss: Math.round(full.loss * 1000) / 10 });
      refreshJobs();
    } catch (error) {
      pushError("netsim", error);
    }
  };

  const shownProfile = stat?.profile && IMPAIR_PRESETS.includes(stat.profile as ImpairPreset) ? t(`ns.preset.${stat.profile}` as TKey) : stat?.profile;

  return (
    <div>
      <div className="view-head">
        <h1 data-tip={t("ns.blurb")}>{t("ns.title")}</h1>
      </div>

      <div className="cols side">
        <div className="panel">
          <p className="section-label">{t("ns.relay")}</p>
          <div className="field">
            <label htmlFor={fid("protocol")} data-tip={t("ns.protocolHint")}>{t("ns.protocol")}</label>
            <select id={fid("protocol")} value={protocol} onChange={(e) => setProtocol(e.target.value as RelayProtocol)} disabled={!!job}>
              <option value="udp">UDP</option>
              <option value="tcp">TCP</option>
            </select>
          </div>
          <div className="field">
            <label htmlFor={fid("listen")} data-tip={t(protocol === "tcp" ? "ns.howToBodyTcp" : "ns.howToBody", { listen: listen.replace("0.0.0.0", "127.0.0.1"), target })}>{t("ns.listen")}</label>
            <input id={fid("listen")} value={listen} onChange={(e) => setListen(e.target.value)} disabled={!!job} />
          </div>
          <div className="field">
            <label htmlFor={fid("target")} data-tip={t("ns.targetHint")}>{t("ns.target")}</label>
            <input id={fid("target")} value={target} onChange={(e) => setTarget(e.target.value)} disabled={!!job} />
          </div>

          <p className="section-label" style={{ marginTop: 20 }} data-tip={t(protocol === "tcp" ? "ns.profileHintTcp" : "ns.profileHint")}>{t("ns.profile")}</p>
          <ImpairProfileFields profile={profile} onChange={setProfile} protocol={protocol} />

          <div className="btn-row">
            <button className={job ? "danger" : "primary"} onClick={toggle}>
              {job ? t("ns.stopRelay") : t("ns.startRelay")}
            </button>
          </div>
        </div>

        <div className="panel">
          <p className="section-label">{t("ns.live")}</p>
          {job && shownProfile && <p className="impair-now" role="status">{t("ns.now", { profile: shownProfile })}</p>}
          {(job?.params?.protocol ?? protocol) === "tcp" ? <div className="metrics">
            <div className="metric"><div className="k">{t("ns.connections")}</div><div className="v">{fmtNum(stat?.connections ?? 0)}</div></div>
            <div className="metric"><div className="k" data-tip={t("ns.chunksHint")}>{t("ns.received")}</div><div className="v">{fmtNum(stat?.received ?? 0)}</div></div>
            <div className="metric"><div className="k" data-tip={t("ns.chunksHint")}>{t("ns.forwarded")}</div><div className="v accent">{fmtNum(stat?.forwarded ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.resets")}</div><div className="v red">{fmtNum(stat?.reset ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.stalled")}</div><div className="v amber">{fmtNum(stat?.stalled ?? 0)}</div></div>
            <div className="metric"><div className="k" data-tip={t("ns.heldHint")}>{t("ns.held")}</div><div className="v amber">{fmtNum(stat?.throttled ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("common.volume")}</div><div className="v">{fmtBytes(stat?.bytes ?? 0)}</div></div>
          </div> : <div className="metrics">
            <div className="metric"><div className="k">{t("ns.received")}</div><div className="v">{fmtNum(stat?.received ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.forwarded")}</div><div className="v accent">{fmtNum(stat?.forwarded ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.dropped")}</div><div className="v red">{fmtNum(stat?.dropped ?? 0)}</div></div>
            <div className="metric"><div className="k" data-tip={t("ns.throttledHint")}>{t("ns.throttled")}</div><div className="v red">{fmtNum(stat?.throttled ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.duplicated")}</div><div className="v amber">{fmtNum(stat?.duplicated ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.corrupted")}</div><div className="v amber">{fmtNum(stat?.corrupted ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("ns.reordered")}</div><div className="v amber">{fmtNum(stat?.reordered ?? 0)}</div></div>
            <div className="metric"><div className="k">{t("common.volume")}</div><div className="v">{fmtBytes(stat?.bytes ?? 0)}</div></div>
          </div>}
        </div>
      </div>
    </div>
  );
}
