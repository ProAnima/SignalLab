import type { ExperimentNode } from "../lib/api";
import { isWait, missingText, missingTip } from "../lib/experimentText";
import type { NodeTest, Preview } from "../lib/useExperimentNodeTests";
import { fmtBytes, fmtTime } from "../lib/format";
import { useT } from "../lib/i18n";
import { ErrorMessage } from "./ErrorMessage";
import { JsonPicker } from "./JsonPicker";
import { MockThis } from "./MockThis";

/** What the selected node will send, wait for or compare, with its templates resolved by the engine. */
export function ExperimentNodePreview({ node, preview }: { node: ExperimentNode; preview: Preview }) {
  const t = useT();
  return <div className={`experiment-preview ${preview.error ? "error" : ""}`} aria-live="polite">
    <p className="section-label">{t(isWait(node) ? "exp.previewWait" : ["assert_value", "branch_value", "loop"].includes(node.type) ? "exp.previewCheck" : "exp.preview")}</p>
    {preview.error !== undefined ? <ErrorMessage error={preview.error} /> : preview.lines.map((line, index) => <code key={index}>{line}</code>)}
    {preview.missing.length > 0 && <p className="experiment-preview-missing" data-tip={missingTip(preview.missing, t)}>{missingText(preview.missing, t)}</p>}
  </div>;
}

/**
 * Send now (Listen now for a wait) and what it did last: the result, the
 * values it set and an HTTP response, whose values can become Extract nodes.
 */
export function ExperimentNodeTest({ node, test, sending, onSend, onExtract, onShowEmulator }: {
  node: ExperimentNode;
  test: NodeTest | undefined;
  /** The node being sent right now, if any. */
  sending: string | null;
  onSend: (node: ExperimentNode) => void;
  /** A value picked in the response: extract it after this node. */
  onExtract: (node: ExperimentNode, path: (string | number)[], value: unknown) => void;
  /** An emulator a response was mocked into, to open it. */
  onShowEmulator?: (id: string) => void;
}) {
  const t = useT();
  return <div className="experiment-node-test">
    <div className="experiment-node-test-row">
      <button className="ghost sm" disabled={!!sending} onClick={() => onSend(node)} data-tip={`${t(isWait(node) ? "exp.listenNowHint" : "exp.sendNowHint")} · Ctrl+Enter`}>
        {sending === node.id ? t(isWait(node) ? "exp.listening" : "common.sending") : `▶ ${t(isWait(node) ? "exp.listenNow" : "exp.sendNow")}`}</button>
    </div>
    {test && <div className={`experiment-test-result ${test.ok ? "ok" : "fail"}`} role="status">
      {test.error !== undefined ? <ErrorMessage error={test.error} />
        : <span data-tip={test.missing ? missingTip(test.missing, t) : undefined}>{test.ok ? "✓" : "✕"} {test.missing ? missingText(test.missing, t) : test.text}</span>}
      <time>{fmtTime(test.ts).slice(0, 8)}</time></div>}
    {node.type === "http" && test?.response && !test.response.error && <div className="experiment-node-test-row">
      <MockThis method={node.request.method} url={node.request.url} response={test.response} onShow={onShowEmulator} />
    </div>}
    {test?.values && Object.keys(test.values).length > 0 && <p className="experiment-test-values">{t("exp.extractedValues", { values: Object.entries(test.values).map(([name, value]) => `${name} = ${typeof value === "string" ? value : JSON.stringify(value)}`).join(" · ") })}</p>}
    {test?.response && test.body && <details className="experiment-test-body" open={!!test.json}><summary>{t("http.response")} · {fmtBytes(test.response.body_bytes)}</summary>
      {test.json !== undefined ? <JsonPicker value={test.json} tip={t("exp.extractHere")} onPick={(path, value) => onExtract(node, path, value)} /> : <pre>{test.body}</pre>}</details>}
  </div>;
}
