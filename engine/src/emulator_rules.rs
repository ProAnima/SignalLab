//! An emulator checked and compiled: fields fixed when it starts resolved
//! with parameters, patterns compiled, reply templates checked — every
//! problem named by its rule (`rule`), response (`response`) and field before
//! anything listens. Also which response a route's next request gets.

use std::collections::BTreeMap;
use std::net::SocketAddr;

use super::emulator::{
    ArgType, Condition, Delimiter, Emulator, EmulatorKind, MqttOut, MqttRetained, MqttRule, OscOut, OscRule, Order, Outage, Response, Route, TcpRule,
    UdpRule, MAX_ARGS, MAX_DATAGRAM, MAX_DELAY_MS, MAX_HEADERS, MAX_NAME, MAX_OUTAGE_MS, MAX_RESPONSES, MAX_RULES, MAX_TEXT, MIN_OUTAGE_MS,
};
use super::emulator_match::{header_name_valid, RequestMatcher};
use super::error::{EngineError, EngineResult, Field};
use super::experiment_data as data;
use super::experiment_fields::check_bind;
use super::matching::{self, ArgRule, OscMatcher, UdpMatcher, UdpMode};
use super::osc_codec::OscArg;
use super::signals::RawPayload;
use super::subscribe::{filter_valid, topic_name_valid};
use super::template::{self, Ref, Rng};

/// Mixed into the seed, so picking a response and drawing a delay never draw
/// the numbers a template's generators do.
const PICK_STREAM: u64 = 0x7069_636b_0000_0001;

pub(crate) fn required(field: Field) -> EngineError {
    EngineError::new("node.required").in_field(field)
}

pub(crate) fn range(field: Field, min: u64, max: u64) -> EngineError {
    EngineError::new("node.range").with("min", min).with("max", max).in_field(field)
}

pub(crate) fn too_long(field: Field, max: usize) -> EngineError {
    EngineError::new("node.too_long").with("max", max).in_field(field)
}

/// A field fixed when the emulator starts: text and parameters only.
fn fixed(text: &str, field: Field, params: &BTreeMap<String, String>) -> EngineResult<String> {
    let parsed = template::parse(text).map_err(|error| error.in_field(field.clone()))?;
    for reference in parsed.refs() {
        let problem = match reference {
            Ref::Param(name) | Ref::Bare(name) if params.contains_key(&name) => continue,
            Ref::Param(name) => EngineError::new("param.unknown").with("name", name),
            Ref::Secret(_) => EngineError::new("emulator.secret_unsupported"),
            Ref::Bare(name) | Ref::Var(name) => EngineError::new("emulator.params_only").with("name", name),
        };
        return Err(problem.in_field(field));
    }
    // Generators (`{{uuid}}`) change from one request to the next; a pattern cannot.
    if !parsed.is_static(params) {
        return Err(EngineError::new("emulator.params_only").with("name", text).in_field(field));
    }
    data::static_text(text, params).ok_or_else(|| EngineError::new("emulator.params_only").with("name", text).in_field(field))
}

/// A field rendered for each reply: it may read what arrived (`request`) when
/// `request` is true, parameters and the generators — nothing else.
fn reply_template(text: &str, field: Field, params: &BTreeMap<String, String>, request: bool) -> EngineResult<()> {
    if text.len() > MAX_TEXT {
        return Err(too_long(field, MAX_TEXT));
    }
    let parsed = template::parse(text).map_err(|error| error.in_field(field.clone()))?;
    for reference in parsed.refs() {
        let problem = match reference {
            Ref::Bare(name) if request && name == "request" => continue,
            Ref::Param(name) | Ref::Bare(name) if params.contains_key(&name) => continue,
            Ref::Param(name) => EngineError::new("param.unknown").with("name", name),
            Ref::Secret(_) => EngineError::new("emulator.secret_unsupported"),
            Ref::Bare(name) | Ref::Var(name) => EngineError::new("emulator.name_unknown").with("name", name),
        };
        return Err(problem.in_field(field));
    }
    Ok(())
}

