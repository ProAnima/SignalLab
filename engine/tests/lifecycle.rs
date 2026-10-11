//! Jobs started, stopped and started again, as a person (or a script) does it:
//! Stop gives the port back before it answers, so a start right after binds;
//! nothing keeps sending after Stop; the registry is empty afterwards.

use std::sync::Arc;

use serde_json::{json, Value};
use signal_lab_engine::host::Recorder;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::{Capture, Host, Mode, Service};

fn service() -> (Service, Arc<Recorder>) {
    let recorder = Recorder::new();
    let host = Host::new(recorder.clone(), Capture::new());
    (Service::new(host, Mode::Server, Arc::new(FileStore::new(None))), recorder)
}

/// A free loopback port for UDP and TCP alike, as far as one can tell.
fn free_port() -> u16 {
    loop {
        let tcp = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = tcp.local_addr().unwrap().port();
        if std::net::UdpSocket::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
}

async fn start(service: &Service, command: &str, args: Value) -> u64 {
    match service.invoke(command, args.clone()).await {
        Ok(job) => job["id"].as_u64().unwrap_or_else(|| panic!("{command}: no id in {job}")),
        Err(failure) => panic!("{command} {args}: {}", serde_json::to_value(&failure).unwrap()),
    }
}

async fn stop(service: &Service, id: u64) {
    assert_eq!(service.invoke("job_stop", json!({ "id": id })).await.unwrap(), json!(true), "job {id} was running");
}

/// The commands that hold a port while they run, each on `port`.
fn binders(port: u16, target: u16) -> Vec<(&'static str, Value)> {
    let bind = format!("127.0.0.1:{port}");
    let target = format!("127.0.0.1:{target}");
    vec![
        ("osc_monitor_start", json!({ "bind": bind })),
        ("discovery_start", json!({ "config": { "bind": bind, "reuse": false } })),
        ("netsim_start", json!({ "config": { "listen": bind, "target": target, "profile": {} } })),
        ("netsim_start", json!({ "config": { "listen": bind, "target": target, "profile": {}, "protocol": "tcp" } })),
        ("emulator_start", json!({ "emulator": { "name": "h", "bind": bind, "protocol": "http", "routes": [] } })),
        ("emulator_start", json!({ "emulator": { "name": "u", "bind": bind, "protocol": "udp", "rules": [] } })),
        ("emulator_start", json!({ "emulator": { "name": "o", "bind": bind, "protocol": "osc", "rules": [] } })),
        ("emulator_start", json!({ "emulator": { "name": "t", "bind": bind, "protocol": "tcp", "rules": [] } })),
        ("emulator_start", json!({ "emulator": { "name": "m", "bind": bind, "protocol": "mqtt" } })),
    ]
}

/// Stop answers once the job has let go of its port: the same start, at once,
/// binds again — twenty times over, for every kind of job that listens.
#[tokio::test(flavor = "multi_thread")]
async fn a_stopped_job_gives_its_port_back_before_stop_answers() {
    let (service, _) = service();
    let target = free_port();
    for (command, args) in binders(free_port(), target) {
        for round in 0..20 {
            let id = start(&service, command, args.clone()).await;
            stop(&service, id).await;
            assert!(service.jobs().list().is_empty(), "{command} round {round}: still listed");
        }
        // Stop all, too.
        start(&service, command, args.clone()).await;
        service.invoke("jobs_stop_all", Value::Null).await.unwrap();
        let id = start(&service, command, args.clone()).await;
        stop(&service, id).await;
    }
}

/// The same with a client connected to each TCP listener: the connections a
/// job accepted go with it, and the port binds again at once.
#[tokio::test(flavor = "multi_thread")]
async fn a_stopped_listener_with_clients_connected_gives_its_port_back() {
    use tokio::io::AsyncReadExt;
    let (service, _) = service();
    // Something for the TCP relay to reach.
    let upstream = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target = upstream.local_addr().unwrap().port();
    let held = tokio::spawn(async move {
        let mut kept = Vec::new();
        while let Ok((stream, _)) = upstream.accept().await {
            kept.push(stream);
        }
    });
    let port = free_port();
    let tcp: Vec<_> = binders(port, target).into_iter().filter(|(_, args)| {
        let text = args.to_string();
        text.contains("\"tcp\"") || text.contains("\"http\"") || text.contains("\"mqtt\"")
    }).collect();
    assert_eq!(tcp.len(), 4);
    for (command, args) in tcp {
        for round in 0..5 {
            let id = start(&service, command, args.clone()).await;
            let mut clients = Vec::new();
            for _ in 0..3 {
                clients.push(tokio::net::TcpStream::connect(("127.0.0.1", port)).await.unwrap());
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            stop(&service, id).await;
            // Every client hears the end: EOF or a reset, not silence.
            for client in &mut clients {
                let mut buf = [0u8; 256];
                let ended = tokio::time::timeout(std::time::Duration::from_secs(3), async {
                    loop {
                        match client.read(&mut buf).await {
                            Ok(0) | Err(_) => break,
                            Ok(_) => continue,
                        }
                    }
                })
                .await;
                assert!(ended.is_ok(), "{command} {args} round {round}: a client still connected after Stop");
            }
        }
    }
    held.abort();
}

/// A run holds its waits', emulators' and relays' ports from before its first
/// step; Stop frees all of them, so Run again binds, and a stopped run sends
/// no `job://ended` (docs/api/events.md), saves no report.
#[tokio::test(flavor = "multi_thread")]
async fn a_stopped_run_gives_its_ports_back_before_stop_answers() {
    let data = std::env::temp_dir().join(format!("signallab-lifecycle-{}", std::process::id()));
    std::fs::create_dir_all(&data).unwrap();
    signal_lab_engine::paths::set_data_dir(data.clone());
    let (service, recorder) = service();
    let (emulated, relay, wait, http, target) = (free_port(), free_port(), free_port(), free_port(), free_port());
    let node = |id: &str, x: i64, mut kind: Value| {
        kind["id"] = json!(id);
        kind["x"] = json!(x);
        kind["y"] = json!(0);
        kind
    };
    let document = json!({ "version": 9, "name": "Held", "params": [], "profiles": [], "profile": null, "seed": 3,
        "nodes": [
            node("start", 0, json!({ "type": "start" })),
            node("device", 100, json!({ "type": "emulator", "emulator": { "name": "d", "bind": format!("127.0.0.1:{emulated}"), "protocol": "udp", "rules": [] } })),
            node("api", 200, json!({ "type": "emulator", "emulator": { "name": "a", "bind": format!("127.0.0.1:{http}"), "protocol": "http", "routes": [] } })),
            node("relay", 300, json!({ "type": "impairment", "listen": format!("127.0.0.1:{relay}"), "target": format!("127.0.0.1:{target}"), "profile": {} })),
            node("wait", 400, json!({ "type": "wait_osc", "bind": format!("127.0.0.1:{wait}"), "address": "/never", "timeout_ms": 60000 })),
            node("end", 500, json!({ "type": "end" })),
        ],
        "edges": [
            { "from": "start", "to": "device", "port": "next" }, { "from": "device", "to": "api", "port": "next" },
            { "from": "api", "to": "relay", "port": "next" }, { "from": "relay", "to": "wait", "port": "next" },
            { "from": "wait", "to": "end", "port": "matched" },
        ] });
    for round in 0..10 {
        let id = start(&service, "experiment_start", json!({ "document": document })).await;
        if round % 2 == 1 {
            // Let it reach the wait before it is stopped.
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        stop(&service, id).await;
        assert!(service.jobs().list().is_empty(), "round {round}: still listed");
    }
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    assert!(recorder.payloads("job://ended").is_empty(), "a stopped run ends silently");
    let reports = data.join("runs");
    assert!(!reports.exists() || std::fs::read_dir(&reports).unwrap().next().is_none(), "a stopped run saves no report");
}

/// Jobs that end on their own say so once, with no error, and leave the list.
#[tokio::test(flavor = "multi_thread")]
async fn a_job_that_ends_on_its_own_says_so_once() {
    let (service, recorder) = service();
    let (udp, _, udp_task) = udp_sink().await;
    let (tcp, _, tcp_task) = tcp_sink().await;
    let udp_target = format!("127.0.0.1:{udp}");
    let tcp_target = format!("127.0.0.1:{tcp}");
    let ending = [
        ("osc_generator_start", json!({ "config": { "target": udp_target, "address": "/x", "rate": 500, "waveform": "sine", "freq": 1, "min": 0, "max": 1, "duration_s": 0.3 } })),
        ("broadcast_beacon_start", json!({ "config": { "mode": "list", "target": udp_target, "payload": { "kind": "text", "text": "hi" }, "rate": 100, "count": 5 } })),
        ("storm_start", json!({ "config": { "target": udp_target, "protocol": "udp", "size": 64, "rate": 1000, "duration_s": 0.3 } })),
        ("storm_start", json!({ "config": { "target": tcp_target, "protocol": "tcp", "size": 64, "rate": 1000, "duration_s": 0.3 } })),
        ("http_burst_start", json!({ "config": { "method": "GET", "url": format!("http://{tcp_target}/"), "concurrency": 4, "total": 20 } })),
        ("scan_start", json!({ "config": { "host": "127.0.0.1", "port_start": tcp, "port_end": tcp, "concurrency": 1, "timeout_ms": 500 } })),
    ];
    let mut ids = Vec::new();
    for (command, args) in ending {
        ids.push((command, start(&service, command, args).await));
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !service.jobs().list().is_empty() && std::time::Instant::now() < deadline {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert!(service.jobs().list().is_empty(), "still listed: {:?}", service.jobs().list());
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let ended = recorder.payloads("job://ended");
    for (command, id) in ids {
        let mine: Vec<_> = ended.iter().filter(|event| event["job_id"] == id).collect();
        assert_eq!(mine.len(), 1, "{command}: {mine:?}");
        assert!(mine[0]["error"].is_null(), "{command}: {}", mine[0]);
    }
    udp_task.abort();
    tcp_task.abort();
}

/// A loopback UDP sink counting what reaches it.
async fn udp_sink() -> (u16, Arc<std::sync::atomic::AtomicU64>, tokio::task::JoinHandle<()>) {
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    let count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let counted = count.clone();
    let task = tokio::spawn(async move {
        let mut buf = vec![0u8; 65_536];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok(_) => {
                    counted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                Err(_) => continue,
            }
        }
    });
    (port, count, task)
}

/// A loopback TCP/HTTP sink counting bytes or requests that reach it.
async fn tcp_sink() -> (u16, Arc<std::sync::atomic::AtomicU64>, tokio::task::JoinHandle<()>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let counted = count.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let counted = counted.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                while let Ok(n) = stream.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    counted.fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                    // Answer anything that looks like an HTTP request, so a burst keeps going.
                    if buf[..n].windows(4).any(|w| w == b"\r\n\r\n") {
                        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\n\r\n").await;
                    }
                }
            });
        }
    });
    (port, count, task)
}

