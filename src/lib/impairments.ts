/**
 * Impairment profiles in the interface: the presets, which preset a profile
 * still is, and what a profile does in the notation the engine's timeline
 * uses. Pure — the relay itself is the engine's (`engine/src/netsim.rs`).
 */
import type { ImpairProfile, RelayProtocol } from "./api";

export const IMPAIR_PRESETS = ["lan", "wifi", "4g", "satellite", "intermittent", "offline"] as const;
export type ImpairPreset = (typeof IMPAIR_PRESETS)[number];

const CLEAN: Required<ImpairProfile> = {
  name: "", latency_ms: 0, jitter_ms: 0, loss: 0, duplicate: 0, corrupt: 0, reorder: 0, rate_kbps: 0, burst_start: 0, burst_length: 0, offline: false, reset: 0, stall: 0,
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

/**
 * The same links for a TCP stream: what a stream meets of them — the delay and
 * the bandwidth (a stream retransmits its losses and orders itself), and on an
 * intermittent link, now and then a connection left half-open.
 */
const TCP_VALUES: Record<ImpairPreset, Partial<ImpairProfile>> = {
  lan: { latency_ms: 1, jitter_ms: 1 },
  wifi: { latency_ms: 20, jitter_ms: 30 },
  "4g": { latency_ms: 60, jitter_ms: 25, rate_kbps: 20000 },
  satellite: { latency_ms: 300, jitter_ms: 30, rate_kbps: 2000 },
  intermittent: { latency_ms: 30, jitter_ms: 20, stall: 0.002 },
  offline: { offline: true },
};

/** Every field filled in, as the engine reads a profile. */
export const fullProfile = (profile?: ImpairProfile | null): Required<ImpairProfile> => ({ ...CLEAN, ...profile });

/** What a relay of `protocol` reads of a profile (the engine's `for_protocol`): the other protocol's values left out. */
export function forProtocol(profile: ImpairProfile, protocol: RelayProtocol = "udp"): Required<ImpairProfile> {
  const full = fullProfile(profile);
  return protocol === "udp" ? { ...full, reset: 0, stall: 0 } : { ...full, loss: 0, duplicate: 0, corrupt: 0, reorder: 0, burst_start: 0, burst_length: 0 };
}

export const presetProfile = (preset: ImpairPreset, protocol: RelayProtocol = "udp"): Required<ImpairProfile> =>
  ({ ...CLEAN, ...(protocol === "tcp" ? TCP_VALUES : VALUES)[preset], name: preset });

/** The preset a profile is for a relay of `protocol`, while what that relay reads of it is still the preset's. */
export function presetOf(profile: ImpairProfile, protocol: RelayProtocol = "udp"): ImpairPreset | null {
  const name = profile.name as ImpairPreset;
  if (!IMPAIR_PRESETS.includes(name)) return null;
  const preset = presetProfile(name, protocol);
  const read = forProtocol(profile, protocol);
  return (Object.keys(CLEAN) as (keyof ImpairProfile)[]).every((key) => preset[key] === read[key]) ? name : null;
}

/**
 * A profile with `change` applied. A preset's name goes once its values are
 * not the preset's any more: the timeline must not say "4G" for what is not.
 */
export function changedProfile(profile: ImpairProfile, change: Partial<ImpairProfile>, protocol: RelayProtocol = "udp"): Required<ImpairProfile> {
  const next = { ...fullProfile(profile), ...change };
  if (IMPAIR_PRESETS.includes(next.name as ImpairPreset) && !presetOf(next, protocol)) next.name = "";
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
  for (const [value, what] of [[full.loss, "loss"], [full.duplicate, "dup"], [full.corrupt, "corrupt"], [full.reorder, "reorder"], [full.reset, "reset"], [full.stall, "stall"]] as const) {
    if (value > 0) parts.push(`${what} ${percent(value)}`);
  }
  if (full.burst_start > 0) parts.push(`bursts ${percent(full.burst_start)} × ${full.burst_length}`);
  if (full.rate_kbps > 0) parts.push(`${full.rate_kbps} kbps`);
  return parts.length ? parts.join(" · ") : "clean";
}

/** How a profile is named on the canvas: a preset in the reader's language, else what it does on a relay of `protocol`. */
export function profileLabel(profile: ImpairProfile, presetName: (preset: ImpairPreset) => string, protocol: RelayProtocol = "udp"): string {
  const preset = presetOf(profile, protocol);
  if (preset) return presetName(preset);
  const name = (profile.name ?? "").trim();
  return name || impairNotation(forProtocol(profile, protocol));
}
