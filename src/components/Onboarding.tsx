import { useEffect, useId, useLayoutEffect, useRef, useState, type CSSProperties } from "react";
import { createPortal } from "react-dom";
import { useI18n } from "../lib/i18n";
import { fmtNum } from "../lib/format";
import { openDocs } from "../lib/platform";
import { useUpdates } from "../lib/updates";
import { placeCard, type Box, type ScreenSide } from "../lib/tooltip";
import {
  FIRST_STEPS_PAGE, TOUR_STEPS, back, first, isFirst, isLast, keyAction, markDone, next, opensByItself, progress, settle,
  type TourAction, type TourStorage,
} from "../lib/onboarding";

/** After the first render, so the shell is laid out (and in its language) before the tour measures it. */
const START_DELAY = 350;
/** The cut-out leaves this much room around its element. */
const HOLE_PAD = 6;
const WATCH = 400;

/** Opened once this session already: without storage, that is all there is to go by. */
let seenThisSession = false;

function storage(): TourStorage | null {
  try { return window.localStorage; } catch { return null; }
}

const find = (target: string) => document.querySelector<HTMLElement>(`[data-tour="${CSS.escape(target)}"]`);

/** There, drawn, not in a hidden screen, and at least partly in the window. */
function shown(target: string): boolean {
  const element = find(target);
  if (!element || element.closest("[hidden]")) return false;
  const box = element.getBoundingClientRect();
  if (box.width <= 0 || box.height <= 0 || getComputedStyle(element).visibility === "hidden") return false;
  return box.right > 0 && box.bottom > 0 && box.left < window.innerWidth && box.top < window.innerHeight;
}

interface Layout { hole: Box | null; left: number; top: number; side: ScreenSide | null; arrow: number }

const sameLayout = (a: Layout | null, b: Layout) => !!a && a.left === b.left && a.top === b.top && a.side === b.side && a.arrow === b.arrow
  && (a.hole === b.hole || (!!a.hole && !!b.hole && a.hole.left === b.hole.left && a.hole.top === b.hole.top && a.hole.width === b.hole.width && a.hole.height === b.hole.height));

/**
 * The introduction tour: on the first launch (and from About, when asked) the
 * window dims, a cut-out shows one part of the shell at a time, and a card
 * beside it says what it is. Next, Back, Skip tour; Enter and the arrows; Esc
 * skips. Finishing or skipping keeps `signal-lab.onboarding.done`, so it does
 * not open by itself again.
 *
 * Mounted once in the shell; drawn over everything in a layer of its own, with
 * the app behind it inert while it is open.
 */
