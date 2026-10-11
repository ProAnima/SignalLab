//! The server's rules under attack, spoken in raw HTTP where a client library
//! would tidy the request up: Host names a rebinding page can have, origins
//! and content types a cross-site page can send, paths that try to leave the
//! data and interface folders, oversize bodies, and a client that never
//! finishes its request.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use signal_lab_server::{serve, Config};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

const TOKEN: &str = "hostile-token-0123456789abcdef0123456789";
/// What a file outside the interface and data folders holds; no response may carry it.
const PRIVATE: &str = "private-marker-5f0c1d";

fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("signallab-server-hostile-{}", std::process::id()));
        let dir = root.join("data");
        std::fs::create_dir_all(dir.join("ui").join("assets")).unwrap();
        std::fs::write(dir.join("ui").join("index.html"), "<!doctype html><title>Signal Lab</title>").unwrap();
        // Beside the data folder, and one level up from the interface: what traversal would reach.
        std::fs::write(root.join("private.txt"), PRIVATE).unwrap();
        std::fs::write(dir.join("private.txt"), PRIVATE).unwrap();
        dir
    })
    .clone()
}

struct Running {
    address: SocketAddr,
    stop: Option<oneshot::Sender<()>>,
    done: tokio::task::JoinHandle<std::io::Result<()>>,
}

async fn start(token: Option<&str>) -> Running {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let config = Config {
        listen: address,
        token: token.map(str::to_string),
        made_token: None,
        data_dir: Some(data_dir()),
        secrets_dir: data_dir().join("secrets"),
        ui_dir: Some(data_dir().join("ui")),
        allowed_hosts: vec![],
        secure_cookie: false,
    };
    let (stop, stopped) = oneshot::channel::<()>();
    let done = tokio::spawn(serve(config, listener, async move {
        let _ = stopped.await;
    }));
    Running { address, stop: Some(stop), done }
}

impl Running {
    /// Stop, and say how long serving took to return.
    async fn stop(mut self) -> Duration {
        let began = Instant::now();
        self.stop.take().unwrap().send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(60), &mut self.done).await.expect("the server stops").unwrap().unwrap();
        began.elapsed()
    }

    fn port(&self) -> u16 {
        self.address.port()
    }
}

/// One request as written, the connection closed after it; the status and the whole response.
async fn raw(address: SocketAddr, request: &str) -> (u16, String) {
    raw_bytes(address, request.as_bytes()).await
}

async fn raw_bytes(address: SocketAddr, request: &[u8]) -> (u16, String) {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(request).await.unwrap();
    let mut response = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut response)).await;
    let text = String::from_utf8_lossy(&response).into_owned();
    let status = text.split(' ').nth(1).and_then(|code| code.parse().ok()).unwrap_or(0);
    (status, text)
}

fn get(path: &str, host: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
}

