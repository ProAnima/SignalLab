import type { Experiment, ExperimentNode, ExperimentPort, HttpRequest } from "./api";

export type NodeType = ExperimentNode["type"];
export type Port = ExperimentPort;
export type GraphEdge = Experiment["edges"][number];
export const NODE_WIDTH = 180;
export const NODE_HEIGHT = 80;
/** Horizontal pitch between a node and the one it feeds; matches the templates. */
export const STEP_X = NODE_WIDTH + 50;
const STEP_Y = NODE_HEIGHT + 48;

const defaultRequest: HttpRequest = {
  method: "GET", url: "http://127.0.0.1:8080/", headers: [], body: null, timeout_ms: 4000,
};

export function createNode(type: NodeType, x: number, y: number): ExperimentNode {
  const id = `${type}-${crypto.randomUUID()}`;
  const base = { id, x: Math.max(12, Math.round(x)), y: Math.max(12, Math.round(y)) };
  switch (type) {
    case "start": return { ...base, type };
    case "end": return { ...base, type };
    case "http": return { ...base, type, request: { ...defaultRequest, headers: [] } };
    case "delay": return { ...base, type, ms: 300 };
    case "assert_status": return { ...base, type, status: 200 };
    case "assert_body": return { ...base, type, contains: "ok" };
    case "assert_header": return { ...base, type, name: "content-type", contains: "application/json" };
    case "assert_latency": return { ...base, type, max_ms: 1000 };
    case "mqtt": return { ...base, type, host: "127.0.0.1", port: 1883, topic: "lab/test", payload: "hello", qos: 0, retain: false };
    case "branch_status": return { ...base, type, status: 200 };
    case "fork": return { ...base, type };
    case "join": return { ...base, type };
    case "tcp": return { ...base, type, host: "127.0.0.1", port: 9000, payload: "hello", timeout_ms: 4000 };
    case "log": return { ...base, type, message: "Check point" };
    case "osc": return { ...base, type, target: "127.0.0.1:9000", address: "/test", args: [] };
    case "udp": return { ...base, type, target: "127.0.0.1:9000", text: "hello" };
    case "extract": return { ...base, type, variable: "token", from: "json", expr: "$.token" };
    case "assert_value": return { ...base, type, value: "{{token}}", op: "not_empty", expected: "" };
    case "branch_value": return { ...base, type, value: "{{token}}", op: "eq", expected: "" };
    // Listens on loopback by default, like every shipped target.
    case "wait_osc": return { ...base, type, bind: "127.0.0.1:9001", address: "/pong", args: [], timeout_ms: 2000, variable: "reply" };
    case "wait_udp": return { ...base, type, bind: "127.0.0.1:9001", mode: "contains", pattern: "pong", timeout_ms: 2000, variable: "reply" };
  }
}

/** Every output a node can have, in the order they are drawn. */
export function validPortsFor(type: NodeType): Port[] {
  switch (type) {
    case "branch_status": case "branch_value": return ["yes", "no"];
    case "fork": return ["branch1", "branch2"];
    case "wait_osc": case "wait_udp": return ["matched", "timeout"];
    case "end": return [];
    default: return ["next"];
  }
}

/** Outputs a runnable graph must connect; a wait's Timeout is optional (unwired, a timeout fails the step). */
export function requiredPortsFor(type: NodeType): Port[] {
  return validPortsFor(type).filter((port) => port !== "timeout");
}

export function portOf(edge: GraphEdge): Port { return edge.port ?? "next"; }

/**
 * An output a new node can be attached to. An output may have several wires
 * (their nodes run in parallel); `to` names one of them.
 */
export interface Anchor { from: string; port: Port; to?: string }

/** The wire an anchor means: the one it names, else the output's first. */
function wireOf(doc: Experiment, anchor: Anchor): GraphEdge | undefined {
  return doc.edges.find((edge) => edge.from === anchor.from && portOf(edge) === anchor.port && (anchor.to === undefined || edge.to === anchor.to));
}

/** Is there a wire from this output to `to` already? */
export function isWired(doc: Experiment, from: string, port: Port, to: string): boolean {
  return doc.edges.some((edge) => edge.from === from && portOf(edge) === port && edge.to === to);
}

/** The first unconnected output, else the first output (a new node then splices in). */
export function defaultPort(doc: Experiment, id: string): Port | null {
  const node = doc.nodes.find((item) => item.id === id);
  const ports = node ? validPortsFor(node.type) : [];
  return ports.find((port) => !doc.edges.some((edge) => edge.from === id && portOf(edge) === port)) ?? ports[0] ?? null;
}

/**
 * Where "add after this node" attaches. End has no outputs, so adding "after"
 * it means adding before it, on its only incoming wire.
 */
export function anchorAfter(doc: Experiment, id: string): Anchor | null {
  const node = doc.nodes.find((item) => item.id === id);
  if (!node) return null;
  if (node.type === "end") {
    const incoming = doc.edges.filter((edge) => edge.to === id);
    return incoming.length === 1 ? { from: incoming[0].from, port: portOf(incoming[0]) } : null;
  }
  const port = defaultPort(doc, id);
  return port ? { from: id, port } : null;
}

