//! Reaching an MQTT broker: dial and complete CONNECT/CONNACK before anything
//! else, so a wrong password or a closed port is an error where it was asked
//! for; failures kept as their cause (`MqttFailure`) until a person reads
//! them. And the one-shot publish the signal library and the MQTT node use,
//! which brings its own connection. The live connection is `mqtt.rs`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::host::Host;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::error::{EngineError, EngineResult};
use super::experiment_actions::host_port;
use super::inspect::{self, Frame};
use super::mqtt::MqttConfig;
use super::mqtt_codec::{connack_reason, decode, encode_connect, encode_disconnect, encode_publish, encode_pubrel, summarize, ConnectOpts, Packet};
use super::transport::{self, Cause};

pub(crate) const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(6);

pub(crate) fn connect_opts<'a>(cfg: &'a MqttConfig) -> ConnectOpts<'a> {
    ConnectOpts {
        client_id: &cfg.client_id,
        username: Some(cfg.username.as_str()).filter(|u| !u.is_empty()),
        password: Some(cfg.password.as_str()).filter(|p| !p.is_empty()),
        keep_alive_s: cfg.keep_alive_s,
        clean_session: cfg.clean_session,
        will: cfg.will.as_ref(),
    }
}

/// Why connecting or a one-shot publish failed: the cause, which becomes the
/// code a person reads, and the socket's or the broker's own wording.
#[derive(Debug)]
pub struct MqttFailure {
    pub cause: MqttCause,
    /// Technical detail (an OS error, a packet), possibly empty.
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MqttCause {
    /// The network: refused, unreachable, name not found, timed out, reset.
    Transport(Cause),
    /// CONNACK with this non-zero return code.
    Refused(u8),
    /// The socket is open but the broker did not finish the exchange in time.
    NoAnswer,
    /// The topic is empty or a filter; nothing was sent.
    Topic,
    /// Unexpected or malformed packets: probably not an MQTT broker.
    Protocol,
}

impl MqttFailure {
    pub(crate) fn new(cause: MqttCause, text: impl Into<String>) -> Self {
        MqttFailure { cause, text: text.into() }
    }

    pub(crate) fn io(error: &std::io::Error) -> Self {
        MqttFailure::new(MqttCause::Transport(transport::of_io(error)), error.to_string())
    }

    /// The failure as a person reads it, about `broker`; the text is its detail.
    pub fn error(self, broker: &str) -> EngineError {
        let error = match self.cause {
            MqttCause::Transport(cause) => cause.error(broker),
            // The return codes 3.1.1 defines each have a fix of their own.
            MqttCause::Refused(code) => match code {
                1 => EngineError::new("mqtt.refused_protocol"),
                2 => EngineError::new("mqtt.refused_client_id"),
                3 => EngineError::new("mqtt.refused_unavailable"),
                4 => EngineError::new("mqtt.refused_credentials"),
                5 => EngineError::new("mqtt.refused_not_authorized"),
                _ => EngineError::new("mqtt.refused"),
            }
            .with("broker", broker)
            .with("code", code),
            MqttCause::NoAnswer => EngineError::new("mqtt.no_answer").with("broker", broker),
            MqttCause::Topic => EngineError::new("mqtt.topic_invalid"),
            MqttCause::Protocol => EngineError::new("mqtt.protocol").with("broker", broker),
        };
        error.because(self.text)
    }
}

/// The broker a configuration dials, `host:port` (IPv6 in brackets).
pub fn broker_of(cfg: &MqttConfig) -> String {
    host_port(&cfg.host, cfg.port)
}

/// Dial and complete CONNECT/CONNACK before the job exists, so a wrong password
/// or a closed port is an error on the button rather than a job that dies a
/// moment later somewhere else.
pub(crate) async fn dial(cfg: &MqttConfig) -> Result<(TcpStream, String), MqttFailure> {
    let broker = broker_of(cfg);
    let mut stream = tokio::time::timeout(HANDSHAKE_TIMEOUT, TcpStream::connect(&broker))
        .await
        .map_err(|_| MqttFailure::new(MqttCause::Transport(Cause::Timeout), ""))?
        .map_err(|e| MqttFailure::io(&e))?;
    let _ = stream.set_nodelay(true);

    stream
        .write_all(&encode_connect(&connect_opts(cfg)))
        .await
        .map_err(|e| MqttFailure::io(&e))?;

    let mut buf = Vec::with_capacity(64);
    let mut chunk = [0u8; 512];
    let protocol = |text: String| MqttFailure::new(MqttCause::Protocol, text);
    let connack = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
        loop {
            match stream.read(&mut chunk).await {
                Ok(0) => return Err(protocol("closed the connection without a CONNACK".into())),
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(e) => return Err(MqttFailure::io(&e)),
            }
            match decode(&buf) {
                Ok(Some((Packet::ConnAck { code, .. }, _))) => return Ok(code),
                Ok(Some((other, _))) => return Err(protocol(format!("answered CONNECT with {other:?}"))),
                Ok(None) => continue,
                Err(e) => return Err(protocol(format!("malformed CONNACK: {e}"))),
            }
        }
    })
    .await
    .map_err(|_| MqttFailure::new(MqttCause::NoAnswer, ""))??;

    if connack != 0 {
        return Err(MqttFailure::new(MqttCause::Refused(connack), connack_reason(connack)));
    }
    Ok((stream, broker))
}

