//! Thin Tauri command layer. Each command validates input and delegates to the
//! engine; long-running ones return a `JobInfo` and stream telemetry via events.

use tauri::{AppHandle, State};

use crate::engine::broadcast::{DiscoveryConfig, EmitConfig, EmitResult};
use crate::engine::http::{BurstConfig, HttpRequest, HttpResponse};
use crate::engine::inspect::{CaptureStats, Frame};
use crate::engine::net::{host_info, HostInfo};
use crate::engine::netsim::ProxyConfig;
use crate::engine::osc::{send_once, start_generator, start_monitor, GenConfig};
use crate::engine::osc_codec::OscArg;
use crate::engine::scan::ScanConfig;
use crate::engine::storm::StormConfig;
use crate::engine::{broadcast, http, netsim, scan, storm, Capture, JobInfo, JobRegistry};

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
