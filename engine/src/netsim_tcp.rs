//! The impairment relay for TCP. Clients connect to its listen port, and each
//! connection is joined to one of its own to the target. Each stream —
//! client to target, target to client — is impaired chunk by chunk as it is
//! read: delayed by the latency and its jitter, never ahead of the chunk
//! before it (a stream keeps its order); held to the bandwidth limit, past a
//! second of queue by reading no more, so the sender slows down as on a slow
//! link — nothing is dropped; reset (both sides get a reset); or left
//! half-open (nothing more goes either way, and neither side is told).
//! Offline, nothing flows and a new connection waits for the target; it all
//! resumes when the profile does. Every decision draws from the relay's seed
//! per connection and direction, so the same seed and the same traffic meet
//! the same fate.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Notify};
use tokio::task::JoinSet;

use crate::host::Host;

use super::error::{EngineError, EngineResult};
use super::inspect::{self, describe_payload, Frame, Gate};
use super::net;
use super::netsim::{ImpairProfile, Relay, Stats};
use super::template::Rng;

/// A stream queued for the bandwidth limit past this holds its reader back.
const MAX_QUEUE: Duration = Duration::from_secs(1);
/// Chunks of a stream on their way (delayed) at once; more hold the reader back.
const MAX_CHUNKS: usize = 256;
/// The most one read takes: one chunk.
const CHUNK: usize = 16 * 1024;
/// Mixed into the seed, so a stream's draws never repeat a datagram's or a template's.
const TCP_STREAM: u64 = 0x7463_705f_6c65_6701;

/// Bind the port clients connect to.
pub(crate) async fn open(listen: SocketAddr, listen_text: &str) -> EngineResult<TcpListener> {
    TcpListener::bind(listen).await.map_err(|error| net::bind_error(listen_text, error))
}

/// A chunk's fate, decided when it is read.
#[derive(Debug, PartialEq)]
enum Fate {
    /// Written at `due`; the reader waits `hold` before reading more (the link's queue is full).
    Deliver { due: Instant, hold: Duration },
    Reset,
    Stall,
}

/// One direction of one connection: its seed and name, how many chunks it
/// decided, when its bandwidth-limited link is free, when its last chunk is due.
struct Leg {
    seed: u64,
    name: String,
    chunks: u64,
    link_free: Option<Instant>,
    last_due: Option<Instant>,
}

impl Leg {
    fn new(seed: u64, name: String) -> Self {
        Leg { seed: seed ^ TCP_STREAM, name, chunks: 0, link_free: None, last_due: None }
    }

    /// Each chunk draws from a stream of its own, so how many draws one takes
    /// never moves the next one's.
    fn decide(&mut self, profile: &ImpairProfile, len: usize, now: Instant) -> Fate {
        let mut rng = Rng::for_node(self.seed, &self.name, self.chunks);
        self.chunks += 1;
        if profile.reset > 0.0 && rng.unit() < profile.reset {
            return Fate::Reset;
        }
        if profile.stall > 0.0 && rng.unit() < profile.stall {
            return Fate::Stall;
        }
        // The link sends a chunk after the ones before it, at the rate.
        let (mut sent, mut hold) = (now, Duration::ZERO);
        if profile.rate_kbps > 0.0 {
            let start = self.link_free.filter(|free| *free > now).unwrap_or(now);
            hold = (start - now).saturating_sub(MAX_QUEUE);
            sent = start + Duration::from_secs_f64(len as f64 * 8.0 / (profile.rate_kbps * 1000.0));
            self.link_free = Some(sent);
        } else {
            self.link_free = None;
        }
        let jitter = if profile.jitter_ms > 0.0 { rng.unit() * profile.jitter_ms } else { 0.0 };
        // Never before the chunk ahead of it: a stream arrives in order, late or not.
        let due = (sent + Duration::from_secs_f64((profile.latency_ms + jitter) / 1000.0)).max(self.last_due.unwrap_or(now));
        self.last_due = Some(due);
        Fate::Deliver { due, hold }
    }
}

