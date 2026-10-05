//! Listening sockets for *Wait* steps. A run opens one socket per distinct
//! bind address before its first step, so a reply that arrives before
//! execution reaches the wait is already queued. Each socket keeps a bounded
//! queue; a wait takes the first queued datagram its matcher accepts, so two
//! waits never match the same message. Specified in
//! `docs/develop/design-reactive.md`, section 3.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use crate::host::Host;
use tokio::net::UdpSocket;
use tokio::sync::Notify;

use super::error::{EngineError, EngineResult};
use super::inspect::{self, Frame};
use super::matching::{Datagram, Matcher};
use super::osc_codec::{decode_packet, summarize_messages};

/// Datagrams kept per socket; older ones are dropped and counted.
pub const QUEUE: usize = 1024;
/// Bytes kept per inbox (WebSocket messages are large where datagrams are not);
/// older ones are dropped and counted.
pub const QUEUE_BYTES: usize = 64 << 20;

/// The sockets of one run, by bind address.
pub type Listeners = HashMap<SocketAddr, Arc<Listener>>;

/// Where received datagrams are reported: the Inspector when hosted, nowhere in tests.
/// The Inspector frame's number comes back, so a matched wait can point at it.
pub trait FrameSink: Send + Sync {
    fn received(&self, local: SocketAddr, datagram: &Datagram) -> Option<u64>;
}

impl FrameSink for Host {
    fn received(&self, local: SocketAddr, datagram: &Datagram) -> Option<u64> {
        if !inspect::armed(self) {
            return None;
        }
        let bytes = &datagram.bytes;
        let frame = match decode_packet(bytes) {
            Ok(messages) if !messages.is_empty() => Frame::rx("osc", "experiment-wait").summary(summarize_messages(&messages)),
            _ => Frame::rx("udp", "experiment-wait").summary(inspect::ascii_preview(bytes, 96)),
        };
        inspect::publish(self, frame.local(local).remote(datagram.from).payload(bytes))
    }
}

/// Something besides the waits that answers what a run's socket receives: an
/// OSC or UDP emulator on the same port. It reports the datagram to the
/// Inspector itself (with the rule that answered) and returns the frame's
/// number; called in the receive loop, so it must not block.
pub trait Tap: Send + Sync {
    fn received(&self, socket: &Arc<UdpSocket>, datagram: &Datagram) -> Option<u64>;
}

#[cfg(test)]
pub struct NoFrames;

#[cfg(test)]
impl FrameSink for NoFrames {
    fn received(&self, _local: SocketAddr, _datagram: &Datagram) -> Option<u64> {
        None
    }
}

/// What a source of a run received, kept for its waits: a bounded queue, a
/// wake-up for waiting steps, and the error that ended the source, if any.
/// A UDP socket (`Listener`) and an MQTT subscription (`subscribe`) fill one each.
pub struct Inbox {
    datagrams: Mutex<VecDeque<Datagram>>,
    /// What `datagrams` holds, in bytes.
    held: AtomicUsize,
    dropped: AtomicU64,
    /// Wakes every waiting step when a datagram arrives or the source fails.
    arrived: Notify,
    failed: Mutex<Option<EngineError>>,
}

impl Default for Inbox {
    fn default() -> Self {
        Inbox { datagrams: Mutex::new(VecDeque::new()), held: AtomicUsize::new(0), dropped: AtomicU64::new(0), arrived: Notify::new(), failed: Mutex::new(None) }
    }
}

impl Inbox {
    pub fn push(&self, datagram: Datagram) {
        {
            let mut datagrams = self.datagrams.lock().unwrap();
            let size = datagram.bytes.len();
            while !datagrams.is_empty() && (datagrams.len() == QUEUE || self.held.load(Ordering::Relaxed) + size > QUEUE_BYTES) {
                if let Some(old) = datagrams.pop_front() {
                    self.held.fetch_sub(old.bytes.len(), Ordering::Relaxed);
                }
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            self.held.fetch_add(size, Ordering::Relaxed);
            datagrams.push_back(datagram);
        }
        self.arrived.notify_waiters();
    }

    pub fn fail(&self, error: EngineError) {
        *self.failed.lock().unwrap() = Some(error);
        self.arrived.notify_waiters();
    }

    /// Datagrams dropped because the queue was full.
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// The first datagram that arrived at or after `since` and matches, waiting
    /// up to `timeout` for one. A matched datagram is consumed.
    pub async fn wait(&self, matcher: &dyn Matcher, since: Instant, timeout: Duration) -> EngineResult<WaitOutcome> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            // Registered before looking, so an arrival between the look and the
            // sleep still wakes this wait.
            let arrived = self.arrived.notified();
            tokio::pin!(arrived);
            arrived.as_mut().enable();
            if let Some((datagram, reply)) = self.take(matcher, since) {
                return Ok(WaitOutcome::Matched(datagram, reply));
            }
            if let Some(error) = self.failed.lock().unwrap().clone() {
                return Err(error);
            }
            tokio::select! {
                _ = &mut arrived => {}
                _ = tokio::time::sleep_until(deadline) => {
                    return Ok(WaitOutcome::TimedOut { unmatched: self.arrived_since(since) });
                }
            }
        }
    }

    /// Removes and returns the first datagram since `since` that matches.
    fn take(&self, matcher: &dyn Matcher, since: Instant) -> Option<(Datagram, Value)> {
        let mut datagrams = self.datagrams.lock().unwrap();
        let (index, reply) = datagrams
            .iter()
            .enumerate()
            .filter(|(_, datagram)| datagram.at >= since)
            .find_map(|(index, datagram)| matcher.matches(datagram).map(|reply| (index, reply)))?;
        let taken = datagrams.remove(index)?;
        self.held.fetch_sub(taken.bytes.len(), Ordering::Relaxed);
        Some((taken, reply))
    }

    fn arrived_since(&self, since: Instant) -> usize {
        self.datagrams.lock().unwrap().iter().filter(|datagram| datagram.at >= since).count()
    }
}

