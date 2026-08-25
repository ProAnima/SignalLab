//! Small host-info helpers for the UI header.

use serde::Serialize;
use tokio::net::UdpSocket;

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