const OPEN: u8 = 0;
const RESET: u8 = 1;
const STALLED: u8 = 2;

/// How a connection ended, if it did: shared by its two streams, so the one
/// that decides tells the other.
#[derive(Default)]
struct Ending {
    state: AtomicU8,
    told: Notify,
}

impl Ending {
    /// The first ending decided counts; returns whether it was this one.
    fn end(&self, how: u8) -> bool {
        let first = self.state.compare_exchange(OPEN, how, Ordering::SeqCst, Ordering::SeqCst).is_ok();
        self.told.notify_waiters();
        first
    }

    fn how(&self) -> u8 {
        self.state.load(Ordering::SeqCst)
    }

    async fn ended(&self) {
        loop {
            let told = self.told.notified();
            if self.how() != OPEN {
                return;
            }
            told.await;
        }
    }
}

/// What a chunk is reported to the Inspector with: the relay's listen address
/// as the frame's local side, where the chunk was going as its peer, and the
/// leg after the fate in the verdict.
#[derive(Clone)]
struct Tap {
    host: Host,
    job_id: u64,
    leg: &'static str,
    dir: &'static str,
    local: String,
    peer: String,
    gate: Arc<Gate>,
}

impl Tap {
    fn armed(&self) -> bool {
        inspect::armed(&self.host) && self.gate.allow()
    }

    fn record(&self, bytes: &[u8], verdict: String) {
        let (_, summary, detail) = describe_payload(bytes);
        let mut frame = Frame::new("tcp", self.dir, "netsim").job(self.job_id).local(&self.local).remote(&self.peer).payload(bytes).summary(summary).verdict(format!("{verdict} · {}", self.leg));
        if let Some(detail) = detail {
            frame = frame.detail(detail);
        }
        inspect::publish(&self.host, self.gate.mark(frame));
    }
}

/// A chunk on its way: its bytes, when it is due, the phase that decided it.
struct Pending {
    bytes: Vec<u8>,
    due: Instant,
    delay: Duration,
    phase: Arc<Stats>,
    capture: bool,
}

/// Read one stream chunk by chunk and decide each; what goes through is queued
/// for the writer. Returns at the end of the stream, or when the connection ends.
async fn read(relay: &Relay, leg: &mut Leg, from: &mut OwnedReadHalf, queue: mpsc::Sender<Pending>, ending: &Ending, tap: &Tap) {
    let mut buffer = vec![0u8; CHUNK];
    loop {
        relay.until_online().await;
        let size = match from.read(&mut buffer).await {
            Ok(0) | Err(_) => return,
            Ok(size) => size,
        };
        let (profile, phase) = relay.now();
        relay.count(&phase, |stats| &stats.received, 1);
        let capture = tap.armed();
        let now = Instant::now();
        match leg.decide(&profile, size, now) {
            Fate::Reset => {
                if ending.end(RESET) {
                    relay.count(&phase, |stats| &stats.reset, 1);
                }
                if capture {
                    tap.record(&buffer[..size], "reset".into());
                }
                return;
            }
            Fate::Stall => {
                if ending.end(STALLED) {
                    relay.count(&phase, |stats| &stats.stalled, 1);
                }
                if capture {
                    tap.record(&buffer[..size], "half-open".into());
                }
                return;
            }
            Fate::Deliver { due, hold } => {
                let pending = Pending { bytes: buffer[..size].to_vec(), due, delay: due.saturating_duration_since(now), phase: phase.clone(), capture };
                if queue.send(pending).await.is_err() {
                    return;
                }
                if !hold.is_zero() {
                    relay.count(&phase, |stats| &stats.throttled, 1);
                    tokio::time::sleep(hold).await;
                }
            }
        }
    }
}

