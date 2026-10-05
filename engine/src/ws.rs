//! WebSocket client: the WebSocket screen's connection as a job, the
//! connections an experiment opens with *WebSocket connect*, and the one-shot
//! exchange `signallab send ws` makes.
//!
//! One task owns each socket (as with MQTT): sends and a close reach it
//! through a channel, what arrives goes to a `Sink` — the screen's batch, a
//! run's inbox — and to the Inspector. Dropping the last handle closes the
//! connection with a proper close handshake, so a run that ends or is stopped
//! says goodbye instead of cutting the line.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, Notify};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderName, HeaderValue};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, WebSocketConfig};
use tokio_tungstenite::tungstenite::error::ProtocolError;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::host::Host;

use super::error::{EngineError, EngineResult, Field};
use super::inspect::{self, Budget, Frame};
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::listen::{Inbox, WaitOutcome};
use super::matching::{self, Datagram, Matcher, UdpMatcher, UdpMode};
use super::transport::{self, Cause};

/// The largest message received or sent; a server sending more loses the connection
/// (`ws.message_too_large`), a message to send that is larger is refused
/// (`node.too_long`, as the *WebSocket send* node's field is).
pub const MAX_MESSAGE: usize = 16 << 20;
/// A write that takes longer — the peer stopped reading, its window is full — ends
/// the connection instead of stalling it, and every step waiting on it, forever.
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);
/// Hex a screen message carries of a binary one.
const SHOWN_HEX: usize = 4096;
/// A close frame's reason: 125 bytes less the two of the code.
pub const MAX_CLOSE_REASON: usize = 123;
/// How long a close handshake may take before the line is just dropped.
const CLOSE_GRACE: Duration = Duration::from_secs(2);
const FLUSH_EVERY: Duration = Duration::from_millis(100);
/// Inspector frames one connection draws per second at most.
const FRAMES_PER_SECOND: u64 = 200;
/// Messages one screen batch carries; beyond it the oldest are shed and counted.
const BATCH_CAP: usize = 2_000;
/// Text a screen message carries; the rest is in the Inspector.
const SHOWN_TEXT: usize = 64 * 1024;

fn default_timeout() -> u64 {
    10_000
}

/// Where and how to connect.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WsConfig {
    /// `ws://` or `wss://`.
    pub url: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// Subprotocols to offer, in order of preference (`Sec-WebSocket-Protocol`).
    #[serde(default)]
    pub protocols: Vec<String>,
    /// For the TCP connection, TLS and the upgrade together.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

/// A message to send: text, or bytes written as hex (`de ad be ef`).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WsPayload {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub hex: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outgoing {
    Text(String),
    Binary(Vec<u8>),
}

