/**
 * Emulators in the interface: new ones with sensible defaults, response
 * presets, a route made from a response that worked ("Mock this"), the
 * starter set in the reader's language. Pure — the engine checks and runs
 * them (`engine/src/emulator.rs`).
 */
import type {
  Condition, Emulator, EmulatorProtocol, EmulatorResponse, HttpResponse, OscRule, Route, StoredEmulator, TcpRule, UdpRule,
} from "./api";
import type { TKey } from "./i18n";

export const PROTOCOLS: EmulatorProtocol[] = ["http", "osc", "udp", "tcp"];

/**
 * The starter set's ids, renamed into the reader's language once, when it is
 * written (like the signal library's). A test keeps this equal to the engine's `seed()`.
 */
export const EMULATOR_SEED_IDS = ["demo-api", "osc-device", "udp-device", "tcp-device"] as const;

/** The starter emulators in the reader's language; `text` gives a dictionary text, or null. */
export function localizeEmulatorSeed(list: StoredEmulator[], text: (key: string) => string | null): StoredEmulator[] {
  const seeded = new Set<string>(EMULATOR_SEED_IDS);
  return list.map((stored) => !seeded.has(stored.id) ? stored : {
    ...stored,
    note: text(`seed.emu.${stored.id}.note`) ?? stored.note,
    emulator: { ...stored.emulator, name: text(`seed.emu.${stored.id}.name`) ?? stored.emulator.name },
  });
}

/** The first port a new emulator of each protocol suggests; the next free one after it is taken. */
const FIRST_PORT: Record<EmulatorProtocol, number> = { http: 18080, osc: 9100, udp: 7100, tcp: 7200 };

/** TCP (HTTP, TCP) or UDP (OSC, UDP): two of one kind cannot share a port. */
export const overTcp = (protocol: EmulatorProtocol) => protocol === "http" || protocol === "tcp";

export function portOf(bind: string): number | null {
  const port = Number(bind.slice(bind.lastIndexOf(":") + 1));
  return Number.isInteger(port) && port > 0 ? port : null;
}

/** A loopback address no emulator of the library already listens on for this transport. */
export function freeBind(protocol: EmulatorProtocol, taken: StoredEmulator[]): string {
  const used = new Set(taken.filter((stored) => overTcp(stored.emulator.protocol) === overTcp(protocol)).map((stored) => portOf(stored.emulator.bind)));
  let port = FIRST_PORT[protocol];
  while (used.has(port)) port += 1;
  return `127.0.0.1:${port}`;
}

export const blankResponse = (): EmulatorResponse => ({ status: 200, headers: [], body: "", delay_ms: 0, jitter_ms: 0, fault: "none", weight: 1 });

export const PRESETS = ["ok", "created", "notFound", "error", "unavailable", "slow", "timeout", "reset"] as const;
export type Preset = (typeof PRESETS)[number];
export const presetLabel = (preset: Preset): TKey => `emu.preset.${preset}`;

export function presetResponse(preset: Preset): EmulatorResponse {
  const json = (status: number, body: string): EmulatorResponse => ({ ...blankResponse(), status, body });
  switch (preset) {
    case "ok": return json(200, "{\"ok\":true}");
    case "created": return { ...json(201, "{\"id\":\"{{uuid}}\"}"), headers: [["Location", "{{request.path}}/{{counter}}"]] };
    case "notFound": return json(404, "{\"error\":\"not found\"}");
    case "error": return json(500, "{\"error\":\"internal\"}");
    case "unavailable": return { ...json(503, "{\"error\":\"unavailable\"}"), headers: [["Retry-After", "1"]] };
    case "slow": return { ...json(200, "{\"ok\":true}"), delay_ms: 2000 };
    case "timeout": return { ...blankResponse(), fault: "timeout" };
    case "reset": return { ...blankResponse(), fault: "reset" };
  }
}

