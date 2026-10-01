//! Executing an experiment: branches, joins, listeners, step events, the run
//! report and *Send now*. What each step does is `experiment_steps`; this
//! module decides which step runs next and reports it.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use crate::host::Host;

use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Experiment, Node, NodeKind};
use super::experiment_data as data;
use super::experiment_steps::{self as steps, BranchContext, StepEnv};
use super::experiment_validate::{check_bind, check_reply_bind, validate_run};
use super::http::HttpResponse;
use super::jobs::{now_ms, JobInfo, JobRegistry, TaskGuard};
use super::listen::{FrameSink, Listener, Listeners};
use super::secrets::{self, SecretStore};
use super::subscribe::{self, Subscription, Subscriptions};
use super::paths::data_dir;
use super::template::{Renderer, Scope};

/// A run that takes longer than this is stopped.
const RUN_LIMIT: Duration = Duration::from_secs(300);
/// Version of the run report file.
const REPORT_VERSION: u32 = 2;

#[derive(Clone, Serialize)]
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

#[derive(Serialize)]
struct RunReport<'a> {
    version: u32,
    experiment: &'a str,
    document_version: u32,
    seed: u64,
    profile: Option<&'a str>,
    overrides: &'a BTreeMap<String, String>,
    /// The values the run actually used.
    params: &'a BTreeMap<String, String>,
    started_ms: u64,
    ended_ms: u64,
    outcome: &'static str,
    error: &'a Option<EngineError>,
    steps: &'a [RunEvent],
}

struct JoinBarrier {
    expected: usize,
    arrived: usize,
    /// The Join's inputs in document order; arrivals are merged in this order.
    inputs: Vec<String>,
    /// (predecessor, its context) per arrival.
    states: Vec<(String, BranchContext)>,
}

/// The Join a finished run is stuck on, with how many branches never came: the
/// one that was reached at least once, since that is where the flow stopped.
fn waiting_join(joins: &HashMap<String, JoinBarrier>) -> Option<(String, usize)> {
    joins
        .iter()
        .filter(|(_, barrier)| barrier.arrived < barrier.expected)
        .max_by_key(|(id, barrier)| (barrier.arrived, std::cmp::Reverse((*id).clone())))
        .map(|(id, barrier)| (id.clone(), barrier.expected - barrier.arrived))
}

struct EngineShared {
    host: Host,
    job_id: u64,
    events: Mutex<Vec<RunEvent>>,
    by_id: HashMap<String, Node>,
    /// Every wire of an output, in document order: one output may feed several
    /// nodes, which then run in parallel.
    outgoing: HashMap<(String, String), Vec<String>>,
    joins: Mutex<HashMap<String, JoinBarrier>>,
    stop_flag: AtomicBool,
    first_error: Mutex<Option<EngineError>>,
    end_reached: AtomicBool,
    params: BTreeMap<String, String>,
    seed: u64,
    /// Secret values this run reads, and the same values as the mask list.
    secrets: BTreeMap<String, String>,
    masked: Vec<String>,
    /// Executions per node, for `{{counter}}` and per-execution random streams.
    counts: Mutex<HashMap<String, u64>>,
    /// Every branch task. Branches are spawned, so aborting the job's own task
    /// would otherwise leave them running — sending traffic after "stop".
    tasks: Mutex<TaskGuard>,
    /// Sockets the waits listen on; cleared when the run ends or is stopped.
    listeners: Mutex<Listeners>,
    /// Broker connections the MQTT waits listen on; closed with the run.
    subscriptions: Mutex<Subscriptions>,
    started: Instant,
}

/// Held by `run` for its whole life: when the job is stopped, `run` is dropped
/// mid-await and this takes every branch and listening socket down with it.
struct CancelBranches(Arc<EngineShared>);

impl Drop for CancelBranches {
    fn drop(&mut self) {
        self.0.stop_flag.store(true, Ordering::SeqCst);
        if let Ok(mut tasks) = self.0.tasks.lock() {
            drop(std::mem::take(&mut *tasks));
        }
        if let Ok(mut listeners) = self.0.listeners.lock() {
            listeners.clear();
        }
        if let Ok(mut subscriptions) = self.0.subscriptions.lock() {
            subscriptions.clear();
        }
    }
}

