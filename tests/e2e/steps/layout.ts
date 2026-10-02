/** The editor's wires and every pane handle, by pointer and by keyboard. */
import { T, textOf, until, button, hasButton, buttonWith, unnamed, click, key, go, type Expect } from "../dsl";

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
export async function layout(expect: Expect) {
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
