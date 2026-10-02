/**
 * Turning a dictionary text into the finished string. Pure, so it is tested
 * without a browser; `i18n.tsx` wires it to the current language.
 *
 * A text holds two kinds of placeholder:
 *
 * - `{name}` — the value as it is given. Ports, ids and addresses stay
 *   untouched, so a caller that wants a grouped number passes `fmtNum(n)`.
 * - `{name, plural, one {# file} few {# files} other {# files}}` — a branch
 *   chosen by the language's plural rules (`Intl.PluralRules`), `=0 {…}` for an
 *   exact value first. `#` is the number, formatted for the language. The
 *   categories a language needs are its own: English has one/other, Russian
 *   one/few/many/other; `other` is always required.
 * - `{name, select, true { · retain} other {}}` — a branch by the value itself,
 *   `other` when none matches: for a flag or a kind, never for a count.
 *
 * Anything else in braces is text — `{{template}}` examples in the help keep
 * their braces, and an unknown `{name}` is left for the reader to see.
 */

export type Params = Record<string, string | number>;

const pluralRules = new Map<string, Intl.PluralRules>();
const numbers = new Map<string, Intl.NumberFormat>();

function rules(lang: string) {
  let found = pluralRules.get(lang);
  if (!found) pluralRules.set(lang, found = new Intl.PluralRules(lang));
  return found;
}

/** A number as the language writes it (grouping, decimal sign). */
export function formatNumber(value: number, lang: string, digits?: number): string {
  if (digits === undefined) {
    let found = numbers.get(lang);
    if (!found) numbers.set(lang, found = new Intl.NumberFormat(lang, { maximumFractionDigits: 3 }));
    return found.format(value);
  }
  return value.toLocaleString(lang, { minimumFractionDigits: digits, maximumFractionDigits: digits });
}

const SIMPLE = /^\{(\w+)\}/;
const PLURAL = /^\{(\w+)\s*,\s*(plural|select)\s*,/;

/** One plural (or select) placeholder: its value name and its branches by selector (`one`, `=0`, `true`, …). */
export interface Plural { name: string; kind: "plural" | "select"; branches: Map<string, string>; end: number }

/**
 * The plural placeholder that starts at `from` (on its `{`), or null when the
 * text there is not one — unbalanced braces, no branches, a branch without a
 * message.
 */
export function parsePlural(text: string, from: number): Plural | null {
  const head = PLURAL.exec(text.slice(from));
  if (!head) return null;
  const branches = new Map<string, string>();
  let at = from + head[0].length;
  for (;;) {
    while (/\s/.test(text[at] ?? "")) at++;
    if (text[at] === "}") return branches.size ? { name: head[1], kind: head[2] as Plural["kind"], branches, end: at + 1 } : null;
    const selector = /^(=\d+|[A-Za-z0-9_-]+)\s*\{/.exec(text.slice(at));
    if (!selector) return null;
    let depth = 1;
    let index = at + selector[0].length;
    const start = index;
    for (; index < text.length && depth > 0; index++) {
      if (text[index] === "{") depth++;
      else if (text[index] === "}") depth--;
    }
    if (depth > 0) return null;
    branches.set(selector[1], text.slice(start, index - 1));
    at = index;
  }
}

function pick(plural: Plural, value: number, lang: string): string {
  return plural.branches.get(`=${value}`)
    ?? plural.branches.get(rules(lang).select(value))
    ?? plural.branches.get("other")
    ?? "";
}

/** `text` with its placeholders filled in for `lang`. `hash` is what `#` stands for inside a plural branch. */
export function format(text: string, params: Params | undefined, lang: string, hash?: string): string {
  if (!params && hash === undefined) return text;
  let out = "";
  for (let at = 0; at < text.length;) {
    const char = text[at];
    if (char === "#" && hash !== undefined) { out += hash; at++; continue; }
    if (char !== "{" || !params) { out += char; at++; continue; }
    const simple = SIMPLE.exec(text.slice(at));
    if (simple) {
      const name = simple[1];
      out += name in params ? String(params[name]) : simple[0];
      at += simple[0].length;
      continue;
    }
    const plural = parsePlural(text, at);
    if (plural && plural.name in params && plural.kind === "select") {
      const value = String(params[plural.name]);
      out += format(plural.branches.get(value) ?? plural.branches.get("other") ?? "", params, lang, hash);
      at = plural.end;
      continue;
    }
    if (plural && plural.name in params) {
      const value = Number(params[plural.name]);
      out += Number.isFinite(value)
        ? format(pick(plural, value, lang), params, lang, formatNumber(value, lang))
        : String(params[plural.name]);
      at = plural.end;
      continue;
    }
    if (plural) {
      // A value nobody gave: the placeholder stays whole, as an unknown {name} does.
      out += text.slice(at, plural.end);
      at = plural.end;
      continue;
    }
    out += char;
    at++;
  }
  return out;
}

/** The value names a text uses, simple and plural, sorted — for checking that every language asks for the same ones. */
export function placeholders(text: string): string[] {
  const found = new Set<string>();
  for (let at = 0; at < text.length; at++) {
    if (text[at] !== "{") continue;
    const plural = parsePlural(text, at);
    if (plural) {
      found.add(plural.name);
      for (const branch of plural.branches.values()) for (const name of placeholders(branch)) found.add(name);
      at = plural.end - 1;
      continue;
    }
    const simple = SIMPLE.exec(text.slice(at));
    if (simple && text[at - 1] !== "{") found.add(simple[1]);
  }
  return [...found].sort();
}

/** Every plural placeholder in a text, nested ones included. */
export function plurals(text: string): Plural[] {
  const found: Plural[] = [];
  for (let at = 0; at < text.length; at++) {
    if (text[at] !== "{") continue;
    const plural = parsePlural(text, at);
    if (!plural) continue;
    found.push(plural);
    for (const branch of plural.branches.values()) found.push(...plurals(branch));
    at = plural.end - 1;
  }
  return found;
}

/**
 * The first language a person prefers that the interface has, by the base
 * language (`ru-RU` → `ru`), else `fallback`. `preferred` is in order of
 * preference, as `navigator.languages` gives it.
 */
export function pickLanguage<T extends string>(preferred: readonly string[], supported: readonly T[], fallback: T): T {
  for (const tag of preferred) {
    const wanted = tag.trim().toLowerCase();
    const exact = supported.find((code) => code.toLowerCase() === wanted);
    if (exact) return exact;
    const base = supported.find((code) => code.toLowerCase() === wanted.split("-")[0]);
    if (base) return base;
  }
  return fallback;
}
