//! Hostile peers: what an emulator hears from a client that breaks the
//! protocol on purpose, and what a request carries when a field holds line
//! breaks. Raw sockets on loopback, written by hand where a client library
//! would refuse to send it.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::mqtt_codec;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{paths, Capture, Host, Mode, Service};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn service() -> Arc<Service> {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-hostile-peers-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        dir
    });
    Arc::new(Service::new(Host::new(Recorder::new(), Capture::new()), Mode::Server, Arc::new(FileStore::new(None).with_env(|_| None))))
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

/// Write `request`, then read until the peer closes or `wait` passes.
async fn exchange(port: u16, request: &[u8], wait: Duration) -> (Vec<u8>, bool) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let _ = stream.write_all(request).await;
    let mut answer = Vec::new();
    let closed = tokio::time::timeout(wait, async {
        let mut buffer = [0u8; 4096];
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) | Err(_) => return,
                Ok(n) => answer.extend_from_slice(&buffer[..n]),
            }
        }
    })
    .await
    .is_ok();
    (answer, closed)
}

/// The requests an HTTP emulator's job says it received.
async fn exchanges(service: &Service, job: u64) -> Vec<Value> {
    let snapshot = service.invoke("emulator_exchanges", json!({ "jobId": job })).await.unwrap();
    snapshot["exchanges"].as_array().cloned().unwrap_or_default()
}

/// Content-Length against Transfer-Encoding, the classic way to hide a second
/// request inside the first: the emulator reads a request one way only, never
/// answers the hidden one, and closes a connection that tried.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_http_emulator_cannot_be_smuggled_into() {
    let service = service();
    let port = free_port();
    let job = service
        .invoke("emulator_start", json!({ "emulator": { "name": "Front", "bind": format!("127.0.0.1:{port}"), "protocol": "http",
            "routes": [{ "method": "ANY", "path": "/public", "responses": [{ "body": "public" }] },
                       { "method": "ANY", "path": "/admin", "responses": [{ "body": "SMUGGLED" }] }] } }))
        .await
        .unwrap()["id"]
        .as_u64()
        .unwrap();
    let hidden = "GET /admin HTTP/1.1\r\nHost: x\r\n\r\n";
    let attempts = [
        // CL.TE: Content-Length covers the hidden request, chunked ends before it.
        format!("POST /public HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n{hidden}", 5 + hidden.len()),
        // TE.CL: chunked covers it, Content-Length does not.
        format!("POST /public HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\nContent-Length: 4\r\n\r\n{:x}\r\n{hidden}\r\n0\r\n\r\n", hidden.len()),
        // An obfuscated Transfer-Encoding a lenient parser ignores.
        format!("POST /public HTTP/1.1\r\nHost: x\r\nContent-Length: 5\r\nTransfer-Encoding: xchunked\r\n\r\n0\r\n\r\n{hidden}"),
        format!("POST /public HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked, identity\r\nContent-Length: 5\r\n\r\n0\r\n\r\n{hidden}"),
        // Two lengths that disagree.
        format!("POST /public HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\nContent-Length: {}\r\n\r\n{hidden}", hidden.len()),
        format!("POST /public HTTP/1.1\r\nHost: x\r\nContent-Length: 0, {}\r\n\r\n{hidden}", hidden.len()),
        format!("POST /public HTTP/1.1\r\nHost: x\r\nContent-Length: +{}\r\n\r\n{hidden}", hidden.len()),
        // A header folded onto the next line, and a bare LF line end.
        format!("POST /public HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\n Transfer-Encoding: chunked\r\n\r\n{hidden}"),
        format!("POST /public HTTP/1.1\nHost: x\nContent-Length: 0\nTransfer-Encoding:\tchunked\n\n0\n\n{hidden}"),
        // A space before the colon.
        format!("POST /public HTTP/1.1\r\nHost: x\r\nTransfer-Encoding : chunked\r\nContent-Length: 5\r\n\r\n0\r\n\r\n{hidden}"),
    ];
    for attempt in &attempts {
        let (answer, closed) = exchange(port, attempt.as_bytes(), Duration::from_secs(5)).await;
        let text = String::from_utf8_lossy(&answer);
        assert!(!text.contains("SMUGGLED"), "the hidden request was answered:\n{attempt:?}\n{text}");
        assert!(closed, "the connection stayed open after:\n{attempt:?}");
    }
    let heard = exchanges(&service, job).await;
    assert!(heard.iter().all(|exchange| !exchange["request"].as_str().unwrap_or_default().contains("/admin")), "{heard:#?}");
    // And it still answers an honest client.
    let (answer, _) = exchange(port, b"GET /public HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n", Duration::from_secs(5)).await;
    assert!(String::from_utf8_lossy(&answer).ends_with("public"), "{}", String::from_utf8_lossy(&answer));
    service.jobs().stop_all();
}

