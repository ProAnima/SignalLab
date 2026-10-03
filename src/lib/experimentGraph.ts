import type { ArgRule, Experiment, ExperimentNode, ExperimentPort, HttpRequest, OscArg } from "./api";
import { blankEmulator } from "./emulators.ts";
import { presetProfile } from "./impairments.ts";

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
    case "loop": return { ...base, type, max: 5, until: null };
    // Listens on loopback by default, like every shipped target.
    case "wait_osc": return { ...base, type, bind: "127.0.0.1:9001", address: "/pong", args: [], timeout_ms: 2000, variable: "reply" };
    case "wait_udp": return { ...base, type, bind: "127.0.0.1:9001", mode: "contains", pattern: "pong", timeout_ms: 2000, variable: "reply" };
    case "wait_mqtt": return { ...base, type, host: "127.0.0.1", port: 1883, topic: "lab/#", mode: "any", pattern: "", timeout_ms: 2000, variable: "reply" };
    // A new emulator and a new HTTP wait share an address, so the wait sees what the emulator answers.
    case "emulator": return { ...base, type, emulator: blankEmulator("http", "API", "127.0.0.1:18080") };
    case "wait_http": return { ...base, type, bind: "127.0.0.1:18080", method: "ANY", path: "/*", when: [], timeout_ms: 5000, variable: "request" };
    // A relay in front of the default OSC/UDP target, clean to start with.
    case "impairment": return { ...base, type, listen: "127.0.0.1:9010", target: "127.0.0.1:9000", profile: presetProfile("lan") };
    // Which relay or emulator it changes is chosen in its fields (the first of the document's, when added there).
    case "impairment_change": return { ...base, type, relay: "", profile: presetProfile("offline") };
    case "emulator_state": return { ...base, type, emulator: "", down: true, fault: "unavailable" };
    // Loopback, like every shipped target; the others name it by its id (chosen when added).
    case "ws_connect": return { ...base, type, url: "ws://127.0.0.1:9001/", headers: [], protocols: [], timeout_ms: 5000 };
    case "ws_send": return { ...base, type, connection: "", text: "hello", binary: false };
    case "wait_ws": return { ...base, type, connection: "", mode: "any", pattern: "", timeout_ms: 2000, variable: "reply" };
    case "ws_close": return { ...base, type, connection: "", code: 1000, reason: "" };
  }
}

/**
 * A new node as the editor adds it to `doc`: a Change impairment, an Emulator
 * down/up and a WebSocket send, wait or close name the document's first
 * Impairment, Emulator or WebSocket connect, so they work as they are when
 * there is one.
 */
export function createNodeIn(doc: Experiment, type: NodeType, x: number, y: number): ExperimentNode {
  const node = createNode(type, x, y);
  if (node.type === "impairment_change") node.relay = doc.nodes.find((candidate) => candidate.type === "impairment")?.id ?? "";
  if (node.type === "emulator_state") node.emulator = doc.nodes.find((candidate) => candidate.type === "emulator")?.id ?? "";
  if (node.type === "ws_send" || node.type === "wait_ws" || node.type === "ws_close") {
    node.connection = doc.nodes.find((candidate) => candidate.type === "ws_connect")?.id ?? "";
  }
  return node;
}

/** The port of `host:port`, or none. */
const addressPort = (address: string) => { const port = Number(address.slice(address.lastIndexOf(":") + 1)); return Number.isInteger(port) && port > 0 ? [port] : []; };

/**
 * A loopback port, from 9010 up, that no socket of the document's run takes —
 * a relay, a wait, a reply, an emulator — and that is not `avoid` (the node's
 * own target: a relay listening there would forward to itself).
 */
export function freeRelayPort(doc: Experiment, avoid: string[] = []): number {
  const taken = new Set([...avoid.flatMap(addressPort), ...doc.nodes.flatMap((node) => {
    switch (node.type) {
      case "impairment": return addressPort(node.listen);
      case "wait_osc": case "wait_udp": case "wait_http": return addressPort(node.bind);
      case "osc": case "udp": return node.reply ? addressPort(node.reply.bind) : [];
      case "emulator": return addressPort(node.emulator.bind);
      default: return [];
    }
  })]);
  let port = 9010;
  while (taken.has(port)) port += 1;
  return port;
}

/**
 * "Route through impairment": an Impairment in front of an OSC or UDP node —
 * placed before it, forwarding to its target — and the node pointed at the
 * relay, so the next run degrades what it sends. The node's wires stay; the
 * Impairment is spliced in after whatever led to it.
 */
