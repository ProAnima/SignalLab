import type { Experiment, ProfileIssue } from "../lib/api";
import type { Failure } from "../lib/errors";
import { useT } from "../lib/i18n";

/** The editor's top bar: the document, its name and state, adding, parameters, panes and Run. */
export function ExperimentToolbar({ doc, busy, running, starting, saveState, validationError, profileIssues, describe, addAfter, paramsOpen, propertiesOpen, focusMode, fullscreen,
  onDocuments, onEdit, onCommit, onRevealProblem, onAdd, onParams, onProperties, setFocusMode, onFullscreen, onRun, onStop, onRunWith }: {
  doc: Experiment;
  busy: boolean;
  /** A run is going: Run becomes Stop. */
  running: boolean;
  starting: boolean;
  saveState: "saved" | "saving" | "error";
  validationError: Failure | null;
  profileIssues: ProfileIssue[];
  /** A failure in words, located by node label. */
  describe: (failure: Failure) => string;
  /** A new node would continue the selected one's flow. */
  addAfter: boolean;
  paramsOpen: boolean;
  propertiesOpen: boolean;
  focusMode: boolean;
  fullscreen: boolean;
  onDocuments: () => void;
  onEdit: (update: (current: Experiment) => Experiment) => void;
  onCommit: () => void;
  onRevealProblem: (failure: Failure) => void;
  onAdd: () => void;
  /** Open the parameters under the button. */
  onParams: (anchor: { left: number; top: number }) => void;
  onProperties: () => void;
  setFocusMode: (value: boolean) => void;
  onFullscreen: () => void;
  onRun: () => void;
  onStop: () => void;
  /** Open Run with… under its button, right-aligned to it. */
  onRunWith: (anchor: { right: number; top: number }) => void;
}) {
  const t = useT();
  const addTitle = addAfter ? t("exp.addAfter") : t("exp.addNode");
  return <div className="experiment-toolbar">
    <button className="ghost sm experiment-document-button" data-tip={t("exp.documents")} aria-label={t("exp.documents")} disabled={busy} onClick={onDocuments}>☰</button>
    <div className="experiment-heading"><input aria-label={t("exp.name")} value={doc.name} disabled={busy} onChange={(event) => onEdit((old) => ({ ...old, name: event.target.value }))} /></div>
    <span className="experiment-save">{saveState === "saving" ? t("exp.saving") : saveState === "error" ? t("exp.saveError") : t("exp.saved")}</span>
    {validationError !== null && <button className="ghost sm experiment-validation" data-tip={describe(validationError)} onClick={() => onRevealProblem(validationError)}>⚠ {t("exp.needsLinks")}</button>}
    <button className="ghost sm" data-tip={`${addTitle} · A`} onClick={onAdd} disabled={busy}>＋ {t("exp.addNode")}</button>
    {doc.profiles.length > 0 && <select className="experiment-profile-select" aria-label={t("exp.profile")} data-tip={t("exp.profileSwitch")} disabled={busy}
      value={doc.profile ?? ""} onChange={(event) => { const profile = event.target.value || null; onCommit(); onEdit((current) => ({ ...current, profile })); }}>
      {[null, ...doc.profiles.map((profile) => profile.name)].map((name) => {
        const issue = profileIssues.find((item) => item.profile === name);
        return <option key={name ?? ""} value={name ?? ""}>{name ?? t("exp.noProfile")}{issue ? " ⚠" : ""}</option>;
      })}
    </select>}
    <button className="ghost sm experiment-params-button" aria-pressed={paramsOpen} aria-label={t("exp.params")} data-tip={t("exp.paramsHint")} onClick={(event) => { const rect = event.currentTarget.getBoundingClientRect(); onParams({ left: rect.left, top: rect.bottom + 6 }); }}>{"{ }"} <span className="label">{t("exp.params")}</span>{doc.params.length > 0 && <span className="experiment-node-count">{doc.params.length}</span>}</button>
    <button className="ghost sm" aria-pressed={propertiesOpen} data-tip={t("exp.properties")} aria-label={t("exp.properties")} onClick={onProperties}>☷</button>
    <button className="ghost sm" aria-pressed={focusMode} data-tip={focusMode ? t("exp.exitFocus") : t("exp.focus")} aria-label={focusMode ? t("exp.exitFocus") : t("exp.focus")} onClick={() => setFocusMode(!focusMode)}>{focusMode ? "▣" : "▢"}</button>
    <button className="ghost sm" aria-pressed={fullscreen} data-tip={fullscreen ? t("exp.exitFullscreen") : t("exp.fullscreen")} aria-label={fullscreen ? t("exp.exitFullscreen") : t("exp.fullscreen")} onClick={onFullscreen}>⛶</button>
    <div className="experiment-run-group">
      <button className={running ? "danger" : "primary"} disabled={starting} onClick={() => running ? onStop() : onRun()}>{running ? t("common.stop") : t("exp.run")}</button>
      {!running && <button className="primary experiment-run-more" disabled={starting} aria-label={t("exp.runMenu")} data-tip={t("exp.runWith")}
        onClick={(event) => { const rect = event.currentTarget.getBoundingClientRect(); onRunWith({ right: rect.right, top: rect.bottom + 6 }); }}>▾</button>}
    </div>
  </div>;
}
