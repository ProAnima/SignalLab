/**
 * The introduction tour, first: on fresh data it is up by itself (else it is
 * started from About). Every card is walked with Next — the cut-out over a
 * visible element of the shell, the card's words the dictionary's, the card
 * inside the window — Back goes back, Esc skips, About brings it back, Skip
 * closes it, and afterwards nothing is in the way of the steps that follow.
 * Split in three, so the runner's screenshots show the spotlight on its way.
 */
import { LOCALES } from "../../../src/lib/locales/index";
import { DONE_KEY, TOUR_STEPS } from "../../../src/lib/onboarding";
import { button, click, describe, key, sleep, squash, textOf, unnamed, until, visible, type Expect, type Key, type Words } from "../dsl";

/** The interface's dictionary: the tour opens in whatever language the app picked. */
const words = (): Words => (LOCALES.find((locale) => locale.code === document.documentElement.lang)?.dict ?? LOCALES[0].dict) as Words;
const tour = () => document.querySelector<HTMLElement>(".tour");
const card = () => document.querySelector<HTMLElement>(".tour-card[role=dialog]");
const stepId = () => tour()?.dataset.step ?? null;

/** The card and the cut-out where they settle: placed, and not moving between two looks. */
async function settled(): Promise<{ card: DOMRect; hole: DOMRect }> {
  let last = "";
  return until("the tour's card in place", () => {
    const box = card();
    const hole = document.querySelector<HTMLElement>(".tour-hole");
    if (!box?.classList.contains("placed") || !hole) return null;
    // Not while they move: WebKitGTK starts a transition a few frames late, so two equal looks
    // can both come before it.
    if ([box, hole].some((element) => element.getAnimations().some((animation) => animation instanceof CSSTransition))) return null;
    const now = { card: box.getBoundingClientRect(), hole: hole.getBoundingClientRect() };
    const seen = JSON.stringify(now);
    const same = seen === last;
    last = seen;
    return same ? now : null;
  }, 6000).then(async (found) => { await sleep(0); return found; });
}

/** What is wrong with the card on show, if anything. */
async function checkCard(expect: Expect, id: string) {
  const dict = words();
  const step = TOUR_STEPS.find((item) => item.id === id)!;
  const { card: box, hole } = await settled();
  const dialog = card()!;
  const title = document.getElementById(dialog.getAttribute("aria-labelledby") ?? "");
  const text = document.getElementById(dialog.getAttribute("aria-describedby") ?? "");
  // A server's page has no updates or statistics to speak of: its own text, where the step has one.
  const said = !("__TAURI_INTERNALS__" in window) && step.textServer ? step.textServer : step.text;
  expect(`${id}: the card's title and text are the dictionary's`,
    textOf(title) === squash(dict[step.title as Key]) && textOf(text) === squash(dict[said as Key]), `${textOf(title)} — ${textOf(text)}`);
  expect(`${id}: the card is inside the window`, box.left >= 0 && box.top >= 0 && box.right <= window.innerWidth && box.bottom <= window.innerHeight,
    `${Math.round(box.left)},${Math.round(box.top)} ${Math.round(box.width)}×${Math.round(box.height)} in ${window.innerWidth}×${window.innerHeight}`);
  if (step.target) {
    const target = document.querySelector<HTMLElement>(`[data-tour="${step.target}"]`);
    const shown = target?.getBoundingClientRect();
    // Kept a hair inside the window, so its ring shows at the edges.
    const covers = !!shown && hole.left <= Math.max(shown.left, 2) + 1 && hole.top <= Math.max(shown.top, 2) + 1
      && hole.right >= Math.min(shown.right, window.innerWidth - 2) - 1 && hole.bottom >= Math.min(shown.bottom, window.innerHeight - 2) - 1;
    expect(`${id}: the cut-out is over a visible ${step.target}`, !!target && visible(target) && covers,
      `hole ${Math.round(hole.left)},${Math.round(hole.top)} ${Math.round(hole.width)}×${Math.round(hole.height)}; ${describe(target)} ${shown ? `${Math.round(shown.left)},${Math.round(shown.top)} ${Math.round(shown.width)}×${Math.round(shown.height)}` : ""}`);
    const overlaps = !!shown && box.left < shown.right && box.right > shown.left && box.top < shown.bottom && box.bottom > shown.top;
    expect(`${id}: the card does not cover it`, !overlaps || step.target === "nav");
  } else {
    expect(`${id}: a card in the middle, nothing cut out`, hole.width === 0 && Math.abs(box.left + box.width / 2 - window.innerWidth / 2) < 2);
  }
}

async function next(expect: Expect, from: string) {
  const dict = words();
  const label = from === "welcome" ? dict["onboarding.start"] : dict["onboarding.next"];
  await click(button(card()!, label));
  await until(`the card after ${from}`, () => stepId() !== from);
  expect(`${from}: Next goes on`, true);
}

