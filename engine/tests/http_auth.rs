//! HTTP authentication and cookies against a loopback server that checks
//! them itself: Basic, Bearer, Digest (MD5 and SHA-256, a nonce that goes
//! stale) verified by its own hashing, a burst answering one challenge for all
//! its requests — in parallel too, against a server that refuses a count used
//! twice and keeps `-sess`'s first hash — Digest behind redirects, the HTTP
//! screen's cookie jar, and a run's — off for a file from before version 8,
//! which ran without one — and Basic's base64 masked like the secret in it.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use md5::{Digest as _, Md5};
use serde_json::{json, Value};
use sha2::Sha256;
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_run::{Outcome, RunOptions};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::{FileStore, MASK};
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// The one secret the tests read, as a server is given it.
const LAB_PASS: &str = "s3cret-pass";

fn service() -> (Service, Arc<Recorder>) {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-auth-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        dir
    });
    let recorder = Recorder::new();
    let store = FileStore::new(None).with_env(|key| (key == "SIGNALLAB_SECRET_LAB_PASS").then(|| LAB_PASS.to_string()));
    (Service::new(Host::new(recorder.clone(), Capture::new()), Mode::Server, Arc::new(store)), recorder)
}

#[derive(Default)]
struct Counts {
    challenges: AtomicU32,
    granted: AtomicU32,
    /// Answers to /strict that used a count already used with their nonce.
    replays: AtomicU32,
    /// Requests that carried an Authorization header.
    authorized: AtomicU32,
    /// /strict's (nonce, nc) pairs so far, and each nonce's first `-sess` hash.
    seen: Mutex<HashSet<(String, String)>>,
    sess: Mutex<HashMap<String, String>>,
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
    digest_check(header, method, password, sha256, None)
}

/// With `sess`: MD5-sess as RFC 2617 has a server do it — the first hash made
/// once per nonce, from the first answer's client nonce, and kept.
fn digest_check(header: &str, method: &str, password: &str, sha256: bool, sess: Option<&Mutex<HashMap<String, String>>>) -> Option<HashMap<String, String>> {
    let params = digest_params(header.strip_prefix("Digest ")?);
    let h = |text: String| if sha256 { hex(&Sha256::digest(text.as_bytes())) } else { hex(&Md5::digest(text.as_bytes())) };
    let mut ha1 = h(format!("{}:{}:{password}", params.get("username")?, params.get("realm")?));
    if let Some(sess) = sess {
        let first = h(format!("{ha1}:{}:{}", params.get("nonce")?, params.get("cnonce")?));
        ha1 = sess.lock().unwrap().entry(params.get("nonce")?.clone()).or_insert(first).clone();
    }
    let ha2 = h(format!("{method}:{}", params.get("uri")?));
    let expected = h(format!("{ha1}:{}:{}:{}:{}:{ha2}", params.get("nonce")?, params.get("nc")?, params.get("cnonce")?, params.get("qop")?));
    (params.get("response")? == &expected).then_some(params)
}

