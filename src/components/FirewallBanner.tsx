import { useEffect, useRef, useState } from "react";
import { api, type FirewallStatus } from "../lib/api";
import { describeError, type Failure } from "../lib/errors";
import { useT } from "../lib/i18n";
import { useStore } from "../lib/store";

/** The jobs that listen for what other machines send. */
const LISTENING = new Set(["osc-monitor", "discovery", "netsim", "experiment"]);

/** Other machines' datagrams may not arrive: the firewall is on, and nothing lets them in or something blocks them. */
export const inTheWay = (status: FirewallStatus) => status.applies && status.enabled && (status.blocked || !status.allowed);

/**
 * Windows asks once, the first time Signal Lab listens, whether other
 * machines may reach it — and a "Cancel" there, or a network Windows calls
 * public, silently drops what they send. When something starts listening
 * the desktop app looks at the firewall once, and when it is in the way says
 * so, with the fix a click away: the system's own administrator prompt, then
 * one allow rule. Never on a server, whose firewall is its administrator's.
 */
export function FirewallBanner() {
  const t = useT();
  const { info, jobs, pushLog } = useStore();
  const [status, setStatus] = useState<FirewallStatus | null>(null);
  const [failure, setFailure] = useState<Failure | null>(null);
  const [busy, setBusy] = useState(false);
  const [dismissed, setDismissed] = useState(false);
  const checked = useRef(false);

  const listening = jobs.some((job) => LISTENING.has(job.kind));
  useEffect(() => {
    if (info?.mode !== "desktop" || !listening || checked.current) return;
    checked.current = true;
    api.firewallStatus().then(setStatus).catch(() => {});
  }, [info?.mode, listening]);

  if (!status || dismissed || !inTheWay(status)) return null;
  const onPublic = status.networks.includes("public");
  const network = status.networks.map((name) => t(`fw.network.${name}`)).join(", ");

  const allow = async (alsoPublic: boolean) => {
    setBusy(true);
    setFailure(null);
    try {
      const next = await api.firewallAllow(alsoPublic);
      setStatus(next);
      if (!inTheWay(next)) pushLog("ok", "firewall", "log.firewallAllowed", { networks: network });
    } catch (error) {
      setFailure(error);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="firewall-banner" role="alert">
      <span data-tip={t(status.blocked ? "fw.blockedHint" : "fw.notAllowedHint", { program: status.program })}>
        {t(status.blocked ? "fw.blocked" : "fw.notAllowed", { network })}
      </span>
      {failure !== null && <span className="firewall-error">{describeError(failure, t).text}</span>}
      <span className="firewall-actions">
        {onPublic ? (
          <button className="primary sm" disabled={busy} data-tip={t("fw.allowPublicHint")} onClick={() => void allow(true)}>{t("fw.allowPublic")}</button>
        ) : (
          <button className="primary sm" disabled={busy} data-tip={t("fw.allowHint")} onClick={() => void allow(false)}>{t("fw.allow")}</button>
        )}
        <button className="ghost sm" disabled={busy} onClick={() => setDismissed(true)}>{t("fw.dismiss")}</button>
      </span>
    </div>
  );
}
