//! The one command table. The desktop shell and the server both hand a command
//! name and its JSON arguments to [`Service::invoke`] and get back the JSON
//! result or the failure the interface shows — so the two front doors cannot
//! drift apart. Adding a command is one arm in `invoke` and one wrapper in
//! `src/lib/api.ts`.
//!
//! Arguments arrive as the interface writes them (camelCase, `nodeId`), and an
//! argument the command does not know is an error, not something to ignore.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::broadcast::{self, EmitConfig};
use crate::discovery::{self, DiscoveryConfig};
use crate::emulator::{self, DownFault, Emulator};
use crate::emulator_job::{self, EmulatorHub, StartOptions};
use crate::emulator_state::RECENT;
use crate::emulator_files::{self, EmulatorLibrary};
use crate::error::EngineError;
use crate::experiment::{Experiment, Node};
use crate::experiment_data;
use crate::experiment_files;
use crate::experiment_run::{self, RunHandle, RunOptions};
use crate::experiment_validate;
use crate::firewall;
use crate::host::Host;
use crate::cookies::CookieJar;
use crate::http::{self, BurstConfig, HttpRequest};
use crate::inspect::{self, Capture};
use crate::jobs::JobRegistry;
use crate::mqtt::{self, Cmd, MqttConfig, MqttHub, Sub};
use crate::mqtt_dial;
use crate::net;
use crate::netsim::{self, ImpairProfile, ProxyConfig, RelayHub};
use crate::osc::{self, GenConfig};
use crate::osc_codec::OscArg;
use crate::paths;
use crate::scan::{self, ScanConfig};
use crate::secrets::{self, SecretStore};
use crate::signals::{self, Library};
use crate::storm::{self, StormConfig};
use crate::ws::{self, WsConfig, WsExpect, WsHub, WsPayload};

/// How a command failed: always an engine error, a code the interface words
/// in the reader's language. Serialized as the error itself.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Failure {
    Engine(EngineError),
}

impl From<EngineError> for Failure {
    fn from(error: EngineError) -> Self {
        Failure::Engine(error)
    }
}

pub type Reply = Result<Value, Failure>;

/// Where the engine is running, for the interface to adapt to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Desktop,
    Server,
}

/// What `app_info` reports.
#[derive(Clone, Debug, Serialize)]
pub struct AppInfo {
    pub version: &'static str,
    pub mode: Mode,
    /// Secrets can be set and removed from the interface (false: read-only store).
    pub secrets_writable: bool,
    /// Where files are written; on a server this is a folder on that machine.
    pub data_dir: String,
}

/// Everything a command can reach. One per process.
pub struct Service {
    host: Host,
    jobs: JobRegistry,
    mqtt: MqttHub,
    websockets: WsHub,
    /// The HTTP screen's cookie jar: its requests and burst, while it says to keep cookies.
    cookies: Arc<CookieJar>,
    emulators: EmulatorHub,
    relays: RelayHub,
    secrets: Arc<dyn SecretStore>,
    mode: Mode,
    pump: tokio::task::JoinHandle<()>,
}

impl Drop for Service {
    fn drop(&mut self) {
        self.jobs.stop_all();
        self.pump.abort();
    }
}

/// The engine's version, the same as the app's (one version, see docs/delivery.md).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn parse<T: DeserializeOwned>(command: &str, args: Value) -> Result<T, Failure> {
    // No arguments arrive as null from Tauri and as {} from a browser.
    let args = if args.is_null() { Value::Object(Default::default()) } else { args };
    serde_json::from_value(args).map_err(|error| EngineError::new("command.args_invalid").with("name", command).because(error).into())
}

/// The programs whose firewall rules matter here: this one, and the command
/// line installed next to it (`signallab`), which listens for the same gear.
fn firewall_programs() -> Vec<std::path::PathBuf> {
    let program = firewall::this_program();
    let sibling = program.with_file_name(if cfg!(windows) { "signallab.exe" } else { "signallab" });
    let mut programs = vec![program.clone()];
    if sibling != program && sibling.is_file() {
        programs.push(sibling);
    }
    programs
}