fn post_json(path: &str, host: &str, extra: &str, body: &str) -> String {
    format!("POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
}

/// Without a token the server answers to loopback names only. A rebinding
/// page's name merely *starting* like a loopback address is not one:
/// `127.attacker.example` is a name anyone can point at 127.0.0.1.
#[tokio::test]
async fn host_names_that_only_look_like_loopback_are_refused() {
    let running = start(None).await;
    let port = running.port();
    for host in [format!("127.0.0.1:{port}"), format!("localhost:{port}"), format!("LOCALHOST:{port}"), format!("[::1]:{port}"), format!("app.localhost:{port}"), "127.0.0.1".into(), format!("127.1.2.3:{port}")] {
        let (status, text) = raw(running.address, &post_json("/api/invoke/app_info", &host, "", "{}")).await;
        assert_eq!(status, 200, "{host} is this machine: {text}");
    }
    for host in [
        format!("127.attacker.example:{port}"),
        format!("127.0.0.1.attacker.example:{port}"),
        "127.0.0.1.nip.io".into(),
        format!("[::1].attacker.example:{port}"),
        format!("[::1]x:{port}"),
        format!("localhost.attacker.example:{port}"),
        format!("127.0.0.1:{port}:attacker.example"),
        format!("127.0.0.1:{port}@attacker.example"),
        format!("attacker.example:{port}"),
        "".into(),
    ] {
        let (status, text) = raw(running.address, &post_json("/api/invoke/app_info", &host, "", "{}")).await;
        assert_eq!(status, 403, "{host:?} is not a loopback name: {text}");
        assert!(text.contains("auth.host"), "{text}");
    }
    // Two Host headers: whichever a proxy in front reads, the request is refused.
    let twice = format!("POST /api/invoke/app_info HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nHost: attacker.example\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}");
    assert_eq!(raw(running.address, &twice).await.0, 403, "two Host headers");
    running.stop().await;
}

/// Origins a cross-site page can send, and bodies a form can.
#[tokio::test]
async fn cross_site_requests_are_refused() {
    let running = start(None).await;
    let port = running.port();
    let host = format!("127.0.0.1:{port}");
    for origin in ["null", "http://attacker.example", &format!("http://127.0.0.1:{port}.attacker.example"), &format!("http://attacker.example/{host}"), "file://", ""] {
        let (status, text) = raw(running.address, &post_json("/api/invoke/app_info", &host, &format!("Origin: {origin}\r\n"), "{}")).await;
        assert_eq!(status, 403, "Origin {origin:?}: {text}");
    }
    let (status, _) = raw(running.address, &post_json("/api/invoke/app_info", &host, &format!("Origin: HTTP://127.0.0.1:{port}\r\n"), "{}")).await;
    assert_eq!(status, 200, "the same origin, in capitals");
    // What a <form> can send: never a command, never a run, never a sign-out.
    for kind in ["text/plain", "application/x-www-form-urlencoded", "multipart/form-data; boundary=x", "text/plain; application/json", "application/jsonp"] {
        for path in ["/api/invoke/app_info", "/api/run"] {
            let request = format!("POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: {kind}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}");
            assert_eq!(raw(running.address, &request).await.0, 415, "{path} with {kind}");
        }
    }
    let logout = format!("POST /logout HTTP/1.1\r\nHost: {host}\r\nOrigin: http://attacker.example\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    assert_eq!(raw(running.address, &logout).await.0, 403);
    // The events socket from another page: refused before the upgrade.
    let upgrade = format!(
        "GET /api/events HTTP/1.1\r\nHost: {host}\r\nOrigin: http://attacker.example\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n"
    );
    assert_eq!(raw(running.address, &upgrade).await.0, 403);
    running.stop().await;
}

/// With a token, a session id the client makes up is not a session, and a bad
/// bearer gets nothing.
#[tokio::test]
async fn sessions_cannot_be_chosen_by_the_client() {
    let running = start(Some(TOKEN)).await;
    let host = format!("127.0.0.1:{}", running.port());
    for cookie in ["signallab_session=0000", "signallab_session=", "signallab_session", "signallab_sessionX=1", &format!("signallab_session={TOKEN}")] {
        let (status, _) = raw(running.address, &post_json("/api/invoke/app_info", &host, &format!("Cookie: {cookie}\r\n"), "{}")).await;
        assert_eq!(status, 401, "{cookie}");
    }
    for bearer in [&TOKEN[..TOKEN.len() - 1], &format!("{TOKEN}x"), &TOKEN.to_uppercase(), ""] {
        let (status, _) = raw(running.address, &post_json("/api/invoke/app_info", &host, &format!("Authorization: Bearer {bearer}\r\n"), "{}")).await;
        assert_eq!(status, 401, "{bearer}");
    }
    let (status, _) = raw(running.address, &post_json("/api/invoke/app_info", &host, &format!("Authorization: Bearer {TOKEN}\r\n"), "{}")).await;
    assert_eq!(status, 200);
    running.stop().await;
}

/// Every way of naming a file outside the interface folder, through the
/// interface's own route.
#[tokio::test]
async fn the_interface_never_serves_a_file_outside_its_folder() {
    let running = start(None).await;
    let host = format!("127.0.0.1:{}", running.port());
    let paths = [
        "/../private.txt",
        "/..%2fprivate.txt",
        "/%2e%2e/private.txt",
        "/%2e%2e%2fprivate.txt",
        "/..%5cprivate.txt",
        "/assets/..%5c..%5cprivate.txt",
        "/assets/%2e%2e%5c%2e%2e%5cprivate.txt",
        "/..\\private.txt",
        "/%252e%252e/private.txt",
        "/..%c0%afprivate.txt",
        "//../private.txt",
        "/./../private.txt",
        "/assets/../../private.txt",
        "/assets/..%2f..%2f..%2fprivate.txt",
        "/C:/Windows/win.ini",
        "/C:%5cWindows%5cwin.ini",
        "/%5c%5c127.0.0.1%5cc$%5cWindows%5cwin.ini",
        "/\\\\127.0.0.1\\c$\\Windows\\win.ini",
        "/etc/passwd",
        "/index.html::$DATA",
        "/CON",
        "/assets/NUL.js",
    ];
    for path in paths {
        let (status, text) = raw(running.address, &get(path, &host)).await;
        assert!(!text.contains(PRIVATE) && !text.contains("[fonts]") && !text.contains("root:"), "{path} reached a file outside the interface: {status} {text}");
    }
    running.stop().await;
}

/// `/api/files` hands out files from the data folder, and must decide that
/// before touching anything else on the machine: a path to another
/// machine's share, a device or a named pipe is refused without being opened.
#[tokio::test]
async fn downloads_never_leave_the_data_folder_or_touch_what_is_outside() {
    let running = start(None).await;
    let host = format!("127.0.0.1:{}", running.port());
    let inside = data_dir().join("exports");
    std::fs::create_dir_all(&inside).unwrap();
    std::fs::write(inside.join("ok.json"), "{}").unwrap();
    let encode = |path: &str| path.bytes().map(|b| format!("%{b:02X}")).collect::<String>();
    let ok = raw(running.address, &get(&format!("/api/files?path={}", encode(inside.join("ok.json").to_str().unwrap())), &host)).await;
    assert_eq!(ok.0, 200, "{}", ok.1);
    let data = data_dir();
    let tries = [
        data.join("..").join("private.txt").display().to_string(),
        data.join("exports").join("..").join("..").join("private.txt").display().to_string(),
        "../private.txt".into(),
        "private.txt".into(),
        format!("{}/../private.txt", data.display()),
        format!("{}\\..\\private.txt", data.display()),
        r"\\127.0.0.1\c$\Windows\win.ini".into(),
        r"\\?\C:\Windows\win.ini".into(),
        r"\\.\C:\Windows\win.ini".into(),
        "/etc/passwd".into(),
        "C:/Windows/win.ini".into(),
        "file:///etc/passwd".into(),
        "".into(),
    ];
    for path in &tries {
        let (status, text) = raw(running.address, &get(&format!("/api/files?path={}", encode(path)), &host)).await;
        assert_eq!(status, 404, "{path}: {text}");
        assert!(!text.contains(PRIVATE) && !text.contains("[fonts]"), "{path}: {text}");
    }
    #[cfg(windows)]
    {
        // A named pipe is opened by the very act of resolving its path: the server
        // must refuse it by its name, never connect to it as a client.
        use tokio::net::windows::named_pipe::ServerOptions;
        let name = format!(r"\\.\pipe\signallab-hostile-{}", std::process::id());
        let pipe = ServerOptions::new().first_pipe_instance(true).create(&name).unwrap();
        let (status, _) = raw(running.address, &get(&format!("/api/files?path={}", encode(&name)), &host)).await;
        assert_eq!(status, 404);
        let connected = tokio::time::timeout(Duration::from_millis(300), pipe.connect()).await;
        assert!(connected.is_err(), "the download route opened a named pipe outside the data folder");
    }
    running.stop().await;
}

/// Bodies past the limit are refused, and a client that never finishes its
/// request's head neither holds a connection forever nor keeps the server
/// from stopping.
#[tokio::test]
async fn oversize_and_unfinished_requests_are_let_go() {
    let running = start(None).await;
    let host = format!("127.0.0.1:{}", running.port());
    let big = format!(
        "POST /api/invoke/experiment_parse HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        25 * 1024 * 1024
    );
    let mut stream = TcpStream::connect(running.address).await.unwrap();
    stream.write_all(big.as_bytes()).await.unwrap();
    let chunk = vec![b' '; 1024 * 1024];
    for _ in 0..25 {
        if stream.write_all(&chunk).await.is_err() {
            break;
        }
    }
    let mut response = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(10), stream.read_to_end(&mut response)).await;
    assert!(String::from_utf8_lossy(&response).starts_with("HTTP/1.1 413"), "{}", String::from_utf8_lossy(&response[..response.len().min(200)]));

    // Deeply nested JSON: refused by the parser's own limit, with the command's error.
    let deep = format!("{{\"text\":{}1{}}}", "[".repeat(100_000), "]".repeat(100_000));
    let (status, text) = raw(running.address, &post_json("/api/invoke/experiment_parse", &host, "", &deep)).await;
    assert_eq!(status, 400, "{}", &text[..text.len().min(300)]);
    assert!(text.contains("command.args_invalid"));

    // Half a request head, then silence, on several connections.
    let mut slow = Vec::new();
    for _ in 0..8 {
        let mut stream = TcpStream::connect(running.address).await.unwrap();
        stream.write_all(format!("GET / HTTP/1.1\r\nHost: {host}\r\nX-Slow: ").as_bytes()).await.unwrap();
        slow.push(stream);
    }
    tokio::time::sleep(Duration::from_millis(200)).await;
    let took = running.stop().await;
    // Requests under way get a few seconds (lib.rs); half a head is not one to wait for longer.
    assert!(took < Duration::from_secs(8), "the server took {took:?} to stop while clients held half a request");
    drop(slow);
}

/// A client that sends half a request head and nothing more is disconnected
/// after the head timeout, not kept for as long as it likes. Slow on purpose:
/// it waits out the timeout.
#[tokio::test]
async fn a_half_sent_request_head_is_cut_off() {
    let running = start(None).await;
    let mut stream = TcpStream::connect(running.address).await.unwrap();
    stream.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nX-Slow: ").await.unwrap();
    let began = Instant::now();
    let mut buffer = [0u8; 256];
    let read = tokio::time::timeout(signal_lab_server::HEAD_TIMEOUT + Duration::from_secs(5), stream.read(&mut buffer)).await;
    assert!(read.is_ok(), "the connection was still open after {:?}", began.elapsed());
    running.stop().await;
}
