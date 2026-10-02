//! The emulator library: the emulators someone keeps, in
//! `Documents/SignalLab/emulators.json` (the data folder), written whole by the
//! interface like the signal library. A broken file is reported with its path,
//! never replaced with the starter set. The command line reads the same file,
//! or a file of its own that holds one emulator, a list, or a library.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::emulator::Emulator;
use super::error::{EngineError, EngineResult};

pub const VERSION: u32 = 1;
const MAX_FILE: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredEmulator {
    pub id: String,
    /// What it stands in for, in the person's words.
    #[serde(default)]
    pub note: String,
    pub emulator: Emulator,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmulatorLibrary {
    pub version: u32,
    #[serde(default)]
    pub emulators: Vec<StoredEmulator>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LibraryFile {
    pub path: String,
    pub library: EmulatorLibrary,
    /// The file did not exist and the starter set was written.
    pub seeded: bool,
}

pub fn library_path() -> PathBuf {
    super::paths::data_dir().join("emulators.json")
}

fn stored(id: &str, note: &str, emulator: Value) -> StoredEmulator {
    StoredEmulator { id: id.into(), note: note.into(), emulator: serde_json::from_value(emulator).expect("a starter emulator is a valid document") }
}

/// The starter set: one of each, each showing what it is for. Every bind is
/// loopback on purpose — reaching the network is a choice a person makes.
pub fn seed() -> EmulatorLibrary {
    EmulatorLibrary {
        version: VERSION,
        emulators: vec![
            stored(
                "demo-api",
                "An HTTP API to point an app at: a health check, a user by id, a create, a slow answer and one that fails twice before it works.",
                json!({
                    "name": "Demo API", "bind": "127.0.0.1:8080", "protocol": "http",
                    "routes": [
                        { "method": "GET", "path": "/health", "responses": [{ "body": "{\"status\":\"ok\",\"time\":\"{{now.iso}}\"}" }] },
                        { "method": "GET", "path": "/users/:id", "responses": [{ "body": "{\"id\":\"{{request.params.id}}\",\"name\":\"User {{request.params.id}}\"}" }] },
                        { "method": "POST", "path": "/users", "responses": [{ "status": 201, "headers": [["Location", "/users/{{counter}}"]], "body": "{\"id\":\"{{counter}}\"}" }] },
                        { "method": "GET", "path": "/slow", "responses": [{ "body": "{\"slow\":true}", "delay_ms": 1500 }] },
                        { "method": "ANY", "path": "/flaky", "responses": [{ "status": 503 }, { "status": 503 }, { "body": "{\"ok\":true,\"attempt\":{{counter}}}" }] }
                    ]
                }),
            ),
            stored(
                "osc-device",
                "Answers /ping with /pong and the same number, acknowledges a fader, and takes cues without a word.",
                json!({
                    "name": "Demo OSC device", "bind": "127.0.0.1:9100", "protocol": "osc",
                    "rules": [
                        { "address": "/ping", "reply": { "address": "/pong", "args": [{ "type": "int", "value": "{{counter}}" }] } },
                        { "address": "/fader/*", "reply": { "address": "/ack", "args": [{ "type": "str", "value": "{{request.address}}" }] } },
                        { "address": "/cue/*" }
                    ]
                }),
            ),
            stored(
                "udp-device",
                "Answers PING with PONG and anything else with how many bytes it got.",
                json!({
                    "name": "Demo UDP device", "bind": "127.0.0.1:7100", "protocol": "udp",
                    "rules": [
                        { "mode": "contains", "pattern": "PING", "reply": { "kind": "text", "text": "PONG {{counter}}" } },
                        { "mode": "any", "reply": { "kind": "text", "text": "ACK {{request.bytes}}" } }
                    ]
                }),
            ),
            stored(
                "tcp-device",
                "A line protocol like a projector's: greets, reports and switches power, and hangs up on QUIT.",
                json!({
                    "name": "Demo TCP device", "bind": "127.0.0.1:7200", "protocol": "tcp", "delimiter": "crlf", "greeting": "READY",
                    "rules": [
                        { "mode": "regex", "pattern": "^POWER\\?$", "reply": { "kind": "text", "text": "POWER=ON" } },
                        { "mode": "regex", "pattern": "^POWER (ON|OFF)$", "reply": { "kind": "text", "text": "OK {{request.match}}" } },
                        { "mode": "contains", "pattern": "QUIT", "reply": { "kind": "text", "text": "BYE" }, "close": true }
                    ]
                }),
            ),
            stored(
                "mqtt-broker",
                "A broker to point gear and a controller at: a lamp that reports its state when told ON or OFF, and a status that is retained.",
                json!({
                    "name": "Demo MQTT broker", "bind": "127.0.0.1:1883", "protocol": "mqtt",
                    "retained": [{ "topic": "lab/status", "payload": "online" }],
                    "rules": [
                        { "topic": "lab/+/set", "mode": "regex", "pattern": "^(ON|OFF)$",
                          "reply": { "topic": "lab/{{request.levels[1]}}/state", "payload": "{{request.match}}", "retain": true } }
                    ]
                }),
            ),
        ],
    }
}

fn io_error(path: &Path, error: impl ToString) -> EngineError {
    EngineError::new("file.io").with("path", path.display()).because(error)
}

fn invalid(path: &Path, error: serde_json::Error) -> EngineError {
    EngineError::new("emulators.json_invalid").with("path", path.display()).with("line", error.line()).with("column", error.column()).because(error)
}

fn parse(text: &str, path: &Path) -> EngineResult<EmulatorLibrary> {
    serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|error| invalid(path, error))
}

/// Read the library, writing the starter set the first time.
pub fn load() -> EngineResult<LibraryFile> {
    let path = library_path();
    let shown = path.display().to_string();
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(LibraryFile { path: shown, library: parse(&text, &path)?, seeded: false }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let library = seed();
            save(&library)?;
            Ok(LibraryFile { path: shown, library, seeded: true })
        }
        Err(error) => Err(io_error(&path, error)),
    }
}

