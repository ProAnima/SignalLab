//! MQTT subscriptions for *Wait for MQTT* steps — what `listen` is for UDP. A
//! run subscribes before its first step, so a message published right after
//! the run's action is not missed, and a broker that refuses stops the run
//! before any traffic. Each subscription fills an `Inbox`, and a wait takes the
//! first message its matcher accepts, so two waits never match the same one.
//!
//! The subscription is QoS 0 on a clean session: what arrives is what the
//! broker forwards from now on. Retained messages the broker replays on
//! subscribing describe the past and are ignored.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::host::Host;

use super::error::{EngineError, EngineResult, Field};
use super::experiment_actions::{host_port, mqtt_error};
use super::inspect::{self, Frame};
use super::listen::Inbox;
use super::matching::Datagram;
use super::mqtt::MqttConfig;
use super::mqtt_dial;
use super::mqtt_codec::{decode, encode_pingreq, encode_puback, encode_subscribe, summarize, Packet};

/// A run's subscriptions, by broker (`host:port`) and topic filter.
pub type Subscriptions = HashMap<(String, String), Arc<Subscription>>;

/// Keep-alive the subscription asks for; a ping goes out at half of it.
const KEEP_ALIVE_S: u16 = 30;

/// A subscription filter as MQTT 3.1.1 allows it: `+` is a whole level, `#`
/// only the whole last level.
pub fn filter_valid(filter: &str) -> bool {
    let levels: Vec<&str> = filter.split('/').collect();
    !filter.is_empty()
        && !filter.contains('\0')
        && filter.len() <= u16::MAX as usize
        && levels.iter().enumerate().all(|(index, level)| {
            (!level.contains('+') || *level == "+") && (!level.contains('#') || (*level == "#" && index == levels.len() - 1))
        })
}

/// A topic a message may be published to: not empty, no wildcard, no NUL.
pub fn topic_name_valid(topic: &str) -> bool {
    !topic.is_empty() && !topic.contains(['+', '#', '\0']) && topic.len() <= u16::MAX as usize
}

/// Whether a message on `topic` is one `filter` subscribes to: `+` takes one
/// level, a final `#` this level and everything under it. A topic starting
/// with `$` is not taken by a filter starting with a wildcard.
pub fn topic_matches(filter: &str, topic: &str) -> bool {
    if topic.starts_with('$') && filter.starts_with(['+', '#']) {
        return false;
    }
    let mut levels = topic.split('/');
    for wanted in filter.split('/') {
        match (wanted, levels.next()) {
            ("#", _) => return true,
            ("+", Some(_)) => {}
            (wanted, Some(level)) if wanted == level => {}
            _ => return false,
        }
    }
    levels.next().is_none()
}

/// The key of a subscription in [`Subscriptions`].
pub fn key(host: &str, port: u16, filter: &str) -> (String, String) {
    (host_port(host, port), filter.to_string())
}

