//! Runs read back and compared: the reports in `<data dir>/runs`, newest
//! first, with each load step's headline numbers (`experiment_runs`), and two
//! of them side by side, load step by load step (`experiment_compare`). Read
//! from the files, so the command line's runs count as the app's do. A run is
//! named by its report's file name, never by a path: nothing outside the runs
//! folder is read.

use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

use super::error::{EngineError, EngineResult};
use super::load::{LoadMetrics, Metric, Verdict};
use super::paths::data_dir;

/// The most runs one listing returns.
pub const MAX_RUNS: usize = 500;
/// A change this large (%) in the worse direction is a regression.
pub const REGRESSION_PERCENT: f64 = 5.0;
/// The metrics compared, in the order a table shows them.
pub const COMPARED: [Metric; 9] = [Metric::P50Ms, Metric::P90Ms, Metric::P95Ms, Metric::P99Ms, Metric::MeanMs, Metric::MaxMs, Metric::ErrorRate, Metric::Rps, Metric::Missed];

/// A run, as a list of runs shows it.
#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    /// The report's file name: what `experiment_compare` takes.
    pub name: String,
    pub experiment: String,
    pub started_ms: u64,
    pub ended_ms: u64,
    pub outcome: String,
    pub seed: u64,
    pub profile: Option<String>,
    /// Each load step's headline numbers, in the order they ran.
    pub loads: Vec<LoadHeadline>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoadHeadline {
    pub node: String,
    pub sent: u64,
    pub rps: f64,
    pub p95_ms: f64,
    pub error_rate: f64,
    /// Every threshold held (true without thresholds).
    pub held: bool,
}

/// Two runs side by side: `a` before, `b` after.
#[derive(Debug, Serialize)]
pub struct Comparison {
    pub a: RunSummary,
    pub b: RunSummary,
    /// Every load step of either run, matched by node id: `a`'s order, then what only `b` has.
    pub steps: Vec<StepComparison>,
}