/// What a wait ended with.
pub enum WaitOutcome {
    /// The datagram and the reply value its matcher produced.
    Matched(Datagram, Value),
    /// Nothing matched in time; `unmatched` other datagrams arrived meanwhile.
    TimedOut { unmatched: usize },
}

/// One open socket and its queue. Dropping it closes the socket.
pub struct Listener {
    local: SocketAddr,
    /// Also for sending: a step that expects a reply sends from here, so a
    /// device that answers the sender's port is heard.
    socket: Arc<UdpSocket>,
    queue: Arc<Inbox>,
    task: tokio::task::AbortHandle,
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A socket of a run that just ended closes a moment after the run: its task
/// is cancelled asynchronously. Running again right away retries briefly
/// instead of reporting the port as taken.
const RELEASE_RETRIES: u32 = 10;
const RELEASE_PAUSE: Duration = Duration::from_millis(20);

async fn bind_socket(bind: SocketAddr) -> io::Result<UdpSocket> {
    let mut attempt = 0;
    loop {
        match UdpSocket::bind(bind).await {
            Err(error) if error.kind() == io::ErrorKind::AddrInUse && attempt < RELEASE_RETRIES => {
                attempt += 1;
                tokio::time::sleep(RELEASE_PAUSE).await;
            }
            result => return result,
        }
    }
}

impl Listener {
    pub async fn arm(bind: SocketAddr, sink: Arc<dyn FrameSink>) -> EngineResult<Listener> {
        Self::arm_with(bind, sink, None).await
    }

    /// A socket whose datagrams `tap` answers as well; the waits still see every one.
    pub async fn arm_with(bind: SocketAddr, sink: Arc<dyn FrameSink>, tap: Option<Arc<dyn Tap>>) -> EngineResult<Listener> {
        // A taken port and a foreign address are causes a person can fix.
        let socket = Arc::new(bind_socket(bind).await.map_err(|error| crate::net::bind_error(&bind.to_string(), error))?);
        let local = socket.local_addr().unwrap_or(bind);
        let receiving = socket.clone();
        let queue = Arc::new(Inbox::default());
        let filled = queue.clone();
        let task = tokio::spawn(async move {
            let mut buffer = vec![0u8; 65_536];
            loop {
                match receiving.recv_from(&mut buffer).await {
                    Ok((size, from)) => {
                        let mut datagram = Datagram { bytes: buffer[..size].to_vec(), from, at: Instant::now(), topic: None, frame: None, request: None, binary: false };
                        datagram.frame = match &tap {
                            Some(tap) => tap.received(&receiving, &datagram),
                            None => sink.received(local, &datagram),
                        };
                        filled.push(datagram);
                    }
                    // An ICMP "port unreachable" for an earlier send; the socket itself is fine.
                    Err(error) if crate::net::udp_transient(&error) => continue,
                    Err(error) => {
                        filled.fail(EngineError::new("wait.receive_failed").with("target", local).because(error));
                        break;
                    }
                }
            }
        });
        Ok(Listener { local, socket, queue, task: task.abort_handle() })
    }

    /// Send from this socket: the reply to the sender's port arrives here.
    pub async fn send_to(&self, bytes: &[u8], to: SocketAddr) -> io::Result<usize> {
        self.socket.send_to(bytes, to).await
    }

    pub fn local(&self) -> SocketAddr {
        self.local
    }

    /// Datagrams dropped because the queue was full.
    pub fn dropped(&self) -> u64 {
        self.queue.dropped()
    }

    pub fn inbox(&self) -> &Inbox {
        &self.queue
    }

