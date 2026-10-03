//! WebSocket against a loopback service: the screen's connection as a job, an
//! experiment that connects, is greeted, asks and reads the JSON answer, then
//! closes; a server that closes first; upgrades that fail and say why; the
//! one-shot exchange; a stopped run that still says goodbye; Send now.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_run::{Outcome, RunOptions, RunResult};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;

fn scratch_data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-ws-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        paths::data_dir()
    })
    .clone()
}

fn service() -> (Service, Arc<Recorder>) {
    scratch_data_dir();
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))), recorder)
}

/// What the service saw: the texts it received, the close codes clients sent,
/// and how many clients were connected at once at most.
#[derive(Default)]
struct Seen {
    texts: Mutex<Vec<String>>,
    closes: Mutex<Vec<u16>>,
    open: Mutex<(u32, u32)>,
}

/// Greets with `welcome`; answers `ask <x>` with `{"echo":"<x>","n":<count>}`,
/// `quit` by closing with 4001 "bye", `drop` by cutting the line with no close
/// frame, JSON with itself (an echo service), anything else with `echo <text>`;
/// echoes binary. Takes the first subprotocol offered.
async fn service_on_loopback() -> (String, Arc<Seen>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Seen::default());
    let shared = seen.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let seen = shared.clone();
            tokio::spawn(async move {
                #[allow(clippy::result_large_err)] // the library's callback type: its error is a whole response
                let choose = |request: &tokio_tungstenite::tungstenite::handshake::server::Request, mut response: tokio_tungstenite::tungstenite::handshake::server::Response| {
                    if let Some(first) = request.headers().get("sec-websocket-protocol").and_then(|value| value.to_str().ok()).and_then(|value| value.split(',').next()) {
                        response.headers_mut().insert("sec-websocket-protocol", first.trim().parse().unwrap());
                    }
                    Ok(response)
                };
                let Ok(mut socket) = tokio_tungstenite::accept_hdr_async(stream, choose).await else { return };
                {
                    let mut open = seen.open.lock().unwrap();
                    open.0 += 1;
                    open.1 = open.1.max(open.0);
                }
                struct Leaving(Arc<Seen>);
                impl Drop for Leaving {
                    fn drop(&mut self) {
                        self.0.open.lock().unwrap().0 -= 1;
                    }
                }
                let _leaving = Leaving(seen.clone());
                let _ = socket.send(Message::text("welcome")).await;
                let mut count = 0;
                while let Some(Ok(message)) = socket.next().await {
                    match message {
                        Message::Text(text) => {
                            count += 1;
                            seen.texts.lock().unwrap().push(text.to_string());
                            let answer = if let Some(rest) = text.strip_prefix("ask ") {
                                json!({ "echo": rest, "n": count }).to_string()
                            } else if text.starts_with('{') {
                                text.to_string()
                            } else if text.as_str() == "drop" {
                                return;
                            } else if text.as_str() == "quit" {
                                let _ = socket.close(Some(CloseFrame { code: CloseCode::from(4001), reason: "bye".into() })).await;
                                continue;
                            } else {
                                format!("echo {text}")
                            };
                            if socket.send(Message::text(answer)).await.is_err() {
                                return;
                            }
                        }
                        Message::Binary(bytes) => {
                            let _ = socket.send(Message::Binary(bytes)).await;
                        }
                        Message::Close(frame) => {
                            seen.closes.lock().unwrap().push(frame.map(|frame| u16::from(frame.code)).unwrap_or(1005));
                            break;
                        }
                        _ => {}
                    }
                }
            });
        }
    });
    (url, seen)
}

fn node(id: &str, x: f64, kind: Value) -> Value {
    let mut value = kind;
    value["id"] = json!(id);
    value["x"] = json!(x);
    value["y"] = json!(0);
    value
}

