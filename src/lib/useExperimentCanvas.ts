import { useEffect, useRef, useState, type MouseEvent, type PointerEvent, type RefObject } from "react";
import type { Experiment, ExperimentNode } from "./api";
import { anchorAfter, connect, disconnect, isWired, placeAfter, portOf, NODE_HEIGHT as NODE_H, NODE_WIDTH as NODE_W, type Anchor, type Port } from "./experimentGraph";
import { dragNodes, nodesIn } from "./experimentClipboard";
import { nearestNode, sameWire, type Wire } from "./experimentWires";
import type { Failure } from "./errors";

/** Keep the pointer's moves coming here; a pointer a script made up (the tour) has nothing to capture. */
function capture(target: Element, pointerId: number) {
  try { target.setPointerCapture(pointerId); } catch { /* not an active pointer */ }
}

/** Where the add menu opens (screen) and where its node goes (graph). `branch`: the node goes on a new wire of the anchor's output (parallel work), not into its flow. */
export type Menu = { screenX: number; screenY: number; x: number; y: number; anchor?: Anchor; branch?: boolean };
type Point = { x: number; y: number };

/**
 * What the pointer does on the canvas: select and drag nodes (one, or several
 * together), pan, select with a frame, pick wires, draw new ones from an
 * output, and open the add menu where the next node should go. Zoom and
 * scrolling are `useExperimentViewport`'s; the selection and the wire being
 * drawn belong to the editor, which other panels read too.
 */
