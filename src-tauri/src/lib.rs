mod commands;
mod engine;

use engine::{Capture, JobRegistry, MqttHub};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let capture = Capture::new();
    let pump_capture = capture.clone();

    tauri::Builder::default()
        .manage(JobRegistry::new())
        .manage(capture)
        .manage(MqttHub::new())
        .setup(move |app| {
            // A single pump ships capture batches to the UI for the whole app.
            engine::inspect::spawn_pump(app.handle().clone(), pump_capture);
            Ok(())
        })
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
            commands::broadcast_send,
            commands::broadcast_beacon_start,
            commands::discovery_start,
            commands::inspect_set_enabled,
            commands::inspect_stats,
            commands::inspect_snapshot,
            commands::inspect_clear,
            commands::inspect_export,
            commands::signals_load,
            commands::signals_save,
            commands::mqtt_connect,
            commands::mqtt_publish,
            commands::mqtt_subscribe,
            commands::mqtt_unsubscribe,
            commands::mqtt_publish_once,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