/// Answers by path: /basic, /bearer, /digest (MD5), /digest256 (SHA-256) — a
/// nonce good for 5 uses, then stale — /digest512 (an algorithm nobody speaks),
/// /strict (MD5-sess, a nonce good for 20, a count used twice refused),
/// /old (302 to /digest?from=old), /post-old (303 to /digest), /away?to=URL
/// (302 there), /loop (302 to itself), /echo (the Authorization it got, as the
/// body and as X-Echo), /login (sets a session cookie) and /me (wants it).
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
                    if !authorization.is_empty() {
                        counts.authorized.fetch_add(1, Ordering::SeqCst);
                    }
                    let number = nonce.load(Ordering::SeqCst);
                    let current = format!("n{number}");
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
                        "/strict" => {
                            let challenge = |nonce: &str, stale: bool| format!("WWW-Authenticate: Digest realm=\"lab\", nonce=\"{nonce}\", qop=\"auth\", algorithm=MD5-sess{}\r\n", if stale { ", stale=true" } else { "" });
                            match digest_check(&authorization, method, "secret", false, Some(&counts.sess)) {
                                Some(params) if params["nonce"] == current && params["uri"] == target => {
                                    let used = u32::from_str_radix(&params["nc"], 16).unwrap_or(0);
                                    if !counts.seen.lock().unwrap().insert((params["nonce"].clone(), params["nc"].clone())) {
                                        counts.replays.fetch_add(1, Ordering::SeqCst);
                                        ("401 Unauthorized", challenge(&current, false), String::new())
                                    } else if used > 20 {
                                        // Worn out: the next nonce, once, however many ask at the same moment.
                                        let _ = nonce.compare_exchange(number, number + 1, Ordering::SeqCst, Ordering::SeqCst);
                                        counts.challenges.fetch_add(1, Ordering::SeqCst);
                                        ("401 Unauthorized", challenge(&format!("n{}", nonce.load(Ordering::SeqCst)), true), String::new())
                                    } else {
                                        counts.granted.fetch_add(1, Ordering::SeqCst);
                                        ("200 OK", String::new(), format!("strict ok {used}"))
                                    }
                                }
                                _ => {
                                    counts.challenges.fetch_add(1, Ordering::SeqCst);
                                    ("401 Unauthorized", challenge(&current, false), String::new())
                                }
                            }
                        }
                        "/old" => ("302 Found", "Location: /digest?from=old\r\n".into(), String::new()),
                        "/post-old" => ("303 See Other", "Location: /digest\r\n".into(), String::new()),
                        "/away" => ("302 Found", format!("Location: {}\r\n", target.split_once("?to=").map(|(_, to)| to).unwrap_or("/")), String::new()),
                        "/loop" => ("302 Found", "Location: /loop\r\n".into(), String::new()),
                        "/echo" => ("200 OK", format!("X-Echo: {authorization}\r\n"), authorization.clone()),
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
    let current: Experiment = serde_json::from_value(session_check(&base, 9)).unwrap();
    assert!(current.cookies, "a current document keeps cookies unless it says not to");
    let result = run(current).await;
    assert_eq!(result.outcome, Outcome::Passed, "{:?} {:?}", result.error, result.steps.iter().filter(|step| step.state == "failed").collect::<Vec<_>>());

    // The same flow in a version 7 file: opened, it keeps no cookies, so /me is refused as before.
    let parsed = service.invoke("experiment_parse", json!({ "text": session_check(&base, 7).to_string() })).await.unwrap();
    assert_eq!((parsed["version"].as_u64(), parsed["cookies"].as_bool()), (Some(9), Some(false)), "{parsed}");
    let jarred = service.invoke("experiment_parse", json!({ "text": session_check(&base, 8).to_string() })).await.unwrap();
    assert_eq!((jarred["version"].as_u64(), jarred["cookies"].as_bool()), (Some(9), Some(true)), "a version 8 file had the jar: {jarred}");
    let legacy: Experiment = serde_json::from_value(parsed).unwrap();
    let result = run(legacy).await;
    assert_eq!(result.outcome, Outcome::Failed);
    let error = serde_json::to_value(result.error.unwrap()).unwrap();
    assert_eq!((error["code"].as_str(), error["node"].as_str(), error["params"]["actual"].as_str()), (Some("check.status_failed"), Some("ok"), Some("401")), "{error}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_burst_in_parallel_never_sends_a_count_twice_and_keeps_its_client_nonce() {
    let (service, recorder) = service();
    let (base, counts) = server().await;
    // Eight at once over a nonce good for 20: requests that meet the same new
    // nonce together count on from each other, and -sess's first hash holds.
    let config = json!({ "method": "GET", "url": format!("{base}/strict"), "headers": [], "timeout_ms": 3000,
        "auth": { "scheme": "digest", "username": "lab", "password": "secret" }, "concurrency": 8, "total": 120 });
    service.invoke("http_burst_start", json!({ "config": config })).await.unwrap();
    let waiting = recorder.clone();
    let last = tokio::task::spawn_blocking(move || waiting.wait_for("http://burst-progress", Duration::from_secs(20), |report| report["done"] == true)).await.unwrap().unwrap();
    assert_eq!((last["ok"].as_u64(), last["failed"].as_u64()), (Some(120), Some(0)), "{last}");
    assert_eq!(counts.replays.load(Ordering::SeqCst), 0, "no count sent twice with one nonce");
    assert_eq!(counts.granted.load(Ordering::SeqCst), 120);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn digest_behind_a_redirect_answers_the_url_that_asks_and_never_another_origin() {
    let (service, _) = service();
    let (base, _) = server().await;
    let lab = json!({ "scheme": "digest", "username": "lab", "password": "secret" });
    // 302 within the origin: answered with the path that asked (the server checks the uri).
    let moved = request(&service, format!("{base}/old"), lab.clone(), false).await;
    assert_eq!((moved["status"].as_u64(), moved["digest"]["challenged"].as_bool()), (Some(200), Some(true)), "{moved}");
    // 303 after a POST: the request goes on as GET, and the answer hashes GET.
    let posted = service.invoke("http_request", json!({ "request": { "method": "POST", "url": format!("{base}/post-old"), "headers": [["Content-Type", "application/json"]],
        "body": "{\"a\":1}", "timeout_ms": 3000, "auth": lab }, "cookies": false })).await.unwrap();
    assert_eq!((posted["status"].as_u64(), posted["digest"]["challenged"].as_bool()), (Some(200), Some(true)), "{posted}");

    // Sent on to another origin that asks: not answered, and it never gets an Authorization.
    let (other, elsewhere) = server().await;
    let away = request(&service, format!("{base}/away?to={other}/digest"), lab.clone(), false).await;
    assert_eq!((away["status"].as_u64(), away["digest"]["error"]["code"].as_str(), away["digest"]["error"]["params"]["origin"].as_str()),
        (Some(401), Some("http.digest_other_origin"), Some(other.as_str())), "{away}");
    assert_eq!(elsewhere.authorized.load(Ordering::SeqCst), 0);

    let looped = request(&service, format!("{base}/loop"), lab, false).await;
    assert_eq!((looped["status"].as_u64(), looped["cause"].as_str()), (Some(0), Some("failed")), "{looped}");
    assert!(looped["error"].as_str().unwrap().contains("too many redirects"), "{looped}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn basic_credentials_are_masked_as_base64_too() {
    let (service, recorder) = service();
    let (base, _) = server().await;
    let sent = format!("Basic {}", base64_of(&format!("lab:{LAB_PASS}")));
    let node = |id: &str, x: u32, kind: Value| { let mut kind = kind; kind["id"] = json!(id); kind["x"] = json!(x); kind["y"] = json!(0); kind };
    let doc = json!({ "version": 9, "name": "Echo", "params": [], "nodes": [
        node("start", 0, json!({ "type": "start" })),
        node("echo", 200, json!({ "type": "http", "request": { "method": "GET", "url": format!("{base}/echo"), "headers": [], "timeout_ms": 3000,
            "auth": { "scheme": "basic", "username": "lab", "password": "{{secret.LAB_PASS}}" } } })),
        node("said", 400, json!({ "type": "extract", "variable": "said", "from": "body", "expr": "" })),
        node("end", 600, json!({ "type": "end" })),
    ], "edges": [{ "from": "start", "to": "echo" }, { "from": "echo", "to": "said" }, { "from": "said", "to": "end" }] });

    // Send now: the server echoed the header; what comes back masks it.
    let now = service.invoke("experiment_send_node", json!({ "document": doc, "nodeId": "echo", "vars": {} })).await.unwrap();
    assert_eq!(now["response"]["body"].as_str(), Some(format!("Basic {MASK}").as_str()), "{now}");
    assert!(!now.to_string().contains(&sent[6..]), "nor in the echoed header: {now}");

    // A run: the step that read the body reports it masked.
    let experiment: Experiment = serde_json::from_value(doc).unwrap();
    let result = service.run(experiment, RunOptions { limit: Some(Duration::from_secs(20)), ..Default::default() }).await.unwrap().finished().await;
    assert_eq!(result.outcome, Outcome::Passed, "{:?}", result.error);
    let steps = serde_json::to_string(&result.steps).unwrap();
    assert!(steps.contains("\"said\"") && !steps.contains(&sent[6..]), "{steps}");
    assert!(!recorder.events().iter().any(|(_, payload)| payload.to_string().contains(&sent[6..])), "no event carries it either");
}

fn base64_of(text: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(text)
}
