//! Executing an experiment: what a run starts with, how it is followed and
//! how it ends, and *Send now*. How a run moves from step to step is
//! `experiment_flow`, what each step does `experiment_steps`, the report file
//! `experiment_report`.
//!
//! A run is started one way and can be watched two ways: through the events
//! every host receives (`experiment://step`, `experiment://ended` — the app's
//! timeline), and through the [`RunHandle`] that [`start_followed`] returns,
//! which hands its holder each step and then the result — what the server's
//! `/api/run` and the command line wait for. Both are the same run.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use crate::host::Host;

use super::emulator_run;
use super::emulator_state::{Emulation, EmulatorSummary};
use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Experiment, NodeKind, RUN_LIMIT};
use super::experiment_data as data;
use super::experiment_flow::{self as flow, arm_listeners, arm_subscriptions, Followers, Prepared};
use super::experiment_report::{save_report, RunSettings};
use super::experiment_steps::{self as steps, BranchContext, RunSockets, StepEnv};
use super::experiment_validate::validate_run;
use super::netsim::{ImpairmentSummary, Relay};
use super::netsim_run;
use super::http::HttpResponse;
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::secrets::{self, SecretStore};
use super::template::{Renderer, Scope};
use super::ws::{self, WsConfig};

#[derive(Clone, Debug, Serialize)]
pub struct RunEvent {
    pub job_id: u64,
    pub ts: u64,
    pub node_id: String,
    pub state: &'static str,
    pub detail: String,
    pub message_key: Option<&'static str>,
    pub message_params: Value,
    /// Variables this step wrote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vars: Option<BTreeMap<String, Value>>,
    /// Why the step failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EngineError>,
    /// The Inspector frame of the message a wait matched, to open it from the timeline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<u64>,
    /// What a load measured, its thresholds read: on the step's last event, passed or failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<Box<crate::load::LoadMetrics>>,
}

#[derive(Clone, Serialize)]
struct JobEnded {
    job_id: u64,
    kind: &'static str,
    seed: u64,
    profile: Option<String>,
    /// Some values came from Run with… rather than the document.
    overridden: bool,
    error: Option<EngineError>,
    report_path: Option<String>,
    report_error: Option<EngineError>,
}

/// How a run ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Passed,
    Failed,
    /// Stopped from outside — Stop, Stop all, a server shutting down — before
    /// it ended on its own. Like a run stopped in the app, it saves no report.
    Stopped,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Passed => "passed",
            Outcome::Failed => "failed",
            Outcome::Stopped => "stopped",
        }
    }
}

/// What a run starts with besides the document. `overrides` and `seed` are
/// Run with… values for this run only; the document is not changed.
#[derive(Clone, Debug, Default)]
pub struct RunOptions {
    pub overrides: BTreeMap<String, String>,
    pub seed: Option<u64>,
    /// The run fails with `run.timeout` after this long: 1 s to `RUN_LIMIT`,
    /// which is also the limit when none is given.
    pub limit: Option<Duration>,
}

/// A run that has just started: its job and the values it runs with.
#[derive(Clone, Debug, Serialize)]
pub struct RunStarted {
    pub job_id: u64,
    pub experiment: String,
    pub seed: u64,
    pub profile: Option<String>,
    /// Some values came from Run with… rather than the document.
    pub overridden: bool,
    pub started_ms: u64,
}

/// A run that has ended: what its report says, and where the report is.
#[derive(Clone, Debug, Serialize)]
pub struct RunResult {
    pub job_id: u64,
    pub experiment: String,
    pub outcome: Outcome,
    pub seed: u64,
    pub profile: Option<String>,
    pub overridden: bool,
    /// The parameter values the run used.
    pub params: BTreeMap<String, String>,
    pub started_ms: u64,
    pub ended_ms: u64,
    /// Why it failed: the first failure, with secret values masked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<EngineError>,
    /// Every step, in the order they happened.
    pub steps: Vec<RunEvent>,
    /// What each Emulator node received and answered, rule by rule.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub emulators: Vec<EmulatorSummary>,
    /// What each Impairment node's relay did, phase by phase.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub impairments: Vec<ImpairmentSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    /// The report could not be written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_error: Option<EngineError>,
}

/// What a followed run says next.
#[derive(Debug)]
pub enum Progress {
    Step(RunEvent),
    /// Once, after the last step.
    Ended(RunResult),
}

