//! Document serialization and filesystem persistence. No execution or UI state.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use super::error::{EngineError, EngineResult};
use super::experiment::{starter, Experiment, LEGACY_VERSIONS, VERSION};
use super::experiment_validate::validate_document;
use super::jobs::now_ms;
use super::paths::data_dir;

pub const MAX_DOCUMENT_BYTES: usize = 4 * 1024 * 1024;

fn too_large() -> EngineError {
    EngineError::new("file.too_large").with("max", MAX_DOCUMENT_BYTES / (1024 * 1024))
}

/// A filesystem problem, naming the file; the system's wording is the detail.
fn io_error(path: &Path, error: impl ToString) -> EngineError {
    EngineError::new("file.io").with("path", path.display()).because(error)
}

pub fn parse(text: &str) -> EngineResult<Experiment> {
    if text.len() > MAX_DOCUMENT_BYTES {
        return Err(too_large());
    }
    let mut document: Experiment = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|error| {
        EngineError::new("file.json_invalid").with("line", error.line()).with("column", error.column()).because(error)
    })?;
    // Older versions lack parameters, seed or profiles; serde defaults supply them.
    if LEGACY_VERSIONS.contains(&document.version) {
        // Before version 8 no request carried cookies from an earlier one.
        if document.version < 8 {
            document.cookies = false;
        }
        document.version = VERSION;
    }
    // Enforce the canonical size too, so an accepted import can be saved again.
    encode(&document)?;
    Ok(document)
}

fn encode(document: &Experiment) -> EngineResult<Vec<u8>> {
    validate_document(document)?;
    let bytes = serde_json::to_vec_pretty(document).map_err(|error| EngineError::new("file.encode").because(error))?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(too_large());
    }
    Ok(bytes)
}

/// A file that does not parse is reported with its path, never replaced.
fn read(path: &Path) -> EngineResult<Experiment> {
    let file = fs::File::open(path).map_err(|error| io_error(path, error))?;
    let mut text = String::new();
    file.take((MAX_DOCUMENT_BYTES + 1) as u64)
        .read_to_string(&mut text)
        .map_err(|error| io_error(path, error))?;
    parse(&text).map_err(|error| error.with("path", path.display()))
}

pub fn load() -> EngineResult<Experiment> {
    let path = data_dir().join("experiment.json");
    match fs::metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(starter()),
        Err(error) => Err(io_error(&path, error)),
        Ok(_) => read(&path),
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> EngineResult<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| io_error(path, error))?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(io_error(path, error));
    }
    Ok(())
}

fn save_in(directory: &Path, document: &Experiment) -> EngineResult<String> {
    let bytes = encode(document)?;
    fs::create_dir_all(directory).map_err(|error| io_error(directory, error))?;
    let path = directory.join("experiment.json");
    let temporary = directory.join(format!(".experiment-{:016x}.tmp", rand::random::<u64>()));
    write_new(&temporary, &bytes)?;
    if let Err(error) = fs::rename(&temporary, &path) {
        let _ = fs::remove_file(&temporary);
        return Err(io_error(&path, error));
    }
    Ok(path.display().to_string())
}

pub fn save(document: &Experiment) -> EngineResult<String> {
    save_in(&data_dir(), document)
}

fn export_in(directory: &Path, document: &Experiment) -> EngineResult<String> {
    let bytes = encode(document)?;
    fs::create_dir_all(directory).map_err(|error| io_error(directory, error))?;
    // Never use a document name as a path; each export is a separate snapshot.
    let path = directory.join(format!(
        "experiment-{}-{:016x}.json",
        now_ms(),
        rand::random::<u64>()
    ));
    write_new(&path, &bytes)?;
    Ok(path.display().to_string())
}

pub fn export(document: &Experiment) -> EngineResult<String> {
    export_in(&data_dir().join("exports"), document)
}

#[cfg(test)]
mod tests {
    use super::super::experiment_validate::validate;
    use super::*;

