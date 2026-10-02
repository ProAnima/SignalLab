//! HTTP authentication and cookies against a loopback server that checks
//! them itself: Basic, Bearer, Digest (MD5 and SHA-256, a nonce that goes
//! stale) verified by its own hashing, a burst answering one challenge for all
//! its requests, the HTTP screen's cookie jar, and a run's — off for a file
//! from before version 8, which ran without one.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use md5::{Digest as _, Md5};
use serde_json::{json, Value};
use sha2::Sha256;
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_run::{Outcome, RunOptions};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn service() -> (Service, Arc<Recorder>) {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-auth-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        dir
    });
    let recorder = Recorder::new();
    (Service::new(Host::new(recorder.clone(), Capture::new()), Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))), recorder)
}

#[derive(Default)]
struct Counts {
    challenges: AtomicU32,
    granted: AtomicU32,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// `key="value", key=value` of an Authorization: Digest header.
fn digest_params(text: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        let Some((key, after)) = rest.split_once('=') else { break };
        let (value, after) = if let Some(quoted) = after.strip_prefix('"') {
            let end = quoted.find('"').unwrap_or(quoted.len());
            (quoted[..end].to_string(), &quoted[(end + 1).min(quoted.len())..])
        } else {
            let end = after.find(',').unwrap_or(after.len());
            (after[..end].trim().to_string(), &after[end..])
        };
        params.insert(key.trim().to_string(), value);
        rest = after.trim_start_matches([',', ' ']);
    }
    params
}

/// The server's own check of a Digest answer, for `password`, the RFC 7616 way.
fn digest_ok(header: &str, method: &str, password: &str, sha256: bool) -> Option<HashMap<String, String>> {
    let params = digest_params(header.strip_prefix("Digest ")?);
    let h = |text: String| if sha256 { hex(&Sha256::digest(text.as_bytes())) } else { hex(&Md5::digest(text.as_bytes())) };
    let ha1 = h(format!("{}:{}:{password}", params.get("username")?, params.get("realm")?));
    let ha2 = h(format!("{method}:{}", params.get("uri")?));
    let expected = h(format!("{ha1}:{}:{}:{}:{}:{ha2}", params.get("nonce")?, params.get("nc")?, params.get("cnonce")?, params.get("qop")?));
    (params.get("response")? == &expected).then_some(params)
}

/// Answers by path: /basic, /bearer, /digest (MD5), /digest256 (SHA-256) — a
/// nonce good for 5 uses, then stale — /digest512 (an algorithm nobody speaks),
/// /login (sets a session cookie) and /me (wants it).
async fn server() -> (String, Arc<Counts>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let counts = Arc::new(Counts::default());
    let shared = counts.clone();
    let nonce = Arc::new(AtomicU32::new(1));
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let (counts, nonce) = (shared.clone(), nonce.clone());
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") else {
                        match stream.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                        }
                        continue;
                    };
                    let head = String::from_utf8_lossy(&buffer[..end]).to_string();
                    let mut lines = head.split("\r\n");
                    let request_line = lines.next().unwrap_or_default().to_string();
                    let headers: HashMap<String, String> = lines.filter_map(|line| line.split_once(':')).map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string())).collect();
                    let length: usize = headers.get("content-length").and_then(|value| value.parse().ok()).unwrap_or(0);
                    while buffer.len() < end + 4 + length {
                        match stream.read(&mut chunk).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                        }
                    }
                    buffer.drain(..end + 4 + length);
                    let mut parts = request_line.split(' ');
                    let (method, target) = (parts.next().unwrap_or_default(), parts.next().unwrap_or_default());
                    let path = target.split('?').next().unwrap_or_default();
                    let authorization = headers.get("authorization").cloned().unwrap_or_default();
                    let current = format!("n{}", nonce.load(Ordering::SeqCst));
                    let (status, extra, body): (&str, String, String) = match path {
                        "/basic" if authorization == "Basic bGFiOnNlY3JldA==" => ("200 OK", String::new(), "basic ok".into()),
                        "/basic" => ("401 Unauthorized", "WWW-Authenticate: Basic realm=\"lab\"\r\n".into(), String::new()),
                        "/bearer" if authorization == "Bearer t0ken" => ("200 OK", String::new(), "bearer ok".into()),
                        "/bearer" => ("401 Unauthorized", "WWW-Authenticate: Bearer\r\n".into(), String::new()),
                        "/digest" | "/digest256" => {
                            let sha = path == "/digest256";
                            let algorithm = if sha { "SHA-256" } else { "MD5" };
                            match digest_ok(&authorization, method, "secret", sha) {
                                Some(params) if params["nonce"] == current && params["uri"] == target => {
                                    let used = u32::from_str_radix(&params["nc"], 16).unwrap_or(0);
                                    if used > 5 {
                                        // Good credentials, worn-out nonce: a fresh one, stale.
                                        nonce.fetch_add(1, Ordering::SeqCst);
                                        counts.challenges.fetch_add(1, Ordering::SeqCst);
                                        let fresh = format!("n{}", nonce.load(Ordering::SeqCst));
                                        ("401 Unauthorized", format!("WWW-Authenticate: Digest realm=\"lab\", nonce=\"{fresh}\", qop=\"auth\", algorithm={algorithm}, stale=true\r\n"), String::new())
                                    } else {
                                        counts.granted.fetch_add(1, Ordering::SeqCst);
                                        ("200 OK", String::new(), format!("digest ok {used}"))
                                    }
                                }
                                _ => {
                                    counts.challenges.fetch_add(1, Ordering::SeqCst);
                                    ("401 Unauthorized", format!("WWW-Authenticate: Basic realm=\"lab\", Digest realm=\"lab\", nonce=\"{current}\", qop=\"auth,auth-int\", opaque=\"op\", algorithm={algorithm}\r\n"), String::new())
                                }
                            }
                        }
                        "/digest512" => ("401 Unauthorized", "WWW-Authenticate: Digest realm=\"lab\", nonce=\"x\", algorithm=SHA-512-256\r\n".into(), String::new()),
                        "/login" => ("200 OK", "Set-Cookie: session=abc; Path=/; HttpOnly\r\n".into(), "in".into()),
                        "/me" if headers.get("cookie").is_some_and(|cookie| cookie.contains("session=abc")) => ("200 OK", String::new(), "you".into()),
                        "/me" => ("401 Unauthorized", String::new(), "who?".into()),
                        _ => ("404 Not Found", String::new(), String::new()),
                    };
                    let answer = format!("HTTP/1.1 {status}\r\n{extra}Content-Length: {}\r\n\r\n{body}", body.len());
                    if stream.write_all(answer.as_bytes()).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    (base, counts)
}

