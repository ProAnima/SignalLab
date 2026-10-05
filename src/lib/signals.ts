/**
 * Signal library helpers: firing, describing, and turning a captured frame back
 * into something editable.
 *
 * Firing deliberately goes through the same commands the module views use, so a
 * signal is not a second send path — it is the same one with the fields filled
 * in. That is also why a fired signal shows up in the Inspector under `osc-send`
 * or `http` rather than under a source of its own.
 */
import {
  api,
  type ExperimentNode, type MqttConfig, type Signal, type SignalBody,
} from "./api";
import type { TKey, Translate } from "./i18n";
import { responseFailure } from "./errors";
import { makeId, signalFromFrame, splitBroker } from "./library";

// The pure parts live in library.ts (node tests import it); screens import them from here.
export { makeId, signalFromFrame, splitBroker };

export const TRANSPORTS = ["osc", "udp", "http", "mqtt"] as const;
export type Transport = (typeof TRANSPORTS)[number];

export const transportKey = (t: Transport): TKey => `sig.tr.${t}` as TKey;

/** Where the signal is aimed, for the list column and the palette. */
export function signalTarget(s: Signal): string {
  switch (s.body.transport) {
    case "osc":
    case "udp":
      return s.body.target;
    case "http":
      return s.body.request.url;
    case "mqtt":
      return s.body.broker;
  }
}

/** One line of what it sends, for the list and the palette. */
export function signalSummary(s: Signal): string {
  const b = s.body;
  switch (b.transport) {
    case "osc":
      return b.args.length ? `${b.address} · ${b.args.length}` : b.address;
    case "udp":
      return b.payload.kind === "text" ? b.payload.text : b.payload.hex;
    case "http":
      return b.request.method;
    case "mqtt": {
      const value = b.payload === "" ? "(empty)" : b.payload;
      return `${b.topic} = ${value}${b.retain ? " · retained" : ""}`;
    }
  }
}

/** What a fired signal did, as a console line: its text key and values (the signal's name is added). */
export interface Fired { key: TKey; params: Record<string, string | number> }

/** Where the HTTP screen keeps its *Keep cookies* (on unless turned off). */
export const HTTP_COOKIES_KEY = "signal-lab.http.cookies";

function httpKeepsCookies(): boolean {
  try {
    const raw = localStorage.getItem(HTTP_COOKIES_KEY);
    return raw === null || JSON.parse(raw) !== false;
  } catch {
    return true;
  }
}

/**
 * Fire it. Returns a line for the console; throws the engine's message as-is.
 *
 * `liveMqttJobId` is an open MQTT connection to the signal's own broker, if
 * there is one (`mqttConnectionFor`): an MQTT signal rides it rather than
 * dialling again, so it goes out with that connection's credentials and client
 * id — which is also why the library file stores no password of its own.
 * Without one it connects to its broker for this one publish.
 */
export async function fireSignal(s: Signal, liveMqttJobId?: number | null): Promise<Fired> {
  const b = s.body;
  switch (b.transport) {
    case "osc": {
      const bytes = await api.oscSend(b.target, b.address, b.args);
      return { key: "log.firedOsc", params: { address: b.address, target: b.target, bytes } };
    }
    case "udp": {
      // Fan-out to a one-host list is exactly "send this payload there once".
      const r = await api.broadcastSend({
        mode: "list",
        target: b.target,
        port: 0,
        payload: b.payload,
        bind: null,
        ttl: 1,
        multicast_loop: false,
        rate: 1,
        count: 1,
        duration_s: 0,
      });
      if (r.errors > 0) throw r.error ?? { code: "transport.failed", params: { target: b.target } };
      return { key: "log.firedUdp", params: { target: b.target, bytes: r.bytes } };
    }
    case "http": {
      // With the HTTP screen's jar while it keeps cookies, as its own Send: the same request, the same bytes.
      const r = await api.httpRequest(b.request, httpKeepsCookies());
      // A refused connection is a successful command with a failed request; the
      // library should treat it as a failure, the way a person would.
      if (r.error) throw responseFailure(r, b.request.url);
      return { key: "log.firedHttp", params: { method: b.request.method, url: b.request.url, status: r.status, ms: Math.round(r.latency_ms) } };
    }
    case "mqtt": {
      if (liveMqttJobId != null) {
        await api.mqttPublish(liveMqttJobId, b.topic, b.payload, b.qos, b.retain);
        return { key: "log.firedMqtt", params: { topic: b.topic, id: liveMqttJobId, qos: b.qos, retain: String(b.retain) } };
      }
      const { host, port } = splitBroker(b.broker);
      const config: MqttConfig = {
        host,
        port,
        // A fresh id per fire: reusing one would kick the other client off.
        client_id: `signal-lab-${Math.random().toString(16).slice(2, 8)}`,
        username: "",
        password: "",
        keep_alive_s: 15,
        clean_session: true,
        will: null,
        subscribe: [],
      };
      const summary = await api.mqttPublishOnce(config, b.topic, b.payload, b.qos, b.retain);
      return { key: "log.firedMqttOnce", params: { summary } };
    }
  }
}

const BLANK: Record<Transport, SignalBody> = {
  osc: { transport: "osc", target: "127.0.0.1:9000", address: "/hello", args: [] },
  udp: { transport: "udp", target: "127.0.0.1:9000", payload: { kind: "text", text: "" } },
  http: {
    transport: "http",
    request: { method: "GET", url: "http://127.0.0.1:9000/", headers: [], body: null, timeout_ms: 10000 },
  },
  mqtt: {
    transport: "mqtt",
    broker: "127.0.0.1:1883",
    topic: "global/theme",
    payload: "",
    qos: 0,
    retain: false,
  },
};

export function blankBody(t: Transport): SignalBody {
  return structuredClone(BLANK[t]);
}


/** A topic seen on a broker, as a signal that republishes its current value. */
export function signalFromMqttTopic(
  broker: string,
  topic: string,
  payload: string,
  qos: number,
  retain: boolean,
  taken: Signal[],
  t: Translate,
): Signal {
  return {
    id: makeId(topic, taken),
    name: topic,
    group: t("sig.capturedFolder"),
    note: t(retain ? "sig.capturedRetainedNote" : "sig.capturedNote", { broker }),
    body: { transport: "mqtt", broker, topic, payload, qos, retain },
  };
}

/**
 * A signal as an experiment action with the same parameters. Hex UDP payloads
 * have no node form yet (the UDP node sends text), so they return null.
 */
export function nodeFromSignal(body: SignalBody, x: number, y: number): ExperimentNode | null {
  const base = { x: Math.max(12, Math.round(x)), y: Math.max(12, Math.round(y)) };
  const id = (type: string) => `${type}-${crypto.randomUUID()}`;
  switch (body.transport) {
    case "osc":
      return { ...base, id: id("osc"), type: "osc", target: body.target, address: body.address, args: structuredClone(body.args) };
    case "http":
      return { ...base, id: id("http"), type: "http", request: structuredClone(body.request) };
    case "udp":
      return body.payload.kind === "text"
        ? { ...base, id: id("udp"), type: "udp", target: body.target, text: body.payload.text }
        : null;
    case "mqtt": {
      const { host, port } = splitBroker(body.broker);
      return { ...base, id: id("mqtt"), type: "mqtt", host, port, topic: body.topic, payload: body.payload, qos: body.qos, retain: body.retain };
    }
  }
}
