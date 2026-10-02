//! A Signal Lab server, used the way a script uses it: `POST /api/run` read
//! line by line, `/api/invoke/<command>`, reports from `/api/files`, the token
//! as `Authorization: Bearer`.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use signal_lab_engine::error::EngineError;

use crate::fail::Failure;

/// How long a run's answer may stay silent: the server sends a heartbeat every 15 s.
const SILENCE: Duration = Duration::from_secs(60);

pub struct Remote {
    base: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl Remote {
    /// The server at `url`; the token from `token_file`, else `SIGNALLAB_TOKEN`.
    pub fn new(url: &str, token_file: Option<&Path>) -> Result<Remote, Failure> {
        let base = url.trim().trim_end_matches('/').to_string();
        if !(base.starts_with("http://") || base.starts_with("https://")) || base.len() < "http://x".len() {
            return Err(Failure::invalid(EngineError::new("cli.server_invalid").with("url", url)));
        }
        let token = match token_file {
            Some(path) => {
                let text = std::fs::read_to_string(path).map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", path.display()).because(error)))?;
                let token = text.trim().to_string();
                if token.is_empty() {
                    return Err(Failure::invalid(EngineError::new("cli.token_empty").with("path", path.display())));
                }
                Some(token)
            }
            None => std::env::var("SIGNALLAB_TOKEN").ok().map(|token| token.trim().to_string()).filter(|token| !token.is_empty()),
        };
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!("signallab/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|error| Failure::environment(EngineError::new("http.client_failed").because(error)))?;
        Ok(Remote { base, token, client })
    }

    fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let request = self.client.post(format!("{}{path}", self.base));
        match &self.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }

    fn unreachable(&self, error: reqwest::Error) -> Failure {
        Failure::environment(EngineError::new("cli.server_unreachable").with("url", &self.base).because(chain(&error)))
    }

    /// A refusal: the server's own error when it sent one.
    async fn refused(&self, response: reqwest::Response) -> Failure {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        match serde_json::from_str::<EngineError>(&body) {
            Ok(error) => Failure::starting(error).on_server(),
            Err(_) => Failure::environment(EngineError::new("cli.server_reply").with("url", &self.base).with("status", status).because(excerpt(&body))),
        }
    }

    /// One command of the server's table.
    pub async fn invoke(&self, command: &str, args: &Value) -> Result<Value, Failure> {
        let response = self.post(&format!("/api/invoke/{command}")).json(args).send().await.map_err(|error| self.unreachable(error))?;
        if !response.status().is_success() {
            return Err(self.refused(response).await);
        }
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|error| self.unreachable(error))?;
        serde_json::from_str(&body).map_err(|_| Failure::environment(EngineError::new("cli.server_reply").with("url", &self.base).with("status", status).because(excerpt(&body))))
    }

    /// Run on the server; `line` gets every `started`, `step` and `ended` line as it arrives.
    pub async fn run(&self, request: &Value, line: &mut dyn FnMut(&str, &Value)) -> Result<Value, Failure> {
        let response = self
            .post("/api/run")
            .header(reqwest::header::ACCEPT, "application/x-ndjson")
            .json(request)
            .send()
            .await
            .map_err(|error| self.unreachable(error))?;
        if !response.status().is_success() {
            return Err(self.refused(response).await);
        }
        let status = response.status().as_u16();
        let broken = |detail: String| Failure::environment(EngineError::new("cli.server_ended").with("url", &self.base).because(detail));
        let mut response = response;
        let mut pending: Vec<u8> = Vec::new();
        loop {
            let chunk = match tokio::time::timeout(SILENCE, response.chunk()).await {
                Err(_) => return Err(broken(format!("nothing for {} s", SILENCE.as_secs()))),
                Ok(Err(error)) => return Err(broken(chain(&error))),
                Ok(Ok(None)) => return Err(broken("the answer ended before the run did".into())),
                Ok(Ok(Some(chunk))) => chunk,
            };
            pending.extend_from_slice(&chunk);
            while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
                let text: Vec<u8> = pending.drain(..=end).collect();
                let text = String::from_utf8_lossy(&text);
                if text.trim().is_empty() {
                    continue;
                }
                let value: Value = serde_json::from_str(text.trim()).map_err(|_| {
                    Failure::environment(EngineError::new("cli.server_reply").with("url", &self.base).with("status", status).because(excerpt(&text)))
                })?;
                let kind = value["type"].as_str().unwrap_or_default().to_string();
                match kind.as_str() {
                    "started" | "step" => line(&kind, &value),
                    "ended" => {
                        line(&kind, &value);
                        return Ok(value);
                    }
                    // Heartbeats, and whatever a newer server adds.
                    _ => {}
                }
            }
        }
    }

    /// A file the server wrote (a run report), saved at `to`.
    pub async fn download(&self, path: &str, to: &Path) -> Result<(), Failure> {
        let mut request = self.client.get(format!("{}/api/files", self.base)).query(&[("path", path)]);
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.map_err(|error| self.unreachable(error))?;
        if !response.status().is_success() {
            return Err(self.refused(response).await);
        }
        let bytes = response.bytes().await.map_err(|error| self.unreachable(error))?;
        std::fs::write(to, bytes).map_err(|error| Failure::invalid(EngineError::new("file.io").with("path", to.display()).because(error)))
    }

    /// The request body of a run.
    pub fn run_request(document: &Value, overrides: &std::collections::BTreeMap<String, String>, seed: Option<u64>, timeout: Option<u64>) -> Value {
        let mut request = json!({ "document": document, "overrides": overrides });
        if let Some(seed) = seed {
            request["seed"] = seed.into();
        }
        if let Some(timeout) = timeout {
            request["timeout"] = timeout.into();
        }
        request
    }
}

/// An error with every layer of its cause: reqwest's own message is only the outermost.
fn chain(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let cause_text = cause.to_string();
        if !text.contains(&cause_text) {
            text.push_str(": ");
            text.push_str(&cause_text);
        }
        source = cause.source();
    }
    text
}

/// The start of an unexpected answer, for the detail.
fn excerpt(text: &str) -> String {
    let text = text.trim();
    match text.char_indices().nth(200) {
        Some((at, _)) => format!("{}…", &text[..at]),
        None => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_server_is_named_by_its_http_url() {
        assert!(Remote::new("http://lab.example:1430/", None).is_ok());
        assert_eq!(Remote::new("http://lab.example:1430/", None).unwrap().base, "http://lab.example:1430");
        for bad in ["lab.example:1430", "ftp://x", "http://"] {
            assert_eq!(Remote::new(bad, None).err().unwrap().error.code, "cli.server_invalid", "{bad}");
        }
        assert_eq!(excerpt(&"x".repeat(300)).chars().count(), 201);
    }
}
