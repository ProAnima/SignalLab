//! Signal Lab without a window. The engine runs here; browsers get the same
//! interface as the desktop app, talking to the engine over HTTP and a
//! WebSocket. Design and the rules it follows: docs/delivery.md, section 7.

pub mod auth;
pub mod config;
pub mod events;
mod routes;
mod run;

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::net::TcpListener;
use tokio::sync::watch;

pub use config::{Cli, Command, Config, ConfigError, Options};

/// The data folder must exist and take files before anyone relies on it.
fn prepare_data_dir(config: &Config) -> std::io::Result<()> {
    let Some(dir) = &config.data_dir else { return Ok(()) };
    // A bind-mounted folder owned by another user is the usual cause; say which folder.
    let unwritable = |error: std::io::Error| {
        std::io::Error::new(error.kind(), format!("the data folder {} must be writable by this user: {error}", dir.display()))
    };
    std::fs::create_dir_all(dir).map_err(unwritable)?;
    // Unique per call: two servers in one process must not remove each other's probe.
    let probe = dir.join(format!(".signallab-write-test-{}-{:016x}", std::process::id(), rand::random::<u64>()));
    std::fs::write(&probe, b"").map_err(unwritable)?;
    std::fs::remove_file(&probe).map_err(unwritable)?;
    if !paths::set_data_dir(dir.clone()) && paths::data_dir() != *dir {
        tracing::warn!(requested = %dir.display(), used = %paths::data_dir().display(), "the data folder was already chosen in this process");
    }
    Ok(())
}

/// Serve on `listener` until `shutdown` resolves, then close every page's
/// connection, stop every job and return.
pub async fn serve(config: Config, listener: TcpListener, shutdown: impl Future<Output = ()> + Send + 'static) -> std::io::Result<()> {
    prepare_data_dir(&config)?;
    match &config.ui_dir {
        Some(dir) if dir.join("index.html").is_file() => tracing::info!(ui = %dir.display(), "serving the interface"),
        Some(dir) => tracing::warn!(ui = %dir.display(), "no index.html in the interface folder; only the API is served"),
        None => tracing::warn!("no interface folder found; only the API is served (use --ui-dir)"),
    }
    let (sink, events) = events::Broadcaster::new();
    let host = Host::new(sink, Capture::new());
    let secrets = Arc::new(FileStore::new(Some(config.secrets_dir.clone())));
    let service = Service::new(host, Mode::Server, secrets);
    let (stop, stopping) = watch::channel(false);
    let state = Arc::new(routes::Inner {
        service,
        events,
        auth: auth::Auth::new(config.token.clone(), config.secure_cookie, config.allowed_hosts.clone()),
        ui_dir: config.ui_dir.clone(),
        stopping,
    });
    let app = routes::router(state.clone());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async move {
            shutdown.await;
            tracing::info!("stopping: closing connections and jobs");
            let _ = stop.send(true);
        })
        .await?;
    state.service.jobs().stop_all();
    Ok(())
}
