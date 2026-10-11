//! Files from outside: an experiment someone sends, a library file edited by
//! hand or damaged. Each is refused with a code that says what is wrong —
//! never a panic, a hang, a stack overflow, or a file silently replaced.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::service::Failure;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-hostile-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        dir
    })
    .clone()
}

fn service() -> Arc<Service> {
    data_dir();
    Arc::new(Service::new(Host::new(Recorder::new(), Capture::new()), Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))))
}

/// The code a command failed with, within 10 s and without a panic.
async fn code(service: &Arc<Service>, command: &str, args: Value) -> String {
    let (service, name) = (service.clone(), command.to_string());
    let began = Instant::now();
    let task = tokio::spawn(async move { service.invoke(&name, args).await });
    let reply = tokio::time::timeout(Duration::from_secs(10), task).await.unwrap_or_else(|_| panic!("{command} did not answer within 10 s"));
    let reply = reply.unwrap_or_else(|join| panic!("{command} panicked: {join}"));
    assert!(began.elapsed() < Duration::from_secs(5), "{command} took {:?}", began.elapsed());
    match reply {
        Err(Failure::Engine(error)) => error.code.clone(),
        Ok(_) => "ok".into(),
    }
}

fn starter() -> Value {
    serde_json::to_value(signal_lab_engine::experiment::starter()).unwrap()
}

/// An experiment file as someone could send it: each is refused for what it is.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hostile_experiment_files_are_refused_for_what_they_are() {
    let service = service();
    let with = |change: &dyn Fn(&mut Value)| {
        let mut document = starter();
        change(&mut document);
        document.to_string()
    };
    let first = starter()["nodes"][1]["id"].as_str().unwrap().to_string();
    let node = |id: usize| json!({ "id": format!("n{id}"), "x": 0, "y": 0, "type": "log", "message": "x" });
    let cases: Vec<(&str, String, &str)> = vec![
        ("not JSON", "{".into(), "file.json_invalid"),
        ("nested 100 000 deep", format!("{}{}", "[".repeat(100_000), "]".repeat(100_000)), "file.json_invalid"),
        ("an object nested 100 000 deep", format!("{}1{}", "{\"a\":".repeat(100_000), "}".repeat(100_000)), "file.json_invalid"),
        ("past 4 MiB", format!("{{\"name\":\"{}\"}}", "x".repeat(5 << 20)), "file.too_large"),
        ("a version from the future", with(&|d| d["version"] = json!(999)), "doc.version_unsupported"),
        ("a negative version", with(&|d| d["version"] = json!(-1)), "file.json_invalid"),
        ("a node wired to itself", with(&|d| d["edges"].as_array_mut().unwrap().push(json!({ "from": first, "to": first, "port": "next" }))), "doc.connection_invalid"),
        ("an edge to nowhere", with(&|d| d["edges"].as_array_mut().unwrap().push(json!({ "from": first, "to": "ghost", "port": "next" }))), "doc.connection_invalid"),
        ("two nodes with one id", with(&|d| { let copy = d["nodes"][1].clone(); d["nodes"].as_array_mut().unwrap().push(copy); }), "doc.node_id_duplicate"),
        ("10 000 nodes", with(&|d| d["nodes"] = Value::Array((0..10_000).map(node).collect())), "doc.node_count"),
        ("NUL in a name", with(&|d| d["name"] = json!("a\u{0}b")), "ok"),
    ];
    let mut wrong = Vec::new();
    for (name, text, expected) in cases {
        let got = code(&service, "experiment_parse", json!({ "text": text })).await;
        if got != expected {
            wrong.push(format!("{name}: {got}, expected {expected}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A library file damaged on disk — invalid UTF-8, nested past any parser's
/// stack, truncated — is reported with its path and left as it is.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_damaged_library_file_is_reported_and_kept() {
    let service = service();
    let damaged: Vec<(&str, Vec<u8>)> = vec![
        ("invalid UTF-8", b"{\"version\":2,\"signals\":[{\"name\":\"\xff\xfe\"}]}".to_vec()),
        ("nested 100 000 deep", format!("{{\"version\":2,\"signals\":{}{}}}", "[".repeat(100_000), "]".repeat(100_000)).into_bytes()),
        ("cut short", b"{\"version\":2,\"signals\":[".to_vec()),
        ("NUL bytes", vec![0; 64]),
        // From a newer Signal Lab: what it added would be lost on the next save.
        ("a version from the future", b"{\"version\":99,\"signals\":[],\"emulators\":[],\"future\":{\"kept\":true}}".to_vec()),
    ];
    for (file, command) in [("signals.json", "signals_load"), ("emulators.json", "emulators_load"), ("experiment.json", "experiment_load")] {
        for (name, bytes) in &damaged {
            let path = data_dir().join(file);
            std::fs::write(&path, bytes).unwrap();
            let got = code(&service, command, Value::Null).await;
            assert_ne!(got, "ok", "{file} {name}: read as if it were fine");
            assert_eq!(&std::fs::read(&path).unwrap(), bytes, "{file} {name}: the file was replaced ({got})");
        }
        std::fs::remove_file(data_dir().join(file)).unwrap();
    }
}