/// A run being followed to its end. Dropping the handle does not stop the
/// run: it is a job like any other, ends on its own (or by Stop) and saves its
/// report either way.
pub struct RunHandle {
    pub started: RunStarted,
    pub info: JobInfo,
    steps: mpsc::UnboundedReceiver<RunEvent>,
    end: oneshot::Receiver<RunResult>,
    ended: Option<RunResult>,
    done: bool,
    /// What a stopped run reports — it is aborted, so it hands over nothing itself.
    events: Arc<Mutex<Vec<RunEvent>>>,
    params: BTreeMap<String, String>,
    /// The emulators' and relays' counters, which a stopped run reports too.
    emulators: Vec<(String, Arc<Emulation>)>,
    relays: Vec<(String, Arc<Relay>)>,
}

impl RunHandle {
    /// The next step as it happens, then the result once, then `None`.
    pub async fn next(&mut self) -> Option<Progress> {
        loop {
            // Every step is sent before the result, so steps already waiting go first.
            if let Ok(step) = self.steps.try_recv() {
                return Some(Progress::Step(step));
            }
            if let Some(result) = self.ended.take() {
                self.done = true;
                return Some(Progress::Ended(result));
            }
            if self.done {
                return None;
            }
            let result = tokio::select! {
                biased;
                Some(step) = self.steps.recv() => return Some(Progress::Step(step)),
                // The sender is dropped without a result only when the job was aborted.
                result = &mut self.end => result.ok(),
            };
            let result = result.unwrap_or_else(|| self.stopped());
            self.ended = Some(result);
        }
    }

    /// Wait for the end; steps not taken with `next` are in the result anyway.
    pub async fn finished(mut self) -> RunResult {
        loop {
            match self.next().await {
                Some(Progress::Ended(result)) => return result,
                Some(Progress::Step(_)) => {}
                // `next` returns the end before it returns None.
                None => return self.stopped(),
            }
        }
    }

    /// The job was aborted before it could report: the steps it got to.
    fn stopped(&self) -> RunResult {
        let started = &self.started;
        RunResult {
            job_id: started.job_id,
            experiment: started.experiment.clone(),
            outcome: Outcome::Stopped,
            seed: started.seed,
            profile: started.profile.clone(),
            overridden: started.overridden,
            params: self.params.clone(),
            started_ms: started.started_ms,
            ended_ms: now_ms(),
            error: None,
            steps: self.events.lock().map(|events| events.clone()).unwrap_or_default(),
            emulators: summaries(&self.emulators),
            impairments: netsim_run::summaries(&self.relays),
            report_path: None,
            report_error: None,
        }
    }
}

fn summaries(emulators: &[(String, Arc<Emulation>)]) -> Vec<EmulatorSummary> {
    emulators.iter().map(|(node, emulation)| emulation.summary(Some(node))).collect()
}

/// The time limit of a run: what was asked for, within 1 s and `RUN_LIMIT`.
fn run_limit(limit: Option<Duration>) -> EngineResult<Duration> {
    match limit {
        None => Ok(RUN_LIMIT),
        Some(limit) if limit >= Duration::from_secs(1) && limit <= RUN_LIMIT => Ok(limit),
        Some(_) => Err(EngineError::new("run.limit_range").with("max", RUN_LIMIT.as_secs())),
    }
}

/// Start a run (the Run button). `overrides` and `seed` come from Run with…
/// and apply to this run only; the document is not changed. Progress and the
/// end go out as `experiment://step` and `experiment://ended` events.
pub async fn start(
    host: Host,
    jobs: JobRegistry,
    doc: Experiment,
    overrides: BTreeMap<String, String>,
    seed: Option<u64>,
    store: &dyn SecretStore,
) -> EngineResult<JobInfo> {
    let handle = start_followed(host, jobs, doc, RunOptions { overrides, seed, limit: None }, store).await?;
    Ok(handle.info)
}

