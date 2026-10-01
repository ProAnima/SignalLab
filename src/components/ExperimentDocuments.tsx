import { useEffect, useRef, useState } from "react";
import { api, type Experiment } from "../lib/api";
import { experimentTemplates } from "../lib/experimentTemplates";
import { useT } from "../lib/i18n";
import type { Failure } from "../lib/errors";
import { ErrorMessage } from "./ErrorMessage";

interface Props {
  document: Experiment | null;
  onOpen: (document: Experiment) => void;
  onClose: () => void;
}

/** File/template chooser. It only commits a validated preview after Open. */
export function ExperimentDocuments({ document, onOpen, onClose }: Props) {
  const t = useT();
  const dialog = useRef<HTMLDialogElement>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  const mounted = useRef(false);
  const [templateId, setTemplateId] = useState("empty");
  const [imported, setImported] = useState<{ name: string; document: Experiment } | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<Failure | null>(null);
  const [importFailed, setImportFailed] = useState(false);
  const [exportPath, setExportPath] = useState("");
  const template = experimentTemplates.find((item) => item.id === templateId)!;
  const preview = imported?.document ?? template.document;

  useEffect(() => {
    mounted.current = true;
    const element = dialog.current!;
    element.showModal();
    return () => { mounted.current = false; element.close(); };
  }, []);

  const importFile = async (file: File) => {
    setBusy(true); setError(null); setImportFailed(true);
    try {
      // Checked here too, so a huge file is never read into memory.
      if (file.size > 4 * 1024 * 1024) throw { code: "file.too_large", params: { max: "4" } };
      const parsed = await api.experimentParse(await file.text());
      if (mounted.current) { setImported({ name: file.name, document: parsed }); setImportFailed(false); }
    } catch (reason) { if (mounted.current) setError(reason ?? "?"); }
    finally { if (mounted.current) setBusy(false); }
  };

  const open = async () => {
    setBusy(true); setError(null);
    try {
      const next = imported?.document ?? { ...template.document, name: t(template.title) };
      // Templates and imported files share the engine's schema checks.
      const parsed = await api.experimentParse(JSON.stringify(next));
      if (mounted.current) onOpen(parsed);
    } catch (reason) { if (mounted.current) setError(reason ?? "?"); }
    finally { if (mounted.current) setBusy(false); }
  };

  const exportCurrent = async () => {
    if (!document) return;
    setBusy(true); setError(null); setExportPath("");
    try {
      const path = await api.experimentExport(document);
      if (mounted.current) setExportPath(path);
    } catch (reason) { if (mounted.current) setError(reason ?? "?"); }
    finally { if (mounted.current) setBusy(false); }
  };

  return <dialog ref={dialog} className="experiment-documents" role="dialog" aria-labelledby="experiment-documents-title"
    onCancel={(event) => { event.preventDefault(); onClose(); }} onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <div className="experiment-documents-content">
      <header><div><h2 id="experiment-documents-title">{t("exp.documents")}</h2><p>{t("exp.documentsHint")}</p></div>
        <button className="ghost sm" aria-label={t("exp.close")} onClick={onClose}>×</button>
      </header>
      <div className="experiment-template-list" role="group" aria-label={t("exp.templates")}>
        {experimentTemplates.map((item, index) => <button key={item.id} className={`experiment-template ${!imported && templateId === item.id ? "selected" : ""}`}
          aria-pressed={!imported && templateId === item.id} disabled={busy} onClick={() => { setTemplateId(item.id); setImported(null); setError(null); setImportFailed(false); }}>
          <span className="experiment-template-number">0{index + 1}</span><strong>{t(item.title)}</strong><span>{t(item.description)}</span>
        </button>)}
      </div>
      <div className="experiment-file-import">
        <input ref={fileInput} type="file" accept=".json,application/json" hidden onChange={(event) => {
          const file = event.target.files?.[0]; event.target.value = ""; if (file) void importFile(file);
        }} />
        <button className="ghost sm" disabled={busy} onClick={() => fileInput.current?.click()}>{t("exp.importJson")}</button>
        <span title={imported?.name}>{imported?.name ?? t("exp.importHint")}</span>
      </div>
      <div className="experiment-document-preview" aria-live="polite">
        <strong>{imported ? preview.name : t(template.title)}</strong>
        <span>{t("exp.documentSize", { nodes: preview.nodes.length, edges: preview.edges.length })}</span>
      </div>
      {error !== null && <ErrorMessage className="experiment-file-error" error={error} />}
      {exportPath && <label className="experiment-export-path">{t("exp.exportSaved")}
        <input readOnly value={exportPath} onFocus={(event) => event.target.select()} />
      </label>}
      <footer>
        <button className="ghost sm" disabled={!document || busy} onClick={exportCurrent}>{t("exp.exportJson")}</button>
        <button className="primary" disabled={busy || importFailed} onClick={open}>{busy ? t("exp.fileWorking") : t("exp.openDocument")}</button>
      </footer>
      <p className="experiment-documents-note">{document ? t("exp.replaceHint") : t("exp.newHint")}</p>
    </div>
  </dialog>;
}
