//! The data folder's documents (the signal library, the emulators, the
//! experiment) written from several places at once — two pages of a server,
//! a page and a person's editor — and read meanwhile.

use std::path::PathBuf;
use std::sync::OnceLock;

use signal_lab_engine::{emulator_files, experiment, experiment_files, paths, signals};

fn data_dir() -> PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("signallab-data-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::set_data_dir(dir.clone());
        paths::data_dir()
    })
    .clone()
}

/// The emulator library, like the signal library, is someone's document: a
/// file that does not read is being fixed, and a save must not land over it.
#[test]
fn an_emulator_file_being_fixed_is_never_written_over() {
    let _one = one_at_a_time();
    data_dir();
    let path = emulator_files::library_path();
    let broken = b"{ \"version\": 1, \"emulators\": [ { \"id\": \"mine\", "; // a person mid-edit
    std::fs::write(&path, broken).unwrap();
    let error = emulator_files::save(&emulator_files::seed()).expect_err("saved over a file being fixed");
    assert!(error.is("emulators.json_invalid"), "{error:?}");
    assert_eq!(std::fs::read(&path).unwrap(), broken, "the file is as the person left it");
    std::fs::remove_file(&path).unwrap();
}

/// Two pages (or a page and an editor) hold the same library; the one that
/// read it first saves last. Its save names what it read (`base`) and is
/// refused, so the other's change is not silently thrown away.
#[test]
fn a_save_over_a_library_changed_since_it_was_read_is_refused() {
    let _one = one_at_a_time();
    data_dir();
    // Signals.
    let first = signals::load().unwrap();
    let mut theirs = first.library.clone();
    theirs.signals.truncate(1);
    let saved = signals::save_based(&theirs, Some(&first.stamp)).expect("the first page saves over what it read");
    assert_ne!(saved.stamp, first.stamp);
    let error = signals::save_based(&first.library, Some(&first.stamp)).expect_err("the second page's stale save");
    assert!(error.is("file.changed"), "{error:?}");
    assert_eq!(signals::load().unwrap().library.signals.len(), 1, "the first page's change is kept");
    // After reading again, it saves.
    let again = signals::load().unwrap();
    assert_eq!(again.stamp, saved.stamp, "the stamp names the file's contents");
    signals::save_based(&first.library, Some(&again.stamp)).unwrap();
    // A save that names no base (an older client) still replaces the file.
    signals::save(&theirs).unwrap();

    // Emulators, the same.
    let first = emulator_files::load().unwrap();
    let edited = std::fs::read_to_string(emulator_files::library_path()).unwrap().replace("Demo API", "Demo API, edited by hand");
    std::fs::write(emulator_files::library_path(), edited).unwrap();
    let error = emulator_files::save_based(&first.library, Some(&first.stamp)).expect_err("a stale save over a hand edit");
    assert!(error.is("file.changed"), "{error:?}");
    assert!(std::fs::read_to_string(emulator_files::library_path()).unwrap().contains("edited by hand"));
    let again = emulator_files::load().unwrap();
    emulator_files::save_based(&again.library, Some(&again.stamp)).unwrap();
}

/// Exports in quick succession (two pages, a script) are files of their own:
/// one never truncates and writes over another made in the same millisecond.
#[test]
fn exports_in_the_same_moment_are_files_of_their_own() {
    let _one = one_at_a_time();
    data_dir();
    tokio::runtime::Runtime::new().unwrap().block_on(exports());
}

async fn exports() {
    use signal_lab_engine::{Capture, Host, Mode, Service};
    let host = Host::new(signal_lab_engine::host::Recorder::new(), Capture::new());
    let service = Service::new(host, Mode::Server, std::sync::Arc::new(signal_lab_engine::secrets::FileStore::new(None)));
    service.invoke("inspect_set_enabled", serde_json::json!({ "enabled": true })).await.unwrap();
    service.invoke("osc_send", serde_json::json!({ "target": "127.0.0.1:9", "address": "/x", "args": [] })).await.unwrap();
    let mut paths = std::collections::BTreeSet::new();
    for _ in 0..20 {
        let path = service.invoke("inspect_export", serde_json::json!({ "format": "jsonl" })).await.unwrap();
        let path = path.as_str().unwrap().to_string();
        assert!(std::fs::read_to_string(&path).unwrap().lines().count() == 1, "{path} holds its one frame");
        paths.insert(path);
    }
    assert_eq!(paths.len(), 20, "exports written over each other");
}

/// The tests here share one data folder; they take turns.
fn one_at_a_time() -> std::sync::MutexGuard<'static, ()> {
    static TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());
    TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Saves from several threads at once, and loads between them: every save
/// either lands whole or fails with a reason; a load never sees half a file.
#[test]
fn saves_at_once_never_fail_or_tear_the_file() {
    let _one = one_at_a_time();
    data_dir();
    let library = signals::load().unwrap().library;
    let emulators = emulator_files::load().unwrap().library;
    let document = experiment::starter();
    let failures = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..40 {
                    for (what, result) in [
                        ("signals save", signals::save(&library).map(drop)),
                        ("signals load", signals::load().map(drop)),
                        ("emulators save", emulator_files::save(&emulators).map(drop)),
                        ("emulators load", emulator_files::load().map(drop)),
                        ("experiment save", experiment_files::save(&document).map(drop)),
                        ("experiment load", experiment_files::load().map(drop)),
                    ] {
                        if let Err(error) = result {
                            failures.lock().unwrap().push(format!("{what}: {error:?}"));
                        }
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(failures.is_empty(), "{} of 1920 failed, e.g. {:?}", failures.len(), &failures[..failures.len().min(3)]);
    let leftovers: Vec<_> = std::fs::read_dir(data_dir()).unwrap().filter_map(Result::ok).map(|entry| entry.file_name()).filter(|name| name.to_string_lossy().ends_with(".tmp")).collect();
    assert!(leftovers.is_empty(), "temporary files left: {leftovers:?}");
}
