//! `signallab doctor`: what it looks at, as a script reads it (--json), with a
//! lab server whose token is right and one whose token is not. `firewall allow`
//! is not run on Windows here: it asks for administrator rights on the screen.

mod common;

use common::*;
use serde_json::Value;

fn report(args: Vec<String>) -> (i32, Value) {
    let all: Vec<String> = [vec!["--json".to_string(), "doctor".to_string()], args].concat();
    let output = signallab(&all.iter().map(String::as_str).collect::<Vec<_>>());
    let value: Value = serde_json::from_str(out(&output).trim()).unwrap_or_else(|_| panic!("doctor --json prints one JSON object: {}{}", out(&output), err(&output)));
    (code(&output), value)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn doctor_reports_the_network_the_firewall_and_a_server() {
    let dir = folder("doctor");
    let running = start_server(&dir.join("data")).await;
    let token = dir.join("token.txt");
    std::fs::write(&token, TOKEN).unwrap();
    let wrong = dir.join("wrong.txt");
    std::fs::write(&wrong, "wrong-token-but-long-enough-to-be-one").unwrap();
    let url = running.url.clone();

    let (good, bad) = tokio::task::spawn_blocking(move || {
        let good = report(vec!["--server".into(), url.clone(), "--token-file".into(), token.display().to_string()]);
        let bad = report(vec!["--server".into(), url, "--token-file".into(), wrong.display().to_string()]);
        (good, bad)
    })
    .await
    .unwrap();

    let (exit, value) = good;
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    assert!(value["network"]["address"].is_string() && value["data_dir"]["path"].is_string(), "{value}");
    assert!(value["firewall"].is_array());
    if cfg!(windows) {
        let first = &value["firewall"][0]["status"];
        assert!(first["applies"] == true && first["program"].as_str().unwrap().ends_with("signallab.exe"), "{value}");
    }
    assert_eq!(value["server"]["ok"], true, "{value}");
    let problems = value["problems"].as_u64().unwrap();
    assert_eq!(exit, if problems == 0 { 0 } else { 1 }, "the exit code says whether anything is in the way: {value}");

    let (exit, value) = bad;
    assert_eq!((exit, value["server"]["ok"].clone(), value["server"]["error"]["code"].clone()), (1, Value::Bool(false), Value::from("auth.required")), "{value}");
    running.stop().await;
}

#[test]
#[cfg(not(windows))]
fn firewall_allow_elsewhere_says_what_to_open() {
    let output = signallab(&["firewall", "allow"]);
    assert_eq!(code(&output), 0);
    assert!(err(&output).contains("ufw") || err(&output).contains("nothing to change"), "{}", err(&output));
}