/// Start a run and follow it: the same run as `start` — the same job, events
/// and report — with a handle that hands over each step and then the result.
/// Everything that can stop a run before its first step — validation, a
/// secret that is not stored, a port that cannot be opened — is an error
/// here, before the job exists.
pub async fn start_followed(host: Host, jobs: JobRegistry, doc: Experiment, options: RunOptions, store: &dyn SecretStore) -> EngineResult<RunHandle> {
    let RunOptions { overrides, seed, limit } = options;
    let limit = run_limit(limit)?;
    data::check_seed(seed)?;
    let (_order, params) = validate_run(&doc, &overrides)?;
    let secret_values = data::load_secrets(&doc.nodes, store)?;
    // The job's number and the seed come first: the emulators report under the one and draw from the other.
    let id = jobs.next_id();
    let seed = seed.or(doc.seed).unwrap_or_else(|| rand::random::<u64>() & data::MAX_SEED);
    let mut emulators = emulator_run::arm_run(&host, &doc.nodes, &params, seed, id).await?;
    let mut relays = netsim_run::arm_run(&host, &doc.nodes, &params, seed, id).await?;
    let listeners = arm_listeners(&doc.nodes, Arc::new(host.clone()), &emulators.taps).await?;
    let subscriptions = arm_subscriptions(&host, &doc.nodes, &params).await?;
    let serving = emulators.take_serving();
    let relay_serving = relays.take_serving();
    let reports = std::mem::take(&mut emulators.reports);
    let http = std::mem::take(&mut emulators.http);
    let relay_list = std::mem::take(&mut relays.relays);
    let handle_relays = relay_list.clone();
    let info = JobInfo::new(id, "experiment", doc.name.clone()).with("name", &doc.name);
    let host_cl = host.clone();
    let jobs_cl = jobs.clone();
    let started_ms = info.started_ms;
    let handle_reports = reports.clone();
    let started = RunStarted {
        job_id: id,
        experiment: doc.name.clone(),
        seed,
        profile: doc.profile.clone(),
        overridden: !overrides.is_empty(),
        started_ms,
    };
    let settings = RunSettings { seed, overrides, params };
    let events = Arc::new(Mutex::new(Vec::new()));
    let (steps_tx, steps_rx) = mpsc::unbounded_channel();
    let (end_tx, end_rx) = oneshot::channel();
    let handle_params = settings.params.clone();
    let followers = Followers { events: events.clone(), steps: steps_tx };
    let (ready_tx, ready_rx) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        if ready_rx.await.is_err() {
            return;
        }
        let events = followers.events.clone();
        let prepared = Prepared {
            seed,
            params: settings.params.clone(),
            secrets: secret_values,
            listeners,
            subscriptions,
            http,
            serving,
            relays: relay_list.iter().map(|(node, relay)| (node.clone(), relay.clone())).collect(),
            relay_serving,
            emulations: reports.iter().map(|(node, emulation)| (node.clone(), emulation.clone())).collect(),
            limit,
        };
        let error = flow::run(&host_cl, followers, id, &doc, prepared).await.err();
        let steps = events.lock().map(|list| list.clone()).unwrap_or_default();
        let ended_ms = now_ms();
        let emulators = summaries(&reports);
        let impairments = netsim_run::summaries(&relay_list);
        let (report_path, report_error) = match save_report(id, &settings, started_ms, ended_ms, &doc, &steps, &emulators, &impairments, &error) {
            Ok(path) => (Some(path), None),
            Err(error) => (None, Some(error)),
        };
        jobs_cl.finish(id);
        let ended = JobEnded {
            job_id: id,
            kind: "experiment",
            seed,
            profile: doc.profile.clone(),
            overridden: !settings.overrides.is_empty(),
            error: error.clone(),
            report_path: report_path.clone(),
            report_error: report_error.clone(),
        };
        host_cl.emit("job://ended", ended.clone());
        host_cl.emit("experiment://ended", ended);
        // Nobody following is not an error: the run is complete either way.
        let _ = end_tx.send(RunResult {
            job_id: id,
            experiment: doc.name.clone(),
            outcome: if error.is_some() { Outcome::Failed } else { Outcome::Passed },
            seed,
            profile: doc.profile.clone(),
            overridden: !settings.overrides.is_empty(),
            params: settings.params,
            started_ms,
            ended_ms,
            error,
            steps,
            emulators,
            impairments,
            report_path,
            report_error,
        });
    });
    jobs.insert(info.clone(), task);
    let _ = ready_tx.send(());
    Ok(RunHandle { started, info, steps: steps_rx, end: end_rx, ended: None, done: false, events, params: handle_params, emulators: handle_reports, relays: handle_relays })
}

/// What *Send now* reports. Secret values are masked and the resolved request
/// is not returned, so no value reaches the interface.
#[derive(Serialize)]
pub struct NodeSendResult {
    pub detail: String,
    pub response: Option<HttpResponse>,
    /// Values the step set: a wait's reply, or what the Extract nodes after a
    /// request take from its response.
    pub vars: BTreeMap<String, Value>,
}

