//! Hostile input at the engine's front: every command of `Service::invoke`
//! walked with arguments a careless or malicious client could send — wrong
//! types, huge and negative numbers, `NaN` as text, 10 MB strings, deep
//! nesting, unknown and missing fields. Each must come back as an
//! `EngineError` whose code the interface can word (`err.<code>` in en.ts):
//! never a panic, never a hang. The command list is read from service.rs
//! itself, so a new command is walked without anyone adding it here.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::service::Failure;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-hostile-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        // Nothing here may reach the studio's hub: a closed loopback port instead.
        std::env::set_var("SIGNALLAB_HUB_URL", "http://127.0.0.1:9");
        dir
    })
    .clone()
}

fn service() -> Arc<Service> {
    scratch_data_dir();
    let host = Host::new(Recorder::new(), Capture::new());
    Arc::new(Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))))
}

/// One command arm of `Service::invoke`: its name and its arguments (camelCase).
struct Command {
    name: String,
    fields: Vec<String>,
}

fn camel(name: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// The command table, read from its source: `"name" => …` arms and the
/// fields of their `args!(command, value, { … })`.
fn commands() -> Vec<Command> {
    let source = include_str!("../src/service.rs");
    let body = &source[source.find("match command {").expect("the command table")..source.find("#[cfg(test)]").unwrap()];
    let arm = regex::Regex::new(r#"(?m)^\s*"([a-z_]+)" =>"#).unwrap();
    let starts: Vec<(usize, String)> = arm.captures_iter(body).map(|c| (c.get(0).unwrap().start(), c[1].to_string())).collect();
    let mut commands = Vec::new();
    for (index, (start, name)) in starts.iter().enumerate() {
        let end = starts.get(index + 1).map(|(next, _)| *next).unwrap_or(body.len());
        let text = &body[*start..end];
        let mut fields = Vec::new();
        if let Some(at) = text.find("args!(command, value, {") {
            let inner = &text[at + "args!(command, value, {".len()..];
            let inner = &inner[..inner.find("})").unwrap()];
            // Split on commas outside `<…>`.
            let (mut depth, mut current) = (0i32, String::new());
            for c in inner.chars().chain([',']) {
                match c {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    ',' if depth == 0 => {
                        if let Some((field, _)) = current.split_once(':') {
                            fields.push(camel(field.trim()));
                        }
                        current.clear();
                        continue;
                    }
                    _ => {}
                }
                current.push(c);
            }
        }
        commands.push(Command { name: name.clone(), fields });
    }
    commands
}

fn nested(depth: usize) -> Value {
    (0..depth).fold(json!(1), |inner, level| if level % 2 == 0 { json!([inner]) } else { json!({ "a": inner }) })
}

/// What a client could put where any one value belongs.
fn hostile_values() -> Vec<(&'static str, Value)> {
    vec![
        ("null", Value::Null),
        ("true", json!(true)),
        ("-1", json!(-1)),
        ("i64::MIN", json!(i64::MIN)),
        ("u64::MAX", json!(u64::MAX)),
        ("2^64 as a float", json!(18_446_744_073_709_551_616.0f64)),
        ("1e308", json!(1e308)),
        ("0.5", json!(0.5)),
        ("\"NaN\"", json!("NaN")),
        ("\"-Infinity\"", json!("-Infinity")),
        ("empty string", json!("")),
        ("control characters", json!("a\u{0}b\r\nX-Injected: 1\u{1b}[31m")),
        ("path escape", json!("../../../../etc/passwd")),
        ("10 MB string", Value::String("x".repeat(10 * 1024 * 1024))),
        ("empty array", json!([])),
        ("empty object", json!({})),
        ("an object of junk", json!({ "__proto__": { "polluted": true }, "constructor": 1, "": null })),
        ("120 levels deep", nested(120)),
        ("100 000 items", Value::Array(vec![json!(0); 100_000])),
    ]
}

/// The error codes the interface has words for.
fn known_codes() -> BTreeSet<String> {
    let en = include_str!("../../src/lib/locales/en.ts");
    let key = regex::Regex::new(r#""err\.([a-z0-9_.]+)":"#).unwrap();
    key.captures_iter(en).map(|c| c[1].to_string()).collect()
}

/// Invoke on its own task, so a panic is caught and a hang is timed out.
async fn attempt(service: &Arc<Service>, command: &str, args: Value) -> Result<Result<Value, Failure>, String> {
    let (service, name) = (service.clone(), command.to_string());
    let task = tokio::spawn(async move { service.invoke(&name, args).await });
    match tokio::time::timeout(Duration::from_secs(10), task).await {
        Err(_) => Err("did not answer within 10 s".into()),
        Ok(Err(join)) if join.is_panic() => Err(format!("panicked: {join}")),
        Ok(Err(join)) => Err(format!("task failed: {join}")),
        Ok(Ok(reply)) => Ok(reply),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_command_survives_hostile_arguments() {
    let service = service();
    let commands = commands();
    assert!(commands.len() >= 55, "the command table was read: {} commands", commands.len());
    assert!(commands.iter().any(|c| c.name == "experiment_start" && c.fields == ["document", "overrides", "seed"]), "fields are read");
    let codes = known_codes();
    let values = hostile_values();
    let mut problems = Vec::new();
    let mut tried = 0;
    let started = Instant::now();
    for command in &commands {
        // Whole argument lists that are not an object of the command's fields.
        let mut cases: Vec<(String, Value)> = vec![
            ("an array".into(), json!([1, 2])),
            ("a string".into(), json!("args")),
            ("a number".into(), json!(7)),
            ("an unknown field".into(), json!({ "unexpected": 1 })),
            ("deep nesting".into(), json!({ "a": nested(120) })),
        ];
        for field in &command.fields {
            for (label, value) in &values {
                // One field hostile, the others missing.
                cases.push((format!("{field} = {label}"), json!({ field.as_str(): value })));
            }
            cases.push((format!("{field} twice, in another case"), json!({ field.as_str(): 1, field.to_uppercase(): 1 })));
        }
        for (label, value) in &values {
            // Every field hostile at once.
            let all: serde_json::Map<String, Value> = command.fields.iter().map(|field| (field.clone(), value.clone())).collect();
            if !all.is_empty() {
                cases.push((format!("every field = {label}"), Value::Object(all)));
            }
        }
        for (label, args) in cases {
            tried += 1;
            match attempt(&service, &command.name, args).await {
                Err(problem) => problems.push(format!("{} with {label}: {problem}", command.name)),
                Ok(Err(Failure::Engine(error))) if !codes.contains(&error.code) => {
                    problems.push(format!("{} with {label}: code {} has no err.* text", command.name, error.code))
                }
                Ok(Err(_)) => {}
                // A command without arguments refuses arguments it does not know, like every other.
                Ok(Ok(_)) if command.fields.is_empty() && label == "an unknown field" => {
                    problems.push(format!("{} with {label}: accepted an argument it does not take", command.name))
                }
                Ok(Ok(_)) => {}
            }
        }
        service.jobs().stop_all();
    }
    assert!(problems.is_empty(), "{} of {tried} hostile calls went wrong:\n{}", problems.len(), problems.join("\n"));
    assert!(service.jobs().list().is_empty(), "nothing hostile left a job running");
    eprintln!("{tried} hostile calls over {} commands in {:?}", commands.len(), started.elapsed());
}

/// A loopback port that accepts connections and closes them at once, so a
/// command aimed at it fails fast (a refused connect takes ~2 s on Windows).
async fn closing_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            drop(stream);
        }
    });
    port
}

/// Arguments each command accepts as they are: the starting point of the mutations.
async fn baselines(service: &Arc<Service>) -> Vec<(&'static str, Value)> {
    let sink = closing_port().await;
    let tcp = format!("127.0.0.1:{sink}");
    // The starter's target is a port nothing listens on: here one that answers at once.
    let document: Value = serde_json::from_str(&serde_json::to_string(&signal_lab_engine::experiment::starter()).unwrap().replace("127.0.0.1:8080", &tcp)).unwrap();
    // Feedback goes to a hub that answers at once too.
    std::env::set_var("SIGNALLAB_HUB_URL", format!("http://{tcp}"));
    let node = document["nodes"][1]["id"].clone();
    let library = service.invoke("signals_load", Value::Null).await.unwrap()["library"].clone();
    let emulators = service.invoke("emulators_load", Value::Null).await.unwrap()["library"].clone();
    let mut emulator = emulators["emulators"][0]["emulator"].clone();
    emulator["bind"] = json!("127.0.0.1:0");
    let request = json!({ "method": "POST", "url": format!("http://{tcp}/x?y=1"), "headers": [["X-A", "b"]], "body": "{}", "timeout_ms": 500 });
    let mqtt = json!({ "host": "127.0.0.1", "port": sink, "client_id": "hostile", "username": "u", "password": "p", "keep_alive_s": 5,
        "will": { "topic": "t", "payload": "w", "qos": 1, "retain": false }, "subscribe": [{ "filter": "a/#", "qos": 2 }] });
    let ws = json!({ "url": format!("ws://{tcp}/"), "headers": [["X-A", "b"]], "protocols": ["chat"], "timeout_ms": 500 });
    let profile = json!({ "name": "p", "latency_ms": 5, "jitter_ms": 1, "loss": 0.1, "duplicate": 0.1, "corrupt": 0.1, "reorder": 0.1,
        "rate_kbps": 64, "burst_start": 0.1, "burst_length": 3, "offline": false, "reset": 0.0, "stall": 0.0 });
    let payload = json!({ "kind": "text", "text": "PING" });
    vec![
        ("job_stop", json!({ "id": 1 })),
        ("firewall_allow", json!({ "public": false })),
        ("feedback_send", json!({ "form": { "message": "hello", "email": "a@b.c", "meta": { "os": "x" }, "screenshots": [{ "name": "a.png", "data": "AAAA" }], "logs": [{ "name": "a.log", "text": "x" }] } })),
        ("experiment_save", json!({ "document": document })),
        ("experiment_parse", json!({ "text": document.to_string() })),
        ("experiment_export", json!({ "document": document })),
        ("experiment_validate", json!({ "document": document, "overrides": { "a": "b" } })),
        ("experiment_resolve", json!({ "document": document, "nodeId": node, "vars": { "v": 1 } })),
        ("experiment_send_node", json!({ "document": document, "nodeId": node, "vars": { "v": 1 } })),
        ("experiment_start", json!({ "document": document, "overrides": {}, "seed": 7 })),
        ("experiment_runs", json!({ "name": "experiment", "limit": 5 })),
        ("experiment_compare", json!({ "a": "run-1.json", "b": "run-2.json" })),
        ("secret_status", json!({ "names": ["A", "B"] })),
        ("secret_set", json!({ "name": "A", "value": "v" })),
        ("secret_delete", json!({ "name": "A" })),
        ("osc_send", json!({ "target": "127.0.0.1:9", "address": "/a", "args": [{ "type": "int", "value": 1 }, { "type": "str", "value": "s" }, { "type": "blob", "value": [0, 255] }] })),
        ("osc_monitor_start", json!({ "bind": "127.0.0.1:0" })),
        ("osc_generator_start", json!({ "config": { "target": "127.0.0.1:9", "address": "/g", "rate": 10, "waveform": "sine", "freq": 1, "min": 0, "max": 1, "as_int": false, "duration_s": 1 } })),
        ("http_request", json!({ "request": request, "cookies": true })),
        ("http_burst_start", json!({ "config": { "method": "GET", "url": format!("http://{tcp}/"), "headers": [], "body": null, "timeout_ms": 500, "concurrency": 2, "total": 4, "duration_s": 1, "rate": 10, "cookies": false } })),
        ("netsim_start", json!({ "config": { "listen": "127.0.0.1:0", "target": "127.0.0.1:9", "profile": profile, "seed": 1, "protocol": "udp" } })),
        ("netsim_set_profile", json!({ "jobId": 1, "profile": profile })),
        ("storm_start", json!({ "config": { "target": "127.0.0.1:9", "protocol": "udp", "size": 16, "rate": 10, "duration_s": 1 } })),
        ("scan_start", json!({ "config": { "host": "127.0.0.1", "port_start": sink, "port_end": sink, "concurrency": 4, "timeout_ms": 100, "grab_banner": true } })),
        ("broadcast_send", json!({ "config": { "mode": "list", "target": "127.0.0.1:9", "port": 0, "payload": payload, "bind": null, "ttl": 1, "multicast_loop": true } })),
        ("broadcast_beacon_start", json!({ "config": { "mode": "list", "target": "127.0.0.1:9", "payload": payload, "rate": 2, "count": 1, "duration_s": 1 } })),
        ("discovery_start", json!({ "config": { "bind": "127.0.0.1:0", "groups": [], "reuse": true, "respond": true, "response": payload, "respond_delay_ms": 1, "match_contains": "P" } })),
        ("inspect_set_enabled", json!({ "enabled": true })),
        ("inspect_snapshot", json!({ "limit": 5 })),
        ("inspect_export", json!({ "format": "jsonl" })),
        ("inspect_payload", json!({ "seq": 1 })),
        ("mqtt_connect", json!({ "config": mqtt })),
        ("mqtt_publish", json!({ "jobId": 1, "topic": "a", "payload": "b", "qos": 1, "retain": false })),
        ("mqtt_subscribe", json!({ "jobId": 1, "filters": [{ "filter": "a/+", "qos": 1 }] })),
        ("mqtt_unsubscribe", json!({ "jobId": 1, "filters": ["a/+"] })),
        ("mqtt_publish_once", json!({ "config": mqtt, "topic": "a", "payload": "b", "qos": 2, "retain": true })),
        ("ws_connect", json!({ "config": ws })),
        ("ws_send", json!({ "jobId": 1, "message": { "text": "hi" } })),
        ("ws_close", json!({ "jobId": 1, "code": 1000, "reason": "bye" })),
        ("ws_exchange", json!({ "config": ws, "message": { "hex": "de ad" }, "expect": { "mode": "regex", "pattern": "a+", "timeout_ms": 100 } })),
        ("signals_save", json!({ "library": library })),
        ("emulators_save", json!({ "library": emulators })),
        ("emulator_check", json!({ "emulator": emulator, "params": { "a": "b" } })),
        ("emulator_start", json!({ "emulator": emulator, "params": {}, "seed": 1, "source": "x" })),
        ("emulator_exchanges", json!({ "jobId": 1, "after": 0, "limit": 5 })),
        ("emulator_down", json!({ "jobId": 1, "down": true, "fault": "reset" })),
    ]
}

/// Places tried per command, spread over all of them.
const PLACES: usize = 48;

/// Every place in a JSON value, as the steps to it.
fn places(value: &Value, at: &mut Vec<Value>, out: &mut Vec<Vec<Value>>) {
    if !at.is_empty() {
        out.push(at.clone());
    }
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                at.push(json!(key));
                places(item, at, out);
                at.pop();
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                at.push(json!(index));
                places(item, at, out);
                at.pop();
            }
        }
        _ => {}
    }
}

