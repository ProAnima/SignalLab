//! The feedback form end to end against a stand-in for the studio's hub on
//! loopback: the `feedback_send` command finds the hub through
//! SIGNALLAB_HUB_URL, posts to Signal Lab's feedback endpoint, and the request
//! is read back as the hub reads it (multipart: the message, the address, what
//! the app says about itself, screenshots, logs). The hub's answers — an id,
//! or its refusals in its own shape — come back in the app's words. How the
//! hub mails the form is the hub's own test. One test, since the command finds
//! the hub through the process's environment.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{feedback, hub};
use signal_lab_engine::{Capture, Host, Mode, Service};
use tokio::net::TcpListener;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0";

/// One request as the hub received it.
struct Received {
    method: String,
    path: String,
    content_type: String,
    user_agent: String,
    authorization: bool,
    body: Vec<u8>,
}

/// The hub's answers, in turn: a status and its JSON.
type Answers = Arc<Mutex<VecDeque<(StatusCode, Value)>>>;

/// A stand-in for the hub on loopback: keeps every request, gives the next answer.
async fn hub_stand_in(answers: Answers) -> (String, Arc<Mutex<Vec<Received>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let received = Arc::new(Mutex::new(Vec::new()));
    let kept = received.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let (kept, answers) = (kept.clone(), answers.clone());
            tokio::spawn(async move {
                let service = service_fn(move |request: Request<hyper::body::Incoming>| {
                    let (kept, answers) = (kept.clone(), answers.clone());
                    async move {
                        let header = |name: &str| request.headers().get(name).and_then(|value| value.to_str().ok()).unwrap_or_default().to_string();
                        let (method, path) = (request.method().to_string(), request.uri().path().to_string());
                        let (content_type, user_agent, authorization) = (header("content-type"), header("user-agent"), request.headers().contains_key("authorization"));
                        let body = request.into_body().collect().await.map(|body| body.to_bytes().to_vec()).unwrap_or_default();
                        kept.lock().unwrap().push(Received { method, path, content_type, user_agent, authorization, body });
                        let (status, answer) = answers.lock().unwrap().pop_front().unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, json!({})));
                        let response = Response::builder().status(status).header("content-type", "application/json").body(Full::new(Bytes::from(answer.to_string()))).unwrap();
                        Ok::<_, std::convert::Infallible>(response)
                    }
                });
                let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
            });
        }
    });
    (url, received)
}

/// One part of a multipart body: its field, file name, type and bytes.
#[derive(Debug)]
struct Part {
    name: String,
    file: Option<String>,
    content_type: Option<String>,
    data: Vec<u8>,
}

fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack.get(from..)?.windows(needle.len()).position(|window| window == needle).map(|at| at + from)
}

/// multipart/form-data, read as the hub's form reader does.
fn parts(content_type: &str, body: &[u8]) -> Vec<Part> {
    let boundary = content_type.split("boundary=").nth(1).expect("a boundary").trim_matches('"');
    let delimiter = format!("--{boundary}").into_bytes();
    let mut parts = Vec::new();
    let mut at = find(body, &delimiter, 0).expect("the first boundary") + delimiter.len();
    while !body[at..].starts_with(b"--") {
        let start = at + 2;
        let head_end = find(body, b"\r\n\r\n", start).expect("a part's headers");
        let next = find(body, &[b"\r\n".as_slice(), &delimiter].concat(), head_end).expect("the next boundary");
        let head = String::from_utf8_lossy(&body[start..head_end]).to_string();
        let quoted = |key: &str| head.split(&format!("{key}=\"")).nth(1).and_then(|rest| rest.split('"').next()).map(str::to_string);
        let content_type = head.lines().find_map(|line| line.to_ascii_lowercase().starts_with("content-type:").then(|| line[13..].trim().to_string()));
        parts.push(Part { name: quoted("name").unwrap_or_default(), file: quoted("filename"), content_type, data: body[head_end + 4..next].to_vec() });
        at = next + 2 + delimiter.len();
    }
    parts
}

fn engine() -> Service {
    Service::new(Host::new(Recorder::new(), Capture::new()), Mode::Desktop, Arc::new(FileStore::new(None).with_env(|_| None)))
}