async fn request(service: &Service, url: String, auth: Value, cookies: bool) -> Value {
    let mut request = json!({ "method": "GET", "url": url, "headers": [], "timeout_ms": 3000 });
    if !auth.is_null() {
        request["auth"] = auth;
    }
    service.invoke("http_request", json!({ "request": request, "cookies": cookies })).await.unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn basic_bearer_and_digest_are_answered_as_the_server_checks_them() {
    let (service, _) = service();
    let (base, counts) = server().await;
    let basic = request(&service, format!("{base}/basic"), json!({ "scheme": "basic", "username": "lab", "password": "secret" }), false).await;
    assert_eq!((basic["status"].as_u64(), basic["body"].as_str()), (Some(200), Some("basic ok")));
    let wrong = request(&service, format!("{base}/basic"), json!({ "scheme": "basic", "username": "lab", "password": "nope" }), false).await;
    assert_eq!(wrong["status"], 401);
    let bearer = request(&service, format!("{base}/bearer"), json!({ "scheme": "bearer", "token": "t0ken" }), false).await;
    assert_eq!(bearer["status"], 200);

    for path in ["digest", "digest256"] {
        let url = format!("{base}/{path}?page=1");
        let digest = request(&service, url.clone(), json!({ "scheme": "digest", "username": "lab", "password": "secret" }), false).await;
        assert_eq!((digest["status"].as_u64(), digest["digest"]["challenged"].as_bool()), (Some(200), Some(true)), "{path}: {digest}");
        assert!(digest["body"].as_str().unwrap().starts_with("digest ok"), "the server's own hashing agreed: {digest}");
        let refused = request(&service, url, json!({ "scheme": "digest", "username": "lab", "password": "nope" }), false).await;
        assert_eq!((refused["status"].as_u64(), refused["digest"]["challenged"].as_bool()), (Some(401), Some(true)), "a wrong password stays a 401");
    }
    let unspoken = request(&service, format!("{base}/digest512"), json!({ "scheme": "digest", "username": "lab", "password": "secret" }), false).await;
    assert_eq!((unspoken["status"].as_u64(), unspoken["digest"]["error"]["code"].as_str(), unspoken["digest"]["error"]["params"]["algorithm"].as_str()), (Some(401), Some("http.digest_unsupported"), Some("SHA-512-256")), "{unspoken}");
    let plain = request(&service, format!("{base}/basic"), json!({ "scheme": "digest", "username": "lab", "password": "secret" }), false).await;
    assert_eq!(plain["digest"]["error"]["code"], "http.digest_not_offered", "Basic only: said so, not guessed");
    assert!(counts.granted.load(Ordering::SeqCst) >= 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_burst_answers_one_challenge_for_all_and_a_stale_nonce_once_more() {
    let (service, recorder) = service();
    let (base, counts) = server().await;
    let config = json!({ "method": "GET", "url": format!("{base}/digest"), "headers": [], "timeout_ms": 3000,
        "auth": { "scheme": "digest", "username": "lab", "password": "secret" }, "concurrency": 1, "total": 12 });
    service.invoke("http_burst_start", json!({ "config": config })).await.unwrap();
    let waiting = recorder.clone();
    let last = tokio::task::spawn_blocking(move || waiting.wait_for("http://burst-progress", Duration::from_secs(10), |report| report["done"] == true)).await.unwrap().unwrap();
    assert_eq!((last["ok"].as_u64(), last["failed"].as_u64()), (Some(12), Some(0)), "{last}");
    // 12 requests, a nonce good for 5: the first challenge, then a stale one after the 5th use.
    let challenges = counts.challenges.load(Ordering::SeqCst);
    assert!((2..=3).contains(&challenges), "one round trip per request after the first: {challenges} challenges");
    assert_eq!(counts.granted.load(Ordering::SeqCst), 12);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_screen_keeps_cookies_while_asked_and_forgets_them_when_cleared() {
    let (service, _) = service();
    let (base, _) = server().await;
    assert_eq!(request(&service, format!("{base}/login"), Value::Null, true).await["status"], 200);
    let jar = service.invoke("http_cookies", json!({})).await.unwrap();
    assert_eq!((jar[0]["name"].as_str(), jar[0]["value"].as_str(), jar[0]["http_only"].as_bool(), jar[0]["host_only"].as_bool()), (Some("session"), Some("abc"), Some(true), Some(true)), "{jar}");
    assert_eq!(request(&service, format!("{base}/me"), Value::Null, true).await["body"], "you");
    assert_eq!(request(&service, format!("{base}/me"), Value::Null, false).await["status"], 401, "without the jar, no cookie");
    service.invoke("http_cookies_clear", json!({})).await.unwrap();
    assert_eq!(service.invoke("http_cookies", json!({})).await.unwrap(), json!([]));
    assert_eq!(request(&service, format!("{base}/me"), Value::Null, true).await["status"], 401);
}

fn session_check(base: &str, version: u32) -> Value {
    let node = |id: &str, x: u32, kind: Value| { let mut kind = kind; kind["id"] = json!(id); kind["x"] = json!(x); kind["y"] = json!(0); kind };
    json!({ "version": version, "name": "Session", "params": [], "nodes": [
        node("start", 0, json!({ "type": "start" })),
        node("login", 200, json!({ "type": "http", "request": { "method": "GET", "url": format!("{base}/login"), "headers": [], "timeout_ms": 3000 } })),
        node("me", 400, json!({ "type": "http", "request": { "method": "GET", "url": format!("{base}/me"), "headers": [], "timeout_ms": 3000 } })),
        node("ok", 600, json!({ "type": "assert_status", "status": 200 })),
        node("digest", 800, json!({ "type": "http", "request": { "method": "GET", "url": format!("{base}/digest256"), "headers": [], "timeout_ms": 3000,
            "auth": { "scheme": "digest", "username": "lab", "password": "secret" } } })),
        node("granted", 1000, json!({ "type": "assert_status", "status": 200 })),
        node("end", 1200, json!({ "type": "end" })),
    ], "edges": [
        { "from": "start", "to": "login" }, { "from": "login", "to": "me" }, { "from": "me", "to": "ok" },
        { "from": "ok", "to": "digest" }, { "from": "digest", "to": "granted" }, { "from": "granted", "to": "end" },
    ] })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_run_keeps_its_own_cookies_and_an_old_file_runs_as_it_did() {
    let (service, _) = service();
    let (base, _) = server().await;
    let run = |doc: Experiment| async {
        let handle = service.run(doc, RunOptions { limit: Some(Duration::from_secs(20)), ..Default::default() }).await.unwrap();
        handle.finished().await
    };
    let current: Experiment = serde_json::from_value(session_check(&base, 8)).unwrap();
    assert!(current.cookies, "a version 8 document keeps cookies unless it says not to");
    let result = run(current).await;
    assert_eq!(result.outcome, Outcome::Passed, "{:?} {:?}", result.error, result.steps.iter().filter(|step| step.state == "failed").collect::<Vec<_>>());

    // The same flow in a version 7 file: opened, it keeps no cookies, so /me is refused as before.
    let parsed = service.invoke("experiment_parse", json!({ "text": session_check(&base, 7).to_string() })).await.unwrap();
    assert_eq!((parsed["version"].as_u64(), parsed["cookies"].as_bool()), (Some(8), Some(false)), "{parsed}");
    let legacy: Experiment = serde_json::from_value(parsed).unwrap();
    let result = run(legacy).await;
    assert_eq!(result.outcome, Outcome::Failed);
    let error = serde_json::to_value(result.error.unwrap()).unwrap();
    assert_eq!((error["code"].as_str(), error["node"].as_str(), error["params"]["actual"].as_str()), (Some("check.status_failed"), Some("ok"), Some("401")), "{error}");
}
