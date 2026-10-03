//! JUnit XML, the report every CI system reads (GitHub Actions, GitLab,
//! Jenkins, Azure): a `<testsuite>` per experiment, a `<testcase>` per node
//! that ran — its time from its own steps, a `<failure>` with the message in
//! the reader's language and the technical detail — and a `<skipped>` case for
//! a node the run never reached (the other side of a branch). An experiment
//! that could not start is a suite with one `<error>` case; one --fail-fast
//! never started, a suite with one `<skipped>` case.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::fail::Exit;
use crate::i18n::Texts;
use crate::run::{node_label, step_text, verdict_lines, Ended, NotStarted, Record, Step};

/// Text for an XML attribute or element: escaped, and without the control
/// characters XML 1.0 cannot carry at all.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for char in text.chars() {
        match char {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' | '\n' | '\r' => out.push(char),
            char if (char as u32) < 0x20 || matches!(char as u32, 0xFFFE | 0xFFFF) => {}
            char => out.push(char),
        }
    }
    out
}

fn seconds(ms: u64) -> String {
    format!("{:.3}", ms as f64 / 1000.0)
}

/// `2026-10-02T13:49:06` (UTC) from milliseconds since 1970.
pub fn timestamp(ms: u64) -> String {
    let secs = ms / 1000;
    let (days, rest) = ((secs / 86_400) as i64, secs % 86_400);
    // Civil from days (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}", rest / 3600, rest / 60 % 60, rest % 60)
}

/// One node's part of a run: its steps in order.
struct Case<'a> {
    id: &'a str,
    steps: Vec<&'a Step>,
}

impl Case<'_> {
    fn last_state(&self) -> &str {
        self.steps.last().map(|step| step.state.as_str()).unwrap_or("")
    }

    fn time(&self) -> u64 {
        match (self.steps.first(), self.steps.last()) {
            (Some(first), Some(last)) => last.ts.saturating_sub(first.ts),
            _ => 0,
        }
    }
}

/// The run's nodes in the order they first ran.
fn cases(ended: &Ended) -> Vec<Case<'_>> {
    let mut order: Vec<Case> = Vec::new();
    let mut at: HashMap<&str, usize> = HashMap::new();
    for step in &ended.steps {
        let index = *at.entry(step.node_id.as_str()).or_insert_with(|| {
            order.push(Case { id: &step.node_id, steps: Vec::new() });
            order.len() - 1
        });
        order[index].steps.push(step);
    }
    order
}

/// The suite's `<properties>`: the file as given, and the combination's values.
fn properties_of(file: &str, matrix: &[(String, String)]) -> Vec<(String, String)> {
    let mut properties = vec![("file".to_string(), file.to_string())];
    properties.extend(matrix.iter().map(|(param, value)| (format!("param.{param}"), value.clone())));
    properties
}

fn write_properties(suites: &mut String, properties: Vec<(String, String)>) {
    suites.push_str("    <properties>\n");
    for (key, value) in properties {
        let _ = writeln!(suites, "      <property name=\"{}\" value=\"{}\" />", escape(&key), escape(&value));
    }
    suites.push_str("    </properties>\n");
}

