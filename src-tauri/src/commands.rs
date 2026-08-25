//! Thin Tauri command layer. Each command validates input and delegates to the
//! engine; long-running ones return a `JobInfo` and stream telemetry via events.

use tauri::{AppHandle, State};

use crate::engine::http::{BurstConfig, HttpRequest, HttpResponse};
use crate::engine::net::{host_info, HostInfo};
use crate::engine::netsim::ProxyConfig;
use crate::engine::osc::{GenConfig, send_once, start_generator, start_monitor};
use crate::engine::osc_codec::OscArg;
use crate::engine::scan::ScanConfig;
use crate::engine::storm::StormConfig;
use crate::engine::{http, netsim, scan, storm, JobInfo, JobRegistry};

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
    target: String,
    address: String,
    args: Vec<OscArg>,
) -> Result<usize, String> {
    send_once(target, address, args).await
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
pub async fn http_request(request: HttpRequest) -> Result<HttpResponse, String> {
    http::request_once(request).await
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
