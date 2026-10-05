import type { Frame, JobInfo, Signal, SignalBody } from "./api";
import type { TKey } from "./locales/en";
import type { Translate } from "./i18n";

/**
 * The starter set the engine writes on first run (`engine/src/signals.rs`,
 * `seed()`), by id. Its names, notes and folders are texts of the interface —
 * `seed.<id>.name`, `seed.<id>.note`, `seed.folder.<group>` — so a library
 * starts in the language of the person who opened it; after that it is their
 * document. A test keeps this list equal to the engine's.
 */
export const SEED_IDS = [
  "osc-fader", "osc-types", "osc-id-and-value", "osc-trigger",
  "mqtt-publish", "mqtt-retained", "mqtt-clear-retained", "http-reachable", "udp-raw",
] as const;

/** The starter signals in the reader's language; `text` gives a dictionary text, or null when there is none. */
export function localizeSeed(signals: Signal[], text: (key: string) => string | null): Signal[] {
  const seeded = new Set<string>(SEED_IDS);
  return signals.map((signal) => !seeded.has(signal.id) ? signal : {
    ...signal,
    name: text(`seed.${signal.id}.name`) ?? signal.name,
    note: text(`seed.${signal.id}.note`) ?? signal.note,
    group: signal.group && (text(`seed.folder.${signal.group}`) ?? signal.group),
  });
}

/**
 * Folders of the signal library. A folder is a path of names joined by "/"
 * ("API/Auth"); "" is the root. A signal's `group` is the path of its folder,
 * so a library from before folders opens with each group as a top-level
 * folder. The file's `folders` lists every folder, so an empty one is kept;
 * a group no folder lists (a library from before folders, a hand edit) is a
 * folder too.
 *
 * Everything here is pure: the view calls these and hands the result to the
 * store, which writes the file.
 */

export const SEPARATOR = "/";

/** "  API / Auth/ " → "API/Auth": names trimmed, empty ones dropped. */
export function normalizeFolder(path: string): string {
  return path.split(SEPARATOR).map((name) => name.trim()).filter(Boolean).join(SEPARATOR);
}

export function parentOf(path: string): string {
  const at = path.lastIndexOf(SEPARATOR);
  return at < 0 ? "" : path.slice(0, at);
}

export function nameOf(path: string): string {
  return path.slice(path.lastIndexOf(SEPARATOR) + 1);
}

export function joinFolder(parent: string, name: string): string {
  return normalizeFolder(parent ? `${parent}${SEPARATOR}${name}` : name);
}

/** `path` itself or anything inside it. */
export function within(path: string, folder: string): boolean {
  return path === folder || path.startsWith(folder + SEPARATOR);
}

/** `path` with its leading `from` replaced by `to` (for paths `within` from). */
function rebase(path: string, from: string, to: string): string {
  return normalizeFolder(to + path.slice(from.length));
}

export interface FolderNode {
  path: string;
  name: string;
  folders: FolderNode[];
  /** Signals directly in this folder, in file order. */
  signals: Signal[];
  /** Signals in this folder and every folder inside it. */
  total: number;
}

/** Every folder path there is — kept ones and those signals are in — with their ancestors, sorted. */
export function allFolders(signals: Signal[], folders: string[]): string[] {
  const paths = new Set<string>();
  for (const path of [...folders, ...signals.map((signal) => signal.group)]) {
    let current = normalizeFolder(path);
    while (current) { paths.add(current); current = parentOf(current); }
  }
  return [...paths].sort(compare);
}

const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
const compare = (a: string, b: string) => collator.compare(a, b);

