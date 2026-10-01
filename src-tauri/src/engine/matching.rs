//! Deciding whether a received datagram is the reply a *Wait* step is waiting
//! for, and what the step then knows about it. Pure: no sockets, no clock —
//! the listener supplies datagrams, the runner supplies resolved rules.
//! Specified in `docs/milestone-4-reactive.md`, section 3.

use std::net::SocketAddr;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::error::{EngineError, EngineResult, Field};
use super::osc_codec::{decode_packet, OscArg};

/// Longest address pattern accepted; addresses are short in practice.
pub const MAX_PATTERN: usize = 512;
/// Highest argument index a rule may name.
pub const MAX_ARG_INDEX: usize = 63;
/// Bytes of a payload shown as hex in a reply value.
const HEX_SHOWN: usize = 1024;
/// Values shown in check results and timeline messages are shortened to this.
const SHOWN: usize = 120;

/// One received datagram.
#[derive(Clone, Debug)]
pub struct Datagram {
    pub bytes: Vec<u8>,
    pub from: SocketAddr,
    pub at: Instant,
}

/// Decides whether a datagram is the awaited reply. On a match it returns the
/// reply value the step writes to its variable (the listener adds `ms`).
pub trait Matcher: Send + Sync {
    fn matches(&self, datagram: &Datagram) -> Option<Value>;
}

// ---- comparisons ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Contains,
    Matches,
    Empty,
    NotEmpty,
}

impl CompareOp {
    fn symbol(self) -> &'static str {
        match self {
            CompareOp::Eq => "=",
            CompareOp::Ne => "≠",
            CompareOp::Lt => "<",
            CompareOp::Le => "≤",
            CompareOp::Gt => ">",
            CompareOp::Ge => "≥",
            CompareOp::Contains => "contains",
            CompareOp::Matches => "matches",
            CompareOp::Empty => "is empty",
            CompareOp::NotEmpty => "is not empty",
        }
    }

    pub fn unary(self) -> bool {
        matches!(self, CompareOp::Empty | CompareOp::NotEmpty)
    }
}

/// A comparison as it was made, with both sides shortened for display. The
/// interface words it in the user's language (`cmp.<op>`); `text` is the
/// English form for reports and logs.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Comparison {
    pub value: String,
    pub op: CompareOp,
    pub expected: String,
}

impl Comparison {
    pub fn text(&self) -> String {
        if self.op.unary() {
            format!("{} {}", self.value, self.op.symbol())
        } else {
            format!("{} {} {}", self.value, self.op.symbol(), self.expected)
        }
    }

    /// Message parameters: `value`, `op` (translated by the interface), `expected`.
    pub fn params(&self) -> Value {
        json!({ "value": self.value, "op": self.op, "expected": self.expected })
    }

    /// The error for a comparison that should have held.
    pub fn failed(&self) -> EngineError {
        let error = EngineError::new("check.value_failed");
        error
            .with("value", &self.value)
            .with("op", serde_json::to_value(self.op).ok().and_then(|op| op.as_str().map(str::to_string)).unwrap_or_default())
            .with("expected", &self.expected)
    }
}

/// Text shortened for a message.
pub fn shorten(text: &str) -> String {
    let short: String = text.chars().take(SHOWN).collect();
    if short.len() < text.len() { format!("{short}…") } else { short }
}

fn shown(text: &str) -> String {
    let short = shorten(text);
    if number(text).is_some() { short } else { format!("“{short}”") }
}

fn number(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok().filter(|value| value.is_finite())
}