export const blankRoute = (): Route => ({ method: "GET", path: "/", when: [], order: "sequence", responses: [presetResponse("ok")] });
export const blankCondition = (): Condition => ({ on: "header", name: "", op: "eq", value: "" });
export const blankOscRule = (): OscRule => ({ address: "/ping", args: [], reply: { address: "/pong", args: [{ type: "int", value: "{{counter}}" }] }, to: "", delay_ms: 0, jitter_ms: 0 });
export const blankUdpRule = (): UdpRule => ({ mode: "contains", pattern: "PING", reply: { kind: "text", text: "PONG {{counter}}" }, to: "", delay_ms: 0, jitter_ms: 0 });
export const blankTcpRule = (): TcpRule => ({ mode: "contains", pattern: "PING", reply: { kind: "text", text: "PONG" }, close: false, delay_ms: 0, jitter_ms: 0 });

/** A new emulator of `protocol`: one rule that works as it stands. */
export function blankEmulator(protocol: EmulatorProtocol, name: string, bind: string): Emulator {
  switch (protocol) {
    case "http": return { name, bind, protocol, routes: [{ ...blankRoute(), path: "/health", responses: [{ ...blankResponse(), body: "{\"status\":\"ok\"}" }] }], fallback: null };
    case "osc": return { name, bind, protocol, rules: [blankOscRule()] };
    case "udp": return { name, bind, protocol, rules: [blankUdpRule()] };
    case "tcp": return { name, bind, protocol, delimiter: "lf", greeting: "", rules: [blankTcpRule()] };
  }
}

export function ruleCount(emulator: Emulator): number {
  return emulator.protocol === "http" ? emulator.routes.length : emulator.rules.length;
}

/** An id from the name, unique in the library: `demo-api`, `demo-api-2`. */
export function makeEmulatorId(name: string, taken: StoredEmulator[]): string {
  const base = name.toLowerCase().normalize("NFKD").replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "") || "emulator";
  const ids = new Set(taken.map((stored) => stored.id));
  if (!ids.has(base)) return base;
  for (let n = 2; ; n++) if (!ids.has(`${base}-${n}`)) return `${base}-${n}`;
}

const SCHEME = "http:";

/**
 * Where a person reaches it. An emulator on every address (0.0.0.0, [::]) is
 * reached at `host` — the server's name when the engine runs on a server —
 * else at this machine's loopback.
 */
export function emulatorUrl(emulator: Emulator, local?: string, host?: string): string | null {
  if (emulator.protocol !== "http") return null;
  const everywhere = host ? `${host.includes(":") ? `[${host}]` : host}:` : null;
  const address = (local ?? emulator.bind)
    .replace(/^0\.0\.0\.0:/, everywhere ?? "127.0.0.1:")
    .replace(/^\[::\]:/, everywhere ?? "[::1]:");
  return `${SCHEME}//${address}`;
}

/** Text as it is, for a field read as a template: `{{` that is not one is written `\{{`. */
export const literal = (text: string) => text.replace(/\{\{/g, "\\{{");

/** Headers a recorded response carried that the emulator writes itself, or that tie it to that one exchange. */
const HOP_HEADERS = new Set(["content-length", "transfer-encoding", "connection", "keep-alive", "date", "server", "vary", "etag", "last-modified", "age", "via", "alt-svc"]);

/** "Mock this": a route that answers `method` + the URL's path with exactly this response. */
export function routeFromResponse(method: string, url: string, response: HttpResponse): Route {
  let path = "/";
  try {
    path = new URL(url).pathname || "/";
  } catch {
    // A URL the request went to always parses; anything else answers on /.
  }
  const headers = response.headers.filter(([name]) => !HOP_HEADERS.has(name.toLowerCase())).map(([name, value]): [string, string] => [name, literal(value)]);
  return {
    method: method.toUpperCase(),
    path: literal(path),
    when: [],
    order: "sequence",
    responses: [{ ...blankResponse(), status: response.status || 200, headers, body: literal(response.body) }],
  };
}

/** One line about an emulator: `HTTP · 127.0.0.1:8080 · 5 rules` (the rules counted by the caller's words). */
export function protocolLabel(protocol: EmulatorProtocol): string {
  return protocol.toUpperCase();
}
