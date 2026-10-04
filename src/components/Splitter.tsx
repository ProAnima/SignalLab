import { useRef, type KeyboardEvent, type PointerEvent } from "react";
import { useT, type TKey } from "../lib/i18n";

/** Arrow keys move a handle this far; with Shift, four times as far. */
const STEP = 16;

/**
 * The handle between two panes, as a layout has it: drag it, or focus it and
 * use the arrow keys (Shift for bigger steps, Home and End for the limits);
 * a double click gives the pane its default size back. It is a `separator`
 * that says how big the pane is, so a screen reader can tell.
 *
 * The pane it sizes is the one *after* the handle — below a horizontal one,
 * after a vertical one in reading order (to its right, or its left in a
 * right-to-left page) — so dragging up, or towards the reading's start, makes
 * it bigger.
 */
export function Splitter({ orientation, label, size, min, max, initial, onSize }: {
  /** `horizontal`: a bar that sizes a height; `vertical`: one that sizes a width. */
  orientation: "horizontal" | "vertical";
  label: TKey;
  size: number;
  min: number;
  /** The largest size that still leaves room for the rest (from the current window). */
  max: () => number;
  initial: number;
  onSize: (size: number) => void;
}) {
  const t = useT();
  const drag = useRef<{ start: number; size: number } | null>(null);
  const handle = useRef<HTMLDivElement>(null);
  // The size as last set, ahead of the render that shows it: two quick key
  // presses must add up rather than both start from the same value.
  const latest = useRef(size);
  latest.current = size;
  const resize = (next: number) => { latest.current = next; onSize(next); };
  const horizontal = orientation === "horizontal";
  const clamp = (value: number) => Math.round(Math.min(Math.max(value, min), Math.max(min, max())));
  const position = (event: PointerEvent) => horizontal ? event.clientY : event.clientX;
  // A vertical handle in a right-to-left page sizes the pane on its left: the other way round.
  const rtl = () => !horizontal && !!handle.current && getComputedStyle(handle.current).direction === "rtl";
  const cursor = horizontal ? "resizing-rows" : "resizing-columns";

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    event.preventDefault();
    // Moves keep coming here while the pointer is outside the thin handle; a
    // pointer the browser does not track (a synthetic one) still drags without it.
    try { event.currentTarget.setPointerCapture(event.pointerId); } catch { /* not capturable */ }
    drag.current = { start: position(event), size: latest.current };
    document.body.classList.add(cursor);
    handle.current?.classList.add("dragging");
  };
  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    const d = drag.current;
    if (d) resize(clamp(d.size - (position(event) - d.start) * (rtl() ? -1 : 1)));
  };
  const end = () => {
    drag.current = null;
    document.body.classList.remove(cursor);
    handle.current?.classList.remove("dragging");
  };
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const step = event.shiftKey ? STEP * 4 : STEP;
    const grow = horizontal ? "ArrowUp" : rtl() ? "ArrowRight" : "ArrowLeft";
    const shrink = horizontal ? "ArrowDown" : rtl() ? "ArrowLeft" : "ArrowRight";
    const current = latest.current;
    const next = event.key === grow ? current + step : event.key === shrink ? current - step
      : event.key === "Home" ? min : event.key === "End" ? max() : event.key === "Enter" ? initial : null;
    if (next === null) return;
    event.preventDefault();
    resize(clamp(next));
  };

  return <div ref={handle} className={`splitter ${horizontal ? "rows" : "columns"}`} role="separator" tabIndex={0}
    aria-orientation={orientation} aria-label={t(label)} aria-valuenow={size} aria-valuemin={min} aria-valuemax={Math.max(min, Math.round(max()))}
    data-tip={`${t(label)} · ${t("layout.resizeHint")}`}
    onPointerDown={onPointerDown} onPointerMove={onPointerMove} onPointerUp={end} onPointerCancel={end} onLostPointerCapture={end}
    onDoubleClick={() => resize(clamp(initial))} onKeyDown={onKeyDown} />;
}
