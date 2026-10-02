//! `signallab mcp`: Signal Lab for an LLM — a Model Context Protocol server
//! over stdio (JSON-RPC 2.0, one message per line; stdout carries nothing
//! else). An assistant in Claude Code, Claude Desktop, Cursor or VS Code can
//! learn what an experiment is made of, write one, check it, run it and read
//! why it failed; send one OSC message, datagram, HTTP request or MQTT
//! publish; listen for what a device sends; fire a library signal. With
//! `--server` everything but `listen` happens on a lab server, through its API.
//!
//! Every action goes through the engine's own commands, as in the app, and
//! every failure is the engine's `EngineError`, worded as the interface words it.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};
use signal_lab_engine::error::EngineError;
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_files;
use signal_lab_engine::experiment_run::RunOptions;
use signal_lab_engine::http::HttpResponse;
use signal_lab_engine::osc_codec::{decode_packet, OscArg};
use signal_lab_engine::signals::{RawPayload, SignalBody};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use crate::fail::{Exit, Failure};
use crate::i18n::TEMPLATES;
use crate::run::{failure_text, node_label, read_input, step_text, Engine, Ended, Input};
use crate::send::{datagram, find, hex_bytes, library, mqtt_publish, response_failure};
use crate::{catalog, params, Ctx, McpArgs};

/// The protocol versions this server speaks, newest first.
const PROTOCOLS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
/// The longest response body or capture handed to the model.
const MAX_TEXT: usize = 16 * 1024;
/// Listening is bounded: a tool call should come back while the model waits.
const MAX_LISTEN: Duration = Duration::from_secs(60);

const INSTRUCTIONS: &str = "Signal Lab tests show-control and network gear: OSC, UDP, TCP, HTTP and MQTT. \
Every send and run puts real traffic on the network, so aim at the devices the user named (the bundled \
templates point at loopback). To build an experiment: describe_nodes explains the document and every node; \
get_template gives working examples; validate_experiment checks one without sending anything; \
run_experiment runs it and says, step by step, what happened and why it failed. send_* sends one message; \
listen shows what arrives on a port; list_signals and fire_signal use the user's signal library.";

struct State {
    ctx: Ctx,
    engine: Engine,
    library: Option<PathBuf>,
    /// The job of a run a request started, so cancelling the request stops it.
    runs: Mutex<HashMap<String, u64>>,
    out: mpsc::UnboundedSender<Value>,
}

/// What a tool call gives the model: words, the same as data, and whether it failed.
struct Answer {
    text: String,
    data: Value,
    error: bool,
}

impl Answer {
    fn ok(text: String, data: Value) -> Self {
        Answer { text, data, error: false }
    }

    fn failed(state: &State, failure: &Failure) -> Self {
        let lines = failure.lines(&state.ctx, "");
        Answer { text: lines.join("\n"), data: json!({ "error": failure.error, "exit_code": failure.exit.code() }), error: true }
    }

    /// Arguments the model got wrong: said in words it can correct.
    fn wrong(text: impl Into<String>) -> Self {
        let text = text.into();
        Answer { data: json!({ "error": { "code": "mcp.arguments", "message": text } }), text, error: true }
    }
}

pub async fn serve(ctx: Ctx, args: McpArgs) -> Exit {
    if let Some(client) = &args.print_config {
        return print_config(client, &args);
    }
    // stdout is the protocol's: whatever a person should read goes to stderr.
    let ctx = Ctx { texts: ctx.texts, json: false };
    // The app's data folder unless told otherwise: an assistant's runs and their reports
    // land where the app keeps them, and stay after the session.
    let data_dir = args.data_dir.clone().or_else(|| args.place.server.is_none().then(signal_lab_engine::paths::data_dir));
    let engine = match Engine::new(&args.place, data_dir.as_deref()) {
        Ok(engine) => engine,
        Err(failure) => return failure.report(&ctx, "signallab mcp: "),
    };
    let (out, mut outgoing) = mpsc::unbounded_channel::<Value>();
    let writer = tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(message) = outgoing.recv().await {
            let mut line = message.to_string();
            line.push('\n');
            if stdout.write_all(line.as_bytes()).await.is_err() || stdout.flush().await.is_err() {
                break;
            }
        }
    });
    let state = Arc::new(State { ctx, engine, library: args.library.clone(), runs: Mutex::default(), out: out.clone() });
    let tasks: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>> = Arc::default();

    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        let message: Value = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(_) => {
                let _ = out.send(error(Value::Null, -32700, "Parse error: a message is one JSON object per line"));
                continue;
            }
        };
        // A batch (protocol 2025-03-26) is its messages one by one.
        let messages = match message {
            Value::Array(items) => items,
            other => vec![other],
        };
        for message in messages {
            handle(&state, &tasks, message).await;
        }
    }
    // The client closed stdin: what is still running finishes, then the process ends.
    let pending: Vec<_> = tasks.lock().unwrap().drain().map(|(_, task)| task).collect();
    for task in pending {
        let _ = task.await;
    }
    drop(state);
    drop(out);
    let _ = writer.await;
    Exit::Passed
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn key(id: &Value) -> String {
    id.to_string()
}

