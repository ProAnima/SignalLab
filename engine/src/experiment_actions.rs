//! Performing one network action of an experiment, and saying in a code why
//! it failed. Shared by the runner and *Send now*, so a single send is the
//! same code as a step in a run.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use crate::host::Host;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::broadcast::{self, EmitConfig, Payload, TargetMode};
use super::error::{EngineError, EngineResult, Field};
use super::experiment::NodeKind;
use super::cookies::CookieJar;
use super::http::{self, DigestOutcome, HttpResponse};
use super::inspect::{self, Frame};
use super::listen::Listener;
use super::mqtt;
use super::mqtt_dial::{self, MqttCause, MqttFailure};
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

/// Connect, write the payload, read what comes back within [`TCP_REPLY_WINDOW`].
/// What was written and what was read reach the Inspector like any send.
async fn tcp(host: &Host, target_host: &str, port: u16, payload: &str, timeout_ms: u64) -> EngineResult<ActionOutcome> {
    let target = host_port(target_host, port);
    let in_host = |error: EngineError| error.in_field(Field::new("host"));
    let address = transport::resolve(&target).await.map_err(in_host)?;
    let exchange = async {
        let mut stream = tokio::net::TcpStream::connect(address).await.map_err(|error| io_error(error, &target))?;
        let local = stream.local_addr().map(|local| local.to_string()).unwrap_or_default();
        let frame = |frame: Frame, bytes: &[u8]| frame.local(&local).remote(address).payload(bytes).summary(inspect::ascii_preview(bytes, 96));
        stream.write_all(payload.as_bytes()).await.map_err(|error| io_error(error, &target))?;
        if inspect::armed(host) {
            inspect::publish(host, frame(Frame::tx("tcp", "experiment"), payload.as_bytes()));
        }
        let mut buffer = [0u8; 1024];
        let reply = match tokio::time::timeout(TCP_REPLY_WINDOW, stream.read(&mut buffer)).await {
            Ok(Ok(size)) if size > 0 => {
                if inspect::armed(host) {
                    inspect::publish(host, frame(Frame::rx("tcp", "experiment"), &buffer[..size]));
                }
                format!(" · {size} B reply")
            }
            _ => String::new(),
        };
        Ok(sent(format!("{} B → {target}{reply}", payload.len())))
    };
    match tokio::time::timeout(Duration::from_millis(timeout_ms), exchange).await {
        Ok(result) => result.map_err(in_host),
        Err(_) => Err(in_host(Cause::Timeout.error(&target).with("ms", timeout_ms))),
    }
}

async fn http_request(host: &Host, request: &http::HttpRequest, cookies: Option<Arc<CookieJar>>) -> EngineResult<ActionOutcome> {
    let in_url = |error: EngineError| error.in_field(Field::new("url"));
    let response = http::request_once(host.clone(), request.clone(), cookies).await.map_err(in_url)?;
    match &response.error {
        Some(error) => {
            let cause = response.cause.unwrap_or(Cause::Failed);
            let failed = cause.error(&request.url).because(error);
            Err(in_url(if cause == Cause::Timeout { failed.with("ms", request.timeout_ms) } else { failed }))
        }
        None => match &response.digest {
            // A Digest that could not be answered: the 401 stands, and the reason with it.
            Some(DigestOutcome { error: Some(error), .. }) => Err(error.clone().in_field(Field::new("auth"))),
            _ => Ok(ActionOutcome {
                detail: format!("HTTP {} · {:.0} ms", response.status, response.latency_ms),
                response: Some(response),
            }),
        },
    }
}

