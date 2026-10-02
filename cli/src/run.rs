//! `signallab run`, `validate` and `templates`: experiments given as files or
//! bundled template names, run one after another in this process or on a
//! server, printed as they go, summed up at the end.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};
use signal_lab_engine::error::EngineError;
use signal_lab_engine::experiment::Experiment;
use signal_lab_engine::experiment_files;
use signal_lab_engine::experiment_run::{Progress, RunOptions};
use signal_lab_engine::host::NoEvents;
use signal_lab_engine::secrets::{FileStore, SecretStore, SystemStore};
use signal_lab_engine::{paths, Capture, Host, Mode, Service};

use crate::fail::{Exit, Failure};
use crate::i18n::{Texts, TEMPLATES};
use crate::remote::Remote;
use crate::{junit, params, Ctx, Experiments, Place, RunArgs, SecretSource, ValidateArgs};

/// Where secrets are read from in a container (the Docker secrets layout), as on a server.
const DEFAULT_SECRETS_DIR: &str = "/run/secrets/signallab";

/// One experiment as given: its label (the argument) and its document.
pub struct Input {
    pub label: String,
    pub document: Experiment,
}

/// One step of a run, as the engine and the server report it.
#[derive(Deserialize, Clone, Debug)]
pub struct Step {
    pub ts: u64,
    pub node_id: String,
    pub state: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub message_key: Option<String>,
    #[serde(default)]
    pub message_params: Value,
    #[serde(default)]
    pub error: Option<EngineError>,
}

/// The end of a run.
#[derive(Deserialize, Clone, Debug)]
pub struct Ended {
    pub experiment: String,
    pub outcome: String,
    pub seed: u64,
    #[serde(default)]
    pub profile: Option<String>,
    pub started_ms: u64,
    pub ended_ms: u64,
    #[serde(default)]
    pub error: Option<EngineError>,
    #[serde(default)]
    pub steps: Vec<Step>,
    /// What each Emulator node received: `{node, name, protocol, local, counts}`.
    #[serde(default)]
    pub emulators: Vec<Value>,
    #[serde(default)]
    pub report_path: Option<String>,
    #[serde(default)]
    pub report_error: Option<EngineError>,
}

/// What became of one experiment.
pub struct Record {
    pub label: String,
    pub document: Experiment,
    pub outcome: Result<Ended, Failure>,
    /// Where its report is now: copied by --report, or kept in --data-dir or on the server.
    pub report: Option<String>,
}

impl Record {
    pub fn exit(&self) -> Exit {
        match &self.outcome {
            Ok(ended) if ended.outcome == "passed" => Exit::Passed,
            Ok(_) => Exit::Failed,
            Err(failure) => failure.exit,
        }
    }
}

/// Where experiments run (and, for `signallab mcp`, where sends go).
pub(crate) enum Engine {
    Here { service: Service, temporary: Option<PathBuf> },
    Server(Remote),
}

impl Engine {
    /// The engine in this process: its data folder chosen first, secrets from files or the system store.
    pub(crate) fn here(place: &Place, data_dir: Option<&Path>) -> Result<Engine, Failure> {
        let (dir, temporary) = match data_dir {
            Some(dir) => (dir.to_path_buf(), None),
            None => {
                let dir = std::env::temp_dir().join(format!("signallab-{}-{:08x}", std::process::id(), rand::random::<u32>()));
                (dir.clone(), Some(dir))
            }
        };
        std::fs::create_dir_all(&dir).map_err(|error| Failure::environment(EngineError::new("file.io").with("path", dir.display()).because(error)))?;
        paths::set_data_dir(dir);
        let store: Arc<dyn SecretStore> = match place.secrets {
            SecretSource::System => Arc::new(SystemStore),
            SecretSource::Files => {
                let folder = place.secrets_dir.clone().or_else(|| Some(PathBuf::from(DEFAULT_SECRETS_DIR)).filter(|dir| dir.is_dir()));
                Arc::new(FileStore::new(folder))
            }
        };
        let host = Host::new(Arc::new(NoEvents), Capture::new());
        Ok(Engine::Here { service: Service::new(host, Mode::Server, store), temporary })
    }

    pub(crate) fn new(place: &Place, data_dir: Option<&Path>) -> Result<Engine, Failure> {
        match &place.server {
            Some(url) => Ok(Engine::Server(Remote::new(url, place.token_file.as_deref())?)),
            None => Engine::here(place, data_dir),
        }
    }

