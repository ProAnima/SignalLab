//! OSC, UDP and TCP emulators: "on this, reply that". A datagram (each
//! message of an OSC packet) or a line on a TCP connection is answered by
//! the first rule it matches — the matching of *Wait for OSC* and *Wait for
//! UDP* — with a rendered reply after the rule's delay. Replies leave from the
//! emulator's own socket, so a device that answers the sender's port is what
//! the system under test hears. While its outage has it down, a device hears
//! and answers nothing, and a TCP device drops and refuses its connections.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::task::JoinSet;

use super::emulator::{Delimiter, OscOut};
use super::emulator_rules::{osc_arg, Rules, TcpResponder};
use super::emulator_state::{Context, Exchange};
use super::error::{EngineError, EngineResult, Field};
use super::inspect::{ascii_preview, Frame};
use super::listen::Tap;
use super::matching::{self, Datagram, Matcher};
use super::osc_codec::{arg_str, decode_packet, encode_message, OscMessage};
use super::signals::RawPayload;
use super::transport;

/// Replies waiting for their delay at once; more are dropped and counted.
const MAX_PENDING: usize = 1024;
/// Connections a TCP emulator serves at once.
const MAX_CONNECTIONS: usize = 256;
/// A TCP message without its delimiter is cut here.
const MAX_LINE: usize = 64 * 1024;
const ACCEPT_FAILURES: u32 = 50;
/// Characters of a payload shown in an exchange.
const SHOWN: usize = 96;

/// `/address arg arg` as the monitor writes it.
fn message_line(message: &OscMessage) -> String {
    std::iter::once(message.address.clone()).chain(message.args.iter().map(arg_str)).collect::<Vec<_>>().join(" ")
}

/// A reply payload's bytes, its templates read with `request`.
fn raw_bytes(context: &Context, rule: usize, count: u64, request: &Value, reply: &RawPayload) -> EngineResult<Vec<u8>> {
    context.render(rule, count, request, |renderer| match reply {
        RawPayload::Text { text } => renderer.render(text).map(String::into_bytes).map_err(|error| error.in_field(Field::new("reply"))),
        RawPayload::Hex { hex } => {
            let text = renderer.render(hex).map_err(|error| error.in_field(Field::new("reply")))?;
            matching::parse_hex(&text).map_err(|error| error.in_field(Field::new("reply")))
        }
    })
}

/// A reply message, its address and arguments rendered and typed.
fn osc_packet(context: &Context, rule: usize, count: u64, request: &Value, out: &OscOut) -> EngineResult<(Vec<u8>, String)> {
    context.render(rule, count, request, |renderer| {
        let address = renderer.render(&out.address).map_err(|error| error.in_field(Field::new("reply_address")))?;
        let mut args = Vec::with_capacity(out.args.len());
        for (index, arg) in out.args.iter().enumerate() {
            let field = Field::nth("reply_arg", index + 1);
            let text = renderer.render(&arg.value).map_err(|error| error.in_field(field.clone()))?;
            args.push(osc_arg(arg.kind, &text).map_err(|error| error.in_field(field))?);
        }
        let message = OscMessage { address, args };
        Ok((encode_message(&message.address, &message.args), message_line(&message)))
    })
}

/// Answers what arrives on a UDP socket: an OSC or UDP emulator's, or a run's
/// listener on the same port (then the run's waits see it too).
pub(crate) struct Responder {
    context: Arc<Context>,
    /// Replies waiting for their delay; dropped (and stopped) with the responder.
    pending: Mutex<JoinSet<()>>,
}

impl Responder {
    pub(crate) fn new(context: Arc<Context>) -> Arc<Responder> {
        Arc::new(Responder { context, pending: Mutex::new(JoinSet::new()) })
    }

    fn answer(&self, socket: &Arc<UdpSocket>, datagram: &Datagram) -> Option<u64> {
        if self.context.down_for().is_some() {
            return self.down(datagram);
        }
        match &self.context.compiled.rules {
            Rules::Osc(_) => self.osc(socket, datagram),
            Rules::Udp(_) => self.udp(socket, datagram),
            _ => None,
        }
    }

