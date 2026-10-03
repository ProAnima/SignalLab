//! The studio's hub (ProAnima Hub, a repository of its own), which Signal Lab
//! talks to: the feedback form goes to it (`feedback`), and the desktop app's
//! updater asks it first (`plugins.updater.endpoints` in
//! src-tauri/tauri.conf.json; GitHub's latest.json when the hub cannot be
//! reached). The hub keeps the mailbox's password and decides which published
//! release each install is offered; the app holds no secret for it.

/// Where the hub is unless `SIGNALLAB_HUB_URL` says otherwise.
pub const DEFAULT_URL: &str = "https://hub.proanima.net";

/// Signal Lab's project on the hub.
pub const PROJECT: &str = "signal-lab";

/// `SIGNALLAB_HUB_URL` of this process, else of the build, else the studio's hub.
pub fn url() -> String {
    let url = std::env::var("SIGNALLAB_HUB_URL")
        .ok()
        .filter(|url| !url.trim().is_empty())
        .or_else(|| option_env!("SIGNALLAB_HUB_URL").map(str::to_string))
        .unwrap_or_else(|| DEFAULT_URL.to_string());
    url.trim().trim_end_matches('/').to_string()
}

/// One of Signal Lab's endpoints on the hub: `feedback`, …
pub fn endpoint(path: &str) -> String {
    format!("{}/v1/{PROJECT}/{path}", url())
}
