//! The run report: one JSON file per finished run under `<data dir>/runs`,
//! with the values the run used, every step and the emulators' counters —
//! never written over another report.

use std::collections::BTreeMap;

use serde::Serialize;

use super::emulator_state::EmulatorSummary;
use super::netsim::ImpairmentSummary;
use super::error::{EngineError, EngineResult};
use super::experiment::Experiment;
use super::experiment_run::{Outcome, RunEvent};
use super::paths::data_dir;

/// Version of the run report file: 3 added the emulators' counters, 4 the
/// impairments' phases.
const REPORT_VERSION: u32 = 4;

#[derive(Serialize)]
struct RunReport<'a> {
    version: u32,
    experiment: &'a str,
    document_version: u32,
    seed: u64,
    profile: Option<&'a str>,
    overrides: &'a BTreeMap<String, String>,
    /// The values the run actually used.
    params: &'a BTreeMap<String, String>,
    started_ms: u64,
    ended_ms: u64,
    outcome: &'static str,
    error: &'a Option<EngineError>,
    steps: &'a [RunEvent],
    #[serde(skip_serializing_if = "<[EmulatorSummary]>::is_empty")]
    emulators: &'a [EmulatorSummary],
    #[serde(skip_serializing_if = "<[ImpairmentSummary]>::is_empty")]
    impairments: &'a [ImpairmentSummary],
}

pub(crate) struct RunSettings {
    pub(crate) seed: u64,
    pub(crate) overrides: BTreeMap<String, String>,
    pub(crate) params: BTreeMap<String, String>,
}

fn file_error(path: &std::path::Path, error: impl ToString) -> EngineError {
    EngineError::new("file.io").with("path", path.display()).because(error)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn save_report(id: u64, settings: &RunSettings, started_ms: u64, ended_ms: u64, doc: &Experiment, steps: &[RunEvent], emulators: &[EmulatorSummary], impairments: &[ImpairmentSummary], error: &Option<EngineError>) -> EngineResult<String> {
    let dir = data_dir().join("runs");
    std::fs::create_dir_all(&dir).map_err(|error| file_error(&dir, error))?;
    let report = RunReport {
        version: REPORT_VERSION,
        experiment: &doc.name,
        document_version: doc.version,
        seed: settings.seed,
        profile: doc.profile.as_deref(),
        overrides: &settings.overrides,
        params: &settings.params,
        started_ms,
        ended_ms,
        outcome: if error.is_some() { Outcome::Failed.as_str() } else { Outcome::Passed.as_str() },
        error,
        steps,
        emulators,
        impairments,
    };
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| file_error(&dir, error))?;
    // Never over another report: two processes sharing a data folder (command
    // lines in parallel CI jobs) can start job 1 in the same millisecond.
    let mut attempt = 1u32;
    loop {
        let name = if attempt == 1 { format!("run-{started_ms}-{id}.json") } else { format!("run-{started_ms}-{id}-{attempt}.json") };
        let path = dir.join(name);
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                use std::io::Write;
                file.write_all(&bytes).map_err(|error| file_error(&path, error))?;
                return Ok(path.display().to_string());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt < 1000 => attempt += 1,
            Err(error) => return Err(file_error(&path, error)),
        }
    }
}
