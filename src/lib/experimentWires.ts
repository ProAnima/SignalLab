/**
 * Where wires run on the experiment canvas and how one is told apart from
 * another. Pure geometry in graph coordinates; the canvas scales it by zoom.
 */
import type { ExperimentNode } from "./api";
import { NODE_HEIGHT as NODE_H, NODE_WIDTH as NODE_W, portOf, type Port } from "./experimentGraph.ts";
import { isWait } from "./experimentText.ts";

/** One wire: an output of `from` and the node it leads to. */
export type Wire = { from: string; port: Port; to: string };
type EdgeLike = { from: string; to: string; port?: Port };
export const sameWire = (a: Wire, edge: EdgeLike) => a.from === edge.from && a.port === portOf(edge) && a.to === edge.to;
export const wireKey = (edge: EdgeLike) => `${edge.from}-${portOf(edge)}-${edge.to}`;

/**
 * Where a Loop's way back runs: over its two ends, clear of them — Done and
 * Limit leave the Loop lower down — or under them when the canvas has no
 * room above.
 */
export const backY = (a: ExperimentNode, b: ExperimentNode) => {
  const top = Math.min(a.y, b.y);
  return top >= 80 ? top - 70 : Math.max(a.y, b.y) + NODE_H + 70;
};

/** How far down a node an output sits. */
export const portY = (node: ExperimentNode, port: Port): number => {
  if (node.type === "branch_status" || node.type === "branch_value") return port === "yes" ? 28 : 56;
  if (node.type === "fork") return port === "branch1" ? 28 : 56;
  if (isWait(node)) return port === "matched" ? 28 : 56;
  if (node.type === "loop") return port === "body" ? 16 : port === "done" ? 40 : 64;
  return NODE_H / 2;
};

/** A wire from one point to another, leaving and arriving level. */
export const curve = (x1: number, y1: number, x2: number, y2: number) => `M ${x1} ${y1} C ${x1 + 75} ${y1}, ${x2 - 75} ${y2}, ${x2} ${y2}`;

/** The drawn path of a wire from `a`'s output `port` to `b`'s input. A Loop's way back runs under the body, from its last step to the Loop's input. */
export function wirePath(a: ExperimentNode, b: ExperimentNode, port: Port, back: boolean): string {
  const x1 = a.x + NODE_W, y1 = a.y + portY(a, port);
  const x2 = b.x, y2 = b.y + NODE_H / 2;
  return back ? `M ${x1} ${y1} C ${x1 + 80} ${backY(a, b)}, ${x2 - 80} ${backY(a, b)}, ${x2} ${y2}` : curve(x1, y1, x2, y2);
}

/** Where a wire's ＋ and × sit: halfway, or on a way back, the middle of its curve. */
export function wireMiddle(a: ExperimentNode, b: ExperimentNode, port: Port, back: boolean): { x: number; y: number } {
  const y1 = a.y + portY(a, port); const y2 = b.y + NODE_H / 2; const x = (a.x + NODE_W + b.x) / 2;
  const y = back ? (y1 + 6 * backY(a, b) + y2) / 8 : (y1 + y2) / 2;
  return { x, y };
}

/** The node nearest to a point within `reach` of its edge (graph units), for a wire released just outside one. */
export function nearestNode(nodes: ExperimentNode[], point: { x: number; y: number }, reach: number): string | undefined {
  let best: { id: string; distance: number } | undefined;
  for (const node of nodes) {
    const dx = Math.max(node.x - point.x, 0, point.x - (node.x + NODE_W));
    const dy = Math.max(node.y - point.y, 0, point.y - (node.y + NODE_H));
    const distance = Math.hypot(dx, dy);
    if (distance <= reach && (!best || distance < best.distance)) best = { id: node.id, distance };
  }
  return best?.id;
}
