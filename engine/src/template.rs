//! The template language of experiment fields: `{{token}}`, `{{params.api}}`,
//! `{{items[0].id}}`, `{{random_int(1, 100)}}`. Pure — no runtime, no I/O — so
//! the editor preview, *Send now* and the runner resolve a field identically.
//! Specified in `docs/milestone-3-data.md`; errors are `EngineError` codes.

use std::collections::BTreeMap;

use serde_json::Value;

use super::error::{EngineError, EngineResult};

/// Names with a meaning of their own; parameters and variables cannot take them.
pub const RESERVED: &[&str] = &[
    "vars", "params", "secret", "run", "node", "now", "uuid", "counter", "random_int",
    "random_float", "pick",
];

/// A step into a JSON value: `.key`, `["key"]` or `[3]`.
#[derive(Clone, Debug, PartialEq)]
pub enum Segment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Debug, PartialEq)]
enum Func {
    Uuid,
    RandomInt(i64, i64),
    RandomFloat(f64, f64, usize),
    Pick(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
enum Expr {
    /// `head.rest…` — a variable, parameter or built-in value.
    Path { head: String, rest: Vec<Segment> },
    Call(Func),
}

#[derive(Clone, Debug, PartialEq)]
enum Part {
    Text(String),
    Expr { source: String, expr: Expr },
}

/// What a template reads, for static checks before a run.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ref {
    /// `{{name}}`: a variable set on this path, else a parameter.
    Bare(String),
    Param(String),
    Var(String),
    Secret(String),
}

/// A parsed field. Text outside `{{ }}` is kept exactly.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    parts: Vec<Part>,
}

pub fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn is_key_char(c: char) -> bool {
    c == '_' || c == '-' || c.is_ascii_alphanumeric()
}

/// Parse segments after a head: `.key`, `[0]`, `["a b"]`, `['a b']`.
fn parse_segments(mut rest: &str) -> EngineResult<Vec<Segment>> {
    let bad = |text: &str| EngineError::new("template.bad_path").with("text", text);
    let mut segments = Vec::new();
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('.') {
            let end = after.find(|c: char| !is_key_char(c)).unwrap_or(after.len());
            if end == 0 {
                return Err(bad(rest));
            }
            segments.push(Segment::Key(after[..end].to_string()));
            rest = &after[end..];
        } else if let Some(after) = rest.strip_prefix('[') {
            let close = after.find(']').ok_or_else(|| bad(rest))?;
            let inside = after[..close].trim();
            let quoted = inside.len() >= 2
                && ((inside.starts_with('"') && inside.ends_with('"'))
                    || (inside.starts_with('\'') && inside.ends_with('\'')));
            if quoted {
                segments.push(Segment::Key(inside[1..inside.len() - 1].to_string()));
            } else if !inside.is_empty() && inside.bytes().all(|b| b.is_ascii_digit()) {
                segments.push(Segment::Index(inside.parse().map_err(|_| bad(rest))?));
            } else {
                return Err(bad(rest));
            }
            rest = &after[close + 1..];
        } else {
            return Err(bad(rest));
        }
    }
    Ok(segments)
}

/// A JSON path for extraction: `$.token`, `$.items[0].id`, `$["a b"]`; the
/// leading `$.` may be left out (`token`, `items[0]`).
pub fn parse_json_path(path: &str) -> EngineResult<Vec<Segment>> {
    let path = path.trim();
    let rest = match path.strip_prefix('$') {
        Some(rest) => rest.to_string(),
        None if path.starts_with('[') => path.to_string(),
        None => format!(".{path}"),
    };
    parse_segments(&rest).map_err(|_| EngineError::new("json_path.invalid").with("path", path))
}

pub fn lookup<'a>(value: &'a Value, segments: &[Segment]) -> Option<&'a Value> {
    segments.iter().try_fold(value, |current, segment| match segment {
        Segment::Key(key) => current.get(key.as_str()),
        Segment::Index(index) => current.get(*index),
    })
}

