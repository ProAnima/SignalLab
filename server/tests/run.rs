//! `POST /api/run` end to end — a run waited for, streamed as lines, refused,
//! left behind by its client, and stopped by the server shutting down — and
//! `GET /api/openapi.json`, which must describe the command table as it is.

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_server::{serve, Config};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

const TOKEN: &str = "run-test-token-0123456789abcdef01234567";
const SECRET: &str = "s3cret-for-the-run-test";

/// One data folder for this test binary (the engine chooses it once per process).
fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-server-run-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("secrets")).unwrap();
        std::fs::write(dir.join("secrets").join("API_TOKEN"), format!("{SECRET}\n")).unwrap();
        dir
    })
    .clone()
}

struct Running {
    url: String,
    stop: Option<oneshot::Sender<()>>,
    done: tokio::task::JoinHandle<std::io::Result<()>>,
}

impl Running {
    async fn stop(mut self) {
        self.stop.take().unwrap().send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(10), &mut self.done).await.expect("the server stops").unwrap().unwrap();
    }
}

async fn start(token: Option<&str>) -> Running {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    let config = Config {
        listen: address,
        token: token.map(str::to_string),
        made_token: None,
        data_dir: Some(data_dir()),
        secrets_dir: data_dir().join("secrets"),
        ui_dir: None,
        allowed_hosts: vec![],
        secure_cookie: false,
    };
    let (stop, stopped) = oneshot::channel::<()>();
    let done = tokio::spawn(serve(config, listener, async move {
        let _ = stopped.await;
    }));
    Running { url: format!("http://{address}"), stop: Some(stop), done }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap()
}

/// Start → `middle` → End, with a parameter.
fn document(name: &str, middle: Value) -> Value {
    let mut middle = middle;
    middle["id"] = "middle".into();
    middle["x"] = 200.into();
    middle["y"] = 80.into();
    json!({
        "version": 9,
        "name": name,
        "params": [{ "name": "who", "value": "world" }],
        "nodes": [
            { "id": "start", "type": "start", "x": 0, "y": 80 },
            middle,
            { "id": "end", "type": "end", "x": 400, "y": 80 },
        ],
        "edges": [
            { "from": "start", "to": "middle", "port": "next" },
            { "from": "middle", "to": "end", "port": "next" },
        ],
    })
}

fn run_request(running: &Running, body: &Value) -> reqwest::RequestBuilder {
    client().post(format!("{}/api/run", running.url)).json(body)
}

