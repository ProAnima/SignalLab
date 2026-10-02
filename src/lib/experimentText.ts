/**
 * What the experiment editor says about nodes, wires and steps: titles,
 * one-line summaries, the resolved preview and the names that have no value.
 * Pure — every word comes through `t`; the rest is protocol notation.
 */
import type { ExperimentNode, ExperimentStep } from "./api";
import { portOf, validPortsFor, type Anchor, type NodeType, type Port } from "./experimentGraph.ts";
import { NODE_CATALOG, type NodeGroup } from "./experimentCatalog.ts";
import { describeError, messageParams } from "./errors.ts";
import type { Translate } from "./i18n";

export const GROUP_GLYPH: Record<NodeGroup, string> = { action: "↗", observe: "⇠", emulate: "⧉", data: "{}", check: "✓", flow: "◇" };
/** `:9001` from `127.0.0.1:9001`: the port is what tells waits apart on the canvas. */
const bindPort = (bind: string) => { const at = bind.lastIndexOf(":"); return at >= 0 ? bind.slice(at) : bind; };
export const isWait = (node: ExperimentNode) => node.type === "wait_osc" || node.type === "wait_udp" || node.type === "wait_mqtt" || node.type === "wait_http";
const anyMethod = (method: string) => method.toUpperCase() === "ANY" ? "*" : method.toUpperCase();
/** A node heading's tooltip: what the node does; for the parallel nodes, how they are wired. */
export const nodeHelp = (type: NodeType) => type === "fork" ? "exp.forkHint" as const : type === "join" ? "exp.joinHint" as const : type === "loop" ? "exp.loopHint" as const
  : type === "emulator" ? "exp.emulatorHint" as const : NODE_CATALOG[type].description;
const OP_TEXT: Record<string, string> = { eq: "=", ne: "≠", lt: "<", le: "≤", gt: ">", ge: "≥", contains: "⊃", matches: "~", empty: "= ∅", not_empty: "≠ ∅" };

/** What a node will put on the wire, one line per part, for the resolved preview. */
export function previewLines(node: ExperimentNode): string[] {
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
    case "loop": return node.until ? [`${node.until.value} ${OP_TEXT[node.until.op]} ${node.until.op === "empty" || node.until.op === "not_empty" ? "" : node.until.expected}`.trim()] : [];
    case "wait_osc": return [`${node.address} ⇠ ${node.bind}`, ...node.args.map((rule) => `args[${rule.index}] ${OP_TEXT[rule.op]} ${rule.op === "empty" || rule.op === "not_empty" ? "" : rule.value}`.trim())];
    case "wait_udp": return [`${node.mode === "any" ? "*" : node.pattern} ⇠ ${node.bind}`];
    case "wait_mqtt": return [`${node.topic} ⇠ ${node.host}:${node.port}`, ...(node.mode === "any" ? [] : [node.pattern])];
    case "wait_http": return [`${anyMethod(node.method)} ${node.path} ⇠ ${node.bind}`, ...node.when.map((condition) => `${condition.on}${condition.name ? ` ${condition.name}` : ""} ${OP_TEXT[condition.op]} ${condition.op === "empty" || condition.op === "not_empty" ? "" : condition.value}`.trim())];
    default: return [];
  }
}

function summary(node: ExperimentNode, t: Translate): string {
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
    case "osc": return `${node.address} → ${node.target}${node.reply ? ` ⇠ ${node.reply.address}` : ""}`;
    case "udp": return `${node.text || "∅"} → ${node.target}${node.reply ? ` ⇠ ${node.reply.mode === "any" ? "*" : node.reply.pattern || "∅"}` : ""}`;
    case "extract": return `${node.variable} ← ${node.from === "json" || node.from === "header" || node.from === "regex" ? node.expr : node.from}`;
    case "assert_value": return `${node.value} ${OP_TEXT[node.op]} ${node.op === "empty" || node.op === "not_empty" ? "" : node.expected}`.trim();
    case "branch_value": return `${node.value} ${OP_TEXT[node.op]} ${node.op === "empty" || node.op === "not_empty" ? "" : node.expected} ?`;
    case "loop": return node.until
      ? t("exp.loopSummaryUntil", { max: node.max, condition: `${node.until.value} ${OP_TEXT[node.until.op]} ${node.until.op === "empty" || node.until.op === "not_empty" ? "" : node.until.expected}`.trim() })
      : t("exp.loopSummary", { max: node.max });
    case "wait_osc": return `${node.address} ⇠ ${bindPort(node.bind)} · ${node.timeout_ms} ms`;
    case "wait_udp": return `${node.mode === "any" ? "*" : node.pattern || "∅"} ⇠ ${bindPort(node.bind)} · ${node.timeout_ms} ms`;
    case "wait_mqtt": return `${node.topic} ⇠ ${node.host}:${node.port} · ${node.timeout_ms} ms`;
    case "wait_http": return `${anyMethod(node.method)} ${node.path} ⇠ ${bindPort(node.bind)} · ${node.timeout_ms} ms`;
    case "emulator": return `${node.emulator.protocol.toUpperCase()} ${bindPort(node.emulator.bind)} · ${node.emulator.name}`;
    default: return "";
  }
}

