import { useEffect, useRef, useState } from "react";
import type { Experiment, ProfileIssue } from "../lib/api";
import { addProfile, isIdent, removeParam, removeProfile, renameParam, renameProfile, setParamValue } from "../lib/experimentData";
import { useT } from "../lib/i18n";
import type { Failure } from "../lib/errors";
import { ExperimentSecrets } from "./ExperimentSecrets";

const MAX_SEED = 2 ** 53 - 1;

/**
 * Parameters, profiles and seed, in a panel under the toolbar. The first tab
 * holds the defaults; each profile tab overrides some of them. Every change
 * goes through the document history like any other field.
 */
export function ExperimentParams({ doc, issues, disabled, anchor, describe, onEdit, onClose, onSecretsChanged }: {
  doc: Experiment;
  issues: ProfileIssue[];
  disabled: boolean;
  anchor: { left: number; top: number };
  /** A failure in words, located by node label. */
  describe: (error: Failure) => string;
  onEdit: (update: (doc: Experiment) => Experiment) => void;
  onClose: () => void;
  /** A secret was stored (its name) or removed. */
  onSecretsChanged: (stored?: string) => void;
}) {
  const t = useT();
  const panel = useRef<HTMLDivElement>(null);
  const focus = useRef<"param" | "profile" | null>(null);
  const [tab, setTab] = useState<string | null>(doc.profile);
  const profile = doc.profiles.find((item) => item.name === tab) ?? null;
  const issueOf = (name: string | null) => issues.find((issue) => issue.profile === name);

  useEffect(() => { panel.current?.querySelector<HTMLInputElement>("input")?.focus(); }, []);
  useEffect(() => {
    if (!focus.current) return;
    const selector = focus.current === "param" ? "input[data-param-name]" : "input[data-profile-name]";
    focus.current = null;
    const inputs = panel.current?.querySelectorAll<HTMLInputElement>(selector);
    inputs?.[inputs.length - 1]?.select();
  });
  // A tab whose profile disappeared (undo, delete) falls back to the defaults.
  useEffect(() => { if (tab !== null && !profile) setTab(null); }, [tab, profile]);

  const addParam = () => {
    const taken = new Set(doc.params.map((param) => param.name));
    let n = doc.params.length + 1;
    while (taken.has(`param${n}`)) n++;
    focus.current = "param";
    onEdit((current) => ({ ...current, params: [...current.params, { name: `param${n}`, value: "" }] }));
  };
  const newProfile = () => {
    const { name } = addProfile(doc, t("exp.newProfile"));
    focus.current = "profile";
    onEdit((current) => addProfile(current, t("exp.newProfile")).doc);
    setTab(name);
  };
  const duplicate = (name: string) => doc.params.filter((param) => param.name === name).length > 1;
  const profileNameTaken = (name: string) => doc.profiles.filter((item) => item.name.trim() === name.trim()).length > 1;

  return <div className="experiment-menu-backdrop" onPointerDown={onClose}>
    <div className="experiment-params" role="dialog" aria-label={t("exp.params")} ref={panel}
      style={{ left: Math.max(8, Math.min(anchor.left, window.innerWidth - 448)), top: anchor.top }}
      onPointerDown={(event) => event.stopPropagation()}
      onKeyDown={(event) => { if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); onClose(); } }}>
      <header><strong data-tip={`${t("exp.paramsHint")}\n${t("exp.profilesHint")}`}>{t("exp.params")}</strong><button className="ghost sm" aria-label={t("exp.close")} onClick={onClose}>×</button></header>
      <div className="experiment-profile-tabs" role="tablist" aria-label={t("exp.profile")}>
        {[null, ...doc.profiles.map((item) => item.name)].map((name) => {
          const issue = issueOf(name);
          return <button key={name ?? ""} role="tab" aria-selected={tab === name} className={tab === name ? "selected" : ""}
            data-tip={issue ? t("exp.profileIssue", { error: describe(issue.error) }) : name !== null ? t("exp.profileHint") : undefined} onClick={() => setTab(name)}>
            {doc.profile === name && <span className="experiment-active-dot" aria-label={t("exp.activeProfile")}>●</span>}
            {name ?? t("exp.noProfile")}{issue && <span className="experiment-profile-warning" aria-hidden="true"> ⚠</span>}
          </button>;
        })}
        <button className="experiment-profile-add" disabled={disabled} onClick={newProfile}>＋ {t("exp.addProfile")}</button>
      </div>
      <fieldset disabled={disabled}>
        {profile && <div className="experiment-profile-head">
          <input data-profile-name aria-label={t("exp.profileName")} value={profile.name} maxLength={64}
            className={!profile.name.trim() || profileNameTaken(profile.name) ? "invalid" : undefined}
            onChange={(event) => { const name = event.target.value; onEdit((current) => renameProfile(current, profile.name, name)); setTab(name); }} />
          {doc.profile === profile.name
            ? <span className="experiment-profile-active">● {t("exp.activeProfile")}</span>
            : <button className="ghost sm" onClick={() => onEdit((current) => ({ ...current, profile: profile.name }))}>{t("exp.makeActive")}</button>}
          <button className="ghost sm experiment-delete" onClick={() => { onEdit((current) => removeProfile(current, profile.name)); setTab(null); }}>{t("exp.removeProfile")}</button>
        </div>}
        {tab === null && doc.profile !== null && <button className="ghost sm experiment-use-defaults" onClick={() => onEdit((current) => ({ ...current, profile: null }))}>{t("exp.makeActive")}</button>}
        {issueOf(tab) && <p className="experiment-profile-issue">{t("exp.profileIssue", { error: describe(issueOf(tab)!.error) })}</p>}
        {doc.params.length === 0 && <p className="experiment-empty">{t("exp.noParams")}</p>}
        {doc.params.map((param, index) => {
          if (profile) {
            const overridden = param.name in profile.values;
            return <div className="experiment-param-row profile" key={index}>
              <code data-tip={param.name}>{param.name}</code>
              <input aria-label={param.name} value={profile.values[param.name] ?? ""}
                placeholder={param.value ? t("exp.inherit", { value: param.value }) : t("exp.inheritEmpty")}
                onChange={(event) => { const value = event.target.value; onEdit((current) => setParamValue(current, profile.name, param.name, value === "" ? null : value)); }} />
              <button className="ghost sm" aria-label={t("exp.resetToDefault")} data-tip={t("exp.resetToDefault")} disabled={!overridden}
                onClick={() => onEdit((current) => setParamValue(current, profile.name, param.name, null))}>↺</button>
            </div>;
          }
          const invalid = !isIdent(param.name) || duplicate(param.name);
          return <div className="experiment-param-row" key={index}>
            <input data-param-name aria-label={t("exp.paramName")} placeholder={t("exp.paramName")} value={param.name}
              className={invalid ? "invalid" : undefined} data-tip={invalid ? t("exp.invalidParamName") : undefined} aria-invalid={invalid}
              onChange={(event) => { const name = event.target.value; onEdit((current) => renameParam(current, index, name)); }} />
            <input aria-label={t("exp.paramValue")} placeholder={t("exp.paramValue")} value={param.value}
              onChange={(event) => { const value = event.target.value; onEdit((current) => ({ ...current, params: current.params.map((item, i) => i === index ? { ...item, value } : item) })); }}
              onKeyDown={(event) => { if (event.key === "Enter" && index === doc.params.length - 1) { event.preventDefault(); addParam(); } }} />
            <button className="ghost sm" aria-label={t("exp.removeParam")} data-tip={t("exp.removeParam")} onClick={() => onEdit((current) => removeParam(current, index))}>×</button>
          </div>;
        })}
        {tab === null && <button className="ghost sm" onClick={addParam}>＋ {t("exp.addParam")}</button>}
        <label className="experiment-seed" data-tip={t("exp.seedHint")}>{t("exp.seed")}
          <input inputMode="numeric" placeholder={t("exp.seedRandom")} value={doc.seed ?? ""}
            onChange={(event) => {
              const text = event.target.value.trim();
              if (!text) { onEdit((current) => ({ ...current, seed: null })); return; }
              if (/^\d+$/.test(text) && Number(text) <= MAX_SEED) onEdit((current) => ({ ...current, seed: Number(text) }));
            }} />
        </label>
      </fieldset>
      <ExperimentSecrets doc={doc} onChanged={onSecretsChanged} />
    </div>
  </div>;
}
