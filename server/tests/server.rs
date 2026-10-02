//! The server end to end: started on a loopback port, used like a browser and
//! like a script would use it.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{json, Value};
use signal_lab_server::{serve, Config};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

const TOKEN: &str = "test-token-0123456789abcdef0123456789";

/// One data folder for this test binary (the engine chooses it once per process).
fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-server-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    })
    .clone()
}

/// A tiny interface folder, so static serving can be checked.
fn ui_dir() -> PathBuf {
    let dir = data_dir().join("ui");
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("index.html"), "<!doctype html><title>Signal Lab</title>").unwrap();
    std::fs::write(dir.join("assets").join("app-1234.js"), "console.log(1)").unwrap();
    dir
}

struct Running {
    url: String,
    address: SocketAddr,
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
    let address = listener.local_addr().unwrap();
    let config = Config {
        listen: address,
        token: token.map(str::to_string),
        data_dir: Some(data_dir()),
        secrets_dir: data_dir().join("secrets"),
        ui_dir: Some(ui_dir()),
        allowed_hosts: vec![],
        secure_cookie: false,
    };
    let (stop, stopped) = oneshot::channel::<()>();
    let done = tokio::spawn(serve(config, listener, async move {
        let _ = stopped.await;
    }));
    Running { url: format!("http://{address}"), address, stop: Some(stop), done }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap()
}

async fn invoke(running: &Running, command: &str, args: Value, bearer: Option<&str>) -> reqwest::Response {
    let mut request = client().post(format!("{}/api/invoke/{command}", running.url)).json(&args);
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }
    request.send().await.unwrap()
}

#[tokio::test]
async fn on_loopback_without_a_token_commands_work_but_only_under_loopback_names() {
    let running = start(None).await;
    let health: Value = client().get(format!("{}/api/health", running.url)).send().await.unwrap().json().await.unwrap();
    assert_eq!(health["status"], "ok");

    let response = invoke(&running, "app_info", Value::Null, None).await;
    assert_eq!(response.status(), 200);
    let info: Value = response.json().await.unwrap();
    assert_eq!((info["mode"].as_str(), info["secrets_writable"].as_bool()), (Some("server"), Some(false)));

    // A page served under another name (DNS rebinding) is refused.
    let foreign = client().post(format!("{}/api/invoke/app_info", running.url)).header("host", "attacker.example").json(&json!({})).send().await.unwrap();
    assert_eq!(foreign.status(), 403);
    assert_eq!(foreign.json::<Value>().await.unwrap()["code"], "auth.host");
    // A command from a page of another origin is refused.
    let cross = client().post(format!("{}/api/invoke/app_info", running.url)).header("origin", "http://attacker.example").json(&json!({})).send().await.unwrap();
    assert_eq!(cross.json::<Value>().await.unwrap()["code"], "auth.origin");
    // Only JSON: a plain form post from elsewhere never runs a command.
    let form = client().post(format!("{}/api/invoke/app_info", running.url)).header("content-type", "text/plain").body("{}").send().await.unwrap();
    assert_eq!(form.status(), 415);
    running.stop().await;
}

