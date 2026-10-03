//! The authentication a request carries: Basic, Bearer, or Digest (RFC 7616 —
//! MD5 and SHA-256, their `-sess` variants, qop `auth` and `auth-int`, and
//! RFC 2069's answer without qop). Credentials go into an `Authorization`
//! header added as the request is sent; nothing that reports a request —
//! Inspector frames, steps, reports — carries it.

use std::sync::Mutex;

use base64::Engine as _;
use md5::{Digest as _, Md5};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use super::error::{EngineError, EngineResult};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scheme", rename_all = "snake_case")]
pub enum Auth {
    #[default]
    None,
    Basic {
        username: String,
        #[serde(default)]
        password: String,
    },
    Bearer {
        token: String,
    },
    Digest {
        username: String,
        #[serde(default)]
        password: String,
    },
}

impl Auth {
    pub fn is_none(&self) -> bool {
        matches!(self, Auth::None)
    }
}

/// `Basic dXNlcjpwYXNz`.
pub fn basic(username: &str, password: &str) -> String {
    format!("Basic {}", basic_credentials(username, password))
}

/// `dXNlcjpwYXNz`: what Basic sends of a name and password — a secret in
/// either is masked in this form too.
pub fn basic_credentials(username: &str, password: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    Md5,
    Sha256,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qop {
    Auth,
    AuthInt,
}

/// A Digest challenge, as a server's `WWW-Authenticate` gave it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub realm: String,
    pub nonce: String,
    pub opaque: Option<String>,
    pub algorithm: Algorithm,
    /// `MD5-sess`, `SHA-256-sess`: the first hash includes the nonces.
    pub sess: bool,
    /// `None`: RFC 2069, an answer without qop.
    pub qop: Option<Qop>,
    /// The nonce ran out; the credentials were right.
    pub stale: bool,
}

/// `scheme param=value, param="quoted, value"` — every challenge of one header value.
fn challenges(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let chars: Vec<char> = text.chars().collect();
    let mut found: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let mut at = 0;
    let token_char = |c: char| !c.is_whitespace() && !matches!(c, ',' | '=' | '"');
    while at < chars.len() {
        while at < chars.len() && (chars[at].is_whitespace() || chars[at] == ',') {
            at += 1;
        }
        let start = at;
        while at < chars.len() && token_char(chars[at]) {
            at += 1;
        }
        if start == at {
            at += 1;
            continue;
        }
        let token: String = chars[start..at].iter().collect();
        let mut look = at;
        while look < chars.len() && chars[look].is_whitespace() {
            look += 1;
        }
        if look < chars.len() && chars[look] == '=' {
            // A parameter of the current challenge.
            at = look + 1;
            while at < chars.len() && chars[at].is_whitespace() {
                at += 1;
            }
            let mut value = String::new();
            if at < chars.len() && chars[at] == '"' {
                at += 1;
                while at < chars.len() && chars[at] != '"' {
                    if chars[at] == '\\' && at + 1 < chars.len() {
                        at += 1;
                    }
                    value.push(chars[at]);
                    at += 1;
                }
                at += 1;
            } else {
                while at < chars.len() && chars[at] != ',' && !chars[at].is_whitespace() {
                    value.push(chars[at]);
                    at += 1;
                }
            }
            if let Some((_, params)) = found.last_mut() {
                params.push((token.to_ascii_lowercase(), value));
            }
        } else if let Some(qop) = found.last_mut().and_then(|(_, params)| params.last_mut()).filter(|(key, _)| key == "qop").filter(|_| {
            // `qop=auth,auth-int` unquoted, as some servers write it: the list goes on.
            token.eq_ignore_ascii_case("auth") || token.eq_ignore_ascii_case("auth-int")
        }) {
            qop.1.push(',');
            qop.1.push_str(&token);
        } else {
            found.push((token, Vec::new()));
        }
    }
    found
}