/// Run one node on its own: an action is sent, a wait listens from now on.
/// Resolved with the active profile, the variable values the editor knows and
/// the stored secrets, then performed by the same code a run uses.
pub async fn send_node(
    host: &Host,
    doc: &Experiment,
    node_id: &str,
    vars: &BTreeMap<String, Value>,
    store: &dyn SecretStore,
) -> EngineResult<NodeSendResult> {
    let node = data::find_node(doc, node_id)?;
    if !node.kind.is_action() && !node.kind.is_wait() {
        return Err(EngineError::new("run.not_an_action").at(node_id));
    }
    // A WebSocket send or wait needs its connection: the *WebSocket connect* it names opens one for it.
    let opener = match node.kind.connection() {
        Some(connection) => match doc.nodes.iter().find(|candidate| candidate.id == connection) {
            Some(opener) if matches!(opener.kind, NodeKind::WsConnect { .. }) => Some(opener),
            _ => return Err(EngineError::new("ws.connection_unknown").with("id", connection).in_field(Field::new("connection")).at(node_id)),
        },
        None => None,
    };
    let params = data::effective_params(doc, doc.profile.as_deref(), &BTreeMap::new());
    let secret_values = data::load_secrets(std::iter::once(node).chain(opener), store)?;
    let masked: Vec<String> = secret_values.values().cloned().collect();
    let _redaction = secrets::redact(masked.clone());
    let seed = doc.seed.unwrap_or_else(|| rand::random::<u64>() & data::MAX_SEED);
    let scope = Scope { params: &params, vars, secrets: &secret_values, run_id: 0, seed, node_id, count: 1, now_ms: now_ms() };
    let kind = data::render_kind(&node.kind, &mut Renderer::new(scope)).map_err(|error| error.at(node_id).masked(&masked))?;
    // Basic sends name and password as base64: a secret in either is masked in that form too.
    let derived = data::derived_masks(&kind, &secret_values);
    let _derived = secrets::redact(derived.clone());
    let masked: Vec<String> = masked.into_iter().chain(derived).collect();
    let failed = |error: EngineError| error.at(node_id).masked(&masked);
    let nodes = std::slice::from_ref(node);
    // A *Wait for HTTP request* listens now on a listener of its own, like a run's.
    let mut emulators = emulator_run::arm_run(host, nodes, &params, seed, 0).await.map_err(failed)?;
    let _serving = emulators.take_serving();
    let listeners = arm_listeners(nodes, Arc::new(host.clone()), &emulators.taps).await.map_err(failed)?;
    let subscriptions = arm_subscriptions(host, nodes, &params).await.map_err(failed)?;
    let websockets = RunSockets::default();
    // A greeting read while the connection opens counts for a wait.
    let started = Instant::now();
    if let Some(opener) = opener {
        let scope = Scope { params: &params, vars, secrets: &secret_values, run_id: 0, seed, node_id: &opener.id, count: 1, now_ms: now_ms() };
        let opened = |error: EngineError| error.at(&opener.id).masked(&masked);
        if let NodeKind::WsConnect { url, headers, protocols, timeout_ms } = data::render_kind(&opener.kind, &mut Renderer::new(scope)).map_err(opened)? {
            let config = WsConfig { url, headers, protocols, timeout_ms };
            let socket = ws::open(host, &config, "experiment", None).await.map_err(opened)?;
            websockets.lock().unwrap().insert(opener.id.clone(), Arc::new(socket));
        }
    }
    let env = StepEnv {
        host,
        node_id,
        // A one-off client id: reusing one would knock another connection off the broker.
        client_id: format!("lab-send-{:08x}", rand::random::<u32>()),
        seed,
        listeners,
        subscriptions,
        http: std::mem::take(&mut emulators.http),
        // Send now never takes a relay or an emulator down: those are a run's.
        relays: Default::default(),
        emulators: Default::default(),
        websockets,
        // One request: nothing set before it to send back.
        cookies: None,
        has_timeout: false,
        has_limit: false,
        inputs: 1,
        run_started: started,
    };
    let mut context = BranchContext { vars: vars.clone(), ..Default::default() };
    let outcome = steps::execute(&env, &kind, &mut context).await.map_err(failed)?;
    let mut values = outcome.written.unwrap_or_default();
    if let Some(response) = &context.last_response {
        values.extend(data::extract_after(doc, node_id, response));
    }
    Ok(NodeSendResult {
        detail: secrets::mask(&outcome.detail, &masked),
        response: context.last_response.as_ref().map(|response| data::mask_response(response, &masked)),
        vars: values.into_iter().map(|(name, value)| (name, secrets::mask_value(&value, &masked))).collect(),
    })
}