    fn osc(&self, socket: &Arc<UdpSocket>, datagram: &Datagram) -> Option<u64> {
        let context = &self.context;
        let Rules::Osc(rules) = &context.compiled.rules else { return None };
        let capture = context.capturing();
        let from = datagram.from;
        let messages = match decode_packet(&datagram.bytes) {
            Ok(messages) if !messages.is_empty() => messages,
            // Not OSC: no rule can take it; it is counted and shown all the same.
            _ => {
                let request = ascii_preview(&datagram.bytes, SHOWN);
                let frame = capture.then(|| context.publish(Frame::rx("udp", "emulator").remote(from).payload(&datagram.bytes).summary(&request).verdict("not OSC"))).flatten();
                context.emulation.record(Exchange { from: from.to_string(), request, frame, ..Default::default() });
                return frame;
            }
        };
        let mut first = None;
        for message in &messages {
            let line = message_line(message);
            let found = rules.iter().enumerate().find(|(_, rule)| rule.matcher.accepts(message));
            let verdict = match found {
                Some((index, _)) => format!("#{}", index + 1),
                None => "—".to_string(),
            };
            let frame = capture.then(|| context.publish(Frame::rx("osc", "emulator").remote(from).payload(&datagram.bytes).summary(&line).verdict(verdict))).flatten();
            first = first.or(frame);
            let mut exchange = Exchange { from: from.to_string(), request: matching::shorten(&line), frame, ..Default::default() };
            let Some((index, rule)) = found else {
                context.emulation.record(exchange);
                continue;
            };
            let count = context.emulation.hit(index);
            let request = matching::osc_value(message, from);
            exchange.rule = Some(index + 1);
            exchange.data = request.clone();
            let Some(out) = &rule.reply else {
                context.emulation.record(exchange);
                continue;
            };
            match osc_packet(context, index + 1, count, &request, out) {
                Ok((packet, line)) => {
                    let delay = context.delay(index + 1, count, rule.delay_ms, rule.jitter_ms);
                    self.send(socket, packet, "osc", line, rule.to.unwrap_or(from), delay, capture, exchange);
                }
                Err(error) => {
                    exchange.error = Some(error);
                    context.emulation.record(exchange);
                }
            }
        }
        first
    }

    fn udp(&self, socket: &Arc<UdpSocket>, datagram: &Datagram) -> Option<u64> {
        let context = &self.context;
        let Rules::Udp(rules) = &context.compiled.rules else { return None };
        let capture = context.capturing();
        let from = datagram.from;
        let line = ascii_preview(&datagram.bytes, SHOWN);
        let found = rules.iter().enumerate().find_map(|(index, rule)| rule.matcher.matches(datagram).map(|value| (index, rule, value)));
        let verdict = match &found {
            Some((index, ..)) => format!("#{}", index + 1),
            None => "—".to_string(),
        };
        let frame = capture.then(|| context.publish(Frame::rx("udp", "emulator").remote(from).payload(&datagram.bytes).summary(&line).verdict(verdict))).flatten();
        let mut exchange = Exchange { from: from.to_string(), request: line, frame, ..Default::default() };
        let Some((index, rule, request)) = found else {
            context.emulation.record(exchange);
            return frame;
        };
        let count = context.emulation.hit(index);
        exchange.rule = Some(index + 1);
        exchange.data = request.clone();
        let Some(reply) = &rule.reply else {
            context.emulation.record(exchange);
            return frame;
        };
        match raw_bytes(context, index + 1, count, &request, reply) {
            Ok(packet) => {
                let line = ascii_preview(&packet, SHOWN);
                let delay = context.delay(index + 1, count, rule.delay_ms, rule.jitter_ms);
                self.send(socket, packet, "udp", line, rule.to.unwrap_or(from), delay, capture, exchange);
            }
            Err(error) => {
                exchange.error = Some(error);
                context.emulation.record(exchange);
            }
        }
        frame
    }

