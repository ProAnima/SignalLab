//! Which HTTP requests something takes: a method (`ANY`, and a GET answers
//! HEAD), a path pattern with `:name` segments and a final `*`, and
//! conditions on headers, query, body and JSON. A route's and a *Wait for HTTP
//! request*'s, so both read a request alike.

use serde_json::Value;

use super::emulator::{Condition, ConditionOn, MAX_CONDITIONS, MAX_PATH};
use super::emulator_rules::{required, too_long};
use super::error::{EngineError, EngineResult, Field};
use super::matching::{self, CompareOp, Datagram, Matcher};
use super::template::{self, Segment};

pub(crate) fn header_name_valid(name: &str) -> bool {
    hyper::header::HeaderName::from_bytes(name.trim().as_bytes()).is_ok()
}

/// `ANY` (or nothing) is every method.
fn method_of(method: &str) -> EngineResult<Option<String>> {
    let method = method.trim().to_ascii_uppercase();
    if method.is_empty() || method == "ANY" {
        return Ok(None);
    }
    hyper::Method::from_bytes(method.as_bytes())
        .map(|_| Some(method.clone()))
        .map_err(|_| EngineError::new("node.method_invalid").with("value", method).in_field(Field::new("method")))
}

/// An HTTP path pattern: literal segments, `:name` segments, and a final `*`.
#[derive(Clone, Debug, PartialEq)]
pub struct PathPattern {
    segments: Vec<PathSegment>,
    rest: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum PathSegment {
    Literal(String),
    Param(String),
}

/// Path segments without the leading slash; one trailing slash means nothing.
fn path_segments(path: &str) -> Vec<&str> {
    match path.strip_suffix('/') {
        Some("") | None if path.len() <= 1 => Vec::new(),
        Some(trimmed) => trimmed.split('/').skip(1).collect(),
        None => path.split('/').skip(1).collect(),
    }
}

impl PathPattern {
    pub fn parse(pattern: &str) -> EngineResult<PathPattern> {
        let field = || Field::new("path");
        if pattern.len() > MAX_PATH {
            return Err(too_long(field(), MAX_PATH));
        }
        if !pattern.starts_with('/') {
            return Err(EngineError::new("emulator.path_slash").with("value", pattern).in_field(field()));
        }
        let parts = path_segments(pattern);
        let mut segments = Vec::new();
        let mut rest = false;
        for (index, part) in parts.iter().enumerate() {
            if *part == "*" {
                if index + 1 != parts.len() {
                    return Err(EngineError::new("emulator.path_rest").with("value", pattern).in_field(field()));
                }
                rest = true;
            } else if let Some(name) = part.strip_prefix(':') {
                if !template::is_ident(name) {
                    return Err(EngineError::new("emulator.path_param").with("value", part).in_field(field()));
                }
                segments.push(PathSegment::Param(name.to_string()));
            } else if part.contains('*') {
                return Err(EngineError::new("emulator.path_rest").with("value", pattern).in_field(field()));
            } else {
                segments.push(PathSegment::Literal(part.to_string()));
            }
        }
        Ok(PathPattern { segments, rest })
    }

