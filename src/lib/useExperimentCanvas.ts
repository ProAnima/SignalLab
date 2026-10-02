import { useEffect, useRef, useState, type MouseEvent, type PointerEvent, type RefObject } from "react";
import type { Experiment, ExperimentNode } from "./api";
import { anchorAfter, connect, disconnect, isWired, placeAfter, portOf, NODE_HEIGHT as NODE_H, NODE_WIDTH as NODE_W, type Anchor, type Port } from "./experimentGraph";
import { nearestNode, sameWire, type Wire } from "./experimentWires";
import type { Failure } from "./errors";

/** Where the add menu opens (screen) and where its node goes (graph). `branch`: the node goes on a new wire of the anchor's output (parallel work), not into its flow. */
export type Menu = { screenX: number; screenY: number; x: number; y: number; anchor?: Anchor; branch?: boolean };
type Point = { x: number; y: number };

/**
 * What the pointer does on the canvas: drag nodes, pan, pick wires, draw new
 * ones from an output, and open the add menu where the next node should go.
 * Zoom and scrolling are `useExperimentViewport`'s; selection and the wire
 * being drawn belong to the editor, which other panels read too.
 */
export function useExperimentCanvas({ doc, busy, zoom, scrollRef, selected, setSelected, linkStart, setLinkStart, setLinkPoint, cancelLink, setMenu, edit, commitEdit, patchNode, setProblem }: {
  doc: Experiment | null;
  busy: boolean;
  zoom: number;
  scrollRef: RefObject<HTMLDivElement | null>;
  selected: string | null;
  setSelected: (id: string | null) => void;
  /** The output a wire is being drawn from, by click or by drag. */
  linkStart: Anchor | null;
  setLinkStart: (anchor: Anchor | null) => void;
  setLinkPoint: (point: Point | null) => void;
  cancelLink: () => void;
  setMenu: (menu: Menu | null) => void;
  edit: (update: (current: Experiment) => Experiment) => void;
  commitEdit: () => void;
  patchNode: (id: string, change: Partial<ExperimentNode>, group?: string | null) => void;
  setProblem: (failure: Failure | null) => void;
}) {
  // A wire picked on the canvas; a node and a wire are never both selected.
  const [wire, setWire] = useState<Wire | null>(null);
  // The wire under the pointer (its line or its buttons): it shows its × too.
  const [hoverWire, setHoverWire] = useState<string | null>(null);
  const hoverTimer = useRef<number | undefined>(undefined);
  const hover = (key: string | null) => {
    window.clearTimeout(hoverTimer.current);
    // Leaving the line for its buttons, which sit on it, must not hide them.
    if (key) setHoverWire(key);
    else hoverTimer.current = window.setTimeout(() => setHoverWire(null), 180);
  };
  const suppressPortClick = useRef(false);
  const drag = useRef<{ id: string; x: number; y: number; clientX: number; clientY: number; group: string } | null>(null);
  const pan = useRef<{ clientX: number; clientY: number; left: number; top: number; moved: boolean } | null>(null);
  const portDrag = useRef<{ anchor: Anchor; clientX: number; clientY: number; moved: boolean } | null>(null);

  // Selecting a node lets go of a wire.
  useEffect(() => { if (selected) setWire(null); }, [selected]);

  const graphPoint = (clientX: number, clientY: number) => {
    const scroll = scrollRef.current!;
    const rect = scroll.getBoundingClientRect();
    return { x: (clientX - rect.left + scroll.scrollLeft) / zoom, y: (clientY - rect.top + scroll.scrollTop) / zoom };
  };

  const screenPoint = (x: number, y: number) => {
    const scroll = scrollRef.current!;
    const rect = scroll.getBoundingClientRect();
    return { left: rect.left + x * zoom - scroll.scrollLeft, top: rect.top + y * zoom - scroll.scrollTop };
  };

  const openMenu = (clientX: number, clientY: number, x: number, y: number, anchor?: Anchor, branch = false) => {
    if (busy) return;
    commitEdit();
    setMenu({ screenX: Math.min(clientX, window.innerWidth - 328), screenY: Math.min(clientY, window.innerHeight - 470), x, y, anchor, branch });
    cancelLink();
  };

  /** Toolbar and A: a new node continues the flow from the selected one, or lands mid-view. */
  const openAddMenu = () => {
    const scroll = scrollRef.current;
    if (!doc || busy || !scroll) return;
    const rect = scroll.getBoundingClientRect();
    const anchor = selected ? anchorAfter(doc, selected) : null;
    const spot = anchor ? placeAfter(doc, anchor)
      : { x: (scroll.scrollLeft + scroll.clientWidth / 2) / zoom - NODE_W / 2, y: (scroll.scrollTop + scroll.clientHeight / 2) / zoom - NODE_H / 2 };
    const at = screenPoint(spot.x, spot.y);
    // The menu opens where the node will appear when that is on screen; otherwise where the eye already is.
    const visible = anchor && at.left >= rect.left && at.left <= rect.right - 40 && at.top >= rect.top && at.top <= rect.bottom - 40;
    if (visible) openMenu(at.left, at.top, spot.x, spot.y, anchor);
    else openMenu(rect.left + scroll.clientWidth / 2 - 156, rect.top + Math.min(60, scroll.clientHeight / 3), spot.x, spot.y, anchor ?? undefined);
  };

  const toggleLink = (from: string, port: Port) => {
    if (busy) return;
    if (linkStart?.from === from && linkStart.port === port) { cancelLink(); return; }
    setLinkStart({ from, port }); setLinkPoint(null); setMenu(null);
  };
  const endLink = (to: string, anchor = linkStart) => {
    if (!doc || !anchor || busy) return;
    // The same wire again changes nothing and is not a mistake worth a message.
    if (isWired(doc, anchor.from, anchor.port, to)) { cancelLink(); return; }
    const updated = connect(doc, anchor.from, anchor.port, to);
    if (updated === doc) setProblem("exp.invalidConnection");
    else { commitEdit(); edit(() => updated); }
    cancelLink();
  };

  /** The node a released wire is meant for when it lands just outside one: within 24 screen pixels. */
  const nodeNear = (clientX: number, clientY: number): string | undefined =>
    doc ? nearestNode(doc.nodes, graphPoint(clientX, clientY), 24 / zoom) : undefined;

  // Output ports: drag a wire to a node to connect it, or to empty canvas to
  // create the next node there — always one more wire, so an output can feed
  // several nodes that run in parallel. A click without movement starts
  // click-to-link, which is also what Enter/Space on the focused port does.
  const onPortDown = (event: PointerEvent<HTMLButtonElement>, anchor: Anchor) => {
    if (busy || event.button !== 0) return;
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    portDrag.current = { anchor, clientX: event.clientX, clientY: event.clientY, moved: false };
  };
  const onPortMove = (event: PointerEvent<HTMLButtonElement>) => {
    const d = portDrag.current;
    if (!d) return;
    if (!d.moved && Math.hypot(event.clientX - d.clientX, event.clientY - d.clientY) < 5) return;
    if (!d.moved) { d.moved = true; setLinkStart(d.anchor); setMenu(null); }
    setLinkPoint(graphPoint(event.clientX, event.clientY));
  };
  const onPortUp = (event: PointerEvent<HTMLButtonElement>) => {
    const d = portDrag.current;
    portDrag.current = null;
    if (!d?.moved) return;
    suppressPortClick.current = true;
    const hit = document.elementFromPoint(event.clientX, event.clientY);
    // On a node, or near enough to one: a wire need not land exactly on its edge.
    const target = (hit instanceof HTMLElement ? hit.closest<HTMLElement>("[data-node]")?.dataset.node : undefined) ?? nodeNear(event.clientX, event.clientY);
    const rect = scrollRef.current!.getBoundingClientRect();
    if (target && target !== d.anchor.from) endLink(target, d.anchor);
    else if (!target && event.clientX >= rect.left && event.clientX <= rect.right && event.clientY >= rect.top && event.clientY <= rect.bottom) {
      const point = graphPoint(event.clientX, event.clientY);
      openMenu(event.clientX, event.clientY, point.x, point.y - NODE_H / 2, d.anchor, true);
    } else cancelLink();
  };
  const onPortCancel = () => { portDrag.current = null; cancelLink(); };
  // The click that ends a drag from the port is not a click-to-link.
  const onPortClick = (from: string, port: Port) => {
    if (suppressPortClick.current) { suppressPortClick.current = false; return; }
    toggleLink(from, port);
  };

  // Gone with an undo, a removed node or a reconnection: then nothing is selected.
  const selectedWire = wire && doc?.edges.some((edge) => sameWire(wire, edge)) ? wire : null;
  const selectWire = (edge: { from: string; to: string; port?: Port }) => {
    if (busy || linkStart) return;
    setSelected(null); setMenu(null);
    setWire({ from: edge.from, port: portOf(edge), to: edge.to });
  };
  /** Remove a wire: the selected one, or the one whose × was clicked. */
  const removeWire = (target: Wire | null = selectedWire) => {
    if (!target || busy) return;
    const { from, port, to } = target;
    commitEdit();
    edit((current) => disconnect(current, from, port, to));
    setWire(null); setHoverWire(null);
  };

  const onNodeDown = (event: PointerEvent<HTMLButtonElement>, node: ExperimentNode) => {
    if (busy || event.button !== 0 || linkStart) return;
    drag.current = { id: node.id, x: node.x, y: node.y, clientX: event.clientX, clientY: event.clientY, group: `drag-${crypto.randomUUID()}` };
    event.currentTarget.setPointerCapture(event.pointerId);
    setSelected(node.id);
  };
  const onNodeMove = (event: PointerEvent<HTMLButtonElement>, node: ExperimentNode) => {
    const d = drag.current;
    if (!d || d.id !== node.id) return;
    const x = Math.max(12, Math.round(d.x + (event.clientX - d.clientX) / zoom));
    const y = Math.max(12, Math.round(d.y + (event.clientY - d.clientY) / zoom));
    if (x !== node.x || y !== node.y) patchNode(node.id, { x, y }, d.group);
  };
  const endDrag = () => { drag.current = null; commitEdit(); };

  /** The empty canvas: drag to pan, double-click to add a node there, click to drop the selection. */
  const surface = {
    onDoubleClick: (event: MouseEvent<HTMLDivElement>) => {
      if (event.target !== event.currentTarget) return;
      const point = graphPoint(event.clientX, event.clientY);
      openMenu(event.clientX, event.clientY, point.x - NODE_W / 2, point.y - NODE_H / 2, linkStart ?? undefined, !!linkStart);
    },
    onPointerDown: (event: PointerEvent<HTMLDivElement>) => {
      if (event.target !== event.currentTarget) return;
      const scroll = scrollRef.current!;
      pan.current = { clientX: event.clientX, clientY: event.clientY, left: scroll.scrollLeft, top: scroll.scrollTop, moved: false };
      event.currentTarget.setPointerCapture(event.pointerId);
      if (!linkStart) { setSelected(null); setWire(null); }
    },
    onPointerMove: (event: PointerEvent<HTMLDivElement>) => {
      if (linkStart && !portDrag.current) setLinkPoint(graphPoint(event.clientX, event.clientY));
      const p = pan.current; if (!p) return;
      if (!p.moved && Math.hypot(event.clientX - p.clientX, event.clientY - p.clientY) < 4) return;
      p.moved = true; const scroll = scrollRef.current!; scroll.scrollLeft = p.left - (event.clientX - p.clientX); scroll.scrollTop = p.top - (event.clientY - p.clientY);
    },
    onPointerUp: (event: PointerEvent<HTMLDivElement>) => {
      const p = pan.current; pan.current = null;
      // Click-to-link, then a click on empty canvas: the next node goes right there.
      if (p && !p.moved && linkStart) { const point = graphPoint(event.clientX, event.clientY); openMenu(event.clientX, event.clientY, point.x, point.y - NODE_H / 2, linkStart, true); }
    },
    onPointerCancel: () => { pan.current = null; },
  };

  return {
    selectedWire, setWire, selectWire, removeWire, hoverWire, hover, drag,
    openMenu, openAddMenu, endLink,
    onPortDown, onPortMove, onPortUp, onPortCancel, onPortClick,
    onNodeDown, onNodeMove, endDrag, surface,
  };
}

/** The canvas's handlers and wire state, as the canvas component takes them. */
export type CanvasControls = ReturnType<typeof useExperimentCanvas>;
