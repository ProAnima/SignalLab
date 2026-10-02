import type { ArgRule, CompareOp, Experiment, ExperimentNode, ExtractFrom, UdpMode } from "../lib/api";
import { ExperimentOscFields } from "./ExperimentOscFields";
import { ReplyFields } from "./ExperimentNodeOptions";
import { TemplateField } from "./TemplateField";
import { ConditionsEditor } from "./EmulatorEditor";
import { ExperimentEmulatorFields } from "./ExperimentEmulatorFields";
import { ExperimentFaultFields } from "./ExperimentFaultFields";
import { ExperimentWsFields } from "./ExperimentWsFields";
import { HttpAuthFields } from "./HttpAuthFields";
import { useT, type TKey } from "../lib/i18n";

const HTTP_METHODS = ["ANY", "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

const SOURCES: ExtractFrom[] = ["json", "header", "status", "body", "regex"];
const OPERATORS: CompareOp[] = ["eq", "ne", "lt", "le", "gt", "ge", "contains", "matches", "empty", "not_empty"];
const EXPR_PLACEHOLDER: Partial<Record<ExtractFrom, string>> = { json: "$.token", header: "X-Session", regex: '"token":"(\\w+)"' };
const UDP_MODES: UdpMode[] = ["any", "contains", "regex", "hex"];
const MODE_PLACEHOLDER: Partial<Record<UdpMode, string>> = { contains: "PONG", regex: "PONG (\\d+)", hex: "de ad be ef" };
const unary = (op: CompareOp) => op === "empty" || op === "not_empty";

/** Parameters of one node. Text fields that are sent or compared accept `{{templates}}`; `doc` is where a node that names another finds it. */
export function ExperimentNodeFields({ node, doc, patch }: { node: ExperimentNode; doc: Experiment; patch: (change: Partial<ExperimentNode>) => void }) {
  const t = useT();
  return <>
    {node.type === "assert_body" && <label>{t("exp.contains")}<TemplateField multiline primary value={node.contains} onChange={contains => patch({ contains })} /></label>}
    {node.type === "assert_header" && <><label>{t("exp.headerName")}<TemplateField primary value={node.name} onChange={name => patch({ name })} /></label><label>{t("exp.contains")}<TemplateField value={node.contains} onChange={contains => patch({ contains })} /></label></>}
    {node.type === "assert_latency" && <label>{t("exp.maxLatency")}<input data-primary type="number" min="1" max="120000" value={node.max_ms} onChange={event => patch({ max_ms: Number(event.target.value) })} /></label>}
    {node.type === "log" && <label>{t("exp.logMessage")}<TemplateField multiline primary value={node.message} onChange={message => patch({ message })} /></label>}
    {node.type === "tcp" && <>
      <label>{t("exp.host")}<TemplateField primary value={node.host} onChange={host => patch({ host })} /></label>
      <label>{t("exp.port")}<input type="number" min="1" max="65535" value={node.port} onChange={event => patch({ port: Number(event.target.value) })} /></label>
      <label>{t("common.timeoutMs")}<input type="number" min="1" max="120000" value={node.timeout_ms} onChange={event => patch({ timeout_ms: Number(event.target.value) })} /></label>
      <label>{t("exp.payload")}<TemplateField multiline value={node.payload} onChange={payload => patch({ payload })} /></label>
    </>}
    {node.type === "mqtt" && <>
      <label>{t("exp.broker")}<TemplateField value={node.host} onChange={host => patch({ host })} /></label>
      <label>{t("exp.port")}<input type="number" min="1" max="65535" value={node.port} onChange={event => patch({ port: Number(event.target.value) })} /></label>
      <label>{t("exp.topic")}<TemplateField primary value={node.topic} onChange={topic => patch({ topic })} /></label>
      <label>{t("exp.payload")}<TemplateField multiline value={node.payload} onChange={payload => patch({ payload })} /></label>
      <label>QoS<select value={node.qos} onChange={event => patch({ qos: Number(event.target.value) })}>{[0, 1, 2].map(value => <option key={value}>{value}</option>)}</select></label>
      <label>{t("exp.retain")}<select value={String(node.retain)} onChange={event => patch({ retain: event.target.value === "true" })}><option value="false">{t("exp.no")}</option><option value="true">{t("exp.yes")}</option></select></label>
    </>}
    {node.type === "http" && <>
      <label>{t("exp.method")}<select value={node.request.method} onChange={(event) => patch({ request: { ...node.request, method: event.target.value } })}>{["GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"].map((method) => <option key={method}>{method}</option>)}</select></label>
      <label>URL<TemplateField primary value={node.request.url} onChange={url => patch({ request: { ...node.request, url } })} /></label>
      <label>{t("common.timeoutMs")}<input type="number" min="1" max="120000" value={node.request.timeout_ms} onChange={(event) => patch({ request: { ...node.request, timeout_ms: Number(event.target.value) } })} /></label>
      <div className="experiment-header-fields"><p>{t("exp.headers")}</p>{node.request.headers.map(([name, value], index) => <div key={index}>
        <TemplateField label={t("exp.headerName")} placeholder={t("exp.headerName")} value={name} onChange={text => patch({ request: { ...node.request, headers: node.request.headers.map((pair, i) => i === index ? [text, pair[1]] : pair) } })} />
        <TemplateField label={t("exp.headerValue")} placeholder={t("exp.headerValue")} value={value} onChange={text => patch({ request: { ...node.request, headers: node.request.headers.map((pair, i) => i === index ? [pair[0], text] : pair) } })} />
        <button className="ghost sm" aria-label={t("exp.removeHeader")} data-tip={t("exp.removeHeader")} onClick={() => patch({ request: { ...node.request, headers: node.request.headers.filter((_, i) => i !== index) } })}>×</button>
      </div>)}<button className="ghost sm" onClick={() => patch({ request: { ...node.request, headers: [...node.request.headers, ["", ""]] } })}>＋ {t("exp.addHeader")}</button></div>
      <label>{t("exp.body")}<TemplateField multiline value={node.request.body ?? ""} onChange={body => patch({ request: { ...node.request, body: body || null } })} /></label>
      <HttpAuthFields templates auth={node.request.auth ?? { scheme: "none" }}
        onChange={auth => { const { auth: _, ...rest } = node.request; patch({ request: auth.scheme === "none" ? rest : { ...rest, auth } }); }} />
    </>}
    {node.type === "delay" && <label>{t("exp.delayMs")}<input data-primary type="number" min="0" max="60000" value={node.ms} onChange={(event) => patch({ ms: Number(event.target.value) })} /></label>}
    {(node.type === "assert_status" || node.type === "branch_status") && <label>{t("exp.expectedStatus")}<input data-primary type="number" min="100" max="599" value={node.status} onChange={(event) => patch({ status: Number(event.target.value) })} /></label>}
    {node.type === "osc" && <><label>{t("common.target")}<TemplateField value={node.target} onChange={target => patch({ target })} /></label><label>{t("common.address")}<TemplateField primary value={node.address} onChange={address => patch({ address })} /></label></>}
    {node.type === "osc" && <div className="experiment-osc-args"><p>{t("exp.arguments")}</p><ExperimentOscFields key={node.id} args={node.args} onChange={args => patch({ args })} /></div>}
    {node.type === "udp" && <><label>{t("common.target")}<TemplateField value={node.target} onChange={target => patch({ target })} /></label><label>{t("exp.payload")}<TemplateField multiline primary value={node.text} onChange={text => patch({ text })} /></label></>}
    {(node.type === "osc" || node.type === "udp") && <ReplyFields node={node} patch={patch} />}
    {node.type === "extract" && <>
      <label data-tip={t("exp.extractHint", { name: node.variable || "name" })}>{t("exp.variable")}<input data-primary value={node.variable} spellCheck={false} onChange={event => patch({ variable: event.target.value.replace(/\s+/g, "_") })} /></label>
      <label>{t("exp.extractFrom")}<select value={node.from} onChange={event => patch({ from: event.target.value as ExtractFrom })}>{SOURCES.map(source => <option key={source} value={source}>{t(`exp.from.${source}` as TKey)}</option>)}</select></label>
      {node.from !== "status" && node.from !== "body" && <label>{node.from === "json" ? t("exp.jsonPath") : node.from === "header" ? t("exp.headerName") : t("exp.pattern")}
        <input value={node.expr} spellCheck={false} placeholder={EXPR_PLACEHOLDER[node.from]} onChange={event => patch({ expr: event.target.value })} /></label>}
    </>}
    {(node.type === "assert_value" || node.type === "branch_value") && <>
      <label>{t("exp.value")}<TemplateField primary value={node.value} onChange={value => patch({ value })} /></label>
      <label data-tip={t("exp.valueHint")}>{t("exp.operator")}<select value={node.op} onChange={event => patch({ op: event.target.value as CompareOp })}>{OPERATORS.map(op => <option key={op} value={op}>{t(`exp.op.${op}` as TKey)}</option>)}</select></label>
      {node.op !== "empty" && node.op !== "not_empty" && <label>{t("exp.expected")}<TemplateField value={node.expected} onChange={expected => patch({ expected })} /></label>}
    </>}
    {node.type === "loop" && <>
      <label>{t("exp.loopMax")}<input data-primary type="number" min="1" max="1000" value={node.max} onChange={event => patch({ max: Number(event.target.value) })} /></label>
      <label className="checkbox experiment-check" data-tip={t("exp.loopUntilHint")}>
        <input type="checkbox" checked={!!node.until} onChange={event => patch({ until: event.target.checked ? { value: "", op: "eq", expected: "" } : null })} />{t("exp.loopUntilOn")}
      </label>
      {node.until && <div className="experiment-option-fields">
        <label>{t("exp.value")}<TemplateField value={node.until.value} placeholder="{{reply.args[0]}}" onChange={value => patch({ until: { ...node.until!, value } })} /></label>
        <label data-tip={t("exp.valueHint")}>{t("exp.operator")}<select value={node.until.op} onChange={event => patch({ until: { ...node.until!, op: event.target.value as CompareOp } })}>{OPERATORS.map(op => <option key={op} value={op}>{t(`exp.op.${op}` as TKey)}</option>)}</select></label>
        {!unary(node.until.op) && <label>{t("exp.expected")}<TemplateField value={node.until.expected} onChange={expected => patch({ until: { ...node.until!, expected } })} /></label>}
      </div>}
    </>}
    {(node.type === "wait_osc" || node.type === "wait_udp") && <label data-tip={t("exp.waitHint", { name: node.variable || "reply" })}>{t("exp.listenOn")}
      <input value={node.bind} spellCheck={false} placeholder="0.0.0.0:9001" onChange={event => patch({ bind: event.target.value.trim() })} /></label>}
    {node.type === "wait_osc" && <>
      <label data-tip={t("exp.addressPatternHint")}>{t("exp.addressPattern")}<TemplateField primary value={node.address} onChange={address => patch({ address })} /></label>
      <ArgRules rules={node.args} onChange={args => patch({ args })} />
    </>}
    {node.type === "wait_mqtt" && <>
      <label data-tip={t("exp.mqttWaitHint")}>{t("exp.broker")}<TemplateField value={node.host} onChange={host => patch({ host })} /></label>
      <label>{t("exp.port")}<input type="number" min="1" max="65535" value={node.port} onChange={event => patch({ port: Number(event.target.value) })} /></label>
      <label data-tip={t("exp.topicFilterHint")}>{t("exp.topicFilter")}<TemplateField primary value={node.topic} placeholder="lab/+/state" onChange={topic => patch({ topic })} /></label>
    </>}
    {(node.type === "ws_connect" || node.type === "ws_send" || node.type === "wait_ws" || node.type === "ws_close") && <ExperimentWsFields node={node} doc={doc} patch={patch} />}
    {(node.type === "wait_udp" || node.type === "wait_mqtt" || node.type === "wait_ws") && <>
      <label>{t("exp.waitMode")}<select value={node.mode} onChange={event => patch({ mode: event.target.value as UdpMode })}>{UDP_MODES.map(mode => <option key={mode} value={mode}>{t(`exp.mode.${mode}` as TKey)}</option>)}</select></label>
      {node.mode !== "any" && <label>{t("field.pattern")}<TemplateField primary value={node.pattern} placeholder={MODE_PLACEHOLDER[node.mode]} onChange={pattern => patch({ pattern })} /></label>}
    </>}
    {node.type === "wait_http" && <>
      <label data-tip={t("exp.waitHttpHint", { name: node.variable || "request" })}>{t("exp.listenOn")}
        <input value={node.bind} spellCheck={false} placeholder="127.0.0.1:18080" onChange={event => patch({ bind: event.target.value.trim() })} /></label>
      <label>{t("exp.method")}<select value={node.method.toUpperCase()} onChange={event => patch({ method: event.target.value })}>
        {HTTP_METHODS.map(method => <option key={method} value={method}>{method === "ANY" ? t("emu.methodAny") : method}</option>)}</select></label>
      <label data-tip={t("emu.pathHint")}>{t("exp.path")}<TemplateField primary value={node.path} onChange={path => patch({ path })} /></label>
      <ConditionsEditor conditions={node.when} onChange={when => patch({ when })} idPrefix={`${node.id}-`} />
    </>}
    {node.type === "emulator" && <ExperimentEmulatorFields emulator={node.emulator} onChange={emulator => patch({ emulator })} />}
    {(node.type === "impairment" || node.type === "impairment_change" || node.type === "emulator_state") && <ExperimentFaultFields node={node} doc={doc} patch={patch} />}
    {(node.type === "wait_osc" || node.type === "wait_udp" || node.type === "wait_mqtt" || node.type === "wait_http" || node.type === "wait_ws") && <>
      <label>{t("exp.waitTimeout")}<input type="number" min="1" max="120000" value={node.timeout_ms} onChange={event => patch({ timeout_ms: Number(event.target.value) })} /></label>
      <label>{t("exp.replyVariable")}<input value={node.variable} spellCheck={false} onChange={event => patch({ variable: event.target.value.replace(/\s+/g, "_") })} /></label>
    </>}
  </>;
}

/** `args[n] <op> value` rows of a Wait for OSC (and of an OSC reply). The remove button sits outside every label. */
export function ArgRules({ rules, onChange }: { rules: ArgRule[]; onChange: (rules: ArgRule[]) => void }) {
  const t = useT();
  const update = (index: number, change: Partial<ArgRule>) => onChange(rules.map((rule, i) => i === index ? { ...rule, ...change } : rule));
  const next = rules.length ? Math.min(63, rules[rules.length - 1].index + 1) : 0;
  return <div className="experiment-rule-fields"><p data-tip={t("exp.argRulesHint")}>{t("exp.argRules")}</p>
    {rules.map((rule, index) => <div key={index} className="experiment-rule">
      <input type="number" min="0" max="63" aria-label={`${t("exp.argIndex")} ${index + 1}`} data-tip={`args[${rule.index}]`} value={rule.index}
        onChange={event => update(index, { index: Math.max(0, Math.floor(Number(event.target.value) || 0)) })} />
      <select aria-label={t("exp.operator")} value={rule.op} onChange={event => update(index, { op: event.target.value as CompareOp })}>
        {OPERATORS.map(op => <option key={op} value={op}>{t(`exp.op.${op}` as TKey)}</option>)}</select>
      {!unary(rule.op) && <TemplateField label={t("field.rule_value")} placeholder={t("field.value")} value={rule.value} onChange={value => update(index, { value })} />}
      <button className="ghost sm" aria-label={t("exp.removeRule")} data-tip={t("exp.removeRule")} onClick={() => onChange(rules.filter((_, i) => i !== index))}>×</button>
    </div>)}
    <button className="ghost sm" disabled={rules.length >= 16} onClick={() => onChange([...rules, { index: next, op: "eq", value: "" }])}>＋ {t("exp.addRule")}</button>
  </div>;
}