fn reply(value: impl Serialize) -> Reply {
    serde_json::to_value(value).map_err(|error| EngineError::new("command.reply_invalid").because(error).into())
}

/// The arguments of one command: a struct named after its fields, read from
/// camelCase JSON, unknown fields refused.
macro_rules! args {
    ($command:expr, $value:expr, { $($field:ident : $ty:ty),* $(,)? }) => {{
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        #[allow(dead_code)]
        struct Args { $($field: $ty),* }
        let parsed: Args = parse($command, $value)?;
        parsed
    }};
}

/// Commands that start a long-running job; a server logs who started them.
pub const JOB_COMMANDS: &[&str] = &[
    "experiment_start", "osc_monitor_start", "osc_generator_start", "http_burst_start", "netsim_start",
    "storm_start", "scan_start", "broadcast_beacon_start", "discovery_start", "mqtt_connect", "ws_connect", "emulator_start",
];

impl Service {
    /// Must be called inside a Tokio runtime: it starts the Inspector's pump.
    pub fn new(host: Host, mode: Mode, secrets: Arc<dyn SecretStore>) -> Self {
        let pump = inspect::spawn_pump(host.clone());
        Service { host, jobs: JobRegistry::new(), mqtt: MqttHub::new(), websockets: WsHub::new(), cookies: CookieJar::new(), emulators: EmulatorHub::new(), relays: RelayHub::new(), secrets, mode, pump }
    }

    pub fn jobs(&self) -> &JobRegistry {
        &self.jobs
    }

    fn capture(&self) -> &Capture {
        self.host.capture()
    }

    /// Start a run and follow it to its end: the run `experiment_start` starts
    /// — the same job, events and report — with a handle that hands over each
    /// step and then the result. For front doors that wait for a run (the
    /// server's `/api/run`, the command line); not a command, since a handle
    /// is not JSON.
    pub async fn run(&self, document: Experiment, options: RunOptions) -> Result<RunHandle, EngineError> {
        experiment_run::start_followed(self.host.clone(), self.jobs.clone(), document, options, self.secrets.as_ref()).await
    }

    pub fn info(&self) -> AppInfo {
        AppInfo {
            version: VERSION,
            mode: self.mode,
            secrets_writable: self.secrets.writable(),
            data_dir: paths::data_dir().display().to_string(),
        }
    }

