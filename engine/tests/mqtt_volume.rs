//! The MQTT screen's connection against a broker that floods it: 50 000
//! retained topics at once on subscribing, a firehose on a few topics, a
//! payload of tens of megabytes, packets no broker may send. The topic tree
//! is fed every topic and every count, the batches stay within their bounds,
//! and a broken packet ends the connection with the reason.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::mqtt_codec::{encode_connack, encode_publish, encode_suback};
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn service() -> (Service, Arc<Recorder>) {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-mqtt-volume-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        dir
    });
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))), recorder)
}

/// Reads one whole packet (fixed header and body) from a client.
async fn packet(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut first = [0u8; 1];
    stream.read_exact(&mut first).await.ok()?;
    let (mut length, mut multiplier) = (0usize, 1usize);
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).await.ok()?;
        length += (byte[0] & 0x7f) as usize * multiplier;
        multiplier *= 128;
        if byte[0] & 0x80 == 0 {
            break;
        }
    }
    let mut body = vec![0u8; length];
    stream.read_exact(&mut body).await.ok()?;
    let mut whole = first.to_vec();
    whole.extend(body);
    Some(whole)
}

/// What the broker wrote, and whether it is done writing.
#[derive(Default)]
struct Sent {
    publishes: AtomicU64,
    done: AtomicBool,
}

/// A broker that takes the CONNECT and the SUBSCRIBE, grants it, then runs
/// `then` on the socket; it reads (and drops) what the client sends after.
async fn broker<F, Fut>(then: F) -> (u16, Arc<Sent>)
where
    F: Fn(TcpStream, Arc<Sent>) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let sent = Arc::new(Sent::default());
    let shared = sent.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let (then, sent) = (then.clone(), shared.clone());
            tokio::spawn(async move {
                let Some(_connect) = packet(&mut stream).await else { return };
                if stream.write_all(&encode_connack(false, 0)).await.is_err() {
                    return;
                }
                let Some(subscribe) = packet(&mut stream).await else { return };
                let id = u16::from_be_bytes([subscribe[2], subscribe[3]]);
                if stream.write_all(&encode_suback(id, &[0])).await.is_err() {
                    return;
                }
                then(stream, sent).await;
            });
        }
    });
    (port, sent)
}

async fn connect(service: &Service, port: u16) -> u64 {
    let config = json!({ "host": "127.0.0.1", "port": port, "client_id": "volume", "subscribe": [{ "filter": "#", "qos": 0 }] });
    service.invoke("mqtt_connect", json!({ "config": config })).await.unwrap()["id"].as_u64().unwrap()
}

/// Every batch so far: the messages they carried (each standing for `count`
/// of its topic, 1 when it says none) and what they shed.
fn tally(recorder: &Recorder) -> (std::collections::HashMap<String, u64>, u64, Vec<Value>) {
    let mut topics = std::collections::HashMap::new();
    let mut dropped = 0;
    let batches = recorder.payloads("mqtt://messages");
    for batch in &batches {
        dropped += batch["dropped"].as_u64().unwrap_or(0);
        for message in batch["messages"].as_array().into_iter().flatten() {
            *topics.entry(message["topic"].as_str().unwrap().to_string()).or_insert(0) += message["count"].as_u64().unwrap_or(1);
        }
    }
    (topics, dropped, batches)
}

