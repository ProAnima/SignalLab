//! Minimal, self-contained MQTT 3.1.1 codec (no external crates).
//!
//! Scoped to what a lab client needs and what field gear actually speaks: plain
//! TCP, QoS 0/1/2, retain, wildcards, a last-will, and a clean session. No
//! MQTT 5, no persistent sessions, no TLS.
//!
//! `decode` and the `encode_*` before it are the client's side: what a broker
//! sends, and what a client sends it. `decode_client` and the encoders after
//! it are the broker's side, for the MQTT emulator. Everything is
//! length-prefixed, so a decoder hands back `None` until a whole packet has
//! arrived rather than guessing.

use serde::{Deserialize, Serialize};

// Control packet types, in the wire's numbering.
const CONNECT: u8 = 1;
const CONNACK: u8 = 2;
const PUBLISH: u8 = 3;
const PUBACK: u8 = 4;
const PUBREC: u8 = 5;
const PUBREL: u8 = 6;
const PUBCOMP: u8 = 7;
const SUBSCRIBE: u8 = 8;
const SUBACK: u8 = 9;
const UNSUBSCRIBE: u8 = 10;
const UNSUBACK: u8 = 11;
const PINGREQ: u8 = 12;
const PINGRESP: u8 = 13;
const DISCONNECT: u8 = 14;

/// The largest remaining-length the format can express (four 7-bit groups).
pub const MAX_REMAINING: usize = 268_435_455;

/// A last-will, published by the broker if we drop without saying goodbye. The
/// gear in the field uses this for presence, so it is not optional decoration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Will {
    pub topic: String,
    #[serde(default)]
    pub payload: String,
    #[serde(default)]
    pub qos: u8,
    #[serde(default)]
    pub retain: bool,
}

#[derive(Clone, Debug)]
pub struct ConnectOpts<'a> {
    pub client_id: &'a str,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub keep_alive_s: u16,
    pub clean_session: bool,
    pub will: Option<&'a Will>,
}

/// A packet arriving from the broker.
#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    ConnAck {
        session_present: bool,
        code: u8,
    },
    Publish {
        dup: bool,
        qos: u8,
        retain: bool,
        topic: String,
        packet_id: u16,
        payload: Vec<u8>,
    },
    PubAck(u16),
    PubRec(u16),
    PubRel(u16),
    PubComp(u16),
    SubAck {
        packet_id: u16,
        codes: Vec<u8>,
    },
    UnsubAck(u16),
    PingResp,
}

// ---------------------------------------------------------------------------
// encoding
// ---------------------------------------------------------------------------

fn put_remaining(out: &mut Vec<u8>, mut len: usize) {
    loop {
        let mut byte = (len % 128) as u8;
        len /= 128;
        if len > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if len == 0 {
            break;
        }
    }
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn frame(kind: u8, flags: u8, body: Vec<u8>) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len() + 5);
    out.push((kind << 4) | flags);
    put_remaining(&mut out, body.len());
    out.extend_from_slice(&body);
    out
}

pub fn encode_connect(o: &ConnectOpts) -> Vec<u8> {
    let mut body = Vec::with_capacity(64);
    put_str(&mut body, "MQTT");
    body.push(0x04); // protocol level 4 = 3.1.1

    let mut flags = 0u8;
    if o.clean_session {
        flags |= 0x02;
    }
    if let Some(w) = o.will {
        flags |= 0x04;
        flags |= (w.qos.min(2) & 0x03) << 3;
        if w.retain {
            flags |= 0x20;
        }
    }
    // A password without a username is not expressible in 3.1.1, and brokers
    // reject the attempt; send it only alongside one.
    if o.username.is_some_and(|u| !u.is_empty()) {
        flags |= 0x80;
        if o.password.is_some_and(|p| !p.is_empty()) {
            flags |= 0x40;
        }
    }
    body.push(flags);
    body.extend_from_slice(&o.keep_alive_s.to_be_bytes());

    put_str(&mut body, o.client_id);
    if let Some(w) = o.will {
        put_str(&mut body, &w.topic);
        put_str(&mut body, &w.payload);
    }
    if let Some(u) = o.username.filter(|u| !u.is_empty()) {
        put_str(&mut body, u);
        if let Some(p) = o.password.filter(|p| !p.is_empty()) {
            put_str(&mut body, p);
        }
    }
    frame(CONNECT, 0, body)
}

