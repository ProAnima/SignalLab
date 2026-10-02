import type { Experiment, ExperimentNode } from "../lib/api";
import { useT } from "../lib/i18n";
import { TemplateField } from "./TemplateField";
import { protocolsOf, protocolsText } from "../lib/websocket";

type WsNode = Extract<ExperimentNode, { type: "ws_connect" | "ws_send" | "wait_ws" | "ws_close" }>;
type ConnectNode = Extract<ExperimentNode, { type: "ws_connect" }>;


/** Which connection a send, wait or close uses: one of the document's WebSocket connect nodes. */
function ConnectionPicker({ node, doc, patch }: { node: Exclude<WsNode, ConnectNode>; doc: Experiment; patch: (change: Partial<ExperimentNode>) => void }) {
  const t = useT();
  const connects = doc.nodes.filter((candidate): candidate is ConnectNode => candidate.type === "ws_connect");
  if (connects.length === 0) return <p className="experiment-empty">{t("exp.noConnections")}</p>;
  return <label data-tip={t("exp.wsConnectionHint")}>{t("exp.wsConnection")}<select value={node.connection} onChange={event => patch({ connection: event.target.value })}>
    {!connects.some(connect => connect.id === node.connection) && <option value={node.connection}>—</option>}
    {connects.map(connect => <option key={connect.id} value={connect.id}>{connect.url}</option>)}
  </select></label>;
}

/**
 * The fields of the WebSocket nodes: where a connect goes and with what
 * upgrade; what a send sends; which connection a send, wait (its matching is
 * the shared wait fields) or close uses.
 */
export function ExperimentWsFields({ node, doc, patch }: { node: WsNode; doc: Experiment; patch: (change: Partial<ExperimentNode>) => void }) {
  const t = useT();
  switch (node.type) {
    case "ws_connect": return <>
      <label data-tip={t("exp.wsUrlHint")}>URL<TemplateField primary value={node.url} placeholder="ws://127.0.0.1:9001/" onChange={url => patch({ url })} /></label>
      <div className="experiment-header-fields"><p>{t("exp.headers")}</p>{node.headers.map(([name, value], index) => <div key={index}>
        <TemplateField label={t("exp.headerName")} placeholder={t("exp.headerName")} value={name} onChange={text => patch({ headers: node.headers.map((pair, i) => i === index ? [text, pair[1]] : pair) })} />
        <TemplateField label={t("exp.headerValue")} placeholder={t("exp.headerValue")} value={value} onChange={text => patch({ headers: node.headers.map((pair, i) => i === index ? [pair[0], text] : pair) })} />
        <button className="ghost sm" aria-label={t("exp.removeHeader")} data-tip={t("exp.removeHeader")} onClick={() => patch({ headers: node.headers.filter((_, i) => i !== index) })}>×</button>
      </div>)}<button className="ghost sm" onClick={() => patch({ headers: [...node.headers, ["", ""]] })}>＋ {t("exp.addHeader")}</button></div>
      <label data-tip={t("exp.wsProtocolsHint")}>{t("exp.wsProtocols")}<input value={protocolsText(node.protocols)} spellCheck={false} placeholder="graphql-transport-ws"
        onChange={event => patch({ protocols: protocolsOf(event.target.value) })} /></label>
      <label>{t("common.timeoutMs")}<input type="number" min="1" max="120000" value={node.timeout_ms} onChange={event => patch({ timeout_ms: Number(event.target.value) })} /></label>
    </>;
    case "ws_send": return <>
      <ConnectionPicker node={node} doc={doc} patch={patch} />
      <label data-tip={t("exp.wsFormatHint")}>{t("exp.wsFormat")}<select value={node.binary ? "binary" : "text"} onChange={event => patch({ binary: event.target.value === "binary" })}>
        <option value="text">{t("exp.wsText")}</option>
        <option value="binary">{t("exp.wsBinary")}</option>
      </select></label>
      <label>{t("exp.payload")}<TemplateField multiline primary value={node.text} placeholder={node.binary ? "de ad be ef" : '{"type":"ping"}'} onChange={text => patch({ text })} /></label>
    </>;
    case "wait_ws": return <ConnectionPicker node={node} doc={doc} patch={patch} />;
    case "ws_close": return <>
      <ConnectionPicker node={node} doc={doc} patch={patch} />
      <label data-tip={t("exp.wsCloseCodeHint")}>{t("field.code")}<input data-primary type="number" min="1000" max="4999" value={node.code} onChange={event => patch({ code: Number(event.target.value) })} /></label>
      <label>{t("field.reason")}<TemplateField value={node.reason} onChange={reason => patch({ reason })} /></label>
    </>;
  }
}