impl Outgoing {
    pub fn len(&self) -> usize {
        match self {
            Outgoing::Text(text) => text.len(),
            Outgoing::Binary(bytes) => bytes.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Within `MAX_MESSAGE`: what the screen, a node or an exchange sends.
    pub fn check(&self) -> EngineResult<()> {
        if self.len() > MAX_MESSAGE {
            return Err(EngineError::new("node.too_long").with("max", MAX_MESSAGE).in_field(Field::new("payload")));
        }
        Ok(())
    }

    /// `text` as it is, or as hex bytes when `binary`.
    pub fn of(text: &str, binary: bool) -> EngineResult<Outgoing> {
        if binary {
            matching::parse_hex(text).map(Outgoing::Binary)
        } else {
            Ok(Outgoing::Text(text.to_string()))
        }
    }
}

impl WsPayload {
    pub fn outgoing(&self) -> EngineResult<Outgoing> {
        match (&self.text, &self.hex) {
            (Some(text), None) => Ok(Outgoing::Text(text.clone())),
            (None, Some(hex)) => matching::parse_hex(hex).map(Outgoing::Binary).map_err(|error| error.in_field(Field::new("hex"))),
            _ => Err(EngineError::new("ws.payload_required")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Text,
    Binary,
}

/// A message that arrived.
#[derive(Clone, Debug)]
pub struct Received {
    pub kind: Kind,
    pub bytes: Vec<u8>,
    pub at: Instant,
    /// Its Inspector frame, when capture was armed.
    pub frame: Option<u64>,
}

/// Who ended a connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum By {
    /// We sent the close frame.
    Client,
    /// The server sent it.
    Server,
    /// Nobody: the line broke.
    Lost,
}

#[derive(Clone, Debug, Serialize)]
pub struct Closed {
    /// 1005 when the close frame carried none, 1006 when there was no close frame.
    pub code: u16,
    pub reason: String,
    pub by: By,
    /// Why a lost connection was lost.
    pub error: Option<EngineError>,
}

/// What a connection reports as it lives. Called from the owning task: must not block.
pub trait Sink: Send + Sync + 'static {
    fn received(&self, message: &Received);
    fn sent(&self, _kind: Kind, _bytes: &[u8]) {}
    fn closed(&self, closed: &Closed);
}

/// How the upgrade went.
#[derive(Clone, Debug, Serialize)]
pub struct Handshake {
    pub url: String,
    pub peer: String,
    pub local: String,
    /// The subprotocol the server chose, if any.
    pub protocol: Option<String>,
    /// From the TCP connection to the end of the upgrade.
    pub ms: u64,
}

type Stream = WebSocketStream<MaybeTlsStream<TcpStream>>;

fn invalid_url(url: &str) -> EngineError {
    EngineError::new("ws.url_invalid").with("url", url).in_field(Field::new("url"))
}

/// `host:port` of a `ws://` / `wss://` URL, its default port filled in.
fn authority(url: &str) -> EngineResult<String> {
    let (default_port, rest) = if let Some(rest) = url.strip_prefix("ws://") {
        (80, rest)
    } else if let Some(rest) = url.strip_prefix("wss://") {
        (443, rest)
    } else {
        return Err(invalid_url(url));
    };
    let host_port = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = host_port.rsplit('@').next().unwrap_or_default();
    if host_port.is_empty() {
        return Err(invalid_url(url));
    }
    // An IPv6 literal keeps its brackets; a port after them or after the host.
    let has_port = match host_port.rfind(']') {
        Some(close) => host_port[close..].contains(':'),
        None => host_port.contains(':'),
    };
    Ok(if has_port { host_port.to_string() } else { format!("{host_port}:{default_port}") })
}

/// A `ws://` or `wss://` URL with a host.
pub fn check_url(url: &str) -> EngineResult<()> {
    authority(url).map(|_| ())
}

/// A subprotocol name is an HTTP token (RFC 6455, section 4.1).
pub fn protocol_valid(protocol: &str) -> bool {
    !protocol.is_empty() && protocol.bytes().all(|byte| byte.is_ascii_graphic() && !b"()<>@,;:\\\"/[]?={}".contains(&byte))
}

/// The network's cause of a failed connect, read, write or upgrade.
fn ws_failure(error: WsError, url: &str) -> EngineError {
    match error {
        WsError::Io(error) => transport::of_io(&error).error(url).because(error),
        WsError::Tls(error) => Cause::Tls.error(url).because(error),
        WsError::Url(error) => invalid_url(url).because(error),
        WsError::Http(response) => {
            let status = response.status();
            let body = response.body().as_deref().map(|body| String::from_utf8_lossy(body).chars().take(200).collect::<String>());
            let error = EngineError::new("ws.handshake_status").with("status", status.as_u16()).with("url", url);
            match body.filter(|body| !body.trim().is_empty()) {
                Some(body) => error.because(body),
                None => error,
            }
        }
        WsError::Capacity(error) => EngineError::new("ws.message_too_large").with("max", MAX_MESSAGE).because(error),
        WsError::Protocol(ProtocolError::SecWebSocketSubProtocolError(error)) => EngineError::new("ws.subprotocol_refused").with("url", url).because(error),
        other => EngineError::new("ws.handshake_failed").with("url", url).because(other),
    }
}

/// A close frame's reason fits it: at most `MAX_CLOSE_REASON` bytes.
pub fn check_reason(reason: &str) -> EngineResult<()> {
    if reason.len() > MAX_CLOSE_REASON {
        return Err(EngineError::new("node.too_long").with("max", MAX_CLOSE_REASON).in_field(Field::new("reason")));
    }
    Ok(())
}

/// What broke an open connection: the network's cause, a reset without a
/// close frame, a message over the limit, or the server breaking the protocol
/// — never "the upgrade failed", which it did not.
fn stream_failure(error: WsError, url: &str) -> EngineError {
    match error {
        WsError::Io(error) => transport::of_io(&error).error(url).because(error),
        WsError::Tls(error) => Cause::Tls.error(url).because(error),
        WsError::ConnectionClosed | WsError::AlreadyClosed => Cause::Reset.error(url),
        WsError::Protocol(ProtocolError::ResetWithoutClosingHandshake) => Cause::Reset.error(url),
        WsError::Capacity(error) => EngineError::new("ws.message_too_large").with("max", MAX_MESSAGE).because(error),
        other => EngineError::new("ws.protocol_error").with("url", url).because(other),
    }
}

/// Connect and upgrade: name resolution, TCP, TLS for `wss://`, the HTTP upgrade.
pub async fn dial(cfg: &WsConfig) -> EngineResult<(Stream, Handshake)> {
    let url = cfg.url.trim();
    let target = authority(url)?;
    let mut request = url.into_client_request().map_err(|error| invalid_url(url).because(error))?;
    for (name, value) in &cfg.headers {
        if name.trim().is_empty() {
            continue;
        }
        let header = HeaderName::from_bytes(name.trim().as_bytes()).map_err(|_| EngineError::new("ws.header_invalid").with("name", name.trim()))?;
        let value = HeaderValue::from_str(value).map_err(|_| EngineError::new("ws.header_invalid").with("name", name.trim()))?;
        request.headers_mut().append(header, value);
    }
    let protocols: Vec<&str> = cfg.protocols.iter().map(|protocol| protocol.trim()).filter(|protocol| !protocol.is_empty()).collect();
    if let Some(protocol) = protocols.iter().find(|protocol| !protocol_valid(protocol)) {
        return Err(EngineError::new("ws.protocol_invalid").with("value", protocol).in_field(Field::new("protocols")));
    }
    if !protocols.is_empty() {
        let offered = HeaderValue::from_str(&protocols.join(", ")).map_err(|_| EngineError::new("ws.header_invalid").with("name", "Sec-WebSocket-Protocol"))?;
        request.headers_mut().insert("Sec-WebSocket-Protocol", offered);
    }
    let timeout = Duration::from_millis(cfg.timeout_ms.max(1));
    let started = Instant::now();
    let connecting = async {
        let addresses: Vec<SocketAddr> = tokio::net::lookup_host(&target).await.map_err(|error| Cause::Dns.error(url).because(error))?.collect();
        let mut last = None;
        for address in &addresses {
            match TcpStream::connect(address).await {
                Ok(stream) => {
                    let _ = stream.set_nodelay(true);
                    let (peer, local) = (stream.peer_addr().unwrap_or(*address), stream.local_addr().ok());
                    let config = WebSocketConfig::default().max_message_size(Some(MAX_MESSAGE)).max_frame_size(Some(MAX_MESSAGE));
                    let (socket, response) = tokio_tungstenite::client_async_tls_with_config(request, stream, Some(config), None).await.map_err(|error| ws_failure(error, url))?;
                    let protocol = response.headers().get("sec-websocket-protocol").and_then(|value| value.to_str().ok()).map(str::to_string);
                    return Ok((socket, peer, local, protocol));
                }
                Err(error) => last = Some(error),
            }
        }
        Err(match last {
            Some(error) => transport::of_io(&error).error(url).because(error),
            None => Cause::Dns.error(url),
        })
    };
    let (socket, peer, local, protocol) = tokio::time::timeout(timeout, connecting).await.map_err(|_| Cause::Timeout.error(url).with("ms", cfg.timeout_ms))??;
    let handshake = Handshake {
        url: url.to_string(),
        peer: peer.to_string(),
        local: local.map(|local| local.to_string()).unwrap_or_default(),
        protocol,
        ms: started.elapsed().as_millis() as u64,
    };
    Ok((socket, handshake))
}

enum Command {
    Send(Outgoing, oneshot::Sender<EngineResult<Sent>>),
    Close(u16, String, oneshot::Sender<Closed>),
}

/// A message on the wire: its size, and when it was written — what a wait for
/// its answer counts from, so nothing read before it is taken for the answer.
#[derive(Clone, Copy, Debug)]
pub struct Sent {
    pub bytes: usize,
    pub at: Instant,
}

#[derive(Default)]
struct State {
    closed: Mutex<Option<Closed>>,
    ended: Notify,
    sent: AtomicU64,
    received: AtomicU64,
}

/// A live connection. Cheap to share; the last one dropped closes it.
pub struct Connection {
    pub handshake: Handshake,
    pub peer: SocketAddr,
    commands: mpsc::UnboundedSender<Command>,
    state: Arc<State>,
}

/// What the owning task needs to publish frames.
struct Reporter {
    host: Host,
    source: &'static str,
    job: Option<u64>,
    local: String,
    remote: String,
    budget: Budget,
}

impl Reporter {
    fn frame(&self, frame: Frame) -> Option<u64> {
        let frame = frame.local(&self.local).remote(&self.remote);
        let frame = match self.job {
            Some(job) => frame.job(job),
            None => frame,
        };
        inspect::publish(&self.host, frame)
    }

    /// A data message: every one while traffic is light, at most `FRAMES_PER_SECOND`
    /// when a chatty server floods, the next one saying how many were not shown.
    fn message(&self, dir: &str, kind: Kind, bytes: &[u8]) -> Option<u64> {
        if !inspect::armed(&self.host) {
            return None;
        }
        let refused = self.budget.allow()?;
        let summary = match kind {
            Kind::Text => format!("TEXT {}", inspect::ascii_preview(bytes, 96)),
            Kind::Binary => format!("BINARY {} B {}", bytes.len(), matching::hex(bytes, 16)),
        };
        self.frame(Frame::new("ws", dir, self.source).summary(summary).payload(bytes).not_shown(refused))
    }

    fn control(&self, dir: &str, summary: String) {
        if inspect::armed(&self.host) {
            self.frame(Frame::new("ws", dir, self.source).summary(summary).size(0));
        }
    }
}

/// The connection is over: its close code, and the reason the closing side gave.
fn closed_error(code: u16, reason: &str) -> EngineError {
    let error = EngineError::new("ws.closed").with("code", code);
    if reason.is_empty() { error } else { error.because(reason) }
}

fn close_frame(code: u16, reason: &str) -> Message {
    Message::Close(Some(CloseFrame { code: CloseCode::from(code), reason: reason.to_string().into() }))
}

fn close_summary(code: u16, reason: &str) -> String {
    if reason.is_empty() { format!("CLOSE {code}") } else { format!("CLOSE {code} {reason}") }
}

impl Connection {
    /// Hand `socket` to a task of its own; what arrives goes to `sink`.
    pub fn spawn(host: &Host, socket: Stream, handshake: Handshake, sink: Arc<dyn Sink>, source: &'static str, job: Option<u64>) -> Arc<Connection> {
        let peer = handshake.peer.parse().unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
        let (commands, receiver) = mpsc::unbounded_channel();
        let state = Arc::new(State::default());
        let reporter = Reporter { host: host.clone(), source, job, local: handshake.local.clone(), remote: handshake.url.clone(), budget: Budget::new(FRAMES_PER_SECOND) };
        reporter.control("tx", format!("CONNECT {}{}", handshake.url, handshake.protocol.as_deref().map(|protocol| format!(" ({protocol})")).unwrap_or_default()));
        tokio::spawn(own(socket, receiver, sink, state.clone(), reporter, handshake.url.clone()));
        Arc::new(Connection { handshake, peer, commands, state })
    }

    /// Send `message`; one over `MAX_MESSAGE` is refused, and the connection stays open.
    pub async fn send(&self, message: Outgoing) -> EngineResult<Sent> {
        message.check()?;
        let (ack, answer) = oneshot::channel();
        self.commands.send(Command::Send(message, ack)).map_err(|_| self.gone())?;
        answer.await.map_err(|_| self.gone())?
    }

    /// Close with `code` and `reason`, and wait for the server's answer (at
    /// most `CLOSE_GRACE`). Closing a closed connection says how it ended.
    pub async fn close(&self, code: u16, reason: &str) -> Closed {
        if let Some(closed) = self.closed() {
            return closed;
        }
        let (ack, answer) = oneshot::channel();
        if self.commands.send(Command::Close(code, reason.to_string(), ack)).is_ok() {
            if let Ok(closed) = answer.await {
                return closed;
            }
        }
        self.wait_closed().await
    }

    pub fn closed(&self) -> Option<Closed> {
        self.state.closed.lock().unwrap().clone()
    }

    pub async fn wait_closed(&self) -> Closed {
        loop {
            let ended = self.state.ended.notified();
            tokio::pin!(ended);
            ended.as_mut().enable();
            if let Some(closed) = self.closed() {
                return closed;
            }
            ended.await;
        }
    }

    pub fn counts(&self) -> (u64, u64) {
        (self.state.sent.load(Ordering::Relaxed), self.state.received.load(Ordering::Relaxed))
    }

    /// The error a send or wait meets once the connection is over.
    pub fn gone(&self) -> EngineError {
        match self.closed() {
            Some(Closed { error: Some(error), by: By::Lost, .. }) => error,
            Some(closed) => closed_error(closed.code, &closed.reason),
            None => closed_error(1006, ""),
        }
    }
}

/// The owning task: reads, writes on command, closes when told or when the
/// last handle is gone, and reports how it ended.
async fn own(mut socket: Stream, mut commands: mpsc::UnboundedReceiver<Command>, sink: Arc<dyn Sink>, state: Arc<State>, reporter: Reporter, url: String) {
    // Our close frame is out: how it will read, who waits for it, and until when the server may answer.
    let mut closing: Option<(Closed, Option<oneshot::Sender<Closed>>, tokio::time::Instant)> = None;
    let (ended, ack) = loop {
        let deadline = closing.as_ref().map(|(_, _, deadline)| *deadline);
        tokio::select! {
            // What has arrived is taken before a command: a greeting already
            // there is stamped before the send it might be mistaken to answer.
            biased;
            incoming = socket.next() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    let bytes = text.as_bytes().to_vec();
                    state.received.fetch_add(1, Ordering::Relaxed);
                    let frame = reporter.message("rx", Kind::Text, &bytes);
                    sink.received(&Received { kind: Kind::Text, bytes, at: Instant::now(), frame });
                }
                Some(Ok(Message::Binary(data))) => {
                    let bytes = data.to_vec();
                    state.received.fetch_add(1, Ordering::Relaxed);
                    let frame = reporter.message("rx", Kind::Binary, &bytes);
                    sink.received(&Received { kind: Kind::Binary, bytes, at: Instant::now(), frame });
                }
                // Pings are answered by the library as it reads; pongs need nothing.
                Some(Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_))) => {}
                Some(Ok(Message::Close(frame))) => {
                    let (code, reason) = frame.map(|frame| (u16::from(frame.code), frame.reason.to_string())).unwrap_or((1005, String::new()));
                    reporter.control("rx", close_summary(code, &reason));
                    // The library answers a server's close as it reads on; ours has been
                    // answered. Either way the line ends now, or soon.
                    let _ = tokio::time::timeout(CLOSE_GRACE, async { while socket.next().await.is_some() {} }).await;
                    break match closing.take() {
                        Some((closed, ack, _)) => (closed, ack),
                        None => (Closed { code, reason, by: By::Server, error: None }, None),
                    };
                }
                Some(Err(WsError::ConnectionClosed | WsError::AlreadyClosed)) | None => break match closing.take() {
                    Some((closed, ack, _)) => (closed, ack),
                    None => (Closed { code: 1006, reason: String::new(), by: By::Lost, error: Some(Cause::Reset.error(&url)) }, None),
                },
                Some(Err(error)) => break match closing.take() {
                    Some((closed, ack, _)) => (closed, ack),
                    None => (Closed { code: 1006, reason: String::new(), by: By::Lost, error: Some(stream_failure(error, &url)) }, None),
                },
            },
            command = commands.recv(), if closing.is_none() => match command {
                Some(Command::Send(message, ack)) => {
                    let (kind, bytes, frame) = match message {
                        Outgoing::Text(text) => (Kind::Text, text.as_bytes().to_vec(), Message::Text(text.into())),
                        Outgoing::Binary(bytes) => (Kind::Binary, bytes.clone(), Message::Binary(bytes.into())),
                    };
                    let written = match tokio::time::timeout(WRITE_TIMEOUT, socket.send(frame)).await {
                        Ok(written) => written.map_err(|error| stream_failure(error, &url)),
                        Err(_) => Err(Cause::Timeout.error(&url).with("ms", WRITE_TIMEOUT.as_millis())),
                    };
                    match written {
                        Ok(()) => {
                            let at = Instant::now();
                            state.sent.fetch_add(1, Ordering::Relaxed);
                            reporter.message("tx", kind, &bytes);
                            sink.sent(kind, &bytes);
                            let _ = ack.send(Ok(Sent { bytes: bytes.len(), at }));
                        }
                        Err(error) => {
                            let _ = ack.send(Err(error.clone()));
                            break (Closed { code: 1006, reason: String::new(), by: By::Lost, error: Some(error) }, None);
                        }
                    }
                }
                Some(Command::Close(code, reason, ack)) => {
                    reporter.control("tx", close_summary(code, &reason));
                    // A peer that reads nothing gets no close frame either: hang up.
                    if tokio::time::timeout(CLOSE_GRACE, socket.send(close_frame(code, &reason))).await.is_err() {
                        break (Closed { code, reason, by: By::Client, error: None }, Some(ack));
                    }
                    closing = Some((Closed { code, reason, by: By::Client, error: None }, Some(ack), tokio::time::Instant::now() + CLOSE_GRACE));
                }
                // Every handle is gone: the run ended, the job stopped. Say goodbye.
                None => {
                    reporter.control("tx", close_summary(1000, ""));
                    if tokio::time::timeout(CLOSE_GRACE, socket.send(close_frame(1000, ""))).await.is_err() {
                        break (Closed { code: 1000, reason: String::new(), by: By::Client, error: None }, None);
                    }
                    closing = Some((Closed { code: 1000, reason: String::new(), by: By::Client, error: None }, None, tokio::time::Instant::now() + CLOSE_GRACE));
                }
            },
            // The server never answered our close: hang up anyway.
            _ = tokio::time::sleep_until(deadline.unwrap_or_else(tokio::time::Instant::now)), if deadline.is_some() => {
                let (closed, ack, _) = closing.take().expect("a deadline means closing");
                break (closed, ack);
            }
        }
    };
    // Recorded before anyone hears of it, so `closed()` already says so.
    *state.closed.lock().unwrap() = Some(ended.clone());
    sink.closed(&ended);
    state.ended.notify_waiters();
    if let Some(ack) = ack {
        let _ = ack.send(ended);
    }
}