async fn osc_message(host: &Host, target: &str, address: &str, args: &[super::osc_codec::OscArg]) -> EngineResult<ActionOutcome> {
    let in_target = |error: EngineError| error.in_field(Field::new("target"));
    // Rendered, an address may have lost its slash; a name is looked up here.
    osc::check_address(address)?;
    let to = transport::resolve(target).await.map_err(in_target)?;
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
/// `cookies`: the run's jar, when the experiment keeps cookies.
pub async fn execute(host: &Host, client_id: String, kind: &NodeKind, cookies: Option<Arc<CookieJar>>) -> EngineResult<ActionOutcome> {
    match kind {
        NodeKind::Tcp { host: target_host, port, payload, timeout_ms } => tcp(host, target_host, *port, payload, *timeout_ms).await,
        NodeKind::Http { request } => http_request(host, request, cookies).await,
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
            let publish = mqtt_dial::publish(host.clone(), config, topic.clone(), payload.clone(), *qos, *retain);
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
            osc::check_address(address)?;
            let to = transport::resolve(target).await.map_err(in_target)?;
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

    fn test_host() -> (Host, inspect::Capture) {
        let capture = inspect::Capture::new();
        (Host::new(Arc::new(crate::host::NoEvents), capture.clone()), capture)
    }

    #[tokio::test]
    async fn tcp_failures_are_classified() {
        let (host, _) = test_host();
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let refused = tcp(&host, "127.0.0.1", closed, "hi", 8000).await.err().unwrap();
        assert_eq!((refused.code.as_str(), refused.field.clone()), ("transport.refused", Some(Field::new("host"))));
        let dns = tcp(&host, "no-such-host.invalid", 80, "hi", 2000).await.err().unwrap();
        assert_eq!(dns.code, "transport.dns");

        let open = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = open.local_addr().unwrap().port();
        let done = tcp(&host, "127.0.0.1", port, "hello", 2000).await.unwrap();
        assert!(done.detail.starts_with("5 B → 127.0.0.1:"), "{}", done.detail);
    }

    /// What a TCP step writes and what it reads back are Inspector frames,
    /// with the secret values in use masked — and nothing while capture is off.
    #[tokio::test]
    async fn tcp_exchanges_reach_the_inspector_masked() {
        let device = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = device.local_addr().unwrap().port();
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = device.accept().await else { return };
                let mut buffer = [0u8; 64];
                let size = socket.read(&mut buffer).await.unwrap_or(0);
                let _ = socket.write_all(&[b"ACK ", &buffer[..size]].concat()).await;
            }
        });
        let (host, capture) = test_host();
        let _secret = crate::secrets::redact(vec!["tcp-s3cret-value".into()]);

        tcp(&host, "127.0.0.1", port, "GO tcp-s3cret-value", 2000).await.unwrap();
        assert!(capture.snapshot(16).is_empty(), "disarmed, nothing is captured");

        capture.set_enabled(true);
        let done = tcp(&host, "127.0.0.1", port, "GO tcp-s3cret-value", 2000).await.unwrap();
        assert!(done.detail.ends_with("23 B reply"), "{}", done.detail);
        let frames = capture.snapshot(16);
        let shown: Vec<(&str, &str, &str, usize)> = frames.iter().map(|frame| (frame.proto.as_str(), frame.dir.as_str(), frame.source.as_str(), frame.bytes)).collect();
        assert_eq!(shown, [("tcp", "tx", "experiment", 19), ("tcp", "rx", "experiment", 23)]);
        for frame in &frames {
            assert_eq!(frame.remote, format!("127.0.0.1:{port}"));
            assert!(frame.local.starts_with("127.0.0.1:"), "{}", frame.local);
            assert!(frame.summary.starts_with(if frame.dir == "tx" { "GO " } else { "ACK GO " }), "{}", frame.summary);
            let kept = capture.payload(frame.seq).unwrap();
            assert!(!frame.summary.contains("s3cret") && !kept.hex.contains(&inspect::plain_hex(b"s3cret")), "{} {}", frame.summary, kept.dump);
            assert!(kept.hex.contains(&inspect::plain_hex(b"GO ****")), "{}", kept.dump);
        }
    }

    #[test]
    fn mqtt_causes_become_codes() {
        let failure = |cause| MqttFailure { cause, text: "broker said no".into() };
        assert_eq!(mqtt_error(failure(MqttCause::Refused(5)), "b:1883").params["code"], "5");
        assert_eq!(mqtt_error(failure(MqttCause::Transport(Cause::Refused)), "b:1883").code, "transport.refused");
        let topic = mqtt_error(failure(MqttCause::Topic), "b:1883");
        assert_eq!((topic.code.as_str(), topic.field.as_ref().unwrap().key.as_str(), topic.detail.as_deref()), ("mqtt.topic_invalid", "topic", Some("broker said no")));
    }

    /// `localhost` may list ::1 first; a device listening on IPv4 only still gets
    /// what an OSC or UDP node sends to `localhost:<port>`.
    #[tokio::test]
    async fn osc_and_udp_nodes_reach_an_ipv4_listener_by_name() {
        let (host, _) = test_host();
        let device = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let target = format!("localhost:{}", device.local_addr().unwrap().port());
        let mut buffer = [0u8; 64];
        let osc = NodeKind::Osc { target: target.clone(), address: "/go".into(), args: vec![], reply: None };
        execute(&host, "lab".into(), &osc, None).await.unwrap();
        let (size, _) = tokio::time::timeout(Duration::from_secs(2), device.recv_from(&mut buffer)).await.expect("the OSC message arrived").unwrap();
        assert_eq!(crate::osc_codec::decode_packet(&buffer[..size]).unwrap()[0].address, "/go");
        let udp = NodeKind::Udp { target: target.clone(), text: "hello".into(), reply: None };
        execute(&host, "lab".into(), &udp, None).await.unwrap();
        let (size, _) = tokio::time::timeout(Duration::from_secs(2), device.recv_from(&mut buffer)).await.expect("the datagram arrived").unwrap();
        assert_eq!(&buffer[..size], b"hello");
        let slashless = NodeKind::Osc { target, address: "go".into(), args: vec![], reply: None };
        assert!(execute(&host, "lab".into(), &slashless, None).await.err().unwrap().is("node.osc_address"));
    }
}
