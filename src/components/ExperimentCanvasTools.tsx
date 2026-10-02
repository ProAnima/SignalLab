import { useT } from "../lib/i18n";

/** The bar over the canvas: undo and redo, the node finder, the wire being drawn, arranging and zoom. */
export function ExperimentCanvasTools({ busy, canUndo, canRedo, nodeCount, linking, zoom, onRestore, onFind, onCancelLink, onArrange, onZoom, onFit }: {
  busy: boolean;
  canUndo: boolean;
  canRedo: boolean;
  nodeCount: number;
  /** A wire is being drawn and waits for the node it leads to. */
  linking: boolean;
  zoom: number;
  onRestore: (type: "undo" | "redo") => void;
  onFind: () => void;
  onCancelLink: () => void;
  onArrange: () => void;
  onZoom: (value: number) => void;
  onFit: () => void;
}) {
  const t = useT();
  return <div className="experiment-canvas-tools">
    <div className="experiment-tool-group" role="group" aria-label={t("exp.history")}>
      <button className="ghost sm" data-tip={`${t("exp.undo")} · Ctrl+Z`} aria-label={t("exp.undo")} disabled={busy || !canUndo} onClick={() => onRestore("undo")}>↶</button>
      <button className="ghost sm" data-tip={`${t("exp.redo")} · Ctrl+Shift+Z`} aria-label={t("exp.redo")} disabled={busy || !canRedo} onClick={() => onRestore("redo")}>↷</button>
    </div>
    <button className="ghost sm" data-tip={`${t("exp.findNode")} · Ctrl+F`} onClick={onFind}>{t("exp.nodes")} <span className="experiment-node-count">{nodeCount}</span></button>
    {linking && <span className="experiment-canvas-hint linking" role="status">{t("exp.chooseInput")}</span>}
    {linking && <button className="ghost sm" onClick={onCancelLink}>{t("exp.cancelLink")}</button>}
    <div className="fill" />
    <button className="ghost sm experiment-arrange" data-tip={t("exp.arrangeHint")} onClick={onArrange} disabled={busy}>{t("exp.arrange")}</button>
    <button className="ghost sm" aria-label={t("exp.zoomOut")} data-tip={t("exp.zoomOut")} onClick={() => onZoom(zoom - .1)}>−</button>
    <span className="experiment-zoom" data-tip={t("exp.zoomHint")}>{Math.round(zoom * 100)}%</span>
    <button className="ghost sm" aria-label={t("exp.zoomIn")} data-tip={t("exp.zoomIn")} onClick={() => onZoom(zoom + .1)}>＋</button>
    <button className="ghost sm" data-tip={`${t("exp.resetZoom")} · Ctrl+1`} onClick={() => onZoom(1)}>1:1</button>
    <button className="ghost sm" data-tip={`${t("exp.fit")} · Ctrl+0`} aria-label={t("exp.fit")} onClick={onFit}>⊡</button>
  </div>;
}
