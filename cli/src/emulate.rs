//! `signallab emulate` and `signallab emulators`: emulators from files or from
//! the app's library, started in this process or on a server, what they answer
//! printed as it happens until Ctrl+C or `--for`. A pipeline starts one in the
//! background, runs the system under test against it, and reads the counts at
//! the end; the same commands as the app's (`emulator_start`,
//! `emulator_exchanges`, `job_stop`), so it answers as the app's would.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use signal_lab_engine::emulator::Emulator;
use signal_lab_engine::emulator_files::{self, EmulatorLibrary};
use signal_lab_engine::error::EngineError;

use crate::fail::{Exit, Failure};
use crate::run::{parse_params, Engine};
use crate::{params, Ctx, EmulateArgs, EmulatorsArgs};

/// How often a running emulator is asked what arrived.
const POLL: Duration = Duration::from_millis(200);

/// The library at `path`, else the app's (`emulators.json` in its data folder). Never created here.
pub(crate) fn library(path: Option<&Path>) -> Result<(PathBuf, EmulatorLibrary), Failure> {
    let path = path.map(Path::to_path_buf).unwrap_or_else(emulator_files::library_path);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(Failure::invalid(EngineError::new("file.not_found").with("path", path.display())));
        }
        Err(error) => return Err(Failure::invalid(EngineError::new("file.io").with("path", path.display()).because(error))),
    };
    let library = serde_json::from_str(&text).map_err(|error: serde_json::Error| {
        Failure::invalid(EngineError::new("emulators.json_invalid").with("path", path.display()).with("line", error.line()).with("column", error.column()).because(error))
    })?;
    Ok((path, library))
}

/// The emulators the arguments name: files (one emulator, a list or a library)
/// and, for anything that is not a file, the library's entry by id or name.
fn resolve(given: &[String], library_path: Option<&Path>) -> Result<Vec<(Emulator, Option<String>)>, Failure> {
    let mut found = Vec::new();
    let mut stored: Option<(PathBuf, EmulatorLibrary)> = None;
    for wanted in given {
        let path = Path::new(wanted);
        if path.is_file() {
            let text = std::fs::read_to_string(path).map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", path.display()).because(error)))?;
            let emulators = emulator_files::parse_file(&text, path).map_err(Failure::invalid)?;
            found.extend(emulators.into_iter().map(|emulator| (emulator, None)));
            continue;
        }
        if stored.is_none() {
            stored = Some(library(library_path)?);
        }
        let (path, library) = stored.as_ref().expect("read just above");
        let entry = emulator_files::find(library, wanted)
            .ok_or_else(|| Failure::invalid(EngineError::new("cli.emulator_unknown").with("name", wanted).with("path", path.display())))?;
        found.push((entry.emulator.clone(), Some(entry.id.clone())));
    }
    Ok(found)
}

/// One started emulator and how far its exchanges have been printed.
struct Started {
    job: u64,
    name: String,
    after: u64,
}

/// An emulator that would not start: a port taken or refused is the
/// surroundings (3), a document that cannot start is the input (2).
pub(crate) fn starting(failure: Failure) -> Failure {
    let exit = if crate::fail::environmental(&failure.error.code) { Exit::Environment } else { Exit::Invalid };
    Failure { exit, ..failure }
}

/// `+12.345 s` since the command started: no clock or time zone to agree on.
fn elapsed(since: Instant) -> String {
    format!("+{:>8.3} s", since.elapsed().as_secs_f64())
}

fn print_exchange(ctx: &Ctx, started: &Started, exchange: &Value, since: Instant) {
    if ctx.json {
        println!("{}", json!({ "type": "exchange", "job_id": started.job, "emulator": started.name, "exchange": exchange }));
        return;
    }
    let rule = exchange["rule"].as_u64().map(|rule| format!("#{rule}")).unwrap_or_else(|| "—".into());
    let mut line = format!("{} {}  {rule}  {}", elapsed(since), started.name, exchange["request"].as_str().unwrap_or_default());
    let reply = exchange["reply"].as_str().unwrap_or_default();
    match (exchange.get("error").filter(|error| !error.is_null()), exchange["fault"].as_str()) {
        (Some(error), _) => {
            let error: EngineError = serde_json::from_value(error.clone()).unwrap_or_else(|_| EngineError::new("emulator.failed"));
            line.push_str(&format!(" ✕ {}", ctx.texts.describe(&error, &["cli.err."], &|_| None).text));
        }
        (None, Some(fault)) => line.push_str(&format!(" → {fault}")),
        (None, None) if !reply.is_empty() => line.push_str(&format!(" → {reply}")),
        _ => {}
    }
    // What met an outage, in the notation of the Inspector's verdicts.
    if exchange["down"].as_bool() == Some(true) {
        line.push_str(" · down");
    }
    line.push_str(&format!("  {} ms  ← {}", exchange["ms"].as_u64().unwrap_or(0), exchange["from"].as_str().unwrap_or_default()));
    println!("{line}");
}

