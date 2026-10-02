//! `signallab send …` and `signallab fire`: one message, through the same
//! commands the app's screens use (`osc_send`, `broadcast_send`,
//! `http_request`, `mqtt_publish_once`, `ws_exchange`) — a fired signal is the same bytes as a
//! hand-typed one, as in the app (`src/lib/signals.ts`, `fireSignal`).

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};
use signal_lab_engine::error::EngineError;
use signal_lab_engine::host::NoEvents;
use signal_lab_engine::http::HttpResponse;
use signal_lab_engine::osc_codec::OscArg;
use signal_lab_engine::secrets::FileStore;
use signal_lab_engine::signals::{Library, RawPayload, Signal, SignalBody};
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

use crate::fail::{Exit, Failure};
use crate::{params, Ctx, FireArgs, SendCommand};

/// An engine of its own for one send: nothing is captured, no secrets, no files.
fn engine() -> Service {
    Service::new(Host::new(Arc::new(NoEvents), Capture::new()), Mode::Server, Arc::new(FileStore::new(None)))
}

async fn invoke(service: &Service, command: &str, args: Value) -> Result<Value, Failure> {
    service.invoke(command, args).await.map_err(|failure| match failure {
        signal_lab_engine::Failure::Engine(error) => Failure::sending(error),
    })
}

/// One OSC argument from the command line: `i:3 f:0.5 d:1.5 h:64 s:text b:hex T F N`;
/// a plain integer is i, a plain decimal is f, anything else is s.
pub(crate) fn osc_arg(text: &str) -> Result<OscArg, EngineError> {
    let invalid = || EngineError::new("cli.osc_arg_invalid").with("value", text);
    match text {
        "T" => return Ok(OscArg::Bool(true)),
        "F" => return Ok(OscArg::Bool(false)),
        "N" => return Ok(OscArg::Nil),
        _ => {}
    }
    if let Some((tag, value)) = text.split_once(':').filter(|(tag, _)| tag.len() == 1) {
        return match tag {
            "i" => value.trim().parse().map(OscArg::Int).map_err(|_| invalid()),
            "f" => value.trim().parse().map(OscArg::Float).map_err(|_| invalid()),
            "d" => value.trim().parse().map(OscArg::Double).map_err(|_| invalid()),
            "h" => value.trim().parse().map(OscArg::Long).map_err(|_| invalid()),
            "s" => Ok(OscArg::Str(value.to_string())),
            "b" => hex_bytes(value).map(OscArg::Blob).ok_or_else(invalid),
            _ => Err(invalid()),
        };
    }
    if let Ok(int) = text.parse::<i32>() {
        return Ok(OscArg::Int(int));
    }
    if text.contains('.') {
        if let Ok(float) = text.parse::<f32>() {
            return Ok(OscArg::Float(float));
        }
    }
    Ok(OscArg::Str(text.to_string()))
}

/// "de ad be ef", "deadbeef", "de:ad" — pairs of hex digits.
pub(crate) fn hex_bytes(text: &str) -> Option<Vec<u8>> {
    let digits: String = text.chars().filter(|char| !char.is_whitespace() && !matches!(char, ':' | '-' | ',')).collect();
    if digits.is_empty() || !digits.len().is_multiple_of(2) {
        return None;
    }
    (0..digits.len()).step_by(2).map(|at| u8::from_str_radix(&digits[at..at + 2], 16).ok()).collect()
}

/// A single datagram to one target, the way the app fires a UDP signal.
pub(crate) fn datagram(target: &str, payload: &RawPayload) -> Value {
    json!({ "config": {
        "mode": "list", "target": target, "port": 0, "payload": payload, "bind": null, "ttl": 1,
        "multicast_loop": false, "rate": 1, "count": 1, "duration_s": 0,
    } })
}

