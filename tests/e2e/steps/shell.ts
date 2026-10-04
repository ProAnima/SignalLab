/** The frame around the screens: the language, every screen's names, the interface in Russian and every other language, and what is left at the end. */
import { en } from "../../../src/lib/locales/en";
import { ru } from "../../../src/lib/locales/ru";
import { LOCALES } from "../../../src/lib/locales/index";
import { format } from "../../../src/lib/translate";
import { T, sleep, squash, textOf, numberIn, visible, until, control, button, buttonWith, panel, unnamed, untipped, click, NAV, TITLES, go, closeOverlays, openInspector, firewall, chooseLanguage, type Key, type Words, type StepArgs, type Expect } from "../dsl";

/** What the starter set and the captured signals' folder were called when they were written: documents now, kept as they are. */
const written = (name: string) => name.startsWith("seed.") || name === "sig.capturedFolder";

/**
 * The English texts that would mean a screen was left untranslated in `dict`:
 * long enough to be words, not units; translated otherwise there; and not
 * what that language itself writes somewhere (a German "Status" is German too).
 */
function englishFor(dict: Words): Set<string> {
  const own = new Set(Object.values(dict));
  const words: Words = en;
  return new Set(Object.keys(words)
    .filter((name) => !written(name) && words[name as Key] !== dict[name as Key] && /[a-z]{3}/i.test(words[name as Key]) && !words[name as Key].includes("{") && !own.has(words[name as Key]))
    .map((name) => words[name as Key]));
}

/** The English texts (and labels, tooltips) still shown in `root`. */
function leftovers(root: Element, english: Set<string>): string[] {
  const found = new Set<string>();
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = squash(node.textContent);
    if (text && english.has(text) && node.parentElement && visible(node.parentElement)) found.add(text);
  }
  for (const element of root.querySelectorAll("[placeholder], [aria-label], [data-tip]")) {
    for (const name of ["placeholder", "aria-label", "data-tip"]) {
      const value = squash(element.getAttribute(name));
      if (english.has(value)) found.add(`${name}=${value}`);
    }
  }
  return [...found];
}

export async function shell(expect: Expect, args: StepArgs) {
  await until("the sidebar", () => document.querySelectorAll(".sidebar .nav-item").length === Object.keys(NAV).length);
  await chooseLanguage("en");
  expect("the interface is in English", document.documentElement.lang === "en" && document.documentElement.dir === "ltr");

  const chip = await until("the host chip", () => document.querySelector(".host-chip"));
  expect("the host chip names this machine and its address", /\S+ · \d+\.\d+\.\d+\.\d+/.test(textOf(chip)), textOf(chip));
  expect(args.mode === "server" ? "the Server badge is shown" : "no Server badge in the desktop app",
    !!document.querySelector(".server-badge") === (args.mode === "server"));
  expect("the sidebar shows the version", textOf(document.querySelector(".sidebar .foot")).includes(T("app.version", { version: args.version })), textOf(document.querySelector(".sidebar .foot")));
  expect("no connection banner", !document.querySelector(".connection-banner"));

  for (const view of Object.keys(NAV)) {
    const shown = await go(view);
    if (view === "experiment") {
      expect("Experiments opens the editor", !!shown.querySelector(".experiment-canvas"));
    } else {
      const heading = textOf(shown.querySelector("h1"));
      expect(`${view}: heading “${T(TITLES[view])}”`, heading === T(TITLES[view]), heading);
    }
    const nameless = unnamed(shown);
    expect(`${view}: every field and button has a name`, nameless.length === 0, nameless.join(" | "));
    const silent = untipped(shown);
    expect(`${view}: every icon button has a tooltip`, silent.length === 0, silent.join(" | "));
    const main = document.querySelector<HTMLElement>(".main")!;
    expect(`${view}: nothing wider than the window`, document.documentElement.scrollWidth <= window.innerWidth + 1 && main.scrollWidth <= main.clientWidth + 1,
      `page ${document.documentElement.scrollWidth} / window ${window.innerWidth}, main ${main.scrollWidth} / ${main.clientWidth}`);
  }
  // A field's help is its label's tooltip — on keyboard focus too, read to screen readers.
  const osc = await go("osc");
  const target = control(panel(osc, T("osc.sender")), T("common.target"));
  await sleep(200);
  target.dispatchEvent(new FocusEvent("focusin", { bubbles: true }));
  const tip = await until("the tooltip", () => document.querySelector<HTMLElement>("#signal-lab-tooltip"));
  expect("focus on a field shows its label's help", textOf(tip) === T("common.targetHint"), textOf(tip));
  expect("…and the field is described by it", (target.getAttribute("aria-describedby") ?? "").includes("signal-lab-tooltip"));
  target.dispatchEvent(new FocusEvent("focusout", { bubbles: true }));
  await until("the tooltip gone", () => !document.querySelector("#signal-lab-tooltip"));

  const toggle = document.querySelector<HTMLButtonElement>(".console-toggle")!;
  await click(toggle, "console toggle");
  expect("the console opens", toggle.getAttribute("aria-expanded") === "true");
  await click(toggle, "console toggle");
  expect("…and closes", toggle.getAttribute("aria-expanded") === "false");
}

