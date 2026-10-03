//! Cross-protocol capture bus.
//!
//! Every module (OSC monitor/sender, broadcast emitter, discovery listener, the
//! impairment relay …) publishes a normalized [`Frame`] here. The bus keeps a
//! bounded ring buffer so nothing is lost to a busy UI, and a pump task ships
//! batches to the front-end on `inspect://batch`. Capture is *armed* explicitly
//! — while disarmed, `push` is a single atomic load and costs nothing.
//!
//! A frame keeps its payload's bytes (secrets masked), up to [`FRAME_LIMIT`]:
//! a batch carries only the first [`HEX_LIMIT`] as a hex preview, the rest is
//! asked for by number (`payload`) and written by `export`. The ring is bounded
//! by frames ([`RING_CAPACITY`]) and by the bytes it holds ([`RING_BYTES`]);
//! the oldest frames give way to either.

use std::collections::VecDeque;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine as _;
use serde::Serialize;
use crate::host::Host;

use super::error::{EngineError, EngineResult};
use super::jobs::now_ms;

/// Frames held for export / snapshot, independent of what the UI managed to draw.
pub const RING_CAPACITY: usize = 8192;
/// Upper bound on frames handed to the webview per pump tick.
const MAX_BATCH: usize = 250;
const PUMP_INTERVAL_MS: u64 = 120;
/// Bytes of payload shown as a hex preview with each frame.
pub const HEX_LIMIT: usize = 1024;
/// Bytes of payload a frame keeps: a whole UDP datagram (64 KiB at most), an
/// ordinary HTTP body or WebSocket message; past it, the frame says how much.
pub const FRAME_LIMIT: usize = 256 << 10;
/// Bytes of payload the ring keeps in all; the oldest frames give way.
pub const RING_BYTES: usize = 64 << 20;

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
    /// A hex dump of the first [`HEX_LIMIT`] bytes, for the list's detail pane.
    pub hex: Option<String>,
    /// What happened to it: "dropped", "duplicated", "corrupted", "auto-reply"…
    pub verdict: Option<String>,
    /// Of `bytes`, how many it keeps (`payload` hands them out): all, up to
    /// [`FRAME_LIMIT`]; 0 when only the size was recorded.
    pub kept: usize,
    /// The payload as captured, secrets masked; never sent with a batch.
    #[serde(skip)]
    pub data: Option<Arc<[u8]>>,
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
            kept: 0,
            data: None,
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

    /// Record the payload: sets `bytes`, keeps them (up to [`FRAME_LIMIT`])
    /// and renders the hex preview. Secret values in use are overwritten
    /// before anything is kept.
    pub fn payload(mut self, bytes: &[u8]) -> Self {
        let masked = super::secrets::mask_bytes(bytes, &super::secrets::active());
        self.bytes = bytes.len();
        self.kept = masked.len().min(FRAME_LIMIT);
        self.hex = Some(hex_dump(&masked));
        self.data = Some(Arc::from(&masked[..self.kept]));
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
    ring: Mutex<Ring>,
}

/// The frames held, oldest first, and the payload bytes they keep.
#[derive(Default)]
struct Ring {
    frames: VecDeque<Frame>,
    held: usize,
}

impl Ring {
    fn pop_oldest(&mut self) {
        if let Some(frame) = self.frames.pop_front() {
            self.held -= frame.kept;
        }
    }
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
    /// Payload bytes the frames held keep, of [`RING_BYTES`].
    pub held: usize,
    pub held_limit: usize,
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

    /// Record a frame and return its number; a cheap no-op (`None`) while
    /// capture is disarmed.
    pub fn push(&self, mut frame: Frame) -> Option<u64> {
        if !self.is_enabled() {
            return None;
        }
        if frame.ts == 0 {
            frame.ts = now_ms();
        }
        // The number is taken under the ring's lock, so the ring is always in
        // number order. Taken before it, two sources pushing at once could land
        // out of order, and `drain`'s cursor (the last frame's number) would go
        // back and ship the same frames again on every tick.
        let mut ring = self.inner.ring.lock().unwrap();
        let seq = self.inner.seq.fetch_add(1, Ordering::Relaxed) + 1;
        frame.seq = seq;
        self.inner.total.fetch_add(1, Ordering::Relaxed);
        self.inner.bytes.fetch_add(frame.bytes as u64, Ordering::Relaxed);
        while !ring.frames.is_empty() && (ring.frames.len() >= RING_CAPACITY || ring.held + frame.kept > RING_BYTES) {
            ring.pop_oldest();
        }
        ring.held += frame.kept;
        ring.frames.push_back(frame);
        Some(seq)
    }