export function useExperimentCanvas({ doc, busy, zoom, scrollRef, selected, selection, setSelected, toggleSelected, selectMany, linkStart, setLinkStart, setLinkPoint, cancelLink, setMenu, edit, commitEdit, setProblem }: {
  doc: Experiment | null;
  busy: boolean;
  zoom: number;
  scrollRef: RefObject<HTMLDivElement | null>;
  /** The node the properties show: the last one selected. */
  selected: string | null;
  /** Every selected node, `selected` last. */
  selection: string[];
  setSelected: (id: string | null) => void;
  /** Shift or Ctrl and a click: in the selection, or out of it. */
  toggleSelected: (id: string) => void;
  /** A frame drawn with Shift: what it touches joins the selection. */
  selectMany: (ids: string[]) => void;
  /** The output a wire is being drawn from, by click or by drag. */
  linkStart: Anchor | null;
  setLinkStart: (anchor: Anchor | null) => void;
  setLinkPoint: (point: Point | null) => void;
  cancelLink: () => void;
  setMenu: (menu: Menu | null) => void;
  edit: (update: (current: Experiment) => Experiment, group?: string | null) => void;
  commitEdit: () => void;
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
  // A drag moves every node in `start` (one, or the whole selection); `collapse`:
  // a click on one of several selected nodes, without moving, selects it alone.
  const drag = useRef<{ node: string; start: Map<string, { x: number; y: number }>; clientX: number; clientY: number; group: string; moved: boolean; collapse: boolean } | null>(null);
  // The click that ends a drag, or follows a Shift/Ctrl press that already toggled,
  // does nothing more. It comes at once; a click later than this is a new one (a drag
  // let go outside the window sends none, and must not swallow the next).
  const suppressNodeClick = useRef(0);
  const swallowNextClick = () => { suppressNodeClick.current = performance.now() + 300; };
  const clickSwallowed = () => performance.now() < suppressNodeClick.current;
  const pan = useRef<{ clientX: number; clientY: number; left: number; top: number; moved: boolean } | null>(null);
  const portDrag = useRef<{ anchor: Anchor; clientX: number; clientY: number; moved: boolean } | null>(null);
  // A selection frame drawn with Shift on the empty canvas, in graph units.
  const bandStart = useRef<Point | null>(null);
  const [band, setBand] = useState<{ a: Point; b: Point } | null>(null);

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
    capture(event.currentTarget, event.pointerId);
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

  const modified = (event: { shiftKey: boolean; ctrlKey: boolean; metaKey: boolean }) => event.shiftKey || event.ctrlKey || event.metaKey;

  // Pointer down on a node: with Shift or Ctrl it joins or leaves the
  // selection; on one of several selected nodes it starts dragging them all;
  // otherwise it selects that node alone and starts dragging it.
  const onNodeDown = (event: PointerEvent<HTMLButtonElement>, node: ExperimentNode) => {
    if (busy || event.button !== 0 || linkStart || !doc) return;
    suppressNodeClick.current = 0;
    if (modified(event)) { swallowNextClick(); toggleSelected(node.id); return; }
    const together = selection.length > 1 && selection.includes(node.id) ? selection : [node.id];
    if (together.length === 1) setSelected(node.id);
    const start = new Map(doc.nodes.filter((item) => together.includes(item.id)).map((item) => [item.id, { x: item.x, y: item.y }]));
    drag.current = { node: node.id, start, clientX: event.clientX, clientY: event.clientY, group: `drag-${crypto.randomUUID()}`, moved: false, collapse: together.length > 1 };
    capture(event.currentTarget, event.pointerId);
  };
  const onNodeMove = (event: PointerEvent<HTMLButtonElement>, node: ExperimentNode) => {
    const d = drag.current;
    if (!d || d.node !== node.id) return;
    if (!d.moved && Math.hypot(event.clientX - d.clientX, event.clientY - d.clientY) < 3) return;
    d.moved = true;
    const dx = (event.clientX - d.clientX) / zoom;
    const dy = (event.clientY - d.clientY) / zoom;
    edit((current) => dragNodes(current, d.start, dx, dy), d.group);
  };
  const endDrag = () => {
    const d = drag.current;
    drag.current = null;
    commitEdit();
    if (!d) return;
    if (d.moved) swallowNextClick();
    else if (d.collapse) setSelected(d.node);
  };
  /** A click (or Enter, Space) on a node: ends a wire being drawn, or selects — with Shift or Ctrl, toggles. */
  const onNodeClick = (event: MouseEvent<HTMLButtonElement>, node: ExperimentNode) => {
    if (linkStart) { endLink(node.id); return; }
    if (clickSwallowed()) { suppressNodeClick.current = 0; return; }
    if (modified(event)) toggleSelected(node.id);
    else if (!selection.includes(node.id)) setSelected(node.id);
  };
  /** Tab onto a node selects it; a press that is selecting or dragging already has. */
  const onNodeFocus = (node: ExperimentNode) => {
    if (linkStart || drag.current || clickSwallowed() || selection.includes(node.id)) return;
    setSelected(node.id);
  };

  /** The empty canvas: drag to pan, Shift and drag to select with a frame, double-click to add a node there, click to drop the selection. */
  const surface = {
    onDoubleClick: (event: MouseEvent<HTMLDivElement>) => {
      if (event.target !== event.currentTarget) return;
      const point = graphPoint(event.clientX, event.clientY);
      openMenu(event.clientX, event.clientY, point.x - NODE_W / 2, point.y - NODE_H / 2, linkStart ?? undefined, !!linkStart);
    },
    onPointerDown: (event: PointerEvent<HTMLDivElement>) => {
      if (event.target !== event.currentTarget) return;
      capture(event.currentTarget, event.pointerId);
      if (event.shiftKey && !linkStart && !busy) {
        const point = graphPoint(event.clientX, event.clientY);
        bandStart.current = point;
        setBand({ a: point, b: point });
        return;
      }
      const scroll = scrollRef.current!;
      pan.current = { clientX: event.clientX, clientY: event.clientY, left: scroll.scrollLeft, top: scroll.scrollTop, moved: false };
      if (!linkStart) { setSelected(null); setWire(null); }
    },
    onPointerMove: (event: PointerEvent<HTMLDivElement>) => {
      if (bandStart.current) { setBand({ a: bandStart.current, b: graphPoint(event.clientX, event.clientY) }); return; }
      if (linkStart && !portDrag.current) setLinkPoint(graphPoint(event.clientX, event.clientY));
      const p = pan.current; if (!p) return;
      if (!p.moved && Math.hypot(event.clientX - p.clientX, event.clientY - p.clientY) < 4) return;
      p.moved = true; const scroll = scrollRef.current!; scroll.scrollLeft = p.left - (event.clientX - p.clientX); scroll.scrollTop = p.top - (event.clientY - p.clientY);
    },
    onPointerUp: (event: PointerEvent<HTMLDivElement>) => {
      const start = bandStart.current;
      if (start) {
        bandStart.current = null; setBand(null);
        if (doc) selectMany(nodesIn(doc.nodes, start, graphPoint(event.clientX, event.clientY)));
        return;
      }
      const p = pan.current; pan.current = null;
      // Click-to-link, then a click on empty canvas: the next node goes right there.
      if (p && !p.moved && linkStart) { const point = graphPoint(event.clientX, event.clientY); openMenu(event.clientX, event.clientY, point.x, point.y - NODE_H / 2, linkStart, true); }
    },
    onPointerCancel: () => { pan.current = null; bandStart.current = null; setBand(null); },
  };

  return {
    selectedWire, setWire, selectWire, removeWire, hoverWire, hover, drag, band,
    openMenu, openAddMenu, endLink,
    onPortDown, onPortMove, onPortUp, onPortCancel, onPortClick,
    onNodeDown, onNodeMove, endDrag, onNodeClick, onNodeFocus, surface,
  };
}

/** The canvas's handlers and wire state, as the canvas component takes them. */
export type CanvasControls = ReturnType<typeof useExperimentCanvas>;
