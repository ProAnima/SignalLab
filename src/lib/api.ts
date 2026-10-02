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

/** Credentials a request carries (engine/src/http_auth.rs); its Authorization header is never reported. */
export type HttpAuth =
  | { scheme: "none" }
  | { scheme: "basic" | "digest"; username: string; password: string }
  | { scheme: "bearer"; token: string };

export interface HttpRequest {
  method: string;
  url: string;
  headers: [string, string][];
  body: string | null;
  timeout_ms: number;
  /** Absent: none. Digest answers the server's 401 challenge (MD5, SHA-256). */
  auth?: HttpAuth;
}

/** One cookie of a jar (engine/src/cookies.rs). */
export interface CookieInfo {
  name: string; value: string; domain: string;
  /** No Domain attribute: only the host that set it gets it back. */
  host_only: boolean;
  path: string;
  /** Unix seconds; null: a session cookie. */
  expires: number | null;
  secure: boolean; http_only: boolean; same_site: string | null;
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
  /** A Digest request: its 401 challenge answered (`challenged`), or why it could not be. */
  digest?: { challenged: boolean; error: EngineError | null } | null;
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
  /** Serves for the whole run: opened before the first step, closed with the run. */
  | { type: "emulator"; emulator: Emulator }
  /** A request to the run's HTTP emulator on `bind`, or to a listener of the run's own that answers 204. */
  | { type: "wait_http"; bind: string; method: string; path: string; when: Condition[]; timeout_ms: number; variable: string }
  /** A UDP impairment relay for the whole run: `listen` → `target`; addresses take parameters only. */
  | { type: "impairment"; listen: string; target: string; profile: ImpairProfile }
  /** The run's Impairment `relay` (a node id) impairs with `profile` from now on. */
  | { type: "impairment_change"; relay: string; profile: ImpairProfile }
  /** The run's Emulator `emulator` (a node id) goes down, or comes back up. */
  | { type: "emulator_state"; emulator: string; down: boolean; fault: DownFault }
  /** A WebSocket for the rest of the run; the nodes below name it by this node's id. */
  | { type: "ws_connect"; url: string; headers: [string, string][]; protocols: string[]; timeout_ms: number }
  /** `binary`: `text` is hex bytes, sent as a binary message. */
  | { type: "ws_send"; connection: string; text: string; binary: boolean }
  | { type: "wait_ws"; connection: string; mode: UdpMode; pattern: string; timeout_ms: number; variable: string }
  | { type: "ws_close"; connection: string; code: number; reason: string }
);

// ---- emulators (engine/src/emulator.rs) ----

/** `<on> <name> <op> <value>` about an HTTP request; the value may use parameters. */
export interface Condition { on: "header" | "query" | "body" | "json"; name: string; op: CompareOp; value: string }
export type Fault = "none" | "timeout" | "reset" | "malformed";
export type ResponseOrder = "sequence" | "cycle" | "random";
export interface EmulatorResponse {
  status: number;
  headers: [string, string][];
  /** A template read with what arrived: `{{request.params.id}}`. */
  body: string;
  delay_ms: number;
  jitter_ms: number;
  fault: Fault;
  /** Its share when the route answers at random. */
  weight: number;
}
export interface Route { method: string; path: string; when: Condition[]; order: ResponseOrder; responses: EmulatorResponse[] }
export type ArgType = "int" | "float" | "str" | "long" | "double" | "bool" | "blob" | "nil";
/** A reply argument: its type, and a template for its value. */
export interface ArgOut { type: ArgType; value: string }
export interface OscOut { address: string; args: ArgOut[] }
export interface OscRule { address: string; args: ArgRule[]; reply: OscOut | null; to: string; delay_ms: number; jitter_ms: number }
export interface UdpRule { mode: UdpMode; pattern: string; reply: RawPayload | null; to: string; delay_ms: number; jitter_ms: number }
export interface TcpRule { mode: UdpMode; pattern: string; reply: RawPayload | null; close: boolean; delay_ms: number; jitter_ms: number }
export type Delimiter = "lf" | "crlf" | "cr" | "none";
export interface MqttOut { topic: string; payload: string; qos: number; retain: boolean }
/** On a message to a topic the filter matches, with a matching payload, publish `reply`. */
export interface MqttRule { topic: string; mode: UdpMode; pattern: string; reply: MqttOut | null; delay_ms: number; jitter_ms: number }
export interface MqttRetained { topic: string; payload: string; qos: number }
export type DownFault = "unavailable" | "reset" | "timeout";
/** Up for `up_ms`, then down for `down_ms`, from the start on. */
export interface Outage { up_ms: number; down_ms: number; fault: DownFault }
export type EmulatorProtocol = "http" | "osc" | "udp" | "tcp" | "mqtt";
export type Emulator = { name: string; bind: string; outage?: Outage | null } & (
  | { protocol: "http"; routes: Route[]; fallback: EmulatorResponse | null }
  | { protocol: "osc"; rules: OscRule[] }
  | { protocol: "udp"; rules: UdpRule[] }
  | { protocol: "tcp"; delimiter: Delimiter; greeting: string; rules: TcpRule[] }
  | { protocol: "mqtt"; username: string; password: string; retained: MqttRetained[]; rules: MqttRule[] }
);
export interface StoredEmulator { id: string; note: string; emulator: Emulator }
export interface EmulatorLibrary { version: number; emulators: StoredEmulator[] }
export interface EmulatorLibraryFile { path: string; library: EmulatorLibrary; seeded: boolean }
export interface EmulatorCounts { total: number; unmatched: number; failed: number; down: number; hits: number[];
  /** Messages a broker could not hand to a client too far behind; absent while none were. */
  missed?: number }