/// A one-shot MQTT publish: a fresh client id each time, so a live connection with the same id is never knocked off.
pub(crate) fn mqtt_publish(broker: &str, topic: &str, payload: &str, qos: u8, retain: bool) -> Result<Value, Failure> {
    let (host, port) = match broker.rsplit_once(':') {
        Some((host, port)) => (host.trim_matches(['[', ']']).to_string(), port.parse::<u16>().ok()),
        None => (broker.to_string(), Some(1883)),
    };
    let Some(port) = port.filter(|_| !host.trim().is_empty()) else {
        return Err(Failure::invalid(EngineError::new("transport.target_invalid").with("target", broker)));
    };
    let client_id = format!("signallab-cli-{:06x}", rand::random::<u32>() & 0xff_ffff);
    Ok(json!({
        "config": { "host": host, "port": port, "client_id": client_id, "username": "", "password": "", "keep_alive_s": 15,
                    "clean_session": true, "will": null, "subscribe": [] },
        "topic": topic, "payload": payload, "qos": qos, "retain": retain,
    }))
}

/// A response that never came is the transport failure the app shows for it (`responseFailure`).
pub(crate) fn response_failure(response: &HttpResponse, url: &str) -> Option<EngineError> {
    let error = response.error.as_ref()?;
    Some(match response.cause {
        Some(cause) => cause.error(url).because(error),
        None => EngineError::new("transport.failed").with("target", url).because(error),
    })
}

/// `Name: value` headers from the command line.
fn header_pairs(headers: &[String]) -> Result<Vec<(String, String)>, Failure> {
    headers
        .iter()
        .map(|header| match header.split_once(':') {
            Some((name, value)) if !name.trim().is_empty() => Ok((name.trim().to_string(), value.trim().to_string())),
            _ => Err(Failure::invalid(EngineError::new("cli.header_invalid").with("value", header))),
        })
        .collect()
}

/// Print a send's result: JSON on stdout with --json, else one line.
fn done(ctx: &Ctx, value: &Value, line: String) -> Exit {
    if ctx.json {
        println!("{}", json!({ "type": "sent", "result": value }));
    } else {
        println!("✔ {line}");
    }
    Exit::Passed
}

pub async fn send(ctx: &Ctx, command: SendCommand) -> Exit {
    match send_inner(ctx, command).await {
        Ok(exit) => exit,
        Err(failure) => failure.report(ctx, "✖ "),
    }
}