/// `Demo API: 7 requests, 1 without a rule, 0 failed · #1 3, #2 3` — for the end of
/// `emulate`, and for the emulators of a run.
pub(crate) fn counts_line(texts: &crate::i18n::Texts, name: &str, counts: &Value) -> String {
    let hits: Vec<String> = counts["hits"].as_array().into_iter().flatten().enumerate().map(|(index, hits)| format!("#{} {}", index + 1, hits)).collect();
    let mut line = texts.t("cli.emulatorSummary", &params! { "name" => name, "total" => &counts["total"], "unmatched" => &counts["unmatched"], "failed" => &counts["failed"] });
    if counts["down"].as_u64().unwrap_or_default() > 0 {
        line = format!("{line}, {}", texts.t("cli.emulatorDown", &params! { "n" => &counts["down"] }));
    }
    if counts["missed"].as_u64().unwrap_or_default() > 0 {
        line = format!("{line}, {}", texts.t("cli.emulatorMissed", &params! { "n" => &counts["missed"] }));
    }
    if hits.is_empty() { line } else { format!("{line} · {}", hits.join(", ")) }
}

fn summary(ctx: &Ctx, name: &str, snapshot: &Value) {
    if ctx.json {
        println!("{}", json!({ "type": "summary", "job_id": snapshot["job_id"], "emulator": name, "local": snapshot["local"], "counts": snapshot["counts"] }));
        return;
    }
    ctx.say(&counts_line(&ctx.texts, name, &snapshot["counts"]));
}

/// Stop what was started; a server keeps a job nobody stops.
async fn stop(engine: &Engine, started: &[Started]) {
    for one in started {
        let _ = engine.invoke("job_stop", json!({ "id": one.job })).await;
    }
}

pub async fn emulate(ctx: &Ctx, args: EmulateArgs) -> Exit {
    match emulate_inner(ctx, args).await {
        Ok(exit) => exit,
        Err(failure) => failure.report(ctx, ""),
    }
}

async fn emulate_inner(ctx: &Ctx, args: EmulateArgs) -> Result<Exit, Failure> {
    // Read before the engine moves this process's data folder anywhere.
    let mut emulators = resolve(&args.emulators, args.library.as_deref())?;
    if emulators.is_empty() {
        return Err(Failure::invalid(EngineError::new("cli.emulate_nothing")));
    }
    if let Some(bind) = &args.bind {
        if emulators.len() != 1 {
            return Err(Failure::invalid(EngineError::new("cli.bind_one")));
        }
        emulators[0].0.bind = bind.clone();
    }
    let values: BTreeMap<String, String> = parse_params(&args.params)?.into_iter().collect();
    let engine = Engine::new(&args.place, None)?;
    if args.check {
        for (emulator, _) in &emulators {
            engine.invoke("emulator_check", json!({ "emulator": emulator, "params": values })).await.map_err(starting)?;
            if ctx.json {
                println!("{}", json!({ "type": "valid", "emulator": emulator.name, "bind": emulator.bind }));
            } else {
                println!("{}", ctx.texts.t("cli.emulatorValid", &params! { "name" => emulator.name.as_str(), "bind" => emulator.bind.as_str() }));
            }
        }
        return Ok(Exit::Passed);
    }

    let mut started: Vec<Started> = Vec::new();
    for (emulator, source) in &emulators {
        let request = json!({ "emulator": emulator, "params": values, "seed": args.seed, "source": source });
        match engine.invoke("emulator_start", request).await {
            Ok(job) => {
                let local = job["params"]["local"].as_str().unwrap_or_default().to_string();
                let one = Started { job: job["id"].as_u64().unwrap_or(0), name: emulator.name.clone(), after: 0 };
                if ctx.json {
                    println!("{}", json!({ "type": "started", "job_id": one.job, "emulator": one.name, "protocol": emulator.kind.protocol(), "local": local }));
                } else {
                    ctx.say(&ctx.texts.t("cli.emulating", &params! { "name" => one.name.as_str(), "protocol" => emulator.kind.protocol(), "local" => local }));
                }
                started.push(one);
            }
            Err(failure) => {
                stop(&engine, &started).await;
                return Err(starting(failure));
            }
        }
    }
    match args.duration {
        Some(seconds) => ctx.say(&ctx.texts.t("cli.emulateFor", &params! { "s" => seconds })),
        None => ctx.say(&ctx.texts.plain("cli.emulateUntil")),
    }

    let since = Instant::now();
    let deadline = args.duration.map(|seconds| since + Duration::from_secs(seconds));
    let interrupted = tokio::signal::ctrl_c();
    tokio::pin!(interrupted);
    let mut exit = Exit::Passed;
    let mut last: BTreeMap<u64, Value> = BTreeMap::new();
    'follow: loop {
        tokio::select! {
            _ = &mut interrupted => break 'follow,
            _ = tokio::time::sleep(POLL) => {}
        }
        for one in started.iter_mut() {
            match engine.invoke("emulator_exchanges", json!({ "jobId": one.job, "after": one.after })).await {
                Ok(snapshot) => {
                    for exchange in snapshot["exchanges"].as_array().into_iter().flatten() {
                        one.after = one.after.max(exchange["seq"].as_u64().unwrap_or(0));
                        print_exchange(ctx, one, exchange, since);
                    }
                    last.insert(one.job, snapshot);
                }
                // It ended on its own: a socket that failed. The rest stop with it.
                Err(failure) => {
                    Failure { exit: Exit::Environment, ..failure }.report(ctx, &format!("{}: ", one.name));
                    exit = Exit::Environment;
                    break 'follow;
                }
            }
        }
        if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            break;
        }
    }
    stop(&engine, &started).await;
    for one in &started {
        if let Some(snapshot) = last.get(&one.job) {
            summary(ctx, &one.name, snapshot);
        }
    }
    Ok(exit)
}

