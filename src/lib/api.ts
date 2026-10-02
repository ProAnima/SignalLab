import { invoke, listen, type UnlistenFn, type EventCallback } from "./transport";

// ---- shared types (mirror the Rust engine) ----

export interface JobInfo {
  id: number;
  kind: string;
  /** English, for logs; show `job.<kind>` filled in with `params` instead. */
  label: string;
  /** The values `job.<kind>` names (target, bind, host, …). */
  params?: Record<string, string>;
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
  /** The failure with every layer of its cause, when there was no response. */
  error: string | null;
  /** What kind of failure `error` is; the screen shows `err.transport.<cause>`. */
  cause?: TransportCause | null;
}

/** Why a network operation failed (`engine/transport.rs`). */
export type TransportCause = "refused" | "timeout" | "dns" | "unreachable" | "reset" | "address_in_use"
  | "address_unavailable" | "denied" | "tls" | "target_invalid" | "failed";

/**
 * The engine's one error shape (`engine/error.rs`): a code the interface
 * translates as `err.<code>`, values for its message, where it happened, and
 * the system's own words as `detail`. Every command rejects with one;
 * `lib/errors.ts` renders it.
 */
export interface EngineError {
  code: string;
  params?: Record<string, string>;
  node?: string;
  field?: { key: string; index?: number };
  /** Technical text from the system, a parser or a library. */
  detail?: string;
}

// ---- experiments ----

export type ExtractFrom = "json" | "header" | "status" | "body" | "regex";
export type CompareOp = "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "contains" | "matches" | "empty" | "not_empty";

/** Try a failed send or wait again: attempts in all, the pause before the second, fixed or doubling. */
export interface Retry { attempts: number; delay_ms: number; backoff?: "fixed" | "exponential" }
/**
 * Send an action more than once: `count` sends, or as many as fit in
 * `duration_ms`, `interval_ms` apart plus up to `jitter_ms` (from the seed).
 */
export interface Repeat { until: "count" | "duration"; count: number; duration_ms: number; interval_ms: number; jitter_ms: number }
/** The answer an OSC message waits for in the same step (the matching of Wait for OSC). */
export interface OscReply { bind: string; address: string; args: ArgRule[]; timeout_ms: number; variable: string }
/** The answer a datagram waits for in the same step (the matching of Wait for UDP). */
export interface UdpReply { bind: string; mode: UdpMode; pattern: string; timeout_ms: number; variable: string }

export type ExperimentNode = {
  id: string; x: number; y: number;
  /** Actions and waits only. */
  retry?: Retry;
  /** Actions only. */
  repeat?: Repeat;
} & (
  | { type: "start" | "end" | "fork" | "join" }
  | { type: "delay"; ms: number }
  | { type: "log"; message: string }
  | { type: "http"; request: HttpRequest }
  | { type: "tcp"; host: string; port: number; payload: string; timeout_ms: number }
  | { type: "assert_status"; status: number }
  | { type: "assert_body"; contains: string }
  | { type: "assert_header"; name: string; contains: string }
  | { type: "assert_latency"; max_ms: number }
  | { type: "mqtt"; host: string; port: number; topic: string; payload: string; qos: number; retain: boolean }
  | { type: "branch_status"; status: number }
  | { type: "osc"; target: string; address: string; args: OscArg[]; reply?: OscReply }
  | { type: "udp"; target: string; text: string; reply?: UdpReply }
  | { type: "extract"; variable: string; from: ExtractFrom; expr: string }
  | { type: "loop"; max: number; until?: LoopUntil | null }
  | { type: "assert_value"; value: string; op: CompareOp; expected: string }
  | { type: "branch_value"; value: string; op: CompareOp; expected: string }
  | { type: "wait_osc"; bind: string; address: string; args: ArgRule[]; timeout_ms: number; variable: string }
  | { type: "wait_udp"; bind: string; mode: UdpMode; pattern: string; timeout_ms: number; variable: string }
  /** `topic` is a subscription filter (`+`, `#`); broker and topic may use parameters only. */
  | { type: "wait_mqtt"; host: string; port: number; topic: string; mode: UdpMode; pattern: string; timeout_ms: number; variable: string }
);

/** `args[index] <op> value` on a received OSC message; the value is a template. */
export interface ArgRule { index: number; op: CompareOp; value: string }
export type UdpMode = "any" | "contains" | "regex" | "hex";
export type ExperimentPort = "next" | "yes" | "no" | "branch1" | "branch2" | "matched" | "timeout" | "body" | "done" | "limit";
/** A Loop's exit condition, checked after each iteration: the comparison of Check value. */
export interface LoopUntil { value: string; op: CompareOp; expected: string }