// ---------------------------------------------------------------------------
// a run's connections: what arrives is kept for its waits
// ---------------------------------------------------------------------------

/// Fills an inbox, as a UDP socket or an MQTT subscription does for their waits.
pub struct InboxSink {
    pub inbox: Inbox,
    peer: SocketAddr,
}

impl InboxSink {
    pub fn new(peer: SocketAddr) -> Arc<InboxSink> {
        Arc::new(InboxSink { inbox: Inbox::default(), peer })
    }
}

impl Sink for InboxSink {
    fn received(&self, message: &Received) {
        self.inbox.push(Datagram { bytes: message.bytes.clone(), from: self.peer, at: message.at, topic: None, frame: message.frame, request: None, binary: message.kind == Kind::Binary });
    }

    fn closed(&self, closed: &Closed) {
        let error = match &closed.error {
            Some(error) if closed.by == By::Lost => error.clone(),
            _ => closed_error(closed.code, &closed.reason),
        };
        self.inbox.fail(error);
    }
}

/// An open connection of a run: the socket and what it received.
pub struct RunSocket {
    pub connection: Arc<Connection>,
    pub sink: Arc<InboxSink>,
}

/// Open a connection whose messages a run's waits read.
pub async fn open(host: &Host, cfg: &WsConfig, source: &'static str, job: Option<u64>) -> EngineResult<RunSocket> {
    let (socket, handshake) = dial(cfg).await?;
    let peer = handshake.peer.parse().unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 0)));
    let sink = InboxSink::new(peer);
    let connection = Connection::spawn(host, socket, handshake, sink.clone(), source, job);
    Ok(RunSocket { connection, sink })
}

