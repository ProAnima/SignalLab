//! MQTT 3.1.1 client: one live connection held open as a job. Dialling and
//! the one-shot publish the signal library uses are `mqtt_dial.rs`.
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
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use crate::host::Host;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use super::error::{EngineError, EngineResult};
use super::inspect::{self, Frame, Gate};
use super::jobs::{now_ms, JobInfo, JobRegistry};
use super::mqtt_codec::{
    decode, encode_pingreq, encode_puback, encode_pubcomp, encode_publish, encode_pubrec, encode_pubrel, encode_subscribe,
    encode_unsubscribe, summarize, Packet, Will,
};
use super::mqtt_dial::{broker_of, dial};
use super::transport::{self, Cause};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mqtt_dial::publish_once;

    #[test]
    fn packet_ids_never_land_on_zero() {
        let mut counter = u16::MAX - 1;
        assert_eq!(next_packet_id(&mut counter), u16::MAX);
        assert_eq!(next_packet_id(&mut counter), 1, "must wrap past zero");
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
}