    /// The named segments of `path`, decoded, when it matches.
    pub fn matches(&self, path: &str) -> Option<Value> {
        let parts = path_segments(path);
        let fits = if self.rest { parts.len() >= self.segments.len() } else { parts.len() == self.segments.len() };
        if !fits {
            return None;
        }
        let mut params = serde_json::Map::new();
        for (segment, part) in self.segments.iter().zip(&parts) {
            match segment {
                PathSegment::Literal(literal) if literal == part => {}
                PathSegment::Literal(_) => return None,
                PathSegment::Param(name) => {
                    params.insert(name.clone(), Value::String(percent_decode(part, false)));
                }
            }
        }
        Some(Value::Object(params))
    }
}

/// `%41` → `A` (and `+` → space in a query); a malformed escape stays as written.
pub fn percent_decode(text: &str, plus: bool) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        let hex = |at: usize| bytes.get(at).and_then(|digit| (*digit as char).to_digit(16));
        match byte {
            b'%' => match (hex(index + 1), hex(index + 2)) {
                (Some(high), Some(low)) => {
                    out.push((high * 16 + low) as u8);
                    index += 3;
                    continue;
                }
                _ => out.push(byte),
            },
            b'+' if plus => out.push(b' '),
            _ => out.push(byte),
        }
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A condition with its value resolved and its pattern or path compiled.
#[derive(Clone, Debug)]
pub struct Check {
    on: ConditionOn,
    name: String,
    path: Vec<Segment>,
    op: CompareOp,
    value: String,
    regex: Option<regex::Regex>,
}

impl Check {
    /// `condition` with its name and value already resolved; problems name `field`.
    pub fn new(condition: &Condition, field: Field) -> EngineResult<Check> {
        let name = condition.name.trim().to_string();
        let mut path = Vec::new();
        match condition.on {
            ConditionOn::Header if name.is_empty() => return Err(required(field)),
            ConditionOn::Header if !header_name_valid(&name) => {
                return Err(EngineError::new("node.header_invalid").with("value", &name).in_field(field));
            }
            ConditionOn::Query if name.is_empty() => return Err(required(field)),
            ConditionOn::Json => path = template::parse_json_path(if name.is_empty() { "$" } else { &name }).map_err(|error| error.in_field(field.clone()))?,
            _ => {}
        }
        let regex = match condition.op {
            CompareOp::Matches => Some(matching::compile_regex(&condition.value).map_err(|error| error.in_field(field))?),
            _ => None,
        };
        // Header names arrive in lower case; the request's map is keyed that way.
        let name = if condition.on == ConditionOn::Header { name.to_ascii_lowercase() } else { name };
        Ok(Check { on: condition.on, name, path, op: condition.op, value: condition.value.clone(), regex })
    }

    /// Whether `request` (as templates read it) satisfies it. A comparison
    /// that cannot be made (text against a number) does not hold.
    pub fn holds(&self, request: &Value) -> bool {
        let text = |value: Option<&Value>| value.map(template::value_text).unwrap_or_default();
        let actual = match self.on {
            ConditionOn::Header => text(request["headers"].get(&self.name)),
            ConditionOn::Query => text(request["query"].get(&self.name)),
            ConditionOn::Body => text(request.get("body")),
            ConditionOn::Json => text(template::lookup(&request["json"], &self.path).filter(|value| !value.is_null())),
        };
        match &self.regex {
            Some(regex) => regex.is_match(&actual),
            None => matching::compare(&actual, self.op, &self.value).is_ok_and(|(holds, _)| holds),
        }
    }
}

/// Which HTTP requests something takes: a method, a path and conditions. A
/// route's and a *Wait for HTTP request*'s, so both read a request alike.
#[derive(Clone, Debug)]
pub struct RequestMatcher {
    method: Option<String>,
    path: PathPattern,
    when: Vec<Check>,
}

impl RequestMatcher {
    /// From resolved texts: `path` and the conditions' names and values hold no templates any more.
    pub fn new(method: &str, path: &str, when: &[Condition]) -> EngineResult<RequestMatcher> {
        if when.len() > MAX_CONDITIONS {
            return Err(EngineError::new("emulator.conditions_too_many").with("max", MAX_CONDITIONS).in_field(Field::new("conditions")));
        }
        let when = when.iter().enumerate().map(|(index, condition)| Check::new(condition, Field::nth("condition", index + 1))).collect::<EngineResult<_>>()?;
        Ok(RequestMatcher { method: method_of(method)?, path: PathPattern::parse(path.trim())?, when })
    }

