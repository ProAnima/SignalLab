//! The MQTT side of emulators: a small MQTT 3.1.1 broker on plain TCP. It
//! does what a broker does — clients connect (with the user name and password
//! it asks for, if any), subscribe with wildcards, publish at QoS 0, 1 and 2,
//! retained messages and last wills, a second connection with a client's id
//! takes over from the first — and, like the other emulators, answers what is
//! published to it by rules: on a message to this filter, publish that (a
//! device reporting its new state).
//!
//! Clean sessions only, as Signal Lab's own client: a client asking to keep
//! its session gets a fresh one, and nothing is queued for a client that is
//! away. While its outage has it down, the broker drops every connection and
//! refuses new ones with CONNACK 3 (server unavailable).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Notify};
use tokio::task::JoinSet;

use super::emulator::MqttOut;
use super::emulator_rules::{MqttRules, Rules};
use super::emulator_state::{kept, Context, Exchange};
use super::error::{EngineError, EngineResult, Field};
use super::inspect::Frame;
use super::matching::{self, Datagram, Matcher};
use super::mqtt_codec::{
    decode_client, encode_connack, encode_pingresp, encode_puback, encode_pubcomp, encode_publish, encode_pubrec, encode_pubrel, encode_suback,
    encode_unsuback, summarize, ClientPacket, Connect,
};
use super::subscribe::{filter_valid, topic_matches, topic_name_valid};

/// Clients connected at once; more are closed as they arrive.
const MAX_CONNECTIONS: usize = 256;
/// The longest packet a client may send; a longer one ends its connection.
const MAX_PACKET: usize = 256 * 1024;
/// Subscriptions one client may hold; more are refused (0x80).
const MAX_SUBSCRIPTIONS: usize = 100;
/// Topics the broker retains, and their payloads' bytes; past either, a new
/// retained message is routed, not retained.
const MAX_RETAINED: usize = 1000;
const MAX_RETAINED_BYTES: usize = 16 * 1024 * 1024;
/// Messages, and their payloads' bytes, waiting for one slow client; past
/// either it misses them, and they are counted as `missed`.
const OUTBOX: usize = 1024;
const OUTBOX_BYTES: usize = 8 * 1024 * 1024;
/// A client that takes no bytes for this long while the broker writes to it is gone.
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
/// A client must send its CONNECT within this.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// How often a connection checks its keep-alive and the outage.
const TICK: Duration = Duration::from_millis(100);
const ACCEPT_FAILURES: u32 = 50;
/// Rule replies waiting for their delay at once; more are dropped and counted.
const MAX_PENDING: usize = 1024;

/// A message on its way to subscribers.
struct Message {
    topic: String,
    payload: Vec<u8>,
    qos: u8,
}

struct Session {
    /// Filters and the QoS each was granted.
    subscriptions: Vec<(String, u8)>,
    outbox: mpsc::Sender<(Arc<Message>, u8)>,
    /// The payload bytes waiting in `outbox`.
    queued: Arc<AtomicUsize>,
    /// Told when another connection takes over this client id.
    kick: Arc<Notify>,
    /// Told when the connection has ended and its will is out.
    gone: Arc<Notify>,
}

/// Who is connected, what they subscribed to, what is retained.
#[derive(Default)]
struct Broker {
    sessions: HashMap<u64, Session>,
    /// The connection each client id is on.
    clients: HashMap<String, u64>,
    retained: BTreeMap<String, (Vec<u8>, u8)>,
    /// The payload bytes `retained` holds.
    retained_bytes: usize,
    next: u64,
}

impl Broker {
    /// Hand `message` to every client subscribed to its topic: once each, at
    /// the highest QoS a matching subscription was granted, never above the
    /// message's own. A client too slow to take it misses it rather than
    /// holding up everyone else; how many missed it is the answer.
    fn route(&self, message: &Arc<Message>) -> u64 {
        let mut missed = 0;
        for session in self.sessions.values() {
            let granted = session.subscriptions.iter().filter(|(filter, _)| topic_matches(filter, &message.topic)).map(|(_, qos)| *qos).max();
            let Some(granted) = granted else { continue };
            let size = message.payload.len();
            // Routing holds the broker's lock, so only a delivery can lower `queued` meanwhile.
            let room = session.queued.load(Ordering::Relaxed) + size <= OUTBOX_BYTES;
            if room && session.outbox.try_send((message.clone(), granted.min(message.qos))).is_ok() {
                session.queued.fetch_add(size, Ordering::Relaxed);
            } else {
                missed += 1;
            }
        }
        missed
    }

    /// Keep `message` for whoever subscribes later; an empty payload forgets the topic.
    fn retain(&mut self, message: &Message) {
        let held = self.retained.get(&message.topic).map_or(0, |(payload, _)| payload.len());
        if message.payload.is_empty() {
            self.retained.remove(&message.topic);
            self.retained_bytes -= held;
            return;
        }
        let fits = (held > 0 || self.retained.len() < MAX_RETAINED) && self.retained_bytes - held + message.payload.len() <= MAX_RETAINED_BYTES;
        if fits {
            self.retained.insert(message.topic.clone(), (message.payload.clone(), message.qos));
            self.retained_bytes = self.retained_bytes - held + message.payload.len();
        } else if held > 0 {
            // The older value is no longer the topic's: a late subscriber must not read it.
            self.retained.remove(&message.topic);
            self.retained_bytes -= held;
        }
    }
}

