//! Minimal, self-contained OSC 1.0 codec (no external crates).
//!
//! Sends and reads the common argument types used by lighting / AV / show-control gear:
//! int32 `i`, float32 `f`, string `s`, blob `b`, int64 `h`, double `d`,
//! bool `T`/`F`, and nil `N`. Reads the rest of OSC 1.0's types too, so a
//! message carrying one is shown instead of refused: a time tag `t`, a char
//! `c`, an RGBA colour `r`, a MIDI message `m`, Infinitum `I`, the alternate
//! string `S` (a string) and arrays `[` … `]`. A message without a type tag
//! string (an older implementation's) has no arguments, as the specification
//! asks. Bundles (`#bundle`) are decoded by flattening the contained messages,
//! which is what a monitor wants to display.

use serde::{Deserialize, Serialize};

/// A single OSC argument, tagged for clean JSON round-tripping with the UI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum OscArg {
    Int(i32),
    Float(f32),
    Str(String),
    Long(i64),
    Double(f64),
    Bool(bool),
    Blob(Vec<u8>),
    Nil,
    /// `t`: an NTP time tag, seconds since 1900 in the high 32 bits; 1 is
    /// "immediately". As text in JSON, where a number this large loses digits.
    Time(#[serde(with = "time_text")] u64),
    /// `c`: one character, sent as 32 bits.
    Char(char),
    /// `r`: red, green, blue, alpha.
    Color([u8; 4]),
    /// `m`: port id, status byte, data 1, data 2.
    Midi([u8; 4]),
    /// `I`: Infinitum (OSC 1.1's Impulse), no payload.
    Impulse,
    /// `[` … `]`: arguments grouped in an array.
    Array(Vec<OscArg>),
}

/// A time tag as decimal text in JSON.
mod time_text {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Given {
            Text(String),
            Number(u64),
        }
        match Given::deserialize(deserializer)? {
            Given::Text(text) => text.trim().parse().map_err(serde::de::Error::custom),
            Given::Number(number) => Ok(number),
        }
    }
}

impl OscArg {
    fn write_tag(&self, tags: &mut String) {
        let tag = match self {
            OscArg::Int(_) => 'i',
            OscArg::Float(_) => 'f',
            OscArg::Str(_) => 's',
            OscArg::Long(_) => 'h',
            OscArg::Double(_) => 'd',
            OscArg::Bool(true) => 'T',
            OscArg::Bool(false) => 'F',
            OscArg::Blob(_) => 'b',
            OscArg::Nil => 'N',
            OscArg::Time(_) => 't',
            OscArg::Char(_) => 'c',
            OscArg::Color(_) => 'r',
            OscArg::Midi(_) => 'm',
            OscArg::Impulse => 'I',
            OscArg::Array(items) => {
                tags.push('[');
                for item in items {
                    item.write_tag(tags);
                }
                ']'
            }
        };
        tags.push(tag);
    }