/** Every screen in Russian: headings from the Russian dictionary and no English text left on screen. */
export async function russian(expect: Expect) {
  await chooseLanguage("ru");
  // Not the ones that became a person's document when written — the starter set, the folder of captured
  // signals: the library was created in English at the start of the tour and stays as it was written.
  const english = englishFor(ru);
  for (const view of Object.keys(NAV)) {
    const shown = await go(view, ru);
    if (view !== "experiment") expect(`${view}: heading in Russian`, textOf(shown.querySelector("h1")) === ru[TITLES[view]], textOf(shown.querySelector("h1")));
    const left = leftovers(shown, english);
    expect(`${view}: no English left`, left.length === 0, left.slice(0, 8).join(" | "));
  }
  // Counts agree with their number, by the language's own plural forms.
  const signals = await go("signals", ru);
  const count = textOf([...signals.querySelectorAll(".sig-foot span")].find((element) => /\d/.test(textOf(element))));
  expect("a count reads as Russian", count === format(ru["sig.count"], { n: numberIn(count) }, "ru"), count);
  // The frame around the screens: sidebar, header, and the bottom panel with each of its tabs.
  const inspector = await openInspector(ru);
  const frame = [document.querySelector(".sidebar")!, document.querySelector(".header")!, document.querySelector(".console")!, inspector].flatMap((root) => leftovers(root, english));
  expect("sidebar, header and bottom panel: no English left", frame.length === 0, [...new Set(frame)].slice(0, 8).join(" | "));
  await chooseLanguage("en");
}

/**
 * What is wrong with the tooltip on show, if anything: it must be an element's
 * tip as that element has it now, beside that element. Choosing a language from
 * the keyboard gives the menu's button focus again — and its tooltip — while the
 * page changes language and, for Arabic, turns round.
 */
function staleTip(): string | null {
  const tip = document.getElementById("signal-lab-tooltip");
  if (!tip?.classList.contains("shown")) return null;
  const text = squash(tip.textContent);
  const owner = [...document.querySelectorAll<HTMLElement>("[data-tip]")].find((element) => squash(element.dataset.tip) === text && visible(element));
  if (!owner) return `“${text}” is no element's tooltip now`;
  const box = owner.getBoundingClientRect();
  const shown = tip.getBoundingClientRect();
  return shown.right < box.left - 8 || shown.left > box.right + 8 ? `“${text}” is shown away from its element` : null;
}

/**
 * Every other language, chosen from the menu: each one listed with its flag;
 * on every screen, headings from its dictionary and no English left, nothing
 * wider than the window; Arabic mirrors the page and keeps the canvas left to
 * right. English again at the end.
 */
