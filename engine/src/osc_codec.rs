//! Minimal, self-contained OSC 1.0 codec (no external crates).
//!
//! Supports the common argument types used by lighting / AV / show-control gear:
//! int32 `i`, float32 `f`, string `s`, blob `b`, int64 `h`, double `d`,
//! bool `T`/`F`, and nil `N`. Bundles (`#bundle`) are decoded by flattening the
//! contained messages, which is what a monitor wants to display.

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
}

impl OscArg {
    fn tag(&self) -> char {
        match self {
            OscArg::Int(_) => 'i',
            OscArg::Float(_) => 'f',
            OscArg::Str(_) => 's',
            OscArg::Long(_) => 'h',
            OscArg::Double(_) => 'd',
            OscArg::Bool(true) => 'T',
            OscArg::Bool(false) => 'F',
            OscArg::Blob(_) => 'b',
            OscArg::Nil => 'N',
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
        tags.push(a.tag());
    }
    write_string(&mut out, &tags);

    for a in args {
        match a {
            OscArg::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Float(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Double(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Str(v) => write_string(&mut out, v),
            OscArg::Blob(v) => write_blob(&mut out, v),
            // T / F / N carry no payload
            OscArg::Bool(_) | OscArg::Nil => {}
        }
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
    fn read_f64(&mut self) -> Result<f64, String> {
        let b = self.read_bytes(8)?;
        Ok(f64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    fn read_string(&mut self) -> Result<String, String> {
        let start = self.pos;
        let nul = self.buf[start..]
            .iter()
            .position(|&c| c == 0)
            .ok_or("OSC string missing null terminator")?;
        let s = String::from_utf8_lossy(&self.buf[start..start + nul]).into_owned();
        self.pos = start + pad4(nul + 1);
        Ok(s)
    }
    fn read_blob(&mut self) -> Result<Vec<u8>, String> {
        let len = self.read_i32()? as usize;
        let b = self.read_bytes(len)?.to_vec();
        self.pos += pad4(len) - len;
        Ok(b)
    }
}

/// Decode a UDP OSC packet into a flat list of messages (bundles are flattened).
pub fn decode_packet(buf: &[u8]) -> Result<Vec<OscMessage>, String> {
    let mut out = Vec::new();
    decode_into(buf, &mut out)?;
    Ok(out)
}

fn decode_into(buf: &[u8], out: &mut Vec<OscMessage>) -> Result<(), String> {
    if buf.first() == Some(&b'#') {
        // #bundle: 8-byte "#bundle\0", 8-byte timetag, then (size,element) pairs
        let mut r = Reader::new(buf);
        let tag = r.read_string()?;
        if tag != "#bundle" {
            return Err(format!("unexpected bundle tag: {tag}"));
        }
        let _timetag = r.read_i64()?;
        while r.remaining() >= 4 {
            let size = r.read_i32()? as usize;
            let element = r.read_bytes(size)?;
            decode_into(element, out)?;
        }
        Ok(())
    } else {
        out.push(decode_message(buf)?);
        Ok(())
    }
}

fn decode_message(buf: &[u8]) -> Result<OscMessage, String> {
    let mut r = Reader::new(buf);
    let address = r.read_string()?;
    let tags = r.read_string()?;
    let mut args = Vec::new();
    for t in tags.chars().skip(1) {
        let arg = match t {
            'i' => OscArg::Int(r.read_i32()?),
            'f' => OscArg::Float(r.read_f32()?),
            'h' => OscArg::Long(r.read_i64()?),
            'd' => OscArg::Double(r.read_f64()?),
            's' | 'S' => OscArg::Str(r.read_string()?),
            'b' => OscArg::Blob(r.read_blob()?),
            'T' => OscArg::Bool(true),
            'F' => OscArg::Bool(false),
            'N' | 'I' => OscArg::Nil,
            other => return Err(format!("unsupported OSC type tag '{other}'")),
        };
        args.push(arg);
    }
    Ok(OscMessage { address, args })
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
    }
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
