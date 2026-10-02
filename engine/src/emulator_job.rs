//! An emulator as a job of its own (the Emulators screen, `signallab
//! emulate`, MCP): its socket opened before `start` returns, activity
//! reported every `REPORT_EVERY`, the hub that answers what arrived.

use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use serde::Serialize;
use serde_json::json;

use crate::host::Host;

use super::emulator::Emulator;
use super::emulator_http;
use super::emulator_mqtt;
use super::emulator_net;
use super::emulator_rules::{compile, Rules};
use super::emulator_state::{Context, Counts, Emulation, Exchange, RECENT};
use super::error::{EngineError, EngineResult, Field};
use super::experiment_data as data;
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};

const REPORT_EVERY: Duration = Duration::from_millis(200);

/// The sockets of running emulators, by job, for whoever asks what arrived.
#[derive(Clone, Default)]
pub struct EmulatorHub {
    running: Arc<Mutex<HashMap<u64, Weak<Emulation>>>>,
}

impl EmulatorHub {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&self, id: u64, emulation: &Arc<Emulation>) {
        let mut running = self.running.lock().unwrap();
        running.retain(|_, emulation| emulation.strong_count() > 0);
        running.insert(id, Arc::downgrade(emulation));
    }

    /// The emulator job `id`, while it runs.
    pub fn get(&self, id: u64) -> EngineResult<Arc<Emulation>> {
        self.running.lock().unwrap().get(&id).and_then(Weak::upgrade).ok_or_else(|| EngineError::new("emulator.not_running").with("id", id))
    }
}

/// What `emulator_exchanges` answers: where it listens, its counters, and what arrived.
#[derive(Serialize)]
pub struct Snapshot {
    pub job_id: u64,
    pub name: String,
    pub protocol: &'static str,
    pub local: String,
    pub counts: Counts,
    pub exchanges: Vec<Exchange>,
}

impl EmulatorHub {
    pub fn snapshot(&self, id: u64, after: u64, limit: usize) -> EngineResult<Snapshot> {
        let emulation = self.get(id)?;
        Ok(Snapshot {
            job_id: id,
            name: emulation.name.clone(),
            protocol: emulation.protocol,
            local: emulation.local.to_string(),
            counts: emulation.counts(),
            exchanges: emulation.exchanges(after, limit.clamp(1, RECENT)),
        })
    }
}

/// How a job of its own starts besides its document.
#[derive(Clone, Debug, Default)]
pub struct StartOptions {
    /// Values its templates read as parameters (an experiment's, for its node's *Start now*).
    pub params: BTreeMap<String, String>,
    /// `None`: a fresh seed.
    pub seed: Option<u64>,
    /// The library entry it was started from, for the screen that lists them.
    pub source: Option<String>,
}

/// A port taken a moment ago by an emulator or a run that just stopped is
/// released asynchronously: starting again right away retries briefly.
const RELEASE_RETRIES: u32 = 10;
const RELEASE_PAUSE: Duration = Duration::from_millis(20);

/// The socket an emulator listens on: bound before anything else, so a taken
/// port is an error on the Start button.
pub(crate) enum Socket {
    Tcp(tokio::net::TcpListener),
    Udp(Arc<tokio::net::UdpSocket>),
}

impl Socket {
    pub(crate) async fn open(bind: SocketAddr, over_tcp: bool) -> EngineResult<Socket> {
        let mut attempt = 0;
        loop {
            let bound = if over_tcp {
                tokio::net::TcpListener::bind(bind).await.map(Socket::Tcp)
            } else {
                tokio::net::UdpSocket::bind(bind).await.map(|socket| Socket::Udp(Arc::new(socket)))
            };
            match bound {
                Err(error) if error.kind() == std::io::ErrorKind::AddrInUse && attempt < RELEASE_RETRIES => {
                    attempt += 1;
                    tokio::time::sleep(RELEASE_PAUSE).await;
                }
                Err(error) => return Err(crate::net::bind_error(&bind.to_string(), error).in_field(Field::new("bind"))),
                Ok(socket) => return Ok(socket),
            }
        }
    }