/** One request (message, line) an emulator received, and what became of it. */
export interface Exchange {
  seq: number;
  ts: number;
  from: string;
  request: string;
  /** 1-based; absent when no rule took it. */
  rule?: number;
  reply: string;
  status?: number;
  fault?: Fault;
  ms: number;
  error?: EngineError;
  frame?: number;
  /** It arrived while the emulator was down: no rule was asked. */
  down?: boolean;
  /** What arrived, as templates read it (`emulator_exchanges` only). */
  data?: unknown;
}
/** `forced`: taken down by a person or a run, with what HTTP meets meanwhile. */
export interface EmulatorSnapshot { job_id: number; name: string; protocol: EmulatorProtocol; local: string; counts: EmulatorCounts; forced?: DownFault; exchanges: Exchange[] }
export interface EmulatorActivity { job_id: number; ts: number; counts: EmulatorCounts; forced?: DownFault; exchanges: Exchange[]; dropped: number }

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
  /** Send back what servers set with Set-Cookie, as a browser does. Absent: on (off for files from before version 8). */
  cookies?: boolean;
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
  /** At most this many in flight. */
  concurrency: number;
  total: number;
  duration_s: number;
  /** Requests started per second on a fixed schedule; 0: each worker sends again on its answer. */
  rate?: number;
  /** Use the screen's cookie jar, as its single requests do. */
  cookies?: boolean;
}

/** What an impairment relay does to every packet, both ways (engine/src/netsim.rs). Probabilities are 0..1. */
export interface ImpairProfile {
  /** A preset's key (`4g`) or a person's own label; empty: described by its values. */
  name?: string;
  latency_ms: number;
  jitter_ms: number;
  loss: number;
  duplicate: number;
  corrupt: number;
  reorder?: number;
  /** 0: no limit. */
  rate_kbps?: number;
  /** Bursts of loss: the chance a packet starts one, and how many packets one lasts on average. */
  burst_start?: number;
  burst_length?: number;
  offline?: boolean;
}

