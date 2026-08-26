//! Minimal, self-contained MQTT 3.1.1 codec (no external crates).
//!
//! Scoped to what a lab client needs and what field gear actually speaks: plain
//! TCP, QoS 0/1/2, retain, wildcards, a last-will, and a clean session. No
//! MQTT 5, no persistent sessions, no TLS.
//!
//! Decoding covers the server-to-client direction; encoding covers the other.
//! Everything is length-prefixed, so the decoder hands back `None` until a whole
//! packet has arrived rather than guessing.

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

/// Decode one packet from the front of `buf`.
///
/// `Ok(None)` means "not all here yet, keep reading" — the only correct answer
/// for a stream, and the reason the caller keeps a persistent buffer.
pub fn decode(buf: &[u8]) -> Result<Option<(Packet, usize)>, String> {
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
    if remaining > MAX_REMAINING {
        return Err(format!("packet claims {remaining} bytes"));
    }
    let total = header + remaining;
    if buf.len() < total {
        return Ok(None);
    }
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
}