/// Whether the comparison holds, and the comparison as made (`401 = 200`).
pub fn compare(value: &str, op: CompareOp, expected: &str) -> EngineResult<(bool, Comparison)> {
    let comparison = Comparison { value: shown(value), op, expected: if op.unary() { String::new() } else { shown(expected) } };
    let numeric = || match (number(value), number(expected)) {
        (Some(a), Some(b)) => Ok((a, b)),
        _ => Err(EngineError::new("compare.not_numbers").with("value", shown(value)).with("expected", shown(expected))),
    };
    let holds = match op {
        CompareOp::Eq | CompareOp::Ne => {
            let equal = match (number(value), number(expected)) {
                (Some(a), Some(b)) => a == b,
                _ => value == expected,
            };
            equal == (op == CompareOp::Eq)
        }
        CompareOp::Lt => numeric().map(|(a, b)| a < b)?,
        CompareOp::Le => numeric().map(|(a, b)| a <= b)?,
        CompareOp::Gt => numeric().map(|(a, b)| a > b)?,
        CompareOp::Ge => numeric().map(|(a, b)| a >= b)?,
        CompareOp::Contains => value.contains(expected),
        CompareOp::Matches => compile_regex(expected)?.is_match(value),
        CompareOp::Empty => value.trim().is_empty(),
        CompareOp::NotEmpty => !value.trim().is_empty(),
    };
    Ok((holds, comparison))
}

// ---- OSC address patterns (OSC 1.0) ------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Char(char),
    /// `?`: any one character.
    One,
    /// `*`: any run of characters, possibly empty.
    Many,
    /// `[a-z]`, `[!0-9]`.
    Set { negated: bool, ranges: Vec<(char, char)> },
    /// `{ping,pong}`.
    Either(Vec<Vec<char>>),
}

/// A parsed address pattern. Wildcards never cross a `/`.
#[derive(Clone, Debug, PartialEq)]
pub struct OscPattern {
    segments: Vec<Vec<Token>>,
}

/// Characters OSC does not allow in an address, wildcards aside.
fn forbidden(c: char) -> bool {
    c == ' ' || c == '#' || c.is_control() || !c.is_ascii()
}

impl OscPattern {
    pub fn parse(pattern: &str) -> EngineResult<OscPattern> {
        if pattern.len() > MAX_PATTERN {
            return Err(EngineError::new("osc.pattern_too_long").with("pattern", pattern).with("max", MAX_PATTERN));
        }
        let Some(body) = pattern.strip_prefix('/') else {
            return Err(EngineError::new("osc.pattern_slash").with("pattern", pattern));
        };
        let mut segments = Vec::new();
        let mut position = 1; // 1-based, counting the leading slash
        for segment in body.split('/') {
            if segment.is_empty() {
                return Err(EngineError::new("osc.pattern_empty_segment").with("pattern", pattern).with("position", position + 1));
            }
            segments.push(Self::segment(pattern, segment, position + 1)?);
            position += segment.chars().count() + 1;
        }
        Ok(OscPattern { segments })
    }

    fn segment(pattern: &str, segment: &str, start: usize) -> EngineResult<Vec<Token>> {
        let chars: Vec<char> = segment.chars().collect();
        let mut tokens = Vec::new();
        let mut i = 0;
        let at = |i: usize| start + i;
        while i < chars.len() {
            let c = chars[i];
            match c {
                '?' => tokens.push(Token::One),
                '*' => {
                    // `**` means the same as `*`; keep one so matching stays linear-ish.
                    if tokens.last() != Some(&Token::Many) {
                        tokens.push(Token::Many);
                    }
                }
                '[' => {
                    let close = chars[i + 1..].iter().position(|&c| c == ']').map(|offset| i + 1 + offset);
                    let Some(close) = close else {
                        return Err(EngineError::new("osc.pattern_unclosed").with("pattern", pattern).with("position", at(i)));
                    };
                    let mut inner = &chars[i + 1..close];
                    let negated = inner.first() == Some(&'!');
                    if negated {
                        inner = &inner[1..];
                    }
                    if inner.is_empty() {
                        return Err(EngineError::new("osc.pattern_empty_set").with("pattern", pattern).with("position", at(i)));
                    }
                    let mut ranges = Vec::new();
                    let mut k = 0;
                    while k < inner.len() {
                        let low = inner[k];
                        if forbidden(low) || matches!(low, '[' | '{' | '}' | '*' | '?') {
                            return Err(EngineError::new("osc.pattern_char").with("pattern", pattern).with("char", low).with("position", at(i)));
                        }
                        // `-` between two characters is a range; at either end it is itself.
                        if k + 2 < inner.len() && inner[k + 1] == '-' {
                            let high = inner[k + 2];
                            if high < low {
                                return Err(EngineError::new("osc.pattern_range").with("pattern", pattern).with("range", format!("{low}-{high}")));
                            }
                            ranges.push((low, high));
                            k += 3;
                        } else {
                            ranges.push((low, low));
                            k += 1;
                        }
                    }
                    tokens.push(Token::Set { negated, ranges });
                    i = close;
                }
                '{' => {
                    let close = chars[i + 1..].iter().position(|&c| c == '}').map(|offset| i + 1 + offset);
                    let Some(close) = close else {
                        return Err(EngineError::new("osc.pattern_unclosed").with("pattern", pattern).with("position", at(i)));
                    };
                    let inner = &chars[i + 1..close];
                    if let Some(bad) = inner.iter().find(|&&c| forbidden(c) || matches!(c, '{' | '[' | ']' | '*' | '?')) {
                        return Err(EngineError::new("osc.pattern_char").with("pattern", pattern).with("char", bad).with("position", at(i)));
                    }
                    let options: Vec<Vec<char>> = inner.split(|&c| c == ',').map(<[char]>::to_vec).collect();
                    tokens.push(Token::Either(options));
                    i = close;
                }
                ']' | '}' | ',' => {
                    return Err(EngineError::new("osc.pattern_char").with("pattern", pattern).with("char", c).with("position", at(i)));
                }
                c if forbidden(c) => {
                    let shown = if c == ' ' { "␠".to_string() } else { c.escape_default().to_string() };
                    return Err(EngineError::new("osc.pattern_char").with("pattern", pattern).with("char", shown).with("position", at(i)));
                }
                c => tokens.push(Token::Char(c)),
            }
            i += 1;
        }
        Ok(tokens)
    }