/// Matches like *Wait for UDP*, and adds `json` (a text message parsed, or
/// null) and `kind` (`text` or `binary`).
pub struct WsMatcher(pub UdpMatcher);

impl Matcher for WsMatcher {
    fn matches(&self, datagram: &Datagram) -> Option<Value> {
        let mut reply = self.0.matches(datagram)?;
        reply["json"] = std::str::from_utf8(&datagram.bytes).ok().filter(|_| !datagram.binary).and_then(|text| serde_json::from_str(text).ok()).unwrap_or(Value::Null);
        reply["kind"] = Value::String(if datagram.binary { "binary" } else { "text" }.into());
        Some(reply)
    }
}

// ---------------------------------------------------------------------------
// the screen's connection: a job, its messages batched to the interface
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
struct ScreenMessage {
    ts: u64,
    /// `rx` or `tx`.
    dir: &'static str,
    kind: Kind,
    /// UTF-8 for text (lossy for binary, which also has `hex`), cut at `SHOWN_TEXT`.
    text: String,
    hex: Option<String>,
    bytes: usize,
    truncated: bool,
}

#[derive(Default)]
struct Batch {
    messages: std::collections::VecDeque<ScreenMessage>,
    dropped: u64,
}

struct BatchSink(Mutex<Batch>);

