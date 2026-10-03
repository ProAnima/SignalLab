/**
 * About and the feedback form: who makes the app, a screenshot pasted with
 * Ctrl+V and shown, the logs attached and shown — with the data folder left
 * out — and the form sent to the tour's stand-in for the studio's hub, which
 * the runner then reads. Against a server the tour did not start (the image),
 * nothing is sent: that server would send to the real hub.
 */
import { T, button, click, control, textOf, type, unnamed, untipped, until, type Expect, type StepArgs } from "../dsl";

/** A small PNG made here, as a screenshot tool would put it on the clipboard. */
async function screenshotFile(): Promise<File> {
  const canvas = document.createElement("canvas");
  canvas.width = 240;
  canvas.height = 140;
  const g = canvas.getContext("2d")!;
  g.fillStyle = "#3ee6b0";
  g.fillRect(0, 0, 240, 140);
  const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob((made) => made ? resolve(made) : reject(new Error("no PNG from the canvas")), "image/png"));
  return new File([blob], "image.png", { type: "image/png" });
}

export async function feedback(expect: Expect, args: StepArgs) {
  const header = document.querySelector<HTMLElement>(".header")!;
  await click(button(header, T("about.open")));
  const about = await until("About", () => document.querySelector<HTMLDialogElement>("dialog.about-dialog[open]"));
  const facts = textOf(about);
  expect("About names the studio, the author and how to reach them",
    facts.includes(T("about.studio")) && facts.includes(T("about.authorName")) && facts.includes("info@proanima.net"), facts);
  expect("…and the version", facts.includes(T("about.version", { version: args.version })), facts);
  expect(args.mode === "desktop" ? "…and its updates" : "…and that a server updates with its image",
    args.mode === "desktop" ? !!about.querySelector(".about-updates button") : facts.includes(T("update.server")), facts);
  const nameless = [...unnamed(about), ...untipped(about)];
  expect("every control of About has a name", nameless.length === 0, nameless.join(" | "));

  await click(button(about, T("about.writeUs")));
  const form = await until("the feedback form", () => document.querySelector<HTMLDialogElement>("dialog.feedback-dialog[open]"));
  expect("About gives way to the form", !document.querySelector("dialog.about-dialog[open]"));
  const message = "E2E: the form reaches the developers";
  await type(control(form, T("feedback.message")), message);
  await type(control(form, T("feedback.email")), "tour@example.com");

  // Ctrl+V with an image on the clipboard: the paste event a webview fires.
  const clipboard = new DataTransfer();
  clipboard.items.add(await screenshotFile());
  control(form, T("feedback.message")).dispatchEvent(new ClipboardEvent("paste", { clipboardData: clipboard, bubbles: true, cancelable: true }));
  const thumbnail = await until("the pasted screenshot", () => form.querySelector<HTMLImageElement>(".feedback-shot img"));
  await until("its thumbnail drawn", () => thumbnail.complete && thumbnail.naturalWidth > 0, 4000).catch(() => null);
  expect("a pasted screenshot is shown — the page may draw it", thumbnail.naturalWidth === 240, `${thumbnail.naturalWidth} px wide`);
  expect("…named for when it was pasted", /^screenshot-\d{4}-\d\d-\d\d-\d{6}\.png$/.test(thumbnail.alt), thumbnail.alt);

  await click(button(form, T("feedback.show"), { index: 0 }));
  const shown = await until("the console log", () => form.querySelector<HTMLElement>(".feedback-preview"));
  const log = shown.textContent ?? "";
  expect("the console log is attached, in English", log.includes(T("log.ready")), log.slice(0, 300));
  expect("…without the data folder's path", !!args.dataDir && !log.includes(args.dataDir), log.slice(0, 300));
  const unnamedForm = [...unnamed(form), ...untipped(form)];
  expect("every control of the form has a name", unnamedForm.length === 0, unnamedForm.join(" | "));

  if (!args.send) {
    await click(button(form, T("common.cancel")));
    await until("the form closed", () => !document.querySelector("dialog.feedback-dialog[open]"));
    return;
  }
  await click(button(form, T("common.send")));
  const sent = await until("the answer", () => form.querySelector<HTMLElement>(".feedback-sent") ?? form.querySelector<HTMLElement>(".error-message"), 20_000);
  expect("the service took it and gave a reference", sent.classList.contains("feedback-sent") && textOf(sent).includes("e2e"), textOf(sent));
  await click(button(form, T("common.close")));
  await until("the form closed", () => !document.querySelector("dialog.feedback-dialog[open]"));
  return { message };
}