fn check_delay(delay_ms: u64, jitter_ms: u64) -> EngineResult<()> {
    if delay_ms > MAX_DELAY_MS {
        return Err(range(Field::new("delay_ms"), 0, MAX_DELAY_MS));
    }
    if jitter_ms > MAX_DELAY_MS {
        return Err(range(Field::new("jitter_ms"), 0, MAX_DELAY_MS));
    }
    Ok(())
}

/// Where a reply goes: empty for the sender, else an `IP:port`.
fn reply_target(to: &str, params: &BTreeMap<String, String>) -> EngineResult<Option<SocketAddr>> {
    if to.trim().is_empty() {
        return Ok(None);
    }
    let text = fixed(to, Field::new("to"), params)?;
    text.trim()
        .parse::<SocketAddr>()
        .map(Some)
        .map_err(|_| EngineError::new("node.target_invalid").with("value", text).in_field(Field::new("to")))
}

/// An emulator ready to serve: patterns compiled, fixed fields resolved.
pub(crate) struct Compiled {
    pub name: String,
    pub bind: SocketAddr,
    pub params: BTreeMap<String, String>,
    pub rules: Rules,
    pub outage: Option<Outage>,
}

pub(crate) enum Rules {
    Http { routes: Vec<CompiledRoute>, fallback: Option<Response> },
    Osc(Vec<OscResponder>),
    Udp(Vec<UdpResponder>),
    Tcp { delimiter: Delimiter, greeting: String, rules: Vec<TcpResponder> },
    Mqtt(MqttRules),
}

/// A broker's settings with their parameters filled in.
pub(crate) struct MqttRules {
    /// The user name and password a client must give; `None`: anyone may connect.
    pub login: Option<(String, String)>,
    /// `(topic, payload, qos)` retained from the start.
    pub retained: Vec<(String, Vec<u8>, u8)>,
    pub rules: Vec<MqttResponder>,
}

pub(crate) struct MqttResponder {
    pub filter: String,
    pub matcher: UdpMatcher,
    pub reply: Option<MqttOut>,
    pub delay_ms: u64,
    pub jitter_ms: u64,
}

pub(crate) struct CompiledRoute {
    pub matcher: RequestMatcher,
    pub order: Order,
    pub responses: Vec<Response>,
}

impl CompiledRoute {
    /// The response for the route's `count`th request (1-based) and its index.
    pub fn pick(&self, seed: u64, rule: usize, count: u64) -> (usize, &Response) {
        let index = pick(&self.responses, self.order, seed, rule, count);
        (index, &self.responses[index])
    }
}

/// Which of `responses` the `count`th request of rule `rule` gets.
pub fn pick(responses: &[Response], order: Order, seed: u64, rule: usize, count: u64) -> usize {
    let n = responses.len().max(1);
    let nth = count.saturating_sub(1) as usize;
    match order {
        Order::Sequence => nth.min(n - 1),
        Order::Cycle => nth % n,
        Order::Random => {
            let total: u64 = responses.iter().map(|response| response.weight as u64).sum();
            if total == 0 {
                return 0;
            }
            let mut draw = Rng::for_node(seed ^ PICK_STREAM, &format!("rule-{rule}"), count).below_u64(total);
            for (index, response) in responses.iter().enumerate() {
                if draw < response.weight as u64 {
                    return index;
                }
                draw -= response.weight as u64;
            }
            0
        }
    }
}

pub(crate) struct OscResponder {
    pub matcher: OscMatcher,
    pub reply: Option<OscOut>,
    pub to: Option<SocketAddr>,
    pub delay_ms: u64,
    pub jitter_ms: u64,
}

pub(crate) struct UdpResponder {
    pub matcher: UdpMatcher,
    pub reply: Option<RawPayload>,
    pub to: Option<SocketAddr>,
    pub delay_ms: u64,
    pub jitter_ms: u64,
}

pub(crate) struct TcpResponder {
    pub matcher: UdpMatcher,
    pub reply: Option<RawPayload>,
    pub close: bool,
    pub delay_ms: u64,
    pub jitter_ms: u64,
}