/// The library's emulators: id, name, protocol, address and rules.
pub fn emulators(ctx: &Ctx, args: EmulatorsArgs) -> Exit {
    let (path, library) = match library(args.library.as_deref()) {
        Ok(found) => found,
        Err(failure) => return failure.report(ctx, ""),
    };
    if ctx.json {
        let list: Vec<Value> = library
            .emulators
            .iter()
            .map(|stored| json!({ "id": stored.id, "name": stored.emulator.name, "protocol": stored.emulator.kind.protocol(), "bind": stored.emulator.bind, "rules": stored.emulator.kind.rules(), "note": stored.note }))
            .collect();
        println!("{}", json!({ "path": path.display().to_string(), "emulators": list }));
        return Exit::Passed;
    }
    if library.emulators.is_empty() {
        ctx.say(&ctx.texts.t("cli.emulatorsEmpty", &params! { "path" => path.display().to_string() }));
        return Exit::Passed;
    }
    for stored in &library.emulators {
        let rules = ctx.texts.t("cli.rules", &params! { "n" => stored.emulator.kind.rules() });
        println!("{:<16} {:<24} {:<5} {:<22} {rules}", stored.id, stored.emulator.name, stored.emulator.kind.protocol(), stored.emulator.bind);
    }
    Exit::Passed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_name_files_or_library_entries() {
        let dir = std::env::temp_dir().join(format!("signallab-emulate-{:08x}", rand::random::<u32>()));
        std::fs::create_dir_all(&dir).unwrap();
        let library_path = dir.join("emulators.json");
        std::fs::write(&library_path, serde_json::to_string(&emulator_files::seed()).unwrap()).unwrap();
        let file = dir.join("mock.json");
        std::fs::write(&file, r#"{ "name": "Mine", "bind": "127.0.0.1:1", "protocol": "udp" }"#).unwrap();
        let given = [file.display().to_string(), "osc-device".to_string(), "demo api".to_string()];
        let found = resolve(&given, Some(&library_path)).unwrap();
        let names: Vec<(&str, Option<&str>)> = found.iter().map(|(emulator, source)| (emulator.name.as_str(), source.as_deref())).collect();
        assert_eq!(names, [("Mine", None), ("Demo OSC device", Some("osc-device")), ("Demo API", Some("demo-api"))]);
        let missing = resolve(&["nothing".to_string()], Some(&library_path)).err().unwrap();
        assert_eq!((missing.error.code.as_str(), missing.exit), ("cli.emulator_unknown", Exit::Invalid));
        let no_library = resolve(&["x".to_string()], Some(&dir.join("absent.json"))).err().unwrap();
        assert_eq!(no_library.error.code, "file.not_found");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