impl BatchSink {
    fn push(&self, dir: &'static str, kind: Kind, bytes: &[u8]) {
        let truncated = bytes.len() > if kind == Kind::Binary { SHOWN_HEX } else { SHOWN_TEXT };
        let shown = &bytes[..bytes.len().min(SHOWN_TEXT)];
        let message = ScreenMessage {
            ts: now_ms(),
            dir,
            kind,
            text: String::from_utf8_lossy(shown).into_owned(),
            hex: (kind == Kind::Binary).then(|| matching::hex(bytes, SHOWN_HEX)),
            bytes: bytes.len(),
            truncated,
        };
        let mut batch = self.0.lock().unwrap();
        if batch.messages.len() == BATCH_CAP {
            batch.messages.pop_front();
            batch.dropped += 1;
        }
        batch.messages.push_back(message);
    }

    fn take(&self) -> Batch {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

impl Sink for BatchSink {
    fn received(&self, message: &Received) {
        self.push("rx", message.kind, &message.bytes);
    }

    fn sent(&self, kind: Kind, bytes: &[u8]) {
        self.push("tx", kind, bytes);
    }

    fn closed(&self, _closed: &Closed) {}
}

#[derive(Clone, Serialize)]
struct MessageBatch {
    job_id: u64,
    ts: u64,
    messages: Vec<ScreenMessage>,
    dropped: u64,
}

#[derive(Clone, Serialize)]
struct StateEvent {
    job_id: u64,
    ts: u64,
    /// `connected` or `closed`.
    state: &'static str,
    handshake: Handshake,
    closed: Option<Closed>,
}

/// The screen's live connections, by job id.
#[derive(Clone, Default)]
pub struct WsHub {
    inner: Arc<Mutex<std::collections::HashMap<u64, Arc<Connection>>>>,
}

impl WsHub {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, id: u64) -> EngineResult<Arc<Connection>> {
        self.inner.lock().unwrap().get(&id).cloned().ok_or_else(|| EngineError::new("ws.not_connected").with("id", id))
    }

    pub async fn send(&self, id: u64, payload: &WsPayload) -> EngineResult<usize> {
        let message = payload.outgoing()?;
        Ok(self.get(id)?.send(message).await?.bytes)
    }

    pub async fn close(&self, id: u64, code: Option<u16>, reason: Option<String>) -> EngineResult<Closed> {
        let code = code.unwrap_or(1000);
        check_close_code(code)?;
        check_reason(reason.as_deref().unwrap_or_default())?;
        Ok(self.get(id)?.close(code, reason.as_deref().unwrap_or_default()).await)
    }
}

/// Removes the connection from the hub however the job ends; the last handle
/// dropped closes it.
struct HubGuard {
    hub: WsHub,
    id: u64,
}

impl Drop for HubGuard {
    fn drop(&mut self) {
        self.hub.inner.lock().unwrap().remove(&self.id);
    }
}

/// A close code a client may send: 1000, or 3000–4999 for an application's own.
pub fn check_close_code(code: u16) -> EngineResult<()> {
    if code == 1000 || (3000..=4999).contains(&code) {
        Ok(())
    } else {
        Err(EngineError::new("ws.close_code").with("code", code).in_field(Field::new("code")))
    }
}

pub async fn start_client(host: Host, jobs: JobRegistry, hub: WsHub, cfg: WsConfig) -> EngineResult<JobInfo> {
    let (socket, handshake) = dial(&cfg).await?;
    let id = jobs.next_id();
    let info = JobInfo::new(id, "websocket", format!("WebSocket {}", handshake.url)).with("url", &handshake.url);
    let sink = Arc::new(BatchSink(Mutex::new(Batch::default())));
    let connection = Connection::spawn(&host, socket, handshake.clone(), sink.clone(), "websocket", Some(id));
    hub.inner.lock().unwrap().insert(id, connection.clone());

    let jobs_cl = jobs.clone();
    let handle = tokio::spawn(async move {
        let guard = HubGuard { hub, id };
        let state = |state: &'static str, closed: Option<Closed>| StateEvent { job_id: id, ts: now_ms(), state, handshake: handshake.clone(), closed };
        host.emit("ws://state", state("connected", None));
        let flush = |host: &Host| {
            let batch = sink.take();
            if !batch.messages.is_empty() || batch.dropped > 0 {
                host.emit("ws://messages", MessageBatch { job_id: id, ts: now_ms(), messages: batch.messages.into(), dropped: batch.dropped });
            }
        };
        let mut ticker = tokio::time::interval(FLUSH_EVERY);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let closed = loop {
            tokio::select! {
                _ = ticker.tick() => flush(&host),
                closed = connection.wait_closed() => break closed,
            }
        };
        flush(&host);
        let error = closed.error.clone();
        host.emit("ws://state", state("closed", Some(closed)));
        host.emit("job://ended", json!({ "job_id": id, "kind": "websocket", "error": error }));
        drop(guard);
        jobs_cl.finish(id);
    });
    jobs.insert(info.clone(), handle);
    Ok(info)
}

// ---------------------------------------------------------------------------
// one exchange: connect, send, maybe wait for an answer, close
// ---------------------------------------------------------------------------

/// What one answer looked like.
#[derive(Clone, Debug, Serialize)]
pub struct WsReply {
    pub kind: Kind,
    pub text: String,
    pub hex: String,
    pub bytes: usize,
    pub json: Value,
    /// Since the message was sent (or since connecting, when nothing was).
    pub ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Exchange {
    pub handshake: Handshake,
    pub sent: Option<usize>,
    pub reply: Option<WsReply>,
    pub closed: Closed,
}

/// What to wait for after sending: like *Wait for WebSocket*.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WsExpect {
    #[serde(default)]
    pub mode: UdpMode,
    #[serde(default)]
    pub pattern: String,
    #[serde(default = "default_expect_timeout")]
    pub timeout_ms: u64,
}

fn default_expect_timeout() -> u64 {
    2000
}

pub async fn exchange(host: &Host, cfg: &WsConfig, send: Option<&WsPayload>, expect: Option<&WsExpect>) -> EngineResult<Exchange> {
    let message = send.map(WsPayload::outgoing).transpose()?;
    let matcher = expect.map(|expect| UdpMatcher::new(expect.mode, &expect.pattern)).transpose()?;
    // An answer counts from the send; with nothing to send, from the start: a greeting.
    let connecting = Instant::now();
    let RunSocket { connection, sink } = open(host, cfg, "websocket", None).await?;
    let (since, sent) = match message {
        Some(message) => {
            let sent = connection.send(message).await?;
            (sent.at, Some(sent.bytes))
        }
        None => (connecting, None),
    };
    let reply = match (matcher, expect) {
        (Some(matcher), Some(expect)) => match sink.inbox.wait(&WsMatcher(matcher), since, Duration::from_millis(expect.timeout_ms)).await? {
            WaitOutcome::Matched(datagram, value) => Some(WsReply {
                kind: if datagram.binary { Kind::Binary } else { Kind::Text },
                text: value["text"].as_str().unwrap_or_default().to_string(),
                hex: value["hex"].as_str().unwrap_or_default().to_string(),
                bytes: datagram.bytes.len(),
                json: value["json"].clone(),
                ms: datagram.at.saturating_duration_since(since).as_millis() as u64,
            }),
            WaitOutcome::TimedOut { unmatched } => {
                connection.close(1000, "").await;
                return Err(EngineError::new("wait.timeout").with("ms", expect.timeout_ms).with("unmatched", unmatched).with("target", &cfg.url));
            }
        },
        _ => None,
    };
    let closed = connection.close(1000, "").await;
    Ok(Exchange { handshake: connection.handshake.clone(), sent, reply, closed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_name_their_host_and_port() {
        assert_eq!(authority("ws://127.0.0.1:9001/chat?x=1").unwrap(), "127.0.0.1:9001");
        assert_eq!(authority("ws://example.test/").unwrap(), "example.test:80");
        assert_eq!(authority("wss://example.test").unwrap(), "example.test:443");
        assert_eq!(authority("wss://user:pw@example.test:8443/x").unwrap(), "example.test:8443");
        assert_eq!(authority("ws://[::1]:9001/").unwrap(), "[::1]:9001");
        assert_eq!(authority("ws://[::1]/").unwrap(), "[::1]:80");
        for wrong in ["http://example.test/", "example.test:80", "ws://", "ws:///path"] {
            assert!(authority(wrong).unwrap_err().is("ws.url_invalid"), "{wrong}");
        }
    }

    #[test]
    fn subprotocols_are_tokens() {
        for good in ["graphql-transport-ws", "v2.json", "mqtt", "x_y"] {
            assert!(protocol_valid(good), "{good}");
        }
        for bad in ["", "a b", "a,b", "chat/1", "ünï"] {
            assert!(!protocol_valid(bad), "{bad}");
        }
    }

    #[test]
    fn a_payload_is_text_or_hex_and_close_codes_are_a_clients() {
        assert_eq!(WsPayload { text: Some("hi".into()), hex: None }.outgoing().unwrap(), Outgoing::Text("hi".into()));
        assert_eq!(WsPayload { text: None, hex: Some("de ad".into()) }.outgoing().unwrap(), Outgoing::Binary(vec![0xde, 0xad]));
        assert!(WsPayload::default().outgoing().unwrap_err().is("ws.payload_required"));
        assert!(WsPayload { text: None, hex: Some("xyz".into()) }.outgoing().unwrap_err().is("hex.invalid"));
        assert_eq!(Outgoing::of("01 02", true).unwrap(), Outgoing::Binary(vec![1, 2]));
        assert!(Outgoing::Text("x".repeat(MAX_MESSAGE)).check().is_ok(), "the limit itself is sent");
        let over = Outgoing::Binary(vec![0; MAX_MESSAGE + 1]).check().unwrap_err();
        assert_eq!((over.code.as_str(), over.params["max"].as_str(), over.field.as_ref().unwrap().key.as_str()), ("node.too_long", "16777216", "payload"));
        for code in [1000, 3000, 4999] {
            assert!(check_close_code(code).is_ok());
        }
        for code in [999, 1001, 1006, 2999, 5000] {
            assert!(check_close_code(code).unwrap_err().is("ws.close_code"), "{code}");
        }
    }
}
