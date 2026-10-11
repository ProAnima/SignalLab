//! Where the engine keeps its files: the experiment, the signal library, run
//! reports, exports. On a desktop that is `Documents/SignalLab`; a server
//! chooses its own folder (`SIGNALLAB_DATA_DIR`, or `set_data_dir` at startup).

use std::path::PathBuf;
use std::sync::OnceLock;

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Use `dir` for every file from now on. Set once, at startup, before anything
/// is read; a second call is refused (returns false) rather than moving files
/// out from under running work.
pub fn set_data_dir(dir: PathBuf) -> bool {
    DATA_DIR.set(dir).is_ok()
}

pub fn data_dir() -> PathBuf {
    DATA_DIR.get().cloned().unwrap_or_else(default_data_dir)
}

/// `SIGNALLAB_DATA_DIR` when set, else `Documents/SignalLab` in the home folder.
fn default_data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("SIGNALLAB_DATA_DIR").filter(|dir| !dir.is_empty()) {
        return PathBuf::from(dir);
    }
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).unwrap_or_else(|| ".".into());
    PathBuf::from(home).join("Documents").join("SignalLab")
}

/// What a save of a document in the data folder wrote: where, and the stamp of
/// what the file holds now — the `base` the next save names.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Saved {
    pub path: String,
    pub stamp: String,
}

/// A document's contents in a few characters: what a page read, so a save can
/// say what it means to replace. A file that is not there is `""`.
pub fn stamp(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes).iter().take(8).map(|byte| format!("{byte:02x}")).collect()
}

/// Taken by every save of a document here, so a save's look at the file and
/// its replacing it are one step for every page of this process.
pub fn document_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A save that read the file as `base` replaces it only if it still holds
/// that: another page, or a person in an editor, may have changed it since,
/// and their change must not vanish under a list that never saw it. No base
/// (a client that names none) replaces whatever is there.
pub fn check_base(path: &std::path::Path, existing: Option<&[u8]>, base: Option<&str>) -> crate::error::EngineResult<()> {
    let Some(base) = base else { return Ok(()) };
    let now = existing.map(stamp).unwrap_or_default();
    if now == base {
        Ok(())
    } else {
        Err(crate::error::EngineError::new("file.changed").with("path", path.display()))
    }
}