async fn handle(state: &Arc<State>, tasks: &Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>, message: Value) {
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return; // A response to nothing we asked, or garbage: nothing to answer.
    };
    let id = message.get("id").cloned();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    match (method, id) {
        ("initialize", Some(id)) => {
            let asked = params["protocolVersion"].as_str().unwrap_or_default();
            let version = PROTOCOLS.iter().find(|known| **known == asked).copied().unwrap_or(PROTOCOLS[0]);
            let _ = state.out.send(result(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": { "tools": { "listChanged": false } },
                    "serverInfo": { "name": "signallab", "title": "Signal Lab", "version": env!("CARGO_PKG_VERSION") },
                    "instructions": INSTRUCTIONS,
                }),
            ));
        }
        ("ping", Some(id)) => drop(state.out.send(result(id, json!({})))),
        ("tools/list", Some(id)) => drop(state.out.send(result(id, json!({ "tools": tools() })))),
        ("tools/call", Some(id)) => {
            let name = params["name"].as_str().unwrap_or_default().to_string();
            if !tools().iter().any(|tool| tool["name"] == name.as_str()) {
                let _ = state.out.send(error(id, -32602, &format!("Unknown tool: {name}")));
                return;
            }
            let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
            let progress = params["_meta"]["progressToken"].clone();
            let request = key(&id);
            let (state, registry, mine) = (state.clone(), tasks.clone(), request.clone());
            let task = tokio::spawn(async move {
                let answer = call(&state, &name, &arguments, &mine, progress).await;
                state.runs.lock().unwrap().remove(&mine);
                let mut content = json!({ "content": [{ "type": "text", "text": answer.text }], "isError": answer.error });
                if answer.data.is_object() {
                    content["structuredContent"] = answer.data;
                }
                let _ = state.out.send(result(id, content));
                registry.lock().unwrap().remove(&mine);
            });
            // Kept so a cancellation can stop it; a call that is already done just leaves a finished handle.
            tasks.lock().unwrap().insert(request, task);
        }
        ("notifications/cancelled", None) => {
            let request = key(&params["requestId"]);
            let job = state.runs.lock().unwrap().remove(&request);
            if let Some(job) = job {
                let _ = state.engine.invoke("job_stop", json!({ "id": job })).await;
            }
            if let Some(task) = tasks.lock().unwrap().remove(&request) {
                task.abort();
            }
        }
        // Other notifications (initialized, progress, roots…) need nothing from us.
        (_, None) => {}
        (_, Some(id)) => drop(state.out.send(error(id, -32601, &format!("Method not found: {method}")))),
    }
}

// ---- the tools ------------------------------------------------------------------

fn experiment_source() -> Value {
    json!({
        "document": { "type": "object", "description": "An experiment document (see describe_nodes), as the app saves it." },
        "file": { "type": "string", "description": "Path of an experiment file on the machine signallab runs on." },
        "template": { "type": "string", "description": "Name of a bundled template (list_templates)." },
        "params": { "type": "object", "additionalProperties": { "type": "string" }, "description": "Parameter values for this run: {name: value}." },
        "profile": { "type": "string", "description": "Run with this profile of the document; \"\" for the defaults." },
    })
}

fn tool(name: &str, title: &str, description: &str, properties: Value, required: &[&str], read_only: bool) -> Value {
    json!({
        "name": name,
        "title": title,
        "description": description,
        "inputSchema": { "type": "object", "properties": properties, "required": required, "additionalProperties": false },
        "annotations": { "title": title, "readOnlyHint": read_only, "openWorldHint": !read_only, "destructiveHint": false, "idempotentHint": read_only },
    })
}