function overlaps(nodes: ExperimentNode[], x: number, y: number): boolean {
  return nodes.some((node) => Math.abs(node.x - x) < NODE_WIDTH + 12 && Math.abs(node.y - y) < NODE_HEIGHT + 12);
}

/** A free spot beside the anchor: in line with the wire it joins, or below a sibling. */
export function placeAfter(doc: Experiment, anchor: Anchor): { x: number; y: number } {
  const source = doc.nodes.find((node) => node.id === anchor.from);
  if (!source) return { x: 40, y: 40 };
  const edge = wireOf(doc, anchor);
  const target = edge && doc.nodes.find((node) => node.id === edge.to);
  const x = source.x + STEP_X;
  if (target) return { x, y: target.y };
  let y = source.y + Math.max(0, validPortsFor(source.type).indexOf(anchor.port)) * STEP_Y;
  while (overlaps(doc.nodes, x, y)) y += STEP_Y;
  return { x, y };
}

/**
 * Attach `node` to an output. A connected output keeps its flow: the node is
 * spliced into the wire (the one the anchor names, else the first). Without a
 * usable anchor the node is only added.
 */
export function addAfter(doc: Experiment, anchor: Anchor | null, node: ExperimentNode): Experiment {
  const source = anchor && doc.nodes.find((item) => item.id === anchor.from);
  if (!anchor || !source || !validPortsFor(source.type).includes(anchor.port)) return { ...doc, nodes: [...doc.nodes, node] };
  const edge = wireOf(doc, anchor);
  if (edge) return insertOnEdge(doc, edge, node);
  return { ...doc, nodes: [...doc.nodes, node], edges: [...doc.edges, { from: anchor.from, to: node.id, port: anchor.port }] };
}

/**
 * Attach `node` on a new wire of the output, beside the ones it has: a wire
 * dragged out of a connected output starts parallel work instead of cutting
 * into the existing flow.
 */
export function addBranch(doc: Experiment, anchor: Anchor | null, node: ExperimentNode): Experiment {
  const source = anchor && doc.nodes.find((item) => item.id === anchor.from);
  if (!anchor || !source || !validPortsFor(source.type).includes(anchor.port)) return { ...doc, nodes: [...doc.nodes, node] };
  return { ...doc, nodes: [...doc.nodes, node], edges: [...doc.edges, { from: anchor.from, to: node.id, port: anchor.port }] };
}

/** Outputs that must be wired before the graph can run, as `id:port`. */
export function missingOutputs(doc: Experiment): Set<string> {
  const missing = new Set<string>();
  for (const node of doc.nodes) for (const port of requiredPortsFor(node.type)) {
    if (!doc.edges.some((edge) => edge.from === node.id && portOf(edge) === port)) missing.add(`${node.id}:${port}`);
  }
  return missing;
}

/** Nodes the run can never reach, because no wire leads to them from Start. */
export function unreachableNodes(doc: Experiment): Set<string> {
  const start = doc.nodes.find((node) => node.type === "start");
  const seen = new Set<string>();
  const stack = start ? [start.id] : [];
  while (stack.length) {
    const id = stack.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);
    for (const edge of doc.edges) if (edge.from === id) stack.push(edge.to);
  }
  return new Set(doc.nodes.filter((node) => !seen.has(node.id)).map((node) => node.id));
}

function reaches(edges: GraphEdge[], from: string, target: string): boolean {
  const visited = new Set<string>();
  const stack = [from];
  while (stack.length) {
    const id = stack.pop()!;
    if (id === target) return true;
    if (visited.has(id)) continue;
    visited.add(id);
    for (const edge of edges) if (edge.from === id) stack.push(edge.to);
  }
  return false;
}

/**
 * Add a wire. An output keeps the wires it has — several wires run their
 * nodes in parallel. Refused (the same document back) for a wire that exists,
 * a loop, or a port the node does not have.
 */
export function connect(doc: Experiment, from: string, port: Port, to: string): Experiment {
  const source = doc.nodes.find((node) => node.id === from);
  const target = doc.nodes.find((node) => node.id === to);
  if (!source || !target || from === to || source.type === "end" || target.type === "start") return doc;
  if (!validPortsFor(source.type).includes(port) || isWired(doc, from, port, to)) return doc;
  if (reaches(doc.edges, to, from)) return doc;
  return { ...doc, edges: [...doc.edges, { from, to, port }] };
}

/** Remove one wire of an output, or all of them without `to`. */
export function disconnect(doc: Experiment, from: string, port: Port, to?: string): Experiment {
  return { ...doc, edges: doc.edges.filter((edge) => !(edge.from === from && portOf(edge) === port && (to === undefined || edge.to === to))) };
}

