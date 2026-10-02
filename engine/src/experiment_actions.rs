//! Performing one network action of an experiment, and saying in a code why
//! it failed. Shared by the runner and *Send now*, so a single send is the
//! same code as a step in a run.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use crate::host::Host;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::broadcast::{self, EmitConfig, Payload, TargetMode};
use super::error::{EngineError, EngineResult, Field};
use super::experiment::NodeKind;
use super::http::{self, HttpResponse};
use super::inspect::{self, Frame};
use super::listen::Listener;
use super::mqtt::{self, MqttCause, MqttFailure};
use super::osc;
use super::osc_codec::{arg_str, encode_message};
use super::transport::{self, Cause};

/// A one-shot publish gets this long for connect, publish and acknowledgement.
const MQTT_TIMEOUT: Duration = Duration::from_secs(15);
/// How long a TCP step listens for a reply after sending.
const TCP_REPLY_WINDOW: Duration = Duration::from_millis(250);

/// What one network action did; HTTP keeps its response for checks and extraction.
pub struct ActionOutcome {
    pub detail: String,
    pub response: Option<HttpResponse>,
}

fn sent(detail: String) -> ActionOutcome {
    ActionOutcome { detail, response: None }
}

/// `host:port`, with brackets around an IPv6 address.
pub(crate) fn host_port(host: &str, port: u16) -> String {
    match host.trim().parse::<IpAddr>() {
        Ok(IpAddr::V6(ip)) => format!("[{ip}]:{port}"),
        _ => format!("{}:{port}", host.trim()),
    }
}

fn io_error(error: std::io::Error, target: &str) -> EngineError {
    transport::of_io(&error).error(target).because(error)
}

/// The MQTT screen's error, about the node's broker or topic field.
pub(crate) fn mqtt_error(failure: MqttFailure, broker: &str) -> EngineError {
    let field = if failure.cause == MqttCause::Topic { Field::new("topic") } else { Field::new("broker") };
    failure.error(broker).in_field(field)
}

async fn tcp(host: &str, port: u16, payload: &str, timeout_ms: u64) -> EngineResult<ActionOutcome> {
    let target = host_port(host, port);
    let in_host = |error: EngineError| error.in_field(Field::new("host"));
    let address = transport::resolve(&target).await.map_err(in_host)?;
    let exchange = async {
        let mut stream = tokio::net::TcpStream::connect(address).await.map_err(|error| io_error(error, &target))?;
        stream.write_all(payload.as_bytes()).await.map_err(|error| io_error(error, &target))?;
        let mut buffer = [0u8; 1024];
        let reply = match tokio::time::timeout(TCP_REPLY_WINDOW, stream.read(&mut buffer)).await {
            Ok(Ok(size)) if size > 0 => format!(" · {size} B reply"),
            _ => String::new(),
        };
        Ok(sent(format!("{} B → {target}{reply}", payload.len())))
    };
    match tokio::time::timeout(Duration::from_millis(timeout_ms), exchange).await {
        Ok(result) => result.map_err(in_host),
        Err(_) => Err(in_host(Cause::Timeout.error(&target).with("ms", timeout_ms))),
    }
}

async fn http_request(host: &Host, request: &http::HttpRequest) -> EngineResult<ActionOutcome> {
    let in_url = |error: EngineError| error.in_field(Field::new("url"));
    let response = http::request_once(host.clone(), request.clone()).await.map_err(in_url)?;
    match &response.error {
        Some(error) => {
            let cause = response.cause.unwrap_or(Cause::Failed);
            let failed = cause.error(&request.url).because(error);
            Err(in_url(if cause == Cause::Timeout { failed.with("ms", request.timeout_ms) } else { failed }))
        }
        None => Ok(ActionOutcome {
            detail: format!("HTTP {} · {:.0} ms", response.status, response.latency_ms),
            response: Some(response),
        }),
    }
}

async fn osc_message(host: &Host, target: &str, address: &str, args: &[super::osc_codec::OscArg]) -> EngineResult<ActionOutcome> {
    let in_target = |error: EngineError| error.in_field(Field::new("target"));
    let to: SocketAddr = target.trim().parse().map_err(|_| in_target(Cause::TargetInvalid.error(target)))?;
    let bytes = osc::send_to(host, to, address.to_string(), args).await.map_err(|error| in_target(io_error(error, target)))?;
    Ok(sent(format!("{bytes} B → {to}")))
}

async fn udp(host: &Host, target: &str, text: &str) -> EngineResult<ActionOutcome> {
    let in_target = |error: EngineError| error.in_field(Field::new("target"));
    // Resolved here, so a name that does not resolve is reported as such.
    let mut resolved = Vec::new();
    for part in target.split([',', ';', '\n']).map(str::trim).filter(|part| !part.is_empty()) {
        resolved.push(transport::resolve(part).await.map_err(in_target)?.to_string());
    }
    if resolved.is_empty() {
        return Err(in_target(EngineError::new("node.required")));
    }
    let config = EmitConfig {
        mode: TargetMode::List,
        target: resolved.join(","),
        port: 0,
        payload: Payload::Text { text: text.to_string() },
        bind: None,
        ttl: 1,
        multicast_loop: true,
        rate: 0.0,
        count: 1,
        duration_s: 0.0,
    };
    let result = broadcast::send_once(host.clone(), config).await.map_err(in_target)?;
    if result.errors > 0 {
        let (cause, text) = result.first_error.unwrap_or((Cause::Failed, String::new()));
        let failed = cause.error(target).with("failed", result.errors).with("total", result.targets).because(text);
        return Err(in_target(failed));
    }
    Ok(sent(format!("{} B → {target}", result.bytes)))
}

