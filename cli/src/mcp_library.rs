//! The tools that use the person's libraries: signals (list_signals,
//! fire_signal, sent exactly as the app fires them) and emulators
//! (list_emulators, start_emulator, emulator_exchanges).

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{json, Value};
use signal_lab_engine::error::EngineError;
use signal_lab_engine::signals::{RawPayload, SignalBody};

use crate::fail::Failure;
use crate::mcp::{Answer, State};
use crate::mcp_send::{clip, http, text_arg, udp_send};
use crate::params;
use crate::send::{find, library, mqtt_publish};

pub(crate) fn list_signals(state: &State, arguments: &Value) -> Answer {
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

pub(crate) async fn fire(state: &State, arguments: &Value) -> Answer {
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

// ---- emulators --------------------------------------------------------------------

fn emulator_library_path(state: &State, arguments: &Value) -> Option<PathBuf> {
    arguments["library"].as_str().map(PathBuf::from).or_else(|| state.emulators.clone())
}

pub(crate) fn list_emulators(state: &State, arguments: &Value) -> Answer {
    match crate::emulate::library(emulator_library_path(state, arguments).as_deref()) {
        Ok((path, library)) => {
            let mut lines = vec![format!("{} emulators in {}", library.emulators.len(), path.display())];
            let mut list = Vec::new();
            for stored in &library.emulators {
                let emulator = &stored.emulator;
                lines.push(format!("{} [{}] — {} on {}, {} rules{}", emulator.name, stored.id, emulator.kind.protocol(), emulator.bind, emulator.kind.rules(), if stored.note.is_empty() { String::new() } else { format!(" — {}", stored.note) }));
                list.push(json!({ "id": stored.id, "note": stored.note, "emulator": emulator }));
            }
            Answer::ok(lines.join("\n"), json!({ "path": path.display().to_string(), "emulators": list }))
        }
        Err(failure) => Answer::failed(state, &failure),
    }
}

pub(crate) async fn start_emulator(state: &State, arguments: &Value) -> Answer {
    let (mut emulator, source) = match (arguments.get("emulator").filter(|value| !value.is_null()), arguments["name"].as_str()) {
        (Some(document), None) => match serde_json::from_value::<signal_lab_engine::emulator::Emulator>(document.clone()) {
            Ok(emulator) => (emulator, None),
            Err(error) => return Answer::wrong(format!("emulator is not an emulator document: {error} — describe_nodes shows the shape under \"emulators\"")),
        },
        (None, Some(name)) => {
            let (path, library) = match crate::emulate::library(emulator_library_path(state, arguments).as_deref()) {
                Ok(found) => found,
                Err(failure) => return Answer::failed(state, &failure),
            };
            match signal_lab_engine::emulator_files::find(&library, name) {
                Some(stored) => (stored.emulator.clone(), Some(stored.id.clone())),
                None => return Answer::failed(state, &Failure::invalid(EngineError::new("cli.emulator_unknown").with("name", name).with("path", path.display()))),
            }
        }
        _ => return Answer::wrong("Give emulator (a document) or name (from the library), one of them"),
    };
    if let Some(bind) = arguments["bind"].as_str() {
        emulator.bind = bind.to_string();
    }
    let params: BTreeMap<String, String> = arguments["params"].as_object().into_iter().flatten().map(|(name, value)| (name.clone(), value.as_str().map(str::to_string).unwrap_or_else(|| value.to_string()))).collect();
    let request = json!({ "emulator": emulator, "params": params, "seed": arguments["seed"].as_u64(), "source": source });
    match state.engine.invoke("emulator_start", request).await {
        Ok(job) => {
            let (id, local) = (job["id"].as_u64().unwrap_or_default(), job["params"]["local"].as_str().unwrap_or_default().to_string());
            let protocol = emulator.kind.protocol();
            let url = (protocol == "http").then(|| format!("http://{local}"));
            let text = format!(
                "{} ({protocol}) is answering on {local}{} — job #{id}. emulator_exchanges with job_id {id} says what arrives; stop_job ends it.",
                emulator.name,
                url.as_ref().map(|url| format!(" ({url})")).unwrap_or_default()
            );
            Answer::ok(text, json!({ "job_id": id, "name": emulator.name, "protocol": protocol, "local": local, "url": url }))
        }
        Err(failure) => Answer::failed(state, &crate::emulate::starting(failure)),
    }
}

pub(crate) async fn emulator_exchanges(state: &State, arguments: &Value) -> Answer {
    let Some(id) = arguments["job_id"].as_u64() else { return Answer::wrong("job_id is the number start_emulator gave") };
    match state.engine.invoke("emulator_exchanges", json!({ "jobId": id, "after": arguments["after"].as_u64() })).await {
        Ok(snapshot) => {
            let mut text = crate::emulate::counts_line(&state.ctx.texts, snapshot["name"].as_str().unwrap_or_default(), &snapshot["counts"]);
            for exchange in snapshot["exchanges"].as_array().into_iter().flatten() {
                let rule = exchange["rule"].as_u64().map(|rule| format!("#{rule}")).unwrap_or_else(|| "no rule".into());
                let answered = match (exchange.get("error").filter(|error| !error.is_null()), exchange["fault"].as_str(), exchange["reply"].as_str().unwrap_or_default()) {
                    (Some(error), _, _) => {
                        let error: EngineError = serde_json::from_value(error.clone()).unwrap_or_else(|_| EngineError::new("emulator.failed"));
                        format!("failed: {}", state.ctx.texts.describe(&error, &["cli.err."], &|_| None).text)
                    }
                    (None, Some(fault), _) => fault.to_string(),
                    (None, None, "") => "no reply".to_string(),
                    (None, None, reply) => reply.to_string(),
                };
                text.push_str(&format!("\n[{}] {} {rule} → {answered} · {} ms ← {}", exchange["seq"], exchange["request"].as_str().unwrap_or_default(), exchange["ms"], exchange["from"].as_str().unwrap_or_default()));
            }
            Answer::ok(clip(&text), snapshot)
        }
        Err(failure) => Answer::failed(state, &failure),
    }
}

pub(crate) async fn set_emulator_down(state: &State, arguments: &Value) -> Answer {
    let (Some(id), Some(down)) = (arguments["job_id"].as_u64(), arguments["down"].as_bool()) else {
        return Answer::wrong("job_id is the number start_emulator gave, down is true or false");
    };
    let fault = arguments["fault"].as_str().unwrap_or("unavailable");
    match state.engine.invoke("emulator_down", json!({ "jobId": id, "down": down, "fault": fault })).await {
        Ok(_) if down => Answer::ok(format!("Emulator job #{id} is down until set_emulator_down brings it up (HTTP meets {fault})."), json!({ "job_id": id, "down": true, "fault": fault })),
        Ok(_) => Answer::ok(format!("Emulator job #{id} answers again."), json!({ "job_id": id, "down": false })),
        Err(failure) => Answer::failed(state, &failure),
    }
}
