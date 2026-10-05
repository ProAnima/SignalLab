import { useEffect, useId, useMemo, useRef, useState } from "react";
import { api } from "../lib/api";
import { logText, useStore, type LogEntry } from "../lib/store";
import { translator, useI18n } from "../lib/i18n";
import { usePersistentState } from "../lib/hooks";
import { fmtBytes } from "../lib/format";
import { clock, IMAGE_TYPES, LIMITS, metaOf, readImage, refusalOf, scrub, systemText, type Picked } from "../lib/feedback";
import type { HostInfo } from "../lib/api";
import type { Failure } from "../lib/errors";
import { ErrorMessage } from "./ErrorMessage";

/** The console as text, one entry a line, in English whatever the interface speaks, scrubbed. */
function consoleText(entries: LogEntry[], where: { host: HostInfo | null; dataDir?: string }): string {
  const t = translator("en");
  const lines = entries.map((entry) => `${clock(entry.ts)}  ${entry.level.padEnd(4)}  ${entry.tag.padEnd(10)}  ${logText(entry, t)}`);
  return scrub(lines.join("\n") + "\n", where);
}

/**
 * A message to the developers, mailed by the studio's hub (docs/develop/hub.md): the
 * text, an address for the answer, screenshots —
 * pasted with Ctrl+V anywhere in the dialog, picked or dropped — and the logs
 * that help, attached on their own, each shown and removable before sending.
 * The text and the address are kept while it is not sent.
 */
