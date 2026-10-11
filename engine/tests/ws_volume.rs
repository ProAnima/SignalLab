//! The WebSocket screen's connection against servers that flood it: tens of
//! thousands of small messages a second, or 16 MiB messages back to back. A
//! close and a stop still end it promptly, and what the screen is sent stays
//! within its bounds — counted, never silently cut.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

fn service() -> (Service, Arc<Recorder>) {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-ws-volume-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        dir
    });
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))), recorder)
}

/// What the flooding server saw: messages written, and whether the client's
/// close frame (or the end of the line) reached it.
#[derive(Default)]
struct Flood {
    written: AtomicU64,
    ended: AtomicBool,
}

/// A server that sends `message` again and again, as fast as the line takes
/// it, until the client closes or the line ends. It never answers a close
/// with one of its own while it floods — as a busy service would not.
async fn flooding(message: Message) -> (String, Arc<Flood>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/", listener.local_addr().unwrap());
    let flood = Arc::new(Flood::default());
    let seen = flood.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let seen = seen.clone();
            let message = message.clone();
            tokio::spawn(async move {
                let Ok(socket) = tokio_tungstenite::accept_async(stream).await else { return };
                let (mut write, mut read) = socket.split();
                let reader = {
                    let seen = seen.clone();
                    tokio::spawn(async move {
                        while let Some(Ok(message)) = read.next().await {
                            if message.is_close() {
                                break;
                            }
                        }
                        seen.ended.store(true, Ordering::SeqCst);
                    })
                };
                while !seen.ended.load(Ordering::SeqCst) {
                    if write.send(message.clone()).await.is_err() {
                        break;
                    }
                    seen.written.fetch_add(1, Ordering::Relaxed);
                }
                seen.ended.store(true, Ordering::SeqCst);
                reader.abort();
            });
        }
    });
    (url, flood)
}

