//! Transport Capability Registry
//!
//! Provides a unified way to query what features each transport supports.
//! This enables dynamic UI enable/disable based on transport capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Transport capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // Data handling
    /// Supports binary data transmission
    Binary,
    /// Supports text/ASCII only
    TextOnly,
    /// Supports streaming data
    Streaming,
    /// Supports packet/message based communication
    PacketBased,
    
    // Connection features
    /// Supports auto-reconnect
    AutoReconnect,
    /// Supports connection timeout configuration
    ConfigurableTimeout,
    /// Supports keep-alive
    KeepAlive,
    /// Supports multiple simultaneous channels
    MultiChannel,
    
    // Flow control
    /// Supports hardware flow control
    HardwareFlowControl,
    /// Supports software flow control (XON/XOFF)
    SoftwareFlowControl,
    /// Supports break signal
    BreakSignal,
    /// Supports modem control lines (DTR/RTS/CTS/DSR)
    ModemLines,
    
    // Interactive features
    /// Supports interactive shell/terminal
    Interactive,
    /// Supports PTY allocation
    Pty,
    /// Supports terminal resize
    TerminalResize,
    
    // File transfer
    /// Supports file transfer (any protocol)
    FileTransfer,
    /// Supports XMODEM
    Xmodem,
    /// Supports YMODEM
    Ymodem,
    /// Supports ZMODEM
    Zmodem,
    /// Supports SFTP
    Sftp,
    /// Supports SCP
    Scp,
    
    // Security
    /// Supports encryption
    Encrypted,
    /// Supports password authentication
    PasswordAuth,
    /// Supports key-based authentication
    KeyAuth,
    /// Supports agent-based authentication
    AgentAuth,
    /// Supports port forwarding
    PortForwarding,
    
    // Protocol features
    /// Supports Telnet option negotiation
    TelnetOptions,
    /// Supports terminal type negotiation
    TerminalTypeNegotiation,
    /// Supports echo negotiation
    EchoNegotiation,
    
    // Bluetooth specific
    /// Supports BLE GATT
    BleGatt,
    /// Supports BLE notifications
    BleNotify,
    /// Supports BLE scanning
    BleScan,
    /// Supports RFCOMM/SPP
    Rfcomm,
    /// Supports Bluetooth pairing
    BlePairing,
    
    // Device features
    /// Supports baud rate configuration
    BaudRate,
    /// Supports data bits configuration
    DataBits,
    /// Supports parity configuration
    Parity,
    /// Supports stop bits configuration
    StopBits,
}

/// Transport capability set
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
    description: String,
}

impl CapabilitySet {
    /// Create a new empty capability set
    pub fn new() -> Self {
        Self::default()
    }

    /// Create capability set with description
    pub fn with_description(description: &str) -> Self {
        Self {
            capabilities: HashSet::new(),
            description: description.to_string(),
        }
    }

    /// Add a capability
    pub fn add(&mut self, cap: Capability) -> &mut Self {
        self.capabilities.insert(cap);
        self
    }

    /// Add multiple capabilities
    pub fn add_all(&mut self, caps: &[Capability]) -> &mut Self {
        for cap in caps {
            self.capabilities.insert(*cap);
        }
        self
    }

    /// Remove a capability
    pub fn remove(&mut self, cap: Capability) -> &mut Self {
        self.capabilities.remove(&cap);
        self
    }

    /// Check if capability is supported
    pub fn supports(&self, cap: Capability) -> bool {
        self.capabilities.contains(&cap)
    }

    /// Check if all capabilities are supported
    pub fn supports_all(&self, caps: &[Capability]) -> bool {
        caps.iter().all(|c| self.capabilities.contains(c))
    }

    /// Check if any capability is supported
    pub fn supports_any(&self, caps: &[Capability]) -> bool {
        caps.iter().any(|c| self.capabilities.contains(c))
    }

    /// Get all capabilities
    pub fn all(&self) -> &HashSet<Capability> {
        &self.capabilities
    }

    /// Get description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Check if supports binary data
    pub fn is_binary(&self) -> bool {
        self.supports(Capability::Binary)
    }

    /// Check if supports file transfer
    pub fn can_transfer_files(&self) -> bool {
        self.supports(Capability::FileTransfer)
    }

    /// Check if supports interactive terminal
    pub fn is_interactive(&self) -> bool {
        self.supports(Capability::Interactive)
    }

    /// Check if supports reconnection
    pub fn can_reconnect(&self) -> bool {
        self.supports(Capability::AutoReconnect)
    }

    /// Check if encrypted
    pub fn is_encrypted(&self) -> bool {
        self.supports(Capability::Encrypted)
    }
}

/// Capability registry for all transport types
pub struct CapabilityRegistry;

