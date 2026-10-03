import type { ExperimentNode, ExperimentStep } from "../lib/api";
import { NODE_CATALOG } from "../lib/experimentCatalog";
import { validPortsFor as outputPorts, NODE_HEIGHT as NODE_H, NODE_WIDTH as NODE_W, type Anchor } from "../lib/experimentGraph";
import { nodeLabel, nodeSummary, portLabel } from "../lib/experimentText";
import type { CanvasControls } from "../lib/useExperimentCanvas";
import { useT } from "../lib/i18n";

const STATE_GLYPH: Record<string, string> = { running: "●", passed: "✓", retry: "↻", repeating: "⟳" };

/**
 * One node on the canvas: its body (select — with Shift or Ctrl, one more —,
 * drag, double-click to edit), its badges and the state of the run, its input
 * and its outputs.
 */
export function ExperimentCanvasNode({ node, selected, state, detached, invalid, missing, linkStart, controls, onEdit }: {
  node: ExperimentNode;
  selected: boolean;
  /** Where the run is with this node, if it got there. */
  state: ExperimentStep["state"] | undefined;
  /** No path from Start reaches it. */
  detached: boolean;
  /** What stops the document from running is about this node. */
  invalid: boolean;
  /** Outputs (`id:port`) that need a wire. */
  missing: Set<string>;
  linkStart: Anchor | null;
  controls: CanvasControls;
  /** Open the node's fields and put the cursor in the first. */
  onEdit: (id: string) => void;
}) {
  const t = useT();
  const { endLink, onNodeDown, onNodeMove, endDrag, onNodeClick, onNodeFocus, onPortDown, onPortMove, onPortUp, onPortCancel, onPortClick } = controls;
  const label = nodeLabel(node.type, t);
  const summary = nodeSummary(node, t);
  return <div data-node={node.id} data-group={NODE_CATALOG[node.type].group}
    className={`experiment-node ${selected ? "selected" : ""} ${state ?? ""} ${detached ? "detached" : ""} ${invalid ? "invalid" : ""} ${linkStart && linkStart.from !== node.id ? "link-target" : ""}`}
    style={{ left: node.x, top: node.y, width: NODE_W, height: NODE_H }}>
    <button className="experiment-node-body" data-node-id={node.id} data-tip={`${label} — ${summary}${detached ? `\n${t("exp.detachedHint")}` : ""}`}
      onPointerDown={(event) => onNodeDown(event, node)} onPointerMove={(event) => onNodeMove(event, node)} onPointerUp={endDrag} onPointerCancel={endDrag}
      onFocus={() => onNodeFocus(node)} onClick={(event) => onNodeClick(event, node)}
      onDoubleClick={() => onEdit(node.id)} aria-pressed={selected}
      aria-label={`${label}: ${summary}`}>
      <div className="experiment-node-header">
        <span className="experiment-node-type">{label}</span>
        {node.repeat && (() => { const tip = node.repeat.until === "count" ? t("exp.repeatBadgeCount", { count: node.repeat.count }) : t("exp.repeatBadgeFor", { s: node.repeat.duration_ms / 1000 });
          return <span className="experiment-node-repeat" data-tip={tip} aria-label={tip}>{node.repeat.until === "count" ? `×${node.repeat.count}` : `${node.repeat.duration_ms / 1000}s`}</span>; })()}
        {node.retry && <span className="experiment-node-retry" data-tip={t("exp.retryBadge", { attempts: node.retry.attempts })} aria-label={t("exp.retryBadge", { attempts: node.retry.attempts })}>↻{node.retry.attempts}</span>}
        {state !== undefined && <span className={`node-status-badge ${state}`} aria-label={t(`exp.${state}`)}>
          {STATE_GLYPH[state] ?? "✕"}
        </span>}
      </div>
      <span className="experiment-node-subtitle">{summary}</span>
    </button>
    {node.type !== "start" && <button className={`experiment-port input ${linkStart ? "awaiting" : ""}`} data-tip={t("exp.inputPort")} aria-label={`${label}: ${t("exp.inputPort")}`} onClick={() => endLink(node.id)} />}
    {outputPorts(node.type).map((port) => <button key={port}
      className={`experiment-port output ${port} ${missing.has(`${node.id}:${port}`) ? "missing" : ""} ${linkStart?.from === node.id && linkStart.port === port ? "active" : ""}`}
      data-tip={`${portLabel(port, t)} — ${t("exp.portHint")}`} aria-label={`${label}: ${portLabel(port, t)}`}
      onPointerDown={(event) => onPortDown(event, { from: node.id, port })} onPointerMove={onPortMove} onPointerUp={onPortUp} onPointerCancel={onPortCancel}
      onClick={() => onPortClick(node.id, port)} />)}
  </div>;
}