async fn send_inner(ctx: &Ctx, command: SendCommand) -> Result<Exit, Failure> {
    let service = engine();
    let texts = &ctx.texts;
    match command {
        SendCommand::Osc { target, address, args } => {
            let args: Vec<OscArg> = args.iter().map(|arg| osc_arg(arg)).collect::<Result<_, _>>().map_err(Failure::invalid)?;
            let bytes = invoke(&service, "osc_send", json!({ "target": target, "address": address, "args": args })).await?;
            let line = texts.t("log.oscSent", &params! { "address" => address.as_str(), "bytes" => bytes.as_u64().unwrap_or_default(), "target" => target.as_str() });
            Ok(done(ctx, &bytes, line))
        }
        SendCommand::Udp { target, text, hex } => {
            let payload = match (text, hex) {
                (Some(text), _) => RawPayload::Text { text },
                (None, Some(hex)) => RawPayload::Hex { hex },
                (None, None) => unreachable!("clap requires --text or --hex"),
            };
            let result = invoke(&service, "broadcast_send", datagram(&target, &payload)).await?;
            if result["errors"].as_u64().unwrap_or_default() > 0 {
                let error = serde_json::from_value::<EngineError>(result["error"].clone()).unwrap_or_else(|_| EngineError::new("transport.failed").with("target", &target));
                return Err(Failure::sending(error));
            }
            let line = texts.t("cli.sentUdp", &params! { "target" => target.as_str(), "bytes" => result["bytes"].as_u64().unwrap_or_default() });
            Ok(done(ctx, &result, line))
        }
        SendCommand::Http { method, url, headers, body, expect_status, timeout, user, digest, bearer } => {
            let headers = header_pairs(&headers)?;
            let auth = match (user, bearer) {
                (Some(user), _) => {
                    let (username, password) = user.split_once(':').unwrap_or((user.as_str(), ""));
                    json!({ "scheme": if digest { "digest" } else { "basic" }, "username": username, "password": password })
                }
                (None, Some(token)) => json!({ "scheme": "bearer", "token": token }),
                (None, None) => json!({ "scheme": "none" }),
            };
            let body = match body {
                Some(body) => match body.strip_prefix('@') {
                    Some(file) => Some(std::fs::read_to_string(file).map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", file).because(error)))?),
                    None => Some(body),
                },
                None => None,
            };
            let request = json!({ "method": method.to_ascii_uppercase(), "url": url, "headers": headers, "body": body, "timeout_ms": timeout, "auth": auth });
            let value = invoke(&service, "http_request", json!({ "request": request })).await?;
            let response: HttpResponse = serde_json::from_value(value.clone())
                .map_err(|error| Failure::sending(EngineError::new("command.reply_invalid").because(error)))?;
            if let Some(error) = response_failure(&response, &url) {
                return Err(Failure { error, exit: Exit::Failed, remote: false });
            }
            // A Digest the server's 401 could not be answered with: why, after the response.
            let unanswered = response.digest.as_ref().and_then(|digest| digest.error.clone());
            let ms = (response.latency_ms.round()) as u64;
            if ctx.json {
                println!("{}", json!({ "type": "response", "response": value }));
            } else {
                let size = crate::i18n::format_number(response.body_bytes as f64, texts.lang);
                eprintln!("{}", texts.t("cli.httpStatus", &params! { "status" => u64::from(response.status), "text" => response.status_text.as_str(), "ms" => ms, "bytes" => size }));
                print!("{}", response.body);
                if !response.body.ends_with('\n') && !response.body.is_empty() {
                    println!();
                }
                if response.truncated {
                    eprintln!("{}", texts.plain("cli.httpTruncated"));
                }
            }
            match (expect_status, unanswered) {
                (_, Some(error)) => Ok(Failure { error, exit: Exit::Failed, remote: false }.report(ctx, "✖ ")),
                (Some(expected), None) if expected != response.status => {
                    let error = EngineError::new("cli.status_unexpected").with("expected", expected).with("actual", response.status);
                    Ok(Failure { error, exit: Exit::Failed, remote: false }.report(ctx, "✖ "))
                }
                _ => Ok(Exit::Passed),
            }
        }
        SendCommand::Ws { url, text, hex, headers, protocols, expect, expect_regex, wait, timeout } => {
            let headers = header_pairs(&headers)?;
            let message = match (text, hex) {
                (Some(text), _) => Some(json!({ "text": text })),
                (None, Some(hex)) => Some(json!({ "hex": hex })),
                (None, None) => None,
            };
            let expect = match (expect, expect_regex, wait) {
                (Some(text), _, _) => Some(json!({ "mode": "contains", "pattern": text, "timeout_ms": timeout })),
                (None, Some(regex), _) => Some(json!({ "mode": "regex", "pattern": regex, "timeout_ms": timeout })),
                (None, None, true) => Some(json!({ "mode": "any", "pattern": "", "timeout_ms": timeout })),
                _ => None,
            };
            let config = json!({ "url": url, "headers": headers, "protocols": protocols });
            let exchange = match invoke(&service, "ws_exchange", json!({ "config": config, "message": message, "expect": expect })).await {
                // An answer that does not come is the exchange failing, not the invocation.
                Err(failure) if failure.error.code == "wait.timeout" => return Err(Failure { exit: Exit::Failed, ..failure }),
                other => other?,
            };
            if ctx.json {
                println!("{}", json!({ "type": "exchange", "result": exchange }));
                return Ok(Exit::Passed);
            }
            let handshake = &exchange["handshake"];
            let (address, ms) = (handshake["url"].as_str().unwrap_or_default(), handshake["ms"].as_u64().unwrap_or_default());
            let connected = match handshake["protocol"].as_str() {
                Some(protocol) => texts.t("exp.step.wsConnectedAs", &params! { "url" => address, "ms" => ms, "protocol" => protocol }),
                None => texts.t("exp.step.wsConnected", &params! { "url" => address, "ms" => ms }),
            };
            eprintln!("{connected}");
            if let Some(bytes) = exchange["sent"].as_u64() {
                eprintln!("{}", texts.t("exp.step.wsSent", &params! { "bytes" => bytes }));
            }
            if let Some(reply) = exchange.get("reply").filter(|reply| !reply.is_null()) {
                let shown = if reply["kind"] == "binary" { reply["hex"].as_str() } else { reply["text"].as_str() };
                println!("{}", shown.unwrap_or_default());
            }
            Ok(Exit::Passed)
        }
        SendCommand::Mqtt { broker, topic, payload, qos, retain } => {
            let args = mqtt_publish(&broker, &topic, &payload, qos, retain)?;
            let summary = invoke(&service, "mqtt_publish_once", args).await?;
            let line = summary.as_str().unwrap_or_default().to_string();
            Ok(done(ctx, &summary, line))
        }
    }
}

/// The library at `path`, else the app's (`signals.json` in its data folder). Never created here.
pub(crate) fn library(path: Option<PathBuf>) -> Result<(PathBuf, Library), Failure> {
    let path = path.unwrap_or_else(|| paths::data_dir().join("signals.json"));
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(Failure::invalid(EngineError::new("file.not_found").with("path", path.display())));
        }
        Err(error) => return Err(Failure::invalid(EngineError::new("file.io").with("path", path.display()).because(error))),
    };
    let library = serde_json::from_str::<Library>(&text).map_err(|error| {
        Failure::invalid(EngineError::new("signals.json_invalid").with("path", path.display()).with("line", error.line()).with("column", error.column()).because(&error))
    })?;
    Ok((path, library))
}

