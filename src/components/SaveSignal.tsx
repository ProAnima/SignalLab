import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import type { Signal, SignalBody } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { usePersistentState } from "../lib/hooks";
import { allFolders, canonicalBody, normalizeFolder, signalPlace } from "../lib/library";
import { makeId } from "../lib/signals";

/**
 * Ctrl+S inside a sender saves it, as the Save button there would: an update
 * when it is bound and changed, the dialog when it is not yet in the library.
 */
export function saveShortcut(event: KeyboardEvent<HTMLElement>) {
  if (!(event.ctrlKey || event.metaKey) || event.code !== "KeyS" || event.shiftKey || event.altKey) return;
  event.preventDefault();
  event.currentTarget.querySelector<HTMLButtonElement>("[data-save-signal]:not(:disabled)")?.click();
}

/**
 * What a sender (HTTP, OSC, MQTT) needs to keep its message in the signal
 * library. Once saved — or opened from the library — the sender is bound to
 * that signal: Save updates it, Save as… makes another, and the chip says
 * where it lives and opens it there. `signalId` is the sender's to keep, so
 * the binding survives a restart.
 */
export function SaveSignal({ body, signalId, onSignalId, suggestName, onShow }: {
  /** What the sender would send right now. */
  body: SignalBody;
  signalId: string | null;
  onSignalId: (id: string | null) => void;
  /** A name for a new signal, from what is being sent ("GET /api/login"). */
  suggestName: string;
  /** Open a signal in the library. */
  onShow?: (id: string) => void;
}) {
  const t = useT();
  const { library, setLibrary, pushLog } = useStore();
  const [asking, setAsking] = useState(false);
  const bound = signalId ? library.find((signal) => signal.id === signalId) ?? null : null;
  const changed = !!bound && canonicalBody(bound.body) !== canonicalBody(body);

  const update = () => {
    if (!bound) { setAsking(true); return; }
    if (!changed) return;
    setLibrary(library.map((signal) => signal.id === bound.id ? { ...signal, body: structuredClone(body) } : signal));
    pushLog("ok", "signals", "log.signalUpdated", { name: signalPlace(bound) });
  };

  return <span className="save-signal">
    {bound
      ? <button className="ghost" disabled={!changed} data-tip={changed ? `${t("sig.saveHint", { name: signalPlace(bound) })} · Ctrl+S` : t("sig.savedHint")} onClick={update} data-save-signal>
        {changed ? t("sig.save") : `✓ ${t("sig.savedState")}`}</button>
      : <button className="ghost" data-tip={`${t("sig.saveNewHint")} · Ctrl+S`} onClick={() => setAsking(true)} data-save-signal>{t("sig.saveNew")}</button>}
    {bound && <button className="ghost sm" data-tip={t("sig.saveAsHint")} onClick={() => setAsking(true)}>{t("sig.saveAs")}</button>}
    {bound && <button className="link-btn save-signal-chip" data-tip={t("sig.showInLibrary")} aria-label={`${t("sig.showInLibrary")}: ${signalPlace(bound)}`}
      onClick={() => onShow?.(bound.id)}>❖ {signalPlace(bound)}{changed && <em> · {t("sig.changed")}</em>}</button>}
    {asking && <SaveDialog body={body} suggestName={bound?.name ?? suggestName} folder={bound?.group}
      onClose={() => setAsking(false)} onSaved={(signal) => { setAsking(false); onSignalId(signal.id); }} />}
  </span>;
}

/** Name and folder for a new signal; the folder can be a new one ("API/Auth"). */
function SaveDialog({ body, suggestName, folder, onClose, onSaved }: {
  body: SignalBody;
  suggestName: string;
  folder?: string;
  onClose: () => void;
  onSaved: (signal: Signal) => void;
}) {
  const t = useT();
  const { library, folders, setLibrary, pushLog } = useStore();
  // Where the last one went is where the next one usually goes.
  const [lastFolder, setLastFolder] = usePersistentState("signal-lab.save.folder", "");
  const [name, setName] = useState(suggestName);
  const [place, setPlace] = useState(folder ?? lastFolder);
  const dialog = useRef<HTMLDialogElement>(null);
  const nameField = useRef<HTMLInputElement>(null);
  const id = useId();

  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    nameField.current?.select();
    return () => element.close();
  }, []);

  const save = () => {
    const clean = name.trim();
    if (!clean) { nameField.current?.focus(); return; }
    const group = normalizeFolder(place);
    const signal: Signal = { id: makeId(clean, library), name: clean, group, note: "", body: structuredClone(body) };
    setLibrary([...library, signal]);
    setLastFolder(group);
    pushLog("ok", "signals", "log.signalSaved", { name: signalPlace(signal) });
    onSaved(signal);
  };

  return <dialog ref={dialog} className="save-signal-dialog" aria-labelledby={`${id}title`}
    onCancel={(event) => { event.preventDefault(); onClose(); }} onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <form method="dialog" onSubmit={(event) => { event.preventDefault(); save(); }}>
      <h2 id={`${id}title`}>{t("sig.saveTitle")}</h2>
      <div className="field">
        <label htmlFor={`${id}name`}>{t("sig.name")}</label>
        <input id={`${id}name`} ref={nameField} value={name} required onChange={(event) => setName(event.target.value)} />
      </div>
      <div className="field">
        <label htmlFor={`${id}folder`} data-tip={t("sig.folderHint")}>{t("sig.group")}</label>
        <input id={`${id}folder`} list={`${id}folders`} value={place} placeholder={t("sig.topLevel")} onChange={(event) => setPlace(event.target.value)} />
        <datalist id={`${id}folders`}>{allFolders(library, folders).map((path) => <option key={path} value={path} />)}</datalist>
      </div>
      <footer className="btn-row">
        <button type="button" className="ghost" onClick={onClose}>{t("common.cancel")}</button>
        <button type="submit" className="primary">{t("sig.saveConfirm")}</button>
      </footer>
    </form>
  </dialog>;
}