fn check_response(response: &Response, params: &BTreeMap<String, String>) -> EngineResult<()> {
    if !(100..=599).contains(&response.status) {
        return Err(range(Field::new("status"), 100, 599));
    }
    if response.headers.len() > MAX_HEADERS {
        return Err(EngineError::new("emulator.headers_too_many").with("max", MAX_HEADERS).in_field(Field::new("headers")));
    }
    for (index, (name, value)) in response.headers.iter().enumerate() {
        let name_field = Field::nth("header_name", index + 1);
        if name.trim().is_empty() {
            return Err(required(name_field));
        }
        let name = fixed(name, name_field.clone(), params)?;
        if !header_name_valid(&name) {
            return Err(EngineError::new("node.header_invalid").with("value", name).in_field(name_field));
        }
        let value_field = Field::nth("header_value", index + 1);
        reply_template(value, value_field.clone(), params, true)?;
        if let Some(text) = data::static_text(value, params) {
            if hyper::header::HeaderValue::from_str(&text).is_err() {
                return Err(EngineError::new("emulator.header_value_invalid").in_field(value_field));
            }
        }
    }
    reply_template(&response.body, Field::new("body"), params, true)?;
    check_delay(response.delay_ms, response.jitter_ms)
}

fn compile_route(route: &Route, params: &BTreeMap<String, String>) -> EngineResult<CompiledRoute> {
    if route.path.trim().is_empty() {
        return Err(required(Field::new("path")));
    }
    let path = fixed(&route.path, Field::new("path"), params)?;
    let when = route
        .when
        .iter()
        .enumerate()
        .map(|(index, condition)| {
            let field = Field::nth("condition", index + 1);
            Ok(Condition { name: fixed(&condition.name, field.clone(), params)?, value: fixed(&condition.value, field, params)?, ..condition.clone() })
        })
        .collect::<EngineResult<Vec<_>>>()?;
    let matcher = RequestMatcher::new(&route.method, &path, &when)?;
    if route.responses.is_empty() {
        return Err(EngineError::new("emulator.response_required").in_field(Field::new("responses")));
    }
    if route.responses.len() > MAX_RESPONSES {
        return Err(EngineError::new("emulator.responses_too_many").with("max", MAX_RESPONSES).in_field(Field::new("responses")));
    }
    for (index, response) in route.responses.iter().enumerate() {
        check_response(response, params).map_err(|error| error.with("response", index + 1))?;
    }
    if route.order == Order::Random && route.responses.iter().all(|response| response.weight == 0) {
        return Err(EngineError::new("emulator.weights_zero").in_field(Field::new("weight")));
    }
    Ok(CompiledRoute { matcher, order: route.order, responses: route.responses.clone() })
}

/// A reply argument's text as the type it says; empty text is that type's zero.
pub fn osc_arg(kind: ArgType, text: &str) -> EngineResult<OscArg> {
    let value = text.trim();
    let invalid = || EngineError::new("emulator.arg_value").with("value", text).with("type", serde_json::to_value(kind).ok().and_then(|kind| kind.as_str().map(str::to_string)).unwrap_or_default());
    fn parsed<T: std::str::FromStr + Default>(value: &str) -> Option<T> {
        if value.is_empty() { Some(T::default()) } else { value.parse().ok() }
    }
    Ok(match kind {
        ArgType::Int => OscArg::Int(parsed(value).ok_or_else(invalid)?),
        ArgType::Long => OscArg::Long(parsed(value).ok_or_else(invalid)?),
        ArgType::Float => OscArg::Float(parsed::<f32>(value).filter(|number| number.is_finite()).ok_or_else(invalid)?),
        ArgType::Double => OscArg::Double(parsed::<f64>(value).filter(|number| number.is_finite()).ok_or_else(invalid)?),
        ArgType::Str => OscArg::Str(text.to_string()),
        ArgType::Bool => match value.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => OscArg::Bool(true),
            "false" | "0" | "no" | "off" | "" => OscArg::Bool(false),
            _ => return Err(invalid()),
        },
        ArgType::Blob if value.is_empty() => OscArg::Blob(Vec::new()),
        ArgType::Blob => OscArg::Blob(matching::parse_hex(value).map_err(|_| invalid())?),
        ArgType::Nil => OscArg::Nil,
    })
}

