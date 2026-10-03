//! How a run moves: the sockets and broker connections its waits need,
//! opened before the first step; one task per branch; every wire of an output
//! followed (each past the first a parallel branch with a copy of the
//! variables); Joins merged in document order; End passed once, after the last
//! branch; each step retried and repeated as its node says. Every step is
//! reported as it happens, secret values masked. What a step does is
//! `experiment_steps`; starting and following a run is `experiment_run`.

use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::mpsc;
use crate::host::Host;

use super::emulator_run::{HttpListeners, RunServing};
use super::emulator_state::Emulation;
use super::netsim::Relay;
use super::netsim_run;
use super::error::{EngineError, EngineResult, Field};
use super::experiment::{Experiment, Node, NodeKind, Repeat, RepeatUntil};
use super::experiment_data as data;
use super::experiment_run::RunEvent;
use super::cookies::CookieJar;
use super::experiment_steps::{self as steps, BranchContext, RunSockets, StepEnv, StepOutcome};
use super::experiment_fields::{check_bind, check_reply_bind};
use super::jobs::{now_ms, TaskGuard};
use super::listen::{FrameSink, Listener, Listeners, Tap};
use super::secrets;
use super::subscribe::{self, Subscription, Subscriptions};
use super::template::{Renderer, Scope};