fn document(nodes: Vec<Value>, edges: Vec<(&str, &str, &str)>) -> Value {
    let edges: Vec<Value> = edges.into_iter().map(|(from, port, to)| json!({ "from": from, "to": to, "port": port })).collect();
    json!({ "version": 8, "name": "WebSocket", "params": [{ "name": "who", "value": "lab" }], "profiles": [], "profile": null, "seed": 3, "nodes": nodes, "edges": edges })
}

async fn run(service: &Service, doc: &Value) -> RunResult {
    let doc: Experiment = serde_json::from_value(doc.clone()).unwrap();
    let handle = service.run(doc, RunOptions { limit: Some(Duration::from_secs(30)), ..Default::default() }).await.unwrap();
    handle.finished().await
}

fn failure(result: &RunResult) -> String {
    format!("{:?} {:#?}", result.error, result.steps.iter().filter(|step| step.state == "failed").collect::<Vec<_>>())
}

fn step(result: &RunResult, id: &str) -> Value {
    serde_json::to_value(result.steps.iter().rfind(|step| step.node_id == id && step.state != "running").unwrap_or_else(|| panic!("no step {id}"))).unwrap()
}

async fn eventually(test: impl Fn() -> bool) -> bool {
    for _ in 0..100 {
        if test() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_screen_connects_sends_hears_and_closes_as_a_job() {
    let (service, recorder) = service();
    let (url, seen) = service_on_loopback().await;
    let job = service.invoke("ws_connect", json!({ "config": { "url": format!("{url}/chat"), "protocols": ["lab.v1"], "headers": [["X-Lab", "1"]] } })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    assert_eq!((job["kind"].as_str(), job["params"]["url"].as_str()), (Some("websocket"), Some(format!("{url}/chat").as_str())));
    assert_eq!(service.invoke("ws_send", json!({ "jobId": id, "message": { "text": "hello" } })).await.unwrap(), json!(5));
    assert_eq!(service.invoke("ws_send", json!({ "jobId": id, "message": { "hex": "01 02 03" } })).await.unwrap(), json!(3));

    let wait = |test: fn(&Value) -> bool| {
        let recorder = recorder.clone();
        tokio::task::spawn_blocking(move || recorder.wait_for("ws://messages", Duration::from_secs(5), test))
    };
    wait(|batch| batch["messages"].as_array().into_iter().flatten().any(|message| message["text"] == "echo hello" && message["dir"] == "rx"))
        .await
        .unwrap()
        .expect("the echo reached the screen");
    let all: Vec<Value> = recorder.payloads("ws://messages").iter().flat_map(|batch| batch["messages"].as_array().cloned().unwrap_or_default()).collect();
    assert!(all.iter().any(|message| message["text"] == "welcome" && message["dir"] == "rx"), "the greeting: {all:?}");
    assert!(all.iter().any(|message| message["text"] == "hello" && message["dir"] == "tx"), "what was sent is listed too");
    let connected = recorder.payloads("ws://state").into_iter().find(|state| state["state"] == "connected").unwrap();
    assert_eq!(connected["handshake"]["protocol"], "lab.v1", "{connected}");

    let closed = service.invoke("ws_close", json!({ "jobId": id, "code": 4000, "reason": "done" })).await.unwrap();
    assert_eq!((closed["code"].as_u64(), closed["by"].as_str()), (Some(4000), Some("client")), "{closed}");
    assert!(eventually(|| service.jobs().list().is_empty()).await, "the job ends with the connection");
    assert!(eventually(|| seen.closes.lock().unwrap().contains(&4000)).await, "the server got our close frame: {:?}", seen.closes.lock().unwrap());
    let ended = recorder.payloads("ws://state").into_iter().find(|state| state["state"] == "closed").unwrap();
    assert_eq!(ended["closed"]["by"], "client");

    let gone = service.invoke("ws_send", json!({ "jobId": id, "message": { "text": "x" } })).await.unwrap_err();
    assert_eq!(serde_json::to_value(&gone).unwrap()["code"], "ws.not_connected");
    let wrong = service.invoke("ws_close", json!({ "jobId": id, "code": 1006 })).await.unwrap_err();
    assert_eq!(serde_json::to_value(&wrong).unwrap()["code"], "ws.close_code", "1006 is never sent");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_run_connects_is_greeted_asks_reads_json_and_closes() {
    let (service, _) = service();
    let (url, seen) = service_on_loopback().await;
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("socket", 100.0, json!({ "type": "ws_connect", "url": format!("{url}/live?who={{{{who}}}}"), "protocols": ["lab.v1"], "timeout_ms": 3000 })),
            node("greeted", 200.0, json!({ "type": "wait_ws", "connection": "socket", "mode": "contains", "pattern": "welcome", "timeout_ms": 3000 })),
            node("ask", 300.0, json!({ "type": "ws_send", "connection": "socket", "text": "ask {{who}}" })),
            node("answer", 400.0, json!({ "type": "wait_ws", "connection": "socket", "mode": "regex", "pattern": "\"echo\":\"lab\"", "timeout_ms": 3000, "variable": "answer" })),
            node("check", 500.0, json!({ "type": "assert_value", "value": "{{answer.json.echo}}-{{answer.json.n}}", "op": "eq", "expected": "lab-1" })),
            node("bytes", 600.0, json!({ "type": "ws_send", "connection": "socket", "text": "ca fe", "binary": true })),
            node("back", 700.0, json!({ "type": "wait_ws", "connection": "socket", "mode": "hex", "pattern": "cafe", "timeout_ms": 3000 })),
            node("bye", 800.0, json!({ "type": "ws_close", "connection": "socket", "code": 1000, "reason": "done" })),
            node("end", 900.0, json!({ "type": "end" })),
        ],
        vec![
            ("start", "next", "socket"), ("socket", "next", "greeted"), ("greeted", "matched", "ask"), ("ask", "next", "answer"), ("answer", "matched", "check"),
            ("check", "next", "bytes"), ("bytes", "next", "back"), ("back", "matched", "bye"), ("bye", "next", "end"),
        ],
    );
    let result = run(&service, &doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    assert_eq!(seen.texts.lock().unwrap().as_slice(), ["ask lab"], "the template was rendered");
    let socket = step(&result, "socket");
    assert!(socket["detail"].as_str().unwrap().contains("/live?who=lab (lab.v1)"), "the URL rendered, the subprotocol chosen: {socket}");
    let closed = step(&result, "bye");
    assert!(closed["detail"].as_str().unwrap().starts_with("Closed 1000 by the client"), "{closed}");
    assert!(eventually(|| seen.closes.lock().unwrap().contains(&1000)).await, "the close reached the server");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_server_that_closes_first_ends_the_waits_and_the_close_says_so() {
    let (service, _) = service();
    let (url, _) = service_on_loopback().await;
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("socket", 100.0, json!({ "type": "ws_connect", "url": url, "timeout_ms": 3000 })),
            node("quit", 200.0, json!({ "type": "ws_send", "connection": "socket", "text": "quit" })),
            node("more", 300.0, json!({ "type": "wait_ws", "connection": "socket", "mode": "contains", "pattern": "never", "timeout_ms": 3000 })),
            node("end", 400.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "socket"), ("socket", "next", "quit"), ("quit", "next", "more"), ("more", "matched", "end")],
    );
    let started = std::time::Instant::now();
    let result = run(&service, &doc).await;
    assert_eq!(result.outcome, Outcome::Failed);
    let error = serde_json::to_value(result.error.as_ref().unwrap()).unwrap();
    assert_eq!((error["code"].as_str(), error["params"]["code"].as_str(), error["detail"].as_str(), error["node"].as_str()), (Some("ws.closed"), Some("4001"), Some("bye"), Some("more")), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2), "the wait ends with the connection, not at its timeout");

    // Closed by the server before the close step: the step passes and says who closed it.
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("socket", 100.0, json!({ "type": "ws_connect", "url": url, "timeout_ms": 3000 })),
            node("quit", 200.0, json!({ "type": "ws_send", "connection": "socket", "text": "quit" })),
            node("pause", 300.0, json!({ "type": "delay", "ms": 300 })),
            node("bye", 400.0, json!({ "type": "ws_close", "connection": "socket" })),
            node("end", 500.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "socket"), ("socket", "next", "quit"), ("quit", "next", "pause"), ("pause", "next", "bye"), ("bye", "next", "end")],
    );
    let result = run(&service, &doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    assert!(step(&result, "bye")["detail"].as_str().unwrap().starts_with("Closed 4001 by the server"), "{}", step(&result, "bye"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn upgrades_that_fail_say_why() {
    let (service, _) = service();
    // An HTTP server that refuses the upgrade.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buffer = [0u8; 2048];
                let _ = stream.read(&mut buffer).await;
                let _ = stream.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 9\r\nConnection: close\r\n\r\nno entry!").await;
            });
        }
    });
    let code_of = |result: Result<Value, signal_lab_engine::Failure>| serde_json::to_value(result.unwrap_err()).unwrap();
    let forbidden = code_of(service.invoke("ws_connect", json!({ "config": { "url": format!("ws://{address}/") } })).await);
    assert_eq!((forbidden["code"].as_str(), forbidden["params"]["status"].as_str(), forbidden["detail"].as_str()), (Some("ws.handshake_status"), Some("403"), Some("no entry!")), "{forbidden}");
    let closed_port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let refused = code_of(service.invoke("ws_connect", json!({ "config": { "url": format!("ws://127.0.0.1:{closed_port}/"), "timeout_ms": 3000 } })).await);
    assert!(matches!(refused["code"].as_str(), Some("transport.refused" | "transport.timeout")), "{refused}");
    let wrong = code_of(service.invoke("ws_connect", json!({ "config": { "url": "http://127.0.0.1/" } })).await);
    assert_eq!(wrong["code"], "ws.url_invalid");

    // In a document the address is checked before the run, and so is what a node names.
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("socket", 100.0, json!({ "type": "ws_connect", "url": "http://127.0.0.1:1/" })),
            node("end", 200.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "socket"), ("socket", "next", "end")],
    );
    let invalid = code_of(service.invoke("experiment_validate", json!({ "document": doc })).await);
    assert_eq!((invalid["code"].as_str(), invalid["node"].as_str(), invalid["field"]["key"].as_str()), (Some("ws.url_invalid"), Some("socket"), Some("url")), "{invalid}");
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("say", 100.0, json!({ "type": "ws_send", "connection": "start", "text": "x" })),
            node("end", 200.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "say"), ("say", "next", "end")],
    );
    let unknown = code_of(service.invoke("experiment_validate", json!({ "document": doc })).await);
    assert_eq!((unknown["code"].as_str(), unknown["node"].as_str()), (Some("ws.connection_unknown"), Some("say")), "{unknown}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_exchange_sends_waits_and_closes() {
    let (service, _) = service();
    let (url, seen) = service_on_loopback().await;
    // The expectation's fields are written like a node's (snake_case), as a config's are.
    let exchange = service
        .invoke("ws_exchange", json!({ "config": { "url": url }, "message": { "text": "ask x" }, "expect": { "mode": "contains", "pattern": "echo", "timeout_ms": 2000 } }))
        .await
        .unwrap();
    assert_eq!((exchange["sent"].as_u64(), exchange["reply"]["json"]["echo"].as_str()), (Some(5), Some("x")), "{exchange}");
    assert_eq!(exchange["closed"]["by"], "client");
    assert!(eventually(|| seen.closes.lock().unwrap().contains(&1000)).await);
    let silent = service.invoke("ws_exchange", json!({ "config": { "url": url }, "message": { "text": "ask x" }, "expect": { "mode": "contains", "pattern": "never", "timeout_ms": 200 } })).await.unwrap_err();
    assert_eq!(serde_json::to_value(silent).unwrap()["code"], "wait.timeout");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_stopped_run_still_says_goodbye_and_send_now_opens_its_connection() {
    let (service, _) = service();
    let (url, seen) = service_on_loopback().await;
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("socket", 100.0, json!({ "type": "ws_connect", "url": url })),
            node("idle", 200.0, json!({ "type": "delay", "ms": 20000 })),
            node("say", 300.0, json!({ "type": "ws_send", "connection": "socket", "text": "now {{who}}" })),
            node("heard", 400.0, json!({ "type": "wait_ws", "connection": "socket", "mode": "contains", "pattern": "echo now", "timeout_ms": 2000 })),
            node("end", 500.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "socket"), ("socket", "next", "idle"), ("idle", "next", "say"), ("say", "next", "heard"), ("heard", "matched", "end")],
    );
    let started = service.invoke("experiment_start", json!({ "document": doc })).await.unwrap();
    let id = started["id"].as_u64().unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(service.jobs().stop(id));
    assert!(eventually(|| seen.closes.lock().unwrap().contains(&1000)).await, "stopping the run closed its connection properly: {:?}", seen.closes.lock().unwrap());

    // Send now on a send: the connect node it names opens a connection for it.
    let sent = service.invoke("experiment_send_node", json!({ "document": doc, "nodeId": "say", "vars": {} })).await.unwrap();
    assert_eq!(sent["detail"], "Sent 7 B", "{sent}");
    assert!(eventually(|| seen.texts.lock().unwrap().iter().any(|text| text == "now lab")).await);
    // And a wait listens on one: the greeting is what it hears.
    let heard = service.invoke("experiment_send_node", json!({ "document": doc, "nodeId": "heard", "vars": {} })).await.unwrap_err();
    assert_eq!(serde_json::to_value(heard).unwrap()["code"], "wait.timeout", "nothing sent there says echo now");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_echo_template_passes_against_an_echo_service() {
    let (service, _) = service();
    let (url, seen) = service_on_loopback().await;
    let doc: Experiment = serde_json::from_str(include_str!("../../experiments/templates/websocket-echo.json")).unwrap();
    let overrides = std::collections::BTreeMap::from([("service".to_string(), format!("{url}/echo"))]);
    let handle = service.run(doc, RunOptions { overrides, limit: Some(Duration::from_secs(30)), ..Default::default() }).await.unwrap();
    let result = handle.finished().await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    assert!(result.steps.iter().any(|step| step.node_id == "same" && step.state == "passed"), "the echo came back unchanged");
    let sent = seen.texts.lock().unwrap().clone();
    assert!(sent.len() == 1 && sent[0].starts_with("{\"type\":\"ping\",\"run\":\""), "{sent:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_line_cut_mid_session_is_a_reset_not_a_failed_upgrade() {
    let (service, _) = service();
    let (url, _) = service_on_loopback().await;
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("socket", 100.0, json!({ "type": "ws_connect", "url": url })),
            node("cut", 200.0, json!({ "type": "ws_send", "connection": "socket", "text": "drop" })),
            node("more", 300.0, json!({ "type": "wait_ws", "connection": "socket", "mode": "contains", "pattern": "never", "timeout_ms": 3000 })),
            node("end", 400.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "socket"), ("socket", "next", "cut"), ("cut", "next", "more"), ("more", "matched", "end")],
    );
    let result = run(&service, &doc).await;
    let error = serde_json::to_value(result.error.as_ref().unwrap()).unwrap();
    assert_eq!((error["code"].as_str(), error["node"].as_str()), (Some("transport.reset"), Some("more")), "{error}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_answer_is_what_came_after_the_send_and_keeps_its_kind() {
    let (service, _) = service();
    let (url, _) = service_on_loopback().await;
    // The greeting is already there when the message goes: the answer is the echo, not it.
    for _ in 0..5 {
        let exchange = service.invoke("ws_exchange", json!({ "config": { "url": url }, "message": { "text": "ask q" }, "expect": { "mode": "any", "timeout_ms": 2000 } })).await.unwrap();
        assert_eq!(exchange["reply"]["json"]["echo"], "q", "{exchange}");
    }
    // Bytes that happen to be UTF-8 still came as a binary message.
    let binary = service.invoke("ws_exchange", json!({ "config": { "url": url }, "message": { "hex": "7b 7d" }, "expect": { "mode": "hex", "pattern": "7b7d", "timeout_ms": 2000 } })).await.unwrap();
    assert_eq!((binary["reply"]["kind"].as_str(), binary["reply"]["json"].is_null()), (Some("binary"), true), "{binary}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_rendered_reason_must_fit_a_close_frame_and_a_send_needs_its_connect_before_it() {
    let (service, _) = service();
    let (url, _) = service_on_loopback().await;
    let long = "x".repeat(200);
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("socket", 100.0, json!({ "type": "ws_connect", "url": url })),
            node("bye", 200.0, json!({ "type": "ws_close", "connection": "socket", "reason": "{{why}}" })),
            node("end", 300.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "socket"), ("socket", "next", "bye"), ("bye", "next", "end")],
    );
    let mut doc = doc;
    doc["params"] = json!([{ "name": "why", "value": long }]);
    let result = run(&service, &doc).await;
    let error = serde_json::to_value(result.error.as_ref().unwrap()).unwrap();
    assert_eq!((error["code"].as_str(), error["node"].as_str(), error["params"]["max"].as_str()), (Some("node.too_long"), Some("bye"), Some("123")), "{error}");
    let refused = service.invoke("ws_close", json!({ "jobId": 1, "reason": long })).await.unwrap_err();
    assert_eq!(serde_json::to_value(refused).unwrap()["code"], "node.too_long");

    // A send on a branch beside its connect would race it: refused before the run.
    let beside = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("fork", 100.0, json!({ "type": "fork" })),
            node("socket", 200.0, json!({ "type": "ws_connect", "url": url })),
            node("say", 200.0, json!({ "type": "ws_send", "connection": "socket", "text": "hi" })),
            node("join", 300.0, json!({ "type": "join" })),
            node("end", 400.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "fork"), ("fork", "branch1", "socket"), ("fork", "branch2", "say"), ("socket", "next", "join"), ("say", "next", "join"), ("join", "next", "end")],
    );
    let error = serde_json::to_value(service.invoke("experiment_validate", json!({ "document": beside })).await.unwrap_err()).unwrap();
    assert_eq!((error["code"].as_str(), error["node"].as_str()), (Some("ws.connection_after"), Some("say")), "{error}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_connect_run_again_closes_its_old_connection_first() {
    let (service, _) = service();
    let (url, seen) = service_on_loopback().await;
    let doc = document(
        vec![
            node("start", 0.0, json!({ "type": "start" })),
            node("again", 100.0, json!({ "type": "loop", "max": 3 })),
            node("socket", 200.0, json!({ "type": "ws_connect", "url": url })),
            node("greeted", 300.0, json!({ "type": "wait_ws", "connection": "socket", "mode": "contains", "pattern": "welcome", "timeout_ms": 3000 })),
            node("end", 400.0, json!({ "type": "end" })),
        ],
        vec![("start", "next", "again"), ("again", "body", "socket"), ("socket", "next", "greeted"), ("greeted", "matched", "again"), ("again", "done", "end")],
    );
    let result = run(&service, &doc).await;
    assert_eq!(result.outcome, Outcome::Passed, "{}", failure(&result));
    assert!(eventually(|| seen.open.lock().unwrap().0 == 0).await, "and the last one closes with the run");
    assert_eq!(seen.open.lock().unwrap().1, 1, "never two connections at once");
}
