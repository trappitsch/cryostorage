//! This module holds connection adapters, i.e., how we connect to different instruments.
//!
//! All adapters must be ser/de compliant to be saved in the program config file.

use std::{net::Ipv4Addr, time::Duration};

use serde::{Deserialize, Serialize};

/// Timeout for TCP/IP connections
pub const TCP_IP_TIMEOUT: Duration = Duration::from_millis(500);

/// A TCP/IP adapter that connects to instruments via the Moxa serial device server.
///
/// Only IP and port are needed as the Moxa handles the serial connection internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpIpAdapter {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl TcpIpAdapter {
    /// Create a new TCP/IP adapter from IP and port given in one string.
    ///
    /// If invalid format is provided, the default adapter is returned.
    pub fn new_from_str(addr: &str) -> Self {
        let parts = addr.split(':').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Self::default();
        }

        let ip = parts[0]
            .parse::<Ipv4Addr>()
            .unwrap_or(Ipv4Addr::new(192, 168, 1, 2));
        let port = parts[1].parse::<u16>().unwrap_or(4001);

        Self { ip, port }
    }
    /// Get the simple address string to use with `InstrumentRs`.
    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

impl Default for TcpIpAdapter {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::new(192, 168, 1, 2),
            port: 4001,
        }
    }
}

/// A serial adapter that connects to the instrument via a local serial port.
///
/// Here, we only need a port name as baud rate and other instrument specific settings are handled
/// in Instrumentrs.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SerialAdapter {
    pub port_name: String,
}

impl SerialAdapter {
    /// Get the simple address string to use with `InstrumentRs`.
    pub fn get_address(&self) -> String {
        self.port_name.clone()
    }
}

/// The adapter for connecting to the poststation server.
///
/// A combination of the TCP/IP adapter and a serial number of the device.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PoststationAdapter {
    pub serial_number: u64,
    pub tcp_ip: TcpIpAdapter,
}

impl PoststationAdapter {
    /// Get the address string to connect to.
    pub fn get_address(&self) -> String {
        self.tcp_ip.get_address()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tcp_ip_adapter_address() {
        let adapter = TcpIpAdapter {
            ip: Ipv4Addr::new(192, 168, 1, 100),
            port: 4001,
        };
        assert_eq!(adapter.get_address(), "192.168.1.100:4001");
    }
}