/// A server faster than any client: after the upgrade it writes ready-made
/// text frames straight onto the socket, a megabyte at a time, and never reads
/// again — the client's buffer is never empty.
async fn firehose() -> (String, Arc<Flood>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/", listener.local_addr().unwrap());
    let flood = Arc::new(Flood::default());
    let seen = flood.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let seen = seen.clone();
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut chunk = [0u8; 2048];
                while !head.windows(4).any(|window| window == b"\r\n\r\n") {
                    match stream.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(read) => head.extend_from_slice(&chunk[..read]),
                    }
                }
                let text = String::from_utf8_lossy(&head);
                let key = text.lines().find_map(|line| line.strip_prefix("Sec-WebSocket-Key: ").or_else(|| line.strip_prefix("sec-websocket-key: "))).unwrap().trim().to_string();
                let accept = tokio_tungstenite::tungstenite::handshake::derive_accept_key(key.as_bytes());
                let answer = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n");
                if stream.write_all(answer.as_bytes()).await.is_err() {
                    return;
                }
                // "tick", unmasked, as a server sends it: FIN + text, length 4.
                let frames: Vec<u8> = std::iter::repeat_n([0x81u8, 4, b't', b'i', b'c', b'k'], 1 << 17).flatten().collect();
                while stream.write_all(&frames).await.is_ok() {
                    seen.written.fetch_add(1 << 17, Ordering::Relaxed);
                }
                seen.ended.store(true, Ordering::SeqCst);
            });
        }
    });
    (url, flood)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_firehose_never_starves_a_close_or_a_stop() {
    let (service, recorder) = service();
    let (url, flood) = firehose().await;
    let job = service.invoke("ws_connect", json!({ "config": { "url": url } })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    let began = Instant::now();
    let closed = tokio::time::timeout(Duration::from_secs(10), service.invoke("ws_close", json!({ "jobId": id, "code": 1000 }))).await;
    println!("closed in {:?} after {} written: {closed:?}", began.elapsed(), flood.written.load(Ordering::Relaxed));
    assert!(closed.is_ok(), "the close came back");
    assert!(began.elapsed() < Duration::from_secs(4), "a close frame, then at most the grace: {:?}", began.elapsed());
    let began = Instant::now();
    while !flood.ended.load(Ordering::SeqCst) && began.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(flood.ended.load(Ordering::SeqCst), "the line is down: nothing reads the flood any more");
    let received = batches(&recorder).last().and_then(|batch| batch["received"].as_u64()).unwrap_or(0);
    println!("{received} received");

    // And a stop, with nobody asking for a close.
    let (url, flood) = firehose().await;
    let job = service.invoke("ws_connect", json!({ "config": { "url": url } })).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(service.jobs().stop(job["id"].as_u64().unwrap()));
    let began = Instant::now();
    while !flood.ended.load(Ordering::SeqCst) && began.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    println!("stopped: the line went down in {:?}", began.elapsed());
    assert!(flood.ended.load(Ordering::SeqCst) && began.elapsed() < Duration::from_secs(4), "{:?}", began.elapsed());
}

fn batches(recorder: &Recorder) -> Vec<Value> {
    recorder.payloads("ws://messages")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_flooded_connection_still_closes_when_asked() {
    let (service, recorder) = service();
    let (url, flood) = flooding(Message::text("tick")).await;
    let job = service.invoke("ws_connect", json!({ "config": { "url": url } })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(flood.written.load(Ordering::Relaxed) > 1000, "it floods: {}", flood.written.load(Ordering::Relaxed));

    let began = Instant::now();
    let closed = tokio::time::timeout(Duration::from_secs(10), service.invoke("ws_close", json!({ "jobId": id, "code": 1000 }))).await;
    let took = began.elapsed();
    println!("closed in {took:?} after {} written", flood.written.load(Ordering::Relaxed));
    assert!(closed.is_ok(), "the close came back within 10 s");
    let closed = closed.unwrap().unwrap();
    assert_eq!(closed["by"], "client", "{closed}");
    assert!(took < Duration::from_secs(4), "a close frame and at most the grace for an answer: {took:?}");
    for _ in 0..100 {
        if service.jobs().list().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(service.jobs().list().is_empty(), "the job ended with the connection");

    // What the screen got: every message it was not sent is counted, and the
    // totals say how many came.
    let all = batches(&recorder);
    let shown: u64 = all.iter().map(|batch| batch["messages"].as_array().map_or(0, |messages| messages.len() as u64)).sum();
    let dropped: u64 = all.iter().map(|batch| batch["dropped"].as_u64().unwrap_or(0)).sum();
    println!("{} batches, {shown} shown, {dropped} dropped", all.len());
    let received = all.last().and_then(|batch| batch["received"].as_u64()).expect("each batch says how many came so far");
    assert_eq!(shown + dropped, received, "every message shown or counted as not shown");
    assert!(received > 1000, "{received}");
    let per_batch = all.iter().map(|batch| batch["messages"].as_array().map_or(0, Vec::len)).max().unwrap();
    assert!(per_batch <= 2000, "{per_batch}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_stopped_job_hangs_up_on_a_flood() {
    let (service, _) = service();
    let (url, flood) = flooding(Message::text("tick")).await;
    let job = service.invoke("ws_connect", json!({ "config": { "url": url } })).await.unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(service.jobs().stop(job["id"].as_u64().unwrap()));
    let began = Instant::now();
    while !flood.ended.load(Ordering::SeqCst) && began.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(flood.ended.load(Ordering::SeqCst), "the server saw the connection end after a stop");
    assert!(began.elapsed() < Duration::from_secs(4), "{:?}", began.elapsed());
    // A second connection right after is made. (This stand-in floods only its
    // first connection — its "ended" flag is shared — so it hangs up on the second
    // at once, and that job may have ended by itself before it is stopped.)
    let again = service.invoke("ws_connect", json!({ "config": { "url": url } })).await.unwrap();
    assert!(again["id"].as_u64().is_some(), "{again}");
    service.jobs().stop(again["id"].as_u64().unwrap());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn huge_messages_back_to_back_reach_the_screen_within_a_budget() {
    let (service, recorder) = service();
    let (url, _flood) = flooding(Message::binary(vec![0xab; 16 << 20])).await;
    let job = service.invoke("ws_connect", json!({ "config": { "url": url } })).await.unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;
    let _ = service.invoke("ws_close", json!({ "jobId": job["id"], "code": 1000 })).await;
    let all = batches(&recorder);
    let biggest = all.iter().map(|batch| batch.to_string().len()).max().unwrap_or(0);
    let total: usize = all.iter().map(|batch| batch.to_string().len()).sum();
    println!("{} batches, the largest {biggest} B, {total} B in all", all.len());
    assert!(biggest <= 4 << 20, "a batch is at most a few MiB of JSON: {biggest}");
    let message = all.iter().flat_map(|batch| batch["messages"].as_array().cloned().unwrap_or_default()).next().expect("a message shown");
    assert_eq!((message["bytes"].as_u64(), message["truncated"].as_bool()), (Some(16 << 20), Some(true)), "the size is the message's");
    assert!(message["text"].as_str().unwrap().len() <= 4096 * 3, "a binary message's text is no more than its hex");
}
