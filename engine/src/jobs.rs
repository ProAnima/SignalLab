//! Job registry: tracks every long-running task (OSC monitor/generator, network
//! impairment proxy, storm generator, port scan) so the UI can list and stop them.

use std::collections::HashMap;
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

#[derive(Clone, Serialize)]
pub struct JobInfo {
    pub id: u64,
    /// Machine-readable kind: "osc-monitor", "osc-gen", "http-burst", "netsim",
    /// "storm", "scan".
    pub kind: String,
    /// Human-readable label shown in the Jobs panel.
    pub label: String,
    pub started_ms: u64,
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
        for child in &self.children {
            child.abort();
        }
    }
}

/// Cheaply-cloneable handle to the shared registry (managed in Tauri state).
#[derive(Clone, Default)]
pub struct JobRegistry {
    inner: Arc<Mutex<HashMap<u64, JobEntry>>>,
    counter: Arc<AtomicU64>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn insert(&self, info: JobInfo, handle: JoinHandle<()>) {
        self.inner
            .lock()
            .unwrap()
            .insert(info.id, JobEntry { info, handle });
    }

    /// Abort a running job. Returns true if a job with that id existed.
    pub fn stop(&self, id: u64) -> bool {
        if let Some(entry) = self.inner.lock().unwrap().remove(&id) {
            entry.handle.abort();
            true
        } else {
            false
        }
    }

    /// Remove a job that finished on its own (without abort).
    pub fn finish(&self, id: u64) {
        self.inner.lock().unwrap().remove(&id);
    }

    pub fn list(&self) -> Vec<JobInfo> {
        let mut v: Vec<JobInfo> = self
            .inner
            .lock()
            .unwrap()
            .values()
            .map(|e| e.info.clone())
            .collect();
        v.sort_by_key(|j| j.id);
        v
    }

    pub fn stop_all(&self) {
        let mut guard = self.inner.lock().unwrap();
        for (_, entry) in guard.drain() {
            entry.handle.abort();
        }
    }
}
