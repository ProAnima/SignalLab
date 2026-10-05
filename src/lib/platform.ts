import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent } from "@tauri-apps/plugin-updater";
import { DOCS_ONLINE, docsPath } from "./docs";
import { isDesktop } from "./transport";

/**
 * What differs between the desktop app and a browser, in one place: the
 * window, files the engine wrote, links that leave the app, the documentation,
 * signing out, and updating — which only an installed app does. Screens ask here instead of
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

/**
 * A web page or a mail address, in the system's browser or mail program. The
 * desktop app may open only the project's pages and its address
 * (src-tauri/capabilities/default.json); a browser opens a page in a new tab.
 */
export async function openExternal(url: string): Promise<void> {
  if (isDesktop) return openUrl(url);
  if (url.startsWith("mailto:")) window.location.href = url;
  else window.open(url, "_blank", "noopener,noreferrer");
}

/**
 * The documentation at `page` (`protocols/osc`) in `lang`, beside the
 * interface: the pages built into the app or the server (`/docs/`), so it is
 * there offline. The desktop app shows it in a window of its own, which sends
 * links out of it to the system's browser; a browser in a tab of its own. A
 * development build has none inside and opens the published pages.
 */
export async function openDocs(page: string, lang: string): Promise<void> {
  const path = docsPath(page, lang);
  if (import.meta.env.DEV) return openExternal(`${DOCS_ONLINE}${path}`);
  if (isDesktop) return tauriInvoke("open_docs", { page: `docs/${path}` });
  window.open(`/docs/${path}`, "signal-lab-docs");
}

// ---- updates: the desktop app only ------------------------------------------------

/** A newer release, found and ready to install. */
export interface FoundUpdate {
  version: string;
  /** When it was published, ISO 8601. */
  date?: string;
  /** Its release notes (the CHANGELOG section). */
  notes?: string;
  /**
   * Download it — its signature checked against the app's key before anything
   * runs — and install it. On Windows the installer takes over and restarts the
   * app, so this does not return there; elsewhere `restartApp` follows.
   */
  install: (progress: (received: number, total: number | undefined) => void) => Promise<void>;
}

/** Whether this build looks for updates on its own (a release build, not the tour's). */
export async function updateChecksEnabled(): Promise<boolean> {
  if (!isDesktop) return false;
  try {
    return await tauriInvoke<boolean>("update_checks");
  } catch {
    return false;
  }
}

const INSTALL_ID = "signal-lab.installId";

/**
 * This install's random number, which the update check gives the hub so that a
 * release can reach a share of installs first (the same ones, as the share
 * grows). Made once and kept with the app's other settings; it says nothing
 * about the person or the computer. Where nothing can be kept there is none,
 * and the hub offers only a release that reaches everyone.
 */
function installId(): string | null {
  try {
    const kept = localStorage.getItem(INSTALL_ID);
    if (kept && /^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$/.test(kept)) return kept;
    // A version 4 UUID, from the platform's random numbers.
    const bytes = crypto.getRandomValues(new Uint8Array(16));
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    const hex = [...bytes].map((byte) => byte.toString(16).padStart(2, "0")).join("");
    const made = `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
    localStorage.setItem(INSTALL_ID, made);
    return made;
  } catch {
    return null;
  }
}

/**
 * The newest published release offered to this app, if it is newer; never in a
 * browser. The hub answers first (`plugins.updater.endpoints`): what its stable
 * channel offers this install, or nothing; GitHub's latest release only when
 * the hub cannot be reached.
 */
export async function findUpdate(): Promise<FoundUpdate | null> {
  if (!isDesktop) return null;
  const id = installId();
  const update = await check({ timeout: 30_000, headers: id ? { "X-Install-Id": id } : undefined });
  if (!update) return null;
  return {
    version: update.version,
    date: update.date ?? undefined,
    notes: update.body ?? undefined,
    install: async (progress) => {
      let received = 0;
      let total: number | undefined;
      await update.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === "Started") total = event.data.contentLength ?? undefined;
        if (event.event === "Progress") received += event.data.chunkLength;
        progress(received, total);
      });
    },
  };
}

/** Start the app again, the installed version. */
export async function restartApp(): Promise<void> {
  if (isDesktop) await relaunch();
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
