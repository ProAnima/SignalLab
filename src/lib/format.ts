import { formatNumber } from "./translate.ts";

/** Unit names for sizes, from the dictionary of the current language. */
export interface SizeUnits { b: string; kb: string; mb: string; gb: string }

let language = "en";
let units: SizeUnits = { b: "B", kb: "KB", mb: "MB", gb: "GB" };

/**
 * Numbers and sizes follow the interface language (grouping, decimal sign,
 * unit names). Set by the language provider before anything renders.
 */
export function setFormatLanguage(lang: string, names: SizeUnits) {
  language = lang;
  units = names;
}

/**
 * A latency as precise as it is short — a loopback answer in 0.4 ms is not
 * "0 ms" — and in seconds from one, so a timeout fits where 0.4 ms does.
 * The value and the key of its unit.
 */
export function latencyParts(ms: number): [string, "unit.ms" | "unit.s"] {
  if (ms >= 1000) return [formatNumber(ms / 1000, language, ms < 10_000 ? 2 : 1), "unit.s"];
  return [formatNumber(ms, language, ms < 10 ? 2 : ms < 100 ? 1 : 0), "unit.ms"];
}

/** A rate as it was typed: 0.5/s is not "1/s". */
export function fmtRate(rate: number): string {
  return formatNumber(rate, language, Number.isInteger(rate) ? 0 : rate < 1 ? 2 : 1);
}

export function fmtBytes(n: number): string {
  if (n < 1024) return `${formatNumber(n, language, 0)} ${units.b}`;
  if (n < 1024 * 1024) return `${formatNumber(n / 1024, language, 1)} ${units.kb}`;
  if (n < 1024 * 1024 * 1024) return `${formatNumber(n / 1024 / 1024, language, 2)} ${units.mb}`;
  return `${formatNumber(n / 1024 / 1024 / 1024, language, 2)} ${units.gb}`;
}

export function fmtNum(n: number, digits = 0): string {
  return formatNumber(n, language, digits);
}

export function fmtTime(ts: number): string {
  const d = new Date(ts);
  const p = (x: number, n = 2) => String(x).padStart(n, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}.${p(d.getMilliseconds(), 3)}`;
}

export function statusClass(code: number): string {
  return `status-${Math.floor(code / 100)}`;
}

/**
 * A JSON body, indented for reading — or null when it is not a JSON object or
 * array (or too large to be worth reformatting on every render).
 */
export function prettyJson(text: string): string | null {
  const trimmed = text.trim();
  if (trimmed.length > 1024 * 1024 || !/^[[{]/.test(trimmed)) return null;
  try {
    return JSON.stringify(JSON.parse(trimmed), null, 2);
  } catch {
    return null;
  }
}