    /// The first datagram that arrived at or after `since` and matches, waiting
    /// up to `timeout` for one. A matched datagram is consumed.
    pub async fn wait(&self, matcher: &dyn Matcher, since: Instant, timeout: Duration) -> EngineResult<WaitOutcome> {
        self.queue.wait(matcher, since, timeout).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matching::{UdpMatcher, UdpMode};

    async fn armed() -> Listener {
        Listener::arm("127.0.0.1:0".parse().unwrap(), Arc::new(NoFrames)).await.unwrap()
    }

    async fn send(to: SocketAddr, text: &str) {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        socket.send_to(text.as_bytes(), to).await.unwrap();
    }

    fn contains(text: &str) -> UdpMatcher {
        UdpMatcher::new(UdpMode::Contains, text).unwrap()
    }

    /// Lets the receive task queue what was sent.
    async fn settle(listener: &Listener, count: usize) {
        for _ in 0..200 {
            if listener.queue.datagrams.lock().unwrap().len() >= count {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        panic!("datagrams never arrived");
    }

    #[tokio::test]
    async fn a_reply_that_beats_the_wait_is_still_matched_once() {
        let listener = armed().await;
        let since = Instant::now();
        send(listener.local(), "PONG 1").await;
        settle(&listener, 1).await;
        let WaitOutcome::Matched(datagram, reply) = listener.wait(&contains("PONG"), since, Duration::from_millis(500)).await.unwrap() else {
            panic!("the queued reply must match");
        };
        assert_eq!((datagram.bytes.as_slice(), reply["text"].as_str()), (&b"PONG 1"[..], Some("PONG 1")));
        // Consumed: a second wait does not see it again.
        let again = listener.wait(&contains("PONG"), since, Duration::from_millis(50)).await.unwrap();
        assert!(matches!(again, WaitOutcome::TimedOut { unmatched: 0 }));
    }

    #[tokio::test]
    async fn messages_from_before_the_request_do_not_count_but_are_reported() {
        let listener = armed().await;
        send(listener.local(), "stale PONG").await;
        settle(&listener, 1).await;
        let since = Instant::now();
        send(listener.local(), "noise").await;
        settle(&listener, 2).await;
        let outcome = listener.wait(&contains("PONG"), since, Duration::from_millis(60)).await.unwrap();
        assert!(matches!(outcome, WaitOutcome::TimedOut { unmatched: 1 }), "only the noise arrived since");
    }

    #[tokio::test]
    async fn a_wait_wakes_when_its_reply_arrives() {
        let listener = Arc::new(armed().await);
        let since = Instant::now();
        let waiting = {
            let listener = listener.clone();
            tokio::spawn(async move { listener.wait(&contains("late"), since, Duration::from_secs(5)).await })
        };
        tokio::time::sleep(Duration::from_millis(30)).await;
        send(listener.local(), "other").await;
        send(listener.local(), "late reply").await;
        let started = Instant::now();
        let outcome = waiting.await.unwrap().unwrap();
        assert!(matches!(outcome, WaitOutcome::Matched(..)));
        assert!(started.elapsed() < Duration::from_secs(2), "woken by the arrival, not the timeout");
    }

    #[tokio::test]
    async fn taken_ports_are_reported_and_closed_ports_can_be_bound_again() {
        let listener = armed().await;
        let local = listener.local();
        let taken = Listener::arm(local, Arc::new(NoFrames)).await.err().unwrap();
        assert_eq!((taken.code.as_str(), taken.params["target"].clone()), ("transport.address_in_use", local.to_string()));
        assert!(taken.detail.is_some());
        drop(listener);
        let mut reopened = None;
        for _ in 0..100 {
            if let Ok(listener) = Listener::arm(local, Arc::new(NoFrames)).await {
                reopened = Some(listener);
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(reopened.is_some(), "dropping the listener closes its socket");
    }

    #[tokio::test]
    async fn a_device_answering_the_senders_port_reaches_the_queue() {
        let listener = armed().await;
        // An echo device: replies to wherever the datagram came from.
        let device = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let device_address = device.local_addr().unwrap();
        tokio::spawn(async move {
            let mut buffer = [0u8; 64];
            let (size, from) = device.recv_from(&mut buffer).await.unwrap();
            device.send_to(&[b"re: ", &buffer[..size]].concat(), from).await.unwrap();
        });
        let since = Instant::now();
        listener.send_to(b"ping", device_address).await.unwrap();
        let outcome = listener.wait(&contains("re: ping"), since, Duration::from_secs(2)).await.unwrap();
        let WaitOutcome::Matched(datagram, _) = outcome else { panic!("the echo came back to the listener") };
        assert_eq!(datagram.from, device_address);
    }

    #[test]
    fn the_queue_is_bounded_and_counts_what_it_drops() {
        let queue = Inbox::default();
        let from: SocketAddr = "127.0.0.1:1".parse().unwrap();
        for index in 0..QUEUE + 6 {
            queue.push(Datagram { bytes: index.to_string().into_bytes(), from, at: Instant::now(), topic: None, frame: None, request: None, binary: false });
        }
        assert_eq!((queue.datagrams.lock().unwrap().len(), queue.dropped.load(Ordering::Relaxed)), (QUEUE, 6));
        assert_eq!(queue.datagrams.lock().unwrap().front().unwrap().bytes, b"6", "the oldest are dropped");
    }
}
