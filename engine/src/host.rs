//! What the engine needs from whatever runs it: somewhere to deliver events,
//! and the capture bus. The desktop app delivers events over Tauri, the server
//! over WebSockets, tests into a recorder — nothing in the engine knows which.

use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::inspect::Capture;

/// Delivers one event to the interface. Called from the engine's tasks, some
/// at high rate, so an implementation must not block.
pub trait EventSink: Send + Sync + 'static {
    fn emit(&self, event: &str, payload: Value);
}

/// The engine's surroundings. Cheap to clone; every module that reports
/// anything holds one.
#[derive(Clone)]
pub struct Host {
    events: Arc<dyn EventSink>,
    capture: Capture,
}

impl Host {
    pub fn new(events: Arc<dyn EventSink>, capture: Capture) -> Self {
        Host { events, capture }
    }

    /// Deliver `payload` as `event`. A payload that does not serialize is an
    /// engine bug; it is logged, never sent half-formed.
    pub fn emit(&self, event: &str, payload: impl Serialize) {
        match serde_json::to_value(payload) {
            Ok(value) => self.events.emit(event, value),
            Err(error) => tracing::error!(event, %error, "event payload does not serialize"),
        }
    }

    pub fn capture(&self) -> &Capture {
        &self.capture
    }
}

/// Discards every event: for tools that only want results.
pub struct NoEvents;

impl EventSink for NoEvents {
    fn emit(&self, _event: &str, _payload: Value) {}
}

/// Keeps every event, for tests and diagnostics.
#[derive(Default)]
pub struct Recorder {
    events: Mutex<Vec<(String, Value)>>,
    arrived: Condvar,
}

impl Recorder {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Every event so far, oldest first.
    pub fn events(&self) -> Vec<(String, Value)> {
        self.events.lock().unwrap().clone()
    }

    /// The payloads of `event` so far.
    pub fn payloads(&self, event: &str) -> Vec<Value> {
        self.events.lock().unwrap().iter().filter(|(name, _)| name == event).map(|(_, payload)| payload.clone()).collect()
    }

    /// Blocks until an `event` payload satisfies `test`, or `timeout` passes.
    /// For tests on a multi-threaded runtime; call it from a blocking context.
    pub fn wait_for(&self, event: &str, timeout: Duration, test: impl Fn(&Value) -> bool) -> Option<Value> {
        let deadline = Instant::now() + timeout;
        let mut events = self.events.lock().unwrap();
        loop {
            if let Some((_, payload)) = events.iter().find(|(name, payload)| name == event && test(payload)) {
                return Some(payload.clone());
            }
            let left = deadline.checked_duration_since(Instant::now())?;
            events = self.arrived.wait_timeout(events, left).unwrap().0;
        }
    }
}

impl EventSink for Recorder {
    fn emit(&self, event: &str, payload: Value) {
        self.events.lock().unwrap().push((event.to_string(), payload));
        self.arrived.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_reach_the_sink_as_json() {
        let recorder = Recorder::new();
        let host = Host::new(recorder.clone(), Capture::new());
        host.emit("job://ended", serde_json::json!({ "job_id": 7 }));
        #[derive(Serialize)]
        struct Tick {
            n: u32,
        }
        host.emit("tick", Tick { n: 3 });
        assert_eq!(recorder.payloads("job://ended"), [serde_json::json!({ "job_id": 7 })]);
        assert_eq!(recorder.wait_for("tick", Duration::from_millis(10), |payload| payload["n"] == 3), Some(serde_json::json!({ "n": 3 })));
        assert_eq!(recorder.wait_for("tick", Duration::from_millis(10), |payload| payload["n"] == 4), None);
        assert_eq!(recorder.events().len(), 2);
    }
}
