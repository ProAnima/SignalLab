import { useEffect, useId, useRef, useState } from "react";
import { useStore } from "../lib/store";
import { useI18n } from "../lib/i18n";
import { useUpdates } from "../lib/updates";
import { copyText, openDocs, openExternal } from "../lib/platform";
import { START_PAGE } from "../lib/docs";
import { fmtBytes } from "../lib/format";
import { BrandMark } from "./Brand";

export const CONTACT = "info@proanima.net";
export const REPOSITORY = "https://github.com/ProAnima/SignalLab";

/**
 * Who makes Signal Lab and how to reach them, and — in the desktop app — its
 * updates: the newest published release, what is new in it, and installing
 * it, which saves what is pending, stops every job and restarts the app.
 */
export function AboutDialog({ onClose, onFeedback }: { onClose: () => void; onFeedback: () => void }) {
  const { t, lang } = useI18n();
  const { info, jobs } = useStore();
  const updates = useUpdates();
  const id = useId();
  const dialog = useRef<HTMLDialogElement>(null);
  const [copied, setCopied] = useState(false);
  const [notes, setNotes] = useState(false);

  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);

  const { state, found } = updates;
  const working = state.phase === "downloading" || state.phase === "installing";
  const version = info?.version ?? __APP_VERSION__;

  return <dialog ref={dialog} className="save-signal-dialog about-dialog" aria-labelledby={`${id}title`}
    onCancel={(event) => { event.preventDefault(); if (!working) onClose(); }}
    onClick={(event) => { if (event.target === event.currentTarget && !working) onClose(); }}>
    <div className="about-body">
      <header className="about-head">
        <BrandMark size={44} />
        <div>
          <h2 id={`${id}title`}>{t("app.name")}</h2>
          <p className="about-version">{t("about.version", { version })} · {t("about.tagline")}</p>
        </div>
      </header>

      <dl className="about-facts">
        <dt>{t("about.developer")}</dt>
        <dd>{t("about.studio")}</dd>
        <dt>{t("about.author")}</dt>
        <dd>{t("about.authorName")}</dd>
        <dt>{t("about.contact")}</dt>
        <dd className="about-contact">
          <button type="button" className="link-btn" onClick={() => void openExternal(`mailto:${CONTACT}`)}>{CONTACT}</button>
          <button type="button" className="ghost sm" aria-label={t("about.copyEmail")} data-tip={t("about.copyEmail")}
            onClick={() => void copyText(CONTACT).then(setCopied)}>{copied ? t("about.copied") : "⧉"}</button>
        </dd>
        <dt>{t("about.source")}</dt>
        <dd><button type="button" className="link-btn" onClick={() => void openExternal(REPOSITORY)}>github.com/ProAnima/SignalLab</button></dd>
        <dt>{t("about.license")}</dt>
        <dd>MIT · © 2026 ProAnimaStudio</dd>
      </dl>

      <section className="about-updates" aria-labelledby={`${id}updates`}>
        <h3 id={`${id}updates`}>{t("update.title")}</h3>
        {!updates.supported ? <p className="about-note" data-tip={t("update.serverHint")}>{t("update.server")}</p> : <>
          <p className="about-update-state" role="status">
            {state.phase === "checking" && t("update.checking")}
            {state.phase === "current" && t("update.current")}
            {state.phase === "downloading" && (state.total
              ? t("update.downloading", { percent: Math.floor((state.received / state.total) * 100) })
              : t("update.downloadingBytes", { size: fmtBytes(state.received) }))}
            {state.phase === "installing" && t("update.installing")}
            {(state.phase === "available" || (state.phase === "failed" && state.during === "install")) && found && t("update.available", { version: found.version })}
          </p>
          {state.phase === "downloading" && <progress className="about-progress" max={state.total ?? undefined} value={state.total ? state.received : undefined}
            aria-label={t("update.title")} />}
          {state.phase === "failed" && <div className="error-message" role="alert">
            <p><span>{t(state.during === "check" ? "update.checkFailed" : "update.installFailed")}</span></p>
            <details className="error-detail"><summary>{t("err.details")}</summary><code>{state.detail}</code></details>
          </div>}
          {found && !working && found.notes && <div className="about-notes">
            <button type="button" className="ghost sm" aria-expanded={notes} onClick={() => setNotes(!notes)}>{t("update.notes")}</button>
            {notes && <pre className="feedback-preview" tabIndex={0} aria-label={t("update.notes")}>{found.notes}</pre>}
          </div>}
          <div className="btn-row">
            {found && !working && <button type="button" className="primary" onClick={() => void updates.install()}>
              {jobs.length > 0 ? t("update.installStop", { n: jobs.length }) : t("update.install")}
            </button>}
            {!working && <button type="button" className="ghost" disabled={state.phase === "checking"} onClick={() => void updates.check()}>{t("update.check")}</button>}
          </div>
          <label className="checkbox" data-tip={t("update.autoHint")}>
            <input type="checkbox" checked={updates.auto} onChange={(event) => updates.setAuto(event.target.checked)} disabled={working} />
            {t("update.auto")}
          </label>
        </>}
      </section>

      <footer className="btn-row">
        <button type="button" className="ghost" onClick={() => void openDocs(START_PAGE, lang)} data-tip={t("app.docsHint")}>{t("app.docs")}</button>
        <button type="button" className="ghost" onClick={onFeedback} disabled={working}>{t("about.writeUs")}</button>
        <span className="spacer" />
        <button type="button" className="primary" onClick={onClose} disabled={working}>{t("common.close")}</button>
      </footer>
    </div>
  </dialog>;
}