    /// A datagram that arrived while the device is down: counted, not answered.
    fn down(&self, datagram: &Datagram) -> Option<u64> {
        let context = &self.context;
        let osc = matches!(context.compiled.rules, Rules::Osc(_));
        let request = match decode_packet(&datagram.bytes) {
            Ok(messages) if osc && !messages.is_empty() => matching::shorten(&messages.iter().map(message_line).collect::<Vec<_>>().join("; ")),
            _ => ascii_preview(&datagram.bytes, SHOWN),
        };
        let proto = if osc { "osc" } else { "udp" };
        let frame = context.capturing().then(|| context.publish(Frame::rx(proto, "emulator").remote(datagram.from).payload(&datagram.bytes).summary(&request).verdict("down"))).flatten();
        context.emulation.record(Exchange { from: datagram.from.to_string(), request, frame, down: true, ..Default::default() });
        frame
    }

    /// Send a reply after its delay, from `socket`, and record the exchange then.
    #[allow(clippy::too_many_arguments)]
    fn send(&self, socket: &Arc<UdpSocket>, packet: Vec<u8>, proto: &'static str, line: String, to: SocketAddr, delay: Duration, capture: bool, mut exchange: Exchange) {
        let started = Instant::now();
        let mut pending = self.pending.lock().unwrap();
        while pending.try_join_next().is_some() {}
        if pending.len() >= MAX_PENDING {
            exchange.error = Some(EngineError::new("emulator.busy").with("max", MAX_PENDING));
            self.context.emulation.record(exchange);
            return;
        }
        let (socket, context) = (socket.clone(), self.context.clone());
        pending.spawn(async move {
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
            match socket.send_to(&packet, to).await {
                Ok(_) => {
                    if capture {
                        context.publish(Frame::tx(proto, "emulator").remote(to).payload(&packet).summary(&line));
                    }
                    exchange.reply = matching::shorten(&line);
                }
                Err(error) => exchange.error = Some(transport::of_io(&error).error(&to.to_string()).because(error)),
            }
            exchange.ms = started.elapsed().as_millis() as u64;
            context.emulation.record(exchange);
        });
    }
}

impl Tap for Responder {
    fn received(&self, socket: &Arc<UdpSocket>, datagram: &Datagram) -> Option<u64> {
        self.answer(socket, datagram)
    }
}

/// An OSC or UDP emulator of its own: receive and answer until the socket fails.
pub(crate) async fn serve_datagrams(socket: Arc<UdpSocket>, context: Arc<Context>) -> EngineError {
    let local = context.emulation.local;
    let responder = Responder::new(context);
    let mut buffer = vec![0u8; 65_536];
    loop {
        match socket.recv_from(&mut buffer).await {
            Ok((size, from)) => {
                let datagram = Datagram { bytes: buffer[..size].to_vec(), from, at: Instant::now(), topic: None, frame: None, request: None };
                responder.answer(&socket, &datagram);
            }
            // An ICMP "port unreachable" for an earlier reply; the socket is fine.
            Err(error) if crate::net::udp_transient(&error) => continue,
            Err(error) => return EngineError::new("wait.receive_failed").with("target", local).because(error),
        }
    }
}

/// The next message in `buffer` up to `delimiter`, without it; an empty line is skipped.
fn take_message(buffer: &mut Vec<u8>, delimiter: Delimiter) -> Option<Vec<u8>> {
    loop {
        let (end, skip) = match delimiter {
            Delimiter::None => (buffer.len(), 0),
            Delimiter::Lf => (buffer.iter().position(|byte| *byte == b'\n')?, 1),
            Delimiter::Cr => (buffer.iter().position(|byte| *byte == b'\r')?, 1),
            Delimiter::Crlf => (buffer.windows(2).position(|pair| pair == b"\r\n")?, 2),
        };
        if delimiter == Delimiter::None && end == 0 {
            return None;
        }
        let mut message: Vec<u8> = buffer.drain(..end + skip).take(end).collect();
        if delimiter == Delimiter::Lf && message.last() == Some(&b'\r') {
            message.pop();
        }
        if !message.is_empty() {
            return Some(message);
        }
    }
}