fn refusal(status: StatusCode, code: &str, params: Value) -> (StatusCode, Value) {
    (status, json!({ "error": { "code": code, "params": params } }))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_form_reaches_the_hub_as_it_reads_it_and_its_refusals_come_back_in_words() {
    let answers: Answers = Arc::new(Mutex::new(VecDeque::from([
        (StatusCode::ACCEPTED, json!({ "id": "0123456789abcdef" })),
        refusal(StatusCode::BAD_REQUEST, "feedback.file_type", json!({ "name": "x.png" })),
        refusal(StatusCode::FORBIDDEN, "feedback.disabled", json!({})),
        refusal(StatusCode::TOO_MANY_REQUESTS, "feedback.rate_limited", json!({ "retry_after_s": "300" })),
        refusal(StatusCode::NOT_FOUND, "project.unknown", json!({ "project": "signal-lab" })),
    ])));
    let (hub_url, received) = hub_stand_in(answers).await;
    // The only test of this binary, so the process's variable is its own; a trailing slash is no matter.
    std::env::set_var("SIGNALLAB_HUB_URL", format!("{hub_url}/"));
    assert_eq!(hub::url(), hub_url);
    assert_eq!(feedback::url(), format!("{hub_url}/v1/signal-lab/feedback"));
    let service = engine();
    let info = service.invoke("app_info", json!({})).await.unwrap();
    assert_eq!((info["os"].as_str(), info["arch"].as_str()), (Some(std::env::consts::OS), Some(std::env::consts::ARCH)));

    let form = json!({
        "message": "  The scope freezes\nwhen the window is hidden.\n",
        "email": "person@example.com",
        "meta": { "version": "0.4.0", "os": info["os"], "mode": "desktop", "lang": "en" },
        "screenshots": [{ "name": "Screenshot 2026-10-03.png", "data": base64::engine::general_purpose::STANDARD.encode(PNG) }],
        "logs": [{ "name": "signallab-console.txt", "text": "12:00:01 ok ready\n" }, { "name": "signallab-system.json", "text": "{\"mode\":\"desktop\"}" }],
    });
    let sent = service.invoke("feedback_send", json!({ "form": form })).await.unwrap();
    assert_eq!(sent["id"], "0123456789abcdef", "the hub's reference, for the person to quote");
    let request = received.lock().unwrap().pop().unwrap();
    assert_eq!((request.method.as_str(), request.path.as_str()), ("POST", "/v1/signal-lab/feedback"));
    assert!(request.user_agent.starts_with("SignalLab/"), "{}", request.user_agent);
    assert!(!request.authorization, "the app has no secret to give");
    let parts = parts(&request.content_type, &request.body);
    let field = |name: &str| parts.iter().filter(|part| part.name == name).collect::<Vec<_>>();
    assert_eq!(field("message")[0].data, b"The scope freezes\nwhen the window is hidden.", "trimmed, line breaks kept");
    assert_eq!(field("email")[0].data, b"person@example.com");
    let meta: Value = serde_json::from_slice(&field("meta")[0].data).unwrap();
    assert_eq!((meta["version"].as_str(), meta["mode"].as_str()), (Some("0.4.0"), Some("desktop")), "the subject is made of the version");
    let shot = field("screenshot");
    assert_eq!((shot.len(), shot[0].file.as_deref()), (1, Some("Screenshot 2026-10-03.png")));
    assert_eq!(shot[0].data, PNG, "the screenshot byte for byte; its type is the hub's to tell from them");
    let logs = field("log");
    assert_eq!(logs.iter().map(|part| part.file.as_deref().unwrap_or_default()).collect::<Vec<_>>(), ["signallab-console.txt", "signallab-system.json"]);
    assert_eq!(logs[0].content_type.as_deref(), Some("text/plain; charset=utf-8"));
    assert_eq!(logs[0].data, b"12:00:01 ok ready\n");
    let names: Vec<&str> = parts.iter().map(|part| part.name.as_str()).collect();
    assert_eq!(names, ["message", "email", "meta", "screenshot", "log", "log"], "nothing else: where it goes is the hub's to say");

    // The hub's refusal, in its words: a screenshot that is not an image.
    let fake = json!({ "message": "hi", "screenshots": [{ "name": "x.png", "data": base64::engine::general_purpose::STANDARD.encode(b"<svg/>") }] });
    let refused = serde_json::to_value(service.invoke("feedback_send", json!({ "form": fake })).await.unwrap_err()).unwrap();
    assert_eq!((refused["code"].as_str(), refused["params"]["name"].as_str()), (Some("feedback.file_type"), Some("x.png")), "{refused}");
    // Checked here first: nothing is uploaded that the hub would refuse for what it is.
    let before = received.lock().unwrap().len();
    let empty = serde_json::to_value(service.invoke("feedback_send", json!({ "form": { "message": " " } })).await.unwrap_err()).unwrap();
    assert_eq!(empty["code"], "feedback.message_required");
    let unknown = serde_json::to_value(service.invoke("feedback_send", json!({ "form": { "message": "hi", "to": "x@y.test" } })).await.unwrap_err()).unwrap();
    assert_eq!(unknown["code"], "command.args_invalid", "the recipient is not the app's to give");
    assert_eq!(received.lock().unwrap().len(), before, "neither reached the hub");

    let once = || serde_json::from_value::<feedback::Form>(json!({ "message": "hi" })).unwrap();
    // Switched off on the hub.
    assert!(feedback::send(once()).await.unwrap_err().is("feedback.disabled"));
    // Over the limit: the wait, as the hub says it.
    let limited = feedback::send(once()).await.unwrap_err();
    assert!(limited.is("feedback.rate_limited") && limited.params["retry_after_s"] == "300", "{limited:?}");
    // A code of the hub's own is not the app's to show: the status, the code as detail.
    let unknown_project = feedback::send(once()).await.unwrap_err();
    assert!(unknown_project.is("feedback.failed") && unknown_project.params["status"] == "404", "{unknown_project:?}");
    assert_eq!(unknown_project.detail.as_deref(), Some("project.unknown"));

    // The hub cannot be reached: a network cause about it.
    let closed = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let nowhere = format!("http://{}/v1/signal-lab/feedback", closed.local_addr().unwrap());
    drop(closed);
    let unreachable = feedback::send_to(&nowhere, once()).await.unwrap_err();
    assert!(unreachable.is("transport.refused") && unreachable.params["target"] == nowhere, "{unreachable:?}");
}