    pub fn matches(&self, address: &str) -> bool {
        let Some(body) = address.strip_prefix('/') else { return false };
        let parts: Vec<&str> = body.split('/').collect();
        parts.len() == self.segments.len()
            && parts.iter().zip(&self.segments).all(|(part, tokens)| {
                let chars: Vec<char> = part.chars().collect();
                matches_tokens(tokens, &chars)
            })
    }
}

fn matches_tokens(tokens: &[Token], text: &[char]) -> bool {
    let Some((first, rest)) = tokens.split_first() else {
        return text.is_empty();
    };
    match first {
        Token::Char(c) => text.first() == Some(c) && matches_tokens(rest, &text[1..]),
        Token::One => !text.is_empty() && matches_tokens(rest, &text[1..]),
        Token::Many => (0..=text.len()).any(|skip| matches_tokens(rest, &text[skip..])),
        Token::Set { negated, ranges } => match text.first() {
            Some(c) => ranges.iter().any(|(low, high)| (low..=high).contains(&c)) != *negated && matches_tokens(rest, &text[1..]),
            None => false,
        },
        Token::Either(options) => options
            .iter()
            .any(|option| text.starts_with(option) && matches_tokens(rest, &text[option.len()..])),
    }
}

// ---- argument rules -------------------------------------------------------------

/// `args[index] <op> value`. The value is a template, resolved when the wait runs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArgRule {
    pub index: usize,
    pub op: CompareOp,
    #[serde(default)]
    pub value: String,
}

/// A rule with its value resolved and its pattern compiled.
struct Rule {
    index: usize,
    op: CompareOp,
    value: String,
    regex: Option<regex::Regex>,
}

/// A literal regular expression, or the error for it.
pub fn compile_regex(pattern: &str) -> EngineResult<regex::Regex> {
    regex::Regex::new(pattern).map_err(|error| EngineError::new("regex.invalid").with("pattern", pattern).because(error))
}

impl Rule {
    fn new(rule: &ArgRule) -> EngineResult<Rule> {
        let regex = match rule.op {
            CompareOp::Matches => Some(compile_regex(&rule.value)?),
            _ => None,
        };
        Ok(Rule { index: rule.index, op: rule.op, value: rule.value.clone(), regex })
    }