/// Every NDJSON line of a response, parsed.
async fn lines(mut response: reqwest::Response) -> Vec<Value> {
    let mut text = String::new();
    while let Some(chunk) = response.chunk().await.unwrap() {
        text.push_str(std::str::from_utf8(&chunk).unwrap());
    }
    text.lines().map(|line| serde_json::from_str(line).unwrap_or_else(|_| panic!("not JSON: {line}"))).collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_run_is_waited_for_and_its_result_is_the_answer_failed_or_not() {
    let running = start(None).await;
    let passing = document("Greeting", json!({ "type": "log", "message": "hello {{who}} token={{secret.API_TOKEN}}" }));
    let response = run_request(&running, &json!({ "document": passing, "overrides": { "who": "lab" }, "seed": 9 })).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "application/json");
    let text = response.text().await.unwrap();
    assert!(!text.contains(SECRET), "secret values never leave the engine: {text}");
    let result: Value = serde_json::from_str(&text).unwrap();
    assert_eq!((result["outcome"].as_str(), result["seed"].as_u64(), result["experiment"].as_str()), (Some("passed"), Some(9), Some("Greeting")));
    assert!(result.get("error").is_none(), "no error, no field: {result}");
    let logged = result["steps"].as_array().unwrap().iter().find(|step| step["node_id"] == "middle" && step["state"] == "passed").unwrap();
    assert_eq!(logged["detail"], "hello lab token=••••");
    assert_eq!(result["params"]["who"], "lab");
    // The report is in the data folder and comes down through /api/files.
    let report_path = result["report_path"].as_str().unwrap();
    assert!(PathBuf::from(report_path).starts_with(data_dir()));
    let report: Value = client().get(format!("{}/api/files", running.url)).query(&[("path", report_path)]).send().await.unwrap().json().await.unwrap();
    assert_eq!((report["outcome"].as_str(), report["seed"].as_u64()), (Some("passed"), Some(9)));

    // A run that fails is still an answer: 200, outcome failed, the error.
    let port = std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let silent = json!({ "type": "wait_udp", "bind": format!("127.0.0.1:{port}"), "timeout_ms": 100 });
    let mut failing = document("Silent", silent);
    failing["edges"][1]["port"] = "matched".into();
    let failed: Value = run_request(&running, &json!({ "document": failing })).send().await.unwrap().json().await.unwrap();
    assert_eq!((failed["outcome"].as_str(), failed["error"]["code"].as_str(), failed["error"]["node"].as_str()), (Some("failed"), Some("wait.timeout"), Some("middle")));

    // A bundled template by its file name.
    let empty: Value = run_request(&running, &json!({ "template": "empty.json" })).send().await.unwrap().json().await.unwrap();
    assert_eq!((empty["outcome"].as_str(), empty["experiment"].as_str()), (Some("passed"), Some("Empty experiment")));
    running.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn requests_that_cannot_start_a_run_are_refused_with_an_engine_error() {
    let running = start(None).await;
    let refused = |body: Value| {
        let request = run_request(&running, &body);
        async move {
            let response = request.send().await.unwrap();
            let status = response.status().as_u16();
            (status, response.json::<Value>().await.unwrap()["code"].as_str().unwrap_or_default().to_string())
        }
    };
    assert_eq!(refused(json!({})).await, (400, "api.run_source".into()));
    assert_eq!(refused(json!({ "template": "empty", "sede": 1 })).await, (400, "api.run_invalid".into()), "a misspelled field");
    assert_eq!(refused(json!({ "template": "nope" })).await, (422, "api.template_unknown".into()));
    assert_eq!(refused(json!({ "template": "empty", "overrides": { "nobody": "x" } })).await, (422, "run.override_unknown".into()));
    assert_eq!(refused(json!({ "template": "empty", "profile": "Stage" })).await, (422, "profile.active_missing".into()));
    assert_eq!(refused(json!({ "template": "empty", "timeout": 0 })).await, (422, "run.limit_range".into()));
    assert_eq!(refused(json!({ "document": { "version": 99, "name": "x", "nodes": [], "edges": [] } })).await, (422, "doc.version_unsupported".into()));
    let needs_secret = document("Secretive", json!({ "type": "log", "message": "{{secret.NOT_SET}}" }));
    assert_eq!(refused(json!({ "document": needs_secret })).await, (422, "secret.missing".into()));
    let form = client().post(format!("{}/api/run", running.url)).header("content-type", "text/plain").body("{}").send().await.unwrap();
    assert_eq!(form.status(), 415);
    let jobs: Value = client().post(format!("{}/api/invoke/jobs_list", running.url)).json(&json!({})).send().await.unwrap().json().await.unwrap();
    assert_eq!(jobs, json!([]), "nothing was started");
    running.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn with_ndjson_every_step_arrives_as_a_line_and_the_result_comes_last() {
    let running = start(None).await;
    let body = json!({ "document": document("Lines", json!({ "type": "delay", "ms": 50 })) });
    let response = run_request(&running, &body).header("accept", "application/x-ndjson").send().await.unwrap();
    assert_eq!((response.status().as_u16(), response.headers()["content-type"].to_str().unwrap()), (200, "application/x-ndjson"));
    let lines = lines(response).await;
    let kinds: Vec<&str> = lines.iter().map(|line| line["type"].as_str().unwrap()).collect();
    assert_eq!(kinds.first(), Some(&"started"));
    assert_eq!(kinds.last(), Some(&"ended"));
    assert_eq!(kinds.iter().filter(|kind| **kind == "step").count(), 6, "{kinds:?}");
    let (started, ended) = (&lines[0], lines.last().unwrap());
    assert_eq!(started["job_id"], ended["job_id"]);
    assert_eq!(started["seed"], ended["seed"]);
    assert_eq!(ended["outcome"], "passed");
    assert_eq!(ended["steps"].as_array().unwrap().len(), 6);
    let order: Vec<(String, String)> = lines[1..lines.len() - 1].iter().map(|line| (line["node_id"].as_str().unwrap().into(), line["state"].as_str().unwrap().into())).collect();
    assert_eq!(order[2], ("middle".to_string(), "running".to_string()));
    running.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn with_a_token_runs_need_it_and_the_origin_rule_still_holds() {
    let running = start(Some(TOKEN)).await;
    let body = json!({ "template": "empty" });
    let denied = run_request(&running, &body).send().await.unwrap();
    assert_eq!((denied.status().as_u16(), denied.json::<Value>().await.unwrap()["code"].clone()), (401, json!("auth.required")));
    assert_eq!(run_request(&running, &body).bearer_auth("wrong-token").send().await.unwrap().status(), 401);
    // A script: the token and no Origin.
    let ok: Value = run_request(&running, &body).bearer_auth(TOKEN).send().await.unwrap().json().await.unwrap();
    assert_eq!(ok["outcome"], "passed");
    // A page of another site cannot start a run, token or not.
    let cross = run_request(&running, &body).bearer_auth(TOKEN).header("origin", "http://attacker.example").send().await.unwrap();
    assert_eq!((cross.status().as_u16(), cross.json::<Value>().await.unwrap()["code"].clone()), (403, json!("auth.origin")));
    let foreign = run_request(&running, &body).header("host", "attacker.example").send().await.unwrap();
    assert_eq!(foreign.status(), 401, "with a token any host name is allowed, but the token is still needed");
    let openapi = client().get(format!("{}/api/openapi.json", running.url)).send().await.unwrap();
    assert_eq!(openapi.status(), 401, "the description is behind the token too");
    running.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_that_goes_away_leaves_the_run_to_finish_and_save_its_report() {
    let running = start(None).await;
    let body = json!({ "document": document("Abandoned", json!({ "type": "delay", "ms": 800 })) });
    let mut response = run_request(&running, &body).header("accept", "application/x-ndjson").send().await.unwrap();
    let first = response.chunk().await.unwrap().unwrap();
    let started: Value = serde_json::from_slice(first.split(|byte| *byte == b'\n').next().unwrap()).unwrap();
    assert_eq!(started["type"], "started");
    let id = started["job_id"].as_u64().unwrap();
    drop(response);

    // Still a job after the client left, then gone once the run has ended…
    let jobs = || async {
        client().post(format!("{}/api/invoke/jobs_list", running.url)).json(&json!({})).send().await.unwrap().json::<Value>().await.unwrap()
    };
    assert!(jobs().await.as_array().unwrap().iter().any(|job| job["id"] == id), "the run goes on");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while jobs().await.as_array().unwrap().iter().any(|job| job["id"] == id) {
        assert!(tokio::time::Instant::now() < deadline, "the run never ended");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    // …with its report saved, passed. (Every test's server counts jobs from 1, so the name is what tells.)
    let report = std::fs::read_dir(data_dir().join("runs"))
        .unwrap()
        .map(|entry| serde_json::from_slice::<Value>(&std::fs::read(entry.unwrap().path()).unwrap()).unwrap())
        .find(|report| report["experiment"] == "Abandoned")
        .expect("the run saved its report");
    assert_eq!(report["outcome"], "passed");
    running.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_server_that_stops_stops_the_run_and_says_so() {
    let running = start(None).await;
    let body = json!({ "document": document("Interrupted", json!({ "type": "delay", "ms": 30000 })) });
    let response = run_request(&running, &body).header("accept", "application/x-ndjson").send().await.unwrap();
    let reading = tokio::spawn(lines(response));
    tokio::time::sleep(Duration::from_millis(300)).await;
    running.stop().await;
    let lines = tokio::time::timeout(Duration::from_secs(5), reading).await.expect("the answer ended").unwrap();
    let ended = lines.last().unwrap();
    assert_eq!((ended["type"].as_str(), ended["outcome"].as_str()), (Some("ended"), Some("stopped")));
    assert!(ended.get("report_path").is_none(), "a stopped run saves no report, as in the app");
}

#[tokio::test]
async fn the_api_description_is_served_and_lists_every_command_there_is() {
    let running = start(None).await;
    let response = client().get(format!("{}/api/openapi.json", running.url)).send().await.unwrap();
    assert_eq!((response.status().as_u16(), response.headers()["content-type"].to_str().unwrap()), (200, "application/json"));
    let api: Value = response.json().await.unwrap();
    assert!(api["openapi"].as_str().unwrap().starts_with("3.1"));
    for path in ["/api/health", "/api/run", "/api/invoke/{command}", "/api/events", "/api/files", "/api/openapi.json"] {
        assert!(api["paths"].get(path).is_some(), "{path} is described");
    }
    let invoke = &api["paths"]["/api/invoke/{command}"]["post"];
    let listed: BTreeSet<String> = invoke["parameters"][0]["schema"]["enum"].as_array().unwrap().iter().map(|name| name.as_str().unwrap().to_string()).collect();

    // The command table, read off its match arms.
    let service = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/src/service.rs")).unwrap().replace("\r\n", "\n");
    let table = &service[service.find("match command {").unwrap()..service.find("_ => Err(EngineError::new(\"command.unknown\")").unwrap()];
    let arm = regex_lite_arms(table);
    assert!(arm.len() > 30, "the scan found the commands: {arm:?}");
    assert_eq!(listed, arm, "openapi.json lists exactly the commands of engine/src/service.rs");
    let described: BTreeSet<String> = invoke["x-signallab-commands"].as_object().unwrap().keys().cloned().collect();
    assert_eq!(described, arm, "and describes each");
    let jobs: BTreeSet<String> = invoke["x-signallab-commands"].as_object().unwrap().iter().filter(|(_, command)| command["job"] == true).map(|(name, _)| name.clone()).collect();
    let job_commands: BTreeSet<String> = signal_lab_engine::service::JOB_COMMANDS.iter().map(|name| name.to_string()).collect();
    assert_eq!(jobs, job_commands, "the commands that start a job are marked");
    let templates: Vec<&str> = api["components"]["schemas"]["RunRequest"]["properties"]["template"]["enum"].as_array().unwrap().iter().map(|name| name.as_str().unwrap()).collect();
    let mut files: Vec<String> = std::fs::read_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../experiments/templates"))
        .unwrap()
        .filter_map(|entry| entry.unwrap().file_name().to_string_lossy().strip_suffix(".json").map(str::to_string))
        .collect();
    files.sort();
    let mut templates: Vec<String> = templates.into_iter().map(str::to_string).collect();
    templates.sort();
    assert_eq!(templates, files, "the templates a run may name");
    running.stop().await;
}

/// `"name" =>` at the start of a line of the match: the commands.
fn regex_lite_arms(table: &str) -> BTreeSet<String> {
    table
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"'))
        .filter_map(|rest| rest.split_once("\" =>").map(|(name, _)| name.to_string()))
        .collect()
}