fn tools() -> Vec<Value> {
    let mut run = experiment_source();
    run["seed"] = json!({ "type": "integer", "minimum": 0, "description": "Seed of the random values; a failed run reports the one it used." });
    run["timeout"] = json!({ "type": "integer", "minimum": 1, "maximum": 300, "description": "Seconds the run may take (default 300)." });
    vec![
        tool("describe_nodes", "Describe experiment nodes", "How an experiment document is put together, every kind of node with its fields, outputs and an example, and the {{template}} language. Read this before writing an experiment.", json!({}), &[], true),
        tool("list_templates", "List templates", "The bundled experiments, by name, with their parameters.", json!({}), &[], true),
        tool("get_template", "Get a template", "One bundled experiment as a document, to run or adapt.", json!({ "name": { "type": "string" } }), &["name"], true),
        tool("validate_experiment", "Validate an experiment", "Check an experiment as the editor does before a run — graph, fields, templates, parameters, secrets — without sending anything. Give one of document, file or template.", experiment_source(), &[], true),
        tool("run_experiment", "Run an experiment", "Run an experiment to its end and report every step: what was sent, what came back, and why it failed. Sends real traffic. Give one of document, file or template.", run, &[], false),
        tool("send_osc", "Send OSC", "Send one OSC message. args: numbers (whole → int, else float), strings, booleans, null, or {type: int|float|str|long|double|bool|blob|nil, value}.",
            json!({ "target": { "type": "string", "description": "host:port" }, "address": { "type": "string", "description": "/osc/address" }, "args": { "type": "array" } }), &["target", "address"], false),
        tool("send_udp", "Send UDP", "Send one UDP datagram: text, or hex bytes.",
            json!({ "target": { "type": "string", "description": "host:port" }, "text": { "type": "string" }, "hex": { "type": "string", "description": "e.g. \"de ad be ef\"" } }), &["target"], false),
        tool("send_http", "Send HTTP", "Send one HTTP request; returns the status, the time, the headers and the body (the first 16 KB).",
            json!({ "method": { "type": "string", "description": "GET, POST, …" }, "url": { "type": "string" }, "headers": { "type": "object", "additionalProperties": { "type": "string" } },
                    "body": { "type": "string" }, "timeout_ms": { "type": "integer", "minimum": 1 } }), &["method", "url"], false),
        tool("send_mqtt", "Publish MQTT", "Publish one MQTT 3.1.1 message. An empty payload with retain clears a retained value.",
            json!({ "broker": { "type": "string", "description": "host:port (1883 when no port)" }, "topic": { "type": "string" }, "payload": { "type": "string" },
                    "qos": { "type": "integer", "enum": [0, 1, 2] }, "retain": { "type": "boolean" } }), &["broker", "topic"], false),
        tool("listen", "Listen on a port", "Listen on a UDP port of the machine signallab runs on for a while and return what arrived: OSC messages decoded, other datagrams as text and hex. Not with --server.",
            json!({ "protocol": { "type": "string", "enum": ["osc", "udp"] }, "bind": { "type": "string", "description": "IP:port, e.g. 0.0.0.0:9000" },
                    "seconds": { "type": "number", "minimum": 0.1, "maximum": 60 }, "max": { "type": "integer", "minimum": 1, "maximum": 1000 } }), &["bind"], false),
        tool("list_signals", "List signals", "The signals of the user's library (the app's signals.json, or the file given), with what each one sends.",
            json!({ "library": { "type": "string", "description": "Path of a library file" } }), &[], true),
        tool("fire_signal", "Fire a signal", "Send a library signal, by its id or name, exactly as the app fires it.",
            json!({ "signal": { "type": "string" }, "library": { "type": "string" } }), &["signal"], false),
        tool("list_jobs", "List jobs", "What is running: monitors, generators, runs.", json!({}), &[], true),
        tool("stop_job", "Stop a job", "Stop a running job by its id.", json!({ "id": { "type": "integer" } }), &["id"], false),
    ]
}

async fn call(state: &State, name: &str, arguments: &Value, request: &str, progress: Value) -> Answer {
    match name {
        "describe_nodes" => {
            let catalogue = catalog::describe(&state.ctx.texts);
            Answer::ok(serde_json::to_string_pretty(&catalogue).unwrap_or_default(), catalogue)
        }
        "list_templates" => list_templates(),
        "get_template" => match TEMPLATES.iter().find(|(name, _)| Some(*name) == arguments["name"].as_str().map(|name| name.trim_end_matches(".json"))) {
            Some((_, text)) => {
                let document: Value = serde_json::from_str(text).unwrap_or_default();
                Answer::ok(text.to_string(), json!({ "document": document }))
            }
            None => Answer::wrong(format!("No template {} — list_templates names them", arguments["name"])),
        },
        "validate_experiment" => validate(state, arguments).await,
        "run_experiment" => run(state, arguments, request, progress).await,
        "send_osc" => send_osc(state, arguments).await,
        "send_udp" => send_udp(state, arguments).await,
        "send_http" => send_http(state, arguments).await,
        "send_mqtt" => send_mqtt(state, arguments).await,
        "listen" => listen(state, arguments).await,
        "list_signals" => list_signals(state, arguments),
        "fire_signal" => fire(state, arguments).await,
        "list_jobs" => match state.engine.invoke("jobs_list", json!({})).await {
            Ok(jobs) => {
                let lines: Vec<String> = jobs.as_array().into_iter().flatten().map(|job| format!("#{} {} — {}", job["id"], job["kind"].as_str().unwrap_or_default(), job["label"].as_str().unwrap_or_default())).collect();
                let text = if lines.is_empty() { "Nothing is running.".to_string() } else { lines.join("\n") };
                Answer::ok(text, json!({ "jobs": jobs }))
            }
            Err(failure) => Answer::failed(state, &failure),
        },
        "stop_job" => match arguments["id"].as_u64() {
            Some(id) => match state.engine.invoke("job_stop", json!({ "id": id })).await {
                Ok(stopped) => Answer::ok(if stopped == Value::Bool(true) { format!("Stopped job #{id}.") } else { format!("No job #{id} is running.") }, json!({ "stopped": stopped })),
                Err(failure) => Answer::failed(state, &failure),
            },
            None => Answer::wrong("id is the number list_jobs shows"),
        },
        _ => Answer::wrong(format!("Unknown tool {name}")),
    }
}