/// What an MQTT client that breaks 3.1.1 sends the broker emulator. Each
/// connection is closed without a crash, and the broker keeps serving the
/// clients that follow the rules.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_mqtt_broker_emulator_survives_malformed_clients() {
    let service = service();
    let port = free_port();
    service.invoke("emulator_start", json!({ "emulator": { "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt" } })).await.unwrap();
    let connect = |id: &str| mqtt_codec::encode_connect(&mqtt_codec::ConnectOpts { client_id: id, username: None, password: None, keep_alive_s: 30, clean_session: true, will: None });
    let with = |first: Vec<u8>, then: &[u8]| [first, then.to_vec()].concat();
    let mut wrong_protocol = connect("a");
    wrong_protocol[4..8].copy_from_slice(b"MQTX");
    let mut wrong_level = connect("a");
    wrong_level[8] = 9;
    let mut reserved_flag = connect("a");
    reserved_flag[9] |= 0x01;
    let attempts: Vec<(&str, Vec<u8>)> = vec![
        ("random bytes", (0..64u8).map(|i| i.wrapping_mul(97)).collect()),
        ("nothing at all", Vec::new()),
        ("a PUBLISH before CONNECT", mqtt_codec::encode_publish("a", b"x", 0, false, 0, false)),
        ("a remaining length of five bytes", vec![0x10, 0xff, 0xff, 0xff, 0xff, 0x7f]),
        ("a remaining length of 256 MB, never sent", vec![0x10, 0xff, 0xff, 0xff, 0x7f]),
        ("another protocol name", wrong_protocol),
        ("protocol level 9", wrong_level),
        ("the reserved CONNECT flag", reserved_flag),
        ("a string longer than its packet", vec![0x10, 0x0c, 0x00, 0xff, b'M', b'Q', b'T', b'T', 4, 2, 0, 30, 0, 1]),
        ("CONNECT twice", with(connect("twice"), &connect("twice"))),
        ("QoS 3", with(connect("q3"), &[0x36, 0x05, 0x00, 0x01, b'a', 0x00, 0x01])),
        ("a wildcard in a PUBLISH topic", with(connect("w"), &mqtt_codec::encode_publish("a/+/#", b"x", 0, false, 0, false))),
        ("SUBSCRIBE with # in the middle", with(connect("s"), &mqtt_codec::encode_subscribe(1, &[("a/#/b".into(), 0)]))),
        ("SUBSCRIBE with no filters", with(connect("e"), &[0x82, 0x02, 0x00, 0x01])),
        ("SUBSCRIBE asking QoS 3", with(connect("s3"), &[0x82, 0x06, 0x00, 0x01, 0x00, 0x01, b'a', 0x03])),
        ("an empty topic", with(connect("t"), &mqtt_codec::encode_publish("", b"x", 0, false, 0, false))),
        ("a topic with NUL", with(connect("n"), &mqtt_codec::encode_publish("a\u{0}b", b"x", 0, false, 0, false))),
        ("PUBREL for nothing", with(connect("r"), &mqtt_codec::encode_pubrel(77))),
        ("a packet type 15", with(connect("f"), &[0xf0, 0x00])),
        ("a packet type 0", with(connect("z"), &[0x00, 0x00])),
        ("half a packet, then silence", with(connect("h"), &[0x30, 0x7f, 0x00])),
    ];
    // All at once: one that stops halfway before CONNECT is let go when CONNECT is
    // overdue (10 s); halfway after it, when its keep-alive runs out (45 s here).
    let tries = attempts.into_iter().map(|(name, bytes)| {
        tokio::spawn(async move {
            let (answer, closed) = exchange(port, &bytes, Duration::from_secs(13)).await;
            (name, answer, closed)
        })
    });
    let mut problems = Vec::new();
    for attempt in futures_util::future::join_all(tries).await {
        let (name, answer, closed) = attempt.unwrap();
        let fine = match name {
            "half a packet, then silence" => true,
            // A filter it cannot take is refused in the SUBACK (0x80), the connection kept.
            "SUBSCRIBE with # in the middle" => answer.windows(5).any(|w| w == [0x90, 0x03, 0x00, 0x01, 0x80]),
            // A PUBREL is answered with PUBCOMP whether or not the id is known (MQTT-4.3.3).
            "PUBREL for nothing" => answer.windows(4).any(|w| w == [0x70, 0x02, 0x00, 77]),
            _ => closed,
        };
        if !fine {
            problems.push(format!("{name}: closed {closed}, answered {answer:02x?}"));
        }
    }
    assert!(problems.is_empty(), "the broker answered a broken client wrongly:\n{}", problems.join("\n"));
    // An honest client still gets what it subscribed to.
    let mut subscriber = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    subscriber.write_all(&connect("sub")).await.unwrap();
    subscriber.write_all(&mqtt_codec::encode_subscribe(1, &[("lab/#".into(), 1)])).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let mut publisher = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    publisher.write_all(&connect("pub")).await.unwrap();
    publisher.write_all(&mqtt_codec::encode_publish("lab/x", b"hello", 1, false, 5, false)).await.unwrap();
    let received = tokio::time::timeout(Duration::from_secs(5), async {
        let mut seen = Vec::new();
        let mut buffer = [0u8; 1024];
        while !seen.windows(5).any(|window| window == b"hello") {
            let n = subscriber.read(&mut buffer).await.unwrap();
            assert!(n > 0, "the broker closed an honest subscriber");
            seen.extend_from_slice(&buffer[..n]);
        }
    })
    .await;
    assert!(received.is_ok(), "the broker stopped delivering after the malformed clients");
    service.jobs().stop_all();
}