pub fn encode_publish(
    topic: &str,
    payload: &[u8],
    qos: u8,
    retain: bool,
    packet_id: u16,
    dup: bool,
) -> Vec<u8> {
    let qos = qos.min(2);
    let mut body = Vec::with_capacity(topic.len() + payload.len() + 4);
    put_str(&mut body, topic);
    if qos > 0 {
        body.extend_from_slice(&packet_id.to_be_bytes());
    }
    body.extend_from_slice(payload);

    let mut flags = qos << 1;
    if retain {
        flags |= 0x01;
    }
    if dup {
        flags |= 0x08;
    }
    frame(PUBLISH, flags, body)
}

pub fn encode_subscribe(packet_id: u16, filters: &[(String, u8)]) -> Vec<u8> {
    let mut body = Vec::with_capacity(filters.len() * 16 + 2);
    body.extend_from_slice(&packet_id.to_be_bytes());
    for (filter, qos) in filters {
        put_str(&mut body, filter);
        body.push(qos.min(&2).to_owned());
    }
    // The 0x02 flag is required on SUBSCRIBE; brokers close the connection on 0.
    frame(SUBSCRIBE, 0x02, body)
}

pub fn encode_unsubscribe(packet_id: u16, filters: &[String]) -> Vec<u8> {
    let mut body = Vec::with_capacity(filters.len() * 16 + 2);
    body.extend_from_slice(&packet_id.to_be_bytes());
    for filter in filters {
        put_str(&mut body, filter);
    }
    frame(UNSUBSCRIBE, 0x02, body)
}

fn ack(kind: u8, flags: u8, packet_id: u16) -> Vec<u8> {
    frame(kind, flags, packet_id.to_be_bytes().to_vec())
}

pub fn encode_puback(packet_id: u16) -> Vec<u8> {
    ack(PUBACK, 0, packet_id)
}
pub fn encode_pubrec(packet_id: u16) -> Vec<u8> {
    ack(PUBREC, 0, packet_id)
}
/// PUBREL carries a mandatory 0x02 in its flags, unlike the other acks.
pub fn encode_pubrel(packet_id: u16) -> Vec<u8> {
    ack(PUBREL, 0x02, packet_id)
}
pub fn encode_pubcomp(packet_id: u16) -> Vec<u8> {
    ack(PUBCOMP, 0, packet_id)
}
pub fn encode_pingreq() -> Vec<u8> {
    frame(PINGREQ, 0, Vec::new())
}
pub fn encode_disconnect() -> Vec<u8> {
    frame(DISCONNECT, 0, Vec::new())
}

// ---------------------------------------------------------------------------
// decoding
// ---------------------------------------------------------------------------

fn take_str(buf: &[u8], at: &mut usize) -> Result<String, String> {
    if *at + 2 > buf.len() {
        return Err("string length runs past the packet".into());
    }
    let len = u16::from_be_bytes([buf[*at], buf[*at + 1]]) as usize;
    *at += 2;
    if *at + len > buf.len() {
        return Err("string runs past the packet".into());
    }
    let s = String::from_utf8_lossy(&buf[*at..*at + len]).into_owned();
    *at += len;
    Ok(s)
}

fn take_u16(buf: &[u8], at: &mut usize) -> Result<u16, String> {
    if *at + 2 > buf.len() {
        return Err("packet id runs past the packet".into());
    }
    let v = u16::from_be_bytes([buf[*at], buf[*at + 1]]);
    *at += 2;
    Ok(v)
}

/// The fixed header at the front of `buf`: type, flags, its own length and
/// the packet's whole length — `None` until all of the packet is there.
fn fixed_header(buf: &[u8], max: usize) -> Result<Option<(u8, u8, usize, usize)>, String> {
    if buf.len() < 2 {
        return Ok(None);
    }
    let kind = buf[0] >> 4;
    let flags = buf[0] & 0x0F;

    // Remaining length: up to four bytes, 7 bits each.
    let mut remaining = 0usize;
    let mut multiplier = 1usize;
    let mut header = 1usize;
    loop {
        if header >= buf.len() {
            return Ok(None);
        }
        if header > 4 {
            return Err("remaining length is longer than four bytes".into());
        }
        let byte = buf[header];
        header += 1;
        remaining += (byte & 0x7F) as usize * multiplier;
        if byte & 0x80 == 0 {
            break;
        }
        multiplier *= 128;
    }
    if remaining > max {
        return Err(format!("packet claims {remaining} bytes"));
    }
    let total = header + remaining;
    if buf.len() < total {
        return Ok(None);
    }
    Ok(Some((kind, flags, header, total)))
}

