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
} as const;
