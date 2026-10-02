import type { Signal, SignalBody } from "./api";

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
