/** The experiment editor: the bundled templates opened, edited and run, and an experiment exported. */
import { downloads } from "../page";
import { T, textOf, numberIn, until, control, checkbox, button, hasButton, buttonWith, unnamed, click, type, setChecked, key, go, type Key, type StepArgs, type Expect } from "../dsl";

/** Open one of the bundled templates through the Experiments dialog. */
export async function openTemplate(title: Key) {
  const editor = await go("experiment");
  await click(button(editor, T("exp.documents")));
  const dialog = await until("the Experiments dialog", () => document.querySelector<HTMLDialogElement>("dialog.experiment-documents[open]"));
  await click(buttonWith(dialog, T(title)));
  await click(button(dialog, T("exp.openDocument")));
  await until("the template to open", () => !document.querySelector("dialog.experiment-documents[open]"));
  await until("its name", () => (editor.querySelector<HTMLInputElement>(".experiment-heading input")?.value ?? "") === T(title));
  return editor;
}

export async function selectNode(editor: HTMLElement, type: Key, index = 0, expect?: Expect) {
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

export async function runAndWait(editor: HTMLElement, timeout = 15000) {
  // Every run writes a report of its own: an outcome beside the last run's report is the last run's,
  // still shown while this one validates and saves (a run again in the same editor).
  const report = () => editor.querySelector(".experiment-report")?.getAttribute("data-tip") ?? "";
  const before = report();
  await click(button(editor, T("exp.run")));
  const outcome = await until("the run to end", () => {
    const text = textOf(editor.querySelector(".experiment-timeline-title strong"));
    return (text === T("exp.passed") || text.startsWith(T("exp.failed"))) && hasButton(editor, T("exp.run")) && report() !== before && text;
  }, timeout);
  const rows = [...editor.querySelectorAll(".experiment-events button")].map(textOf);
  return { outcome, rows };
}

export async function experimentHttp(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateHttp");
  expect("HTTP check has 4 nodes", editor.querySelectorAll(".experiment-node").length === 4);
  const properties = await selectNode(editor, "exp.node.http", 0, expect);
  await type(control(properties, "URL"), `http://127.0.0.1:${args.port}/e2e/experiment`);
  await click(buttonWith(properties, T("exp.sendNow")));
  const tested = await until("Send now", () => properties.querySelector(".experiment-test-result.ok"));
  expect("Send now performs the request", textOf(tested).includes("HTTP 200"), textOf(tested));
  expect("a response that worked can be mocked", [...properties.querySelectorAll("button")].some((element) => textOf(element).includes(T("http.mockThis"))));

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

/** An HTTP node with Digest, its password a parameter, and the run's cookie setting. */
export async function experimentAuth(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateHttp");
  await click(button(editor, T("exp.params")));
  const params = await until("the parameters", () => document.querySelector<HTMLElement>(".experiment-params"));
  const cookies = params.querySelector<HTMLInputElement>("label.checkbox input[type=checkbox]")!;
  expect("an experiment keeps cookies unless told otherwise", cookies.checked);
  await click(button(params, T("exp.close")));
  const properties = await selectNode(editor, "exp.node.http", 0, expect);
  await type(control(properties, "URL"), `http://127.0.0.1:${args.port}/digest/run`);
  await type(control(properties, T("field.auth")), "digest");
  await type(control(properties, T("field.username")), "tour");
  await type(control(properties, T("field.password")), "e2e-secret");
  const nameless = unnamed(properties);
  expect("the node's authentication fields have names", nameless.length === 0, nameless.join(" | "));
  const { outcome, rows } = await runAndWait(editor);
  expect("the run with a Digest request passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
}

export async function experimentOsc(expect: Expect, args: StepArgs) {
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
export async function experimentRepeat(expect: Expect, args: StepArgs) {
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
export async function experimentLoop(expect: Expect, args: StepArgs) {
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

/**
 * Several nodes at once: Shift and a click, a frame drawn with Shift, Ctrl+A;
 * Copy and Paste through the clipboard (the page's own events: a key press, the
 * hidden field it goes through, the paste a person's Ctrl+V makes there),
 * Duplicate, a drag of the group, Delete, Escape — each undone, so the template
 * is as it was.
 */
export async function experimentSelection(expect: Expect) {
  const editor = await openTemplate("exp.templateHttp");
  const count = () => editor.querySelectorAll(".experiment-node").length;
  const wires = () => editor.querySelectorAll(".experiment-wires path.experiment-wire-hit").length;
  const chosen = () => [...editor.querySelectorAll(".experiment-node.selected .experiment-node-type")].map(textOf);
  const ctrl = (code: string) => key(document.activeElement ?? document.body, { key: code.slice(3).toLowerCase(), code, ctrlKey: true });
  const before = { nodes: count(), wires: wires() };
  const http = T("exp.node.http");

  await selectNode(editor, "exp.node.http");
  const next = [...editor.querySelectorAll<HTMLButtonElement>(".experiment-node-body")]
    .find((body) => body.dataset.nodeId && !chosen().includes(textOf(body.querySelector(".experiment-node-type"))) && ![T("exp.node.start"), T("exp.node.end")].includes(textOf(body.querySelector(".experiment-node-type"))))!;
  next.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, shiftKey: true }));
  await until("two nodes selected", () => chosen().length === 2);
  const properties = editor.querySelector<HTMLElement>(".experiment-properties")!;
  expect("Shift and a click select a second node", chosen().includes(http) && chosen().length === 2, chosen().join(", "));
  expect("…and the properties say how many", textOf(properties.querySelector("h2")) === T("exp.selectedCount", { n: 2 }), textOf(properties.querySelector("h2")));
  const nameless = unnamed(properties);
  expect("…with Copy, Duplicate and Delete, each named", !!hasButton(properties, T("exp.copy")) && !!hasButton(properties, T("exp.duplicate")) && !!hasButton(properties, T("exp.deleteSelected")) && !nameless.length, nameless.join(" | "));

  ctrl("KeyC");
  const sink = document.querySelector<HTMLTextAreaElement>(".clipboard-sink");
  const copied = sink?.value ?? "";
  let clip: { nodes?: unknown[]; edges?: unknown[] } = {};
  try { clip = JSON.parse(copied); } catch { /* reported below */ }
  expect("Ctrl+C puts the two nodes and the wire between them on the clipboard", clip.nodes?.length === 2 && clip.edges?.length === 1, copied.slice(0, 120));
  await until("the focus back from the clipboard field", () => document.activeElement !== sink);

  ctrl("KeyV");
  expect("Ctrl+V takes what the system pastes", document.activeElement === sink);
  const data = new DataTransfer();
  data.setData("text/plain", copied);
  sink!.dispatchEvent(new ClipboardEvent("paste", { clipboardData: data, bubbles: true, cancelable: true }));
  await until("the pasted nodes", () => count() === before.nodes + 2);
  expect("Ctrl+V adds the copies with their wire, selected", wires() === before.wires + 1 && chosen().length === 2, `${count()} nodes, ${wires()} wires, ${chosen().join(", ")}`);
  ctrl("KeyZ");
  await until("the paste undone", () => count() === before.nodes);
  expect("one Ctrl+Z takes the paste back", wires() === before.wires && chosen().length === 0);

  ctrl("KeyA");
  await until("everything selected", () => chosen().length === before.nodes);
  ctrl("KeyD");
  await until("the duplicates", () => count() > before.nodes);
  expect("Ctrl+A, Ctrl+D: all but Start and End duplicated", count() === before.nodes * 2 - 2, `${count()} nodes`);
  ctrl("KeyZ");
  await until("the duplicates undone", () => count() === before.nodes);

  // A click on the empty canvas drops the selection; Shift and a drag there draw a
  // frame from above-left of the HTTP node to past the next one, which selects both.
  const surface = editor.querySelector<HTMLElement>(".experiment-canvas")!;
  const nextName = textOf(next.querySelector(".experiment-node-type"));
  const nodeOf = (name: string) => [...editor.querySelectorAll<HTMLElement>(".experiment-node")].find((node) => textOf(node.querySelector(".experiment-node-type")) === name)!;
  const pointer = (target: HTMLElement, kind: string, x: number, y: number, shiftKey = false) => target.dispatchEvent(new PointerEvent(kind,
    { bubbles: true, cancelable: true, pointerId: 9, pointerType: "mouse", button: 0, buttons: kind === "pointerup" ? 0 : 1, clientX: x, clientY: y, shiftKey }));
  nodeOf(http).scrollIntoView({ block: "center", inline: "center" });
  const a = nodeOf(http).getBoundingClientRect();
  const b = nodeOf(nextName).getBoundingClientRect();
  const from = { x: Math.min(a.left, b.left) - 14, y: Math.min(a.top, b.top) - 14 };
  const to = { x: Math.max(a.right, b.right) + 6, y: Math.max(a.bottom, b.bottom) + 6 };
  pointer(surface, "pointerdown", from.x, from.y);
  pointer(surface, "pointerup", from.x, from.y);
  await until("nothing selected", () => chosen().length === 0);
  pointer(surface, "pointerdown", from.x, from.y, true);
  pointer(surface, "pointermove", (from.x + to.x) / 2, (from.y + to.y) / 2, true);
  await until("the frame drawn", () => editor.querySelector(".experiment-band"));
  pointer(surface, "pointermove", to.x, to.y, true);
  pointer(surface, "pointerup", to.x, to.y, true);
  await until("the framed nodes selected", () => chosen().length >= 2);
  expect("Shift and a drag on the canvas select what the frame touches", chosen().length === 2 && chosen().includes(http) && chosen().includes(nextName) && !editor.querySelector(".experiment-band"), chosen().join(", "));

  // A drag of one of them moves both; one undo puts both back.
  const left = (name: string) => Number.parseFloat(nodeOf(name).style.left);
  const start = { http: left(http), next: left(nextName) };
  const handle = nodeOf(http).querySelector<HTMLElement>(".experiment-node-body")!;
  const grip = handle.getBoundingClientRect();
  pointer(handle, "pointerdown", grip.left + 20, grip.top + 20);
  pointer(handle, "pointermove", grip.left + 80, grip.top + 20);
  pointer(handle, "pointerup", grip.left + 80, grip.top + 20);
  // A browser follows a drag with a click on the node; it must not undo the group.
  handle.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await until("the group moved", () => left(http) > start.http);
  const moved = { http: left(http) - start.http, next: left(nextName) - start.next };
  expect("dragging one selected node moves the group", moved.http > 0 && moved.http === moved.next && chosen().length === 2, JSON.stringify(moved));
  ctrl("KeyZ");
  await until("the drag undone", () => left(http) === start.http);
  expect("…and one Ctrl+Z puts it back", left(nextName) === start.next);

  key(document.activeElement ?? document.body, { key: "Delete", code: "Delete" });
  await until("the group deleted", () => count() === before.nodes - 2);
  expect("Delete removes the group and joins the flow around it", editor.querySelectorAll(".experiment-wires path.experiment-wire-hit").length >= 1, `${count()} nodes`);
  ctrl("KeyZ");
  await until("the delete undone", () => count() === before.nodes && wires() === before.wires);

  ctrl("KeyA");
  await until("all selected again", () => chosen().length === before.nodes);
  key(document.activeElement ?? document.body, { key: "Escape", code: "Escape" });
  await until("Escape drops a group", () => chosen().length === 0);
  expect("the template is as it was", count() === before.nodes && wires() === before.wires);
}

/**
 * Load on the HTTP node: 20 requests a second for 1.5 s, judged by its two
 * thresholds, its numbers in the properties; then a threshold that cannot hold
 * fails the step and says by how much.
 */
export async function experimentLoad(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateHttp");
  // A load is measured, not checked: the status check goes, its wires bridged to End.
  const check = await selectNode(editor, "exp.node.assert_status");
  await click(button(check, T("exp.delete")));
  await until("the check removed", () => editor.querySelectorAll(".experiment-node").length === 3);
  const properties = await selectNode(editor, "exp.node.http");
  await type(control(properties, "URL"), `http://127.0.0.1:${args.port}/e2e/load`);
  await setChecked(checkbox(properties, T("exp.loadOn")), true);
  expect("Load replaces Repeat and Retry", !textOf(properties).includes(T("exp.repeatOn")) && !textOf(properties).includes(T("exp.retryOn")));
  await type(control(properties, T("exp.loadShape")), "constant");
  await type(control(properties, T("exp.loadRate")), 20);
  await type(control(properties, T("exp.loadDuration")), 1500);
  await type(control(properties, T("exp.loadConcurrency")), 4);
  const planned = T("exp.loadPlanned", { approx: "false", n: 30, s: "1.5" });
  const chart = await until("the profile's chart", () => properties.querySelector(".experiment-load-chart figcaption"));
  expect("the chart adds the profile up", textOf(chart).includes(planned), textOf(chart));
  const nameless = unnamed(properties);
  expect("the load's fields have names", nameless.length === 0, nameless.join(" | "));
  const badge = await until("the load badge", () => editor.querySelector(".experiment-node-load"));
  expect("the node shows its rate", textOf(badge) === "⚡20/s", textOf(badge));

  const { outcome, rows } = await runAndWait(editor);
  expect("the run under load passes its thresholds", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("the timeline says what the load sent", rows.some((row) => row.includes(T("exp.node.http")) && row.includes("30 requests")), rows.join(" | "));
  const result = await until("the load's numbers", () => properties.querySelector<HTMLElement>(".experiment-load-result"));
  const verdicts = [...result.querySelectorAll(".experiment-verdicts li")];
  expect("both thresholds held", verdicts.length === 2 && verdicts.every((verdict) => verdict.classList.contains("held")), textOf(result));
  expect("every request answered 200", textOf(result.querySelector(".experiment-load-statuses")) === "200 × 30", textOf(result.querySelector(".experiment-load-statuses")));

  // p95 below nothing cannot hold: the step fails on it, the verdict in red.
  await type(properties.querySelector<HTMLInputElement>(".experiment-threshold input")!, 0);
  const failed = await runAndWait(editor);
  const rule = T("err.load.threshold", { metric: "p95_ms", op: "<", value: "0", actual: "" }).split(", ").pop()!;
  expect("a threshold that does not hold fails the run, saying which", failed.outcome.startsWith(T("exp.failed")) && failed.outcome.includes(rule), `${failed.outcome} · ${rule}`);
  const broken = await until("the broken threshold", () => properties.querySelector(".experiment-verdicts li.broken"));
  expect("…and the result shows which", textOf(broken).includes(`${T("exp.metric.p95_ms")} < 0`), textOf(broken));

  // Compare: this run beside the one before it.
  await click(button(editor, T("exp.compare")));
  const compare = await until("the Compare dialog", () => document.querySelector<HTMLDialogElement>("dialog.experiment-compare[open]"));
  const table = await until("the comparison", () => compare.querySelector(".experiment-compare-step table"));
  expect("each metric before and after", [...table.querySelectorAll("tbody th")].map(textOf).includes(T("exp.metric.p95_ms")), textOf(table));
  const threshold = compare.querySelector(".experiment-compare-step .experiment-verdicts li");
  expect("the first threshold held before and not after", textOf(threshold).includes("✓ → ✕"), textOf(threshold));
  const unnamedInDialog = unnamed(compare);
  expect("the dialog's controls have names", unnamedInDialog.length === 0, unnamedInDialog.join(" | "));
  await click(button(compare, T("exp.close")));
  await until("the dialog to close", () => !document.querySelector("dialog.experiment-compare[open]"));
}

export async function experimentParallel(expect: Expect, args: StepArgs) {
  const editor = await openTemplate("exp.templateParallel");
  const properties = await selectNode(editor, "exp.node.http");
  await type(control(properties, "URL"), `http://127.0.0.1:${args.port}/e2e/parallel`);
  const { outcome, rows } = await runAndWait(editor);
  expect("both branches run and the run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  expect("End passes once", rows.filter((row) => row.includes(T("exp.node.end")) && row.includes(T("exp.passed"))).length === 1, rows.join(" | "));
}

export async function experimentExport(expect: Expect, args: StepArgs) {
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

/** The bundled flaky API: an Emulator node edited in its dialog, and a run that retries until it answers. */
export async function experimentEmulator(expect: Expect, args: StepArgs) {
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

/**
 * Fault phases: a relay in front of an emulated device, switched clean →
 * lossy → offline → clean by a branch of its own while the other sends.
 */
export async function experimentFaults(expect: Expect) {
  const editor = await openTemplate("exp.templateFaults");
  expect("a relay and three switches, in the Faults group", editor.querySelectorAll('[data-group="fault"]').length === 4, String(editor.querySelectorAll('[data-group="fault"]').length));
  const properties = await selectNode(editor, "exp.node.impairment_change", 1, expect);
  const offline = button(properties, T("ns.preset.offline"));
  expect("the switch's preset is shown as chosen", offline.getAttribute("aria-pressed") === "true");
  const nameless = unnamed(properties);
  expect("the profile's fields all have names", nameless.length === 0, nameless.join(" | "));
  const { outcome, rows } = await runAndWait(editor, 30000);
  expect("the fault phases run passes", outcome === T("exp.passed"), `${outcome} · ${rows.join(" | ")}`);
  for (const preset of ["wifi", "offline", "lan"] as const) {
    expect(`the timeline says the network went ${preset}`, rows.some((row) => row.includes(T("exp.step.impaired", { profile: T(`ns.preset.${preset}`) }))), rows.join(" | "));
  }
}
