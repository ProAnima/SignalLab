//! Who may use the server. On loopback without a token, anyone on this machine
//! — but only under a loopback host name, so a web page cannot reach it through
//! DNS rebinding. Everywhere else, a token: sent as `Authorization: Bearer` by
//! scripts, exchanged once at `/login` for a session cookie by browsers.
//! State-changing requests must also come from this origin.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::{header, HeaderMap, HeaderValue};

pub const COOKIE: &str = "signallab_session";
/// How long a browser stays signed in.
pub const SESSION_TTL: Duration = Duration::from_secs(7 * 24 * 3600);
/// Sessions kept at once; the oldest goes first.
const MAX_SESSIONS: usize = 1024;

pub struct Auth {
    token: Option<String>,
    sessions: Mutex<HashMap<String, Instant>>,
    secure_cookie: bool,
    allowed_hosts: Vec<String>,
}

/// Equal in time independent of where the inputs differ.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// `host` of a `host[:port]` authority, lowercased; `[::1]:80` → `[::1]`.
fn host_name(authority: &str) -> String {
    let authority = authority.trim().to_ascii_lowercase();
    if authority.starts_with('[') {
        return authority.split_once(']').map(|(host, _)| format!("{host}]")).unwrap_or(authority);
    }
    authority.rsplit_once(':').filter(|(_, port)| port.chars().all(|c| c.is_ascii_digit())).map(|(host, _)| host.to_string()).unwrap_or(authority)
}

fn is_loopback_name(host: &str) -> bool {
    host == "localhost" || host.ends_with(".localhost") || host == "[::1]" || host.starts_with("127.")
}

impl Auth {
    pub fn new(token: Option<String>, secure_cookie: bool, allowed_hosts: Vec<String>) -> Self {
        Auth { token, sessions: Mutex::new(HashMap::new()), secure_cookie, allowed_hosts }
    }

    /// Does every request need a token or a session?
    pub fn required(&self) -> bool {
        self.token.is_some()
    }

    pub fn token_matches(&self, candidate: &str) -> bool {
        self.token.as_deref().is_some_and(|token| same(token, candidate.trim()))
    }

    /// The `Host` header names this server. Without a token only loopback
    /// names do (a page served from another name cannot talk to it); with one,
    /// any name unless `--allowed-host` narrows it.
    pub fn host_allowed(&self, headers: &HeaderMap) -> bool {
        let Some(host) = headers.get(header::HOST).and_then(|value| value.to_str().ok()).map(host_name) else {
            return false;
        };
        if is_loopback_name(&host) || self.allowed_hosts.contains(&host) {
            return true;
        }
        self.required() && self.allowed_hosts.is_empty()
    }

    /// A browser sends `Origin` with every state-changing request; it must be
    /// this server. Requests without one (scripts, curl) pass: they need the
    /// token anyway, or come from this machine.
    pub fn origin_allowed(&self, headers: &HeaderMap) -> bool {
        let Some(origin) = headers.get(header::ORIGIN).and_then(|value| value.to_str().ok()) else {
            return true;
        };
        let Some(host) = headers.get(header::HOST).and_then(|value| value.to_str().ok()) else {
            return false;
        };
        let authority = origin.split_once("://").map(|(_, rest)| rest).unwrap_or("");
        authority.eq_ignore_ascii_case(host)
    }

    /// The request carries the token or a live session.
    pub fn authenticated(&self, headers: &HeaderMap) -> bool {
        if !self.required() {
            return true;
        }
        let bearer = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .is_some_and(|token| self.token_matches(token));
        bearer || self.session_from(headers).is_some_and(|session| self.session_valid(&session))
    }

    pub fn session_from(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .find_map(|pair| pair.trim().strip_prefix(COOKIE).and_then(|rest| rest.strip_prefix('=')))
            .map(str::to_string)
    }