fn check_osc_out(out: &OscOut, params: &BTreeMap<String, String>) -> EngineResult<()> {
    let field = Field::new("reply_address");
    if out.address.trim().is_empty() {
        return Err(required(field));
    }
    reply_template(&out.address, field.clone(), params, true)?;
    if let Some(address) = data::static_text(&out.address, params).filter(|address| !address.starts_with('/')) {
        return Err(EngineError::new("node.osc_address").with("value", address).in_field(field));
    }
    if out.args.len() > MAX_ARGS {
        return Err(EngineError::new("emulator.args_too_many").with("max", MAX_ARGS).in_field(Field::new("reply_args")));
    }
    for (index, arg) in out.args.iter().enumerate() {
        let field = Field::nth("reply_arg", index + 1);
        reply_template(&arg.value, field.clone(), params, true)?;
        if let Some(text) = data::static_text(&arg.value, params) {
            osc_arg(arg.kind, &text).map_err(|error| error.in_field(field))?;
        }
    }
    Ok(())
}

fn check_raw(reply: &RawPayload, params: &BTreeMap<String, String>) -> EngineResult<()> {
    let field = Field::new("reply");
    match reply {
        RawPayload::Text { text } => {
            if text.len() > MAX_DATAGRAM {
                return Err(too_long(field, MAX_DATAGRAM));
            }
            reply_template(text, field, params, true)
        }
        RawPayload::Hex { hex } => {
            reply_template(hex, field.clone(), params, true)?;
            match data::static_text(hex, params) {
                Some(text) => matching::parse_hex(&text).map(|_| ()).map_err(|error| error.in_field(field)),
                None => Ok(()),
            }
        }
    }
}

fn payload_matcher(mode: UdpMode, pattern: &str, params: &BTreeMap<String, String>) -> EngineResult<UdpMatcher> {
    let field = Field::new("pattern");
    if mode != UdpMode::Any && pattern.is_empty() {
        return Err(required(field));
    }
    let pattern = if mode == UdpMode::Any { String::new() } else { fixed(pattern, field.clone(), params)? };
    UdpMatcher::new(mode, &pattern).map_err(|error| error.in_field(field))
}

fn compile_osc(rule: &OscRule, params: &BTreeMap<String, String>) -> EngineResult<OscResponder> {
    if rule.address.trim().is_empty() {
        return Err(required(Field::new("address")));
    }
    let address = fixed(&rule.address, Field::new("address"), params)?;
    if rule.args.len() > MAX_ARGS {
        return Err(EngineError::new("node.rules_too_many").with("max", MAX_ARGS).in_field(Field::new("rules")));
    }
    let mut conditions = Vec::new();
    for (index, arg) in rule.args.iter().enumerate() {
        let field = Field::nth("rule_value", index + 1);
        if arg.index > matching::MAX_ARG_INDEX {
            return Err(range(Field::nth("rule", index + 1), 0, matching::MAX_ARG_INDEX as u64));
        }
        conditions.push(ArgRule { value: fixed(&arg.value, field, params)?, ..arg.clone() });
    }
    let matcher = OscMatcher::new(&address, &conditions)?;
    if let Some(out) = &rule.reply {
        check_osc_out(out, params)?;
    }
    check_delay(rule.delay_ms, rule.jitter_ms)?;
    Ok(OscResponder { matcher, reply: rule.reply.clone(), to: reply_target(&rule.to, params)?, delay_ms: rule.delay_ms, jitter_ms: rule.jitter_ms })
}

fn compile_udp(rule: &UdpRule, params: &BTreeMap<String, String>) -> EngineResult<UdpResponder> {
    let matcher = payload_matcher(rule.mode, &rule.pattern, params)?;
    if let Some(reply) = &rule.reply {
        check_raw(reply, params)?;
    }
    check_delay(rule.delay_ms, rule.jitter_ms)?;
    Ok(UdpResponder { matcher, reply: rule.reply.clone(), to: reply_target(&rule.to, params)?, delay_ms: rule.delay_ms, jitter_ms: rule.jitter_ms })
}

fn compile_tcp(rule: &TcpRule, params: &BTreeMap<String, String>) -> EngineResult<TcpResponder> {
    let matcher = payload_matcher(rule.mode, &rule.pattern, params)?;
    if let Some(reply) = &rule.reply {
        check_raw(reply, params)?;
    }
    check_delay(rule.delay_ms, rule.jitter_ms)?;
    Ok(TcpResponder { matcher, reply: rule.reply.clone(), close: rule.close, delay_ms: rule.delay_ms, jitter_ms: rule.jitter_ms })
}