export async function languages(expect: Expect) {
  const opener = document.querySelector<HTMLButtonElement>(".lang-menu-button")!;
  await click(opener, "the language menu");
  const menu = await until("the language menu", () => document.querySelector<HTMLElement>(".lang-menu-list:popover-open"));
  const items = [...menu.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')];
  expect("the menu lists every language, in the order of the dictionaries", items.map((item) => item.dataset.lang).join() === LOCALES.map((locale) => locale.code).join(), items.map((item) => item.dataset.lang).join());
  const flags = [...menu.querySelectorAll<HTMLImageElement>("img.flag")];
  await until("the flags drawn", () => flags.every((flag) => flag.complete && flag.naturalWidth > 0));
  expect("…each with its flag and its own name", flags.length === LOCALES.length && items.every((item, index) => textOf(item).includes(LOCALES[index].name)));
  closeOverlays();
  await until("the menu closed", () => !document.querySelector(".lang-menu-list:popover-open"));

  for (const { code, dict } of LOCALES) {
    if (code === "en" || code === "ru") continue;
    await chooseLanguage(code);
    await sleep(100);
    const stale = staleTip();
    expect(`${code}: a tooltip still showing is in the language, by its element`, !stale, stale ?? "");
    const words = dict as Words;
    const english = englishFor(words);
    const misses: string[] = [];
    for (const view of Object.keys(NAV)) {
      const shown = await go(view, words);
      if (view !== "experiment" && textOf(shown.querySelector("h1")) !== words[TITLES[view]]) misses.push(`${view}: heading “${textOf(shown.querySelector("h1"))}”`);
      for (const left of leftovers(shown, english)) misses.push(`${view}: ${left}`);
      if (document.documentElement.scrollWidth > window.innerWidth + 1) misses.push(`${view}: wider than the window`);
    }
    for (const root of [document.querySelector(".sidebar")!, document.querySelector(".header")!, document.querySelector(".console")!]) {
      for (const left of leftovers(root, english)) misses.push(`frame: ${left}`);
    }
    expect(`${code}: every screen in the language`, misses.length === 0, misses.slice(0, 8).join(" | "));
    const rtl = code === "ar";
    expect(`${code}: the page reads ${rtl ? "right to left" : "left to right"}`, document.documentElement.dir === (rtl ? "rtl" : "ltr"));
    if (rtl) {
      const sidebar = document.querySelector(".sidebar")!.getBoundingClientRect();
      const main = document.querySelector(".main")!.getBoundingClientRect();
      expect("ar: the sidebar is on the right", sidebar.left > main.left, `sidebar ${Math.round(sidebar.left)}, main ${Math.round(main.left)}`);
      const editor = await go("experiment", words);
      expect("ar: the experiment canvas stays left to right", getComputedStyle(editor.querySelector(".experiment-canvas-scroll")!).direction === "ltr");
    }
  }
  await chooseLanguage("en");
  await sleep(100);
  const stale = staleTip();
  expect("back in English from Arabic: a tooltip still showing is English, by its element", !stale, stale ?? "");
}

export async function cleanup(expect: Expect) {
  closeOverlays();
  const stopAll = button(document.querySelector(".header")!, T("app.stopAll"));
  if (!stopAll.disabled) await click(stopAll);
  await until("no jobs", () => textOf(document.querySelector(".sidebar .foot")).includes(T("app.activeJobs", { n: 0 })), 5000);
  expect("no job is left running", true);
  expect("no connection banner appeared", !document.querySelector(".connection-banner"));
  // On a machine whose firewall has no rule for the app yet, the notice came up and was answered "Not now".
  const notice = document.querySelector(".firewall-banner");
  if (notice) {
    await click(buttonWith(notice as HTMLElement, T("fw.dismiss")));
    firewall.notices += 1;
  }
  expect("the firewall notice, when it came, was answered", !document.querySelector(".firewall-banner"), firewall.notices ? `answered ${firewall.notices}×` : "it did not come");
}
