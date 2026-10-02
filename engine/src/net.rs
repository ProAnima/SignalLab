//! Small network helpers: host info for the UI header, what a UDP receive
//! error means, and why a local socket could not be opened.

use std::io;

use serde::Serialize;
use tokio::net::UdpSocket;

use super::error::EngineError;
use super::transport::{self, Cause};

/// Whether a UDP receive error is only news about an earlier send, with the
/// socket itself fine: Windows reports an ICMP "port unreachable" as
/// `ConnectionReset` on the next receive (WSAECONNRESET), and a connected
/// socket elsewhere as `ConnectionRefused`. A receive loop carries on; any
/// other error ends it.
pub fn udp_transient(error: &io::Error) -> bool {
    matches!(error.kind(), io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionRefused)
}

/// Opening a local socket on `bind` failed. A taken port, an address of
/// another machine and a refusal are causes a person can fix; anything else is
/// `wait.bind_failed`. The system's wording is the detail either way.
pub fn bind_error(bind: &str, error: io::Error) -> EngineError {
    match transport::of_io(&error) {
        cause @ (Cause::AddressInUse | Cause::AddressUnavailable | Cause::Denied) => cause.error(bind),
        _ => EngineError::new("wait.bind_failed").with("target", bind),
    }
    .because(error)
}

#[derive(Serialize)]
pub struct HostInfo {
    pub local_ip: String,
    pub hostname: String,
}

/// Discover the primary outbound IPv4 by opening a connected UDP socket toward a
/// public address. No packets are actually sent — this only resolves the local
/// endpoint the OS would use.
pub async fn host_info() -> HostInfo {
    let local_ip = async {
        let sock = UdpSocket::bind("0.0.0.0:0").await.ok()?;
        sock.connect("8.8.8.8:80").await.ok()?;
        sock.local_addr().ok().map(|a| a.ip().to_string())
    }
    .await
    .unwrap_or_else(|| "127.0.0.1".to_string());

    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string());

    HostInfo { local_ip, hostname }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_failures_say_what_to_fix() {
        let taken = bind_error("0.0.0.0:9000", io::Error::new(io::ErrorKind::AddrInUse, "in use"));
        assert_eq!((taken.code.as_str(), taken.params["target"].as_str(), taken.detail.as_deref()), ("transport.address_in_use", "0.0.0.0:9000", Some("in use")));
        assert!(bind_error("10.9.9.9:1", io::Error::new(io::ErrorKind::AddrNotAvailable, "x")).is("transport.address_unavailable"));
        assert!(bind_error("0.0.0.0:80", io::Error::new(io::ErrorKind::PermissionDenied, "x")).is("transport.denied"));
        assert!(bind_error("0.0.0.0:0", io::Error::other("odd")).is("wait.bind_failed"));
    }
}
