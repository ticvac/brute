pub mod friend;
pub mod node;
pub mod watcher;
pub mod leader_snapshot;
pub mod backups;
pub mod backup_watcher;

use std::net::UdpSocket;

/// Get the local IP address by creating a UDP socket to an external address
fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    // Connect to a public IP (doesn't actually send data, just determines route)
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

pub fn parse_address(input: &str) -> String {
    if input.contains(':') {
        input.to_string()
    } else {
        let ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
        format!("{}:{}", ip, input)
    }
}