//! Secrets for experiments. Values live in the operating system's credential
//! store and only names appear in documents; nothing here hands a value to the
//! interface. While a run or a *Send now* uses values, they are masked in
//! everything the engine reports, the Inspector included. Specified in
//! `docs/develop/design-data.md`, section 11.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use serde_json::Value;

use super::error::{EngineError, EngineResult};
use super::template;

/// The credential-store service every entry is filed under (stores exist on Windows and macOS).
#[cfg(any(windows, target_os = "macos"))]
pub const SERVICE: &str = "SignalLab";
/// What a secret value is shown as.
pub const MASK: &str = "••••";
pub const MAX_SECRET_BYTES: usize = 16 * 1024;
const MAX_NAME: usize = 128;

/// Where secret values are kept. The engine uses the system store; tests use
/// the in-memory one, so no test touches the real credential store.
pub trait SecretStore: Send + Sync {
    fn get(&self, name: &str) -> EngineResult<Option<String>>;
    fn set(&self, name: &str, value: &str) -> EngineResult<()>;
    fn delete(&self, name: &str) -> EngineResult<()>;
    /// Whether `set` and `delete` can succeed; the interface hides them otherwise.
    fn writable(&self) -> bool {
        true
    }
}

/// The Windows Credential Manager or the macOS Keychain.
pub struct SystemStore;

/// The credential store failed; its own wording goes into the detail.
#[cfg(any(windows, target_os = "macos"))]
fn store_error(name: &str, error: keyring::Error) -> EngineError {
    EngineError::new("secret.store").with("name", name).because(error)
}

#[cfg(any(windows, target_os = "macos"))]
impl SecretStore for SystemStore {
    fn get(&self, name: &str) -> EngineResult<Option<String>> {
        match keyring::Entry::new(SERVICE, name).and_then(|entry| entry.get_password()) {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(store_error(name, error)),
        }
    }

    fn set(&self, name: &str, value: &str) -> EngineResult<()> {
        keyring::Entry::new(SERVICE, name)
            .and_then(|entry| entry.set_password(value))
            .map_err(|error| store_error(name, error))
    }

    fn delete(&self, name: &str) -> EngineResult<()> {
        match keyring::Entry::new(SERVICE, name).and_then(|entry| entry.delete_credential()) {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(store_error(name, error)),
        }
    }
}

/// No persistent store on this platform: say so rather than keep values in memory.
#[cfg(not(any(windows, target_os = "macos")))]
impl SecretStore for SystemStore {
    fn get(&self, _name: &str) -> EngineResult<Option<String>> {
        Err(EngineError::new("secret.unsupported"))
    }
    fn set(&self, _name: &str, _value: &str) -> EngineResult<()> {
        Err(EngineError::new("secret.unsupported"))
    }
    fn delete(&self, _name: &str) -> EngineResult<()> {
        Err(EngineError::new("secret.unsupported"))
    }
}

/// Where a server gets secrets: there is no credential store in a container,
/// and a value typed into a browser must not end up somewhere weaker. Values
/// are read, never written: `SIGNALLAB_SECRET_<NAME>` from the environment, or
/// the file `<dir>/<NAME>` — the Docker secrets layout, e.g.
/// `/run/secrets/signallab/API_TOKEN`. Setting one from the interface is
/// refused with `secret.read_only`.
pub struct FileStore {
    dir: Option<std::path::PathBuf>,
    env: EnvLookup,
}

/// Reads one environment variable.
type EnvLookup = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;

impl FileStore {
    pub fn new(dir: Option<std::path::PathBuf>) -> Self {
        FileStore { dir, env: Box::new(|key| std::env::var(key).ok()) }
    }

    /// Environment lookups through `env` instead of the process environment.
    pub fn with_env(mut self, env: impl Fn(&str) -> Option<String> + Send + Sync + 'static) -> Self {
        self.env = Box::new(env);
        self
    }
}

impl SecretStore for FileStore {
    fn get(&self, name: &str) -> EngineResult<Option<String>> {
        // Names are identifiers, so a name can never reach outside the folder.
        check_name(name)?;
        if let Some(value) = (self.env)(&format!("SIGNALLAB_SECRET_{name}")).filter(|value| !value.is_empty()) {
            return Ok(Some(value));
        }
        let Some(dir) = &self.dir else { return Ok(None) };
        let path = dir.join(name);
        let failed = |error: std::io::Error| EngineError::new("secret.store").with("name", name).because(error);
        match std::fs::metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(failed(error)),
            Ok(meta) if meta.len() > MAX_SECRET_BYTES as u64 => {
                return Err(EngineError::new("secret.too_large").with("name", name).with("max", MAX_SECRET_BYTES / 1024));
            }
            Ok(_) => {}
        }
        // A file written by an editor or `echo` ends in a newline that is not part of the value.
        let value = std::fs::read_to_string(&path).map_err(failed)?;
        let value = value.trim_end_matches(['\r', '\n']);
        Ok((!value.is_empty()).then(|| value.to_string()))
    }

    fn set(&self, name: &str, _value: &str) -> EngineResult<()> {
        Err(EngineError::new("secret.read_only").with("name", name))
    }

    fn delete(&self, name: &str) -> EngineResult<()> {
        Err(EngineError::new("secret.read_only").with("name", name))
    }

    fn writable(&self) -> bool {
        false
    }
}