    /// The editor's check: problems that stop a run are the error, the other profiles' come back.
    pub(crate) async fn validate(&self, document: &Experiment, overrides: &BTreeMap<String, String>) -> Result<Vec<Value>, Failure> {
        let args = json!({ "document": document, "overrides": overrides });
        let issues = match self {
            Engine::Here { service, .. } => service.invoke("experiment_validate", args).await.map_err(|failure| match failure {
                signal_lab_engine::Failure::Engine(error) => Failure::starting(error),
            })?,
            Engine::Server(remote) => remote.invoke("experiment_validate", &args).await?,
        };
        Ok(issues.as_array().cloned().unwrap_or_default())
    }

    /// Run `document`; `line` gets every `started`, `step` and `ended` as it happens.
    /// One command of the engine's table, here or on the server; a refusal is the command's own failure.
    pub(crate) async fn invoke(&self, command: &str, args: Value) -> Result<Value, Failure> {
        match self {
            Engine::Here { service, .. } => service.invoke(command, args).await.map_err(|failure| match failure {
                signal_lab_engine::Failure::Engine(error) => Failure::sending(error),
            }),
            Engine::Server(remote) => remote.invoke(command, &args).await,
        }
    }

    pub(crate) async fn run(&self, document: Experiment, options: RunOptions, line: &mut (dyn FnMut(&str, &Value) + Send)) -> Result<Value, Failure> {
        match self {
            Engine::Here { service, .. } => {
                let mut handle = service.run(document, options).await.map_err(Failure::starting)?;
                line("started", &serde_json::to_value(&handle.started).unwrap_or_default());
                loop {
                    match handle.next().await {
                        Some(Progress::Step(step)) => line("step", &serde_json::to_value(&step).unwrap_or_default()),
                        Some(Progress::Ended(result)) => {
                            let value = serde_json::to_value(&result).unwrap_or_default();
                            line("ended", &value);
                            return Ok(value);
                        }
                        None => unreachable!("a followed run ends before it says nothing more"),
                    }
                }
            }
            Engine::Server(remote) => {
                let document = serde_json::to_value(&document).unwrap_or_default();
                let request = Remote::run_request(&document, &options.overrides, options.seed, options.limit.map(|limit| limit.as_secs()));
                remote.run(&request, line).await
            }
        }
    }

