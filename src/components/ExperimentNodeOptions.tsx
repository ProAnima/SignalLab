import type { ExperimentNode, OscReply, Repeat, Retry, UdpMode, UdpReply } from "../lib/api";
import { DEFAULT_REPEAT, DEFAULT_RETRY, defaultReply } from "../lib/experimentData";
import { useT, type TKey } from "../lib/i18n";
import { ArgRules } from "./ExperimentNodeFields";
import { TemplateField } from "./TemplateField";

const UDP_MODES: UdpMode[] = ["any", "contains", "regex", "hex"];
const MODE_PLACEHOLDER: Partial<Record<UdpMode, string>> = { contains: "ACK", regex: "^ACK (\\d+)$", hex: "de ad be ef" };

type Patch = (change: Partial<ExperimentNode>) => void;

/**
 * "Wait for a reply" on an OSC message or a UDP datagram: the same step then
 * sends from the port it listens on and passes only with a matching answer.
 */
export function ReplyFields({ node, patch }: { node: Extract<ExperimentNode, { type: "osc" | "udp" }>; patch: Patch }) {
  const t = useT();
  const toggle = (on: boolean) => node.type === "osc"
    ? patch({ reply: on ? defaultReply("osc") : undefined })
    : patch({ reply: on ? defaultReply("udp") : undefined });
  return <div className="experiment-option">
    <label className="checkbox experiment-check" data-tip={t("exp.expectReplyHint")}>
      <input type="checkbox" checked={!!node.reply} onChange={(event) => toggle(event.target.checked)} />{t("exp.expectReply")}
    </label>
    {node.type === "osc" && node.reply && <OscReplyFields reply={node.reply} onChange={(reply) => patch({ reply })} />}
    {node.type === "udp" && node.reply && <UdpReplyFields reply={node.reply} onChange={(reply) => patch({ reply })} />}
  </div>;
}

function CommonReplyFields<R extends OscReply | UdpReply>({ reply, onChange }: { reply: R; onChange: (reply: R) => void }) {
  const t = useT();
  return <>
    <label>{t("exp.waitTimeout")}<input type="number" min="1" max="120000" value={reply.timeout_ms} onChange={(event) => onChange({ ...reply, timeout_ms: Number(event.target.value) })} /></label>
    <label>{t("exp.replyVariable")}<input value={reply.variable} spellCheck={false} onChange={(event) => onChange({ ...reply, variable: event.target.value.replace(/\s+/g, "_") })} /></label>
  </>;
}

function BindField({ bind, onChange }: { bind: string; onChange: (bind: string) => void }) {
  const t = useT();
  return <label data-tip={t("exp.replyOnHint")}>{t("exp.replyOn")}
    <input value={bind} spellCheck={false} placeholder="0.0.0.0:0" onChange={(event) => onChange(event.target.value.trim())} /></label>;
}

function OscReplyFields({ reply, onChange }: { reply: OscReply; onChange: (reply: OscReply) => void }) {
  const t = useT();
  return <div className="experiment-option-fields">
    <BindField bind={reply.bind} onChange={(bind) => onChange({ ...reply, bind })} />
    <label data-tip={t("exp.addressPatternHint")}>{t("exp.replyAddress")}<TemplateField value={reply.address} onChange={(address) => onChange({ ...reply, address })} /></label>
    <ArgRules rules={reply.args} onChange={(args) => onChange({ ...reply, args })} />
    <CommonReplyFields reply={reply} onChange={onChange} />
  </div>;
}

