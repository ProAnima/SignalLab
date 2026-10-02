/** The Inspector in the bottom panel: capture armed at the start, what it holds at the end. */
import { downloads } from "../page";
import { T, textOf, numberIn, until, button, hasButton, panel, unnamed, click, type, logLines, openInspector, type StepArgs, type Expect } from "../dsl";

export async function inspectArm(expect: Expect) {
  const shown = await openInspector();
  const nameless = unnamed(shown);
  expect("inspector: every field and button has a name", nameless.length === 0, nameless.join(" | "));
  if (!hasButton(shown, T("ins.disarm"))) await click(button(shown, T("ins.arm")));
  await until("capture armed", () => hasButton(shown, T("ins.disarm")));
  expect("capture is armed", !!shown.querySelector(".rec-dot.live"));
}

export async function inspectCheck(expect: Expect, args: StepArgs) {
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
