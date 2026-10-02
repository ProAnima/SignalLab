//! `signallab mcp` as an LLM client drives it: the real process, JSON-RPC over
//! its stdin and stdout, every tool against the loopback gear, progress while a
//! run goes on, a cancelled run that stops, protocol errors, a lab server
//! behind `--server`, and the configuration a client is given.

mod common;

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use common::*;
use serde_json::{json, Value};

struct Client {
    child: Child,
    input: ChildStdin,
    lines: Receiver<Value>,
    next: u64,
}

impl Client {
    fn start(args: &[&str], env: &[(&str, &str)]) -> Client {
        let mut command = command(&[["mcp"].as_slice(), args].concat());
        for (name, value) in env {
            command.env(name, value);
        }
        let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        let (send, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let Ok(line) = line else { break };
                let value: Value = serde_json::from_str(&line).unwrap_or_else(|_| panic!("stdout carries only JSON-RPC, not: {line}"));
                if send.send(value).is_err() {
                    break;
                }
            }
        });
        let mut client = Client { child, input, lines, next: 1 };
        let hello = client.request("initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } }));
        assert_eq!(hello["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(hello["result"]["serverInfo"]["name"], "signallab");
        assert!(hello["result"]["capabilities"]["tools"].is_object());
        client.notify("notifications/initialized", json!({}));
        client
    }

    fn send(&mut self, message: Value) {
        writeln!(self.input, "{message}").unwrap();
        self.input.flush().unwrap();
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    fn start_request(&mut self, method: &str, params: Value) -> u64 {
        let id = self.next;
        self.next += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        id
    }

    /// The answer to `id`, and the notifications that came before it.
    fn wait(&mut self, id: u64, within: Duration) -> (Value, Vec<Value>) {
        let mut notes = Vec::new();
        loop {
            let message = self.lines.recv_timeout(within).unwrap_or_else(|_| panic!("no answer to request {id} within {within:?}"));
            if message["id"] == id {
                return (message, notes);
            }
            notes.push(message);
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.start_request(method, params);
        self.wait(id, Duration::from_secs(30)).0
    }

    /// A tool's result: its text, its data, whether it failed.
    fn call(&mut self, name: &str, arguments: Value) -> (String, Value, bool) {
        let answer = self.request("tools/call", json!({ "name": name, "arguments": arguments }));
        let result = &answer["result"];
        assert!(answer.get("error").is_none(), "{name}: {answer}");
        (result["content"][0]["text"].as_str().unwrap_or_default().to_string(), result["structuredContent"].clone(), result["isError"] == true)
    }

    fn close(mut self) {
        drop(self.input);
        let status = self.child.wait().unwrap();
        assert!(status.success(), "signallab mcp ends cleanly when its client closes stdin");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_tool_works_for_a_client_here() {
    let gear = gear().await;
    let dir = folder("mcp");
    let library = dir.join("signals.json");
    std::fs::write(&library, json!({ "version": 2, "signals": [
        { "id": "go", "name": "Go cue", "group": "Show", "body": { "transport": "osc", "target": gear.osc.to_string(), "address": "/go", "args": [] } },
        { "id": "hello", "name": "Hello", "body": { "transport": "udp", "target": gear.udp.to_string(), "payload": { "kind": "text", "text": "hello" } } },
    ] }).to_string()).unwrap();
    let (osc, udp, http, mqtt) = (gear.osc.to_string(), gear.udp.to_string(), gear.http, gear.mqtt.to_string());
    let seen = gear.seen.clone();
    let listen_port = free_udp_port();
    let library_path = library.display().to_string();

    tokio::task::spawn_blocking(move || {
        let mut client = Client::start(&["--library", &library_path], &[]);

        let tools = client.request("tools/list", json!({}));
        let names: Vec<&str> = tools["result"]["tools"].as_array().unwrap().iter().map(|tool| tool["name"].as_str().unwrap()).collect();
        for name in ["describe_nodes", "list_templates", "get_template", "validate_experiment", "run_experiment", "send_osc", "send_udp", "send_http", "send_mqtt", "listen", "list_signals", "fire_signal", "list_jobs", "stop_job"] {
            assert!(names.contains(&name), "{name} in {names:?}");
        }

        // What an experiment is made of, and the templates to start from.
        let (_, catalogue, failed) = client.call("describe_nodes", json!({}));
        assert!(!failed && catalogue["nodes"].as_array().unwrap().len() == 23, "{catalogue}");
        let (text, _, _) = client.call("list_templates", json!({}));
        assert!(text.contains("osc-ping-reply") && text.contains("device="), "{text}");
        let (_, template, _) = client.call("get_template", json!({ "name": "empty" }));
        assert_eq!(template["document"]["name"], "Empty experiment");

        // Validate, then run — a document the model wrote, with a parameter.
        let document = json!({ "version": 5, "name": "Model's check", "params": [{ "name": "who", "value": "x" }],
            "nodes": [{ "id": "start", "type": "start", "x": 0, "y": 0 }, { "id": "say", "type": "log", "x": 200, "y": 0, "message": "hi {{who}}" },
                      { "id": "get", "type": "http", "x": 400, "y": 0, "request": { "method": "GET", "url": format!("http://{http}/") } },
                      { "id": "ok", "type": "assert_status", "x": 600, "y": 0, "status": 200 }, { "id": "end", "type": "end", "x": 800, "y": 0 }],
            "edges": [{ "from": "start", "to": "say" }, { "from": "say", "to": "get" }, { "from": "get", "to": "ok" }, { "from": "ok", "to": "end" }] });
        let (text, data, failed) = client.call("validate_experiment", json!({ "document": document }));
        assert!(!failed && data["valid"] == true, "{text}");
        let broken = json!({ "version": 5, "name": "Broken", "nodes": [{ "id": "start", "type": "start", "x": 0, "y": 0 }], "edges": [] });
        let (text, data, failed) = client.call("validate_experiment", json!({ "document": broken }));
        assert!(failed && data["error"]["code"].is_string(), "a broken document says why: {text}");

        let id = client.start_request("tools/call", json!({ "name": "run_experiment", "arguments": { "document": document, "params": { "who": "model" } }, "_meta": { "progressToken": "run-1" } }));
        let (answer, notes) = client.wait(id, Duration::from_secs(30));
        let result = &answer["result"];
        assert_eq!(result["isError"], false);
        assert_eq!(result["structuredContent"]["outcome"], "passed", "{answer}");
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.starts_with("PASSED · Model's check") && text.contains("hi model") && text.contains("HTTP 200"), "{text}");
        let progress: Vec<&Value> = notes.iter().filter(|note| note["method"] == "notifications/progress").collect();
        assert!(progress.len() >= 5 && progress.iter().all(|note| note["params"]["progressToken"] == "run-1"), "progress while it ran: {notes:?}");

        // A run that fails is an answer, with the reason located.
        let (text, data, failed) = client.call("run_experiment", json!({ "template": "http-check", "timeout": 10 }));
        assert!(!failed && data["outcome"] == "failed" && text.starts_with("FAILED"), "{text}");
        let (text, _, failed) = client.call("run_experiment", json!({ "template": "empty", "params": { "nobody": "x" } }));
        assert!(failed && text.contains("nobody"), "an unknown parameter is refused before it runs: {text}");
        let (text, _, failed) = client.call("run_experiment", json!({ "template": "empty", "file": "x.json" }));
        assert!(failed && text.contains("exactly one"), "{text}");

        // One message of each kind, to the gear.
        let (text, _, failed) = client.call("send_osc", json!({ "target": osc, "address": "/ping", "args": [1, 0.5, "s", true] }));
        assert!(!failed && text.contains("/ping"), "{text}");
        let (_, _, failed) = client.call("send_udp", json!({ "target": udp, "text": "hello" }));
        assert!(!failed);
        let (text, data, failed) = client.call("send_http", json!({ "method": "get", "url": format!("http://{http}/status"), "headers": { "Accept": "application/json" } }));
        assert!(!failed && data["status"] == 200 && text.contains("\"state\":\"ready\""), "{text}");
        let (text, _, failed) = client.call("send_mqtt", json!({ "broker": mqtt, "topic": "lab/model", "payload": "1" }));
        assert!(!failed && text.contains("lab/model"), "{text}");
        let (text, data, failed) = client.call("send_osc", json!({ "target": "127.0.0.1", "address": "/x" }));
        assert!(failed && data["error"]["code"] == "transport.target_invalid", "{text}");

        // What arrives on a port, while something sends to it.
        let sender = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
            socket.send_to(&signal_lab_engine::osc_codec::encode_message("/fader/1", &[signal_lab_engine::osc_codec::OscArg::Float(0.25)]), ("127.0.0.1", listen_port)).unwrap();
        });
        let (text, data, failed) = client.call("listen", json!({ "protocol": "osc", "bind": format!("127.0.0.1:{listen_port}"), "seconds": 3, "max": 1 }));
        sender.join().unwrap();
        assert!(!failed && data["received"][0]["osc"][0]["address"] == "/fader/1", "{text}");

        // The library.
        let (text, _, failed) = client.call("list_signals", json!({}));
        assert!(!failed && text.contains("Show / Go cue [go]"), "{text}");
        let before = seen.osc.load(Ordering::SeqCst);
        let (text, _, failed) = client.call("fire_signal", json!({ "signal": "go cue" }));
        assert!(!failed && text.contains("Go cue"), "{text}");
        // UDP: the send returns before the device has read it.
        let reached = (0..50).any(|_| {
            std::thread::sleep(Duration::from_millis(20));
            seen.osc.load(Ordering::SeqCst) > before
        });
        assert!(reached, "the signal reached the device: {text} (seen {before}, now {})", seen.osc.load(Ordering::SeqCst));
        let (_, _, failed) = client.call("fire_signal", json!({ "signal": "nothing" }));
        assert!(failed);

        // A run that is cancelled stops, and nothing answers the cancelled request.
        let slow = json!({ "version": 5, "name": "Slow", "nodes": [{ "id": "start", "type": "start", "x": 0, "y": 0 }, { "id": "wait", "type": "delay", "x": 200, "y": 0, "ms": 20000 }, { "id": "end", "type": "end", "x": 400, "y": 0 }],
            "edges": [{ "from": "start", "to": "wait" }, { "from": "wait", "to": "end" }] });
        let cancelled = client.start_request("tools/call", json!({ "name": "run_experiment", "arguments": { "document": slow } }));
        std::thread::sleep(Duration::from_millis(500));
        let (text, data, _) = client.call("list_jobs", json!({}));
        assert_eq!(data["jobs"].as_array().unwrap().len(), 1, "{text}");
        client.notify("notifications/cancelled", json!({ "requestId": cancelled, "reason": "test" }));
        std::thread::sleep(Duration::from_millis(500));
        let (text, data, _) = client.call("list_jobs", json!({}));
        assert!(data["jobs"].as_array().unwrap().is_empty(), "the cancelled run stopped: {text}");
        let (_, data, _) = client.call("stop_job", json!({ "id": 999 }));
        assert_eq!(data["stopped"], false);

        // Protocol errors.
        let unknown_tool = client.request("tools/call", json!({ "name": "format_disk", "arguments": {} }));
        assert_eq!(unknown_tool["error"]["code"], -32602);
        let unknown_method = client.request("resources/subscribe", json!({}));
        assert_eq!(unknown_method["error"]["code"], -32601);
        let ping = client.request("ping", json!({}));
        assert_eq!(ping["result"], json!({}));
        client.send(Value::String("not an object".into()));
        client.close();
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn with_server_the_runs_happen_on_the_lab_server() {
    let dir = folder("mcp-server");
    let running = start_server(&dir.join("data")).await;
    let url = running.url.clone();
    tokio::task::spawn_blocking(move || {
        let mut client = Client::start(&["--server", &url], &[("SIGNALLAB_TOKEN", TOKEN)]);
        let (text, data, failed) = client.call("run_experiment", json!({ "template": "empty" }));
        assert!(!failed && data["outcome"] == "passed", "{text}");
        assert!(data["report_path"].as_str().unwrap().contains("data"), "the report stays on the server: {data}");
        let (text, _, failed) = client.call("listen", json!({ "bind": "127.0.0.1:0", "seconds": 1 }));
        assert!(failed && text.contains("this machine"), "{text}");
        client.close();

        let mut refused = Client::start(&["--server", &url], &[("SIGNALLAB_TOKEN", "wrong-token-but-long-enough-to-be-one")]);
        let (text, data, failed) = refused.call("run_experiment", json!({ "template": "empty" }));
        assert!(failed && data["error"]["code"] == "auth.required" && data["exit_code"] == 3, "{text}");
        refused.close();
    })
    .await
    .unwrap();
    running.stop().await;
}

#[test]
fn a_client_is_given_its_configuration() {
    let desktop = signallab(&["mcp", "--print-config", "claude-desktop"]);
    let config: Value = serde_json::from_str(&out(&desktop)).unwrap();
    let server = &config["mcpServers"]["signallab"];
    assert!(server["command"].as_str().unwrap().contains("signallab") && server["args"] == json!(["mcp"]), "{config}");
    let vscode: Value = serde_json::from_str(&out(&signallab(&["mcp", "--print-config", "vscode"]))).unwrap();
    assert_eq!(vscode["servers"]["signallab"]["type"], "stdio");
    let code = out(&signallab(&["mcp", "--print-config", "claude-code", "--server", "http://lab:1430"]));
    assert!(code.starts_with("claude mcp add signallab -e SIGNALLAB_TOKEN=") && code.trim_end().ends_with("mcp --server http://lab:1430"), "{code}");
    assert_eq!(common::code(&signallab(&["mcp", "--print-config", "notepad"])), 2);
}
