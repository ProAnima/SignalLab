//! MQTT 3.1.1 as the specification writes it, both ways. No MQTT library is at
//! hand to test against, so the peers here are written from the
//! specification's own text, byte by byte — never with Signal Lab's codec:
//!
//! - the client (the MQTT screen's connection) against brokers that keep
//!   silent after CONNACK (keep-alive, 3.1.2.10), redeliver a QoS 2 message
//!   (4.3.3), and never acknowledge one publish while 65 535 others go by
//!   (packet identifiers, 2.3.1);
//! - the broker emulator against a client that speaks in the specification's
//!   examples: CONNECT, SUBSCRIBE with its example's filters and SUBACK, QoS 1
//!   and 2 flows, the RETAIN flag's two meanings (3.3.1.3), `$` topics and
//!   wildcards (4.7.2), a will at QoS 1 retained, PINGREQ, UNSUBSCRIBE, DISCONNECT.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn service() -> (Service, Arc<Recorder>) {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-mqtt-conformance-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        dir
    });
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (Service::new(host, Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))), recorder)
}

/// One whole control packet: the fixed header's first byte, then the body
/// (its Remaining Length read as 2.2.3 describes it, 7 bits a byte).
async fn packet(stream: &mut TcpStream) -> Option<(u8, Vec<u8>)> {
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
    Some((first[0], body))
}

/// The next packet within `wait`, or None.
async fn next(stream: &mut TcpStream, wait: Duration) -> Option<(u8, Vec<u8>)> {
    tokio::time::timeout(wait, packet(stream)).await.ok().flatten()
}

/// A UTF-8 string as 1.5.3 encodes it: two bytes of length, then the bytes.
fn string(text: &str) -> Vec<u8> {
    let mut out = (text.len() as u16).to_be_bytes().to_vec();
    out.extend_from_slice(text.as_bytes());
    out
}

/// A packet: the first byte, the Remaining Length (one byte: these are short), the body.
fn framed(first: u8, body: &[u8]) -> Vec<u8> {
    assert!(body.len() < 128);
    [&[first, body.len() as u8][..], body].concat()
}

/// A broker that answers CONNECT with CONNACK 0, then hands the socket to `then`.
async fn broker<F, Fut>(then: F) -> u16
where
    F: Fn(TcpStream) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let then = then.clone();
            tokio::spawn(async move {
                let Some((0x10, _)) = packet(&mut stream).await else { return };
                if stream.write_all(&[0x20, 0x02, 0x00, 0x00]).await.is_ok() {
                    then(stream).await;
                }
            });
        }
    });
    port
}

fn client(port: u16, keep_alive_s: u16) -> Value {
    json!({ "host": "127.0.0.1", "port": port, "client_id": "conformance", "keep_alive_s": keep_alive_s })
}