export function Onboarding({ request }: { request: number }) {
  const { t, lang } = useI18n();
  const desktop = useUpdates().supported;
  const [index, setIndex] = useState<number | null>(null);
  const [layout, setLayout] = useState<Layout | null>(null);
  // Off for the first placement, so the card does not fly in from the corner.
  const [moving, setMoving] = useState(false);
  const card = useRef<HTMLDivElement>(null);
  const primary = useRef<HTMLButtonElement>(null);
  const id = useId();
  const open = index !== null;

  const start = () => {
    seenThisSession = true;
    setLayout(null);
    setMoving(false);
    setIndex(first(TOUR_STEPS, shown) ?? 0);
  };
  const close = () => {
    seenThisSession = true;
    markDone(storage());
    setIndex(null);
    setLayout(null);
  };

  // The first launch: once the shell has drawn itself.
  useEffect(() => {
    if (!opensByItself(storage(), seenThisSession)) return;
    const timer = window.setTimeout(() => { if (opensByItself(storage(), seenThisSession)) start(); }, START_DELAY);
    return () => window.clearTimeout(timer);
  }, []);
  // Asked for (About): at once — a frame later, once the dialog that asked has closed.
  useEffect(() => {
    if (!request) return;
    const frame = requestAnimationFrame(start);
    return () => cancelAnimationFrame(frame);
  }, [request]);

  const act = (action: TourAction) => {
    if (index === null) return;
    if (action === "skip") return close();
    if (action === "back") return setIndex(back(TOUR_STEPS, index, shown));
    const after = next(TOUR_STEPS, index, shown);
    if (after === null) close(); else setIndex(after);
  };
  const actRef = useRef(act);
  actRef.current = act;

  // Where the cut-out and the card go, measured again whenever anything may have moved.
  const measure = useRef<(scroll?: boolean) => void>(() => {});
  measure.current = (scroll = false) => {
    if (index === null) return;
    const settled = settle(TOUR_STEPS, index, shown);
    if (settled === null) return close();
    if (settled !== index) return setIndex(settled);
    const step = TOUR_STEPS[index];
    const element = step.target ? find(step.target) : null;
    if (element && scroll) element.scrollIntoView({ block: "nearest", inline: "nearest" });
    const view = { width: window.innerWidth, height: window.innerHeight };
    const size = { width: card.current?.offsetWidth ?? 0, height: card.current?.offsetHeight ?? 0 };
    let next: Layout;
    if (element) {
      const box = element.getBoundingClientRect();
      const left = Math.max(2, box.left - HOLE_PAD);
      const top = Math.max(2, box.top - HOLE_PAD);
      const hole = {
        left: Math.round(left), top: Math.round(top),
        width: Math.round(Math.min(view.width - 2, box.right + HOLE_PAD) - left),
        height: Math.round(Math.min(view.height - 2, box.bottom + HOLE_PAD) - top),
      };
      // Read now: the language's direction is written to the page after this renders.
      const place = placeCard(hole, size, view, step.prefer, document.documentElement.dir === "rtl");
      next = { hole, left: place.left, top: place.top, side: place.side, arrow: place.arrow };
    } else {
      next = { hole: null, left: Math.round((view.width - size.width) / 2), top: Math.round((view.height - size.height) / 2), side: null, arrow: 0 };
    }
    setLayout((prior) => sameLayout(prior, next) ? prior : next);
  };
  useLayoutEffect(() => { measure.current(true); }, [index, lang]);
  useEffect(() => {
    if (!layout || moving) return;
    const frame = requestAnimationFrame(() => setMoving(true));
    return () => cancelAnimationFrame(frame);
  }, [layout, moving]);

  // While open: the window's size, the shell's layout and its elements are watched.
  useEffect(() => {
    if (!open) return;
    const again = () => measure.current();
    window.addEventListener("resize", again);
    const observer = new ResizeObserver(again);
    observer.observe(document.body);
    const app = document.getElementById("root");
    if (app) observer.observe(app);
    const page = new MutationObserver(again);
    page.observe(document.documentElement, { attributes: true, attributeFilter: ["dir", "lang"] });
    const timer = window.setInterval(again, WATCH);
    return () => { window.removeEventListener("resize", again); observer.disconnect(); page.disconnect(); window.clearInterval(timer); };
  }, [open]);

  // While open, the app behind takes neither clicks nor focus; focus comes back where it was.
  useEffect(() => {
    if (!open) return;
    const before = document.activeElement as HTMLElement | null;
    const app = document.getElementById("root");
    app?.setAttribute("inert", "");
    return () => {
      app?.removeAttribute("inert");
      if (before?.isConnected && before !== document.body) before.focus({ preventScroll: true });
    };
  }, [open]);

  // Focus stays in the card: on the step's main button unless it is on another button there.
  // Once placed — a card still hidden while it is measured cannot take focus.
  const placed = layout !== null;
  useEffect(() => {
    if (!open || !placed) return;
    if (!card.current?.contains(document.activeElement)) primary.current?.focus({ preventScroll: true });
  }, [open, placed, index]);

  // Keys: the tour's own, and nothing reaches the app behind it (F1, Ctrl+K, a screen's shortcuts).
  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      event.stopPropagation();
      const box = card.current;
      if (!box) return;
      if (event.key === "Tab") {
        event.preventDefault();
        const buttons = [...box.querySelectorAll<HTMLButtonElement>("button:not(:disabled)")];
        if (!buttons.length) return;
        const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
        const to = at < 0 ? 0 : (at + (event.shiftKey ? -1 : 1) + buttons.length) % buttons.length;
        buttons[to].focus();
        return;
      }
      if (event.key === "F1" || event.ctrlKey || event.metaKey || event.altKey) { event.preventDefault(); return; }
      const action = keyAction(event.key, document.documentElement.dir === "rtl");
      if (!action) return;
      // Enter on one of the card's buttons is that button's own press.
      if (event.key === "Enter" && document.activeElement instanceof HTMLButtonElement && box.contains(document.activeElement)) return;
      event.preventDefault();
      actRef.current(action);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [open]);

  if (index === null) return null;
  const step = TOUR_STEPS[index];
  const atStart = isFirst(TOUR_STEPS, index, shown);
  const atEnd = isLast(TOUR_STEPS, index, shown);
  const { n, total } = progress(TOUR_STEPS, index, shown);
  const counted = t("onboarding.progress", { n: fmtNum(n), total: fmtNum(total) });
  const hole = layout?.hole;
  const holeStyle: CSSProperties = hole
    ? { transform: `translate(${hole.left}px, ${hole.top}px)`, width: hole.width, height: hole.height }
    : { transform: `translate(${Math.round(window.innerWidth / 2)}px, ${Math.round(window.innerHeight / 2)}px)`, width: 0, height: 0 };
  const cardStyle = { transform: layout ? `translate(${layout.left}px, ${layout.top}px)` : undefined, "--arrow": `${layout?.arrow ?? 0}px` } as CSSProperties;

  return createPortal(
    <div className={"tour" + (moving ? " moving" : "")} data-step={step.id}>
      <div className={"tour-hole" + (hole ? "" : " none")} style={holeStyle} aria-hidden="true" />
      <div ref={card} className={"tour-card" + (layout ? " placed" : "") + (layout?.side ? ` side-${layout.side}` : " centred")} style={cardStyle}
        role="dialog" aria-modal="true" aria-labelledby={`${id}title`} aria-describedby={`${id}text`}>
        {layout?.side && <span className="tour-arrow" aria-hidden="true" />}
        <div className="tour-progress">
          <span className="tour-dots" aria-hidden="true">
            {Array.from({ length: total }, (_, at) => <span key={at} className={"tour-dot" + (at === n - 1 ? " on" : at < n - 1 ? " past" : "")} />)}
          </span>
          <span className="tour-count">{counted}</span>
        </div>
        <h2 id={`${id}title`} className="tour-title">{t(step.title)}</h2>
        <p id={`${id}text`} className="tour-text">{t(!desktop && step.textServer ? step.textServer : step.text)}</p>
        {atEnd && <button type="button" className="link-btn tour-docs" onClick={() => { close(); openDocs(FIRST_STEPS_PAGE, lang).catch(() => {}); }}>{t("onboarding.firstSteps")}</button>}
        <div className="tour-buttons">
          {atStart ? <>
            <button type="button" className="ghost" onClick={() => act("skip")}>{t("onboarding.skipShort")}</button>
            <span className="spacer" />
            <button ref={primary} type="button" className="primary" onClick={() => act("next")}>{t("onboarding.start")}</button>
          </> : <>
            <button type="button" className="ghost tour-skip" onClick={() => act("skip")}>{t("onboarding.skip")}</button>
            <span className="spacer" />
            <button type="button" className="ghost" onClick={() => act("back")}>{t("onboarding.back")}</button>
            <button ref={primary} type="button" className="primary" onClick={() => act("next")}>{atEnd ? t("onboarding.finish") : t("onboarding.next")}</button>
          </>}
        </div>
      </div>
      <div className="tour-live" aria-live="polite">{`${counted} · ${t(step.title)}`}</div>
    </div>,
    document.body,
  );
}
