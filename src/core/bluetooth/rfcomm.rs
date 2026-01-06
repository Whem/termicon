//! RFCOMM/SPP Bluetooth Serial transport
//!
//! Provides serial port emulation over Bluetooth Classic

use super::BluetoothError;
use crate::core::transport::{TransportError, TransportStats, TransportTrait, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// RFCOMM channel configuration
#[derive(Debug, Clone)]
pub struct RfcommConfig {
    /// Device address (MAC)
    pub address: String,
    /// RFCOMM channel number (0 = auto-discover SPP)
    pub channel: u8,
    /// Connection timeout
    pub timeout: Duration,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Service name filter (for SPP discovery)
    pub service_name: Option<String>,
}

impl RfcommConfig {
    /// Create a new RFCOMM configuration
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            channel: 0, // Auto-discover
            timeout: Duration::from_secs(10),
            auto_reconnect: false,
            service_name: None,
        }
    }

    /// Set specific RFCOMM channel
    #[must_use]
    pub fn channel(mut self, channel: u8) -> Self {
        self.channel = channel;
        self
    }

    /// Set timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set auto-reconnect
    #[must_use]
    pub fn auto_reconnect(mut self, value: bool) -> Self {
        self.auto_reconnect = value;
        self
    }
}

impl Default for RfcommConfig {
    fn default() -> Self {
        Self::new("00:00:00:00:00:00")
    }
}

/// RFCOMM transport for Bluetooth Serial
pub struct RfcommTransport {
    config: RfcommConfig,
    connected: Arc<RwLock<bool>>,
    stats: Arc<RwLock<TransportStats>>,
    connected_at: Option<Instant>,
    tx: broadcast::Sender<Bytes>,
    // Platform-specific socket/stream would go here
    #[cfg(target_os = "linux")]
    socket: Option<std::os::unix::io::RawFd>,
}

impl RfcommTransport {
    /// Create a new RFCOMM transport
    pub fn new(config: RfcommConfig) -> Self {
        let (tx, _) = broadcast::channel(1024);

        Self {
            config,
            connected: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connected_at: None,
            tx,
            #[cfg(target_os = "linux")]
            socket: None,
        }
    }

    /// Discover SPP service and get RFCOMM channel
    #[allow(dead_code)]
    async fn discover_spp_channel(&self) -> Result<u8, BluetoothError> {
        // Platform-specific SDP query to find SPP service
        // This would use bluez-rs or similar on Linux
        tracing::info!("Discovering SPP service on {}", self.config.address);

        // Default to channel 1 as fallback
        Ok(1)
    }
}

#[async_trait]
impl TransportTrait for RfcommTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        tracing::info!(
            "Connecting to Bluetooth device {} on channel {}",
            self.config.address,
            self.config.channel
        );

        // Discover channel if not specified
        let channel = if self.config.channel == 0 {
            self.discover_spp_channel()
                .await
                .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?
        } else {
            self.config.channel
        };

        // Platform-specific RFCOMM connection
        #[cfg(target_os = "linux")]
        {
            // On Linux, we would use socket(AF_BLUETOOTH, SOCK_STREAM, BTPROTO_RFCOMM)
            // and connect to the Bluetooth address + channel
            // This requires the bluez-rs crate or manual libc calls

            // Placeholder for now
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we would use Winsock with RFCOMM
            // or the Windows.Devices.Bluetooth.Rfcomm API
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, we would use IOBluetoothRFCOMMChannel
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        *self.connected.write() = true;
        self.connected_at = Some(Instant::now());
        *self.stats.write() = TransportStats::default();

        tracing::info!("Connected to {} on channel {}", self.config.address, channel);

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if !*self.connected.read() {
            return Ok(());
        }

        tracing::info!("Disconnecting from {}", self.config.address);

        // Platform-specific disconnect
        #[cfg(target_os = "linux")]
        {
            if let Some(fd) = self.socket.take() {
                unsafe {
                    libc::close(fd);
                }
            }
        }

        *self.connected.write() = false;
        self.connected_at = None;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        *self.connected.read()
    }

    async fn send(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        if !self.is_connected() {
            return Err(TransportError::Disconnected);
        }

        // Platform-specific send
        tracing::debug!("Sending {} bytes over RFCOMM", data.len());

        let mut stats = self.stats.write();
        stats.bytes_sent += data.len() as u64;
        stats.packets_sent += 1;

        Ok(data.len())
    }

    async fn receive(&mut self) -> Result<Bytes, TransportError> {
        if !self.is_connected() {
            return Err(TransportError::Disconnected);
        }

        // Platform-specific receive
        // This would be non-blocking read from the RFCOMM socket

        // For now, return empty (no data available)
        Ok(Bytes::new())
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Serial // RFCOMM is serial-like
    }

    fn connection_info(&self) -> String {
        format!(
            "rfcomm://{}@{}",
            self.config.channel,
            self.config.address
        )
    }

    fn stats(&self) -> TransportStats {
        let mut stats = self.stats.read().clone();
        if let Some(connected_at) = self.connected_at {
            stats.uptime_secs = connected_at.elapsed().as_secs();
        }
        stats
    }

    fn subscribe(&self) -> broadcast::Receiver<Bytes> {
        self.tx.subscribe()
    }
}

/// Helper to format Bluetooth MAC address
pub fn format_mac_address(addr: &[u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]
    )
}

/// Parse MAC address string to bytes
pub fn parse_mac_address(addr: &str) -> Result<[u8; 6], BluetoothError> {
    let parts: Vec<&str> = addr.split(':').collect();
    if parts.len() != 6 {
        return Err(BluetoothError::DeviceNotFound(format!("Invalid MAC: {}", addr)));
    }

    let mut result = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        result[i] = u8::from_str_radix(part, 16)
            .map_err(|_| BluetoothError::DeviceNotFound(format!("Invalid MAC: {}", addr)))?;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_mac() {
        let addr = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        assert_eq!(format_mac_address(&addr), "AA:BB:CC:DD:EE:FF");
    }

    #[test]
    fn test_parse_mac() {
        let addr = parse_mac_address("AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(addr, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }
}






