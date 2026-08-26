import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn, type EventCallback } from "@tauri-apps/api/event";

// ---- shared types (mirror the Rust engine) ----

export interface JobInfo {
  id: number;
  kind: string;
  label: string;
  started_ms: number;
}

export type OscArg =
  | { type: "int"; value: number }
  | { type: "float"; value: number }
  | { type: "str"; value: string }
  | { type: "long"; value: number }
  | { type: "double"; value: number }
  | { type: "bool"; value: boolean }
  | { type: "blob"; value: number[] }
  | { type: "nil" };

export interface OscMessage {
  address: string;
  args: OscArg[];
}

export interface HttpRequest {
  method: string;
  url: string;
  headers: [string, string][];
  body: string | null;
  timeout_ms: number;
}

export interface HttpResponse {
  ok: boolean;
  status: number;
  status_text: string;
  latency_ms: number;
  headers: [string, string][];
  body: string;
  body_bytes: number;
  truncated: boolean;
  error: string | null;
}

export type Waveform =
  | "sine" | "triangle" | "saw" | "square" | "random" | "ramp" | "constant";

export interface GenConfig {
  target: string;
  address: string;
  rate: number;
  waveform: Waveform;
  freq: number;
  min: number;
  max: number;
  as_int: boolean;
  duration_s: number;
}

export interface BurstConfig extends HttpRequest {
  concurrency: number;
  total: number;
  duration_s: number;
}

export interface ImpairProfile {
  latency_ms: number;
  jitter_ms: number;
  loss: number;
  duplicate: number;
  corrupt: number;
}

export interface ProxyConfig {
  listen: string;
  target: string;
  profile: ImpairProfile;
}

export interface StormConfig {
  target: string;
  protocol: "udp" | "tcp";
  size: number;
  rate: number;
  duration_s: number;
}

export interface ScanConfig {
  host: string;
  port_start: number;
  port_end: number;
  concurrency: number;
  timeout_ms: number;
  grab_banner: boolean;
}

export interface HostInfo {
  local_ip: string;
  hostname: string;
}

// ---- broadcast / discovery ----

export type TargetMode = "list" | "broadcast" | "multicast" | "sweep";

export type Payload =
  | { kind: "osc"; address: string; args: OscArg[] }
  | { kind: "text"; text: string }
  | { kind: "hex"; hex: string };

export interface EmitConfig {
  mode: TargetMode;
  target: string;
  /** Port for `sweep` mode; the other modes carry it inside `target`. */
  port: number;
  payload: Payload;
  bind: string | null;
  ttl: number;
  multicast_loop: boolean;
  /** Beacon rounds per second — one round is one packet per target. */
  rate: number;
  count: number;
  duration_s: number;
}

export interface EmitResult {
  targets: number;
  packets: number;
  bytes: number;
  errors: number;
  resolved: string[];
  summary: string;
}

export interface DiscoveryConfig {
  bind: string;
  groups: string[];
  interface: string | null;
  reuse: boolean;
  respond: boolean;
  response: Payload | null;
  respond_delay_ms: number;
  match_contains: string | null;
}

export interface Peer {
  addr: string;
  proto: string;
  packets: number;
  bytes: number;
  first_ms: number;
  last_ms: number;
  last_summary: string;
  responded: number;
}

// ---- mqtt ----

export interface MqttWill {
  topic: string;
  payload: string;
  qos: number;
  retain: boolean;
}

export interface MqttSub {
  filter: string;
  qos: number;
}

export interface MqttConfig {
  host: string;
  port: number;
  client_id: string;
  username: string;
  password: string;
  keep_alive_s: number;
  clean_session: boolean;
  will: MqttWill | null;
  /** Subscribed the moment the connection is up. `#` scans the whole broker. */
  subscribe: MqttSub[];
}

export interface MqttMessage {
  ts: number;
  topic: string;
  payload: string;
  bytes: number;
  qos: number;
  retain: boolean;
  dup: boolean;
}

export interface MqttGrant {
  filter: string;
  qos: number;
  accepted: boolean;
}

// ---- signal library ----

/**
 * Bytes for a raw signal — narrower than `Payload` on purpose: an OSC message
 * is what the `osc` transport is for. It is still assignable to `Payload`, so
 * firing one goes straight to `broadcast_send`.
 */
export type RawPayload =
  | { kind: "text"; text: string }
  | { kind: "hex"; hex: string };

/** What a stored signal puts on the wire; the tag is its transport. */
export type SignalBody =
  | { transport: "osc"; target: string; address: string; args: OscArg[] }
  | { transport: "udp"; target: string; payload: RawPayload }
  | { transport: "http"; request: HttpRequest }
  | { transport: "mqtt"; broker: string; topic: string; payload: string; qos: number; retain: boolean };

export interface Signal {
  id: string;
  name: string;
  /** Folder in the explorer; empty is the ungrouped root. */
  group: string;
  note: string;
  body: SignalBody;
}

export interface Library {
  version: number;
  signals: Signal[];
}

export interface LibraryFile {
  path: string;
  library: Library;
  /** The file did not exist and the starter set was written. */
  seeded: boolean;
}

// ---- inspector ----

export interface Frame {
  seq: number;
  ts: number;
  proto: string;
  dir: "tx" | "rx";
  source: string;
  job_id: number | null;
  local: string;
  remote: string;
  bytes: number;
  summary: string;
  detail: string | null;
  hex: string | null;
  verdict: string | null;
}

export interface CaptureStats {
  enabled: boolean;
  total: number;
  bytes: number;
  skipped: number;
  buffered: number;
  capacity: number;
}

// ---- event payloads ----

