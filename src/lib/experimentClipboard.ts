import type { Experiment, ExperimentNode } from "./api";
import { NODE_CATALOG } from "./experimentCatalog.ts";
import { addressPort, nodePorts, removeNode, NODE_HEIGHT, NODE_WIDTH, type GraphEdge } from "./experimentGraph.ts";

/**
 * Several nodes at once: what Copy puts on the clipboard, what Paste and
 * Duplicate add, and what a selection removes or moves — pure functions over
 * the document, so the editor and the tests mean the same thing by them.
 *
 * A clip is text a person can paste into another experiment, or another
 * window: the nodes and the wires between them, never Start or End (one of a
 * kind). Pasted, every node gets a new id; a node that names another by id
 * (Change impairment, Emulator down/up, the WebSocket nodes) names the copy
 * when the other was copied too.
 */
export interface Clip { signalLab: "nodes"; version: 1; nodes: ExperimentNode[]; edges: GraphEdge[] }

/** Which field of a node names another node, by the node's type. */
const REFERENCES: Partial<Record<ExperimentNode["type"], { field: "relay" | "emulator" | "connection"; to: ExperimentNode["type"] }>> = {
  impairment_change: { field: "relay", to: "impairment" },
  emulator_state: { field: "emulator", to: "emulator" },
  ws_send: { field: "connection", to: "ws_connect" },
  wait_ws: { field: "connection", to: "ws_connect" },
  ws_close: { field: "connection", to: "ws_connect" },
};

const unique = (node: ExperimentNode) => node.type === "start" || node.type === "end";

/** The selected nodes and the wires between them; none when only Start and End were selected. */
export function copyOf(doc: Experiment, ids: Iterable<string>): Clip | null {
  const wanted = new Set(ids);
  const nodes = doc.nodes.filter((node) => wanted.has(node.id) && !unique(node));
  if (!nodes.length) return null;
  const kept = new Set(nodes.map((node) => node.id));
  const edges = doc.edges.filter((edge) => kept.has(edge.from) && kept.has(edge.to));
  return { signalLab: "nodes", version: 1, nodes: structuredClone(nodes), edges: structuredClone(edges) };
}

export const clipText = (clip: Clip): string => JSON.stringify(clip, null, 2);

/** A clip from pasted text: only what Copy wrote — nodes of known kinds with ids and places. */
export function readClip(text: string): Clip | null {
  let value: unknown;
  try { value = JSON.parse(text); } catch { return null; }
  const clip = value as Partial<Clip> | null;
  if (!clip || clip.signalLab !== "nodes" || clip.version !== 1 || !Array.isArray(clip.nodes) || !Array.isArray(clip.edges) || !clip.nodes.length) return null;
  const valid = (node: ExperimentNode) => !!node && typeof node.id === "string" && typeof node.type === "string" && node.type in NODE_CATALOG
    && !unique(node) && Number.isFinite(node.x) && Number.isFinite(node.y);
  if (!clip.nodes.every(valid)) return null;
  const ids = new Set(clip.nodes.map((node) => node.id));
  if (ids.size !== clip.nodes.length) return null;
  const edges = clip.edges.filter((edge) => !!edge && ids.has(edge.from) && ids.has(edge.to));
  return { signalLab: "nodes", version: 1, nodes: clip.nodes, edges };
}

/** `host:port` with the next port up from its own that is not in `taken`. */
function movedOff(address: string, taken: Set<number>): string {
  const [port] = addressPort(address);
  if (port === undefined || !taken.has(port)) return address;
  let next = port + 1;
  while (taken.has(next) && next < 65535) next += 1;
  return `${address.slice(0, address.lastIndexOf(":") + 1)}${next}`;
}

/**
 * The clip's nodes with ids of their own, for `doc`: a reference to a copied
 * node follows it; one to a node left behind stays when `doc` has it, else
 * names `doc`'s first node of that kind (as a new node does), or none. An
 * Emulator or Impairment whose port `doc` already listens on moves to the next
 * free one — two cannot share it; what sends to the original still does.
 */
