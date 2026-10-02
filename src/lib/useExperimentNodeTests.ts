import { useEffect, useState, type Dispatch, type SetStateAction } from "react";
import { api, type Experiment, type ExperimentNode, type HttpResponse } from "./api";
import type { Failure } from "./errors";
import { isWait, missingParts, nodeLabel, previewLines } from "./experimentText";
import { fmtBytes, fmtNum, prettyJson } from "./format";
import { useT } from "./i18n";
import { signalBodyOfNode } from "./signals";
import { useStore } from "./store";

/** What the last Send now of a node did. A failure is kept as it came and described when shown. */
export type NodeTest = { ok: boolean; text: string; ts: number; error?: Failure; missing?: string[]; response?: HttpResponse; body?: string; json?: unknown; values?: Record<string, unknown> };
export type Preview = { nodeId: string; lines: string[]; missing: string[]; error?: Failure };

/** Send now applies to actions, and Listen now to waits. */
export const canSendNow = (node: ExperimentNode) => !!signalBodyOfNode(node) || isWait(node);

/**
 * One node on its own: Send now / Listen now, and the resolved preview of the
 * selected node. The engine does both with the runner's code, so a node does
 * exactly what a run would and secret values never reach the interface.
 */
export function useExperimentNodeTests({ doc, selectedNode, busy, knownVars, setKnownVars, secretsVersion }: {
  doc: Experiment | null;
  selectedNode: ExperimentNode | null;
  busy: boolean;
  knownVars: Record<string, unknown>;
  setKnownVars: Dispatch<SetStateAction<Record<string, unknown>>>;
  /** Changes when the secret store did, so the preview resolves again. */
  secretsVersion: number;
}) {
  const t = useT();
  const { pushLog, pushError } = useStore();
  const [tests, setTests] = useState<Record<string, NodeTest>>({});
  const [sending, setSending] = useState<string | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);

  /** Send one action node on its own, or listen with one wait. */
  const sendNode = async (node: ExperimentNode) => {
    if (!doc || !canSendNow(node) || sending || busy) return;
    const name = nodeLabel(node.type, t);
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
        const text = `HTTP ${response.status} ${response.status_text} · ${fmtNum(response.latency_ms)} ${t("unit.ms")} · ${fmtBytes(response.body_bytes)}`;
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
  const previewKey = selectedNode && JSON.stringify(selectedNode).includes("{{") ? JSON.stringify([selectedNode, doc?.params, doc?.profiles, doc?.profile, knownVars, secretsVersion]) : "";
  useEffect(() => {
    if (!doc || !selectedNode || !previewKey) { setPreview(null); return; }
    let current = true;
    const timer = window.setTimeout(() => {
      api.experimentResolve(doc, selectedNode.id, knownVars)
        .then((resolved) => { if (current) setPreview({ nodeId: selectedNode.id, lines: previewLines(resolved.node), missing: resolved.missing }); })
        .catch((error) => { if (current) setPreview({ nodeId: selectedNode.id, lines: [], missing: [], error: error ?? "?" }); });
    }, 250);
    return () => { current = false; window.clearTimeout(timer); };
    // previewKey captures the node, parameters, profiles and the known values.
  }, [previewKey]);

  /** Another document is open: its nodes have not been sent. */
  const reset = () => setTests({});

  return { tests, sending, preview, sendNode, reset };
}