    pub(crate) fn local(&self, bind: SocketAddr) -> SocketAddr {
        match self {
            Socket::Tcp(listener) => listener.local_addr().unwrap_or(bind),
            Socket::Udp(socket) => socket.local_addr().unwrap_or(bind),
        }
    }

    /// Serve until the socket fails; the error says why.
    pub(crate) async fn serve(self, context: Arc<Context>) -> EngineError {
        match (self, &context.compiled.rules) {
            (Socket::Tcp(listener), Rules::Http { .. }) => emulator_http::serve(listener, context).await,
            (Socket::Tcp(listener), Rules::Tcp { .. }) => emulator_net::serve_tcp(listener, context).await,
            (Socket::Tcp(listener), Rules::Mqtt(_)) => emulator_mqtt::serve(listener, context).await,
            (Socket::Udp(socket), _) => emulator_net::serve_datagrams(socket, context).await,
            (Socket::Tcp(_), _) => EngineError::new("emulator.failed"),
        }
    }
}

#[derive(Serialize)]
struct Activity {
    job_id: u64,
    ts: u64,
    counts: Counts,
    exchanges: Vec<Exchange>,
    /// Exchanges that happened but were not sent in this event.
    dropped: u64,
}

fn flush(host: &Host, id: u64, emulation: &Emulation) {
    if let Some((exchanges, dropped)) = emulation.take_fresh() {
        host.emit("emulator://activity", Activity { job_id: id, ts: now_ms(), counts: emulation.counts(), exchanges, dropped });
    }
}

/// Start an emulator as a job: its socket is open when this returns, and
/// `emulator://activity` reports what it answers until it is stopped.
pub async fn start(host: Host, jobs: JobRegistry, hub: EmulatorHub, emulator: Emulator, options: StartOptions) -> EngineResult<JobInfo> {
    data::check_seed(options.seed)?;
    let compiled = compile(&emulator, &options.params)?;
    let socket = Socket::open(compiled.bind, emulator.kind.over_tcp()).await?;
    let local = socket.local(compiled.bind);
    let protocol = emulator.kind.protocol();
    let emulation = Emulation::new(&compiled.name, protocol, local, emulator.kind.rules());
    let id = jobs.next_id();
    let name = compiled.name.clone();
    let mut info = JobInfo::new(id, "emulator", format!("Emulator {name} ({protocol}) on {local}")).with("name", &name).with("protocol", protocol).with("local", local);
    if let Some(source) = &options.source {
        info = info.with("source", source);
    }
    let seed = options.seed.unwrap_or_else(|| rand::random::<u64>() & data::MAX_SEED);
    let context = Context::new(host.clone(), compiled, emulation.clone(), seed, Some(id), None);
    hub.insert(id, &emulation);
    let jobs_cl = jobs.clone();
    let task = tokio::spawn(async move {
        // Stopping the job drops this future and, with the guard, the server
        // and the reporter it spawned.
        let mut guard = TaskGuard::new();
        let serving = tokio::spawn(socket.serve(context));
        guard.watch(serving.abort_handle());
        let reporter = {
            let (host, emulation) = (host.clone(), emulation.clone());
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(REPORT_EVERY).await;
                    flush(&host, id, &emulation);
                }
            })
        };
        guard.watch(reporter.abort_handle());
        let error = serving.await.unwrap_or_else(|panic| EngineError::new("emulator.failed").because(panic));
        drop(guard);
        flush(&host, id, &emulation);
        host.emit("job://ended", json!({ "job_id": id, "kind": "emulator", "error": error }));
        jobs_cl.finish(id);
    });
    jobs.insert(info.clone(), task);
    Ok(info)
}