/// The Digest challenge among a response's `WWW-Authenticate` values — the
/// strongest one, when it offers several. `Ok(None)`: it asks for no Digest.
pub fn digest_challenge<'a>(values: impl IntoIterator<Item = &'a str>) -> EngineResult<Option<Challenge>> {
    let mut best: Option<Challenge> = None;
    let mut refused: Option<EngineError> = None;
    for value in values {
        for (scheme, params) in challenges(value) {
            if !scheme.eq_ignore_ascii_case("digest") {
                continue;
            }
            let param = |name: &str| params.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone());
            let named = param("algorithm").unwrap_or_else(|| "MD5".into());
            let (algorithm, sess) = match named.to_ascii_uppercase().as_str() {
                "MD5" => (Algorithm::Md5, false),
                "MD5-SESS" => (Algorithm::Md5, true),
                "SHA-256" => (Algorithm::Sha256, false),
                "SHA-256-SESS" => (Algorithm::Sha256, true),
                _ => {
                    refused = Some(EngineError::new("http.digest_unsupported").with("algorithm", &named));
                    continue;
                }
            };
            let qop = match param("qop") {
                None => None,
                Some(offered) => {
                    let offered: Vec<String> = offered.split(',').map(|qop| qop.trim().to_ascii_lowercase()).collect();
                    if offered.iter().any(|qop| qop == "auth") {
                        Some(Qop::Auth)
                    } else if offered.iter().any(|qop| qop == "auth-int") {
                        Some(Qop::AuthInt)
                    } else {
                        refused = Some(EngineError::new("http.digest_unsupported").with("algorithm", format!("qop={}", offered.join(","))));
                        continue;
                    }
                }
            };
            let (Some(realm), Some(nonce)) = (param("realm"), param("nonce")) else {
                refused = Some(EngineError::new("http.digest_invalid"));
                continue;
            };
            let challenge = Challenge {
                realm,
                nonce,
                opaque: param("opaque"),
                algorithm,
                sess,
                qop,
                stale: param("stale").is_some_and(|stale| stale.eq_ignore_ascii_case("true")),
            };
            // SHA-256 over MD5 when the server offers both (RFC 7616, section 3.7).
            if best.as_ref().is_none_or(|best| best.algorithm == Algorithm::Md5 && challenge.algorithm == Algorithm::Sha256) {
                best = Some(challenge);
            }
        }
    }
    match (best, refused) {
        (Some(challenge), _) => Ok(Some(challenge)),
        (None, Some(error)) => Err(error),
        (None, None) => Ok(None),
    }
}