    /// A comparison that cannot be made (text against a number) does not hold:
    /// the message is simply not the one awaited.
    fn holds(&self, args: &[OscArg]) -> bool {
        let Some(arg) = args.get(self.index) else { return false };
        let text = arg_text(arg);
        match &self.regex {
            Some(regex) => regex.is_match(&text),
            None => compare(&text, self.op, &self.value).is_ok_and(|(holds, _)| holds),
        }
    }
}

/// How an argument reads in a comparison: strings without quotes.
fn arg_text(arg: &OscArg) -> String {
    match arg {
        OscArg::Int(v) => v.to_string(),
        OscArg::Float(v) => v.to_string(),
        OscArg::Long(v) => v.to_string(),
        OscArg::Double(v) => v.to_string(),
        OscArg::Str(v) => v.clone(),
        OscArg::Bool(v) => v.to_string(),
        OscArg::Blob(v) => hex(v, usize::MAX),
        OscArg::Nil => String::new(),
    }
}

/// An argument as a JSON value; a float keeps its shortest decimal form.
fn arg_value(arg: &OscArg) -> Value {
    match arg {
        OscArg::Int(v) => json!(v),
        OscArg::Float(v) => v.to_string().parse::<f64>().map(Value::from).unwrap_or(Value::Null),
        OscArg::Long(v) => json!(v),
        OscArg::Double(v) => json!(v),
        OscArg::Str(v) => json!(v),
        OscArg::Bool(v) => json!(v),
        OscArg::Blob(v) => json!(hex(v, HEX_SHOWN)),
        OscArg::Nil => Value::Null,
    }
}

pub struct OscMatcher {
    pattern: OscPattern,
    rules: Vec<Rule>,
}

impl OscMatcher {
    /// `rules` carry resolved values; a `matches` rule's pattern is compiled here.
    /// A problem names the field it is in, as validation does.
    pub fn new(pattern: &str, rules: &[ArgRule]) -> EngineResult<OscMatcher> {
        let rules = rules
            .iter()
            .enumerate()
            .map(|(index, rule)| Rule::new(rule).map_err(|error| error.in_field(Field::nth("rule_value", index + 1))))
            .collect::<EngineResult<_>>()?;
        Ok(OscMatcher {
            pattern: OscPattern::parse(pattern).map_err(|error| error.in_field(Field::new("address")))?,
            rules,
        })
    }
}

impl Matcher for OscMatcher {
    /// A bundle matches when one of its messages does; that message is the reply.
    fn matches(&self, datagram: &Datagram) -> Option<Value> {
        let messages = decode_packet(&datagram.bytes).ok()?;
        let message = messages
            .iter()
            .find(|message| self.pattern.matches(&message.address) && self.rules.iter().all(|rule| rule.holds(&message.args)))?;
        Some(json!({
            "address": message.address,
            "args": message.args.iter().map(arg_value).collect::<Vec<_>>(),
            "from": datagram.from.to_string(),
        }))
    }
}

// ---- UDP payloads ---------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UdpMode {
    /// Any datagram.
    #[default]
    Any,
    /// The payload, read as UTF-8, contains the text.
    Contains,
    /// The payload, read as UTF-8, matches the regular expression.
    Regex,
    /// The payload contains the byte sequence.
    Hex,
}

/// `de ad be ef`, `deadbeef`, `0xde,0xad`, `DE:AD` — pairs of hex digits.
pub fn parse_hex(text: &str) -> EngineResult<Vec<u8>> {
    let invalid = || EngineError::new("hex.invalid").with("value", text);
    let mut digits = String::new();
    for token in text.split(|c: char| c.is_whitespace() || matches!(c, ',' | ':' | '-')) {
        let token = token.strip_prefix("0x").or_else(|| token.strip_prefix("0X")).unwrap_or(token);
        digits.push_str(token);
    }
    if digits.is_empty() || !digits.len().is_multiple_of(2) || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    (0..digits.len()).step_by(2).map(|i| u8::from_str_radix(&digits[i..i + 2], 16).map_err(|_| invalid())).collect()
}

/// Bytes as `de ad be ef`, at most `limit` of them.
pub fn hex(bytes: &[u8], limit: usize) -> String {
    let shown: Vec<String> = bytes.iter().take(limit).map(|byte| format!("{byte:02x}")).collect();
    let mut text = shown.join(" ");
    if bytes.len() > limit {
        text.push_str(" …");
    }
    text
}

