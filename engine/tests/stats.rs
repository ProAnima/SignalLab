//! The usage statistics end to end against a stand-in for the studio's hub on
//! loopback: the sender waits for the interface's setting, posts `launch` until
//! the hub takes one (a 503 and a 429 do not lose it), then `heartbeat` each
//! beat; turned off it sends nothing, turned on again it carries on. The beat is
//! shortened; the requests are read back as the hub reads them.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::Value;
use signal_lab_engine::stats::{self, Stats};
use tokio::net::TcpListener;

const ID: &str = "0f8b3c2e-7d41-4a8e-9c3b-5e2d1a6f7b90";
const BEAT: Duration = Duration::from_millis(40);

/// One request as the hub received it.
struct Received {
    method: String,
    path: String,
    content_type: String,
    body: Value,
}

type Kept = Arc<Mutex<Vec<Received>>>;

/// A stand-in for the hub: keeps every request, answers the next status in turn (202 after).
async fn hub_stand_in(statuses: Vec<StatusCode>) -> (String, Kept) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let received: Kept = Arc::default();
    let statuses = Arc::new(Mutex::new(VecDeque::from(statuses)));
    let kept = received.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let (kept, statuses) = (kept.clone(), statuses.clone());
            tokio::spawn(async move {
                let service = service_fn(move |request: Request<hyper::body::Incoming>| {
                    let (kept, statuses) = (kept.clone(), statuses.clone());
                    async move {
                        let (method, path) = (request.method().to_string(), request.uri().path().to_string());
                        let content_type = request.headers().get("content-type").and_then(|value| value.to_str().ok()).unwrap_or_default().to_string();
                        let body = request.into_body().collect().await.unwrap().to_bytes();
                        kept.lock().unwrap().push(Received { method, path, content_type, body: serde_json::from_slice(&body).unwrap_or(Value::Null) });
                        let status = statuses.lock().unwrap().pop_front().unwrap_or(StatusCode::ACCEPTED);
                        Ok::<_, std::convert::Infallible>(Response::builder().status(status).body(Full::new(Bytes::from_static(b"{}"))).unwrap())
                    }
                });
                let _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service).await;
            });
        }
    });
    (url, received)
}

/// The kinds received so far, once there are at least `n` (or what came in two seconds).
async fn kinds(received: &Kept, n: usize) -> Vec<String> {
    for _ in 0..200 {
        if received.lock().unwrap().len() >= n {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    received.lock().unwrap().iter().map(|request| request.body["events"][0]["kind"].as_str().unwrap_or_default().to_string()).collect()
}

#[tokio::test]
async fn a_launch_until_the_hub_takes_it_then_heartbeats() {
    let (hub, received) = hub_stand_in(vec![StatusCode::SERVICE_UNAVAILABLE, StatusCode::TOO_MANY_REQUESTS, StatusCode::ACCEPTED, StatusCode::TOO_MANY_REQUESTS]).await;
    let stats = Arc::new(Stats::default());
    let sender = tokio::spawn(stats::run(stats.clone(), format!("{hub}/v1/signal-lab/events"), "1.2.3".into(), BEAT));

    tokio::time::sleep(BEAT * 3).await;
    assert!(received.lock().unwrap().is_empty(), "nothing before the interface has said whether it may");

    stats.configure(Some(ID), true);
    let sent = kinds(&received, 6).await;
    assert_eq!(sent[..6], ["launch", "launch", "launch", "heartbeat", "heartbeat", "heartbeat"], "a 503 and a 429 keep the launch; a 429 on a heartbeat waits for the next");
    {
        let received = received.lock().unwrap();
        let first = &received[0];
        assert_eq!(first.method, "POST");
        assert_eq!(first.path, "/v1/signal-lab/events");
        assert!(first.content_type.starts_with("application/json"), "{}", first.content_type);
        assert_eq!(first.body["install_id"], ID);
        assert_eq!(first.body["version"], "1.2.3");
        assert_eq!(first.body["os"], stats::os());
        assert_eq!(first.body["arch"], stats::arch());
        assert_eq!(first.body["channel"], "stable");
        assert_eq!(first.body.as_object().unwrap().len(), 6, "nothing else: {}", first.body);
    }

    // Off: the next tick sends nothing. On again: heartbeats, never a second launch.
    stats.configure(Some(ID), false);
    tokio::time::sleep(BEAT * 2).await;
    let before = received.lock().unwrap().len();
    tokio::time::sleep(BEAT * 4).await;
    assert_eq!(received.lock().unwrap().len(), before, "turned off, nothing goes");
    stats.configure(Some(ID), true);
    let sent = kinds(&received, before + 2).await;
    assert!(sent[before..].iter().all(|kind| kind == "heartbeat"), "{sent:?}");
    sender.abort();
}

#[tokio::test]
async fn without_an_install_id_nothing_is_sent() {
    let (hub, received) = hub_stand_in(Vec::new()).await;
    let stats = Arc::new(Stats::default());
    stats.configure(None, true);
    let sender = tokio::spawn(stats::run(stats.clone(), format!("{hub}/v1/signal-lab/events"), "1.2.3".into(), BEAT));
    tokio::time::sleep(BEAT * 5).await;
    assert!(received.lock().unwrap().is_empty());
    sender.abort();
}

#[tokio::test]
async fn a_hub_that_cannot_be_reached_is_tried_again_in_silence() {
    // A port nothing listens on: the launch fails, and the sender keeps its turn.
    let free = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/v1/signal-lab/events", free.local_addr().unwrap());
    drop(free);
    let stats = Arc::new(Stats::default());
    stats.configure(Some(ID), true);
    let sender = tokio::spawn(stats::run(stats, url, "1.2.3".into(), BEAT));
    tokio::time::sleep(BEAT * 4).await;
    assert!(!sender.is_finished(), "a failure never ends the sender");
    sender.abort();
}
