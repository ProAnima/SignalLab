//! MQTT 3.1.1 client: one live connection held open as a job, plus a one-shot
//! publish for the signal library.
//!
//! The connection is a job so the console strip can list and stop it like any
//! other. Interactive work — subscribe, publish, clear a retained value — is
//! sent to that live task through a channel, keyed by job id: there is exactly
//! one socket and one place that owns it, so no locking around the wire.
//!
//! Inbound messages are batched to the UI every 100 ms. A `#` subscription on a
//! busy broker is a firehose, and a topic tree only needs the latest value per
//! topic — but dropping events silently would make the tree quietly wrong, so
//! the batch carries every message and says how many it had to shed if the
//! accumulator ever overflows.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::error::{EngineError, EngineResult};
use super::experiment_actions::host_port;
use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::transport::{self, Cause};
use super::mqtt_codec::{
    connack_reason, decode, encode_connect, encode_disconnect, encode_pingreq, encode_puback,
    encode_pubcomp, encode_publish, encode_pubrec, encode_pubrel, encode_subscribe,
    encode_unsubscribe, summarize, ConnectOpts, Packet, Will,
};

/// One subscription: a filter and the QoS we ask the broker for.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sub {
    pub filter: String,
    #[serde(default)]
    pub qos: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_keep_alive")]
    pub keep_alive_s: u16,
    #[serde(default = "default_true")]
    pub clean_session: bool,
    #[serde(default)]
    pub will: Option<Will>,
    /// Subscribed the moment the connection is up. `#` scans the whole broker.
    #[serde(default)]
    pub subscribe: Vec<Sub>,
}

fn default_keep_alive() -> u16 {
    60
}
fn default_true() -> bool {
    true
}

pub(crate) const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(6);
const FLUSH_EVERY: Duration = Duration::from_millis(100);
/// Beyond this the accumulator sheds oldest and reports the count: the alternative
/// is unbounded memory while the UI is behind.
const BATCH_CAP: usize = 4_000;

// ---------------------------------------------------------------------------
// events
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
struct MqttMessage {
    ts: u64,
    topic: String,
    /// UTF-8 as the gear sends it; lossy so a stray binary payload still shows.
    payload: String,
    bytes: usize,
    qos: u8,
    retain: bool,
    dup: bool,
}

#[derive(Clone, Serialize)]
struct MessageBatch {
    job_id: u64,
    ts: u64,
    messages: Vec<MqttMessage>,
    dropped: u64,
}

#[derive(Clone, Serialize)]
struct Grant {
    filter: String,
    qos: u8,
    accepted: bool,
}

#[derive(Clone, Serialize)]
struct StateEvent {
    job_id: u64,
    ts: u64,
    /// `connected`, `subscribed`, `closed`.
    state: &'static str,
    broker: String,
    /// Why a `closed` connection ended; none for a clean close.
    error: Option<EngineError>,
    grants: Vec<Grant>,
}

#[derive(Clone, Serialize)]
struct AckEvent {
    job_id: u64,
    ts: u64,
    /// `published` (QoS 1/2 completed), `unsubscribed`.
    kind: &'static str,
    packet_id: u16,
    topic: Option<String>,
}

fn emit_state(
    host: &Host,
    job_id: u64,
    broker: &str,
    state: &'static str,
    error: Option<EngineError>,
    grants: Vec<Grant>,
) {
    host.emit(
        "mqtt://state",
        StateEvent {
            job_id,
            ts: now_ms(),
            state,
            broker: broker.to_string(),
            error,
            grants,
        },
    );
}

// ---------------------------------------------------------------------------
// the hub: how a command reaches a live connection
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum Cmd {
    Publish {
        topic: String,
        payload: Vec<u8>,
        qos: u8,
        retain: bool,
    },
    Subscribe(Vec<Sub>),
    Unsubscribe(Vec<String>),
}

/// Live connections, by job id. Managed by Tauri so commands can find them.
#[derive(Clone, Default)]
pub struct MqttHub {
    inner: Arc<Mutex<HashMap<u64, UnboundedSender<Cmd>>>>,
}

