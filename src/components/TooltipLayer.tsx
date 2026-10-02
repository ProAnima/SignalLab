import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { ENTRY, placeTip, type Placement } from "../lib/tooltip";

/** Hover this long before a tooltip appears; right after one was shown, the next is immediate. */
const SHOW_DELAY = 450;
const WARM_FOR = 500;
/** Focus this soon after a press came from the pointer, not the keyboard. */
const POINTER_FOCUS = 150;
const TIP_ID = "signal-lab-tooltip";

/** `target` carries the tip and places it; `described` is what a screen reader hears it on. */
interface Shown { target: HTMLElement; text: string; described: HTMLElement }

const tipOf = (node: EventTarget | null) =>
  node instanceof Element ? node.closest<HTMLElement>("[data-tip]") : null;

/** On focus: the control's own tip, or else the one on the label that names it. */
const focusTipOf = (node: EventTarget | null) => {
  const own = tipOf(node);
  if (own) return own;
  if (node instanceof HTMLInputElement || node instanceof HTMLSelectElement || node instanceof HTMLTextAreaElement) {
    for (const label of node.labels ?? []) if (label.dataset.tip?.trim()) return label;
  }
  return null;
};

/**
 * The one tooltip of the interface. Any element with `data-tip` gets it: on
 * hover after a short delay, at once on keyboard focus (a field shows the tip of
 * its label); gone on Escape, typing, a press, scrolling or leaving. While
 * shown, the element is described by it (`aria-describedby`), so a screen
 * reader reads the same help.
 *
 * Mounted once in the shell. Use `data-tip`, never the native `title`.
 */