    /// Put the report of a run at `to`.
    async fn copy_report(&self, report: &str, to: &Path) -> Result<(), Failure> {
        if let Some(parent) = to.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", parent.display()).because(error)))?;
        }
        match self {
            Engine::Here { .. } => std::fs::copy(report, to)
                .map(|_| ())
                .map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", to.display()).because(error))),
            Engine::Server(remote) => remote.download(report, to).await,
        }
    }

    /// Reports stay where a person can find them: in --data-dir, on the server; a temporary folder goes.
    fn keeps_reports(&self) -> bool {
        !matches!(self, Engine::Here { temporary: Some(_), .. })
    }

    pub(crate) fn remote(&self) -> bool {
        matches!(self, Engine::Server(_))
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Engine::Here { temporary: Some(dir), .. } = self {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

/// A file, else a bundled template by name.
pub(crate) fn read_input(label: &str) -> Result<Input, Failure> {
    let path = Path::new(label);
    let text = if path.is_file() {
        std::fs::read_to_string(path).map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", path.display()).because(error)))?
    } else if let Some((_, text)) = TEMPLATES.iter().find(|(name, _)| *name == label.strip_suffix(".json").unwrap_or(label)) {
        text.to_string()
    } else if path.exists() {
        return Err(Failure::invalid(EngineError::new("file.io").with("path", path.display())));
    } else {
        return Err(Failure::invalid(EngineError::new("cli.file_unknown").with("name", label)));
    };
    // Read as the app imports a file: older versions are migrated, a broken one refused.
    let document = experiment_files::parse(&text).map_err(Failure::invalid)?;
    Ok(Input { label: label.to_string(), document })
}

/// `NAME=VALUE` pairs.
pub(crate) fn parse_params(given: &[String]) -> Result<Vec<(String, String)>, Failure> {
    given
        .iter()
        .map(|pair| match pair.split_once('=') {
            Some((name, value)) if !name.trim().is_empty() => Ok((name.trim().to_string(), value.to_string())),
            _ => Err(Failure::invalid(EngineError::new("cli.param_invalid").with("value", pair))),
        })
        .collect()
}

/// An experiment ready to run, with the parameter values it takes.
type Prepared = (Input, BTreeMap<String, String>);

/// Every input, its profile applied, and the values each one takes: a value
/// applies to the experiments that have that parameter, and must name a
/// parameter of at least one of them. A failure names the input it is about.
fn prepare(experiments: &Experiments) -> Result<Vec<Prepared>, (String, Failure)> {
    let params = parse_params(&experiments.params).map_err(|failure| (String::new(), failure))?;
    let mut prepared = Vec::new();
    for label in &experiments.files {
        let mut input = read_input(label).map_err(|failure| (label.clone(), failure))?;
        if let Some(profile) = &experiments.profile {
            input.document.profile = (!profile.is_empty()).then(|| profile.clone());
        }
        let known: BTreeSet<&str> = input.document.params.iter().map(|param| param.name.as_str()).collect();
        let overrides = params.iter().filter(|(name, _)| known.contains(name.as_str())).cloned().collect();
        prepared.push((input, overrides));
    }
    if let Some((name, _)) = params.iter().find(|(name, _)| !prepared.iter().any(|(input, _)| input.document.params.iter().any(|param| &param.name == name))) {
        let label = if prepared.len() == 1 { prepared[0].0.label.clone() } else { String::new() };
        return Err((label, Failure::invalid(EngineError::new("run.override_unknown").with("name", name))));
    }
    Ok(prepared)
}

/// `label: ` before a message about one experiment.
fn about(label: &str) -> String {
    if label.is_empty() {
        String::new()
    } else {
        format!("{label}: ")
    }
}

/// Prints a run as it goes: steps on stderr like the app's timeline, or JSON lines on stdout.
struct Printer<'a> {
    ctx: &'a Ctx,
    label: &'a str,
    document: &'a Experiment,
    first_ts: Option<u64>,
    width: usize,
}

impl<'a> Printer<'a> {
    fn new(ctx: &'a Ctx, label: &'a str, document: &'a Experiment) -> Self {
        let width = document.nodes.iter().map(|node| node_label(&ctx.texts, document, &node.id).chars().count()).max().unwrap_or(0);
        Printer { ctx, label, document, first_ts: None, width }
    }

    fn line(&mut self, kind: &str, value: &Value) {
        if self.ctx.json {
            let mut value = value.clone();
            if let Value::Object(map) = &mut value {
                map.insert("type".into(), kind.into());
                if kind != "step" {
                    map.insert("file".into(), self.label.into());
                }
            }
            println!("{value}");
            return;
        }
        let texts = &self.ctx.texts;
        match kind {
            "started" => {
                let mut values = params! {
                    "name" => value["experiment"].as_str().unwrap_or_default(),
                    "file" => self.label,
                    "seed" => value["seed"].as_u64().unwrap_or_default(),
                };
                let key = match value["profile"].as_str() {
                    Some(profile) => {
                        values.insert("profile".into(), profile.into());
                        "cli.startedProfile"
                    }
                    None => "cli.started",
                };
                eprintln!("▶ {}", texts.t(key, &values));
            }
            "step" => {
                let Ok(step) = serde_json::from_value::<Step>(value.clone()) else { return };
                let first = *self.first_ts.get_or_insert(step.ts);
                let at = step.ts.saturating_sub(first) as f64 / 1000.0;
                let label = node_label(texts, self.document, &step.node_id);
                let said = step_text(texts, &step);
                let state = texts.plain(&format!("exp.{}", step.state));
                let text = if said.is_empty() { state } else { format!("{state} · {said}") };
                eprintln!("  {at:>8.3}  {label:<width$}  {text}", width = self.width);
            }
            _ => {}
        }
    }
}

/// A node as the app names it: the name of its type.
pub fn node_label(texts: &Texts, document: &Experiment, id: &str) -> String {
    match document.nodes.iter().find(|node| node.id == id) {
        Some(node) => {
            let kind = serde_json::to_value(&node.kind).ok().and_then(|value| value["type"].as_str().map(str::to_string)).unwrap_or_default();
            texts.node_label(&kind)
        }
        None => id.to_string(),
    }
}

/// What a step did or why it failed, in words — the app's timeline row.
pub fn step_text(texts: &Texts, step: &Step) -> String {
    let said = match &step.message_key {
        Some(key) => texts.t(key, &texts.message_params(&step.message_params)),
        None => step.detail.clone(),
    };
    let described = step.error.as_ref().map(|error| texts.describe(error, &["cli.err."], &|_| None).text);
    match (step.state.as_str(), described) {
        ("retry", Some(error)) => format!("{said} — {error}"),
        (_, Some(error)) => error,
        _ => said,
    }
}

/// The run's failure in words, located by node.
pub(crate) fn failure_text(texts: &Texts, document: &Experiment, error: &EngineError, remote: bool) -> (String, Option<String>) {
    let wording: &[&str] = if remote { &["cli.serverErr.", "cli.err."] } else { &["cli.err."] };
    let described = texts.describe(error, wording, &|id| document.nodes.iter().any(|node| node.id == id).then(|| node_label(texts, document, id)));
    (described.text, described.detail)
}

/// Where the report of experiment `index` of `count` goes under --report.
fn report_target(report: &Path, index: usize, count: usize, label: &str) -> PathBuf {
    if count == 1 {
        return report.to_path_buf();
    }
    let stem: String = Path::new(label)
        .file_stem()
        .map(|stem| stem.to_string_lossy().chars().map(|char| if char.is_alphanumeric() || "-_.".contains(char) { char } else { '_' }).collect())
        .unwrap_or_else(|| "run".into());
    report.join(format!("{:02}-{stem}.json", index + 1))
}

pub async fn run(ctx: &Ctx, args: RunArgs) -> Exit {
    let prepared = match prepare(&args.experiments) {
        Ok(prepared) => prepared,
        Err((label, failure)) => return failure.report(ctx, &about(&label)),
    };
    let engine = match Engine::new(&args.place, args.data_dir.as_deref()) {
        Ok(engine) => engine,
        Err(failure) => return failure.report(ctx, ""),
    };
    // Every experiment would start before any runs: a broken third file stops the first one too.
    for (input, overrides) in &prepared {
        if let Err(failure) = engine.validate(&input.document, overrides).await {
            return failure.report(ctx, &about(&input.label));
        }
    }

    let count = prepared.len();
    let mut records = Vec::new();
    let mut worst = Exit::Passed;
    for (index, (input, overrides)) in prepared.into_iter().enumerate() {
        let options = RunOptions { overrides, seed: args.seed, limit: args.timeout.map(Duration::from_secs) };
        let outcome = {
            let mut printer = Printer::new(ctx, &input.label, &input.document);
            engine.run(input.document.clone(), options, &mut |kind, value| printer.line(kind, value)).await
        };
        let outcome = outcome.and_then(|value| {
            serde_json::from_value::<Ended>(value).map_err(|error| Failure::environment(EngineError::new("cli.server_reply").with("url", "").with("status", 200).because(error)))
        });
        let mut record = Record { label: input.label, document: input.document, outcome, report: None };
        if let Ok(ended) = &record.outcome {
            summarize(ctx, &record, ended, engine.remote());
            if let Some(report) = &ended.report_path {
                record.report = engine.keeps_reports().then(|| report.clone());
                if let Some(target) = &args.report {
                    let target = report_target(target, index, count, &record.label);
                    match engine.copy_report(report, &target).await {
                        Ok(()) => record.report = Some(target.display().to_string()),
                        Err(failure) => worst = worst.max(failure.report(ctx, "")),
                    }
                }
            }
            if let Some(path) = &record.report {
                ctx.say(&format!("  {}", ctx.texts.t("cli.report", &params! { "path" => path.as_str() })));
            }
        } else if let Err(failure) = &record.outcome {
            if ctx.json {
                println!("{}", json!({ "type": "error", "file": record.label, "error": failure.error, "exit_code": failure.exit.code() }));
            } else {
                for line in failure.lines(ctx, &format!("✖ {}", about(&record.label))) {
                    eprintln!("{line}");
                }
            }
        }
        worst = worst.max(record.exit());
        records.push(record);
    }

    if let Some(path) = &args.junit {
        let xml = junit::write(&ctx.texts, &records, engine.remote());
        let written = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(path, xml));
        match written {
            Ok(()) => ctx.say(&ctx.texts.t("cli.junit", &params! { "path" => path.display().to_string() })),
            Err(error) => worst = worst.max(Failure::invalid(EngineError::new("file.io").with("path", path.display()).because(error)).report(ctx, "")),
        }
    }
    let passed = records.iter().filter(|record| record.exit() == Exit::Passed).count();
    if ctx.json {
        println!("{}", json!({ "type": "summary", "total": records.len(), "passed": passed, "failed": records.len() - passed, "exit_code": worst.code() }));
    } else if records.len() > 1 {
        println!("{}", ctx.texts.t("cli.summary", &params! { "n" => records.len(), "passed" => passed, "failed" => records.len() - passed }));
    }
    worst
}

