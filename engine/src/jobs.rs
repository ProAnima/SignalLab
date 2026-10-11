//! Job registry: tracks every long-running task (OSC monitor/generator, network
//! impairment proxy, storm generator, port scan) so the UI can list and stop them.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::task::JoinHandle;

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Clone, Debug, Serialize)]
pub struct JobInfo {
    pub id: u64,
    /// Machine-readable kind: "experiment", "osc-monitor", "osc-gen", "mqtt",
    /// "beacon", "discovery", "http-burst", "netsim", "storm", "scan".
    pub kind: String,
    /// One English line for logs (the server writes it); the interface shows
    /// `job.<kind>` filled in with `params` instead.
    pub label: String,
    /// The values the label is made of (target, bind, host, …), by name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
    pub started_ms: u64,
}

impl JobInfo {
    /// A job of `kind`, started now.
    pub fn new(id: u64, kind: &str, label: impl Into<String>) -> Self {
        JobInfo { id, kind: kind.into(), label: label.into(), params: BTreeMap::new(), started_ms: now_ms() }
    }

    /// One value of the label, for `job.<kind>`.
    pub fn with(mut self, name: &str, value: impl ToString) -> Self {
        self.params.insert(name.into(), value.to_string());
        self
    }
}

struct JobEntry {
    info: JobInfo,
    handle: JoinHandle<()>,
}

/// Aborts a set of child tasks when dropped.
///
/// Stopping a job aborts only its top-level task, and tokio *detaches* a child
/// `spawn` when its `JoinHandle` is dropped rather than cancelling it — so a
/// module that spawns workers or a stats reporter would keep them running after
/// "stop": a scanner still probing ports, a relay still forwarding, a reporter
/// still emitting. Holding the children here, inside the job's own async block,
/// means the outer task's cancellation drops this guard and takes them with it.
#[derive(Default)]
pub struct TaskGuard {
    children: Vec<tokio::task::AbortHandle>,
}

impl TaskGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a child task so it is aborted when the job ends, however it ends.
    /// Takes an `AbortHandle` (from `JoinHandle::abort_handle`) rather than the
    /// handle itself, so the spawner can still await the task on its normal path
    /// while the guard remains able to abort it on cancel.
    pub fn watch(&mut self, handle: tokio::task::AbortHandle) {
        self.children.push(handle);
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        let mut settling = SETTLING.lock().unwrap();
        settling.retain(|child| !child.is_finished());
        for child in self.children.drain(..) {
            child.abort();
            if !child.is_finished() {
                settling.push(child);
            }
        }
    }
}

/// Children aborted by a [`TaskGuard`] and not dropped yet: an abort takes
/// effect when the runtime next polls the task, so a socket a child holds is
/// still bound for a moment after its job is gone. [`settle`] waits for them.
static SETTLING: Mutex<Vec<tokio::task::AbortHandle>> = Mutex::new(Vec::new());

/// How long a stop waits for a job to let go. A task that never yields cannot
/// be cancelled at all; past this, Stop answers anyway rather than hang.
const SETTLE_LIMIT: std::time::Duration = std::time::Duration::from_secs(2);

