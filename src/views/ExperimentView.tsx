import { useCallback, useEffect, useImperativeHandle, useMemo, useRef, useState, type CSSProperties, type PointerEvent, type Ref } from "react";
import { api, EV, on, type Experiment, type ExperimentEnded, type ExperimentNode, type ExperimentStep, type HttpResponse, type JobInfo, type Signal, type SignalBody } from "../lib/api";
import { addAfter, addBranch, anchorAfter, arrangeNodes, connect, createNode, disconnect, duplicateNode, isWired, missingOutputs, placeAfter, portOf, removeNode, unreachableNodes, validPortsFor, NODE_WIDTH as NODE_W, NODE_HEIGHT as NODE_H, STEP_X, type Anchor, type NodeType, type Port } from "../lib/experimentGraph";
import { useExperimentDocument } from "../lib/useExperimentDocument";
import { useExperimentViewport } from "../lib/useExperimentViewport";
import { ExperimentFinder } from "../components/ExperimentFinder";
import { ExperimentDocuments } from "../components/ExperimentDocuments";
import { NODE_CATALOG, ADDABLE_NODES, NODE_GROUPS, type NodeGroup } from "../lib/experimentCatalog";
import { ExperimentNodeFields } from "../components/ExperimentNodeFields";
import { ExperimentParams } from "../components/ExperimentParams";
import { ExperimentRunWith, type RunOptions } from "../components/ExperimentRunWith";
import { JsonPicker } from "../components/JsonPicker";
import { TemplateSuggestions, type TemplateSuggestion } from "../components/TemplateField";
import { GENERATORS, jsonPath, replyFields, secretNames, suggestVariableName, variablesBefore, writtenVariable } from "../lib/experimentData";
import { describeError, failureNode, messageParams, type Failure } from "../lib/errors";
import { ErrorMessage } from "../components/ErrorMessage";
import { downloadUrl, isFullscreen, onFullscreenChange, setFullscreen as setWindowFullscreen } from "../lib/platform";
import { nodeFromSignal, signalBodyOfNode, signalTarget, transportKey } from "../lib/signals";
import { fmtBytes, fmtTime, prettyJson } from "../lib/format";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";

/** What the shell can ask of the editor while another screen is showing. */
export interface ExperimentHandle {
  /** Append an action built from a direct instrument; false when the editor cannot take it now. */
  add: (body: SignalBody) => boolean;
}

/** `branch`: the node goes on a new wire of the anchor's output (parallel work), not into its flow. */
type Menu = { screenX: number; screenY: number; x: number; y: number; anchor?: Anchor; branch?: boolean };
type MenuItem = { kind: "node"; type: NodeType } | { kind: "signal"; signal: Signal };
/** What the last Send now of a node did. A failure is kept as it came and described when shown. */
type NodeTest = { ok: boolean; text: string; ts: number; error?: Failure; missing?: string[]; response?: HttpResponse; body?: string; json?: unknown; values?: Record<string, unknown> };
type Preview = { nodeId: string; lines: string[]; missing: string[]; error?: Failure };
type Outcome = { kind: "passed" | "failed" | "stopped"; error?: Failure } | null;

const GROUP_GLYPH: Record<NodeGroup, string> = { action: "↗", observe: "⇠", data: "{}", check: "✓", flow: "◇" };
/** `:9001` from `127.0.0.1:9001`: the port is what tells waits apart on the canvas. */
const bindPort = (bind: string) => { const at = bind.lastIndexOf(":"); return at >= 0 ? bind.slice(at) : bind; };
const isWait = (node: ExperimentNode) => node.type === "wait_osc" || node.type === "wait_udp";
/** A node heading's tooltip: what the node does; for the parallel nodes, how they are wired. */
const nodeHelp = (type: NodeType) => type === "fork" ? "exp.forkHint" as const : type === "join" ? "exp.joinHint" as const : NODE_CATALOG[type].description;
const OP_TEXT: Record<string, string> = { eq: "=", ne: "≠", lt: "<", le: "≤", gt: ">", ge: "≥", contains: "⊃", matches: "~", empty: "= ∅", not_empty: "≠ ∅" };

/** What a node will put on the wire, one line per part, for the resolved preview. */
function previewLines(node: ExperimentNode): string[] {
  const clip = (text: string) => text.length > 300 ? `${text.slice(0, 300)}…` : text;
  switch (node.type) {
    case "http": return [`${node.request.method} ${node.request.url}`, ...node.request.headers.filter(([name]) => name).map(([name, value]) => `${name}: ${value}`), ...(node.request.body ? [clip(node.request.body)] : [])];
    case "osc": return [`${node.address} → ${node.target}`, ...node.args.filter((arg) => arg.type === "str").map((arg) => `"${(arg as { value: string }).value}"`)];
    case "udp": return [`→ ${node.target}`, clip(node.text)];
    case "tcp": return [`→ ${node.host}:${node.port}`, clip(node.payload)];
    case "mqtt": return [`${node.topic} → ${node.host}:${node.port}`, clip(node.payload)];
    case "log": return [clip(node.message)];
    case "assert_body": return [`⊃ ${node.contains}`];
    case "assert_header": return [`${node.name}: ${node.contains}`];
    case "assert_value": case "branch_value": return [`${node.value} ${OP_TEXT[node.op]} ${node.op === "empty" || node.op === "not_empty" ? "" : node.expected}`.trim()];
    case "wait_osc": return [`${node.address} ⇠ ${node.bind}`, ...node.args.map((rule) => `args[${rule.index}] ${OP_TEXT[rule.op]} ${rule.op === "empty" || rule.op === "not_empty" ? "" : rule.value}`.trim())];
    case "wait_udp": return [`${node.mode === "any" ? "*" : node.pattern} ⇠ ${node.bind}`];
    default: return [];
  }
}

function summary(node: ExperimentNode, t: (key: any) => string): string {
  switch (node.type) {
    case "http": return `${node.request.method} ${node.request.url}`;
    case "tcp": return `${node.host}:${node.port}`;
    case "log": return node.message || t("exp.node.log");
    case "fork": return t("exp.forkSummary");
    case "join": return t("exp.joinSummary");
    case "assert_body": return `⊃ ${node.contains}`;
    case "assert_header": return `${node.name}: ${node.contains}`;
    case "assert_latency": return `≤ ${node.max_ms} ms`;
    case "mqtt": return `${node.topic} → ${node.host}:${node.port}`;
    case "delay": return `${node.ms} ms`;
    case "assert_status": return `HTTP = ${node.status}`;
    case "branch_status": return `HTTP = ${node.status} ?`;
    case "osc": return `${node.address} → ${node.target}`;
    case "udp": return `${node.text || "∅"} → ${node.target}`;
    case "extract": return `${node.variable} ← ${node.from === "json" || node.from === "header" || node.from === "regex" ? node.expr : node.from}`;
    case "assert_value": return `${node.value} ${OP_TEXT[node.op]} ${node.op === "empty" || node.op === "not_empty" ? "" : node.expected}`.trim();
    case "branch_value": return `${node.value} ${OP_TEXT[node.op]} ${node.op === "empty" || node.op === "not_empty" ? "" : node.expected} ?`;
    case "wait_osc": return `${node.address} ⇠ ${bindPort(node.bind)} · ${node.timeout_ms} ms`;
    case "wait_udp": return `${node.mode === "any" ? "*" : node.pattern || "∅"} ⇠ ${bindPort(node.bind)} · ${node.timeout_ms} ms`;
    default: return "";
  }
}