/// A TCP emulator: one task per connection, owned here.
pub(crate) async fn serve_tcp(listener: TcpListener, context: Arc<Context>) -> EngineError {
    let local = context.emulation.local;
    let active = Arc::new(AtomicUsize::new(0));
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
        // Down: the connection is closed as it arrives, before the greeting.
        if context.down_for().is_some() {
            drop(stream);
            context.emulation.record(Exchange { from: peer.to_string(), request: "connect".into(), down: true, ..Default::default() });
            continue;
        }
        active.fetch_add(1, Ordering::Relaxed);
        let (context, active) = (context.clone(), active.clone());
        connections.spawn(async move {
            connection(stream, peer, &context).await;
            active.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

async fn connection(mut stream: TcpStream, peer: SocketAddr, context: &Context) {
    let Rules::Tcp { delimiter, greeting, rules } = &context.compiled.rules else { return };
    let delimiter = *delimiter;
    if !greeting.is_empty() {
        let request = json!({ "from": peer.to_string() });
        match context.render(0, 1, &request, |renderer| renderer.render(greeting).map_err(|error| error.in_field(Field::new("greeting")))) {
            Ok(text) => {
                let bytes = [text.as_bytes(), delimiter.bytes()].concat();
                if stream.write_all(&bytes).await.is_err() {
                    return;
                }
                if context.capturing() {
                    context.publish(Frame::tx("tcp", "emulator").remote(peer).payload(&bytes).summary(ascii_preview(&bytes, SHOWN)));
                }
            }
            Err(error) => context.emulation.record(Exchange { from: peer.to_string(), error: Some(error), ..Default::default() }),
        }
    }
    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 8192];
    // An outage drops connections that are quiet too, not only the next to speak.
    let outage = context.compiled.outage.is_some();
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        let size = tokio::select! {
            read = stream.read(&mut chunk) => match read {
                Ok(0) | Err(_) => return,
                Ok(size) => size,
            },
            _ = tick.tick(), if outage => {
                if context.down_for().is_some() {
                    let _ = stream.shutdown().await;
                    return;
                }
                continue;
            }
        };
        buffer.extend_from_slice(&chunk[..size]);
        loop {
            let message = match take_message(&mut buffer, delimiter) {
                Some(message) => message,
                // A line too long for its delimiter is a message as it stands.
                None if buffer.len() > MAX_LINE => std::mem::take(&mut buffer),
                None => break,
            };
            if !reply_line(&mut stream, peer, &message, context, rules, delimiter).await {
                return;
            }
        }
    }
}

/// Answer one message; false when the connection is to close.
async fn reply_line(stream: &mut TcpStream, peer: SocketAddr, message: &[u8], context: &Context, rules: &[TcpResponder], delimiter: Delimiter) -> bool {
    let started = Instant::now();
    let capture = context.capturing();
    let line = ascii_preview(message, SHOWN);
    // Down: nothing is answered, and the connection drops.
    if context.down_for().is_some() {
        let frame = capture.then(|| context.publish(Frame::rx("tcp", "emulator").remote(peer).payload(message).summary(&line).verdict("down"))).flatten();
        context.emulation.record(Exchange { from: peer.to_string(), request: line, frame, down: true, ..Default::default() });
        let _ = stream.shutdown().await;
        return false;
    }
    let datagram = Datagram { bytes: message.to_vec(), from: peer, at: started, topic: None, frame: None, request: None };
    let found = rules.iter().enumerate().find_map(|(index, rule)| rule.matcher.matches(&datagram).map(|value| (index, rule, value)));
    let verdict = match &found {
        Some((index, ..)) => format!("#{}", index + 1),
        None => "—".to_string(),
    };
    let frame = capture.then(|| context.publish(Frame::rx("tcp", "emulator").remote(peer).payload(message).summary(&line).verdict(verdict))).flatten();
    let mut exchange = Exchange { from: peer.to_string(), request: line, frame, ..Default::default() };
    let Some((index, rule, request)) = found else {
        context.emulation.record(exchange);
        return true;
    };
    let count = context.emulation.hit(index);
    exchange.rule = Some(index + 1);
    exchange.data = request.clone();
    let mut open = !rule.close;
    if let Some(reply) = &rule.reply {
        match raw_bytes(context, index + 1, count, &request, reply) {
            Ok(text) => {
                tokio::time::sleep(context.delay(index + 1, count, rule.delay_ms, rule.jitter_ms)).await;
                let bytes = [text.as_slice(), delimiter.bytes()].concat();
                match stream.write_all(&bytes).await {
                    Ok(()) => {
                        if capture {
                            context.publish(Frame::tx("tcp", "emulator").remote(peer).payload(&bytes).summary(ascii_preview(&bytes, SHOWN)));
                        }
                        exchange.reply = ascii_preview(&text, SHOWN);
                    }
                    Err(error) => {
                        exchange.error = Some(transport::of_io(&error).error(&peer.to_string()).because(error));
                        open = false;
                    }
                }
            }
            Err(error) => exchange.error = Some(error),
        }
    }
    exchange.ms = started.elapsed().as_millis() as u64;
    context.emulation.record(exchange);
    if !open {
        let _ = stream.shutdown().await;
    }
    open
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::Emulator;
    use crate::emulator_job::{self, EmulatorHub, StartOptions};
    use crate::host::{Host, Recorder};
    use crate::inspect::Capture;
    use crate::jobs::JobRegistry;
    use crate::osc_codec::OscArg;

    async fn started(document: Value) -> (JobRegistry, EmulatorHub, u64, SocketAddr) {
        let host = Host::new(Recorder::new(), Capture::new());
        let (jobs, hub) = (JobRegistry::new(), EmulatorHub::new());
        let emulator: Emulator = serde_json::from_value(document).unwrap();
        let info = emulator_job::start(host, jobs.clone(), hub.clone(), emulator, StartOptions { seed: Some(1), ..Default::default() }).await.unwrap();
        let local = info.params["local"].parse().unwrap();
        (jobs, hub, info.id, local)
    }

    fn udp_port() -> u16 {
        std::net::UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
    }

    async fn receive(socket: &UdpSocket) -> (Vec<u8>, SocketAddr) {
        let mut buffer = vec![0u8; 2048];
        let (size, from) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buffer)).await.expect("no reply within 2 s").unwrap();
        (buffer[..size].to_vec(), from)
    }

    #[tokio::test]
    async fn an_osc_device_answers_the_sender_from_its_own_port() {
        let (jobs, hub, id, local) = started(json!({
            "name": "Device", "bind": format!("127.0.0.1:{}", udp_port()), "protocol": "osc",
            "rules": [
                { "address": "/ping", "args": [{ "index": 0, "op": "gt", "value": "0" }],
                  "reply": { "address": "/pong", "args": [{ "type": "int", "value": "{{request.args[0]}}" }, { "type": "str", "value": "n{{counter}}" }] } },
                { "address": "/cue/*" }
            ]
        }))
        .await;
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(&encode_message("/ping", &[OscArg::Int(42)]), local).await.unwrap();
        let (packet, from) = receive(&client).await;
        assert_eq!(from, local, "the reply leaves the emulator's port");
        let reply = &decode_packet(&packet).unwrap()[0];
        assert_eq!((reply.address.as_str(), reply.args.clone()), ("/pong", vec![OscArg::Int(42), OscArg::Str("n1".into())]));
        client.send_to(&encode_message("/ping", &[OscArg::Int(0)]), local).await.unwrap();
        client.send_to(&encode_message("/cue/go", &[]), local).await.unwrap();
        client.send_to(b"not osc", local).await.unwrap();
        tokio::time::sleep(Duration::from_millis(150)).await;
        let snapshot = hub.snapshot(id, 0, 10).unwrap();
        assert_eq!((snapshot.counts.hits.clone(), snapshot.counts.unmatched), (vec![1, 1], 2), "a /ping 0 and a non-OSC datagram take no rule");
        let answered = snapshot.exchanges.iter().find(|exchange| exchange.rule == Some(1)).unwrap();
        assert_eq!((answered.request.as_str(), answered.reply.as_str()), ("/ping 42", "/pong 42 \"n1\""));
        jobs.stop(id);
    }

    #[tokio::test]
    async fn a_udp_device_replies_by_payload_after_its_delay() {
        let (jobs, hub, id, local) = started(json!({
            "name": "Echo", "bind": format!("127.0.0.1:{}", udp_port()), "protocol": "udp",
            "rules": [
                { "mode": "regex", "pattern": "^PING (\\d+)$", "reply": { "kind": "text", "text": "PONG {{request.match}}" }, "delay_ms": 150 },
                { "mode": "hex", "pattern": "ff", "reply": { "kind": "hex", "hex": "de ad" } }
            ]
        }))
        .await;
        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let sent = Instant::now();
        client.send_to(b"PING 7", local).await.unwrap();
        assert_eq!(receive(&client).await.0, b"PONG 7");
        assert!(sent.elapsed() >= Duration::from_millis(150), "the delay is kept");
        client.send_to(&[0x00, 0xff], local).await.unwrap();
        assert_eq!(receive(&client).await.0, [0xde, 0xad]);
        tokio::time::sleep(Duration::from_millis(50)).await;
        let exchanges = hub.snapshot(id, 0, 10).unwrap().exchanges;
        assert!(exchanges[0].ms >= 150 && exchanges[0].reply == "PONG 7");
        jobs.stop(id);
    }

    #[tokio::test]
    async fn a_tcp_device_greets_answers_lines_and_closes_when_told() {
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let (jobs, hub, id, local) = started(json!({
            "name": "Projector", "bind": format!("127.0.0.1:{port}"), "protocol": "tcp", "delimiter": "crlf", "greeting": "READY",
            "rules": [
                { "mode": "contains", "pattern": "POWER?", "reply": { "kind": "text", "text": "POWER=ON" } },
                { "mode": "contains", "pattern": "QUIT", "reply": { "kind": "text", "text": "BYE" }, "close": true }
            ]
        }))
        .await;
        let mut stream = TcpStream::connect(local).await.unwrap();
        // Two lines in one write, and one split across writes.
        stream.write_all(b"POWER?\r\nHELLO\r\nPOW").await.unwrap();
        stream.write_all(b"ER?\r\nQUIT\r\n").await.unwrap();
        let mut received = Vec::new();
        tokio::time::timeout(Duration::from_secs(2), stream.read_to_end(&mut received)).await.unwrap().unwrap();
        assert_eq!(String::from_utf8(received).unwrap(), "READY\r\nPOWER=ON\r\nPOWER=ON\r\nBYE\r\n", "closed after BYE");
        let snapshot = hub.snapshot(id, 0, 10).unwrap();
        assert_eq!((snapshot.counts.hits.clone(), snapshot.counts.unmatched), (vec![2, 1], 1));
        jobs.stop(id);
    }

    #[test]
    fn messages_split_at_their_delimiter() {
        let mut buffer = b"a\r\nb\n\nc".to_vec();
        assert_eq!(take_message(&mut buffer, Delimiter::Lf), Some(b"a".to_vec()));
        assert_eq!(take_message(&mut buffer, Delimiter::Lf), Some(b"b".to_vec()), "an empty line is skipped");
        assert_eq!(take_message(&mut buffer, Delimiter::Lf), None);
        assert_eq!(buffer, b"c");
        let mut cr = b"x\ry".to_vec();
        assert_eq!(take_message(&mut cr, Delimiter::Cr), Some(b"x".to_vec()));
        let mut chunk = b"raw".to_vec();
        assert_eq!((take_message(&mut chunk, Delimiter::None), chunk.is_empty()), (Some(b"raw".to_vec()), true));
        assert_eq!(take_message(&mut chunk, Delimiter::None), None);
    }
}
