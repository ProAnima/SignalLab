/** The experiment editor: the bundled templates opened, edited and run, and an experiment exported. */
import { downloads } from "../page";
import { T, textOf, numberIn, until, control, checkbox, button, hasButton, buttonWith, unnamed, click, type, setChecked, key, go, type Key, type StepArgs, type Expect } from "../dsl";

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