fn replaced(value: &Value, at: &[Value], with: &Value) -> Value {
    let mut copy = value.clone();
    let mut target = &mut copy;
    for step in at {
        target = match step {
            Value::String(key) => &mut target[key.as_str()],
            index => &mut target[index.as_u64().unwrap() as usize],
        };
    }
    *target = with.clone();
    copy
}

/// The same walk one level down: arguments that parse, with one value in them
/// made hostile — so the checks past parsing are reached, not only serde's.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_value_inside_valid_arguments_can_be_hostile() {
    let service = service();
    let codes = known_codes();
    let commands: BTreeSet<String> = commands().into_iter().filter(|c| !c.fields.is_empty()).map(|c| c.name).collect();
    let baselines = baselines(&service).await;
    let covered: BTreeSet<String> = baselines.iter().map(|(name, _)| name.to_string()).collect();
    assert_eq!(covered, commands, "every command with arguments has a baseline here");
    let values: Vec<(&str, Value)> = vec![
        ("null", Value::Null),
        ("true", json!(true)),
        ("-1", json!(-1)),
        ("i64::MIN", json!(i64::MIN)),
        ("u64::MAX", json!(u64::MAX)),
        ("2^64 as a float", json!(18_446_744_073_709_551_616.0f64)),
        ("1e308", json!(1e308)),
        ("0.5", json!(0.5)),
        ("\"NaN\"", json!("NaN")),
        ("empty string", json!("")),
        ("control characters", json!("a\u{0}b\r\nX-Injected: 1")),
        ("1 MB of {{a}}", Value::String("{{a}}".repeat(200 * 1024))),
        ("empty array", json!([])),
        ("empty object", json!({})),
        ("60 levels deep", nested(60)),
    ];
    let (mut problems, mut tried) = (Vec::new(), 0);
    let started = Instant::now();
    for (command, baseline) in &baselines {
        // The baseline itself parses: anything else would make the walk shallow.
        if let Ok(Err(Failure::Engine(error))) = attempt(&service, command, baseline.clone()).await {
            assert_ne!(error.code, "command.args_invalid", "{command}'s baseline does not parse: {:?}", error.detail);
        }
        service.jobs().stop_all();
        let mut found = Vec::new();
        places(baseline, &mut Vec::new(), &mut found);
        // A document has hundreds of places: an even spread of them keeps the walk quick.
        let stride = found.len().div_ceil(PLACES);
        let began_command = Instant::now();
        for at in found.into_iter().step_by(stride.max(1)) {
            for (label, value) in &values {
                tried += 1;
                let args = replaced(baseline, &at, value);
                let place = at.iter().map(|step| step.to_string().trim_matches('"').to_string()).collect::<Vec<_>>().join(".");
                let began = Instant::now();
                match attempt(&service, command, args).await {
                    Err(problem) => problems.push(format!("{command}: {place} = {label}: {problem}")),
                    Ok(Err(Failure::Engine(error))) if !codes.contains(&error.code) => {
                        problems.push(format!("{command}: {place} = {label}: code {} has no err.* text", error.code))
                    }
                    Ok(_) => {}
                }
                // Bounded is not enough when it is long: an answer within seconds.
                if began.elapsed() > Duration::from_secs(8) {
                    problems.push(format!("{command}: {place} = {label}: took {:?}", began.elapsed()));
                }
                service.jobs().stop_all();
            }
        }
        eprintln!("{command}: {:?}", began_command.elapsed());
    }
    assert!(problems.is_empty(), "{} of {tried} hostile calls went wrong:\n{}", problems.len(), problems.join("\n"));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(service.jobs().list().is_empty(), "nothing hostile left a job running: {:?}", service.jobs().list());
    eprintln!("{tried} hostile values in valid arguments in {:?}", started.elapsed());
}