/// One step event. Nothing a run reports carries a secret value.
struct Step {
    state: &'static str,
    detail: String,
    message: Option<(&'static str, Value)>,
    vars: Option<BTreeMap<String, Value>>,
    error: Option<EngineError>,
    frame: Option<u64>,
}

impl Step {
    fn running() -> Self {
        Step { state: "running", detail: String::new(), message: None, vars: None, error: None, frame: None }
    }

    fn failed(error: EngineError) -> Self {
        Step { state: "failed", detail: String::new(), message: None, vars: None, error: Some(error), frame: None }
    }

    /// An attempt failed and the step will run again after `pause`.
    fn retrying(error: EngineError, attempt: u32, attempts: u32, pause: Duration) -> Self {
        let ms = pause.as_millis() as u64;
        Step {
            state: "retry",
            detail: format!("Attempt {attempt} of {attempts} failed; again in {ms} ms"),
            message: Some(("exp.step.retrying", serde_json::json!({ "attempt": attempt, "attempts": attempts, "ms": ms }))),
            vars: None,
            error: Some(error),
            frame: None,
        }
    }
}

fn emit(shared: &EngineShared, node_id: &str, step: Step) {
    let masked = &shared.masked;
    let event = RunEvent {
        job_id: shared.job_id,
        ts: now_ms(),
        node_id: node_id.into(),
        state: step.state,
        detail: secrets::mask(&step.detail, masked),
        message_key: step.message.as_ref().map(|(key, _)| *key),
        message_params: step.message.map(|(_, params)| secrets::mask_value(&params, masked)).unwrap_or_default(),
        vars: step.vars.map(|vars| vars.into_iter().map(|(name, value)| (name, secrets::mask_value(&value, masked))).collect()),
        error: step.error.map(|error| error.masked(masked)),
        frame: step.frame,
    };
    if let Ok(mut list) = shared.events.lock() {
        list.push(event.clone());
    }
    shared.host.emit("experiment://step", event);
}

/// The first failure of a run stops every branch; later ones are only reported.
fn fail(shared: &EngineShared, node_id: &str, error: EngineError) {
    let error = error.at(node_id);
    shared.stop_flag.store(true, Ordering::SeqCst);
    {
        let mut first = shared.first_error.lock().unwrap();
        if first.is_none() {
            *first = Some(error.clone());
        }
    }
    emit(shared, node_id, Step::failed(error));
}

fn spawn_branch(
    shared: Arc<EngineShared>,
    start_node: String,
    context: BranchContext,
    active_tasks: Arc<AtomicUsize>,
    done_tx: tokio::sync::mpsc::Sender<()>,
) {
    active_tasks.fetch_add(1, Ordering::SeqCst);
    let tracked = shared.clone();
    let handle = tokio::spawn(async move {
        run_branch(shared, start_node, context, active_tasks, done_tx).await;
    });
    if let Ok(mut tasks) = tracked.tasks.lock() {
        tasks.watch(handle.abort_handle());
    };
}

/// Where a branch goes after `from` leaves through `ports`: every wire, in
/// document order. The first target continues this branch; each further one
/// starts a parallel branch with a copy of its variables. A Join counts the
/// arrival and lets only the last one through, with the merged variables — so
/// a branch whose every target is a waiting Join ends here (`None`).
fn fan_out(
    shared: &Arc<EngineShared>,
    from: &str,
    ports: &[&str],
    context: &BranchContext,
    active_tasks: &Arc<AtomicUsize>,
    done_tx: &tokio::sync::mpsc::Sender<()>,
) -> Option<(String, BranchContext)> {
    let mut stay = None;
    for port in ports {
        let Some(targets) = shared.outgoing.get(&(from.to_string(), port.to_string())) else { continue };
        for target in targets {
            let next = if shared.by_id.get(target).is_some_and(|node| matches!(node.kind, NodeKind::Join)) {
                match arrive(shared, target, from, context) {
                    Some(merged) => merged,
                    None => continue,
                }
            } else {
                context.clone()
            };
            if stay.is_none() {
                stay = Some((target.clone(), next));
            } else {
                spawn_branch(shared.clone(), target.clone(), next, active_tasks.clone(), done_tx.clone());
            }
        }
    }
    stay
}

/// Count the arrival at a Join. Returns the merged context when every input
/// has arrived, `None` while the Join is still waiting.
fn arrive(shared: &EngineShared, join: &str, from: &str, context: &BranchContext) -> Option<BranchContext> {
    let mut joins = shared.joins.lock().unwrap();
    let barrier = joins.get_mut(join)?;
    barrier.arrived += 1;
    barrier.states.push((from.to_string(), context.clone()));
    if barrier.arrived < barrier.expected {
        return None;
    }
    // Merge in the order of the Join's connections, not of arrival, so the
    // result never depends on which branch happened to finish first.
    let inputs = barrier.inputs.clone();
    barrier.states.sort_by_key(|(from, _)| inputs.iter().position(|input| input == from).unwrap_or(usize::MAX));
    Some(BranchContext::merge(barrier.states.iter().map(|(_, state)| state)))
}

async fn run_branch(
    shared: Arc<EngineShared>,
    start_node: String,
    mut context: BranchContext,
    active_tasks: Arc<AtomicUsize>,
    done_tx: tokio::sync::mpsc::Sender<()>,
) {
    let mut current = start_node;
    loop {
        if shared.stop_flag.load(Ordering::Relaxed) {
            break;
        }
        let Some(node) = shared.by_id.get(&current).cloned() else {
            fail(&shared, &current, EngineError::new("run.node_missing").with("id", &current));
            break;
        };
        // End is reached by every parallel branch that leads there, but the run
        // completes once: End shows as running from the first arrival and passes
        // in `run`, after the last branch — unless a branch failed meanwhile.
        if matches!(node.kind, NodeKind::End) {
            if !shared.end_reached.swap(true, Ordering::SeqCst) {
                emit(&shared, &current, Step::running());
            }
            break;
        }
        emit(&shared, &current, Step::running());

        // Templates are resolved per execution, against this branch's variables.
        let count = {
            let mut counts = shared.counts.lock().unwrap();
            let count = counts.entry(current.clone()).or_insert(0);
            *count += 1;
            *count
        };
        let scope = Scope {
            params: &shared.params,
            vars: &context.vars,
            secrets: &shared.secrets,
            run_id: shared.job_id,
            seed: shared.seed,
            node_id: &current,
            count,
            now_ms: now_ms(),
        };
        let rendered = data::render_kind(&node.kind, &mut Renderer::new(scope));
        let env = StepEnv {
            host: &shared.host,
            client_id: format!("lab-{}", shared.job_id),
            seed: shared.seed,
            listeners: shared.listeners.lock().map(|listeners| listeners.clone()).unwrap_or_default(),
            subscriptions: shared.subscriptions.lock().map(|subscriptions| subscriptions.clone()).unwrap_or_default(),
            has_timeout: shared.outgoing.contains_key(&(current.clone(), "timeout".to_string())),
            inputs: shared.joins.lock().ok().and_then(|joins| joins.get(&current).map(|barrier| barrier.expected)).unwrap_or(1),
            run_started: shared.started,
        };
        // A failed attempt of an action or wait is reported and made again
        // after its pause; a template that does not resolve is not retried.
        let result = match rendered {
            Ok(kind) => {
                let retry = node.retry.as_ref().filter(|_| node.kind.retries());
                let mut attempt = 1;
                loop {
                    let result = steps::execute(&env, &kind, &mut context).await;
                    let Some(retry) = retry else { break result };
                    match result {
                        Err(error) if attempt < retry.attempts && !shared.stop_flag.load(Ordering::Relaxed) => {
                            let pause = retry.pause_before(attempt + 1);
                            emit(&shared, &current, Step::retrying(error.at(&current), attempt, retry.attempts, pause));
                            // Stop aborts this task, so the pause ends with it.
                            tokio::time::sleep(pause).await;
                            attempt += 1;
                        }
                        other => break other,
                    }
                }
            }
            Err(error) => Err(error),
        };
        let port = match result {
            Ok(outcome) => {
                let step = Step { state: "passed", detail: outcome.detail, message: outcome.message, vars: outcome.written, error: None, frame: outcome.frame };
                emit(&shared, &current, step);
                outcome.port
            }
            Err(error) => {
                fail(&shared, &current, error);
                break;
            }
        };

        // A Parallel branch leaves through both of its outputs; any other node
        // through the one its step chose. Either may have several wires.
        let ports: &[&str] = if matches!(node.kind, NodeKind::Fork) { &["branch1", "branch2"] } else { &[port] };
        match fan_out(&shared, &current, ports, &context, &active_tasks, &done_tx) {
            Some((next, next_context)) => {
                current = next;
                context = next_context;
            }
            None => break,
        }
    }

    if active_tasks.fetch_sub(1, Ordering::SeqCst) == 1 {
        let _ = done_tx.send(()).await;
    }
}

/// One socket per distinct bind address, opened before the first step so a
/// reply that beats the wait is not lost. A port that cannot be opened stops
/// the run before any traffic, at the first wait that uses it.
async fn arm_listeners(nodes: &[Node], sink: Arc<dyn FrameSink>) -> EngineResult<Listeners> {
    let mut listeners = Listeners::new();
    for node in nodes {
        let Some(bind) = node.kind.bind() else { continue };
        // A send that expects a reply may listen on any free port (0): it sends from there.
        let (address, field) = if node.kind.expects_reply() {
            (check_reply_bind(bind).map_err(|error| error.at(&node.id))?, "reply_bind")
        } else {
            (check_bind(bind).map_err(|error| error.at(&node.id))?, "bind")
        };
        if listeners.contains_key(&address) {
            continue;
        }
        let listener = Listener::arm(address, sink.clone()).await.map_err(|error| error.in_field(Field::new(field)).at(&node.id))?;
        listeners.insert(address, Arc::new(listener));
    }
    Ok(listeners)
}

/// One MQTT connection per broker and topic filter of the *Wait for MQTT*
/// steps, subscribed before the first step — so their broker and topic may use
/// parameters only. A refusal stops the run before any traffic, at that wait.
async fn arm_subscriptions(host: &Host, nodes: &[Node], params: &BTreeMap<String, String>) -> EngineResult<Subscriptions> {
    let mut subscriptions = Subscriptions::new();
    for node in nodes {
        let Some((broker, port, topic)) = node.kind.subscription() else { continue };
        let at = |error: EngineError| error.at(&node.id);
        let fixed = |text: &str, field: &'static str| {
            data::static_text(text, params).ok_or_else(|| at(EngineError::new("node.params_only").in_field(Field::new(field))))
        };
        let (broker, topic) = (fixed(broker, "broker")?, fixed(topic, "topic")?);
        let key = subscribe::key(&broker, port, &topic);
        if subscriptions.contains_key(&key) {
            continue;
        }
        let subscription = Subscription::arm(host.clone(), &broker, port, &topic).await.map_err(at)?;
        subscriptions.insert(key, Arc::new(subscription));
    }
    Ok(subscriptions)
}

