import { useCallback, useEffect, useImperativeHandle, useRef, useState, type CSSProperties, type Ref } from "react";
import type { Experiment, ExperimentNode, SignalBody } from "../lib/api";
import { addAfter, addBranch, anchorAfter, arrangeNodes, createNode, createNodeIn, duplicateNode, placeAfter, removeNode, NODE_HEIGHT as NODE_H, type Anchor, type NodeType, type Port } from "../lib/experimentGraph";
import { useExperimentDocument } from "../lib/useExperimentDocument";
import { useExperimentViewport } from "../lib/useExperimentViewport";
import { useExperimentRun } from "../lib/useExperimentRun";
import { useExperimentFullscreen } from "../lib/useExperimentFullscreen";
import { useExperimentCanvas, type Menu } from "../lib/useExperimentCanvas";
import { useExperimentNodeTests } from "../lib/useExperimentNodeTests";
import { useExperimentShortcuts } from "../lib/useExperimentShortcuts";
import { nodeLabel, nodeSummary } from "../lib/experimentText";
import { jsonPath, suggestVariableName, writtenVariable } from "../lib/experimentData";
import { describeError, failureNode, type Failure } from "../lib/errors";
import { nodeFromSignal } from "../lib/signals";
import { useStore } from "../lib/store";
import { usePersistentState } from "../lib/hooks";
import { useT } from "../lib/i18n";
import { ErrorMessage } from "../components/ErrorMessage";
import { Splitter } from "../components/Splitter";
import { ExperimentToolbar } from "../components/ExperimentToolbar";
import { ExperimentCanvasTools } from "../components/ExperimentCanvasTools";
import { ExperimentCanvas } from "../components/ExperimentCanvas";
import { ExperimentProperties } from "../components/ExperimentProperties";
import { ExperimentTimeline } from "../components/ExperimentTimeline";
import { ExperimentAddMenu, type MenuItem } from "../components/ExperimentAddMenu";
import { ExperimentFinder } from "../components/ExperimentFinder";
import { ExperimentDocuments } from "../components/ExperimentDocuments";
import { ExperimentParams } from "../components/ExperimentParams";
import { ExperimentRunWith, type RunOptions } from "../components/ExperimentRunWith";

/** What the shell can ask of the editor while another screen is showing. */
export interface ExperimentHandle {
  /** Append an action built from a direct instrument; false when the editor cannot take it now. */
  add: (body: SignalBody) => boolean;
  /** Append a node made elsewhere (Wait for this); placed and selected like `add`. */
  addNode: (node: ExperimentNode) => boolean;
}

/** Default pane sizes, restored by a double click on their handle. */
const PROPERTIES_WIDTH = 254;
const TIMELINE_HEIGHT = 142;
const sizeValid = (value: unknown) => typeof value === "number" && Number.isFinite(value) && value > 0;

/**
 * The experiment editor. It holds what several panes share — the selection,
 * the wire being drawn, the add menu, the problem shown — and the edits that
 * cross them; the document, the run, the canvas pointer, Send now and the
 * keyboard are hooks of their own, and each pane is a component.
 */
