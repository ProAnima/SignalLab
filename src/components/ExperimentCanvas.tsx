import { useMemo, type CSSProperties, type RefObject } from "react";
import type { Experiment, ExperimentStep } from "../lib/api";
import { isBackEdge, missingOutputs, portOf, unreachableNodes, NODE_HEIGHT as NODE_H, NODE_WIDTH as NODE_W, STEP_X, type Anchor } from "../lib/experimentGraph";
import { nodeLabel, wireName } from "../lib/experimentText";
import { curve, portY, sameWire, wireKey, wireMiddle, wirePath } from "../lib/experimentWires";
import type { CanvasControls } from "../lib/useExperimentCanvas";
import { useT } from "../lib/i18n";
import { ExperimentCanvasNode } from "./ExperimentCanvasNode";

/**
 * The graph, scaled by zoom inside its scroll area: the wires (each with its
 * ＋ and ×), the wire being drawn, and the nodes. What the pointer does is
 * `useExperimentCanvas`'s; this only draws.
 */
export function ExperimentCanvas({ doc, scrollRef, zoom, width, height, busy, events, selection, invalidNode, linkStart, linkPoint, controls, onEdit }: {
  doc: Experiment;
  /** The scroll area, which `useExperimentViewport` measures and moves. */
  scrollRef: RefObject<HTMLDivElement | null>;
  zoom: number;
  /** The canvas in graph units. */
  width: number;
  height: number;
  busy: boolean;
  /** The steps of the current or last run, which colour nodes and wires. */
  events: ExperimentStep[];
  /** Every selected node. */
  selection: string[];
  invalidNode: string | null;
  linkStart: Anchor | null;
  linkPoint: { x: number; y: number } | null;
  controls: CanvasControls;
  onEdit: (id: string) => void;
}) {
  const t = useT();
  const { selectedWire, hoverWire, hover, selectWire, removeWire, openMenu, surface, band } = controls;
  const chosen = useMemo(() => new Set(selection), [selection]);
  const nodeStates = useMemo(() => new Map(events.map((event) => [event.node_id, event.state])), [events]);
  const missing = useMemo(() => missingOutputs(doc), [doc]);
  const detached = useMemo(() => unreachableNodes(doc), [doc]);
  const linkSource = linkStart ? doc.nodes.find((node) => node.id === linkStart.from) : undefined;

  return <div className={`experiment-canvas-scroll ${linkStart ? "linking" : ""}`} ref={scrollRef}>
    <div className="experiment-canvas-space" style={{ width: width * zoom, height: height * zoom }}>
      <div className="experiment-canvas" style={{ width, height, transform: `scale(${zoom})`, "--zoom": zoom } as CSSProperties} {...surface}>
        <svg className="experiment-wires" width={width} height={height}>
          {doc.edges.map((edge) => { const a = doc.nodes.find((node) => node.id === edge.from); const b = doc.nodes.find((node) => node.id === edge.to); if (!a || !b) return null;
            const p = portOf(edge);
            const back = isBackEdge(doc, edge);
            const busyState = (id: string) => ["running", "retry", "repeating"].includes(nodeStates.get(id) ?? "");
            const isRunning = busyState(a.id) || busyState(b.id);
            const isPassed = nodeStates.get(a.id) === "passed" && (nodeStates.get(b.id) === "passed" || nodeStates.get(b.id) === "running");
            const d = wirePath(a, b, p, back);
            const picked = !!selectedWire && sameWire(selectedWire, edge);
            // A wide, invisible stroke takes the pointer, so a wire is easy to pick; the drawn one sits on it.
            return <g key={`${edge.from}-${p}-${edge.to}`}>
              <path className="experiment-wire-hit" d={d} role="button" tabIndex={busy ? -1 : 0} aria-pressed={picked}
                aria-label={`${t("exp.wire")}: ${wireName(doc.nodes, edge, t)}`} onClick={() => selectWire(edge)} onFocus={() => selectWire(edge)}
                onPointerEnter={() => hover(wireKey(edge))} onPointerLeave={() => hover(null)} />
              <path aria-hidden="true" className={`wire ${p} ${back ? "back" : ""} ${isRunning ? "active-flow" : ""} ${isPassed ? "passed-flow" : ""} ${picked ? "selected" : hoverWire === wireKey(edge) ? "hovered" : ""}`} d={d} />
            </g>;
          })}
          {linkSource && linkPoint && <path aria-hidden="true" className="wire preview" d={curve(linkSource.x + NODE_W, linkSource.y + portY(linkSource, linkStart!.port), linkPoint.x, linkPoint.y)} />}
        </svg>
        {doc.edges.map((edge) => { const a = doc.nodes.find((node) => node.id === edge.from); const b = doc.nodes.find((node) => node.id === edge.to); if (!a || !b) return null;
          const p = portOf(edge);
          const { x, y } = wireMiddle(a, b, p, isBackEdge(doc, edge));
          const picked = !!selectedWire && sameWire(selectedWire, edge);
          // ＋ inserts into the wire; × — above it, clear of the ports — removes it, shown while the wire is hovered or selected.
          return <span key={wireKey(edge)} className="experiment-edge-tools" onPointerEnter={() => hover(wireKey(edge))} onPointerLeave={() => hover(null)}>
            <button className="experiment-edge-add" style={{ left: x - 11, top: y - 11 }} data-tip={t("exp.insertNode")} aria-label={`${t("exp.insertNode")}: ${nodeLabel(a.type, t)} → ${nodeLabel(b.type, t)}`} disabled={busy} onClick={(event) => openMenu(event.clientX, event.clientY, Math.max(x - NODE_W / 2, a.x + STEP_X), y - NODE_H / 2, { from: edge.from, port: p, to: edge.to })}>＋</button>
            {(picked || hoverWire === wireKey(edge)) && !busy && <button className="experiment-edge-remove" style={{ left: x - 11, top: y - 37 }} data-tip={`${t("exp.removeWire")} · Delete`} aria-label={`${t("exp.removeWire")}: ${wireName(doc.nodes, edge, t)}`}
              onClick={() => removeWire({ from: edge.from, port: p, to: edge.to })}>×</button>}
          </span>;
        })}
        {doc.nodes.map((node) => <ExperimentCanvasNode key={node.id} node={node} selected={chosen.has(node.id)} state={nodeStates.get(node.id)}
          detached={detached.has(node.id)} invalid={invalidNode === node.id} missing={missing} linkStart={linkStart} controls={controls} onEdit={onEdit} />)}
        {band && <div className="experiment-band" aria-hidden="true" style={{ left: Math.min(band.a.x, band.b.x), top: Math.min(band.a.y, band.b.y),
          width: Math.abs(band.b.x - band.a.x), height: Math.abs(band.b.y - band.a.y) }} />}
      </div>
    </div>
  </div>;
}