export function TooltipLayer() {
  const [shown, setShown] = useState<Shown | null>(null);
  const [place, setPlace] = useState<Placement | null>(null);
  const tipRef = useRef<HTMLDivElement>(null);
  const current = useRef<Shown | null>(null);

  useEffect(() => {
    let timer = 0;
    let hiddenAt = 0;
    let pressedAt = 0;
    // Pressed: no tooltip for it again until the pointer has left it.
    let pressed: HTMLElement | null = null;
    const show = (target: HTMLElement, described: HTMLElement = target) => {
      const text = target.dataset.tip?.trim();
      if (!text) return;
      current.current = { target, text, described };
      setShown(current.current);
    };
    const hide = () => {
      window.clearTimeout(timer);
      if (current.current) hiddenAt = Date.now();
      current.current = null;
      setShown(null);
    };
    const schedule = (target: HTMLElement) => {
      window.clearTimeout(timer);
      if (current.current || Date.now() - hiddenAt < WARM_FOR) show(target);
      else timer = window.setTimeout(() => show(target), SHOW_DELAY);
    };

    const onOver = (event: PointerEvent) => {
      if (event.pointerType === "touch") return;
      let target = tipOf(event.target);
      // A label's help shows over its text, not while the pointer rests in its field.
      const entry = event.target instanceof Element ? event.target.closest(ENTRY) : null;
      if (target && entry && entry !== target && target.contains(entry)) target = null;
      if (target && target === current.current?.target) return;
      if (!target) return hide();
      if (target === pressed) return;
      schedule(target);
    };
    const onOut = (event: PointerEvent) => {
      const target = tipOf(event.target);
      if (!target || tipOf(event.relatedTarget) === target) return;
      if (target === pressed) pressed = null;
      window.clearTimeout(timer);
      if (current.current?.target === target) hide();
    };
    const onPress = (event: PointerEvent) => {
      pressedAt = Date.now();
      pressed = tipOf(event.target);
      hide();
    };
    const onFocus = (event: FocusEvent) => {
      if (Date.now() - pressedAt < POINTER_FOCUS) return;
      const target = focusTipOf(event.target);
      if (!target) return;
      window.clearTimeout(timer);
      show(target, target.contains(event.target as Node) ? target : event.target as HTMLElement);
    };
    const onBlur = (event: FocusEvent) => {
      if (current.current && focusTipOf(event.target) === current.current.target) hide();
    };
    // Tab moves on (and focus shows the next one); any other key means work has started.
    const onKey = (event: KeyboardEvent) => {
      if (current.current && event.key !== "Tab" && event.key !== "Shift") hide();
    };
    // Text that arrives without a key press (paste, an input method) is typing too.
    const onInput = () => { if (current.current) hide(); };
    const onScroll = () => { if (current.current) hide(); };

    document.addEventListener("pointerover", onOver);
    document.addEventListener("pointerout", onOut);
    document.addEventListener("pointerdown", onPress, true);
    document.addEventListener("focusin", onFocus);
    document.addEventListener("focusout", onBlur);
    document.addEventListener("keydown", onKey, true);
    document.addEventListener("input", onInput, true);
    document.addEventListener("scroll", onScroll, true);
    window.addEventListener("blur", hide);
    // An element can disappear under a still pointer (a re-render, a closed menu).
    const watch = window.setInterval(() => {
      const target = current.current?.target;
      if (target && (!target.isConnected || target.getClientRects().length === 0)) hide();
    }, 400);
    return () => {
      window.clearTimeout(timer);
      window.clearInterval(watch);
      document.removeEventListener("pointerover", onOver);
      document.removeEventListener("pointerout", onOut);
      document.removeEventListener("pointerdown", onPress, true);
      document.removeEventListener("focusin", onFocus);
      document.removeEventListener("focusout", onBlur);
      document.removeEventListener("keydown", onKey, true);
      document.removeEventListener("input", onInput, true);
      document.removeEventListener("scroll", onScroll, true);
      window.removeEventListener("blur", hide);
    };
  }, []);

  // A tooltip follows its element's text, e.g. when the language changes.
  useEffect(() => {
    if (!shown) return;
    const observer = new MutationObserver(() => {
      const text = shown.target.dataset.tip?.trim();
      if (!text) { current.current = null; setShown(null); }
      else if (text !== shown.text) { current.current = { ...shown, text }; setShown(current.current); }
    });
    observer.observe(shown.target, { attributes: true, attributeFilter: ["data-tip"] });
    return () => observer.disconnect();
  }, [shown]);

  // Described by the tooltip while it shows — unless it only repeats the element's name.
  useEffect(() => {
    if (!shown) return;
    const { target, text, described } = shown;
    const name = (target.getAttribute("aria-label") ?? target.textContent ?? "").trim();
    if (name === text) return;
    const before = described.getAttribute("aria-describedby");
    described.setAttribute("aria-describedby", before ? `${before} ${TIP_ID}` : TIP_ID);
    return () => {
      if (before) described.setAttribute("aria-describedby", before);
      else described.removeAttribute("aria-describedby");
    };
  }, [shown]);

  useLayoutEffect(() => {
    const tip = tipRef.current;
    if (!shown || !tip) { setPlace(null); return; }
    // In the top layer, raised on every show, so it is drawn over a modal dialog too.
    if (typeof tip.showPopover === "function") {
      if (tip.matches(":popover-open")) tip.hidePopover();
      tip.showPopover();
    }
    // Measured at the origin, so its width does not depend on where the last one was.
    tip.style.left = "0px";
    tip.style.top = "0px";
    const box = shown.target.getBoundingClientRect();
    setPlace(placeTip(box, { width: tip.offsetWidth, height: tip.offsetHeight }, { width: window.innerWidth, height: window.innerHeight }));
  }, [shown]);

  if (!shown) return null;
  return (
    <div
      ref={tipRef}
      id={TIP_ID}
      role="tooltip"
      popover="manual"
      className={"tooltip" + (place ? ` shown ${place.side}` : "")}
      style={place ? { left: place.left, top: place.top } : { left: 0, top: 0 }}
    >
      {shown.text}
    </div>
  );
}
