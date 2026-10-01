/**
 * Editor-side helpers for values in experiments. The language itself is
 * resolved by the engine (`experiment_resolve`); these only help the user type
 * it: suggestions, the variables visible at a node, JSON paths for "Extract as
 * variable". See docs/milestone-3-data.md.
 */
import type { Experiment, ExperimentNode, ExperimentProfile, OscReply, Retry, UdpMode, UdpReply } from "./api";
import type { TKey } from "./i18n";

/** Names with a meaning of their own; mirrors `template::RESERVED`. */
export const RESERVED = ["vars", "params", "secret", "run", "node", "now", "uuid", "counter", "random_int", "random_float", "pick"];

export const GENERATORS: { insert: string; description: TKey }[] = [
  { insert: "uuid", description: "exp.gen.uuid" },
  { insert: "now", description: "exp.gen.now" },
  { insert: "now.iso", description: "exp.gen.nowIso" },
  { insert: "counter", description: "exp.gen.counter" },
  { insert: "random_int(1, 100)", description: "exp.gen.randomInt" },
  { insert: "random_float(0, 1)", description: "exp.gen.randomFloat" },
  { insert: "pick(a, b, c)", description: "exp.gen.pick" },
  { insert: "run.seed", description: "exp.gen.seed" },
];

export const isIdent = (name: string) => /^[A-Za-z_][A-Za-z0-9_]*$/.test(name) && !RESERVED.includes(name);

