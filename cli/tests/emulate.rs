//! `signallab emulate` as a pipeline uses it: an emulator from a file, in the
//! background for a while, answering what the system under test sends, and
//! the counts at the end — in this process and on a server; its check; a port
//! already taken; names from the library.

mod common;

use std::path::Path;
use std::process::Output;
use std::time::{Duration, Instant};

use common::*;
use serde_json::{json, Value};

fn tcp_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn api(port: u16) -> Value {
    json!({ "name": "Items API", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
        "routes": [{ "method": "GET", "path": "/items/:id", "responses": [{ "body": "item {{request.params.id}} for {{who}}" }] }] })
}

fn write(dir: &Path, name: &str, value: &Value) -> String {
    let path = dir.join(name);
    std::fs::write(&path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    path.display().to_string()
}

fn lines(output: &Output) -> Vec<Value> {
    out(output).lines().filter_map(|line| serde_json::from_str(line).ok()).collect()
}

/// Ask until the emulator answers: it is starting in another process.
async fn get(url: &str) -> reqwest::Response {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match reqwest::get(url).await {
            Ok(response) => return response,
            Err(_) if Instant::now() < deadline => tokio::time::sleep(Duration::from_millis(50)).await,
            Err(error) => panic!("{url} never answered: {error}"),
        }
    }
}

async fn exercised(args: Vec<String>, port: u16) -> Output {
    let running = tokio::spawn(signallab_async(args));
    let item = get(&format!("http://127.0.0.1:{port}/items/5")).await;
    assert_eq!(item.text().await.unwrap(), "item 5 for ci");
    assert_eq!(reqwest::get(format!("http://127.0.0.1:{port}/nowhere")).await.unwrap().status().as_u16(), 404);
    running.await.unwrap()
}

fn assert_counted(output: &Output, port: u16) {
    assert_eq!(code(output), 0, "{}{}", out(output), err(output));
    let lines = lines(output);
    let started = lines.iter().find(|line| line["type"] == "started").unwrap();
    assert_eq!((started["protocol"].as_str(), started["local"].as_str()), (Some("http"), Some(format!("127.0.0.1:{port}").as_str())));
    let exchanges: Vec<&Value> = lines.iter().filter(|line| line["type"] == "exchange").collect();
    assert_eq!(exchanges.len(), 2, "{}", out(output));
    assert_eq!((exchanges[0]["exchange"]["request"].as_str(), exchanges[0]["exchange"]["status"].as_u64()), (Some("GET /items/5"), Some(200)));
    let summary = lines.iter().find(|line| line["type"] == "summary").unwrap();
    assert_eq!(summary["counts"], json!({ "total": 2, "unmatched": 1, "failed": 0, "hits": [1] }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_emulator_answers_for_a_while_here_and_on_a_server() {
    let dir = folder("emulate");
    let port = tcp_port();
    let file = write(&dir, "api.json", &api(port));
    let args = |extra: &[&str]| -> Vec<String> {
        let mut args: Vec<String> = ["--json", "emulate", &file, "--param", "who=ci", "--for", "2"].iter().map(|arg| arg.to_string()).collect();
        args.extend(extra.iter().map(|arg| arg.to_string()));
        args
    };
    let here = exercised(args(&[]), port).await;
    assert_counted(&here, port);

    // In words: what it got as it got it, and the counts at the end.
    let port = tcp_port();
    let file = write(&dir, "api-text.json", &api(port));
    let running = tokio::spawn(signallab_async(vec!["emulate".into(), file, "-p".into(), "who=ci".into(), "--for".into(), "2".into()]));
    get(&format!("http://127.0.0.1:{port}/items/9")).await;
    let text = running.await.unwrap();
    assert_eq!(code(&text), 0);
    assert!(out(&text).contains("Items API  #1  GET /items/9 → 200 OK · 13 B"), "{}", out(&text));
    assert!(err(&text).contains("Items API (http) answering on") && err(&text).contains("Items API: 1 request, 0 without a rule, 0 failed · #1 1"), "{}", err(&text));

    // On a server: started there, followed through its API, stopped at the end.
    let running = start_server(&dir.join("server")).await;
    let token = dir.join("token.txt");
    std::fs::write(&token, TOKEN).unwrap();
    let port = tcp_port();
    let file = write(&dir, "api-server.json", &api(port));
    let remote = exercised(
        ["--json", "emulate", &file, "--param", "who=ci", "--for", "2", "--server", &running.url, "--token-file", &token.display().to_string()].iter().map(|arg| arg.to_string()).collect(),
        port,
    )
    .await;
    assert_counted(&remote, port);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(reqwest::get(format!("http://127.0.0.1:{port}/items/1")).await.is_err(), "the server's emulator was stopped");
    running.stop().await;
}

#[test]
fn a_check_a_taken_port_and_names_from_the_library() {
    let dir = folder("emulate-check");
    let port = tcp_port();
    let file = write(&dir, "api.json", &api(port));
    let check = signallab(&["emulate", &file, "--check", "-p", "who=ci"]);
    assert_eq!((code(&check), out(&check).trim()), (0, format!("Items API would answer on 127.0.0.1:{port}").as_str()));
    let unknown = signallab(&["--json", "emulate", &file, "--check"]);
    let error: Value = serde_json::from_str(out(&unknown).trim()).unwrap();
    assert_eq!((code(&unknown), error["error"]["code"].as_str(), error["error"]["params"]["rule"].as_str()), (2, Some("emulator.name_unknown"), Some("1")), "{{who}} is no parameter without -p");

    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let busy = signallab(&["emulate", &file, "-p", "who=ci", "--bind", &taken.local_addr().unwrap().to_string(), "--for", "1"]);
    assert_eq!(code(&busy), 3, "a port in use is the surroundings: {}", err(&busy));
    assert!(err(&busy).contains("already in use"), "{}", err(&busy));

    let library = dir.join("emulators.json");
    std::fs::write(&library, serde_json::to_string(&signal_lab_engine::emulator_files::seed()).unwrap()).unwrap();
    let listed = signallab(&["emulators", "--library", &library.display().to_string()]);
    assert_eq!(code(&listed), 0);
    assert!(out(&listed).lines().any(|line| line.starts_with("demo-api") && line.contains("Demo API") && line.contains("127.0.0.1:8080") && line.ends_with("5 rules")), "{}", out(&listed));
    let named = signallab(&["emulate", "demo osc device", "--library", &library.display().to_string(), "--check"]);
    assert_eq!((code(&named), out(&named).trim()), (0, "Demo OSC device would answer on 127.0.0.1:9100"));
    let missing = signallab(&["--json", "emulate", "nothing", "--library", &library.display().to_string()]);
    let error: Value = serde_json::from_str(out(&missing).trim()).unwrap();
    assert_eq!((code(&missing), error["error"]["code"].as_str()), (2, Some("cli.emulator_unknown")));
    let two = signallab(&["emulate", &file, &file, "--bind", "127.0.0.1:1"]);
    assert_eq!(code(&two), 2, "{}", err(&two));
}
