//! `signallab` as a pipeline runs it: the built binary, its exit codes, what it
//! prints, the JUnit and report files it writes, sends that arrive on a socket
//! of the test's own, and runs on a server started here.

use std::net::{SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_server::{serve, Config};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

const TOKEN: &str = "cli-test-token-0123456789abcdef0123456789";

/// A folder of the test's own.
fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("signallab-cli-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The binary, in English, with nothing of the environment's Signal Lab settings or CI.
fn signallab(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_signallab"))
        .args(args)
        .env("SIGNALLAB_LANG", "en")
        .env_remove("SIGNALLAB_SERVER")
        .env_remove("SIGNALLAB_TOKEN")
        .env_remove("SIGNALLAB_TOKEN_FILE")
        .env_remove("GITHUB_ACTIONS")
        .output()
        .unwrap()
}

fn code(output: &Output) -> i32 {
    output.status.code().unwrap()
}

fn out(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn err(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Start → `middle` → End, with a parameter `who`, written to a file.
fn experiment(dir: &Path, name: &str, middle: Value, port: &str) -> String {
    let mut middle = middle;
    middle["id"] = "middle".into();
    middle["x"] = 200.into();
    middle["y"] = 80.into();
    let document = json!({
        "version": 6,
        "name": name,
        "params": [{ "name": "who", "value": "world" }],
        "nodes": [{ "id": "start", "type": "start", "x": 0, "y": 80 }, middle, { "id": "end", "type": "end", "x": 400, "y": 80 }],
        "edges": [{ "from": "start", "to": "middle", "port": "next" }, { "from": "middle", "to": "end", "port": port }],
    });
    let path = dir.join(format!("{}.json", name.to_lowercase().replace(' ', "-")));
    std::fs::write(&path, serde_json::to_string_pretty(&document).unwrap()).unwrap();
    path.display().to_string()
}

/// A wait on a port nobody sends to: it times out, so the run fails.
fn silent(dir: &Path) -> String {
    let port = UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    experiment(dir, "Silent", json!({ "type": "wait_udp", "bind": format!("127.0.0.1:{port}"), "timeout_ms": 100 }), "matched")
}

#[test]
fn version_and_the_bundled_templates() {
    let version = signallab(&["version"]);
    assert_eq!((code(&version), out(&version).trim()), (0, format!("signallab {}", env!("CARGO_PKG_VERSION")).as_str()));
    let templates = signallab(&["templates"]);
    assert_eq!(code(&templates), 0);
    for name in ["empty", "http-check", "osc-ping-reply", "poll-until-ready", "flaky-api"] {
        assert!(out(&templates).contains(name), "{}", out(&templates));
    }
    assert_eq!(code(&signallab(&["run", "empty"])), 0, "a template runs by its name");
}

#[test]
fn a_passing_run_exits_0_and_leaves_its_junit_and_report() {
    let dir = folder("pass");
    let file = experiment(&dir, "Greeting", json!({ "type": "log", "message": "hello {{who}}" }), "next");
    let (junit, report) = (dir.join("out/junit.xml"), dir.join("out/run.json"));
    let output = signallab(&["run", &file, "--param", "who=CI", "--seed", "7", "--junit", junit.to_str().unwrap(), "--report", report.to_str().unwrap()]);
    assert_eq!(code(&output), 0, "{}{}", out(&output), err(&output));
    assert!(out(&output).contains("✔ Greeting passed in"), "{}", out(&output));
    assert!(err(&output).contains("hello CI"), "the steps, as the app's timeline shows them: {}", err(&output));
    let xml = std::fs::read_to_string(&junit).unwrap();
    assert!(xml.contains("<testsuite name=\"Greeting\" tests=\"3\" failures=\"0\" errors=\"0\" skipped=\"0\""), "{xml}");
    assert!(xml.contains("<property name=\"seed\" value=\"7\" />"));
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!((saved["outcome"].as_str(), saved["seed"].as_u64(), saved["params"]["who"].as_str()), (Some("passed"), Some(7), Some("CI")));
}

#[test]
fn a_failing_run_exits_1_says_why_and_how_to_run_it_again() {
    let dir = folder("fail");
    let file = silent(&dir);
    let junit = dir.join("junit.xml");
    let output = signallab(&["run", &file, "--seed", "11", "--junit", junit.to_str().unwrap()]);
    assert_eq!(code(&output), 1, "{}{}", out(&output), err(&output));
    assert!(out(&output).contains("✖ Silent failed after"), "{}", out(&output));
    assert!(err(&output).contains("--seed 11"), "{}", err(&output));
    let xml = std::fs::read_to_string(&junit).unwrap();
    assert!(xml.contains("type=\"wait.timeout\"") && xml.contains("failures=\"1\""), "{xml}");

    // The same in Russian.
    let russian = Command::new(env!("CARGO_BIN_EXE_signallab")).args(["--lang", "ru", "run", &file]).env_remove("GITHUB_ACTIONS").output().unwrap();
    assert_eq!(code(&russian), 1);
    assert!(out(&russian).contains("Silent: не пройден"), "{}", out(&russian));

    // On GitHub Actions the failure is also an annotation.
    let annotated = Command::new(env!("CARGO_BIN_EXE_signallab")).args(["run", &file]).env("SIGNALLAB_LANG", "en").env("GITHUB_ACTIONS", "true").output().unwrap();
    assert!(out(&annotated).contains("::error title=Signal Lab · Silent::"), "{}", out(&annotated));
}

#[test]
fn wrong_input_exits_2_before_anything_runs() {
    let dir = folder("invalid");
    let unknown = signallab(&["run", "no-such-experiment"]);
    assert_eq!(code(&unknown), 2);
    assert!(err(&unknown).contains("neither a file nor a bundled template"), "{}", err(&unknown));
    assert_eq!(code(&signallab(&["run", "empty", "--param", "nobody=1"])), 2, "a value for a parameter no experiment has");
    assert_eq!(code(&signallab(&["run", "empty", "--param", "novalue"])), 2);
    let secretive = experiment(&dir, "Secretive", json!({ "type": "log", "message": "{{secret.NOT_SET}}" }), "next");
    let missing = signallab(&["run", &secretive]);
    assert_eq!(code(&missing), 2);
    assert!(err(&missing).contains("SIGNALLAB_SECRET_NOT_SET"), "the command line's own wording: {}", err(&missing));
    // A broken third file stops the first one too: nothing runs.
    let broken = dir.join("broken.json");
    std::fs::write(&broken, r#"{ "version": 99, "name": "x", "nodes": [], "edges": [] }"#).unwrap();
    let both = signallab(&["run", "empty", broken.to_str().unwrap()]);
    assert_eq!(code(&both), 2);
    assert!(!out(&both).contains("✔"), "{}", out(&both));
    assert_eq!(code(&signallab(&["validate", "empty", "http-check"])), 0);
    assert_eq!(code(&signallab(&["validate", broken.to_str().unwrap()])), 2);
    assert_eq!(code(&signallab(&["run", "empty", "--timeout", "0"])), 2);
}

#[test]
fn a_secret_comes_from_the_environment_and_never_shows() {
    let dir = folder("secret");
    let file = experiment(&dir, "Secretive", json!({ "type": "log", "message": "token={{secret.API_TOKEN}}" }), "next");
    let output = Command::new(env!("CARGO_BIN_EXE_signallab"))
        .args(["--json", "run", &file])
        .env("SIGNALLAB_SECRET_API_TOKEN", "do-not-print-me")
        .output()
        .unwrap();
    assert_eq!(code(&output), 0, "{}", err(&output));
    assert!(!out(&output).contains("do-not-print-me") && !err(&output).contains("do-not-print-me"));
    assert!(out(&output).contains("token=••••"), "{}", out(&output));
}

#[test]
fn json_mode_is_one_object_per_line() {
    let output = signallab(&["--json", "run", "empty"]);
    assert_eq!(code(&output), 0);
    let lines: Vec<Value> = out(&output).lines().map(|line| serde_json::from_str(line).unwrap_or_else(|_| panic!("not JSON: {line}"))).collect();
    let kinds: Vec<&str> = lines.iter().map(|line| line["type"].as_str().unwrap()).collect();
    assert_eq!((kinds.first(), kinds[kinds.len() - 2], kinds.last()), (Some(&"started"), "ended", Some(&"summary")), "{kinds:?}");
    assert!(kinds.contains(&"step"));
    assert_eq!(lines.last().unwrap()["exit_code"], 0);
    assert!(err(&output).is_empty(), "nothing for people in --json mode: {}", err(&output));
}

/// A socket of the test's own, and what arrives on it.
fn receiver() -> (UdpSocket, SocketAddr) {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let address = socket.local_addr().unwrap();
    (socket, address)
}

fn received(socket: &UdpSocket) -> Vec<u8> {
    let mut buffer = [0u8; 2048];
    let (n, _) = socket.recv_from(&mut buffer).expect("a datagram arrives");
    buffer[..n].to_vec()
}

#[test]
fn sends_arrive_as_the_app_would_send_them() {
    let (socket, address) = receiver();
    let osc = signallab(&["send", "osc", &address.to_string(), "/cue/go", "i:3", "s:hi", "T", "0.5"]);
    assert_eq!(code(&osc), 0, "{}", err(&osc));
    let packet = received(&socket);
    assert!(packet.starts_with(b"/cue/go\0,isTf\0\0\0"), "{packet:?}");

    let udp = signallab(&["send", "udp", &address.to_string(), "--hex", "de ad be ef"]);
    assert_eq!(code(&udp), 0, "{}", err(&udp));
    assert_eq!(received(&socket), [0xde, 0xad, 0xbe, 0xef]);
    assert!(out(&udp).contains("sent 4 bytes →"), "{}", out(&udp));

    assert_eq!(code(&signallab(&["send", "osc", "127.0.0.1", "/x"])), 2, "a target without a port is the invocation's fault");
    assert_eq!(code(&signallab(&["send", "osc", &address.to_string(), "/x", "i:nope"])), 2);
    // Nothing listens on this port: the request fails, which is the send failing.
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let http = signallab(&["send", "http", "GET", &format!("http://127.0.0.1:{closed}/")]);
    assert_eq!(code(&http), 1, "{}", err(&http));
}

#[test]
fn a_library_signal_fires_by_its_name() {
    let dir = folder("fire");
    let (socket, address) = receiver();
    let library = dir.join("signals.json");
    std::fs::write(
        &library,
        serde_json::to_string(&json!({ "version": 2, "signals": [
            { "id": "go", "name": "Go cue", "group": "Show", "body": { "transport": "osc", "target": address.to_string(), "address": "/go", "args": [{ "type": "int", "value": 1 }] } },
        ] }))
        .unwrap(),
    )
    .unwrap();
    let fired = signallab(&["fire", "go cue", "--library", library.to_str().unwrap()]);
    assert_eq!(code(&fired), 0, "{}", err(&fired));
    assert!(received(&socket).starts_with(b"/go\0,i\0\0"));
    assert_eq!(code(&signallab(&["fire", "nothing", "--library", library.to_str().unwrap()])), 2);
}

struct Running {
    url: String,
    stop: Option<oneshot::Sender<()>>,
    done: tokio::task::JoinHandle<std::io::Result<()>>,
}

async fn start_server(dir: &Path) -> Running {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let config = Config {
        listen: address,
        token: Some(TOKEN.into()),
        made_token: None,
        data_dir: Some(dir.to_path_buf()),
        secrets_dir: dir.join("secrets"),
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

/// The binary, without blocking the server this test runs.
async fn signallab_async(args: Vec<String>) -> Output {
    tokio::task::spawn_blocking(move || signallab(&args.iter().map(String::as_str).collect::<Vec<_>>())).await.unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_run_on_a_server_reads_like_one_here_and_its_report_comes_down() {
    let dir = folder("server");
    let running = start_server(&dir.join("data")).await;
    let token = dir.join("token.txt");
    std::fs::write(&token, format!("{TOKEN}\n")).unwrap();
    let file = experiment(&dir, "Greeting", json!({ "type": "log", "message": "hello {{who}}" }), "next");
    let report = dir.join("report.json");
    let output = signallab_async(vec![
        "run".into(), file.clone(), "--param".into(), "who=server".into(), "--server".into(), running.url.clone(),
        "--token-file".into(), token.display().to_string(), "--report".into(), report.display().to_string(),
    ])
    .await;
    assert_eq!(code(&output), 0, "{}{}", out(&output), err(&output));
    assert!(err(&output).contains("hello server"), "the steps arrive as lines: {}", err(&output));
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(saved["outcome"], "passed", "the report, downloaded from the server");

    let failing = silent(&dir);
    let failed = signallab_async(vec!["run".into(), failing, "--server".into(), running.url.clone(), "--token-file".into(), token.display().to_string()]).await;
    assert_eq!(code(&failed), 1, "{}", err(&failed));

    // A wrong token, and a server that is not there: nothing could run.
    std::fs::write(dir.join("wrong.txt"), "wrong-token-but-long-enough-to-be-one").unwrap();
    let refused = signallab_async(vec!["run".into(), "empty".into(), "--server".into(), running.url.clone(), "--token-file".into(), dir.join("wrong.txt").display().to_string()]).await;
    assert_eq!(code(&refused), 3, "{}", err(&refused));
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let away = signallab_async(vec!["run".into(), "empty".into(), "--server".into(), format!("http://127.0.0.1:{closed}")]).await;
    assert_eq!(code(&away), 3, "{}", err(&away));
    assert!(err(&away).contains("Cannot reach the server"), "{}", err(&away));

    running.stop.unwrap().send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(10), running.done).await.unwrap().unwrap().unwrap();
}
