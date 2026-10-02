//! The tools `signallab mcp` offers: their schemas, which one a call runs, and
//! the experiment tools — the catalogue, templates, validate and run.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::error::EngineError;
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_files;
use signal_lab_engine::experiment_run::RunOptions;

use crate::catalog;
use crate::fail::Failure;
use crate::i18n::TEMPLATES;
use crate::mcp::{Answer, State};
use crate::mcp_library::{emulator_exchanges, fire, list_emulators, list_signals, start_emulator};
use crate::mcp_send::{listen, send_http, send_mqtt, send_osc, send_udp};
use crate::run::{failure_text, node_label, read_input, step_text, Ended, Input};

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

pub(crate) fn tools() -> Vec<Value> {
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
        tool("list_emulators", "List emulators", "The emulators of the user's library (the app's emulators.json, or the file given): what each one plays, where it listens, how many rules. describe_nodes (\"emulators\") explains the document.",
            json!({ "library": { "type": "string", "description": "Path of an emulator library file" } }), &[], true),
        tool("start_emulator", "Start an emulator", "Play the other side — an HTTP API, an OSC, UDP or TCP device, an MQTT broker — until stop_job: open its port and answer by its rules. Give an emulator document (describe_nodes, \"emulators\") or the id or name of one in the library. Returns the job id; emulator_exchanges says what arrived.",
            json!({ "emulator": { "type": "object", "description": "An emulator document: {name, bind, protocol, …}" }, "name": { "type": "string", "description": "Id or name of one in the library" },
                    "library": { "type": "string", "description": "Path of an emulator library file" }, "bind": { "type": "string", "description": "Listen here instead, IP:port" },
                    "params": { "type": "object", "additionalProperties": { "type": "string" }, "description": "Values its templates read as parameters" }, "seed": { "type": "integer", "minimum": 0 } }), &[], false),
        tool("emulator_exchanges", "What an emulator received", "The requests a running emulator received and what it answered, rule by rule, each with what it carried (method, path, headers, body, JSON; address and arguments; text). after: only those after this sequence number.",
            json!({ "job_id": { "type": "integer" }, "after": { "type": "integer", "minimum": 0 } }), &["job_id"], true),
        tool("list_jobs", "List jobs", "What is running: monitors, generators, emulators, runs.", json!({}), &[], true),
        tool("stop_job", "Stop a job", "Stop a running job by its id.", json!({ "id": { "type": "integer" } }), &["id"], false),
    ]
}

pub(crate) async fn call(state: &State, name: &str, arguments: &Value, request: &str, progress: Value) -> Answer {
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
        "list_emulators" => list_emulators(state, arguments),
        "start_emulator" => start_emulator(state, arguments).await,
        "emulator_exchanges" => emulator_exchanges(state, arguments).await,
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
    // What the emulators of the run were asked: the proof the system under test called them.
    if !ended.emulators.is_empty() {
        text.push_str("\n\nEmulators:");
        for emulator in &ended.emulators {
            text.push_str(&format!("\n  {}", crate::emulate::counts_line(texts, emulator["name"].as_str().unwrap_or_default(), &emulator["counts"])));
        }
    }
    if let Some(path) = &ended.report_path {
        text.push_str(&format!("\n\nReport: {path}"));
    }
    let data = json!({
        "outcome": ended.outcome, "experiment": ended.experiment, "seed": ended.seed, "duration_ms": ended.ended_ms.saturating_sub(ended.started_ms),
        "error": ended.error, "steps": steps, "emulators": ended.emulators, "report_path": ended.report_path,
    });
    // A run that failed is an answer, not a failed call: the steps say why.
    Answer::ok(text, data)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