async fn settle(recorder: &Recorder, until: impl Fn(&std::collections::HashMap<String, u64>, u64) -> bool) {
    let began = Instant::now();
    while began.elapsed() < Duration::from_secs(15) {
        let (topics, dropped, _) = tally(recorder);
        if until(&topics, dropped) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fifty_thousand_retained_topics_at_once_all_reach_the_tree() {
    const TOPICS: u64 = 50_000;
    let (service, recorder) = service();
    let (port, _) = broker(|mut stream: TcpStream, sent: Arc<Sent>| async move {
        let mut all = Vec::new();
        for n in 0..TOPICS {
            all.extend(encode_publish(&format!("site/{}/dev{n}/state", n % 50), format!("{{\"on\":{}}}", n % 2).as_bytes(), 0, true, 0, false));
        }
        let _ = stream.write_all(&all).await;
        sent.publishes.store(TOPICS, Ordering::SeqCst);
        let mut sink = [0u8; 1024];
        while matches!(stream.read(&mut sink).await, Ok(read) if read > 0) {}
    })
    .await;
    let began = Instant::now();
    connect(&service, port).await;
    settle(&recorder, |topics, dropped| topics.len() as u64 + dropped >= TOPICS).await;
    let (topics, dropped, batches) = tally(&recorder);
    let largest = batches.iter().map(|batch| batch.to_string().len()).max().unwrap_or(0);
    println!("{} topics, {dropped} shed, in {} batches (largest {largest} B), {:?}", topics.len(), batches.len(), began.elapsed());
    assert_eq!((topics.len() as u64, dropped), (TOPICS, 0), "every retained topic is in the tree");
    assert!(largest <= 4 << 20, "a batch stays a few MiB: {largest}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_firehose_on_a_few_topics_is_counted_whole() {
    let (service, recorder) = service();
    let (port, sent) = broker(|mut stream: TcpStream, sent: Arc<Sent>| async move {
        let mut block = Vec::new();
        for n in 0..1000u64 {
            block.extend(encode_publish(&format!("hall/fader{}", n % 20), b"0.5", 0, false, 0, false));
        }
        let began = Instant::now();
        while began.elapsed() < Duration::from_millis(1500) {
            if stream.write_all(&block).await.is_err() {
                return;
            }
            sent.publishes.fetch_add(1000, Ordering::SeqCst);
        }
        sent.done.store(true, Ordering::SeqCst);
        let mut sink = [0u8; 1024];
        while matches!(stream.read(&mut sink).await, Ok(read) if read > 0) {}
    })
    .await;
    connect(&service, port).await;
    let began = Instant::now();
    while !sent.done.load(Ordering::SeqCst) && began.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let written = sent.publishes.load(Ordering::SeqCst);
    settle(&recorder, |topics, dropped| topics.values().sum::<u64>() + dropped >= written).await;
    let (topics, dropped, batches) = tally(&recorder);
    let counted: u64 = topics.values().sum();
    let most = batches.iter().map(|batch| batch["messages"].as_array().map_or(0, Vec::len)).max().unwrap_or(0);
    println!("{written} written: {counted} counted on {} topics, {dropped} shed, {} batches, at most {most} messages in one", topics.len(), batches.len());
    assert_eq!(topics.len(), 20);
    assert_eq!((counted, dropped), (written, 0), "every message counted on its topic");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_huge_payload_reaches_the_screen_cut_with_its_size() {
    const SIZE: usize = 64 << 20;
    let (service, recorder) = service();
    let (port, _) = broker(|mut stream: TcpStream, _| async move {
        let _ = stream.write_all(&encode_publish("cam/1/frame", &vec![b'z'; SIZE], 0, true, 0, false)).await;
        let _ = stream.write_all(&encode_publish("cam/1/state", b"ok", 0, false, 0, false)).await;
        let mut sink = [0u8; 1024];
        while matches!(stream.read(&mut sink).await, Ok(read) if read > 0) {}
    })
    .await;
    connect(&service, port).await;
    settle(&recorder, |topics, _| topics.contains_key("cam/1/state")).await;
    let (_, _, batches) = tally(&recorder);
    let frame = batches.iter().flat_map(|batch| batch["messages"].as_array().cloned().unwrap_or_default()).find(|message| message["topic"] == "cam/1/frame").expect("the frame");
    assert_eq!((frame["bytes"].as_u64(), frame["truncated"].as_bool()), (Some(SIZE as u64), Some(true)), "{}", frame["bytes"]);
    assert!(frame["payload"].as_str().unwrap().len() <= 4096);
}

async fn ended_with(port: u16) -> Value {
    let (service, recorder) = service();
    let id = connect(&service, port).await;
    let ended = tokio::task::spawn_blocking(move || recorder.wait_for("job://ended", Duration::from_secs(10), |ended| ended["job_id"] == id)).await.unwrap();
    ended.expect("the connection ends")["error"].clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn packets_no_broker_may_send_end_the_connection_with_the_reason() {
    for (name, bytes) in [
        ("QoS 3", vec![0x36, 0x03, 0x00, 0x01, b'a']),
        ("a length of five bytes", vec![0x30, 0xff, 0xff, 0xff, 0xff, 0x7f]),
        ("a topic longer than its packet", vec![0x30, 0x03, 0x00, 0x09, b'a']),
        ("a CONNECT", vec![0x10, 0x00]),
    ] {
        let (port, _) = broker(move |mut stream: TcpStream, _| {
            let bytes = bytes.clone();
            async move {
                let _ = stream.write_all(&bytes).await;
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        })
        .await;
        let error = ended_with(port).await;
        assert_eq!(error["code"], "mqtt.protocol", "{name}: {error}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_broker_that_answers_connect_with_megabytes_is_refused_in_a_line() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let _ = packet(&mut stream).await;
                let _ = stream.write_all(&encode_publish("x", &vec![7u8; 8 << 20], 0, false, 0, false)).await;
                tokio::time::sleep(Duration::from_secs(8)).await;
            });
        }
    });
    let (service, _) = service();
    let began = Instant::now();
    let config = json!({ "host": "127.0.0.1", "port": port, "client_id": "volume" });
    let error = serde_json::to_value(service.invoke("mqtt_connect", json!({ "config": config })).await.unwrap_err()).unwrap();
    let detail = error["detail"].as_str().unwrap_or_default().len();
    println!("refused in {:?}: {} with {detail} B of detail", began.elapsed(), error["code"]);
    assert_eq!(error["code"], "mqtt.protocol", "{}", error["code"]);
    assert!(detail < 200, "the reason, not the packet: {detail} B");
    assert!(began.elapsed() < Duration::from_secs(2), "refused on the first byte, not after reading it all");
}