/// After Stop answers, nothing more reaches the target: no worker, reporter or
/// connection of the job keeps sending.
#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_sent_after_stop_answers() {
    let (service, _) = service();
    let (udp, udp_count, udp_task) = udp_sink().await;
    let (tcp, tcp_count, tcp_task) = tcp_sink().await;
    let udp_target = format!("127.0.0.1:{udp}");
    let tcp_target = format!("127.0.0.1:{tcp}");
    let senders: Vec<(&str, Value, &Arc<std::sync::atomic::AtomicU64>)> = vec![
        ("osc_generator_start", json!({ "config": { "target": udp_target, "address": "/x", "rate": 2000, "waveform": "sine", "freq": 1, "min": 0, "max": 1 } }), &udp_count),
        ("broadcast_beacon_start", json!({ "config": { "mode": "list", "target": udp_target, "payload": { "kind": "text", "text": "hi" }, "rate": 2000 } }), &udp_count),
        ("storm_start", json!({ "config": { "target": udp_target, "protocol": "udp", "size": 64, "rate": 20000 } }), &udp_count),
        ("storm_start", json!({ "config": { "target": tcp_target, "protocol": "tcp", "size": 1024, "rate": 20000 } }), &tcp_count),
        ("http_burst_start", json!({ "config": { "method": "GET", "url": format!("http://{tcp_target}/"), "concurrency": 16 } }), &tcp_count),
    ];
    for (command, args, count) in senders {
        let id = start(&service, command, args.clone()).await;
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let before = count.load(std::sync::atomic::Ordering::Relaxed);
        assert!(before > 0, "{command}: nothing arrived while it ran");
        stop(&service, id).await;
        // What was already in flight lands within a few milliseconds on loopback.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let settled = count.load(std::sync::atomic::Ordering::Relaxed);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let after = count.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(after, settled, "{command}: {} more arrived after Stop", after - settled);
        assert!(service.jobs().list().is_empty(), "{command}: still listed");
    }
    udp_task.abort();
    tcp_task.abort();
}