pub fn path_text(segments: &[Segment]) -> String {
    let mut text = String::from("$");
    for segment in segments {
        match segment {
            Segment::Key(key) if !key.is_empty() && key.chars().all(is_key_char) => {
                text.push('.');
                text.push_str(key);
            }
            Segment::Key(key) => text.push_str(&format!("[\"{key}\"]")),
            Segment::Index(index) => text.push_str(&format!("[{index}]")),
        }
    }
    text
}

/// Split call arguments on commas outside quotes.
fn split_args(text: &str) -> EngineResult<Vec<String>> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for c in text.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => {
                quote = None;
                current.push(c);
            }
            (Some(_), c) => current.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                current.push(c);
            }
            (None, ',') => args.push(std::mem::take(&mut current)),
            (None, c) => current.push(c),
        }
    }
    if quote.is_some() {
        return Err(EngineError::new("template.arg_quote"));
    }
    args.push(current);
    args.into_iter()
        .map(|arg| {
            let arg = arg.trim();
            let quoted = arg.len() >= 2
                && ((arg.starts_with('"') && arg.ends_with('"'))
                    || (arg.starts_with('\'') && arg.ends_with('\'')));
            if quoted {
                Ok(arg[1..arg.len() - 1].to_string())
            } else if arg.is_empty() {
                Err(EngineError::new("template.arg_empty"))
            } else if arg.chars().all(|c| is_key_char(c) || matches!(c, '.' | ':' | '/' | '+')) {
                Ok(arg.to_string())
            } else {
                Err(EngineError::new("template.arg_invalid").with("value", arg))
            }
        })
        .collect()
}

fn number(arg: &str, function: &str) -> EngineResult<f64> {
    arg.parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .ok_or_else(|| EngineError::new("template.arg_number").with("function", function).with("value", arg))
}

fn integer(arg: &str, function: &str) -> EngineResult<i64> {
    arg.parse::<i64>()
        .map_err(|_| EngineError::new("template.arg_integer").with("function", function).with("value", arg))
}

fn parse_call(name: &str, args: &str) -> EngineResult<Func> {
    let args = split_args(args)?;
    match name {
        "uuid" if args.is_empty() => Ok(Func::Uuid),
        "uuid" => Err(EngineError::new("template.args_uuid")),
        "random_int" if args.len() == 2 => {
            let (min, max) = (integer(&args[0], name)?, integer(&args[1], name)?);
            if min > max {
                return Err(EngineError::new("template.range").with("function", name).with("min", min).with("max", max));
            }
            Ok(Func::RandomInt(min, max))
        }
        "random_int" => Err(EngineError::new("template.args_random_int")),
        "random_float" if (2..=3).contains(&args.len()) => {
            let (min, max) = (number(&args[0], name)?, number(&args[1], name)?);
            if min >= max {
                return Err(EngineError::new("template.range").with("function", name).with("min", min).with("max", max));
            }
            let digits = match args.get(2) {
                Some(arg) => match integer(arg, name)? {
                    digits @ 0..=9 => digits as usize,
                    _ => return Err(EngineError::new("template.digits").with("max", 9)),
                },
                None => 3,
            };
            Ok(Func::RandomFloat(min, max, digits))
        }
        "random_float" => Err(EngineError::new("template.args_random_float")),
        "pick" if !args.is_empty() => Ok(Func::Pick(args)),
        "pick" => Err(EngineError::new("template.args_pick")),
        _ => Err(EngineError::new("template.unknown_function").with("name", name)),
    }
}