    pub fn clear(&self) {
        *self.inner.ring.lock().unwrap() = Ring::default();
        self.inner.total.store(0, Ordering::Relaxed);
        self.inner.bytes.store(0, Ordering::Relaxed);
        self.inner.skipped.store(0, Ordering::Relaxed);
        self.inner
            .cursor
            .store(self.inner.seq.load(Ordering::Relaxed), Ordering::Relaxed);
    }

    pub fn stats(&self) -> CaptureStats {
        let (buffered, held) = {
            let ring = self.inner.ring.lock().unwrap();
            (ring.frames.len(), ring.held)
        };
        CaptureStats {
            enabled: self.is_enabled(),
            total: self.inner.total.load(Ordering::Relaxed),
            bytes: self.inner.bytes.load(Ordering::Relaxed),
            skipped: self.inner.skipped.load(Ordering::Relaxed),
            buffered,
            capacity: RING_CAPACITY,
            held,
            held_limit: RING_BYTES,
        }
    }

    /// Newest `limit` frames, oldest first — used to repopulate the UI on mount.
    pub fn snapshot(&self, limit: usize) -> Vec<Frame> {
        let ring = self.inner.ring.lock().unwrap();
        let skip = ring.frames.len().saturating_sub(limit);
        ring.frames.iter().skip(skip).cloned().collect()
    }

    /// The frame numbered `seq`, while the ring still holds it.
    pub fn frame(&self, seq: u64) -> Option<Frame> {
        let ring = self.inner.ring.lock().unwrap();
        // In number order, so a binary search finds it.
        let at = ring.frames.binary_search_by_key(&seq, |frame| frame.seq).ok()?;
        ring.frames.get(at).cloned()
    }

    /// The bytes a frame keeps: a whole dump and the plain hex, for the detail
    /// pane and Save as signal. `inspect.frame_gone` once the ring let it go.
    pub fn payload(&self, seq: u64) -> EngineResult<Payload> {
        let frame = self.frame(seq).ok_or_else(|| EngineError::new("inspect.frame_gone").with("seq", seq))?;
        let data = frame.data.ok_or_else(|| EngineError::new("inspect.no_payload").with("seq", seq))?;
        Ok(Payload { seq, bytes: frame.bytes, kept: frame.kept, dump: hex_dump_all(&data), hex: plain_hex(&data) })
    }

    /// Take everything the UI hasn't seen, newest-capped at [`MAX_BATCH`].
    fn drain(&self) -> (Vec<Frame>, u64) {
        let cursor = self.inner.cursor.load(Ordering::Relaxed);
        let fresh: Vec<Frame> = {
            let ring = self.inner.ring.lock().unwrap();
            ring.frames.iter().filter(|f| f.seq > cursor).cloned().collect()
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

    /// Write the whole ring to the data folder. Returns the file path.
    pub fn export(&self, format: &str) -> EngineResult<String> {
        let frames = self.snapshot(RING_CAPACITY);
        if frames.is_empty() {
            return Err(EngineError::new("inspect.empty"));
        }
        let dir = export_dir();
        let failed = |path: &std::path::Path, e: &dyn std::fmt::Display| EngineError::new("file.io").with("path", path.display()).because(e);
        std::fs::create_dir_all(&dir).map_err(|e| failed(&dir, &e))?;

        let ext = if format == "txt" { "txt" } else { "jsonl" };
        let path = dir.join(format!("capture-{}.{ext}", now_ms()));
        let file = std::fs::File::create(&path).map_err(|e| failed(&path, &e))?;
        let mut out = std::io::BufWriter::new(file);
        let io = |e: std::io::Error| failed(&path, &e);

        if ext == "jsonl" {
            for f in &frames {
                // The bytes it keeps go with it, base64; plain data, so this fails only as the writing does.
                let line = Exported { frame: f, data: f.data.as_deref().map(|data| base64::engine::general_purpose::STANDARD.encode(data)) };
                serde_json::to_writer(&mut out, &line).map_err(|e| failed(&path, &e))?;
                writeln!(out).map_err(io)?;
            }
        } else {
            writeln!(out, "Signal Lab capture — {} frames", frames.len()).map_err(io)?;
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
                .map_err(io)?;
                if let Some(d) = &f.detail {
                    writeln!(out, "  {}", d.replace('\n', "\n  ")).map_err(io)?;
                }
                match (&f.data, &f.hex) {
                    (Some(data), _) => {
                        write!(out, "{}", hex_dump_all(data)).map_err(io)?;
                        if f.kept < f.bytes {
                            writeln!(out, "… {} more bytes not kept", f.bytes - f.kept).map_err(io)?;
                        }
                    }
                    (None, Some(h)) => write!(out, "{h}").map_err(io)?,
                    (None, None) => {}
                }
            }
        }
        out.flush().map_err(io)?;
        Ok(path.to_string_lossy().into_owned())
    }
}

/// A frame as `export` writes it: with the bytes it keeps.
#[derive(Serialize)]
struct Exported<'a> {
    #[serde(flatten)]
    frame: &'a Frame,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

/// What `inspect_payload` answers: the bytes a frame keeps, dumped and as plain hex.
#[derive(Clone, Debug, Serialize)]
pub struct Payload {
    pub seq: u64,
    pub bytes: usize,
    pub kept: usize,
    /// `offset  hex  |ascii|` rows, every one of them.
    pub dump: String,
    /// `48 65 6c …`: what a replay sends.
    pub hex: String,
}

fn export_dir() -> std::path::PathBuf {
    super::paths::data_dir()
}

/// Publish a frame from anywhere that holds a `Host`; its number, when capture is armed.
pub fn publish(host: &Host, frame: Frame) -> Option<u64> {
    host.capture().push(redact(frame, &super::secrets::active()))
}

/// A frame with secret values in use masked in every text it carries.
fn redact(mut frame: Frame, values: &[String]) -> Frame {
    if values.is_empty() {
        return frame;
    }
    let mask = |text: &str| super::secrets::mask(text, values);
    frame.summary = mask(&frame.summary);
    frame.detail = frame.detail.as_deref().map(mask);
    frame.remote = mask(&frame.remote);
    frame.local = mask(&frame.local);
    frame.verdict = frame.verdict.as_deref().map(mask);
    frame
}

/// Is capture armed? Lets hot paths skip building a frame at all.
pub fn armed(host: &Host) -> bool {
    host.capture().is_enabled()
}

/// Ship batches to the interface; one pump per host, for as long as the
/// returned task runs (the owner aborts it when it goes away).
pub fn spawn_pump(host: Host) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let capture = host.capture().clone();
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
            if frames.is_empty() && !idle_ticks.is_multiple_of(8) {
                continue;
            }
            host.emit(
                "inspect://batch",
                Batch {
                    frames,
                    stats: capture.stats(),
                    skipped_now,
                },
            );
        }
    })
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// `0000  48 65 6c 6c 6f 20 77 6f  72 6c 64 21   |Hello world!|`, the first
/// [`HEX_LIMIT`] bytes and a line saying how many more there are.
pub fn hex_dump(bytes: &[u8]) -> String {
    let mut out = dump_rows(&bytes[..bytes.len().min(HEX_LIMIT)]);
    if bytes.len() > HEX_LIMIT {
        out.push_str(&format!("… {} more bytes\n", bytes.len() - HEX_LIMIT));
    }
    out
}

