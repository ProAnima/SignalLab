import type { AppInfo, FeedbackForm, HostInfo, JobInfo } from "./api";
import type { Lang } from "./i18n";

/**
 * What the feedback form attaches and sends (docs/hub.md): the limits of the
 * studio's hub, checked before anything is uploaded; screenshots read as the
 * hub takes them; and the logs that help — the console, in English for the
 * developers, and what the app runs on. Neither carries this machine's name,
 * its address or its folders.
 */

/** The hub's limits, as the engine has them (engine/src/feedback.rs; a test keeps them equal). */
export const LIMITS = {
  messageChars: 20_000,
  images: 6,
  imageBytes: 8 << 20,
  totalBytes: 15 << 20,
};

export const IMAGE_TYPES = ["image/png", "image/jpeg", "image/webp", "image/gif"];

/** A screenshot to send: its bytes base64, and the same as a `data:` URL for its thumbnail. */
export interface Picked {
  id: string;
  name: string;
  size: number;
  data: string;
  url: string;
}

/**
 * `text` without what names this machine or its person: the host name, its
 * address and the data folder become `{host}`, `{this-ip}`, `{data}`, and
 * the user's folder in any path (`C:\Users\name\`, `/home/name/`) loses the name.
 */
export function scrub(text: string, { host, dataDir }: { host?: HostInfo | null; dataDir?: string }): string {
  const known: [string, string][] = [];
  if (dataDir) known.push([dataDir, "{data}"]);
  if (host?.hostname && host.hostname.length >= 3) known.push([host.hostname, "{host}"]);
  if (host?.local_ip && !host.local_ip.startsWith("127.")) known.push([host.local_ip, "{this-ip}"]);
  let out = text;
  // Longest first, so a folder inside another is replaced whole.
  for (const [value, marker] of known.sort((a, b) => b[0].length - a[0].length)) out = out.split(value).join(marker);
  return out
    .replace(/([A-Za-z]:\\(?:Users|Documents and Settings)\\)[^\\\s"'<>]+/gi, "$1…")
    .replace(/(\/(?:home|Users)\/)[^/\s"'<>]+/g, "$1…");
}

/** `hh:mm:ss.mmm`, the console's own time. */
export function clock(ts: number): string {
  const at = new Date(ts);
  const two = (value: number) => String(value).padStart(2, "0");
  return `${two(at.getHours())}:${two(at.getMinutes())}:${two(at.getSeconds())}.${String(at.getMilliseconds()).padStart(3, "0")}`;
}

export interface Surroundings {
  info: AppInfo | null;
  lang: Lang;
  jobs: JobInfo[];
  connection: string;
}

/** What the app runs on, for the developers: versions and kinds, never names, addresses or paths. */
export function systemText({ info, lang, jobs, connection }: Surroundings): string {
  const system = {
    app: "Signal Lab",
    version: info?.version ?? __APP_VERSION__,
    interface: __APP_VERSION__,
    mode: info?.mode ?? null,
    os: info?.os ?? null,
    arch: info?.arch ?? null,
    language: lang,
    screen: `${window.screen.width}×${window.screen.height} @${window.devicePixelRatio}`,
    window: `${window.innerWidth}×${window.innerHeight}`,
    webview: navigator.userAgent,
    connection,
    jobs: jobs.map((job) => job.kind),
  };
  return JSON.stringify(system, null, 2) + "\n";
}

/** The form's `meta`: what the mail's subject and footer say about the app. */
export function metaOf({ info, lang }: Surroundings): FeedbackForm["meta"] {
  const meta: Record<string, string> = { version: info?.version ?? __APP_VERSION__, lang, screen: `${window.screen.width}x${window.screen.height}` };
  if (info) Object.assign(meta, { os: info.os, arch: info.arch, mode: info.mode });
  return meta;
}

/** Why a file cannot be attached, as a text key and its values; null when it can. */
export function refusalOf(file: { name: string; size: number; type: string }, picked: Picked[]): { key: string; params: Record<string, string | number> } | null {
  if (!IMAGE_TYPES.includes(file.type)) return { key: "feedback.notImage", params: { name: file.name } };
  if (file.size > LIMITS.imageBytes) return { key: "feedback.imageTooLarge", params: { name: file.name, mb: LIMITS.imageBytes >> 20 } };
  if (picked.length >= LIMITS.images) return { key: "feedback.tooManyImages", params: { n: LIMITS.images } };
  const total = picked.reduce((sum, item) => sum + item.size, 0) + file.size;
  if (total > LIMITS.totalBytes) return { key: "feedback.tooLarge", params: { mb: LIMITS.totalBytes >> 20 } };
  return null;
}

const EXTENSIONS: Record<string, string> = { "image/png": "png", "image/jpeg": "jpg", "image/webp": "webp", "image/gif": "gif" };

/**
 * A file's name as it is sent: a pasted image is called `image.png` by every
 * clipboard, so it gets the time it was pasted instead.
 */
export function nameOf(file: { name: string; type: string }, pasted: boolean, at = new Date()): string {
  if (!pasted && file.name) return file.name;
  const two = (value: number) => String(value).padStart(2, "0");
  return `screenshot-${at.getFullYear()}-${two(at.getMonth() + 1)}-${two(at.getDate())}-${two(at.getHours())}${two(at.getMinutes())}${two(at.getSeconds())}.${EXTENSIONS[file.type] ?? "png"}`;
}

/**
 * A file read for sending: base64, and the `data:` URL it came as for its
 * thumbnail — the server's page allows `data:` images, not `blob:` ones.
 */
export function readImage(file: File, pasted: boolean): Promise<Picked> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error);
    reader.onload = () => {
      const url = String(reader.result);
      resolve({
        id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
        name: nameOf(file, pasted),
        size: file.size,
        data: url.slice(url.indexOf(",") + 1),
        url,
      });
    };
    reader.readAsDataURL(file);
  });
}
