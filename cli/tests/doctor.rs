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

/// `doctor` says its lines in the language asked for, as the rest of the command line does.
#[test]
fn doctor_speaks_the_chosen_language() {
    let ru = signallab(&["--lang", "ru", "doctor"]);
    let said = err(&ru);
    assert!(said.contains("Сеть:") && said.contains("Папка данных:"), "{said}");
    assert!(!said.contains("Network:") && !said.contains("Data folder:") && !said.contains("Firewall"), "no English is left: {said}");
    let verdict = out(&ru);
    assert!(verdict.contains("ничего не мешает") || verdict.contains("найдено препятствий"), "{verdict}");
    assert!(!verdict.contains("in the way"), "{verdict}");

    let en = signallab(&["doctor"]);
    assert!(err(&en).contains("Network:") && err(&en).contains("Data folder:"), "{}", err(&en));
    assert!(out(&en).contains("in the way"), "{}", out(&en));
}

#[test]
#[cfg(not(windows))]
fn firewall_allow_elsewhere_speaks_the_chosen_language() {
    let output = signallab(&["--lang", "ru", "firewall", "allow"]);
    assert_eq!(code(&output), 0);
    let said = err(&output);
    assert!(said.contains("брандмауэр") || said.contains("ufw"), "{said}");
    assert!(!said.contains("nothing to change") && !said.contains("lets in only"), "{said}");
}

/// A library that is not there is said as this machine's matter, not as a server's data folder.
#[test]
fn a_missing_library_is_not_blamed_on_a_server() {
    let missing = folder("no-library").join("absent.json").display().to_string();
    for args in [vec!["emulators", "--library", missing.as_str()], vec!["fire", "go", "--library", missing.as_str()]] {
        let output = signallab(&args);
        assert_eq!(code(&output), 2, "{args:?}");
        let said = err(&output);
        assert!(said.contains("There is no library at") && said.contains("absent.json") && !said.contains("server"), "{args:?}: {said}");
    }
}
