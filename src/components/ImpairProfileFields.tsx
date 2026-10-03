import { useEffect, useId, useState } from "react";
import type { ImpairProfile, RelayProtocol } from "../lib/api";
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

/** The bandwidth limit as typed: kept as text while it is being typed ("1" on the way to
 * "1000"), handed on once it is a limit the engine takes (0, or 8 and up), clamped on leaving. */
function RateField({ id, value, onChange, disabled }: { id: string; value: number; onChange: (value: number) => void; disabled?: boolean }) {
  const [text, setText] = useState(String(value));
  // A preset or another field changed it: show that.
  useEffect(() => { setText((current) => Number(current) === value ? current : String(value)); }, [value]);
  const usable = (typed: number) => typed === 0 || (typed >= 8 && typed <= 10000000);
  return <input id={id} type="number" min={0} max={10000000} step={100} value={text} disabled={disabled}
    onChange={(event) => {
      setText(event.target.value);
      const typed = Math.round(Number(event.target.value));
      if (event.target.value.trim() !== "" && Number.isFinite(typed) && usable(typed)) onChange(typed);
    }}
    onBlur={() => {
      const typed = Math.max(0, Math.round(Number(text) || 0));
      // Below the engine's floor a limit means nothing a link has; 0 is none.
      const settled = typed === 0 ? 0 : Math.min(10000000, Math.max(8, typed));
      setText(String(settled));
      if (settled !== value) onChange(settled);
    }} />;
}

/** A probability as the slider shows it: percent, to a tenth. */
const pct = (value: number) => Math.round(value * 1000) / 10;

/**
 * What an impairment does: a preset to start from, and every value of it a
 * relay of `protocol` reads — a datagram's fates over UDP, a stream's over
 * TCP. The Impairment screen and the experiment's Impairment nodes share it.
 */
export function ImpairProfileFields({ profile, onChange, disabled, protocol = "udp" }: { profile: ImpairProfile; onChange: (next: ImpairProfile) => void; disabled?: boolean; protocol?: RelayProtocol }) {
  const t = useT();
  const offlineId = useId();
  const rateId = useId();
  const lengthId = useId();
  const full = fullProfile(profile);
  const preset = presetOf(profile, protocol);
  const set = (change: Partial<ImpairProfile>) => onChange(changedProfile(full, change, protocol));
  const ms = ` ${t("unit.ms")}`;
  const tcp = protocol === "tcp";
  return <div className="impair-fields">
    <div className="chips impair-presets" role="group" aria-label={t("ns.preset")}>
      {IMPAIR_PRESETS.map((key) => (
        <button key={key} className={"chip" + (preset === key ? " on" : "")} aria-pressed={preset === key} disabled={disabled}
          data-tip={t(`${tcp ? "ns.presetHintTcp" : "ns.presetHint"}.${key}` as TKey)} onClick={() => onChange(presetProfile(key, protocol))}>
          {t(`ns.preset.${key}` as TKey)}
        </button>
      ))}
    </div>
    <label className="checkbox impair-offline" htmlFor={offlineId} data-tip={t(tcp ? "ns.offlineHintTcp" : "ns.offlineHint")}>
      <input id={offlineId} type="checkbox" checked={full.offline} disabled={disabled} onChange={(event) => set({ offline: event.target.checked })} />
      {t("ns.offline")}
    </label>
    <Slider label={t("ns.latency")} tip={t(tcp ? "ns.latencyHintTcp" : "ns.latencyHint")} value={full.latency_ms} onChange={(latency_ms) => set({ latency_ms })} min={0} max={1000} step={5} unit={ms} disabled={disabled} />
    <Slider label={t("ns.jitter")} tip={t(tcp ? "ns.jitterHintTcp" : "ns.jitterHint")} value={full.jitter_ms} onChange={(jitter_ms) => set({ jitter_ms })} min={0} max={500} step={5} unit={ms} disabled={disabled} />
    {tcp && <>
      <Slider label={t("ns.reset")} tip={t("ns.resetHint")} value={pct(full.reset)} onChange={(value) => set({ reset: value / 100 })} min={0} max={100} step={0.5} unit="%" disabled={disabled} />
      <Slider label={t("ns.stall")} tip={t("ns.stallHint")} value={pct(full.stall)} onChange={(value) => set({ stall: value / 100 })} min={0} max={100} step={0.5} unit="%" disabled={disabled} />
    </>}
    {!tcp && <><Slider label={t("ns.loss")} tip={t("ns.lossHint")} value={pct(full.loss)} onChange={(value) => set({ loss: value / 100 })} min={0} max={100} step={0.5} unit="%" disabled={disabled} />
    <Slider label={t("ns.burst")} tip={t("ns.burstHint")} value={pct(full.burst_start)} onChange={(value) => set({ burst_start: value / 100, burst_length: value > 0 && full.burst_length < 1 ? 5 : full.burst_length })} min={0} max={20} step={0.5} unit="%" disabled={disabled} />
    {full.burst_start > 0 && <div className="field">
      <label htmlFor={lengthId} data-tip={t("ns.burstLengthHint")}>{t("ns.burstLength")}</label>
      <input id={lengthId} type="number" min={1} max={1000} value={full.burst_length} disabled={disabled}
        onChange={(event) => set({ burst_length: Math.min(1000, Math.max(1, Math.round(Number(event.target.value) || 1))) })} />
    </div>}
    <Slider label={t("ns.duplicate")} tip={t("ns.duplicateHint")} value={pct(full.duplicate)} onChange={(value) => set({ duplicate: value / 100 })} min={0} max={100} step={1} unit="%" disabled={disabled} />
    <Slider label={t("ns.corrupt")} tip={t("ns.corruptHint")} value={pct(full.corrupt)} onChange={(value) => set({ corrupt: value / 100 })} min={0} max={100} step={1} unit="%" disabled={disabled} />
    <Slider label={t("ns.reorder")} tip={t("ns.reorderHint")} value={pct(full.reorder)} onChange={(value) => set({ reorder: value / 100 })} min={0} max={100} step={1} unit="%" disabled={disabled} /></>}
    <div className="field">
      <label htmlFor={rateId} data-tip={t(tcp ? "ns.rateHintTcp" : "ns.rateHint")}>{t("ns.rate")}</label>
      <RateField id={rateId} value={full.rate_kbps} disabled={disabled} onChange={(rate_kbps) => set({ rate_kbps })} />
    </div>
  </div>;
}
