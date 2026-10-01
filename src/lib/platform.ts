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
