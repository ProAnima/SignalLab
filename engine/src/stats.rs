//! Anonymous usage statistics for the studio's hub, sent by the desktop app
//! only (never a server, never the command line): `launch` once per start, then
//! `heartbeat` every five minutes while the app runs — the hub counts who is
//! online by it. Each request is `POST {hub}/v1/signal-lab/events` with the
//! install's random id (the one the update check sends as `X-Install-Id`), the
//! version, and the system and architecture in the updater's own words, so the
//! hub takes both for one install. Nothing else: no address, no host or user
//! name, nothing about what is sent or opened.
//!
//! The person turns it off in About (*Send anonymous usage statistics*); the
//! interface hands the setting and the id to the shell, which calls
//! [`Stats::configure`]. The shell starts [`run`] only when
//! `hub::automatic_here` says so — a release build without
//! `SIGNALLAB_NO_UPDATE_CHECK` — so development and the end-to-end tour never
//! reach the hub. A `launch` the hub did not take is sent again at the next
//! tick; a busy hub (429) is asked again at the next tick; a failure is a debug
//! line, never something the person sees.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::watch;

use crate::hub;

/// How often a running app says it is still there.
pub const BEAT: Duration = Duration::from_secs(5 * 60);
/// How long one request may take.
pub const TIMEOUT: Duration = Duration::from_secs(20);
/// The release channel the app is on: the one its update check asks.
pub const CHANNEL: &str = "stable";

/// Where the events go: Signal Lab's `events` on the hub.
pub fn endpoint() -> String {
    hub::endpoint("events")
}

/// The system as the updater names it in `{{target}}`.
pub fn os() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        std::env::consts::OS
    }
}

/// The processor as the updater names it in `{{arch}}`.
pub fn arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86" => "i686",
        "arm" => "armv7",
        other => other,
    }
}

/// What one event says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Launch,
    Heartbeat,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Launch => "launch",
            Kind::Heartbeat => "heartbeat",
        }
    }
}

/// The body of one request: the install, the app, and one event.
pub fn payload(install_id: &str, version: &str, kind: Kind) -> Value {
    json!({
        "install_id": install_id,
        "version": version,
        "os": os(),
        "arch": arch(),
        "channel": CHANNEL,
        "events": [{ "kind": kind.as_str() }],
    })
}

/// How a request ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The hub took it (2xx).
    Accepted,
    /// The hub will not take it, now or later (4xx but 429): sending it again is pointless.
    Refused,
    /// The hub asks to wait (429).
    Busy,
    /// No answer, or the hub failed (5xx): it may work next time.
    Failed,
}

impl Outcome {
    pub fn of_status(status: u16) -> Outcome {
        match status {
            200..=299 => Outcome::Accepted,
            429 => Outcome::Busy,
            400..=499 => Outcome::Refused,
            _ => Outcome::Failed,
        }
    }
}

/// Which event comes next: `launch` until the hub has taken one (or refused it
/// for good), `heartbeat` after.
#[derive(Debug, Default)]
pub struct Sequence {
    launched: bool,
}

impl Sequence {
    pub fn next(&self) -> Kind {
        if self.launched { Kind::Heartbeat } else { Kind::Launch }
    }

    pub fn record(&mut self, kind: Kind, outcome: Outcome) {
        if kind == Kind::Launch && matches!(outcome, Outcome::Accepted | Outcome::Refused) {
            self.launched = true;
        }
    }
}

/// An install id the hub takes: 8 to 64 letters, digits and dashes (a UUID fits).
pub fn valid_install(id: &str) -> bool {
    (8..=64).contains(&id.len()) && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/// What the interface said: the install's id (none where it cannot keep one)
/// and whether the person lets the app send.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Settings {
    install_id: Option<String>,
    enabled: bool,
    configured: bool,
}

/// The setting, shared between the shell's command and the sender.
#[derive(Debug)]
pub struct Stats {
    settings: watch::Sender<Settings>,
}

impl Default for Stats {
    fn default() -> Self {
        Stats { settings: watch::Sender::new(Settings::default()) }
    }
}

impl Stats {
    /// The interface's setting, at start and whenever it changes; read at the next tick.
    /// An id the hub would not take counts as none.
    pub fn configure(&self, install_id: Option<&str>, enabled: bool) {
        let install_id = install_id.map(str::trim).filter(|id| valid_install(id)).map(str::to_string);
        self.settings.send_replace(Settings { install_id, enabled, configured: true });
    }

    /// The id to send as, when sending is allowed and there is one.
    pub fn sending_as(&self) -> Option<String> {
        let settings = self.settings.borrow();
        if settings.enabled { settings.install_id.clone() } else { None }
    }
}