/// By id, else by name (ignoring case): one signal, or why not.
pub(crate) fn find<'a>(library: &'a Library, wanted: &str, path: &std::path::Path) -> Result<&'a Signal, Failure> {
    if let Some(signal) = library.signals.iter().find(|signal| signal.id == wanted) {
        return Ok(signal);
    }
    let named: Vec<&Signal> = library.signals.iter().filter(|signal| signal.name.eq_ignore_ascii_case(wanted.trim())).collect();
    match named.as_slice() {
        [one] => Ok(one),
        [] => Err(Failure::invalid(EngineError::new("cli.signal_unknown").with("name", wanted).with("path", path.display()))),
        many => {
            let ids: Vec<&str> = many.iter().map(|signal| signal.id.as_str()).collect();
            Err(Failure::invalid(EngineError::new("cli.signal_ambiguous").with("name", wanted).with("ids", ids.join(", "))))
        }
    }
}

pub async fn fire(ctx: &Ctx, args: FireArgs) -> Exit {
    match fire_inner(ctx, args).await {
        Ok(exit) => exit,
        Err(failure) => failure.report(ctx, "✖ "),
    }
}

async fn fire_inner(ctx: &Ctx, args: FireArgs) -> Result<Exit, Failure> {
    let (path, library) = library(args.library)?;
    let signal = find(&library, &args.signal, &path)?;
    let service = engine();
    let texts = &ctx.texts;
    let name = signal.name.as_str();
    let (value, line) = match &signal.body {
        SignalBody::Osc { target, address, args } => {
            let bytes = invoke(&service, "osc_send", json!({ "target": target, "address": address, "args": args })).await?;
            let line = texts.t("log.firedOsc", &params! { "name" => name, "address" => address.as_str(), "target" => target.as_str(), "bytes" => bytes.as_u64().unwrap_or_default() });
            (bytes, line)
        }
        SignalBody::Udp { target, payload } => {
            let result = invoke(&service, "broadcast_send", datagram(target, payload)).await?;
            if result["errors"].as_u64().unwrap_or_default() > 0 {
                let error = serde_json::from_value::<EngineError>(result["error"].clone()).unwrap_or_else(|_| EngineError::new("transport.failed").with("target", target));
                return Err(Failure::sending(error));
            }
            let line = texts.t("log.firedUdp", &params! { "name" => name, "target" => target.as_str(), "bytes" => result["bytes"].as_u64().unwrap_or_default() });
            (result, line)
        }
        SignalBody::Http { request } => {
            let value = invoke(&service, "http_request", json!({ "request": request })).await?;
            let response: HttpResponse = serde_json::from_value(value.clone())
                .map_err(|error| Failure::sending(EngineError::new("command.reply_invalid").because(error)))?;
            if let Some(error) = response_failure(&response, &request.url) {
                return Err(Failure { error, exit: Exit::Failed, remote: false });
            }
            let line = texts.t(
                "log.firedHttp",
                &params! { "name" => name, "method" => request.method.as_str(), "url" => request.url.as_str(), "status" => u64::from(response.status), "ms" => response.latency_ms.round() as u64 },
            );
            (value, line)
        }
        SignalBody::Mqtt { broker, topic, payload, qos, retain } => {
            let summary = invoke(&service, "mqtt_publish_once", mqtt_publish(broker, topic, payload, *qos, *retain)?).await?;
            let line = texts.t("log.firedMqttOnce", &params! { "name" => name, "summary" => summary.as_str().unwrap_or_default() });
            (summary, line)
        }
    };
    Ok(done(ctx, &value, line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc_arguments_are_typed_or_inferred() {
        let args: Vec<Value> = ["i:3", "f:0.5", "d:1.5", "h:4294967296", "s:hi there", "b:de ad", "T", "F", "N", "7", "2.5", "word", "s:7", "-3"]
            .iter()
            .map(|text| serde_json::to_value(osc_arg(text).unwrap()).unwrap())
            .collect();
        assert_eq!(
            args,
            [
                json!({ "type": "int", "value": 3 }),
                json!({ "type": "float", "value": 0.5 }),
                json!({ "type": "double", "value": 1.5 }),
                json!({ "type": "long", "value": 4_294_967_296_i64 }),
                json!({ "type": "str", "value": "hi there" }),
                json!({ "type": "blob", "value": [0xde, 0xad] }),
                json!({ "type": "bool", "value": true }),
                json!({ "type": "bool", "value": false }),
                json!({ "type": "nil" }),
                json!({ "type": "int", "value": 7 }),
                json!({ "type": "float", "value": 2.5 }),
                json!({ "type": "str", "value": "word" }),
                json!({ "type": "str", "value": "7" }),
                json!({ "type": "int", "value": -3 }),
            ]
        );
        for bad in ["i:x", "f:", "b:abc", "q:1"] {
            assert_eq!(osc_arg(bad).unwrap_err().code, "cli.osc_arg_invalid", "{bad}");
        }
    }

    #[test]
    fn a_broker_is_host_and_port_and_each_publish_has_its_own_client() {
        let first = mqtt_publish("127.0.0.1:1883", "a/b", "1", 1, true).unwrap();
        assert_eq!((first["config"]["host"].as_str(), first["config"]["port"].as_u64()), (Some("127.0.0.1"), Some(1883)));
        assert_eq!(mqtt_publish("[::1]:1884", "a", "", 0, false).unwrap()["config"]["host"], "::1");
        assert_eq!(mqtt_publish("broker", "a", "", 0, false).unwrap()["config"]["port"], 1883, "the MQTT port when none is given");
        assert_ne!(first["config"]["client_id"], mqtt_publish("127.0.0.1:1883", "a/b", "1", 1, true).unwrap()["config"]["client_id"]);
        assert_eq!(mqtt_publish("x:notaport", "a", "", 0, false).unwrap_err().error.code, "transport.target_invalid");
    }

    #[test]
    fn a_signal_is_found_by_id_then_by_name() {
        let library: Library = serde_json::from_value(json!({ "version": 2, "signals": [
            { "id": "go", "name": "Go", "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/go", "args": [] } },
            { "id": "a1", "name": "Twice", "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/a", "args": [] } },
            { "id": "a2", "name": "twice", "body": { "transport": "osc", "target": "127.0.0.1:9000", "address": "/b", "args": [] } },
        ] }))
        .unwrap();
        let path = std::path::Path::new("signals.json");
        assert_eq!(find(&library, "go", path).unwrap().id, "go");
        assert_eq!(find(&library, "GO", path).unwrap().id, "go", "a name ignores case");
        assert_eq!(find(&library, "a2", path).unwrap().id, "a2", "an id wins over a name");
        assert_eq!(find(&library, "Twice", path).unwrap_err().error.code, "cli.signal_ambiguous");
        assert_eq!(find(&library, "nothing", path).unwrap_err().error.code, "cli.signal_unknown");
    }

    #[test]
    fn hex_is_pairs_of_digits() {
        assert_eq!(hex_bytes("de ad:be-ef"), Some(vec![0xde, 0xad, 0xbe, 0xef]));
        assert_eq!(hex_bytes("abc"), None);
        assert_eq!(hex_bytes(""), None);
        assert_eq!(hex_bytes("zz"), None);
    }
}
