//! The HTTP client against loopback servers that misbehave by volume: a body
//! that stalls halfway, one that never ends, one far larger than is shown, a
//! gzip body that inflates — what is kept, what is counted, what it says.

use std::time::{Duration, Instant};

use signal_lab_engine::host::Recorder;
use signal_lab_engine::http::{request_once, start_burst, BurstConfig, HttpRequest};
use signal_lab_engine::{Capture, Host, JobRegistry};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// A server that answers every connection with `answer(stream)`, after reading the request head.
async fn server<F, Fut>(answer: F) -> String
where
    F: Fn(TcpStream) -> Fut + Send + Sync + 'static + Clone,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let answer = answer.clone();
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut byte = [0u8; 1024];
                while !head.windows(4).any(|window| window == b"\r\n\r\n") {
                    match stream.read(&mut byte).await {
                        Ok(0) | Err(_) => return,
                        Ok(read) => head.extend_from_slice(&byte[..read]),
                    }
                }
                answer(stream).await;
            });
        }
    });
    url
}

fn request(url: &str, timeout_ms: u64) -> HttpRequest {
    HttpRequest { method: "GET".into(), url: url.into(), headers: Vec::new(), body: None, timeout_ms, auth: Default::default() }
}

fn host() -> Host {
    Host::new(Recorder::new(), Capture::new())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_body_that_stalls_halfway_is_a_timeout_not_an_empty_success() {
    let url = server(|mut stream: TcpStream| async move {
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 1000\r\n\r\nfirst ten.").await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    })
    .await;
    let response = request_once(host(), request(&url, 400), None).await.unwrap();
    assert_eq!(response.status, 200, "the status did arrive");
    assert!(!response.ok, "a body cut short is no success: {response:?}");
    assert_eq!(response.cause, Some(signal_lab_engine::transport::Cause::Timeout), "{response:?}");
    assert_eq!((response.body.as_str(), response.body_bytes), ("first ten.", 10), "what did arrive is kept and counted");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_body_cut_by_the_server_is_a_reset() {
    let url = server(|mut stream: TcpStream| async move {
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 1000\r\n\r\nfirst ten.").await;
        // Gone with 990 bytes owed.
    })
    .await;
    let response = request_once(host(), request(&url, 3000), None).await.unwrap();
    assert!(!response.ok && response.error.is_some(), "{response:?}");
    assert_eq!(response.body_bytes, 10);
}

/// A chunked body that never ends, written as fast as the client reads.
async fn endless(mut stream: TcpStream) {
    if stream.write_all(b"HTTP/1.1 200 OK\r\ntransfer-encoding: chunked\r\n\r\n").await.is_err() {
        return;
    }
    let chunk = {
        let data = vec![b'x'; 64 * 1024];
        let mut chunk = format!("{:x}\r\n", data.len()).into_bytes();
        chunk.extend_from_slice(&data);
        chunk.extend_from_slice(b"\r\n");
        chunk
    };
    while stream.write_all(&chunk).await.is_ok() {}
}

#[tokio::test(flavor = "multi_thread")]
async fn an_endless_body_ends_at_the_timeout_counted_and_cut_for_display() {
    let url = server(endless).await;
    let began = Instant::now();
    let response = request_once(host(), request(&url, 1500), None).await.unwrap();
    let took = began.elapsed();
    assert!(took < Duration::from_secs(4), "{took:?}");
    assert_eq!(response.cause, Some(signal_lab_engine::transport::Cause::Timeout), "{:?}", response.error);
    assert!(response.body_bytes > 1 << 20, "every byte read is counted: {}", response.body_bytes);
    assert!(response.truncated && response.body.len() <= 256 * 1024, "only the preview is kept: {}", response.body.len());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_hundred_megabyte_body_is_counted_whole_and_kept_as_a_preview() {
    const SIZE: usize = 100 << 20;
    let url = server(|mut stream: TcpStream| async move {
        let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\ncontent-length: {SIZE}\r\n\r\n").as_bytes()).await;
        let block = vec![b'y'; 1 << 20];
        for _ in 0..SIZE >> 20 {
            if stream.write_all(&block).await.is_err() {
                return;
            }
        }
    })
    .await;
    let response = request_once(host(), request(&url, 30_000), None).await.unwrap();
    assert!(response.ok && response.error.is_none(), "{:?}", response.error);
    assert_eq!(response.body_bytes, SIZE);
    assert!(response.truncated && response.body.len() == 256 * 1024);
}

/// Measurement, not a check: a burst against a server that closes every
/// connection, so each request is a new one. Run by hand with `--ignored`.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn measure_a_burst_on_connections_that_close() {
    let total: u64 = std::env::var("BURST_TOTAL").ok().and_then(|value| value.parse().ok()).unwrap_or(4000);
    let url = server(|mut stream: TcpStream| async move {
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nconnection: close\r\ncontent-length: 2\r\n\r\nok").await;
    })
    .await;
    let recorder = Recorder::new();
    let config = BurstConfig { request: request(&url, 5000), concurrency: 512, total, duration_s: 0.0, rate: 0.0, cookies: false };
    let began = Instant::now();
    start_burst(Host::new(recorder.clone(), Capture::new()), JobRegistry::new(), config, None).await.unwrap();
    let last = tokio::task::spawn_blocking(move || recorder.wait_for("http://burst-progress", Duration::from_secs(300), |report| report["done"] == true))
        .await
        .unwrap()
        .expect("the burst ends");
    println!("{total} requests in {:?}: {last}", began.elapsed());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_burst_counts_a_stalled_body_as_failed() {
    let url = server(|mut stream: TcpStream| async move {
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 100\r\n\r\nhalf").await;
        tokio::time::sleep(Duration::from_secs(3)).await;
    })
    .await;
    let recorder = Recorder::new();
    let config = BurstConfig { request: request(&url, 300), concurrency: 4, total: 8, duration_s: 0.0, rate: 0.0, cookies: false };
    start_burst(Host::new(recorder.clone(), Capture::new()), JobRegistry::new(), config, None).await.unwrap();
    let last = tokio::task::spawn_blocking(move || recorder.wait_for("http://burst-progress", Duration::from_secs(10), |report| report["done"] == true))
        .await
        .unwrap()
        .expect("the burst ends");
    assert_eq!((last["sent"].as_u64(), last["ok"].as_u64(), last["failed"].as_u64()), (Some(8), Some(0), Some(8)), "{last}");
}