/// Runs `work` on a thread of its own and says whether it ended within `limit`:
/// a hang fails the test instead of stalling it.
fn within<T: Send + 'static>(limit: Duration, work: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (done, finished) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = done.send(work());
    });
    finished.recv_timeout(limit).ok()
}

fn scope<'a>(params: &'a std::collections::BTreeMap<String, String>, vars: &'a std::collections::BTreeMap<String, Value>, secrets: &'a std::collections::BTreeMap<String, String>) -> signal_lab_engine::template::Scope<'a> {
    signal_lab_engine::template::Scope { params, vars, secrets, run_id: 1, seed: 1, node_id: "n", count: 1, now_ms: 0 }
}

/// A field from a file is parsed in time proportional to its length, however
/// many `{{…}}` it holds — the position each expression's error would name is
/// counted as the parser goes, not from the start each time.
#[test]
fn a_template_with_many_expressions_parses_in_linear_time() {
    let text = "{{a}}".repeat(200_000);
    let parsed = within(Duration::from_secs(5), move || {
        let began = Instant::now();
        let parsed = signal_lab_engine::template::parse(&text).map(|_| ());
        (parsed, began.elapsed())
    });
    let (parsed, took) = parsed.expect("1 MB of {{a}} parsed within 5 s");
    assert!(parsed.is_ok() && took < Duration::from_secs(2), "{parsed:?} in {took:?}");
    // An error's position is still the character it is at.
    let error = signal_lab_engine::template::parse("ой {{a}} {{ }}").unwrap_err();
    assert_eq!((error.code.as_str(), error.params["position"].as_str()), ("template.empty", "10"));
}

