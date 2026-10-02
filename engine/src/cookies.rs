//! Cookie jars: what a server set with `Set-Cookie`, sent back with later
//! requests to it as a browser does (RFC 6265: host and domain, path, Secure,
//! Max-Age and Expires). The HTTP screen keeps one for its requests and its
//! burst; a run keeps one of its own, from its first step to its last.
//!
//! It is reqwest's own cookie store, held here so a jar can be listed and
//! cleared — what `reqwest::cookie::Jar` keeps to itself.

use std::sync::{Arc, Mutex};

use reqwest::header::HeaderValue;
use reqwest::Url;
use serde::Serialize;

#[derive(Default)]
pub struct CookieJar(Mutex<cookie_store::CookieStore>);

/// One cookie as a jar holds it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CookieInfo {
    pub name: String,
    pub value: String,
    pub domain: String,
    /// No Domain attribute: sent to exactly the host that set it.
    pub host_only: bool,
    pub path: String,
    /// Unix seconds; `None` for a session cookie (gone with the jar).
    pub expires: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

impl CookieJar {
    pub fn new() -> Arc<CookieJar> {
        Arc::new(CookieJar::default())
    }

    /// Every cookie that has not expired, by domain, path and name.
    pub fn list(&self) -> Vec<CookieInfo> {
        let store = self.0.lock().unwrap();
        let mut cookies: Vec<CookieInfo> = store
            .iter_unexpired()
            .map(|cookie| {
                let (domain, host_only) = match &cookie.domain {
                    cookie_store::CookieDomain::HostOnly(host) => (host.clone(), true),
                    cookie_store::CookieDomain::Suffix(suffix) => (suffix.clone(), false),
                    _ => (String::new(), true),
                };
                CookieInfo {
                    name: cookie.name().to_string(),
                    value: cookie.value().to_string(),
                    domain,
                    host_only,
                    path: cookie.path.as_ref().to_string(),
                    expires: match &cookie.expires {
                        cookie_store::CookieExpiration::AtUtc(at) => Some(at.unix_timestamp()),
                        cookie_store::CookieExpiration::SessionEnd => None,
                    },
                    secure: cookie.secure().unwrap_or(false),
                    http_only: cookie.http_only().unwrap_or(false),
                    same_site: cookie.same_site().map(|same_site| same_site.to_string()),
                }
            })
            .collect();
        cookies.sort_by(|a, b| (&a.domain, &a.path, &a.name).cmp(&(&b.domain, &b.path, &b.name)));
        cookies
    }

    pub fn clear(&self) {
        self.0.lock().unwrap().clear();
    }

    /// The `Cookie` header this jar would send to `url`.
    pub fn header_for(&self, url: &str) -> Option<String> {
        let url = Url::parse(url).ok()?;
        let pairs: Vec<String> = self.0.lock().unwrap().get_request_values(&url).map(|(name, value)| format!("{name}={value}")).collect();
        (!pairs.is_empty()).then(|| pairs.join("; "))
    }
}

impl reqwest::cookie::CookieStore for CookieJar {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &Url) {
        let cookies: Vec<cookie_store::RawCookie<'static>> = cookie_headers
            .filter_map(|value| value.to_str().ok())
            .filter_map(|text| cookie_store::RawCookie::parse(text.to_string()).ok())
            .collect();
        self.0.lock().unwrap().store_response_cookies(cookies.into_iter(), url);
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let pairs: Vec<String> = self.0.lock().unwrap().get_request_values(url).map(|(name, value)| format!("{name}={value}")).collect();
        if pairs.is_empty() {
            return None;
        }
        HeaderValue::from_str(&pairs.join("; ")).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::cookie::CookieStore as _;

    fn set(jar: &CookieJar, url: &str, headers: &[&str]) {
        let values: Vec<HeaderValue> = headers.iter().map(|header| HeaderValue::from_str(header).unwrap()).collect();
        jar.set_cookies(&mut values.iter(), &Url::parse(url).unwrap());
    }

    #[test]
    fn a_jar_sends_back_what_was_set_where_it_belongs() {
        let jar = CookieJar::new();
        set(&jar, "http://api.lab.test/login", &["session=abc; Path=/; HttpOnly", "pref=dark; Domain=lab.test; Path=/ui", "tls=1; Secure"]);
        assert_eq!(jar.header_for("http://api.lab.test/orders").as_deref(), Some("session=abc"), "a host-only cookie, the path /, not the Secure one over http");
        assert_eq!(jar.header_for("http://www.lab.test/ui/x").as_deref(), Some("pref=dark"), "a domain cookie reaches the sibling host, on its path");
        assert_eq!(jar.header_for("http://other.test/").as_deref(), None);
        let secure = jar.header_for("https://api.lab.test/").unwrap();
        assert!(secure.contains("tls=1") && secure.contains("session=abc"), "{secure}");

        let listed = jar.list();
        let session = listed.iter().find(|cookie| cookie.name == "session").unwrap();
        assert_eq!((session.domain.as_str(), session.host_only, session.http_only, session.expires), ("api.lab.test", true, true, None));
        let pref = listed.iter().find(|cookie| cookie.name == "pref").unwrap();
        assert_eq!((pref.domain.as_str(), pref.host_only, pref.path.as_str()), ("lab.test", false, "/ui"));

        // An expiry in the past removes a cookie; Max-Age in the future keeps it, dated.
        set(&jar, "http://api.lab.test/logout", &["session=; Max-Age=0; Path=/", "remember=1; Max-Age=3600; Path=/"]);
        let listed = jar.list();
        assert!(!listed.iter().any(|cookie| cookie.name == "session"), "logged out");
        assert!(listed.iter().find(|cookie| cookie.name == "remember").unwrap().expires.is_some());
        jar.clear();
        assert!(jar.list().is_empty() && jar.header_for("http://api.lab.test/").is_none());
    }
}
