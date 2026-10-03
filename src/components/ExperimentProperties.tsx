import { useMemo, type RefObject } from "react";
import type { Experiment, ExperimentNode, LoadMetrics } from "../lib/api";
import { NODE_CATALOG } from "../lib/experimentCatalog";
import { canLoad, canRepeat, canRetry, GENERATORS, replyFields, secretNames, variablesBefore } from "../lib/experimentData";
import { disconnect, portOf, routeThroughImpairment } from "../lib/experimentGraph";
import { nodeHelp, nodeLabel, portLabel, wireName } from "../lib/experimentText";
import type { Wire } from "../lib/experimentWires";
import type { Failure } from "../lib/errors";
import { canSendNow, type NodeTest, type Preview } from "../lib/useExperimentNodeTests";
import { editable } from "../lib/useExperimentShortcuts";
import { useT } from "../lib/i18n";
import { ErrorMessage } from "./ErrorMessage";
import { LoadFields, LoadResult } from "./ExperimentLoad";
import { ExperimentNodeFields } from "./ExperimentNodeFields";
import { RepeatFields, RetryFields } from "./ExperimentNodeOptions";
import { ExperimentNodePreview, ExperimentNodeTest } from "./ExperimentNodeTest";
import { TemplateSuggestions, type TemplateSuggestion } from "./TemplateField";

/**
 * The properties pane: the selected node's fields, its preview, Send now,
 * its wires and actions — or, with several selected, what can be done to them
 * together — or the selected wire.
 */
