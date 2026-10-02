import { useEffect, useId, useRef, useState } from "react";
import type { Experiment, ProfileIssue } from "../lib/api";
import { effectiveParams } from "../lib/experimentData";
import { useT } from "../lib/i18n";

const MAX_SEED = 2 ** 53 - 1;

export interface RunOptions {
  profile: string | null;
  /** Only parameters the user typed a value for. */
  overrides: Record<string, string>;
  seed: number | null;
}

/**
 * Run with…: another profile, other values or a given seed for one run. The
 * document is not changed; the form keeps what was typed for the session.
 */
export function ExperimentRunWith({ doc, issues, lastSeed, initial, anchor, onRun, onClose }: {
  doc: Experiment;
  issues: ProfileIssue[];
  lastSeed: number | null;
  initial: RunOptions | null;
  anchor: { right: number; top: number };
  onRun: (options: RunOptions) => void;
  onClose: () => void;
}) {
  const t = useT();
  const form = useRef<HTMLFormElement>(null);
  const seedId = useId();
  const known = (name: string | null) => name === null || doc.profiles.some((profile) => profile.name === name);
  const [profile, setProfile] = useState<string | null>(initial && known(initial.profile) ? initial.profile : doc.profile);
  const [values, setValues] = useState<Record<string, string>>(() => Object.fromEntries(
    Object.entries(initial?.overrides ?? {}).filter(([name]) => doc.params.some((param) => param.name === name))));
  const [seed, setSeed] = useState(initial?.seed != null ? String(initial.seed) : "");
  const placeholders = effectiveParams(doc, profile);
  const seedValid = seed.trim() === "" || (/^\d+$/.test(seed.trim()) && Number(seed.trim()) <= MAX_SEED);

  useEffect(() => { form.current?.querySelector<HTMLElement>("select, input")?.focus(); }, []);

  const submit = () => {
    if (!seedValid) return;
    const overrides = Object.fromEntries(Object.entries(values).filter(([, value]) => value !== ""));
    onRun({ profile, overrides, seed: seed.trim() ? Number(seed.trim()) : null });
  };

  return <div className="experiment-menu-backdrop" onPointerDown={onClose}>
    <form className="experiment-run-with" role="dialog" aria-label={t("exp.runWith")} ref={form}
      style={{ right: Math.max(8, window.innerWidth - anchor.right), top: anchor.top }}
      onPointerDown={(event) => event.stopPropagation()}
      onKeyDown={(event) => { if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); onClose(); } }}
      onSubmit={(event) => { event.preventDefault(); submit(); }}>
      <header><strong data-tip={t("exp.runWithHint")}>{t("exp.runWith")}</strong><button type="button" className="ghost sm" aria-label={t("exp.close")} data-tip={t("common.closeHint")} onClick={onClose}>×</button></header>
      {doc.profiles.length > 0 && <label>{t("exp.profile")}
        <select value={profile ?? ""} onChange={(event) => setProfile(event.target.value || null)}>
          {[null, ...doc.profiles.map((item) => item.name)].map((name) => <option key={name ?? ""} value={name ?? ""}>
            {name ?? t("exp.noProfile")}{issues.some((issue) => issue.profile === name) ? " ⚠" : ""}</option>)}
        </select></label>}
      {doc.params.map((param) => <label key={param.name} className="experiment-run-param"><code>{param.name}</code>
        <input value={values[param.name] ?? ""} placeholder={placeholders[param.name] ?? ""}
          className={values[param.name] ? "overridden" : undefined}
          onChange={(event) => setValues((prior) => ({ ...prior, [param.name]: event.target.value }))} /></label>)}
      {/* The "last seed" button sits beside the label, not in it: a click inside a label also focuses its input. */}
      <div className="experiment-run-seed">
        <label htmlFor={seedId}>{t("exp.seed")}</label>
        <span>
          <input id={seedId} inputMode="numeric" value={seed} className={seedValid ? undefined : "invalid"}
            placeholder={doc.seed !== null ? t("exp.pinnedSeedPlaceholder", { seed: doc.seed }) : t("exp.seedRandom")}
            onChange={(event) => setSeed(event.target.value)} />
          {lastSeed !== null && <button type="button" className="ghost sm" onClick={() => setSeed(String(lastSeed))}>{t("exp.lastSeed", { seed: lastSeed })}</button>}
        </span>
      </div>
      <footer>
        <button type="button" className="ghost sm" onClick={() => { setValues({}); setSeed(""); setProfile(doc.profile); }}>{t("exp.resetOverrides")}</button>
        <button type="submit" className="primary" disabled={!seedValid}>{t("exp.run")}</button>
      </footer>
    </form>
  </div>;
}
