//! Why a network operation failed, in terms a person can act on. "Connection
//! refused", "timed out" and "name not found" have different fixes, so each is
//! its own `Cause`; the operating system's wording stays in the error's detail.
//! The experiment engine reports a cause as `transport.<cause>`, and the HTTP
//! screen shows the same localized message for the same cause.

use std::error::Error as _;
use std::io;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use super::error::{EngineError, EngineResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    /// Nothing is listening on that port.
    Refused,
    /// No answer in time.
    Timeout,
    /// The host name does not resolve.
    Dns,
    /// No route to the network or host.
    Unreachable,
    /// The other side closed or reset the connection.
    Reset,
    /// A local port is already taken.
    AddressInUse,
    /// A local address does not belong to this machine.
    AddressUnavailable,
    /// The system or a firewall did not allow it.
    Denied,
    /// HTTPS could not be established (certificate, protocol).
    Tls,
    /// The target is not `IP:port` or not a valid URL.
    TargetInvalid,
    /// Anything else; the detail says what.
    Failed,
}

impl Cause {
    /// The localized error for this cause, about `target`.
    pub fn error(self, target: &str) -> EngineError {
        let error = match self {
            Cause::Refused => EngineError::new("transport.refused"),
            Cause::Timeout => EngineError::new("transport.timeout"),
            Cause::Dns => EngineError::new("transport.dns"),
            Cause::Unreachable => EngineError::new("transport.unreachable"),
            Cause::Reset => EngineError::new("transport.reset"),
            Cause::AddressInUse => EngineError::new("transport.address_in_use"),
            Cause::AddressUnavailable => EngineError::new("transport.address_unavailable"),
            Cause::Denied => EngineError::new("transport.denied"),
            Cause::Tls => EngineError::new("transport.tls"),
            Cause::TargetInvalid => EngineError::new("transport.target_invalid"),
            Cause::Failed => EngineError::new("transport.failed"),
        };
        error.with("target", target)
    }
}

/// Name-resolution failures carry no `ErrorKind` of their own.
fn is_dns(error: &io::Error) -> bool {
    // WSAHOST_NOT_FOUND, WSATRY_AGAIN, WSANO_RECOVERY, WSANO_DATA.
    matches!(error.raw_os_error(), Some(11001..=11004) if cfg!(windows))
        || error.to_string().contains("failed to lookup address")
}

pub fn of_io(error: &io::Error) -> Cause {
    use io::ErrorKind::*;
    match error.kind() {
        ConnectionRefused => Cause::Refused,
        TimedOut => Cause::Timeout,
        HostUnreachable | NetworkUnreachable | NetworkDown => Cause::Unreachable,
        ConnectionReset | ConnectionAborted | BrokenPipe | UnexpectedEof => Cause::Reset,
        AddrInUse => Cause::AddressInUse,
        AddrNotAvailable => Cause::AddressUnavailable,
        PermissionDenied => Cause::Denied,
        _ if is_dns(error) => Cause::Dns,
        InvalidInput => Cause::TargetInvalid,
        _ => Cause::Failed,
    }
}

/// reqwest wraps the interesting part several layers deep (hyper, the
/// connector, the socket); the first layer that says something specific wins.
pub fn of_reqwest(error: &reqwest::Error) -> Cause {
    if error.is_timeout() {
        return Cause::Timeout;
    }
    if error.is_builder() {
        return Cause::TargetInvalid;
    }
    let mut source = error.source();
    while let Some(inner) = source {
        if let Some(io) = inner.downcast_ref::<io::Error>() {
            match of_io(io) {
                Cause::Failed => {}
                cause => return cause,
            }
        }
        let text = inner.to_string().to_ascii_lowercase();
        if text.starts_with("dns error") {
            return Cause::Dns;
        }
        if text.contains("certificate") || text.contains("tls") || text.contains("ssl") {
            return Cause::Tls;
        }
        source = inner.source();
    }
    Cause::Failed
}