/// Write what the reader queued, each chunk when it is due; the end of the
/// stream is passed on once everything before it went.
async fn write(relay: &Relay, to: &mut OwnedWriteHalf, queue: &mut mpsc::Receiver<Pending>, tap: &Tap) {
    while let Some(chunk) = queue.recv().await {
        tokio::time::sleep_until(chunk.due.into()).await;
        relay.until_online().await;
        if to.write_all(&chunk.bytes).await.is_err() {
            return;
        }
        relay.count(&chunk.phase, |stats| &stats.forwarded, 1);
        relay.count(&chunk.phase, |stats| &stats.bytes, chunk.bytes.len() as u64);
        if chunk.capture {
            tap.record(&chunk.bytes, format!("forwarded +{:.0}ms", chunk.delay.as_secs_f64() * 1000.0));
        }
    }
    let _ = to.shutdown().await;
}

/// One stream, both its reader and its writer, until it ends — or the
/// connection does: reset, both sockets close with a reset; left half-open,
/// both stay open, untouched, until the relay goes.
async fn stream(relay: &Relay, mut leg: Leg, mut from: OwnedReadHalf, mut to: OwnedWriteHalf, ending: &Ending, tap: &Tap) {
    let (queue, mut queued) = mpsc::channel(MAX_CHUNKS);
    tokio::select! {
        _ = async { tokio::join!(read(relay, &mut leg, &mut from, queue, ending, tap), write(relay, &mut to, &mut queued, tap)) } => {}
        _ = ending.ended() => {}
    }
    match ending.how() {
        RESET => {
            // Closed with a linger of nothing: a reset, not a goodbye.
            let _ = from.as_ref().set_zero_linger();
            let _ = to.as_ref().set_zero_linger();
        }
        STALLED => std::future::pending::<()>().await,
        _ => {}
    }
}

/// One client's connection: joined to the target (once the relay is online),
/// both streams impaired until both end. A target that refuses is passed on
/// as a reset.
#[allow(clippy::too_many_arguments)]
async fn connection(relay: Arc<Relay>, host: Host, job_id: u64, name: String, n: u64, client: TcpStream, peer: SocketAddr, gate: Arc<Gate>) {
    let (_, phase) = relay.now();
    relay.count(&phase, |stats| &stats.connections, 1);
    relay.until_online().await;
    let target = match TcpStream::connect(relay.target).await {
        Ok(target) => target,
        Err(_) => {
            let _ = client.set_zero_linger();
            return;
        }
    };
    let _ = (client.set_nodelay(true), target.set_nodelay(true));
    let tap = |leg: &'static str, dir: &'static str, peer: String| Tap { host: host.clone(), job_id, leg, dir, local: relay.listen.to_string(), peer, gate: gate.clone() };
    let (to_target, to_client) = (tap("client→target", "tx", relay.target.to_string()), tap("target→client", "rx", peer.to_string()));
    let (client_read, client_write) = client.into_split();
    let (target_read, target_write) = target.into_split();
    let ending = Ending::default();
    let outward = Leg::new(relay.seed, format!("{name}:{n}:client"));
    let inward = Leg::new(relay.seed, format!("{name}:{n}:target"));
    tokio::join!(
        stream(&relay, outward, client_read, target_write, &ending, &to_target),
        stream(&relay, inward, target_read, client_write, &ending, &to_client),
    );
}

/// Accepting fails now and then for one connection only (it was reset before it was taken).
fn accept_transient(error: &std::io::Error) -> bool {
    matches!(error.kind(), std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::Interrupted)
}

