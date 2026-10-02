//! The desktop shell: a window, and one command that hands everything the
//! interface asks for to the engine's command table. Events travel back as
//! Tauri events. All behaviour lives in `signal-lab-engine`.

use std::sync::Arc;

use serde_json::Value;
use signal_lab_engine::secrets::SystemStore;
use signal_lab_engine::{Capture, EventSink, Failure, Host, Mode, Service};
use tauri::{AppHandle, Emitter, Manager, State};

/// Engine events become Tauri events of the same name.
struct TauriEvents(AppHandle);

impl EventSink for TauriEvents {
    fn emit(&self, event: &str, payload: Value) {
        if let Err(error) = self.0.emit(event, payload) {
            tracing::warn!(event, %error, "could not deliver an event to the window");
        }
    }
}

/// Every engine command: `invoke("engine", { command, args })` from the interface.
#[tauri::command]
async fn engine(service: State<'_, Arc<Service>>, command: String, args: Option<Value>) -> Result<Value, Failure> {
    service.invoke(&command, args.unwrap_or(Value::Null)).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let host = Host::new(Arc::new(TauriEvents(app.handle().clone())), Capture::new());
            // The service starts the Inspector's pump, which needs the runtime.
            let service = tauri::async_runtime::block_on(async move { Service::new(host, Mode::Desktop, Arc::new(SystemStore)) });
            app.manage(Arc::new(service));
            // The window of tauri.conf.json, made here (it has "create": false) so a debug
            // build can open the webview's DevTools for the end-to-end tour — only when the
            // tour asks, and never in a release build.
            let config = app.config().app.windows.first().cloned().ok_or("tauri.conf.json has no window")?;
            #[allow(unused_mut)]
            let mut window = tauri::WebviewWindowBuilder::from_config(app.handle(), &config)?;
            #[cfg(all(windows, debug_assertions))]
            if let Ok(port) = std::env::var("SIGNALLAB_E2E_DEVTOOLS_PORT") {
                // Tauri's own defaults, which giving any arguments replaces.
                window = window.additional_browser_args(&format!("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --remote-debugging-port={port}"));
            }
            window.build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![engine])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