enum Payload {
    Any,
    Text(String),
    Pattern(regex::Regex),
    Bytes(Vec<u8>),
}

pub struct UdpMatcher {
    payload: Payload,
}

impl UdpMatcher {
    /// `pattern` is resolved; it is compiled or parsed here.
    pub fn new(mode: UdpMode, pattern: &str) -> EngineResult<UdpMatcher> {
        let payload = match mode {
            UdpMode::Any => Payload::Any,
            UdpMode::Contains => Payload::Text(pattern.to_string()),
            UdpMode::Regex => Payload::Pattern(compile_regex(pattern).map_err(|error| error.in_field(Field::new("pattern")))?),
            UdpMode::Hex => Payload::Bytes(parse_hex(pattern).map_err(|error| error.in_field(Field::new("pattern")))?),
        };
        Ok(UdpMatcher { payload })
    }
}

impl Matcher for UdpMatcher {
    fn matches(&self, datagram: &Datagram) -> Option<Value> {
        let bytes = &datagram.bytes;
        let text = String::from_utf8_lossy(bytes);
        let found: Option<String> = match &self.payload {
            Payload::Any => None,
            Payload::Text(needle) => text.contains(needle.as_str()).then(|| needle.clone()),
            Payload::Pattern(regex) => {
                let captures = regex.captures(&text)?;
                Some(captures.get(1).or_else(|| captures.get(0)).map(|m| m.as_str().to_string()).unwrap_or_default())
            }
            Payload::Bytes(needle) => bytes.windows(needle.len()).any(|window| window == needle.as_slice()).then(|| hex(needle, HEX_SHOWN)),
        };
        if found.is_none() && !matches!(self.payload, Payload::Any) {
            return None;
        }
        let mut reply = json!({
            "text": text,
            "hex": hex(bytes, HEX_SHOWN),
            "bytes": bytes.len(),
            "from": datagram.from.to_string(),
        });
        if let Some(found) = found {
            reply["match"] = Value::String(found);
        }
        Some(reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::osc_codec::encode_message;

    fn datagram(bytes: Vec<u8>) -> Datagram {
        Datagram { bytes, from: "127.0.0.1:9000".parse().unwrap(), at: Instant::now() }
    }

    #[test]
    fn osc_patterns_follow_osc_1_0() {
        for (pattern, address, expected) in [
            ("/pong", "/pong", true),
            ("/pong", "/pong/x", false),
            ("/pong", "/Pong", false),
            ("/p?ng", "/pang", true),
            ("/p?ng", "/png", false),
            ("/pong*", "/pong", true),
            ("/pong*", "/pong42", true),
            ("/*", "/a/b", false),
            ("/*/b", "/anything/b", true),
            ("/a*z", "/abcz", true),
            ("/a*z", "/abc", false),
            ("/cue/[0-9]", "/cue/7", true),
            ("/cue/[0-9]", "/cue/x", false),
            ("/cue/[!0-9]", "/cue/x", true),
            ("/cue/[!0-9]", "/cue/7", false),
            ("/cue/[a-c-]", "/cue/-", true),
            ("/{ping,pong}", "/pong", true),
            ("/{ping,pong}", "/pung", false),
            ("/{ping,pong}/*", "/ping/ok", true),
            ("/x{,s}", "/x", true),
            ("/x{,s}", "/xs", true),
            ("/**", "/abc", true),
        ] {
            assert_eq!(OscPattern::parse(pattern).unwrap().matches(address), expected, "{pattern} vs {address}");
        }
    }

    #[test]
    fn malformed_patterns_name_the_problem() {
        for (pattern, code) in [
            ("pong", "osc.pattern_slash"),
            ("", "osc.pattern_slash"),
            ("/a//b", "osc.pattern_empty_segment"),
            ("/a/", "osc.pattern_empty_segment"),
            ("/cue/[0-9", "osc.pattern_unclosed"),
            ("/{a,b", "osc.pattern_unclosed"),
            ("/[]", "osc.pattern_empty_set"),
            ("/[z-a]", "osc.pattern_range"),
            ("/a b", "osc.pattern_char"),
            ("/a#b", "osc.pattern_char"),
            ("/a,b", "osc.pattern_char"),
            ("/a]", "osc.pattern_char"),
            ("/{a{b}}", "osc.pattern_char"),
            ("/кю", "osc.pattern_char"),
        ] {
            let error = OscPattern::parse(pattern).unwrap_err();
            assert_eq!(error.code, code, "{pattern}");
            assert_eq!(error.params["pattern"], pattern);
        }
        assert!(OscPattern::parse(&format!("/{}", "a".repeat(MAX_PATTERN))).unwrap_err().is("osc.pattern_too_long"));
        assert_eq!(OscPattern::parse("/a/[x").unwrap_err().params["position"], "4");
    }

    #[test]
    fn osc_replies_match_address_and_argument_rules() {
        let packet = encode_message("/pong", &[OscArg::Int(42), OscArg::Str("ok".into()), OscArg::Float(0.5)]);
        let rule = |index, op, value: &str| ArgRule { index, op, value: value.into() };
        let reply = OscMatcher::new("/pong*", &[rule(0, CompareOp::Eq, "42"), rule(1, CompareOp::Matches, "^o")])
            .unwrap()
            .matches(&datagram(packet.clone()))
            .unwrap();
        assert_eq!(reply, json!({ "address": "/pong", "args": [42, "ok", 0.5], "from": "127.0.0.1:9000" }));
        for rules in [
            vec![rule(0, CompareOp::Gt, "42")],
            vec![rule(1, CompareOp::Lt, "5")],
            vec![rule(3, CompareOp::Empty, "")],
            vec![rule(1, CompareOp::Eq, "OK")],
        ] {
            assert!(OscMatcher::new("/pong", &rules).unwrap().matches(&datagram(packet.clone())).is_none(), "{rules:?}");
        }
        assert!(OscMatcher::new("/ping", &[]).unwrap().matches(&datagram(packet)).is_none());
        assert!(OscMatcher::new("/pong", &[]).unwrap().matches(&datagram(b"not osc".to_vec())).is_none());
        assert!(OscMatcher::new("/pong", &[rule(0, CompareOp::Matches, "(")]).err().unwrap().is("regex.invalid"));
    }

    #[test]
    fn udp_modes_and_hex() {
        let packet = datagram(b"PONG 42\x00\xff".to_vec());
        let reply = UdpMatcher::new(UdpMode::Any, "").unwrap().matches(&packet).unwrap();
        assert_eq!((reply["bytes"].as_u64(), reply["hex"].as_str()), (Some(9), Some("50 4f 4e 47 20 34 32 00 ff")));
        assert!(reply.get("match").is_none());
        assert_eq!(UdpMatcher::new(UdpMode::Contains, "NG 4").unwrap().matches(&packet).unwrap()["match"], "NG 4");
        assert!(UdpMatcher::new(UdpMode::Contains, "ping").unwrap().matches(&packet).is_none());
        assert_eq!(UdpMatcher::new(UdpMode::Regex, r"PONG (\d+)").unwrap().matches(&packet).unwrap()["match"], "42");
        assert!(UdpMatcher::new(UdpMode::Regex, r"^\d").unwrap().matches(&packet).is_none());
        assert!(UdpMatcher::new(UdpMode::Hex, "00 ff").unwrap().matches(&packet).is_some());
        assert!(UdpMatcher::new(UdpMode::Hex, "ff 00").unwrap().matches(&packet).is_none());
        assert!(UdpMatcher::new(UdpMode::Regex, "(").err().unwrap().is("regex.invalid"));

        assert_eq!(parse_hex("de ad BE ef").unwrap(), [0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(parse_hex("0xde,0xad").unwrap(), [0xde, 0xad]);
        assert_eq!(parse_hex("DE:AD-01").unwrap(), [0xde, 0xad, 0x01]);
        for bad in ["", "abc", "zz", "de ad b", "0x"] {
            assert!(parse_hex(bad).unwrap_err().is("hex.invalid"), "{bad}");
        }
        assert_eq!(hex(&[1, 2, 3], 2), "01 02 …");
    }
}