/** The library as a tree: folders by name (numbers in order), signals in file order. */
export function libraryTree(signals: Signal[], folders: string[]): FolderNode {
  const root: FolderNode = { path: "", name: "", folders: [], signals: [], total: 0 };
  const nodes = new Map<string, FolderNode>([["", root]]);
  for (const path of allFolders(signals, folders)) {
    const node: FolderNode = { path, name: nameOf(path), folders: [], signals: [], total: 0 };
    nodes.set(path, node);
  }
  for (const [path, node] of nodes) if (path) nodes.get(parentOf(path))!.folders.push(node);
  for (const signal of signals) nodes.get(normalizeFolder(signal.group))!.signals.push(signal);
  const count = (node: FolderNode): number => {
    node.folders.sort((a, b) => compare(a.name, b.name));
    node.total = node.signals.length + node.folders.reduce((sum, child) => sum + count(child), 0);
    return node.total;
  };
  count(root);
  return root;
}

/** A name not yet taken among `parent`'s folders: "New folder", "New folder 2", … */
export function freeFolderName(signals: Signal[], folders: string[], parent: string, base: string): string {
  const taken = new Set(allFolders(signals, folders));
  if (!taken.has(joinFolder(parent, base))) return base;
  for (let i = 2; ; i++) if (!taken.has(joinFolder(parent, `${base} ${i}`))) return `${base} ${i}`;
}

export interface LibraryState { signals: Signal[]; folders: string[] }

/**
 * Every folder there is, listed: a folder stays until it is removed, even when
 * its last signal is deleted or moved out — as in a file manager.
 */
function tidy(state: LibraryState): LibraryState {
  return { signals: state.signals, folders: allFolders(state.signals, state.folders) };
}

export function addFolder(state: LibraryState, path: string): LibraryState {
  const clean = normalizeFolder(path);
  return clean ? tidy({ ...state, folders: [...state.folders, clean] }) : state;
}

/**
 * Rename a folder (its last name only). Null when the new name is empty, has
 * a "/" or is taken by a sibling — a rename never merges two folders.
 */
export function renameFolder(state: LibraryState, path: string, name: string): LibraryState | null {
  const clean = name.trim();
  if (!clean || clean.includes(SEPARATOR)) return null;
  const target = joinFolder(parentOf(path), clean);
  if (target === path) return state;
  if (allFolders(state.signals, state.folders).includes(target)) return null;
  return moveTree(state, path, target);
}

/** Move a folder with everything in it into `parent` (merging with a folder of the same name there). */
export function moveFolder(state: LibraryState, path: string, parent: string): LibraryState | null {
  const into = normalizeFolder(parent);
  if (within(into, path)) return null;
  const target = joinFolder(into, nameOf(path));
  return target === path ? state : moveTree(state, path, target);
}

function moveTree(state: LibraryState, from: string, to: string): LibraryState {
  return tidy({
    signals: state.signals.map((signal) => within(normalizeFolder(signal.group), from) ? { ...signal, group: rebase(normalizeFolder(signal.group), from, to) } : signal),
    folders: state.folders.map((folder) => within(folder, from) ? rebase(folder, from, to) : folder),
  });
}

/**
 * Remove a folder but not what is in it: its signals and folders move up to
 * its parent. Nothing on a show machine should vanish with one click.
 */
export function removeFolder(state: LibraryState, path: string): LibraryState {
  const parent = parentOf(path);
  const moved = moveTree(state, path, parent);
  return tidy({ ...moved, folders: moved.folders.filter((folder) => folder !== path) });
}

export function moveSignal(state: LibraryState, id: string, folder: string): LibraryState {
  const clean = normalizeFolder(folder);
  return tidy({ ...state, signals: state.signals.map((signal) => signal.id === id ? { ...signal, group: clean } : signal) });
}

/** Folders on the way to `path`, outermost first ("A", "A/B" for "A/B"): what to open to show it. */
export function ancestors(path: string): string[] {
  const out: string[] = [];
  let current = normalizeFolder(path);
  while (current) { out.unshift(current); current = parentOf(current); }
  return out;
}

/**
 * A message as text that is the same whenever the message is: keys sorted
 * (what comes back from the engine has them in its own order) and OSC floats
 * at the 32-bit precision they are stored with.
 */