fn list_templates() -> Answer {
    let mut lines = Vec::new();
    let mut list = Vec::new();
    for (name, text) in TEMPLATES {
        let Ok(document) = experiment_files::parse(text) else { continue };
        let params: Vec<String> = document.params.iter().map(|param| format!("{}={}", param.name, param.value)).collect();
        lines.push(format!("{name} — {}{}", document.name, if params.is_empty() { String::new() } else { format!(" (params: {})", params.join(", ")) }));
        list.push(json!({ "name": name, "experiment": document.name, "params": document.params, "nodes": document.nodes.len() }));
    }
    Answer::ok(lines.join("\n"), json!({ "templates": list }))
}

/// The experiment a call names, its profile applied, and its parameter values.
fn source(state: &State, arguments: &Value) -> Result<(Input, BTreeMap<String, String>), Answer> {
    let given = ["document", "file", "template"].iter().filter(|name| arguments.get(**name).is_some_and(|value| !value.is_null())).count();
    if given != 1 {
        return Err(Answer::wrong("Give exactly one of document, file or template"));
    }
    let failed = |failure: Failure| Answer::failed(state, &failure);
    let mut input = if let Some(document) = arguments.get("document").filter(|value| !value.is_null()) {
        let document = experiment_files::parse(&document.to_string()).map_err(|error| failed(Failure::invalid(error)))?;
        Input { label: document.name.clone(), document }
    } else {
        let label = arguments["file"].as_str().or(arguments["template"].as_str()).unwrap_or_default();
        read_input(label).map_err(failed)?
    };
    if let Some(profile) = arguments["profile"].as_str() {
        input.document.profile = (!profile.is_empty()).then(|| profile.to_string());
    }
    let mut overrides = BTreeMap::new();
    if let Some(given) = arguments["params"].as_object() {
        for (name, value) in given {
            let value = match value {
                Value::String(text) => text.clone(),
                Value::Number(_) | Value::Bool(_) => value.to_string(),
                _ => return Err(Answer::wrong(format!("params.{name} is a string, number or boolean"))),
            };
            overrides.insert(name.clone(), value);
        }
    }
    Ok((input, overrides))
}

/// A failure that stopped a call, worded where the document names its node.
fn refused(state: &State, document: &Experiment, failure: &Failure) -> Answer {
    let (text, detail) = failure_text(&state.ctx.texts, document, &failure.error, failure.remote);
    let text = match &detail {
        Some(detail) => format!("{text}\n{}: {detail}", state.ctx.texts.plain("err.details")),
        None => text,
    };
    Answer { text, data: json!({ "error": failure.error, "exit_code": failure.exit.code() }), error: true }
}

async fn validate(state: &State, arguments: &Value) -> Answer {
    let (input, overrides) = match source(state, arguments) {
        Ok(source) => source,
        Err(answer) => return answer,
    };
    match state.engine.validate(&input.document, &overrides).await {
        Ok(issues) => {
            let mut text = format!("{} would run.", input.document.name);
            for issue in &issues {
                let Ok(error) = serde_json::from_value::<EngineError>(issue["error"].clone()) else { continue };
                let profile = issue["profile"].as_str().unwrap_or("defaults");
                text.push_str(&format!("\nWith profile {profile} it would not: {}", failure_text(&state.ctx.texts, &input.document, &error, state.engine.remote()).0));
            }
            Answer::ok(text, json!({ "valid": true, "experiment": input.document.name, "profile_issues": issues }))
        }
        Err(failure) => refused(state, &input.document, &failure),
    }
}

