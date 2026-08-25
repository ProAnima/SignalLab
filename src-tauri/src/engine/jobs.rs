//! Job registry: tracks every long-running task (OSC monitor/generator, network
//! impairment proxy, storm generator, port scan) so the UI can list and stop them.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::async_runtime::JoinHandle;

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