/// Decode one packet from the front of `buf`.
///
/// `Ok(None)` means "not all here yet, keep reading" — the only correct answer
/// for a stream, and the reason the caller keeps a persistent buffer.
pub fn decode(buf: &[u8]) -> Result<Option<(Packet, usize)>, String> {
    let Some((kind, flags, header, total)) = fixed_header(buf, MAX_REMAINING)? else { return Ok(None) };
    let body = &buf[header..total];
    let mut at = 0usize;

    let packet = match kind {
        CONNACK => {
            if body.len() < 2 {
                return Err("CONNACK is shorter than two bytes".into());
            }
            Packet::ConnAck {
                session_present: body[0] & 0x01 != 0,
                code: body[1],
            }
        }
        PUBLISH => {
            let qos = (flags >> 1) & 0x03;
            if qos > 2 {
                return Err("PUBLISH claims QoS 3".into());
            }
            let topic = take_str(body, &mut at)?;
            let packet_id = if qos > 0 { take_u16(body, &mut at)? } else { 0 };
            Packet::Publish {
                dup: flags & 0x08 != 0,
                qos,
                retain: flags & 0x01 != 0,
                topic,
                packet_id,
                payload: body[at..].to_vec(),
            }
        }
        PUBACK => Packet::PubAck(take_u16(body, &mut at)?),
        PUBREC => Packet::PubRec(take_u16(body, &mut at)?),
        PUBREL => Packet::PubRel(take_u16(body, &mut at)?),
        PUBCOMP => Packet::PubComp(take_u16(body, &mut at)?),
        SUBACK => {
            let packet_id = take_u16(body, &mut at)?;
            Packet::SubAck {
                packet_id,
                codes: body[at..].to_vec(),
            }
        }
        UNSUBACK => Packet::UnsubAck(take_u16(body, &mut at)?),
        PINGRESP => Packet::PingResp,
        other => return Err(format!("unexpected control packet type {other} from a broker")),
    };
    Ok(Some((packet, total)))
}

// ---------------------------------------------------------------------------
// the broker's side
// ---------------------------------------------------------------------------

/// A CONNECT as a broker reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct Connect {
    pub protocol: String,
    /// 4 is 3.1.1.
    pub level: u8,
    pub clean_session: bool,
    pub keep_alive_s: u16,
    pub client_id: String,
    pub will: Option<ConnectWill>,
    pub username: Option<String>,
    pub password: Option<Vec<u8>>,
}

/// A client's last will, as its CONNECT carries it.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectWill {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
}

/// A packet arriving from a client.
#[derive(Clone, Debug, PartialEq)]
pub enum ClientPacket {
    Connect(Connect),
    Publish {
        dup: bool,
        qos: u8,
        retain: bool,
        topic: String,
        packet_id: u16,
        payload: Vec<u8>,
    },
    PubAck(u16),
    PubRec(u16),
    PubRel(u16),
    PubComp(u16),
    Subscribe {
        packet_id: u16,
        filters: Vec<(String, u8)>,
    },
    Unsubscribe {
        packet_id: u16,
        filters: Vec<String>,
    },
    PingReq,
    Disconnect,
}

fn take_bytes(buf: &[u8], at: &mut usize) -> Result<Vec<u8>, String> {
    if *at + 2 > buf.len() {
        return Err("field length runs past the packet".into());
    }
    let len = u16::from_be_bytes([buf[*at], buf[*at + 1]]) as usize;
    *at += 2;
    if *at + len > buf.len() {
        return Err("field runs past the packet".into());
    }
    let bytes = buf[*at..*at + len].to_vec();
    *at += len;
    Ok(bytes)
}