export function ExperimentView({ active, focusMode, setFocusMode, onShowFrame, onShowEmulator, ref }: {
  active: boolean; focusMode: boolean; setFocusMode: (value: boolean) => void; onShowFrame?: (seq: number) => void; onShowEmulator?: (id: string) => void; ref?: Ref<ExperimentHandle>;
}) {
  const t = useT();
  const { pushLog } = useStore();
  const { document: doc, history, dispatch, save: saveDocument, replace, saveState, validationError, profileIssues, revalidate, error: documentError } = useExperimentDocument();
  const [selected, setSelected] = useState<string | null>(null);
  // The output a wire is being drawn from, and where its loose end is.
  const [linkStart, setLinkStart] = useState<Anchor | null>(null);
  const [linkPoint, setLinkPoint] = useState<{ x: number; y: number } | null>(null);
  const [menu, setMenu] = useState<Menu | null>(null);
  const [problem, setProblem] = useState<Failure | null>(null);
  const [propertiesWidth, setPropertiesWidth] = usePersistentState("signal-lab.layout.properties", PROPERTIES_WIDTH, sizeValid);
  const [timelineHeight, setTimelineHeight] = usePersistentState("signal-lab.layout.timeline", TIMELINE_HEIGHT, sizeValid);
  const viewRef = useRef<HTMLDivElement>(null);
  const workspaceRef = useRef<HTMLDivElement>(null);
  const { scrollRef, zoom, zoomAt, fit, reveal, ensureVisible, canvasWidth, canvasHeight } = useExperimentViewport(doc?.nodes);
  const [propertiesOpen, setPropertiesOpen] = useState(true);
  // Opens by itself when a run starts; until then the canvas gets the room.
  const [timelineOpen, setTimelineOpen] = useState(false);
  const [finderOpen, setFinderOpen] = useState(false);
  const [documentsOpen, setDocumentsOpen] = useState(false);
  const [runWithAt, setRunWithAt] = useState<{ right: number; top: number } | null>(null);
  // What Run with… was last used with, for the next time it opens this session.
  const [runWithLast, setRunWithLast] = useState<RunOptions | null>(null);
  const [paramsAt, setParamsAt] = useState<{ left: number; top: number } | null>(null);
  // Secret names stored from the Secrets panel this session, for suggestions;
  // the counter re-runs the preview after the store changed.
  const [storedSecrets, setStoredSecrets] = useState<string[]>([]);
  const [secretsVersion, setSecretsVersion] = useState(0);
  const editGroup = useRef<string | null>(null);
  const selectionInitialized = useRef(false);
  const propertiesRef = useRef<HTMLElement>(null);
  const focusFieldOf = useRef<string | null>(null);
  const pendingReveal = useRef<string | null>(null);

  const label = (kind: NodeType): string => nodeLabel(kind, t);

  const edit = useCallback((fn: (current: Experiment) => Experiment, group = editGroup.current) => {
    dispatch({ type: "edit", update: (current) => current ? fn(current) : current, group });
    setProblem(null);
  }, [dispatch]);
  const commitEdit = () => { editGroup.current = null; dispatch({ type: "commit" }); };
  const patchNode = (id: string, change: Partial<ExperimentNode>, group = editGroup.current) => edit((current) => ({
    ...current, nodes: current.nodes.map((node) => node.id === id ? { ...node, ...change } as ExperimentNode : node),
  }), group);
  const cancelLink = () => { setLinkStart(null); setLinkPoint(null); };

  const focusNode = (id: string) => scrollRef.current?.querySelector<HTMLButtonElement>(`[data-node-id="${CSS.escape(id)}"]`)?.focus({ preventScroll: true });
  const showNode = (node: ExperimentNode) => {
    setSelected(node.id); reveal(node); setFinderOpen(false);
    focusNode(node.id);
  };

  /** A node as the reader knows it: its type, not its generated id. */
  const nodeName = (id: string): string | null => {
    const node = doc?.nodes.find((item) => item.id === id);
    return node ? label(node.type) : null;
  };
  /** Any failure in one line, located by node and field. */
  const describe = (failure: Failure): string => describeError(failure, t, nodeName).text;
  const problemNodeId = failureNode(validationError) ?? null;

  /** Say what is wrong in words and put the node it is about in front of the user. */
  const revealProblem = (failure: Failure) => {
    if (!doc) return;
    setProblem(failure);
    const id = failureNode(failure);
    const node = id ? doc.nodes.find((item) => item.id === id) : null;
    if (node) showNode(node);
  };

  const { events, job, outcome, reportPath, starting, busy, knownVars, setKnownVars, lastSeed, lastRun, run, stop, reset: resetRun } = useExperimentRun({
    doc, save: saveDocument, setProblem, revealProblem,
    onLaunch: () => { cancelLink(); setTimelineOpen(true); },
  });
  const { fullscreen, toggleFullscreen, exitFullscreen } = useExperimentFullscreen(focusMode, setFocusMode, setProblem);
  const canvas = useExperimentCanvas({ doc, busy, zoom, scrollRef, selected, setSelected, linkStart, setLinkStart, setLinkPoint, cancelLink, setMenu, edit, commitEdit, patchNode, setProblem });
  const selectedNode = doc?.nodes.find((node) => node.id === selected) ?? null;
  const { tests, sending, preview, sendNode, reset: resetTests } = useExperimentNodeTests({ doc, selectedNode, busy, knownVars, setKnownVars, secretsVersion });

  const restore = (type: "undo" | "redo") => {
    if (busy) return;
    editGroup.current = null; canvas.drag.current = null;
    dispatch({ type }); cancelLink(); setMenu(null); setProblem(null);
  };

  /** Select a node that was just put on the canvas, bring it into view and start editing it. */
  const inserted = (id: string) => {
    setSelected(id); setPropertiesOpen(true);
    focusFieldOf.current = id; pendingReveal.current = id;
  };

  const removeSelected = () => {
    if (!selected || busy) return;
    edit((current) => removeNode(current, selected));
    setSelected(null);
  };

  const duplicateSelected = () => {
    if (!doc || !selected || busy) return;
    const copy = duplicateNode(doc, selected);
    if (!copy) return;
    edit((current) => ({ ...current, nodes: [...current.nodes, copy] }));
    setSelected(copy.id); reveal(copy);
  };

  const arrange = () => {
    if (!doc || busy) return;
    const arranged = arrangeNodes(doc);
    edit(() => arranged); fit(arranged.nodes);
  };

  useEffect(() => {
    if (!doc || selectionInitialized.current) return;
    selectionInitialized.current = true;
    setSelected(doc.nodes.find((node) => node.type === "http")?.id ?? doc.nodes[0]?.id ?? null);
  }, [doc]);

  // A node that was just created gets its most important field focused, so
  // "A, type, Enter, type the URL" never needs the mouse.
  useEffect(() => {
    const id = focusFieldOf.current;
    if (!id || selected !== id || !propertiesOpen || !active) return;
    focusFieldOf.current = null;
    // A text field is selected to be typed over; a node whose main control is a button (Emulator: Edit…) just focuses it.
    const field = propertiesRef.current?.querySelector<HTMLElement>("[data-primary]");
    if (field) {
      field.focus();
      if (field instanceof HTMLInputElement || field instanceof HTMLTextAreaElement) field.select();
    } else focusNode(id);
  });

  // A new node is scrolled into view once it is on the canvas — for one added from
  // another screen, once this screen has a size again.
  useEffect(() => {
    if (!active || !doc || !pendingReveal.current) return;
    const node = doc.nodes.find((item) => item.id === pendingReveal.current);
    if (!node) return;
    pendingReveal.current = null;
    ensureVisible(node);
  }, [active, doc, ensureVisible]);

  /** A click on a response value: an Extract node right after the request, value known at once. */
  const extractPicked = (request: ExperimentNode, path: (string | number)[], value: unknown) => {
    if (!doc || busy) return;
    const taken = [...doc.params.map((param) => param.name), ...doc.nodes.flatMap((node) => { const written = writtenVariable(node); return written ? [written.name] : []; })];
    const name = suggestVariableName(path, taken);
    const anchor = { from: request.id, port: "next" as Port };
    const spot = placeAfter(doc, anchor);
    const { id, x, y } = createNode("extract", spot.x, spot.y);
    const node: ExperimentNode = { id, x, y, type: "extract", variable: name, from: "json", expr: jsonPath(path) };
    commitEdit();
    edit((current) => addAfter(current, anchor, node));
    setKnownVars((prior) => ({ ...prior, [name]: value }));
    inserted(node.id);
  };

  const addFromMenu = (item: MenuItem) => {
    if (!menu || !doc) return;
    const node = item.kind === "node" ? createNodeIn(doc, item.type, menu.x, menu.y) : nodeFromSignal(item.signal.body, menu.x, menu.y);
    if (!node) return;
    commitEdit();
    edit((current) => (menu.branch ? addBranch : addAfter)(current, menu.anchor ?? null, node));
    setMenu(null);
    inserted(node.id);
  };

  /** A node from a direct instrument, as the next step before End (the selection is only a fallback). */
  const appendNode = (make: (x: number, y: number) => ExperimentNode | null): boolean => {
    if (!doc || busy) return false;
    // The end of the flow is the predictable place; the selection is only a fallback.
    const end = doc.nodes.find((node) => node.type === "end");
    const anchor = (end && anchorAfter(doc, end.id)) || (selected ? anchorAfter(doc, selected) : null);
    const spot = anchor ? placeAfter(doc, anchor)
      : { x: Math.min(...doc.nodes.map((node) => node.x)), y: Math.max(...doc.nodes.map((node) => node.y)) + NODE_H + 48 };
    const node = make(spot.x, spot.y);
    if (!node) return false;
    commitEdit();
    edit((current) => addAfter(current, anchor, node));
    setSelected(node.id); setPropertiesOpen(true);
    pendingReveal.current = node.id;
    pushLog("ok", "experiment", "log.addedToExperiment", { node: label(node.type), name: doc.name });
    return true;
  };
  useImperativeHandle(ref, () => ({
    add: (body) => appendNode((x, y) => nodeFromSignal(body, x, y)),
    addNode: (node) => appendNode((x, y) => ({ ...node, id: `${node.type}-${crypto.randomUUID()}`, x, y })),
  }));

  useExperimentShortcuts({
    active, busy, doc, selected, selectedNode, selectedWire: canvas.selectedWire,
    menuOpen: !!menu, linking: !!linkStart, fullscreen, focusMode,
    closeMenu: () => setMenu(null), cancelLink, dropWire: () => canvas.setWire(null), exitFullscreen, setFocusMode,
    restore, duplicateSelected, openFinder: () => setFinderOpen(true), fit, zoomAt,
    sendNode, removeWire: canvas.removeWire, removeSelected, openAddMenu: canvas.openAddMenu,
    editGroup, patchNode, commitEdit,
  });

  const openDocument = (document: Experiment) => {
    if (busy) return;
    commitEdit(); replace(document);
    setDocumentsOpen(false); setMenu(null); cancelLink(); setFinderOpen(false);
    setSelected(document.nodes[0]?.id ?? null);
    resetRun(); setProblem(null); resetTests();
    requestAnimationFrame(() => fit(document.nodes));
  };

  const documentsDialog = documentsOpen && <ExperimentDocuments document={doc} onOpen={openDocument} onClose={() => setDocumentsOpen(false)} />;

  if (!doc) return <div className="experiment-loading">{documentError !== null ? <ErrorMessage error={documentError} /> : t("exp.loading")}
    {documentError !== null && <button className="ghost" onClick={() => setDocumentsOpen(true)}>{t("exp.documents")}</button>}{documentsDialog}
  </div>;

  return <div ref={viewRef} className={`experiment-view ${propertiesOpen ? "" : "properties-closed"} ${timelineOpen ? "" : "timeline-closed"}`}
    style={{ "--properties-w": `${propertiesWidth}px`, "--timeline-h": `${timelineHeight}px` } as CSSProperties}
    onFocusCapture={(event) => { if (event.target.matches("input, textarea, select")) editGroup.current = `field-${crypto.randomUUID()}`; }}
    onBlurCapture={(event) => { if (event.target.matches("input, textarea, select")) commitEdit(); }}>
    <ExperimentToolbar doc={doc} busy={busy} running={!!job} starting={starting} saveState={saveState} validationError={validationError} profileIssues={profileIssues}
      describe={describe} addAfter={!!selectedNode && !!anchorAfter(doc, selectedNode.id)} paramsOpen={!!paramsAt} propertiesOpen={propertiesOpen} focusMode={focusMode} fullscreen={fullscreen}
      onDocuments={() => { setMenu(null); setDocumentsOpen(true); }} onEdit={edit} onCommit={commitEdit} onRevealProblem={revealProblem} onAdd={canvas.openAddMenu}
      onParams={(anchor) => { setMenu(null); setParamsAt(anchor); }} onProperties={() => setPropertiesOpen(!propertiesOpen)} setFocusMode={setFocusMode}
      onFullscreen={toggleFullscreen} onRun={() => run()} onStop={stop} onRunWith={(anchor) => { setMenu(null); setRunWithAt(anchor); }} />
    {(problem ?? documentError) !== null && <ErrorMessage className="experiment-problem" error={problem ?? documentError} nodeLabel={nodeName}
      onShow={(id) => { const node = doc.nodes.find((item) => item.id === id); if (node) showNode(node); }} />}
    <div className="experiment-workspace" ref={workspaceRef}>
      {propertiesOpen && <Splitter orientation="vertical" label="layout.properties" size={propertiesWidth} min={220} initial={PROPERTIES_WIDTH}
        max={() => Math.max(220, (workspaceRef.current?.clientWidth ?? 1200) - 360)} onSize={setPropertiesWidth} />}
      <div className="experiment-left">
        <ExperimentCanvasTools busy={busy} canUndo={history.past.length > 0} canRedo={history.future.length > 0} nodeCount={doc.nodes.length} linking={!!linkStart} zoom={zoom}
          onRestore={restore} onFind={() => setFinderOpen(true)} onCancelLink={cancelLink} onArrange={arrange} onZoom={zoomAt} onFit={() => fit(doc.nodes)} />
        <ExperimentCanvas doc={doc} scrollRef={scrollRef} zoom={zoom} width={canvasWidth} height={canvasHeight} busy={busy} events={events} selected={selected}
          invalidNode={problemNodeId} linkStart={linkStart} linkPoint={linkPoint} controls={canvas} onSelect={setSelected}
          onEdit={(id) => { setPropertiesOpen(true); focusFieldOf.current = id; setSelected(id); }} />
      </div>
      {propertiesOpen && <ExperimentProperties panelRef={propertiesRef} doc={doc} node={selectedNode} wire={canvas.selectedWire} busy={busy}
        problemNodeId={problemNodeId} validationError={validationError} storedSecrets={storedSecrets} preview={preview}
        test={selectedNode ? tests[selectedNode.id] : undefined} sending={sending} onSend={sendNode} onExtract={extractPicked} onShowEmulator={onShowEmulator}
        onPatch={patchNode} onEdit={edit} onAddNext={canvas.openAddMenu} onDuplicate={duplicateSelected} onDelete={removeSelected}
        onRemoveWire={canvas.removeWire} onLeave={focusNode} />}
    </div>
    <ExperimentTimeline doc={doc} events={events} outcome={outcome} running={!!job} reportPath={reportPath} lastRun={lastRun} lastSeed={lastSeed} busy={busy}
      open={timelineOpen} onToggle={() => setTimelineOpen(!timelineOpen)} height={timelineHeight} initialHeight={TIMELINE_HEIGHT}
      maxHeight={() => Math.max(90, (viewRef.current?.clientHeight ?? 700) - 260)} onHeight={setTimelineHeight}
      describe={describe} onEdit={edit} onShowNode={showNode} onShowFrame={onShowFrame} />
    {menu && <ExperimentAddMenu menu={menu} nodes={doc.nodes} onAdd={addFromMenu} onClose={() => setMenu(null)} />}
    {paramsAt && <ExperimentParams doc={doc} issues={profileIssues} disabled={busy} anchor={paramsAt} describe={describe}
      onClose={() => { commitEdit(); setParamsAt(null); }} onEdit={(update) => edit(update)}
      onSecretsChanged={(name) => { if (name) setStoredSecrets((prior) => prior.includes(name) ? prior : [...prior, name]); setSecretsVersion((v) => v + 1); revalidate(); }} />}
    {runWithAt && <ExperimentRunWith doc={doc} issues={profileIssues} lastSeed={lastSeed} initial={runWithLast} anchor={runWithAt}
      onClose={() => setRunWithAt(null)} onRun={(options) => { setRunWithLast(options); setRunWithAt(null); void run(options); }} />}
    {finderOpen && <ExperimentFinder nodes={doc.nodes} label={label} summary={(node) => nodeSummary(node, t)} onSelect={showNode} onClose={() => setFinderOpen(false)} />}
    {documentsDialog}
  </div>;
}