export interface OscInbound {
  job_id: number;
  ts: number;
  from: string;
  bytes: number;
  messages: OscMessage[];
  error: string | null;
}
export interface GenTick { job_id: number; ts: number; value: number; sent: number; }
export interface BurstProgress {
  job_id: number; ts: number; sent: number; ok: number; failed: number;
  rps: number; last_latency_ms: number; min_latency_ms: number;
  max_latency_ms: number; avg_latency_ms: number;
}
export interface ProxyStat {
  job_id: number; ts: number; forwarded: number; dropped: number;
  duplicated: number; corrupted: number; bytes: number;
}
export interface StormStat {
  job_id: number; ts: number; packets: number; bytes: number;
  errors: number; pps: number; mbps: number;
}
export interface OpenPort { job_id: number; ts: number; port: number; banner: string | null; }
export interface ScanProgress { job_id: number; ts: number; done: number; total: number; open: number; }
export interface JobEnded { job_id: number; kind: string; error: string | null; }
export interface EmitStat {
  job_id: number; ts: number; rounds: number; packets: number;
  bytes: number; errors: number; pps: number;
}
export interface PeerReport {
  job_id: number; ts: number; peers: Peer[];
  packets: number; bytes: number; responses: number;
}
export interface MqttBatch {
  job_id: number;
  ts: number;
  messages: MqttMessage[];
  /** Messages the accumulator had to shed while the UI was behind. */
  dropped: number;
}
export interface MqttStateEvent {
  job_id: number;
  ts: number;
  state: "connected" | "subscribed" | "closed";
  broker: string;
  error: string | null;
  grants: MqttGrant[];
}
export interface MqttAck {
  job_id: number;
  ts: number;
  kind: "published" | "unsubscribed";
  packet_id: number;
  topic: string | null;
}
export interface InspectBatch {
  frames: Frame[];
  stats: CaptureStats;
  /** Frames that existed but never reached the UI since the last batch. */
  skipped_now: number;
}

// ---- command wrappers ----

export const api = {
  hostInfo: () => invoke<HostInfo>("get_host_info"),
  jobsList: () => invoke<JobInfo[]>("jobs_list"),
  jobStop: (id: number) => invoke<boolean>("job_stop", { id }),
  jobsStopAll: () => invoke<void>("jobs_stop_all"),

  oscSend: (target: string, address: string, args: OscArg[]) =>
    invoke<number>("osc_send", { target, address, args }),
  oscMonitorStart: (bind: string) => invoke<JobInfo>("osc_monitor_start", { bind }),
  oscGeneratorStart: (config: GenConfig) => invoke<JobInfo>("osc_generator_start", { config }),

  httpRequest: (request: HttpRequest) => invoke<HttpResponse>("http_request", { request }),
  httpBurstStart: (config: BurstConfig) => invoke<JobInfo>("http_burst_start", { config }),

  netsimStart: (config: ProxyConfig) => invoke<JobInfo>("netsim_start", { config }),
  stormStart: (config: StormConfig) => invoke<JobInfo>("storm_start", { config }),
  scanStart: (config: ScanConfig) => invoke<JobInfo>("scan_start", { config }),

  broadcastSend: (config: EmitConfig) => invoke<EmitResult>("broadcast_send", { config }),
  broadcastBeaconStart: (config: EmitConfig) =>
    invoke<JobInfo>("broadcast_beacon_start", { config }),
  discoveryStart: (config: DiscoveryConfig) => invoke<JobInfo>("discovery_start", { config }),

  inspectSetEnabled: (enabled: boolean) =>
    invoke<CaptureStats>("inspect_set_enabled", { enabled }),
  inspectStats: () => invoke<CaptureStats>("inspect_stats"),
  inspectSnapshot: (limit: number) => invoke<Frame[]>("inspect_snapshot", { limit }),
  inspectClear: () => invoke<CaptureStats>("inspect_clear"),
  inspectExport: (format: "jsonl" | "txt") => invoke<string>("inspect_export", { format }),

  mqttConnect: (config: MqttConfig) => invoke<JobInfo>("mqtt_connect", { config }),
  mqttPublish: (jobId: number, topic: string, payload: string, qos: number, retain: boolean) =>
    invoke<void>("mqtt_publish", { jobId, topic, payload, qos, retain }),
  mqttSubscribe: (jobId: number, filters: MqttSub[]) =>
    invoke<void>("mqtt_subscribe", { jobId, filters }),
  mqttUnsubscribe: (jobId: number, filters: string[]) =>
    invoke<void>("mqtt_unsubscribe", { jobId, filters }),
  mqttPublishOnce: (
    config: MqttConfig, topic: string, payload: string, qos: number, retain: boolean
  ) => invoke<string>("mqtt_publish_once", { config, topic, payload, qos, retain }),

  signalsLoad: () => invoke<LibraryFile>("signals_load"),
  signalsSave: (library: Library) => invoke<string>("signals_save", { library }),
};

// Typed event subscription helper.
export function on<T>(event: string, handler: EventCallback<T>): Promise<UnlistenFn> {
  return listen<T>(event, handler);
}

export const EV = {
  oscMessage: "osc://message",
  oscGenTick: "osc://gen-tick",
  burstProgress: "http://burst-progress",
  netsimStat: "netsim://stat",
  stormStat: "storm://stat",
  scanOpen: "scan://open",
  scanProgress: "scan://progress",
  jobEnded: "job://ended",
  emitStat: "broadcast://emit-stat",
  peers: "broadcast://peers",
  inspectBatch: "inspect://batch",
  mqttMessages: "mqtt://messages",
  mqttState: "mqtt://state",
  mqttAck: "mqtt://ack",
} as const;