/// How long a run took, as a person reads it: milliseconds under a second, else seconds.
pub fn duration(texts: &Texts, ms: u64) -> String {
    if ms < 1000 {
        format!("{} {}", crate::i18n::format_number(ms as f64, texts.lang), texts.plain("unit.ms"))
    } else {
        let seconds = (ms as f64 / 10.0).round() / 100.0;
        format!("{} {}", crate::i18n::format_number(seconds, texts.lang), texts.plain("unit.s"))
    }
}

/// One line per experiment on stdout; why it failed and its seed on stderr.
fn summarize(ctx: &Ctx, record: &Record, ended: &Ended, remote: bool) {
    if ctx.json {
        return;
    }
    let texts = &ctx.texts;
    let values = params! { "name" => ended.experiment.as_str(), "time" => duration(texts, ended.ended_ms.saturating_sub(ended.started_ms)), "seed" => ended.seed };
    match ended.outcome.as_str() {
        "passed" => println!("✔ {}", texts.t("cli.passed", &values)),
        "stopped" => println!("■ {}", texts.t("cli.stopped", &values)),
        _ => {
            let (text, detail) = match &ended.error {
                Some(error) => failure_text(texts, &record.document, error, remote),
                None => (texts.plain("exp.failed"), None),
            };
            let mut values = values;
            values.insert("error".into(), text.into());
            println!("✖ {}", texts.t("cli.failed", &values));
            if let Some(detail) = detail {
                eprintln!("  {}: {detail}", texts.plain("err.details"));
            }
            eprintln!("  {}", texts.t("cli.rerun", &params! { "seed" => ended.seed }));
        }
    }
    // What the dependencies the run played were asked: the proof a mock was called.
    for emulator in &ended.emulators {
        eprintln!("  {}", crate::emulate::counts_line(texts, emulator["name"].as_str().unwrap_or_default(), &emulator["counts"]));
    }
    if let Some(error) = &ended.report_error {
        eprintln!("  {}", texts.describe(error, &["cli.err."], &|_| None).text);
    }
    // GitHub Actions shows this on the run's page, next to the failing step.
    if ended.outcome != "passed" && std::env::var("GITHUB_ACTIONS").is_ok_and(|value| value == "true") {
        let error = ended.error.as_ref().map(|error| failure_text(texts, &record.document, error, remote).0).unwrap_or_else(|| texts.plain(&format!("exp.{}", ended.outcome)));
        let escape = |text: &str| text.replace('%', "%25").replace('\r', "%0D").replace('\n', "%0A");
        println!("::error title={}::{}", escape(&format!("Signal Lab · {}", ended.experiment)).replace(',', "%2C").replace(':', "%3A"), escape(&format!("{}: {error}", record.label)));
    }
}