export function canonicalBody(body: SignalBody): string {
  return JSON.stringify(body, (_, value) => {
    if (!value || typeof value !== "object" || Array.isArray(value)) return value;
    const entries = Object.entries(value).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
    const sorted = Object.fromEntries(entries);
    return sorted.type === "float" && typeof sorted.value === "number" ? { ...sorted, value: Math.fround(sorted.value) } : sorted;
  });
}

/** "API / Auth / Login": a signal as the library shows where it is. */
export function signalPlace(signal: Signal): string {
  return [...normalizeFolder(signal.group).split("/").filter(Boolean), signal.name].join(" / ");
}

// ---- what a captured frame becomes, and which connection a signal rides ----

/** host:port, split at the last colon so an IPv6 literal survives; 1883 when there is no port. */
export function splitBroker(broker: string): { host: string; port: number } {
  const text = broker.trim();
  if (text.startsWith("[") && text.includes("]")) {
    const end = text.indexOf("]");
    const rest = text.slice(end + 1);
    return { host: text.slice(0, end + 1), port: rest.startsWith(":") ? Number(rest.slice(1)) || 1883 : 1883 };
  }
  const at = text.lastIndexOf(":");
  // No port, or an IPv6 address without brackets (and so without one).
  if (at < 1 || text.indexOf(":") !== at) return { host: text, port: 1883 };
  return { host: text.slice(0, at).trim(), port: Number(text.slice(at + 1)) || 1883 };
}

/**
 * One broker, however it is written: the host ignoring case and an IPv6
 * literal's brackets, the port 1883 when none is given. `localhost` and
 * `127.0.0.1` are two names, not one — nothing here looks names up.
 */
export function sameBroker(a: string, b: string): boolean {
  const key = (text: string) => {
    const { host, port } = splitBroker(text);
    return `${host.replace(/^\[(.*)\]$/, "$1").toLowerCase()}:${port}`;
  };
  return key(a) === key(b);
}

/**
 * The open MQTT connection an MQTT signal to `broker` goes out on: the first
 * one connected to that broker (its job names it, `params.broker`). None, and
 * the signal makes a connection of its own to its own broker.
 */
export function mqttConnectionFor(broker: string, jobs: Pick<JobInfo, "id" | "kind" | "params">[]): number | null {
  return jobs.find((job) => job.kind === "mqtt" && job.params?.broker !== undefined && sameBroker(job.params.broker, broker))?.id ?? null;
}

/**
 * A socket listening on every address is reached here on loopback:
 * `0.0.0.0:9000` → `127.0.0.1:9000`, `[::]:9000` → `[::1]:9000`. Any other
 * address stays as it is.
 */
export function reachable(address: string): string {
  const text = address.trim();
  if (text.startsWith("0.0.0.0:")) return `127.0.0.1${text.slice("0.0.0.0".length)}`;
  if (text.startsWith("[::]:")) return `[::1]${text.slice("[::]".length)}`;
  return text;
}

/** `IP:port` or `host:port` (IPv6 in brackets), the port 1–65535: where a datagram can be sent. */
export function isHostPort(text: string): boolean {
  const match = /^(\[[0-9A-Fa-f:.]+\]|[A-Za-z0-9._-]+):(\d{1,5})$/.exec(text.trim());
  return !!match && Number(match[2]) > 0 && Number(match[2]) < 65536;
}

/**
 * A frame keeps every one of its bytes: it can be sent again as it was. An
 * MQTT message may be empty — that is how a retained value is cleared — but
 * any other frame of no bytes kept nothing.
 */
export const wholeFrame = (frame: Frame) => frame.kept === frame.bytes && (frame.bytes > 0 || !!frame.publish);

/**
 * Where a captured datagram is sent again. A relay's frame — either way —
 * to where it was going (its peer); a received frame to the socket that
 * received it, since saving a reader's packet is to stand in for its sender
 * later; a sent one to where it was sent. A socket on every address is
 * reached on loopback.
 */