function UdpReplyFields({ reply, onChange }: { reply: UdpReply; onChange: (reply: UdpReply) => void }) {
  const t = useT();
  return <div className="experiment-option-fields">
    <BindField bind={reply.bind} onChange={(bind) => onChange({ ...reply, bind })} />
    <label>{t("exp.replyMode")}<select value={reply.mode} onChange={(event) => onChange({ ...reply, mode: event.target.value as UdpMode })}>
      {UDP_MODES.map((mode) => <option key={mode} value={mode}>{t(`exp.mode.${mode}` as TKey)}</option>)}</select></label>
    {reply.mode !== "any" && <label>{t("field.pattern")}<TemplateField value={reply.pattern} placeholder={MODE_PLACEHOLDER[reply.mode]} onChange={(pattern) => onChange({ ...reply, pattern })} /></label>}
    <CommonReplyFields reply={reply} onChange={onChange} />
  </div>;
}

/** Send again and again, for the steps that send: a number of times or for a time. */
export function RepeatFields({ repeat, patch }: { repeat: Repeat | undefined; patch: Patch }) {
  const t = useT();
  const change = (next: Partial<Repeat>) => repeat && patch({ repeat: { ...repeat, ...next } });
  return <div className="experiment-option">
    <label className="checkbox experiment-check" data-tip={t("exp.repeatHint")}>
      <input type="checkbox" checked={!!repeat} onChange={(event) => patch({ repeat: event.target.checked ? { ...DEFAULT_REPEAT } : undefined })} />{t("exp.repeatOn")}
    </label>
    {repeat && <div className="experiment-option-fields experiment-repeat-fields">
      <label>{t("exp.repeatBy")}<select value={repeat.until} onChange={(event) => change({ until: event.target.value as Repeat["until"] })}>
        <option value="count">{t("exp.repeatBy.count")}</option>
        <option value="duration">{t("exp.repeatBy.duration")}</option>
      </select></label>
      {repeat.until === "count"
        ? <label>{t("exp.repeatCount")}<input type="number" min="2" max="10000" value={repeat.count} onChange={(event) => change({ count: Number(event.target.value) })} /></label>
        : <label>{t("exp.repeatDuration")}<input type="number" min="1" max="300000" step="1000" value={repeat.duration_ms} onChange={(event) => change({ duration_ms: Number(event.target.value) })} /></label>}
      <label>{t("exp.repeatInterval")}<input type="number" min="10" max="60000" step="100" value={repeat.interval_ms} onChange={(event) => change({ interval_ms: Number(event.target.value) })} /></label>
      <label data-tip={t("exp.repeatJitterHint")}>{t("exp.repeatJitter")}<input type="number" min="0" max="60000" step="10" value={repeat.jitter_ms} onChange={(event) => change({ jitter_ms: Number(event.target.value) })} /></label>
    </div>}
  </div>;
}

/** Retry on failure, for the steps that send or listen. */
export function RetryFields({ retry, patch }: { retry: Retry | undefined; patch: Patch }) {
  const t = useT();
  const change = (next: Partial<Retry>) => retry && patch({ retry: { ...retry, ...next } });
  return <div className="experiment-option">
    <label className="checkbox experiment-check" data-tip={t("exp.retryHint")}>
      <input type="checkbox" checked={!!retry} onChange={(event) => patch({ retry: event.target.checked ? { ...DEFAULT_RETRY } : undefined })} />{t("exp.retryOn")}
    </label>
    {retry && <div className="experiment-option-fields experiment-retry-fields">
      <label>{t("exp.attempts")}<input type="number" min="2" max="10" value={retry.attempts} onChange={(event) => change({ attempts: Number(event.target.value) })} /></label>
      <label>{t("exp.retryDelay")}<input type="number" min="0" max="60000" step="100" value={retry.delay_ms} onChange={(event) => change({ delay_ms: Number(event.target.value) })} /></label>
      <label>{t("exp.backoff")}<select value={retry.backoff ?? "fixed"} onChange={(event) => change({ backoff: event.target.value as Retry["backoff"] })}>
        <option value="fixed">{t("exp.backoff.fixed")}</option>
        <option value="exponential">{t("exp.backoff.exponential")}</option>
      </select></label>
    </div>}
  </div>;
}
