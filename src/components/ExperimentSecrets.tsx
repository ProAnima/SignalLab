import { useEffect, useState, type KeyboardEvent } from "react";
import { api, type Experiment } from "../lib/api";
import { isIdent, secretNames } from "../lib/experimentData";
import { secretsTip } from "../lib/experimentText";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import type { Failure } from "../lib/errors";
import { ErrorMessage } from "./ErrorMessage";

/**
 * Secrets the experiment uses, with their state on this machine. Values go
 * straight to the credential store: a field is cleared once saved, and no
 * command can read a value back.
 */
export function ExperimentSecrets({ doc, onChanged }: { doc: Experiment; onChanged: (stored?: string) => void }) {
  const t = useT();
  const { pushLog, info } = useStore();
  // On a server, secrets come from its environment or files and cannot be set here.
  const readOnly = info !== null && !info.secrets_writable;
  const [extra, setExtra] = useState<string[]>([]);
  const [status, setStatus] = useState<Record<string, boolean>>({});
  const [error, setError] = useState<Failure | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const [newName, setNewName] = useState("");
  const [draft, setDraft] = useState("");
  const names = [...new Set([...secretNames(doc), ...extra])].sort();
  const key = names.join("\n");

  const refresh = () => api.secretStatus(names).then((result) => { setStatus(result); setError(null); }, (reason) => setError(reason ?? "?"));
  useEffect(() => { if (names.length) void refresh(); }, [key]);

  const close = () => { setEditing(null); setAdding(false); setDraft(""); setNewName(""); };
  const save = async (name: string) => {
    if (!draft || !isIdent(name)) return;
    try {
      await api.secretSet(name, draft);
      pushLog("ok", "experiment", "log.secretStored", { name });
      if (!names.includes(name)) setExtra((prior) => [...prior, name]);
      close();
      onChanged(name);
      await refresh();
    } catch (reason) { setError(reason ?? "?"); }
  };
  const remove = async (name: string) => {
    try {
      await api.secretDelete(name);
      pushLog("warn", "experiment", "log.secretRemoved", { name });
      onChanged();
      await refresh();
    } catch (reason) { setError(reason ?? "?"); }
  };
  // Escape cancels the edit, not the whole panel; Enter saves.
  const keys = (name: string) => (event: KeyboardEvent) => {
    if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); close(); }
    if (event.key === "Enter") { event.preventDefault(); void save(name); }
  };
  const valueInput = (name: string) => <input type="password" autoComplete="new-password" spellCheck={false} autoFocus={editing === name}
    aria-label={`${name}: ${t("exp.secretValue")}`} placeholder={t("exp.secretValue")} value={draft}
    onChange={(event) => setDraft(event.target.value)} onKeyDown={keys(name)} />;

  return <section className="experiment-secrets" aria-label={t("exp.secrets")}>
    <h3 data-tip={t(secretsTip(info))}>{t("exp.secrets")}</h3>
    {error !== null && <ErrorMessage className="experiment-file-error" error={error} />}
    {names.length === 0 && !adding && <p className="experiment-empty">{t("exp.noSecrets")}</p>}
    {readOnly && names.map((name) => <div className="experiment-secret-row" key={name}>
      <code data-tip={`{{secret.${name}}}`}>{name}</code>
      <span className={status[name] ? "stored" : "missing"}>{status[name] ? `● ${t("exp.secretOnServer")}` : `⚠ ${t("exp.secretNotOnServer")}`}</span>
    </div>)}
    {!readOnly && names.map((name) => <div className="experiment-secret-row" key={name}>
      <code data-tip={`{{secret.${name}}}`}>{name}</code>
      {editing === name ? <>
        {valueInput(name)}
        <button className="ghost sm" disabled={!draft} onClick={() => save(name)}>{t("exp.secretSave")}</button>
        <button className="ghost sm" onClick={close}>{t("exp.secretCancel")}</button>
      </> : <>
        <span className={status[name] ? "stored" : "missing"}>{status[name] ? `● ${t("exp.secretStored")}` : `⚠ ${t("exp.secretMissing")}`}</span>
        <button className="ghost sm" onClick={() => { close(); setEditing(name); }}>{status[name] ? t("exp.secretReplace") : t("exp.secretSet")}</button>
        {status[name] && <button className="ghost sm experiment-delete" onClick={() => remove(name)}>{t("exp.secretRemove")}</button>}
      </>}
    </div>)}
    {readOnly ? null : adding ? <div className="experiment-secret-row adding">
      <input autoFocus spellCheck={false} aria-label={t("exp.secretName")} placeholder={t("exp.secretName")} value={newName}
        className={newName && !isIdent(newName) ? "invalid" : undefined} onChange={(event) => setNewName(event.target.value.trim())}
        onKeyDown={(event) => { if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); close(); } }} />
      {valueInput(newName)}
      <button className="ghost sm" disabled={!draft || !isIdent(newName)} onClick={() => save(newName)}>{t("exp.secretSave")}</button>
      <button className="ghost sm" onClick={close}>{t("exp.secretCancel")}</button>
    </div> : <button className="ghost sm" onClick={() => { close(); setAdding(true); }}>＋ {t("exp.addSecret")}</button>}
  </section>;
}