export function routeThroughImpairment(doc: Experiment, nodeId: string): { doc: Experiment; relay: ExperimentNode } | null {
  const node = doc.nodes.find((candidate) => candidate.id === nodeId);
  if (!node || (node.type !== "osc" && node.type !== "udp")) return null;
  const listen = `127.0.0.1:${freeRelayPort(doc, [node.target])}`;
  const relay: ExperimentNode = { ...createNode("impairment", node.x, Math.max(12, node.y - STEP_Y)), type: "impairment", listen, target: node.target, profile: presetProfile("lan") };
  const incoming = doc.edges.filter((edge) => edge.to === nodeId);
  const edges = [
    ...doc.edges.filter((edge) => edge.to !== nodeId),
    ...incoming.map((edge) => ({ ...edge, to: relay.id })),
    { from: relay.id, to: nodeId, port: "next" as Port },
  ];
  const nodes = [...doc.nodes.map((candidate) => candidate.id === nodeId ? { ...candidate, target: listen } as ExperimentNode : candidate), relay];
  return { doc: { ...doc, nodes, edges }, relay };
}

/** Argument rules a Wait for OSC may hold, and the highest argument index (as the engine allows). */
const MAX_RULES = 16;
const MAX_ARG_INDEX = 63;

/**
 * A Wait for OSC that recognises a message seen in the monitor again: on the
 * monitor's port, by its address, and by its text, whole-number and true/false
 * arguments. Floats are measurements that change, so they are left out.
 */
export function waitForOscMessage(bind: string, address: string, args: OscArg[]): ExperimentNode {
  const rules = args.flatMap((arg, index): ArgRule[] => {
    if (index > MAX_ARG_INDEX) return [];
    if (arg.type === "str") return [{ index, op: "eq", value: arg.value }];
    if (arg.type === "int" || arg.type === "long" || arg.type === "bool") return [{ index, op: "eq", value: String(arg.value) }];
    return [];
  }).slice(0, MAX_RULES);
  return { ...createNode("wait_osc", 0, 0), type: "wait_osc", bind, address, args: rules, timeout_ms: 2000, variable: "reply" };
}

/** A Wait for MQTT on a topic seen in the broker tree, any payload. */
export function waitForMqttMessage(host: string, port: number, topic: string): ExperimentNode {
  return { ...createNode("wait_mqtt", 0, 0), type: "wait_mqtt", host, port, topic, mode: "any", pattern: "", timeout_ms: 2000, variable: "reply" };
}

/** Every output a node can have, in the order they are drawn. */
export function validPortsFor(type: NodeType): Port[] {
  switch (type) {
    case "branch_status": case "branch_value": return ["yes", "no"];
    case "fork": return ["branch1", "branch2"];
    case "wait_osc": case "wait_udp": case "wait_mqtt": case "wait_http": case "wait_ws": return ["matched", "timeout"];
    case "loop": return ["body", "done", "limit"];
    case "end": return [];
    default: return ["next"];
  }
}

/**
 * Outputs a runnable graph must connect. A wait's Timeout and a Loop's Limit
 * are optional: unwired, a timeout or running out of iterations fails the step.
 */
export function requiredPortsFor(type: NodeType): Port[] {
  return validPortsFor(type).filter((port) => port !== "timeout" && port !== "limit");
}

/**
 * A Loop's body: the nodes reached from its Body output that lead back to it
 * (as the engine's `LoopShape` reads it).
 */
export function loopBody(doc: Experiment, loopId: string): Set<string> {
  const forward = new Set<string>();
  const stack = doc.edges.filter((edge) => edge.from === loopId && portOf(edge) === "body").map((edge) => edge.to);
  while (stack.length) {
    const id = stack.pop()!;
    if (id === loopId || forward.has(id)) continue;
    forward.add(id);
    for (const edge of doc.edges) if (edge.from === id) stack.push(edge.to);
  }
  const back = new Set<string>();
  const behind = doc.edges.filter((edge) => edge.to === loopId).map((edge) => edge.from);
  while (behind.length) {
    const id = behind.pop()!;
    if (id === loopId || back.has(id)) continue;
    back.add(id);
    for (const edge of doc.edges) if (edge.to === id) behind.push(edge.from);
  }
  return new Set([...forward].filter((id) => back.has(id)));
}

/** A wire from a Loop's body back to the Loop: the one way a flow may go back. */
export function isBackEdge(doc: Experiment, edge: GraphEdge): boolean {
  const target = doc.nodes.find((node) => node.id === edge.to);
  return target?.type === "loop" && loopBody(doc, edge.to).has(edge.from);
}

