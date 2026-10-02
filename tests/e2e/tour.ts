/**
 * The end-to-end tour, run inside the real interface — the desktop app's
 * webview or a browser on the server — by scripts/e2e.mjs. Every step works
 * the screens the way a person does: it finds controls by their visible
 * labels (from the English dictionary, so a renamed text does not break it),
 * types into fields, presses buttons, and reads the result off the screen.
 * The runner checks the far end of each send against its loopback fixtures.
 *
 * A step returns its checks; a step that cannot go on throws. The runner
 * calls them one by one (`start` + `poll`), screenshots each, and does the
 * host-side work between them.
 */
import { en } from "../../src/lib/locales/en";
import { ru } from "../../src/lib/locales/ru";
import { format } from "../../src/lib/translate";

type Key = keyof typeof en;
type Words = Record<Key, string>;
type Params = Record<string, string | number>;
type Control = HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;
export interface Check { name: string; ok: boolean; detail?: string }
type StepArgs = Record<string, any>;

/** The English text of a key, as the interface shows it (placeholders and plurals filled the same way). */
const T = (key: Key, params?: Params) => format(en[key], params, "en");
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
const squash = (text: string | null | undefined) => (text ?? "").replace(/\s+/g, " ").trim();
/** Text as it reads on screen: separate elements are separate words ("02 HTTP check", not "02HTTP check"). */
function words(node: Node): string {
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
const textOf = (element: Element | null | undefined) => element ? squash(words(element)) : "";
const numberIn = (text: string) => Number(text.replace(/[^\d.-]/g, "")) || 0;

// ---- page errors, collected from the moment the tour is injected -----------------

const pageErrors: string[] = [];
window.addEventListener("error", (event) => pageErrors.push(`error: ${event.message} @ ${event.filename}:${event.lineno}`));
window.addEventListener("unhandledrejection", (event) => pageErrors.push(`unhandled rejection: ${String((event.reason as Error)?.message ?? JSON.stringify(event.reason))}`));
const consoleError = console.error.bind(console);
console.error = (...args: unknown[]) => { pageErrors.push(`console.error: ${args.map((arg) => typeof arg === "string" ? arg : (arg as Error)?.message ?? JSON.stringify(arg)).join(" ")}`); consoleError(...args); };

// Downloads in a browser: the link is recorded and fetched by the tour instead of saved.
const downloads: string[] = [];
document.addEventListener("click", (event) => {
  const link = (event.target as Element | null)?.closest?.("a[download]") as HTMLAnchorElement | null;
  if (!link) return;
  event.preventDefault();
  downloads.push(link.href);
}, true);

// ---- finding things ---------------------------------------------------------------

/** What a person would call an element in a failure message. */
function describe(element: Element | null): string {
  if (!element) return "nothing";
  const name = element.getAttribute("aria-label") ?? textOf(element).slice(0, 40);
  return `<${element.tagName.toLowerCase()}${element.className ? ` class="${String(element.className).slice(0, 60)}"` : ""}>${name ? ` “${name}”` : ""}`;
}

function visible(element: Element): boolean {
  const html = element as HTMLElement;
  if (html.closest("[hidden]")) return false;
  const rect = html.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0 && getComputedStyle(html).visibility !== "hidden";
}

async function until<V>(what: string, probe: () => V | null | undefined | false, timeout = 8000): Promise<V> {
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
function screen(): HTMLElement {
  const host = [...document.querySelectorAll<HTMLElement>(".view-host, .experiment-host")].find((element) => !element.hidden);
  if (!host) throw new Error("no screen is showing");
  return host;
}

/** A label's own words, without the text of the controls inside it. */
function labelText(label: Element): string {
  const copy = label.cloneNode(true) as Element;
  copy.querySelectorAll("input, select, textarea, option, .template-suggestions").forEach((element) => element.remove());
  return textOf(copy);
}

/**
 * The field a person would find by its label: a `.field` with that label, a
 * `<label>` wrapping its control, a `label[for]`, or an `aria-label`.
 * `index` picks among several equal labels (the second "Target" on a screen).
 */
function control(root: ParentNode, label: string, index = 0): Control {
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
function checkbox(root: ParentNode, label: string): HTMLInputElement {
  for (const element of root.querySelectorAll("label")) {
    const box = element.querySelector<HTMLInputElement>("input[type=checkbox]");
    if (box && labelText(element) === label && visible(element)) return box;
  }
  throw new Error(`no checkbox “${label}” on this screen`);
}

/** A visible button by its text or accessible name (exact, or starting with `label` when `prefix`). */
function button(root: ParentNode, label: string, { prefix = false, index = 0 } = {}): HTMLButtonElement {
  const matches = [...root.querySelectorAll<HTMLButtonElement>("button")].filter((element) => {
    if (!visible(element)) return false;
    const names = [textOf(element), squash(element.getAttribute("aria-label"))];
    return names.some((name) => prefix ? name.startsWith(label) : name === label);
  });
  const element = matches[index];
  if (!element) throw new Error(`no button “${label}” on this screen`);
  return element;
}

const hasButton = (root: ParentNode, label: string, prefix = false) => { try { return button(root, label, { prefix }); } catch { return null; } };

/** A visible button whose text contains `words` (a count in front, a glyph before it). */
function buttonWith(root: ParentNode, words: string): HTMLButtonElement {
  const element = [...root.querySelectorAll<HTMLButtonElement>("button")].find((item) => visible(item) && textOf(item).includes(words));
  if (!element) throw new Error(`no button with “${words}” on this screen`);
  return element;
}

/** The value shown under a metric's caption. */
function metric(root: ParentNode, caption: string): string {
  const card = [...root.querySelectorAll(".metric")].find((element) => textOf(element.querySelector(".k")) === caption);
  if (!card) throw new Error(`no metric “${caption}” on this screen`);
  return textOf(card.querySelector(".v"));
}

/** The panel whose section label reads `title`. */
function panel(root: ParentNode, title: string): HTMLElement {
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
function unnamed(root: ParentNode): string[] {
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
function untipped(root: ParentNode): string[] {
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
/** How often the firewall notice was in the way and answered (the shell step reports it). */
let firewallNotices = 0;

async function click(element: HTMLElement, what = describe(element)) {
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
      firewallNotices += 1;
      const buttons = notice.querySelectorAll<HTMLButtonElement>("button");
      buttons[buttons.length - 1]?.click();
      await sleep(50);
    }
  }
  throw new Error(`${what} is covered by ${describe(hit!)}`);
}

/** Type a value the way React sees typing: the native setter, then input and change. */
async function type(element: Control, value: string | number) {
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

async function setChecked(box: HTMLInputElement, on: boolean) {
  if (box.checked !== on) await click(box, `checkbox ${describe(box)}`);
  if (box.checked !== on) throw new Error(`checkbox ${describe(box)} did not change`);
}

function key(target: EventTarget, init: KeyboardEventInit) {
  target.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init }));
  target.dispatchEvent(new KeyboardEvent("keyup", { bubbles: true, cancelable: true, ...init }));
}

const NAV: Record<string, Key> = {
  experiment: "nav.experiment", signals: "nav.signals", emulators: "nav.emulators", osc: "nav.osc", mqtt: "nav.mqtt", broadcast: "nav.broadcast",
  http: "nav.http", netsim: "nav.netsim", storm: "nav.storm", scan: "nav.scan",
};
const TITLES: Record<string, Key> = {
  signals: "sig.title", emulators: "emu.title", osc: "osc.title", mqtt: "mq.title", broadcast: "bc.title",
  http: "http.title", netsim: "ns.title", storm: "st.title", scan: "sc.title",
};

async function go(view: string, dict: Words = en): Promise<HTMLElement> {
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
const result = (root: ParentNode) => root.querySelector<HTMLElement>(".send-result");

/** Lines in the console strip, newest last. */
const logLines = () => [...document.querySelectorAll(".log-list .log-line .msg")].map(textOf);

function closeOverlays() {
  // Menus, popovers and dialogs all close on Escape; a modal <dialog> also on its cancel.
  for (const dialog of document.querySelectorAll("dialog[open]")) dialog.dispatchEvent(new Event("cancel", { cancelable: true }));
  key(document.activeElement ?? document.body, { key: "Escape", code: "Escape" });
}

// ---- the steps ---------------------------------------------------------------------

type Expect = (name: string, ok: boolean, detail?: string) => void;

async function shell(expect: Expect, args: StepArgs) {
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

/** The Inspector, in the bottom panel, opened from its tab (from any screen). */
async function openInspector(dict: Words = en): Promise<HTMLElement> {
  const tab = [...document.querySelectorAll<HTMLButtonElement>(".dock-tab")].find((element) => textOf(element).includes(dict["dock.inspector"]));
  if (!tab) throw new Error("no Inspector tab in the bottom panel");
  await click(tab, "the Inspector tab");
  return until("the Inspector", () => { const panel = document.querySelector<HTMLElement>("#dock-inspector"); return panel && visible(panel) ? panel : null; });
}

async function inspectArm(expect: Expect) {
  const shown = await openInspector();
  const nameless = unnamed(shown);
  expect("inspector: every field and button has a name", nameless.length === 0, nameless.join(" | "));
  if (!hasButton(shown, T("ins.disarm"))) await click(button(shown, T("ins.arm")));
  await until("capture armed", () => hasButton(shown, T("ins.disarm")));
  expect("capture is armed", !!shown.querySelector(".rec-dot.live"));
}

async function osc(expect: Expect, args: StepArgs) {
  const shown = await go("osc");
  const monitor = panel(shown, T("osc.monitor"));
  await type(control(monitor, T("common.bind")), `127.0.0.1:${args.port}`);
  await click(button(monitor, T("osc.listen")));
  await until("the monitor to listen", () => hasButton(monitor, T("common.stop")));
  expect("the monitor runs as a job", !!document.querySelector(".job-pill"));

  const sender = panel(shown, T("osc.sender"));
  await type(control(sender, T("common.target")), `127.0.0.1:${args.port}`);
  await type(control(sender, T("common.address")), "/e2e/osc");
  // Start from one float argument and add a string, through the argument editor.
  while (sender.querySelectorAll(".row.tight").length > 1) await click(sender.querySelector<HTMLButtonElement>(".row.tight button")!, "remove argument");
  if (!sender.querySelector(".row.tight")) await click(button(sender, T("common.addArgument")));
  const first = sender.querySelector(".row.tight")!;
  await type(first.querySelector("select")!, "float");
  await type(first.querySelector("input")!, "0.5");
  await click(button(sender, T("common.addArgument")));
  const second = sender.querySelectorAll(".row.tight")[1];
  await type(second.querySelector("select")!, "str");
  await type(second.querySelector("input")!, "hello e2e");
  await click(button(sender, T("common.send")));
  const sent = await until("the send result", () => result(sender)?.classList.contains("ok") && result(sender));
  expect("Send reports the bytes and the target", textOf(sent).includes(`/e2e/osc · `) && textOf(sent).includes(`127.0.0.1:${args.port}`), textOf(sent));
  const row = await until("the message in the monitor", () => [...monitor.querySelectorAll("tbody tr")].find((tr) => textOf(tr).includes("/e2e/osc")));
  expect("the monitor decodes both arguments", textOf(row).includes("0.5") && textOf(row).includes("hello e2e"), textOf(row));

  // Enter in the address field sends again; the result counts the repeat.
  key(control(sender, T("common.address")), { key: "Enter", code: "Enter" });
  await until("a repeat count", () => textOf(result(sender)).includes("×2"));
  expect("Enter sends again (×2)", true);

  // The generator, into the same monitor.
  const generator = panel(shown, T("osc.generator"));
  await type(control(generator, T("common.target")), `127.0.0.1:${args.port}`);
  await type(control(generator, T("osc.address")), "/e2e/lfo");
  await type(control(generator, T("osc.rate")), 20);
  await click(button(generator, T("osc.startGen")));
  await until("generated messages", () => [...monitor.querySelectorAll("tbody tr")].some((tr) => textOf(tr).includes("/e2e/lfo")));
  const canvas = generator.querySelector("canvas");
  expect("the generator's scope draws", !!canvas && canvas.width > 0);
  await click(button(generator, T("osc.stopGen")));
  await until("the generator to stop", () => hasButton(generator, T("osc.startGen")));
  expect("the generator stops", true);

  // Wait for this: the message becomes a Wait for OSC in the experiment.
  const waitButton = [...monitor.querySelectorAll("tbody tr")].find((tr) => textOf(tr).includes("/e2e/osc"))?.querySelector<HTMLButtonElement>(`button[aria-label="${T("osc.waitForThis")}"]`);
  if (!waitButton) throw new Error("no Wait for this on the message");
  await click(waitButton, "Wait for this");
  const editor = await until("the experiment editor", () => !document.querySelector<HTMLElement>(".experiment-host")!.hidden && document.querySelector(".experiment-properties h2"));
  expect("Wait for this adds a Wait for OSC", textOf(editor) === T("exp.node.wait_osc"), textOf(editor));
  const properties = document.querySelector(".experiment-properties")!;
  expect("…on the monitor's port, for this address", control(properties, T("exp.listenOn")).value === `127.0.0.1:${args.port}` && control(properties, T("exp.addressPattern")).value === "/e2e/osc");
  expect("…with the string argument as a rule", [...properties.querySelectorAll<HTMLInputElement>(".experiment-rule input")].some((input) => input.value === "hello e2e"));
  await click(button(properties, T("exp.delete")));
  await go("osc");
  // The monitor keeps running for the Signals step.
  expect("the monitor survived the trip", hasButton(panel(screen(), T("osc.monitor")), T("common.stop")) !== null);
}

async function signals(expect: Expect, args: StepArgs) {
  const shown = await go("signals");
  const count = () => numberIn(textOf([...shown.querySelectorAll(".sig-foot span")].find((element) => /\d/.test(textOf(element)))));
  await until("the library", () => count() > 0);
  const before = count();
  expect("the starter library loads", before > 0, String(before));
  await click(button(shown, T("sig.new")));
  await until("the new signal", () => (shown.querySelector<HTMLInputElement>("#sig-name")?.value ?? "") === T("sig.newName"));
  await type(control(shown, T("sig.name")), "E2E signal");
  await type(control(shown, T("common.target")), `127.0.0.1:${args.port}`);
  await type(control(shown, T("common.address")), "/e2e/signal");
  await until("the library to save", () => textOf(shown.querySelector(".sig-foot")).includes(T("sig.saved")), 5000);
  expect("a new signal is saved to the library file", count() === before + 1);
  await click(button(shown, T("sig.fire")));
  await until("the signal to be sent", () => textOf(shown).includes(T("sig.lastFired", { at: "" }).trim()));
  expect("Send marks when it was sent", true);

  // Ctrl+K from anywhere: the palette finds it.
  key(window, { key: "k", code: "KeyK", ctrlKey: true });
  const palette = await until("the palette", () => document.querySelector<HTMLElement>(".palette"));
  await type(palette.querySelector("input")!, "E2E signal");
  expect("the palette finds the signal", textOf(palette).includes("E2E signal"));
  key(palette.querySelector("input")!, { key: "Escape", code: "Escape" });
  await until("the palette to close", () => !document.querySelector(".palette"));

  await click(button(shown, T("sig.delete")));
  await click(button(shown, T("sig.confirmDelete")));
  await until("the signal to go", () => count() === before);
  expect("Delete asks once more, then removes it", count() === before);
}

async function oscStop(expect: Expect) {
  const shown = await go("osc");
  const monitor = panel(shown, T("osc.monitor"));
  await until("the signal in the monitor", () => [...monitor.querySelectorAll("tbody tr")].some((tr) => textOf(tr).includes("/e2e/signal")));
  expect("the library signal reached the monitor", true);
  await click(button(monitor, T("common.stop")));
  await until("the monitor to stop", () => hasButton(monitor, T("osc.listen")));
  expect("the monitor stops", true);
}

async function mqtt(expect: Expect, args: StepArgs) {
  const shown = await go("mqtt");
  await type(control(shown, T("mq.host")), "127.0.0.1");
  await type(control(shown, T("common.port")), args.port);
  await click(button(shown, T("mq.connect")));
  await until("the connection", () => hasButton(shown, T("mq.disconnect")));
  await until("the # subscription", () => textOf(shown).includes(T("mq.subscriptions")) && textOf(shown).includes("qos0"));
  expect("connects and subscribes to #", true);

  const publisher = panel(shown, T("mq.publish"));
  await type(control(publisher, T("mq.topic")), "e2e/lights/hall");
  await type(control(publisher, T("sig.payload")), "on");
  await setChecked(checkbox(publisher, T("mq.retain")), true);
  await click(button(publisher, T("mq.publishBtn")));
  await type(control(shown, T("mq.topics")), "hall");
  const found = await until("the topic in the tree", () => [...shown.querySelectorAll<HTMLButtonElement>(".topic-row")].find((row) => textOf(row).includes("e2e/lights/hall")));
  expect("the published topic comes back with its value", textOf(found).includes("on"), textOf(found));
  await click(found, "the topic");
  const detail = await until("the topic's panel", () => [...shown.querySelectorAll(".panel")].find((element) => textOf(element.querySelector(".section-label")) === "e2e/lights/hall"));

  // A new subscription gets the broker's retained value, marked as retained.
  await type(control(shown, T("mq.addSubscription")), "e2e/+/hall");
  await click(button(shown, T("mq.subscribe")));
  await until("the second subscription", () => textOf(shown).includes("e2e/+/hall"));
  await until("the retained replay", () => textOf(detail).includes(`${T("mq.retain")} ${T("mq.yes")}`));
  expect("a new subscription receives the retained value, marked R", [...shown.querySelectorAll(".topic-row")].some((row) => textOf(row).includes("e2e/lights/hall") && textOf(row).includes("R")));
  const drop = [...shown.querySelectorAll(".row.tight")].find((row) => textOf(row).includes("e2e/+/hall"))?.querySelector<HTMLButtonElement>("button");
  if (!drop) throw new Error("no drop button on the subscription");
  await click(drop, "drop the subscription");
  await until("the subscription to go", () => !textOf(shown).includes("e2e/+/hall"));
  expect("subscribe and unsubscribe", true);

  await click(buttonWith(detail, T("mq.waitForThis")), "Wait for this");
  const editor = await until("the experiment editor", () => !document.querySelector<HTMLElement>(".experiment-host")!.hidden && document.querySelector(".experiment-properties h2"));
  expect("Wait for this adds a Wait for MQTT", textOf(editor) === T("exp.node.wait_mqtt"), textOf(editor));
  const properties = document.querySelector(".experiment-properties")!;
  expect("…on this topic", control(properties, T("exp.topicFilter")).value === "e2e/lights/hall");
  await click(button(properties, T("exp.delete")));

  const back = await go("mqtt");
  const again = [...back.querySelectorAll(".panel")].find((element) => textOf(element.querySelector(".section-label")) === "e2e/lights/hall")!;
  await click(button(again, T("mq.clearRetained")));
  await click(button(again, T("mq.clearConfirm")));
  await sleep(300);
  await click(button(back, T("mq.disconnect")));
  await until("the disconnect", () => hasButton(back, T("mq.connect")));
  expect("disconnects", true);
}

async function broadcast(expect: Expect, args: StepArgs) {
  const shown = await go("broadcast");
  await click(button(shown, T("bc.mode.list")));
  await type(control(shown, T("bc.targetList")), `127.0.0.1:${args.port}`);
  const discovery = panel(shown, T("bc.discovery"));
  await type(control(discovery, T("common.bind")), `127.0.0.1:${args.port}`);
  await type(control(discovery, T("bc.joinGroups")), "");
  await setChecked(checkbox(discovery, T("bc.respond")), true);
  await click(button(discovery, T("bc.startListen")));
  await until("discovery to listen", () => hasButton(discovery, T("bc.stopListen")));

  await click(button(shown, T("bc.sendOnce")));
  await until("the emit result", () => shown.querySelector(".metrics"));
  expect("Send once reaches one target without errors", metric(shown, T("bc.targets")) === "1" && metric(shown, T("common.errors")) === "0",
    `${metric(shown, T("bc.targets"))} targets, ${metric(shown, T("common.errors"))} errors`);
  const peer = await until("the probe in discovery", () => [...discovery.querySelectorAll("tbody tr")].find((tr) => textOf(tr).includes("127.0.0.1:")));
  expect("discovery sees the probe as OSC", textOf(peer).includes("osc"), textOf(peer));
  await until("the answer", () => Array.from({ length: 50 }, (_, k) => T("bc.peersReplies", { n: k + 1 })).some((text) => textOf(discovery).includes(text)));
  expect("…and answers it", true);

  await type(control(shown, T("bc.beaconRate")), 10);
  await click(button(shown, T("bc.startBeacon")));
  await until("beacon rounds", () => numberIn(metric(shown, T("bc.rounds"))) >= 3, 6000);
  expect("the beacon repeats", true);
  await click(button(shown, T("bc.stopBeacon")));
  await click(button(discovery, T("bc.stopListen")));
  await until("both to stop", () => hasButton(shown, T("bc.startBeacon")) && hasButton(discovery, T("bc.startListen")));
}

async function netsimStart(expect: Expect, args: StepArgs) {
  const shown = await go("netsim");
  await type(control(shown, T("ns.listen")), `127.0.0.1:${args.relay}`);
  await type(control(shown, T("ns.target")), `127.0.0.1:${args.sink}`);
  for (const [label, value] of [["ns.latency", 10], ["ns.jitter", 0], ["ns.loss", 0], ["ns.duplicate", 0], ["ns.corrupt", 0]] as [Key, number][]) {
    await type(control(shown, T(label)), value);
  }
  await click(button(shown, T("ns.startRelay")));
  await until("the relay", () => hasButton(shown, T("ns.stopRelay")));
  expect("the relay starts", true);
}

async function netsimCheck(expect: Expect, args: StepArgs) {
  const shown = screen();
  await until(`${args.n} forwarded`, () => numberIn(metric(shown, T("ns.forwarded"))) === args.n);
  expect(`forwards all ${args.n} datagrams with no loss configured`, metric(shown, T("ns.dropped")) === "0");
  await click(button(shown, T("ns.stopRelay")));
  await until("the relay to stop", () => hasButton(shown, T("ns.startRelay")));
  return { forwarded: numberIn(metric(shown, T("ns.forwarded"))) };
}

async function storm(expect: Expect, args: StepArgs) {
  const shown = await go("storm");
  const run = async (protocol: "udp" | "tcp", port: number) => {
    await type(control(shown, T("common.target")), `127.0.0.1:${port}`);
    await type(control(shown, T("common.protocol")), protocol);
    await type(control(shown, T("st.payloadSize")), 64);
    await type(control(shown, T("st.rate")), 200);
    await type(control(shown, T("st.duration")), 1);
    await click(button(shown, T("st.launch")));
    await until("the storm to start", () => hasButton(shown, T("st.stop")));
    await until("the storm to end by itself", () => hasButton(shown, T("st.launch")), 8000);
    return { packets: numberIn(metric(shown, T("common.packets"))), errors: numberIn(metric(shown, T("common.errors"))) };
  };
  const udp = await run("udp", args.sink);
  expect("a 1 s UDP storm at 200 pps sends about 200", udp.packets >= 150 && udp.packets <= 260 && udp.errors === 0, JSON.stringify(udp));
  const tcp = await run("tcp", args.tcp);
  expect("a TCP storm sends without errors", tcp.packets > 0 && tcp.errors === 0, JSON.stringify(tcp));
  return { udp: udp.packets, tcp: tcp.packets };
}

async function scan(expect: Expect, args: StepArgs) {
  const shown = await go("scan");
  await type(control(shown, T("sc.host")), "127.0.0.1");
  await type(control(shown, T("sc.fromPort")), args.port - 2);
  await type(control(shown, T("sc.toPort")), args.port + 2);
  await type(control(shown, T("common.timeoutMs")), 300);
  await click(button(shown, T("sc.startScan")));
  await until("the scan to finish", () => textOf(shown).includes("5 / 5 (100%)") && hasButton(shown, T("sc.startScan")), 10000);
  const open = [...shown.querySelectorAll("tbody tr")].map((tr) => textOf(tr.querySelector("td")));
  expect(`finds the open port ${args.port}`, open.includes(String(args.port)), open.join(", "));
}

async function http(expect: Expect, args: StepArgs) {
  const shown = await go("http");
  const request = panel(shown, T("http.request"));
  await type(request.querySelector("select")!, "GET");
  await type(control(request, "URL"), `http://127.0.0.1:${args.port}/e2e?from=tour`);
  await click(button(request, T("common.send")));
  const verdict = await until("the response", () => result(request)?.classList.contains("ok") && result(request));
  expect("GET answers 200", textOf(verdict).startsWith("200"), textOf(verdict));
  const response = panel(shown, T("http.response"));
  const body = response.querySelector<HTMLTextAreaElement>("textarea")!;
  expect("the JSON body is shown formatted", body.value.includes('"ok": true') && body.value.includes('"path": "/e2e?from=tour"'), body.value.slice(0, 120));
  await click(buttonWith(response, T("http.responseHeaders", { n: 5 }).replace("5", "").trim()));
  expect("response headers open", textOf(response).includes("x-fixture"));
  await click(button(response, T("http.rawBody")));
  expect("raw shows the body as it came", !body.value.includes('\n  "ok"'));
  await click(button(response, T("http.formatJson")));

  await type(request.querySelector("select")!, "POST");
  await type(control(request, T("http.body")), '{"e2e":1}');
  await click(button(request, T("common.send")));
  await until("the POST response", () => body.value.includes('"method": "POST"'));
  expect("POST sends its body", body.value.includes('\\"e2e\\":1') || body.value.includes('{\\"e2e\\":1}'), body.value.slice(0, 200));
  await type(request.querySelector("select")!, "GET");

  const burst = panel(shown, T("http.burst"));
  await type(control(burst, T("common.concurrency")), 4);
  await type(control(burst, T("http.total")), 40);
  await type(control(burst, T("http.duration")), 0);
  await click(button(burst, T("http.startBurst")));
  await until("the burst to finish", () => numberIn(metric(burst, T("http.sent"))) === 40 && hasButton(burst, T("http.startBurst")), 15000);
  expect("a burst of 40 all succeed", metric(burst, T("http.ok")) === "40" && metric(burst, T("http.failed")) === "0",
    `${metric(burst, T("http.ok"))} ok, ${metric(burst, T("http.failed"))} failed`);

  await click(button(request, T("common.toExperiment")));
  const editor = await until("the experiment editor", () => !document.querySelector<HTMLElement>(".experiment-host")!.hidden && document.querySelector(".experiment-properties h2"));
  expect("To experiment adds the request as a node", textOf(editor) === T("exp.node.http"));
  await click(button(document.querySelector(".experiment-properties")!, T("exp.delete")));
}

/** A drag of `what` onto `where`, as the browser sends it (dragstart … drop). */
function dragOnto(what: Element, where: Element) {
  const data = new DataTransfer();
  what.dispatchEvent(new DragEvent("dragstart", { bubbles: true, cancelable: true, dataTransfer: data }));
  where.dispatchEvent(new DragEvent("dragover", { bubbles: true, cancelable: true, dataTransfer: data }));
  where.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: data }));
  what.dispatchEvent(new DragEvent("dragend", { bubbles: true, cancelable: true, dataTransfer: data }));
}

/** Save a request from HTTP into a folder, keep it saved, and work the folders in Signals. */
async function library(expect: Expect, args: StepArgs) {
  const http = await go("http");
  const request = panel(http, T("http.request"));
  const url = `http://127.0.0.1:${args.port}/e2e/library`;
  await type(control(request, "URL"), url);
  await click(button(request, T("sig.saveNew")));
  const dialog = await until("the save dialog", () => document.querySelector<HTMLDialogElement>("dialog.save-signal-dialog[open]"));
  await type(control(dialog, T("sig.name")), "E2E check");
  await type(control(dialog, T("sig.group")), "E2E / API");
  await click(button(dialog, T("sig.saveConfirm")));
  const chip = await until("the saved chip", () => request.querySelector(".save-signal-chip"));
  expect("the request is saved where it was told", textOf(chip).includes("E2E / API / E2E check"), textOf(chip));
  expect("…and says so", !!hasButton(request, `✓ ${T("sig.savedState")}`));

  await type(control(request, "URL"), `${url}?v=2`);
  await until("a changed request", () => textOf(request.querySelector(".save-signal-chip")).includes(T("sig.changed")));
  key(control(request, "URL"), { key: "s", code: "KeyS", ctrlKey: true });
  await until("saved again", () => hasButton(request, `✓ ${T("sig.savedState")}`));
  expect("Ctrl+S updates the saved request", !textOf(request.querySelector(".save-signal-chip")).includes(T("sig.changed")));

  await click(request.querySelector<HTMLButtonElement>(".save-signal-chip")!, "the saved chip");
  const signals = await until("Signals", () => { const shown = screen(); return textOf(shown.querySelector("h1")) === T("sig.title") && shown; });
  await until("the signal selected", () => (signals.querySelector<HTMLInputElement>("#sig-name")?.value ?? "") === "E2E check");
  const folderRow = (path: string) => signals.querySelector<HTMLButtonElement>(`button.sig-folder[data-folder="${CSS.escape(path)}"]`);
  expect("the chip opens it in Signals, its folders open", !!folderRow("E2E") && !!folderRow("E2E/API") && folderRow("E2E/API")!.getAttribute("aria-expanded") === "true");
  expect("its folder field shows the path", signals.querySelector<HTMLInputElement>("#sig-group")?.value === "E2E/API");

  await click(buttonWith(signals, T("sig.newFolder")));
  const rename = await until("renaming the new folder", () => signals.querySelector<HTMLInputElement>("input.sig-folder-rename"));
  await type(rename, "Archive");
  key(rename, { key: "Enter", code: "Enter" });
  const archive = await until("the folder", () => folderRow("E2E/API/Archive"));
  expect("a new folder goes inside the current one and is named at once", true);

  const item = [...signals.querySelectorAll<HTMLElement>(".sig-item")].find((element) => textOf(element).includes("E2E check"))!;
  dragOnto(item, archive.closest(".sig-folder-row")!);
  await until("the signal moved", () => signals.querySelector<HTMLInputElement>("#sig-group")?.value === "E2E/API/Archive");
  expect("dragging a signal onto a folder files it there", true);

  folderRow("E2E/API/Archive")!.focus();
  key(folderRow("E2E/API/Archive")!, { key: "F2", code: "F2" });
  const second = await until("renaming", () => signals.querySelector<HTMLInputElement>("input.sig-folder-rename"));
  await type(second, "Old");
  key(second, { key: "Enter", code: "Enter" });
  await until("the renamed folder", () => folderRow("E2E/API/Old") && signals.querySelector<HTMLInputElement>("#sig-group")?.value === "E2E/API/Old");
  expect("F2 renames a folder and its signals follow", true);

  await click(folderRow("E2E")!, "the E2E folder");
  await until("E2E closed", () => folderRow("E2E")!.getAttribute("aria-expanded") === "false" && !folderRow("E2E/API"));
  expect("a folder closes, and stays closed", true);
  await click(folderRow("E2E")!, "the E2E folder");
  await until("E2E open", () => folderRow("E2E/API"));

  await click(buttonWith(signals, T("sig.openIn", { screen: T("nav.http") })));
  const back = await until("HTTP", () => { const shown = screen(); return textOf(shown.querySelector("h1")) === T("http.title") && shown; });
  expect("Open in HTTP loads the saved request", control(panel(back, T("http.request")), "URL").value === `${url}?v=2`);

  const again = await go("signals");
  const remove = button(again, `${T("sig.removeFolder")}: E2E`);
  await click(remove);
  await click(button(again, `${T("sig.removeFolder")}: E2E`));
  await until("E2E gone", () => !again.querySelector(`button.sig-folder[data-folder="E2E"]`));
  expect("removing a folder keeps what was in it, a level up", !!again.querySelector(`button.sig-folder[data-folder="API/Old"]`));
  const leftover = [...again.querySelectorAll<HTMLElement>(".sig-item")].find((element) => textOf(element).includes("E2E check"))!;
  await click(leftover);
  await click(button(again, T("sig.delete")));
  await click(button(again, T("sig.confirmDelete")));
  await until("the signal gone", () => ![...again.querySelectorAll(".sig-item")].some((element) => textOf(element).includes("E2E check")));
  const nameless = unnamed(again);
  expect("signals: every field and button has a name", nameless.length === 0, nameless.join(" | "));
  return { signalsFile: true };
}

/** Open one of the bundled templates through the Experiments dialog. */
async function openTemplate(title: Key) {
  const editor = await go("experiment");
  await click(button(editor, T("exp.documents")));
  const dialog = await until("the Experiments dialog", () => document.querySelector<HTMLDialogElement>("dialog.experiment-documents[open]"));
  await click(buttonWith(dialog, T(title)));
  await click(button(dialog, T("exp.openDocument")));
  await until("the template to open", () => !document.querySelector("dialog.experiment-documents[open]"));
  await until("its name", () => (editor.querySelector<HTMLInputElement>(".experiment-heading input")?.value ?? "") === T(title));
  return editor;
}

async function selectNode(editor: HTMLElement, type: Key, index = 0, expect?: Expect) {
  const bodies = [...editor.querySelectorAll<HTMLButtonElement>(".experiment-node-body")].filter((element) => textOf(element.querySelector(".experiment-node-type")) === T(type));
  if (!bodies[index]) throw new Error(`no ${T(type)} node on the canvas`);
  await click(bodies[index], T(type));
  await until(`${T(type)} in the properties`, () => textOf(editor.querySelector(".experiment-properties h2")) === T(type));
  const properties = editor.querySelector<HTMLElement>(".experiment-properties")!;
  if (expect) {
    const nameless = unnamed(properties);
    expect(`${T(type)}: every field and button has a name`, nameless.length === 0, nameless.join(" | "));
  }
  return properties;
}

async function runAndWait(editor: HTMLElement, timeout = 15000) {
  await click(button(editor, T("exp.run")));
  const outcome = await until("the run to end", () => {
    const text = textOf(editor.querySelector(".experiment-timeline-title strong"));
    return (text === T("exp.passed") || text.startsWith(T("exp.failed"))) && hasButton(editor, T("exp.run")) && text;
  }, timeout);
  const rows = [...editor.querySelectorAll(".experiment-events button")].map(textOf);
  return { outcome, rows };
}

async function experimentHttp(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateHttp");
  expect("HTTP check has 4 nodes", editor.querySelectorAll(".experiment-node").length === 4);
  const properties = await selectNode(editor, "exp.node.http", 0, expect);
  await type(control(properties, "URL"), `http://127.0.0.1:${args.port}/e2e/experiment`);
  await click(buttonWith(properties, T("exp.sendNow")));
  const tested = await until("Send now", () => properties.querySelector(".experiment-test-result.ok"));
  expect("Send now performs the request", textOf(tested).includes("HTTP 200"), textOf(tested));

  const { outcome, rows } = await runAndWait(editor);
  expect("the run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("the timeline lists every step", [T("exp.node.start"), T("exp.node.http"), T("exp.node.assert_status"), T("exp.node.end")].every((name) => rows.some((row) => row.includes(name))), rows.join(" | "));
  expect("a report is written", textOf(editor.querySelector(".experiment-timeline-title")).includes(T("exp.reportSaved")));

  // A: the add menu; Enter adds the highlighted match; Undo takes it back.
  const nodes = () => editor.querySelectorAll(".experiment-node").length;
  const before = nodes();
  let nextPress = 0;
  editor.querySelector<HTMLButtonElement>(".experiment-node-body")!.focus();
  // Right after a run a press can land before the editor is ready again (seen in WebView2 in the
  // background); a person presses again, so the tour does too — at most three times.
  let presses = 0;
  const search = await until("the add menu", () => {
    const menu = document.querySelector<HTMLInputElement>(".experiment-add-menu input");
    if (!menu && presses < 3 && performance.now() >= nextPress) {
      key(editor.querySelector(".experiment-node-body")!, { key: "a", code: "KeyA" });
      presses++;
      nextPress = performance.now() + 1500;
    }
    return menu;
  });
  expect("A opens the add menu", presses <= 2, `${presses} presses`);
  await type(search, T("exp.node.delay"));
  key(search, { key: "Enter", code: "Enter" });
  await until("the new node", () => nodes() === before + 1);
  expect("A, type, Enter adds a node", textOf(editor.querySelector(".experiment-properties h2")) === T("exp.node.delay"));
  await click(button(editor, T("exp.undo")));
  await until("undo", () => nodes() === before);
  expect("Undo removes it", true);
}

async function experimentOsc(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templatePingReply");
  await click(button(editor, T("exp.params")));
  const params = await until("the parameters", () => document.querySelector<HTMLElement>(".experiment-params"));
  await type(control(params, T("exp.paramValue")), `127.0.0.1:${args.device}`);
  await click(button(params, T("exp.close")));
  await selectNode(editor, "exp.node.osc", 0, expect);
  const properties = await selectNode(editor, "exp.node.wait_osc", 0, expect);
  await type(control(properties, T("exp.listenOn")), `127.0.0.1:${args.pong}`);
  const { outcome, rows } = await runAndWait(editor);
  expect("the ping-reply run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("the device's /pong is matched and logged", rows.some((row) => row.includes(`/pong from 127.0.0.1:${args.device}`)), rows.join(" | "));
  // The matched message's frame, from the timeline into the Inspector.
  const link = await until("the frame link", () => [...editor.querySelectorAll<HTMLButtonElement>(".experiment-frame-links button")].find((element) => textOf(element).includes(T("exp.node.wait_osc"))));
  const seq = numberIn(textOf(link).split("#").pop() ?? "");
  await click(link, "the frame link");
  const inspector = await until("the Inspector", () => document.querySelector<HTMLElement>(`[data-frame="${seq}"].picked`));
  expect(`the frame link selects frame #${seq} in the Inspector`, textOf(inspector).includes("/pong"), textOf(inspector));
}

/** Repeat through the properties: the ping is sent three times, each one numbered. */
async function experimentRepeat(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templatePingReply");
  await click(button(editor, T("exp.params")));
  const params = await until("the parameters", () => document.querySelector<HTMLElement>(".experiment-params"));
  await type(control(params, T("exp.paramValue")), `127.0.0.1:${args.device}`);
  await click(button(params, T("exp.close")));
  const wait = await selectNode(editor, "exp.node.wait_osc");
  await type(control(wait, T("exp.listenOn")), `127.0.0.1:${args.pong}`);
  const ping = await selectNode(editor, "exp.node.osc");
  await setChecked(checkbox(ping, T("exp.repeatOn")), true);
  await type(control(ping, T("exp.repeatCount")), 3);
  await type(control(ping, T("exp.repeatInterval")), 50);
  const nameless = unnamed(ping);
  expect("the repeat fields have names", nameless.length === 0, nameless.join(" | "));
  const badge = await until("the repeat badge", () => editor.querySelector(".experiment-node-repeat"));
  expect("the node shows ×3", textOf(badge) === "×3", textOf(badge));
  const { outcome, rows } = await runAndWait(editor);
  expect("the run with a repeated ping passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("the timeline says how many were sent", rows.some((row) => row.includes(T("exp.node.osc")) && /Sends: 3 in \d+ ms/.test(row)), rows.join(" | "));
}

/** The bundled loop: a device that is busy twice, then ready. */
async function experimentLoop(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templatePoll");
  expect("the wire back is drawn under the body", !!editor.querySelector(".experiment-wires path.back"));
  await click(button(editor, T("exp.params")));
  const params = await until("the parameters", () => document.querySelector<HTMLElement>(".experiment-params"));
  await type(control(params, T("exp.paramValue")), `127.0.0.1:${args.device}`);
  await click(button(params, T("exp.close")));
  await selectNode(editor, "exp.node.loop", 0, expect);
  const { outcome, rows } = await runAndWait(editor, 20000);
  expect("the poll run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  const iterations = rows.filter((row) => row.includes(T("exp.node.loop")) && row.includes("Iteration"));
  expect("three iterations", iterations.length === 3 && iterations[2].includes(T("exp.step.loopIteration", { n: 3, max: 10 })), iterations.join(" | "));
  expect("the loop leaves when the device is ready", rows.some((row) => row.includes(T("exp.step.loopDone", { n: 3 }))), rows.join(" | "));
  expect("what follows Done reads the last answer", rows.some((row) => row.includes(`127.0.0.1:${args.device} is ready`)), rows.join(" | "));
}

async function experimentParallel(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateParallel");
  const properties = await selectNode(editor, "exp.node.http");
  await type(control(properties, "URL"), `http://127.0.0.1:${args.port}/e2e/parallel`);
  const { outcome, rows } = await runAndWait(editor);
  expect("both branches run and the run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("End passes once", rows.filter((row) => row.includes(T("exp.node.end")) && row.includes(T("exp.passed"))).length === 1, rows.join(" | "));
}

async function experimentExport(expect: Expect, args: StepArgs) {
  const editor = await go("experiment");
  await click(button(editor, T("exp.documents")));
  const dialog = await until("the Experiments dialog", () => document.querySelector<HTMLDialogElement>("dialog.experiment-documents[open]"));
  await click(button(dialog, T("exp.exportJson")));
  const path = await until("the export path", () => dialog.querySelector<HTMLInputElement>(".experiment-export-path input")?.value);
  expect("Export writes a file in the data folder", path.includes(args.dataDir) && path.endsWith(".json"), path);
  const download = dialog.querySelector<HTMLAnchorElement>(".experiment-export-path a[download]");
  if (args.mode === "server") {
    if (!download) throw new Error("no Download link in a browser");
    await click(download, "Download");
    const href = downloads.pop()!;
    const response = await fetch(href, { credentials: "same-origin" });
    const text = await response.text();
    expect("…which the browser can download", response.ok && text.includes('"nodes"'), `${response.status} ${text.slice(0, 80)}`);
  } else {
    expect("no download link in the desktop app (the file is already here)", !download);
  }
  await click(button(dialog, T("exp.close")));
  await until("the dialog to close", () => !document.querySelector("dialog.experiment-documents[open]"));
}

async function inspectCheck(expect: Expect, args: StepArgs) {
  const shown = await openInspector();
  const captured = numberIn(textOf([...shown.querySelectorAll(".tag-chip")][0]));
  // High-rate sources (the storm, a burst) are sampled by design, so this is a floor, not a count.
  expect("the capture holds the tour's traffic", captured >= 20, String(captured));
  const protocols = new Set([...shown.querySelectorAll("tbody tr[data-frame] td:nth-child(3)")].map(textOf));
  expect("OSC, UDP, TCP, HTTP and MQTT frames are all there", ["osc", "udp", "tcp", "http", "mqtt"].every((proto) => protocols.has(proto)), [...protocols].join(", "));
  await click(button(shown, "osc"));
  const rows = [...shown.querySelectorAll("tbody tr[data-frame]")];
  expect("the osc chip leaves only OSC", rows.length > 0 && rows.every((tr) => textOf(tr.querySelector("td:nth-child(3)")) === "osc"));
  const numbers = rows.map((tr) => tr.getAttribute("data-frame"));
  const showing = textOf([...shown.querySelectorAll(".capture-bar span")].find((element) => textOf(element).startsWith("showing")));
  expect("every frame is listed once, as many as the count says", new Set(numbers).size === numbers.length && showing.startsWith(`showing ${rows.length} `), `${rows.length} rows (${new Set(numbers).size} distinct) · ${showing}`);
  await type(shown.querySelector<HTMLInputElement>(`input[placeholder="${T("ins.filterPlaceholder")}"]`)!, "/e2e/osc");
  const row = await until("a /e2e/osc frame", () => shown.querySelector<HTMLElement>("tbody tr[data-frame]"));
  await click(row, "the frame");
  const detail = await until("the frame detail", () => panel(shown, T("ins.detail")).querySelector("pre.hex"));
  expect("the frame decodes its arguments", textOf(panel(shown, T("ins.detail"))).includes("hello e2e"), textOf(detail).slice(0, 80));
  await click(button(shown, T("sig.fromFrame")));
  await until("the log line", () => logLines().some((line) => line.includes("saved as «/e2e/osc")));
  expect("Save as signal keeps the frame in the library", true);
  await click(button(shown, T("common.reset")));

  await click(button(shown, T("ins.exportJsonl")));
  const saved = await until("the export", () => logLines().reverse().find((line) => line.startsWith("saved ") && line.endsWith(".jsonl")));
  expect("Export writes the capture to the data folder", saved.includes(args.dataDir), saved);
  if (args.mode === "server") {
    const href = await until("the download", () => downloads.find((link) => link.includes("/api/files")));
    const response = await fetch(href, { credentials: "same-origin" });
    const text = await response.text();
    expect("…and a browser downloads it", response.ok && text.includes("/e2e/osc"), `${response.status} ${text.slice(0, 80)}`);
  }
  await click(button(shown, T("ins.disarm")));
  await until("disarmed", () => hasButton(shown, T("ins.arm")));
}

/** Every screen in Russian: headings from the Russian dictionary and no English text left on screen. */
async function russian(expect: Expect) {
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

/** A pointer drag of a pane handle by `dy` / `dx` screen pixels. */
function drag(handle: Element, dx: number, dy: number) {
  const rect = handle.getBoundingClientRect();
  const at = { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
  const fire = (kind: string, x: number, y: number) => handle.dispatchEvent(new PointerEvent(kind, { bubbles: true, cancelable: true, pointerId: 7, pointerType: "mouse", button: 0, buttons: kind === "pointerup" ? 0 : 1, clientX: x, clientY: y }));
  fire("pointerdown", at.x, at.y);
  fire("pointermove", at.x + dx / 2, at.y + dy / 2);
  fire("pointermove", at.x + dx, at.y + dy);
  fire("pointerup", at.x + dx, at.y + dy);
}

/** Wires picked and removed on the canvas, and every pane handle, by pointer and by keyboard. */
async function layout(expect: Expect) {
  const editor = await go("experiment");
  const wires = () => [...editor.querySelectorAll<SVGPathElement>("path.experiment-wire-hit")];
  const count = wires().length;
  const wire = wires().find((path) => path.getAttribute("aria-label") === `${T("exp.wire")}: ${T("exp.node.start")} → ${T("exp.node.fork")}`);
  if (!wire) throw new Error(`no wire Start → Parallel branch (${wires().map((path) => path.getAttribute("aria-label")).join(" | ")})`);
  // A click on the line picks it (in a background window a script's focus() fires no focus event).
  expect("a wire is reachable with Tab", wire.getAttribute("tabindex") === "0" && wire.getAttribute("role") === "button");
  wire.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await until("the wire selected", () => wire.getAttribute("aria-pressed") === "true");
  const properties = editor.querySelector<HTMLElement>(".experiment-properties")!;
  expect("a picked wire is described in the properties", textOf(properties.querySelector("h2")) === T("exp.wire") && textOf(properties).includes(`${T("exp.node.start")} → ${T("exp.node.fork")}`), textOf(properties));
  expect("…and drawn as selected", !!editor.querySelector("path.wire.selected"));
  key(wire, { key: "Delete", code: "Delete" });
  await until("the wire removed", () => wires().length === count - 1);
  expect("Delete removes the picked wire, and only it", editor.querySelectorAll(".experiment-node").length > 0);
  await click(button(editor, T("exp.undo")));
  await until("undo", () => wires().length === count);
  expect("Undo brings it back", true);

  // Hovering a wire's ＋ shows its ×, which removes that wire without picking it.
  const add = button(editor, `${T("exp.insertNode")}: ${T("exp.node.fork")} → ${T("exp.node.log")}`);
  add.dispatchEvent(new PointerEvent("pointerover", { bubbles: true, pointerType: "mouse" }));
  const remove = await until("the × of the hovered wire", () => hasButton(editor, `${T("exp.removeWire")}: ${T("exp.node.fork")} · ${T("exp.branch2")} → ${T("exp.node.log")}`));
  await click(remove, "×");
  await until("the wire removed", () => wires().length === count - 1);
  expect("the × on a hovered wire removes it", true);
  await click(button(editor, T("exp.undo")));
  await until("undo", () => wires().length === count);

  // Pane handles: the console, the properties and the timeline.
  const toggle = document.querySelector<HTMLButtonElement>(".console-toggle")!;
  if (toggle.getAttribute("aria-expanded") !== "true") await click(toggle, "console toggle");
  const consoleHandle = await until("the console's handle", () => document.querySelector<HTMLElement>(".console > .splitter"));
  const consolePane = document.querySelector<HTMLElement>(".console")!;
  const height = () => Math.round(consolePane.getBoundingClientRect().height);
  const before = height();
  consoleHandle.focus();
  key(consoleHandle, { key: "ArrowUp", code: "ArrowUp" });
  key(consoleHandle, { key: "ArrowUp", code: "ArrowUp" });
  await until("a taller console", () => height() === before + 32);
  expect("the console grows by the arrow keys", consoleHandle.getAttribute("aria-valuenow") === String(before + 32), `${before} → ${height()}`);
  drag(consoleHandle, 0, -100);
  await until("a dragged console", () => height() === before + 132);
  expect("…and by dragging its top edge", true);
  key(consoleHandle, { key: "Enter", code: "Enter" });
  await until("the default height", () => height() === 188);
  expect("Enter gives it its default height back", true);
  await click(toggle, "console toggle");

  const propertiesHandle = editor.querySelector<HTMLElement>(".experiment-workspace > .splitter")!;
  const width = () => Math.round(properties.getBoundingClientRect().width);
  const wide = width();
  drag(propertiesHandle, -120, 0);
  await until("wider properties", () => width() === wide + 120);
  expect("the properties pane widens by dragging its edge", true, `${wide} → ${width()}`);
  propertiesHandle.focus();
  key(propertiesHandle, { key: "Enter", code: "Enter" });
  await until("the default width", () => width() === 254);

  const timeline = editor.querySelector<HTMLElement>(".experiment-timeline")!;
  const timelineToggle = buttonWith(timeline, T("exp.timeline"));
  if (timelineToggle.getAttribute("aria-expanded") !== "true") await click(timelineToggle, "timeline toggle");
  const timelineHandle = await until("the timeline's handle", () => timeline.querySelector<HTMLElement>(":scope > .splitter"));
  const tall = Math.round(timeline.getBoundingClientRect().height);
  timelineHandle.focus();
  key(timelineHandle, { key: "ArrowDown", code: "ArrowDown" });
  await until("a shorter timeline", () => Math.round(timeline.getBoundingClientRect().height) === tall - 16);
  expect("the timeline resizes too", true);
  key(timelineHandle, { key: "Enter", code: "Enter" });
  const nameless = unnamed(editor);
  expect("the editor still names every control", nameless.length === 0, nameless.join(" | "));
}

/**
 * An HTTP emulator made on its screen: it answers the HTTP screen, Mock this
 * turns that answer into a route of its own, the exchange is listed with its
 * rule, a restart takes the new route, and it is stopped and deleted again.
 */
async function emulators(expect: Expect, args: StepArgs) {
  const shown = await go("emulators");
  const library = panel(shown, T("emu.library"));
  expect("the starter set is in the library", library.querySelectorAll(".emu-item").length >= 4, textOf(library));
  await click(button(library, `＋ ${T("emu.new.http")}`));
  const editor = await until("the new emulator", () => textOf(shown.querySelector(".emu-item.active")).includes(T("emu.newName.http")) && shown.querySelector<HTMLElement>(".emu-editor"));
  const local = `127.0.0.1:${args.port}`;
  await type(control(editor, T("emu.bind")), local);
  await type(control(editor, T("emu.path")), "/e2e/:id");
  await type(control(editor, T("emu.body")), '{"id":"{{request.params.id}}","via":"emulator"}');
  await sleep(600);
  expect("it checks itself while being written: nothing in the way", !shown.querySelector(".emu-problem"), textOf(shown.querySelector(".emu-problem")));
  await click(button(shown, T("emu.start")));
  await until("it to answer", () => textOf(shown.querySelector(".emu-state")) === T("emu.runningOn", { local }));
  expect("the console lists its job", textOf(document.querySelector(".jobs-strip")).includes(T("job.emulator", { name: T("emu.newName.http"), local })), textOf(document.querySelector(".jobs-strip")));
  const nameless = unnamed(shown);
  expect("emulators: every field and button has a name", nameless.length === 0, nameless.join(" | "));

  // The HTTP screen talks to it.
  const http = await go("http");
  const request = panel(http, T("http.request"));
  await type(request.querySelector("select")!, "GET");
  await type(control(request, "URL"), `http://${local}/e2e/7`);
  await click(button(request, T("common.send")));
  const verdict = await until("the emulator's answer", () => result(request)?.classList.contains("ok") && result(request));
  expect("the emulator answers 200", textOf(verdict).startsWith("200"), textOf(verdict));
  const response = panel(http, T("http.response"));
  const body = response.querySelector<HTMLTextAreaElement>("textarea")!;
  expect("with the id from its path", body.value.includes('"id": "7"') && body.value.includes('"via": "emulator"'), body.value.slice(0, 120));

  // Mock this: that answer becomes a route of the same emulator, first in its list.
  await click(buttonWith(response, T("http.mockThis")));
  const dialog = await until("the Mock this dialog", () => document.querySelector<HTMLDialogElement>("dialog[open]"));
  const into = control(dialog, T("http.mockInto")) as HTMLSelectElement;
  const option = [...into.options].find((item) => item.text.startsWith(T("emu.newName.http")));
  if (!option) throw new Error(`no ${T("emu.newName.http")} to mock into: ${[...into.options].map((item) => item.text).join(", ")}`);
  await type(into, option.value);
  await click(button(dialog, T("http.mockAdd")));
  const back = await until("the route on the Emulators screen", () => {
    const now = screen();
    return textOf(now.querySelector(".emu-rule .emu-rule-summary")) === "GET /e2e/7 → 200" && now;
  });
  expect("Mock this adds the route first", true);
  expect("the running emulator offers a restart for it", !!hasButton(back, T("emu.restart")));

  // What it received, as it arrived.
  const live = panel(back, T("emu.live"));
  const row = await until("the exchange", () => [...live.querySelectorAll("tbody tr")].map(textOf).find((text) => text.includes("GET /e2e/7")));
  expect("the request is listed with its rule and answer", row.includes("#1") && row.includes("200 OK"), row);
  expect("and counted", metric(live, T("emu.total")) === "1", metric(live, T("emu.total")));
  await click(button(back, T("emu.restart")));
  await until("the restart", () => !hasButton(back, T("emu.restart")) && textOf(back.querySelector(".emu-state")) === T("emu.runningOn", { local }));
  expect("a restart takes the rules as they are now", true);

  await click(button(back, T("emu.stop")));
  await until("it to stop", () => textOf(back.querySelector(".emu-state")) === T("emu.notRunning"));
  await click(button(back, T("emu.delete")));
  await click(button(back, T("emu.confirmDelete")));
  await until("it gone from the library", () => ![...back.querySelectorAll(".emu-item")].some((element) => textOf(element).includes(T("emu.newName.http"))));
  expect("stopped and deleted", true);
}

/** The bundled flaky API: an Emulator node edited in its dialog, and a run that retries until it answers. */
async function experimentEmulator(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateFlaky");
  const local = `127.0.0.1:${args.port}`;
  await click(button(editor, T("exp.params")));
  const params = await until("the parameters", () => document.querySelector<HTMLElement>(".experiment-params"));
  await type(control(params, T("exp.paramValue")), `http://${local}`);
  await click(button(params, T("exp.close")));
  const properties = await selectNode(editor, "exp.node.emulator", 0, expect);
  await click(button(properties, T("emu.edit")));
  const dialog = await until("the emulator dialog", () => document.querySelector<HTMLDialogElement>("dialog.emu-dialog[open]"));
  const nameless = unnamed(dialog);
  expect("the emulator dialog names every control", nameless.length === 0, nameless.join(" | "));
  await type(control(dialog, T("emu.bind")), local);
  await click(button(dialog, T("emu.done")));
  await until("the dialog to close", () => !document.querySelector("dialog.emu-dialog[open]"));
  expect("the node shows its address", textOf(properties).includes(local), textOf(properties));
  const { outcome, rows } = await runAndWait(editor, 20000);
  expect("the flaky API run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("the emulator served the run", rows.some((row) => row.includes(T("exp.step.emulating", { name: "Flaky API", local }))), rows.join(" | "));
  expect("answered on the third attempt", rows.some((row) => row.includes(`http://${local} answered on attempt 3`)), rows.join(" | "));
}

async function cleanup(expect: Expect) {
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
    firewallNotices += 1;
  }
  expect("the firewall notice, when it came, was answered", !document.querySelector(".firewall-banner"), firewallNotices ? `answered ${firewallNotices}×` : "it did not come");
}

const STEPS: Record<string, (expect: Expect, args: StepArgs) => Promise<Record<string, unknown> | void>> = {
  shell, inspectArm, osc, signals, oscStop, mqtt, broadcast, netsimStart, netsimCheck, storm, scan, http,
  library, emulators, experimentHttp, experimentOsc, experimentRepeat, experimentLoop, experimentParallel, experimentEmulator, experimentExport, layout,
  inspectCheck, russian, cleanup,
};

// ---- the runner's side --------------------------------------------------------------

interface Run { state: "running" | "done" | "failed"; checks: Check[]; data?: Record<string, unknown>; error?: string; ms: number }
const runs: Run[] = [];

declare global { interface Window { __signalLabTour?: unknown } }

window.__signalLabTour = {
  steps: Object.keys(STEPS),
  start(name: string, args: StepArgs = {}): number {
    const step = STEPS[name];
    const run: Run = { state: "running", checks: [], ms: 0 };
    runs.push(run);
    const started = performance.now();
    const expect: Expect = (check, ok, detail) => run.checks.push({ name: check, ok, detail: ok ? undefined : detail });
    if (!step) { run.state = "failed"; run.error = `no step “${name}”`; return runs.length - 1; }
    step(expect, args).then((data) => { run.data = data ?? undefined; run.state = "done"; })
      .catch((error: Error) => { run.state = "failed"; run.error = error?.message ?? String(error); closeOverlays(); })
      .finally(() => { run.ms = Math.round(performance.now() - started); });
    return runs.length - 1;
  },
  poll(id: number): Run | null { return runs[id] ?? null; },
  errors(): string[] { return pageErrors.splice(0); },
};