fn parse_expr(inner: &str) -> EngineResult<Expr> {
    let inner = inner.trim();
    if inner.is_empty() {
        return Err(EngineError::new("template.empty"));
    }
    if let Some(open) = inner.find('(') {
        let name = inner[..open].trim();
        if !is_ident(name) || !inner.ends_with(')') {
            return Err(EngineError::new("template.bad_call").with("text", inner));
        }
        return parse_call(name, &inner[open + 1..inner.len() - 1]).map(Expr::Call);
    }
    let end = inner.find(|c: char| c != '_' && !c.is_ascii_alphanumeric()).unwrap_or(inner.len());
    let head = &inner[..end];
    if !is_ident(head) {
        return Err(EngineError::new("template.bad_name").with("text", inner));
    }
    let rest = parse_segments(&inner[end..]).map_err(|error| error.with("text", inner))?;
    if head == "uuid" && rest.is_empty() {
        return Ok(Expr::Call(Func::Uuid));
    }
    Ok(Expr::Path { head: head.to_string(), rest })
}

pub fn parse(text: &str) -> EngineResult<Template> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        if rest.starts_with("\\{{") {
            literal.push_str("{{");
            i += 3;
        } else if let Some(inside) = rest.strip_prefix("{{") {
            let position = text[..i].chars().count() + 1;
            let close = inside
                .find("}}")
                .ok_or_else(|| EngineError::new("template.unclosed").with("position", position))?;
            let source = &rest[..close + 4];
            let expr = parse_expr(&inside[..close]).map_err(|error| error.with("position", position))?;
            if !literal.is_empty() {
                parts.push(Part::Text(std::mem::take(&mut literal)));
            }
            parts.push(Part::Expr { source: source.to_string(), expr });
            i += close + 4;
        } else {
            let c = rest.chars().next().unwrap();
            literal.push(c);
            i += c.len_utf8();
        }
    }
    if !literal.is_empty() {
        parts.push(Part::Text(literal));
    }
    Ok(Template { parts })
}

impl Template {
    /// Data this template reads (built-ins and generators are not listed).
    pub fn refs(&self) -> Vec<Ref> {
        let mut refs = Vec::new();
        for part in &self.parts {
            let Part::Expr { expr: Expr::Path { head, rest }, .. } = part else { continue };
            let name = || match rest.first() {
                Some(Segment::Key(name)) => name.clone(),
                _ => String::new(),
            };
            match head.as_str() {
                "params" => refs.push(Ref::Param(name())),
                "vars" => refs.push(Ref::Var(name())),
                "secret" => refs.push(Ref::Secret(name())),
                "run" | "node" | "now" | "counter" => {}
                _ => refs.push(Ref::Bare(head.clone())),
            }
        }
        refs
    }

    /// Reads nothing but parameters, so the value is known before a run.
    pub fn is_static(&self, params: &BTreeMap<String, String>) -> bool {
        self.parts.iter().all(|part| match part {
            Part::Text(_) => true,
            Part::Expr { expr: Expr::Path { head, .. }, .. } if head == "params" => true,
            Part::Expr { expr: Expr::Path { head, .. }, .. } => params.contains_key(head),
            Part::Expr { .. } => false,
        })
    }
}

