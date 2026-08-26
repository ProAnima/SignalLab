//! Thin Tauri command layer. Each command validates input and delegates to the
//! engine; long-running ones return a `JobInfo` and stream telemetry via events.

use tauri::{AppHandle, State};

use crate::engine::broadcast::{DiscoveryConfig, EmitConfig, EmitResult};
use crate::engine::http::{BurstConfig, HttpRequest, HttpResponse};
use crate::engine::inspect::{CaptureStats, Frame};
use crate::engine::net::{host_info, HostInfo};
use crate::engine::mqtt::{Cmd, MqttConfig, MqttHub, Sub};
use crate::engine::netsim::ProxyConfig;
use crate::engine::osc::{send_once, start_generator, start_monitor, GenConfig};
use crate::engine::osc_codec::OscArg;
use crate::engine::scan::ScanConfig;
use crate::engine::signals::{Library, LibraryFile};
use crate::engine::storm::StormConfig;
use crate::engine::{
    broadcast, http, mqtt, netsim, scan, signals, storm, Capture, JobInfo, JobRegistry,
};

// ---- host ----------------------------------------------------------------

#[tauri::command]
pub async fn get_host_info() -> HostInfo {
    host_info().await
}

// ---- jobs ----------------------------------------------------------------

#[tauri::command]
pub fn jobs_list(jobs: State<'_, JobRegistry>) -> Vec<JobInfo> {
    jobs.list()
}

#[tauri::command]
pub fn job_stop(jobs: State<'_, JobRegistry>, id: u64) -> bool {
    jobs.stop(id)
}

#[tauri::command]
pub fn jobs_stop_all(jobs: State<'_, JobRegistry>) {
    jobs.stop_all()
}

// ---- OSC -----------------------------------------------------------------

#[tauri::command]
pub async fn osc_send(
    app: AppHandle,
    target: String,
    address: String,
    args: Vec<OscArg>,
) -> Result<usize, String> {
    send_once(app, target, address, args).await
}

#[tauri::command]
pub async fn osc_monitor_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    bind: String,
) -> Result<JobInfo, String> {
    start_monitor(app, jobs.inner().clone(), bind).await
}

#[tauri::command]
pub async fn osc_generator_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    config: GenConfig,
) -> Result<JobInfo, String> {
    start_generator(app, jobs.inner().clone(), config).await
}

// ---- HTTP ----------------------------------------------------------------

#[tauri::command]
pub async fn http_request(app: AppHandle, request: HttpRequest) -> Result<HttpResponse, String> {
    http::request_once(app, request).await
}

#[tauri::command]
pub async fn http_burst_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    config: BurstConfig,
) -> Result<JobInfo, String> {
    http::start_burst(app, jobs.inner().clone(), config).await
}

// ---- Network impairment --------------------------------------------------

#[tauri::command]
pub async fn netsim_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    config: ProxyConfig,
) -> Result<JobInfo, String> {
    netsim::start_proxy(app, jobs.inner().clone(), config).await
}

// ---- Storm ---------------------------------------------------------------

#[tauri::command]
pub async fn storm_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    config: StormConfig,
) -> Result<JobInfo, String> {
    storm::start_storm(app, jobs.inner().clone(), config).await
}

// ---- Scan ----------------------------------------------------------------

#[tauri::command]
pub async fn scan_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    config: ScanConfig,
) -> Result<JobInfo, String> {
    scan::start_scan(app, jobs.inner().clone(), config).await
}

// ---- Broadcast / multicast / discovery ------------------------------------

/// Fan a payload out once — to a list, a broadcast address, a multicast group,
/// or every host in a CIDR block.
#[tauri::command]
pub async fn broadcast_send(app: AppHandle, config: EmitConfig) -> Result<EmitResult, String> {
    broadcast::send_once(app, config).await
}

/// The same fan-out, repeating on an interval as a stoppable job.
#[tauri::command]
pub async fn broadcast_beacon_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    config: EmitConfig,
) -> Result<JobInfo, String> {
    broadcast::start_beacon(app, jobs.inner().clone(), config).await
}