    pub fn new_session(&self) -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let id: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        let mut sessions = self.sessions.lock().unwrap();
        let now = Instant::now();
        sessions.retain(|_, started| now.duration_since(*started) < SESSION_TTL);
        if sessions.len() >= MAX_SESSIONS {
            if let Some(oldest) = sessions.iter().min_by_key(|(_, started)| **started).map(|(id, _)| id.clone()) {
                sessions.remove(&oldest);
            }
        }
        sessions.insert(id.clone(), now);
        id
    }

    pub fn session_valid(&self, id: &str) -> bool {
        let sessions = self.sessions.lock().unwrap();
        sessions.iter().any(|(known, started)| same(known, id) && started.elapsed() < SESSION_TTL)
    }

    pub fn end_session(&self, id: &str) {
        self.sessions.lock().unwrap().remove(id);
    }

    pub fn session_cookie(&self, id: &str) -> HeaderValue {
        let secure = if self.secure_cookie { "; Secure" } else { "" };
        let cookie = format!("{COOKIE}={id}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{secure}", SESSION_TTL.as_secs());
        HeaderValue::from_str(&cookie).expect("a hex session id makes a valid header")
    }

    pub fn cleared_cookie(&self) -> HeaderValue {
        let secure = if self.secure_cookie { "; Secure" } else { "" };
        HeaderValue::from_str(&format!("{COOKIE}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0{secure}")).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.append(axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap(), HeaderValue::from_str(value).unwrap());
        }
        map
    }

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn hosts_guard_against_rebinding() {
        let open = Auth::new(None, false, vec![]);
        for host in ["127.0.0.1:1430", "localhost:1430", "[::1]:1430", "app.localhost"] {
            assert!(open.host_allowed(&headers(&[("host", host)])), "{host}");
        }
        assert!(!open.host_allowed(&headers(&[("host", "evil.example:1430")])), "without a token only loopback names");
        assert!(!open.host_allowed(&headers(&[])));
        let named = Auth::new(None, false, vec!["lab.example".into()]);
        assert!(named.host_allowed(&headers(&[("host", "LAB.example:1430")])));
        let guarded = Auth::new(Some(TOKEN.into()), false, vec![]);
        assert!(guarded.host_allowed(&headers(&[("host", "10.0.0.5:1430")])), "with a token any name, unless narrowed");
        let narrowed = Auth::new(Some(TOKEN.into()), false, vec!["lab.example".into()]);
        assert!(!narrowed.host_allowed(&headers(&[("host", "10.0.0.5:1430")])));
        assert_eq!(host_name("[::1]:1430"), "[::1]");
        assert_eq!(host_name("Example.COM"), "example.com");
    }

    #[test]
    fn origins_must_be_this_server() {
        let auth = Auth::new(None, false, vec![]);
        assert!(auth.origin_allowed(&headers(&[("host", "127.0.0.1:1430"), ("origin", "http://127.0.0.1:1430")])));
        assert!(!auth.origin_allowed(&headers(&[("host", "127.0.0.1:1430"), ("origin", "http://evil.example")])));
        assert!(!auth.origin_allowed(&headers(&[("host", "127.0.0.1:1430"), ("origin", "null")])));
        assert!(auth.origin_allowed(&headers(&[("host", "127.0.0.1:1430")])), "scripts send no Origin");
    }

    #[test]
    fn tokens_and_sessions() {
        let auth = Auth::new(Some(TOKEN.into()), true, vec![]);
        assert!(!auth.authenticated(&headers(&[])));
        assert!(auth.authenticated(&headers(&[("authorization", &format!("Bearer {TOKEN}"))])));
        assert!(!auth.authenticated(&headers(&[("authorization", "Bearer wrong")])));
        assert!(!auth.token_matches(&TOKEN[..TOKEN.len() - 1]));
        let session = auth.new_session();
        let cookie = auth.session_cookie(&session).to_str().unwrap().to_string();
        assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Strict") && cookie.contains("Secure"));
        assert!(auth.authenticated(&headers(&[("cookie", &format!("theme=dark; {COOKIE}={session}"))])));
        auth.end_session(&session);
        assert!(!auth.authenticated(&headers(&[("cookie", &format!("{COOKIE}={session}"))])), "a signed-out session is gone");
        assert!(Auth::new(None, false, vec![]).authenticated(&headers(&[])), "loopback without a token needs nothing");
    }
}