/// SplitMix64: tiny, fast, and — unlike a library generator — guaranteed to
/// produce the same stream on every platform and every future build.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    /// The stream of one node execution: independent of the order in which
    /// parallel branches happen to run.
    pub fn for_node(seed: u64, node_id: &str, count: u64) -> Self {
        // FNV-1a over the id, then mixed with the seed and the execution count.
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for byte in node_id.bytes() {
            hash = (hash ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        let mut rng = Rng(seed ^ hash.rotate_left(17) ^ count.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        rng.next_u64();
        rng
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in `0..bound` without modulo bias worth measuring.
    fn below(&mut self, bound: u128) -> u128 {
        (self.next_u64() as u128 * bound) >> 64
    }

    /// Uniform in `0..bound`.
    pub fn below_u64(&mut self, bound: u64) -> u64 {
        self.below(bound as u128) as u64
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Everything a template can read while one node executes.
pub struct Scope<'a> {
    pub params: &'a BTreeMap<String, String>,
    pub vars: &'a BTreeMap<String, Value>,
    /// Secret values loaded for this run (or masks, for the preview).
    pub secrets: &'a BTreeMap<String, String>,
    pub run_id: u64,
    pub seed: u64,
    pub node_id: &'a str,
    /// Executions of this node so far in the run, including this one.
    pub count: u64,
    pub now_ms: u64,
}

/// Resolves fields for one node execution. In lenient mode (the editor
/// preview) a name without a value is left as written and listed in
/// `missing`; otherwise it is an error.
pub struct Renderer<'a> {
    scope: Scope<'a>,
    rng: Rng,
    lenient: bool,
    pub missing: Vec<String>,
}

impl<'a> Renderer<'a> {
    pub fn new(scope: Scope<'a>) -> Self {
        let rng = Rng::for_node(scope.seed, scope.node_id, scope.count);
        Renderer { scope, rng, lenient: false, missing: Vec::new() }
    }

    pub fn lenient(scope: Scope<'a>) -> Self {
        Renderer { lenient: true, ..Renderer::new(scope) }
    }

    pub fn render(&mut self, text: &str) -> EngineResult<String> {
        let template = parse(text)?;
        let mut out = String::with_capacity(text.len());
        for part in &template.parts {
            match part {
                Part::Text(text) => out.push_str(text),
                Part::Expr { source, expr } => match self.eval(expr) {
                    Ok(value) => out.push_str(&value_text(&value)),
                    Err(Unresolved::Missing(name)) if self.lenient => {
                        if !self.missing.contains(&name) {
                            self.missing.push(name);
                        }
                        out.push_str(source);
                    }
                    Err(Unresolved::Missing(name)) => return Err(EngineError::new("name.no_value").with("name", name)),
                    Err(Unresolved::Invalid(error)) => return Err(error),
                },
            }
        }
        Ok(out)
    }

    fn eval(&mut self, expr: &Expr) -> Result<Value, Unresolved> {
        match expr {
            Expr::Call(func) => Ok(self.call(func)),
            Expr::Path { head, rest } => {
                let (value, rest, name) = self.head(head, rest)?;
                lookup(&value, rest).cloned().ok_or_else(|| {
                    let path = path_text(rest);
                    Unresolved::Invalid(EngineError::new("template.no_field").with("name", name).with("path", path.trim_start_matches('$')))
                })
            }
        }
    }

    /// The value a path starts from, the segments still to apply and a name for errors.
    fn head<'p>(&self, head: &str, rest: &'p [Segment]) -> Result<(Value, &'p [Segment], String), Unresolved> {
        let scope = &self.scope;
        let named = |rest: &'p [Segment]| match rest.first() {
            Some(Segment::Key(name)) => Ok((name.clone(), &rest[1..])),
            _ => Err(Unresolved::Invalid(EngineError::new("template.needs_name").with("head", head))),
        };
        match head {
            "params" => {
                let (name, rest) = named(rest)?;
                let value = scope.params.get(&name).ok_or(Unresolved::Missing(format!("params.{name}")))?;
                Ok((param_value(value, rest), rest, format!("params.{name}")))
            }
            "vars" => {
                let (name, rest) = named(rest)?;
                let value = scope.vars.get(&name).ok_or(Unresolved::Missing(name.clone()))?;
                Ok((value.clone(), rest, name))
            }
            "secret" => {
                let (name, rest) = named(rest)?;
                let value = scope.secrets.get(&name).ok_or(Unresolved::Missing(format!("secret.{name}")))?;
                Ok((Value::String(value.clone()), rest, format!("secret.{name}")))
            }
            "run" => match rest.first() {
                Some(Segment::Key(key)) if key == "id" => Ok((scope.run_id.into(), &rest[1..], "run.id".into())),
                Some(Segment::Key(key)) if key == "seed" => Ok((scope.seed.into(), &rest[1..], "run.seed".into())),
                _ => Err(Unresolved::Invalid(EngineError::new("template.run_fields"))),
            },
            "node" => match rest.first() {
                Some(Segment::Key(key)) if key == "id" => Ok((scope.node_id.into(), &rest[1..], "node.id".into())),
                _ => Err(Unresolved::Invalid(EngineError::new("template.node_fields"))),
            },
            "now" => match rest.first() {
                None => Ok((scope.now_ms.into(), rest, "now".into())),
                Some(Segment::Key(key)) if key == "iso" && rest.len() == 1 => {
                    Ok((iso_time(scope.now_ms).into(), &rest[1..], "now.iso".into()))
                }
                _ => Err(Unresolved::Invalid(EngineError::new("template.now_fields"))),
            },
            "counter" if rest.is_empty() => Ok((scope.count.into(), rest, "counter".into())),
            name => {
                if let Some(value) = scope.vars.get(name) {
                    Ok((value.clone(), rest, name.into()))
                } else if let Some(value) = scope.params.get(name) {
                    Ok((param_value(value, rest), rest, name.into()))
                } else {
                    Err(Unresolved::Missing(name.into()))
                }
            }
        }
    }

    fn call(&mut self, func: &Func) -> Value {
        match func {
            Func::Uuid => uuid(&mut self.rng).into(),
            Func::RandomInt(min, max) => {
                let span = (*max as i128 - *min as i128 + 1) as u128;
                ((*min as i128 + self.rng.below(span) as i128) as i64).into()
            }
            Func::RandomFloat(min, max, digits) => {
                let value = min + (max - min) * self.rng.unit();
                // Fixed decimals, so the text is what was asked for.
                Value::String(format!("{value:.digits$}"))
            }
            Func::Pick(options) => options[self.rng.below(options.len() as u128) as usize].clone().into(),
        }
    }
}

enum Unresolved {
    /// The name has no value yet (a variable not set, a parameter not defined).
    Missing(String),
    /// The reference itself is wrong.
    Invalid(EngineError),
}

/// A parameter is text; it is read as JSON only when a field of it is asked for.
fn param_value(text: &str, rest: &[Segment]) -> Value {
    if rest.is_empty() {
        return Value::String(text.to_string());
    }
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_string()))
}