/// For tests: never touches the real credential store.
#[cfg(test)]
#[derive(Default)]
pub struct MemoryStore(Mutex<BTreeMap<String, String>>);

#[cfg(test)]
impl SecretStore for MemoryStore {
    fn get(&self, name: &str) -> EngineResult<Option<String>> {
        Ok(self.0.lock().unwrap().get(name).cloned())
    }
    fn set(&self, name: &str, value: &str) -> EngineResult<()> {
        self.0.lock().unwrap().insert(name.into(), value.into());
        Ok(())
    }
    fn delete(&self, name: &str) -> EngineResult<()> {
        self.0.lock().unwrap().remove(name);
        Ok(())
    }
}

pub fn check_name(name: &str) -> EngineResult<()> {
    if !template::is_ident(name) || name.len() > MAX_NAME {
        return Err(EngineError::new("secret.name_invalid").with("name", name).with("max", MAX_NAME));
    }
    Ok(())
}

/// Which of `names` are stored.
pub fn status(store: &dyn SecretStore, names: &[String]) -> EngineResult<BTreeMap<String, bool>> {
    names.iter().map(|name| Ok((name.clone(), store.get(name)?.is_some()))).collect()
}

pub fn set(store: &dyn SecretStore, name: &str, value: &str) -> EngineResult<()> {
    check_name(name)?;
    if value.is_empty() {
        return Err(EngineError::new("secret.empty").with("name", name));
    }
    if value.len() > MAX_SECRET_BYTES {
        return Err(EngineError::new("secret.too_large").with("name", name).with("max", MAX_SECRET_BYTES / 1024));
    }
    store.set(name, value)
}

/// Removes a stored value; removing one that is not stored is not an error.
pub fn delete(store: &dyn SecretStore, name: &str) -> EngineResult<()> {
    check_name(name)?;
    store.delete(name)
}

/// The values of `names`; a name that is not stored is an error naming it.
pub fn load(store: &dyn SecretStore, names: &[String]) -> EngineResult<BTreeMap<String, String>> {
    names
        .iter()
        .map(|name| match store.get(name)? {
            Some(value) => Ok((name.clone(), value)),
            None => Err(EngineError::new("secret.missing").with("name", name)),
        })
        .collect()
}

/// Every occurrence of a value replaced by the mask. Longest values first, so
/// a value that contains another is masked whole.
pub fn mask(text: &str, values: &[String]) -> String {
    let mut sorted: Vec<&String> = values.iter().filter(|value| !value.is_empty()).collect();
    sorted.sort_by_key(|value| std::cmp::Reverse(value.len()));
    let mut out = text.to_string();
    for value in sorted {
        if out.contains(value.as_str()) {
            out = out.replace(value.as_str(), MASK);
        }
    }
    out
}

/// `mask` over every string inside a JSON value.
pub fn mask_value(value: &Value, values: &[String]) -> Value {
    match value {
        Value::String(text) => Value::String(mask(text, values)),
        Value::Array(items) => Value::Array(items.iter().map(|item| mask_value(item, values)).collect()),
        Value::Object(map) => Value::Object(map.iter().map(|(key, item)| (key.clone(), mask_value(item, values))).collect()),
        other => other.clone(),
    }
}

/// Bytes with every occurrence of a value overwritten by `*`, same length, so
/// offsets in a hex dump stay true.
pub fn mask_bytes<'a>(bytes: &'a [u8], values: &[String]) -> Cow<'a, [u8]> {
    let mut out: Option<Vec<u8>> = None;
    for value in values.iter().map(String::as_bytes).filter(|value| !value.is_empty()) {
        let current = out.as_deref().unwrap_or(bytes);
        if current.len() < value.len() || !current.windows(value.len()).any(|window| window == value) {
            continue;
        }
        let buffer = out.get_or_insert_with(|| bytes.to_vec());
        let mut at = 0;
        while at + value.len() <= buffer.len() {
            if &buffer[at..at + value.len()] == value {
                buffer[at..at + value.len()].fill(b'*');
                at += value.len();
            } else {
                at += 1;
            }
        }
    }
    match out {
        Some(buffer) => Cow::Owned(buffer),
        None => Cow::Borrowed(bytes),
    }
}

// ---- values in use, for the Inspector ------------------------------------------

static ACTIVE_COUNT: AtomicUsize = AtomicUsize::new(0);
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static ACTIVE: Mutex<Vec<(u64, Vec<String>)>> = Mutex::new(Vec::new());

/// Registers values as in use until dropped; captured frames are masked meanwhile.
pub struct Redaction(Option<u64>);