export function replayTarget(frame: Frame): string {
  if (frame.source === "netsim") return reachable(frame.remote);
  return reachable(frame.dir === "rx" ? frame.local : frame.remote);
}

/**
 * Why a frame cannot become a signal, as the text key of the button's tip;
 * null when it can. A signal sends one datagram (an OSC or UDP frame) or one
 * MQTT publish with a text payload; a TCP chunk, an HTTP exchange, a
 * WebSocket message or an MQTT control packet has no such form.
 */
export function frameSignalBlock(frame: Frame): TKey | null {
  if (frame.proto === "mqtt") {
    if (!frame.publish) return "ins.noSignalForm";
    if (!wholeFrame(frame)) return "ins.noExactCopy";
    return frame.publish.text ? null : "ins.notText";
  }
  if (frame.proto !== "osc" && frame.proto !== "udp") return "ins.noSignalForm";
  if (!wholeFrame(frame)) return "ins.noExactCopy";
  return isHostPort(replayTarget(frame)) ? null : "ins.noSignalForm";
}

/** `48 65 6c`, `inspect_payload`'s plain hex, as bytes; null when it is not that. */
function hexBytes(hex: string): Uint8Array | null {
  const pairs = hex.trim() === "" ? [] : hex.trim().split(/\s+/);
  if (!pairs.every((pair) => /^[0-9A-Fa-f]{2}$/.test(pair))) return null;
  return Uint8Array.from(pairs, (pair) => parseInt(pair, 16));
}

/**
 * A captured frame as a signal's message, from the bytes the engine kept of
 * it (`hex`, `inspect_payload`'s) — never re-parsed out of a display string,
 * which is where a replay stops being the same packet. A datagram becomes a
 * raw UDP signal with those bytes; an MQTT publish an MQTT signal to its
 * broker, with its topic, QoS, retain flag and the same payload. Null for a
 * frame `frameSignalBlock` refuses, or bytes that are not all of it.
 */
export function frameSignalBody(frame: Frame, hex: string): SignalBody | null {
  if (frameSignalBlock(frame)) return null;
  const bytes = hexBytes(hex);
  if (!bytes || bytes.length !== frame.bytes) return null;
  if (frame.publish) {
    let payload: string;
    try {
      // The bytes as they are: no byte order mark dropped, nothing replaced.
      payload = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
    } catch {
      return null;
    }
    const { broker, topic, qos, retain } = frame.publish;
    return { transport: "mqtt", broker: reachable(broker), topic, payload, qos, retain };
  }
  return { transport: "udp", target: replayTarget(frame), payload: { kind: "hex", hex: hex.trim() } };
}

/** A slug that reads in the file and cannot collide with an existing one. */
export function makeId(name: string, taken: Signal[]): string {
  const base =
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "")
      .slice(0, 40) || "signal";
  const used = new Set(taken.map((s) => s.id));
  if (!used.has(base)) return base;
  for (let i = 2; ; i++) {
    const candidate = `${base}-${i}`;
    if (!used.has(candidate)) return candidate;
  }
}

/**
 * A captured frame, as a replayable signal (`frameSignalBody`: a datagram as
 * raw UDP, an MQTT publish as MQTT), from the bytes the engine kept of it —
 * `inspect_payload`'s plain `hex`. Null for a frame with no signal form, or
 * not kept whole: half a packet replayed is a different packet.
 */
export function signalFromFrame(frame: Frame, hex: string, taken: Signal[], name: string, t: Translate): Signal | null {
  const body = frameSignalBody(frame, hex);
  if (!body) return null;
  const peer = frame.remote || frame.publish?.broker || frame.local;
  return {
    id: makeId(name, taken),
    name,
    group: t("sig.capturedFolder"),
    note: `#${frame.seq} ${frame.proto} ${frame.dir === "rx" ? "←" : "→"} ${peer} · ${frame.summary}`,
    body,
  };
}