/// How a value appears inside a field.
pub fn value_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => "null".into(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        other => other.to_string(),
    }
}

fn uuid(rng: &mut Rng) -> String {
    let (high, low) = (rng.next_u64(), rng.next_u64());
    let high = (high & !0xf000) | 0x4000; // version 4
    let low = (low & !(0b11 << 62)) | (0b10 << 62); // RFC 4122 variant
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        high >> 32,
        (high >> 16) & 0xffff,
        high & 0xffff,
        low >> 48,
        low & 0xffff_ffff_ffff
    )
}

/// `2026-09-30T12:34:56.789Z` from Unix milliseconds (civil-from-days, no dependency).
pub fn iso_time(ms: u64) -> String {
    let (days, day_ms) = ((ms / 86_400_000) as i64, ms % 86_400_000);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        day_ms / 3_600_000,
        day_ms / 60_000 % 60,
        day_ms / 1000 % 60,
        day_ms % 1000
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn params() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("api".to_string(), "http://127.0.0.1:8080".to_string()),
            ("config".to_string(), r#"{"room":"A","ports":[9000,9001]}"#.to_string()),
        ])
    }

    fn vars() -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("token".to_string(), json!("abc123")),
            ("user".to_string(), json!({ "name": "Ada", "tags": ["x", "y"], "first name": "A" })),
            ("count".to_string(), json!(42)),
            ("ratio".to_string(), json!(0.5)),
            ("ok".to_string(), json!(true)),
        ])
    }

    fn render_with(text: &str, seed: u64, node: &str, count: u64) -> EngineResult<String> {
        let (params, vars) = (params(), vars());
        let secrets = BTreeMap::from([("API_TOKEN".to_string(), "t0k3n".to_string())]);
        let scope = Scope { params: &params, vars: &vars, secrets: &secrets, run_id: 7, seed, node_id: node, count, now_ms: 1_790_000_000_123 };
        Renderer::new(scope).render(text)
    }

    fn render(text: &str) -> EngineResult<String> {
        render_with(text, 1, "node", 1)
    }

    #[test]
    fn literal_text_is_kept_exactly() {
        for text in ["", "plain", "}} alone", "{ not a template }", "юникод ✓", r#"{"json": [1, 2]}"#] {
            assert_eq!(render(text).unwrap(), text);
            assert!(parse(text).unwrap().is_static(&BTreeMap::new()));
        }
        assert_eq!(render(r"\{{token}}").unwrap(), "{{token}}");
    }

    #[test]
    fn names_resolve_variables_then_parameters() {
        assert_eq!(render("{{api}}/login?t={{ token }}").unwrap(), "http://127.0.0.1:8080/login?t=abc123");
        assert_eq!(render("{{params.api}}|{{vars.token}}").unwrap(), "http://127.0.0.1:8080|abc123");
        assert_eq!(render("{{user.name}} {{user.tags[1]}} {{user[\"first name\"]}}").unwrap(), "Ada y A");
        assert_eq!(render("{{config.ports[0]}}/{{params.config.room}}").unwrap(), "9000/A");
        assert_eq!(render("{{count}} {{ratio}} {{ok}}").unwrap(), "42 0.5 true");
        assert_eq!(render("{{user.tags}}").unwrap(), r#"["x","y"]"#);
        assert_eq!(render("{{run.id}}:{{run.seed}}:{{node.id}}:{{counter}}").unwrap(), "7:1:node:1");
        assert_eq!(render("{{now}} {{now.iso}}").unwrap(), "1790000000123 2026-09-21T14:13:20.123Z");
        assert_eq!(render("Bearer {{secret.API_TOKEN}}").unwrap(), "Bearer t0k3n");
    }

    #[test]
    fn unknown_names_fail_with_codes_or_stay_visible_in_the_preview() {
        let missing = render("{{missing}}").unwrap_err();
        assert_eq!((missing.code.as_str(), missing.params["name"].as_str()), ("name.no_value", "missing"));
        assert_eq!(render("{{params.nope}}").unwrap_err().params["name"], "params.nope");
        let field = render("{{user.age}}").unwrap_err();
        assert_eq!((field.code.as_str(), field.params["name"].as_str(), field.params["path"].as_str()), ("template.no_field", "user", ".age"));
        assert_eq!(render("{{secret.KEY}}").unwrap_err().params["name"], "secret.KEY");
        assert!(render("{{secret}}").unwrap_err().is("template.needs_name"));
        assert!(render("{{run.x}}").unwrap_err().is("template.run_fields"));
        let (params, vars, secrets) = (params(), BTreeMap::new(), BTreeMap::new());
        let scope = Scope { params: &params, vars: &vars, secrets: &secrets, run_id: 0, seed: 0, node_id: "n", count: 1, now_ms: 0 };
        let mut preview = Renderer::lenient(scope);
        assert_eq!(preview.render("{{api}}/x/{{token}}/{{ token }}").unwrap(), "http://127.0.0.1:8080/x/{{token}}/{{ token }}");
        assert_eq!(preview.missing, ["token"]);
    }

    #[test]
    fn syntax_errors_name_the_cause_and_the_position() {
        for (text, code, position) in [
            ("a {{token", "template.unclosed", 3),
            ("{{}}", "template.empty", 1),
            ("x {{ 1abc }}", "template.bad_name", 3),
            ("{{a.}}", "template.bad_path", 1),
            ("{{a[x]}}", "template.bad_path", 1),
            ("{{nope()}}", "template.unknown_function", 1),
            ("{{random_int(5, 1)}}", "template.range", 1),
            ("{{random_int(1)}}", "template.args_random_int", 1),
            ("{{random_int(a, 2)}}", "template.arg_integer", 1),
            ("{{random_float(1, 1)}}", "template.range", 1),
            ("{{random_float(x, 1)}}", "template.arg_number", 1),
            ("{{random_float(0, 1, 12)}}", "template.digits", 1),
            ("{{uuid(1)}}", "template.args_uuid", 1),
            ("{{pick()}}", "template.args_pick", 1),
            ("{{pick(\"a, b)}}", "template.arg_quote", 1),
            ("{{pick(a,,b)}}", "template.arg_empty", 1),
            ("{{pick(a;b)}}", "template.arg_invalid", 1),
            ("{{f(}}", "template.bad_call", 1),
        ] {
            let error = parse(text).unwrap_err();
            assert_eq!(error.code, code, "{text}");
            assert_eq!(error.params["position"], position.to_string(), "{text}");
        }
    }

    #[test]
    fn generators_are_bounded_and_repeat_with_the_same_seed() {
        let text = "{{uuid}} {{random_int(1, 6)}} {{random_float(0, 1)}} {{pick(red, 'dark blue', 3)}}";
        let first = render_with(text, 99, "a", 1).unwrap();
        assert_eq!(first, render_with(text, 99, "a", 1).unwrap());
        assert_ne!(first, render_with(text, 100, "a", 1).unwrap());
        assert_ne!(first, render_with(text, 99, "b", 1).unwrap());
        assert_ne!(first, render_with(text, 99, "a", 2).unwrap());
        for seed in 0..500 {
            let out = render_with("{{random_int(-2, 2)}}|{{random_float(10, 20, 2)}}|{{pick(a,b)}}|{{uuid()}}", seed, "n", 1).unwrap();
            let fields: Vec<&str> = out.split('|').collect();
            assert!((-2..=2).contains(&fields[0].parse::<i64>().unwrap()));
            let float: f64 = fields[1].parse().unwrap();
            assert!((10.0..20.0).contains(&float) && fields[1].split('.').nth(1).unwrap().len() == 2);
            assert!(fields[2] == "a" || fields[2] == "b");
            let uuid = fields[3];
            assert_eq!(uuid.len(), 36);
            assert_eq!(&uuid[14..15], "4");
            assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"));
        }
        assert!(render("{{random_int(-9223372036854775808, 9223372036854775807)}}").is_ok());
    }

    #[test]
    fn refs_and_static_fields_drive_validation() {
        let template = parse("{{api}}/{{params.path}}/{{vars.id}}/{{uuid}}/{{counter}}/{{secret.KEY}}").unwrap();
        assert_eq!(
            template.refs(),
            [Ref::Bare("api".into()), Ref::Param("path".into()), Ref::Var("id".into()), Ref::Secret("KEY".into())]
        );
        let params = params();
        assert!(parse("{{api}}/login").unwrap().is_static(&params));
        assert!(parse("{{params.api}}").unwrap().is_static(&params));
        assert!(!parse("{{api}}/{{token}}").unwrap().is_static(&params));
        assert!(!parse("{{uuid}}").unwrap().is_static(&params));
    }

    #[test]
    fn json_paths_accept_the_documented_subset() {
        let body = json!({ "token": "t", "items": [{ "id": 5 }], "a b": { "c-d": 1 } });
        for (path, expected) in [
            ("$.token", json!("t")),
            ("token", json!("t")),
            ("$.items[0].id", json!(5)),
            ("items[0].id", json!(5)),
            ("$[\"a b\"].c-d", json!(1)),
            ("$['a b']['c-d']", json!(1)),
            ("$", body.clone()),
        ] {
            let segments = parse_json_path(path).unwrap();
            assert_eq!(lookup(&body, &segments), Some(&expected), "{path}");
        }
        assert_eq!(lookup(&body, &parse_json_path("$.items[3]").unwrap()), None);
        let error = parse_json_path("$.items[").unwrap_err();
        assert_eq!((error.code.as_str(), error.params["path"].as_str()), ("json_path.invalid", "$.items["));
        assert_eq!(path_text(&parse_json_path("$['a b'].items[2].x").unwrap()), "$[\"a b\"].items[2].x");
    }

    #[test]
    fn iso_time_matches_known_instants() {
        assert_eq!(iso_time(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso_time(951_782_400_000), "2000-02-29T00:00:00.000Z");
        assert_eq!(iso_time(4_102_444_799_999), "2099-12-31T23:59:59.999Z");
    }
}
