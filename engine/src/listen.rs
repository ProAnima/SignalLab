//! Listening sockets for *Wait* steps. A run opens one socket per distinct
//! bind address before its first step, so a reply that arrives before
//! execution reaches the wait is already queued. Each socket keeps a bounded
//! queue; a wait takes the first queued datagram its matcher accepts, so two
//! waits never match the same message. Specified in
//! `docs/milestone-4-reactive.md`, section 3.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
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
use super::transport::{self, Cause};

/// Datagrams kept per socket; older ones are dropped and counted.
pub const QUEUE: usize = 1024;

/// The sockets of one run, by bind address.
pub type Listeners = HashMap<SocketAddr, Arc<Listener>>;

/// Where received datagrams are reported: the Inspector when hosted, nowhere in tests.
pub trait FrameSink: Send + Sync {
    fn received(&self, local: SocketAddr, datagram: &Datagram);
}

impl FrameSink for Host {
    fn received(&self, local: SocketAddr, datagram: &Datagram) {
        if !inspect::armed(self) {
            return;
        }
        let bytes = &datagram.bytes;
        let frame = match decode_packet(bytes) {
            Ok(messages) if !messages.is_empty() => Frame::rx("osc", "experiment-wait").summary(summarize_messages(&messages)),
            _ => Frame::rx("udp", "experiment-wait").summary(inspect::ascii_preview(bytes, 96)),
        };
        inspect::publish(self, frame.local(local).remote(datagram.from).payload(bytes));
    }
}

#[cfg(test)]
pub struct NoFrames;

#[cfg(test)]
impl FrameSink for NoFrames {
    fn received(&self, _local: SocketAddr, _datagram: &Datagram) {}
}

struct Queue {
    datagrams: Mutex<VecDeque<Datagram>>,
    dropped: AtomicU64,
    /// Wakes every waiting step when a datagram arrives or the socket fails.
    arrived: Notify,
    failed: Mutex<Option<EngineError>>,
}

impl Queue {
    fn push(&self, datagram: Datagram) {
        {
            let mut datagrams = self.datagrams.lock().unwrap();
            if datagrams.len() == QUEUE {
                datagrams.pop_front();
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            datagrams.push_back(datagram);
        }
        self.arrived.notify_waiters();
    }

    fn fail(&self, error: EngineError) {
        *self.failed.lock().unwrap() = Some(error);
        self.arrived.notify_waiters();
    }

    /// Removes and returns the first datagram since `since` that matches.
    fn take(&self, matcher: &dyn Matcher, since: Instant) -> Option<(Datagram, Value)> {
        let mut datagrams = self.datagrams.lock().unwrap();
        let (index, reply) = datagrams
            .iter()
            .enumerate()
            .filter(|(_, datagram)| datagram.at >= since)
            .find_map(|(index, datagram)| matcher.matches(datagram).map(|reply| (index, reply)))?;
        datagrams.remove(index).map(|datagram| (datagram, reply))
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
    queue: Arc<Queue>,
    task: tokio::task::AbortHandle,
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Opening the socket failed: a taken port and a foreign address are causes
/// a person can fix, anything else carries the system's wording.
fn bind_error(bind: SocketAddr, error: io::Error) -> EngineError {
    let target = bind.to_string();
    match transport::of_io(&error) {
        cause @ (Cause::AddressInUse | Cause::AddressUnavailable | Cause::Denied) => cause.error(&target),
        _ => EngineError::new("wait.bind_failed").with("target", target),
    }
    .because(error)
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
        let socket = bind_socket(bind).await.map_err(|error| bind_error(bind, error))?;
        let local = socket.local_addr().unwrap_or(bind);
        let queue = Arc::new(Queue {
            datagrams: Mutex::new(VecDeque::new()),
            dropped: AtomicU64::new(0),
            arrived: Notify::new(),
            failed: Mutex::new(None),
        });
        let filled = queue.clone();
        let task = tokio::spawn(async move {
            let mut buffer = vec![0u8; 65_536];
            loop {
                match socket.recv_from(&mut buffer).await {
                    Ok((size, from)) => {
                        let datagram = Datagram { bytes: buffer[..size].to_vec(), from, at: Instant::now() };
                        sink.received(local, &datagram);
                        filled.push(datagram);
                    }
                    // Windows reports an ICMP "port unreachable" for an earlier
                    // send as a receive error; the socket itself is fine.
                    Err(error) if error.kind() == io::ErrorKind::ConnectionReset => continue,
                    Err(error) => {
                        filled.fail(EngineError::new("wait.receive_failed").with("target", local).because(error));
                        break;
                    }
                }
            }
        });
        Ok(Listener { local, queue, task: task.abort_handle() })
    }

    pub fn local(&self) -> SocketAddr {
        self.local
    }

    /// Datagrams dropped because the queue was full.
    pub fn dropped(&self) -> u64 {
        self.queue.dropped.load(Ordering::Relaxed)
    }

    /// The first datagram that arrived at or after `since` and matches, waiting
    /// up to `timeout` for one. A matched datagram is consumed.
    pub async fn wait(&self, matcher: &dyn Matcher, since: Instant, timeout: Duration) -> EngineResult<WaitOutcome> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            // Registered before looking, so an arrival between the look and the
            // sleep still wakes this wait.
            let arrived = self.queue.arrived.notified();
            tokio::pin!(arrived);
            arrived.as_mut().enable();
            if let Some((datagram, reply)) = self.queue.take(matcher, since) {
                return Ok(WaitOutcome::Matched(datagram, reply));
            }
            if let Some(error) = self.queue.failed.lock().unwrap().clone() {
                return Err(error);
            }
            tokio::select! {
                _ = &mut arrived => {}
                _ = tokio::time::sleep_until(deadline) => {
                    return Ok(WaitOutcome::TimedOut { unmatched: self.queue.arrived_since(since) });
                }
            }
        }
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

    #[test]
    fn the_queue_is_bounded_and_counts_what_it_drops() {
        let queue = Queue { datagrams: Mutex::new(VecDeque::new()), dropped: AtomicU64::new(0), arrived: Notify::new(), failed: Mutex::new(None) };
        let from: SocketAddr = "127.0.0.1:1".parse().unwrap();
        for index in 0..QUEUE + 6 {
            queue.push(Datagram { bytes: index.to_string().into_bytes(), from, at: Instant::now() });
        }
        assert_eq!((queue.datagrams.lock().unwrap().len(), queue.dropped.load(Ordering::Relaxed)), (QUEUE, 6));
        assert_eq!(queue.datagrams.lock().unwrap().front().unwrap().bytes, b"6", "the oldest are dropped");
    }
}
