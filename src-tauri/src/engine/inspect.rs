//! Cross-protocol capture bus.
//!
//! Every module (OSC monitor/sender, broadcast emitter, discovery listener, the
//! impairment relay …) publishes a normalized [`Frame`] here. The bus keeps a
//! bounded ring buffer so nothing is lost to a busy UI, and a pump task ships
//! batches to the front-end on `inspect://batch`. Capture is *armed* explicitly
//! — while disarmed, `push` is a single atomic load and costs nothing.

use std::collections::VecDeque;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::jobs::now_ms;

/// Frames held for export / snapshot, independent of what the UI managed to draw.
pub const RING_CAPACITY: usize = 8192;
/// Upper bound on frames handed to the webview per pump tick.
const MAX_BATCH: usize = 250;
const PUMP_INTERVAL_MS: u64 = 120;
/// Bytes of payload kept for the hex pane.
const HEX_LIMIT: usize = 1024;

/// One captured packet / request, normalized across protocols.
#[derive(Clone, Serialize)]
pub struct Frame {
    pub seq: u64,
    pub ts: u64,
    /// "osc", "udp", "tcp", "http".
    pub proto: String,
    /// "tx" (we sent it) or "rx" (we received it).
    pub dir: String,
    /// Which module captured it, e.g. "osc-monitor", "netsim", "discovery".
    pub source: String,
    pub job_id: Option<u64>,
    pub local: String,
    pub remote: String,
    pub bytes: usize,
    /// One-line human summary, e.g. `/hello/lfo f 0.42`.
    pub summary: String,
    /// Optional multi-line decode shown in the detail pane.
    pub detail: Option<String>,
    pub hex: Option<String>,
    /// What happened to it: "dropped", "duplicated", "corrupted", "auto-reply"…
    pub verdict: Option<String>,
}

impl Frame {
    pub fn new(proto: &str, dir: &str, source: &str) -> Self {
        Frame {
            seq: 0,
            ts: 0,
            proto: proto.to_string(),
            dir: dir.to_string(),
            source: source.to_string(),
            job_id: None,
            local: String::new(),
            remote: String::new(),
            bytes: 0,
            summary: String::new(),
            detail: None,
            hex: None,
            verdict: None,
        }
    }

    pub fn rx(proto: &str, source: &str) -> Self {
        Self::new(proto, "rx", source)
    }

    pub fn tx(proto: &str, source: &str) -> Self {
        Self::new(proto, "tx", source)
    }

    pub fn job(mut self, id: u64) -> Self {
        self.job_id = Some(id);
        self
    }

    pub fn local(mut self, addr: impl std::fmt::Display) -> Self {
        self.local = addr.to_string();
        self
    }

    pub fn remote(mut self, addr: impl std::fmt::Display) -> Self {
        self.remote = addr.to_string();
        self
    }

    pub fn summary(mut self, s: impl Into<String>) -> Self {
        self.summary = s.into();
        self
    }

    pub fn detail(mut self, s: impl Into<String>) -> Self {
        self.detail = Some(s.into());
        self
    }

    pub fn verdict(mut self, s: impl Into<String>) -> Self {
        self.verdict = Some(s.into());
        self
    }

    /// Record the payload: sets `bytes` and renders the hex pane.
    pub fn payload(mut self, bytes: &[u8]) -> Self {
        self.bytes = bytes.len();
        self.hex = Some(hex_dump(bytes));
        self
    }

    /// Record only the size, for paths where copying the payload isn't worth it.
    pub fn size(mut self, n: usize) -> Self {
        self.bytes = n;
        self
    }
}

#[derive(Default)]
struct Inner {
    enabled: AtomicBool,
    seq: AtomicU64,
    total: AtomicU64,
    bytes: AtomicU64,
    /// Frames that existed but never reached the UI (batch overflow or eviction).
    skipped: AtomicU64,
    /// Highest seq handed to the UI.
    cursor: AtomicU64,
    ring: Mutex<VecDeque<Frame>>,
}

