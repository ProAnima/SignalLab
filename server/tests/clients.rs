//! Several pages on one server at once: what one page does, the others see;
//! a page that stops reading is let go rather than held forever.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde_json::{json, Value};
use signal_lab_server::{serve, Config};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-server-clients-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    })
    .clone()
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

async fn start_on(listener: TcpListener) -> Running {
    let address = listener.local_addr().unwrap();
    let config = Config {
        listen: address,
        token: None,
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
    Running { url: format!("http://{address}"), address, stop: Some(stop), done }
}

async fn start() -> Running {
    start_on(TcpListener::bind("127.0.0.1:0").await.unwrap()).await
}

async fn invoke(running: &Running, command: &str, args: Value) -> Value {
    let response = reqwest::Client::new().post(format!("{}/api/invoke/{command}", running.url)).json(&args).send().await.unwrap();
    let status = response.status();
    let body: Value = response.json().await.unwrap();
    assert!(status.is_success(), "{command}: {status} {body}");
    body
}

type Page = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn page(running: &Running) -> Page {
    let request = format!("ws://{}/api/events", running.address).into_client_request().unwrap();
    tokio_tungstenite::connect_async(request).await.unwrap().0
}

/// The next event on `page` that `test` accepts.
async fn event(page: &mut Page, within: Duration, test: impl Fn(&Value) -> bool) -> Value {
    tokio::time::timeout(within, async {
        while let Some(message) = page.next().await {
            let Ok(text) = message.unwrap().into_text() else { continue };
            let event: Value = serde_json::from_str(&text).unwrap();
            if test(&event) {
                return event;
            }
        }
        panic!("the page's socket closed");
    })
    .await
    .expect("the event arrived")
}

fn free_udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

/// One page starts a monitor: both pages see its traffic; the other page stops
/// it: it is gone for both, and its port is free at once.
#[tokio::test(flavor = "multi_thread")]
async fn what_one_page_starts_and_stops_every_page_sees() {
    let running = start().await;
    let (mut first, mut second) = (page(&running).await, page(&running).await);
    let port = free_udp_port();
    let job = invoke(&running, "osc_monitor_start", json!({ "bind": format!("127.0.0.1:{port}") })).await;
    let id = job["id"].as_u64().unwrap();
    let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    sender.send_to(&signal_lab_engine::osc_codec::encode_message("/both", &[]), ("127.0.0.1", port)).unwrap();
    for page in [&mut first, &mut second] {
        let message = event(page, Duration::from_secs(5), |event| event["event"] == "osc://message").await;
        assert_eq!(message["payload"]["messages"][0]["address"], "/both");
    }
    assert_eq!(invoke(&running, "job_stop", json!({ "id": id })).await, json!(true));
    assert_eq!(invoke(&running, "jobs_list", json!({})).await, json!([]));
    // Stopped means let go: the same port binds again straight away.
    std::net::UdpSocket::bind(("127.0.0.1", port)).expect("the stopped monitor's port is free");
    drop((first, second));
    running.stop().await;
}

/// A page that stops reading (a frozen tab, a script that never reads) holds
/// its socket and a task for good unless it is let go: the server closes it
/// once a send has waited too long, and the pages that read go on receiving.
#[tokio::test(flavor = "multi_thread")]
async fn a_page_that_never_reads_is_let_go() {
    let running = start().await;
    let stuck = page(&running).await;
    let mut reading = page(&running).await;
    let port = free_udp_port();
    invoke(&running, "osc_monitor_start", json!({ "bind": format!("127.0.0.1:{port}") })).await;
    // Large messages, so the stuck page's buffers fill in moments.
    let big = signal_lab_engine::osc_codec::encode_message("/big", &[signal_lab_engine::osc_codec::OscArg::Str("x".repeat(30_000))]);
    let flooding = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let flood = {
        let flooding = flooding.clone();
        std::thread::spawn(move || {
            let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
            while flooding.load(std::sync::atomic::Ordering::Relaxed) {
                let _ = sender.send_to(&big, ("127.0.0.1", port));
                std::thread::sleep(Duration::from_millis(2));
            }
        })
    };
    // The page that reads keeps up throughout.
    let until = Instant::now() + Duration::from_secs(14);
    while Instant::now() < until {
        event(&mut reading, Duration::from_secs(5), |event| event["event"] == "osc://message").await;
    }
    flooding.store(false, std::sync::atomic::Ordering::Relaxed);
    flood.join().unwrap();
    // The stuck page now reads: what was buffered, then the end — not an endless stream.
    let mut stuck = stuck;
    let ended = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match stuck.next().await {
                None | Some(Err(_)) => return,
                Some(Ok(message)) if message.is_close() => return,
                Some(Ok(_)) => {}
            }
        }
    })
    .await;
    assert!(ended.is_ok(), "the page that never read is still being sent to");
    drop(reading);
    running.stop().await;
}

/// A server stopped and started again on the same address: a page reconnects
/// and is served, and the jobs of the old one are gone with it.
#[tokio::test(flavor = "multi_thread")]
async fn a_restarted_server_serves_a_reconnecting_page() {
    let running = start().await;
    let address = running.address;
    let port = free_udp_port();
    invoke(&running, "osc_monitor_start", json!({ "bind": format!("127.0.0.1:{port}") })).await;
    let mut old = page(&running).await;
    running.stop().await;
    // The page hears the end rather than waiting on a dead socket.
    let closed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match old.next().await {
                None | Some(Err(_)) => return,
                Some(Ok(message)) if message.is_close() => return,
                Some(Ok(_)) => {}
            }
        }
    })
    .await;
    assert!(closed.is_ok(), "the page was told the server went");
    let running = start_on(TcpListener::bind(address).await.expect("the address is free again")).await;
    // The old server's monitor let go of its port.
    let job = invoke(&running, "osc_monitor_start", json!({ "bind": format!("127.0.0.1:{port}") })).await;
    assert_eq!(invoke(&running, "jobs_list", json!({})).await.as_array().unwrap().len(), 1);
    let mut again = page(&running).await;
    let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    sender.send_to(&signal_lab_engine::osc_codec::encode_message("/again", &[]), ("127.0.0.1", port)).unwrap();
    let message = event(&mut again, Duration::from_secs(5), |event| event["event"] == "osc://message").await;
    assert_eq!(message["payload"]["job_id"], job["id"]);
    drop(again);
    running.stop().await;
}