fn hash(algorithm: Algorithm, data: &[u8]) -> String {
    let bytes: Vec<u8> = match algorithm {
        Algorithm::Md5 => Md5::digest(data).to_vec(),
        Algorithm::Sha256 => Sha256::digest(data).to_vec(),
    };
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The `Authorization` value answering `challenge` for one request: `uri` is
/// the request target (path and query), `nc` counts the requests made with
/// this nonce (from 1), `body` is hashed for `auth-int`.
#[allow(clippy::too_many_arguments)]
pub fn authorization(challenge: &Challenge, username: &str, password: &str, method: &str, uri: &str, nc: u32, cnonce: &str, body: &[u8]) -> String {
    let h = |text: &str| hash(challenge.algorithm, text.as_bytes());
    let mut ha1 = h(&format!("{username}:{}:{password}", challenge.realm));
    if challenge.sess {
        ha1 = h(&format!("{ha1}:{}:{cnonce}", challenge.nonce));
    }
    let ha2 = match challenge.qop {
        Some(Qop::AuthInt) => h(&format!("{method}:{uri}:{}", hash(challenge.algorithm, body))),
        _ => h(&format!("{method}:{uri}")),
    };
    let nc = format!("{nc:08x}");
    let qop = challenge.qop.map(|qop| if qop == Qop::Auth { "auth" } else { "auth-int" });
    let response = match qop {
        Some(qop) => h(&format!("{ha1}:{}:{nc}:{cnonce}:{qop}:{ha2}", challenge.nonce)),
        None => h(&format!("{ha1}:{}:{ha2}", challenge.nonce)),
    };
    let algorithm = match (challenge.algorithm, challenge.sess) {
        (Algorithm::Md5, false) => "MD5",
        (Algorithm::Md5, true) => "MD5-sess",
        (Algorithm::Sha256, false) => "SHA-256",
        (Algorithm::Sha256, true) => "SHA-256-sess",
    };
    let mut header = format!(
        "Digest username={}, realm={}, nonce={}, uri={}, algorithm={algorithm}, response={}",
        quoted(username),
        quoted(&challenge.realm),
        quoted(&challenge.nonce),
        quoted(uri),
        quoted(&response)
    );
    if let Some(qop) = qop {
        header.push_str(&format!(", qop={qop}, nc={nc}, cnonce={}", quoted(cnonce)));
    }
    if let Some(opaque) = &challenge.opaque {
        header.push_str(&format!(", opaque={}", quoted(opaque)));
    }
    header
}

/// A client nonce: 16 random bytes, hex.
pub fn cnonce() -> String {
    (0..16).map(|_| format!("{:02x}", rand::random::<u8>())).collect()
}

/// One answer to a challenge: this request's count of its nonce and the client nonce.
#[derive(Clone, Debug)]
pub struct Answering {
    pub challenge: Challenge,
    pub nc: u32,
    pub cnonce: String,
}

impl Answering {
    pub fn header(&self, username: &str, password: &str, method: &str, uri: &str, body: &[u8]) -> String {
        authorization(&self.challenge, username, password, method, uri, self.nc, &self.cnonce, body)
    }
}

/// The Digest challenge of one origin, remembered across the requests of a
/// burst so each one after the first answers it at once — one round trip, as
/// a browser does. Every answer to one nonce counts on with one client nonce
/// (which `-sess` hashes into its first hash), so requests in flight together
/// that meet the same new nonce never send the same count twice.
#[derive(Default)]
pub struct DigestMemory(Mutex<Option<Remembered>>);

struct Remembered {
    origin: String,
    challenge: Challenge,
    nc: u32,
    cnonce: String,
}

impl Remembered {
    fn answer(&mut self) -> Answering {
        self.nc += 1;
        Answering { challenge: self.challenge.clone(), nc: self.nc, cnonce: self.cnonce.clone() }
    }
}

impl DigestMemory {
    /// The next answer to what `origin` asked last, if it asked.
    pub fn next(&self, origin: &str) -> Option<Answering> {
        let mut held = self.0.lock().unwrap();
        Some(held.as_mut().filter(|held| held.origin == origin)?.answer())
    }

    /// The answer to a challenge `origin` just gave: its nonce counted on when
    /// it is the one held, else counted from 1 with a new client nonce.
    pub fn remember(&self, origin: &str, challenge: Challenge) -> Answering {
        let mut held = self.0.lock().unwrap();
        if let Some(same) = held.as_mut().filter(|held| held.origin == origin && held.challenge.realm == challenge.realm && held.challenge.nonce == challenge.nonce) {
            same.challenge = challenge;
            return same.answer();
        }
        let fresh = held.insert(Remembered { origin: origin.into(), challenge, nc: 0, cnonce: cnonce() });
        fresh.answer()
    }

    /// An answer to `nonce` was refused: forgotten, unless a newer one is held by now.
    pub fn forget(&self, origin: &str, nonce: &str) {
        let mut held = self.0.lock().unwrap();
        if held.as_ref().is_some_and(|held| held.origin == origin && held.challenge.nonce == nonce) {
            *held = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_2617_and_7616_examples_give_their_answers() {
        // RFC 2617, section 3.5: MD5, qop=auth.
        let rfc2617 = Challenge {
            realm: "testrealm@host.com".into(),
            nonce: "dcd98b7102dd2f0e8b11d0f600bfb0c093".into(),
            opaque: Some("5ccc069c403ebaf9f0171e9517f40e41".into()),
            algorithm: Algorithm::Md5,
            sess: false,
            qop: Some(Qop::Auth),
            stale: false,
        };
        let header = authorization(&rfc2617, "Mufasa", "Circle Of Life", "GET", "/dir/index.html", 1, "0a4f113b", b"");
        assert!(header.contains("response=\"6629fae49393a05397450978507c4ef1\""), "{header}");
        assert!(header.contains("nc=00000001") && header.contains("qop=auth,") && header.contains("opaque=\"5ccc069c403ebaf9f0171e9517f40e41\""), "{header}");

        // RFC 7616, section 3.9.1: the same request with SHA-256, then with MD5.
        let rfc7616 = Challenge {
            realm: "http-auth@example.org".into(),
            nonce: "7ypf/xlj9XXwfDPEoM4URrv/xwf94BcCAzFZH4GiTo0v".into(),
            opaque: Some("FQhe/qaU925kfnzjCev0ciny7QMkPqMAFRtzCUYo5tdS".into()),
            algorithm: Algorithm::Sha256,
            sess: false,
            qop: Some(Qop::Auth),
            stale: false,
        };
        let cnonce = "f2/wE4q74E6zIJEtWaHKaf5wv/H5QzzpXusqGemxURZJ";
        let sha = authorization(&rfc7616, "Mufasa", "Circle of Life", "GET", "/dir/index.html", 1, cnonce, b"");
        assert!(sha.contains("response=\"753927fa0e85d155564e2e272a28d1802ca10daf4496794697cf8db5856cb6c1\"") && sha.contains("algorithm=SHA-256,"), "{sha}");
        let md5 = authorization(&Challenge { algorithm: Algorithm::Md5, ..rfc7616 }, "Mufasa", "Circle of Life", "GET", "/dir/index.html", 1, cnonce, b"");
        assert!(md5.contains("response=\"8ca523f5e9506fed4657c9700eebdbec\""), "{md5}");
    }

    #[test]
    fn sess_auth_int_and_no_qop_hash_what_they_should() {
        let base = Challenge { realm: "r".into(), nonce: "n".into(), opaque: None, algorithm: Algorithm::Md5, sess: false, qop: None, stale: false };
        let h = |text: &str| hash(Algorithm::Md5, text.as_bytes());
        // RFC 2069: H(HA1:nonce:HA2), no qop, nc or cnonce in the header.
        let legacy = authorization(&base, "u", "p", "GET", "/x", 1, "c", b"");
        let expected = h(&format!("{}:n:{}", h("u:r:p"), h("GET:/x")));
        assert!(legacy.contains(&format!("response=\"{expected}\"")) && !legacy.contains("qop") && !legacy.contains("cnonce"), "{legacy}");
        // -sess: HA1 = H(H(user:realm:pass):nonce:cnonce); auth-int: HA2 hashes the body too.
        let both = authorization(&Challenge { sess: true, qop: Some(Qop::AuthInt), ..base.clone() }, "u", "p", "POST", "/x", 2, "c", b"{}");
        let ha1 = h(&format!("{}:n:c", h("u:r:p")));
        let ha2 = h(&format!("POST:/x:{}", h("{}")));
        let expected = h(&format!("{ha1}:n:00000002:c:auth-int:{ha2}"));
        assert!(both.contains(&format!("response=\"{expected}\"")) && both.contains("algorithm=MD5-sess,") && both.contains("nc=00000002"), "{both}");
        // Quotes and backslashes in a name are escaped.
        assert!(authorization(&base, "a\"b\\c", "p", "GET", "/", 1, "c", b"").starts_with("Digest username=\"a\\\"b\\\\c\""));
    }

    #[test]
    fn challenges_are_read_from_however_the_server_writes_them() {
        let one = digest_challenge(["Digest realm=\"lab, inc\", nonce=\"abc\", qop=\"auth,auth-int\", opaque=xyz, algorithm=MD5-sess"]).unwrap().unwrap();
        assert_eq!((one.realm.as_str(), one.nonce.as_str(), one.opaque.as_deref(), one.algorithm, one.sess, one.qop), ("lab, inc", "abc", Some("xyz"), Algorithm::Md5, true, Some(Qop::Auth)));
        // Several challenges in one value, several values: the strongest Digest wins.
        let many = digest_challenge(["Basic realm=\"x\", Digest realm=\"r\", nonce=\"1\", algorithm=MD5", "DIGEST realm=\"r\", nonce=\"2\", algorithm=SHA-256, stale=TRUE"]).unwrap().unwrap();
        assert_eq!((many.nonce.as_str(), many.algorithm, many.stale), ("2", Algorithm::Sha256, true));
        assert_eq!(digest_challenge(["Basic realm=\"only\"", "Bearer"]).unwrap(), None, "no Digest asked for");
        assert!(digest_challenge(["Digest realm=\"r\", nonce=\"1\", algorithm=SHA-512-256"]).unwrap_err().is("http.digest_unsupported"));
        assert!(digest_challenge(["Digest realm=\"r\""]).unwrap_err().is("http.digest_invalid"), "no nonce");
        let escaped = digest_challenge([r#"Digest realm="a \"quoted\" realm", nonce="n""#]).unwrap().unwrap();
        assert_eq!(escaped.realm, "a \"quoted\" realm");
        // A qop list some servers leave unquoted is still one list, and what follows it still counts.
        let bare = digest_challenge(["Digest realm=\"r\", qop=auth-int,auth, nonce=\"n\", algorithm=SHA-256"]).unwrap().unwrap();
        assert_eq!((bare.qop, bare.nonce.as_str(), bare.algorithm), (Some(Qop::Auth), "n", Algorithm::Sha256));
        let then = digest_challenge(["Digest realm=\"r\", nonce=\"n\", qop=auth-int, Basic realm=\"b\""]).unwrap().unwrap();
        assert_eq!(then.qop, Some(Qop::AuthInt), "Basic after it is a challenge of its own");
    }

    #[test]
    fn basic_is_base64_of_name_and_password_and_memory_counts_uses() {
        assert_eq!(basic("Aladdin", "open sesame"), "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
        const LAB: &str = "http://127.0.0.1:8080";
        let memory = DigestMemory::default();
        assert!(memory.next(LAB).is_none());
        let challenge = digest_challenge(["Digest realm=\"r\", nonce=\"n\""]).unwrap().unwrap();
        let first = memory.remember(LAB, challenge.clone());
        assert_eq!(first.nc, 1);
        assert!(memory.next("http://127.0.0.1:8081").is_none(), "another origin is not answered");
        assert_eq!(memory.next(LAB).map(|answer| answer.nc), Some(2));
        // Requests in flight together meet the same new nonce: they count on, one client nonce.
        let again = memory.remember(LAB, challenge.clone());
        assert_eq!((again.nc, again.cnonce.as_str()), (3, first.cnonce.as_str()));
        let stale = digest_challenge(["Digest realm=\"r\", nonce=\"m\", stale=true"]).unwrap().unwrap();
        let fresh = memory.remember(LAB, stale);
        assert!(fresh.nc == 1 && fresh.cnonce != first.cnonce, "a new nonce counts from 1");
        memory.forget(LAB, "n");
        assert_eq!(memory.next(LAB).map(|answer| answer.nc), Some(2), "a refusal of the old nonce leaves the new one");
        memory.forget(LAB, "m");
        assert!(memory.next(LAB).is_none());
        assert_eq!(cnonce().len(), 32);
        assert_eq!(serde_json::to_value(Auth::Digest { username: "u".into(), password: "p".into() }).unwrap(), serde_json::json!({ "scheme": "digest", "username": "u", "password": "p" }));
        assert!(serde_json::from_value::<Auth>(serde_json::json!({ "scheme": "none" })).unwrap().is_none());
    }
}
