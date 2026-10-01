//! The signal library: named packets the operator can fire again without
//! rebuilding them field by field.
//!
//! Only storage lives here. Firing a signal goes back out through the ordinary
//! commands — `osc_send`, `broadcast_send`, `http_request` — so a fired signal
//! is indistinguishable from a hand-typed one, lands in the Inspector under its
//! real source, and no send path exists twice.
//!
//! The file is plain JSON in `~/Documents/SignalLab/signals.json`: readable,
//! hand-editable, and small enough to commit next to the project whose gear it
//! describes.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::http::HttpRequest;
use super::osc_codec::OscArg;

/// The library file, in the data folder (`paths::data_dir`).
fn library_path() -> PathBuf {
    super::paths::data_dir().join("signals.json")
}

/// Bytes for a raw signal. Deliberately narrower than the broadcast module's
/// payload: an OSC message is what the `osc` transport is for, and a raw signal
/// that could secretly be OSC would need two ways to say the same thing.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RawPayload {
    Text { text: String },
    Hex { hex: String },
}

/// What a signal actually puts on the wire. The tag is the transport, so adding
/// one (MQTT next) is a new variant and not a new shape.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "lowercase")]
pub enum SignalBody {
    /// One OSC message to one host:port.
    Osc {
        target: String,
        address: String,
        #[serde(default)]
        args: Vec<OscArg>,
    },
    /// Opaque UDP: text or a hex dump, which is how a captured frame comes back.
    Udp { target: String, payload: RawPayload },
    /// A single HTTP request.
    Http { request: HttpRequest },
    /// One MQTT publish. No credentials are stored here on purpose: the library
    /// file is a plain document in someone's Documents folder. A signal fired
    /// while a connection is open rides that connection, with its own auth.
    Mqtt {
        /// host:port of the broker.
        broker: String,
        topic: String,
        #[serde(default)]
        payload: String,
        #[serde(default)]
        qos: u8,
        #[serde(default)]
        retain: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub name: String,
    /// Folder in the explorer. Empty means the ungrouped root.
    #[serde(default)]
    pub group: String,
    /// Why this signal exists and what it should make happen.
    #[serde(default)]
    pub note: String,
    pub body: SignalBody,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Library {
    pub version: u32,
    #[serde(default)]
    pub signals: Vec<Signal>,
}

/// The library plus where it came from, so the UI can point at the file.
#[derive(Clone, Debug, Serialize)]
pub struct LibraryFile {
    pub path: String,
    pub library: Library,
    /// True when the file did not exist and the starter set was written.
    pub seeded: bool,
}

const VERSION: u32 = 1;

fn sig(id: &str, group: &str, name: &str, note: &str, body: SignalBody) -> Signal {
    Signal {
        id: id.to_string(),
        name: name.to_string(),
        group: group.to_string(),
        note: note.to_string(),
        body,
    }
}

fn osc(target: &str, address: &str, args: Vec<OscArg>) -> SignalBody {
    SignalBody::Osc {
        target: target.to_string(),
        address: address.to_string(),
        args,
    }
}

/// The starter library: one worked example per transport, chosen for the thing
/// about the protocol that is easy to get wrong.
///
/// Every target is loopback on purpose. These are recipes meant to be pointed at
/// real gear, and a shipped default aimed at a host we do not own is how a test
/// packet ends up in someone else's show.
pub fn seed() -> Library {
    Library {
        version: VERSION,
        signals: vec![
            sig(
                "osc-fader",
                "OSC",
                "Fader value",
                "A single float on an address — the shape most show-control gear listens for.",
                osc("127.0.0.1:9000", "/fader/1", vec![OscArg::Float(0.75)]),
            ),
            sig(
                "osc-types",
                "OSC",
                "Every argument type",
                "What the far end decodes back, for a device that is picky about type tags. Note that                  an OSC bool is a tag (T or F) and carries no payload byte — sending the string                  \"true\" instead is the classic silent drop.",
                osc(
                    "127.0.0.1:9000",
                    "/types",
                    vec![
                        OscArg::Int(-7),
                        OscArg::Float(1.5),
                        OscArg::Str("hi".into()),
                        OscArg::Bool(true),
                        OscArg::Long(4_294_967_296),
                        OscArg::Double(0.125),
                        OscArg::Nil,
                    ],
                ),
            ),
            sig(
                "osc-id-and-value",
                "OSC",
                "Identifier and value",
                "Two strings: which device, and what it read. Card readers and scanners usually report                  this way, and receivers often check the first argument and ignore anything that does                  not match — so a mismatch looks exactly like nothing arriving at all.",
                osc(
                    "127.0.0.1:9000",
                    "/tag",
                    vec![
                        OscArg::Str("reader-1".into()),
                        OscArg::Str("04a1b2c3".into()),
                    ],
                ),
            ),
            sig(
                "osc-trigger",
                "OSC",
                "Trigger with no arguments",
                "An address on its own. A type tag string is still on the wire, which strict parsers                  require and hand-rolled ones often forget to send.",
                osc("127.0.0.1:9000", "/cue/go", Vec::new()),
            ),
            sig(
                "mqtt-publish",
                "MQTT",
                "Publish a value",
                "QoS 0: out on the wire and forgotten. Fine for telemetry, wrong for a command you                  need to know landed.",
                SignalBody::Mqtt {
                    broker: "127.0.0.1:1883".into(),
                    topic: "lab/example/value".into(),
                    payload: "1".into(),
                    qos: 0,
                    retain: false,
                },
            ),
            sig(
                "mqtt-retained",
                "MQTT",
                "Set a retained value",
                "The broker keeps it and hands it to every client that subscribes afterwards — which                  is how a device picks up its configuration at boot without asking anyone for it.",
                SignalBody::Mqtt {
                    broker: "127.0.0.1:1883".into(),
                    topic: "lab/example/config".into(),
                    payload: "night".into(),
                    qos: 1,
                    retain: true,
                },
            ),
            sig(
                "mqtt-clear-retained",
                "MQTT",
                "Clear a retained value",
                "An empty payload with retain set is the only way to remove one. A stale retained                  value is a classic reason a device boots into the wrong state with nothing in the                  logs to explain it.",
                SignalBody::Mqtt {
                    broker: "127.0.0.1:1883".into(),
                    topic: "lab/example/config".into(),
                    payload: String::new(),
                    qos: 1,
                    retain: true,
                },
            ),
            sig(
                "http-reachable",
                "HTTP",
                "Is the service up?",
                "A plain reachability check. A refused connection is reported as an error rather than                  a status code, which is the difference between \"wrong answer\" and \"nobody home\".",
                SignalBody::Http {
                    request: HttpRequest {
                        method: "GET".into(),
                        url: "http://127.0.0.1:8080/".into(),
                        headers: Vec::new(),
                        body: None,
                        timeout_ms: 4_000,
                    },
                },
            ),
            sig(
                "udp-raw",
                "Raw",
                "Raw UDP bytes",
                "An opaque payload — the same form a frame saved out of the Inspector takes, so a                  captured packet replays byte for byte.",
                SignalBody::Udp {
                    target: "127.0.0.1:9000".into(),
                    payload: RawPayload::Hex {
                        hex: "de ad be ef".into(),
                    },
                },
            ),
        ],
    }
}

/// Read the library, writing the starter set the first time.
pub fn load() -> Result<LibraryFile, String> {
    let path = library_path();
    let shown = path.display().to_string();
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let library: Library = serde_json::from_str(&text)
                // Naming the file matters more than naming the serde path: the
                // fix is to edit or delete it, and it is hand-editable by design.
                .map_err(|e| format!("{shown} is not a valid signal library: {e}"))?;
            Ok(LibraryFile {
                path: shown,
                library,
                seeded: false,
            })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let library = seed();
            save(&library)?;
            Ok(LibraryFile {
                path: shown,
                library,
                seeded: true,
            })
        }
        Err(e) => Err(format!("cannot read {shown}: {e}")),
    }
}