/// Replace the file with this library, through a temporary file, so a
/// failed write leaves the previous one.
pub fn save(library: &EmulatorLibrary) -> EngineResult<String> {
    let dir = super::paths::data_dir();
    std::fs::create_dir_all(&dir).map_err(|error| io_error(&dir, error))?;
    let path = library_path();
    let text = serde_json::to_string_pretty(library).map_err(|error| EngineError::new("emulators.encode").because(error))?;
    let temporary = dir.join(format!(".emulators-{:016x}.tmp", rand::random::<u64>()));
    std::fs::write(&temporary, text).map_err(|error| io_error(&temporary, error))?;
    if let Err(error) = std::fs::rename(&temporary, &path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(io_error(&path, error));
    }
    Ok(path.display().to_string())
}

/// The emulators in a file of the command line's: one emulator, a list of
/// them, or a library as the app writes it.
pub fn parse_file(text: &str, path: &Path) -> EngineResult<Vec<Emulator>> {
    if text.len() > MAX_FILE {
        return Err(EngineError::new("file.too_large").with("max", MAX_FILE / (1024 * 1024)));
    }
    let value: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|error| invalid(path, error))?;
    fn read<T: serde::de::DeserializeOwned>(value: Value, path: &Path) -> EngineResult<T> {
        serde_json::from_value(value).map_err(|error| invalid(path, error))
    }
    match &value {
        Value::Object(object) if object.contains_key("emulators") => Ok(read::<EmulatorLibrary>(value, path)?.emulators.into_iter().map(|stored| stored.emulator).collect()),
        Value::Array(_) => read(value, path),
        _ => Ok(vec![read(value, path)?]),
    }
}

/// The library entry named `wanted`: its id, or its name ignoring case.
pub fn find<'a>(library: &'a EmulatorLibrary, wanted: &str) -> Option<&'a StoredEmulator> {
    let wanted = wanted.trim();
    library.emulators.iter().find(|stored| stored.id == wanted).or_else(|| library.emulators.iter().find(|stored| stored.emulator.name.trim().eq_ignore_ascii_case(wanted)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulator::{self, EmulatorKind};
    use std::collections::BTreeMap;

    #[test]
    fn the_starter_set_is_valid_round_trips_and_stays_on_loopback() {
        let library = seed();
        let text = serde_json::to_string(&library).unwrap();
        let back: EmulatorLibrary = serde_json::from_str(&text).unwrap();
        assert_eq!(back.emulators.len(), 5);
        let mut protocols: Vec<&str> = back.emulators.iter().map(|stored| stored.emulator.kind.protocol()).collect();
        protocols.sort_unstable();
        assert_eq!(protocols, ["http", "mqtt", "osc", "tcp", "udp"], "one of each");
        for stored in &back.emulators {
            emulator::check(&stored.emulator, &BTreeMap::new()).unwrap_or_else(|error| panic!("{}: {error}", stored.id));
            assert!(stored.emulator.bind.starts_with("127.0.0.1:"), "{} listens on {}", stored.id, stored.emulator.bind);
            if let EmulatorKind::Osc { rules } = &stored.emulator.kind {
                assert!(rules.iter().all(|rule| rule.to.is_empty()), "replies go back to the sender");
            }
            for text in [&stored.emulator.name, &stored.note] {
                assert!(!text.contains("  ") && text.trim() == text.as_str(), "{}: {text:?}", stored.id);
            }
        }
        let mut ids: Vec<&str> = back.emulators.iter().map(|stored| stored.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn files_hold_one_a_list_or_a_library_and_a_broken_one_names_itself() {
        let path = Path::new("mock.json");
        let one = r#"{ "name": "API", "bind": "127.0.0.1:8080", "protocol": "http" }"#;
        assert_eq!(parse_file(one, path).unwrap().len(), 1);
        assert_eq!(parse_file(&format!("[{one}, {one}]"), path).unwrap().len(), 2);
        let library = serde_json::to_string(&seed()).unwrap();
        assert_eq!(parse_file(&library, path).unwrap().len(), 5);
        let broken = parse_file("{\n  \"name\": \"x\",\n  oops", path).unwrap_err();
        assert_eq!((broken.code.as_str(), broken.params["line"].as_str(), broken.params["path"].as_str()), ("emulators.json_invalid", "3", "mock.json"));
        assert!(parse_file(r#"{ "name": "x", "bind": "1", "protocol": "smtp" }"#, path).unwrap_err().is("emulators.json_invalid"));
        let seeded = seed();
        assert_eq!(find(&seeded, "osc-device").unwrap().emulator.name, "Demo OSC device");
        assert_eq!(find(&seeded, " demo api ").unwrap().id, "demo-api");
        assert!(find(&seeded, "nothing").is_none());
    }
}