impl CapabilityRegistry {
    /// Get capabilities for Serial transport
    pub fn serial() -> CapabilitySet {
        let mut caps = CapabilitySet::with_description("Serial Port (RS-232/RS-485/USB)");
        caps.add_all(&[
            Capability::Binary,
            Capability::Streaming,
            Capability::AutoReconnect,
            Capability::ConfigurableTimeout,
            Capability::HardwareFlowControl,
            Capability::SoftwareFlowControl,
            Capability::BreakSignal,
            Capability::ModemLines,
            Capability::FileTransfer,
            Capability::Xmodem,
            Capability::Ymodem,
            Capability::Zmodem,
            Capability::BaudRate,
            Capability::DataBits,
            Capability::Parity,
            Capability::StopBits,
        ]);
        caps
    }

    /// Get capabilities for TCP transport
    pub fn tcp() -> CapabilitySet {
        let mut caps = CapabilitySet::with_description("Raw TCP Socket");
        caps.add_all(&[
            Capability::Binary,
            Capability::Streaming,
            Capability::AutoReconnect,
            Capability::ConfigurableTimeout,
            Capability::KeepAlive,
        ]);
        caps
    }

    /// Get capabilities for Telnet transport
    pub fn telnet() -> CapabilitySet {
        let mut caps = CapabilitySet::with_description("Telnet Protocol");
        caps.add_all(&[
            Capability::Binary,
            Capability::Streaming,
            Capability::AutoReconnect,
            Capability::ConfigurableTimeout,
            Capability::Interactive,
            Capability::TelnetOptions,
            Capability::TerminalTypeNegotiation,
            Capability::EchoNegotiation,
        ]);
        caps
    }

    /// Get capabilities for SSH transport
    pub fn ssh() -> CapabilitySet {
        let mut caps = CapabilitySet::with_description("SSH-2 Protocol");
        caps.add_all(&[
            Capability::Binary,
            Capability::Streaming,
            Capability::AutoReconnect,
            Capability::ConfigurableTimeout,
            Capability::KeepAlive,
            Capability::MultiChannel,
            Capability::Interactive,
            Capability::Pty,
            Capability::TerminalResize,
            Capability::FileTransfer,
            Capability::Sftp,
            Capability::Scp,
            Capability::Encrypted,
            Capability::PasswordAuth,
            Capability::KeyAuth,
            Capability::AgentAuth,
            Capability::PortForwarding,
        ]);
        caps
    }

    /// Get capabilities for Bluetooth LE transport
    pub fn bluetooth_le() -> CapabilitySet {
        let mut caps = CapabilitySet::with_description("Bluetooth Low Energy");
        caps.add_all(&[
            Capability::Binary,
            Capability::PacketBased,
            Capability::AutoReconnect,
            Capability::ConfigurableTimeout,
            Capability::BleGatt,
            Capability::BleNotify,
            Capability::BleScan,
            Capability::BlePairing,
        ]);
        caps
    }

    /// Get capabilities for Bluetooth Classic (RFCOMM) transport
    pub fn bluetooth_classic() -> CapabilitySet {
        let mut caps = CapabilitySet::with_description("Bluetooth Classic (RFCOMM/SPP)");
        caps.add_all(&[
            Capability::Binary,
            Capability::Streaming,
            Capability::AutoReconnect,
            Capability::ConfigurableTimeout,
            Capability::Rfcomm,
            Capability::BlePairing,
        ]);
        caps
    }
}

/// Trait for transports to declare their capabilities
pub trait HasCapabilities {
    /// Get the capability set for this transport
    fn capabilities(&self) -> CapabilitySet;
    
    /// Check if a specific capability is supported
    fn supports(&self, cap: Capability) -> bool {
        self.capabilities().supports(cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_capabilities() {
        let caps = CapabilityRegistry::serial();
        assert!(caps.supports(Capability::Binary));
        assert!(caps.supports(Capability::BaudRate));
        assert!(caps.supports(Capability::Xmodem));
        assert!(!caps.supports(Capability::Sftp));
        assert!(!caps.supports(Capability::Encrypted));
    }

    #[test]
    fn test_ssh_capabilities() {
        let caps = CapabilityRegistry::ssh();
        assert!(caps.supports(Capability::Encrypted));
        assert!(caps.supports(Capability::Sftp));
        assert!(caps.supports(Capability::PortForwarding));
        assert!(caps.is_encrypted());
        assert!(caps.can_transfer_files());
    }

    #[test]
    fn test_capability_set_operations() {
        let mut caps = CapabilitySet::new();
        caps.add(Capability::Binary);
        caps.add(Capability::Streaming);
        
        assert!(caps.supports_all(&[Capability::Binary, Capability::Streaming]));
        assert!(!caps.supports_all(&[Capability::Binary, Capability::Encrypted]));
        assert!(caps.supports_any(&[Capability::Binary, Capability::Encrypted]));
    }
}