#[derive(Debug, Serialize)]
pub struct StepComparison {
    pub node: String,
    /// The run it is missing from, if one ran it and the other did not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_in: Option<&'static str>,
    pub metrics: Vec<MetricChange>,
    pub sent: [u64; 2],
    pub thresholds_a: Vec<Verdict>,
    pub thresholds_b: Vec<Verdict>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct MetricChange {
    pub metric: Metric,
    pub a: f64,
    pub b: f64,
    /// `b - a`.
    pub change: f64,
    /// The change in % of `a`; none when `a` is 0.
    pub percent: Option<f64>,
    /// Moved the wrong way — slower, more errors, more missed, a lower rate —
    /// by `REGRESSION_PERCENT` or more (from nothing to something, for a count).
    pub worse: bool,
}

/// Higher is worse for every metric but the rate achieved.
fn higher_is_worse(metric: Metric) -> bool {
    !matches!(metric, Metric::Rps)
}

pub fn change(metric: Metric, a: f64, b: f64) -> MetricChange {
    let change = b - a;
    let percent = (a != 0.0).then(|| change / a * 100.0);
    let bad = if higher_is_worse(metric) { change > 0.0 } else { change < 0.0 };
    let worse = bad && percent.map_or(change.abs() > 0.0, |percent| percent.abs() >= REGRESSION_PERCENT);
    MetricChange { metric, a, b, change, percent, worse }
}

/// Each load step's last measurements in a report, in the order they ran.
fn loads_of(report: &Value) -> Vec<(String, LoadMetrics)> {
    let mut found: Vec<(String, LoadMetrics)> = Vec::new();
    for step in report["steps"].as_array().into_iter().flatten() {
        let (Some(node), Some(load)) = (step["node_id"].as_str(), step.get("load").filter(|load| !load.is_null())) else { continue };
        let Ok(metrics) = serde_json::from_value::<LoadMetrics>(load.clone()) else { continue };
        match found.iter_mut().find(|(id, _)| id == node) {
            Some(slot) => slot.1 = metrics,
            None => found.push((node.to_string(), metrics)),
        }
    }
    found
}

fn summary(name: &str, report: &Value) -> RunSummary {
    RunSummary {
        name: name.to_string(),
        experiment: report["experiment"].as_str().unwrap_or_default().to_string(),
        started_ms: report["started_ms"].as_u64().unwrap_or(0),
        ended_ms: report["ended_ms"].as_u64().unwrap_or(0),
        outcome: report["outcome"].as_str().unwrap_or_default().to_string(),
        seed: report["seed"].as_u64().unwrap_or(0),
        profile: report["profile"].as_str().map(str::to_string),
        loads: loads_of(report)
            .into_iter()
            .map(|(node, metrics)| LoadHeadline {
                node,
                sent: metrics.sent,
                rps: metrics.rps,
                p95_ms: metrics.p95_ms,
                error_rate: metrics.error_rate,
                held: metrics.thresholds.iter().all(|verdict| verdict.held),
            })
            .collect(),
    }
}

/// Two reports side by side, load step by load step.
pub fn compare(a: (&str, &Value), b: (&str, &Value)) -> Comparison {
    let (before, after) = (loads_of(a.1), loads_of(b.1));
    let empty = LoadMetrics::default();
    let mut steps: Vec<StepComparison> = before
        .iter()
        .map(|(node, metrics)| (node.clone(), Some(metrics), after.iter().find(|(id, _)| id == node).map(|(_, metrics)| metrics)))
        .chain(after.iter().filter(|(node, _)| !before.iter().any(|(id, _)| id == node)).map(|(node, metrics)| (node.clone(), None, Some(metrics))))
        .map(|(node, x, y)| StepComparison {
            missing_in: match (x, y) {
                (None, _) => Some("a"),
                (_, None) => Some("b"),
                _ => None,
            },
            metrics: COMPARED.iter().map(|metric| change(*metric, metric.of(x.unwrap_or(&empty)), metric.of(y.unwrap_or(&empty)))).collect(),
            sent: [x.unwrap_or(&empty).sent, y.unwrap_or(&empty).sent],
            thresholds_a: x.map(|metrics| metrics.thresholds.clone()).unwrap_or_default(),
            thresholds_b: y.map(|metrics| metrics.thresholds.clone()).unwrap_or_default(),
            node,
        })
        .collect();
    // A step one run lacks shows no changes: there is nothing to compare it with.
    for step in steps.iter_mut().filter(|step| step.missing_in.is_some()) {
        for metric in &mut step.metrics {
            metric.worse = false;
        }
    }
    Comparison { a: summary(a.0, a.1), b: summary(b.0, b.1), steps }
}

fn runs_dir() -> PathBuf {
    data_dir().join("runs")
}

/// A report's file name as a run's name: `run-<ms>-<job>….json`, no folder.
fn check_name(name: &str) -> EngineResult<()> {
    let plain = !name.contains(['/', '\\', ':']) && !name.contains("..");
    if plain && name.starts_with("run-") && name.ends_with(".json") {
        Ok(())
    } else {
        Err(EngineError::new("runs.name_invalid").with("name", name))
    }
}

fn read(name: &str) -> EngineResult<Value> {
    check_name(name)?;
    let path = runs_dir().join(name);
    let text = std::fs::read_to_string(&path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => EngineError::new("runs.not_found").with("name", name),
        _ => EngineError::new("file.io").with("path", path.display()).because(error),
    })?;
    serde_json::from_str(&text).map_err(|error| EngineError::new("file.json_invalid").with("line", error.line()).with("column", error.column()).because(error))
}

/// The runs in the runs folder, newest first — of the experiment `name` when
/// given — at most `limit`. A report that cannot be read is left out.
pub fn runs(name: Option<&str>, limit: usize) -> EngineResult<Vec<RunSummary>> {
    let dir = runs_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(EngineError::new("file.io").with("path", dir.display()).because(error)),
    };
    // `run-<started ms>-<job>`: the newest by name first, so a long history is not read whole.
    let mut files: Vec<(u64, String)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|file| check_name(file).is_ok())
        .map(|file| (file.trim_start_matches("run-").split(['-', '.']).next().and_then(|ms| ms.parse().ok()).unwrap_or(0), file))
        .collect();
    files.sort_by(|x, y| y.cmp(x));
    let mut found = Vec::new();
    for (_, file) in files {
        if found.len() >= limit.min(MAX_RUNS) {
            break;
        }
        let Ok(report) = read(&file) else { continue };
        let run = summary(&file, &report);
        if name.is_none_or(|name| run.experiment == name) {
            found.push(run);
        }
    }
    Ok(found)
}