impl MqttHub {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&self, id: u64, tx: UnboundedSender<Cmd>) {
        if let Ok(mut map) = self.inner.lock() {
            map.insert(id, tx);
        }
    }

    fn remove(&self, id: u64) {
        if let Ok(mut map) = self.inner.lock() {
            map.remove(&id);
        }
    }

    pub fn send(&self, id: u64, cmd: Cmd) -> EngineResult<()> {
        let closed = || EngineError::new("mqtt.not_connected").with("id", id);
        let map = self.inner.lock().map_err(|_| closed())?;
        let tx = map.get(&id).ok_or_else(closed)?;
        tx.send(cmd).map_err(|_| closed())
    }
}

/// Removes the connection from the hub however the task ends — including a
/// `job_stop`, which drops the future without running anything after the loop.
struct HubGuard {
    hub: MqttHub,
    id: u64,
}

impl Drop for HubGuard {
    fn drop(&mut self) {
        self.hub.remove(self.id);
    }
}

// ---------------------------------------------------------------------------
// connecting
// ---------------------------------------------------------------------------

fn connect_opts<'a>(cfg: &'a MqttConfig) -> ConnectOpts<'a> {
    ConnectOpts {
        client_id: &cfg.client_id,
        username: Some(cfg.username.as_str()).filter(|u| !u.is_empty()),
        password: Some(cfg.password.as_str()).filter(|p| !p.is_empty()),
        keep_alive_s: cfg.keep_alive_s,
        clean_session: cfg.clean_session,
        will: cfg.will.as_ref(),
    }
}

/// Why connecting or a one-shot publish failed: the cause, which becomes the
/// code a person reads, and the socket's or the broker's own wording.
#[derive(Debug)]
pub struct MqttFailure {
    pub cause: MqttCause,
    /// Technical detail (an OS error, a packet), possibly empty.
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MqttCause {
    /// The network: refused, unreachable, name not found, timed out, reset.
    Transport(Cause),
    /// CONNACK with this non-zero return code.
    Refused(u8),
    /// The socket is open but the broker did not finish the exchange in time.
    NoAnswer,
    /// The topic is empty or a filter; nothing was sent.
    Topic,
    /// Unexpected or malformed packets: probably not an MQTT broker.
    Protocol,
}

impl MqttFailure {
    fn new(cause: MqttCause, text: impl Into<String>) -> Self {
        MqttFailure { cause, text: text.into() }
    }

    fn io(error: &std::io::Error) -> Self {
        MqttFailure::new(MqttCause::Transport(transport::of_io(error)), error.to_string())
    }

