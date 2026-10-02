import { getCurrentWindow } from "@tauri-apps/api/window";
import { isDesktop } from "./transport";

/**
 * What differs between the desktop app and a browser, in one place: the
 * window, files the engine wrote, and signing out. Screens ask here instead of
 * checking where they run.
 */

export { isDesktop };

export async function isFullscreen(): Promise<boolean> {
  if (isDesktop) return getCurrentWindow().isFullscreen();
  return document.fullscreenElement !== null;
}

/** Enter or leave fullscreen; resolves to the new state. */
export async function setFullscreen(on: boolean): Promise<boolean> {
  if (isDesktop) {
    await getCurrentWindow().setFullscreen(on);
    return on;
  }
  if (on && !document.fullscreenElement) await document.documentElement.requestFullscreen();
  if (!on && document.fullscreenElement) await document.exitFullscreen();
  return on;
}

/** Calls `changed` whenever fullscreen may have changed (window resize, Escape in a browser). */
export function onFullscreenChange(changed: () => void): () => void {
  const event = isDesktop ? "resize" : "fullscreenchange";
  const target: EventTarget = isDesktop ? window : document;
  target.addEventListener(event, changed);
  return () => target.removeEventListener(event, changed);
}

/**
 * A link that downloads a file the engine wrote (an export, a run report). In
 * a browser the file is on the server, so it is fetched from there; inside the
 * desktop app it is already on this computer and the path is shown instead.
 */
export function downloadUrl(path: string): string | null {
  return isDesktop ? null : `/api/files?path=${encodeURIComponent(path)}`;
}

/**
 * Save a file the server offers, without leaving the page: a link with
 * `download` is followed as a download whatever the response is, where
 * navigating to it would replace the interface with an error page.
 */
export function saveDownload(url: string): void {
  const link = document.createElement("a");
  link.href = url;
  link.download = "";
  link.hidden = true;
  document.body.append(link);
  link.click();
  link.remove();
}

/**
 * Put `text` on the clipboard; whether it worked. A server reached over plain
 * HTTP on a LAN address is not a secure context and has no clipboard API, so
 * a selected, hidden text area is copied the old way instead.
 */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    // Refused: try the old way.
  }
  const area = document.createElement("textarea");
  area.value = text;
  area.setAttribute("readonly", "");
  area.style.position = "fixed";
  area.style.opacity = "0";
  document.body.append(area);
  area.select();
  let copied = false;
  try {
    copied = document.execCommand("copy");
  } catch {
    copied = false;
  }
  area.remove();
  return copied;
}

/** Whether the server serving this page asks for a token. */
export async function serverNeedsSignIn(): Promise<boolean> {
  if (isDesktop) return false;
  try {
    const health = await (await fetch("/api/health", { credentials: "same-origin" })).json();
    return health?.auth === true;
  } catch {
    return false;
  }
}

/** End this browser's session on the server. */
export async function signOut(): Promise<void> {
  try {
    await fetch("/logout", { method: "POST", credentials: "same-origin" });
  } finally {
    window.location.assign("/login");
  }
}
