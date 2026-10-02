/** The signal library: a signal made and fired, a request saved from HTTP, its folders worked in Signals. */
import { T, textOf, numberIn, until, screen, control, button, hasButton, buttonWith, panel, unnamed, click, type, key, go, type StepArgs, type Expect } from "../dsl";

export async function signals(expect: Expect, args: StepArgs) {
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

/** A drag of `what` onto `where`, as the browser sends it (dragstart … drop). */
function dragOnto(what: Element, where: Element) {
  const data = new DataTransfer();
  what.dispatchEvent(new DragEvent("dragstart", { bubbles: true, cancelable: true, dataTransfer: data }));
  where.dispatchEvent(new DragEvent("dragover", { bubbles: true, cancelable: true, dataTransfer: data }));
  where.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: data }));
  what.dispatchEvent(new DragEvent("dragend", { bubbles: true, cancelable: true, dataTransfer: data }));
}

/** Save a request from HTTP into a folder, keep it saved, and work the folders in Signals. */
export async function library(expect: Expect, args: StepArgs) {
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