/// The XML for every experiment of a `signallab run`, and those it did not start.
pub fn write(texts: &Texts, records: &[Record], not_started: &[NotStarted], remote: bool) -> String {
    let wording: &[&str] = if remote { &["cli.serverErr.", "cli.err."] } else { &["cli.err."] };
    let mut suites = String::new();
    let (mut total, mut failures, mut errors, mut time) = (0usize, 0usize, 0usize, 0u64);
    for record in records {
        // A combination of the matrix is a suite of its own: its values are in its name.
        let name = escape(&crate::run::run_name(
            match &record.outcome {
                Ok(ended) => &ended.experiment,
                Err(_) => &record.document.name,
            },
            &record.matrix,
        ));
        let mut properties = properties_of(&record.file, &record.matrix);
        let mut body = String::new();
        let (mut tests, mut failed, mut broken, mut skipped, mut suite_time, mut stamp) = (0usize, 0usize, 0usize, 0usize, 0u64, None);
        match &record.outcome {
            Ok(ended) => {
                properties.push(("seed".into(), ended.seed.to_string()));
                properties.push(("outcome".into(), ended.outcome.clone()));
                if let Some(profile) = &ended.profile {
                    properties.push(("profile".into(), profile.clone()));
                }
                if let Some(report) = &record.report {
                    properties.push(("report".into(), report.clone()));
                }
                suite_time = ended.ended_ms.saturating_sub(ended.started_ms);
                stamp = Some(timestamp(ended.started_ms));
                let label = |id: &str| node_label(texts, &record.document, id);
                let ran = cases(ended);
                // The failure the run ended with is told on its node; one on no node gets a case of its own.
                let run_error = ended.error.as_ref().map(|error| texts.describe(error, wording, &|id| Some(label(id))));
                let mut told = false;
                for case in &ran {
                    tests += 1;
                    let _ = write!(body, "    <testcase name=\"{}\" classname=\"{name}\" time=\"{}\"", escape(&format!("{} ({})", label(case.id), case.id)), seconds(case.time()));
                    let error_here = ended.error.as_ref().is_some_and(|error| error.node.as_deref() == Some(case.id));
                    let step_failed = case.last_state() == "failed";
                    let unfinished = matches!(case.last_state(), "running" | "retry" | "repeating" | "load") && ended.outcome != "passed";
                    if error_here || step_failed || unfinished {
                        failed += 1;
                        let described = if error_here {
                            told = true;
                            run_error.clone()
                        } else {
                            case.steps.iter().rev().find_map(|step| step.error.as_ref()).map(|error| texts.describe(error, wording, &|id| Some(label(id))))
                        };
                        let message = described.as_ref().map(|described| described.message.clone()).unwrap_or_else(|| texts.plain(&format!("exp.{}", ended.outcome)));
                        let kind = ended.error.as_ref().filter(|_| error_here).map(|error| error.code.clone()).unwrap_or_else(|| ended.outcome.clone());
                        let mut text = String::new();
                        for step in &case.steps {
                            let said = step_text(texts, step);
                            let state = texts.plain(&format!("exp.{}", step.state));
                            let _ = writeln!(text, "{state}{}", if said.is_empty() { String::new() } else { format!(" · {said}") });
                            for verdict in step.load.as_ref().map(|load| verdict_lines(texts, load)).unwrap_or_default() {
                                let _ = writeln!(text, "  {verdict}");
                            }
                        }
                        if let Some(detail) = described.as_ref().and_then(|described| described.detail.clone()) {
                            let _ = writeln!(text, "{}: {detail}", texts.plain("err.details"));
                        }
                        let _ = write!(body, ">\n      <failure message=\"{}\" type=\"{}\">{}</failure>\n    </testcase>\n", escape(&message), escape(&kind), escape(text.trim_end()));
                    } else {
                        let lines: Vec<String> = case
                            .steps
                            .iter()
                            .flat_map(|step| std::iter::once(step_text(texts, step)).chain(step.load.as_ref().map(|load| verdict_lines(texts, load)).unwrap_or_default().into_iter().map(|verdict| format!("  {verdict}"))))
                            .filter(|text| !text.is_empty())
                            .collect();
                        if lines.is_empty() {
                            body.push_str(" />\n");
                        } else {
                            let _ = write!(body, ">\n      <system-out>{}</system-out>\n    </testcase>\n", escape(&lines.join("\n")));
                        }
                    }
                }
                if let (Some(described), false) = (&run_error, told) {
                    tests += 1;
                    failed += 1;
                    let code = ended.error.as_ref().map(|error| error.code.as_str()).unwrap_or_default();
                    let detail = described.detail.as_deref().map(|detail| format!("{}: {detail}", texts.plain("err.details"))).unwrap_or_default();
                    let _ = write!(
                        body,
                        "    <testcase name=\"{}\" classname=\"{name}\" time=\"{}\">\n      <failure message=\"{}\" type=\"{}\">{}</failure>\n    </testcase>\n",
                        escape(&texts.plain("cli.junitRun")),
                        seconds(suite_time),
                        escape(&described.text),
                        escape(code),
                        escape(&detail)
                    );
                }
                // A node the run never reached: the other side of a branch, or what a failure cut off.
                let reached: Vec<&str> = ran.iter().map(|case| case.id).collect();
                for node in record.document.nodes.iter().filter(|node| !reached.contains(&node.id.as_str())) {
                    tests += 1;
                    skipped += 1;
                    let _ = writeln!(
                        body,
                        "    <testcase name=\"{}\" classname=\"{name}\" time=\"0.000\">\n      <skipped message=\"{}\" />\n    </testcase>",
                        escape(&format!("{} ({})", label(&node.id), node.id)),
                        escape(&texts.plain("cli.junitNotReached"))
                    );
                }
            }
            Err(failure) => {
                tests = 1;
                broken = 1;
                let described = texts.describe(&failure.error, failure.wording(), &|id| Some(node_label(texts, &record.document, id)));
                let detail = described.detail.as_deref().map(|detail| format!("{}: {detail}", texts.plain("err.details"))).unwrap_or_default();
                let _ = write!(
                    body,
                    "    <testcase name=\"{}\" classname=\"{name}\" time=\"0.000\">\n      <error message=\"{}\" type=\"{}\">{}</error>\n    </testcase>\n",
                    escape(&texts.plain("cli.junitStart")),
                    escape(&described.text),
                    escape(&failure.error.code),
                    escape(&detail)
                );
            }
        }
        total += tests;
        failures += failed;
        errors += broken;
        time += suite_time;
        let stamp = stamp.map(|stamp| format!(" timestamp=\"{stamp}\"")).unwrap_or_default();
        let _ = writeln!(
            suites,
            "  <testsuite name=\"{name}\" tests=\"{tests}\" failures=\"{failed}\" errors=\"{broken}\" skipped=\"{skipped}\" time=\"{}\"{stamp}>",
            seconds(suite_time)
        );
        write_properties(&mut suites, properties);
        suites.push_str(&body);
        suites.push_str("  </testsuite>\n");
        debug_assert!(record.exit() != Exit::Passed || failed + broken == 0, "a passed run has no failures");
    }
    // Not started (--fail-fast): there, and said why, so a report reads as the whole matrix.
    for skipped in not_started {
        let name = escape(&crate::run::run_name(&skipped.document.name, &skipped.matrix));
        total += 1;
        let _ = writeln!(suites, "  <testsuite name=\"{name}\" tests=\"1\" failures=\"0\" errors=\"0\" skipped=\"1\" time=\"0.000\">");
        write_properties(&mut suites, properties_of(&skipped.file, &skipped.matrix));
        let _ = writeln!(
            suites,
            "    <testcase name=\"{}\" classname=\"{name}\" time=\"0.000\">\n      <skipped message=\"{}\" />\n    </testcase>\n  </testsuite>",
            escape(&texts.plain("cli.junitStart")),
            escape(&texts.plain("cli.junitNotStarted"))
        );
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites name=\"Signal Lab\" tests=\"{total}\" failures=\"{failures}\" errors=\"{errors}\" time=\"{}\">\n{suites}</testsuites>\n",
        seconds(time)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fail::Failure;
    use crate::i18n::Lang;
    use signal_lab_engine::error::EngineError;
    use signal_lab_engine::experiment_files;

    fn step(ts: u64, node: &str, state: &str, error: Option<EngineError>) -> Step {
        Step { ts, node_id: node.into(), state: state.into(), detail: String::new(), message_key: None, message_params: serde_json::Value::Null, error, load: None }
    }

    fn document() -> signal_lab_engine::experiment::Experiment {
        let text = crate::i18n::TEMPLATES.iter().find(|(name, _)| *name == "status-branch").unwrap().1;
        experiment_files::parse(text).unwrap()
    }

    #[test]
    fn xml_is_escaped_and_carries_no_control_characters() {
        assert_eq!(escape("a<b & \"c\" 'd'>\u{1}\u{7}\ttab\nline"), "a&lt;b &amp; &quot;c&quot; &apos;d&apos;&gt;\ttab\nline");
        assert_eq!(timestamp(0), "1970-01-01T00:00:00");
        assert_eq!(timestamp(1_790_000_000_000), "2026-09-21T14:13:20");
        assert_eq!(timestamp(951_782_400_000), "2000-02-29T00:00:00", "a leap day");
    }

    #[test]
    fn a_suite_per_experiment_a_case_per_node_failures_where_they_happened() {
        let texts = Texts::new(Lang::En);
        let doc = document();
        let ids: Vec<String> = doc.nodes.iter().map(|node| node.id.clone()).collect();
        let (start, failing) = (&ids[0], &ids[1]);
        let error = EngineError::new("transport.refused").with("target", "http://127.0.0.1:9/").at(failing).because("os error 10061 <&>");
        let ended = Ended {
            experiment: "Status <branch>".into(),
            outcome: "failed".into(),
            seed: 42,
            profile: None,
            started_ms: 1_790_000_000_000,
            ended_ms: 1_790_000_000_250,
            error: Some(error.clone()),
            steps: vec![step(1_790_000_000_000, start, "running", None), step(1_790_000_000_001, start, "passed", None), step(1_790_000_000_010, failing, "running", None), step(1_790_000_000_200, failing, "failed", Some(error))],
            emulators: Vec::new(),
            impairments: Vec::new(),
            report_path: None,
            report_error: None,
        };
        let records = vec![
            Record { label: "a.json".into(), file: "a.json".into(), matrix: Vec::new(), document: doc.clone(), outcome: Ok(ended), report: Some("runs/r.json".into()) },
            Record { label: "b.json".into(), file: "b.json".into(), matrix: Vec::new(), document: doc.clone(), outcome: Err(Failure::invalid(EngineError::new("run.override_unknown").with("name", "x"))), report: None },
        ];
        let xml = write(&texts, &records, &[], false);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites name=\"Signal Lab\""));
        let skipped = doc.nodes.len() - 2;
        assert!(xml.contains(&format!("<testsuite name=\"Status &lt;branch&gt;\" tests=\"{}\" failures=\"1\" errors=\"0\" skipped=\"{skipped}\" time=\"0.250\" timestamp=\"2026-09-21T14:13:20\">", doc.nodes.len())), "{xml}");
        assert!(xml.contains("<property name=\"seed\" value=\"42\" />") && xml.contains("<property name=\"report\" value=\"runs/r.json\" />"));
        let refused = texts.t("err.transport.refused", &crate::params! { "target" => "http://127.0.0.1:9/" });
        assert!(xml.contains(&format!("<failure message=\"{}\" type=\"transport.refused\">", escape(&refused))), "{xml}");
        assert!(xml.contains("os error 10061 &lt;&amp;&gt;"), "the detail, escaped");
        assert!(xml.contains("time=\"0.190\""), "a node's time is its own steps'");
        assert_eq!(xml.matches("<skipped message=").count(), skipped);
        assert!(xml.contains("<error message=") && xml.contains("type=\"run.override_unknown\""), "an experiment that could not start is an error");
        let total = doc.nodes.len() + 1;
        assert!(xml.contains(&format!("<testsuites name=\"Signal Lab\" tests=\"{total}\" failures=\"1\" errors=\"1\"")), "{xml}");

        // --fail-fast stopped before the next combination: it is in the report, skipped.
        let skipped = [NotStarted { file: "c.json".into(), matrix: vec![("host".into(), "b".into())], document: doc.clone() }];
        let xml = write(&texts, &records[..1], &skipped, false);
        assert!(xml.contains(&format!("<testsuite name=\"{} [host=b]\" tests=\"1\" failures=\"0\" errors=\"0\" skipped=\"1\"", escape(&doc.name))), "{xml}");
        assert!(xml.contains("<property name=\"file\" value=\"c.json\" />") && xml.contains("<property name=\"param.host\" value=\"b\" />"), "{xml}");
        assert!(xml.contains(&format!("<skipped message=\"{}\" />", escape(&texts.plain("cli.junitNotStarted")))), "{xml}");
        assert!(xml.contains(&format!("<testsuites name=\"Signal Lab\" tests=\"{}\"", doc.nodes.len() + 1)), "{xml}");
    }
}