/// How often a repeating step reports how far it has got.
const REPEAT_REPORT_EVERY: Duration = Duration::from_secs(1);

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
    /// Every step so far; shared with the handle, which reads it if the run is stopped.
    events: Arc<Mutex<Vec<RunEvent>>>,
    /// Each step as it happens, for whoever follows the run (gone when nobody does).
    observer: mpsc::UnboundedSender<RunEvent>,
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
    /// Secret values this run reads; the mask list is those values and what a
    /// request derived from them (Basic's base64), added as requests render.
    secrets: BTreeMap<String, String>,
    masked: RwLock<Vec<String>>,
    /// The Inspector masks the derived values too while the run lasts.
    derived: Mutex<Vec<secrets::Redaction>>,
    /// Executions per node, for `{{counter}}` and per-execution random streams.
    counts: Mutex<HashMap<String, u64>>,
    /// Every branch task. Branches are spawned, so aborting the job's own task
    /// would otherwise leave them running — sending traffic after "stop".
    tasks: Mutex<TaskGuard>,
    /// Sockets the waits listen on; cleared when the run ends or is stopped.
    listeners: Mutex<Listeners>,
    /// Broker connections the MQTT waits listen on; closed with the run.
    subscriptions: Mutex<Subscriptions>,
    /// The HTTP listeners of the *Wait for HTTP request* steps.
    http: HttpListeners,
    /// The run's emulators and HTTP listeners, serving until the run ends.
    serving: Mutex<RunServing>,
    /// The run's impairment relays and emulators by node id, for the steps that change them.
    relays: HashMap<String, Arc<Relay>>,
    emulations: HashMap<String, Arc<Emulation>>,
    /// The relays' tasks: closed with the run, so nothing stays impaired.
    relay_serving: Mutex<netsim_run::RunServing>,
    /// The WebSocket connections the run opened; closed with it.
    websockets: RunSockets,
    /// What the run's servers set with Set-Cookie, sent back as a browser would.
    cookies: Option<Arc<CookieJar>>,
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
        if let Ok(mut serving) = self.0.serving.lock() {
            drop(std::mem::take(&mut *serving));
        }
        if let Ok(mut serving) = self.0.relay_serving.lock() {
            drop(std::mem::take(&mut *serving));
        }
        // Each connection says goodbye (a close frame) as its last handle goes.
        if let Ok(mut websockets) = self.0.websockets.lock() {
            websockets.clear();
        }
        if let Ok(mut derived) = self.0.derived.lock() {
            derived.clear();
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

    /// A repeating step's progress: the sends so far.
    fn repeating(sent: u32, repeat: &Repeat, elapsed: Duration) -> Self {
        let (key, params) = match repeat.until {
            RepeatUntil::Count => ("exp.step.repeatingCount", serde_json::json!({ "n": sent, "count": repeat.count })),
            RepeatUntil::Duration => ("exp.step.repeatingFor", serde_json::json!({
                "n": sent,
                "s": format!("{:.1}", elapsed.as_secs_f64()),
                "total": format!("{:.1}", repeat.duration_ms as f64 / 1000.0),
            })),
        };
        Step { state: "repeating", detail: format!("Sent {sent} times"), message: Some((key, params)), vars: None, error: None, frame: None }
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
    let masked = shared.masked.read().unwrap();
    let masked = masked.as_slice();
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
    // Nobody following the run is not an error.
    let _ = shared.observer.send(event.clone());
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

        // Templates are resolved per execution, against this branch's variables —
        // except a Loop's exit condition as an iteration starts: it is read when
        // the body comes back, and may test what the body sets.
        let count = shared.next_count(&current);
        let entering_loop = matches!(node.kind, NodeKind::Loop { .. }) && !context.loops.contains_key(&current);
        let rendered = if entering_loop { Ok(node.kind.clone()) } else { render(&shared, &node, &context, count) };
        let env = StepEnv {
            host: &shared.host,
            node_id: &current,
            client_id: format!("lab-{}", shared.job_id),
            seed: shared.seed,
            listeners: shared.listeners.lock().map(|listeners| listeners.clone()).unwrap_or_default(),
            subscriptions: shared.subscriptions.lock().map(|subscriptions| subscriptions.clone()).unwrap_or_default(),
            http: shared.http.clone(),
            relays: shared.relays.clone(),
            emulators: shared.emulations.clone(),
            websockets: shared.websockets.clone(),
            cookies: shared.cookies.clone(),
            has_timeout: shared.outgoing.contains_key(&(current.clone(), "timeout".to_string())),
            has_limit: shared.outgoing.contains_key(&(current.clone(), "limit".to_string())),
            inputs: shared.joins.lock().ok().and_then(|joins| joins.get(&current).map(|barrier| barrier.expected)).unwrap_or(1),
            run_started: shared.started,
        };
        // A template that does not resolve is neither retried nor repeated.
        let result = match rendered {
            Ok(kind) => match node.repeat.as_ref().filter(|_| node.kind.repeats()) {
                Some(repeat) => repeated(&shared, &env, &node, kind, repeat, &mut context).await,
                None => attempts(&shared, &env, &node, &kind, &mut context).await,
            },
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

impl EngineShared {
    /// Values a rendered request derived from a secret, masked from now on.
    fn mask_more(&self, values: Vec<String>) {
        let mut masked = self.masked.write().unwrap();
        let new: Vec<String> = values.into_iter().filter(|value| !value.is_empty() && !masked.contains(value)).collect();
        if new.is_empty() {
            return;
        }
        masked.extend(new.iter().cloned());
        self.derived.lock().unwrap().push(secrets::redact(new));
    }

    /// One more execution of `node`: its number, for `{{counter}}` and its random stream.
    fn next_count(&self, node: &str) -> u64 {
        let mut counts = self.counts.lock().unwrap();
        let count = counts.entry(node.to_string()).or_insert(0);
        *count += 1;
        *count
    }
}

/// A node's fields with this execution's values in its templates.
fn render(shared: &EngineShared, node: &Node, context: &BranchContext, count: u64) -> EngineResult<NodeKind> {
    let scope = Scope {
        params: &shared.params,
        vars: &context.vars,
        secrets: &shared.secrets,
        run_id: shared.job_id,
        seed: shared.seed,
        node_id: &node.id,
        count,
        now_ms: now_ms(),
    };
    let kind = data::render_kind(&node.kind, &mut Renderer::new(scope))?;
    shared.mask_more(data::derived_masks(&kind, &shared.secrets));
    Ok(kind)
}

/// One execution of a step, retried: a failed attempt of an action or wait is
/// reported and made again after its pause.
async fn attempts(shared: &EngineShared, env: &StepEnv<'_>, node: &Node, kind: &NodeKind, context: &mut BranchContext) -> EngineResult<StepOutcome> {
    let retry = node.retry.as_ref().filter(|_| node.kind.retries());
    let mut attempt = 1;
    loop {
        let result = steps::execute(env, kind, context).await;
        let Some(retry) = retry else { return result };
        match result {
            Err(error) if attempt < retry.attempts && !shared.stop_flag.load(Ordering::Relaxed) => {
                let pause = retry.pause_before(attempt + 1);
                emit(shared, &node.id, Step::retrying(error.at(&node.id), attempt, retry.attempts, pause));
                // Stop aborts this task, so the pause ends with it.
                tokio::time::sleep(pause).await;
                attempt += 1;
            }
            other => return other,
        }
    }
}

/// An action made again and again. Each send reads its templates afresh —
/// `{{counter}}` is its number, `{{now}}` its time — and is retried like a
/// single one; a send that fails for good fails the step. Progress is reported
/// at most once a second, and the step passes with the last send's outcome.
async fn repeated(shared: &EngineShared, env: &StepEnv<'_>, node: &Node, first: NodeKind, repeat: &Repeat, context: &mut BranchContext) -> EngineResult<StepOutcome> {
    let started = Instant::now();
    let mut reported = started;
    let mut kind = first;
    let mut sent = 0;
    loop {
        let mut outcome = attempts(shared, env, node, &kind, context).await?;
        sent += 1;
        let pause = repeat.pause_before(shared.seed, &node.id, sent + 1);
        // Another branch failing stops the run; this one sends no more either.
        if !repeat.more(sent, started.elapsed(), pause) || shared.stop_flag.load(Ordering::Relaxed) {
            outcome.message = Some(("exp.step.repeated", serde_json::json!({ "n": sent, "ms": started.elapsed().as_millis() as u64 })));
            return Ok(outcome);
        }
        if reported.elapsed() >= REPEAT_REPORT_EVERY {
            reported = Instant::now();
            emit(shared, &node.id, Step::repeating(sent, repeat, started.elapsed()));
        }
        // Stop aborts this task, so the pause ends with it.
        tokio::time::sleep(pause).await;
        kind = render(shared, node, context, shared.next_count(&node.id))?;
    }
}

/// One socket per distinct bind address, opened before the first step so a
/// reply that beats the wait is not lost. A port that cannot be opened stops
/// the run before any traffic, at the first wait that uses it. An OSC or UDP
/// emulator's port is one of them, with `taps` answering what arrives.
pub(crate) async fn arm_listeners(nodes: &[Node], sink: Arc<dyn FrameSink>, taps: &HashMap<SocketAddr, Arc<dyn Tap>>) -> EngineResult<Listeners> {
    let mut listeners = Listeners::new();
    for node in nodes {
        let NodeKind::Emulator { emulator } = &node.kind else { continue };
        let Some((address, tap)) = emulator.bind.trim().parse::<SocketAddr>().ok().and_then(|address| taps.get(&address).map(|tap| (address, tap))) else { continue };
        let listener = Listener::arm_with(address, sink.clone(), Some(tap.clone())).await.map_err(|error| error.in_field(Field::new("bind")).at(&node.id))?;
        listeners.insert(address, Arc::new(listener));
    }
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
pub(crate) async fn arm_subscriptions(host: &Host, nodes: &[Node], params: &BTreeMap<String, String>) -> EngineResult<Subscriptions> {
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

pub(crate) struct Prepared {
    pub(crate) seed: u64,
    pub(crate) params: BTreeMap<String, String>,
    pub(crate) secrets: BTreeMap<String, String>,
    pub(crate) listeners: Listeners,
    pub(crate) subscriptions: Subscriptions,
    pub(crate) http: HttpListeners,
    pub(crate) serving: RunServing,
    pub(crate) relays: HashMap<String, Arc<Relay>>,
    pub(crate) relay_serving: netsim_run::RunServing,
    pub(crate) emulations: HashMap<String, Arc<Emulation>>,
    /// The run fails with `run.timeout` after this long.
    pub(crate) limit: Duration,
}

/// Where the steps of a run go besides the host: its list, and its follower.
pub(crate) struct Followers {
    pub(crate) events: Arc<Mutex<Vec<RunEvent>>>,
    pub(crate) steps: mpsc::UnboundedSender<RunEvent>,
}

pub(crate) async fn run(host: &Host, followers: Followers, job_id: u64, doc: &Experiment, prepared: Prepared) -> Result<(), EngineError> {
    let mut joins = HashMap::new();
    for node in doc.nodes.iter().filter(|node| matches!(node.kind, NodeKind::Join)) {
        let inputs: Vec<String> = doc.edges.iter().filter(|edge| edge.to == node.id).map(|edge| edge.from.clone()).collect();
        joins.insert(node.id.clone(), JoinBarrier { expected: inputs.len(), arrived: 0, inputs, states: Vec::new() });
    }
    let limit = prepared.limit;
    let shared = Arc::new(EngineShared {
        host: host.clone(),
        job_id,
        events: followers.events,
        observer: followers.steps,
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
        masked: RwLock::new(prepared.secrets.values().cloned().collect()),
        derived: Mutex::new(Vec::new()),
        secrets: prepared.secrets,
        counts: Mutex::new(HashMap::new()),
        tasks: Mutex::new(TaskGuard::new()),
        listeners: Mutex::new(prepared.listeners),
        subscriptions: Mutex::new(prepared.subscriptions),
        http: prepared.http,
        serving: Mutex::new(prepared.serving),
        relays: prepared.relays,
        emulations: prepared.emulations,
        relay_serving: Mutex::new(prepared.relay_serving),
        websockets: RunSockets::default(),
        cookies: doc.cookies.then(CookieJar::new),
        started: Instant::now(),
    });
    let _cancel = CancelBranches(shared.clone());
    // The Inspector masks these values while the run lasts (or until it is stopped).
    let _redaction = secrets::redact(shared.secrets.values().cloned().collect());

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel(1);
    // validate_document guarantees exactly one Start.
    let start_id = doc.nodes.iter().find(|node| matches!(node.kind, NodeKind::Start)).map(|node| node.id.clone()).unwrap_or_default();
    let active_tasks = Arc::new(AtomicUsize::new(0));
    spawn_branch(shared.clone(), start_id.clone(), BranchContext::default(), active_tasks, done_tx.clone());
    drop(done_tx);

    if tokio::time::timeout(limit, done_rx.recv()).await.is_err() {
        shared.stop_flag.store(true, Ordering::SeqCst);
        let mut first = shared.first_error.lock().unwrap();
        if first.is_none() {
            *first = Some(EngineError::new("run.timeout").with("seconds", limit.as_secs()));
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
    let first = shared.first_error.lock().unwrap().clone();
    match first {
        Some(error) => Err(error.masked(&shared.masked.read().unwrap())),
        None => Ok(()),
    }
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
        let listeners = arm_listeners(&nodes, Arc::new(NoFrames), &HashMap::new()).await.unwrap();
        assert_eq!(listeners.len(), 1);
        drop(listeners);

        let busy = [wait_node("free", &free.to_string()), wait_node("busy", &taken.local_addr().unwrap().to_string())];
        let error = arm_listeners(&busy, Arc::new(NoFrames), &HashMap::new()).await.err().unwrap();
        assert_eq!((error.code.as_str(), error.node.as_deref(), error.field.as_ref().map(|field| field.key.clone())), ("transport.address_in_use", Some("busy"), Some("bind".to_string())));
    }
}