fn check_qos(qos: u8) -> EngineResult<()> {
    if qos > 2 {
        return Err(range(Field::new("qos"), 0, 2));
    }
    Ok(())
}

/// A topic a message is published to, once its parameters are filled in.
fn topic_name(text: &str, field: Field) -> EngineResult<()> {
    if text.is_empty() {
        return Err(required(field));
    }
    if !topic_name_valid(text) {
        return Err(EngineError::new("node.topic_wildcard").in_field(field));
    }
    Ok(())
}

fn compile_mqtt(rule: &MqttRule, params: &BTreeMap<String, String>) -> EngineResult<MqttResponder> {
    let field = Field::new("topic");
    if rule.topic.is_empty() {
        return Err(required(field));
    }
    let filter = fixed(&rule.topic, field.clone(), params)?;
    if !filter_valid(&filter) {
        return Err(EngineError::new("node.topic_filter").with("value", filter).in_field(field));
    }
    let matcher = payload_matcher(rule.mode, &rule.pattern, params)?;
    if let Some(out) = &rule.reply {
        let field = Field::new("reply_topic");
        reply_template(&out.topic, field.clone(), params, true)?;
        match data::static_text(&out.topic, params) {
            Some(topic) => topic_name(&topic, field)?,
            None if out.topic.is_empty() => return Err(required(field)),
            None => {}
        }
        reply_template(&out.payload, Field::new("reply_payload"), params, true)?;
        check_qos(out.qos)?;
    }
    check_delay(rule.delay_ms, rule.jitter_ms)?;
    Ok(MqttResponder { filter, matcher, reply: rule.reply.clone(), delay_ms: rule.delay_ms, jitter_ms: rule.jitter_ms })
}

fn compile_retained(retained: &MqttRetained, params: &BTreeMap<String, String>) -> EngineResult<(String, Vec<u8>, u8)> {
    let topic = fixed(&retained.topic, Field::new("topic"), params)?;
    topic_name(&topic, Field::new("topic"))?;
    if retained.payload.len() > MAX_TEXT {
        return Err(too_long(Field::new("payload"), MAX_TEXT));
    }
    let payload = fixed(&retained.payload, Field::new("payload"), params)?;
    check_qos(retained.qos)?;
    Ok((topic, payload.into_bytes(), retained.qos))
}

fn compile_broker(username: &str, password: &str, retained: &[MqttRetained], rules: &[MqttRule], params: &BTreeMap<String, String>) -> EngineResult<MqttRules> {
    let username = fixed(username, Field::new("username"), params)?;
    let password = fixed(password, Field::new("password"), params)?;
    // 3.1.1 cannot carry a password without a user name, so no client could connect.
    if username.is_empty() && !password.is_empty() {
        return Err(EngineError::new("emulator.mqtt_password_alone").in_field(Field::new("password")));
    }
    if retained.len() > MAX_RULES {
        return Err(EngineError::new("emulator.rules_too_many").with("max", MAX_RULES).in_field(Field::new("retained")));
    }
    let retained = retained
        .iter()
        .enumerate()
        .map(|(index, message)| compile_retained(message, params).map_err(|error| error.with("retained", index + 1)))
        .collect::<EngineResult<_>>()?;
    let rules = rules.iter().enumerate().map(|(index, rule)| compile_mqtt(rule, params).map_err(|error| error.with("rule", index + 1))).collect::<EngineResult<_>>()?;
    Ok(MqttRules { login: (!username.is_empty()).then_some((username, password)), retained, rules })
}

fn check_outage(outage: &Outage) -> EngineResult<()> {
    if !(MIN_OUTAGE_MS..=MAX_OUTAGE_MS).contains(&outage.up_ms) {
        return Err(range(Field::new("outage_up"), MIN_OUTAGE_MS, MAX_OUTAGE_MS));
    }
    if !(MIN_OUTAGE_MS..=MAX_OUTAGE_MS).contains(&outage.down_ms) {
        return Err(range(Field::new("outage_down"), MIN_OUTAGE_MS, MAX_OUTAGE_MS));
    }
    Ok(())
}