/// What every connection of one broker shares.
struct Shared {
    context: Arc<Context>,
    broker: Arc<Mutex<Broker>>,
    /// Rule replies waiting for their delay; stopped with the broker.
    pending: Mutex<JoinSet<()>>,
}

/// Serve until accepting fails for good; the error says why.
pub(crate) async fn serve(listener: TcpListener, context: Arc<Context>) -> EngineError {
    let Rules::Mqtt(rules) = &context.compiled.rules else { return EngineError::new("emulator.failed") };
    let mut broker = Broker::default();
    for (topic, payload, qos) in &rules.retained {
        broker.retain(&Message { topic: topic.clone(), payload: payload.clone(), qos: *qos });
    }
    let local = context.emulation.local;
    let shared = Arc::new(Shared { context, broker: Arc::new(Mutex::new(broker)), pending: Mutex::new(JoinSet::new()) });
    let active = Arc::new(AtomicUsize::new(0));
    // Owned here: when the broker stops, every connection stops with it.
    let mut connections = JoinSet::new();
    let mut failures = 0;
    loop {
        while connections.try_join_next().is_some() {}
        let (stream, peer) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(error) => {
                failures += 1;
                if failures >= ACCEPT_FAILURES {
                    return EngineError::new("emulator.accept_failed").with("target", local).because(error);
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
                continue;
            }
        };
        failures = 0;
        if active.load(Ordering::Relaxed) >= MAX_CONNECTIONS {
            drop(stream);
            continue;
        }
        active.fetch_add(1, Ordering::Relaxed);
        let (shared, active) = (shared.clone(), active.clone());
        connections.spawn(async move {
            connection(stream, peer, &shared).await;
            active.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

/// The next whole packet, reading as needed; `None` when the client is gone.
async fn next_packet(reader: &mut OwnedReadHalf, buffer: &mut Vec<u8>, chunk: &mut [u8]) -> Option<ClientPacket> {
    loop {
        match decode_client(buffer, MAX_PACKET) {
            Ok(Some((packet, used))) => {
                buffer.drain(..used);
                return Some(packet);
            }
            Ok(None) => {}
            Err(_) => return None,
        }
        match reader.read(chunk).await {
            Ok(0) | Err(_) => return None,
            Ok(size) => buffer.extend_from_slice(&chunk[..size]),
        }
    }
}

/// The CONNACK return code for `connect`, and why a refused one was refused.
fn verdict(rules: &MqttRules, connect: &Connect, down: bool) -> (u8, Option<EngineError>) {
    if down {
        return (3, None);
    }
    if connect.protocol != "MQTT" || connect.level != 4 {
        return (1, Some(EngineError::new("emulator.mqtt_protocol").with("protocol", &connect.protocol).with("level", connect.level)));
    }
    if connect.client_id.is_empty() && !connect.clean_session {
        return (2, Some(EngineError::new("emulator.mqtt_client_id")));
    }
    if let Some((username, password)) = &rules.login {
        if connect.username.as_deref() != Some(username.as_str()) || connect.password.as_deref().unwrap_or_default() != password.as_bytes() {
            return (4, Some(EngineError::new("emulator.mqtt_login").with("client", &connect.client_id)));
        }
    }
    (0, None)
}

/// How a connection ended, which decides whether its will goes out: not
/// after DISCONNECT, and not when the broker itself goes down.
#[derive(PartialEq)]
enum Ending {
    Disconnected,
    Lost,
    Down,
}

/// What a client's packet asks of its connection.
enum Next {
    Go,
    End(Ending),
}

/// One client's connection, as a broker sees it.
struct Client<'a> {
    shared: &'a Arc<Shared>,
    id: u64,
    client_id: String,
    peer: SocketAddr,
    writer: OwnedWriteHalf,
    /// The bytes waiting in this client's outbox (`Session::queued`).
    queued: Arc<AtomicUsize>,
    /// QoS 2 packet ids received and not yet released: a duplicate is acknowledged, not delivered again.
    received: HashSet<u16>,
    packet_id: u16,
}

impl Client<'_> {
    fn next_packet_id(&mut self) -> u16 {
        self.packet_id = self.packet_id.checked_add(1).unwrap_or(1);
        self.packet_id
    }

    /// A client that stopped reading is given up after `WRITE_TIMEOUT`, so
    /// takeovers, outages and keep-alive are never held up for long.
    async fn write(&mut self, bytes: &[u8]) -> bool {
        matches!(tokio::time::timeout(WRITE_TIMEOUT, self.writer.write_all(bytes)).await, Ok(Ok(())))
    }

    /// Every whole packet `buffer` holds, handled in order — also those that
    /// arrived together with the CONNECT, before its CONNACK went out.
    async fn drain(&mut self, buffer: &mut Vec<u8>) -> Option<Ending> {
        loop {
            match decode_client(buffer, MAX_PACKET) {
                Ok(Some((packet, used))) => {
                    buffer.drain(..used);
                    if let Next::End(ending) = self.handle(packet).await {
                        return Some(ending);
                    }
                }
                Ok(None) => return None,
                Err(_) => return Some(Ending::Lost),
            }
        }
    }

    async fn send(&mut self, message: &Message, qos: u8, retain: bool) -> bool {
        let packet_id = if qos > 0 { self.next_packet_id() } else { 0 };
        let bytes = encode_publish(&message.topic, &message.payload, qos, retain, packet_id, false);
        self.write(&bytes).await
    }

    async fn handle(&mut self, packet: ClientPacket) -> Next {
        let written = match packet {
            // A second CONNECT is a protocol violation.
            ClientPacket::Connect(_) => return Next::End(Ending::Lost),
            ClientPacket::Publish { qos, retain, topic, packet_id, payload, .. } => {
                if !topic_name_valid(&topic) {
                    return Next::End(Ending::Lost);
                }
                // A QoS 2 id seen again before its PUBREL is a duplicate: acknowledged, not delivered again.
                let fresh = qos < 2 || self.received.insert(packet_id);
                if fresh {
                    published(self.shared, Message { topic, payload, qos }, retain, &self.client_id, self.peer, false);
                }
                // What fully arrived is delivered even when its acknowledgement cannot go back.
                match qos {
                    1 => self.write(&encode_puback(packet_id)).await,
                    2 => self.write(&encode_pubrec(packet_id)).await,
                    _ => true,
                }
            }
            ClientPacket::PubRel(packet_id) => {
                self.received.remove(&packet_id);
                self.write(&encode_pubcomp(packet_id)).await
            }
            // The client's side of a QoS 2 message the broker sent it.
            ClientPacket::PubRec(packet_id) => self.write(&encode_pubrel(packet_id)).await,
            ClientPacket::PubAck(_) | ClientPacket::PubComp(_) => true,
            ClientPacket::Subscribe { packet_id, filters } => return self.subscribe(packet_id, filters).await,
            ClientPacket::Unsubscribe { packet_id, filters } => {
                if let Some(session) = self.shared.broker.lock().unwrap().sessions.get_mut(&self.id) {
                    session.subscriptions.retain(|(filter, _)| !filters.contains(filter));
                }
                self.write(&encode_unsuback(packet_id)).await
            }
            ClientPacket::PingReq => self.write(&encode_pingresp()).await,
            ClientPacket::Disconnect => return Next::End(Ending::Disconnected),
        };
        if written {
            Next::Go
        } else {
            Next::End(Ending::Lost)
        }
    }

    /// Grant what can be granted, then send what is retained under it.
    async fn subscribe(&mut self, packet_id: u16, filters: Vec<(String, u8)>) -> Next {
        let mut codes = Vec::with_capacity(filters.len());
        let mut retained: BTreeMap<String, (Vec<u8>, u8)> = BTreeMap::new();
        {
            let mut guard = self.shared.broker.lock().unwrap();
            // Two fields of one broker: what it retains, and this client's session.
            let broker = &mut *guard;
            let held = &broker.retained;
            let Some(session) = broker.sessions.get_mut(&self.id) else { return Next::End(Ending::Lost) };
            for (filter, qos) in &filters {
                let known = session.subscriptions.iter().position(|(held, _)| held == filter);
                if !filter_valid(filter) || (known.is_none() && session.subscriptions.len() >= MAX_SUBSCRIPTIONS) {
                    codes.push(0x80);
                    continue;
                }
                match known {
                    Some(index) => session.subscriptions[index].1 = *qos,
                    None => session.subscriptions.push((filter.clone(), *qos)),
                }
                codes.push(*qos);
                for (topic, (payload, stored)) in held.iter().filter(|(topic, _)| topic_matches(filter, topic)) {
                    let qos = (*stored).min(*qos);
                    let entry = retained.entry(topic.clone()).or_insert_with(|| (payload.clone(), qos));
                    entry.1 = entry.1.max(qos);
                }
            }
        }
        let context = &self.shared.context;
        if context.capturing() {
            let named: Vec<&str> = filters.iter().map(|(filter, _)| filter.as_str()).collect();
            context.publish(Frame::rx("mqtt", "emulator").remote(self.peer).summary(format!("SUBSCRIBE {}", named.join(", "))).verdict(format!("{codes:?}")));
        }
        if !self.write(&encode_suback(packet_id, &codes)).await {
            return Next::End(Ending::Lost);
        }
        for (topic, (payload, qos)) in retained {
            if !self.send(&Message { topic, payload, qos }, qos, true).await {
                return Next::End(Ending::Lost);
            }
        }
        Next::Go
    }
}

async fn connection(stream: TcpStream, peer: SocketAddr, shared: &Arc<Shared>) {
    let context = &shared.context;
    let Rules::Mqtt(rules) = &context.compiled.rules else { return };
    let _ = stream.set_nodelay(true);
    let (mut reader, mut writer) = stream.into_split();
    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 8192];
    let Ok(Some(ClientPacket::Connect(connect))) = tokio::time::timeout(CONNECT_TIMEOUT, next_packet(&mut reader, &mut buffer, &mut chunk)).await else { return };
    // A will it could never publish is a malformed CONNECT: the connection is closed.
    if connect.will.as_ref().is_some_and(|will| !topic_name_valid(&will.topic)) {
        return;
    }
    let request = format!("CONNECT {}", connect.client_id).trim_end().to_string();
    let down = context.down().is_some();
    let (code, refusal) = verdict(rules, &connect, down);
    if code != 0 {
        let _ = tokio::time::timeout(WRITE_TIMEOUT, async {
            let _ = writer.write_all(&encode_connack(false, code)).await;
            let _ = writer.shutdown().await;
        })
        .await;
        context.emulation.record(Exchange { from: peer.to_string(), request, reply: format!("CONNACK {code}"), down, error: refusal, ..Default::default() });
        return;
    }
    // An empty id with a clean session is the broker's to choose.
    let client_id = if connect.client_id.is_empty() { format!("auto-{:08x}", rand::random::<u32>()) } else { connect.client_id.clone() };
    let (outbox, mut deliveries) = mpsc::channel(OUTBOX);
    let (kick, gone, queued) = (Arc::new(Notify::new()), Arc::new(Notify::new()), Arc::new(AtomicUsize::new(0)));
    let (id, replaced) = {
        let mut broker = shared.broker.lock().unwrap();
        broker.next += 1;
        let id = broker.next;
        // A second connection with a client's id takes over from the first.
        let replaced = broker.clients.insert(client_id.clone(), id).and_then(|old| broker.sessions.get(&old)).map(|old| (old.kick.clone(), old.gone.clone()));
        broker.sessions.insert(id, Session { subscriptions: Vec::new(), outbox, queued: queued.clone(), kick: kick.clone(), gone: gone.clone() });
        (id, replaced)
    };
    // As on any broker, the old connection's will is out before the new one is
    // accepted: a device that reconnects and says `online` is not overwritten
    // by its own `offline`.
    if let Some((old_kick, old_gone)) = replaced {
        old_kick.notify_one();
        let _ = tokio::time::timeout(WRITE_TIMEOUT + Duration::from_secs(1), old_gone.notified()).await;
    }
    let mut client = Client { shared, id, client_id, peer, writer, queued, received: HashSet::new(), packet_id: 0 };
    let ending = if client.write(&encode_connack(false, 0)).await {
        serve_client(&mut client, &connect, &mut reader, &mut buffer, &mut chunk, &mut deliveries, &kick).await
    } else {
        Ending::Lost
    };
    {
        let mut broker = shared.broker.lock().unwrap();
        broker.sessions.remove(&id);
        if broker.clients.get(&client.client_id) == Some(&id) {
            broker.clients.remove(&client.client_id);
        }
    }
    let _ = tokio::time::timeout(WRITE_TIMEOUT, client.writer.shutdown()).await;
    if ending == Ending::Lost {
        if let Some(will) = connect.will {
            published(shared, Message { topic: will.topic, payload: will.payload, qos: will.qos }, will.retain, &client.client_id, peer, true);
        }
    }
    gone.notify_one();
}

/// A connected client, until it leaves, is lost or the broker goes down.
async fn serve_client(
    client: &mut Client<'_>,
    connect: &Connect,
    reader: &mut OwnedReadHalf,
    buffer: &mut Vec<u8>,
    chunk: &mut [u8],
    deliveries: &mut mpsc::Receiver<(Arc<Message>, u8)>,
    kick: &Notify,
) -> Ending {
    let context = client.shared.context.clone();
    // 3.1.1: silent for one and a half times its keep-alive, a client is gone.
    let keep_alive = (connect.keep_alive_s > 0).then(|| Duration::from_millis(u64::from(connect.keep_alive_s) * 1500));
    let mut heard = Instant::now();
    let mut tick = tokio::time::interval(TICK);
    if let Some(ending) = client.drain(buffer).await {
        return ending;
    }
    loop {
        tokio::select! {
            read = reader.read(chunk) => {
                let size = match read {
                    Ok(0) | Err(_) => return Ending::Lost,
                    Ok(size) => size,
                };
                heard = Instant::now();
                buffer.extend_from_slice(&chunk[..size]);
                if let Some(ending) = client.drain(buffer).await {
                    return ending;
                }
            }
            Some((message, qos)) = deliveries.recv() => {
                client.queued.fetch_sub(message.payload.len(), Ordering::Relaxed);
                if !client.send(&message, qos, false).await {
                    return Ending::Lost;
                }
            }
            _ = kick.notified() => return Ending::Lost,
            _ = tick.tick() => {
                if context.down().is_some() {
                    return Ending::Down;
                }
                if keep_alive.is_some_and(|limit| heard.elapsed() > limit) {
                    return Ending::Lost;
                }
            }
        }
    }
}

/// What a published message is to templates: `{{request.topic}}`,
/// `{{request.levels[1]}}`, `{{request.payload}}`, `{{request.json.state}}`.
fn request_value(message: &Message, retain: bool, client: &str, peer: SocketAddr, will: bool) -> Value {
    let text = String::from_utf8_lossy(&message.payload).into_owned();
    let json = if text.trim().is_empty() { Value::Null } else { serde_json::from_str(&text).unwrap_or(Value::Null) };
    let mut value = json!({
        "topic": message.topic,
        "levels": message.topic.split('/').collect::<Vec<_>>(),
        "payload": text,
        "json": json,
        "qos": message.qos,
        "retain": retain,
        "client": client,
        "from": peer.to_string(),
    });
    if will {
        value["will"] = Value::Bool(true);
    }
    value
}

/// A rule's reply with what arrived in its topic and payload.
fn reply_of(context: &Context, rule: usize, count: u64, request: &Value, out: &MqttOut) -> EngineResult<Message> {
    context.render(rule, count, request, |renderer| {
        let field = Field::new("reply_topic");
        let topic = renderer.render(&out.topic).map_err(|error| error.in_field(field.clone()))?;
        if topic.is_empty() {
            return Err(EngineError::new("node.required").in_field(field));
        }
        if !topic_name_valid(&topic) {
            return Err(EngineError::new("node.topic_wildcard").in_field(field));
        }
        let payload = renderer.render(&out.payload).map_err(|error| error.in_field(Field::new("reply_payload")))?;
        Ok(Message { topic, payload: payload.into_bytes(), qos: out.qos })
    })
}

/// A message a client published (or its will): retained when asked, routed to
/// every subscriber, and answered by the first rule it matches.
fn published(shared: &Shared, message: Message, retain: bool, client: &str, peer: SocketAddr, will: bool) {
    let context = &shared.context;
    let Rules::Mqtt(rules) = &context.compiled.rules else { return };
    let started = Instant::now();
    let datagram = Datagram { bytes: message.payload.clone(), from: peer, at: started, topic: Some(message.topic.clone()), frame: None, request: None, binary: false };
    let found = rules
        .rules
        .iter()
        .enumerate()
        .find_map(|(index, rule)| topic_matches(&rule.filter, &message.topic).then(|| rule.matcher.matches(&datagram)).flatten().map(|matched| (index, rule, matched)));
    let line = format!("{}{}", if will { "will " } else { "" }, summarize(&message.topic, &message.payload, message.qos, retain));
    let mut request = request_value(&message, retain, client, peer, will);
    let message = Arc::new(message);
    let missed = {
        let mut broker = shared.broker.lock().unwrap();
        if retain {
            broker.retain(&message);
        }
        broker.route(&message)
    };
    context.emulation.miss(missed);
    let verdict = match &found {
        Some((index, ..)) => format!("#{}", index + 1),
        None => "—".to_string(),
    };
    let capture = context.capturing();
    let frame = capture.then(|| context.publish(Frame::rx("mqtt", "emulator").remote(peer).payload(&message.payload).summary(&line).verdict(verdict))).flatten();
    let mut exchange = Exchange { from: peer.to_string(), request: matching::shorten(&line), frame, ..Default::default() };
    let Some((index, rule, matched)) = found else {
        exchange.data = kept(&request);
        context.emulation.record(exchange);
        return;
    };
    // A pattern's match (a regex's first group) is `{{request.match}}`.
    if let Some(found) = matched.get("match") {
        request["match"] = found.clone();
    }
    let count = context.emulation.hit(index);
    exchange.rule = Some(index + 1);
    exchange.data = kept(&request);
    let Some(out) = &rule.reply else {
        context.emulation.record(exchange);
        return;
    };
    match reply_of(context, index + 1, count, &request, out) {
        Ok(reply) => {
            let delay = context.delay(index + 1, count, rule.delay_ms, rule.jitter_ms);
            later(shared, reply, out.retain, delay, capture, exchange, started);
        }
        Err(error) => {
            exchange.error = Some(error);
            context.emulation.record(exchange);
        }
    }
}

/// Publish a rule's reply after its delay — at once without one, so it
/// follows the message it answers — and record the exchange then.
fn later(shared: &Shared, reply: Message, retain: bool, delay: Duration, capture: bool, mut exchange: Exchange, started: Instant) {
    let (context, broker) = (shared.context.clone(), shared.broker.clone());
    let deliver = move |exchange: &mut Exchange| {
        let line = summarize(&reply.topic, &reply.payload, reply.qos, retain);
        let reply = Arc::new(reply);
        let missed = {
            let mut broker = broker.lock().unwrap();
            if retain {
                broker.retain(&reply);
            }
            broker.route(&reply)
        };
        context.emulation.miss(missed);
        if capture {
            context.publish(Frame::tx("mqtt", "emulator").payload(&reply.payload).summary(&line));
        }
        exchange.reply = matching::shorten(&line);
        exchange.ms = started.elapsed().as_millis() as u64;
        context.emulation.record(std::mem::take(exchange));
    };
    if delay.is_zero() {
        deliver(&mut exchange);
        return;
    }
    let mut pending = shared.pending.lock().unwrap();
    while pending.try_join_next().is_some() {}
    if pending.len() >= MAX_PENDING {
        exchange.error = Some(EngineError::new("emulator.busy").with("max", MAX_PENDING));
        shared.context.emulation.record(exchange);
        return;
    }
    pending.spawn(async move {
        tokio::time::sleep(delay).await;
        deliver(&mut exchange);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::Emulator;
    use crate::emulator_job::{self, EmulatorHub, StartOptions};
    use crate::host::{Host, Recorder};
    use crate::inspect::Capture;
    use crate::jobs::JobRegistry;
    use crate::mqtt_codec::{decode, encode_connect, encode_disconnect, encode_subscribe, ConnectOpts, Packet, Will};

    fn free_port() -> u16 {
        std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
    }

    async fn started(document: Value) -> (JobRegistry, EmulatorHub, u64, SocketAddr) {
        let host = Host::new(Recorder::new(), Capture::new());
        let (jobs, hub) = (JobRegistry::new(), EmulatorHub::new());
        let emulator: Emulator = serde_json::from_value(document).unwrap();
        let info = emulator_job::start(host, jobs.clone(), hub.clone(), emulator, StartOptions { seed: Some(3), ..Default::default() }).await.unwrap();
        let local = info.params["local"].parse().unwrap();
        (jobs, hub, info.id, local)
    }

    /// A raw MQTT client: what it sends, and the packets that come back.
    struct Raw {
        stream: TcpStream,
        buffer: Vec<u8>,
    }

    impl Raw {
        async fn connect(broker: SocketAddr, client_id: &str, login: Option<(&str, &str)>, will: Option<&Will>, keep_alive_s: u16) -> (Raw, u8) {
            let stream = TcpStream::connect(broker).await.unwrap();
            let mut raw = Raw { stream, buffer: Vec::new() };
            let opts = ConnectOpts { client_id, username: login.map(|(user, _)| user), password: login.map(|(_, password)| password), keep_alive_s, clean_session: true, will };
            raw.send(&encode_connect(&opts)).await;
            let Packet::ConnAck { code, .. } = raw.next().await.expect("a CONNACK") else { panic!("not a CONNACK") };
            (raw, code)
        }

        async fn send(&mut self, bytes: &[u8]) {
            self.stream.write_all(bytes).await.unwrap();
        }

        /// The next packet within a second; `None` when nothing (more) comes or the broker closed.
        async fn next(&mut self) -> Option<Packet> {
            let mut chunk = [0u8; 4096];
            loop {
                if let Some((packet, used)) = decode(&self.buffer).unwrap() {
                    self.buffer.drain(..used);
                    return Some(packet);
                }
                match tokio::time::timeout(Duration::from_secs(1), self.stream.read(&mut chunk)).await {
                    Ok(Ok(size)) if size > 0 => self.buffer.extend_from_slice(&chunk[..size]),
                    _ => return None,
                }
            }
        }

        async fn subscribe(&mut self, filters: &[(&str, u8)]) -> Vec<u8> {
            let filters: Vec<(String, u8)> = filters.iter().map(|(filter, qos)| (filter.to_string(), *qos)).collect();
            self.send(&encode_subscribe(1, &filters)).await;
            match self.next().await {
                Some(Packet::SubAck { codes, .. }) => codes,
                other => panic!("not a SUBACK: {other:?}"),
            }
        }

        /// The next PUBLISH: topic, payload, qos, retain.
        async fn message(&mut self) -> Option<(String, String, u8, bool)> {
            loop {
                match self.next().await? {
                    Packet::Publish { topic, payload, qos, retain, packet_id, .. } => {
                        if qos == 1 {
                            self.send(&encode_puback(packet_id)).await;
                        }
                        return Some((topic, String::from_utf8(payload).unwrap(), qos, retain));
                    }
                    _ => continue,
                }
            }
        }
    }

    #[tokio::test]
    async fn clients_publish_and_subscribe_through_it_and_a_rule_answers_like_a_device() {
        let port = free_port();
        let (jobs, hub, id, broker) = started(json!({
            "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt",
            "retained": [{ "topic": "lab/lamp/state", "payload": "OFF" }],
            "rules": [{ "topic": "lab/+/set", "mode": "regex", "pattern": "^(ON|OFF)$",
                        "reply": { "topic": "lab/{{request.levels[1]}}/state", "payload": "{{request.match}}", "qos": 1, "retain": true } }]
        }))
        .await;
        let (mut watcher, code) = Raw::connect(broker, "watcher", None, None, 30).await;
        assert_eq!(code, 0);
        assert_eq!(watcher.subscribe(&[("lab/#", 1), ("bad/#/x", 0)]).await, [1, 0x80], "granted, and a filter 3.1.1 forbids refused");
        assert_eq!(watcher.message().await, Some(("lab/lamp/state".into(), "OFF".into(), 0, true)), "what is retained comes first, marked retained");

        let (mut panel, _) = Raw::connect(broker, "panel", None, None, 30).await;
        panel.send(&encode_publish("lab/lamp/set", b"ON", 1, false, 7, false)).await;
        assert_eq!(panel.next().await, Some(Packet::PubAck(7)));
        assert_eq!(watcher.message().await, Some(("lab/lamp/set".into(), "ON".into(), 1, false)), "routed as any broker does");
        assert_eq!(watcher.message().await, Some(("lab/lamp/state".into(), "ON".into(), 1, false)), "and answered by the rule");
        panel.send(&encode_publish("lab/lamp/set", b"BLINK", 0, false, 0, false)).await;
        assert_eq!(watcher.message().await, Some(("lab/lamp/set".into(), "BLINK".into(), 0, false)), "no rule takes it: routed only");

        // The rule retained its reply: a newcomer reads the lamp's state.
        let (mut late, _) = Raw::connect(broker, "late", None, None, 30).await;
        late.subscribe(&[("lab/+/state", 2)]).await;
        assert_eq!(late.message().await, Some(("lab/lamp/state".into(), "ON".into(), 1, true)));

        let snapshot = hub.snapshot(id, 0, 10).unwrap();
        assert_eq!((snapshot.counts.total, snapshot.counts.unmatched, snapshot.counts.hits.clone()), (2, 1, vec![1]));
        let answered = &snapshot.exchanges[0];
        assert_eq!((answered.request.as_str(), answered.rule, answered.reply.as_str()), ("lab/lamp/set = ON  qos1", Some(1), "lab/lamp/state = ON  qos1  retained"));
        assert_eq!((answered.data["client"].as_str(), answered.data["levels"][1].as_str()), (Some("panel"), Some("lamp")));
        jobs.stop(id);
    }

    #[tokio::test]
    async fn qos_2_is_delivered_once_and_a_lost_client_leaves_its_will() {
        let port = free_port();
        let (jobs, hub, id, broker) = started(json!({ "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt" })).await;
        let (mut watcher, _) = Raw::connect(broker, "watcher", None, None, 30).await;
        watcher.subscribe(&[("#", 2)]).await;

        let (mut sender, _) = Raw::connect(broker, "sender", None, None, 30).await;
        sender.send(&encode_publish("show/cue", b"go", 2, false, 11, false)).await;
        assert_eq!(sender.next().await, Some(Packet::PubRec(11)));
        // The same id again before PUBREL: acknowledged, not delivered twice.
        sender.send(&encode_publish("show/cue", b"go", 2, false, 11, true)).await;
        assert_eq!(sender.next().await, Some(Packet::PubRec(11)));
        sender.send(&encode_pubrel(11)).await;
        assert_eq!(sender.next().await, Some(Packet::PubComp(11)));
        match watcher.next().await {
            Some(Packet::Publish { topic, qos: 2, packet_id, .. }) => {
                assert_eq!(topic, "show/cue");
                watcher.send(&encode_pubrec(packet_id)).await;
                assert_eq!(watcher.next().await, Some(Packet::PubRel(packet_id)), "the broker's side of QoS 2");
                watcher.send(&encode_pubcomp(packet_id)).await;
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(watcher.message().await, None, "once");

        let will = Will { topic: "show/sender/online".into(), payload: "false".into(), qos: 0, retain: false };
        let (gone, _) = Raw::connect(broker, "gone", None, Some(&will), 30).await;
        drop(gone);
        assert_eq!(watcher.message().await, Some(("show/sender/online".into(), "false".into(), 0, false)), "a dropped connection leaves its will");
        let (mut polite, _) = Raw::connect(broker, "polite", None, Some(&will), 30).await;
        polite.send(&encode_disconnect()).await;
        drop(polite);
        assert_eq!(watcher.message().await, None, "DISCONNECT takes the will back");
        let wills: Vec<bool> = hub.snapshot(id, 0, 10).unwrap().exchanges.iter().map(|exchange| exchange.request.starts_with("will ")).collect();
        assert_eq!(wills, [false, true]);
        jobs.stop(id);
    }

    #[tokio::test]
    async fn logins_takeovers_and_outages_are_a_brokers() {
        let port = free_port();
        let (jobs, hub, id, broker) = started(json!({
            "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt", "username": "lab", "password": "pw"
        }))
        .await;
        assert_eq!(Raw::connect(broker, "a", None, None, 30).await.1, 4, "no credentials");
        assert_eq!(Raw::connect(broker, "a", Some(("lab", "nope")), None, 30).await.1, 4);
        let (mut first, code) = Raw::connect(broker, "panel", Some(("lab", "pw")), None, 30).await;
        assert_eq!(code, 0);
        let (_second, code) = Raw::connect(broker, "panel", Some(("lab", "pw")), None, 30).await;
        assert_eq!(code, 0);
        assert_eq!(first.next().await, None, "the first connection with an id is closed when another takes it");
        let refused = hub.snapshot(id, 0, 10).unwrap();
        assert_eq!((refused.counts.failed, refused.exchanges[0].error.as_ref().unwrap().code.as_str()), (2, "emulator.mqtt_login"));
        jobs.stop(id);

        let port = free_port();
        let (jobs, hub, id, broker) = started(json!({
            "name": "Flapping", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt", "outage": { "up_ms": 400, "down_ms": 3_600_000 }
        }))
        .await;
        let (mut early, code) = Raw::connect(broker, "early", None, None, 30).await;
        assert_eq!(code, 0);
        tokio::time::sleep(Duration::from_millis(600)).await;
        assert_eq!(early.next().await, None, "going down drops who is connected");
        assert_eq!(Raw::connect(broker, "late", None, None, 30).await.1, 3, "and refuses who comes: server unavailable");
        assert_eq!(hub.snapshot(id, 0, 10).unwrap().counts.down, 1);
        jobs.stop(id);
    }

    #[tokio::test]
    async fn packets_sent_with_the_connect_are_taken_and_a_v5_client_hears_why_not() {
        let port = free_port();
        let (jobs, hub, id, broker) = started(json!({ "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt" })).await;
        let (mut watcher, _) = Raw::connect(broker, "watcher", None, None, 30).await;
        watcher.subscribe(&[("#", 0)]).await;

        // CONNECT, SUBSCRIBE in one write: the SUBACK comes without waiting for more.
        let mut eager = TcpStream::connect(broker).await.unwrap();
        let mut bytes = encode_connect(&ConnectOpts { client_id: "eager", username: None, password: None, keep_alive_s: 0, clean_session: true, will: None });
        bytes.extend(encode_subscribe(5, &[("a/#".into(), 1)]));
        eager.write_all(&bytes).await.unwrap();
        let mut eager = Raw { stream: eager, buffer: Vec::new() };
        assert!(matches!(eager.next().await, Some(Packet::ConnAck { code: 0, .. })));
        assert!(matches!(eager.next().await, Some(Packet::SubAck { packet_id: 5, .. })), "the SUBACK came at once");

        // CONNECT with a will, PUBLISH, DISCONNECT in one write, then gone: the message is routed, the will is not.
        let will = Will { topic: "lab/gone".into(), payload: "x".into(), qos: 0, retain: false };
        let mut quick = TcpStream::connect(broker).await.unwrap();
        let mut bytes = encode_connect(&ConnectOpts { client_id: "quick", username: None, password: None, keep_alive_s: 30, clean_session: true, will: Some(&will) });
        bytes.extend(encode_publish("lab/note", b"hello", 0, false, 0, false));
        bytes.extend(encode_disconnect());
        quick.write_all(&bytes).await.unwrap();
        drop(quick);
        assert_eq!(watcher.message().await, Some(("lab/note".into(), "hello".into(), 0, false)));
        assert_eq!(watcher.message().await, None, "DISCONNECT came before the close: no will");

        // An MQTT 5 CONNECT (level 5, a properties length) is answered 1, and said why.
        let mut v5 = TcpStream::connect(broker).await.unwrap();
        let body = [&[0u8, 4][..], b"MQTT", &[5, 0x02, 0, 30, 0], &[0, 2], b"v5"].concat();
        v5.write_all(&[&[0x10, body.len() as u8][..], &body].concat()).await.unwrap();
        let mut v5 = Raw { stream: v5, buffer: Vec::new() };
        assert!(matches!(v5.next().await, Some(Packet::ConnAck { code: 1, .. })));
        let refused = hub.snapshot(id, 0, 10).unwrap().exchanges.into_iter().find(|exchange| exchange.error.is_some()).unwrap();
        assert_eq!((refused.request.as_str(), refused.error.unwrap().params["level"].as_str()), ("CONNECT", "5"));
        jobs.stop(id);
    }

    #[tokio::test]
    async fn a_takeover_publishes_the_old_will_before_the_new_connection_speaks() {
        let port = free_port();
        let (jobs, _hub, id, broker) = started(json!({ "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt" })).await;
        let will = Will { topic: "dev/1/online".into(), payload: "false".into(), qos: 0, retain: true };
        let (_old, _) = Raw::connect(broker, "dev-1", None, Some(&will), 30).await;
        let (mut new, code) = Raw::connect(broker, "dev-1", None, None, 30).await;
        assert_eq!(code, 0);
        new.send(&encode_publish("dev/1/online", b"true", 0, true, 0, false)).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        let (mut late, _) = Raw::connect(broker, "late", None, None, 30).await;
        late.subscribe(&[("dev/+/online", 0)]).await;
        assert_eq!(late.message().await, Some(("dev/1/online".into(), "true".into(), 0, true)), "the retained value is the new connection's");
        jobs.stop(id);
    }

    #[test]
    fn a_full_outbox_counts_what_a_slow_client_missed() {
        let mut broker = Broker::default();
        let (outbox, _deliveries) = mpsc::channel(1);
        let queued = Arc::new(AtomicUsize::new(0));
        broker.sessions.insert(1, Session { subscriptions: vec![("#".into(), 1)], outbox, queued: queued.clone(), kick: Arc::new(Notify::new()), gone: Arc::new(Notify::new()) });
        let message = Arc::new(Message { topic: "a".into(), payload: vec![0; 10], qos: 1 });
        assert_eq!((broker.route(&message), broker.route(&message)), (0, 1), "the second does not fit");
        assert_eq!(queued.load(Ordering::Relaxed), 10);
        let big = Arc::new(Message { topic: "a".into(), payload: vec![0; OUTBOX_BYTES + 1], qos: 0 });
        assert_eq!(broker.route(&big), 1);
    }

    #[test]
    fn retained_messages_stay_within_their_bounds() {
        let mut broker = Broker::default();
        let message = |topic: &str, size: usize| Message { topic: topic.into(), payload: vec![b'x'; size], qos: 0 };
        broker.retain(&message("a", 10));
        broker.retain(&message("a", 4));
        assert_eq!((broker.retained.len(), broker.retained_bytes), (1, 4), "a topic's newer value replaces the older");
        broker.retain(&message("big", MAX_RETAINED_BYTES));
        assert!(!broker.retained.contains_key("big"), "past the byte bound it is routed, not retained");
        broker.retain(&message("a", MAX_RETAINED_BYTES + 1));
        assert!(!broker.retained.contains_key("a"), "a value that does not fit takes the older one away");
        broker.retain(&message("a", 4));
        broker.retain(&message("a", 0));
        assert_eq!((broker.retained.len(), broker.retained_bytes), (0, 0), "an empty payload forgets the topic");
        for index in 0..MAX_RETAINED + 5 {
            broker.retain(&message(&format!("t/{index}"), 1));
        }
        assert_eq!(broker.retained.len(), MAX_RETAINED);
    }

    #[tokio::test]
    async fn a_will_on_a_wildcard_is_a_malformed_connect() {
        let port = free_port();
        let (jobs, _hub, id, broker) = started(json!({ "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt" })).await;
        let will = Will { topic: "lab/+/online".into(), payload: "false".into(), qos: 0, retain: false };
        let mut stream = TcpStream::connect(broker).await.unwrap();
        stream.write_all(&encode_connect(&ConnectOpts { client_id: "x", username: None, password: None, keep_alive_s: 30, clean_session: true, will: Some(&will) })).await.unwrap();
        let mut chunk = [0u8; 16];
        let read = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut chunk)).await.unwrap();
        assert!(matches!(read, Ok(0) | Err(_)), "closed without a CONNACK: {read:?}");
        jobs.stop(id);
    }

    #[tokio::test]
    async fn a_client_silent_past_its_keep_alive_is_gone() {
        let port = free_port();
        let (jobs, _hub, id, broker) = started(json!({ "name": "Broker", "bind": format!("127.0.0.1:{port}"), "protocol": "mqtt" })).await;
        let (mut quiet, _) = Raw::connect(broker, "quiet", None, None, 1).await;
        let begun = Instant::now();
        assert_eq!(quiet.next().await, None);
        let mut chunk = [0u8; 16];
        let closed = tokio::time::timeout(Duration::from_secs(3), quiet.stream.read(&mut chunk)).await.unwrap();
        assert!(matches!(closed, Ok(0) | Err(_)), "closed after 1.5 s of silence");
        assert!(begun.elapsed() >= Duration::from_millis(1400));
        jobs.stop(id);
    }
}