async fn run(state: &State, arguments: &Value, request: &str, progress: Value) -> Answer {
    let (input, overrides) = match source(state, arguments) {
        Ok(source) => source,
        Err(answer) => return answer,
    };
    let options = RunOptions { overrides, seed: arguments["seed"].as_u64(), limit: arguments["timeout"].as_u64().map(Duration::from_secs) };
    let texts = &state.ctx.texts;
    let document = input.document.clone();
    let mut count = 0u64;
    let outcome = state
        .engine
        .run(input.document, options, &mut |kind, value| match kind {
            "started" => {
                if let Some(job) = value["job_id"].as_u64() {
                    state.runs.lock().unwrap().insert(request.to_string(), job);
                }
            }
            // The model's client shows these while the run goes on, when it asked for them.
            "step" if !progress.is_null() => {
                count += 1;
                if let Ok(step) = serde_json::from_value::<crate::run::Step>(value.clone()) {
                    let message = format!("{} · {}", node_label(texts, &document, &step.node_id), texts.plain(&format!("exp.{}", step.state)));
                    let _ = state.out.send(json!({ "jsonrpc": "2.0", "method": "notifications/progress", "params": { "progressToken": progress, "progress": count, "message": message } }));
                }
            }
            _ => {}
        })
        .await;
    let value = match outcome {
        Ok(value) => value,
        Err(failure) => return refused(state, &document, &failure),
    };
    let Ok(ended) = serde_json::from_value::<Ended>(value.clone()) else {
        return Answer::wrong("the run ended with an answer this signallab cannot read");
    };
    let duration = crate::run::duration(texts, ended.ended_ms.saturating_sub(ended.started_ms));
    let mut text = match ended.outcome.as_str() {
        "passed" => format!("PASSED · {} · {duration} · seed {}", ended.experiment, ended.seed),
        "stopped" => format!("STOPPED · {} · after {duration}", ended.experiment),
        _ => {
            let why = ended.error.as_ref().map(|error| failure_text(texts, &document, error, state.engine.remote())).map(|(text, detail)| match detail {
                Some(detail) => format!("{text}\n{}: {detail}", texts.plain("err.details")),
                None => text,
            });
            format!("FAILED · {} · after {duration} · seed {}\n{}", ended.experiment, ended.seed, why.unwrap_or_default())
        }
    };
    text.push_str("\n\nSteps:");
    let first = ended.steps.first().map(|step| step.ts).unwrap_or_default();
    let mut steps = Vec::new();
    for step in &ended.steps {
        let label = node_label(texts, &document, &step.node_id);
        let said = step_text(texts, step);
        steps.push(json!({ "node_id": step.node_id, "node": label, "state": step.state, "text": said, "ms": step.ts.saturating_sub(first) }));
        // The words keep what each step came to; "running" is in the data.
        if step.state == "running" {
            continue;
        }
        text.push_str(&format!("\n{:>7.3}s  {label} ({})  {}{}", step.ts.saturating_sub(first) as f64 / 1000.0, step.node_id, step.state, if said.is_empty() { String::new() } else { format!(" · {said}") }));
    }
    if let Some(path) = &ended.report_path {
        text.push_str(&format!("\n\nReport: {path}"));
    }
    let data = json!({
        "outcome": ended.outcome, "experiment": ended.experiment, "seed": ended.seed, "duration_ms": ended.ended_ms.saturating_sub(ended.started_ms),
        "error": ended.error, "steps": steps, "report_path": ended.report_path,
    });
    // A run that failed is an answer, not a failed call: the steps say why.
    Answer::ok(text, data)
}

/// OSC arguments as JSON: numbers, strings, booleans, null, or {type, value}.
fn osc_args(given: &Value) -> Result<Vec<OscArg>, String> {
    let Some(items) = given.as_array() else {
        return if given.is_null() { Ok(Vec::new()) } else { Err("args is an array".into()) };
    };
    items
        .iter()
        .map(|item| match item {
            Value::Bool(flag) => Ok(OscArg::Bool(*flag)),
            Value::Null => Ok(OscArg::Nil),
            Value::String(text) => Ok(OscArg::Str(text.clone())),
            Value::Number(number) => match number.as_i64() {
                Some(int) => Ok(i32::try_from(int).map(OscArg::Int).unwrap_or(OscArg::Long(int))),
                None => Ok(OscArg::Float(number.as_f64().unwrap_or_default() as f32)),
            },
            Value::Object(_) => serde_json::from_value(item.clone()).map_err(|error| format!("{item} is not an OSC argument: {error}")),
            Value::Array(_) => Err(format!("{item} is not an OSC argument")),
        })
        .collect()
}

fn text_arg<'a>(arguments: &'a Value, name: &str) -> Result<&'a str, Answer> {
    arguments[name].as_str().filter(|text| !text.trim().is_empty()).ok_or_else(|| Answer::wrong(format!("{name} is required")))
}

async fn send_osc(state: &State, arguments: &Value) -> Answer {
    let (target, address) = match (text_arg(arguments, "target"), text_arg(arguments, "address")) {
        (Ok(target), Ok(address)) => (target, address),
        (Err(answer), _) | (_, Err(answer)) => return answer,
    };
    let args = match osc_args(&arguments["args"]) {
        Ok(args) => args,
        Err(text) => return Answer::wrong(text),
    };
    match state.engine.invoke("osc_send", json!({ "target": target, "address": address, "args": args })).await {
        Ok(bytes) => Answer::ok(
            state.ctx.texts.t("log.oscSent", &params! { "address" => address, "bytes" => bytes.as_u64().unwrap_or_default(), "target" => target }),
            json!({ "sent": true, "bytes": bytes }),
        ),
        Err(failure) => Answer::failed(state, &failure),
    }
}