    fn write_value(&self, out: &mut Vec<u8>) {
        match self {
            OscArg::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Float(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Double(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Str(v) => write_string(out, v),
            OscArg::Blob(v) => write_blob(out, v),
            OscArg::Time(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Char(c) => out.extend_from_slice(&u32::from(*c).to_be_bytes()),
            OscArg::Color(bytes) | OscArg::Midi(bytes) => out.extend_from_slice(bytes),
            OscArg::Array(items) => items.iter().for_each(|item| item.write_value(out)),
            // T / F / N / I carry no payload
            OscArg::Bool(_) | OscArg::Nil | OscArg::Impulse => {}
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OscMessage {
    pub address: String,
    pub args: Vec<OscArg>,
}

fn pad4(n: usize) -> usize {
    (n + 3) & !3
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(s.as_bytes());
    // trailing null + pad to 4-byte boundary
    let padded = pad4(s.len() + 1);
    out.resize(out.len() + (padded - s.len()), 0);
}

fn write_blob(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as i32).to_be_bytes());
    out.extend_from_slice(b);
    let padded = pad4(b.len());
    out.resize(out.len() + (padded - b.len()), 0);
}

/// Encode a single OSC message into a UDP-ready byte buffer.
pub fn encode_message(address: &str, args: &[OscArg]) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + args.len() * 8);
    write_string(&mut out, address);

    let mut tags = String::with_capacity(args.len() + 1);
    tags.push(',');
    for a in args {
        a.write_tag(&mut tags);
    }
    write_string(&mut out, &tags);

    for a in args {
        a.write_value(&mut out);
    }
    out
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }
    fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }
    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.remaining() < n {
            return Err("unexpected end of OSC packet".into());
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn read_i32(&mut self) -> Result<i32, String> {
        let b = self.read_bytes(4)?;
        Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn read_f32(&mut self) -> Result<f32, String> {
        let b = self.read_bytes(4)?;
        Ok(f32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn read_i64(&mut self) -> Result<i64, String> {
        let b = self.read_bytes(8)?;
        Ok(i64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    fn read_4(&mut self) -> Result<[u8; 4], String> {
        let b = self.read_bytes(4)?;
        Ok([b[0], b[1], b[2], b[3]])
    }
    fn read_f64(&mut self) -> Result<f64, String> {
        let b = self.read_bytes(8)?;
        Ok(f64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    /// Moves past `n` bytes of padding; a packet that ends before its padding
    /// does stops at its end (the next read says what is missing).
    fn skip(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.buf.len());
    }
    fn read_string(&mut self) -> Result<String, String> {
        let rest = self.buf.get(self.pos..).unwrap_or_default();
        let nul = rest
            .iter()
            .position(|&c| c == 0)
            .ok_or("OSC string missing null terminator")?;
        let s = String::from_utf8_lossy(&rest[..nul]).into_owned();
        self.skip(pad4(nul + 1));
        Ok(s)
    }
    fn read_blob(&mut self) -> Result<Vec<u8>, String> {
        let len = usize::try_from(self.read_i32()?).map_err(|_| "negative OSC blob size")?;
        let b = self.read_bytes(len)?.to_vec();
        self.skip(pad4(len) - len);
        Ok(b)
    }
}

/// Decode a UDP OSC packet into a flat list of messages (bundles are flattened).
pub fn decode_packet(buf: &[u8]) -> Result<Vec<OscMessage>, String> {
    let mut out = Vec::new();
    decode_into(buf, &mut out, 0)?;
    Ok(out)
}

/// Bundles inside bundles, at most this deep. A datagram has room for some
/// three thousand levels, each a call deeper — enough to overflow a thread's
/// stack, which ends the whole program, not just the socket that received it.
pub const MAX_BUNDLE_DEPTH: usize = 32;

fn decode_into(buf: &[u8], out: &mut Vec<OscMessage>, depth: usize) -> Result<(), String> {
    if buf.first() == Some(&b'#') {
        if depth >= MAX_BUNDLE_DEPTH {
            return Err(format!("OSC bundles nested deeper than {MAX_BUNDLE_DEPTH}"));
        }
        // #bundle: 8-byte "#bundle\0", 8-byte timetag, then (size,element) pairs
        let mut r = Reader::new(buf);
        let tag = r.read_string()?;
        if tag != "#bundle" {
            return Err(format!("unexpected bundle tag: {tag}"));
        }
        let _timetag = r.read_i64()?;
        while r.remaining() >= 4 {
            let size = usize::try_from(r.read_i32()?).map_err(|_| "negative OSC bundle element size")?;
            let element = r.read_bytes(size)?;
            decode_into(element, out, depth + 1)?;
        }
        Ok(())
    } else {
        out.push(decode_message(buf)?);
        Ok(())
    }
}

/// Arrays inside arrays, at most this deep (a type tag string could open thousands).
const MAX_ARRAY_DEPTH: usize = 16;

fn decode_message(buf: &[u8]) -> Result<OscMessage, String> {
    let mut r = Reader::new(buf);
    let address = r.read_string()?;
    // An OSC packet is a message (its address starts with "/") or a bundle.
    if !address.starts_with('/') {
        return Err(format!("not an OSC address: {:?}", address.chars().take(32).collect::<String>()));
    }
    // OSC 1.0: older implementations omit the type tag string — a message
    // without one has no arguments that can be read.
    if r.buf.get(r.pos) != Some(&b',') {
        // …but the address is still a whole OSC string, padded to four bytes.
        if !r.pos.is_multiple_of(4) {
            return Err("unexpected end of OSC packet".into());
        }
        return Ok(OscMessage { address, args: Vec::new() });
    }
    let tags = r.read_string()?;
    let mut tags = tags.chars().skip(1);
    let args = decode_args(&mut r, &mut tags, 0)?;
    Ok(OscMessage { address, args })
}

/// The arguments the tags name, up to the `]` that closes an array (`depth` > 0) or their end.
fn decode_args(r: &mut Reader, tags: &mut impl Iterator<Item = char>, depth: usize) -> Result<Vec<OscArg>, String> {
    let mut args = Vec::new();
    while let Some(t) = tags.next() {
        let arg = match t {
            'i' => OscArg::Int(r.read_i32()?),
            'f' => OscArg::Float(r.read_f32()?),
            'h' => OscArg::Long(r.read_i64()?),
            'd' => OscArg::Double(r.read_f64()?),
            's' | 'S' => OscArg::Str(r.read_string()?),
            'b' => OscArg::Blob(r.read_blob()?),
            'T' => OscArg::Bool(true),
            'F' => OscArg::Bool(false),
            'N' => OscArg::Nil,
            'I' => OscArg::Impulse,
            't' => OscArg::Time(r.read_i64()? as u64),
            'c' => OscArg::Char(char::from_u32(r.read_i32()? as u32).unwrap_or(char::REPLACEMENT_CHARACTER)),
            'r' => OscArg::Color(r.read_4()?),
            'm' => OscArg::Midi(r.read_4()?),
            '[' => {
                if depth >= MAX_ARRAY_DEPTH {
                    return Err(format!("OSC arrays nested deeper than {MAX_ARRAY_DEPTH}"));
                }
                OscArg::Array(decode_args(r, tags, depth + 1)?)
            }
            ']' if depth > 0 => return Ok(args),
            ']' => return Err("OSC array closed that was never opened".into()),
            other => return Err(format!("unsupported OSC type tag '{other}'")),
        };
        args.push(arg);
    }
    if depth > 0 {
        return Err("OSC array never closed".into());
    }
    Ok(args)
}

/// Render one argument for logs and the Inspector's summary column.
pub fn arg_str(a: &OscArg) -> String {
    match a {
        OscArg::Int(v) => v.to_string(),
        OscArg::Float(v) => v.to_string(),
        OscArg::Long(v) => v.to_string(),
        OscArg::Double(v) => v.to_string(),
        OscArg::Str(v) => format!("\"{v}\""),
        OscArg::Bool(v) => v.to_string(),
        OscArg::Blob(v) => format!("blob[{}]", v.len()),
        OscArg::Nil => "nil".to_string(),
        OscArg::Time(v) => time_str(*v),
        OscArg::Char(c) => format!("'{c}'"),
        OscArg::Color([r, g, b, a]) => format!("#{r:02x}{g:02x}{b:02x}{a:02x}"),
        OscArg::Midi(bytes) => format!("midi[{}]", bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" ")),
        OscArg::Impulse => "impulse".to_string(),
        OscArg::Array(items) => format!("[{}]", items.iter().map(arg_str).collect::<Vec<_>>().join(" ")),
    }
}

/// A time tag as notation: `t:immediate` (1, "immediately" in OSC 1.0), else
/// `t:` and the seconds since 1900 with their fraction.
pub fn time_str(tag: u64) -> String {
    if tag == 1 {
        return "t:immediate".to_string();
    }
    let fraction = (tag & 0xffff_ffff) as f64 / 4_294_967_296.0;
    format!("t:{:.6}", (tag >> 32) as f64 + fraction)
}

/// One-line summary of a decoded packet: the first message, plus a count when a
/// bundle carried more.
pub fn summarize_messages(msgs: &[OscMessage]) -> String {
    let Some(first) = msgs.first() else {
        return "(empty packet)".to_string();
    };
    let args = first.args.iter().map(arg_str).collect::<Vec<_>>().join(" ");
    let head = if args.is_empty() {
        first.address.clone()
    } else {
        format!("{} {}", first.address, args)
    };
    if msgs.len() > 1 {
        format!("{head}  +{} more in bundle", msgs.len() - 1)
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_a_bundle() {
        let msgs = vec![
            OscMessage {
                address: "/a".into(),
                args: vec![OscArg::Int(1)],
            },
            OscMessage {
                address: "/b".into(),
                args: vec![],
            },
        ];
        assert_eq!(summarize_messages(&msgs), "/a 1  +1 more in bundle");
        assert_eq!(summarize_messages(&[]), "(empty packet)");
    }

    /// What a sender can put in a datagram: every one decodes or fails with a
    /// reason, never a panic (a monitor, a wait, an emulator and the Inspector
    /// all decode what arrives).
    #[test]
    fn truncated_packets_fail_instead_of_panicking() {
        let full = encode_message("/abc", &[OscArg::Int(1), OscArg::Str("hello".into()), OscArg::Blob(vec![1, 2, 3])]);
        for end in 0..full.len() {
            let _ = decode_packet(&full[..end]);
        }
        for packet in [&b"/x\0"[..], b"/x\0\0,s", b"/x\0\0,s\0\0ab\0", b"/x\0\0,b\0\0\0\0\0\x05ab", b"#bundle\0", b"#bundle\0\0\0\0\0\0\0\0\0\0\0\0\x03/x\0"] {
            let _ = decode_packet(packet);
        }
        assert!(decode_packet(b"/x\0").is_err());
    }

    /// A bundle inside a bundle inside a bundle… as deep as a datagram allows.
    #[test]
    fn bundles_nested_as_deep_as_a_datagram_allows_are_refused_not_recursed() {
        let mut packet = encode_message("/deep", &[]);
        while packet.len() + 20 <= 65_507 {
            let mut outer = b"#bundle\0".to_vec();
            outer.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
            outer.extend_from_slice(&(packet.len() as i32).to_be_bytes());
            outer.extend_from_slice(&packet);
            packet = outer;
        }
        // On a thread with a tokio worker's stack, as a monitor decodes it.
        let result = std::thread::Builder::new().stack_size(2 << 20).spawn(move || decode_packet(&packet).map(|m| m.len())).unwrap().join().unwrap();
        assert!(result.is_err(), "{result:?}");
    }

    /// The two messages the OSC 1.0 specification spells out byte by byte
    /// ("Examples" under OSC Messages), encoded exactly and read back.
    #[test]
    fn the_specification_s_examples_byte_for_byte() {
        let oscillator: Vec<u8> = [
            &b"/oscillator/4/frequency\0"[..],
            b",f\0\0",
            &[0x43, 0xdc, 0x00, 0x00],
        ]
        .concat();
        assert_eq!(encode_message("/oscillator/4/frequency", &[OscArg::Float(440.0)]), oscillator);
        let read = decode_packet(&oscillator).unwrap();
        assert_eq!((read[0].address.as_str(), read[0].args.clone()), ("/oscillator/4/frequency", vec![OscArg::Float(440.0)]));

        let foo: Vec<u8> = [
            &b"/foo\0\0\0\0"[..],
            b",iisff\0\0",
            &[0x00, 0x00, 0x03, 0xe8],
            &[0xff, 0xff, 0xff, 0xff],
            b"hello\0\0\0",
            &[0x3f, 0x9d, 0xf3, 0xb6],
            &[0x40, 0xb5, 0xb2, 0x2d],
        ]
        .concat();
        let args = vec![OscArg::Int(1000), OscArg::Int(-1), OscArg::Str("hello".into()), OscArg::Float(1.234), OscArg::Float(5.678)];
        assert_eq!(encode_message("/foo", &args), foo);
        assert_eq!(decode_packet(&foo).unwrap()[0].args, args);
    }

    /// Every type tag OSC 1.0 names is read, not refused: before, a message
    /// with a time tag, char, colour, MIDI message or array failed whole
    /// ("unsupported OSC type tag"), and a wait or an emulator rule never saw it.
    #[test]
    fn every_type_tag_of_osc_1_0_is_read() {
        let packet: Vec<u8> = [
            &b"/all\0\0\0\0"[..],
            b",tcrmIhdSTFNb[i[f]]\0",
            &[0, 0, 0, 0, 0, 0, 0, 1],                   // t: immediately
            &[0, 0, 0, 0x41],                            // c: 'A'
            &[0xff, 0x80, 0x00, 0x7f],                   // r
            &[0x01, 0x90, 0x3c, 0x7f],                   // m: port 1, note on 60, velocity 127
            &[0, 0, 0, 1, 0, 0, 0, 0],                   // h: 2^32
            &[0x3f, 0xf8, 0, 0, 0, 0, 0, 0],             // d: 1.5
            b"alt\0",                                    // S
            &[0, 0, 0, 3, 0xde, 0xad, 0xbe, 0],          // b: three bytes, padded
            &[0, 0, 0, 7],                               // [ i
            &[0x3f, 0x00, 0x00, 0x00],                   //   [ f ] ]
        ]
        .concat();
        assert_eq!(packet.len() % 4, 0);
        let read = decode_packet(&packet).unwrap();
        let expected = vec![
            OscArg::Time(1),
            OscArg::Char('A'),
            OscArg::Color([0xff, 0x80, 0x00, 0x7f]),
            OscArg::Midi([0x01, 0x90, 0x3c, 0x7f]),
            OscArg::Impulse,
            OscArg::Long(1 << 32),
            OscArg::Double(1.5),
            OscArg::Str("alt".into()),
            OscArg::Bool(true),
            OscArg::Bool(false),
            OscArg::Nil,
            OscArg::Blob(vec![0xde, 0xad, 0xbe]),
            OscArg::Array(vec![OscArg::Int(7), OscArg::Array(vec![OscArg::Float(0.5)])]),
        ];
        assert_eq!(read[0].args, expected);
        // Written back the same (S is written as s, its equal).
        let again = encode_message("/all", &expected);
        assert_eq!(decode_packet(&again).unwrap()[0].args, expected);
        assert_eq!(again.len(), packet.len());
        assert_eq!(
            summarize_messages(&read),
            "/all t:immediate 'A' #ff80007f midi[01 90 3c 7f] impulse 4294967296 1.5 \"alt\" true false nil blob[3] [7 [0.5]]"
        );
        // JSON as the interface reads it: a time tag as text, its 64 bits kept.
        assert_eq!(serde_json::to_value(OscArg::Time(u64::MAX)).unwrap(), serde_json::json!({ "type": "time", "value": "18446744073709551615" }));
        assert_eq!(serde_json::from_value::<OscArg>(serde_json::json!({ "type": "time", "value": "1" })).unwrap(), OscArg::Time(1));
        assert_eq!(serde_json::to_value(OscArg::Color([1, 2, 3, 4])).unwrap(), serde_json::json!({ "type": "color", "value": [1, 2, 3, 4] }));
        // Arrays that never close, close twice or nest without end fail with a reason.
        for tags in [&b",[i\0"[..], b",]\0\0", b",[[[[[[[[[[[[[[[[[[[[\0\0\0"] {
            let packet = [&b"/a\0\0"[..], tags, &[0u8; 8][..]].concat();
            assert!(decode_packet(&packet).is_err(), "{:?}", String::from_utf8_lossy(tags));
        }
        assert_eq!(time_str(1u64 << 32 | 1u64 << 31), "t:1.500000");
    }

    /// OSC 1.0: "OSC implementations should be robust in the case of a missing
    /// OSC Type Tag String" — before, such a message failed to decode. And a
    /// packet whose address does not start with "/" is no OSC message.
    #[test]
    fn a_message_without_type_tags_has_no_arguments_and_an_address_starts_with_a_slash() {
        let old = decode_packet(b"/go\0").unwrap();
        assert_eq!((old[0].address.as_str(), old[0].args.len()), ("/go", 0));
        let old_with_data = decode_packet(b"/level\0\0\0\0\0\x07").unwrap();
        assert_eq!((old_with_data[0].address.as_str(), old_with_data[0].args.len()), ("/level", 0));
        assert!(decode_packet(b"hello\0\0\0").is_err(), "text with a NUL is not OSC");
        assert!(decode_packet(b"hello\0\0\0,i\0\0\0\0\0\x01").is_err());
        let bundle = [&b"#bundle\0"[..], &[0, 0, 0, 0, 0, 0, 0, 1], &[0, 0, 0, 8], b"nope\0\0\0\0"].concat();
        assert!(decode_packet(&bundle).is_err(), "nor is a bundle element without one");
    }

    #[test]
    fn round_trips_common_types() {
        let args = vec![
            OscArg::Int(42),
            OscArg::Float(1.5),
            OscArg::Str("hi".into()),
            OscArg::Bool(true),
        ];
        let bytes = encode_message("/test", &args);
        assert_eq!(bytes.len() % 4, 0);
        let msgs = decode_packet(&bytes).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].address, "/test");
        assert_eq!(msgs[0].args.len(), 4);
    }
}