/** Up by itself on fresh data; or started from About. The welcome card, then Next to Experiments. */
export async function onboarding(expect: Expect, args: { mode?: string }) {
  await until("the sidebar", () => document.querySelector(".sidebar .nav-item"));
  const byItself = await until("the tour up by itself", () => tour(), 4000).then(() => true, () => false);
  if (!byItself) {
    expect("the tour was finished before, so it stays closed", localStorage.getItem(DONE_KEY) !== null);
    await click(button(document.querySelector(".header")!, words()["about.open"]));
    const about = await until("About", () => document.querySelector<HTMLDialogElement>("dialog.about-dialog[open]"));
    await click(button(about, words()["about.tour"]));
    await until("the tour from About", () => tour());
  }
  expect(`the tour opens on the first launch (${args.mode ?? "?"})`, byItself, "it had to be started from About");
  const dialog = card()!;
  expect("the card is a modal dialog named by its title", dialog.getAttribute("aria-modal") === "true" && !!dialog.getAttribute("aria-labelledby") && !!dialog.getAttribute("aria-describedby"));
  expect("the app behind it is inert", document.getElementById("root")?.inert === true);
  await checkCard(expect, "welcome");
  expect("the welcome offers Start the tour and Skip", !!button(dialog, words()["onboarding.start"]) && !!button(dialog, words()["onboarding.skipShort"]));
  expect("focus is in the card", dialog.contains(document.activeElement));
  const nameless = unnamed(dialog);
  expect("every button of the card has a name", nameless.length === 0, nameless.join(" | "));
  // A click on the dimmed window does nothing.
  document.elementFromPoint(5, window.innerHeight / 2)?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  expect("a click outside the card leaves it open", !!tour() && stepId() === "welcome");
  await next(expect, "welcome");
  await checkCard(expect, "nav");
  expect("…and shows 2 / n", textOf(card()!.querySelector(".tour-count")).startsWith("2"), textOf(card()!.querySelector(".tour-count")));
  expect("Skip tour is on every card after the first", !!button(card()!, words()["onboarding.skip"]));
  await next(expect, "nav");
  await checkCard(expect, "experiment");
}

/** On through the screens, the bottom panel and the header, Back once on the way; left open on About. */
export async function onboardingWalk(expect: Expect) {
  let id = stepId();
  if (!id) throw new Error("the tour is not open");
  for (const step of TOUR_STEPS.slice(TOUR_STEPS.findIndex((item) => item.id === id) + 1)) {
    await next(expect, id);
    id = stepId()!;
    expect(`${step.id}: comes next`, id === step.id, `${id} came`);
    await checkCard(expect, id);
    if (id === "dock") {
      await click(button(card()!, words()["onboarding.back"]));
      await until("Back", () => stepId() === "emulators");
      expect("Back goes to the card before", true);
      key(document.activeElement ?? document.body, { key: document.documentElement.dir === "rtl" ? "ArrowLeft" : "ArrowRight" });
      await until("→ on again", () => stepId() === "dock");
      expect("the arrow key goes on", true);
    }
    if (id === "about") break;
  }
}

/** The finish, Esc, About's Show the introduction tour, Skip — and the app free again. */
export async function onboardingEnd(expect: Expect) {
  if (stepId() !== "about") throw new Error(`the tour is on ${stepId() ?? "nothing"}, not About`);
  await next(expect, "about");
  await checkCard(expect, "finish");
  const dialog = card()!;
  expect("the last card offers First steps and Finish", !!button(dialog, words()["onboarding.firstSteps"]) && !!button(dialog, words()["onboarding.finish"]));
  key(document.activeElement ?? document.body, { key: "Escape", code: "Escape" });
  await until("Esc closes the tour", () => !tour());
  expect("Esc skips: the tour is closed and kept as done", localStorage.getItem(DONE_KEY) !== null);
  expect("…and the app takes clicks again", document.getElementById("root")?.inert === false);

  await click(button(document.querySelector(".header")!, words()["about.open"]));
  const about = await until("About", () => document.querySelector<HTMLDialogElement>("dialog.about-dialog[open]"));
  await click(button(about, words()["about.tour"]));
  await until("the tour again", () => tour());
  expect("Show the introduction tour closes About and starts the tour", !document.querySelector("dialog.about-dialog[open]") && stepId() === "welcome");
  await checkCard(expect, "welcome");
  await click(button(card()!, words()["onboarding.skipShort"]));
  await until("Skip closes it", () => !tour());
  expect("Skip closes it", true);

  // Nothing left in the way: a sidebar item is what a click there meets.
  const item = document.querySelector<HTMLElement>(".sidebar .nav-item")!;
  const box = item.getBoundingClientRect();
  const hit = document.elementFromPoint(box.left + box.width / 2, box.top + box.height / 2);
  expect("afterwards nothing covers the app", !!hit && (hit === item || item.contains(hit)), describe(hit));
  await sleep(500);
  expect("…and it does not open again by itself", !tour());
}
