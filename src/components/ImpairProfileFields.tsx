import { useId } from "react";
import type { ImpairProfile } from "../lib/api";
import { useT, type TKey } from "../lib/i18n";
import { changedProfile, fullProfile, IMPAIR_PRESETS, presetOf, presetProfile } from "../lib/impairments";

function Slider({ label, tip, value, onChange, min, max, step, unit, disabled }: {
  label: string; tip: string; value: number; onChange: (value: number) => void;
  min: number; max: number; step: number; unit: string; disabled?: boolean;
}) {
  const id = useId();
  return (
    <div className="field">
      <label htmlFor={id} data-tip={tip} style={{ display: "flex", justifyContent: "space-between" }}>
        <span>{label}</span>
        <span style={{ fontFamily: "var(--mono)", color: "var(--accent)" }}>{value}{unit}</span>
      </label>
      {/* aria-valuetext: a screen reader says the value with its unit, as the label shows it. */}
      <input id={id} type="range" min={min} max={max} step={step} value={value} disabled={disabled} aria-valuetext={`${value}${unit}`} onChange={(event) => onChange(+event.target.value)} />
    </div>
  );
}

/** A probability as the slider shows it: percent, to a tenth. */
const pct = (value: number) => Math.round(value * 1000) / 10;

/**
 * What an impairment does: a preset to start from, and every value of it.
 * The Impairment screen and the experiment's Impairment nodes share it.
 */
export function ImpairProfileFields({ profile, onChange, disabled }: { profile: ImpairProfile; onChange: (next: ImpairProfile) => void; disabled?: boolean }) {
  const t = useT();
  const offlineId = useId();
  const rateId = useId();
  const lengthId = useId();
  const full = fullProfile(profile);
  const preset = presetOf(profile);
  const set = (change: Partial<ImpairProfile>) => onChange(changedProfile(full, change));
  const ms = ` ${t("unit.ms")}`;
  return <div className="impair-fields">
    <div className="chips impair-presets" role="group" aria-label={t("ns.preset")}>
      {IMPAIR_PRESETS.map((key) => (
        <button key={key} className={"chip" + (preset === key ? " on" : "")} aria-pressed={preset === key} disabled={disabled}
          data-tip={t(`ns.presetHint.${key}` as TKey)} onClick={() => onChange(presetProfile(key))}>
          {t(`ns.preset.${key}` as TKey)}
        </button>
      ))}
    </div>
    <label className="checkbox impair-offline" htmlFor={offlineId} data-tip={t("ns.offlineHint")}>
      <input id={offlineId} type="checkbox" checked={full.offline} disabled={disabled} onChange={(event) => set({ offline: event.target.checked })} />
      {t("ns.offline")}
    </label>
    <Slider label={t("ns.latency")} tip={t("ns.latencyHint")} value={full.latency_ms} onChange={(latency_ms) => set({ latency_ms })} min={0} max={1000} step={5} unit={ms} disabled={disabled} />
    <Slider label={t("ns.jitter")} tip={t("ns.jitterHint")} value={full.jitter_ms} onChange={(jitter_ms) => set({ jitter_ms })} min={0} max={500} step={5} unit={ms} disabled={disabled} />
    <Slider label={t("ns.loss")} tip={t("ns.lossHint")} value={pct(full.loss)} onChange={(value) => set({ loss: value / 100 })} min={0} max={100} step={0.5} unit="%" disabled={disabled} />
    <Slider label={t("ns.burst")} tip={t("ns.burstHint")} value={pct(full.burst_start)} onChange={(value) => set({ burst_start: value / 100, burst_length: value > 0 && full.burst_length < 1 ? 5 : full.burst_length })} min={0} max={20} step={0.5} unit="%" disabled={disabled} />
    {full.burst_start > 0 && <div className="field">
      <label htmlFor={lengthId} data-tip={t("ns.burstLengthHint")}>{t("ns.burstLength")}</label>
      <input id={lengthId} type="number" min={1} max={1000} value={full.burst_length} disabled={disabled}
        onChange={(event) => set({ burst_length: Math.min(1000, Math.max(1, Math.round(Number(event.target.value) || 1))) })} />
    </div>}
    <Slider label={t("ns.duplicate")} tip={t("ns.duplicateHint")} value={pct(full.duplicate)} onChange={(value) => set({ duplicate: value / 100 })} min={0} max={100} step={1} unit="%" disabled={disabled} />
    <Slider label={t("ns.corrupt")} tip={t("ns.corruptHint")} value={pct(full.corrupt)} onChange={(value) => set({ corrupt: value / 100 })} min={0} max={100} step={1} unit="%" disabled={disabled} />
    <Slider label={t("ns.reorder")} tip={t("ns.reorderHint")} value={pct(full.reorder)} onChange={(value) => set({ reorder: value / 100 })} min={0} max={100} step={1} unit="%" disabled={disabled} />
    <div className="field">
      <label htmlFor={rateId} data-tip={t("ns.rateHint")}>{t("ns.rate")}</label>
      <input id={rateId} type="number" min={0} max={10000000} step={100} value={full.rate_kbps} disabled={disabled}
        onChange={(event) => {
          const value = Math.max(0, Math.round(Number(event.target.value) || 0));
          // Below the engine's floor a limit means nothing a link has; 0 is none.
          set({ rate_kbps: value === 0 ? 0 : Math.min(10000000, Math.max(8, value)) });
        }} />
    </div>
  </div>;
}