    /// The failure as a person reads it, about `broker`; the text is its detail.
    pub fn error(self, broker: &str) -> EngineError {
        let error = match self.cause {
            MqttCause::Transport(cause) => cause.error(broker),
            // The return codes 3.1.1 defines each have a fix of their own.
            MqttCause::Refused(code) => match code {
                1 => EngineError::new("mqtt.refused_protocol"),
                2 => EngineError::new("mqtt.refused_client_id"),
                3 => EngineError::new("mqtt.refused_unavailable"),
                4 => EngineError::new("mqtt.refused_credentials"),
                5 => EngineError::new("mqtt.refused_not_authorized"),
                _ => EngineError::new("mqtt.refused"),
            }
            .with("broker", broker)
            .with("code", code),
            MqttCause::NoAnswer => EngineError::new("mqtt.no_answer").with("broker", broker),
            MqttCause::Topic => EngineError::new("mqtt.topic_invalid"),
            MqttCause::Protocol => EngineError::new("mqtt.protocol").with("broker", broker),
        };
        error.because(self.text)
    }
}

/// The broker a configuration dials, `host:port` (IPv6 in brackets).
pub fn broker_of(cfg: &MqttConfig) -> String {
    host_port(&cfg.host, cfg.port)
}

/// Dial and complete CONNECT/CONNACK before the job exists, so a wrong password
/// or a closed port is an error on the button rather than a job that dies a
/// moment later somewhere else.
pub(crate) async fn dial(cfg: &MqttConfig) -> Result<(TcpStream, String), MqttFailure> {
    let broker = broker_of(cfg);
    let mut stream = tokio::time::timeout(HANDSHAKE_TIMEOUT, TcpStream::connect(&broker))
        .await
        .map_err(|_| MqttFailure::new(MqttCause::Transport(Cause::Timeout), ""))?
        .map_err(|e| MqttFailure::io(&e))?;
    let _ = stream.set_nodelay(true);

    stream
        .write_all(&encode_connect(&connect_opts(cfg)))
        .await
        .map_err(|e| MqttFailure::io(&e))?;

    let mut buf = Vec::with_capacity(64);
    let mut chunk = [0u8; 512];
    let protocol = |text: String| MqttFailure::new(MqttCause::Protocol, text);
    let connack = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
        loop {
            match stream.read(&mut chunk).await {
                Ok(0) => return Err(protocol("closed the connection without a CONNACK".into())),
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(e) => return Err(MqttFailure::io(&e)),
            }
            match decode(&buf) {
                Ok(Some((Packet::ConnAck { code, .. }, _))) => return Ok(code),
                Ok(Some((other, _))) => return Err(protocol(format!("answered CONNECT with {other:?}"))),
                Ok(None) => continue,
                Err(e) => return Err(protocol(format!("malformed CONNACK: {e}"))),
            }
        }
    })
    .await
    .map_err(|_| MqttFailure::new(MqttCause::NoAnswer, ""))??;

    if connack != 0 {
        return Err(MqttFailure::new(MqttCause::Refused(connack), connack_reason(connack)));
    }
    Ok((stream, broker))
}

fn next_packet_id(counter: &mut u16) -> u16 {
    *counter = counter.checked_add(1).unwrap_or(1);
    if *counter == 0 {
        *counter = 1;
    }
    *counter
}

// ---------------------------------------------------------------------------
// the live connection
// ---------------------------------------------------------------------------

pub async fn start_client(
    host: Host,
    jobs: JobRegistry,
    hub: MqttHub,
    cfg: MqttConfig,
) -> EngineResult<JobInfo> {
    // Brokers reject an empty one, or invent one nobody can recognize.
    if cfg.client_id.trim().is_empty() {
        return Err(EngineError::new("mqtt.client_id_required"));
    }
    let (stream, broker) = dial(&cfg).await.map_err(|failure| failure.error(&broker_of(&cfg)))?;

    let id = jobs.next_id();
    let info = JobInfo::new(id, "mqtt", format!("MQTT {broker} as {}", cfg.client_id))
        .with("broker", &broker)
        .with("client", &cfg.client_id);

    let (tx, rx) = unbounded_channel();
    hub.insert(id, tx);

    let jobs_cl = jobs.clone();
    let handle = tokio::spawn(async move {
        let guard = HubGuard { hub, id };
        let error = run(&host, id, &broker, stream, cfg, rx).await;
        emit_state(&host, id, &broker, "closed", error.clone(), Vec::new());
        host.emit(
            "job://ended",
            serde_json::json!({ "job_id": id, "kind": "mqtt", "error": error }),
        );
        drop(guard);
        jobs_cl.finish(id);
    });

    jobs.insert(info.clone(), handle);
    Ok(info)
}