/// A loopback server that keeps every request's head as it arrived.
async fn recorder() -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let heads = Arc::new(Mutex::new(Vec::new()));
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut buffer = [0u8; 4096];
                while !head.windows(4).any(|window| window == b"\r\n\r\n") {
                    match stream.read(&mut buffer).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => head.extend_from_slice(&buffer[..n]),
                    }
                }
                kept.lock().unwrap().push(String::from_utf8_lossy(&head).into_owned());
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await;
            });
        }
    });
    (port, heads)
}

/// A line break in a header, a URL or a method — typed, or brought in by a
/// template — never becomes a header of its own on the wire.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn line_breaks_in_a_request_never_reach_the_wire_as_headers() {
    let service = service();
    let (port, heads) = recorder().await;
    let injected = "a\r\nX-Injected: 1";
    let requests = [
        json!({ "method": "GET", "url": format!("http://127.0.0.1:{port}/h"), "headers": [["X-A", injected]], "body": null, "timeout_ms": 2000 }),
        json!({ "method": "GET", "url": format!("http://127.0.0.1:{port}/n"), "headers": [[injected, "b"]], "body": null, "timeout_ms": 2000 }),
        json!({ "method": "GET", "url": format!("http://127.0.0.1:{port}/u\r\nX-Injected: 1"), "headers": [], "body": null, "timeout_ms": 2000 }),
        json!({ "method": "GET /m HTTP/1.1\r\nX-Injected: 1\r\n", "url": format!("http://127.0.0.1:{port}/m"), "headers": [], "body": null, "timeout_ms": 2000 }),
        json!({ "method": "GET", "url": format!("http://127.0.0.1:{port}/a"), "headers": [], "body": null, "timeout_ms": 2000,
            "auth": { "scheme": "bearer", "token": injected } }),
        json!({ "method": "GET", "url": format!("http://127.0.0.1:{port}/b"), "headers": [], "body": null, "timeout_ms": 2000,
            "auth": { "scheme": "basic", "username": injected, "password": injected } }),
    ];
    for request in requests {
        let answer = service.invoke("http_request", json!({ "request": request })).await;
        if let Err(signal_lab_engine::Failure::Engine(error)) = &answer {
            assert!(!error.code.is_empty());
        }
    }
    // The same through an experiment: a parameter's value with a line break in a templated header.
    let document: signal_lab_engine::experiment::Experiment = serde_json::from_value(json!({
        "version": 9, "name": "Inject", "params": [{ "name": "evil", "value": injected }], "profiles": [], "profile": null, "seed": 1,
        "nodes": [
            { "id": "start", "x": 0, "y": 0, "type": "start" },
            { "id": "send", "x": 0, "y": 0, "type": "http", "request": { "method": "GET", "url": format!("http://127.0.0.1:{port}/t"),
                "headers": [["X-A", "{{evil}}"]], "body": null, "timeout_ms": 2000 } },
            { "id": "end", "x": 0, "y": 0, "type": "end" },
        ],
        "edges": [{ "from": "start", "to": "send", "port": "next" }, { "from": "send", "to": "end", "port": "next" }]
    }))
    .unwrap();
    let handle = service.run(document, signal_lab_engine::experiment_run::RunOptions { limit: Some(Duration::from_secs(10)), ..Default::default() }).await;
    if let Ok(handle) = handle {
        let result = handle.finished().await;
        assert_ne!(result.outcome, signal_lab_engine::experiment_run::Outcome::Passed, "a header that cannot be sent fails the step");
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    let heads = heads.lock().unwrap().clone();
    for head in &heads {
        assert!(!head.lines().any(|line| line.to_ascii_lowercase().starts_with("x-injected")), "a line break became a header:\n{head}");
    }
}