export function FeedbackDialog({ onClose }: { onClose: () => void }) {
  const { t, lang } = useI18n();
  const { info, host, jobs, log, connection, pushLog } = useStore();
  const id = useId();
  const dialog = useRef<HTMLDialogElement>(null);
  const picker = useRef<HTMLInputElement>(null);
  const [message, setMessage] = usePersistentState("signal-lab.feedback.draft", "");
  const [email, setEmail] = usePersistentState("signal-lab.feedback.email", "");
  const [pictures, setPictures] = useState<Picked[]>([]);
  const [refused, setRefused] = useState<{ key: string; params: Record<string, string | number> } | null>(null);
  const [withConsole, setWithConsole] = useState(true);
  const [withSystem, setWithSystem] = useState(true);
  const [shown, setShown] = useState<"console" | "system" | null>(null);
  const [sending, setSending] = useState(false);
  const [failure, setFailure] = useState<Failure | null>(null);
  const [sent, setSent] = useState<string | null>(null);

  // The logs as they are when the dialog opens: what is shown is what is sent.
  const surroundings = { info, lang, jobs, connection };
  const consoleLog = useMemo(() => consoleText(log, { host, dataDir: info?.data_dir }), []); // eslint-disable-line react-hooks/exhaustive-deps
  const system = useMemo(() => systemText(surroundings), []); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);

  const add = async (files: File[], pasted: boolean) => {
    let next = pictures;
    for (const file of files) {
      const why = refusalOf(file, next);
      if (why) { setRefused(why); return; }
      try {
        next = [...next, await readImage(file, pasted)];
      } catch (error) {
        setFailure(error ?? "?");
        return;
      }
    }
    setRefused(null);
    setPictures(next);
  };

  const remove = (picked: Picked) => {
    setPictures(pictures.filter((item) => item.id !== picked.id));
    setRefused(null);
  };

  const images = (list: DataTransfer | null) => list ? [...list.files].filter((file) => file.type.startsWith("image/")) : [];

  const send = async () => {
    if (sending || !message.trim()) return;
    setSending(true);
    setFailure(null);
    try {
      const logs = [
        ...(withConsole ? [{ name: "signallab-console.txt", text: consoleLog }] : []),
        ...(withSystem ? [{ name: "signallab-system.json", text: system }] : []),
      ];
      const answer = await api.feedbackSend({
        message: message.trim(),
        email: email.trim() || undefined,
        meta: metaOf(surroundings),
        screenshots: pictures.map((picked) => ({ name: picked.name, data: picked.data })),
        logs,
      });
      setSent(answer.id);
      setMessage("");
      setPictures([]);
      pushLog("ok", "feedback", "log.feedbackSent", { id: answer.id });
    } catch (error) {
      setFailure(error ?? "?");
    } finally {
      setSending(false);
    }
  };

  const total = pictures.reduce((sum, picked) => sum + picked.size, 0);

  return <dialog ref={dialog} className="save-signal-dialog feedback-dialog" aria-labelledby={`${id}title`}
    onCancel={(event) => { event.preventDefault(); if (!sending) onClose(); }}
    onPaste={(event) => {
      const files = images(event.clipboardData);
      if (!files.length) return;
      event.preventDefault();
      void add(files, true);
    }}
    onDragOver={(event) => { if (event.dataTransfer.types.includes("Files")) event.preventDefault(); }}
    onDrop={(event) => {
      const files = images(event.dataTransfer);
      if (!files.length) return;
      event.preventDefault();
      void add(files, false);
    }}>
    <form method="dialog" onSubmit={(event) => { event.preventDefault(); void send(); }}>
      <h2 id={`${id}title`}>{t("feedback.title")}</h2>
      {sent ? <>
        <p className="feedback-sent" role="status">{t("feedback.sent", { id: sent })}</p>
        <footer className="btn-row">
          <button type="button" className="primary" onClick={onClose}>{t("common.close")}</button>
        </footer>
      </> : <>
        <div className="field">
          <label htmlFor={`${id}message`} data-tip={t("feedback.messageHint")}>{t("feedback.message")}</label>
          <textarea id={`${id}message`} value={message} rows={6} maxLength={LIMITS.messageChars} autoFocus
            onChange={(event) => setMessage(event.target.value)}
            onKeyDown={(event) => { if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) { event.preventDefault(); void send(); } }} />
        </div>
        <div className="field">
          <label htmlFor={`${id}email`} data-tip={t("feedback.emailHint")}>{t("feedback.email")}</label>
          <input id={`${id}email`} type="email" autoComplete="email" spellCheck={false} value={email} onChange={(event) => setEmail(event.target.value)} />
        </div>

        <div className="field" role="group" aria-labelledby={`${id}shots`}>
          <span id={`${id}shots`} className="feedback-label" data-tip={t("feedback.screenshotsHint", { n: LIMITS.images, mb: LIMITS.imageBytes >> 20 })}>
            {t("feedback.screenshots")}{pictures.length > 0 && <span className="feedback-size"> · {fmtBytes(total)}</span>}
          </span>
          <div className="feedback-shots">
            {pictures.map((picked) => <figure key={picked.id} className="feedback-shot">
              <img src={picked.url} alt={picked.name} />
              <figcaption>{picked.name}</figcaption>
              <button type="button" className="ghost sm feedback-remove" aria-label={t("feedback.removeScreenshot", { name: picked.name })}
                data-tip={t("feedback.removeScreenshot", { name: picked.name })} onClick={() => remove(picked)}>✕</button>
            </figure>)}
            {pictures.length < LIMITS.images && <button type="button" className="ghost feedback-add" onClick={() => picker.current?.click()}>
              {t("feedback.addScreenshot")}
            </button>}
          </div>
          <input ref={picker} type="file" accept={IMAGE_TYPES.join(",")} multiple hidden aria-label={t("feedback.addScreenshot")}
            onChange={(event) => { const files = [...(event.target.files ?? [])]; event.target.value = ""; void add(files, false); }} />
          {refused && <p className="feedback-refused" role="alert">{t(refused.key, refused.params)}</p>}
        </div>

        <div className="field" role="group" aria-labelledby={`${id}logs`}>
          <span id={`${id}logs`} className="feedback-label" data-tip={t("feedback.logsHint")}>{t("feedback.logs")}</span>
          <div className="feedback-log">
            <label className="checkbox"><input type="checkbox" checked={withConsole} onChange={(event) => setWithConsole(event.target.checked)} />
              {t("feedback.consoleLog", { n: log.length })}</label>
            <button type="button" className="ghost sm" aria-expanded={shown === "console"} onClick={() => setShown(shown === "console" ? null : "console")}>
              {t(shown === "console" ? "feedback.hide" : "feedback.show")}</button>
          </div>
          {shown === "console" && <pre className="feedback-preview" tabIndex={0} aria-label={t("feedback.consoleLog", { n: log.length })}>{consoleLog}</pre>}
          <div className="feedback-log">
            <label className="checkbox"><input type="checkbox" checked={withSystem} onChange={(event) => setWithSystem(event.target.checked)} />
              {t("feedback.systemInfo")}</label>
            <button type="button" className="ghost sm" aria-expanded={shown === "system"} onClick={() => setShown(shown === "system" ? null : "system")}>
              {t(shown === "system" ? "feedback.hide" : "feedback.show")}</button>
          </div>
          {shown === "system" && <pre className="feedback-preview" tabIndex={0} aria-label={t("feedback.systemInfo")}>{system}</pre>}
        </div>

        {failure !== null && <ErrorMessage error={failure} />}
        <footer className="btn-row">
          <button type="button" className="ghost" onClick={onClose} disabled={sending}>{t("common.cancel")}</button>
          <button type="submit" className="primary" disabled={sending || !message.trim()} data-tip={`${t("common.send")} · Ctrl+Enter`}>
            {sending ? t("common.sending") : t("common.send")}
          </button>
        </footer>
      </>}
    </form>
  </dialog>;
}
