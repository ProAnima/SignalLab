use std::io::IsTerminal;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use signal_lab_server::config::{new_token, LogFormat};
use signal_lab_server::{serve, Cli, Command};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Token) => {
            println!("{}", new_token());
            return ExitCode::SUCCESS;
        }
        Some(Command::Healthcheck) => return healthcheck(cli.options.listen).await,
        None => {}
    }

    init_logging(&cli.options.log, cli.options.log_format);

    let config = match cli.options.resolve() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("signal-lab-server: {error}");
            return ExitCode::from(2);
        }
    };
    let listener = match TcpListener::bind(config.listen).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("signal-lab-server: cannot listen on {}: {error}", config.listen);
            return ExitCode::FAILURE;
        }
    };
    let access = if config.token.is_some() { "token required" } else { "this machine only, no token" };
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Signal Lab server on http://{} ({access})", config.listen);
    if let Some(dir) = &config.data_dir {
        tracing::info!(data = %dir.display(), "data folder");
    }
    match serve(config, listener, shutdown_signal()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("signal-lab-server: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Colours only on a terminal (and never with NO_COLOR): `docker logs` and log
/// collectors get plain lines, or JSON for machines.
fn init_logging(filter: &str, format: LogFormat) {
    let filter = tracing_subscriber::EnvFilter::try_new(filter).unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter).with_target(false);
    match format {
        LogFormat::Json => builder.json().init(),
        LogFormat::Text => {
            let colour = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty());
            builder.with_ansi(colour).init()
        }
    }
}

/// Ctrl+C everywhere; SIGTERM too on Unix, which is how a container is stopped.
async fn shutdown_signal() {
    let interrupt = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = interrupt => {}
        _ = terminate => {}
    }
}

/// `GET /api/health` on this machine; exit 0 when the server answers.
async fn healthcheck(listen: SocketAddr) -> ExitCode {
    // A server listening on every address is reached through loopback.
    let ip = match listen.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        ip => ip,
    };
    let url = format!("http://{}/api/health", SocketAddr::new(ip, listen.port()));
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(3)).build() {
        Ok(client) => client,
        Err(_) => return ExitCode::FAILURE,
    };
    // Under a loopback name, which --allowed-host never narrows away, even when
    // the server listens on one specific address.
    match client.get(&url).header(reqwest::header::HOST, format!("localhost:{}", listen.port())).send().await {
        Ok(response) if response.status().is_success() => ExitCode::SUCCESS,
        Ok(response) => {
            eprintln!("{url}: {}", response.status());
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("{url}: {error}");
            ExitCode::FAILURE
        }
    }
}
