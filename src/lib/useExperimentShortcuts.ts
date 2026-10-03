import { useEffect, useRef, type RefObject } from "react";
import type { Experiment, ExperimentNode } from "./api";
import type { Wire } from "./experimentWires";
import { offer, receive } from "./clipboardSink";

/** A control that takes typing: the editor's single-key shortcuts leave it alone. */
export const editable = (target: EventTarget | null) => target instanceof HTMLElement && !!target.closest("input, textarea, select, [contenteditable=true]");

/** What the editor's keys read and do, as of the latest render. */
export interface ShortcutContext {
  /** The editor is the screen showing; hidden, it takes no keys. */
  active: boolean;
  busy: boolean;
  doc: Experiment | null;
  /** The last node selected: the one the properties show. */
  selected: string | null;
  /** How many nodes are selected, `selected` among them. */
  selectionCount: number;
  selectedNode: ExperimentNode | null;
  selectedWire: Wire | null;
  menuOpen: boolean;
  linking: boolean;
  fullscreen: boolean;
  focusMode: boolean;
  closeMenu: () => void;
  cancelLink: () => void;
  dropWire: () => void;
  exitFullscreen: () => void;
  setFocusMode: (value: boolean) => void;
  restore: (type: "undo" | "redo") => void;
  duplicateSelected: () => void;
  openFinder: () => void;
  fit: (nodes: ExperimentNode[]) => void;
  zoomAt: (value: number) => void;
  sendNode: (node: ExperimentNode) => Promise<void>;
  /** Removes the selected wire. */
  removeWire: () => void;
  removeSelected: () => void;
  openAddMenu: () => void;
  selectAll: () => void;
  clearSelection: () => void;
  /** Every selected node, by the arrow keys' step (in the history group of `editGroup`). */
  moveSelection: (dx: number, dy: number) => void;
  /** The selection as clipboard text, none when there is nothing to copy. */
  copySelection: () => string | null;
  /** The same, and the selection removed. */
  cutSelection: () => string | null;
  /** Pasted text: nodes when it is a copy of some, else nothing (false). */
  pasteText: (text: string) => boolean;
  /** The history group arrow-key nudges go into, committed when the key is let go. */
  editGroup: RefObject<string | null>;
  commitEdit: () => void;
}

/** Text the person selected on the page (the console, a report): Ctrl+C is theirs then, unless the editor has the focus. */
function textSelected(): boolean {
  const selection = window.getSelection();
  return !!selection && !selection.isCollapsed && selection.toString().trim() !== "";
}
/** The selection's keys are the editor's while it has the focus, or nothing has it and no text is selected. */
function selectionKeys(): boolean {
  const focus = document.activeElement;
  if (focus?.closest(".experiment-view")) return true;
  return (!focus || focus === document.body) && !textSelected();
}

/**
 * The editor's keyboard: Escape, A, Delete, arrows, Ctrl+Z/Y/D/F/0/1/Enter,
 * and the selection's Ctrl+A/C/X/V. Registered once; the listener always
 * reads the latest render's context, so a shortcut never acts on state an
 * earlier render captured.
 */
export function useExperimentShortcuts(context: ShortcutContext) {
  const latest = useRef(context);
  latest.current = context;
  useEffect(() => {
    const down = (event: KeyboardEvent) => onKeyDown(event, latest.current);
    const up = (event: KeyboardEvent) => onKeyUp(event, latest.current);
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => { window.removeEventListener("keydown", down); window.removeEventListener("keyup", up); };
  }, []);
}

function onKeyDown(event: KeyboardEvent, c: ShortcutContext) {
  if (!c.active) return;
  if (event.defaultPrevented || (event.target as HTMLElement).closest('dialog, [role="dialog"]')) return;
  if (event.key === "Escape") {
    if (c.menuOpen) c.closeMenu();
    else if (c.linking) c.cancelLink();
    else if (c.selectedWire) c.dropWire();
    else if (c.selectionCount > 1) c.clearSelection();
    else if (c.fullscreen) c.exitFullscreen();
    else if (c.focusMode) c.setFocusMode(false);
    return;
  }
  const target = event.target as HTMLElement;
  if (editable(target)) return;
  // Virtual keyboards can omit the physical code. Keep shortcuts usable there too.
  const key = event.key.toLowerCase();
  const fallbackKey = ({ я: "z", н: "y", в: "d", а: "f", ф: "a", с: "c", ч: "x", м: "v" } as Record<string, string>)[key] ?? key;
  const code = event.code && event.code !== "Unidentified" ? event.code
    : /^[a-z]$/.test(fallbackKey) ? `Key${fallbackKey.toUpperCase()}` : /^\d$/.test(key) ? `Digit${key}` : event.code;
  if (event.ctrlKey || event.metaKey) {
    // Copy, cut and paste go through the system's own handling of text (clipboardSink),
    // so the key's default must run: no preventDefault for them.
    const mine = selectionKeys() && !event.shiftKey && !event.altKey;
    if ((code === "KeyC" || code === "KeyX") && mine && c.selectionCount) {
      const text = code === "KeyX" ? c.cutSelection() : c.copySelection();
      if (text) offer(text);
      return;
    }
    if (code === "KeyV" && mine && !c.busy) { receive((text) => { c.pasteText(text); }); return; }
    if (code === "KeyA" && mine) { event.preventDefault(); c.selectAll(); }
    if (code === "KeyZ") { event.preventDefault(); c.restore(event.shiftKey ? "redo" : "undo"); }
    if (code === "KeyY") { event.preventDefault(); c.restore("redo"); }
    if (code === "KeyD") { event.preventDefault(); c.duplicateSelected(); }
    if (code === "KeyF") { event.preventDefault(); c.closeMenu(); c.openFinder(); }
    if (code === "Digit0" && c.doc) { event.preventDefault(); c.fit(c.doc.nodes); }
    if (code === "Digit1") { event.preventDefault(); c.zoomAt(1); }
    if (event.key === "Enter" && c.selectedNode) { event.preventDefault(); void c.sendNode(c.selectedNode); }
    return;
  }
  if ((event.key === "Delete" || event.key === "Backspace") && c.selectedWire && !c.busy) { event.preventDefault(); c.removeWire(); return; }
  if ((event.key === "Delete" || event.key === "Backspace") && c.selectionCount && !c.busy) { event.preventDefault(); c.removeSelected(); }
  if (code === "KeyA" && !c.menuOpen) { event.preventDefault(); c.openAddMenu(); }
  if (c.selectionCount && !c.busy && target.closest(".experiment-node-body") && ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) {
    event.preventDefault(); c.editGroup.current ??= `nudge-${crypto.randomUUID()}`;
    const step = event.shiftKey ? 20 : 5;
    c.moveSelection(event.key === "ArrowRight" ? step : event.key === "ArrowLeft" ? -step : 0, event.key === "ArrowDown" ? step : event.key === "ArrowUp" ? -step : 0);
  }
}

function onKeyUp(event: KeyboardEvent, c: ShortcutContext) {
  if (event.key.startsWith("Arrow") && c.editGroup.current?.startsWith("nudge-")) c.commitEdit();
}