/** The wires that only go forward: every one but the ways back of loops. */
function forwardEdges(doc: Experiment): GraphEdge[] {
  return doc.edges.filter((edge) => !isBackEdge(doc, edge));
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

/**
 * A free spot beside the anchor: one step to the right, in the source's row
 * (lower for its second output), moved down past any node already there — a
 * wire may lead to a node in another row, which is no place for the new one.
 */
export function placeAfter(doc: Experiment, anchor: Anchor): { x: number; y: number } {
  const source = doc.nodes.find((node) => node.id === anchor.from);
  if (!source) return { x: 40, y: 40 };
  const x = source.x + STEP_X;
  let y = source.y + Math.max(0, validPortsFor(source.type).indexOf(anchor.port)) * STEP_Y;
  // Splicing into a wire shifts what follows it, so its own target does not count.
  const target = wireOf(doc, anchor)?.to;
  while (overlaps(doc.nodes.filter((node) => node.id !== target), x, y)) y += STEP_Y;
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
  return closeLoop({ ...doc, nodes: [...doc.nodes, node], edges: [...doc.edges, { from: anchor.from, to: node.id, port: anchor.port }] }, anchor, node);
}

/**
 * The first step put on an empty Body also leads back to its Loop, so the body
 * is whole at once — when the step has one output to do it with.
 */
function closeLoop(doc: Experiment, anchor: Anchor, node: ExperimentNode): Experiment {
  const source = doc.nodes.find((item) => item.id === anchor.from);
  const outputs = validPortsFor(node.type);
  const firstOnBody = source?.type === "loop" && anchor.port === "body" && doc.edges.filter((edge) => edge.from === source.id && portOf(edge) === "body").length === 1;
  if (!firstOnBody || outputs.length !== 1) return doc;
  return { ...doc, edges: [...doc.edges, { from: node.id, to: anchor.from, port: outputs[0] }] };
}

/**
 * Attach `node` on a new wire of the output, beside the ones it has: a wire
 * dragged out of a connected output starts parallel work instead of cutting
 * into the existing flow.
 */
export function addBranch(doc: Experiment, anchor: Anchor | null, node: ExperimentNode): Experiment {
  const source = anchor && doc.nodes.find((item) => item.id === anchor.from);
  if (!anchor || !source || !validPortsFor(source.type).includes(anchor.port)) return { ...doc, nodes: [...doc.nodes, node] };
  return closeLoop({ ...doc, nodes: [...doc.nodes, node], edges: [...doc.edges, { from: anchor.from, to: node.id, port: anchor.port }] }, anchor, node);
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
 * a cycle, or a port the node does not have. The one wire that may go back is
 * from a Loop's body to that Loop: it closes an iteration.
 */
export function connect(doc: Experiment, from: string, port: Port, to: string): Experiment {
  const source = doc.nodes.find((node) => node.id === from);
  const target = doc.nodes.find((node) => node.id === to);
  if (!source || !target || from === to || source.type === "end" || target.type === "start") return doc;
  if (!validPortsFor(source.type).includes(port) || isWired(doc, from, port, to)) return doc;
  const added = { ...doc, edges: [...doc.edges, { from, to, port }] };
  const closesLoop = target.type === "loop" && reaches(forwardEdges(doc).filter((edge) => !(edge.from === to && portOf(edge) !== "body")), to, from);
  if (closesLoop) return added;
  if (reaches(forwardEdges(doc), to, from)) return doc;
  return added;
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
  // What follows moves right to make room — along forward wires only: a Loop's
  // way back would make everything "follow".
  const downstream = new Set<string>();
  const forward = isBackEdge(doc, edge) ? [] : forwardEdges(doc);
  const stack = isBackEdge(doc, edge) ? [] : [edge.to];
  while (stack.length) {
    const id = stack.pop()!;
    if (downstream.has(id)) continue;
    downstream.add(id);
    for (const candidate of forward) if (candidate.from === id) stack.push(candidate.to);
  }
  // Only this wire is cut; the output's other wires keep their nodes.
  const before = doc.edges.filter((candidate) => !(candidate.from === edge.from && portOf(candidate) === portOf(edge) && candidate.to === edge.to));
  // A Loop continues the flow through Done; its Body is the new loop's inside.
  const defaultOutPort = node.type === "loop" ? "done" : validPortsFor(node.type)[0] ?? "next";
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
  // No bridge onto itself: removing a Loop's only body step leaves Body unwired.
  const bridge = incoming.length === 1 && outgoing.length === 1 && incoming[0].from !== outgoing[0].to && !isWired(doc, incoming[0].from, portOf(incoming[0]), outgoing[0].to)
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
  // A Loop's way back is not part of the order; its body comes first in each
  // column, so the body stays in the Loop's row and Done continues below it.
  const bodyFirst = forwardEdges(doc).sort((a, b) => Number(portOf(b) === "body") - Number(portOf(a) === "body"));
  for (const edge of bodyFirst) {
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
  // A Loop's way back arcs over its body: the top row leaves it room.
  const top = doc.nodes.some((node) => node.type === "loop") ? 120 : 48;
  const positions = new Map(queue.map((id) => {
    const column = rank.get(id)!;
    const row = rows.get(column) ?? 0;
    rows.set(column, row + 1);
    return [id, { x: 40 + column * (NODE_WIDTH + 72), y: top + row * (NODE_HEIGHT + 48) }];
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
