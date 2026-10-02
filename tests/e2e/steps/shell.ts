/** The frame around the screens: the language, every screen's names, the interface in Russian, and what is left at the end. */
import { en } from "../../../src/lib/locales/en";
import { ru } from "../../../src/lib/locales/ru";
import { format } from "../../../src/lib/translate";
import { T, sleep, squash, textOf, numberIn, visible, until, control, button, buttonWith, panel, unnamed, untipped, click, NAV, TITLES, go, closeOverlays, openInspector, firewall, type Key, type Words, type StepArgs, type Expect } from "../dsl";

export async function shell(expect: Expect, args: StepArgs) {
  await until("the sidebar", () => document.querySelectorAll(".sidebar .nav-item").length === Object.keys(NAV).length);
  const english = [...document.querySelectorAll<HTMLButtonElement>(".lang-switch button")].find((element) => textOf(element) === "EN");
  if (!english) throw new Error("no language switch");
  await click(english, "EN");
  await until("English", () => document.documentElement.lang === "en");
  expect("the interface is in English", document.documentElement.lang === "en");

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
  const russianButton = [...document.querySelectorAll<HTMLButtonElement>(".lang-switch button")].find((element) => textOf(element) === "RU")!;
  await click(russianButton, "RU");
  await until("Russian", () => document.documentElement.lang === "ru");
  // English texts that Russian translates differently and that are long enough to be words, not units.
  // Not the ones that become a person's document when written — the starter set, the folder of captured
  // signals: the library was created in English at the start of the tour and stays as it was written.
  const words: Words = en;
  const written = (name: string) => name.startsWith("seed.") || name === "sig.capturedFolder";
  const english = new Set<string>(Object.keys(words).filter((name) => !written(name) && words[name as Key] !== ru[name as Key] && /[a-z]{3}/i.test(words[name as Key]) && !words[name as Key].includes("{")).map((name) => words[name as Key]));
  const leftovers = (root: Element) => {
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
  };
  for (const view of Object.keys(NAV)) {
    const shown = await go(view, ru);
    if (view !== "experiment") expect(`${view}: heading in Russian`, textOf(shown.querySelector("h1")) === ru[TITLES[view]], textOf(shown.querySelector("h1")));
    const left = leftovers(shown);
    expect(`${view}: no English left`, left.length === 0, left.slice(0, 8).join(" | "));
  }
  // Counts agree with their number, by the language's own plural forms.
  const signals = await go("signals", ru);
  const count = textOf([...signals.querySelectorAll(".sig-foot span")].find((element) => /\d/.test(textOf(element))));
  expect("a count reads as Russian", count === format(ru["sig.count"], { n: numberIn(count) }, "ru"), count);
  // The frame around the screens: sidebar, header, and the bottom panel with each of its tabs.
  const inspector = await openInspector(ru);
  const frame = [...leftovers(document.querySelector(".sidebar")!), ...leftovers(document.querySelector(".header")!), ...leftovers(document.querySelector(".console")!), ...leftovers(inspector)];
  expect("sidebar, header and bottom panel: no English left", frame.length === 0, [...new Set(frame)].slice(0, 8).join(" | "));
  const englishButton = [...document.querySelectorAll<HTMLButtonElement>(".lang-switch button")].find((element) => textOf(element) === "EN")!;
  await click(englishButton, "EN");
  await until("English", () => document.documentElement.lang === "en");
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