/// Cheaply-cloneable handle to the capture bus (managed in Tauri state).
#[derive(Clone, Default)]
pub struct Capture {
    inner: Arc<Inner>,
}

#[derive(Clone, Serialize)]
pub struct CaptureStats {
    pub enabled: bool,
    pub total: u64,
    pub bytes: u64,
    pub skipped: u64,
    pub buffered: usize,
    pub capacity: usize,
}

#[derive(Clone, Serialize)]
struct Batch {
    frames: Vec<Frame>,
    stats: CaptureStats,
    /// Frames skipped since the previous batch — the UI draws a gap marker.
    skipped_now: u64,
}

impl Capture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&self, on: bool) {
        self.inner.enabled.store(on, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.enabled.load(Ordering::Relaxed)
    }

    /// Record a frame. Cheap no-op while capture is disarmed.
    pub fn push(&self, mut frame: Frame) {
        if !self.is_enabled() {
            return;
        }
        frame.seq = self.inner.seq.fetch_add(1, Ordering::Relaxed) + 1;
        if frame.ts == 0 {
            frame.ts = now_ms();
        }
        self.inner.total.fetch_add(1, Ordering::Relaxed);
        self.inner.bytes.fetch_add(frame.bytes as u64, Ordering::Relaxed);

        let mut ring = self.inner.ring.lock().unwrap();
        if ring.len() >= RING_CAPACITY {
            ring.pop_front();
        }
        ring.push_back(frame);
    }