export interface ProxyConfig {
  listen: string;
  target: string;
  profile: ImpairProfile;
  seed?: number | null;
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

// ---- websocket (engine/src/ws.rs) ----

export interface WsConfig {
  /** ws:// or wss:// */
  url: string;
  headers: [string, string][];
  /** Subprotocols to offer, in order of preference. */
  protocols: string[];
  timeout_ms: number;
}
/** Text, or bytes as hex (a binary message). */
export type WsPayload = { text: string } | { hex: string };
export interface WsHandshake { url: string; peer: string; local: string; protocol: string | null; ms: number }
export interface WsClosed { code: number; reason: string; by: "client" | "server" | "lost"; error: EngineError | null }
export interface WsMessage {
  ts: number;
  dir: "rx" | "tx";
  kind: "text" | "binary";
  /** UTF-8 (lossy for binary, which has `hex` too), cut at 64 KB: then `truncated`. */
  text: string;
  hex: string | null;
  bytes: number;
  truncated: boolean;
}
export interface WsBatch { job_id: number; ts: number; messages: WsMessage[]; dropped: number }
export interface WsStateEvent { job_id: number; ts: number; state: "connected" | "closed"; handshake: WsHandshake; closed: WsClosed | null }

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
  /** Due while every worker was busy, and skipped (a paced burst). */
  missed: number;
  /** Over the last report's window; in the last report (`done`), over the whole burst. */
  rps: number; last_latency_ms: number; min_latency_ms: number;
  max_latency_ms: number; avg_latency_ms: number;
  /** Over every request so far, failures included. */
  p50_ms: number; p90_ms: number; p95_ms: number; p99_ms: number;
  done: boolean;
}
export interface ProxyStat {
  job_id: number; ts: number; received: number; forwarded: number; dropped: number; throttled: number;
  duplicated: number; corrupted: number; reordered: number; bytes: number;
  /** The profile it impairs with now, as the timeline names it. */
  profile: string;
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
/** What the firewall does with what other machines send to this program (`engine/firewall.rs`). */
export interface FirewallStatus {
  /** There is a per-program firewall here (Windows). */
  applies: boolean;
  program: string;
  /** On for the network the machine is on now. */
  enabled: boolean;
  /** "domain", "private", "public". */
  networks: string[];
  allowed: boolean;
  /** A rule blocks it (a "Cancel" at the system's prompt); it wins over an allow. */
  blocked: boolean;
  rules: number;
}

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
  firewallStatus: () => invoke<FirewallStatus>("firewall_status"),
  /** The system asks for administrator rights first; the new status comes back. */
  firewallAllow: (alsoPublic: boolean) => invoke<FirewallStatus>("firewall_allow", { public: alsoPublic }),
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

  /** `cookies`: send the screen's jar and keep what the answer sets. */
  httpRequest: (request: HttpRequest, cookies = false) => invoke<HttpResponse>("http_request", { request, cookies }),
  httpCookies: () => invoke<CookieInfo[]>("http_cookies"),
  httpCookiesClear: () => invoke<null>("http_cookies_clear"),
  httpBurstStart: (config: BurstConfig) => invoke<JobInfo>("http_burst_start", { config }),

  netsimStart: (config: ProxyConfig) => invoke<JobInfo>("netsim_start", { config }),
  /** A running relay impairs with `profile` from now on, without rebinding. */
  netsimSetProfile: (jobId: number, profile: ImpairProfile) => invoke<null>("netsim_set_profile", { jobId, profile }),
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

  /** A connection held open as a job; events `ws://state`, `ws://messages`. */
  wsConnect: (config: WsConfig) => invoke<JobInfo>("ws_connect", { config }),
  wsSend: (jobId: number, message: WsPayload) => invoke<number>("ws_send", { jobId, message }),
  /** A close handshake; the job ends once the server has answered. */
  wsClose: (jobId: number, code = 1000, reason = "") => invoke<WsClosed>("ws_close", { jobId, code, reason }),
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

  emulatorsLoad: () => invoke<EmulatorLibraryFile>("emulators_load"),
  emulatorsSave: (library: EmulatorLibrary) => invoke<string>("emulators_save", { library }),
  /** Throws what Start would be refused for; `params` are the values its templates may read. */
  emulatorCheck: (emulator: Emulator, params?: Record<string, string>) => invoke<void>("emulator_check", { emulator, params: params ?? null }),
  /** `source`: the library entry it comes from, kept on the job (`params.source`). */
  emulatorStart: (emulator: Emulator, source: string | null, params?: Record<string, string>) =>
    invoke<JobInfo>("emulator_start", { emulator, params: params ?? null, seed: null, source }),
  emulatorExchanges: (jobId: number, after = 0) => invoke<EmulatorSnapshot>("emulator_exchanges", { jobId, after, limit: null }),
  /** Take a running emulator down (HTTP meets `fault`) until brought up, whatever its outage says. */
  emulatorDown: (jobId: number, down: boolean, fault: DownFault = "unavailable") => invoke<null>("emulator_down", { jobId, down, fault }),
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
  wsMessages: "ws://messages",
  wsState: "ws://state",
  mqttMessages: "mqtt://messages",
  mqttState: "mqtt://state",
  mqttAck: "mqtt://ack",
  emulatorActivity: "emulator://activity",
  /** Server only: events this page missed because it fell behind. */
  serverLagged: "server://lagged",
} as const;
