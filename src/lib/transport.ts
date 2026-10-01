import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { listen as tauriListen } from "@tauri-apps/api/event";

/**
 * How the interface reaches the engine. Every command goes through one door —
 * `Service::invoke` in the engine (engine/src/service.rs) — so this module only
 * has to know how to get there:
 *
 * - inside the desktop app, the Tauri command `engine` and Tauri events;
 * - in a browser, `POST /api/invoke/<command>` on the server that served the
 *   page, and its event stream on the WebSocket `/api/events`.
 *
 * Nothing else in the interface talks to Tauri or to the server for engine work.
 */

export type UnlistenFn = () => void;

/** One event from the engine, shaped like Tauri's so handlers read `payload`. */
export interface EngineEvent<T> {
  event: string;
  payload: T;
}

export type EventCallback<T> = (event: EngineEvent<T>) => void;

/** Inside the desktop app (Tauri injects its internals before the page loads). */
export const isDesktop = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Run an engine command; rejects with what the engine reported (an `EngineError` or text). */
export function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return isDesktop ? tauriInvoke<T>("engine", { command, args: args ?? null }) : webInvoke<T>(command, args);
}

/** Receive an engine event until the returned function is called. */
export function listen<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  if (isDesktop) return tauriListen<T>(event, (received) => handler({ event: received.event, payload: received.payload }));
  return Promise.resolve(events.add(event, handler as EventCallback<unknown>));
}

// ---- in a browser -------------------------------------------------------------

/**
 * A failure of the connection itself, as a dictionary key the interface
 * translates (`describeError` passes keys through `t`).
 */
const UNREACHABLE = "web.unreachable";
const BAD_RESPONSE = "web.badResponse";

async function webInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`/api/invoke/${encodeURIComponent(command)}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(args ?? {}),
      credentials: "same-origin",
    });
  } catch {
    throw UNREACHABLE;
  }
  const text = await response.text();
  let body: unknown = null;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    throw BAD_RESPONSE;
  }
  // The session ended (expired, or signed out elsewhere): sign in again.
  if (response.status === 401) {
    window.location.assign("/login");
  }
  if (!response.ok) throw body ?? BAD_RESPONSE;
  return body as T;
}

export type ConnectionState = "connecting" | "open" | "lost";

/**
 * One WebSocket per page for every engine event, reconnecting with backoff.
 * Events that happen while it is down are not replayed; the interface says
 * the connection was lost, and running jobs report their state again.
 */
class EventHub {
  private handlers = new Map<string, Set<EventCallback<unknown>>>();
  private socket: WebSocket | null = null;
  private attempts = 0;
  private state: ConnectionState = "connecting";
  private watchers = new Set<(state: ConnectionState) => void>();

  add(event: string, handler: EventCallback<unknown>): UnlistenFn {
    const set = this.handlers.get(event) ?? new Set();
    set.add(handler);
    this.handlers.set(event, set);
    this.connect();
    return () => { set.delete(handler); };
  }

  watch(watcher: (state: ConnectionState) => void): UnlistenFn {
    this.watchers.add(watcher);
    watcher(this.state);
    this.connect();
    return () => { this.watchers.delete(watcher); };
  }

  private setState(state: ConnectionState) {
    if (this.state === state) return;
    this.state = state;
    for (const watcher of this.watchers) watcher(state);
  }

  private connect() {
    if (this.socket) return;
    const url = `${window.location.protocol === "https:" ? "wss" : "ws"}://${window.location.host}/api/events`;
    const socket = new WebSocket(url);
    this.socket = socket;
    socket.onopen = () => { this.attempts = 0; this.setState("open"); };
    socket.onmessage = (message) => {
      let event: EngineEvent<unknown>;
      try { event = JSON.parse(String(message.data)); } catch { return; }
      for (const handler of this.handlers.get(event.event) ?? []) handler(event);
    };
    socket.onclose = () => {
      this.socket = null;
      this.setState(this.attempts === 0 && this.state === "connecting" ? "connecting" : "lost");
      // 0.5 s, 1 s, 2 s … up to 15 s between attempts.
      const delay = Math.min(15_000, 500 * 2 ** this.attempts++);
      window.setTimeout(() => this.connect(), delay);
    };
  }
}

const events = new EventHub();

/** The state of the event connection; always "open" inside the desktop app. */
export function watchConnection(watcher: (state: ConnectionState) => void): UnlistenFn {
  if (isDesktop) { watcher("open"); return () => {}; }
  return events.watch(watcher);
}