/// A field rendered from a large value many times over would take memory
/// without bound — 64 KiB parameter × 800 000 references is 50 GB. A rendered
/// field has a size limit, and passing it is an error, not an allocation.
#[test]
fn a_rendered_field_cannot_grow_without_bound() {
    use std::collections::BTreeMap;
    let params = BTreeMap::from([("big".to_string(), "x".repeat(64 * 1024))]);
    let (vars, secrets) = (BTreeMap::new(), BTreeMap::new());
    let text = "{{big}}".repeat(600_000);
    let rendered = signal_lab_engine::template::Renderer::new(scope(&params, &vars, &secrets)).render(&text);
    let error = rendered.expect_err("refused before it was built");
    assert_eq!(error.code, "template.too_large");
    // Up to the limit, rendering is as before.
    let fits = "{{big}}".repeat(signal_lab_engine::template::MAX_RENDERED / (64 * 1024));
    assert_eq!(signal_lab_engine::template::Renderer::new(scope(&params, &vars, &secrets)).render(&fits).unwrap().len(), fits.len() / 7 * 64 * 1024);
}

/// OSC address patterns come from a document; the addresses they are matched
/// against come from the network. Matching must stay polynomial however the
/// wildcards are combined: a datagram must never stall a listener.
#[test]
fn osc_patterns_match_in_polynomial_time() {
    use signal_lab_engine::matching::OscPattern;
    let cases = [
        ("/a*b*c*d", format!("/a{}", "b".repeat(20_000))),
        ("/*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*b", format!("/{}", "a".repeat(60))),
        ("/{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}{a,aa}b", format!("/{}", "a".repeat(40))),
        ("/*?*?*?*?*?*?*?*?*?*?*?*?*?*?*?*?*?*?*?*?*!", format!("/{}", "?".repeat(0) + &"x".repeat(200))),
    ];
    for (pattern, address) in cases {
        let compiled = OscPattern::parse(pattern).unwrap_or_else(|error| panic!("{pattern}: {error:?}"));
        let shown = pattern.to_string();
        let matched = within(Duration::from_secs(5), move || compiled.matches(&address));
        assert_eq!(matched, Some(false), "{shown} against a hostile address did not answer within 5 s");
    }
    // And the answers are still OSC 1.0's.
    let check = |pattern: &str, address: &str| OscPattern::parse(pattern).unwrap().matches(address);
    assert!(check("/a*b*c*d", "/aXbYcZd") && check("/*", "/") && check("/a*", "/a") && !check("/a?", "/a"));
    assert!(check("/{ping,pong}/[0-9]*", "/pong/7x") && !check("/{ping,pong}/[!0-9]", "/ping/3"));
    assert!(check("/{a,aa}{a,aa}b", "/aaab") && check("/{a,aa}{a,aa}b", "/aab") && !check("/{a,aa}{a,aa}b", "/ab"));
    assert!(check("/*/x", "/anything/x") && !check("/*", "/a/b"), "a wildcard never crosses a slash");
}