/// An error and its sources as one line, without repeating a layer that only
/// restates the one below it.
pub fn chain(error: &dyn std::error::Error) -> String {
    let mut parts: Vec<String> = vec![error.to_string()];
    let mut source = error.source();
    while let Some(inner) = source {
        let text = inner.to_string();
        if !parts.last().is_some_and(|last| last.contains(&text)) {
            parts.push(text);
        }
        source = inner.source();
    }
    parts.join(": ")
}

/// `IP:port`, or `host:port` resolved through DNS, to one address.
pub async fn resolve(spec: &str) -> EngineResult<SocketAddr> {
    let spec = spec.trim();
    if let Ok(address) = spec.parse::<SocketAddr>() {
        return Ok(address);
    }
    let port_ok = spec.rsplit_once(':').is_some_and(|(host, port)| !host.is_empty() && port.parse::<u16>().is_ok());
    if !port_ok {
        return Err(Cause::TargetInvalid.error(spec));
    }
    match tokio::net::lookup_host(spec).await {
        Ok(mut found) => found.next().ok_or_else(|| Cause::Dns.error(spec)),
        Err(error) => {
            let cause = match of_io(&error) {
                Cause::Failed | Cause::TargetInvalid => Cause::Dns,
                cause => cause,
            };
            Err(cause.error(spec).because(error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_kinds_map_to_causes() {
        for (kind, cause) in [
            (io::ErrorKind::ConnectionRefused, Cause::Refused),
            (io::ErrorKind::TimedOut, Cause::Timeout),
            (io::ErrorKind::AddrInUse, Cause::AddressInUse),
            (io::ErrorKind::AddrNotAvailable, Cause::AddressUnavailable),
            (io::ErrorKind::ConnectionReset, Cause::Reset),
            (io::ErrorKind::PermissionDenied, Cause::Denied),
            (io::ErrorKind::Other, Cause::Failed),
        ] {
            assert_eq!(of_io(&io::Error::new(kind, "x")), cause, "{kind:?}");
        }
        let error = Cause::Refused.error("127.0.0.1:9");
        assert_eq!((error.code.as_str(), error.params["target"].as_str()), ("transport.refused", "127.0.0.1:9"));
    }

    #[tokio::test]
    async fn targets_resolve_or_say_why_not() {
        assert_eq!(resolve(" 127.0.0.1:9000 ").await.unwrap(), "127.0.0.1:9000".parse().unwrap());
        assert!(resolve("localhost:9000").await.unwrap().ip().is_loopback());
        for bad in ["127.0.0.1", "nonsense", ":9000", "host:99999"] {
            assert!(resolve(bad).await.unwrap_err().is("transport.target_invalid"), "{bad}");
        }
        // `.invalid` is reserved and never resolves (RFC 6761).
        assert!(resolve("no-such-host.invalid:9000").await.unwrap_err().is("transport.dns"));
    }

    #[tokio::test]
    async fn refused_and_timed_out_http_requests_are_told_apart() {
        // A port nothing listens on: bind one, note it, close it.
        let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let client = reqwest::Client::builder().timeout(std::time::Duration::from_millis(8000)).build().unwrap();
        let refused = client.get(format!("http://127.0.0.1:{port}/")).send().await.unwrap_err();
        assert_eq!(of_reqwest(&refused), Cause::Refused, "{}", chain(&refused));
        assert!(chain(&refused).len() > refused.to_string().len(), "the chain names the socket error");

        // Accepts the connection, never answers.
        let silent = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", silent.local_addr().unwrap());
        let quick = reqwest::Client::builder().timeout(std::time::Duration::from_millis(150)).build().unwrap();
        let timeout = quick.get(url).send().await.unwrap_err();
        assert_eq!(of_reqwest(&timeout), Cause::Timeout);
        assert_eq!(of_reqwest(&client.get("http://[::1").send().await.unwrap_err()), Cause::TargetInvalid);
    }
}
