//! Exit codes and the failures behind them.

use signal_lab_engine::error::EngineError;

use crate::Ctx;

/// What the process exits with, least serious first: with several
/// experiments the most serious one decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exit {
    /// 0: every experiment passed, the send succeeded.
    Passed,
    /// 1: an experiment ran and failed (or ran out of time); a send failed.
    Failed,
    /// 3: nothing could run for a reason outside the experiment: the server
    /// cannot be reached or refuses the token, a port cannot be opened.
    Environment,
    /// 2: the invocation or a document is wrong: an argument, a file that
    /// cannot be read, a validation error, an unknown parameter, a missing secret.
    Invalid,
}

impl Exit {
    pub fn code(self) -> u8 {
        match self {
            Exit::Passed => 0,
            Exit::Failed => 1,
            Exit::Invalid => 2,
            Exit::Environment => 3,
        }
    }
}

/// A code that says the surroundings are wrong, not the experiment or the arguments.
pub fn environmental(code: &str) -> bool {
    let mqtt_document = ["mqtt.topic_invalid", "mqtt.topic_required", "mqtt.filter_required", "mqtt.client_id_required"];
    (code.starts_with("transport.") && code != "transport.target_invalid")
        || code.starts_with("auth.")
        || (code.starts_with("mqtt.") && !mqtt_document.contains(&code))
        || matches!(
            code,
            "wait.bind_failed"
                | "ws.handshake_status"
                | "ws.handshake_failed"
                | "ws.closed"
                | "socket.option_failed"
                | "http.client_failed"
                | "secret.store"
                | "secret.unsupported"
                | "api.not_found"
                | "cli.server_unreachable"
                | "cli.server_reply"
                | "cli.server_ended"
        )
}

/// A failure the command line reports, and how serious it is.
#[derive(Debug, Clone)]
pub struct Failure {
    pub error: EngineError,
    pub exit: Exit,
    /// It came from a server: worded for the server, not this machine.
    pub remote: bool,
}

impl Failure {
    pub fn invalid(error: EngineError) -> Self {
        Failure { error, exit: Exit::Invalid, remote: false }
    }

    pub fn environment(error: EngineError) -> Self {
        Failure { error, exit: Exit::Environment, remote: false }
    }

    /// Something that kept an experiment from starting, by its code.
    pub fn starting(error: EngineError) -> Self {
        let exit = if environmental(&error.code) { Exit::Environment } else { Exit::Invalid };
        Failure { error, exit, remote: false }
    }

    /// A send that went wrong: its arguments (2), or the send itself (1).
    pub fn sending(error: EngineError) -> Self {
        let exit = if environmental(&error.code) { Exit::Failed } else { Exit::Invalid };
        Failure { error, exit, remote: false }
    }

    pub fn on_server(mut self) -> Self {
        self.remote = true;
        self
    }

    /// The command line's own wording of a code, tried before the app's.
    pub fn wording(&self) -> &'static [&'static str] {
        if self.remote {
            &["cli.serverErr.", "cli.err."]
        } else {
            &["cli.err."]
        }
    }

    /// The failure in words: `where — what`, and the detail on its own line.
    pub fn lines(&self, ctx: &Ctx, prefix: &str) -> Vec<String> {
        let described = ctx.texts.describe(&self.error, self.wording(), &|_| None);
        let mut lines = vec![format!("{prefix}{}", described.text)];
        if let Some(detail) = described.detail {
            lines.push(format!("  {}: {detail}", ctx.texts.plain("err.details")));
        }
        lines
    }

    /// Say it on stderr (or as a JSON line) and give its exit code.
    pub fn report(&self, ctx: &Ctx, prefix: &str) -> Exit {
        if ctx.json {
            println!("{}", serde_json::json!({ "type": "error", "error": self.error, "exit_code": self.exit.code() }));
        } else {
            for line in self.lines(ctx, prefix) {
                eprintln!("{line}");
            }
        }
        self.exit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_surroundings_are_told_apart_from_the_document() {
        for code in ["transport.refused", "transport.address_in_use", "mqtt.refused", "mqtt.no_answer", "auth.required", "cli.server_unreachable", "secret.store", "wait.bind_failed"] {
            assert_eq!(Failure::starting(EngineError::new(code)).exit, Exit::Environment, "{code}");
        }
        for code in ["transport.target_invalid", "graph.unreachable", "run.override_unknown", "secret.missing", "mqtt.topic_invalid", "file.json_invalid", "run.limit_range"] {
            assert_eq!(Failure::starting(EngineError::new(code)).exit, Exit::Invalid, "{code}");
        }
        assert_eq!(Failure::sending(EngineError::new("transport.unreachable")).exit, Exit::Failed);
        assert_eq!(Failure::sending(EngineError::new("hex.invalid")).exit, Exit::Invalid);
        assert!(Exit::Invalid > Exit::Environment && Exit::Environment > Exit::Failed && Exit::Failed > Exit::Passed);
        assert_eq!([Exit::Passed, Exit::Failed, Exit::Invalid, Exit::Environment].map(Exit::code), [0, 1, 2, 3]);
    }
}