    pub fn clear(&self) {
        self.inner.ring.lock().unwrap().clear();
        self.inner.total.store(0, Ordering::Relaxed);
        self.inner.bytes.store(0, Ordering::Relaxed);
        self.inner.skipped.store(0, Ordering::Relaxed);
        self.inner
            .cursor
            .store(self.inner.seq.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn stats(&self) -> CaptureStats {
        CaptureStats {
            enabled: self.is_enabled(),
            total: self.inner.total.load(Ordering::Relaxed),
            bytes: self.inner.bytes.load(Ordering::Relaxed),
            skipped: self.inner.skipped.load(Ordering::Relaxed),
            buffered: self.inner.ring.lock().unwrap().len(),
            capacity: RING_CAPACITY,
        }
    }

    /// Newest `limit` frames, oldest first — used to repopulate the UI on mount.
    pub fn snapshot(&self, limit: usize) -> Vec<Frame> {
        let ring = self.inner.ring.lock().unwrap();
        let skip = ring.len().saturating_sub(limit);
        ring.iter().skip(skip).cloned().collect()
    }

    /// Take everything the UI hasn't seen, newest-capped at [`MAX_BATCH`].
    fn drain(&self) -> (Vec<Frame>, u64) {
        let cursor = self.inner.cursor.load(Ordering::Relaxed);
        let fresh: Vec<Frame> = {
            let ring = self.inner.ring.lock().unwrap();
            ring.iter().filter(|f| f.seq > cursor).cloned().collect()
        };

        let batch: Vec<Frame> = if fresh.len() > MAX_BATCH {
            fresh[fresh.len() - MAX_BATCH..].to_vec()
        } else {
            fresh
        };

        let Some(last) = batch.last() else {
            return (batch, 0);
        };
        let new_cursor = last.seq;
        // Everything between the old and new cursor that isn't in the batch was
        // either evicted from the ring or trimmed by the batch cap.
        let advanced = new_cursor.saturating_sub(cursor);
        let skipped_now = advanced.saturating_sub(batch.len() as u64);
        self.inner.cursor.store(new_cursor, Ordering::Relaxed);
        if skipped_now > 0 {
            self.inner.skipped.fetch_add(skipped_now, Ordering::Relaxed);
        }
        (batch, skipped_now)
    }

    /// Write the whole ring to `~/Documents/SignalLab/`. Returns the file path.
    pub fn export(&self, format: &str) -> Result<String, String> {
        let frames = self.snapshot(RING_CAPACITY);
        if frames.is_empty() {
            return Err("capture buffer is empty".into());
        }
        let dir = export_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;

        let ext = if format == "txt" { "txt" } else { "jsonl" };
        let path = dir.join(format!("capture-{}.{ext}", now_ms()));
        let file = std::fs::File::create(&path).map_err(|e| format!("create file: {e}"))?;
        let mut out = std::io::BufWriter::new(file);

        if ext == "jsonl" {
            for f in &frames {
                let line = serde_json::to_string(f).map_err(|e| e.to_string())?;
                writeln!(out, "{line}").map_err(|e| e.to_string())?;
            }
        } else {
            writeln!(out, "Signal Lab capture — {} frames", frames.len())
                .map_err(|e| e.to_string())?;
            for f in &frames {
                writeln!(
                    out,
                    "\n#{seq} ts={ts} {dir} {proto} {remote} {bytes}B {verdict}\n  {summary}",
                    seq = f.seq,
                    ts = f.ts,
                    dir = f.dir.to_uppercase(),
                    proto = f.proto,
                    remote = f.remote,
                    bytes = f.bytes,
                    verdict = f.verdict.as_deref().unwrap_or(""),
                    summary = f.summary,
                )
                .map_err(|e| e.to_string())?;
                if let Some(d) = &f.detail {
                    writeln!(out, "  {}", d.replace('\n', "\n  ")).map_err(|e| e.to_string())?;
                }
                if let Some(h) = &f.hex {
                    write!(out, "{h}").map_err(|e| e.to_string())?;
                }
            }
        }
        out.flush().map_err(|e| e.to_string())?;
        Ok(path.to_string_lossy().into_owned())
    }
}

fn export_dir() -> std::path::PathBuf {
    super::signals::data_dir()
}

/// Publish a frame from anywhere that holds an `AppHandle`.
pub fn publish(app: &AppHandle, frame: Frame) {
    if let Some(cap) = app.try_state::<Capture>() {
        cap.push(frame);
    }
}

/// Is capture armed? Lets hot paths skip building a frame at all.
pub fn armed(app: &AppHandle) -> bool {
    app.try_state::<Capture>()
        .map(|c| c.is_enabled())
        .unwrap_or(false)
}

/// Ship batches to the UI. Started once from the Tauri setup hook.
pub fn spawn_pump(app: AppHandle, capture: Capture) {
    tauri::async_runtime::spawn(async move {
        let mut idle_ticks: u32 = 0;
        loop {
            tokio::time::sleep(Duration::from_millis(PUMP_INTERVAL_MS)).await;
            if !capture.is_enabled() {
                continue;
            }
            let (frames, skipped_now) = capture.drain();
            idle_ticks = if frames.is_empty() { idle_ticks + 1 } else { 0 };
            // Emit whenever there is traffic, and occasionally when idle so the
            // counters in the UI stay honest without a per-tick event storm.
            if frames.is_empty() && idle_ticks % 8 != 0 {
                continue;
            }
            let _ = app.emit(
                "inspect://batch",
                Batch {
                    frames,
                    stats: capture.stats(),
                    skipped_now,
                },
            );
        }
    });
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// `0000  48 65 6c 6c 6f 20 77 6f  72 6c 64 21   |Hello world!|`
pub fn hex_dump(bytes: &[u8]) -> String {
    let shown = &bytes[..bytes.len().min(HEX_LIMIT)];
    let mut out = String::with_capacity(shown.len() * 4);
    for (row, chunk) in shown.chunks(16).enumerate() {
        out.push_str(&format!("{:04x}  ", row * 16));
        for (i, b) in chunk.iter().enumerate() {
            out.push_str(&format!("{b:02x} "));
            if i == 7 {
                out.push(' ');
            }
        }
        for i in chunk.len()..16 {
            out.push_str("   ");
            if i == 7 {
                out.push(' ');
            }
        }
        out.push_str(" |");
        for b in chunk {
            out.push(if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            });
        }
        out.push_str("|\n");
    }
    if bytes.len() > HEX_LIMIT {
        out.push_str(&format!("… {} more bytes\n", bytes.len() - HEX_LIMIT));
    }
    out
}

/// Best-effort decode of an opaque datagram: OSC when it looks like OSC,
/// otherwise a printable preview. Returns `(proto, summary, detail)`.
pub fn describe_payload(bytes: &[u8]) -> (&'static str, String, Option<String>) {
    use super::osc_codec::{arg_str, decode_packet, summarize_messages};

    if matches!(bytes.first(), Some(b'/') | Some(b'#')) {
        if let Ok(msgs) = decode_packet(bytes) {
            if !msgs.is_empty() {
                let detail = msgs
                    .iter()
                    .map(|m| {
                        let args = m.args.iter().map(arg_str).collect::<Vec<_>>().join(" ");
                        if args.is_empty() {
                            m.address.clone()
                        } else {
                            format!("{} {}", m.address, args)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                return ("osc", summarize_messages(&msgs), Some(detail));
            }
        }
    }
    ("udp", ascii_preview(bytes, 96), None)
}

/// Short printable preview of an opaque payload, for the summary column.
pub fn ascii_preview(bytes: &[u8], max: usize) -> String {
    let mut s: String = bytes
        .iter()
        .take(max)
        .map(|b| {
            if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            }
        })
        .collect();
    if bytes.len() > max {
        s.push('…');
    }
    s
}

/// A cheap sampling gate: allows one event per `min_ms`, lock-free.
pub struct Gate {
    last_ms: AtomicU64,
    min_ms: u64,
}

impl Gate {
    pub fn new(min_ms: u64) -> Self {
        Gate {
            last_ms: AtomicU64::new(0),
            min_ms,
        }
    }

    /// True at most once per `min_ms`. Safe to call from many tasks.
    pub fn allow(&self) -> bool {
        let now = now_ms();
        let last = self.last_ms.load(Ordering::Relaxed);
        if now.saturating_sub(last) < self.min_ms {
            return false;
        }
        self.last_ms
            .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_dump_renders_offsets_and_ascii() {
        let d = hex_dump(b"Hi");
        assert!(d.starts_with("0000  48 69 "), "unexpected dump: {d}");
        assert!(d.contains("|Hi|"));
    }

    #[test]
    fn disarmed_capture_records_nothing() {
        let cap = Capture::new();
        cap.push(Frame::rx("osc", "test"));
        assert_eq!(cap.stats().total, 0);
        cap.set_enabled(true);
        cap.push(Frame::rx("osc", "test").summary("/x"));
        assert_eq!(cap.stats().total, 1);
    }

    #[test]
    fn drain_advances_cursor_and_counts_skips() {
        let cap = Capture::new();
        cap.set_enabled(true);
        for _ in 0..(MAX_BATCH + 10) {
            cap.push(Frame::rx("udp", "test").size(4));
        }
        let (batch, skipped) = cap.drain();
        assert_eq!(batch.len(), MAX_BATCH);
        assert_eq!(skipped, 10);
        let (empty, _) = cap.drain();
        assert!(empty.is_empty());
    }

    #[test]
    fn snapshot_returns_newest_frames_in_order() {
        let cap = Capture::new();
        cap.set_enabled(true);
        for i in 0..10u64 {
            cap.push(Frame::rx("udp", "test").summary(format!("#{i}")));
        }
        let snap = cap.snapshot(3);
        assert_eq!(snap.len(), 3);
        assert_eq!(snap[0].summary, "#7");
        assert_eq!(snap[2].summary, "#9");
    }
}