async fn udp_send(state: &State, target: &str, payload: &RawPayload) -> Result<Value, Failure> {
    let result = state.engine.invoke("broadcast_send", datagram(target, payload)).await?;
    if result["errors"].as_u64().unwrap_or_default() > 0 {
        let error = serde_json::from_value::<EngineError>(result["error"].clone()).unwrap_or_else(|_| EngineError::new("transport.failed").with("target", target));
        return Err(Failure::sending(error));
    }
    Ok(result)
}

async fn send_udp(state: &State, arguments: &Value) -> Answer {
    let target = match text_arg(arguments, "target") {
        Ok(target) => target,
        Err(answer) => return answer,
    };
    let payload = match (arguments["text"].as_str(), arguments["hex"].as_str()) {
        (Some(text), None) => RawPayload::Text { text: text.into() },
        (None, Some(hex)) if hex_bytes(hex).is_some() => RawPayload::Hex { hex: hex.into() },
        (None, Some(hex)) => return Answer::wrong(format!("{hex} is not pairs of hex digits")),
        _ => return Answer::wrong("Give text or hex, one of them"),
    };
    match udp_send(state, target, &payload).await {
        Ok(result) => Answer::ok(state.ctx.texts.t("cli.sentUdp", &params! { "target" => target, "bytes" => result["bytes"].as_u64().unwrap_or_default() }), json!({ "sent": true, "result": result })),
        Err(failure) => Answer::failed(state, &failure),
    }
}

/// At most `MAX_TEXT` bytes of `text`, on a character boundary, said when cut.
fn clip(text: &str) -> String {
    if text.len() <= MAX_TEXT {
        return text.to_string();
    }
    let mut end = MAX_TEXT;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n… ({} more bytes)", &text[..end], text.len() - end)
}

async fn http(state: &State, request: Value, url: &str) -> Result<HttpResponse, Failure> {
    let value = state.engine.invoke("http_request", json!({ "request": request })).await?;
    let response: HttpResponse = serde_json::from_value(value).map_err(|error| Failure::sending(EngineError::new("command.reply_invalid").because(error)))?;
    match response_failure(&response, url) {
        Some(error) => Err(Failure { error, exit: Exit::Failed, remote: false }),
        None => Ok(response),
    }
}

async fn send_http(state: &State, arguments: &Value) -> Answer {
    let (method, url) = match (text_arg(arguments, "method"), text_arg(arguments, "url")) {
        (Ok(method), Ok(url)) => (method.to_ascii_uppercase(), url),
        (Err(answer), _) | (_, Err(answer)) => return answer,
    };
    let headers: Vec<(String, String)> = arguments["headers"].as_object().into_iter().flatten().map(|(name, value)| (name.clone(), value.as_str().map(str::to_string).unwrap_or_else(|| value.to_string()))).collect();
    let request = json!({ "method": method, "url": url, "headers": headers, "body": arguments["body"].as_str(), "timeout_ms": arguments["timeout_ms"].as_u64().unwrap_or(10_000) });
    match http(state, request, url).await {
        Ok(response) => {
            let head: Vec<String> = response.headers.iter().map(|(name, value)| format!("{name}: {value}")).collect();
            let text = format!("HTTP {} {} · {} ms · {} bytes\n{}\n\n{}", response.status, response.status_text, response.latency_ms.round(), response.body_bytes, head.join("\n"), clip(&response.body));
            let data = json!({ "status": response.status, "status_text": response.status_text, "latency_ms": response.latency_ms, "headers": response.headers, "body": clip(&response.body), "body_bytes": response.body_bytes });
            Answer::ok(text, data)
        }
        Err(failure) => Answer::failed(state, &failure),
    }
}

async fn send_mqtt(state: &State, arguments: &Value) -> Answer {
    let (broker, topic) = match (text_arg(arguments, "broker"), text_arg(arguments, "topic")) {
        (Ok(broker), Ok(topic)) => (broker, topic),
        (Err(answer), _) | (_, Err(answer)) => return answer,
    };
    let qos = arguments["qos"].as_u64().unwrap_or(0);
    if qos > 2 {
        return Answer::wrong("qos is 0, 1 or 2");
    }
    let args = match mqtt_publish(broker, topic, arguments["payload"].as_str().unwrap_or_default(), qos as u8, arguments["retain"].as_bool().unwrap_or(false)) {
        Ok(args) => args,
        Err(failure) => return Answer::failed(state, &failure),
    };
    match state.engine.invoke("mqtt_publish_once", args).await {
        Ok(summary) => Answer::ok(format!("Published {}", summary.as_str().unwrap_or_default()), json!({ "sent": true, "summary": summary })),
        Err(failure) => Answer::failed(state, &failure),
    }
}