pub async fn validate(ctx: &Ctx, args: ValidateArgs) -> Exit {
    let prepared = match prepare(&args.experiments) {
        Ok(prepared) => prepared,
        Err((label, failure)) => return failure.report(ctx, &about(&label)),
    };
    let engine = match Engine::new(&args.place, None) {
        Ok(engine) => engine,
        Err(failure) => return failure.report(ctx, ""),
    };
    let texts = &ctx.texts;
    let mut worst = Exit::Passed;
    for (input, overrides) in &prepared {
        let result = engine.validate(&input.document, overrides).await;
        if ctx.json {
            let line = match &result {
                Ok(issues) => json!({ "file": input.label, "experiment": input.document.name, "valid": true, "profile_issues": issues }),
                Err(failure) => json!({ "file": input.label, "experiment": input.document.name, "valid": false, "error": failure.error, "exit_code": failure.exit.code() }),
            };
            println!("{line}");
        }
        match result {
            Ok(issues) => {
                if !ctx.json {
                    println!("✔ {}", texts.t("cli.valid", &params! { "file" => input.label.as_str(), "name" => input.document.name.as_str() }));
                }
                for issue in issues {
                    let Ok(error) = serde_json::from_value::<EngineError>(issue["error"].clone()) else { continue };
                    let profile = issue["profile"].as_str().map(str::to_string).unwrap_or_else(|| texts.plain("exp.noProfile"));
                    let (text, _) = failure_text(texts, &input.document, &error, engine.remote());
                    ctx.say(&format!("  {profile}: {}", texts.t("exp.profileIssue", &params! { "error" => text })));
                }
            }
            Err(failure) => {
                if !ctx.json {
                    let (text, detail) = failure_text(texts, &input.document, &failure.error, failure.remote);
                    println!("✖ {}{text}", about(&input.label));
                    if let Some(detail) = detail {
                        eprintln!("  {}: {detail}", texts.plain("err.details"));
                    }
                }
                worst = worst.max(failure.exit);
            }
        }
    }
    worst
}