/// The flags 3.1.1 fixes for a packet type; a client that sends others is closed.
fn flags_must_be(kind: &str, flags: u8, wanted: u8) -> Result<(), String> {
    if flags == wanted {
        Ok(())
    } else {
        Err(format!("{kind} with flags {flags:#x}"))
    }
}

fn decode_connect(body: &[u8]) -> Result<Connect, String> {
    let mut at = 0usize;
    let protocol = take_str(body, &mut at)?;
    let (Some(&level), Some(&flags)) = (body.get(at), body.get(at + 1)) else { return Err("CONNECT ends before its flags".into()) };
    at += 2;
    // Another version lays the rest out differently (MQTT 5 has properties):
    // what it is is enough for the CONNACK that refuses it.
    if protocol != "MQTT" || level != 4 {
        return Ok(Connect { protocol, level, clean_session: false, keep_alive_s: 0, client_id: String::new(), will: None, username: None, password: None });
    }
    if flags & 0x01 != 0 {
        return Err("CONNECT sets the reserved flag".into());
    }
    let keep_alive_s = take_u16(body, &mut at)?;
    let client_id = take_str(body, &mut at)?;
    let will = if flags & 0x04 != 0 {
        let topic = take_str(body, &mut at)?;
        let payload = take_bytes(body, &mut at)?;
        let qos = (flags >> 3) & 0x03;
        if qos > 2 {
            return Err("CONNECT claims a will of QoS 3".into());
        }
        Some(ConnectWill { topic, payload, qos, retain: flags & 0x20 != 0 })
    } else {
        None
    };
    let username = if flags & 0x80 != 0 { Some(take_str(body, &mut at)?) } else { None };
    let password = if flags & 0x40 != 0 { Some(take_bytes(body, &mut at)?) } else { None };
    Ok(Connect { protocol, level, clean_session: flags & 0x02 != 0, keep_alive_s, client_id, will, username, password })
}

/// Decode one packet a client sent from the front of `buf`; a packet longer
/// than `max` bytes, or one 3.1.1 forbids, is an error and ends the connection.
pub fn decode_client(buf: &[u8], max: usize) -> Result<Option<(ClientPacket, usize)>, String> {
    let Some((kind, flags, header, total)) = fixed_header(buf, max)? else { return Ok(None) };
    let body = &buf[header..total];
    let mut at = 0usize;
    let packet = match kind {
        CONNECT => {
            flags_must_be("CONNECT", flags, 0)?;
            ClientPacket::Connect(decode_connect(body)?)
        }
        PUBLISH => {
            let qos = (flags >> 1) & 0x03;
            if qos > 2 {
                return Err("PUBLISH claims QoS 3".into());
            }
            let topic = take_str(body, &mut at)?;
            let packet_id = if qos > 0 { take_u16(body, &mut at)? } else { 0 };
            ClientPacket::Publish { dup: flags & 0x08 != 0, qos, retain: flags & 0x01 != 0, topic, packet_id, payload: body[at..].to_vec() }
        }
        PUBACK => ClientPacket::PubAck(take_u16(body, &mut at)?),
        PUBREC => ClientPacket::PubRec(take_u16(body, &mut at)?),
        PUBREL => {
            flags_must_be("PUBREL", flags, 0x02)?;
            ClientPacket::PubRel(take_u16(body, &mut at)?)
        }
        PUBCOMP => ClientPacket::PubComp(take_u16(body, &mut at)?),
        SUBSCRIBE => {
            flags_must_be("SUBSCRIBE", flags, 0x02)?;
            let packet_id = take_u16(body, &mut at)?;
            let mut filters = Vec::new();
            while at < body.len() {
                let filter = take_str(body, &mut at)?;
                let Some(&qos) = body.get(at) else { return Err("SUBSCRIBE ends before a filter's QoS".into()) };
                at += 1;
                if qos > 2 {
                    return Err(format!("SUBSCRIBE asks for QoS {qos}"));
                }
                filters.push((filter, qos));
            }
            if filters.is_empty() {
                return Err("SUBSCRIBE without a filter".into());
            }
            ClientPacket::Subscribe { packet_id, filters }
        }
        UNSUBSCRIBE => {
            flags_must_be("UNSUBSCRIBE", flags, 0x02)?;
            let packet_id = take_u16(body, &mut at)?;
            let mut filters = Vec::new();
            while at < body.len() {
                filters.push(take_str(body, &mut at)?);
            }
            if filters.is_empty() {
                return Err("UNSUBSCRIBE without a filter".into());
            }
            ClientPacket::Unsubscribe { packet_id, filters }
        }
        PINGREQ => ClientPacket::PingReq,
        DISCONNECT => ClientPacket::Disconnect,
        other => return Err(format!("unexpected control packet type {other} from a client")),
    };
    Ok(Some((packet, total)))
}