// ---------------------------------------------------------------------------
// one-shot publish, for the signal library
// ---------------------------------------------------------------------------

/// Publishing to a filter is a typo that costs a connection and comes back as a
/// confusing broker-side error, so it is refused before dialling.
pub(crate) fn validate_publish_topic(topic: &str) -> Option<EngineError> {
    if topic.trim().is_empty() {
        return Some(EngineError::new("mqtt.topic_required"));
    }
    if topic.contains('+') || topic.contains('#') {
        return Some(EngineError::new("node.topic_wildcard"));
    }
    None
}

/// Connect, publish, wait for the acknowledgement the QoS calls for, disconnect.
///
/// A library signal has to work with nothing set up, the same way an OSC signal
/// does — so it brings its own connection rather than requiring a live one.
pub async fn publish_once(
    host: Host,
    cfg: MqttConfig,
    topic: String,
    payload: String,
    qos: u8,
    retain: bool,
) -> EngineResult<String> {
    if let Some(refused) = validate_publish_topic(&topic) {
        return Err(refused);
    }
    let broker = broker_of(&cfg);
    publish(host, cfg, topic, payload, qos, retain).await.map_err(|failure| failure.error(&broker))
}

/// `publish_once` with the cause of a failure kept.
pub async fn publish(
    host: Host,
    cfg: MqttConfig,
    topic: String,
    payload: String,
    qos: u8,
    retain: bool,
) -> Result<String, MqttFailure> {
    if validate_publish_topic(&topic).is_some() {
        return Err(MqttFailure::new(MqttCause::Topic, ""));
    }
    // Brokers evict the older client when a new one arrives with the same id, so
    // a one-shot publish that reused it would knock the live connection off the
    // broker. Kept short: 3.1.1 only guarantees 23 characters are accepted.
    static ONCE: AtomicU64 = AtomicU64::new(0);
    let mut cfg = cfg;
    let base: String = cfg.client_id.trim().chars().take(12).collect();
    cfg.client_id = format!("{base}-o{:x}", ONCE.fetch_add(1, Ordering::Relaxed) & 0xffff);

    let (mut stream, broker) = dial(&cfg).await?;
    let qos = qos.min(2);
    let bytes = payload.into_bytes();
    let framed = encode_publish(&topic, &bytes, qos, retain, 1, false);

    if inspect::armed(&host) {
        let local = stream
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_default();
        inspect::publish(
            &host,
            Frame::tx("mqtt", "mqtt-send")
                .local(local)
                .remote(&broker)
                .payload(&bytes)
                .publish(&broker, &topic, qos, retain)
                .summary(summarize(&topic, &bytes, qos, retain))
                .verdict(if bytes.is_empty() && retain {
                    "clears retained"
                } else {
                    "one-shot"
                }),
        );
    }

    stream
        .write_all(&framed)
        .await
        .map_err(|e| MqttFailure::io(&e))?;

    // QoS 0 is done when the bytes are out; the others owe us a round trip, and
    // reporting success before it lands would defeat the point of asking for it.
    if qos > 0 {
        let mut buf = Vec::with_capacity(64);
        let mut chunk = [0u8; 512];
        let settled = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
            loop {
                match stream.read(&mut chunk).await {
                    Ok(0) => {
                        let text = "closed the connection before acknowledging the publish";
                        return Err(MqttFailure::new(MqttCause::Transport(Cause::Reset), text));
                    }
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    Err(e) => return Err(MqttFailure::io(&e)),
                }
                let mut at = 0usize;
                loop {
                    match decode(&buf[at..]) {
                        Ok(Some((packet, used))) => {
                            at += used;
                            match packet {
                                Packet::PubAck(_) if qos == 1 => return Ok(()),
                                Packet::PubRec(pid) => {
                                    stream
                                        .write_all(&encode_pubrel(pid))
                                        .await
                                        .map_err(|e| MqttFailure::io(&e))?;
                                }
                                Packet::PubComp(_) => return Ok(()),
                                _ => {}
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            return Err(MqttFailure::new(MqttCause::Protocol, format!("malformed acknowledgement: {e}")))
                        }
                    }
                }
                if at > 0 {
                    buf.drain(..at);
                }
            }
        })
        .await
        .map_err(|_| MqttFailure::new(MqttCause::NoAnswer, "the publish was never acknowledged"))?;
        settled?;
    }

    let _ = stream.write_all(&encode_disconnect()).await;
    Ok(format!(
        "{topic} → {broker} · {} B · qos{qos}{}",
        bytes.len(),
        if retain { " retained" } else { "" }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_are_only_sent_when_present() {
        let cfg = MqttConfig {
            host: "127.0.0.1".into(),
            port: 1883,
            client_id: "c".into(),
            username: String::new(),
            password: "ignored".into(),
            keep_alive_s: 60,
            clean_session: true,
            will: None,
            subscribe: Vec::new(),
        };
        let opts = connect_opts(&cfg);
        assert!(opts.username.is_none(), "an empty username must not be sent");
        assert!(opts.password.is_some(), "the codec decides, and it drops a lone password");
        let bytes = encode_connect(&opts);
        assert!(!String::from_utf8_lossy(&bytes).contains("ignored"));
    }

    #[test]
    fn failures_become_codes_about_the_broker() {
        let refused = |code| MqttFailure::new(MqttCause::Refused(code), connack_reason(code)).error("b:1883");
        let credentials = refused(4);
        assert_eq!((credentials.code.as_str(), credentials.params["broker"].as_str(), credentials.params["code"].as_str()), ("mqtt.refused_credentials", "b:1883", "4"));
        assert_eq!(credentials.detail.as_deref(), Some("bad username or password"));
        let codes: Vec<String> = [1, 2, 3, 5, 9].into_iter().map(|code| refused(code).into_code()).collect();
        assert_eq!(codes, ["mqtt.refused_protocol", "mqtt.refused_client_id", "mqtt.refused_unavailable", "mqtt.refused_not_authorized", "mqtt.refused"]);
        let lost = MqttFailure::io(&std::io::Error::from(std::io::ErrorKind::ConnectionRefused)).error("b:1883");
        assert_eq!((lost.code.as_str(), lost.params["target"].as_str()), ("transport.refused", "b:1883"));
        assert_eq!(MqttFailure::new(MqttCause::NoAnswer, "").error("b:1883").detail, None, "no text, no detail");
        assert_eq!(broker_of(&MqttConfig { host: " ::1 ".into(), ..config() }), "[::1]:1883");
    }

    fn config() -> MqttConfig {
        MqttConfig {
            host: "127.0.0.1".into(),
            port: 1883,
            client_id: "lab".into(),
            username: String::new(),
            password: String::new(),
            keep_alive_s: 60,
            clean_session: true,
            will: None,
            subscribe: Vec::new(),
        }
    }

    #[test]
    fn a_wildcard_cannot_be_published_to() {
        let refused = |topic: &str| validate_publish_topic(topic).map(EngineError::into_code);
        assert_eq!(refused("zone/+/command").as_deref(), Some("node.topic_wildcard"));
        assert_eq!(refused("global/#").as_deref(), Some("node.topic_wildcard"));
        assert_eq!(refused("   ").as_deref(), Some("mqtt.topic_required"));
        assert!(validate_publish_topic("site/device/command/restart").is_none());
        // A retained value is cleared by publishing an empty payload to a real
        // topic, so an empty *payload* must stay legal.
        assert!(validate_publish_topic("site/device/status/online").is_none());
    }
}
