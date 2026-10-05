//! The desktop shell: a window, and one command that hands everything the
//! interface asks for to the engine's command table. Events travel back as
//! Tauri events. All behaviour lives in `signal-lab-engine` — except what only
//! an installed app does: updating itself from the project's GitHub releases
//! (the updater plugin checks each update's signature against the key in
//! tauri.conf.json before it installs anything), restarting, opening a link in
//! the system's browser or mail program, and showing the documentation built
//! into the app in a window of its own.

use std::sync::Arc;

use serde_json::Value;
use signal_lab_engine::secrets::SystemStore;
use signal_lab_engine::error::EngineError;
use signal_lab_engine::{Capture, EventSink, Failure, Host, Mode, Service};
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Emitter, Manager, State, Url, Webview, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

/// The window of the interface (tauri.conf.json); the documentation's is `DOCS`.
const MAIN: &str = "main";
const DOCS: &str = "docs";

/// Engine events become Tauri events of the same name.
struct TauriEvents(AppHandle);

impl EventSink for TauriEvents {
    fn emit(&self, event: &str, payload: Value) {
        if let Err(error) = self.0.emit(event, payload) {
            tracing::warn!(event, %error, "could not deliver an event to the window");
        }
    }
}

/// Every engine command: `invoke("engine", { command, args })` from the interface —
/// from its window only. The documentation's window shares the app's origin, so it
/// could reach the bridge; it is told the command does not exist.
#[tauri::command]
async fn engine(webview: Webview, service: State<'_, Arc<Service>>, command: String, args: Option<Value>) -> Result<Value, Failure> {
    if webview.label() != MAIN {
        return Err(EngineError::new("command.unknown").with("name", &command).into());
    }
    service.invoke(&command, args.unwrap_or(Value::Null)).await
}

/// The documentation at `page` (`docs/ru/protocols/osc.html#send`), from the pages
/// built into the app, in a window of its own: opened, or brought forward and
/// turned to that page. It may show only the app's own pages; a link out of them
/// (GitHub, a download) opens in the system's browser.
#[tauri::command]
fn open_docs(app: AppHandle, webview: Webview, page: String) -> Result<(), String> {
    if webview.label() != MAIN {
        return Err("only the interface opens the documentation".into());
    }
    if !page.starts_with("docs/") || page.contains("..") || page.contains(':') || page.contains('\\') {
        return Err(format!("{page} is not a page of the documentation"));
    }
    let app_url = webview.url().map_err(|error| error.to_string())?;
    let url = app_url.join(&format!("/{page}")).map_err(|error| error.to_string())?;
    if let Some(window) = app.get_webview_window(DOCS) {
        window.navigate(url).map_err(|error| error.to_string())?;
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }
    let ours = move |url: &Url| url.origin() == app_url.origin();
    let (outside, opened) = (app.clone(), app.clone());
    WebviewWindowBuilder::new(&app, DOCS, WebviewUrl::App(page.into()))
        .title("Signal Lab")
        .inner_size(1180.0, 820.0)
        .min_inner_size(480.0, 360.0)
        .on_navigation(move |url| {
            let here = ours(url);
            if !here {
                in_browser(&outside, url);
            }
            here
        })
        .on_new_window(move |url, _| {
            in_browser(&opened, &url);
            NewWindowResponse::Deny
        })
        .build()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// A link out of the documentation, in the system's browser or mail program.
fn in_browser(app: &AppHandle, url: &Url) {
    if !matches!(url.scheme(), "http" | "https" | "mailto") {
        return;
    }
    if let Err(error) = app.opener().open_url(url.as_str(), None::<&str>) {
        tracing::warn!(%url, %error, "could not open a link of the documentation");
    }
}

/// Whether this build looks for updates on its own: a release build, unless
/// SIGNALLAB_NO_UPDATE_CHECK is set (the end-to-end tour sets it). Asking by
/// hand, from About, always works.
#[tauri::command]
fn update_checks() -> bool {
    !cfg!(debug_assertions) && std::env::var_os("SIGNALLAB_NO_UPDATE_CHECK").is_none()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
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
        .invoke_handler(tauri::generate_handler![engine, update_checks, open_docs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