/// Patterns from a document (extract, `matches`, emulator rules) are compiled
/// by a regex engine that never backtracks: the classic catastrophic patterns
/// match a hostile input in linear time, and one too big to build is refused.
#[test]
fn regular_expressions_cannot_be_made_to_backtrack() {
    use signal_lab_engine::matching::compile_regex;
    let input = format!("{}!", "a".repeat(64 * 1024));
    for pattern in ["(a+)+$", "(a|a)*$", "(a|aa)+$", "^(([a-z])+.)+[A-Z]([a-z])+$", "(.*a){20}$"] {
        let compiled = compile_regex(pattern).unwrap();
        let input = input.clone();
        let shown = pattern.to_string();
        let matched = within(Duration::from_secs(5), move || compiled.is_match(&input));
        assert!(matched.is_some(), "{shown} against 64 KiB of a's did not answer within 5 s");
    }
    for pattern in ["(?:a{1000}){1000}", r"\w{1000}{1000}", "((((a{100}){100}){100}){100})"] {
        let shown = pattern.to_string();
        let compiled = within(Duration::from_secs(5), move || compile_regex(&shown).map(|_| ()));
        let error = compiled.expect("compiled or refused within 5 s").expect_err(pattern);
        assert_eq!(error.code, "regex.invalid", "{pattern}");
    }
}

