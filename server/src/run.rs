//! `POST /api/run`: run an experiment to its end — how a script or a CI
//! pipeline uses the server. The request names the experiment, as a document
//! or a bundled template, and what to run it with:
//!
//! ```json
//! { "document": { … } | "template": "http-check",
//!   "overrides": { "api": "http://10.0.0.5" }, "profile": "Stage", "seed": 42, "timeout": 60 }
//! ```
//!
//! By default the response is the result, once the run has ended:
//! `{ outcome, seed, profile, error?, steps, report_path?, … }` — a run that
//! fails is still a `200` with `"outcome": "failed"`; an HTTP error means no
//! run was started (`400` the request could not be read, `422` the experiment
//! could not start — the same `EngineError` `/api/invoke` gives). Until then the
//! body carries a space every 15 s, which JSON ignores, so a proxy does not
//! take a quiet run for a dead connection.
//!
//! With `Accept: application/x-ndjson` the response is one JSON object per
//! line as things happen: `{"type":"started"}`, a `{"type":"step"}` per step,
//! `{"type":"heartbeat"}` every 15 s, then `{"type":"ended", …the result}`.
//!
//! The run is a job like one started from the interface: it is listed, can be
//! stopped from there, and sends its events to every page. A client that goes
//! away does not stop it — it runs to its end and saves its report, as a run
//! started in a browser does when the tab is closed. When the server shuts
//! down the run is stopped and the response ends with `"outcome": "stopped"`.

use std::collections::BTreeMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;

use axum::body::{Body, Bytes};
use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use signal_lab_engine::error::EngineError;
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_files;
use signal_lab_engine::experiment_run::{Progress, RunHandle, RunOptions};
use signal_lab_engine::service::Failure;
use tokio::sync::watch;

use crate::routes::{failure, is_json, AppState};

/// The experiments the app's template chooser offers, by file name.
pub const TEMPLATES: &[(&str, &str)] = &[
    ("empty", include_str!("../../experiments/templates/empty.json")),
    ("http-check", include_str!("../../experiments/templates/http-check.json")),
    ("status-branch", include_str!("../../experiments/templates/status-branch.json")),
    ("parallel-flows", include_str!("../../experiments/templates/parallel-flows.json")),
    ("osc-ping-reply", include_str!("../../experiments/templates/osc-ping-reply.json")),
    ("poll-until-ready", include_str!("../../experiments/templates/poll-until-ready.json")),
    ("flaky-api", include_str!("../../experiments/templates/flaky-api.json")),
    ("fault-phases", include_str!("../../experiments/templates/fault-phases.json")),
    ("dependency-outage", include_str!("../../experiments/templates/dependency-outage.json")),
    ("websocket-echo", include_str!("../../experiments/templates/websocket-echo.json")),
];

/// How often a quiet response says it is still there.
const HEARTBEAT: Duration = Duration::from_secs(15);
pub const NDJSON: &str = "application/x-ndjson";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunRequest {
    #[serde(default)]
    document: Option<Value>,
    #[serde(default)]
    template: Option<String>,
    /// Parameter values for this run only: strings, numbers or booleans.
    #[serde(default)]
    overrides: BTreeMap<String, Value>,
    #[serde(default)]
    seed: Option<u64>,
    /// Run with this profile; "" runs with the defaults.
    #[serde(default)]
    profile: Option<String>,
    /// Seconds before the run fails with `run.timeout` (at most the engine's limit).
    #[serde(default)]
    timeout: Option<u64>,
}

/// Why no run was started: the status and the error the body carries.
type Refusal = (StatusCode, EngineError);

fn unreadable(detail: impl ToString) -> Refusal {
    (StatusCode::BAD_REQUEST, EngineError::new("api.run_invalid").because(detail))
}

/// A bundled template by its file name, with or without `.json`.
pub fn template(name: &str) -> Option<&'static str> {
    let name = name.strip_suffix(".json").unwrap_or(name);
    TEMPLATES.iter().find(|(known, _)| *known == name).map(|(_, text)| *text)
}