export const hasTemplate = (text: string) => /(^|[^\\])\{\{/.test(text);

/** The variable a node writes and the output it is set on: a wait has no reply on Timeout. */
export function writtenVariable(node: ExperimentNode): { name: string; port: string } | null {
  if (node.type === "extract") return node.variable ? { name: node.variable, port: "next" } : null;
  if (node.type === "wait_osc" || node.type === "wait_udp" || node.type === "wait_mqtt") return node.variable ? { name: node.variable, port: "matched" } : null;
  // A send that waits for its reply passes only with one, so the reply exists on Next.
  if ((node.type === "osc" || node.type === "udp") && node.reply) return node.reply.variable ? { name: node.reply.variable, port: "next" } : null;
  return null;
}

/**
 * Every variable set upstream of `id` (on some path), nearest first: Extract
 * nodes, and waits whose Matched output leads here.
 */
export function variablesBefore(doc: Experiment, id: string): { name: string; node: ExperimentNode }[] {
  const byId = new Map(doc.nodes.map((node) => [node.id, node]));
  // `id` and every node that reaches it, in breadth-first order from `id`.
  const reaches = new Set<string>([id]);
  const order: string[] = [];
  let frontier = [id];
  while (frontier.length) {
    const next: string[] = [];
    for (const current of frontier) for (const edge of doc.edges) {
      if (edge.to !== current || reaches.has(edge.from)) continue;
      reaches.add(edge.from);
      order.push(edge.from);
      next.push(edge.from);
    }
    frontier = next;
  }
  const found: { name: string; node: ExperimentNode }[] = [];
  for (const nodeId of order) {
    const node = byId.get(nodeId);
    const written = node && writtenVariable(node);
    if (!node || !written || found.some((item) => item.name === written.name)) continue;
    const leadsHere = doc.edges.some((edge) => edge.from === nodeId && (edge.port ?? "next") === written.port && reaches.has(edge.to));
    if (leadsHere) found.push({ name: written.name, node });
  }
  return found;
}

/** The fields of a wait's reply value, for suggestions (`reply.args[0]`). */
export function replyFields(node: ExperimentNode): string[] {
  const osc = ["address", "args[0]", "from", "ms"];
  const udp = (mode: UdpMode) => mode === "any" ? ["text", "hex", "bytes", "from", "ms"] : ["text", "match", "hex", "bytes", "from", "ms"];
  if (node.type === "wait_osc" || (node.type === "osc" && node.reply)) return osc;
  if (node.type === "wait_udp") return udp(node.mode);
  if (node.type === "wait_mqtt") return ["topic", ...udp(node.mode)];
  if (node.type === "udp" && node.reply) return udp(node.reply.mode);
  return [];
}

/** A reply to wait for, as the node first gets it: on any free port, the matching of a Wait. */
export function defaultReply(type: "osc"): OscReply;
export function defaultReply(type: "udp"): UdpReply;
export function defaultReply(type: "osc" | "udp"): OscReply | UdpReply {
  return type === "osc"
    ? { bind: "0.0.0.0:0", address: "/*", args: [], timeout_ms: 2000, variable: "reply" }
    : { bind: "0.0.0.0:0", mode: "any", pattern: "", timeout_ms: 2000, variable: "reply" };
}

/** Retry as the node first gets it: three attempts, half a second apart. */
export const DEFAULT_RETRY: Retry = { attempts: 3, delay_ms: 500, backoff: "fixed" };

/** Sends or listens, so a second attempt may succeed (the engine's `NodeKind::retries`). */
export function canRetry(node: ExperimentNode): boolean {
  return ["http", "tcp", "mqtt", "osc", "udp", "wait_osc", "wait_udp", "wait_mqtt"].includes(node.type);
}

/** `$.items[0]["first name"]` from the steps into a JSON value. */
export function jsonPath(segments: (string | number)[]): string {
  return "$" + segments.map((segment) => typeof segment === "number" ? `[${segment}]`
    : /^[A-Za-z0-9_-]+$/.test(segment) ? `.${segment}` : `[${JSON.stringify(segment)}]`).join("");
}

/** A variable name for a picked JSON value: its key, made valid and unique. */
export function suggestVariableName(segments: (string | number)[], taken: string[]): string {
  const key = [...segments].reverse().find((segment): segment is string => typeof segment === "string") ?? "value";
  let base = key.replace(/[^A-Za-z0-9_]+/g, "_").replace(/^_+|_+$/g, "") || "value";
  if (/^\d/.test(base)) base = `v_${base}`;
  if (RESERVED.includes(base)) base = `${base}_value`;
  if (!taken.includes(base)) return base;
  for (let n = 2; ; n++) if (!taken.includes(`${base}_${n}`)) return `${base}_${n}`;
}

/** The unfinished `{{name` right before the caret, for suggestions; null when there is none. */
export function templateAt(text: string, caret: number): { start: number; query: string } | null {
  const before = text.slice(0, caret);
  const open = before.lastIndexOf("{{");
  if (open < 0 || before.slice(open).includes("}}") || (open > 0 && before[open - 1] === "\\")) return null;
  const query = before.slice(open + 2);
  return /^[\sA-Za-z0-9_.[\]"'-]*$/.test(query) ? { start: open, query: query.trim() } : null;
}

// ---- parameters and profiles (pure document edits; see docs/milestone-3-data.md §10) ----

/** Defaults, then the profile, then Run with… values; mirrors `effective_params`. */
export function effectiveParams(doc: Experiment, profile: string | null, overrides: Record<string, string> = {}): Record<string, string> {
  const values: Record<string, string> = Object.fromEntries(doc.params.map((param) => [param.name, param.value]));
  const chosen = doc.profiles.find((item) => item.name === profile);
  for (const [name, value] of Object.entries({ ...chosen?.values, ...overrides })) if (name in values) values[name] = value;
  return values;
}

export function uniqueName(base: string, taken: string[]): string {
  if (!taken.includes(base)) return base;
  for (let n = 2; ; n++) if (!taken.includes(`${base} ${n}`)) return `${base} ${n}`;
}

const mapProfiles = (doc: Experiment, fn: (profile: ExperimentProfile) => ExperimentProfile): Experiment =>
  ({ ...doc, profiles: doc.profiles.map(fn) });

/**
 * Rename a parameter in the defaults and in every profile. While the old or
 * new name is shared with another parameter the owner of a profile value is
 * ambiguous, so profile values are then left where they are.
 */
export function renameParam(doc: Experiment, index: number, name: string): Experiment {
  const old = doc.params[index]?.name;
  if (old === undefined || old === name) return doc;
  const next = { ...doc, params: doc.params.map((param, i) => i === index ? { ...param, name } : param) };
  const shared = (candidate: string) => doc.params.some((param, i) => i !== index && param.name === candidate);
  if (shared(old) || shared(name)) return next;
  return mapProfiles(next, (profile) => {
    if (!(old in profile.values)) return profile;
    const { [old]: value, ...rest } = profile.values;
    return { ...profile, values: { ...rest, [name]: value } };
  });
}

export function removeParam(doc: Experiment, index: number): Experiment {
  const name = doc.params[index]?.name;
  const next = { ...doc, params: doc.params.filter((_, i) => i !== index) };
  if (name === undefined || next.params.some((param) => param.name === name)) return next;
  return mapProfiles(next, (profile) => {
    const { [name]: _removed, ...values } = profile.values;
    return { ...profile, values };
  });
}

/** A default (`profile` null) or a profile value; `null` makes a profile inherit the default again. */
export function setParamValue(doc: Experiment, profile: string | null, name: string, value: string | null): Experiment {
  if (profile === null) return { ...doc, params: doc.params.map((param) => param.name === name ? { ...param, value: value ?? "" } : param) };
  return mapProfiles(doc, (item) => {
    if (item.name !== profile) return item;
    const { [name]: _old, ...rest } = item.values;
    return { ...item, values: value === null ? rest : { ...rest, [name]: value } };
  });
}

export function addProfile(doc: Experiment, base: string): { doc: Experiment; name: string } {
  const name = uniqueName(base, doc.profiles.map((profile) => profile.name));
  return { doc: { ...doc, profiles: [...doc.profiles, { name, values: {} }] }, name };
}

/** Renames the profile and, when it is active, the active reference too. */
export function renameProfile(doc: Experiment, old: string, name: string): Experiment {
  return { ...mapProfiles(doc, (profile) => profile.name === old ? { ...profile, name } : profile), profile: doc.profile === old ? name : doc.profile };
}

export function removeProfile(doc: Experiment, name: string): Experiment {
  return { ...doc, profiles: doc.profiles.filter((profile) => profile.name !== name), profile: doc.profile === name ? null : doc.profile };
}

/** Secret names the experiment's fields mention (`{{secret.NAME}}`), sorted. */
export function secretNames(doc: Experiment): string[] {
  const names = new Set<string>();
  for (const match of JSON.stringify(doc.nodes).matchAll(/\{\{\s*secret\.([A-Za-z_][A-Za-z0-9_]*)/g)) names.add(match[1]);
  return [...names].sort();
}