pub(crate) fn compile(emulator: &Emulator, params: &BTreeMap<String, String>) -> EngineResult<Compiled> {
    let name = emulator.name.trim();
    if name.is_empty() {
        return Err(required(Field::new("name")));
    }
    if name.chars().count() > MAX_NAME {
        return Err(too_long(Field::new("name"), MAX_NAME));
    }
    let bind = check_bind(&emulator.bind)?;
    if emulator.kind.rules() > MAX_RULES {
        return Err(EngineError::new("emulator.rules_too_many").with("max", MAX_RULES).in_field(Field::new("rules")));
    }
    let at = |index: usize| move |error: EngineError| error.with("rule", index + 1);
    let rules = match &emulator.kind {
        EmulatorKind::Http { routes, fallback } => {
            let routes = routes.iter().enumerate().map(|(index, route)| compile_route(route, params).map_err(at(index))).collect::<EngineResult<_>>()?;
            if let Some(fallback) = fallback {
                check_response(fallback, params).map_err(|error| error.in_field(Field::new("fallback")))?;
            }
            Rules::Http { routes, fallback: fallback.clone() }
        }
        EmulatorKind::Osc { rules } => Rules::Osc(rules.iter().enumerate().map(|(index, rule)| compile_osc(rule, params).map_err(at(index))).collect::<EngineResult<_>>()?),
        EmulatorKind::Udp { rules } => Rules::Udp(rules.iter().enumerate().map(|(index, rule)| compile_udp(rule, params).map_err(at(index))).collect::<EngineResult<_>>()?),
        EmulatorKind::Tcp { delimiter, greeting, rules } => {
            // A greeting goes out before anything has arrived: no `request` yet.
            reply_template(greeting, Field::new("greeting"), params, false)?;
            let rules = rules.iter().enumerate().map(|(index, rule)| compile_tcp(rule, params).map_err(at(index))).collect::<EngineResult<_>>()?;
            Rules::Tcp { delimiter: *delimiter, greeting: greeting.clone(), rules }
        }
        EmulatorKind::Mqtt { username, password, retained, rules } => Rules::Mqtt(compile_broker(username, password, retained, rules, params)?),
    };
    if let Some(outage) = &emulator.outage {
        check_outage(outage)?;
    }
    Ok(Compiled { name: name.to_string(), bind, params: params.clone(), rules, outage: emulator.outage.clone() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::check;
    use serde_json::{json, Value};

    fn doc(value: Value) -> Emulator {
        serde_json::from_value(value).unwrap()
    }

    fn code(result: EngineResult<()>) -> (String, BTreeMap<String, String>, Option<Field>) {
        let error = result.unwrap_err();
        (error.code.clone(), error.params.clone(), error.field.clone())
    }

    fn responses(statuses: &[(u16, u32)]) -> Vec<Response> {
        statuses.iter().map(|(status, weight)| serde_json::from_value(json!({ "status": status, "weight": weight })).unwrap()).collect()
    }

    #[test]
    fn sequences_repeat_their_last_cycles_wrap_and_draws_follow_the_seed() {
        let three = responses(&[(500, 1), (500, 1), (200, 1)]);
        assert_eq!((1..=5).map(|count| three[pick(&three, Order::Sequence, 1, 1, count)].status).collect::<Vec<_>>(), [500, 500, 200, 200, 200]);
        assert_eq!((1..=5).map(|count| pick(&three, Order::Cycle, 1, 1, count)).collect::<Vec<_>>(), [0, 1, 2, 0, 1]);
        let mix = responses(&[(200, 80), (429, 10), (500, 10)]);
        let draws: Vec<usize> = (1..=2000).map(|count| pick(&mix, Order::Random, 42, 1, count)).collect();
        let ok = draws.iter().filter(|index| **index == 0).count();
        assert!((1450..=1750).contains(&ok), "about 80 % of 2000: {ok}");
        assert!(draws.contains(&1) && draws.contains(&2));
        assert_eq!(draws, (1..=2000).map(|count| pick(&mix, Order::Random, 42, 1, count)).collect::<Vec<_>>(), "the same seed, the same draws");
        assert_ne!(draws, (1..=2000).map(|count| pick(&mix, Order::Random, 43, 1, count)).collect::<Vec<_>>());
        let never = responses(&[(200, 0), (503, 1)]);
        assert!((1..=50).all(|count| pick(&never, Order::Random, 7, 1, count) == 1), "weight 0 is never drawn");
    }

    #[test]
    fn problems_name_the_rule_the_response_and_the_field() {
        let params = BTreeMap::from([("port".to_string(), "9".to_string())]);
        let http = |route: Value| doc(json!({ "name": "API", "bind": "127.0.0.1:8080", "protocol": "http", "routes": [{ "path": "/ok", "responses": [{}] }, route] }));
        let (code_, params_, field) = code(check(&http(json!({ "path": "/x", "responses": [{}, { "body": "{{vars.token}}" }] })), &params));
        assert_eq!((code_.as_str(), params_["rule"].as_str(), params_["response"].as_str(), field), ("emulator.name_unknown", "2", "2", Some(Field::new("body"))));
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "body": "{{secret.KEY}}" }] })), &params)).0, "emulator.secret_unsupported");
        assert_eq!(code(check(&http(json!({ "path": "/{{request.path}}", "responses": [{}] })), &params)).0, "emulator.params_only");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [] })), &params)).0, "emulator.response_required");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "status": 99 }] })), &params)).0, "node.range");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "headers": [["bad name", "x"]] }] })), &params)).2, Some(Field::nth("header_name", 1)));
        assert_eq!(code(check(&http(json!({ "path": "/x", "order": "random", "responses": [{ "weight": 0 }] })), &params)).0, "emulator.weights_zero");
        assert_eq!(code(check(&http(json!({ "path": "/x", "responses": [{ "delay_ms": 60001 }] })), &params)).2, Some(Field::new("delay_ms")));
        // Parameters are welcome everywhere, and `request` in what is rendered per request.
        let fine = http(json!({ "path": "/items/:id", "when": [{ "on": "header", "name": "x-port", "op": "eq", "value": "{{params.port}}" }],
            "responses": [{ "headers": [["X-Id", "{{request.params.id}}"]], "body": "{{request.json.name}} {{port}} {{uuid}} {{counter}}" }] }));
        assert!(check(&fine, &params).is_ok());

        let osc = |rule: Value| doc(json!({ "name": "Device", "bind": "127.0.0.1:9100", "protocol": "osc", "rules": [rule] }));
        let reply = osc(json!({ "address": "/ping", "reply": { "address": "/pong", "args": [{ "type": "int", "value": "seven" }] } }));
        let (code_, params_, field) = code(check(&reply, &params));
        assert_eq!((code_.as_str(), params_["rule"].as_str(), field), ("emulator.arg_value", "1", Some(Field::nth("reply_arg", 1))));
        assert!(check(&osc(json!({ "address": "/ping", "reply": { "address": "/pong", "args": [{ "type": "int", "value": "{{request.args[0]}}" }] } })), &params).is_ok());
        assert_eq!(code(check(&osc(json!({ "address": "ping" })), &params)).0, "osc.pattern_slash");
        assert_eq!(code(check(&osc(json!({ "address": "/ping", "to": "nowhere" })), &params)).0, "node.target_invalid");
        let udp = doc(json!({ "name": "Echo", "bind": "127.0.0.1:7100", "protocol": "udp", "rules": [{ "mode": "hex", "pattern": "zz" }] }));
        assert_eq!(code(check(&udp, &params)).0, "hex.invalid");
        let greeting = doc(json!({ "name": "Device", "bind": "127.0.0.1:7200", "protocol": "tcp", "greeting": "{{request.from}}" }));
        assert_eq!(code(check(&greeting, &params)).0, "emulator.name_unknown", "nothing has arrived when the greeting goes out");
        assert_eq!(code(check(&doc(json!({ "name": " ", "bind": "127.0.0.1:1", "protocol": "udp" })), &params)).2, Some(Field::new("name")));
        assert_eq!(code(check(&doc(json!({ "name": "x", "bind": "127.0.0.1:0", "protocol": "udp" })), &params)).0, "node.bind_invalid");
    }

    #[test]
    fn reply_arguments_take_their_type_from_the_text() {
        assert_eq!(osc_arg(ArgType::Int, " 42 ").unwrap(), OscArg::Int(42));
        assert_eq!(osc_arg(ArgType::Float, "0.5").unwrap(), OscArg::Float(0.5));
        assert_eq!(osc_arg(ArgType::Double, "").unwrap(), OscArg::Double(0.0));
        assert_eq!(osc_arg(ArgType::Bool, "on").unwrap(), OscArg::Bool(true));
        assert_eq!(osc_arg(ArgType::Blob, "de ad").unwrap(), OscArg::Blob(vec![0xde, 0xad]));
        assert_eq!(osc_arg(ArgType::Str, " a ").unwrap(), OscArg::Str(" a ".into()));
        assert_eq!(osc_arg(ArgType::Nil, "ignored").unwrap(), OscArg::Nil);
        let error = osc_arg(ArgType::Int, "1.5").unwrap_err();
        assert_eq!((error.code.as_str(), error.params["type"].as_str()), ("emulator.arg_value", "int"));
        assert!(osc_arg(ArgType::Float, "inf").is_err() && osc_arg(ArgType::Bool, "maybe").is_err());
    }

    #[test]
    fn a_broker_and_an_outage_are_checked_like_the_rest() {
        let params = BTreeMap::from([("site".to_string(), "lab".to_string())]);
        let broker = |extra: Value| {
            let mut document = json!({ "name": "Broker", "bind": "127.0.0.1:1883", "protocol": "mqtt" });
            document.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            doc(document)
        };
        let fine = broker(json!({ "username": "{{site}}", "password": "pw", "retained": [{ "topic": "{{params.site}}/state", "payload": "idle" }],
            "rules": [{ "topic": "{{site}}/+/set", "reply": { "topic": "{{site}}/{{request.levels[1]}}/state", "payload": "{{request.payload}}", "qos": 2, "retain": true } }] }));
        assert!(check(&fine, &params).is_ok());
        let (code_, params_, field) = code(check(&broker(json!({ "rules": [{ "topic": "a" }, { "topic": "a/#/b" }] })), &params));
        assert_eq!((code_.as_str(), params_["rule"].as_str(), field), ("node.topic_filter", "2", Some(Field::new("topic"))));
        assert_eq!(code(check(&broker(json!({ "rules": [{ "topic": "a/+", "reply": { "topic": "a/+/state" } }] })), &params)).2, Some(Field::new("reply_topic")));
        assert_eq!(code(check(&broker(json!({ "rules": [{ "topic": "a", "reply": { "topic": "" } }] })), &params)).0, "node.required");
        assert_eq!(code(check(&broker(json!({ "rules": [{ "topic": "a", "reply": { "topic": "b", "qos": 3 } }] })), &params)).2, Some(Field::new("qos")));
        assert_eq!(code(check(&broker(json!({ "rules": [{ "topic": "{{request.topic}}" }] })), &params)).0, "emulator.params_only", "a filter is fixed when it starts");
        assert_eq!(code(check(&broker(json!({ "password": "pw" })), &params)).0, "emulator.mqtt_password_alone");
        let (code_, params_, _) = code(check(&broker(json!({ "retained": [{ "topic": "a/#" }] })), &params));
        assert_eq!((code_.as_str(), params_["retained"].as_str()), ("node.topic_wildcard", "1"));

        let flapping = |up: u64, down: u64| doc(json!({ "name": "x", "bind": "127.0.0.1:1", "protocol": "udp", "outage": { "up_ms": up, "down_ms": down } }));
        assert!(check(&flapping(1000, 500), &params).is_ok());
        assert_eq!(code(check(&flapping(5, 500), &params)).2, Some(Field::new("outage_up")));
        assert_eq!(code(check(&flapping(1000, 3_600_001), &params)).2, Some(Field::new("outage_down")));
        let round: Emulator = serde_json::from_value(serde_json::to_value(flapping(1000, 500)).unwrap()).unwrap();
        assert_eq!(round.outage.map(|outage| (outage.up_ms, outage.down_ms, outage.fault)), Some((1000, 500, crate::emulator::DownFault::Unavailable)));
    }
}