/// Listen for broadcast/multicast traffic, table the peers, optionally answer.
#[tauri::command]
pub async fn discovery_start(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    config: DiscoveryConfig,
) -> Result<JobInfo, String> {
    broadcast::start_discovery(app, jobs.inner().clone(), config).await
}

// ---- Inspector -----------------------------------------------------------

/// Arm or disarm the capture bus. While disarmed no module pays any cost.
#[tauri::command]
pub fn inspect_set_enabled(capture: State<'_, Capture>, enabled: bool) -> CaptureStats {
    capture.set_enabled(enabled);
    capture.stats()
}

#[tauri::command]
pub fn inspect_stats(capture: State<'_, Capture>) -> CaptureStats {
    capture.stats()
}

/// Newest frames still in the ring — lets the Inspector repopulate on mount.
#[tauri::command]
pub fn inspect_snapshot(capture: State<'_, Capture>, limit: usize) -> Vec<Frame> {
    capture.snapshot(limit.clamp(1, crate::engine::inspect::RING_CAPACITY))
}

#[tauri::command]
pub fn inspect_clear(capture: State<'_, Capture>) -> CaptureStats {
    capture.clear();
    capture.stats()
}

/// Dump the ring to `~/Documents/SignalLab/`. Returns the written path.
#[tauri::command]
pub fn inspect_export(capture: State<'_, Capture>, format: String) -> Result<String, String> {
    capture.export(&format)
}

// ---- MQTT ----------------------------------------------------------------

/// Open a connection and hold it as a job. CONNECT/CONNACK completes here, so a
/// refused password is an error on the button rather than a job that dies.
#[tauri::command]
pub async fn mqtt_connect(
    app: AppHandle,
    jobs: State<'_, JobRegistry>,
    hub: State<'_, MqttHub>,
    config: MqttConfig,
) -> Result<JobInfo, String> {
    mqtt::start_client(app, jobs.inner().clone(), hub.inner().clone(), config).await
}

/// Publish on an open connection. An empty payload with `retain` is how a
/// retained value is cleared — the one operation nobody can do by hand.
#[tauri::command]
pub fn mqtt_publish(
    hub: State<'_, MqttHub>,
    job_id: u64,
    topic: String,
    payload: String,
    qos: u8,
    retain: bool,
) -> Result<(), String> {
    hub.send(
        job_id,
        Cmd::Publish {
            topic,
            payload: payload.into_bytes(),
            qos,
            retain,
        },
    )
}

#[tauri::command]
pub fn mqtt_subscribe(
    hub: State<'_, MqttHub>,
    job_id: u64,
    filters: Vec<Sub>,
) -> Result<(), String> {
    if filters.is_empty() {
        return Err("nothing to subscribe to".into());
    }
    hub.send(job_id, Cmd::Subscribe(filters))
}

#[tauri::command]
pub fn mqtt_unsubscribe(
    hub: State<'_, MqttHub>,
    job_id: u64,
    filters: Vec<String>,
) -> Result<(), String> {
    if filters.is_empty() {
        return Err("nothing to unsubscribe from".into());
    }
    hub.send(job_id, Cmd::Unsubscribe(filters))
}

/// Connect, publish, wait for the ack the QoS calls for, disconnect. This is
/// what a library signal uses: it must work with nothing set up.
#[tauri::command]
pub async fn mqtt_publish_once(
    app: AppHandle,
    config: MqttConfig,
    topic: String,
    payload: String,
    qos: u8,
    retain: bool,
) -> Result<String, String> {
    mqtt::publish_once(app, config, topic, payload, qos, retain).await
}

// ---- Signal library ------------------------------------------------------

/// Read the library, seeding it on first run. Firing a signal is not a command:
/// the UI dispatches it through `osc_send` / `broadcast_send` / `http_request`,
/// so there is exactly one send path per transport.
#[tauri::command]
pub fn signals_load() -> Result<LibraryFile, String> {
    signals::load()
}

/// Write the whole library back. Returns the path, for the console line.
#[tauri::command]
pub fn signals_save(library: Library) -> Result<String, String> {
    signals::save(&library)
}