export function ExperimentProperties({ panelRef, doc, node, count, wire, busy, problemNodeId, validationError, storedSecrets, preview, test, sending, measured,
  onSend, onExtract, onShowEmulator, onPatch, onEdit, onAddNext, onCopy, onDuplicate, onDelete, onRemoveWire, onLeave }: {
  /** The pane, where a new node's first field is found and focused. */
  panelRef: RefObject<HTMLElement | null>;
  doc: Experiment;
  /** The last node selected. */
  node: ExperimentNode | null;
  /** How many nodes are selected. */
  count: number;
  wire: Wire | null;
  busy: boolean;
  /** The node the validation failure is about, if any. */
  problemNodeId: string | null;
  validationError: Failure | null;
  /** Secret names stored this session, offered with the document's own. */
  storedSecrets: string[];
  preview: Preview | null;
  test: NodeTest | undefined;
  sending: string | null;
  /** What the last run's load on this node measured. */
  measured: LoadMetrics | undefined;
  onSend: (node: ExperimentNode) => void;
  onExtract: (node: ExperimentNode, path: (string | number)[], value: unknown) => void;
  /** Opens an emulator a Send now response was mocked into. */
  onShowEmulator?: (id: string) => void;
  onPatch: (id: string, change: Partial<ExperimentNode>) => void;
  onEdit: (update: (current: Experiment) => Experiment) => void;
  onAddNext: () => void;
  /** The selection to the clipboard, to paste here or into another experiment. */
  onCopy: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
  /** Removes the selected wire. */
  onRemoveWire: () => void;
  /** Back from the form to the node on the canvas. */
  onLeave: (id: string) => void;
}) {
  const t = useT();
  const selected = node?.id ?? null;
  const suggestions = useMemo((): TemplateSuggestion[] => {
    if (!selected) return [];
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
  const label = (type: ExperimentNode["type"]) => nodeLabel(type, t);

  return <aside className="experiment-properties" ref={panelRef}
    onKeyDown={(event) => {
      if (event.key === "Enter" && (event.ctrlKey || event.metaKey) && node) { event.preventDefault(); void onSend(node); }
      // Escape leaves the form for the canvas, where A adds the next node.
      if (event.key === "Escape" && node && editable(event.target)) { event.preventDefault(); onLeave(node.id); }
    }}>
    <div className="section-label">{t("exp.properties")}</div>
    {count > 1 ? <>
      <h2 data-tip={t("exp.selectionHint")}>{t("exp.selectedCount", { n: count })}</h2>
      <div className="experiment-node-actions">
        <button className="ghost sm" data-tip={`${t("exp.copy")} · Ctrl+C`} onClick={onCopy}>{t("exp.copy")}</button>
        <button className="ghost sm" data-tip={`${t("exp.duplicate")} · Ctrl+D`} disabled={busy} onClick={onDuplicate}>{t("exp.duplicate")}</button>
        <button className="ghost sm experiment-delete" data-tip={`${t("exp.deleteSelected")} · Delete`} disabled={busy} onClick={onDelete}>{t("exp.deleteSelected")}</button>
      </div>
    </> : node ? <><h2 data-tip={t(nodeHelp(node.type))}>{label(node.type)}</h2>
      {/* What stops this node from running, next to the fields it is about. */}
      {problemNodeId === node.id && <ErrorMessage className="experiment-node-problem" error={validationError} />}
      <fieldset disabled={busy}>
      <TemplateSuggestions.Provider value={suggestions}>
        <ExperimentNodeFields node={node} doc={doc} patch={(change) => onPatch(node.id, change)} />
        {/* Load replaces Repeat and Retry: a failed request is counted, not tried again. */}
        {canLoad(node) && <LoadFields load={node.load} patch={(change) => onPatch(node.id, change)} />}
        {canRepeat(node) && !node.load && <RepeatFields repeat={node.repeat} patch={(change) => onPatch(node.id, change)} />}
        {canRetry(node) && !node.load && <RetryFields retry={node.retry} patch={(change) => onPatch(node.id, change)} />}
      </TemplateSuggestions.Provider>
      {measured && <LoadResult metrics={measured} />}
      {preview?.nodeId === node.id && <ExperimentNodePreview node={node} preview={preview} />}
      {canSendNow(node) && <ExperimentNodeTest node={node} test={test} sending={sending} onSend={onSend} onExtract={onExtract} onShowEmulator={onShowEmulator} />}
      {doc.edges.filter((edge) => edge.from === node.id).map((edge) => <div className="experiment-connection" key={`${portOf(edge)}-${edge.to}`}><span>{portLabel(portOf(edge), t)} → {label(doc.nodes.find((item) => item.id === edge.to)?.type ?? "end")}</span><button className="ghost sm" data-tip={t("exp.disconnect")} aria-label={t("exp.disconnect")} onClick={() => onEdit((current) => disconnect(current, edge.from, portOf(edge), edge.to))}>×</button></div>)}
      {(node.type === "osc" || node.type === "udp") && <div className="experiment-node-actions">
        <button className="ghost sm" data-tip={t("exp.routeThroughHint")} onClick={() => onEdit((current) => routeThroughImpairment(current, node.id)?.doc ?? current)}>⚡ {t("exp.routeThrough")}</button>
      </div>}
      {node.type !== "end" && <div className="experiment-node-actions"><button className="ghost sm" data-tip={`${t("exp.addAfter")} · A`} onClick={onAddNext}>＋ {t("exp.addNext")}</button>
        {node.type !== "start" && <><button className="ghost sm" data-tip={`${t("exp.copy")} · Ctrl+C`} onClick={onCopy}>{t("exp.copy")}</button><button className="ghost sm" data-tip={`${t("exp.duplicate")} · Ctrl+D`} onClick={onDuplicate}>{t("exp.duplicate")}</button><button className="ghost sm experiment-delete" data-tip={`${t("exp.delete")} · Delete`} onClick={onDelete}>{t("exp.delete")}</button></>}</div>}
    </fieldset></> : wire ? <>
      <h2>{t("exp.wire")}</h2>
      <p className="experiment-wire-name">{wireName(doc.nodes, wire, t)}</p>
      <div className="experiment-node-actions"><button className="ghost sm experiment-delete" data-tip={`${t("exp.removeWire")} · Delete`} disabled={busy} onClick={() => onRemoveWire()}>{t("exp.removeWire")}</button></div>
    </> : <p className="experiment-empty" data-tip={t("exp.selectionHint")}>{t("exp.selectNode")}</p>}
  </aside>;
}
