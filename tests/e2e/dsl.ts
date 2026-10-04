/**
 * The tour's vocabulary: find a control by the words a person reads next to it,
 * click it the way a person can, type into it the way React sees typing, wait
 * for the screen to answer. Every step (steps/*.ts) is written in these words.
 */
import { en } from "../../src/lib/locales/en";
import { format } from "../../src/lib/translate";

export type Key = keyof typeof en;
export type Words = Record<Key, string>;
export type Params = Record<string, string | number>;
export type Control = HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;
export interface Check { name: string; ok: boolean; detail?: string }
export type StepArgs = Record<string, any>;

/** The English text of a key, as the interface shows it (placeholders and plurals filled the same way). */
export const T = (key: Key, params?: Params) => format(en[key], params, "en");
export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
export const squash = (text: string | null | undefined) => (text ?? "").replace(/\s+/g, " ").trim();
/** Text as it reads on screen: separate elements are separate words ("02 HTTP check", not "02HTTP check"). */
export function words(node: Node): string {
  if (node.nodeType === Node.TEXT_NODE) return node.textContent ?? "";
  let out = "";
  let afterElement = false;
  for (const child of node.childNodes) {
    const element = child.nodeType === Node.ELEMENT_NODE;
    out += (out && (element || afterElement) ? " " : "") + words(child);
    afterElement = element;
  }
  return out;
}
export const textOf = (element: Element | null | undefined) => element ? squash(words(element)) : "";
export const numberIn = (text: string) => Number(text.replace(/[^\d.-]/g, "")) || 0;

// ---- finding things ---------------------------------------------------------------

/** What a person would call an element in a failure message. */
export function describe(element: Element | null): string {
  if (!element) return "nothing";
  const name = element.getAttribute("aria-label") ?? textOf(element).slice(0, 40);
  return `<${element.tagName.toLowerCase()}${element.className ? ` class="${String(element.className).slice(0, 60)}"` : ""}>${name ? ` “${name}”` : ""}`;
}

export function visible(element: Element): boolean {
  const html = element as HTMLElement;
  if (html.closest("[hidden]")) return false;
  const rect = html.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0 && getComputedStyle(html).visibility !== "hidden";
}

export async function until<V>(what: string, probe: () => V | null | undefined | false, timeout = 8000): Promise<V> {
  const started = performance.now();
  for (;;) {
    let value: V | null | undefined | false = null;
    try { value = probe(); } catch { value = null; }
    if (value) return value;
    if (performance.now() - started > timeout) throw new Error(`timed out after ${timeout / 1000} s waiting for ${what}`);
    await sleep(50);
  }
}

/** The screen on show. */
export function screen(): HTMLElement {
  const host = [...document.querySelectorAll<HTMLElement>(".view-host, .experiment-host")].find((element) => !element.hidden);
  if (!host) throw new Error("no screen is showing");
  return host;
}

/** A label's own words, without the text of the controls inside it. */
export function labelText(label: Element): string {
  const copy = label.cloneNode(true) as Element;
  copy.querySelectorAll("input, select, textarea, option, .template-suggestions").forEach((element) => element.remove());
  return textOf(copy);
}

/**
 * The field a person would find by its label: a `.field` with that label, a
 * `<label>` wrapping its control, a `label[for]`, or an `aria-label`.
 * `index` picks among several equal labels (the second "Target" on a screen).
 */