export function insertOnEdge(doc: Experiment, edge: GraphEdge, node: ExperimentNode): Experiment {
  const source = doc.nodes.find((item) => item.id === edge.from);
  const target = doc.nodes.find((item) => item.id === edge.to);
  if (!source || !target) return doc;
  const x = Math.max(node.x, source.x + STEP_X);
  const shift = Math.max(0, x + STEP_X - target.x);
  const downstream = new Set<string>();
  const stack = [edge.to];
  while (stack.length) {
    const id = stack.pop()!;
    if (downstream.has(id)) continue;
    downstream.add(id);
    for (const candidate of doc.edges) if (candidate.from === id) stack.push(candidate.to);
  }
  // Only this wire is cut; the output's other wires keep their nodes.
  const before = doc.edges.filter((candidate) => !(candidate.from === edge.from && portOf(candidate) === portOf(edge) && candidate.to === edge.to));
  const defaultOutPort = validPortsFor(node.type)[0] ?? "next";
  return {
    ...doc,
    nodes: [...doc.nodes.map((item) => downstream.has(item.id) ? { ...item, x: item.x + shift } : item), { ...node, x }],
    edges: [...before, { from: edge.from, to: node.id, port: portOf(edge) },
      { from: node.id, to: edge.to, port: defaultOutPort }],
  };
}

export function removeNode(doc: Experiment, id: string): Experiment {
  const node = doc.nodes.find((item) => item.id === id);
  if (!node || node.type === "start" || node.type === "end") return doc;
  const incoming = doc.edges.filter((edge) => edge.to === id);
  const outgoing = doc.edges.filter((edge) => edge.from === id);
  const bridge = incoming.length === 1 && outgoing.length === 1 && !isWired(doc, incoming[0].from, portOf(incoming[0]), outgoing[0].to)
    ? [{ from: incoming[0].from, to: outgoing[0].to, port: portOf(incoming[0]) }]
    : [];
  return { ...doc, nodes: doc.nodes.filter((item) => item.id !== id),
    edges: [...doc.edges.filter((edge) => edge.from !== id && edge.to !== id), ...bridge] };
}

/** Copies parameters, but not wires. Start and End remain unique. */
export function duplicateNode(doc: Experiment, id: string): ExperimentNode | null {
  const source = doc.nodes.find((node) => node.id === id);
  if (!source || source.type === "start" || source.type === "end") return null;
  const copy = structuredClone(source);
  copy.id = `${source.type}-${crypto.randomUUID()}`;
  copy.x += 32;
  copy.y += NODE_HEIGHT + 32;
  while (doc.nodes.some((node) => Math.abs(node.x - copy.x) < NODE_WIDTH + 12 && Math.abs(node.y - copy.y) < NODE_HEIGHT + 12)) {
    copy.y += NODE_HEIGHT + 32;
  }
  return copy;
}

/** Stable left-to-right DAG layout; disconnected drafts are included. */
export function arrangeNodes(doc: Experiment): Experiment {
  const indegree = new Map(doc.nodes.map((node) => [node.id, 0]));
  const outgoing = new Map<string, string[]>();
  const rank = new Map(doc.nodes.map((node) => [node.id, 0]));
  for (const edge of doc.edges) {
    if (!indegree.has(edge.from) || !indegree.has(edge.to)) continue;
    indegree.set(edge.to, indegree.get(edge.to)! + 1);
    outgoing.set(edge.from, [...(outgoing.get(edge.from) ?? []), edge.to]);
  }
  const queue = doc.nodes.filter((node) => indegree.get(node.id) === 0).map((node) => node.id);
  for (let index = 0; index < queue.length; index++) {
    const id = queue[index];
    for (const target of outgoing.get(id) ?? []) {
      rank.set(target, Math.max(rank.get(target)!, rank.get(id)! + 1));
      indegree.set(target, indegree.get(target)! - 1);
      if (indegree.get(target) === 0) queue.push(target);
    }
  }
  // Imported cyclic drafts must never lose nodes or get stuck in layout.
  if (queue.length !== doc.nodes.length) return doc;
  const rows = new Map<number, number>();
  const positions = new Map(queue.map((id) => {
    const column = rank.get(id)!;
    const row = rows.get(column) ?? 0;
    rows.set(column, row + 1);
    return [id, { x: 40 + column * (NODE_WIDTH + 72), y: 48 + row * (NODE_HEIGHT + 48) }];
  }));
  const nodes = doc.nodes.map((node) => ({ ...node, ...positions.get(node.id)! }));
  if (nodes.every((node, index) => node.x === doc.nodes[index].x && node.y === doc.nodes[index].y)) return doc;
  return { ...doc, nodes };
}

export function graphBounds(nodes: ExperimentNode[]) {
  if (!nodes.length) return { x: 0, y: 0, width: NODE_WIDTH, height: NODE_HEIGHT };
  const x = Math.min(...nodes.map((node) => node.x));
  const y = Math.min(...nodes.map((node) => node.y));
  return { x, y, width: Math.max(...nodes.map((node) => node.x + NODE_WIDTH)) - x,
    height: Math.max(...nodes.map((node) => node.y + NODE_HEIGHT)) - y };
}
