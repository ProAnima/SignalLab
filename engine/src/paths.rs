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