export function control(root: ParentNode, label: string, index = 0): Control {
  const found: Control[] = [];
  const pick = (element: Element | null) => { if (element && visible(element) && !found.includes(element as Control)) found.push(element as Control); };
  for (const element of root.querySelectorAll("label")) {
    const words = labelText(element);
    if (words !== label && !words.startsWith(`${label} `)) continue;
    const own = element.querySelector("input:not([type=checkbox]), select, textarea");
    if (own) { pick(own); continue; }
    const forId = element.getAttribute("for");
    if (forId) { pick(document.getElementById(forId)); continue; }
    const field = element.closest(".field");
    pick(field?.querySelector("input:not([type=checkbox]), select, textarea") ?? null);
  }
  for (const element of root.querySelectorAll(`[aria-label="${CSS.escape(label)}"]`)) {
    if (element.matches("input, select, textarea")) pick(element);
  }
  const element = found[index];
  if (!element) throw new Error(`no field labelled “${label}”${index ? ` (#${index + 1})` : ""} on this screen`);
  return element;
}

/** A checkbox by the words next to it. */
export function checkbox(root: ParentNode, label: string): HTMLInputElement {
  for (const element of root.querySelectorAll("label")) {
    const box = element.querySelector<HTMLInputElement>("input[type=checkbox]");
    if (box && labelText(element) === label && visible(element)) return box;
  }
  throw new Error(`no checkbox “${label}” on this screen`);
}

/** A visible button by its text or accessible name (exact, or starting with `label` when `prefix`). */
export function button(root: ParentNode, label: string, { prefix = false, index = 0 } = {}): HTMLButtonElement {
  const matches = [...root.querySelectorAll<HTMLButtonElement>("button")].filter((element) => {
    if (!visible(element)) return false;
    const names = [textOf(element), squash(element.getAttribute("aria-label"))];
    return names.some((name) => prefix ? name.startsWith(label) : name === label);
  });
  const element = matches[index];
  if (!element) throw new Error(`no button “${label}” on this screen`);
  return element;
}

export const hasButton = (root: ParentNode, label: string, prefix = false) => { try { return button(root, label, { prefix }); } catch { return null; } };

/** A visible button whose text contains `words` (a count in front, a glyph before it). */
export function buttonWith(root: ParentNode, words: string): HTMLButtonElement {
  const element = [...root.querySelectorAll<HTMLButtonElement>("button")].find((item) => visible(item) && textOf(item).includes(words));
  if (!element) throw new Error(`no button with “${words}” on this screen`);
  return element;
}

/** The value shown under a metric's caption. */
export function metric(root: ParentNode, caption: string): string {
  const card = [...root.querySelectorAll(".metric")].find((element) => textOf(element.querySelector(".k")) === caption);
  if (!card) throw new Error(`no metric “${caption}” on this screen`);
  return textOf(card.querySelector(".v"));
}

/** The panel whose section label reads `title`. */
export function panel(root: ParentNode, title: string): HTMLElement {
  const label = [...root.querySelectorAll(".section-label")].find((element) => textOf(element) === title && visible(element));
  const found = label?.closest<HTMLElement>(".panel");
  if (!found) throw new Error(`no panel “${title}” on this screen`);
  return found;
}

/**
 * Controls on screen that a screen reader cannot name: a field with no label
 * tied to it (a label next to it is not enough), or a button whose only text is
 * a glyph such as ✕ or ☰ and that has no aria-label.
 */
export function unnamed(root: ParentNode): string[] {
  const found: string[] = [];
  const named = (element: Element) => {
    if (squash(element.getAttribute("aria-label"))) return true;
    const by = element.getAttribute("aria-labelledby");
    if (by && by.split(/\s+/).some((id) => textOf(document.getElementById(id)))) return true;
    if (element.id && [...document.querySelectorAll(`label[for="${CSS.escape(element.id)}"]`)].some((label) => labelText(label))) return true;
    const wrapping = element.closest("label");
    return !!wrapping && !!labelText(wrapping);
  };
  for (const element of root.querySelectorAll("input:not([type=hidden]), select, textarea")) {
    if (!visible(element) || named(element)) continue;
    const field = element.closest(".field")?.querySelector("label");
    found.push(`${element.tagName.toLowerCase()}${field ? ` under “${labelText(field)}”` : (element as HTMLInputElement).placeholder ? ` “${(element as HTMLInputElement).placeholder}”` : ""}`);
  }
  for (const element of root.querySelectorAll("button")) {
    if (!visible(element) || squash(element.getAttribute("aria-label")) || /[\p{L}\p{N}]/u.test(textOf(element))) continue;
    found.push(`button “${textOf(element)}”`);
  }
  return found;
}

/**
 * Icon buttons with no tooltip: a ✕ or ☰ says nothing until the pointer
 * rests on it, so each one needs a `data-tip` (in the current language).
 */
export function untipped(root: ParentNode): string[] {
  return [...root.querySelectorAll<HTMLButtonElement>("button")]
    .filter((element) => visible(element) && !/[\p{L}\p{N}]/u.test(textOf(element)) && !squash(element.getAttribute("data-tip")) && !element.closest("[data-tip]")?.contains(element.parentElement ?? element))
    .map((element) => `“${textOf(element)}” (${squash(element.getAttribute("aria-label")) || "no name"})`);
}

// ---- doing things -------------------------------------------------------------------

/**
 * A click a person could make: the control is on screen, enabled and not under
 * something else. A view may still be moving something into view (a reveal
 * after a screen opened); a person waits for that, so the check is made again
 * for a moment before it counts as covered.
 */
/** How often the firewall notice was in the way and answered (the shell step reports it); an object, so that step can add to it. */
export const firewall = { notices: 0 };

export async function click(element: HTMLElement, what = describe(element)) {
  if ((element as HTMLButtonElement).disabled) throw new Error(`${what} is disabled`);
  let hit: Element | null = null;
  for (let attempt = 0; attempt < 8; attempt++) {
    element.scrollIntoView({ block: "center", inline: "nearest" });
    await sleep(attempt === 0 ? 0 : 120);
    const rect = element.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) throw new Error(`${what} has no size`);
    hit = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2);
    if (!hit || hit === element || element.contains(hit) || hit.contains(element)) {
      element.click();
      await sleep(30);
      return;
    }
    // The firewall notice of the desktop app (a fresh machine's first listener): a person answers "Not now".
    const notice = hit.closest(".firewall-banner");
    if (notice && !notice.contains(element)) {
      firewall.notices += 1;
      const buttons = notice.querySelectorAll<HTMLButtonElement>("button");
      buttons[buttons.length - 1]?.click();
      await sleep(50);
    }
  }
  throw new Error(`${what} is covered by ${describe(hit!)}`);
}