impl RunRequest {
    /// The document as it will run, and the options it runs with.
    fn prepare(self) -> Result<(Experiment, RunOptions), Refusal> {
        let text = match (self.document, self.template) {
            (Some(document), None) => document.to_string(),
            (None, Some(name)) => match template(&name) {
                Some(text) => text.to_string(),
                None => return Err((StatusCode::UNPROCESSABLE_ENTITY, EngineError::new("api.template_unknown").with("name", name))),
            },
            _ => return Err((StatusCode::BAD_REQUEST, EngineError::new("api.run_source"))),
        };
        // Parsed as an imported file is: older versions are migrated, a broken one refused.
        let mut document = experiment_files::parse(&text).map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, error))?;
        if let Some(profile) = self.profile {
            document.profile = (!profile.is_empty()).then_some(profile);
        }
        let mut overrides = BTreeMap::new();
        for (name, value) in self.overrides {
            let value = match value {
                Value::String(text) => text,
                Value::Number(number) => number.to_string(),
                Value::Bool(flag) => flag.to_string(),
                other => return Err(unreadable(format!("overrides.{name}: a string, number or boolean, not {other}"))),
            };
            overrides.insert(name, value);
        }
        let limit = self.timeout.map(Duration::from_secs);
        Ok((document, RunOptions { overrides, seed: self.seed, limit }))
    }
}

/// The client asked for the steps as they happen.
fn wants_lines(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::ACCEPT)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|kind| kind.split(';').next().is_some_and(|kind| kind.trim().eq_ignore_ascii_case(NDJSON)))
}

pub async fn run(State(app): State<AppState>, ConnectInfo(peer): ConnectInfo<SocketAddr>, headers: HeaderMap, body: Bytes) -> Response {
    if !is_json(&headers) {
        return failure(StatusCode::UNSUPPORTED_MEDIA_TYPE, EngineError::new("command.json_required"));
    }
    let request: RunRequest = match serde_json::from_slice(&body) {
        Ok(request) => request,
        Err(error) => return failure(StatusCode::BAD_REQUEST, unreadable(error).1),
    };
    let (document, options) = match request.prepare() {
        Ok(prepared) => prepared,
        Err((status, error)) => return failure(status, error),
    };
    let handle = match app.service.run(document, options).await {
        Ok(handle) => handle,
        Err(error) => {
            tracing::debug!(client = %peer, command = "run", "run refused");
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(Failure::Engine(error))).into_response();
        }
    };
    // Logged like every job start, with who started it.
    tracing::info!(client = %peer, command = "run", job = handle.info.id, "job started");
    follow(app, handle, wants_lines(&headers))
}

/// What the response is at.
enum Phase {
    Starting,
    Running,
    Done,
}

/// The response body: the run as lines, or its result at the end.
struct Follow {
    app: AppState,
    handle: RunHandle,
    lines: bool,
    phase: Phase,
    heartbeat: tokio::time::Interval,
    stopping: watch::Receiver<bool>,
}

/// One NDJSON line: `item` with its `type`.
fn line(kind: &str, item: &impl Serialize) -> Bytes {
    let mut value = serde_json::to_value(item).unwrap_or_else(|_| Value::Object(Default::default()));
    if let Value::Object(map) = &mut value {
        map.insert("type".into(), kind.into());
    }
    let mut text = value.to_string();
    text.push('\n');
    Bytes::from(text)
}

impl Follow {
    async fn next_chunk(&mut self) -> Option<Bytes> {
        match self.phase {
            Phase::Done => return None,
            Phase::Starting => {
                self.phase = Phase::Running;
                if self.lines {
                    return Some(line("started", &self.handle.started));
                }
            }
            Phase::Running => {}
        }
        loop {
            tokio::select! {
                progress = self.handle.next() => match progress {
                    Some(Progress::Step(step)) if self.lines => return Some(line("step", &step)),
                    Some(Progress::Step(_)) => {}
                    Some(Progress::Ended(result)) => {
                        self.phase = Phase::Done;
                        return Some(if self.lines { line("ended", &result) } else { Bytes::from(serde_json::to_vec(&result).unwrap_or_default()) });
                    }
                    None => {
                        self.phase = Phase::Done;
                        return None;
                    }
                },
                _ = self.heartbeat.tick() => {
                    return Some(if self.lines { line("heartbeat", &serde_json::json!({})) } else { Bytes::from_static(b" ") });
                }
                // The server is stopping: so is this run, and the response ends with that.
                Ok(()) = self.stopping.changed() => {
                    if *self.stopping.borrow() {
                        self.app.service.jobs().stop(self.handle.info.id);
                    }
                }
            }
        }
    }
}

