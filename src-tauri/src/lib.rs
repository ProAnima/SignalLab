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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![engine])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
