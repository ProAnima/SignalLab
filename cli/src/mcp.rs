//! `signallab mcp`: Signal Lab for an LLM — a Model Context Protocol server
//! over stdio (JSON-RPC 2.0, one message per line; stdout carries nothing
//! else). An assistant in Claude Code, Claude Desktop, Cursor or VS Code can
//! learn what an experiment is made of, write one, check it, run it and read
//! why it failed; send one OSC message, datagram, HTTP request or MQTT
//! publish; listen for what a device sends; fire a library signal; play a
//! dependency with an emulator. With `--server` everything but `listen`
//! happens on a lab server, through its API.
//!
//! Every action goes through the engine's own commands, as in the app, and
//! every failure is the engine's `EngineError`, worded as the interface words
//! it. This module is the protocol; the tools are `mcp_tools` (schemas, the
//! experiment tools), `mcp_send` (one message, listening) and `mcp_library`
//! (signals and emulators).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use crate::fail::{Exit, Failure};
use crate::mcp_tools::{call, tools};
use crate::run::Engine;
use crate::{Ctx, McpArgs};

/// The protocol versions this server speaks, newest first.
const PROTOCOLS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "Signal Lab tests show-control and network gear: OSC, UDP, TCP, HTTP and MQTT. \
Every send and run puts real traffic on the network, so aim at the devices the user named (the bundled \
templates point at loopback). To build an experiment: describe_nodes explains the document and every node; \
get_template gives working examples; validate_experiment checks one without sending anything; \
run_experiment runs it and says, step by step, what happened and why it failed. send_* sends one message; \
listen shows what arrives on a port; list_signals and fire_signal use the user's signal library. To play a \
dependency the system under test calls — an HTTP API, an OSC, UDP or TCP device, an MQTT broker — start_emulator starts one \
(list_emulators names the user's), emulator_exchanges shows what it received and answered, stop_job ends it; \
inside an experiment, an emulator node and wait_http do the same for one run.";

pub(crate) struct State {
    pub(crate) ctx: Ctx,
    pub(crate) engine: Engine,
    pub(crate) library: Option<PathBuf>,
    /// The emulator library for list_emulators and start_emulator; `None`: the app's.
    pub(crate) emulators: Option<PathBuf>,
    /// The job of a run a request started, so cancelling the request stops it.
    pub(crate) runs: Mutex<HashMap<String, u64>>,
    pub(crate) out: mpsc::UnboundedSender<Value>,
}

/// What a tool call gives the model: words, the same as data, and whether it failed.
pub(crate) struct Answer {
    pub(crate) text: String,
    pub(crate) data: Value,
    pub(crate) error: bool,
}

impl Answer {
    pub(crate) fn ok(text: String, data: Value) -> Self {
        Answer { text, data, error: false }
    }

    pub(crate) fn failed(state: &State, failure: &Failure) -> Self {
        let lines = failure.lines(&state.ctx, "");
        Answer { text: lines.join("\n"), data: json!({ "error": failure.error, "exit_code": failure.exit.code() }), error: true }
    }

    /// Arguments the model got wrong: said in words it can correct.
    pub(crate) fn wrong(text: impl Into<String>) -> Self {
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
    // Read from the app's folder even when the runs go elsewhere.
    let emulators = args.emulators.clone().or_else(|| Some(signal_lab_engine::emulator_files::library_path()));
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
    let state = Arc::new(State { ctx, engine, library: args.library.clone(), emulators, runs: Mutex::default(), out: out.clone() });
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