fn follow(app: AppState, handle: RunHandle, lines: bool) -> Response {
    let stopping = app.stopping.clone();
    let heartbeat = tokio::time::interval_at(tokio::time::Instant::now() + HEARTBEAT, HEARTBEAT);
    let state = Follow { app, handle, lines, phase: Phase::Starting, heartbeat, stopping };
    // A client that disconnects drops this stream and the handle with it; the run goes on.
    let body = futures_util::stream::unfold(state, |mut state| async move {
        let chunk = state.next_chunk().await?;
        Some((Ok::<_, Infallible>(chunk), state))
    });
    let kind = if lines { NDJSON } else { "application/json" };
    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static(kind)),
            // nginx would otherwise hold the lines back until its buffer is full.
            (header::HeaderName::from_static("x-accel-buffering"), HeaderValue::from_static("no")),
        ],
        Body::from_stream(body),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bundled_template_is_offered_and_parses() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../experiments/templates");
        let mut files: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter_map(|name| name.strip_suffix(".json").map(str::to_string))
            .collect();
        files.sort();
        let mut offered: Vec<String> = TEMPLATES.iter().map(|(name, _)| name.to_string()).collect();
        offered.sort();
        assert_eq!(offered, files, "TEMPLATES lists experiments/templates exactly");
        for (name, text) in TEMPLATES {
            experiment_files::parse(text).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
        assert!(template("empty.json").is_some() && template("empty").is_some() && template("nothing").is_none());
    }

    #[test]
    fn a_request_names_one_experiment_and_values_of_any_scalar_kind() {
        let request = |value: Value| serde_json::from_value::<RunRequest>(value).unwrap().prepare();
        let (document, options) = request(serde_json::json!({
            "template": "osc-ping-reply", "overrides": { "device": "127.0.0.1:9", "n": 3, "on": true }, "profile": "", "seed": 7, "timeout": 30,
        }))
        .unwrap();
        assert_eq!(document.name, "OSC ping → reply");
        assert_eq!(document.profile, None, "an empty profile is the defaults");
        assert_eq!((options.overrides["n"].as_str(), options.overrides["on"].as_str(), options.seed), ("3", "true", Some(7)));
        assert_eq!(options.limit, Some(Duration::from_secs(30)));
        assert_eq!(request(serde_json::json!({})).unwrap_err().1.code, "api.run_source");
        assert_eq!(request(serde_json::json!({ "template": "empty", "document": {} })).unwrap_err().1.code, "api.run_source");
        assert_eq!(request(serde_json::json!({ "template": "nope" })).unwrap_err().1.code, "api.template_unknown");
        assert_eq!(request(serde_json::json!({ "document": { "version": 5 } })).unwrap_err().1.code, "file.json_invalid");
        let (status, error) = request(serde_json::json!({ "template": "empty", "overrides": { "x": [1] } })).unwrap_err();
        assert_eq!((status, error.code.as_str()), (StatusCode::BAD_REQUEST, "api.run_invalid"));
        assert!(serde_json::from_value::<RunRequest>(serde_json::json!({ "template": "empty", "sede": 1 })).is_err(), "a misspelled field is refused");
    }

    #[test]
    fn lines_are_asked_for_by_accept() {
        let headers = |accept: &str| {
            let mut headers = HeaderMap::new();
            headers.insert(header::ACCEPT, HeaderValue::from_str(accept).unwrap());
            headers
        };
        assert!(wants_lines(&headers("application/x-ndjson")));
        assert!(wants_lines(&headers("application/json;q=0.5, Application/X-NDJSON")));
        assert!(!wants_lines(&headers("application/json")));
        assert!(!wants_lines(&HeaderMap::new()));
        assert_eq!(&line("step", &serde_json::json!({ "a": 1 }))[..], b"{\"a\":1,\"type\":\"step\"}\n");
    }
}