    /// Run `command` with its JSON arguments.
    pub async fn invoke(&self, command: &str, value: Value) -> Reply {
        let host = || self.host.clone();
        let jobs = || self.jobs.clone();
        let store = self.secrets.as_ref();
        match command {
            "app_info" => reply(self.info()),
            // Whether the firewall lets other machines reach this program (Windows: per program).
            "firewall_status" => reply(firewall::status(firewall_programs()).await?),
            // Asked by a person: the system shows its own administrator prompt. Not on a server,
            // where nobody is at the screen to answer it: its firewall is the host's admin's.
            "firewall_allow" => {
                let a = args!(command, value, { public: bool });
                if self.mode == Mode::Server {
                    return Err(EngineError::new("firewall.server").into());
                }
                firewall::allow(firewall_programs(), a.public).await?;
                reply(firewall::status(firewall_programs()).await?)
            }
            "get_host_info" => reply(net::host_info().await),

            // ---- jobs
            "jobs_list" => reply(self.jobs.list()),
            "job_stop" => {
                let a = args!(command, value, { id: u64 });
                reply(self.jobs.stop(a.id))
            }
            "jobs_stop_all" => {
                self.jobs.stop_all();
                reply(())
            }

            // ---- experiments: failures are EngineErrors
            "experiment_load" => reply(experiment_files::load()?),
            "experiment_save" => {
                let a = args!(command, value, { document: Experiment });
                reply(experiment_files::save(&a.document)?)
            }
            "experiment_parse" => {
                let a = args!(command, value, { text: String });
                reply(experiment_files::parse(&a.text)?)
            }
            "experiment_export" => {
                let a = args!(command, value, { document: Experiment });
                reply(experiment_files::export(&a.document)?)
            }
            // Blocking problems are the error; problems of the other profiles come
            // back on success, so the switcher can warn before anyone switches.
            "experiment_validate" => {
                let a = args!(command, value, { document: Experiment, overrides: Option<BTreeMap<String, String>> });
                experiment_validate::validate_run(&a.document, &a.overrides.unwrap_or_default())?;
                experiment_data::load_secrets(&a.document.nodes, store)?;
                reply(experiment_validate::profile_issues(&a.document))
            }
            "experiment_resolve" => {
                let a = args!(command, value, { document: Experiment, node_id: String, vars: BTreeMap<String, Value> });
                #[derive(Serialize)]
                struct Resolved {
                    node: Node,
                    missing: Vec<String>,
                }
                let node = experiment_data::find_node(&a.document, &a.node_id)?;
                let status = secrets::status(store, &experiment_data::node_secrets(&node.kind))?;
                let stored: Vec<String> = status.into_iter().filter(|(_, stored)| *stored).map(|(name, _)| name).collect();
                let (node, missing) = experiment_data::resolve_node(&a.document, &a.node_id, &a.vars, &stored)?;
                reply(Resolved { node, missing })
            }
            // Send now: an action is sent, a wait listens.
            "experiment_send_node" => {
                let a = args!(command, value, { document: Experiment, node_id: String, vars: BTreeMap<String, Value> });
                reply(experiment_run::send_node(&self.host, &a.document, &a.node_id, &a.vars, store).await?)
            }
            // The wait listeners open before the job exists: a taken port is an
            // error on the Run button, like a validation problem.
            "experiment_start" => {
                let a = args!(command, value, { document: Experiment, overrides: Option<BTreeMap<String, String>>, seed: Option<u64> });
                reply(experiment_run::start(host(), jobs(), a.document, a.overrides.unwrap_or_default(), a.seed, store).await?)
            }

            // ---- secrets: values never leave the engine
            "secret_status" => {
                let a = args!(command, value, { names: Vec<String> });
                reply(secrets::status(store, &a.names)?)
            }
            "secret_set" => {
                let a = args!(command, value, { name: String, value: String });
                secrets::set(store, &a.name, &a.value)?;
                reply(())
            }
            "secret_delete" => {
                let a = args!(command, value, { name: String });
                secrets::delete(store, &a.name)?;
                reply(())
            }

            // ---- OSC
            "osc_send" => {
                let a = args!(command, value, { target: String, address: String, args: Vec<OscArg> });
                reply(osc::send_once(host(), a.target, a.address, a.args).await?)
            }
            "osc_monitor_start" => {
                let a = args!(command, value, { bind: String });
                reply(osc::start_monitor(host(), jobs(), a.bind).await?)
            }
            "osc_generator_start" => {
                let a = args!(command, value, { config: GenConfig });
                reply(osc::start_generator(host(), jobs(), a.config).await?)
            }

            // ---- HTTP
            // `cookies`: send the screen's jar and keep what the answer sets.
            "http_request" => {
                let a = args!(command, value, { request: HttpRequest, cookies: Option<bool> });
                let jar = a.cookies.unwrap_or(false).then(|| self.cookies.clone());
                reply(http::request_once(host(), a.request, jar).await?)
            }
            "http_burst_start" => {
                let a = args!(command, value, { config: BurstConfig });
                reply(http::start_burst(host(), jobs(), a.config, Some(self.cookies.clone())).await?)
            }
            "http_cookies" => reply(self.cookies.list()),
            "http_cookies_clear" => {
                self.cookies.clear();
                reply(())
            }

            // ---- impairment, load, scan
            "netsim_start" => {
                let a = args!(command, value, { config: ProxyConfig });
                reply(netsim::start_proxy(host(), jobs(), self.relays.clone(), a.config).await?)
            }
            // A running relay impairs with another profile from now on, without rebinding.
            "netsim_set_profile" => {
                let a = args!(command, value, { job_id: u64, profile: ImpairProfile });
                self.relays.set(a.job_id, a.profile)?;
                reply(())
            }
            "storm_start" => {
                let a = args!(command, value, { config: StormConfig });
                reply(storm::start_storm(host(), jobs(), a.config).await?)
            }
            "scan_start" => {
                let a = args!(command, value, { config: ScanConfig });
                reply(scan::start_scan(host(), jobs(), a.config).await?)
            }

            // ---- broadcast, multicast, discovery
            "broadcast_send" => {
                let a = args!(command, value, { config: EmitConfig });
                reply(broadcast::send_once(host(), a.config).await?)
            }
            "broadcast_beacon_start" => {
                let a = args!(command, value, { config: EmitConfig });
                reply(broadcast::start_beacon(host(), jobs(), a.config).await?)
            }
            "discovery_start" => {
                let a = args!(command, value, { config: DiscoveryConfig });
                reply(discovery::start_discovery(host(), jobs(), a.config).await?)
            }

            // ---- Inspector
            "inspect_set_enabled" => {
                let a = args!(command, value, { enabled: bool });
                self.capture().set_enabled(a.enabled);
                reply(self.capture().stats())
            }
            "inspect_stats" => reply(self.capture().stats()),
            "inspect_snapshot" => {
                let a = args!(command, value, { limit: usize });
                reply(self.capture().snapshot(a.limit.clamp(1, inspect::RING_CAPACITY)))
            }
            "inspect_clear" => {
                self.capture().clear();
                reply(self.capture().stats())
            }
            "inspect_export" => {
                let a = args!(command, value, { format: String });
                reply(self.capture().export(&a.format)?)
            }

            // ---- MQTT: one live connection per job, plus a one-shot publish
            "mqtt_connect" => {
                let a = args!(command, value, { config: MqttConfig });
                reply(mqtt::start_client(host(), jobs(), self.mqtt.clone(), a.config).await?)
            }
            // An empty payload with `retain` is how a retained value is cleared.
            "mqtt_publish" => {
                let a = args!(command, value, { job_id: u64, topic: String, payload: String, qos: u8, retain: bool });
                let publish = Cmd::Publish { topic: a.topic, payload: a.payload.into_bytes(), qos: a.qos, retain: a.retain };
                self.mqtt.send(a.job_id, publish)?;
                reply(())
            }
            "mqtt_subscribe" => {
                let a = args!(command, value, { job_id: u64, filters: Vec<Sub> });
                if a.filters.is_empty() {
                    return Err(EngineError::new("mqtt.filter_required").into());
                }
                self.mqtt.send(a.job_id, Cmd::Subscribe(a.filters))?;
                reply(())
            }
            "mqtt_unsubscribe" => {
                let a = args!(command, value, { job_id: u64, filters: Vec<String> });
                if a.filters.is_empty() {
                    return Err(EngineError::new("mqtt.filter_required").into());
                }
                self.mqtt.send(a.job_id, Cmd::Unsubscribe(a.filters))?;
                reply(())
            }
            "mqtt_publish_once" => {
                let a = args!(command, value, { config: MqttConfig, topic: String, payload: String, qos: u8, retain: bool });
                reply(mqtt_dial::publish_once(host(), a.config, a.topic, a.payload, a.qos, a.retain).await?)
            }

            // ---- WebSocket: one live connection per job, plus a one-shot exchange
            "ws_connect" => {
                let a = args!(command, value, { config: WsConfig });
                reply(ws::start_client(host(), jobs(), self.websockets.clone(), a.config).await?)
            }
            "ws_send" => {
                let a = args!(command, value, { job_id: u64, message: WsPayload });
                reply(self.websockets.send(a.job_id, &a.message).await?)
            }
            // A close handshake; the job ends when the server has answered.
            "ws_close" => {
                let a = args!(command, value, { job_id: u64, code: Option<u16>, reason: Option<String> });
                reply(self.websockets.close(a.job_id, a.code, a.reason).await?)
            }
            "ws_exchange" => {
                let a = args!(command, value, { config: WsConfig, message: Option<WsPayload>, expect: Option<WsExpect> });
                reply(ws::exchange(&self.host, &a.config, a.message.as_ref(), a.expect.as_ref()).await?)
            }

            // ---- signal library: storage only; firing goes through the commands above
            "signals_load" => reply(signals::load()?),
            "signals_save" => {
                let a = args!(command, value, { library: Library });
                reply(signals::save(&a.library)?)
            }

            // ---- emulators: Signal Lab as the other side
            "emulators_load" => reply(emulator_files::load()?),
            "emulators_save" => {
                let a = args!(command, value, { library: EmulatorLibrary });
                reply(emulator_files::save(&a.library)?)
            }
            // The problems an emulator would be refused for, before anyone presses Start.
            "emulator_check" => {
                let a = args!(command, value, { emulator: Emulator, params: Option<BTreeMap<String, String>> });
                emulator::check(&a.emulator, &a.params.unwrap_or_default())?;
                reply(())
            }
            // `source`: the library entry it came from, kept on the job for the screen.
            "emulator_start" => {
                let a = args!(command, value, { emulator: Emulator, params: Option<BTreeMap<String, String>>, seed: Option<u64>, source: Option<String> });
                let options = StartOptions { params: a.params.unwrap_or_default(), seed: a.seed, source: a.source };
                reply(emulator_job::start(host(), jobs(), self.emulators.clone(), a.emulator, options).await?)
            }
            // What a running emulator received and answered after `after` (a sequence number).
            "emulator_exchanges" => {
                let a = args!(command, value, { job_id: u64, after: Option<u64>, limit: Option<usize> });
                reply(self.emulators.snapshot(a.job_id, a.after.unwrap_or(0), a.limit.unwrap_or(RECENT))?)
            }
            // Pull the plug, or put it back: down until brought up, whatever its outage says.
            "emulator_down" => {
                let a = args!(command, value, { job_id: u64, down: bool, fault: Option<DownFault> });
                self.emulators.force(a.job_id, a.down.then(|| a.fault.unwrap_or_default()))?;
                reply(())
            }

            _ => Err(EngineError::new("command.unknown").with("name", command).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Recorder;
    use crate::secrets::MemoryStore;
    use serde_json::json;

    fn service() -> (Service, Arc<Recorder>) {
        let recorder = Recorder::new();
        let host = Host::new(recorder.clone(), Capture::new());
        (Service::new(host, Mode::Server, Arc::new(MemoryStore::default())), recorder)
    }

    fn code(reply: Reply) -> String {
        match reply {
            Err(Failure::Engine(error)) => error.into_code(),
            Ok(value) => panic!("expected an engine error, got {value}"),
        }
    }

    #[tokio::test]
    async fn commands_answer_and_refuse_what_they_do_not_know() {
        let (service, _) = service();
        let info = service.invoke("app_info", Value::Null).await.unwrap();
        assert_eq!((info["mode"].as_str(), info["version"].as_str(), info["secrets_writable"].as_bool()), (Some("server"), Some(VERSION), Some(true)));
        assert_eq!(service.invoke("jobs_list", json!({})).await.unwrap(), json!([]));
        assert_eq!(code(service.invoke("no_such_command", Value::Null).await), "command.unknown");
        // A misspelled or extra argument is a clear error, not silently ignored.
        assert_eq!(code(service.invoke("job_stop", json!({ "jobid": 1 })).await), "command.args_invalid");
        assert_eq!(code(service.invoke("job_stop", json!({ "id": 1, "force": true })).await), "command.args_invalid");
        assert_eq!(service.invoke("job_stop", json!({ "id": 99 })).await.unwrap(), json!(false));
        // camelCase arguments, as the interface sends them.
        let parsed = service.invoke("experiment_parse", json!({ "text": serde_json::to_string(&crate::experiment::starter()).unwrap() })).await.unwrap();
        assert_eq!(parsed["version"], crate::experiment::VERSION);
        let failure = service.invoke("experiment_parse", json!({ "text": "{" })).await.unwrap_err();
        assert!(matches!(&failure, Failure::Engine(error) if error.is("file.json_invalid")), "{failure:?}");
    }

    /// Every module fails with a code; nothing reaches the interface as a sentence.
    #[tokio::test]
    async fn the_tool_screens_fail_with_codes_too() {
        let (service, recorder) = service();
        let failures = [
            ("mqtt_subscribe", json!({ "jobId": 1, "filters": [] }), "mqtt.filter_required"),
            ("mqtt_unsubscribe", json!({ "jobId": 1, "filters": [] }), "mqtt.filter_required"),
            ("mqtt_publish", json!({ "jobId": 7, "topic": "a", "payload": "", "qos": 0, "retain": false }), "mqtt.not_connected"),
            ("osc_send", json!({ "target": "127.0.0.1", "address": "/x", "args": [] }), "transport.target_invalid"),
            ("osc_monitor_start", json!({ "bind": "nowhere" }), "node.bind_invalid"),
            ("scan_start", json!({ "config": { "host": " ", "port_start": 1, "port_end": 2 } }), "scan.host_required"),
            ("storm_start", json!({ "config": { "target": "x", "protocol": "udp", "size": 1, "rate": 1 } }), "transport.target_invalid"),
            ("inspect_export", json!({ "format": "jsonl" }), "inspect.empty"),
        ];
        for (command, args, expected) in failures {
            let failure = service.invoke(command, args).await.unwrap_err();
            let value = serde_json::to_value(&failure).unwrap();
            assert_eq!(value["code"], expected, "{command}: {value}");
        }
        let mqtt = service.invoke("mqtt_publish", json!({ "jobId": 7, "topic": "a", "payload": "", "qos": 0, "retain": false })).await.unwrap_err();
        assert_eq!(serde_json::to_value(mqtt).unwrap(), json!({ "code": "mqtt.not_connected", "params": { "id": "7" } }));
        let client = json!({ "host": "127.0.0.1", "port": 1, "client_id": " " });
        assert_eq!(code(service.invoke("mqtt_connect", json!({ "config": client })).await), "mqtt.client_id_required");
        assert!(service.jobs().list().is_empty() && recorder.events().is_empty(), "nothing started, nothing emitted");
    }

    #[tokio::test]
    async fn secrets_go_through_the_configured_store() {
        let (service, _) = service();
        service.invoke("secret_set", json!({ "name": "API_TOKEN", "value": "s3cret" })).await.unwrap();
        let status = service.invoke("secret_status", json!({ "names": ["API_TOKEN", "OTHER"] })).await.unwrap();
        assert_eq!(status, json!({ "API_TOKEN": true, "OTHER": false }));
        assert_eq!(code(service.invoke("secret_set", json!({ "name": "bad name", "value": "x" })).await), "secret.name_invalid");
    }

    #[tokio::test]
    async fn the_inspector_is_reached_through_the_host() {
        let (service, _) = service();
        let stats = service.invoke("inspect_set_enabled", json!({ "enabled": true })).await.unwrap();
        assert_eq!(stats["enabled"], true);
        let sent = service.invoke("osc_send", json!({ "target": "127.0.0.1:9", "address": "/x", "args": [] })).await.unwrap();
        assert!(sent.as_u64().unwrap() > 0);
        let frames = service.invoke("inspect_snapshot", json!({ "limit": 10 })).await.unwrap();
        assert_eq!(frames.as_array().unwrap().len(), 1, "the send was captured");
        assert_eq!(service.invoke("inspect_clear", Value::Null).await.unwrap()["buffered"], 0);
    }
}