const outputPorts = validPortsFor;
const editable = (target: EventTarget | null) => target instanceof HTMLElement && !!target.closest("input, textarea, select, [contenteditable=true]");

export function ExperimentView({ active, focusMode, setFocusMode, ref }: { active: boolean; focusMode: boolean; setFocusMode: (value: boolean) => void; ref?: Ref<ExperimentHandle> }) {
  const t = useT();
  const { refreshJobs, library, pushLog, pushError } = useStore();
  const { document: doc, history, dispatch, save: saveDocument, replace, saveState, validationError, profileIssues, revalidate, error: documentError } = useExperimentDocument();
  const [selected, setSelected] = useState<string | null>(null);
  const [events, setEvents] = useState<ExperimentStep[]>([]);
  const [job, setJob] = useState<JobInfo | null>(null);
  const [outcome, setOutcome] = useState<Outcome>(null);
  const [reportPath, setReportPath] = useState("");
  const [problem, setProblem] = useState<Failure | null>(null);
  const { scrollRef, zoom, zoomAt, fit, reveal, ensureVisible, canvasWidth, canvasHeight } = useExperimentViewport(doc?.nodes);
  const [propertiesOpen, setPropertiesOpen] = useState(true);
  // Opens by itself when a run starts; until then the canvas gets the room.
  const [timelineOpen, setTimelineOpen] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);
  const [menu, setMenu] = useState<Menu | null>(null);
  const [search, setSearch] = useState("");
  const [menuIndex, setMenuIndex] = useState(0);
  const [linkStart, setLinkStart] = useState<Anchor | null>(null);
  const [linkPoint, setLinkPoint] = useState<{ x: number; y: number } | null>(null);
  const [finderOpen, setFinderOpen] = useState(false);
  const [documentsOpen, setDocumentsOpen] = useState(false);
  const [starting, setStarting] = useState(false);
  const [tests, setTests] = useState<Record<string, NodeTest>>({});
  const [sending, setSending] = useState<string | null>(null);
  // Variable values seen in the last run and from Send now; the preview and Send now use them.
  const [knownVars, setKnownVars] = useState<Record<string, unknown>>({});
  const [lastSeed, setLastSeed] = useState<number | null>(null);
  // How the last run was set up, shown next to its result.
  const [lastRun, setLastRun] = useState<{ profile: string | null; overridden: boolean } | null>(null);
  const [runWithAt, setRunWithAt] = useState<{ right: number; top: number } | null>(null);
  // What Run with… was last used with, for the next time it opens this session.
  const [runWithLast, setRunWithLast] = useState<RunOptions | null>(null);
  const [paramsAt, setParamsAt] = useState<{ left: number; top: number } | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  // Secret names stored from the Secrets panel this session, for suggestions;
  // the counter re-runs the preview after the store changed.
  const [storedSecrets, setStoredSecrets] = useState<string[]>([]);
  const [secretsVersion, setSecretsVersion] = useState(0);
  const launchPending = useRef(false);
  const busy = !!job || starting;
  const editGroup = useRef<string | null>(null);
  const selectionInitialized = useRef(false);
  const activeId = useRef<number | null>(null);
  const focusBeforeFullscreen = useRef(false);
  const propertiesRef = useRef<HTMLElement>(null);
  const focusFieldOf = useRef<string | null>(null);
  const pendingReveal = useRef<string | null>(null);
  const suppressPortClick = useRef(false);
  const drag = useRef<{ id: string; x: number; y: number; clientX: number; clientY: number; group: string } | null>(null);
  const pan = useRef<{ clientX: number; clientY: number; left: number; top: number; moved: boolean } | null>(null);
  const portDrag = useRef<{ anchor: Anchor; clientX: number; clientY: number; moved: boolean } | null>(null);

  const label = (kind: NodeType): string => t(NODE_CATALOG[kind].title);
  const nodeSummary = (node: ExperimentNode) => node.type === "start" ? t("exp.entry") : node.type === "end" ? t("exp.result") : summary(node, t);

  const portY = (node: ExperimentNode, port: Port): number => {
    if (node.type === "branch_status" || node.type === "branch_value") return port === "yes" ? 28 : 56;
    if (node.type === "fork") return port === "branch1" ? 28 : 56;
    if (isWait(node)) return port === "matched" ? 28 : 56;
    return NODE_H / 2;
  };

  const portLabel = (port: Port): string => {
    switch (port) {
      case "yes": return t("exp.yes");
      case "no": return t("exp.no");
      case "branch1": return t("exp.branch1");
      case "branch2": return t("exp.branch2");
      case "matched": return t("exp.portMatched");
      case "timeout": return t("exp.portTimeout");
      default: return t("exp.outputPort");
    }
  };

  const anchorLabel = (anchor: Anchor): string => {
    const node = doc?.nodes.find((item) => item.id === anchor.from);
    if (!node) return "";
    return outputPorts(node.type).length > 1 ? `${label(node.type)} · ${portLabel(anchor.port)}` : label(node.type);
  };

  useEffect(() => {
    if (!doc || selectionInitialized.current) return;
    selectionInitialized.current = true;
    setSelected(doc.nodes.find((node) => node.type === "http")?.id ?? doc.nodes[0]?.id ?? null);
  }, [doc]);

  useEffect(() => {
    isFullscreen().then(setFullscreen).catch(() => {});
    const stopWatching = onFullscreenChange(() => { isFullscreen().then(setFullscreen).catch(() => {}); });
    const steps = on<ExperimentStep>(EV.experimentStep, (event) => {
      if (activeId.current === null || (activeId.current !== -1 && event.payload.job_id !== activeId.current)) return;
      setEvents((prior) => [...prior, event.payload]);
      const written = event.payload.vars;
      if (written) setKnownVars((prior) => ({ ...prior, ...written }));
    });
    const ended = on<ExperimentEnded>(EV.experimentEnded, (event) => {
      if (activeId.current === null || (activeId.current !== -1 && event.payload.job_id !== activeId.current)) return;
      setOutcome(event.payload.error ? { kind: "failed", error: event.payload.error } : { kind: "passed" });
      setLastSeed(event.payload.seed ?? null);
      setLastRun({ profile: event.payload.profile ?? null, overridden: !!event.payload.overridden });
      setReportPath(event.payload.report_path ?? "");
      if (event.payload.report_error) setProblem(event.payload.report_error);
      setJob(null); activeId.current = null; refreshJobs();
    });
    return () => { steps.then((off) => off()); ended.then((off) => off()); stopWatching(); };
  }, [refreshJobs]);

  // The search field is focused by autoFocus as it mounts, so typing right after A is never lost.
  useEffect(() => { if (menu) { setSearch(""); setMenuIndex(0); } }, [menu]);

  // A node that was just created gets its most important field focused, so
  // "A, type, Enter, type the URL" never needs the mouse.
  useEffect(() => {
    const id = focusFieldOf.current;
    if (!id || selected !== id || !propertiesOpen || !active) return;
    focusFieldOf.current = null;
    const field = propertiesRef.current?.querySelector<HTMLInputElement | HTMLTextAreaElement>("[data-primary]");
    if (field) { field.focus(); field.select(); }
    else focusNode(id);
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

  const edit = useCallback((fn: (current: Experiment) => Experiment, group = editGroup.current) => {
    dispatch({ type: "edit", update: (current) => current ? fn(current) : current, group });
    setProblem(null);
  }, [dispatch]);

  const commitEdit = () => { editGroup.current = null; dispatch({ type: "commit" }); };
  const cancelLink = () => { setLinkStart(null); setLinkPoint(null); };
  const restore = (type: "undo" | "redo") => {
    if (busy) return;
    editGroup.current = null; drag.current = null;
    dispatch({ type }); cancelLink(); setMenu(null); setProblem(null);
  };

  const patchNode = (id: string, change: Partial<ExperimentNode>, group = editGroup.current) => edit((current) => ({
    ...current, nodes: current.nodes.map((node) => node.id === id ? { ...node, ...change } as ExperimentNode : node),
  }), group);

  const focusNode = (id: string) => scrollRef.current?.querySelector<HTMLButtonElement>(`[data-node-id="${CSS.escape(id)}"]`)?.focus({ preventScroll: true });

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

  /** Send one action node on its own, through the same path as the direct instruments. */
  /** Names without a value, in words: variables wait for a run, secrets for the Secrets panel. */
  const missingParts = (missing: string[]): { key: "exp.missingValues" | "exp.missingSecrets"; names: string }[] => {
    const quote = (names: string[]) => names.map((item) => `{{${item}}}`).join(", ");
    const secretsMissing = missing.filter((item) => item.startsWith("secret."));
    const varsMissing = missing.filter((item) => !item.startsWith("secret."));
    return [
      ...(varsMissing.length ? [{ key: "exp.missingValues" as const, names: quote(varsMissing) }] : []),
      ...(secretsMissing.length ? [{ key: "exp.missingSecrets" as const, names: quote(secretsMissing) }] : []),
    ];
  };
  const missingText = (missing: string[]): string => missingParts(missing).map(({ key, names }) => t(key, { names })).join(" · ");
  /** What to do about them, for the tooltip. */
  const missingTip = (missing: string[]): string => missingParts(missing).map(({ key }) => t(key === "exp.missingValues" ? "exp.missingValuesHint" : "exp.missingSecretsHint")).join("\n");

  /** Send now applies to actions, and Listen now to waits. */
  const canSendNow = (node: ExperimentNode) => !!signalBodyOfNode(node) || isWait(node);

  /** Send one action node on its own, or listen with one wait. The engine performs it with
   * the runner's code, so it does exactly what a run would and secret values never reach
   * the interface. */
  const sendNode = async (node: ExperimentNode) => {
    if (!doc || !canSendNow(node) || sending || busy) return;
    const name = label(node.type);
    setSending(node.id);
    try {
      // Say which names have no value before anything is sent.
      const resolved = await api.experimentResolve(doc, node.id, knownVars);
      if (resolved.missing.length) {
        setTests((prior) => ({ ...prior, [node.id]: { ok: false, text: "", missing: resolved.missing, ts: Date.now() } }));
        // Keys, not finished text: the console re-renders in the next language too.
        for (const { key, names } of missingParts(resolved.missing)) pushLog("warn", "experiment", key, { names });
        return;
      }
      const result = await api.experimentSendNode(doc, node.id, knownVars);
      // A wait's reply, or the values the Extract nodes after a request take, known without a full run.
      const values = result.vars ?? {};
      if (Object.keys(values).length) setKnownVars((prior) => ({ ...prior, ...values }));
      const response = result.response;
      if (response) {
        const text = `HTTP ${response.status} ${response.status_text} · ${response.latency_ms.toFixed(0)} ms · ${fmtBytes(response.body_bytes)}`;
        const shown = response.body + (response.truncated ? `\n${t("http.truncated")}` : "");
        let json: unknown;
        try { json = response.truncated ? undefined : JSON.parse(response.body); } catch { json = undefined; }
        setTests((prior) => ({ ...prior, [node.id]: { ok: response.ok, text, ts: Date.now(), response, body: prettyJson(response.body) ?? shown, json: typeof json === "object" && json !== null ? json : undefined, values } }));
        pushLog(response.ok ? "ok" : "warn", "experiment", "log.nodeSent", { name, detail: text });
      } else {
        setTests((prior) => ({ ...prior, [node.id]: { ok: true, text: result.detail, ts: Date.now(), values } }));
        pushLog("ok", "experiment", "log.nodeSent", { name, detail: result.detail });
      }
    } catch (error) {
      setTests((prior) => ({ ...prior, [node.id]: { ok: false, text: "", error, ts: Date.now() } }));
      pushError("experiment", error, "log.nodeSendFailed", { name });
    } finally { setSending(null); }
  };

  // Resolved preview of the selected node, debounced; the engine does the resolving.
  const previewNode = doc?.nodes.find((node) => node.id === selected) ?? null;
  const previewKey = previewNode && JSON.stringify(previewNode).includes("{{") ? JSON.stringify([previewNode, doc?.params, doc?.profiles, doc?.profile, knownVars, secretsVersion]) : "";
  useEffect(() => {
    if (!doc || !previewNode || !previewKey) { setPreview(null); return; }
    let current = true;
    const timer = window.setTimeout(() => {
      api.experimentResolve(doc, previewNode.id, knownVars)
        .then((resolved) => { if (current) setPreview({ nodeId: previewNode.id, lines: previewLines(resolved.node), missing: resolved.missing }); })
        .catch((error) => { if (current) setPreview({ nodeId: previewNode.id, lines: [], missing: [], error: error ?? "?" }); });
    }, 250);
    return () => { current = false; window.clearTimeout(timer); };
    // previewKey captures the node, parameters, profiles and the known values.
  }, [previewKey]);

  const suggestions = useMemo((): TemplateSuggestion[] => {
    if (!doc || !selected) return [];
    const vars = variablesBefore(doc, selected).flatMap(({ name, node }): TemplateSuggestion[] => {
      const detail = t("exp.setBy", { node: t(NODE_CATALOG[node.type].title) });
      // A reply is an object; its fields are what later steps use.
      return [{ insert: name, group: "vars", detail }, ...replyFields(node).map((field): TemplateSuggestion => ({ insert: `${name}.${field}`, group: "vars", detail }))];
    });
    const params = doc.params.filter((param) => param.name).map((param): TemplateSuggestion => ({ insert: param.name, group: "params", detail: param.value }));
    const secrets = [...new Set([...secretNames(doc), ...storedSecrets])].sort()
      .map((name): TemplateSuggestion => ({ insert: `secret.${name}`, group: "secrets", detail: t("exp.secretSuggest") }));
    const generators = GENERATORS.map((item): TemplateSuggestion => ({ insert: item.insert, group: "generators", detail: t(item.description) }));
    return [...params, ...vars, ...secrets, ...generators];
  }, [doc, selected, storedSecrets, t]);

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

  // Registered once; the listener always calls the handler of the latest render, so a
  // shortcut never acts on state an earlier render captured.
  const keyHandlers = useRef<{ down: (event: KeyboardEvent) => void; up: (event: KeyboardEvent) => void } | null>(null);
  {
    const onKey = (event: KeyboardEvent) => {
      if (!active) return;
      if (event.defaultPrevented || (event.target as HTMLElement).closest('[role="dialog"]')) return;
      if (event.key === "Escape") {
        if (menu) setMenu(null);
        else if (linkStart) cancelLink();
        else if (fullscreen) { setWindowFullscreen(false).then(() => { setFullscreen(false); setFocusMode(focusBeforeFullscreen.current); }).catch(() => {}); }
        else if (focusMode) setFocusMode(false);
        return;
      }
      const target = event.target as HTMLElement;
      if (editable(target)) return;
      // Virtual keyboards can omit the physical code. Keep shortcuts usable there too.
      const key = event.key.toLowerCase();
      const fallbackKey = ({ я: "z", н: "y", в: "d", а: "f", ф: "a" } as Record<string, string>)[key] ?? key;
      const code = event.code && event.code !== "Unidentified" ? event.code
        : /^[a-z]$/.test(fallbackKey) ? `Key${fallbackKey.toUpperCase()}` : /^\d$/.test(key) ? `Digit${key}` : event.code;
      if (event.ctrlKey || event.metaKey) {
        if (code === "KeyZ") { event.preventDefault(); restore(event.shiftKey ? "redo" : "undo"); }
        if (code === "KeyY") { event.preventDefault(); restore("redo"); }
        if (code === "KeyD") { event.preventDefault(); duplicateSelected(); }
        if (code === "KeyF") { event.preventDefault(); setMenu(null); setFinderOpen(true); }
        if (code === "Digit0" && doc) { event.preventDefault(); fit(doc.nodes); }
        if (code === "Digit1") { event.preventDefault(); zoomAt(1); }
        if (event.key === "Enter" && selectedNode) { event.preventDefault(); void sendNode(selectedNode); }
        return;
      }
      if ((event.key === "Delete" || event.key === "Backspace") && selected && !busy) { event.preventDefault(); removeSelected(); }
      if (code === "KeyA" && !menu) { event.preventDefault(); openAddMenu(); }
      if (selected && !busy && target.closest(".experiment-node-body") && ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) {
        const node = doc?.nodes.find((item) => item.id === selected);
        if (!node) return;
        event.preventDefault(); editGroup.current ??= `nudge-${selected}`;
        const step = event.shiftKey ? 20 : 5;
        patchNode(node.id, { x: Math.max(12, node.x + (event.key === "ArrowRight" ? step : event.key === "ArrowLeft" ? -step : 0)),
          y: Math.max(12, node.y + (event.key === "ArrowDown" ? step : event.key === "ArrowUp" ? -step : 0)) });
      }
    };
    const onKeyUp = (event: KeyboardEvent) => { if (event.key.startsWith("Arrow") && editGroup.current?.startsWith("nudge-")) commitEdit(); };
    keyHandlers.current = { down: onKey, up: onKeyUp };
  }
  useEffect(() => {
    const down = (event: KeyboardEvent) => keyHandlers.current?.down(event);
    const up = (event: KeyboardEvent) => keyHandlers.current?.up(event);
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => { window.removeEventListener("keydown", down); window.removeEventListener("keyup", up); };
  }, []);

  /** Run the document; Run with… passes another profile, overrides or a seed for this run only. */
  const run = async (options?: RunOptions) => {
    if (!doc || job || launchPending.current) return;
    launchPending.current = true; setStarting(true);
    try {
      const runDoc = options ? { ...doc, profile: options.profile } : doc;
      try { await api.experimentValidate(runDoc, options?.overrides); }
      catch (error) { revealProblem(error); activeId.current = null; return; }
      await saveDocument(doc);
      setProblem(null); setEvents([]); setOutcome(null); setReportPath(""); cancelLink(); setTimelineOpen(true);
      activeId.current = -1;
      const started = await api.experimentStart(runDoc, options?.overrides, options?.seed);
      if (activeId.current === null) { refreshJobs(); return; }
      if (activeId.current === -1) activeId.current = started.id;
      setJob(started); refreshJobs();
    // Opening a wait's port can still fail here, at the wait that needs it.
    } catch (error) { activeId.current = null; revealProblem(error); }
    finally { launchPending.current = false; setStarting(false); }
  };

  const stop = async () => {
    if (!job) return;
    try {
      await api.jobStop(job.id);
      activeId.current = null; setJob(null); setOutcome({ kind: "stopped" }); refreshJobs();
    } catch (error) { setProblem(error); }
  };

  const toggleFullscreen = async () => {
    try {
      await setWindowFullscreen(!fullscreen);
      if (fullscreen) setFocusMode(focusBeforeFullscreen.current);
      else { focusBeforeFullscreen.current = focusMode; setFocusMode(true); }
      setFullscreen(!fullscreen);
    }
    catch (error) { setProblem(error); }
  };

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

  const menuItems = useMemo((): MenuItem[] => {
    const query = search.trim().toLowerCase();
    // Every word must appear somewhere: "check val" finds "Check value".
    const words = query.split(/\s+/).filter(Boolean);
    const matches = (text: string) => { const lower = text.toLowerCase(); return words.every((word) => lower.includes(word)); };
    const nodes: MenuItem[] = NODE_GROUPS.flatMap((group) => ADDABLE_NODES
      .filter((type) => NODE_CATALOG[type].group === group && matches(`${t(NODE_CATALOG[type].title)} ${t(NODE_CATALOG[type].description)} ${type}`))
      .map((type): MenuItem => ({ kind: "node", type })));
    const signals: MenuItem[] = library
      .filter((signal) => nodeFromSignal(signal.body, 0, 0) && matches(`${signal.name} ${signal.group} ${signal.body.transport} ${signalTarget(signal)}`))
      .map((signal): MenuItem => ({ kind: "signal", signal }));
    return [...nodes, ...signals];
  }, [search, library, t]);

  const addFromMenu = (item: MenuItem) => {
    if (!menu || !doc) return;
    const node = item.kind === "node" ? createNode(item.type, menu.x, menu.y) : nodeFromSignal(item.signal.body, menu.x, menu.y);
    if (!node) return;
    commitEdit();
    edit((current) => (menu.branch ? addBranch : addAfter)(current, menu.anchor ?? null, node));
    setMenu(null);
    inserted(node.id);
  };

  useImperativeHandle(ref, () => ({
    add: (body) => {
      if (!doc || busy) return false;
      // The end of the flow is the predictable place; the selection is only a fallback.
      const end = doc.nodes.find((node) => node.type === "end");
      const anchor = (end && anchorAfter(doc, end.id)) || (selected ? anchorAfter(doc, selected) : null);
      const spot = anchor ? placeAfter(doc, anchor)
        : { x: Math.min(...doc.nodes.map((node) => node.x)), y: Math.max(...doc.nodes.map((node) => node.y)) + NODE_H + 48 };
      const node = nodeFromSignal(body, spot.x, spot.y);
      if (!node) return false;
      commitEdit();
      edit((current) => addAfter(current, anchor, node));
      setSelected(node.id); setPropertiesOpen(true);
      pendingReveal.current = node.id;
      pushLog("ok", "experiment", "log.addedToExperiment", { node: label(node.type), name: doc.name });
      return true;
    },
  }));

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
  /** The node a released wire is meant for when it lands just outside one: within 24 screen pixels. */
  const nodeNear = (clientX: number, clientY: number): string | undefined => {
    if (!doc) return undefined;
    const point = graphPoint(clientX, clientY);
    const reach = 24 / zoom;
    let best: { id: string; distance: number } | undefined;
    for (const node of doc.nodes) {
      const dx = Math.max(node.x - point.x, 0, point.x - (node.x + NODE_W));
      const dy = Math.max(node.y - point.y, 0, point.y - (node.y + NODE_H));
      const distance = Math.hypot(dx, dy);
      if (distance <= reach && (!best || distance < best.distance)) best = { id: node.id, distance };
    }
    return best?.id;
  };

  const selectedNode = doc?.nodes.find((node) => node.id === selected) ?? null;
  const nodeStates = useMemo(() => new Map(events.map((event) => [event.node_id, event.state])), [events]);
  const missing = useMemo(() => doc ? missingOutputs(doc) : new Set<string>(), [doc]);
  const detached = useMemo(() => doc ? unreachableNodes(doc) : new Set<string>(), [doc]);

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

  const openDocument = (document: Experiment) => {
    if (busy) return;
    commitEdit(); replace(document);
    setDocumentsOpen(false); setMenu(null); cancelLink(); setFinderOpen(false);
    setSelected(document.nodes[0]?.id ?? null);
    setEvents([]); setOutcome(null); setReportPath(""); setProblem(null); setTests({}); setKnownVars({}); setLastSeed(null); setLastRun(null);
    requestAnimationFrame(() => fit(document.nodes));
  };

  const documentsDialog = documentsOpen && <ExperimentDocuments document={doc} onOpen={openDocument} onClose={() => setDocumentsOpen(false)} />;

  if (!doc) return <div className="experiment-loading">{documentError !== null ? <ErrorMessage error={documentError} /> : t("exp.loading")}
    {documentError !== null && <button className="ghost" onClick={() => setDocumentsOpen(true)}>{t("exp.documents")}</button>}{documentsDialog}
  </div>;

  const addTitle = selectedNode && anchorAfter(doc, selectedNode.id) ? t("exp.addAfter") : t("exp.addNode");
  const outcomeText = !outcome ? (job ? t("exp.running") : "")
    : outcome.kind === "failed" ? `${t("exp.failed")} · ${describe(outcome.error)}`
    : outcome.kind === "passed" ? t("exp.passed") : t("exp.stopped");
  /** A timeline row: why it failed, or what it did, in the current language. The row already names the node. */
  const stepText = (event: ExperimentStep): string => event.error ? describeError(event.error, t).text
    : event.message_key ? t(event.message_key, messageParams(event.message_params, t)) : event.detail;
  const linkSource = linkStart ? doc.nodes.find((node) => node.id === linkStart.from) : undefined;
  const selectedTest = selectedNode ? tests[selectedNode.id] : undefined;
  let menuCursor = 0;
  const menuButton = (item: MenuItem, group: string, title: string, description: string, glyph: string) => {
    const index = menuCursor++;
    const id = item.kind === "node" ? `catalog-${item.type}` : `catalog-signal-${item.signal.id}`;
    return <button key={id} id={id} className={menuIndex === index ? "highlighted" : ""} data-group={group} data-tip={description} onClick={() => addFromMenu(item)} onMouseEnter={() => setMenuIndex(index)}>
      <span className="experiment-kind">{glyph}</span><span><b>{title}</b></span></button>;
  };

  return <div className={`experiment-view ${propertiesOpen ? "" : "properties-closed"} ${timelineOpen ? "" : "timeline-closed"}`}
    onFocusCapture={(event) => { if (event.target.matches("input, textarea, select")) editGroup.current = `field-${crypto.randomUUID()}`; }}
    onBlurCapture={(event) => { if (event.target.matches("input, textarea, select")) commitEdit(); }}>
    <div className="experiment-toolbar">
      <button className="ghost sm experiment-document-button" data-tip={t("exp.documents")} aria-label={t("exp.documents")} disabled={busy} onClick={() => { setMenu(null); setDocumentsOpen(true); }}>☰</button>
      <div className="experiment-heading"><input aria-label={t("exp.name")} value={doc.name} disabled={busy} onChange={(event) => edit((old) => ({ ...old, name: event.target.value }))} /></div>
      <span className="experiment-save">{saveState === "saving" ? t("exp.saving") : saveState === "error" ? t("exp.saveError") : t("exp.saved")}</span>
      {validationError !== null && <button className="ghost sm experiment-validation" data-tip={describe(validationError)} onClick={() => revealProblem(validationError)}>⚠ {t("exp.needsLinks")}</button>}
      <button className="ghost sm" data-tip={`${addTitle} · A`} onClick={openAddMenu} disabled={busy}>＋ {t("exp.addNode")}</button>
      {doc.profiles.length > 0 && <select className="experiment-profile-select" aria-label={t("exp.profile")} data-tip={t("exp.profileSwitch")} disabled={busy}
        value={doc.profile ?? ""} onChange={(event) => { const profile = event.target.value || null; commitEdit(); edit((current) => ({ ...current, profile })); }}>
        {[null, ...doc.profiles.map((profile) => profile.name)].map((name) => {
          const issue = profileIssues.find((item) => item.profile === name);
          return <option key={name ?? ""} value={name ?? ""}>{name ?? t("exp.noProfile")}{issue ? " ⚠" : ""}</option>;
        })}
      </select>}
      <button className="ghost sm experiment-params-button" aria-pressed={!!paramsAt} aria-label={t("exp.params")} data-tip={t("exp.paramsHint")} onClick={(event) => { const rect = event.currentTarget.getBoundingClientRect(); setMenu(null); setParamsAt({ left: rect.left, top: rect.bottom + 6 }); }}>{"{ }"} <span className="label">{t("exp.params")}</span>{doc.params.length > 0 && <span className="experiment-node-count">{doc.params.length}</span>}</button>
      <button className="ghost sm" aria-pressed={propertiesOpen} data-tip={t("exp.properties")} aria-label={t("exp.properties")} onClick={() => setPropertiesOpen(!propertiesOpen)}>☷</button>
      <button className="ghost sm" aria-pressed={focusMode} data-tip={focusMode ? t("exp.exitFocus") : t("exp.focus")} aria-label={focusMode ? t("exp.exitFocus") : t("exp.focus")} onClick={() => setFocusMode(!focusMode)}>{focusMode ? "▣" : "▢"}</button>
      <button className="ghost sm" aria-pressed={fullscreen} data-tip={fullscreen ? t("exp.exitFullscreen") : t("exp.fullscreen")} aria-label={fullscreen ? t("exp.exitFullscreen") : t("exp.fullscreen")} onClick={toggleFullscreen}>⛶</button>
      <div className="experiment-run-group">
        <button className={job ? "danger" : "primary"} disabled={starting} onClick={() => job ? stop() : run()}>{job ? t("common.stop") : t("exp.run")}</button>
        {!job && <button className="primary experiment-run-more" disabled={starting} aria-label={t("exp.runMenu")} data-tip={t("exp.runWith")}
          onClick={(event) => { const rect = event.currentTarget.getBoundingClientRect(); setMenu(null); setRunWithAt({ right: rect.right, top: rect.bottom + 6 }); }}>▾</button>}
      </div>
    </div>
    {(problem ?? documentError) !== null && <ErrorMessage className="experiment-problem" error={problem ?? documentError} nodeLabel={nodeName}
      onShow={(id) => { const node = doc.nodes.find((item) => item.id === id); if (node) showNode(node); }} />}
    <div className="experiment-workspace">
      <div className="experiment-left">
        <div className="experiment-canvas-tools">
          <div className="experiment-tool-group" role="group" aria-label={t("exp.history")}>
            <button className="ghost sm" data-tip={`${t("exp.undo")} · Ctrl+Z`} aria-label={t("exp.undo")} disabled={busy || !history.past.length} onClick={() => restore("undo")}>↶</button>
            <button className="ghost sm" data-tip={`${t("exp.redo")} · Ctrl+Shift+Z`} aria-label={t("exp.redo")} disabled={busy || !history.future.length} onClick={() => restore("redo")}>↷</button>
          </div>
          <button className="ghost sm" data-tip={`${t("exp.findNode")} · Ctrl+F`} onClick={() => setFinderOpen(true)}>{t("exp.nodes")} <span className="experiment-node-count">{doc.nodes.length}</span></button>
          {linkStart && <span className="experiment-canvas-hint linking" role="status">{t("exp.chooseInput")}</span>}
          {linkStart && <button className="ghost sm" onClick={cancelLink}>{t("exp.cancelLink")}</button>}
          <div className="fill" />
          <button className="ghost sm experiment-arrange" onClick={arrange} disabled={busy}>{t("exp.arrange")}</button>
          <button className="ghost sm" aria-label={t("exp.zoomOut")} data-tip={t("exp.zoomOut")} onClick={() => zoomAt(zoom - .1)}>−</button>
          <span className="experiment-zoom" data-tip={t("exp.zoomHint")}>{Math.round(zoom * 100)}%</span>
          <button className="ghost sm" aria-label={t("exp.zoomIn")} data-tip={t("exp.zoomIn")} onClick={() => zoomAt(zoom + .1)}>＋</button>
          <button className="ghost sm" data-tip={`${t("exp.resetZoom")} · Ctrl+1`} onClick={() => zoomAt(1)}>1:1</button>
          <button className="ghost sm" data-tip={`${t("exp.fit")} · Ctrl+0`} aria-label={t("exp.fit")} onClick={() => fit(doc.nodes)}>⊡</button>
        </div>
        <div className={`experiment-canvas-scroll ${linkStart ? "linking" : ""}`} ref={scrollRef}>
          <div className="experiment-canvas-space" style={{ width: canvasWidth * zoom, height: canvasHeight * zoom }}>
            <div className="experiment-canvas" style={{ width: canvasWidth, height: canvasHeight, transform: `scale(${zoom})`, "--zoom": zoom } as CSSProperties}
              onDoubleClick={(event) => { if (event.target !== event.currentTarget) return; const point = graphPoint(event.clientX, event.clientY); openMenu(event.clientX, event.clientY, point.x - NODE_W / 2, point.y - NODE_H / 2, linkStart ?? undefined, !!linkStart); }}
              onPointerDown={(event) => { if (event.target !== event.currentTarget) return; const scroll = scrollRef.current!; pan.current = { clientX: event.clientX, clientY: event.clientY, left: scroll.scrollLeft, top: scroll.scrollTop, moved: false }; event.currentTarget.setPointerCapture(event.pointerId); if (!linkStart) setSelected(null); }}
              onPointerMove={(event) => {
                if (linkStart && !portDrag.current) setLinkPoint(graphPoint(event.clientX, event.clientY));
                const p = pan.current; if (!p) return;
                if (!p.moved && Math.hypot(event.clientX - p.clientX, event.clientY - p.clientY) < 4) return;
                p.moved = true; const scroll = scrollRef.current!; scroll.scrollLeft = p.left - (event.clientX - p.clientX); scroll.scrollTop = p.top - (event.clientY - p.clientY);
              }}
              onPointerUp={(event) => {
                const p = pan.current; pan.current = null;
                // Click-to-link, then a click on empty canvas: the next node goes right there.
                if (p && !p.moved && linkStart) { const point = graphPoint(event.clientX, event.clientY); openMenu(event.clientX, event.clientY, point.x, point.y - NODE_H / 2, linkStart, true); }
              }} onPointerCancel={() => { pan.current = null; }}>
              <svg className="experiment-wires" width={canvasWidth} height={canvasHeight} aria-hidden="true">
                {doc.edges.map((edge) => { const a = doc.nodes.find((node) => node.id === edge.from); const b = doc.nodes.find((node) => node.id === edge.to); if (!a || !b) return null;
                  const p = portOf(edge);
                  const x1 = a.x + NODE_W, y1 = a.y + portY(a, p);
                  const x2 = b.x, y2 = b.y + NODE_H / 2;
                  const isRunning = nodeStates.get(a.id) === "running" || nodeStates.get(b.id) === "running";
                  const isPassed = nodeStates.get(a.id) === "passed" && (nodeStates.get(b.id) === "passed" || nodeStates.get(b.id) === "running");
                  return <path key={`${edge.from}-${p}-${edge.to}`} className={`wire ${p} ${isRunning ? "active-flow" : ""} ${isPassed ? "passed-flow" : ""}`} d={`M ${x1} ${y1} C ${x1 + 75} ${y1}, ${x2 - 75} ${y2}, ${x2} ${y2}`} />;
                })}
                {linkSource && linkPoint && (() => { const x1 = linkSource.x + NODE_W, y1 = linkSource.y + portY(linkSource, linkStart!.port), { x: x2, y: y2 } = linkPoint;
                  return <path className="wire preview" d={`M ${x1} ${y1} C ${x1 + 75} ${y1}, ${x2 - 75} ${y2}, ${x2} ${y2}`} />; })()}
              </svg>
              {doc.edges.map((edge) => { const a = doc.nodes.find((node) => node.id === edge.from); const b = doc.nodes.find((node) => node.id === edge.to); if (!a || !b) return null;
                const p = portOf(edge);
                const y1 = a.y + portY(a, p); const x = (a.x + NODE_W + b.x) / 2; const y = (y1 + b.y + NODE_H / 2) / 2;
                return <button key={`${edge.from}-${p}-${edge.to}`} className="experiment-edge-add" style={{ left: x - 11, top: y - 11 }} data-tip={t("exp.insertNode")} aria-label={`${t("exp.insertNode")}: ${label(a.type)} → ${label(b.type)}`} disabled={busy} onClick={(event) => openMenu(event.clientX, event.clientY, Math.max(x - NODE_W / 2, a.x + STEP_X), y - NODE_H / 2, { from: edge.from, port: p, to: edge.to })}>＋</button>;
              })}
              {doc.nodes.map((node) => <div key={node.id} data-node={node.id} data-group={NODE_CATALOG[node.type].group}
                className={`experiment-node ${selected === node.id ? "selected" : ""} ${nodeStates.get(node.id) ?? ""} ${detached.has(node.id) ? "detached" : ""} ${problemNodeId === node.id ? "invalid" : ""} ${linkStart && linkStart.from !== node.id ? "link-target" : ""}`}
                style={{ left: node.x, top: node.y, width: NODE_W, height: NODE_H }}>
                <button className="experiment-node-body" data-node-id={node.id} data-tip={`${label(node.type)} — ${nodeSummary(node)}${detached.has(node.id) ? `\n${t("exp.detachedHint")}` : ""}`}
                  onPointerDown={(event) => onNodeDown(event, node)} onPointerMove={(event) => onNodeMove(event, node)} onPointerUp={endDrag} onPointerCancel={endDrag}
                  onFocus={() => { if (!linkStart) setSelected(node.id); }} onClick={() => { if (linkStart) endLink(node.id); else setSelected(node.id); }}
                  onDoubleClick={() => { setPropertiesOpen(true); focusFieldOf.current = node.id; setSelected(node.id); }}
                  aria-label={`${label(node.type)}: ${nodeSummary(node)}`}>
                  <div className="experiment-node-header">
                    <span className="experiment-node-type">{label(node.type)}</span>
                    {nodeStates.has(node.id) && <span className={`node-status-badge ${nodeStates.get(node.id)}`} aria-label={t(`exp.${nodeStates.get(node.id)}`)}>
                      {nodeStates.get(node.id) === "running" ? "●" : nodeStates.get(node.id) === "passed" ? "✓" : "✕"}
                    </span>}
                  </div>
                  <span className="experiment-node-subtitle">{nodeSummary(node)}</span>
                </button>
                {node.type !== "start" && <button className={`experiment-port input ${linkStart ? "awaiting" : ""}`} data-tip={t("exp.inputPort")} aria-label={`${label(node.type)}: ${t("exp.inputPort")}`} onClick={() => endLink(node.id)} />}
                {outputPorts(node.type).map((port) => <button key={port}
                  className={`experiment-port output ${port} ${missing.has(`${node.id}:${port}`) ? "missing" : ""} ${linkStart?.from === node.id && linkStart.port === port ? "active" : ""}`}
                  data-tip={`${portLabel(port)} — ${t("exp.portHint")}`} aria-label={`${label(node.type)}: ${portLabel(port)}`}
                  onPointerDown={(event) => onPortDown(event, { from: node.id, port })} onPointerMove={onPortMove} onPointerUp={onPortUp} onPointerCancel={onPortCancel}
                  onClick={() => { if (suppressPortClick.current) { suppressPortClick.current = false; return; } toggleLink(node.id, port); }} />)}
              </div>)}
            </div>
          </div>
        </div>
      </div>
      {propertiesOpen && <aside className="experiment-properties" ref={propertiesRef}
        onKeyDown={(event) => {
          if (event.key === "Enter" && (event.ctrlKey || event.metaKey) && selectedNode) { event.preventDefault(); void sendNode(selectedNode); }
          // Escape leaves the form for the canvas, where A adds the next node.
          if (event.key === "Escape" && selectedNode && editable(event.target)) { event.preventDefault(); focusNode(selectedNode.id); }
        }}>
        <div className="section-label">{t("exp.properties")}</div>
        {selectedNode ? <><h2 data-tip={t(nodeHelp(selectedNode.type))}>{label(selectedNode.type)}</h2>
          {/* What stops this node from running, next to the fields it is about. */}
          {problemNodeId === selectedNode.id && <ErrorMessage className="experiment-node-problem" error={validationError} />}
          <fieldset disabled={busy}>
          <TemplateSuggestions.Provider value={suggestions}>
            <ExperimentNodeFields node={selectedNode} patch={(change) => patchNode(selectedNode.id, change)} />
          </TemplateSuggestions.Provider>
          {preview?.nodeId === selectedNode.id && <div className={`experiment-preview ${preview.error ? "error" : ""}`} aria-live="polite">
            <p className="section-label">{t(isWait(selectedNode) ? "exp.previewWait" : "exp.preview")}</p>
            {preview.error !== undefined ? <ErrorMessage error={preview.error} /> : preview.lines.map((line, index) => <code key={index}>{line}</code>)}
            {preview.missing.length > 0 && <p className="experiment-preview-missing" data-tip={missingTip(preview.missing)}>{missingText(preview.missing)}</p>}
          </div>}
          {canSendNow(selectedNode) && <div className="experiment-node-test">
            <div className="experiment-node-test-row">
              <button className="ghost sm" disabled={!!sending} onClick={() => sendNode(selectedNode)} data-tip={`${t(isWait(selectedNode) ? "exp.listenNowHint" : "exp.sendNowHint")} · Ctrl+Enter`}>
                {sending === selectedNode.id ? t(isWait(selectedNode) ? "exp.listening" : "common.sending") : `▶ ${t(isWait(selectedNode) ? "exp.listenNow" : "exp.sendNow")}`}</button>
            </div>
            {selectedTest && <div className={`experiment-test-result ${selectedTest.ok ? "ok" : "fail"}`} role="status">
              {selectedTest.error !== undefined ? <ErrorMessage error={selectedTest.error} />
                : <span data-tip={selectedTest.missing ? missingTip(selectedTest.missing) : undefined}>{selectedTest.ok ? "✓" : "✕"} {selectedTest.missing ? missingText(selectedTest.missing) : selectedTest.text}</span>}
              <time>{fmtTime(selectedTest.ts).slice(0, 8)}</time></div>}
            {selectedTest?.values && Object.keys(selectedTest.values).length > 0 && <p className="experiment-test-values">{t("exp.extractedValues", { values: Object.entries(selectedTest.values).map(([name, value]) => `${name} = ${typeof value === "string" ? value : JSON.stringify(value)}`).join(" · ") })}</p>}
            {selectedTest?.response && selectedTest.body && <details className="experiment-test-body" open={!!selectedTest.json}><summary>{t("http.response")} · {fmtBytes(selectedTest.response.body_bytes)}</summary>
              {selectedTest.json !== undefined ? <JsonPicker value={selectedTest.json} tip={t("exp.extractHere")} onPick={(path, value) => extractPicked(selectedNode, path, value)} /> : <pre>{selectedTest.body}</pre>}</details>}
          </div>}
          {doc.edges.filter((edge) => edge.from === selectedNode.id).map((edge) => <div className="experiment-connection" key={`${portOf(edge)}-${edge.to}`}><span>{portLabel(portOf(edge))} → {label(doc.nodes.find((node) => node.id === edge.to)?.type ?? "end")}</span><button className="ghost sm" data-tip={t("exp.disconnect")} aria-label={t("exp.disconnect")} onClick={() => edit((current) => disconnect(current, edge.from, portOf(edge), edge.to))}>×</button></div>)}
          {selectedNode.type !== "end" && <div className="experiment-node-actions"><button className="ghost sm" data-tip={`${t("exp.addAfter")} · A`} onClick={openAddMenu}>＋ {t("exp.addNext")}</button>
            {selectedNode.type !== "start" && <><button className="ghost sm" data-tip={`${t("exp.duplicate")} · Ctrl+D`} onClick={duplicateSelected}>{t("exp.duplicate")}</button><button className="ghost sm experiment-delete" data-tip={`${t("exp.delete")} · Delete`} onClick={removeSelected}>{t("exp.delete")}</button></>}</div>}
        </fieldset></> : <p className="experiment-empty">{t("exp.selectNode")}</p>}
      </aside>}
    </div>
    <div className="experiment-timeline"><div className="experiment-timeline-title"><button className="ghost sm" onClick={() => setTimelineOpen(!timelineOpen)} aria-expanded={timelineOpen}>{timelineOpen ? "▾" : "▸"} {t("exp.timeline")}{!timelineOpen && events.length > 0 && <span className="experiment-node-count">{events.length}</span>}</button>{reportPath && (downloadUrl(reportPath)
        ? <a className="experiment-report download-link" href={downloadUrl(reportPath)!} download data-tip={reportPath}>{t("exp.reportSaved")} ↓</a>
        : <span className="experiment-report" data-tip={reportPath}>{t("exp.reportSaved")}</span>)}{lastRun && (lastRun.overridden || doc.profiles.length > 0) && <span className="experiment-run-profile">
      {lastRun.profile ? t("exp.runProfile", { name: lastRun.profile }) : t("exp.runDefaults")}{lastRun.overridden && ` · ${t("exp.overridden")}`}</span>}{doc.seed !== null
      ? <button className="ghost sm experiment-seed-chip pinned" data-tip={t("exp.unpinSeedHint")} onClick={() => edit((current) => ({ ...current, seed: null }))}>{t("exp.runSeed", { seed: doc.seed })} · {t("exp.unpinSeed")}</button>
      : lastSeed !== null && <button className="ghost sm experiment-seed-chip" data-tip={t("exp.pinSeedHint")} disabled={busy} onClick={() => edit((current) => ({ ...current, seed: lastSeed }))}>{t("exp.runSeed", { seed: lastSeed })} · {t("exp.pinSeed")}</button>}<strong className={outcome?.kind === "failed" ? "fail" : ""} data-tip={outcome?.kind === "failed" ? describe(outcome.error) : undefined}>{outcomeText}</strong></div>
      {timelineOpen && <div className="experiment-events">{events.length === 0 ? <span className="experiment-empty">{t("exp.noEvents")}</span> : events.map((event, index) => <button key={index} onClick={() => { const node = doc.nodes.find((item) => item.id === event.node_id); if (node) showNode(node); }} className={event.state}><time>{new Date(event.ts).toLocaleTimeString()}</time><b>{label(doc.nodes.find((node) => node.id === event.node_id)?.type ?? "end")}</b><span data-tip={event.error?.detail}>{t(`exp.${event.state}`)}{stepText(event) && ` · ${stepText(event)}`}</span></button>)}</div>}
    </div>
    {menu && <div className="experiment-menu-backdrop" onPointerDown={() => setMenu(null)}><div className="experiment-add-menu" role="dialog" aria-label={t("exp.addNode")} style={{ left: Math.max(8, menu.screenX), top: Math.max(8, menu.screenY) }} onPointerDown={(event) => event.stopPropagation()}>
      {menu.anchor && <p className="experiment-menu-context">{t(menu.branch ? "exp.addingBranch" : "exp.addingAfter", { node: anchorLabel(menu.anchor) })}</p>}
      <input autoFocus value={search} onChange={(event) => { setSearch(event.target.value); setMenuIndex(0); }} placeholder={t("exp.searchNodes")} aria-label={t("exp.searchNodes")} data-tip={t("exp.menuHint")} aria-controls="experiment-catalog" onKeyDown={(event) => {
        if (event.key === "Escape") { event.preventDefault(); setMenu(null); return; }
        if (event.key === "Enter" && menuItems[menuIndex]) { event.preventDefault(); addFromMenu(menuItems[menuIndex]); }
        if (["ArrowDown", "ArrowUp"].includes(event.key) && menuItems.length) { event.preventDefault(); const next = (menuIndex + (event.key === "ArrowDown" ? 1 : -1) + menuItems.length) % menuItems.length; setMenuIndex(next); const item = menuItems[next]; document.getElementById(item.kind === "node" ? `catalog-${item.type}` : `catalog-signal-${item.signal.id}`)?.scrollIntoView({ block: "nearest" }); } }} />
      <div id="experiment-catalog" className="experiment-catalog-results">{NODE_GROUPS.map(group => {
        const items = menuItems.filter((item) => item.kind === "node" && NODE_CATALOG[item.type].group === group);
        return items.length ? <section key={group}><h3>{t(`exp.group.${group}`)}</h3>{items.map((item) => item.kind === "node" && menuButton(item, group, label(item.type), t(NODE_CATALOG[item.type].description), GROUP_GLYPH[group]))}</section> : null;
      })}
        {menuItems.some((item) => item.kind === "signal") && <section><h3>{t("exp.group.signals")}</h3>{menuItems.map((item) => item.kind === "signal" && menuButton(item, "action", item.signal.name, `${t(transportKey(item.signal.body.transport))} · ${signalTarget(item.signal)}`, "❖"))}</section>}
        {!menuItems.length && <p className="experiment-empty">{t("exp.noMatches")}</p>}</div>
    </div></div>}
    {paramsAt && <ExperimentParams doc={doc} issues={profileIssues} disabled={busy} anchor={paramsAt} describe={describe}
      onClose={() => { commitEdit(); setParamsAt(null); }} onEdit={(update) => edit(update)}
      onSecretsChanged={(name) => { if (name) setStoredSecrets((prior) => prior.includes(name) ? prior : [...prior, name]); setSecretsVersion((v) => v + 1); revalidate(); }} />}
    {runWithAt && <ExperimentRunWith doc={doc} issues={profileIssues} lastSeed={lastSeed} initial={runWithLast} anchor={runWithAt}
      onClose={() => setRunWithAt(null)} onRun={(options) => { setRunWithLast(options); setRunWithAt(null); void run(options); }} />}
    {finderOpen && <ExperimentFinder nodes={doc.nodes} label={label} summary={nodeSummary} onSelect={showNode} onClose={() => setFinderOpen(false)} />}
    {documentsDialog}
  </div>;
}
