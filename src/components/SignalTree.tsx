import { useEffect, useMemo, useState, type DragEvent, type KeyboardEvent } from "react";
import type { Signal } from "../lib/api";
import { useStore } from "../lib/store";
import { useT } from "../lib/i18n";
import { usePersistentState } from "../lib/hooks";
import { allFolders, ancestors, libraryTree, moveFolder, moveSignal, removeFolder, renameFolder, within, type FolderNode } from "../lib/library";
import { signalTarget, transportKey } from "../lib/signals";

/** What is being dragged in the tree. */
type Dragged = { kind: "signal" | "folder"; id: string };
const DRAG_TYPE = "application/x-signal-lab-tree";
const isPaths = (value: unknown) => Array.isArray(value) && value.every((item) => typeof item === "string");

/**
 * The signal library as folders: open and close them (kept for next time),
 * drag signals and folders into folders — or onto the list's empty space for
 * the top level — rename a folder (✎ or F2) and remove one (× twice; what is
 * in it moves up a level). While a search is on, every folder with a match is
 * open and the empty ones are hidden.
 */
export function SignalTree({ signals, searching, selectedId, onSelect, onFire, folder, onFolder, reveal, onNewFolder }: {
  /** The signals to show (already filtered by the search). */
  signals: Signal[];
  searching: boolean;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onFire: (signal: Signal) => void;
  /** The folder new signals and folders go into: the one last picked. */
  folder: string;
  onFolder: (path: string) => void;
  /** A signal to bring into view (its folders are opened). */
  reveal?: string | null;
  /** Make a folder inside `folder`. */
  onNewFolder: () => void;
}) {
  const t = useT();
  const { library, folders, setLibraryState, pushLog } = useStore();
  const [collapsed, setCollapsed] = usePersistentState<string[]>("signal-lab.signals.collapsed", [], isPaths);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [confirming, setConfirming] = useState<string | null>(null);
  const [dragged, setDragged] = useState<Dragged | null>(null);
  const [dropAt, setDropAt] = useState<string | null>(null);
  const tree = useMemo(() => libraryTree(signals, searching ? [] : folders), [signals, folders, searching]);

  // A signal brought from elsewhere (a sender's chip) is shown: its folders open.
  useEffect(() => {
    const signal = reveal ? library.find((item) => item.id === reveal) : null;
    if (!signal) return;
    const open = ancestors(signal.group);
    setCollapsed((prior) => prior.filter((path) => !open.includes(path)));
    requestAnimationFrame(() => document.querySelector(`[data-signal="${CSS.escape(signal.id)}"]`)?.scrollIntoView({ block: "nearest" }));
  }, [reveal]); // eslint-disable-line react-hooks/exhaustive-deps

  const isOpen = (path: string) => searching || !collapsed.includes(path);
  const toggle = (path: string) => setCollapsed((prior) => prior.includes(path) ? prior.filter((item) => item !== path) : [...prior, path]);
  const state = { signals: library, folders };

  const rename = (path: string, name: string) => {
    setRenaming(null);
    const next = renameFolder(state, path, name);
    if (!next) { if (name.trim() && name.trim() !== path.split("/").pop()) pushLog("warn", "signals", "log.folderTaken", { name: name.trim() }); return; }
    if (next === state) return;
    setLibraryState(next);
    const renamed = path.replace(/[^/]*$/, name.trim());
    setCollapsed((prior) => prior.map((item) => within(item, path) ? renamed + item.slice(path.length) : item));
    if (within(folder, path)) onFolder(renamed + folder.slice(path.length));
  };

  const remove = (path: string) => {
    setConfirming(null);
    setLibraryState(removeFolder(state, path));
    if (within(folder, path)) onFolder(path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "");
    pushLog("ok", "signals", "log.folderRemoved", { name: path });
  };

  /** Drop what is being dragged into `path` ("" is the top level). */
  const drop = (event: DragEvent, path: string) => {
    event.preventDefault(); event.stopPropagation();
    setDropAt(null);
    const what = dragged ?? readDragged(event);
    setDragged(null);
    if (!what) return;
    if (what.kind === "signal") { setLibraryState(moveSignal(state, what.id, path)); return; }
    const next = moveFolder(state, what.id, path);
    if (next && next !== state) setLibraryState(next);
  };
  const over = (event: DragEvent, path: string) => {
    if (!dragged) return;
    // Not into itself, nor into a folder inside it.
    if (dragged.kind === "folder" && within(path, dragged.id)) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
    setDropAt(path);
  };
  const start = (event: DragEvent, what: Dragged) => {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(DRAG_TYPE, JSON.stringify(what));
    setDragged(what);
  };

  const folderKeys = (event: KeyboardEvent, node: FolderNode) => {
    if (event.key === "F2") { event.preventDefault(); setRenaming(node.path); }
    if (event.key === "Delete") { event.preventDefault(); if (confirming === node.path) remove(node.path); else setConfirming(node.path); }
    if (event.key === "ArrowRight" && !isOpen(node.path)) { event.preventDefault(); toggle(node.path); }
    if (event.key === "ArrowLeft" && isOpen(node.path) && !searching) { event.preventDefault(); toggle(node.path); }
  };

  const indent = (depth: number) => ({ paddingInlineStart: 6 + depth * 14 });

  const renderSignal = (signal: Signal, depth: number) => <button key={signal.id} data-signal={signal.id} style={indent(depth)}
    className={"sig-item" + (signal.id === selectedId ? " active" : "")} aria-current={signal.id === selectedId ? "true" : undefined}
    draggable onDragStart={(event) => start(event, { kind: "signal", id: signal.id })} onDragEnd={() => { setDragged(null); setDropAt(null); }}
    data-tip={t("sig.itemHint")} onClick={() => onSelect(signal.id)} onDoubleClick={() => onFire(signal)}>
    <span className="sig-item-name">{signal.name}</span>
    <span className="sig-item-meta"><span className="sig-badge">{t(transportKey(signal.body.transport))}</span>{signalTarget(signal)}</span>
  </button>;

  const renderFolder = (node: FolderNode, depth: number) => {
    const open = isOpen(node.path);
    return <div key={node.path} className="sig-folder-block" role="group" aria-label={node.name}>
      <div className={`sig-folder-row ${folder === node.path ? "current" : ""} ${dropAt === node.path ? "drop" : ""}`} style={indent(depth)}
        onDragOver={(event) => over(event, node.path)} onDragLeave={() => setDropAt((at) => at === node.path ? null : at)} onDrop={(event) => drop(event, node.path)}>
        {renaming === node.path
          ? <input className="sig-folder-rename" autoFocus defaultValue={node.name} aria-label={t("sig.folderName")}
            onFocus={(event) => event.target.select()}
            onKeyDown={(event) => { if (event.key === "Enter") rename(node.path, event.currentTarget.value); if (event.key === "Escape") { event.preventDefault(); setRenaming(null); } }}
            onBlur={(event) => rename(node.path, event.currentTarget.value)} />
          : <button className="sig-folder" aria-expanded={open} draggable data-folder={node.path}
            data-tip={t("sig.folderRowHint")} onDragStart={(event) => start(event, { kind: "folder", id: node.path })} onDragEnd={() => { setDragged(null); setDropAt(null); }}
            onClick={() => { if (!searching) toggle(node.path); onFolder(node.path); }} onKeyDown={(event) => folderKeys(event, node)}>
            <span className="sig-folder-twist" aria-hidden="true">{open ? "▾" : "▸"}</span>
            <span className="sig-folder-name">{node.name}</span>
            <span className="sig-folder-count" aria-label={t("sig.count", { n: node.total })}>{node.total}</span>
          </button>}
        {renaming !== node.path && <span className="sig-folder-tools">
          <button className="ghost xs" aria-label={`${t("sig.renameFolder")}: ${node.name}`} data-tip={`${t("sig.renameFolder")} · F2`} onClick={() => setRenaming(node.path)}>✎</button>
          <button className={`ghost xs ${confirming === node.path ? "danger" : ""}`} aria-label={`${t("sig.removeFolder")}: ${node.name}`} data-tip={`${t("sig.removeFolderHint")} · Delete`}
            onClick={() => confirming === node.path ? remove(node.path) : setConfirming(node.path)} onBlur={() => setConfirming((at) => at === node.path ? null : at)}>
            {confirming === node.path ? t("sig.confirmRemoveFolder") : "×"}</button>
        </span>}
      </div>
      {open && <>{node.folders.map((child) => renderFolder(child, depth + 1))}{node.signals.map((signal) => renderSignal(signal, depth + 1))}</>}
    </div>;
  };

  const everyFolder = allFolders(library, folders);
  return <>
    <div className="sig-tree-tools" role="group" aria-label={t("sig.library")}>
      <button className="ghost xs" aria-label={t("sig.expandAll")} data-tip={t("sig.expandAll")} disabled={searching || collapsed.length === 0} onClick={() => setCollapsed([])}>⊞</button>
      <button className="ghost xs" aria-label={t("sig.collapseAll")} data-tip={t("sig.collapseAll")} disabled={searching || everyFolder.length === 0} onClick={() => setCollapsed(everyFolder)}>⊟</button>
      <button className="ghost xs" data-tip={t("sig.newFolderHint")} onClick={onNewFolder}>＋ {t("sig.newFolder")}</button>
      {folder && <button className="link-btn sig-current-folder" data-tip={t("sig.currentFolderHint")} onClick={() => onFolder("")}>{t("sig.inFolder", { folder: folder.split("/").join(" / ") })} ×</button>}
    </div>
    <div className={`sig-list scroll-y ${dropAt === "" ? "drop" : ""}`}
      onDragOver={(event) => { if (!dragged) return; event.preventDefault(); setDropAt(""); }} onDragLeave={(event) => { if (event.target === event.currentTarget) setDropAt(null); }}
      onDrop={(event) => drop(event, "")}>
      {tree.folders.map((node) => renderFolder(node, 0))}
      {tree.signals.map((signal) => renderSignal(signal, 0))}
    </div>
  </>;
}

/** What another window or a test dragged, from the transfer data. */
function readDragged(event: DragEvent): Dragged | null {
  try {
    const value = JSON.parse(event.dataTransfer.getData(DRAG_TYPE));
    return (value?.kind === "signal" || value?.kind === "folder") && typeof value.id === "string" ? value : null;
  } catch {
    return null;
  }
}