export interface ExperimentParam { name: string; value: string }
/** What Send now reports; the resolved request never comes back. */
export interface NodeSendResult {
  detail: string;
  response: HttpResponse | null;
  /** A wait's reply, or the values the Extract nodes after a request take from its response. */
  vars: Record<string, unknown>;
}
/** A named set of parameter values; parameters it does not set keep their default. */
export interface ExperimentProfile { name: string; values: Record<string, string> }
/** A profile (null: the defaults) that would fail validation if it were active. */
export interface ProfileIssue { profile: string | null; error: EngineError }

export interface Experiment {
  version: number;
  name: string;
  /** Default values. */
  params: ExperimentParam[];
  profiles: ExperimentProfile[];
  /** The active profile; null runs with the defaults. */
  profile: string | null;
  /** null: a fresh seed for every run. */
  seed: number | null;
  nodes: ExperimentNode[];
  edges: { from: string; to: string; port?: ExperimentPort }[];
}

export interface ExperimentStep {
  job_id: number; ts: number; node_id: string;
  /** `retry`: an attempt failed (`error` says why) and the step runs again after a pause. */
  state: "running" | "passed" | "failed" | "retry" | "repeating"; detail: string;
  message_key?: string | null; message_params?: Record<string, string | number>;
  /** Variables this step wrote. */
  vars?: Record<string, unknown>;
  /** The Inspector frame of the message a wait matched (when capture was armed). */
  frame?: number;
  /** Why the step failed. */
  error?: EngineError;
}

export interface ExperimentEnded {
  job_id: number; kind: string; seed: number; profile: string | null; overridden: boolean; error: EngineError | null;
  report_path: string | null; report_error: EngineError | null;
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
  /** Why the first failed packet failed, when one did. */
  error?: EngineError;
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
  /** Every folder, as paths ("API/Auth"), empty ones included; a signal's `group` is a folder too. */
  folders?: string[];
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
  /** A packet that did not decode (`osc.packet_malformed`, the parser's words as detail). */
  error: EngineError | null;
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
/** Experiments end with an `EngineError`; the other jobs with text. */
export interface JobEnded { job_id: number; kind: string; error: EngineError | null; }
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
  error: EngineError | null;
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

/** Where the engine runs, for the interface to adapt to (engine/src/service.rs). */
export interface AppInfo {
  version: string;
  mode: "desktop" | "server";
  /** False on a server: secrets are read from its environment or files. */
  secrets_writable: boolean;
  /** Where files are written — on a server, a folder on that machine. */
  data_dir: string;
}

// ---- command wrappers ----

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  hostInfo: () => invoke<HostInfo>("get_host_info"),
  jobsList: () => invoke<JobInfo[]>("jobs_list"),
  jobStop: (id: number) => invoke<boolean>("job_stop", { id }),
  jobsStopAll: () => invoke<void>("jobs_stop_all"),
  experimentLoad: () => invoke<Experiment>("experiment_load"),
  experimentSave: (document: Experiment) => invoke<string>("experiment_save", { document }),
  experimentParse: (text: string) => invoke<Experiment>("experiment_parse", { text }),
  experimentExport: (document: Experiment) => invoke<string>("experiment_export", { document }),
  /** Throws the blocking problem; otherwise returns what the other profiles would fail on. */
  experimentValidate: (document: Experiment, overrides?: Record<string, string>) =>
    invoke<ProfileIssue[]>("experiment_validate", { document, overrides: overrides ?? null }),
  /** `overrides` and `seed` (Run with…) apply to this run only. */
  experimentStart: (document: Experiment, overrides?: Record<string, string>, seed?: number | null) =>
    invoke<JobInfo>("experiment_start", { document, overrides: overrides ?? null, seed: seed ?? null }),
  /** A node with its templates resolved; names without a value stay as written and are listed. */
  experimentResolve: (document: Experiment, nodeId: string, vars: Record<string, unknown>) =>
    invoke<{ node: ExperimentNode; missing: string[] }>("experiment_resolve", { document, nodeId, vars }),
  /** Send one node through the runner's code; secret values are masked in what comes back. */
  experimentSendNode: (document: Experiment, nodeId: string, vars: Record<string, unknown>) =>
    invoke<NodeSendResult>("experiment_send_node", { document, nodeId, vars }),
  /** Which secret names are stored on this machine. Values are never returned. */
  secretStatus: (names: string[]) => invoke<Record<string, boolean>>("secret_status", { names }),
  secretSet: (name: string, value: string) => invoke<void>("secret_set", { name, value }),
  secretDelete: (name: string) => invoke<void>("secret_delete", { name }),

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
  experimentStep: "experiment://step",
  experimentEnded: "experiment://ended",
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
  /** Server only: events this page missed because it fell behind. */
  serverLagged: "server://lagged",
} as const;