/// SplitMix64 for the decoders below: the same bytes every run.
struct Bytes(u64);

impl Bytes {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// `base` with a few bytes changed, cut, or grown: what a broken or hostile peer sends.
    fn mutate(&mut self, base: &[u8]) -> Vec<u8> {
        let mut out = base.to_vec();
        for _ in 0..1 + self.next() % 4 {
            let at = if out.is_empty() { 0 } else { (self.next() as usize) % out.len() };
            match self.next() % 5 {
                0 if !out.is_empty() => out[at] = self.next() as u8,
                1 if !out.is_empty() => out[at] = [0x00, 0x7f, 0x80, 0xff][(self.next() % 4) as usize],
                2 => out.truncate(at),
                3 => out.insert(at.min(out.len()), self.next() as u8),
                _ => out.extend((0..self.next() % 16).map(|_| self.next() as u8)),
            }
        }
        out
    }
}

/// Every decoder of bytes from the network, fed mutations of valid packets and
/// random bytes: an error or a value, never a panic.
#[test]
fn decoders_survive_mutated_packets() {
    use signal_lab_engine::{mqtt_codec, osc_codec};
    let osc_message = osc_codec::encode_message("/a/b", &[osc_codec::OscArg::Int(1), osc_codec::OscArg::Str("s".into()), osc_codec::OscArg::Blob(vec![1, 2, 3]), osc_codec::OscArg::Double(0.5)]);
    let mut bundle = b"#bundle\0\0\0\0\0\0\0\0\x01".to_vec();
    bundle.extend((osc_message.len() as u32).to_be_bytes());
    bundle.extend(&osc_message);
    let connect = mqtt_codec::encode_connect(&mqtt_codec::ConnectOpts { client_id: "c", username: Some("u"), password: Some("p"), keep_alive_s: 30, clean_session: true, will: None });
    let publish = mqtt_codec::encode_publish("a/b", b"payload", 2, true, 7, false);
    let subscribe = mqtt_codec::encode_subscribe(3, &[("a/+".into(), 1), ("#".into(), 2)]);
    let seeds: Vec<Vec<u8>> = vec![osc_message, bundle, connect, publish, subscribe, mqtt_codec::encode_pubrel(5), mqtt_codec::encode_connack(false, 0), vec![]];
    let mut random = Bytes(0x5eed);
    let survived = std::panic::catch_unwind(move || {
        for round in 0..20_000 {
            let base = &seeds[round % seeds.len()];
            let bytes = if round % 7 == 0 { (0..random.next() % 64).map(|_| random.next() as u8).collect() } else { random.mutate(base) };
            let _ = osc_codec::decode_packet(&bytes);
            let _ = mqtt_codec::decode(&bytes);
            let _ = mqtt_codec::decode_client(&bytes, 1 << 20);
            let text = String::from_utf8_lossy(&bytes);
            let _ = signal_lab_engine::template::parse(&text);
            let _ = signal_lab_engine::template::parse_json_path(&text);
            let _ = signal_lab_engine::matching::OscPattern::parse(&format!("/{text}")).map(|pattern| pattern.matches("/a/b"));
            let _ = signal_lab_engine::emulator_match::PathPattern::parse(&format!("/{text}")).map(|pattern| pattern.matches("/a/b"));
            let _ = signal_lab_engine::subscribe::topic_matches(&text, "a/b/c");
            let _ = signal_lab_engine::emulator_match::percent_decode(&text, true);
        }
    });
    assert!(survived.is_ok(), "a decoder panicked on hostile bytes");
}

