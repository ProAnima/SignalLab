//! What the command-line tests share: the binary, folders, a server started in
//! the test, and loopback stand-ins for the gear an experiment talks to — an
//! HTTP API, a TCP sink, an OSC device, a UDP device and an MQTT broker.

#![allow(dead_code)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use signal_lab_engine::osc_codec::{decode_packet, encode_message, OscArg};
use signal_lab_server::{serve, Config};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{mpsc, oneshot};

pub const TOKEN: &str = "cli-test-token-0123456789abcdef0123456789";

/// A folder of the test's own.
pub fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("signallab-cli-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The binary, in English, with nothing of the environment's Signal Lab settings or CI.
pub fn command(args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_signallab"));
    command
        .args(args)
        // Never the person's own Documents/SignalLab: what defaults to the app's folder lands here.
        .env("SIGNALLAB_DATA_DIR", std::env::temp_dir().join(format!("signallab-cli-test-{}-data", std::process::id())))
        .env("SIGNALLAB_LANG", "en")
        .env_remove("SIGNALLAB_SERVER")
        .env_remove("SIGNALLAB_TOKEN")
        .env_remove("SIGNALLAB_TOKEN_FILE")
        .env_remove("GITHUB_ACTIONS");
    command
}

pub fn signallab(args: &[&str]) -> Output {
    command(args).output().unwrap()
}

/// The binary, without blocking the servers and devices this test runs.
pub async fn signallab_async(args: Vec<String>) -> Output {
    tokio::task::spawn_blocking(move || signallab(&args.iter().map(String::as_str).collect::<Vec<_>>())).await.unwrap()
}

pub fn code(output: &Output) -> i32 {
    output.status.code().unwrap()
}

pub fn out(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn err(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub struct Running {
    pub url: String,
    stop: Option<oneshot::Sender<()>>,
    done: tokio::task::JoinHandle<std::io::Result<()>>,
}

impl Running {
    pub async fn stop(mut self) {
        self.stop.take().unwrap().send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(10), &mut self.done).await.expect("the server stops").unwrap().unwrap();
    }
}

/// A server with a token, its data folder under `dir`.
pub async fn start_server(dir: &Path) -> Running {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let config = Config {
        listen: address,
        token: Some(TOKEN.into()),
        made_token: None,
        data_dir: Some(dir.to_path_buf()),
        secrets_dir: dir.join("secrets"),
        ui_dir: None,
        allowed_hosts: vec![],
        secure_cookie: false,
    };
    let (stop, stopped) = oneshot::channel::<()>();
    let done = tokio::spawn(serve(config, listener, async move {
        let _ = stopped.await;
    }));
    Running { url: format!("http://{address}"), stop: Some(stop), done }
}

// ---- the stand-ins --------------------------------------------------------------

/// What the devices saw.
#[derive(Default)]
pub struct Seen {
    pub http: AtomicUsize,
    pub tcp_bytes: AtomicUsize,
    pub osc: AtomicUsize,
    pub udp: AtomicUsize,
    pub mqtt: Mutex<Vec<String>>,
}

pub struct Gear {
    pub http: SocketAddr,
    pub tcp: SocketAddr,
    /// Answers `/ping` with `/pong 1` to the sender, then tells `osc_notify` `/status "ready"`.
    pub osc: SocketAddr,
    /// Answers any datagram with `ack <payload>` to the sender, then tells `udp_notify` `hello udp`.
    pub udp: SocketAddr,
    pub mqtt: SocketAddr,
    pub osc_notify: u16,
    pub udp_notify: u16,
    pub seen: Arc<Seen>,
}

/// The next datagram. Windows reports "port unreachable" for an earlier answer
/// (to a socket that has closed since) on the next receive; a device carries
/// on, as every receive loop of the engine does (`net::udp_transient`).
async fn receive(socket: &UdpSocket, buffer: &mut [u8]) -> Option<(usize, SocketAddr)> {
    loop {
        match socket.recv_from(buffer).await {
            Ok(got) => return Some(got),
            Err(error) if signal_lab_engine::net::udp_transient(&error) => continue,
            Err(_) => return None,
        }
    }
}

/// A free UDP port on loopback (a wait binds it itself).
pub fn free_udp_port() -> u16 {
    std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

pub async fn gear() -> Gear {
    let seen = Arc::new(Seen::default());
    let (osc_notify, udp_notify) = (free_udp_port(), free_udp_port());

    // HTTP: one JSON answer with a header of its own, for every request.
    let http = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let http_address = http.local_addr().unwrap();
    let counter = seen.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = http.accept().await {
            let counter = counter.clone();
            tokio::spawn(async move {
                let mut request = Vec::new();
                let mut buffer = [0u8; 4096];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    match stream.read(&mut buffer).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => request.extend_from_slice(&buffer[..n]),
                    }
                }
                counter.http.fetch_add(1, Ordering::SeqCst);
                let body = r#"{"state":"ready","items":[{"id":7}]}"#;
                let answer = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Lab: ready-1\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(answer.as_bytes()).await;
                let _ = stream.shutdown().await;
            });
        }
    });

    // TCP: takes whatever comes.
    let tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tcp_address = tcp.local_addr().unwrap();
    let counter = seen.clone();
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = tcp.accept().await {
            let counter = counter.clone();
            tokio::spawn(async move {
                let mut buffer = [0u8; 4096];
                while let Ok(n) = stream.read(&mut buffer).await {
                    if n == 0 {
                        break;
                    }
                    counter.tcp_bytes.fetch_add(n, Ordering::SeqCst);
                }
            });
        }
    });

    // OSC device.
    let osc = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let osc_address = osc.local_addr().unwrap();
    let counter = seen.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 2048];
        while let Some((n, from)) = receive(&osc, &mut buffer).await {
            let Ok(messages) = decode_packet(&buffer[..n]) else { continue };
            counter.osc.fetch_add(1, Ordering::SeqCst);
            if messages.iter().any(|message| message.address == "/ping") {
                let _ = osc.send_to(&encode_message("/pong", &[OscArg::Int(1)]), from).await;
                let _ = osc.send_to(&encode_message("/status", &[OscArg::Str("ready".into())]), ("127.0.0.1", osc_notify)).await;
            }
        }
    });

    // UDP device.
    let udp = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let udp_address = udp.local_addr().unwrap();
    let counter = seen.clone();
    tokio::spawn(async move {
        let mut buffer = [0u8; 2048];
        while let Some((n, from)) = receive(&udp, &mut buffer).await {
            counter.udp.fetch_add(1, Ordering::SeqCst);
            let answer = [b"ack ".as_slice(), &buffer[..n]].concat();
            let _ = udp.send_to(&answer, from).await;
            let _ = udp.send_to(b"hello udp", ("127.0.0.1", udp_notify)).await;
        }
    });

    let mqtt = broker(seen.clone()).await;
    Gear { http: http_address, tcp: tcp_address, osc: osc_address, udp: udp_address, mqtt, osc_notify, udp_notify, seen }
}