/// Every byte, dumped as `hex_dump` does.
pub fn hex_dump_all(bytes: &[u8]) -> String {
    dump_rows(bytes)
}

/// `48 65 6c 6c 6f`: bytes as a replay takes them.
pub fn plain_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 3);
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn dump_rows(shown: &[u8]) -> String {
    let mut out = String::with_capacity(shown.len() * 4 + 16);
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

/// A budget of frames per second, lock-free: every frame while traffic is
/// light (a request and its answer a millisecond apart are both kept), at
/// most `per_second` when it is not. What a full second refused is handed
/// to the next frame allowed, to say how many it stands for.
pub struct Budget {
    window_ms: AtomicU64,
    used: AtomicU64,
    refused: AtomicU64,
    per_second: u64,
}

impl Budget {
    pub fn new(per_second: u64) -> Self {
        Budget { window_ms: AtomicU64::new(0), used: AtomicU64::new(0), refused: AtomicU64::new(0), per_second }
    }

    /// `Some(n)` when this frame may be drawn, `n` frames having been refused
    /// before it; `None` when the second's budget is spent.
    pub fn allow(&self) -> Option<u64> {
        let now = now_ms();
        let window = self.window_ms.load(Ordering::Relaxed);
        if now.saturating_sub(window) >= 1000 && self.window_ms.compare_exchange(window, now, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            self.used.store(0, Ordering::Relaxed);
        }
        if self.used.fetch_add(1, Ordering::Relaxed) < self.per_second {
            Some(self.refused.swap(0, Ordering::Relaxed))
        } else {
            self.refused.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_budget_keeps_light_traffic_whole_and_caps_a_flood() {
        let budget = Budget::new(3);
        assert_eq!([budget.allow(), budget.allow(), budget.allow()], [Some(0), Some(0), Some(0)], "three at once are all kept");
        assert_eq!((budget.allow(), budget.allow()), (None, None), "the fourth and fifth in the same second are not");
        budget.window_ms.store(0, Ordering::Relaxed);
        assert_eq!(budget.allow(), Some(2), "the next second's first frame says two were not shown");
        assert_eq!(budget.allow(), Some(0));
    }

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
        assert!(cap.export("jsonl").unwrap_err().is("inspect.empty"), "nothing to write is said, not written");
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

    /// Many sources at once (a scan's workers, both legs of a relay): every
    /// frame reaches the interface once, in number order, or is counted as skipped.
    #[test]
    fn concurrent_pushes_are_drained_once_and_in_order() {
        let cap = Capture::new();
        cap.set_enabled(true);
        let pushers: Vec<_> = (0..8)
            .map(|_| {
                let cap = cap.clone();
                std::thread::spawn(move || {
                    for _ in 0..500 {
                        cap.push(Frame::rx("tcp", "test"));
                    }
                })
            })
            .collect();
        let mut delivered = Vec::new();
        let mut skipped = 0;
        while pushers.iter().any(|pusher| !pusher.is_finished()) {
            let (batch, skipped_now) = cap.drain();
            delivered.extend(batch.iter().map(|frame| frame.seq));
            skipped += skipped_now;
        }
        for pusher in pushers {
            pusher.join().unwrap();
        }
        let (batch, skipped_now) = cap.drain();
        delivered.extend(batch.iter().map(|frame| frame.seq));
        skipped += skipped_now;
        assert!(delivered.windows(2).all(|pair| pair[0] < pair[1]), "a frame was shipped twice or out of order");
        assert_eq!(delivered.len() as u64 + skipped, 4000, "every frame is shipped or counted as skipped");
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

    #[test]
    fn a_frame_keeps_its_whole_payload_and_hands_it_out_by_number() {
        let cap = Capture::new();
        cap.set_enabled(true);
        let datagram: Vec<u8> = (0..3000u32).map(|i| (i % 251) as u8).collect();
        let seq = cap.push(Frame::rx("udp", "test").payload(&datagram)).unwrap();
        let shown = cap.snapshot(1).remove(0);
        assert_eq!((shown.bytes, shown.kept), (3000, 3000));
        assert!(shown.hex.as_deref().unwrap().ends_with("… 1976 more bytes\n"), "the batch carries a preview");
        let json = serde_json::to_value(&shown).unwrap();
        assert!(json.get("data").is_none() && json["kept"] == 3000, "the bytes never travel with a batch");
        let payload = cap.payload(seq).unwrap();
        assert_eq!(payload.hex.split(' ').count(), 3000);
        assert!(payload.hex.starts_with("00 01 02") && payload.hex.ends_with(&format!("{:02x}", 2999 % 251)));
        assert!(payload.dump.contains("\n0bb0  ") && !payload.dump.contains("more bytes"), "every row");
        assert!(cap.payload(seq + 1).unwrap_err().is("inspect.frame_gone"));
        let sized = cap.push(Frame::rx("udp", "test").size(10)).unwrap();
        assert!(cap.payload(sized).unwrap_err().is("inspect.no_payload"));
    }

    #[test]
    fn a_frame_keeps_at_most_its_limit_and_the_ring_its_budget() {
        let cap = Capture::new();
        cap.set_enabled(true);
        let big = vec![7u8; FRAME_LIMIT + 10];
        let seq = cap.push(Frame::rx("http", "test").payload(&big)).unwrap();
        let frame = cap.frame(seq).unwrap();
        assert_eq!((frame.bytes, frame.kept), (FRAME_LIMIT + 10, FRAME_LIMIT), "past the limit, the frame says how much");
        // Frames of the limit each: the ring holds what fits its budget, the oldest giving way.
        let fits = RING_BYTES / FRAME_LIMIT;
        for _ in 0..fits + 3 {
            cap.push(Frame::rx("udp", "test").payload(&big[..FRAME_LIMIT]));
        }
        let stats = cap.stats();
        assert_eq!(stats.buffered, fits);
        assert!(stats.held <= RING_BYTES && stats.held == fits * FRAME_LIMIT, "{} of {}", stats.held, stats.held_limit);
        assert!(cap.payload(seq).unwrap_err().is("inspect.frame_gone"), "the first went first");
        cap.clear();
        assert_eq!((cap.stats().buffered, cap.stats().held), (0, 0));
    }

    #[test]
    fn frames_mask_secret_values_in_use() {
        let guard = super::super::secrets::redact(vec!["hunter2-token".into()]);
        let frame = Frame::tx("http", "http")
            .remote("http://127.0.0.1/?key=hunter2-token")
            .summary("GET with hunter2-token")
            .detail("Authorization: Bearer hunter2-token")
            .payload(b"key=hunter2-token");
        let frame = redact(frame, &super::super::secrets::active());
        assert!(!frame.remote.contains("hunter2") && !frame.summary.contains("hunter2"));
        assert!(!frame.detail.as_deref().unwrap().contains("hunter2"));
        let hex = frame.hex.unwrap();
        assert!(!hex.contains("hunter") && hex.contains("|key=****"), "{hex}");
        drop(guard);
        let clean = Frame::tx("udp", "test").payload(b"hunter2-token");
        assert!(clean.hex.unwrap().contains("hunter2-token"), "after the run nothing is masked");
    }
}