/// Returns the reason it ended, or `None` for a clean close.
async fn run(
    host: &Host,
    id: u64,
    broker: &str,
    stream: TcpStream,
    cfg: MqttConfig,
    mut rx: UnboundedReceiver<Cmd>,
) -> Option<EngineError> {
    let local = stream
        .local_addr()
        .map(|a| a.to_string())
        .unwrap_or_default();
    let (mut rd, mut wr) = tokio::io::split(stream);
    // Reading or writing failed: the network's cause, about the broker.
    let lost = |e: std::io::Error| Some(transport::of_io(&e).error(broker).because(e));

    emit_state(host, id, broker, "connected", None, Vec::new());

    let mut counter: u16 = 0;
    // Inbound QoS 2 ids still awaiting PUBREL: the spec's exactly-once contract
    // is what stops a redelivery from showing up twice in the tree.
    let mut pending_in: HashSet<u16> = HashSet::new();
    let mut pending_subs: HashMap<u16, Vec<Sub>> = HashMap::new();
    let mut pending_pub: HashMap<u16, String> = HashMap::new();
    let mut batch: Vec<MqttMessage> = Vec::new();
    let mut dropped: u64 = 0;
    let gate = Gate::new(200);

    if !cfg.subscribe.is_empty() {
        let pid = next_packet_id(&mut counter);
        let filters: Vec<(String, u8)> = cfg
            .subscribe
            .iter()
            .map(|s| (s.filter.clone(), s.qos))
            .collect();
        if let Err(e) = wr.write_all(&encode_subscribe(pid, &filters)).await {
            return lost(e);
        }
        pending_subs.insert(pid, cfg.subscribe.clone());
    }

    // Ping at half the keepalive, which is what every broker expects in practice.
    let ping_every = if cfg.keep_alive_s == 0 {
        Duration::from_secs(3600)
    } else {
        Duration::from_secs((cfg.keep_alive_s as u64).max(2) / 2)
    };
    let mut pinger = tokio::time::interval(ping_every);
    pinger.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut flusher = tokio::time::interval(FLUSH_EVERY);
    flusher.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut inbuf: Vec<u8> = Vec::with_capacity(8192);
    let mut chunk = [0u8; 8192];

    loop {
        tokio::select! {
            read = rd.read(&mut chunk) => {
                match read {
                    Ok(0) => return Some(Cause::Reset.error(broker)),
                    Ok(n) => inbuf.extend_from_slice(&chunk[..n]),
                    Err(e) => return lost(e),
                }
                let mut at = 0usize;
                loop {
                    match decode(&inbuf[at..]) {
                        Ok(Some((packet, used))) => {
                            at += used;
                            match packet {
                                Packet::Publish { dup, qos, retain, topic, packet_id, payload } => {
                                    // QoS 2 delivers exactly once: a redelivery
                                    // before PUBREL is an ack we owe, not a message.
                                    let fresh = qos < 2 || pending_in.insert(packet_id);
                                    if fresh {
                                        if batch.len() >= BATCH_CAP {
                                            batch.remove(0);
                                            dropped += 1;
                                        }
                                        batch.push(MqttMessage {
                                            ts: now_ms(),
                                            payload: String::from_utf8_lossy(&payload).into_owned(),
                                            bytes: payload.len(),
                                            topic: topic.clone(),
                                            qos, retain, dup,
                                        });
                                        if inspect::armed(host) && gate.allow() {
                                            inspect::publish(host, Frame::rx("mqtt", "mqtt")
                                                .job(id)
                                                .local(&local)
                                                .remote(broker)
                                                .payload(&payload)
                                                .summary(summarize(&topic, &payload, qos, retain)));
                                        }
                                    }
                                    let reply = match qos {
                                        1 => Some(encode_puback(packet_id)),
                                        2 => Some(encode_pubrec(packet_id)),
                                        _ => None,
                                    };
                                    if let Some(bytes) = reply {
                                        if let Err(e) = wr.write_all(&bytes).await {
                                            return lost(e);
                                        }
                                    }
                                }
                                Packet::PubRel(pid) => {
                                    pending_in.remove(&pid);
                                    if let Err(e) = wr.write_all(&encode_pubcomp(pid)).await {
                                        return lost(e);
                                    }
                                }
                                // Our own QoS 2 publish: the broker has it, release it.
                                Packet::PubRec(pid) => {
                                    if let Err(e) = wr.write_all(&encode_pubrel(pid)).await {
                                        return lost(e);
                                    }
                                }
                                Packet::PubAck(pid) | Packet::PubComp(pid) => {
                                    let topic = pending_pub.remove(&pid);
                                    host.emit("mqtt://ack", AckEvent {
                                        job_id: id, ts: now_ms(), kind: "published",
                                        packet_id: pid, topic,
                                    });
                                }
                                Packet::SubAck { packet_id, codes } => {
                                    let asked = pending_subs.remove(&packet_id).unwrap_or_default();
                                    let grants = asked.iter().enumerate().map(|(i, s)| {
                                        let code = codes.get(i).copied().unwrap_or(0x80);
                                        Grant {
                                            filter: s.filter.clone(),
                                            qos: if code == 0x80 { 0 } else { code },
                                            accepted: code != 0x80,
                                        }
                                    }).collect();
                                    emit_state(host, id, broker, "subscribed", None, grants);
                                }
                                Packet::UnsubAck(pid) => {
                                    host.emit("mqtt://ack", AckEvent {
                                        job_id: id, ts: now_ms(), kind: "unsubscribed",
                                        packet_id: pid, topic: None,
                                    });
                                }
                                Packet::PingResp | Packet::ConnAck { .. } => {}
                            }
                        }
                        Ok(None) => break,
                        Err(e) => return Some(EngineError::new("mqtt.protocol").with("broker", broker).because(e)),
                    }
                }
                if at > 0 { inbuf.drain(..at); }
            }

            cmd = rx.recv() => {
                let cmd = cmd?;
                let bytes = match cmd {
                    Cmd::Publish { topic, payload, qos, retain } => {
                        let qos = qos.min(2);
                        let pid = if qos > 0 { next_packet_id(&mut counter) } else { 0 };
                        if qos > 0 { pending_pub.insert(pid, topic.clone()); }
                        let framed = encode_publish(&topic, &payload, qos, retain, pid, false);
                        if inspect::armed(host) {
                            inspect::publish(host, Frame::tx("mqtt", "mqtt")
                                .job(id)
                                .local(&local)
                                .remote(broker)
                                .payload(&framed)
                                .summary(summarize(&topic, &payload, qos, retain))
                                .verdict(if payload.is_empty() && retain { "clears retained" } else { "publish" }));
                        }
                        framed
                    }
                    Cmd::Subscribe(subs) => {
                        let pid = next_packet_id(&mut counter);
                        let filters: Vec<(String, u8)> =
                            subs.iter().map(|s| (s.filter.clone(), s.qos)).collect();
                        pending_subs.insert(pid, subs);
                        encode_subscribe(pid, &filters)
                    }
                    Cmd::Unsubscribe(filters) => {
                        let pid = next_packet_id(&mut counter);
                        encode_unsubscribe(pid, &filters)
                    }
                };
                if let Err(e) = wr.write_all(&bytes).await {
                    return lost(e);
                }
            }

            _ = pinger.tick() => {
                if cfg.keep_alive_s > 0 {
                    if let Err(e) = wr.write_all(&encode_pingreq()).await {
                        return lost(e);
                    }
                }
            }

            _ = flusher.tick() => {
                if !batch.is_empty() || dropped > 0 {
                    host.emit("mqtt://messages", MessageBatch {
                        job_id: id,
                        ts: now_ms(),
                        messages: std::mem::take(&mut batch),
                        dropped: std::mem::take(&mut dropped),
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// one-shot publish, for the signal library
// ---------------------------------------------------------------------------

/// Publishing to a filter is a typo that costs a connection and comes back as a
/// confusing broker-side error, so it is refused before dialling.
fn validate_publish_topic(topic: &str) -> Option<EngineError> {
    if topic.trim().is_empty() {
        return Some(EngineError::new("mqtt.topic_required"));
    }
    if topic.contains('+') || topic.contains('#') {
        return Some(EngineError::new("node.topic_wildcard"));
    }
    None
}

/// Connect, publish, wait for the acknowledgement the QoS calls for, disconnect.
///
/// A library signal has to work with nothing set up, the same way an OSC signal
/// does — so it brings its own connection rather than requiring a live one.
pub async fn publish_once(
    host: Host,
    cfg: MqttConfig,
    topic: String,
    payload: String,
    qos: u8,
    retain: bool,
) -> EngineResult<String> {
    if let Some(refused) = validate_publish_topic(&topic) {
        return Err(refused);
    }
    let broker = broker_of(&cfg);
    publish(host, cfg, topic, payload, qos, retain).await.map_err(|failure| failure.error(&broker))
}

/// `publish_once` with the cause of a failure kept.
pub async fn publish(
    host: Host,
    cfg: MqttConfig,
    topic: String,
    payload: String,
    qos: u8,
    retain: bool,
) -> Result<String, MqttFailure> {
    if validate_publish_topic(&topic).is_some() {
        return Err(MqttFailure::new(MqttCause::Topic, ""));
    }
    // Brokers evict the older client when a new one arrives with the same id, so
    // a one-shot publish that reused it would knock the live connection off the
    // broker. Kept short: 3.1.1 only guarantees 23 characters are accepted.
    static ONCE: AtomicU64 = AtomicU64::new(0);
    let mut cfg = cfg;
    let base: String = cfg.client_id.trim().chars().take(12).collect();
    cfg.client_id = format!("{base}-o{:x}", ONCE.fetch_add(1, Ordering::Relaxed) & 0xffff);

    let (mut stream, broker) = dial(&cfg).await?;
    let qos = qos.min(2);
    let bytes = payload.into_bytes();
    let framed = encode_publish(&topic, &bytes, qos, retain, 1, false);

    if inspect::armed(&host) {
        let local = stream
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();
        inspect::publish(
            &host,
            Frame::tx("mqtt", "mqtt-send")
                .local(local)
                .remote(&broker)
                .payload(&framed)
                .summary(summarize(&topic, &bytes, qos, retain))
                .verdict(if bytes.is_empty() && retain {
                    "clears retained"
                } else {
                    "one-shot"
                }),
        );
    }

    stream
        .write_all(&framed)
        .await
        .map_err(|e| MqttFailure::io(&e))?;

    // QoS 0 is done when the bytes are out; the others owe us a round trip, and
    // reporting success before it lands would defeat the point of asking for it.
    if qos > 0 {
        let mut buf = Vec::with_capacity(64);
        let mut chunk = [0u8; 512];
        let settled = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
            loop {
                match stream.read(&mut chunk).await {
                    Ok(0) => {
                        let text = "closed the connection before acknowledging the publish";
                        return Err(MqttFailure::new(MqttCause::Transport(Cause::Reset), text));
                    }
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    Err(e) => return Err(MqttFailure::io(&e)),
                }
                let mut at = 0usize;
                loop {
                    match decode(&buf[at..]) {
                        Ok(Some((packet, used))) => {
                            at += used;
                            match packet {
                                Packet::PubAck(_) if qos == 1 => return Ok(()),
                                Packet::PubRec(pid) => {
                                    stream
                                        .write_all(&encode_pubrel(pid))
                                        .await
                                        .map_err(|e| MqttFailure::io(&e))?;
                                }
                                Packet::PubComp(_) => return Ok(()),
                                _ => {}
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            return Err(MqttFailure::new(MqttCause::Protocol, format!("malformed acknowledgement: {e}")))
                        }
                    }
                }
                if at > 0 {
                    buf.drain(..at);
                }
            }
        })
        .await
        .map_err(|_| MqttFailure::new(MqttCause::NoAnswer, "the publish was never acknowledged"))?;
        settled?;
    }

    let _ = stream.write_all(&encode_disconnect()).await;
    Ok(format!(
        "{topic} → {broker} · {} B · qos{qos}{}",
        bytes.len(),
        if retain { " retained" } else { "" }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_ids_never_land_on_zero() {
        let mut counter = u16::MAX - 1;
        assert_eq!(next_packet_id(&mut counter), u16::MAX);
        assert_eq!(next_packet_id(&mut counter), 1, "must wrap past zero");
    }

    #[test]
    fn credentials_are_only_sent_when_present() {
        let cfg = MqttConfig {
            host: "127.0.0.1".into(),
            port: 1883,
            client_id: "c".into(),
            username: String::new(),
            password: "ignored".into(),
            keep_alive_s: 60,
            clean_session: true,
            will: None,
            subscribe: Vec::new(),
        };
        let opts = connect_opts(&cfg);
        assert!(opts.username.is_none(), "an empty username must not be sent");
        assert!(opts.password.is_some(), "the codec decides, and it drops a lone password");
        let bytes = encode_connect(&opts);
        assert!(!String::from_utf8_lossy(&bytes).contains("ignored"));
    }

    #[test]
    fn failures_become_codes_about_the_broker() {
        let refused = |code| MqttFailure::new(MqttCause::Refused(code), connack_reason(code)).error("b:1883");
        let credentials = refused(4);
        assert_eq!((credentials.code.as_str(), credentials.params["broker"].as_str(), credentials.params["code"].as_str()), ("mqtt.refused_credentials", "b:1883", "4"));
        assert_eq!(credentials.detail.as_deref(), Some("bad username or password"));
        let codes: Vec<String> = [1, 2, 3, 5, 9].into_iter().map(|code| refused(code).into_code()).collect();
        assert_eq!(codes, ["mqtt.refused_protocol", "mqtt.refused_client_id", "mqtt.refused_unavailable", "mqtt.refused_not_authorized", "mqtt.refused"]);
        let lost = MqttFailure::io(&std::io::Error::from(std::io::ErrorKind::ConnectionRefused)).error("b:1883");
        assert_eq!((lost.code.as_str(), lost.params["target"].as_str()), ("transport.refused", "b:1883"));
        assert_eq!(MqttFailure::new(MqttCause::NoAnswer, "").error("b:1883").detail, None, "no text, no detail");
        assert_eq!(broker_of(&MqttConfig { host: " ::1 ".into(), ..config() }), "[::1]:1883");
    }

    #[tokio::test]
    async fn a_closed_port_and_a_missing_client_id_are_refused_before_any_job() {
        let host = Host::new(crate::host::Recorder::new(), crate::inspect::Capture::new());
        let jobs = JobRegistry::new();
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let closed = MqttConfig { port, ..config() };
        let error = start_client(host.clone(), jobs.clone(), MqttHub::new(), closed.clone()).await.unwrap_err();
        assert_eq!((error.code.as_str(), error.params["target"].clone()), ("transport.refused", format!("127.0.0.1:{port}")));
        let nameless = MqttConfig { client_id: "  ".into(), ..closed.clone() };
        assert!(start_client(host.clone(), jobs.clone(), MqttHub::new(), nameless).await.unwrap_err().is("mqtt.client_id_required"));
        let topic = publish_once(host.clone(), closed.clone(), "a/#".into(), String::new(), 0, false).await.unwrap_err();
        assert!(topic.is("node.topic_wildcard"), "refused before dialling: {topic}");
        let publish = publish_once(host, closed, "a/b".into(), String::new(), 0, false).await.unwrap_err();
        assert!(publish.is("transport.refused"), "{publish}");
        assert!(jobs.list().is_empty());
    }

    fn config() -> MqttConfig {
        MqttConfig {
            host: "127.0.0.1".into(),
            port: 1883,
            client_id: "lab".into(),
            username: String::new(),
            password: String::new(),
            keep_alive_s: 60,
            clean_session: true,
            will: None,
            subscribe: Vec::new(),
        }
    }

    #[test]
    fn a_wildcard_cannot_be_published_to() {
        let refused = |topic: &str| validate_publish_topic(topic).map(EngineError::into_code);
        assert_eq!(refused("zone/+/command").as_deref(), Some("node.topic_wildcard"));
        assert_eq!(refused("global/#").as_deref(), Some("node.topic_wildcard"));
        assert_eq!(refused("   ").as_deref(), Some("mqtt.topic_required"));
        assert!(validate_publish_topic("site/device/command/restart").is_none());
        // A retained value is cleared by publishing an empty payload to a real
        // topic, so an empty *payload* must stay legal.
        assert!(validate_publish_topic("site/device/status/online").is_none());
    }
}
