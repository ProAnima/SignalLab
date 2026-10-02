/**
 * Impairment profiles in the interface: the presets, which preset a profile
 * still is, and what a profile does in the notation the engine's timeline
 * uses. Pure — the relay itself is the engine's (`engine/src/netsim.rs`).
 */
import type { ImpairProfile } from "./api";

export const IMPAIR_PRESETS = ["lan", "wifi", "4g", "satellite", "intermittent", "offline"] as const;
export type ImpairPreset = (typeof IMPAIR_PRESETS)[number];

const CLEAN: Required<ImpairProfile> = {
  name: "", latency_ms: 0, jitter_ms: 0, loss: 0, duplicate: 0, corrupt: 0, reorder: 0, rate_kbps: 0, burst_start: 0, burst_length: 0, offline: false,
};

/** What each preset does: links as they are met in the field, from a cable to none at all. */
const VALUES: Record<ImpairPreset, Partial<ImpairProfile>> = {
  lan: { latency_ms: 1, jitter_ms: 1 },
  wifi: { latency_ms: 20, jitter_ms: 30, loss: 0.01, duplicate: 0.005, reorder: 0.02, burst_start: 0.01, burst_length: 3 },
  "4g": { latency_ms: 60, jitter_ms: 25, loss: 0.005, reorder: 0.005, rate_kbps: 20000 },
  satellite: { latency_ms: 300, jitter_ms: 30, loss: 0.01, rate_kbps: 2000 },
  intermittent: { latency_ms: 30, jitter_ms: 20, burst_start: 0.03, burst_length: 15 },
  offline: { offline: true },
};

/** Every field filled in, as the engine reads a profile. */
export const fullProfile = (profile?: ImpairProfile | null): Required<ImpairProfile> => ({ ...CLEAN, ...profile });

export const presetProfile = (preset: ImpairPreset): Required<ImpairProfile> => ({ ...CLEAN, ...VALUES[preset], name: preset });

/** The preset a profile is, while its values are still the preset's. */
export function presetOf(profile: ImpairProfile): ImpairPreset | null {
  const name = profile.name as ImpairPreset;
  if (!IMPAIR_PRESETS.includes(name)) return null;
  const preset = presetProfile(name);
  const full = fullProfile(profile);
  return (Object.keys(CLEAN) as (keyof ImpairProfile)[]).every((key) => preset[key] === full[key]) ? name : null;
}

/**
 * A profile with `change` applied. A preset's name goes once its values are
 * not the preset's any more: the timeline must not say "4G" for what is not.
 */
export function changedProfile(profile: ImpairProfile, change: Partial<ImpairProfile>): Required<ImpairProfile> {
  const next = { ...fullProfile(profile), ...change };
  if (IMPAIR_PRESETS.includes(next.name as ImpairPreset) && !presetOf(next)) next.name = "";
  return next;
}

const percent = (value: number) => `${Math.round(value * 1000) / 10}%`;

/**
 * What a profile does, as the engine's `ImpairProfile::label` writes it when
 * the profile has no name: `60 ms ±25 · loss 2% · 20000 kbps`.
 */
export function impairNotation(profile: ImpairProfile): string {
  const full = fullProfile(profile);
  if (full.offline) return "offline";
  const parts: string[] = [];
  if (full.latency_ms > 0 || full.jitter_ms > 0) parts.push(full.jitter_ms > 0 ? `${full.latency_ms} ms ±${full.jitter_ms}` : `${full.latency_ms} ms`);
  for (const [value, what] of [[full.loss, "loss"], [full.duplicate, "dup"], [full.corrupt, "corrupt"], [full.reorder, "reorder"]] as const) {
    if (value > 0) parts.push(`${what} ${percent(value)}`);
  }
  if (full.burst_start > 0) parts.push(`bursts ${percent(full.burst_start)} × ${full.burst_length}`);
  if (full.rate_kbps > 0) parts.push(`${full.rate_kbps} kbps`);
  return parts.length ? parts.join(" · ") : "clean";
}

/** How a profile is named on the canvas: a preset in the reader's language, else what it does. */
export function profileLabel(profile: ImpairProfile, presetName: (preset: ImpairPreset) => string): string {
  const preset = presetOf(profile);
  if (preset) return presetName(preset);
  const name = (profile.name ?? "").trim();
  return name || impairNotation(profile);
}