pub fn templates(ctx: &Ctx) -> Exit {
    let listed: Vec<(&str, String)> = TEMPLATES
        .iter()
        .map(|(name, text)| (*name, experiment_files::parse(text).map(|document| document.name).unwrap_or_default()))
        .collect();
    if ctx.json {
        let list: Vec<Value> = listed.iter().map(|(name, experiment)| json!({ "name": name, "experiment": experiment })).collect();
        println!("{}", Value::Array(list));
    } else {
        let width = listed.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
        for (name, experiment) in listed {
            println!("{name:<width$}  {experiment}");
        }
    }
    Exit::Passed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_duration_reads_in_milliseconds_or_seconds() {
        let (en, ru) = (Texts::new(crate::i18n::Lang::En), Texts::new(crate::i18n::Lang::Ru));
        assert_eq!(duration(&en, 2), "2 ms");
        assert_eq!(duration(&en, 1250), "1.25 s");
        assert_eq!(duration(&ru, 1250), "1,25 с");
        assert_eq!(duration(&en, 61_005), "61.01 s");
    }

    #[test]
    fn values_are_name_equals_value() {
        assert_eq!(parse_params(&["a=1".into(), "url=http://x/?q=1".into(), "empty=".into()]).unwrap(), [("a".into(), "1".into()), ("url".into(), "http://x/?q=1".into()), ("empty".into(), String::new())]);
        for bad in ["novalue", "=x"] {
            assert_eq!(parse_params(&[bad.into()]).unwrap_err().error.code, "cli.param_invalid", "{bad}");
        }
    }

    #[test]
    fn a_value_applies_where_its_parameter_is_and_must_be_somewhere() {
        let experiments = |params: &[&str]| Experiments {
            files: vec!["osc-ping-reply".into(), "empty".into()],
            params: params.iter().map(|param| param.to_string()).collect(),
            profile: None,
        };
        let prepared = prepare(&experiments(&["device=10.0.0.5:9000"])).unwrap();
        assert_eq!(prepared[0].1["device"], "10.0.0.5:9000");
        assert!(prepared[1].1.is_empty(), "the empty template has no such parameter and takes nothing");
        let (_, failure) = prepare(&experiments(&["devise=x"])).err().unwrap();
        assert_eq!((failure.error.code.as_str(), failure.exit), ("run.override_unknown", Exit::Invalid));
        let (label, failure) = prepare(&Experiments { files: vec!["no-such-thing".into()], params: vec![], profile: None }).err().unwrap();
        assert_eq!((label.as_str(), failure.error.code.as_str()), ("no-such-thing", "cli.file_unknown"));
    }

    #[test]
    fn several_reports_go_into_a_folder_one_goes_where_it_is_told() {
        let folder = Path::new("out");
        assert_eq!(report_target(Path::new("report.json"), 0, 1, "a.json"), PathBuf::from("report.json"));
        assert_eq!(report_target(folder, 1, 3, "tests/smoke check.json"), folder.join("02-smoke_check.json"));
        assert_eq!(report_target(folder, 0, 2, "empty"), folder.join("01-empty.json"));
    }
}