export function freshCopy(doc: Experiment, clip: Clip): Clip {
  const renamed = new Map(clip.nodes.map((node) => [node.id, `${node.type}-${crypto.randomUUID()}`]));
  const nodes = clip.nodes.map((source) => {
    const node = structuredClone(source) as ExperimentNode & Record<string, unknown>;
    node.id = renamed.get(source.id)!;
    const reference = REFERENCES[node.type];
    if (reference) {
      const target = node[reference.field] as string;
      node[reference.field] = renamed.get(target)
        ?? (doc.nodes.some((candidate) => candidate.id === target) ? target : doc.nodes.find((candidate) => candidate.type === reference.to)?.id ?? "");
    }
    return node as ExperimentNode;
  });
  const taken = new Set(doc.nodes.flatMap(nodePorts));
  for (const node of nodes) {
    if (node.type === "emulator") node.emulator.bind = movedOff(node.emulator.bind, taken);
    if (node.type === "impairment") node.listen = movedOff(node.listen, taken);
    for (const port of nodePorts(node)) taken.add(port);
  }
  const edges = clip.edges.map((edge) => ({ ...edge, from: renamed.get(edge.from)!, to: renamed.get(edge.to)! }));
  return { ...clip, nodes, edges };
}

const overlapping = (a: ExperimentNode, b: ExperimentNode) => Math.abs(a.x - b.x) < NODE_WIDTH + 12 && Math.abs(a.y - b.y) < NODE_HEIGHT + 12;

/**
 * `copy` (from `freshCopy`) added to `doc`, below and to the right of where it
 * came from, moved further down until none of it covers a node already there.
 */
export function placeCopy(doc: Experiment, copy: Clip): Experiment {
  const step = { x: 32, y: NODE_HEIGHT + 32 };
  let dx = step.x;
  let dy = step.y;
  const at = (node: ExperimentNode): ExperimentNode => ({ ...node, x: Math.max(12, Math.round(node.x + dx)), y: Math.max(12, Math.round(node.y + dy)) });
  for (let tries = 0; tries < 60 && copy.nodes.some((node) => doc.nodes.some((other) => overlapping(at(node), other))); tries++) dy += step.y;
  return { ...doc, nodes: [...doc.nodes, ...copy.nodes.map(at)], edges: [...doc.edges, ...copy.edges] };
}

/** The selected nodes removed, one after another, each leaving the bridge `removeNode` leaves; Start and End stay. */
export function removeNodes(doc: Experiment, ids: Iterable<string>): Experiment {
  let current = doc;
  for (const id of ids) current = removeNode(current, id);
  return current;
}

/** The selected nodes moved together, none above or left of the canvas's margin. */
export function moveNodes(doc: Experiment, ids: Iterable<string>, dx: number, dy: number): Experiment {
  const moving = new Set(ids);
  const nodes = doc.nodes.filter((node) => moving.has(node.id));
  if (!nodes.length) return doc;
  // The group keeps its shape: it stops at the margin as a whole.
  const x = Math.max(dx, 12 - Math.min(...nodes.map((node) => node.x)));
  const y = Math.max(dy, 12 - Math.min(...nodes.map((node) => node.y)));
  if (x === 0 && y === 0) return doc;
  return { ...doc, nodes: doc.nodes.map((node) => moving.has(node.id) ? { ...node, x: Math.round(node.x + x), y: Math.round(node.y + y) } : node) };
}

/**
 * A drag: the nodes put where it began (`start`) plus the pointer's travel,
 * the group stopping at the margin as a whole. Absolute, so a drag of many
 * moves never adds up rounding.
 */
export function dragNodes(doc: Experiment, start: Map<string, { x: number; y: number }>, dx: number, dy: number): Experiment {
  const from = [...start.values()];
  if (!from.length) return doc;
  const x = Math.max(dx, 12 - Math.min(...from.map((point) => point.x)));
  const y = Math.max(dy, 12 - Math.min(...from.map((point) => point.y)));
  let changed = false;
  const nodes = doc.nodes.map((node) => {
    const origin = start.get(node.id);
    if (!origin) return node;
    const next = { x: Math.round(origin.x + x), y: Math.round(origin.y + y) };
    if (next.x === node.x && next.y === node.y) return node;
    changed = true;
    return { ...node, ...next };
  });
  return changed ? { ...doc, nodes } : doc;
}

/** The nodes a rectangle (in graph units, any corner first) touches. */
export function nodesIn(nodes: ExperimentNode[], a: { x: number; y: number }, b: { x: number; y: number }): string[] {
  const left = Math.min(a.x, b.x);
  const right = Math.max(a.x, b.x);
  const top = Math.min(a.y, b.y);
  const bottom = Math.max(a.y, b.y);
  return nodes.filter((node) => node.x < right && node.x + NODE_WIDTH > left && node.y < bottom && node.y + NODE_HEIGHT > top).map((node) => node.id);
}