struct Prepared {
    seed: u64,
    params: BTreeMap<String, String>,
    secrets: BTreeMap<String, String>,
    listeners: Listeners,
    subscriptions: Subscriptions,
}

async fn run(host: &Host, events: &mut Vec<RunEvent>, job_id: u64, doc: &Experiment, prepared: Prepared) -> Result<(), EngineError> {
    let mut joins = HashMap::new();
    for node in doc.nodes.iter().filter(|node| matches!(node.kind, NodeKind::Join)) {
        let inputs: Vec<String> = doc.edges.iter().filter(|edge| edge.to == node.id).map(|edge| edge.from.clone()).collect();
        joins.insert(node.id.clone(), JoinBarrier { expected: inputs.len(), arrived: 0, inputs, states: Vec::new() });
    }
    let shared = Arc::new(EngineShared {
        host: host.clone(),
        job_id,
        events: Mutex::new(Vec::new()),
        by_id: doc.nodes.iter().map(|node| (node.id.clone(), node.clone())).collect(),
        outgoing: doc.edges.iter().fold(HashMap::new(), |mut outgoing: HashMap<(String, String), Vec<String>>, edge| {
            outgoing.entry((edge.from.clone(), edge.port.clone())).or_default().push(edge.to.clone());
            outgoing
        }),
        joins: Mutex::new(joins),
        stop_flag: AtomicBool::new(false),
        first_error: Mutex::new(None),
        end_reached: AtomicBool::new(false),
        params: prepared.params,
        seed: prepared.seed,
        masked: prepared.secrets.values().cloned().collect(),
        secrets: prepared.secrets,
        counts: Mutex::new(HashMap::new()),
        tasks: Mutex::new(TaskGuard::new()),
        listeners: Mutex::new(prepared.listeners),
        subscriptions: Mutex::new(prepared.subscriptions),
        started: Instant::now(),
    });
    let _cancel = CancelBranches(shared.clone());
    // The Inspector masks these values while the run lasts (or until it is stopped).
    let _redaction = secrets::redact(shared.masked.clone());

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel(1);
    // validate_document guarantees exactly one Start.
    let start_id = doc.nodes.iter().find(|node| matches!(node.kind, NodeKind::Start)).map(|node| node.id.clone()).unwrap_or_default();
    let active_tasks = Arc::new(AtomicUsize::new(0));
    spawn_branch(shared.clone(), start_id.clone(), BranchContext::default(), active_tasks, done_tx.clone());
    drop(done_tx);

    if tokio::time::timeout(RUN_LIMIT, done_rx.recv()).await.is_err() {
        shared.stop_flag.store(true, Ordering::SeqCst);
        let mut first = shared.first_error.lock().unwrap();
        if first.is_none() {
            *first = Some(EngineError::new("run.timeout").with("seconds", RUN_LIMIT.as_secs()));
        }
    }

    // Every branch finished without an error, yet nobody reached End: a Join is
    // waiting for a branch that was never taken (e.g. behind a status branch).
    // That is a failed run, not a quiet pass.
    let stalled = shared.first_error.lock().unwrap().is_none() && !shared.end_reached.load(Ordering::SeqCst);
    if stalled {
        let waiting = waiting_join(&shared.joins.lock().unwrap());
        match waiting {
            Some((id, missing)) => fail(&shared, &id, EngineError::new("run.join_waiting").with("count", missing)),
            None => fail(&shared, &start_id, EngineError::new("run.no_end")),
        }
    }

    // Every branch has finished: now the run is complete.
    if shared.first_error.lock().unwrap().is_none() {
        if let Some(end) = doc.nodes.iter().find(|node| matches!(node.kind, NodeKind::End)) {
            let complete = Step { state: "passed", detail: "Complete".into(), message: Some(("exp.step.complete", serde_json::json!({}))), vars: None, error: None, frame: None };
            emit(&shared, &end.id, complete);
        }
    }
    if let Ok(list) = shared.events.lock() {
        *events = list.clone();
    }
    let first = shared.first_error.lock().unwrap().clone();
    match first {
        Some(error) => Err(error.masked(&shared.masked)),
        None => Ok(()),
    }
}