/// Sends the events to `url` while the app runs: waits for the interface's
/// first word, then one event per `beat` while it lets the app send. Never ends
/// on its own; the shell drops it with the app.
pub async fn run(stats: Arc<Stats>, url: String, version: String, beat: Duration) {
    let client = match reqwest::Client::builder().timeout(TIMEOUT).user_agent(concat!("SignalLab/", env!("CARGO_PKG_VERSION"))).build() {
        Ok(client) => client,
        Err(error) => {
            tracing::debug!(%error, "usage statistics off: no HTTP client");
            return;
        }
    };
    let mut settings = stats.settings.subscribe();
    if settings.wait_for(|settings| settings.configured).await.is_err() {
        return;
    }
    let mut sequence = Sequence::default();
    loop {
        if let Some(install_id) = stats.sending_as() {
            let kind = sequence.next();
            let outcome = match client.post(&url).json(&payload(&install_id, &version, kind)).send().await {
                Ok(response) => Outcome::of_status(response.status().as_u16()),
                Err(error) => {
                    tracing::debug!(%error, kind = kind.as_str(), "usage statistics not sent");
                    Outcome::Failed
                }
            };
            if outcome != Outcome::Accepted {
                tracing::debug!(?outcome, kind = kind.as_str(), "usage statistics not taken");
            }
            sequence.record(kind, outcome);
        }
        tokio::time::sleep(beat).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "0f8b3c2e-7d41-4a8e-9c3b-5e2d1a6f7b90";

    #[test]
    fn a_request_says_the_install_the_app_and_one_event_and_nothing_else() {
        let body = payload(ID, "1.2.3", Kind::Launch);
        let object = body.as_object().unwrap();
        let mut keys: Vec<_> = object.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["arch", "channel", "events", "install_id", "os", "version"]);
        assert_eq!(body["install_id"], ID);
        assert_eq!(body["version"], "1.2.3");
        assert_eq!(body["channel"], "stable");
        assert_eq!(body["events"], json!([{ "kind": "launch" }]));
        assert_eq!(payload(ID, "1.2.3", Kind::Heartbeat)["events"], json!([{ "kind": "heartbeat" }]));
    }

    #[test]
    fn the_system_and_processor_are_named_as_the_updater_names_them() {
        // tauri-plugin-updater's {{target}} and {{arch}}: the hub merges both into one install.
        assert!(["windows", "linux", "darwin"].contains(&os()), "{}", os());
        assert!(["x86_64", "aarch64", "i686", "armv7", "riscv64"].contains(&arch()), "{}", arch());
        #[cfg(windows)]
        assert_eq!(os(), "windows");
        #[cfg(target_os = "linux")]
        assert_eq!(os(), "linux");
        #[cfg(target_arch = "x86_64")]
        assert_eq!(arch(), "x86_64");
        let body = payload(ID, "1.0.0", Kind::Heartbeat);
        assert_eq!(body["os"], os());
        assert_eq!(body["arch"], arch());
    }

    #[test]
    fn the_events_go_to_signal_labs_events_on_the_hub() {
        assert_eq!(endpoint(), hub::endpoint("events"));
        assert!(endpoint().ends_with("/v1/signal-lab/events"));
    }

    #[test]
    fn launch_until_the_hub_takes_it_then_heartbeats() {
        let mut sequence = Sequence::default();
        assert_eq!(sequence.next(), Kind::Launch);
        sequence.record(Kind::Launch, Outcome::Failed);
        assert_eq!(sequence.next(), Kind::Launch, "a launch that failed goes again");
        sequence.record(Kind::Launch, Outcome::Busy);
        assert_eq!(sequence.next(), Kind::Launch, "so does one the hub asked to wait with");
        sequence.record(Kind::Launch, Outcome::Accepted);
        assert_eq!(sequence.next(), Kind::Heartbeat);
        sequence.record(Kind::Heartbeat, Outcome::Failed);
        assert_eq!(sequence.next(), Kind::Heartbeat, "a heartbeat lost is not a launch");

        let mut refused = Sequence::default();
        refused.record(Kind::Launch, Outcome::Refused);
        assert_eq!(refused.next(), Kind::Heartbeat, "a launch refused for good is not sent for ever");
    }

    #[test]
    fn statuses_are_taken_busy_refused_or_failed() {
        assert_eq!(Outcome::of_status(202), Outcome::Accepted);
        assert_eq!(Outcome::of_status(204), Outcome::Accepted);
        assert_eq!(Outcome::of_status(429), Outcome::Busy);
        assert_eq!(Outcome::of_status(400), Outcome::Refused);
        assert_eq!(Outcome::of_status(404), Outcome::Refused);
        assert_eq!(Outcome::of_status(500), Outcome::Failed);
        assert_eq!(Outcome::of_status(503), Outcome::Failed);
    }

    #[test]
    fn nothing_is_sent_without_an_id_or_without_consent() {
        let stats = Stats::default();
        assert_eq!(stats.sending_as(), None, "not configured yet");
        stats.configure(Some(ID), true);
        assert_eq!(stats.sending_as().as_deref(), Some(ID));
        stats.configure(Some(ID), false);
        assert_eq!(stats.sending_as(), None, "turned off");
        stats.configure(None, true);
        assert_eq!(stats.sending_as(), None, "no id: nothing could be kept");
        stats.configure(Some("not an id!"), true);
        assert_eq!(stats.sending_as(), None, "an id the hub would refuse");
        assert!(!valid_install("short"));
        assert!(valid_install(ID));
    }
}