// ---- secrets ------------------------------------------------------------------

/// A token of the kind APIs hand out: base64 with `+`, `/` and `=`, so a URL,
/// a form, JSON and Basic's base64 each write it differently.
const TOKEN: &str = "Zm9v+YmFy/cXV4==";
/// A password with a quote and a space: a URL percent-encodes it, JSON escapes it.
const PASSWORD: &str = "pa\"ss wd";

fn secret_service() -> (Arc<Service>, Arc<Recorder>) {
    scratch_data_dir();
    let recorder = Recorder::new();
    let store = FileStore::new(None).with_env(|key| match key {
        "SIGNALLAB_SECRET_TOKEN" => Some(TOKEN.to_string()),
        "SIGNALLAB_SECRET_PASSWORD" => Some(PASSWORD.to_string()),
        _ => None,
    });
    (Arc::new(Service::new(Host::new(recorder.clone(), Capture::new()), Mode::Server, Arc::new(store))), recorder)
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

/// The forms a secret takes on its way through HTTP and back: as written, in a
/// URL (percent-encoded, or a query's `+` read as a space), in JSON, in base64.
fn leaked_forms(text: &str) -> Vec<&'static str> {
    use base64::Engine as _;
    let forms: Vec<(&'static str, String)> = vec![
        ("the token as written", TOKEN.to_string()),
        ("the token percent-encoded", TOKEN.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D")),
        ("the token with a query's + read as a space", TOKEN.replace('+', " ")),
        ("the token in base64", base64::engine::general_purpose::STANDARD.encode(TOKEN)),
        ("Basic's base64 of user:token", base64::engine::general_purpose::STANDARD.encode(format!("lab:{TOKEN}"))),
        ("the password as written", PASSWORD.to_string()),
        ("the password in a URL", PASSWORD.replace('"', "%22").replace(' ', "%20")),
        ("the password in a form", PASSWORD.replace('"', "%22").replace(' ', "+")),
        ("the password in JSON", PASSWORD.replace('"', "\\\"")),
    ];
    forms.into_iter().filter(|(_, form)| text.contains(form.as_str())).map(|(name, _)| name).collect()
}

/// A run sends a secret every way a request can carry it — header, query,
/// body, Basic — to an emulator that echoes what it got, and a wait reads the
/// request too. Nothing the engine reports may carry the secret in any form a
/// reader could use: the steps, the result, the report, the events, the
/// Inspector's frames and their bytes.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_secret_echoed_back_in_any_form_stays_masked() {
    let (service, recorder) = secret_service();
    service.invoke("inspect_set_enabled", json!({ "enabled": true })).await.unwrap();
    let port = free_port();
    let bind = format!("127.0.0.1:{port}");
    let node = |id: &str, kind: Value| {
        let mut value = kind;
        value["id"] = json!(id);
        value["x"] = json!(0);
        value["y"] = json!(0);
        value
    };
    let echo = "{{request.headers.x-token}}|{{request.query.q}}|{{request.body}}|{{request.headers.authorization}}|{{request.path}}|{{request.headers}}|{{request.query}}";
    let document: signal_lab_engine::experiment::Experiment = serde_json::from_value(json!({
        "version": 9, "name": "Leak", "params": [], "profiles": [], "profile": null, "seed": 3,
        "nodes": [
            node("start", json!({ "type": "start" })),
            node("echo", json!({ "type": "emulator", "emulator": { "name": "Echo", "bind": bind, "protocol": "http",
                "routes": [{ "method": "ANY", "path": "/*", "responses": [{ "status": 200, "headers": [["X-Echo", "{{request.headers.x-token}}"]], "body": echo }] }] } })),
            node("send", json!({ "type": "http", "request": { "method": "POST",
                "url": format!("http://{bind}/p/{{{{secret.TOKEN}}}}?q={{{{secret.TOKEN}}}}&r={{{{secret.PASSWORD}}}}"),
                "headers": [["X-Token", "{{secret.TOKEN}}"], ["X-Password", "{{secret.PASSWORD}}"]], "body": "token={{secret.TOKEN}}", "timeout_ms": 3000,
                "auth": { "scheme": "basic", "username": "lab", "password": "{{secret.TOKEN}}" } } })),
            node("body", json!({ "type": "extract", "variable": "answer", "from": "body", "expr": "" })),
            node("say", json!({ "type": "log", "message": "{{answer}}" })),
            node("end", json!({ "type": "end" })),
        ],
        "edges": [
            { "from": "start", "to": "echo", "port": "next" },
            { "from": "echo", "to": "send", "port": "next" },
            { "from": "send", "to": "body", "port": "next" },
            { "from": "body", "to": "say", "port": "next" },
            { "from": "say", "to": "end", "port": "next" },
        ]
    }))
    .unwrap();
    let handle = service.run(document, signal_lab_engine::experiment_run::RunOptions { limit: Some(Duration::from_secs(20)), ..Default::default() }).await.unwrap();
    let result = handle.finished().await;
    let steps = serde_json::to_string(&result).unwrap();
    assert_eq!(result.outcome, signal_lab_engine::experiment_run::Outcome::Passed, "{steps}");
    // The echo really happened: the answer the run read carries the masked value.
    assert!(steps.contains("••••|"), "{steps}");

    let mut places = vec![("the result", steps)];
    if let Some(path) = &result.report_path {
        places.push(("the report", std::fs::read_to_string(path).unwrap()));
    }
    places.push(("the events", serde_json::to_string(&recorder.events()).unwrap()));
    let frames = service.invoke("inspect_snapshot", json!({ "limit": 500 })).await.unwrap();
    for frame in frames.as_array().unwrap() {
        if let Ok(payload) = service.invoke("inspect_payload", json!({ "seq": frame["seq"] })).await {
            let hex: String = payload["hex"].as_str().unwrap_or_default().split_whitespace().collect();
            let bytes: Vec<u8> = (0..hex.len() / 2).filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()).collect();
            places.push(("a frame's bytes", String::from_utf8_lossy(&bytes).into_owned()));
        }
    }
    places.push(("the Inspector's frames", frames.to_string()));
    let exported = service.invoke("inspect_export", json!({ "format": "jsonl" })).await.unwrap();
    places.push(("the Inspector's export", std::fs::read_to_string(exported.as_str().unwrap()).unwrap()));
    let leaks: Vec<String> = places
        .iter()
        .flat_map(|(place, text)| leaked_forms(text).into_iter().map(move |form| format!("{place}: {form}")))
        .collect();
    assert!(leaks.is_empty(), "the secret got out:\n{}", leaks.join("\n"));
}