/** Type a value the way React sees typing: the native setter, then input and change. */
export async function type(element: Control, value: string | number) {
  if (element.disabled) throw new Error(`${describe(element)} is disabled`);
  element.focus();
  const proto = element instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype
    : element instanceof HTMLSelectElement ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
  Object.getOwnPropertyDescriptor(proto, "value")!.set!.call(element, String(value));
  element.dispatchEvent(new Event("input", { bubbles: true }));
  element.dispatchEvent(new Event("change", { bubbles: true }));
  await sleep(20);
  if (element instanceof HTMLSelectElement ? element.value !== String(value) : element.value !== String(value)) {
    throw new Error(`${describe(element)} did not take “${value}” (shows “${element.value}”)`);
  }
}

export async function setChecked(box: HTMLInputElement, on: boolean) {
  if (box.checked !== on) await click(box, `checkbox ${describe(box)}`);
  if (box.checked !== on) throw new Error(`checkbox ${describe(box)} did not change`);
}

export function key(target: EventTarget, init: KeyboardEventInit) {
  target.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init }));
  target.dispatchEvent(new KeyboardEvent("keyup", { bubbles: true, cancelable: true, ...init }));
}

export const NAV: Record<string, Key> = {
  experiment: "nav.experiment", signals: "nav.signals", emulators: "nav.emulators", osc: "nav.osc", mqtt: "nav.mqtt", broadcast: "nav.broadcast",
  http: "nav.http", ws: "nav.ws", netsim: "nav.netsim", storm: "nav.storm", scan: "nav.scan",
};
export const TITLES: Record<string, Key> = {
  signals: "sig.title", emulators: "emu.title", osc: "osc.title", mqtt: "mq.title", broadcast: "bc.title",
  http: "http.title", ws: "ws.title", netsim: "ns.title", storm: "st.title", scan: "sc.title",
};

export async function go(view: string, dict: Words = en): Promise<HTMLElement> {
  const label = dict[NAV[view]];
  const item = [...document.querySelectorAll<HTMLButtonElement>(".sidebar .nav-item")].find((element) => element.getAttribute("aria-label") === label);
  if (!item) throw new Error(`no navigation item “${label}”`);
  await click(item, `navigation “${label}”`);
  await until(`the ${view} screen`, () => item.getAttribute("aria-current") === "page");
  const shown = screen();
  if (view === "experiment") await until("the experiment editor", () => shown.querySelector(".experiment-toolbar"));
  return shown;
}

/** A send result line under a sender (`.send-result`), once it has appeared or changed. */
export const result = (root: ParentNode) => root.querySelector<HTMLElement>(".send-result");

/** Lines in the console strip, newest last. */
export const logLines = () => [...document.querySelectorAll(".log-list .log-line .msg")].map(textOf);

/** The interface in `code`, chosen from the header's language menu as a person does. */
export async function chooseLanguage(code: string) {
  const opener = document.querySelector<HTMLButtonElement>(".lang-menu-button");
  if (!opener) throw new Error("no language menu in the header");
  if (document.documentElement.lang === code) return;
  await click(opener, "the language menu");
  const item = await until(`${code} in the language menu`, () => document.querySelector<HTMLButtonElement>(`.lang-menu-list:popover-open [data-lang="${code}"]`));
  await click(item, `the language ${code}`);
  await until(`the interface in ${code}`, () => document.documentElement.lang === code);
}

export function closeOverlays() {
  // Menus, popovers and dialogs all close on Escape; a modal <dialog> also on its cancel.
  for (const dialog of document.querySelectorAll("dialog[open]")) dialog.dispatchEvent(new Event("cancel", { cancelable: true }));
  key(document.activeElement ?? document.body, { key: "Escape", code: "Escape" });
}

export type Expect = (name: string, ok: boolean, detail?: string) => void;

/** The Inspector, in the bottom panel, opened from its tab (from any screen). */
export async function openInspector(dict: Words = en): Promise<HTMLElement> {
  const tab = [...document.querySelectorAll<HTMLButtonElement>(".dock-tab")].find((element) => textOf(element).includes(dict["dock.inspector"]));
  if (!tab) throw new Error("no Inspector tab in the bottom panel");
  await click(tab, "the Inspector tab");
  return until("the Inspector", () => { const panel = document.querySelector<HTMLElement>("#dock-inspector"); return panel && visible(panel) ? panel : null; });
}
