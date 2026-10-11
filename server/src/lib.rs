//! Signal Lab without a window. The engine runs here; browsers get the same
//! interface as the desktop app, talking to the engine over HTTP and a
//! WebSocket. Design and the rules it follows: docs/develop/delivery.md, section 7.

pub mod auth;
pub mod config;
pub mod events;
mod routes;
mod run;

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ConnectInfo;
use hyper::server::conn::http1;
use hyper_util::rt::{TokioIo, TokioTimer};
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinSet;
use tower::ServiceExt;

pub use config::{Cli, Command, Config, ConfigError, Options};

/// A client must send a request's head within this (also how long an idle
/// connection is kept); one that does not is disconnected — a page sends its
/// head at once, a slow-loris client never finishes it.
pub const HEAD_TIMEOUT: Duration = Duration::from_secs(30);
/// Connections served at once; more are closed as they arrive. A browser
/// keeps a handful; this bounds what a flood of idle ones can hold.
pub const MAX_CONNECTIONS: usize = 1024;
/// On stopping, how long requests under way get to finish.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

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
    let mut shutdown = std::pin::pin!(shutdown);
    // Owned here: when serving ends, so does every connection still open.
    let mut connections = JoinSet::new();
    loop {
        while connections.try_join_next().is_some() {}
        let (stream, peer) = tokio::select! {
            _ = &mut shutdown => break,
            accepted = listener.accept() => match accepted {
                Ok(accepted) => accepted,
                // A client that gave up during the handshake, a moment without file handles: carry on.
                Err(error) => {
                    tracing::debug!(%error, "accept failed");
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            },
        };
        if connections.len() >= MAX_CONNECTIONS {
            tracing::warn!(client = %peer, max = MAX_CONNECTIONS, "too many connections: closed as it arrived");
            drop(stream);
            continue;
        }
        let _ = stream.set_nodelay(true);
        let router = app.clone();
        // What `ConnectInfo` reads: who is asking (the server logs who started a job).
        let service = hyper::service::service_fn(move |mut request: hyper::Request<hyper::body::Incoming>| {
            request.extensions_mut().insert(ConnectInfo(peer));
            router.clone().oneshot(request)
        });
        let mut stopping = stop.subscribe();
        connections.spawn(async move {
            let connection = http1::Builder::new()
                .timer(TokioTimer::new())
                .header_read_timeout(HEAD_TIMEOUT)
                .serve_connection(TokioIo::new(stream), service)
                .with_upgrades();
            let mut connection = std::pin::pin!(connection);
            tokio::select! {
                _ = connection.as_mut() => return,
                _ = stopping.wait_for(|stop| *stop) => connection.as_mut().graceful_shutdown(),
            }
            let _ = connection.await;
        });
    }
    tracing::info!("stopping: closing connections and jobs");
    drop(listener);
    let _ = stop.send(true);
    // Requests under way get a moment to finish (a followed run says it was stopped);
    // a client that would hold the server open past it is cut off.
    if tokio::time::timeout(DRAIN_TIMEOUT, async { while connections.join_next().await.is_some() {} }).await.is_err() {
        tracing::debug!(open = connections.len(), "connections cut off on stopping");
        connections.abort_all();
    }
    state.service.jobs().stop_all();
    Ok(())
}