/// One broker connection subscribed to one filter. Dropping it closes the connection.
pub struct Subscription {
    broker: String,
    inbox: Arc<Inbox>,
    task: tokio::task::AbortHandle,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Subscription {
    /// Connect, subscribe and wait for the broker's answer; then keep reading in
    /// the background. A refusal is an error here, before the run sends anything.
    pub async fn arm(host: Host, broker_host: &str, port: u16, filter: &str) -> EngineResult<Subscription> {
        let broker = host_port(broker_host, port);
        let in_topic = |error: EngineError| error.in_field(Field::new("topic"));
        let config = MqttConfig {
            host: broker_host.trim().to_string(),
            port,
            // Unique and short (3.1.1 guarantees 23 characters), so no other client is knocked off.
            client_id: format!("lab-w{:08x}", rand::random::<u32>()),
            username: String::new(),
            password: String::new(),
            keep_alive_s: KEEP_ALIVE_S,
            clean_session: true,
            will: None,
            subscribe: vec![],
        };
        let (mut stream, broker) = mqtt_dial::dial(&config).await.map_err(|failure| mqtt_error(failure, &broker))?;
        let peer: SocketAddr = stream.peer_addr().unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], port)));
        let local = stream.local_addr().map(|address| address.to_string()).unwrap_or_default();
        let lost = |error: std::io::Error| EngineError::new("wait.receive_failed").with("target", &broker).because(error).in_field(Field::new("broker"));
        stream.write_all(&encode_subscribe(1, &[(filter.to_string(), 0)])).await.map_err(lost)?;

        // The answer to the SUBSCRIBE; anything else that arrives first is ignored.
        let mut buffer = Vec::with_capacity(256);
        let mut chunk = [0u8; 4096];
        let codes = tokio::time::timeout(mqtt_dial::HANDSHAKE_TIMEOUT, async {
            loop {
                let size = stream.read(&mut chunk).await.map_err(lost)?;
                if size == 0 {
                    return Err(EngineError::new("wait.receive_failed").with("target", &broker).because("the broker closed the connection").in_field(Field::new("broker")));
                }
                buffer.extend_from_slice(&chunk[..size]);
                while let Some((packet, used)) = decode(&buffer).map_err(|error| EngineError::new("mqtt.protocol").with("broker", &broker).because(error))? {
                    buffer.drain(..used);
                    if let Packet::SubAck { packet_id: 1, codes } = packet {
                        return Ok(codes);
                    }
                }
            }
        })
        .await
        .map_err(|_| EngineError::new("mqtt.no_answer").with("broker", &broker).in_field(Field::new("broker")))??;
        if codes.first().is_none_or(|code| *code == 0x80) {
            return Err(in_topic(EngineError::new("mqtt.subscribe_refused").with("filter", filter).with("broker", &broker)));
        }

        let inbox = Arc::new(Inbox::default());
        let filled = inbox.clone();
        let target = broker.clone();
        let task = tokio::spawn(async move {
            let (mut reader, mut writer) = stream.into_split();
            let mut ping = tokio::time::interval(Duration::from_secs(u64::from(KEEP_ALIVE_S) / 2));
            ping.tick().await;
            let failed = |reason: String| EngineError::new("wait.receive_failed").with("target", &target).because(reason);
            loop {
                tokio::select! {
                    read = reader.read(&mut chunk) => {
                        let size = match read {
                            Ok(0) => { filled.fail(failed("the broker closed the connection".into())); break; }
                            Ok(size) => size,
                            Err(error) => { filled.fail(failed(error.to_string())); break; }
                        };
                        buffer.extend_from_slice(&chunk[..size]);
                        loop {
                            match decode(&buffer) {
                                Ok(Some((packet, used))) => {
                                    buffer.drain(..used);
                                    if let Packet::Publish { qos, retain, topic, packet_id, payload, .. } = packet {
                                        // A QoS 0 subscription receives QoS 0; acknowledge anything else anyway.
                                        if qos == 1 && writer.write_all(&encode_puback(packet_id)).await.is_err() {
                                            break;
                                        }
                                        if retain {
                                            continue;
                                        }
                                        let frame = if inspect::armed(&host) {
                                            inspect::publish(&host, Frame::rx("mqtt", "experiment-wait").local(&local).remote(peer).payload(&payload).publish(&target, &topic, qos, retain).summary(summarize(&topic, &payload, qos, retain)))
                                        } else {
                                            None
                                        };
                                        filled.push(Datagram { bytes: payload, from: peer, at: Instant::now(), topic: Some(topic), frame, request: None, binary: false });
                                    }
                                }
                                Ok(None) => break,
                                Err(error) => { filled.fail(failed(format!("malformed packet: {error}"))); return; }
                            }
                        }
                    }
                    _ = ping.tick() => {
                        if writer.write_all(&encode_pingreq()).await.is_err() {
                            filled.fail(failed("the connection dropped".into()));
                            break;
                        }
                    }
                }
            }
        });
        Ok(Subscription { broker, inbox, task: task.abort_handle() })
    }

    pub fn broker(&self) -> &str {
        &self.broker
    }

    pub fn inbox(&self) -> &Inbox {
        &self.inbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_follow_mqtt_wildcard_rules() {
        for good in ["lab/#", "lab/+/state", "#", "+", "a/b/c", "+/+/#"] {
            assert!(filter_valid(good), "{good}");
        }
        for bad in ["", "lab/#/x", "lab/a#", "lab/+x", "a/b#", "a\0b"] {
            assert!(!filter_valid(bad), "{bad}");
        }
    }

    #[test]
    fn topics_match_filters_level_by_level() {
        for (filter, topic) in [("a/b", "a/b"), ("a/+", "a/b"), ("a/#", "a"), ("a/#", "a/b/c"), ("#", "a/b"), ("+/+", "a/b"), ("a/+/c", "a//c"), ("+", "")] {
            assert!(topic_matches(filter, topic), "{filter} takes {topic}");
        }
        for (filter, topic) in [("a/b", "a/b/c"), ("a/+", "a"), ("a/+", "a/b/c"), ("a/b", "a/B"), ("#", "$SYS/load"), ("+/load", "$SYS/load"), ("a/b/#", "a/c")] {
            assert!(!topic_matches(filter, topic), "{filter} does not take {topic}");
        }
        assert!(topic_matches("$SYS/#", "$SYS/load"));
        assert!(topic_name_valid("a/b") && !topic_name_valid("a/+") && !topic_name_valid("") && !topic_name_valid("a/#"));
    }
}