/// Replace the file with this library. The UI owns the list and hands back the
/// whole thing — with a few dozen entries there is nothing to be gained from a
/// merge protocol, and plenty to lose.
pub fn save(library: &Library) -> Result<String, String> {
    let dir = super::paths::data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let path = library_path();
    let text =
        serde_json::to_string_pretty(library).map_err(|e| format!("cannot encode library: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_round_trips_through_json() {
        let text = serde_json::to_string(&seed()).unwrap();
        let back: Library = serde_json::from_str(&text).unwrap();
        assert_eq!(back.version, VERSION);
        assert_eq!(back.signals.len(), seed().signals.len());
        assert!(
            back.signals.iter().any(|s| matches!(s.body, SignalBody::Mqtt { .. })),
            "the mqtt variant has to survive the round trip too"
        );
    }

    #[test]
    fn seed_ids_are_unique() {
        let library = seed();
        let mut ids: Vec<&str> = library.signals.iter().map(|s| s.id.as_str()).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "duplicate signal id in the seed library");
    }

    /// Storm, Scanner and Broadcast reach real hosts. A shipped default that
    /// points anywhere but this machine is the one bug that cannot be caught
    /// after the fact.
    #[test]
    fn seed_targets_stay_on_loopback() {
        for s in seed().signals {
            let target = match &s.body {
                SignalBody::Osc { target, .. } | SignalBody::Udp { target, .. } => target.clone(),
                SignalBody::Http { request } => request.url.clone(),
                SignalBody::Mqtt { broker, .. } => broker.clone(),
            };
            assert!(
                target.contains("127.0.0.1") || target.contains("localhost"),
                "seed signal '{}' aims at {target}",
                s.name
            );
        }
    }

    #[test]
    fn transport_tag_is_the_discriminant() {
        let body: SignalBody =
            serde_json::from_str(r#"{"transport":"osc","target":"127.0.0.1:9000","address":"/x"}"#)
                .unwrap();
        match body {
            SignalBody::Osc { address, args, .. } => {
                assert_eq!(address, "/x");
                assert!(args.is_empty(), "args must default to empty");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn a_broken_file_names_itself_instead_of_being_replaced() {
        let err = serde_json::from_str::<Library>("{ not json }")
            .map_err(|e| format!("{} is not a valid signal library: {e}", library_path().display()))
            .unwrap_err();
        assert!(err.contains("signals.json"), "error should name the file: {err}");
    }
}