/// 3.1.2.10: a client that sends PINGREQ and hears nothing "SHOULD close the
/// Network Connection". Before, a broker that froze after CONNACK — its TCP
/// connection still open — left the screen saying `connected` for ever.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_broker_that_answers_no_pingreq_is_gone_within_two_keep_alives() {
    let (service, recorder) = service();
    let pings = Arc::new(AtomicU32::new(0));
    let counted = pings.clone();
    let port = broker(move |mut stream| {
        let pings = counted.clone();
        async move {
            // Takes everything, answers nothing.
            while let Some((first, _)) = packet(&mut stream).await {
                if first == 0xc0 {
                    pings.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    })
    .await;
    let started = Instant::now();
    let job = service.invoke("mqtt_connect", json!({ "config": client(port, 2) })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let waiting = recorder.clone();
    let ended = tokio::task::spawn_blocking(move || waiting.wait_for("job://ended", Duration::from_secs(8), |ended| ended["job_id"] == id)).await.unwrap();
    let ended = ended.expect("the connection ended");
    assert_eq!(ended["error"]["code"], "transport.timeout", "{ended}");
    assert!(started.elapsed() >= Duration::from_millis(2900) && started.elapsed() < Duration::from_millis(5500), "{:?}", started.elapsed());
    assert!(pings.load(Ordering::SeqCst) >= 2, "pings went out first");

    // A broker that answers its pings keeps the connection: three keep-alives later it is still up.
    let port = broker(|mut stream| async move {
        while let Some((first, _)) = packet(&mut stream).await {
            if first == 0xc0 && stream.write_all(&[0xd0, 0x00]).await.is_err() {
                return;
            }
        }
    })
    .await;
    let job = service.invoke("mqtt_connect", json!({ "config": client(port, 1) })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    tokio::time::sleep(Duration::from_millis(3500)).await;
    assert!(!recorder.payloads("job://ended").iter().any(|ended| ended["job_id"] == id), "an answering broker is never timed out");
    service.invoke("job_stop", json!({ "id": id })).await.unwrap();
}

/// 4.3.3: a QoS 2 PUBLISH the broker sends again before PUBREL (DUP set) is
/// one message; after PUBREL the same packet id is a new one.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_qos_2_message_sent_again_before_pubrel_is_one_message() {
    let (service, recorder) = service();
    let replies = Arc::new(Mutex::new(Vec::<u8>::new()));
    let seen = replies.clone();
    let port = broker(move |mut stream| {
        let seen = seen.clone();
        async move {
            let publish = |first: u8, payload: &str| framed(first, &[&string("hall/cue")[..], &[0x00, 0x07], payload.as_bytes()].concat());
            // QoS 2, id 7; the same again with DUP; PUBREL; then id 7 again: a new message.
            let mut script = publish(0x34, "one");
            script.extend(publish(0x3c, "one"));
            script.extend([0x62, 0x02, 0x00, 0x07]);
            script.extend(publish(0x34, "two"));
            script.extend([0x62, 0x02, 0x00, 0x07]);
            if stream.write_all(&script).await.is_err() {
                return;
            }
            while let Some((first, body)) = packet(&mut stream).await {
                if body == [0x00, 0x07] {
                    seen.lock().unwrap().push(first);
                }
            }
        }
    })
    .await;
    let job = service.invoke("mqtt_connect", json!({ "config": client(port, 30) })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while replies.lock().unwrap().len() < 5 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // PUBREC for each PUBLISH (the duplicate too), PUBCOMP for each PUBREL.
    assert_eq!(*replies.lock().unwrap(), [0x50, 0x50, 0x70, 0x50, 0x70], "{:?}", recorder.events());
    tokio::time::sleep(Duration::from_millis(250)).await;
    let counted: u64 = recorder.payloads("mqtt://messages").iter().flat_map(|batch| batch["messages"].as_array().cloned().unwrap_or_default()).map(|message| message["count"].as_u64().unwrap_or(0)).sum();
    assert_eq!(counted, 2, "one and two, the duplicate not shown again");
    service.invoke("job_stop", json!({ "id": id })).await.unwrap();
}

/// A broker keeping a session (the screen may ask with *Clean session* off)
/// sends what it kept right behind CONNACK — 3.2.2.2's Session Present — in the
/// same segment. Before, the bytes read with the CONNACK were dropped: the
/// message was lost, never acknowledged, and a packet cut there read from its middle.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn what_a_broker_sends_behind_connack_in_the_same_segment_is_read() {
    let (service, recorder) = service();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let acked = Arc::new(Mutex::new(Vec::<(u8, Vec<u8>)>::new()));
    let seen = acked.clone();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let Some((0x10, _)) = packet(&mut stream).await else { return };
        // CONNACK with Session Present, then a kept QoS 1 message (DUP: a redelivery) — one write.
        let kept = framed(0x3a, &[&string("lab/kept")[..], &[0x01, 0x2c], b"while away"].concat());
        let whole = [&[0x20, 0x02, 0x01, 0x00][..], &kept].concat();
        stream.write_all(&whole).await.unwrap();
        while let Some(reply) = packet(&mut stream).await {
            seen.lock().unwrap().push(reply);
        }
    });
    let config = json!({ "host": "127.0.0.1", "port": port, "client_id": "away", "keep_alive_s": 30, "clean_session": false });
    let job = service.invoke("mqtt_connect", json!({ "config": config })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    let waiting = recorder.clone();
    let batch = tokio::task::spawn_blocking(move || waiting.wait_for("mqtt://messages", Duration::from_secs(3), |batch| batch["job_id"] == id)).await.unwrap();
    let batch = batch.expect("the kept message is shown");
    assert_eq!((batch["messages"][0]["topic"].as_str(), batch["messages"][0]["payload"].as_str(), batch["messages"][0]["dup"].as_bool()), (Some("lab/kept"), Some("while away"), Some(true)));
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(acked.lock().unwrap().contains(&(0x40, vec![0x01, 0x2c])), "and acknowledged: {:?}", acked.lock().unwrap());
    service.invoke("job_stop", json!({ "id": id })).await.unwrap();
}

/// 2.3.1: "If a Client re-sends a particular Control Packet, then it MUST use
/// the same Packet Identifier ... a Client MUST assign it a currently unused
/// Packet Identifier". Before, after 65 535 publishes the count came round to
/// an id the broker had not acknowledged yet.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_id_the_broker_has_not_acknowledged_is_not_given_again_after_65535() {
    let (service, _) = service();
    let reused = Arc::new(AtomicBool::new(false));
    let received = Arc::new(AtomicU32::new(0));
    let (flag, count) = (reused.clone(), received.clone());
    let port = broker(move |mut stream| {
        let (reused, received) = (flag.clone(), count.clone());
        async move {
            let mut held: Option<u16> = None;
            let mut out = Vec::new();
            while let Some((first, body)) = packet(&mut stream).await {
                if first & 0xf0 != 0x30 {
                    continue;
                }
                let topic_length = u16::from_be_bytes([body[0], body[1]]) as usize;
                let id = u16::from_be_bytes([body[2 + topic_length], body[3 + topic_length]]);
                received.fetch_add(1, Ordering::SeqCst);
                match held {
                    // The first is never acknowledged.
                    None => held = Some(id),
                    Some(kept) if kept == id => reused.store(true, Ordering::SeqCst),
                    Some(_) => out.extend([0x40, 0x02, (id >> 8) as u8, id as u8]),
                }
                if out.len() > 4000 || received.load(Ordering::SeqCst) > 65_530 {
                    if stream.write_all(&out).await.is_err() {
                        return;
                    }
                    out.clear();
                }
            }
        }
    })
    .await;
    let job = service.invoke("mqtt_connect", json!({ "config": client(port, 60) })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    for n in 0..65_540u32 {
        service.invoke("mqtt_publish", json!({ "jobId": id, "topic": "count", "payload": "", "qos": 1, "retain": false })).await.unwrap();
        if n % 2000 == 1999 {
            // Let the broker's acknowledgements catch up, as a person's sends would.
            let deadline = Instant::now() + Duration::from_secs(5);
            while received.load(Ordering::SeqCst) < n + 1 && Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        }
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while received.load(Ordering::SeqCst) < 65_540 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(received.load(Ordering::SeqCst), 65_540);
    assert!(!reused.load(Ordering::SeqCst), "the unacknowledged id was given to another publish");
    service.invoke("job_stop", json!({ "id": id })).await.unwrap();
}

/// What the screen refuses before a broker sees it: a filter 3.1.1 forbids
/// (brokers drop the whole connection over one), and a topic too long for
/// MQTT's two-byte length (it went out wrapped: the broker read a shorter
/// topic and took the rest for payload — a publish to another topic).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn filters_3_1_1_forbids_and_topics_past_its_length_are_refused_before_the_wire() {
    let (service, _) = service();
    let written = Arc::new(AtomicU32::new(0));
    let count = written.clone();
    let port = broker(move |mut stream| {
        let count = count.clone();
        async move {
            while let Some((first, body)) = packet(&mut stream).await {
                if first & 0xf0 == 0x80 {
                    count.fetch_add(1, Ordering::SeqCst);
                    let _ = stream.write_all(&[0x90, 0x03, body[0], body[1], 0x00]).await;
                }
                if first & 0xf0 == 0x30 {
                    count.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    })
    .await;
    let code = |result: Result<Value, signal_lab_engine::Failure>| serde_json::to_value(result.unwrap_err()).unwrap();
    let bad = code(service.invoke("mqtt_connect", json!({ "config": { "host": "127.0.0.1", "port": port, "client_id": "c", "subscribe": [{ "filter": "lab/#/x" }] } })).await);
    assert_eq!((bad["code"].as_str(), bad["params"]["value"].as_str()), (Some("node.topic_filter"), Some("lab/#/x")), "{bad}");
    let job = service.invoke("mqtt_connect", json!({ "config": client(port, 30) })).await.unwrap();
    let id = job["id"].as_u64().unwrap();
    for filter in ["a/#/b", "a+", "", "sport/tennis#"] {
        let refused = code(service.invoke("mqtt_subscribe", json!({ "jobId": id, "filters": [{ "filter": filter, "qos": 0 }] })).await);
        assert_eq!(refused["code"], "node.topic_filter", "{filter}: {refused}");
        let refused = code(service.invoke("mqtt_unsubscribe", json!({ "jobId": id, "filters": [filter] })).await);
        assert_eq!(refused["code"], "node.topic_filter", "{filter}: {refused}");
    }
    let long = "a".repeat(70_000);
    let refused = code(service.invoke("mqtt_publish", json!({ "jobId": id, "topic": long, "payload": "x", "qos": 0, "retain": false })).await);
    assert_eq!((refused["code"].as_str(), refused["params"]["max"].as_str()), (Some("node.too_long"), Some("65535")), "{refused}");
    let nul = code(service.invoke("mqtt_publish", json!({ "jobId": id, "topic": "a\u{0}b", "payload": "x", "qos": 0, "retain": false })).await);
    assert_eq!(nul["code"], "mqtt.topic_invalid");
    let once = code(service.invoke("mqtt_publish_once", json!({ "config": client(port, 30), "topic": "a".repeat(70_000), "payload": "", "qos": 0, "retain": false })).await);
    assert_eq!(once["code"], "node.too_long");
    let will = code(service.invoke("mqtt_connect", json!({ "config": { "host": "127.0.0.1", "port": port, "client_id": "w", "will": { "topic": "dev/+/online", "payload": "0" } } })).await);
    assert_eq!(will["code"], "node.topic_wildcard");
    service.invoke("mqtt_subscribe", json!({ "jobId": id, "filters": [{ "filter": "+/+/#", "qos": 1 }] })).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(written.load(Ordering::SeqCst), 1, "only the valid subscription reached the broker");
    service.invoke("job_stop", json!({ "id": id })).await.unwrap();
}

// ---------------------------------------------------------------------------
// the broker emulator against a client written from the specification
// ---------------------------------------------------------------------------

/// CONNECT as 3.1 lays it out: "MQTT", level 4, the flags, keep-alive, the payload.
fn connect(client_id: &str, keep_alive: u16, will: Option<(&str, &str, u8, bool)>) -> Vec<u8> {
    let mut flags = 0x02u8; // clean session
    let mut payload = string(client_id);
    if let Some((topic, message, qos, retain)) = will {
        flags |= 0x04 | (qos << 3) | if retain { 0x20 } else { 0 };
        payload.extend(string(topic));
        payload.extend(string(message));
    }
    let body = [&string("MQTT")[..], &[0x04, flags], &keep_alive.to_be_bytes(), &payload].concat();
    framed(0x10, &body)
}

async fn emulator(service: &Service) -> (u64, String) {
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let job = service.invoke("emulator_start", json!({ "emulator": { "name": "Spec broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt" } })).await.unwrap();
    (job["id"].as_u64().unwrap(), format!("127.0.0.1:{port}"))
}

async fn connected(broker: &str, client_id: &str, will: Option<(&str, &str, u8, bool)>) -> TcpStream {
    let mut stream = TcpStream::connect(broker).await.unwrap();
    stream.write_all(&connect(client_id, 30, will)).await.unwrap();
    // 3.2: CONNACK is 0x20, length 2, no session present, return code 0.
    assert_eq!(next(&mut stream, Duration::from_secs(2)).await, Some((0x20, vec![0x00, 0x00])), "{client_id}");
    stream
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_broker_emulator_answers_the_specification_s_own_examples() {
    let (service, _) = service();
    let (job, broker) = emulator(&service).await;
    let wait = Duration::from_secs(2);
    let mut watcher = connected(&broker, "watcher", None).await;

    // 3.8.3's example: packet id 10, "a/b" at QoS 1, "c/d" at QoS 2 — and 3.9.3's SUBACK.
    let subscribe = [&[0x00, 0x0a][..], &string("a/b"), &[0x01], &string("c/d"), &[0x02]].concat();
    watcher.write_all(&framed(0x82, &subscribe)).await.unwrap();
    assert_eq!(next(&mut watcher, wait).await, Some((0x90, vec![0x00, 0x0a, 0x01, 0x02])));

    let mut sender = connected(&broker, "sender", None).await;
    // QoS 1 publish to a/b, id 10 (3.3.2.3's example values): PUBACK 0x40 back, delivered at QoS 1.
    sender.write_all(&framed(0x32, &[&string("a/b")[..], &[0x00, 0x0a], b"one"].concat())).await.unwrap();
    assert_eq!(next(&mut sender, wait).await, Some((0x40, vec![0x00, 0x0a])));
    let (first, body) = next(&mut watcher, wait).await.unwrap();
    assert_eq!(first, 0x32, "QoS 1, not retained, not DUP");
    assert_eq!(&body[..5], &string("a/b")[..]);
    assert_eq!(&body[7..], b"one");
    let id = [body[5], body[6]];
    assert_ne!(id, [0, 0], "a packet id is never 0");
    watcher.write_all(&[0x40, 0x02, id[0], id[1]]).await.unwrap();

    // QoS 2 to c/d: PUBREC, PUBREL (flags 0010), PUBCOMP; delivered at QoS 2 with the broker's side of it.
    sender.write_all(&framed(0x34, &[&string("c/d")[..], &[0x00, 0x0b], b"two"].concat())).await.unwrap();
    assert_eq!(next(&mut sender, wait).await, Some((0x50, vec![0x00, 0x0b])));
    sender.write_all(&[0x62, 0x02, 0x00, 0x0b]).await.unwrap();
    assert_eq!(next(&mut sender, wait).await, Some((0x70, vec![0x00, 0x0b])));
    let (first, body) = next(&mut watcher, wait).await.unwrap();
    assert_eq!((first, &body[body.len() - 3..]), (0x34, &b"two"[..]));
    let id = [body[5], body[6]];
    watcher.write_all(&[0x50, 0x02, id[0], id[1]]).await.unwrap();
    assert_eq!(next(&mut watcher, wait).await, Some((0x62, id.to_vec())), "PUBREL carries its 0010 flags");
    watcher.write_all(&[0x70, 0x02, id[0], id[1]]).await.unwrap();

    // A QoS 0 message to a subscription granted QoS 1 arrives at QoS 0 (the lower of the two).
    sender.write_all(&framed(0x30, &[&string("a/b")[..], b"zero"].concat())).await.unwrap();
    assert_eq!(next(&mut watcher, wait).await, Some((0x30, [&string("a/b")[..], b"zero"].concat())));

    // 3.3.1.3: retained — to a subscription made later RETAIN is 1; to one that exists, 0.
    sender.write_all(&framed(0x31, &[&string("c/d")[..], b"kept"].concat())).await.unwrap();
    assert_eq!(next(&mut watcher, wait).await, Some((0x30, [&string("c/d")[..], b"kept"].concat())), "routed live: RETAIN 0");
    let mut late = connected(&broker, "late", None).await;
    late.write_all(&framed(0x82, &[&[0x00, 0x01][..], &string("c/+"), &[0x00]].concat())).await.unwrap();
    assert_eq!(next(&mut late, wait).await, Some((0x90, vec![0x00, 0x01, 0x00])));
    assert_eq!(next(&mut late, wait).await, Some((0x31, [&string("c/d")[..], b"kept"].concat())), "from the store: RETAIN 1");

    // 4.7.2: a filter starting with a wildcard does not take a topic starting with $.
    late.write_all(&framed(0x82, &[&[0x00, 0x02][..], &string("#"), &[0x00], &string("+/info"), &[0x00]].concat())).await.unwrap();
    assert_eq!(next(&mut late, wait).await, Some((0x90, vec![0x00, 0x02, 0x00, 0x00])));
    // The retained c/d comes again for "#": a new subscription.
    assert_eq!(next(&mut late, wait).await.map(|(first, _)| first), Some(0x31));
    sender.write_all(&framed(0x30, &[&string("$SYS/info")[..], b"x"].concat())).await.unwrap();
    assert_eq!(next(&mut late, Duration::from_millis(400)).await, None, "$SYS/info is not for # or +/info");
    late.write_all(&framed(0x82, &[&[0x00, 0x03][..], &string("$SYS/#"), &[0x00]].concat())).await.unwrap();
    assert_eq!(next(&mut late, wait).await, Some((0x90, vec![0x00, 0x03, 0x00])));
    sender.write_all(&framed(0x30, &[&string("$SYS/info")[..], b"y"].concat())).await.unwrap();
    assert_eq!(next(&mut late, wait).await, Some((0x30, [&string("$SYS/info")[..], b"y"].concat())), "but for $SYS/#");

    // 3.10/3.11: UNSUBSCRIBE (flags 0010) and UNSUBACK; nothing more on c/d for watcher.
    watcher.write_all(&framed(0xa2, &[&[0x00, 0x0c][..], &string("c/d")].concat())).await.unwrap();
    assert_eq!(next(&mut watcher, wait).await, Some((0xb0, vec![0x00, 0x0c])));
    sender.write_all(&framed(0x30, &[&string("c/d")[..], b"gone"].concat())).await.unwrap();
    assert_eq!(next(&mut watcher, Duration::from_millis(400)).await, None);

    // 3.12/3.13: PINGREQ, PINGRESP.
    watcher.write_all(&[0xc0, 0x00]).await.unwrap();
    assert_eq!(next(&mut watcher, wait).await, Some((0xd0, vec![])));

    // 3.1.2.5: a will at QoS 1, retained, published when the connection is lost — not after DISCONNECT.
    let mut device = connected(&broker, "device", Some(("dev/1/online", "false", 1, true))).await;
    watcher.write_all(&framed(0x82, &[&[0x00, 0x0d][..], &string("dev/+/online"), &[0x01]].concat())).await.unwrap();
    assert_eq!(next(&mut watcher, wait).await, Some((0x90, vec![0x00, 0x0d, 0x01])));
    device.shutdown().await.unwrap();
    drop(device);
    let (first, body) = next(&mut watcher, wait).await.unwrap();
    assert_eq!((first, &body[..14], &body[16..]), (0x32, &string("dev/1/online")[..], &b"false"[..]), "the will, at QoS 1, live: RETAIN 0");
    let mut polite = connected(&broker, "polite", Some(("dev/2/online", "false", 0, false))).await;
    polite.write_all(&[0xe0, 0x00]).await.unwrap();
    drop(polite);
    assert_eq!(next(&mut watcher, Duration::from_millis(400)).await, None, "no will after DISCONNECT");
    let mut later = connected(&broker, "later", None).await;
    later.write_all(&framed(0x82, &[&[0x00, 0x01][..], &string("dev/+/online"), &[0x01]].concat())).await.unwrap();
    assert_eq!(next(&mut later, wait).await, Some((0x90, vec![0x00, 0x01, 0x01])));
    let (first, body) = next(&mut later, wait).await.unwrap();
    assert_eq!((first & 0x0f, &body[16..]), (0x03, &b"false"[..]), "the will was retained at QoS 1");

    service.invoke("job_stop", json!({ "id": job })).await.unwrap();
}
