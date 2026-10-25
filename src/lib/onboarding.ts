/**
 * The introduction tour: its steps and how it moves between them. Pure, so it
 * is tested without a browser (tests/onboarding.test.mjs); the overlay that
 * shows it is `components/Onboarding.tsx`.
 *
 * A step points at an element of the shell by its `data-tour` attribute (never
 * a CSS class), or at nothing: a card in the middle of the window. A step whose
 * element is missing or hidden — a narrow window, a mode that hides the
 * sidebar — is passed over, so the tour can always be finished.
 */
import type { TKey } from "./locales/en";
import type { Side } from "./tooltip";

/** Present once the tour was finished or skipped: it does not open by itself again. */
export const DONE_KEY = "signal-lab.onboarding.done";
/** The documentation page the last card offers. */
export const FIRST_STEPS_PAGE = "guide/first-steps";

export interface TourStep {
  id: string;
  /** The `data-tour` value of the element shown, or null for a card in the middle. */
  target: string | null;
  title: TKey;
  text: TKey;
  /** The text in a browser, where the server's app has no updates or statistics to speak of. */
  textServer?: TKey;
  /** Where the card goes beside its element, first choice first. */
  prefer?: readonly Side[];
}

export const TOUR_STEPS: readonly TourStep[] = [
  { id: "welcome", target: null, title: "onboarding.welcome.title", text: "onboarding.welcome.text" },
  { id: "nav", target: "nav", title: "onboarding.nav.title", text: "onboarding.nav.text", prefer: ["end"] },
  { id: "experiment", target: "nav-experiment", title: "nav.experiment", text: "onboarding.experiment.text", prefer: ["end"] },
  { id: "signals", target: "nav-signals", title: "nav.signals", text: "onboarding.signals.text", prefer: ["end"] },
  { id: "emulators", target: "nav-emulators", title: "nav.emulators", text: "onboarding.emulators.text", prefer: ["end"] },
  { id: "dock", target: "dock", title: "onboarding.dock.title", text: "onboarding.dock.text", prefer: ["above"] },
  { id: "jobs", target: "stop-all", title: "onboarding.jobs.title", text: "onboarding.jobs.text", prefer: ["below"] },
  { id: "language", target: "language", title: "app.language", text: "onboarding.language.text", prefer: ["below"] },
  { id: "docs", target: "docs", title: "app.docs", text: "onboarding.docs.text", prefer: ["below"] },
  { id: "about", target: "about", title: "about.open", text: "onboarding.about.text", textServer: "onboarding.about.textServer", prefer: ["below"] },
  { id: "finish", target: null, title: "onboarding.finish.title", text: "onboarding.finish.text" },
];

/** Whether the element a step names (`data-tour`) is there and visible now. */
export type Shown = (target: string) => boolean;

const can = (step: TourStep, shown: Shown) => step.target === null || shown(step.target);

/** The first step from `from` on, walking `direction`, that can be shown; null past either end. */
export function seek(steps: readonly TourStep[], from: number, direction: 1 | -1, shown: Shown): number | null {
  for (let at = from; at >= 0 && at < steps.length; at += direction) if (can(steps[at], shown)) return at;
  return null;
}

/** Where the tour opens. */
export const first = (steps: readonly TourStep[], shown: Shown) => seek(steps, 0, 1, shown);
/** The step after `index`; null when `index` is the last one that can be shown (Finish). */
export const next = (steps: readonly TourStep[], index: number, shown: Shown) => seek(steps, index + 1, 1, shown);
/** The step before `index`, or `index` itself on the first. */
export const back = (steps: readonly TourStep[], index: number, shown: Shown) => seek(steps, index - 1, -1, shown) ?? index;
export const isFirst = (steps: readonly TourStep[], index: number, shown: Shown) => seek(steps, index - 1, -1, shown) === null;
export const isLast = (steps: readonly TourStep[], index: number, shown: Shown) => next(steps, index, shown) === null;

/**
 * The step to show when the one on show may have lost its element (the window
 * shrank, the layout changed): itself while it can be shown, else the next one,
 * else the one before; null only when nothing can be shown.
 */
export function settle(steps: readonly TourStep[], index: number, shown: Shown): number | null {
  if (index >= 0 && index < steps.length && can(steps[index], shown)) return index;
  return seek(steps, index + 1, 1, shown) ?? seek(steps, index - 1, -1, shown);
}

/** "3 / 11": the step's place among the steps that can be shown now. */
export function progress(steps: readonly TourStep[], index: number, shown: Shown): { n: number; total: number } {
  const showable = steps.map((step, at) => can(step, shown) ? at : -1).filter((at) => at >= 0);
  return { n: showable.filter((at) => at <= index).length, total: showable.length };
}

export type TourAction = "next" | "back" | "skip";

/** What a key does on the tour's card: Enter and → go on, ← back (mirrored right to left), Esc skips. */
export function keyAction(key: string, rtl: boolean): TourAction | null {
  switch (key) {
    case "Enter": return "next";
    case "ArrowRight": return rtl ? "back" : "next";
    case "ArrowLeft": return rtl ? "next" : "back";
    case "Escape": return "skip";
    default: return null;
  }
}

/** The part of storage the tour uses. */
export interface TourStorage { getItem(key: string): string | null; setItem(key: string, value: string): void }

/**
 * Whether the tour opens by itself: the first launch, until finished or
 * skipped. Without storage (a locked-down webview) it opens once a session —
 * `seen` says whether it already did.
 */
export function opensByItself(storage: TourStorage | null, seen: boolean): boolean {
  if (seen) return false;
  try {
    if (!storage) return true;
    return storage.getItem(DONE_KEY) === null;
  } catch {
    return true;
  }
}

/** Finished or skipped: kept, so the tour waits until it is asked for again. */
export function markDone(storage: TourStorage | null): void {
  try {
    storage?.setItem(DONE_KEY, "true");
  } catch {
    // Not kept; it still does not open again this session.
  }
}