/// Perform one resolved network action.
pub async fn execute(host: &Host, client_id: String, kind: &NodeKind) -> EngineResult<ActionOutcome> {
    match kind {
        NodeKind::Tcp { host: target_host, port, payload, timeout_ms } => tcp(target_host, *port, payload, *timeout_ms).await,
        NodeKind::Http { request } => http_request(host, request).await,
        NodeKind::Mqtt { host: broker_host, port, topic, payload, qos, retain } => {
            let broker = host_port(broker_host, *port);
            let config = mqtt::MqttConfig {
                host: broker_host.clone(),
                port: *port,
                client_id,
                username: String::new(),
                password: String::new(),
                keep_alive_s: 60,
                clean_session: true,
                will: None,
                subscribe: vec![],
            };
            let publish = mqtt::publish(host.clone(), config, topic.clone(), payload.clone(), *qos, *retain);
            match tokio::time::timeout(MQTT_TIMEOUT, publish).await {
                Ok(result) => result.map(sent).map_err(|failure| mqtt_error(failure, &broker)),
                Err(_) => {
                    let timeout = Cause::Timeout.error(&broker).with("ms", MQTT_TIMEOUT.as_millis());
                    Err(timeout.in_field(Field::new("broker")))
                }
            }
        }
        NodeKind::Osc { target, address, args, .. } => osc_message(host, target, address, args).await,
        NodeKind::Udp { target, text, .. } => udp(host, target, text).await,
        _ => Err(EngineError::new("run.not_an_action")),
    }
}

/// Send an OSC message or a datagram from `listener`'s socket, for a step that
/// waits there for the answer. Reported to the Inspector like any send.
pub async fn send_via(host: &Host, listener: &Listener, kind: &NodeKind) -> EngineResult<ActionOutcome> {
    let in_target = |error: EngineError| error.in_field(Field::new("target"));
    let (proto, packet, summary, targets) = match kind {
        NodeKind::Osc { target, address, args, .. } => {
            let to: SocketAddr = target.trim().parse().map_err(|_| in_target(Cause::TargetInvalid.error(target)))?;
            let summary = std::iter::once(address.clone()).chain(args.iter().map(arg_str)).collect::<Vec<_>>().join(" ");
            ("osc", encode_message(address, args), summary, vec![to])
        }
        NodeKind::Udp { target, text, .. } => {
            let mut resolved = Vec::new();
            for part in target.split([',', ';', '\n']).map(str::trim).filter(|part| !part.is_empty()) {
                resolved.push(transport::resolve(part).await.map_err(in_target)?);
            }
            if resolved.is_empty() {
                return Err(in_target(EngineError::new("node.required")));
            }
            ("udp", text.as_bytes().to_vec(), inspect::ascii_preview(text.as_bytes(), 96), resolved)
        }
        _ => return Err(EngineError::new("run.not_an_action")),
    };
    for to in &targets {
        listener.send_to(&packet, *to).await.map_err(|error| in_target(io_error(error, &to.to_string())))?;
        if inspect::armed(host) {
            inspect::publish(host, Frame::tx(proto, "experiment").local(listener.local()).remote(to).payload(&packet).summary(summary.clone()));
        }
    }
    let shown: Vec<String> = targets.iter().map(SocketAddr::to_string).collect();
    Ok(sent(format!("{} B → {} (from {})", packet.len(), shown.join(", "), listener.local())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv6_hosts_get_brackets() {
        assert_eq!(host_port(" ::1 ", 80), "[::1]:80");
        assert_eq!(host_port("127.0.0.1", 80), "127.0.0.1:80");
        assert_eq!(host_port("broker.local", 1883), "broker.local:1883");
    }

    #[tokio::test]
    async fn tcp_failures_are_classified() {
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let refused = tcp("127.0.0.1", closed, "hi", 8000).await.err().unwrap();
        assert_eq!((refused.code.as_str(), refused.field.clone()), ("transport.refused", Some(Field::new("host"))));
        let dns = tcp("no-such-host.invalid", 80, "hi", 2000).await.err().unwrap();
        assert_eq!(dns.code, "transport.dns");

        let open = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = open.local_addr().unwrap().port();
        let done = tcp("127.0.0.1", port, "hello", 2000).await.unwrap();
        assert!(done.detail.starts_with("5 B → 127.0.0.1:"), "{}", done.detail);
    }

    #[test]
    fn mqtt_causes_become_codes() {
        let failure = |cause| MqttFailure { cause, text: "broker said no".into() };
        assert_eq!(mqtt_error(failure(MqttCause::Refused(5)), "b:1883").params["code"], "5");
        assert_eq!(mqtt_error(failure(MqttCause::Transport(Cause::Refused)), "b:1883").code, "transport.refused");
        let topic = mqtt_error(failure(MqttCause::Topic), "b:1883");
        assert_eq!((topic.code.as_str(), topic.field.as_ref().unwrap().key.as_str(), topic.detail.as_deref()), ("mqtt.topic_invalid", "topic", Some("broker said no")));
    }
}