struct RunSettings {
    seed: u64,
    overrides: BTreeMap<String, String>,
    params: BTreeMap<String, String>,
}

fn file_error(path: &std::path::Path, error: impl ToString) -> EngineError {
    EngineError::new("file.io").with("path", path.display()).because(error)
}

fn save_report(id: u64, settings: &RunSettings, started_ms: u64, doc: &Experiment, steps: &[RunEvent], error: &Option<EngineError>) -> EngineResult<String> {
    let dir = data_dir().join("runs");
    std::fs::create_dir_all(&dir).map_err(|error| file_error(&dir, error))?;
    let path = dir.join(format!("run-{started_ms}-{id}.json"));
    let report = RunReport {
        version: REPORT_VERSION,
        experiment: &doc.name,
        document_version: doc.version,
        seed: settings.seed,
        profile: doc.profile.as_deref(),
        overrides: &settings.overrides,
        params: &settings.params,
        started_ms,
        ended_ms: now_ms(),
        outcome: if error.is_some() { "failed" } else { "passed" },
        error,
        steps,
    };
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| file_error(&path, error))?;
    std::fs::write(&path, bytes).map_err(|error| file_error(&path, error))?;
    Ok(path.display().to_string())
}

/// Start a run. `overrides` and `seed` come from Run with… and apply to this
/// run only; the document is not changed. Everything that can stop a run
/// before its first step — validation, a secret that is not stored, a port
/// that cannot be opened — is an error here, before the job exists.
pub async fn start(
    host: Host,
    jobs: JobRegistry,
    doc: Experiment,
    overrides: BTreeMap<String, String>,
    seed: Option<u64>,
    store: &dyn SecretStore,
) -> EngineResult<JobInfo> {
    data::check_seed(seed)?;
    let (_order, params) = validate_run(&doc, &overrides)?;
    let secret_values = data::load_secrets(&doc.nodes, store)?;
    let listeners = arm_listeners(&doc.nodes, Arc::new(host.clone())).await?;
    let subscriptions = arm_subscriptions(&host, &doc.nodes, &params).await?;
    let id = jobs.next_id();
    let info = JobInfo { id, kind: "experiment".into(), label: doc.name.clone(), started_ms: now_ms() };
    let host_cl = host.clone();
    let jobs_cl = jobs.clone();
    let started_ms = info.started_ms;
    let seed = seed.or(doc.seed).unwrap_or_else(|| rand::random::<u64>() & data::MAX_SEED);
    let settings = RunSettings { seed, overrides, params };
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
        if ready_rx.await.is_err() {
            return;
        }
        let mut steps = Vec::new();
        let prepared = Prepared { seed, params: settings.params.clone(), secrets: secret_values, listeners, subscriptions };
        let error = run(&host_cl, &mut steps, id, &doc, prepared).await.err();
        let (report_path, report_error) = match save_report(id, &settings, started_ms, &doc, &steps, &error) {
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
            error,
            report_path,
            report_error,
        };
        host_cl.emit("job://ended", ended.clone());
        host_cl.emit("experiment://ended", ended);
    });
    jobs.insert(info.clone(), handle);
    let _ = ready_tx.send(());
    Ok(info)
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
    let params = data::effective_params(doc, doc.profile.as_deref(), &BTreeMap::new());
    let secret_values = data::load_secrets(std::iter::once(node), store)?;
    let masked: Vec<String> = secret_values.values().cloned().collect();
    let _redaction = secrets::redact(masked.clone());
    let seed = doc.seed.unwrap_or_else(|| rand::random::<u64>() & data::MAX_SEED);
    let scope = Scope { params: &params, vars, secrets: &secret_values, run_id: 0, seed, node_id, count: 1, now_ms: now_ms() };
    let failed = |error: EngineError| error.at(node_id).masked(&masked);
    let kind = data::render_kind(&node.kind, &mut Renderer::new(scope)).map_err(failed)?;
    let listeners = arm_listeners(std::slice::from_ref(node), Arc::new(host.clone())).await.map_err(failed)?;
    let subscriptions = arm_subscriptions(host, std::slice::from_ref(node), &params).await.map_err(failed)?;
    let env = StepEnv {
        host,
        // A one-off client id: reusing one would knock another connection off the broker.
        client_id: format!("lab-send-{:08x}", rand::random::<u32>()),
        seed,
        listeners,
        subscriptions,
        has_timeout: false,
        inputs: 1,
        run_started: Instant::now(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::listen::NoFrames;

    #[test]
    fn a_join_left_waiting_is_reported_not_passed() {
        let barrier = |expected, arrived| JoinBarrier { expected, arrived, inputs: Vec::new(), states: Vec::new() };
        let mut joins = HashMap::new();
        joins.insert("done".to_string(), barrier(2, 2));
        assert_eq!(waiting_join(&joins), None);
        // Behind a status branch only one of two inputs is ever taken.
        joins.insert("after-branch".to_string(), barrier(2, 1));
        joins.insert("never-reached".to_string(), barrier(2, 0));
        assert_eq!(waiting_join(&joins), Some(("after-branch".to_string(), 1)));
    }

    fn wait_node(id: &str, bind: &str) -> Node {
        serde_json::from_value(serde_json::json!({ "id": id, "x": 0, "y": 0, "type": "wait_udp", "bind": bind })).unwrap()
    }

    #[tokio::test]
    async fn waits_sharing_a_bind_share_one_socket_and_a_taken_port_names_the_wait() {
        let taken = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let free = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
        let nodes = [wait_node("a", &free.to_string()), wait_node("b", &free.to_string())];
        let listeners = arm_listeners(&nodes, Arc::new(NoFrames)).await.unwrap();
        assert_eq!(listeners.len(), 1);
        drop(listeners);

        let busy = [wait_node("free", &free.to_string()), wait_node("busy", &taken.local_addr().unwrap().to_string())];
        let error = arm_listeners(&busy, Arc::new(NoFrames)).await.err().unwrap();
        assert_eq!((error.code.as_str(), error.node.as_deref(), error.field.as_ref().map(|field| field.key.clone())), ("transport.address_in_use", Some("busy"), Some("bind".to_string())));
    }
}
