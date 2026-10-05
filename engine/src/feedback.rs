//! The feedback form, sent to the studio's hub (`hub`, docs/develop/hub.md), which
//! mails it to the developers. The app holds no secret for it — the hub keeps
//! the mailbox's password and decides where the mail goes. The hub's limits
//! are checked here first, so nothing it would refuse is uploaded; a refusal
//! it gives anyway comes back as its own code (`refusal`), a network failure
//! as `transport.*` about the hub.

use std::collections::BTreeMap;
use std::time::Duration;

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::{EngineError, EngineResult};
use super::{hub, transport};

/// The hub's limits — its src/feedback/form.rs, in its own repository: change
/// both together (src/lib/feedback.ts follows these, a test says so).
pub const MAX_MESSAGE_CHARS: usize = 20_000;
pub const MAX_IMAGES: usize = 6;
pub const MAX_IMAGE_BYTES: usize = 8 << 20;
pub const MAX_LOGS: usize = 4;
pub const MAX_LOG_BYTES: usize = 2 << 20;
pub const MAX_TOTAL_BYTES: usize = 15 << 20;

/// An upload of 15 MB over a slow line, and the hub's own SMTP round trip.
const TIMEOUT: Duration = Duration::from_secs(180);

/// Where the form goes: Signal Lab's feedback on the hub.
pub fn url() -> String {
    hub::endpoint("feedback")
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Form {
    pub message: String,
    /// Where an answer goes; optional.
    #[serde(default)]
    pub email: Option<String>,
    /// What the app says about itself: version, os, arch, mode, lang, screen.
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
    #[serde(default)]
    pub screenshots: Vec<Screenshot>,
    #[serde(default)]
    pub logs: Vec<Log>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screenshot {
    pub name: String,
    /// The image's bytes, base64.
    pub data: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub name: String,
    pub text: String,
}

/// The hub took it; `id` is in the mail the developers get.
#[derive(Debug, Serialize)]
pub struct Sent {
    pub id: String,
}

/// The hub's refusals the app has words for; anything else it says (an
/// unknown project, the hub's own trouble) is `feedback.failed`, so a code is
/// never taken from the network unchecked.
pub fn refusal(code: &str) -> EngineError {
    match code {
        "feedback.message_required" => EngineError::new("feedback.message_required"),
        "feedback.message_too_long" => EngineError::new("feedback.message_too_long"),
        "feedback.email_invalid" => EngineError::new("feedback.email_invalid"),
        "feedback.too_many_files" => EngineError::new("feedback.too_many_files"),
        "feedback.file_type" => EngineError::new("feedback.file_type"),
        "feedback.file_too_large" => EngineError::new("feedback.file_too_large"),
        "feedback.too_large" => EngineError::new("feedback.too_large"),
        "feedback.rate_limited" => EngineError::new("feedback.rate_limited"),
        "feedback.send_failed" => EngineError::new("feedback.send_failed"),
        "feedback.invalid" => EngineError::new("feedback.invalid"),
        // The project takes no feedback now (switched off on the hub).
        "feedback.disabled" => EngineError::new("feedback.disabled"),
        _ => EngineError::new("feedback.failed"),
    }
}

/// What goes into the request, checked against the hub's limits.
#[derive(Debug)]
struct Checked {
    message: String,
    email: Option<String>,
    meta: BTreeMap<String, String>,
    screenshots: Vec<(String, Vec<u8>)>,
    logs: Vec<(String, String)>,
}

fn check(form: Form) -> EngineResult<Checked> {
    let message = form.message.trim().to_string();
    if message.is_empty() {
        return Err(EngineError::new("feedback.message_required"));
    }
    if message.chars().count() > MAX_MESSAGE_CHARS {
        return Err(EngineError::new("feedback.message_too_long").with("max", MAX_MESSAGE_CHARS));
    }
    if form.screenshots.len() > MAX_IMAGES {
        return Err(EngineError::new("feedback.too_many_files").with("kind", "screenshot").with("max", MAX_IMAGES));
    }
    if form.logs.len() > MAX_LOGS {
        return Err(EngineError::new("feedback.too_many_files").with("kind", "log").with("max", MAX_LOGS));
    }
    let mut total = message.len();
    let mut screenshots = Vec::new();
    for screenshot in form.screenshots {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(screenshot.data.trim())
            .map_err(|error| EngineError::new("feedback.invalid").because(format!("{}: {error}", screenshot.name)))?;
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(EngineError::new("feedback.file_too_large").with("name", &screenshot.name).with("max_mb", MAX_IMAGE_BYTES >> 20));
        }
        total += bytes.len();
        screenshots.push((screenshot.name, bytes));
    }
    let mut logs = Vec::new();
    for log in form.logs {
        if log.text.len() > MAX_LOG_BYTES {
            return Err(EngineError::new("feedback.file_too_large").with("name", &log.name).with("max_mb", MAX_LOG_BYTES >> 20));
        }
        total += log.text.len();
        logs.push((log.name, log.text));
    }
    if total > MAX_TOTAL_BYTES {
        return Err(EngineError::new("feedback.too_large").with("max_mb", MAX_TOTAL_BYTES >> 20));
    }
    let email = form.email.map(|email| email.trim().to_string()).filter(|email| !email.is_empty());
    Ok(Checked { message, email, meta: form.meta, screenshots, logs })
}

/// The form to the hub.
pub async fn send(form: Form) -> EngineResult<Sent> {
    send_to(&url(), form).await
}

/// The form to the feedback endpoint at `url`.
pub async fn send_to(url: &str, form: Form) -> EngineResult<Sent> {
    let checked = check(form)?;
    let mut body = reqwest::multipart::Form::new().text("message", checked.message);
    if let Some(email) = checked.email {
        body = body.text("email", email);
    }
    if !checked.meta.is_empty() {
        body = body.text("meta", serde_json::to_string(&checked.meta).unwrap_or_default());
    }
    for (name, bytes) in checked.screenshots {
        // Its type is the hub's to decide, from the bytes.
        body = body.part("screenshot", reqwest::multipart::Part::bytes(bytes).file_name(name));
    }
    for (name, text) in checked.logs {
        let part = reqwest::multipart::Part::text(text).file_name(name).mime_str("text/plain; charset=utf-8").map_err(|error| EngineError::new("feedback.invalid").because(error))?;
        body = body.part("log", part);
    }
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .user_agent(concat!("SignalLab/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| EngineError::new("http.client_failed").because(transport::chain(&error)))?;
    let response = client
        .post(url)
        .multipart(body)
        .send()
        .await
        .map_err(|error| transport::of_reqwest(&error).error(url).because(transport::chain(&error)))?;
    let status = response.status();
    let answer: Value = response.json().await.unwrap_or(Value::Null);
    if status.is_success() {
        return match answer["id"].as_str() {
            Some(id) => Ok(Sent { id: id.to_string() }),
            None => Err(EngineError::new("feedback.failed").with("status", status.as_u16())),
        };
    }
    let error = &answer["error"];
    let Some(code) = error["code"].as_str() else {
        return Err(EngineError::new("feedback.failed").with("status", status.as_u16()));
    };
    let mut refused = refusal(code);
    if refused.code == "feedback.failed" {
        return Err(refused.with("status", status.as_u16()).because(code));
    }
    for (name, value) in error["params"].as_object().into_iter().flatten() {
        if let Some(value) = value.as_str() {
            refused = refused.with(name, value);
        }
    }
    if let Some(detail) = error["detail"].as_str() {
        refused = refused.because(detail);
    }
    Err(refused)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(message: &str) -> Form {
        Form { message: message.into(), email: None, meta: BTreeMap::new(), screenshots: Vec::new(), logs: Vec::new() }
    }

    #[test]
    fn what_the_hub_would_refuse_is_not_uploaded() {
        assert!(check(form("  \n")).unwrap_err().is("feedback.message_required"));
        assert!(check(form(&"ж".repeat(MAX_MESSAGE_CHARS + 1))).unwrap_err().is("feedback.message_too_long"));
        assert_eq!(check(form(&"ж".repeat(MAX_MESSAGE_CHARS))).unwrap().message.chars().count(), MAX_MESSAGE_CHARS, "characters, not bytes");
        let shot = |bytes: usize| Screenshot { name: "s.png".into(), data: base64::engine::general_purpose::STANDARD.encode(vec![0u8; bytes]) };
        let many = Form { screenshots: (0..=MAX_IMAGES).map(|_| shot(1)).collect(), ..form("x") };
        assert_eq!(check(many).unwrap_err().params["max"], MAX_IMAGES.to_string());
        let big = Form { screenshots: vec![shot(MAX_IMAGE_BYTES + 1)], ..form("x") };
        assert!(check(big).unwrap_err().is("feedback.file_too_large"));
        let heavy = Form { screenshots: (0..2).map(|_| shot(MAX_IMAGE_BYTES)).collect(), ..form("x") };
        assert!(check(heavy).unwrap_err().is("feedback.too_large"));
        let broken = Form { screenshots: vec![Screenshot { name: "s.png".into(), data: "not base64!".into() }], ..form("x") };
        assert!(check(broken).unwrap_err().is("feedback.invalid"));
        let blank = Form { email: Some("  ".into()), ..form("x") };
        assert_eq!(check(blank).unwrap().email, None, "an empty address is no address");
    }

    #[test]
    fn a_code_from_the_network_is_one_the_app_knows_or_none() {
        // Every refusal of the hub's feedback endpoint (its api.rs and feedback/form.rs).
        for code in [
            "feedback.message_required",
            "feedback.message_too_long",
            "feedback.email_invalid",
            "feedback.too_many_files",
            "feedback.file_type",
            "feedback.file_too_large",
            "feedback.too_large",
            "feedback.rate_limited",
            "feedback.send_failed",
            "feedback.invalid",
            "feedback.disabled",
        ] {
            assert!(refusal(code).is(code), "the hub says {code}, which the app has words for");
        }
        assert!(refusal("project.unknown").is("feedback.failed"), "the hub's own codes are its business");
        assert!(refusal("auth.required").is("feedback.failed"), "never another module's code");
        assert!(refusal("feedback.whatever").is("feedback.failed"));
    }
}