/** A node as the reader knows it: its type. */
export const nodeLabel = (type: NodeType, t: Translate): string => t(NODE_CATALOG[type].title);
/** The line under a node's type: on the canvas, in the finder and in its tooltip. */
export const nodeSummary = (node: ExperimentNode, t: Translate): string =>
  node.type === "start" ? t("exp.entry") : node.type === "end" ? t("exp.result") : summary(node, t);

export function portLabel(port: Port, t: Translate): string {
  switch (port) {
    case "yes": return t("exp.yes");
    case "no": return t("exp.no");
    case "branch1": return t("exp.branch1");
    case "branch2": return t("exp.branch2");
    case "matched": return t("exp.portMatched");
    case "timeout": return t("exp.portTimeout");
    case "body": return t("exp.portBody");
    case "done": return t("exp.portDone");
    case "limit": return t("exp.portLimit");
    default: return t("exp.outputPort");
  }
}

/** "Fork · Branch 1": an output, named by its port only when the node has several. */
export function anchorLabel(nodes: ExperimentNode[], anchor: Anchor, t: Translate): string {
  const node = nodes.find((item) => item.id === anchor.from);
  if (!node) return "";
  return validPortsFor(node.type).length > 1 ? `${nodeLabel(node.type, t)} · ${portLabel(anchor.port, t)}` : nodeLabel(node.type, t);
}

/** "Start → OSC message", with the output named when the node has several. */
export function wireName(nodes: ExperimentNode[], edge: { from: string; to: string; port?: Port }, t: Translate): string {
  const target = nodes.find((node) => node.id === edge.to);
  return `${anchorLabel(nodes, { from: edge.from, port: portOf(edge) }, t)} → ${target ? nodeLabel(target.type, t) : ""}`;
}

/** Names without a value, in words: variables wait for a run, secrets for the Secrets panel. */
export function missingParts(missing: string[]): { key: "exp.missingValues" | "exp.missingSecrets"; names: string }[] {
  const quote = (names: string[]) => names.map((item) => `{{${item}}}`).join(", ");
  const secretsMissing = missing.filter((item) => item.startsWith("secret."));
  const varsMissing = missing.filter((item) => !item.startsWith("secret."));
  return [
    ...(varsMissing.length ? [{ key: "exp.missingValues" as const, names: quote(varsMissing) }] : []),
    ...(secretsMissing.length ? [{ key: "exp.missingSecrets" as const, names: quote(secretsMissing) }] : []),
  ];
}
export const missingText = (missing: string[], t: Translate): string => missingParts(missing).map(({ key, names }) => t(key, { names })).join(" · ");
/** What to do about them, for the tooltip. */
export const missingTip = (missing: string[], t: Translate): string => missingParts(missing).map(({ key }) => t(key === "exp.missingValues" ? "exp.missingValuesHint" : "exp.missingSecretsHint")).join("\n");

/** A timeline row: why it failed, or what it did, in the current language. The row already names the node. */
export function stepText(event: ExperimentStep, t: Translate): string {
  const said = event.message_key ? t(event.message_key, messageParams(event.message_params, t)) : event.detail;
  // A retry says both: which attempt, and why it failed.
  if (event.state === "retry") return event.error ? `${said} — ${describeError(event.error, t).text}` : said;
  return event.error ? describeError(event.error, t).text : said;
}