/// Two runs by name, `a` before and `b` after.
pub fn compare_runs(a: &str, b: &str) -> EngineResult<Comparison> {
    let (x, y) = (read(a)?, read(b)?);
    Ok(compare((a, &x), (b, &y)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report(experiment: &str, started: u64, loads: &[(&str, Value)]) -> Value {
        let steps: Vec<Value> = loads
            .iter()
            .flat_map(|(node, load)| [json!({ "node_id": node, "state": "running" }), json!({ "node_id": node, "state": "passed", "load": load })])
            .collect();
        json!({ "version": 5, "experiment": experiment, "seed": 3, "profile": null, "started_ms": started, "ended_ms": started + 10, "outcome": "passed", "steps": steps })
    }

    fn load(p95: f64, errors: f64, rps: f64, held: bool) -> Value {
        json!({ "planned": 100, "sent": 100, "ok": 100, "failed": 0, "missed": 0, "duration_ms": 1000, "rps": rps, "error_rate": errors,
                "min_ms": 1, "mean_ms": p95 / 2.0, "max_ms": p95 * 2.0, "p50_ms": p95 / 3.0, "p90_ms": p95 * 0.9, "p95_ms": p95, "p99_ms": p95 * 1.1,
                "received_bytes": 0, "statuses": {}, "seconds": [], "histogram": [],
                "thresholds": [{ "metric": "p95_ms", "op": "lt", "value": 300, "actual": p95, "held": held }] })
    }

    #[test]
    fn a_regression_is_a_change_the_wrong_way_by_five_percent_or_more() {
        assert!(change(Metric::P95Ms, 200.0, 260.0).worse, "slower");
        assert!(!change(Metric::P95Ms, 200.0, 205.0).worse, "2.5 % is noise");
        assert!(!change(Metric::P95Ms, 200.0, 150.0).worse, "faster is better");
        assert!(change(Metric::Rps, 100.0, 80.0).worse, "a lower rate is worse");
        assert!(!change(Metric::Rps, 100.0, 120.0).worse);
        let from_nothing = change(Metric::ErrorRate, 0.0, 0.4);
        assert!(from_nothing.worse && from_nothing.percent.is_none(), "errors where there were none");
        assert_eq!(change(Metric::P95Ms, 200.0, 260.0).percent, Some(30.0));
    }

    #[test]
    fn two_runs_compare_load_step_by_load_step() {
        let a = report("Load", 1, &[("api", load(200.0, 0.0, 100.0, true)), ("old", load(50.0, 0.0, 10.0, true))]);
        let b = report("Load", 2, &[("api", load(320.0, 1.5, 90.0, false)), ("new", load(40.0, 0.0, 10.0, true))]);
        let comparison = compare(("run-1-1.json", &a), ("run-2-2.json", &b));
        assert_eq!(comparison.steps.iter().map(|step| (step.node.as_str(), step.missing_in)).collect::<Vec<_>>(), [("api", None), ("old", Some("b")), ("new", Some("a"))]);
        let api = &comparison.steps[0];
        let p95 = api.metrics.iter().find(|row| row.metric == Metric::P95Ms).unwrap();
        assert_eq!((p95.a, p95.b, p95.change, p95.worse), (200.0, 320.0, 120.0, true));
        assert!(api.metrics.iter().find(|row| row.metric == Metric::Rps).unwrap().worse);
        assert_eq!((api.thresholds_a[0].held, api.thresholds_b[0].held), (true, false));
        assert!(comparison.steps[1].metrics.iter().all(|row| !row.worse), "a step one run lacks has nothing to regress against");
        assert_eq!((comparison.a.loads.len(), comparison.b.loads[0].held), (2, false));
    }

    #[test]
    fn a_run_is_named_by_its_file_never_by_a_path() {
        for name in ["run-1-1.json", "run-1791036643947-12-2.json"] {
            assert!(check_name(name).is_ok(), "{name}");
        }
        for name in ["../secrets/x.json", "run-1-1.json/../../a", "C:\\runs\\run-1.json", "experiment.json", "run-1-1.txt", "/etc/passwd"] {
            assert_eq!(check_name(name).unwrap_err().code, "runs.name_invalid", "{name}");
        }
    }
}