pub fn encode_connack(session_present: bool, code: u8) -> Vec<u8> {
    frame(CONNACK, 0, vec![u8::from(session_present), code])
}

/// A grant per filter, in order: the QoS given, or 0x80 for a refusal.
pub fn encode_suback(packet_id: u16, codes: &[u8]) -> Vec<u8> {
    let mut body = packet_id.to_be_bytes().to_vec();
    body.extend_from_slice(codes);
    frame(SUBACK, 0, body)
}

pub fn encode_unsuback(packet_id: u16) -> Vec<u8> {
    ack(UNSUBACK, 0, packet_id)
}

pub fn encode_pingresp() -> Vec<u8> {
    frame(PINGRESP, 0, Vec::new())
}

/// Why the broker refused, in words rather than a number.
pub fn connack_reason(code: u8) -> &'static str {
    match code {
        0 => "accepted",
        1 => "the broker refuses protocol level 3.1.1",
        2 => "the client id was rejected",
        3 => "the broker is unavailable",
        4 => "bad username or password",
        5 => "not authorized",
        _ => "refused for an unlisted reason",
    }
}

/// One line for the Inspector's summary column.
pub fn summarize(topic: &str, payload: &[u8], qos: u8, retain: bool) -> String {
    let text = String::from_utf8_lossy(payload);
    let shown: String = text.chars().take(60).collect();
    let mut out = format!("{topic} = {shown}");
    if payload.is_empty() {
        out = format!("{topic} = (empty)");
    }
    if qos > 0 {
        out.push_str(&format!("  qos{qos}"));
    }
    if retain {
        out.push_str("  retained");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SUBACK is the one packet with an opaque, variable-length tail, which makes
    /// it the honest way to check that a body is framed at the right length
    /// across all four groups of the varint.
    #[test]
    fn remaining_length_spans_all_four_groups() {
        for len in [2usize, 3, 127, 128, 16_383, 16_384, 2_097_151, 2_097_152] {
            let mut buf = vec![SUBACK << 4];
            put_remaining(&mut buf, len);
            let header = buf.len();
            buf.resize(header + len, 0);
            let (packet, used) = decode(&buf).unwrap().unwrap();
            assert_eq!(used, header + len, "len {len} consumed the wrong span");
            match packet {
                Packet::SubAck { codes, .. } => assert_eq!(codes.len(), len - 2, "len {len}"),
                other => panic!("decoded as {other:?}"),
            }
        }
    }

    #[test]
    fn connect_matches_the_wire_format() {
        let bytes = encode_connect(&ConnectOpts {
            client_id: "cid",
            username: None,
            password: None,
            keep_alive_s: 60,
            clean_session: true,
            will: None,
        });
        assert_eq!(bytes[0], 0x10, "CONNECT type and flags");
        // "MQTT" + level + flags + keepalive + client id
        assert_eq!(&bytes[2..8], b"\x00\x04MQTT");
        assert_eq!(bytes[8], 0x04, "protocol level 3.1.1");
        assert_eq!(bytes[9], 0x02, "clean session only");
        assert_eq!(&bytes[10..12], &[0x00, 0x3C], "keepalive 60");
        assert_eq!(&bytes[12..17], b"\x00\x03cid");
        assert_eq!(bytes.len(), 2 + bytes[1] as usize);
    }

    #[test]
    fn a_password_without_a_username_is_not_sent() {
        let bytes = encode_connect(&ConnectOpts {
            client_id: "c",
            username: None,
            password: Some("secret"),
            keep_alive_s: 15,
            clean_session: true,
            will: None,
        });
        assert_eq!(bytes[9] & 0xC0, 0, "neither credential flag may be set");
        assert!(!String::from_utf8_lossy(&bytes).contains("secret"));
    }

    #[test]
    fn will_flags_carry_qos_and_retain() {
        let will = Will {
            topic: "z/p/status/online".into(),
            payload: "off".into(),
            qos: 2,
            retain: true,
        };
        let bytes = encode_connect(&ConnectOpts {
            client_id: "c",
            username: None,
            password: None,
            keep_alive_s: 60,
            clean_session: true,
            will: Some(&will),
        });
        let flags = bytes[9];
        assert_eq!(flags & 0x04, 0x04, "will flag");
        assert_eq!((flags >> 3) & 0x03, 2, "will qos");
        assert_eq!(flags & 0x20, 0x20, "will retain");
        assert!(String::from_utf8_lossy(&bytes).contains("z/p/status/online"));
    }

    #[test]
    fn publish_round_trips_with_qos_and_retain() {
        let bytes = encode_publish("global/theme", b"night", 2, true, 0x1234, false);
        let (packet, used) = decode(&bytes).unwrap().unwrap();
        assert_eq!(used, bytes.len());
        match packet {
            Packet::Publish {
                qos,
                retain,
                topic,
                packet_id,
                payload,
                dup,
            } => {
                assert_eq!((qos, retain, dup), (2, true, false));
                assert_eq!(topic, "global/theme");
                assert_eq!(packet_id, 0x1234);
                assert_eq!(payload, b"night");
            }
            other => panic!("decoded as {other:?}"),
        }
    }

    #[test]
    fn qos0_publish_carries_no_packet_id() {
        let bytes = encode_publish("t", b"", 0, false, 7, false);
        match decode(&bytes).unwrap().unwrap().0 {
            Packet::Publish { packet_id, qos, .. } => assert_eq!((qos, packet_id), (0, 0)),
            other => panic!("decoded as {other:?}"),
        }
    }

    #[test]
    fn a_partial_packet_asks_for_more_instead_of_failing() {
        let bytes = encode_publish("some/topic", b"payload", 1, false, 9, false);
        for cut in 1..bytes.len() {
            assert_eq!(decode(&bytes[..cut]).unwrap(), None, "cut at {cut} should wait");
        }
        assert!(decode(&bytes).unwrap().is_some());
    }

    #[test]
    fn two_packets_in_one_read_are_taken_one_at_a_time() {
        let mut buf = encode_publish("a", b"1", 0, false, 0, false);
        buf.extend_from_slice(&encode_pingreq());
        let (first, used) = decode(&buf).unwrap().unwrap();
        assert!(matches!(first, Packet::Publish { .. }));
        // PINGREQ is a client packet, so decoding it as a server packet must be
        // an error rather than silently accepted — the framing is what matters.
        assert!(decode(&buf[used..]).is_err());
    }

    #[test]
    fn subscribe_sets_the_reserved_flag_and_grants_come_back() {
        let bytes = encode_subscribe(1, &[("+/+/define_application".into(), 2), ("#".into(), 0)]);
        assert_eq!(bytes[0] & 0x0F, 0x02, "SUBSCRIBE needs flags 0x02");
        let suback = {
            let mut body = vec![0x00, 0x01, 0x02, 0x00];
            let mut out = vec![SUBACK << 4];
            put_remaining(&mut out, body.len());
            out.append(&mut body);
            out
        };
        match decode(&suback).unwrap().unwrap().0 {
            Packet::SubAck { packet_id, codes } => {
                assert_eq!(packet_id, 1);
                assert_eq!(codes, vec![2, 0], "granted qos per filter, in order");
            }
            other => panic!("decoded as {other:?}"),
        }
    }

    #[test]
    fn pubrel_keeps_its_reserved_flag() {
        assert_eq!(encode_pubrel(5)[0] & 0x0F, 0x02);
        assert_eq!(decode(&encode_pubrel(5)).unwrap().unwrap().0, Packet::PubRel(5));
    }

    #[test]
    fn connack_refusals_are_explained() {
        let bytes = vec![CONNACK << 4, 0x02, 0x00, 0x04];
        match decode(&bytes).unwrap().unwrap().0 {
            Packet::ConnAck { code, .. } => assert_eq!(connack_reason(code), "bad username or password"),
            other => panic!("decoded as {other:?}"),
        }
    }

    #[test]
    fn an_empty_payload_reads_as_empty_not_as_nothing() {
        assert!(summarize("t/x", b"", 0, true).contains("(empty)"));
        assert!(summarize("t/x", b"", 0, true).contains("retained"));
    }

    #[test]
    fn a_broker_reads_what_the_client_side_writes() {
        let will = Will { topic: "lab/a/online".into(), payload: "false".into(), qos: 1, retain: true };
        let connect = encode_connect(&ConnectOpts { client_id: "dev-1", username: Some("lab"), password: Some("pw"), keep_alive_s: 30, clean_session: true, will: Some(&will) });
        let (packet, used) = decode_client(&connect, 1024).unwrap().unwrap();
        assert_eq!(used, connect.len());
        let ClientPacket::Connect(read) = packet else { panic!("{packet:?}") };
        assert_eq!((read.protocol.as_str(), read.level, read.clean_session, read.keep_alive_s, read.client_id.as_str()), ("MQTT", 4, true, 30, "dev-1"));
        assert_eq!((read.username.as_deref(), read.password.as_deref()), (Some("lab"), Some(&b"pw"[..])));
        assert_eq!(read.will, Some(ConnectWill { topic: "lab/a/online".into(), payload: b"false".to_vec(), qos: 1, retain: true }));

        let publish = encode_publish("lab/a/cmd", b"ON", 2, true, 9, false);
        assert_eq!(decode_client(&publish, 1024).unwrap().unwrap().0, ClientPacket::Publish { dup: false, qos: 2, retain: true, topic: "lab/a/cmd".into(), packet_id: 9, payload: b"ON".to_vec() });
        let subscribe = encode_subscribe(3, &[("lab/+/state".into(), 1), ("#".into(), 0)]);
        assert_eq!(decode_client(&subscribe, 1024).unwrap().unwrap().0, ClientPacket::Subscribe { packet_id: 3, filters: vec![("lab/+/state".into(), 1), ("#".into(), 0)] });
        assert_eq!(decode_client(&encode_unsubscribe(4, &["#".into()]), 1024).unwrap().unwrap().0, ClientPacket::Unsubscribe { packet_id: 4, filters: vec!["#".into()] });
        assert_eq!(decode_client(&encode_pubrel(5), 1024).unwrap().unwrap().0, ClientPacket::PubRel(5));
        assert_eq!(decode_client(&encode_pingreq(), 1024).unwrap().unwrap().0, ClientPacket::PingReq);
        assert_eq!(decode_client(&encode_disconnect(), 1024).unwrap().unwrap().0, ClientPacket::Disconnect);
        for cut in 1..publish.len() {
            assert_eq!(decode_client(&publish[..cut], 1024).unwrap(), None, "cut at {cut} waits");
        }
    }

    #[test]
    fn a_broker_refuses_what_3_1_1_forbids() {
        assert!(decode_client(&encode_publish("t", &[0; 64], 0, false, 0, false), 16).unwrap_err().contains("claims"), "longer than the broker takes");
        let mut subscribe = encode_subscribe(1, &[("t".into(), 1)]);
        subscribe[0] &= 0xF0;
        assert!(decode_client(&subscribe, 1024).is_err(), "SUBSCRIBE without its 0x02");
        let mut qos3 = encode_subscribe(1, &[("t".into(), 1)]);
        *qos3.last_mut().unwrap() = 3;
        assert!(decode_client(&qos3, 1024).is_err());
        assert!(decode_client(&[SUBSCRIBE << 4 | 0x02, 2, 0, 1], 1024).is_err(), "no filter");
        assert!(decode_client(&[CONNACK << 4, 2, 0, 0], 1024).is_err(), "a broker's packet from a client");
    }

    #[test]
    fn what_a_broker_answers_is_what_a_client_reads() {
        assert_eq!(decode(&encode_connack(false, 4)).unwrap().unwrap().0, Packet::ConnAck { session_present: false, code: 4 });
        assert_eq!(decode(&encode_suback(7, &[1, 0x80])).unwrap().unwrap().0, Packet::SubAck { packet_id: 7, codes: vec![1, 0x80] });
        assert_eq!(decode(&encode_unsuback(8)).unwrap().unwrap().0, Packet::UnsubAck(8));
        assert_eq!(decode(&encode_pingresp()).unwrap().unwrap().0, Packet::PingResp);
    }
}
