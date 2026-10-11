//! The studio's hub (ProAnima Hub, a repository of its own), which Signal Lab
//! talks to: the feedback form goes to it (`feedback`), the desktop app's
//! anonymous usage statistics too (`events`, see `stats`), and the desktop app's
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

/// The variable that keeps an install from reaching the hub on its own: no
/// automatic update check, no statistics (the end-to-end tour sets it).
pub const NO_UPDATE_CHECK: &str = "SIGNALLAB_NO_UPDATE_CHECK";

/// Whether the desktop app reaches the hub on its own — the daily update
/// check and the usage statistics: a release build, unless `NO_UPDATE_CHECK`
/// is set. Development and the tour never do.
pub fn automatic(debug_build: bool, opted_out: bool) -> bool {
    !debug_build && !opted_out
}

/// `automatic` for this process: the build given by the caller (the shell's
/// own `cfg!(debug_assertions)`), the variable read from the environment.
pub fn automatic_here(debug_build: bool) -> bool {
    automatic(debug_build, std::env::var_os(NO_UPDATE_CHECK).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_release_build_not_opted_out_reaches_the_hub_on_its_own() {
        assert!(automatic(false, false));
        assert!(!automatic(true, false), "a development build never does");
        assert!(!automatic(false, true), "SIGNALLAB_NO_UPDATE_CHECK (the tour) turns it off");
        assert!(!automatic(true, true));
    }

    #[test]
    fn an_endpoint_is_under_signal_labs_project() {
        assert!(endpoint("events").ends_with("/v1/signal-lab/events"));
        assert!(!endpoint("events").contains("//v1"));
    }
}