// ---- a small MQTT 3.1.1 broker: CONNECT, SUBSCRIBE, PUBLISH (QoS 0), PINGREQ ------

fn remaining(mut length: usize) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (length % 128) as u8;
        length /= 128;
        if length > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if length == 0 {
            return out;
        }
    }
}

fn publish_packet(topic: &str, payload: &[u8]) -> Vec<u8> {
    let mut body = (topic.len() as u16).to_be_bytes().to_vec();
    body.extend_from_slice(topic.as_bytes());
    body.extend_from_slice(payload);
    let mut packet = vec![0x30];
    packet.extend(remaining(body.len()));
    packet.extend(body);
    packet
}

async fn read_packet(stream: &mut tokio::net::tcp::OwnedReadHalf) -> Option<(u8, Vec<u8>)> {
    let first = stream.read_u8().await.ok()?;
    let (mut length, mut shift) = (0usize, 0);
    loop {
        let byte = stream.read_u8().await.ok()?;
        length |= ((byte & 0x7f) as usize) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            break;
        }
    }
    let mut body = vec![0u8; length];
    stream.read_exact(&mut body).await.ok()?;
    Some((first, body))
}

fn filter_matches(filter: &str, topic: &str) -> bool {
    let (filter, topic): (Vec<&str>, Vec<&str>) = (filter.split('/').collect(), topic.split('/').collect());
    for (index, level) in filter.iter().enumerate() {
        match *level {
            "#" => return true,
            "+" if index < topic.len() => {}
            level if topic.get(index) == Some(&level) => {}
            _ => return false,
        }
    }
    filter.len() == topic.len()
}

type Subscribers = Arc<Mutex<Vec<(String, mpsc::UnboundedSender<Vec<u8>>)>>>;

async fn broker(seen: Arc<Seen>) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let subscribers: Subscribers = Arc::default();
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let (mut reader, mut writer) = stream.into_split();
            let (to_client, mut outgoing) = mpsc::unbounded_channel::<Vec<u8>>();
            tokio::spawn(async move {
                while let Some(bytes) = outgoing.recv().await {
                    if writer.write_all(&bytes).await.is_err() {
                        break;
                    }
                }
            });
            let (state, seen) = (subscribers.clone(), seen.clone());
            tokio::spawn(async move {
                while let Some((first, body)) = read_packet(&mut reader).await {
                    match first >> 4 {
                        1 => drop(to_client.send(vec![0x20, 0x02, 0x00, 0x00])),
                        8 => {
                            let id = [body[0], body[1]];
                            let length = u16::from_be_bytes([body[2], body[3]]) as usize;
                            let filter = String::from_utf8_lossy(&body[4..4 + length]).to_string();
                            drop(to_client.send(vec![0x90, 0x03, id[0], id[1], 0x00]));
                            state.lock().unwrap().push((filter, to_client.clone()));
                        }
                        3 => {
                            let length = u16::from_be_bytes([body[0], body[1]]) as usize;
                            let topic = String::from_utf8_lossy(&body[2..2 + length]).to_string();
                            let payload = body[2 + length..].to_vec();
                            seen.mqtt.lock().unwrap().push(topic.clone());
                            for (filter, client) in state.lock().unwrap().iter() {
                                if filter_matches(filter, &topic) {
                                    drop(client.send(publish_packet(&topic, &payload)));
                                }
                            }
                        }
                        12 => drop(to_client.send(vec![0xd0, 0x00])),
                        14 => break,
                        _ => {}
                    }
                }
            });
        }
    });
    address
}