impl Relay {
    /// Take connections until the port cannot take any more; the error says
    /// why. Every connection is a child of the future: dropping it closes them.
    pub(crate) async fn serve_tcp(self: Arc<Self>, host: Host, job_id: u64, name: String, listener: TcpListener) -> EngineError {
        let gate = Arc::new(Gate::new(25));
        let mut connections = JoinSet::new();
        let mut n = 0u64;
        loop {
            match listener.accept().await {
                Ok((client, peer)) => {
                    n += 1;
                    while connections.try_join_next().is_some() {}
                    connections.spawn(connection(self.clone(), host.clone(), job_id, name.clone(), n, client, peer, gate.clone()));
                }
                Err(error) if accept_transient(&error) => continue,
                Err(error) => return EngineError::new("wait.receive_failed").with("target", self.listen).because(error),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Recorder;
    use crate::inspect::Capture;
    use crate::netsim::RelayProtocol;

    fn calm() -> ImpairProfile {
        ImpairProfile::default()
    }

    #[test]
    fn a_stream_keeps_its_order_whatever_its_jitter() {
        let mut leg = Leg::new(5, "test".into());
        let now = Instant::now();
        let profile = ImpairProfile { latency_ms: 20.0, jitter_ms: 200.0, ..calm() };
        let dues: Vec<Instant> = (0..200)
            .map(|n| match leg.decide(&profile, 10, now + Duration::from_millis(n)) {
                Fate::Deliver { due, hold } => {
                    assert!(hold.is_zero());
                    due
                }
                other => panic!("{other:?}"),
            })
            .collect();
        assert!(dues.windows(2).all(|pair| pair[0] <= pair[1]), "never ahead of the chunk before");
        assert!(dues.windows(2).any(|pair| pair[0] == pair[1]), "a quick chunk waits for a slow one");
        assert!(dues[0] >= now + Duration::from_millis(20));
    }

    #[test]
    fn a_bandwidth_limit_queues_a_stream_and_holds_its_reader_past_a_second() {
        // 1000 bytes at 80 kbps take 100 ms each: the eleventh starts 1 s late, the twelfth holds the reader.
        let mut leg = Leg::new(1, "test".into());
        let now = Instant::now();
        let profile = ImpairProfile { rate_kbps: 80.0, ..calm() };
        let fates: Vec<(u128, u128)> = (0..13)
            .map(|_| match leg.decide(&profile, 1000, now) {
                Fate::Deliver { due, hold } => ((due - now).as_millis(), hold.as_millis()),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(&fates[..3], [(100, 0), (200, 0), (300, 0)]);
        assert_eq!(fates[10], (1100, 0), "a second of queue: still read");
        assert_eq!(fates[11], (1200, 100), "past it the reader waits, nothing is dropped");
    }

    #[test]
    fn resets_and_stalls_are_drawn_from_the_seed() {
        let profile = ImpairProfile { reset: 0.05, stall: 0.05, ..calm() };
        let fates = |seed: u64| {
            let mut leg = Leg::new(seed, "test".into());
            let now = Instant::now();
            (0..400).map(|_| match leg.decide(&profile, 10, now) {
                Fate::Reset => 'r',
                Fate::Stall => 's',
                Fate::Deliver { .. } => '.',
            }).collect::<String>()
        };
        let first = fates(3);
        assert_eq!(first, fates(3), "the same seed, the same fates");
        assert_ne!(first, fates(4));
        let (resets, stalls) = (first.matches('r').count(), first.matches('s').count());
        assert!((8..=35).contains(&resets) && (8..=35).contains(&stalls), "about 5 % each: {resets} resets, {stalls} stalls");
    }

    /// An echo server on loopback: what arrives on a connection goes back on it.
    async fn echo() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buffer = [0u8; 4096];
                    while let Ok(size) = stream.read(&mut buffer).await {
                        if size == 0 || stream.write_all(&buffer[..size]).await.is_err() {
                            return;
                        }
                    }
                });
            }
        });
        address
    }

    async fn relay(target: SocketAddr, profile: ImpairProfile) -> (Arc<Relay>, SocketAddr, tokio::task::JoinHandle<EngineError>) {
        let listener = open("127.0.0.1:0".parse().unwrap(), "127.0.0.1:0").await.unwrap();
        let listen = listener.local_addr().unwrap();
        let relay = Relay::new(listen, target, RelayProtocol::Tcp, profile, 9);
        let serving = tokio::spawn(relay.clone().serve_tcp(Host::new(Recorder::new(), Capture::new()), 0, "test".into(), listener));
        (relay, listen, serving)
    }

    async fn read_some(stream: &mut TcpStream, within: Duration) -> Option<std::io::Result<Vec<u8>>> {
        let mut buffer = vec![0u8; 65_536];
        tokio::time::timeout(within, stream.read(&mut buffer)).await.ok().map(|read| read.map(|size| buffer[..size].to_vec()))
    }

    #[tokio::test]
    async fn latency_delays_each_way_and_the_stream_arrives_whole_and_in_order() {
        let (relay, listen, serving) = relay(echo().await, ImpairProfile { latency_ms: 80.0, jitter_ms: 30.0, ..calm() }).await;
        let mut client = TcpStream::connect(listen).await.unwrap();
        let began = Instant::now();
        let sent: Vec<u8> = (0..60u8).flat_map(|n| format!("{n:02};").into_bytes()).collect();
        for piece in sent.chunks(3) {
            client.write_all(piece).await.unwrap();
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        let mut back = Vec::new();
        while back.len() < sent.len() {
            back.extend(read_some(&mut client, Duration::from_secs(3)).await.expect("the echo came back").unwrap());
        }
        assert_eq!(back, sent, "every byte, in order");
        assert!(began.elapsed() >= Duration::from_millis(160), "80 ms out and 80 ms back at least: {:?}", began.elapsed());
        let counts = relay.counts();
        assert_eq!((counts.connections, counts.reset, counts.stalled), (1, 0, 0));
        assert!(counts.forwarded >= 2 && counts.bytes == 2 * sent.len() as u64, "{counts:?}");
        serving.abort();
    }

    #[tokio::test]
    async fn a_bandwidth_limit_slows_a_stream_without_losing_a_byte() {
        let (_, listen, serving) = relay(echo().await, ImpairProfile { rate_kbps: 64.0, ..calm() }).await;
        let mut client = TcpStream::connect(listen).await.unwrap();
        // 4 KiB out and back at 8 KB/s each way: half a second at least.
        let sent = vec![7u8; 4096];
        let began = Instant::now();
        client.write_all(&sent).await.unwrap();
        let mut back = Vec::new();
        while back.len() < sent.len() {
            back.extend(read_some(&mut client, Duration::from_secs(5)).await.expect("it came back").unwrap());
        }
        assert_eq!(back, sent);
        assert!(began.elapsed() >= Duration::from_millis(450), "{:?}", began.elapsed());
        serving.abort();
    }

    #[tokio::test]
    async fn a_reset_reaches_the_client_and_is_counted() {
        let (relay, listen, serving) = relay(echo().await, ImpairProfile { reset: 1.0, ..calm() }).await;
        let mut client = TcpStream::connect(listen).await.unwrap();
        client.write_all(b"hello").await.unwrap();
        let read = read_some(&mut client, Duration::from_secs(2)).await.expect("the relay answered at once");
        assert!(read.is_err() || read.as_ref().is_ok_and(Vec::is_empty), "a reset, not an echo: {read:?}");
        assert_eq!((relay.counts().connections, relay.counts().reset, relay.counts().forwarded), (1, 1, 0));
        serving.abort();
    }

    #[tokio::test]
    async fn a_half_open_connection_says_nothing_and_stays_open() {
        let (relay, listen, serving) = relay(echo().await, ImpairProfile { stall: 1.0, ..calm() }).await;
        let mut client = TcpStream::connect(listen).await.unwrap();
        client.write_all(b"anyone?").await.unwrap();
        assert!(read_some(&mut client, Duration::from_millis(400)).await.is_none(), "no echo, no end, no reset");
        assert!(client.write_all(b"still there").await.is_ok(), "and the client may go on writing");
        assert_eq!((relay.counts().stalled, relay.counts().forwarded), (1, 0));
        // The relay going closes what it held.
        serving.abort();
        let closed = read_some(&mut client, Duration::from_secs(2)).await.expect("closed with the relay");
        assert!(closed.is_err() || closed.is_ok_and(|bytes| bytes.is_empty()));
    }

    #[tokio::test]
    async fn offline_holds_a_connection_until_the_profile_comes_back() {
        let (relay, listen, serving) = relay(echo().await, ImpairProfile { offline: true, ..calm() }).await;
        let mut client = TcpStream::connect(listen).await.unwrap();
        client.write_all(b"wait for me").await.unwrap();
        assert!(read_some(&mut client, Duration::from_millis(300)).await.is_none(), "nothing flows while offline");
        relay.set(calm());
        let back = read_some(&mut client, Duration::from_secs(2)).await.expect("it flows again").unwrap();
        assert_eq!(back, b"wait for me");
        let summary = relay.summary(None);
        assert_eq!(summary.phases.iter().map(|phase| phase.profile.as_str()).collect::<Vec<_>>(), ["offline", "clean"]);
        serving.abort();
    }

    /// A relayed chunk in the Inspector names the relay's listen address, where
    /// the chunk was going, and the leg after its fate.
    #[tokio::test]
    async fn relayed_chunks_name_the_relay_and_where_each_was_going() {
        // A target that answers after a pause: past the relay's sampling gate.
        let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_address = target.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = target.accept().await.unwrap();
            let mut buffer = [0u8; 64];
            let _ = stream.read(&mut buffer).await;
            tokio::time::sleep(Duration::from_millis(60)).await;
            let _ = stream.write_all(b"pong").await;
            tokio::time::sleep(Duration::from_secs(2)).await;
        });
        let capture = Capture::new();
        capture.set_enabled(true);
        let listener = open("127.0.0.1:0".parse().unwrap(), "127.0.0.1:0").await.unwrap();
        let listen = listener.local_addr().unwrap();
        let relay = Relay::new(listen, target_address, RelayProtocol::Tcp, calm(), 9);
        let serving = tokio::spawn(relay.clone().serve_tcp(Host::new(Recorder::new(), capture.clone()), 0, "test".into(), listener));
        let mut client = TcpStream::connect(listen).await.unwrap();
        client.write_all(b"ping").await.unwrap();
        assert_eq!(read_some(&mut client, Duration::from_secs(3)).await.expect("the answer").unwrap(), b"pong");
        tokio::time::sleep(Duration::from_millis(50)).await;

        let frames = capture.snapshot(16);
        let out = frames.iter().find(|frame| frame.dir == "tx").expect("the chunk to the target");
        assert_eq!((out.proto.as_str(), out.local.clone(), out.remote.clone()), ("tcp", listen.to_string(), target_address.to_string()));
        assert!(out.verdict.as_deref().is_some_and(|verdict| verdict.starts_with("forwarded +") && verdict.ends_with(" · client→target")), "{:?}", out.verdict);
        let back = frames.iter().find(|frame| frame.dir == "rx").expect("the chunk to the client");
        assert_eq!((back.local.clone(), back.remote.clone()), (listen.to_string(), client.local_addr().unwrap().to_string()));
        assert!(back.verdict.as_deref().is_some_and(|verdict| verdict.ends_with(" · target→client")), "{:?}", back.verdict);
        serving.abort();
    }

    #[tokio::test]
    async fn a_target_that_refuses_is_passed_on_as_a_reset() {
        let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
        let (_, listen, serving) = relay(closed, calm()).await;
        let mut client = TcpStream::connect(listen).await.unwrap();
        let read = read_some(&mut client, Duration::from_secs(5)).await.expect("the relay hung up");
        assert!(read.is_err() || read.is_ok_and(|bytes| bytes.is_empty()));
        serving.abort();
    }
}