    /// The path's named segments when `request` is one of these.
    pub fn accepts(&self, request: &Value) -> Option<Value> {
        let method = request["method"].as_str()?;
        if let Some(wanted) = &self.method {
            if wanted != method && !(wanted == "GET" && method == "HEAD") {
                return None;
            }
        }
        let params = self.path.matches(request["path"].as_str()?)?;
        self.when.iter().all(|check| check.holds(request)).then_some(params)
    }
}

/// A *Wait for HTTP request* matches what the run's HTTP listener received.
impl Matcher for RequestMatcher {
    fn matches(&self, datagram: &Datagram) -> Option<Value> {
        let request = datagram.request.as_ref()?;
        let params = self.accepts(request)?;
        let mut value = request.clone();
        value["params"] = params;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn path_patterns_name_segments_and_take_the_rest() {
        let pattern = PathPattern::parse("/users/:id/items").unwrap();
        assert_eq!(pattern.matches("/users/7/items"), Some(json!({ "id": "7" })));
        assert_eq!(pattern.matches("/users/7/items/"), Some(json!({ "id": "7" })), "a trailing slash means nothing");
        assert_eq!(pattern.matches("/users/john%20doe/items"), Some(json!({ "id": "john doe" })));
        assert_eq!(pattern.matches("/users/7"), None);
        assert_eq!(pattern.matches("/Users/7/items"), None, "literal segments are exact");
        let rest = PathPattern::parse("/files/*").unwrap();
        assert!(rest.matches("/files").is_some() && rest.matches("/files/a/b/c").is_some() && rest.matches("/other").is_none());
        let root = PathPattern::parse("/").unwrap();
        assert!(root.matches("/").is_some() && root.matches("/x").is_none());
        for (bad, code) in [("users", "emulator.path_slash"), ("/a/*/b", "emulator.path_rest"), ("/a*", "emulator.path_rest"), ("/:1x", "emulator.path_param")] {
            assert_eq!(PathPattern::parse(bad).unwrap_err().code, code, "{bad}");
        }
        assert_eq!(percent_decode("a+b%2Fc%zz", true), "a b/c%zz");
    }

    fn request(method: &str, path: &str) -> Value {
        json!({ "method": method, "path": path, "query": { "page": "2" }, "headers": { "x-key": "abc", "content-type": "application/json" },
                "body": "{\"name\":\"Ada\",\"age\":36}", "json": { "name": "Ada", "age": 36 }, "params": {}, "from": "127.0.0.1:5000" })
    }

    #[test]
    fn requests_match_by_method_path_and_conditions() {
        let condition = |on: &str, name: &str, op: &str, value: &str| serde_json::from_value::<Condition>(json!({ "on": on, "name": name, "op": op, "value": value })).unwrap();
        let matcher = RequestMatcher::new("post", "/users/:id", &[condition("header", "X-Key", "eq", "abc"), condition("json", "$.age", "ge", "18"), condition("query", "page", "eq", "2")]).unwrap();
        assert_eq!(matcher.accepts(&request("POST", "/users/9")), Some(json!({ "id": "9" })));
        assert!(matcher.accepts(&request("GET", "/users/9")).is_none(), "another method");
        let mut young = request("POST", "/users/9");
        young["json"]["age"] = json!(12);
        assert!(matcher.accepts(&young).is_none(), "a condition that does not hold");
        let body = RequestMatcher::new("ANY", "/*", &[condition("body", "", "contains", "Ada")]).unwrap();
        assert!(body.accepts(&request("DELETE", "/x")).is_some());
        let get = RequestMatcher::new("GET", "/users/:id", &[]).unwrap();
        assert!(get.accepts(&request("HEAD", "/users/1")).is_some(), "a GET route answers HEAD");
        let missing = RequestMatcher::new("ANY", "/", &[condition("header", "x-absent", "empty", "")]).unwrap();
        assert!(missing.accepts(&request("GET", "/")).is_some(), "an absent header is empty");
        assert_eq!(RequestMatcher::new("ANY", "/", &[condition("json", "$.[", "eq", "")]).unwrap_err().code, "json_path.invalid");
        assert_eq!(RequestMatcher::new("BAD METHOD", "/", &[]).unwrap_err().code, "node.method_invalid");
    }
}