pub fn redact(values: Vec<String>) -> Redaction {
    let values: Vec<String> = values.into_iter().filter(|value| !value.is_empty()).collect();
    if values.is_empty() {
        return Redaction(None);
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    ACTIVE.lock().unwrap().push((id, values));
    ACTIVE_COUNT.fetch_add(1, Ordering::SeqCst);
    Redaction(Some(id))
}

impl Drop for Redaction {
    fn drop(&mut self) {
        if let Some(id) = self.0 {
            ACTIVE.lock().unwrap().retain(|(entry, _)| *entry != id);
            ACTIVE_COUNT.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

/// Values currently in use; empty (one atomic load) when no secret is active.
pub fn active() -> Vec<String> {
    if ACTIVE_COUNT.load(Ordering::SeqCst) == 0 {
        return Vec::new();
    }
    ACTIVE.lock().unwrap().iter().flat_map(|(_, values)| values.iter().cloned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn values_are_stored_by_name_and_missing_ones_are_named() {
        let store = MemoryStore::default();
        assert!(set(&store, "API_TOKEN", "t0k3n").is_ok());
        let invalid = set(&store, "2bad", "x").unwrap_err();
        assert_eq!((invalid.code.as_str(), invalid.params["name"].as_str()), ("secret.name_invalid", "2bad"));
        assert!(set(&store, "EMPTY", "").unwrap_err().is("secret.empty"));
        assert!(set(&store, "BIG", &"x".repeat(MAX_SECRET_BYTES + 1)).unwrap_err().is("secret.too_large"));
        let names = vec!["API_TOKEN".to_string(), "OTHER".to_string()];
        assert_eq!(status(&store, &names).unwrap(), BTreeMap::from([("API_TOKEN".into(), true), ("OTHER".into(), false)]));
        let missing = load(&store, &names).unwrap_err();
        assert_eq!((missing.code.as_str(), missing.params["name"].as_str()), ("secret.missing", "OTHER"));
        assert_eq!(load(&store, &names[..1]).unwrap()["API_TOKEN"], "t0k3n");
        delete(&store, "API_TOKEN").unwrap();
        delete(&store, "API_TOKEN").unwrap();
        assert!(delete(&store, "bad name").unwrap_err().is("secret.name_invalid"));
        assert_eq!(store.get("API_TOKEN").unwrap(), None);
    }

    #[test]
    fn a_server_reads_secrets_from_the_environment_or_files_and_never_writes() {
        let dir = std::env::temp_dir().join(format!("signallab-secrets-{:016x}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("API_TOKEN"), "from-file\n").unwrap();
        std::fs::write(dir.join("EMPTY"), "\n").unwrap();
        std::fs::write(dir.join("HUGE"), "x".repeat(MAX_SECRET_BYTES + 1)).unwrap();
        let store = FileStore::new(Some(dir.clone()))
            .with_env(|key| (key == "SIGNALLAB_SECRET_FROM_ENV").then(|| "env-value".to_string()));
        assert_eq!(store.get("API_TOKEN").unwrap().as_deref(), Some("from-file"));
        assert_eq!(store.get("FROM_ENV").unwrap().as_deref(), Some("env-value"));
        assert_eq!(store.get("MISSING").unwrap(), None);
        assert_eq!(store.get("EMPTY").unwrap(), None, "an empty file is no secret");
        assert!(store.get("HUGE").unwrap_err().is("secret.too_large"));
        assert!(store.get("../escape").unwrap_err().is("secret.name_invalid"), "a name cannot leave the folder");
        assert!(set(&store, "API_TOKEN", "x").unwrap_err().is("secret.read_only"));
        assert!(delete(&store, "API_TOKEN").unwrap_err().is("secret.read_only"));
        assert!(!store.writable() && MemoryStore::default().writable());
        assert_eq!(load(&store, &["API_TOKEN".to_string()]).unwrap()["API_TOKEN"], "from-file");
        assert_eq!(FileStore::new(None).with_env(|_| None).get("API_TOKEN").unwrap(), None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn masking_covers_text_json_and_bytes() {
        let values = vec!["s3cret".to_string(), "s3cret-long".to_string()];
        assert_eq!(mask("Bearer s3cret-long and s3cret", &values), "Bearer •••• and ••••");
        assert_eq!(mask("nothing here", &values), "nothing here");
        assert_eq!(
            mask_value(&json!({ "a": ["x s3cret"], "n": 1 }), &values),
            json!({ "a": ["x ••••"], "n": 1 })
        );
        assert_eq!(&*mask_bytes(b"key=s3cret;", &values), b"key=******;");
        assert!(matches!(mask_bytes(b"clean", &values), Cow::Borrowed(_)));
    }

    #[test]
    fn redaction_lasts_as_long_as_its_guard() {
        assert!(!active().contains(&"guarded-value".to_string()));
        let guard = redact(vec!["guarded-value".into(), String::new()]);
        assert!(active().contains(&"guarded-value".to_string()));
        drop(guard);
        assert!(!active().contains(&"guarded-value".to_string()));
        let _none = redact(Vec::new());
    }
}
