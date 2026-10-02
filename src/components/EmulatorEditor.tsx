import { useState } from "react";
import type {
  ArgOut, ArgType, CompareOp, Condition, Delimiter, DownFault, Emulator, EmulatorResponse, Fault, MqttRetained, MqttRule, OscRule, RawPayload, ResponseOrder,
  Route, TcpRule, UdpMode, UdpRule,
} from "../lib/api";
import { ArgRules } from "./ExperimentNodeFields";
import { useFieldIds } from "../lib/hooks";
import { useT, type TKey } from "../lib/i18n";
import { copyText, isDesktop } from "../lib/platform";
import {
  blankCondition, blankMqttRule, blankOscRule, blankOutage, blankRetained, blankRoute, blankTcpRule, blankUdpRule, emulatorUrl, PRESETS, presetLabel, presetResponse,
  protocolLabel, type Preset,
} from "../lib/emulators";

const METHODS = ["ANY", "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
const OPERATORS: CompareOp[] = ["eq", "ne", "lt", "le", "gt", "ge", "contains", "matches", "empty", "not_empty"];
const MODES: UdpMode[] = ["any", "contains", "regex", "hex"];
const ARG_TYPES: ArgType[] = ["int", "float", "str", "long", "double", "bool", "blob", "nil"];
const FAULTS: Fault[] = ["none", "timeout", "reset", "malformed"];
const DOWN_FAULTS: DownFault[] = ["unavailable", "reset", "timeout"];
const QOS = [0, 1, 2];
const ORDERS: ResponseOrder[] = ["sequence", "cycle", "random"];
const DELIMITERS: Delimiter[] = ["lf", "crlf", "cr", "none"];
const ON: Condition["on"][] = ["header", "query", "body", "json"];
const unary = (op: CompareOp) => op === "empty" || op === "not_empty";
const MODE_PLACEHOLDER: Partial<Record<UdpMode, string>> = { contains: "PING", regex: "^POWER (ON|OFF)$", hex: "de ad be ef" };

/** A list with one item replaced, removed, or moved by one place. */
const replaced = <T,>(list: T[], index: number, item: T) => list.map((old, at) => at === index ? item : old);
const removed = <T,>(list: T[], index: number) => list.filter((_, at) => at !== index);
function moved<T>(list: T[], index: number, by: -1 | 1): T[] {
  const to = index + by;
  if (to < 0 || to >= list.length) return list;
  const next = list.slice();
  [next[index], next[to]] = [next[to], next[index]];
  return next;
}

/** A number field that takes only what it may hold: whole, and within its bounds. */
function NumberField({ id, label, tip, value, min, max, onChange, disabled }: {
  id: string; label: string; tip?: string; value: number; min: number; max: number; onChange: (value: number) => void; disabled?: boolean;
}) {
  return <div className="field">
    <label htmlFor={id} data-tip={tip}>{label}</label>
    <input id={id} type="number" min={min} max={max} value={value} disabled={disabled}
      onChange={(event) => onChange(Math.min(max, Math.max(min, Math.round(Number(event.target.value) || 0))))} />
  </div>;
}

/** Every condition must hold: where to look, its name, how to compare, with what. */
export function ConditionsEditor({ conditions, onChange, idPrefix }: { conditions: Condition[]; onChange: (next: Condition[]) => void; idPrefix: string }) {
  const t = useT();
  const fid = useFieldIds();
  return <div className="emu-conditions">
    <p className="emu-subhead" data-tip={t("emu.conditionsHint")}>{t("emu.conditions")}</p>
    {conditions.map((condition, index) => {
      const set = (change: Partial<Condition>) => onChange(replaced(conditions, index, { ...condition, ...change }));
      const n = index + 1;
      return <div className="emu-condition" key={index}>
        <select aria-label={`${t("emu.on")} ${n}`} value={condition.on} onChange={(event) => set({ on: event.target.value as Condition["on"] })}>
          {ON.map((on) => <option key={on} value={on}>{t(`emu.on.${on}` as TKey)}</option>)}
        </select>
        {condition.on !== "body" && <input id={fid(`${idPrefix}c${index}-name`)} aria-label={`${t("emu.conditionName")} ${n}`}
          placeholder={t(`emu.conditionName.${condition.on}` as TKey)} spellCheck={false} value={condition.name} onChange={(event) => set({ name: event.target.value })} />}
        <select aria-label={`${t("exp.operator")} ${n}`} value={condition.op} onChange={(event) => set({ op: event.target.value as CompareOp })}>
          {OPERATORS.map((op) => <option key={op} value={op}>{t(`exp.op.${op}` as TKey)}</option>)}
        </select>
        {!unary(condition.op) && <input aria-label={`${t("field.value")} ${n}`} placeholder={t("field.value")} spellCheck={false} value={condition.value} onChange={(event) => set({ value: event.target.value })} />}
        <button className="ghost sm" aria-label={t("emu.removeCondition")} data-tip={t("emu.removeCondition")} onClick={() => onChange(removed(conditions, index))}>×</button>
      </div>;
    })}
    <button className="ghost sm" disabled={conditions.length >= 16} onClick={() => onChange([...conditions, blankCondition()])}>＋ {t("emu.addCondition")}</button>
  </div>;
}

/** One response of a route: status or fault, timing, headers, body. */
function ResponseEditor({ response, onChange, onRemove, n, random, idPrefix }: {
  response: EmulatorResponse; onChange: (next: EmulatorResponse) => void; onRemove?: () => void; n?: number; random: boolean; idPrefix: string;
}) {
  const t = useT();
  const fid = useFieldIds();
  const id = (name: string) => fid(`${idPrefix}${name}`);
  const set = (change: Partial<EmulatorResponse>) => onChange({ ...response, ...change });
  const answers = response.fault === "none" || response.fault === "malformed";
  return <div className="emu-response">
    {n !== undefined && <div className="emu-response-head">
      <span className="emu-response-n">{t("emu.response", { n })}</span>
      {onRemove && <button className="ghost sm" aria-label={`${t("emu.removeResponse")} ${n}`} data-tip={t("emu.removeResponse")} onClick={onRemove}>×</button>}
    </div>}
    <div className="row">
      <NumberField id={id("status")} label={t("emu.status")} value={response.status} min={100} max={599} disabled={!answers} onChange={(status) => set({ status })} />
      <div className="field">
        <label htmlFor={id("fault")} data-tip={t("emu.faultHint")}>{t("emu.fault")}</label>
        <select id={id("fault")} value={response.fault} onChange={(event) => set({ fault: event.target.value as Fault })}>
          {FAULTS.map((fault) => <option key={fault} value={fault}>{t(`emu.fault.${fault}` as TKey)}</option>)}
        </select>
      </div>
      <NumberField id={id("delay")} label={t("emu.delay")} value={response.delay_ms} min={0} max={60000} onChange={(delay_ms) => set({ delay_ms })} />
      <NumberField id={id("jitter")} label={t("emu.jitter")} tip={t("emu.jitterHint")} value={response.jitter_ms} min={0} max={60000} onChange={(jitter_ms) => set({ jitter_ms })} />
      {random && <NumberField id={id("weight")} label={t("emu.weight")} tip={t("emu.weightHint")} value={response.weight} min={0} max={1000} onChange={(weight) => set({ weight })} />}
    </div>
    {answers && <>
      <div className="emu-headers">
        <p className="emu-subhead">{t("emu.headers")}</p>
        {response.headers.map(([name, value], index) => <div className="row tight" key={index}>
          <input aria-label={`${t("emu.headerName")} ${index + 1}`} placeholder={t("emu.headerName")} spellCheck={false} value={name}
            onChange={(event) => set({ headers: replaced(response.headers, index, [event.target.value, value]) })} />
          <input aria-label={`${t("emu.headerValue")} ${index + 1}`} placeholder={t("emu.headerValue")} spellCheck={false} value={value}
            onChange={(event) => set({ headers: replaced(response.headers, index, [name, event.target.value]) })} />
          <button className="ghost sm" style={{ flex: "0 0 auto" }} aria-label={t("emu.removeHeader")} data-tip={t("emu.removeHeader")}
            onClick={() => set({ headers: removed(response.headers, index) })}>×</button>
        </div>)}
        <button className="ghost sm" disabled={response.headers.length >= 32} onClick={() => set({ headers: [...response.headers, ["", ""]] })}>＋ {t("emu.addHeader")}</button>
      </div>
      <div className="field">
        <label htmlFor={id("body")} data-tip={t("emu.bodyHint")}>{t("emu.body")}</label>
        <textarea id={id("body")} className="emu-body" spellCheck={false} value={response.body} onChange={(event) => set({ body: event.target.value })} />
      </div>
    </>}
  </div>;
}

/** Copy `url` to the clipboard, saying in the button's tip whether it worked. */
function CopyUrl({ url }: { url: string }) {
  const t = useT();
  const [copied, setCopied] = useState<boolean | null>(null);
  const tip = copied === true ? t("emu.copied") : copied === false ? t("emu.copyFailed", { url }) : t("emu.copyRouteUrlHint", { url });
  return <button className="ghost sm emu-copy" data-tip={tip}
    onClick={() => void copyText(url).then(setCopied)} onBlur={() => setCopied(null)}>
    {t("emu.copyRouteUrl")}
  </button>;
}

function RouteBody({ route, onChange, idPrefix, base }: { route: Route; onChange: (next: Route) => void; idPrefix: string; base: string | null }) {
  const t = useT();
  const fid = useFieldIds();
  const id = (name: string) => fid(`${idPrefix}${name}`);
  const set = (change: Partial<Route>) => onChange({ ...route, ...change });
  const random = route.order === "random";
  // A path with :name or * segments is a pattern; copied as it stands, it is one request it takes.
  const url = base && route.path.startsWith("/") && !route.path.includes("{{") ? `${base}${route.path}` : null;
  return <>
    <div className="row">
      <div className="field" style={{ flex: "0 0 112px" }}>
        <label htmlFor={id("method")}>{t("emu.method")}</label>
        <select id={id("method")} value={route.method.toUpperCase()} onChange={(event) => set({ method: event.target.value })}>
          {METHODS.map((method) => <option key={method} value={method}>{method === "ANY" ? t("emu.methodAny") : method}</option>)}
        </select>
      </div>
      <div className="field">
        <label htmlFor={id("path")} data-tip={t("emu.pathHint")}>{t("emu.path")}</label>
        <input id={id("path")} data-primary spellCheck={false} value={route.path} onChange={(event) => set({ path: event.target.value })} />
      </div>
      {url && <CopyUrl url={url} />}
    </div>
    <ConditionsEditor conditions={route.when} onChange={(when) => set({ when })} idPrefix={idPrefix} />
    <p className="emu-subhead">{t("emu.responses")}</p>
    {route.responses.length > 1 && <div className="field">
      <label htmlFor={id("order")} data-tip={t("emu.orderHint")}>{t("emu.order")}</label>
      <select id={id("order")} value={route.order} onChange={(event) => set({ order: event.target.value as ResponseOrder })}>
        {ORDERS.map((order) => <option key={order} value={order}>{t(`emu.order.${order}` as TKey)}</option>)}
      </select>
    </div>}
    {route.responses.map((response, index) => <ResponseEditor key={index} n={index + 1} response={response} random={random} idPrefix={`${idPrefix}r${index}-`}
      onChange={(next) => set({ responses: replaced(route.responses, index, next) })}
      onRemove={route.responses.length > 1 ? () => set({ responses: removed(route.responses, index) }) : undefined} />)}
    {route.responses.length < 16 && <div className="field emu-preset">
      <label htmlFor={id("preset")}>{t("emu.preset")}</label>
      {/* An action from a list: picking one adds that response, and the list goes back to its prompt. */}
      <select id={id("preset")} value="" onChange={(event) => { if (event.target.value) set({ responses: [...route.responses, presetResponse(event.target.value as Preset)] }); }}>
        <option value="">＋</option>
        {PRESETS.map((preset) => <option key={preset} value={preset}>{t(presetLabel(preset))}</option>)}
      </select>
    </div>}
  </>;
}

/** The fields of a reply payload: none, text or hex. */
function PayloadField({ reply, onChange, id }: { reply: RawPayload | null; onChange: (next: RawPayload | null) => void; id: (name: string) => string }) {
  const t = useT();
  const kind = reply?.kind ?? "none";
  return <>
    <div className="field">
      <label htmlFor={id("reply-kind")}>{t("emu.reply")}</label>
      <select id={id("reply-kind")} value={kind} onChange={(event) => {
        const value = event.target.value;
        const text = reply ? (reply.kind === "text" ? reply.text : reply.hex) : "";
        onChange(value === "none" ? null : value === "hex" ? { kind: "hex", hex: reply?.kind === "hex" ? text : "" } : { kind: "text", text: reply?.kind === "text" ? text : "" });
      }}>
        <option value="none">{t("emu.replyOff")}</option>
        <option value="text">{t("emu.replyText")}</option>
        <option value="hex">{t("emu.replyHex")}</option>
      </select>
    </div>
    {reply && <div className="field">
      <label htmlFor={id("reply")} data-tip={t("emu.replyHint")}>{reply.kind === "hex" ? t("emu.replyHex") : t("emu.replyText")}</label>
      <textarea id={id("reply")} className="emu-body" spellCheck={false} value={reply.kind === "hex" ? reply.hex : reply.text}
        onChange={(event) => onChange(reply.kind === "hex" ? { kind: "hex", hex: event.target.value } : { kind: "text", text: event.target.value })} />
    </div>}
  </>;
}

function MatchFields({ mode, pattern, onChange, id }: { mode: UdpMode; pattern: string; onChange: (change: { mode?: UdpMode; pattern?: string }) => void; id: (name: string) => string }) {
  const t = useT();
  return <div className="row">
    <div className="field">
      <label htmlFor={id("mode")}>{t("emu.match")}</label>
      <select id={id("mode")} value={mode} onChange={(event) => onChange({ mode: event.target.value as UdpMode })}>
        {MODES.map((value) => <option key={value} value={value}>{t(`exp.mode.${value}` as TKey)}</option>)}
      </select>
    </div>
    {mode !== "any" && <div className="field">
      <label htmlFor={id("pattern")}>{t("emu.pattern")}</label>
      <input id={id("pattern")} data-primary spellCheck={false} placeholder={MODE_PLACEHOLDER[mode]} value={pattern} onChange={(event) => onChange({ pattern: event.target.value })} />
    </div>}
  </div>;
}

function Timing({ delay, jitter, onChange, id }: { delay: number; jitter: number; onChange: (change: { delay_ms?: number; jitter_ms?: number }) => void; id: (name: string) => string }) {
  const t = useT();
  return <div className="row">
    <NumberField id={id("delay")} label={t("emu.delay")} value={delay} min={0} max={60000} onChange={(delay_ms) => onChange({ delay_ms })} />
    <NumberField id={id("jitter")} label={t("emu.jitter")} tip={t("emu.jitterHint")} value={jitter} min={0} max={60000} onChange={(jitter_ms) => onChange({ jitter_ms })} />
  </div>;
}

function OscRuleBody({ rule, onChange, idPrefix }: { rule: OscRule; onChange: (next: OscRule) => void; idPrefix: string }) {
  const t = useT();
  const fid = useFieldIds();
  const id = (name: string) => fid(`${idPrefix}${name}`);
  const set = (change: Partial<OscRule>) => onChange({ ...rule, ...change });
  const reply = rule.reply;
  const setArgs = (args: ArgOut[]) => reply && set({ reply: { ...reply, args } });
  return <>
    <div className="field">
      <label htmlFor={id("address")} data-tip={t("emu.addressHint")}>{t("emu.address")}</label>
      <input id={id("address")} data-primary spellCheck={false} value={rule.address} onChange={(event) => set({ address: event.target.value })} />
    </div>
    <ArgRules rules={rule.args} onChange={(args) => set({ args })} />
    <label className="checkbox emu-check">
      <input type="checkbox" checked={!!reply} onChange={(event) => set({ reply: event.target.checked ? { address: "/reply", args: [] } : null })} />
      {t("emu.replyOn")}
    </label>
    {reply && <>
      <div className="field">
        <label htmlFor={id("reply-address")}>{t("emu.replyAddress")}</label>
        <input id={id("reply-address")} spellCheck={false} value={reply.address} onChange={(event) => set({ reply: { ...reply, address: event.target.value } })} />
      </div>
      <div className="emu-args">
        <p className="emu-subhead" data-tip={t("emu.replyArgsHint")}>{t("emu.replyArgs")}</p>
        {reply.args.map((arg, index) => <div className="row tight" key={index}>
          <select style={{ flex: "0 0 96px" }} aria-label={`${t("emu.argType")} ${index + 1}`} value={arg.type} onChange={(event) => setArgs(replaced(reply.args, index, { ...arg, type: event.target.value as ArgType }))}>
            {ARG_TYPES.map((type) => <option key={type} value={type}>{type}</option>)}
          </select>
          {arg.type !== "nil" && <input aria-label={`${t("emu.argValue")} ${index + 1}`} placeholder={t("emu.argValue")} spellCheck={false} value={arg.value}
            onChange={(event) => setArgs(replaced(reply.args, index, { ...arg, value: event.target.value }))} />}
          <button className="ghost sm" style={{ flex: "0 0 auto" }} aria-label={t("emu.removeArg")} data-tip={t("emu.removeArg")} onClick={() => setArgs(removed(reply.args, index))}>×</button>
        </div>)}
        <button className="ghost sm" disabled={reply.args.length >= 16} onClick={() => setArgs([...reply.args, { type: "str", value: "" }])}>＋ {t("emu.addArg")}</button>
      </div>
      <div className="field">
        <label htmlFor={id("to")} data-tip={t("emu.toHint")}>{t("emu.to")}</label>
        <input id={id("to")} spellCheck={false} placeholder="—" value={rule.to} onChange={(event) => set({ to: event.target.value })} />
      </div>
      <Timing delay={rule.delay_ms} jitter={rule.jitter_ms} onChange={set} id={id} />
    </>}
  </>;
}

function UdpRuleBody({ rule, onChange, idPrefix }: { rule: UdpRule; onChange: (next: UdpRule) => void; idPrefix: string }) {
  const t = useT();
  const fid = useFieldIds();
  const id = (name: string) => fid(`${idPrefix}${name}`);
  const set = (change: Partial<UdpRule>) => onChange({ ...rule, ...change });
  return <>
    <MatchFields mode={rule.mode} pattern={rule.pattern} onChange={set} id={id} />
    <PayloadField reply={rule.reply} onChange={(reply) => set({ reply })} id={id} />
    {rule.reply && <>
      <div className="field">
        <label htmlFor={id("to")} data-tip={t("emu.toHint")}>{t("emu.to")}</label>
        <input id={id("to")} spellCheck={false} placeholder="—" value={rule.to} onChange={(event) => set({ to: event.target.value })} />
      </div>
      <Timing delay={rule.delay_ms} jitter={rule.jitter_ms} onChange={set} id={id} />
    </>}
  </>;
}

function TcpRuleBody({ rule, onChange, idPrefix }: { rule: TcpRule; onChange: (next: TcpRule) => void; idPrefix: string }) {
  const t = useT();
  const fid = useFieldIds();
  const id = (name: string) => fid(`${idPrefix}${name}`);
  const set = (change: Partial<TcpRule>) => onChange({ ...rule, ...change });
  return <>
    <MatchFields mode={rule.mode} pattern={rule.pattern} onChange={set} id={id} />
    <PayloadField reply={rule.reply} onChange={(reply) => set({ reply })} id={id} />
    <label className="checkbox emu-check">
      <input type="checkbox" checked={rule.close} onChange={(event) => set({ close: event.target.checked })} />
      {t("emu.close")}
    </label>
    {rule.reply && <Timing delay={rule.delay_ms} jitter={rule.jitter_ms} onChange={set} id={id} />}
  </>;
}

function MqttRuleBody({ rule, onChange, idPrefix }: { rule: MqttRule; onChange: (next: MqttRule) => void; idPrefix: string }) {
  const t = useT();
  const fid = useFieldIds();
  const id = (name: string) => fid(`${idPrefix}${name}`);
  const set = (change: Partial<MqttRule>) => onChange({ ...rule, ...change });
  const reply = rule.reply;
  const setReply = (change: Partial<NonNullable<MqttRule["reply"]>>) => reply && set({ reply: { ...reply, ...change } });
  return <>
    <div className="field">
      <label htmlFor={id("topic")} data-tip={t("emu.topicFilterHint")}>{t("emu.topicFilter")}</label>
      <input id={id("topic")} data-primary spellCheck={false} value={rule.topic} onChange={(event) => set({ topic: event.target.value })} />
    </div>
    <MatchFields mode={rule.mode} pattern={rule.pattern} onChange={set} id={id} />
    <label className="checkbox emu-check">
      <input type="checkbox" checked={!!reply} onChange={(event) => set({ reply: event.target.checked ? { topic: "", payload: "", qos: 0, retain: false } : null })} />
      {t("emu.replyOn")}
    </label>
    {reply && <>
      <div className="row">
        <div className="field">
          <label htmlFor={id("reply-topic")} data-tip={t("emu.mqttReplyHint")}>{t("emu.replyTopic")}</label>
          <input id={id("reply-topic")} spellCheck={false} value={reply.topic} onChange={(event) => setReply({ topic: event.target.value })} />
        </div>
        <div className="field" style={{ flex: "0 0 72px" }}>
          <label htmlFor={id("reply-qos")}>{t("emu.qos")}</label>
          <select id={id("reply-qos")} value={reply.qos} onChange={(event) => setReply({ qos: Number(event.target.value) })}>
            {QOS.map((qos) => <option key={qos} value={qos}>{qos}</option>)}
          </select>
        </div>
      </div>
      <div className="field">
        <label htmlFor={id("reply-payload")} data-tip={t("emu.mqttReplyHint")}>{t("emu.replyPayload")}</label>
        <textarea id={id("reply-payload")} className="emu-body" spellCheck={false} value={reply.payload} onChange={(event) => setReply({ payload: event.target.value })} />
      </div>
      <label className="checkbox emu-check">
        <input type="checkbox" checked={reply.retain} onChange={(event) => setReply({ retain: event.target.checked })} />
        {t("emu.retain")}
      </label>
      <Timing delay={rule.delay_ms} jitter={rule.jitter_ms} onChange={set} id={id} />
    </>}
  </>;
}

/** A broker's login and the messages it retains from the start. */
function BrokerFields({ emulator, onChange }: { emulator: Extract<Emulator, { protocol: "mqtt" }>; onChange: (next: Emulator) => void }) {
  const t = useT();
  const fid = useFieldIds();
  const setRetained = (retained: MqttRetained[]) => onChange({ ...emulator, retained });
  return <>
    <div className="row">
      <div className="field">
        <label htmlFor={fid("username")} data-tip={t("emu.usernameHint")}>{t("emu.username")}</label>
        <input id={fid("username")} spellCheck={false} autoComplete="off" value={emulator.username} onChange={(event) => onChange({ ...emulator, username: event.target.value })} />
      </div>
      <div className="field">
        <label htmlFor={fid("password")}>{t("emu.password")}</label>
        <input id={fid("password")} spellCheck={false} autoComplete="off" value={emulator.password} onChange={(event) => onChange({ ...emulator, password: event.target.value })} />
      </div>
    </div>
    <div className="emu-retained">
      <p className="emu-subhead" data-tip={t("emu.retainedHint")}>{t("emu.retained")}</p>
      {emulator.retained.map((message, index) => {
        const n = index + 1;
        const set = (change: Partial<MqttRetained>) => setRetained(replaced(emulator.retained, index, { ...message, ...change }));
        return <div className="row tight" key={index}>
          <input aria-label={`${t("emu.topic")} ${n}`} placeholder={t("emu.topic")} spellCheck={false} value={message.topic} onChange={(event) => set({ topic: event.target.value })} />
          <input aria-label={`${t("emu.payload")} ${n}`} placeholder={t("emu.payload")} spellCheck={false} value={message.payload} onChange={(event) => set({ payload: event.target.value })} />
          <select style={{ flex: "0 0 64px" }} aria-label={`${t("emu.qos")} ${n}`} value={message.qos} onChange={(event) => set({ qos: Number(event.target.value) })}>
            {QOS.map((qos) => <option key={qos} value={qos}>{qos}</option>)}
          </select>
          <button className="ghost sm" style={{ flex: "0 0 auto" }} aria-label={`${t("emu.removeRetained")} · ${n}`} data-tip={t("emu.removeRetained")}
            onClick={() => setRetained(removed(emulator.retained, index))}>×</button>
        </div>;
      })}
      <button className="ghost sm" disabled={emulator.retained.length >= 64} onClick={() => setRetained([...emulator.retained, blankRetained()])}>＋ {t("emu.addRetained")}</button>
    </div>
  </>;
}

/** Down now and then: how long up, how long down, and what HTTP meets meanwhile. */
function OutageFields({ emulator, onChange }: { emulator: Emulator; onChange: (next: Emulator) => void }) {
  const t = useT();
  const fid = useFieldIds();
  const outage = emulator.outage ?? null;
  const set = (change: Partial<NonNullable<Emulator["outage"]>>) => outage && onChange({ ...emulator, outage: { ...outage, ...change } });
  return <div className="emu-outage">
    <label className="checkbox emu-check" data-tip={t("emu.outageHint")}>
      <input type="checkbox" checked={!!outage} onChange={(event) => onChange({ ...emulator, outage: event.target.checked ? blankOutage() : null })} />
      {t("emu.outage")}
    </label>
    {outage && <div className="row">
      <NumberField id={fid("outage-up")} label={t("emu.outageUp")} value={outage.up_ms} min={10} max={3600000} onChange={(up_ms) => set({ up_ms })} />
      <NumberField id={fid("outage-down")} label={t("emu.outageDown")} value={outage.down_ms} min={10} max={3600000} onChange={(down_ms) => set({ down_ms })} />
      {emulator.protocol === "http" && <div className="field">
        <label htmlFor={fid("outage-fault")}>{t("emu.outageFault")}</label>
        <select id={fid("outage-fault")} value={outage.fault} onChange={(event) => set({ fault: event.target.value as DownFault })}>
          {DOWN_FAULTS.map((fault) => <option key={fault} value={fault}>{t(`emu.outageFault.${fault}` as TKey)}</option>)}
        </select>
      </div>}
    </div>}
  </div>;
}

/** One line about a rule for its collapsed card. */
function ruleSummary(emulator: Emulator, index: number, t: ReturnType<typeof useT>): string {
  switch (emulator.protocol) {
    case "http": {
      const route = emulator.routes[index];
      const answers = route.responses.map((response) => response.fault === "none" ? String(response.status) : t(`emu.fault.${response.fault}` as TKey)).join(", ");
      return `${route.method.toUpperCase() === "ANY" ? "*" : route.method.toUpperCase()} ${route.path} → ${answers}`;
    }
    case "osc": {
      const rule = emulator.rules[index];
      return `${rule.address} → ${rule.reply ? rule.reply.address : "—"}`;
    }
    case "udp": case "tcp": {
      const rule = emulator.rules[index];
      const reply = rule.reply ? (rule.reply.kind === "text" ? rule.reply.text : rule.reply.hex) : "—";
      return `${rule.mode === "any" ? "*" : rule.pattern} → ${reply}`;
    }
    case "mqtt": {
      const rule = emulator.rules[index];
      const payload = rule.mode === "any" ? "" : ` ${rule.pattern}`;
      return `${rule.topic}${payload} → ${rule.reply ? rule.reply.topic : "—"}`;
    }
  }
}

/**
 * An emulator's settings and rules. `hits`: what each rule has taken while it
 * runs. Rules start folded, but for one: the first, or the one just added.
 */
export function EmulatorEditor({ emulator, onChange, hits }: { emulator: Emulator; onChange: (next: Emulator) => void; hits?: number[] }) {
  const t = useT();
  const fid = useFieldIds();
  const [open, setOpen] = useState<Set<number>>(() => new Set([0]));
  const toggle = (index: number) => setOpen((prior) => { const next = new Set(prior); if (next.has(index)) next.delete(index); else next.add(index); return next; });
  const count = emulator.protocol === "http" ? emulator.routes.length : emulator.rules.length;
  const http = emulator.protocol === "http";

  /** The rules as one list, whatever the protocol, for adding, moving and removing. */
  const rulesOf = (): unknown[] => emulator.protocol === "http" ? emulator.routes : emulator.rules;
  const withRules = (rules: unknown[]): Emulator => {
    switch (emulator.protocol) {
      case "http": return { ...emulator, routes: rules as Route[] };
      case "osc": return { ...emulator, rules: rules as OscRule[] };
      case "udp": return { ...emulator, rules: rules as UdpRule[] };
      case "tcp": return { ...emulator, rules: rules as TcpRule[] };
      case "mqtt": return { ...emulator, rules: rules as MqttRule[] };
    }
  };
  const blank: Record<Emulator["protocol"], () => unknown> = { http: blankRoute, osc: blankOscRule, udp: blankUdpRule, tcp: blankTcpRule, mqtt: blankMqttRule };
  // Where a person reaches it: the server's name in a browser, this computer's loopback in the app.
  const base = emulatorUrl(emulator, undefined, isDesktop ? undefined : window.location.hostname);
  const add = () => {
    const rule = blank[emulator.protocol]();
    onChange(withRules([...rulesOf(), rule]));
    setOpen(new Set([count]));
  };
  const move = (index: number, by: -1 | 1) => {
    onChange(withRules(moved(rulesOf(), index, by)));
    setOpen((prior) => new Set([...prior].map((at) => at === index ? index + by : at === index + by ? index : at)));
  };
  const remove = (index: number) => {
    onChange(withRules(removed(rulesOf(), index)));
    setOpen((prior) => new Set([...prior].filter((at) => at !== index).map((at) => at > index ? at - 1 : at)));
  };
  const setRule = (index: number, rule: unknown) => onChange(withRules(replaced(rulesOf(), index, rule)));

  return <div className="emu-editor">
    <div className="row">
      <div className="field">
        <label htmlFor={fid("name")}>{t("emu.name")}</label>
        <input id={fid("name")} value={emulator.name} onChange={(event) => onChange({ ...emulator, name: event.target.value })} />
      </div>
      <div className="field">
        <label htmlFor={fid("bind")} data-tip={t("emu.bindHint")}>{t("emu.bind")}</label>
        <input id={fid("bind")} spellCheck={false} value={emulator.bind} onChange={(event) => onChange({ ...emulator, bind: event.target.value.trim() })} />
      </div>
      <span className="sig-badge emu-protocol">{protocolLabel(emulator.protocol)}</span>
    </div>
    {emulator.protocol === "tcp" && <div className="row">
      <div className="field">
        <label htmlFor={fid("delimiter")} data-tip={t("emu.delimiterHint")}>{t("emu.delimiter")}</label>
        <select id={fid("delimiter")} value={emulator.delimiter} onChange={(event) => onChange({ ...emulator, delimiter: event.target.value as Delimiter })}>
          {DELIMITERS.map((delimiter) => <option key={delimiter} value={delimiter}>{t(`emu.delimiter.${delimiter}` as TKey)}</option>)}
        </select>
      </div>
      <div className="field">
        <label htmlFor={fid("greeting")} data-tip={t("emu.greetingHint")}>{t("emu.greeting")}</label>
        <input id={fid("greeting")} spellCheck={false} value={emulator.greeting} onChange={(event) => onChange({ ...emulator, greeting: event.target.value })} />
      </div>
    </div>}

    {emulator.protocol === "mqtt" && <BrokerFields emulator={emulator} onChange={onChange} />}

    <div className="emu-rules-head">
      <p className="section-label" data-tip={t(http ? "emu.routesHint" : emulator.protocol === "mqtt" ? "emu.mqttRulesHint" : "emu.rulesHint")}>{t(http ? "emu.routes" : "emu.rules")}</p>
      <button className="ghost sm" disabled={count >= 64} onClick={add}>＋ {t(http ? "emu.addRoute" : "emu.addRule")}</button>
    </div>
    {count === 0 && <div className="empty-state">{t("emu.noRules")}</div>}
    {Array.from({ length: count }, (_, index) => {
      const expanded = open.has(index);
      const n = index + 1;
      const prefix = `rule${index}-`;
      return <section className={`emu-rule ${expanded ? "open" : ""}`} key={index} aria-label={t("emu.rule", { n })}>
        <div className="emu-rule-head">
          <button className="emu-rule-toggle" aria-expanded={expanded} data-tip={t(expanded ? "emu.collapse" : "emu.expand")} onClick={() => toggle(index)}>
            <span className="emu-rule-twist" aria-hidden="true">{expanded ? "▾" : "▸"}</span>
            <span className="emu-rule-n">#{n}</span>
            <span className="emu-rule-summary">{ruleSummary(emulator, index, t)}</span>
          </button>
          {hits && <span className="emu-hits" data-tip={t("emu.ruleHits", { n: hits[index] ?? 0 })}>{hits[index] ?? 0}</span>}
          <button className="ghost xs" disabled={index === 0} aria-label={`${t("emu.moveUp")} · ${n}`} data-tip={t("emu.moveUp")} onClick={() => move(index, -1)}>↑</button>
          <button className="ghost xs" disabled={index === count - 1} aria-label={`${t("emu.moveDown")} · ${n}`} data-tip={t("emu.moveDown")} onClick={() => move(index, 1)}>↓</button>
          <button className="ghost xs" aria-label={`${t("emu.removeRule")} · ${n}`} data-tip={t("emu.removeRule")} onClick={() => remove(index)}>×</button>
        </div>
        {expanded && <div className="emu-rule-body">
          {emulator.protocol === "http" && <RouteBody route={emulator.routes[index]} onChange={(route) => setRule(index, route)} idPrefix={prefix} base={base} />}
          {emulator.protocol === "osc" && <OscRuleBody rule={emulator.rules[index]} onChange={(rule) => setRule(index, rule)} idPrefix={prefix} />}
          {emulator.protocol === "udp" && <UdpRuleBody rule={emulator.rules[index]} onChange={(rule) => setRule(index, rule)} idPrefix={prefix} />}
          {emulator.protocol === "tcp" && <TcpRuleBody rule={emulator.rules[index]} onChange={(rule) => setRule(index, rule)} idPrefix={prefix} />}
          {emulator.protocol === "mqtt" && <MqttRuleBody rule={emulator.rules[index]} onChange={(rule) => setRule(index, rule)} idPrefix={prefix} />}
        </div>}
      </section>;
    })}

    {emulator.protocol === "http" && <div className="emu-fallback">
      <div className="field">
        <label htmlFor={fid("fallback")}>{t("emu.fallback")}</label>
        <select id={fid("fallback")} value={emulator.fallback ? "custom" : "default"}
          onChange={(event) => onChange({ ...emulator, fallback: event.target.value === "custom" ? presetResponse("notFound") : null })}>
          <option value="default">{t("emu.fallbackDefault")}</option>
          <option value="custom">{t("emu.fallbackCustom")}</option>
        </select>
      </div>
      {emulator.fallback && <ResponseEditor response={emulator.fallback} random={false} idPrefix="fallback-" onChange={(fallback) => onChange({ ...emulator, fallback })} />}
    </div>}
    <OutageFields emulator={emulator} onChange={onChange} />
  </div>;
}