#[tokio::test]
async fn with_a_token_everything_but_health_and_sign_in_needs_it() {
    let running = start(Some(TOKEN)).await;
    let denied = invoke(&running, "app_info", Value::Null, None).await;
    assert_eq!(denied.status(), 401);
    assert_eq!(denied.json::<Value>().await.unwrap()["code"], "auth.required");
    assert_eq!(invoke(&running, "app_info", Value::Null, Some("wrong-token")).await.status(), 401);
    assert_eq!(invoke(&running, "app_info", Value::Null, Some(TOKEN)).await.status(), 200);

    // A browser is sent to the sign-in page, which is in its language.
    let page = client().get(format!("{}/", running.url)).send().await.unwrap();
    assert_eq!((page.status().as_u16(), page.headers()["location"].to_str().unwrap()), (303, "/login"));
    let russian = client().get(format!("{}/login", running.url)).header("accept-language", "ru-RU,ru;q=0.9").send().await.unwrap().text().await.unwrap();
    assert!(russian.contains("Токен доступа"));
    // Help is a tooltip on the field's label, in the page's language, and read to screen readers.
    assert!(russian.contains(r#"data-tip="Токен задаётся там, где запущен сервер"#), "{russian}");
    assert!(russian.contains(r#"aria-describedby="token-help""#));
    let english = client().get(format!("{}/login", running.url)).header("accept-language", "en-GB,en;q=0.8").send().await.unwrap().text().await.unwrap();
    assert!(english.contains(r#"data-tip="The token is set where the server runs"#), "{english}");
    assert!(!english.contains("{{"), "every placeholder is filled in");
    // The most preferred language the page has, by weight — not just the first tag.
    for (accept, lang) in [("de-DE,ru;q=0.9,en;q=0.8", "ru"), ("en;q=0.5, ru", "ru"), ("ru;q=0, en", "en"), ("fr, ja", "en"), ("", "en")] {
        let page = client().get(format!("{}/login", running.url)).header("accept-language", accept).send().await.unwrap().text().await.unwrap();
        assert!(page.contains(&format!(r#"<html lang="{lang}">"#)), "{accept}: {lang}");
    }

    // Signing in sets an HttpOnly, SameSite=Strict session cookie; signing out ends it.
    let wrong = client().post(format!("{}/login", running.url)).form(&[("token", "nope")]).send().await.unwrap();
    assert_eq!(wrong.status(), 401);
    assert!(wrong.headers().get("set-cookie").is_none());
    let signed_in = client().post(format!("{}/login", running.url)).form(&[("token", TOKEN)]).send().await.unwrap();
    assert_eq!(signed_in.status(), 303);
    let cookie = signed_in.headers()["set-cookie"].to_str().unwrap().to_string();
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict"), "{cookie}");
    let session = cookie.split(';').next().unwrap().to_string();
    let with_cookie = client().post(format!("{}/api/invoke/jobs_list", running.url)).header("cookie", &session).json(&json!({})).send().await.unwrap();
    assert_eq!(with_cookie.json::<Value>().await.unwrap(), json!([]));
    client().post(format!("{}/logout", running.url)).header("cookie", &session).send().await.unwrap();
    let after = client().post(format!("{}/api/invoke/jobs_list", running.url)).header("cookie", &session).json(&json!({})).send().await.unwrap();
    assert_eq!(after.status(), 401);
    running.stop().await;
}

#[tokio::test]
async fn failures_keep_their_shape_and_unknown_routes_say_so() {
    let running = start(None).await;
    let unknown = invoke(&running, "no_such_command", json!({}), None).await;
    assert_eq!(unknown.status(), 422);
    assert_eq!(unknown.json::<Value>().await.unwrap()["code"], "command.unknown");
    let invalid = invoke(&running, "experiment_parse", json!({ "text": "{" }), None).await.json::<Value>().await.unwrap();
    assert_eq!(invalid["code"], "file.json_invalid");
    let tool = invoke(&running, "mqtt_subscribe", json!({ "jobId": 1, "filters": [] }), None).await.json::<Value>().await.unwrap();
    assert_eq!(tool["code"], "mqtt.filter_required", "the tool screens' commands fail with codes too");
    let read_only = invoke(&running, "secret_set", json!({ "name": "API_TOKEN", "value": "x" }), None).await.json::<Value>().await.unwrap();
    assert_eq!(read_only["code"], "secret.read_only");
    let missing = client().get(format!("{}/api/nothing", running.url)).send().await.unwrap();
    assert_eq!((missing.status().as_u16(), missing.json::<Value>().await.unwrap()["code"].clone()), (404, json!("api.not_found")));
    running.stop().await;
}

#[tokio::test]
async fn the_interface_is_served_with_security_headers_and_files_only_from_the_data_folder() {
    let running = start(None).await;
    let index = client().get(format!("{}/some/screen", running.url)).send().await.unwrap();
    assert_eq!(index.status(), 200, "unknown paths fall back to the interface");
    let headers = index.headers().clone();
    assert!(headers["content-security-policy"].to_str().unwrap().contains("default-src 'self'"));
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert_eq!(headers["x-frame-options"], "DENY");
    // "no-referrer" would make browsers send `Origin: null` with the sign-in form.
    assert_eq!(headers["referrer-policy"], "same-origin");
    let asset = client().get(format!("{}/assets/app-1234.js", running.url)).send().await.unwrap();
    assert!(asset.headers()["cache-control"].to_str().unwrap().contains("immutable"));

    // An export is written to the data folder and can be downloaded from there.
    let document = serde_json::to_value(signal_lab_engine::experiment::starter()).unwrap();
    let path: String = invoke(&running, "experiment_export", json!({ "document": document }), None).await.json().await.unwrap();
    let download = client().get(format!("{}/api/files", running.url)).query(&[("path", &path)]).send().await.unwrap();
    assert_eq!(download.status(), 200);
    assert!(download.headers()["content-disposition"].to_str().unwrap().starts_with("attachment"));
    let parsed: Value = serde_json::from_slice(&download.bytes().await.unwrap()).unwrap();
    assert_eq!(parsed["name"], document["name"]);
    // Nothing outside it.
    let outside = std::env::temp_dir().join(format!("signallab-outside-{}.txt", std::process::id()));
    std::fs::write(&outside, "private").unwrap();
    let refused = client().get(format!("{}/api/files", running.url)).query(&[("path", outside.to_str().unwrap())]).send().await.unwrap();
    assert_eq!(refused.status(), 404);
    std::fs::remove_file(outside).unwrap();
    running.stop().await;
}

#[tokio::test]
async fn events_reach_a_signed_in_page_over_the_websocket() {
    let running = start(Some(TOKEN)).await;
    let url = format!("ws://{}/api/events", running.address);
    // Without the token the upgrade is refused.
    assert!(tokio_tungstenite::connect_async(url.as_str()).await.is_err());

    let mut request = url.as_str().into_client_request().unwrap();
    request.headers_mut().insert("authorization", format!("Bearer {TOKEN}").parse().unwrap());
    let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();

    // Arm the Inspector and send one OSC message: the capture batch arrives as an event.
    invoke(&running, "inspect_set_enabled", json!({ "enabled": true }), Some(TOKEN)).await;
    invoke(&running, "osc_send", json!({ "target": "127.0.0.1:9", "address": "/hello", "args": [] }), Some(TOKEN)).await;
    let batch = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(message) = socket.next().await {
            let Ok(text) = message.unwrap().into_text() else { continue };
            let event: Value = serde_json::from_str(&text).unwrap();
            if event["event"] == "inspect://batch" && !event["payload"]["frames"].as_array().unwrap().is_empty() {
                return event;
            }
        }
        panic!("the socket closed");
    })
    .await
    .expect("a capture batch arrived");
    assert_eq!(batch["payload"]["frames"][0]["summary"], "/hello");
    drop(socket);
    running.stop().await;
}