/// Wait until the aborted `handles` have been dropped, and every child their
/// guards aborted with them — what they held (a bound port) is free then.
async fn settle(handles: Vec<JoinHandle<()>>) {
    let deadline = tokio::time::Instant::now() + SETTLE_LIMIT;
    for handle in handles {
        if tokio::time::timeout_at(deadline, handle).await.is_err() {
            tracing::warn!("a stopped job did not let go within {SETTLE_LIMIT:?}");
            return;
        }
    }
    loop {
        let pending = {
            let mut settling = SETTLING.lock().unwrap();
            settling.retain(|child| !child.is_finished());
            !settling.is_empty()
        };
        if !pending || tokio::time::Instant::now() >= deadline {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    }
}

/// Cheaply-cloneable handle to the shared registry (managed in Tauri state).
#[derive(Clone, Default)]
pub struct JobRegistry {
    inner: Arc<Mutex<Jobs>>,
    counter: Arc<AtomicU64>,
}

#[derive(Default)]
struct Jobs {
    running: HashMap<u64, JobEntry>,
    /// Finished before they were inserted: a job is spawned before its handle
    /// can be registered, and a short one (a burst of one against a refused
    /// port) may already be over. Inserting one of these is a no-op, so no
    /// job stays listed after it has ended.
    ended_early: HashSet<u64>,
    /// The last id given out when Stop all last ran: a job numbered before it
    /// and inserted after it was still starting then (a run opens its ports
    /// under its number), and is stopped as it arrives rather than left running.
    stopped_through: u64,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn insert(&self, info: JobInfo, handle: JoinHandle<()>) {
        let mut jobs = self.inner.lock().unwrap();
        if jobs.ended_early.remove(&info.id) {
            return;
        }
        if info.id <= jobs.stopped_through {
            handle.abort();
            return;
        }
        jobs.running.insert(info.id, JobEntry { info, handle });
    }

    /// Abort a running job. Returns true if a job with that id existed.
    ///
    /// Only asks: the task is dropped — its sockets closed — when the runtime
    /// next gets to it, which may be after this returns. A front door that
    /// answers a person's Stop uses [`JobRegistry::stop_and_wait`], so a start
    /// right after binds the port the job held.
    pub fn stop(&self, id: u64) -> bool {
        if let Some(entry) = self.inner.lock().unwrap().running.remove(&id) {
            entry.handle.abort();
            true
        } else {
            false
        }
    }

    /// Abort a running job and wait until it has let go of what it held: its
    /// task dropped, and with it the children its [`TaskGuard`] aborts.
    /// Returns true if a job with that id existed.
    pub async fn stop_and_wait(&self, id: u64) -> bool {
        let entry = self.inner.lock().unwrap().running.remove(&id);
        match entry {
            Some(entry) => {
                entry.handle.abort();
                settle(vec![entry.handle]).await;
                true
            }
            None => false,
        }
    }

    /// [`JobRegistry::stop_all`], waiting for every job as `stop_and_wait` does.
    pub async fn stop_all_and_wait(&self) {
        let handles: Vec<JoinHandle<()>> = {
            let mut guard = self.inner.lock().unwrap();
            guard.stopped_through = self.counter.load(Ordering::SeqCst);
            guard.running.drain().map(|(_, entry)| entry.handle).collect()
        };
        for handle in &handles {
            handle.abort();
        }
        settle(handles).await;
    }

    /// Remove a job that finished on its own (without abort).
    pub fn finish(&self, id: u64) {
        let mut jobs = self.inner.lock().unwrap();
        // Only a job given out by `next_id` and not stopped can be inserted later.
        if jobs.running.remove(&id).is_none() && id <= self.counter.load(Ordering::Relaxed) {
            jobs.ended_early.insert(id);
        }
    }

    pub fn list(&self) -> Vec<JobInfo> {
        let mut v: Vec<JobInfo> = self
            .inner
            .lock()
            .unwrap()
            .running
            .values()
            .map(|e| e.info.clone())
            .collect();
        v.sort_by_key(|j| j.id);
        v
    }

    pub fn stop_all(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.stopped_through = self.counter.load(Ordering::SeqCst);
        for (_, entry) in guard.running.drain() {
            entry.handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_job_that_ends_before_it_is_registered_is_not_listed() {
        let jobs = JobRegistry::new();
        let id = jobs.next_id();
        let handle = tokio::spawn(async {});
        jobs.finish(id);
        jobs.insert(JobInfo::new(id, "storm", "x"), handle);
        assert!(jobs.list().is_empty(), "over before it was inserted");
        let id = jobs.next_id();
        jobs.insert(JobInfo::new(id, "storm", "y"), tokio::spawn(std::future::pending()));
        assert_eq!(jobs.list().len(), 1);
        jobs.finish(id);
        assert!(jobs.list().is_empty() && jobs.inner.lock().unwrap().ended_early.is_empty(), "the usual order leaves nothing behind");
    }

    /// Stop all while a start is still opening its ports (a run takes its
    /// number first): the job it inserts afterwards is stopped, not left running.
    #[tokio::test]
    async fn stop_all_stops_a_job_still_starting() {
        let jobs = JobRegistry::new();
        let starting = jobs.next_id();
        jobs.stop_all_and_wait().await;
        let handle = tokio::spawn(std::future::pending::<()>());
        let abort = handle.abort_handle();
        jobs.insert(JobInfo::new(starting, "experiment", "x"), handle);
        assert!(jobs.list().is_empty(), "started after Stop all, listed anyway");
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while !abort.is_finished() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("and its task was stopped");
        // A job numbered after Stop all runs as usual.
        let later = jobs.next_id();
        jobs.insert(JobInfo::new(later, "experiment", "y"), tokio::spawn(std::future::pending()));
        assert_eq!(jobs.list().len(), 1);
        jobs.stop_all();
    }

    #[test]
    fn a_job_carries_the_values_of_its_label() {
        let info = JobInfo::new(3, "scan", "Scan 10.0.0.1 :1-80").with("host", "10.0.0.1").with("from", 1).with("to", 80);
        let value = serde_json::to_value(&info).unwrap();
        assert_eq!(value["params"], serde_json::json!({ "host": "10.0.0.1", "from": "1", "to": "80" }));
        assert_eq!((value["kind"].as_str(), value["label"].as_str()), (Some("scan"), Some("Scan 10.0.0.1 :1-80")));
        let bare = serde_json::to_value(JobInfo::new(4, "storm", "x")).unwrap();
        assert!(bare.get("params").is_none(), "no params, no field");
    }

    /// The interface shows `job.<kind>` with the job's params; a kind without
    /// a text, or a text asking for a value the job does not give, would show
    /// the raw kind or a bare `{placeholder}`.
    #[test]
    fn every_job_kind_has_a_text_that_its_params_fill_in() {
        use std::collections::{BTreeMap, BTreeSet};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        // A Windows checkout may have CRLF line ends; the patterns below are written for LF.
        let read = |path: std::path::PathBuf| std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
        let en = read(root.join("../src/lib/locales/en.ts"));
        // `JobInfo::new(id, "kind", label)` and the `.with("name", …)` calls that follow it.
        let job = regex::Regex::new(r#"JobInfo::new\([^,]+,\s*"([a-z-]+)"((?:[^;]|;[^\n])*?);\n"#).unwrap();
        let with = regex::Regex::new(r#"\.with\("([a-z_]+)""#).unwrap();
        let mut kinds: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for entry in std::fs::read_dir(root.join("src")).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().is_some_and(|name| name == "jobs.rs") || path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let text = read(path);
            for found in job.captures_iter(&text) {
                kinds.entry(found[1].to_string()).or_default().extend(with.captures_iter(&found[2]).map(|name| name[1].to_string()));
            }
        }
        assert!(kinds.len() >= 10, "the scan found the job kinds: {kinds:?}");
        let placeholder = regex::Regex::new(r"\{(\w+)").unwrap();
        for (kind, params) in &kinds {
            let key = format!("\"job.{kind}\": \"");
            let start = en.find(&key).unwrap_or_else(|| panic!("no text in en.ts for job.{kind}")) + key.len();
            let text = &en[start..start + en[start..].find("\",\n").unwrap()];
            for name in placeholder.captures_iter(text).map(|found| found[1].to_string()) {
                assert!(params.contains(&name), "job.{kind} asks for {{{name}}}, which the job does not give ({params:?})");
            }
        }
    }
}