    #[test]
    fn bundled_templates_are_executable_and_round_trip() {
        for text in [
            include_str!("../../experiments/templates/empty.json"),
            include_str!("../../experiments/templates/http-check.json"),
            include_str!("../../experiments/templates/status-branch.json"),
            include_str!("../../experiments/templates/parallel-flows.json"),
            include_str!("../../experiments/templates/osc-ping-reply.json"),
            include_str!("../../experiments/templates/poll-until-ready.json"),
            include_str!("../../experiments/templates/flaky-api.json"),
            include_str!("../../experiments/templates/fault-phases.json"),
            include_str!("../../experiments/templates/dependency-outage.json"),
            include_str!("../../experiments/templates/websocket-echo.json"),
        ] {
            let document = parse(text).unwrap();
            validate(&document).unwrap();
            let decoded = parse(std::str::from_utf8(&encode(&document).unwrap()).unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(document).unwrap(),
                serde_json::to_value(decoded).unwrap()
            );
        }
    }

    #[test]
    fn imports_accept_drafts_and_legacy_edges_but_reject_broken_documents() {
        let mut value = serde_json::to_value(starter()).unwrap();
        value["edges"][0].as_object_mut().unwrap().remove("port");
        let document = parse(&value.to_string()).unwrap();
        assert_eq!(document.edges[0].port, "next");
        value["edges"] = serde_json::json!([]);
        assert!(parse(&format!("\u{feff}{value}")).is_ok());
        value["version"] = 999.into();
        let version = parse(&value.to_string()).err().unwrap();
        assert_eq!((version.code.as_str(), version.params["version"].as_str()), ("doc.version_unsupported", "999"));
    }

    #[test]
    fn older_documents_open_as_the_current_version() {
        for version in LEGACY_VERSIONS {
            let mut value = serde_json::to_value(starter()).unwrap();
            value["version"] = (*version).into();
            for field in ["params", "seed", "profiles", "profile"] {
                value.as_object_mut().unwrap().remove(field);
            }
            let document = parse(&value.to_string()).unwrap();
            assert_eq!(document.version, VERSION);
            assert_eq!(document.cookies, *version >= 8, "version {version}: cookies off only for files from before the jar");
            assert!(document.params.is_empty() && document.seed.is_none());
            assert!(document.profiles.is_empty() && document.profile.is_none());
            let saved: serde_json::Value = serde_json::from_slice(&encode(&document).unwrap()).unwrap();
            assert_eq!(saved["version"], VERSION);
            assert_eq!(saved["profiles"], serde_json::json!([]));
        }
    }

    #[test]
    fn invalid_inputs_never_become_documents() {
        let broken = parse("{\n  \"version\": 3,\n  oops").unwrap_err();
        assert_eq!((broken.code.as_str(), broken.params["line"].as_str()), ("file.json_invalid", "3"));
        assert!(broken.detail.is_some(), "the parser's own message is kept");
        assert!(parse(&" ".repeat(MAX_DOCUMENT_BYTES + 1)).unwrap_err().is("file.too_large"));
        let original = serde_json::to_value(starter()).unwrap();
        for change in [
            "duplicate",
            "missing target",
            "unknown node",
            "negative position",
            "missing end",
        ] {
            let mut value = original.clone();
            match change {
                "duplicate" => value["nodes"][1]["id"] = value["nodes"][0]["id"].clone(),
                "missing target" => value["edges"][0]["to"] = "absent".into(),
                "negative position" => value["nodes"][1]["x"] = (-100).into(),
                "missing end" => {
                    value["nodes"].as_array_mut().unwrap().pop();
                    value["edges"] = serde_json::json!([]);
                }
                _ => value["nodes"][1]["type"] = "unknown".into(),
            }
            assert!(parse(&value.to_string()).is_err(), "{change}");
        }
    }

    #[test]
    fn snapshots_are_independent_and_failed_save_keeps_previous_file() {
        let directory = std::env::temp_dir().join(format!(
            "signallab-files-test-{:016x}",
            rand::random::<u64>()
        ));
        let mut document = starter();
        let saved = save_in(&directory, &document).unwrap();
        let first = export_in(&directory, &document).unwrap();
        document.name = "second".into();
        save_in(&directory, &document).unwrap();
        let second = export_in(&directory, &document).unwrap();
        assert_ne!(first, second);
        assert_ne!(read(Path::new(&first)).unwrap().name, "second");
        assert_eq!(read(Path::new(&saved)).unwrap().name, "second");
        document.version = 999;
        assert!(save_in(&directory, &document).is_err());
        assert_eq!(read(Path::new(&saved)).unwrap().name, "second");
        for entry in fs::read_dir(&directory).unwrap() {
            fs::remove_file(entry.unwrap().path()).unwrap();
        }
        fs::remove_dir(directory).unwrap();
    }
}