async fn listen(state: &State, arguments: &Value) -> Answer {
    if state.engine.remote() {
        return Answer::wrong("listen runs on this machine; on a server, an experiment with a wait node listens there (run_experiment)");
    }
    let bind = match text_arg(arguments, "bind") {
        Ok(bind) => bind,
        Err(answer) => return answer,
    };
    let osc = arguments["protocol"].as_str().unwrap_or("osc") == "osc";
    let seconds = Duration::from_secs_f64(arguments["seconds"].as_f64().unwrap_or(5.0).clamp(0.1, MAX_LISTEN.as_secs_f64()));
    let max = arguments["max"].as_u64().unwrap_or(100).clamp(1, 1000) as usize;
    let address: std::net::SocketAddr = match bind.parse() {
        Ok(address) => address,
        Err(_) => return Answer::failed(state, &Failure::invalid(EngineError::new("node.bind_invalid").with("value", bind))),
    };
    let socket = match tokio::net::UdpSocket::bind(address).await {
        Ok(socket) => socket,
        Err(error) => return Answer::failed(state, &Failure::environment(signal_lab_engine::net::bind_error(bind, error))),
    };
    let started = Instant::now();
    let mut received = Vec::new();
    let mut buffer = vec![0u8; 65_536];
    while received.len() < max {
        let left = seconds.saturating_sub(started.elapsed());
        if left.is_zero() {
            break;
        }
        let (n, from) = match tokio::time::timeout(left, socket.recv_from(&mut buffer)).await {
            Ok(Ok(got)) => got,
            // Windows tells of an earlier send's "port unreachable" on the next receive: carry on.
            Ok(Err(error)) if signal_lab_engine::net::udp_transient(&error) => continue,
            Ok(Err(_)) | Err(_) => break,
        };
        let bytes = &buffer[..n];
        let ms = started.elapsed().as_millis() as u64;
        let entry = match osc.then(|| decode_packet(bytes)).and_then(Result::ok) {
            Some(messages) => json!({ "ms": ms, "from": from.to_string(), "osc": messages.iter().map(|message| json!({ "address": message.address, "args": message.args })).collect::<Vec<_>>() }),
            None => json!({ "ms": ms, "from": from.to_string(), "bytes": n, "text": String::from_utf8_lossy(bytes), "hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" ") }),
        };
        received.push(entry);
    }
    let mut text = format!("{} on {bind} in {:.1} s", match received.len() { 0 => "Nothing arrived".to_string(), 1 => "1 datagram".to_string(), n => format!("{n} datagrams") }, started.elapsed().as_secs_f64());
    if received.is_empty() && address.ip().is_unspecified() {
        text.push_str(". If the sender is another machine, check that the firewall lets UDP in on this port (signallab doctor).");
    }
    for entry in &received {
        let what = match entry.get("osc") {
            Some(messages) => messages.as_array().into_iter().flatten().map(|message| format!("{} {}", message["address"].as_str().unwrap_or_default(), message["args"])).collect::<Vec<_>>().join("; "),
            None => format!("{:?}", entry["text"].as_str().unwrap_or_default()),
        };
        text.push_str(&format!("\n{:>6} ms  {}  {what}", entry["ms"], entry["from"].as_str().unwrap_or_default()));
    }
    Answer::ok(clip(&text), json!({ "received": received }))
}

fn list_signals(state: &State, arguments: &Value) -> Answer {
    let path = arguments["library"].as_str().map(PathBuf::from).or_else(|| state.library.clone());
    match library(path) {
        Ok((path, library)) => {
            let mut lines = vec![format!("{} signals in {}", library.signals.len(), path.display())];
            let mut list = Vec::new();
            for signal in &library.signals {
                let (transport, what) = describe_body(&signal.body);
                let folder = if signal.group.is_empty() { String::new() } else { format!("{} / ", signal.group) };
                lines.push(format!("{folder}{} [{}] — {transport} {what}", signal.name, signal.id));
                list.push(json!({ "id": signal.id, "name": signal.name, "folder": signal.group, "note": signal.note, "transport": transport, "sends": what }));
            }
            Answer::ok(lines.join("\n"), json!({ "path": path.display().to_string(), "signals": list }))
        }
        Err(failure) => Answer::failed(state, &failure),
    }
}

fn describe_body(body: &SignalBody) -> (&'static str, String) {
    match body {
        SignalBody::Osc { target, address, args } => ("osc", format!("{address} {} → {target}", args.iter().map(signal_lab_engine::osc_codec::arg_str).collect::<Vec<_>>().join(" "))),
        SignalBody::Udp { target, payload } => ("udp", match payload {
            RawPayload::Text { text } => format!("{text:?} → {target}"),
            RawPayload::Hex { hex } => format!("hex {hex} → {target}"),
        }),
        SignalBody::Http { request } => ("http", format!("{} {}", request.method, request.url)),
        SignalBody::Mqtt { broker, topic, payload, .. } => ("mqtt", format!("{topic} = {payload:?} → {broker}")),
    }
}

async fn fire(state: &State, arguments: &Value) -> Answer {
    let wanted = match text_arg(arguments, "signal") {
        Ok(wanted) => wanted,
        Err(answer) => return answer,
    };
    let path = arguments["library"].as_str().map(PathBuf::from).or_else(|| state.library.clone());
    let (path, library) = match library(path) {
        Ok(found) => found,
        Err(failure) => return Answer::failed(state, &failure),
    };
    let signal = match find(&library, wanted, &path) {
        Ok(signal) => signal.clone(),
        Err(failure) => return Answer::failed(state, &failure),
    };
    let texts = &state.ctx.texts;
    let name = signal.name.as_str();
    let fired = match &signal.body {
        SignalBody::Osc { target, address, args } => state.engine.invoke("osc_send", json!({ "target": target, "address": address, "args": args })).await.map(|bytes| {
            texts.t("log.firedOsc", &params! { "name" => name, "address" => address.as_str(), "target" => target.as_str(), "bytes" => bytes.as_u64().unwrap_or_default() })
        }),
        SignalBody::Udp { target, payload } => udp_send(state, target, payload)
            .await
            .map(|result| texts.t("log.firedUdp", &params! { "name" => name, "target" => target.as_str(), "bytes" => result["bytes"].as_u64().unwrap_or_default() })),
        SignalBody::Http { request } => http(state, serde_json::to_value(request).unwrap_or_default(), &request.url).await.map(|response| {
            texts.t("log.firedHttp", &params! { "name" => name, "method" => request.method.as_str(), "url" => request.url.as_str(), "status" => u64::from(response.status), "ms" => response.latency_ms.round() as u64 })
        }),
        SignalBody::Mqtt { broker, topic, payload, qos, retain } => match mqtt_publish(broker, topic, payload, *qos, *retain) {
            Ok(args) => state.engine.invoke("mqtt_publish_once", args).await.map(|summary| texts.t("log.firedMqttOnce", &params! { "name" => name, "summary" => summary.as_str().unwrap_or_default() })),
            Err(failure) => Err(failure),
        },
    };
    match fired {
        Ok(line) => Answer::ok(line, json!({ "fired": signal.id })),
        Err(failure) => Answer::failed(state, &failure),
    }
}

// ---- how to hook it up ------------------------------------------------------------

/// The configuration a client needs, with this executable's own path.
fn print_config(client: &str, args: &McpArgs) -> Exit {
    let exe = std::env::current_exe().map(|path| path.display().to_string()).unwrap_or_else(|_| "signallab".into());
    let mut command_args = vec!["mcp".to_string()];
    if let Some(server) = &args.place.server {
        command_args.extend(["--server".into(), server.clone()]);
    }
    let mut env = Map::new();
    if args.place.server.is_some() {
        env.insert("SIGNALLAB_TOKEN".into(), "<the server's token>".into());
    }
    let server = json!({ "command": exe, "args": command_args, "env": env });
    match client {
        "claude-code" => {
            let env = if args.place.server.is_some() { " -e SIGNALLAB_TOKEN=<the server's token>" } else { "" };
            println!("claude mcp add signallab{env} -- \"{exe}\" {}", command_args.join(" "));
        }
        "claude-desktop" | "cursor" => println!("{}", serde_json::to_string_pretty(&json!({ "mcpServers": { "signallab": server } })).unwrap_or_default()),
        "vscode" => {
            let mut server = server;
            server["type"] = "stdio".into();
            println!("{}", serde_json::to_string_pretty(&json!({ "servers": { "signallab": server } })).unwrap_or_default());
        }
        _ => {
            eprintln!("signallab mcp --print-config: claude-code, claude-desktop, cursor or vscode");
            return Exit::Invalid;
        }
    }
    Exit::Passed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc_arguments_from_json() {
        let args = osc_args(&json!([1, 2.5, "x", true, null, 5_000_000_000_i64, { "type": "double", "value": 0.125 }])).unwrap();
        assert_eq!(
            serde_json::to_value(args).unwrap(),
            json!([
                { "type": "int", "value": 1 }, { "type": "float", "value": 2.5 }, { "type": "str", "value": "x" }, { "type": "bool", "value": true },
                { "type": "nil" }, { "type": "long", "value": 5_000_000_000_i64 }, { "type": "double", "value": 0.125 }
            ])
        );
        assert!(osc_args(&json!([[1]])).is_err() && osc_args(&json!("x")).is_err());
        assert!(osc_args(&Value::Null).unwrap().is_empty());
    }

    #[test]
    fn every_tool_has_a_schema_a_client_can_read() {
        for tool in tools() {
            assert!(tool["name"].as_str().unwrap().chars().all(|char| char.is_ascii_lowercase() || char == '_'), "{tool}");
            assert_eq!(tool["inputSchema"]["type"], "object");
            assert!(tool["description"].as_str().unwrap().len() > 20);
            for required in tool["inputSchema"]["required"].as_array().unwrap() {
                assert!(tool["inputSchema"]["properties"].get(required.as_str().unwrap()).is_some(), "{tool}");
            }
        }
        assert_eq!(clip(&"é".repeat(MAX_TEXT)).lines().last().unwrap(), format!("… ({} more bytes)", MAX_TEXT));
    }
}
