//! Transport layer for different connection types
//!
//! Supports:
//! - Serial ports (RS-232, RS-485, USB-Serial)
//! - Raw TCP connections
//! - Telnet protocol
//! - SSH-2 protocol
//! - Bluetooth (BLE and SPP)

mod bluetooth;
mod serial;
mod ssh;
mod tcp;
mod telnet;

pub use bluetooth::{
    BleServiceConfig, BluetoothConfig, BluetoothDevice, BluetoothScanner, BluetoothTransport,
    BluetoothType, GattBrowser, GattCharacteristic, GattService,
};
pub use serial::{SerialConfig, SerialFlowControl, SerialParity, SerialTransport};
pub use ssh::{PortForward, PortForwardType, SftpClient, SshAuth, SshConfig, SshTransport};
pub use tcp::{TcpConfig, TcpTransport};
pub use telnet::{TelnetConfig, TelnetTransport};

use async_trait::async_trait;
use bytes::Bytes;
use std::fmt;
use thiserror::Error;
use tokio::sync::broadcast;

/// Transport type enumeration
#[derive(Debug, Clone)]
pub enum Transport {
    /// Serial port connection
    Serial(SerialConfig),
    /// Raw TCP connection
    Tcp(TcpConfig),
    /// Telnet connection
    Telnet(TelnetConfig),
    /// SSH connection
    Ssh(SshConfig),
    /// Bluetooth connection
    Bluetooth(BluetoothConfig),
}

/// Transport type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportType {
    /// Serial port
    Serial,
    /// Raw TCP
    Tcp,
    /// Telnet
    Telnet,
    /// SSH
    Ssh,
    /// Bluetooth
    Bluetooth,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serial => write!(f, "Serial"),
            Self::Tcp => write!(f, "TCP"),
            Self::Telnet => write!(f, "Telnet"),
            Self::Ssh => write!(f, "SSH"),
            Self::Bluetooth => write!(f, "Bluetooth"),
        }
    }
}

/// Transport error types
#[derive(Error, Debug)]
pub enum TransportError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Connection timeout
    #[error("Connection timeout after {0} seconds")]
    Timeout(u64),

    /// Port not found
    #[error("Port not found: {0}")]
    PortNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Port already in use
    #[error("Port already in use: {0}")]
    PortInUse(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Not connected
    #[error("Not connected")]
    NotConnected,

    /// Disconnected
    #[error("Disconnected")]
    Disconnected,

    /// Send error
    #[error("Send error: {0}")]
    SendError(String),

    /// Receive error
    #[error("Receive error: {0}")]
    ReceiveError(String),
}

/// Transport statistics
#[derive(Debug, Clone, Default)]
pub struct TransportStats {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Errors count
    pub errors: u64,
    /// Connection uptime in seconds
    pub uptime_secs: u64,
}

/// Transport trait for all connection types
#[async_trait]
pub trait TransportTrait: Send + Sync {
    /// Connect to the target
    async fn connect(&mut self) -> Result<(), TransportError>;

    /// Disconnect from the target
    async fn disconnect(&mut self) -> Result<(), TransportError>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Send data
    async fn send(&mut self, data: &[u8]) -> Result<usize, TransportError>;

    /// Receive data (non-blocking, returns immediately with available data)
    async fn receive(&mut self) -> Result<Bytes, TransportError>;

    /// Get transport type
    fn transport_type(&self) -> TransportType;

    /// Get connection info string
    fn connection_info(&self) -> String;

    /// Get statistics
    fn stats(&self) -> TransportStats;

    /// Subscribe to data events
    fn subscribe(&self) -> broadcast::Receiver<Bytes>;

    /// Send break signal (for serial)
    async fn send_break(&mut self) -> Result<(), TransportError> {
        // Default implementation does nothing
        Ok(())
    }

    /// Set DTR line state (for serial)
    async fn set_dtr(&mut self, _state: bool) -> Result<(), TransportError> {
        Ok(())
    }

    /// Set RTS line state (for serial)
    async fn set_rts(&mut self, _state: bool) -> Result<(), TransportError> {
        Ok(())
    }

    /// Get modem lines state (for serial)
    fn modem_lines(&self) -> Option<ModemLines> {
        None
    }
}

/// Modem control lines state
#[derive(Debug, Clone, Copy, Default)]
pub struct ModemLines {
    /// Data Terminal Ready
    pub dtr: bool,
    /// Request To Send
    pub rts: bool,
    /// Clear To Send
    pub cts: bool,
    /// Data Set Ready
    pub dsr: bool,
    /// Data Carrier Detect
    pub dcd: bool,
    /// Ring Indicator
    pub ri: bool,
}

/// Create a transport instance from configuration
pub async fn create_transport(config: Transport) -> Result<Box<dyn TransportTrait>, TransportError> {
    match config {
        Transport::Serial(cfg) => {
            let transport = SerialTransport::new(cfg)?;
            Ok(Box::new(transport))
        }
        Transport::Tcp(cfg) => {
            let transport = TcpTransport::new(cfg);
            Ok(Box::new(transport))
        }
        Transport::Telnet(cfg) => {
            let transport = TelnetTransport::new(cfg);
            Ok(Box::new(transport))
        }
        Transport::Ssh(cfg) => {
            let transport = SshTransport::new(cfg);
            Ok(Box::new(transport))
        }
        Transport::Bluetooth(cfg) => {
            let transport = BluetoothTransport::new(cfg).await?;
            Ok(Box::new(transport))
        }
    }
}

