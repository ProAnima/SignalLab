mod commands;
mod engine;

use engine::JobRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(JobRegistry::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_host_info,
            commands::jobs_list,
            commands::job_stop,
            commands::jobs_stop_all,
            commands::osc_send,
            commands::osc_monitor_start,
            commands::osc_generator_start,
            commands::http_request,
            commands::http_burst_start,
            commands::netsim_start,
            commands::storm_start,
            commands::scan_start,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
