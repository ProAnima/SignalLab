//! Engine events to browsers. Each event is serialized once and fanned out to
//! every connected page over `/api/events`. A page that falls behind is told
//! how many events it missed (`server://lagged`) instead of silently losing
//! them — the same rule the desktop app follows for high-rate data.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use serde_json::{json, Value};
use signal_lab_engine::EventSink;
use tokio::sync::{broadcast, watch};

/// Events buffered per page before it counts as behind.
pub const BACKLOG: usize = 4096;
const PING_EVERY: Duration = Duration::from_secs(20);

/// The engine's event sink in the server.
pub struct Broadcaster {
    sender: broadcast::Sender<Arc<str>>,
}

impl Broadcaster {
    pub fn new() -> (Arc<Self>, broadcast::Sender<Arc<str>>) {
        let (sender, _) = broadcast::channel(BACKLOG);
        (Arc::new(Broadcaster { sender: sender.clone() }), sender)
    }
}

impl EventSink for Broadcaster {
    fn emit(&self, event: &str, payload: Value) {
        // No page listening is not an error.
        let _ = self.sender.send(Arc::from(json!({ "event": event, "payload": payload }).to_string()));
    }
}

/// Stream events to one page until it leaves, stops answering, or the server stops.
pub async fn stream(mut socket: WebSocket, mut events: broadcast::Receiver<Arc<str>>, mut stopping: watch::Receiver<bool>) {
    let mut ping = tokio::time::interval(PING_EVERY);
    ping.tick().await;
    loop {
        tokio::select! {
            received = events.recv() => {
                let text = match received {
                    Ok(text) => text.to_string(),
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "a page fell behind; told it how many events it missed");
                        json!({ "event": "server://lagged", "payload": { "skipped": skipped } }).to_string()
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                };
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => match incoming {
                // Pages only listen; anything but a close or a pong is ignored.
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                Some(Ok(_)) => {}
            },
            _ = ping.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
            _ = stopping.changed() => {
                let _ = socket.send(Message::Close(None)).await;
                break;
            }
        }
    }
}
