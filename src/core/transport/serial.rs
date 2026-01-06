//! Serial port transport implementation

use super::{ModemLines, TransportError, TransportStats, TransportTrait, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// Serial port flow control type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SerialFlowControl {
    /// No flow control
    #[default]
    None,
    /// Hardware flow control (RTS/CTS)
    Hardware,
    /// Software flow control (XON/XOFF)
    Software,
}

/// Serial port parity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SerialParity {
    /// No parity
    #[default]
    None,
    /// Odd parity
    Odd,
    /// Even parity
    Even,
}

impl std::str::FromStr for SerialParity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "n" => Ok(Self::None),
            "odd" | "o" => Ok(Self::Odd),
            "even" | "e" => Ok(Self::Even),
            _ => Ok(Self::None),
        }
    }
}

/// Serial port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialConfig {
    /// Port name (e.g., COM3, /dev/ttyUSB0)
    pub port: String,
    /// Baud rate
    pub baud_rate: u32,
    /// Data bits (5, 6, 7, 8)
    pub data_bits: u8,
    /// Stop bits (1, 2)
    pub stop_bits: u8,
    /// Parity
    pub parity: SerialParity,
    /// Flow control
    pub flow_control: SerialFlowControl,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
}

impl SerialConfig {
    /// Create a new serial configuration with default settings
    pub fn new(port: &str, baud_rate: u32) -> Self {
        Self {
            port: port.to_string(),
            baud_rate,
            data_bits: 8,
            stop_bits: 1,
            parity: SerialParity::None,
            flow_control: SerialFlowControl::None,
            auto_reconnect: false,
        }
    }

    /// Set data bits
    #[must_use]
    pub fn data_bits(mut self, bits: u8) -> Self {
        self.data_bits = bits;
        self
    }

    /// Set stop bits
    #[must_use]
    pub fn stop_bits(mut self, bits: u8) -> Self {
        self.stop_bits = bits;
        self
    }

    /// Set parity
    #[must_use]
    pub fn parity(mut self, parity: SerialParity) -> Self {
        self.parity = parity;
        self
    }

    /// Set flow control
    #[must_use]
    pub fn flow_control(mut self, flow: SerialFlowControl) -> Self {
        self.flow_control = flow;
        self
    }

    /// Enable auto-reconnect
    #[must_use]
    pub fn auto_reconnect(mut self, enable: bool) -> Self {
        self.auto_reconnect = enable;
        self
    }
}

impl Default for SerialConfig {
    fn default() -> Self {
        Self::new("COM1", 115200)
    }
}

/// Serial port transport
pub struct SerialTransport {
    config: SerialConfig,
    port: Arc<parking_lot::Mutex<Option<Box<dyn SerialPort + Send>>>>,
    stats: Arc<RwLock<TransportStats>>,
    connected_at: Option<Instant>,
    tx: broadcast::Sender<Bytes>,
    modem_lines: Arc<RwLock<ModemLines>>,
}

impl SerialTransport {
    /// Create a new serial transport
    pub fn new(config: SerialConfig) -> Result<Self, TransportError> {
        let (tx, _) = broadcast::channel(1024);

        Ok(Self {
            config,
            port: Arc::new(parking_lot::Mutex::new(None)),
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connected_at: None,
            tx,
            modem_lines: Arc::new(RwLock::new(ModemLines::default())),
        })
    }

    fn update_modem_lines(&self) {
        let mut port_guard = self.port.lock();
        if let Some(ref mut port) = *port_guard {
            if let Ok(cts) = port.read_clear_to_send() {
                self.modem_lines.write().cts = cts;
            }
            if let Ok(dsr) = port.read_data_set_ready() {
                self.modem_lines.write().dsr = dsr;
            }
            if let Ok(dcd) = port.read_carrier_detect() {
                self.modem_lines.write().dcd = dcd;
            }
            if let Ok(ri) = port.read_ring_indicator() {
                self.modem_lines.write().ri = ri;
            }
        }
    }
}

#[async_trait]
impl TransportTrait for SerialTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        let data_bits = match self.config.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            _ => DataBits::Eight,
        };

        let stop_bits = match self.config.stop_bits {
            2 => StopBits::Two,
            _ => StopBits::One,
        };

        let parity = match self.config.parity {
            SerialParity::Odd => Parity::Odd,
            SerialParity::Even => Parity::Even,
            SerialParity::None => Parity::None,
        };

        let flow_control = match self.config.flow_control {
            SerialFlowControl::Hardware => FlowControl::Hardware,
            SerialFlowControl::Software => FlowControl::Software,
            SerialFlowControl::None => FlowControl::None,
        };

        let port = serialport::new(&self.config.port, self.config.baud_rate)
            .data_bits(data_bits)
            .stop_bits(stop_bits)
            .parity(parity)
            .flow_control(flow_control)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| match e.kind() {
                serialport::ErrorKind::NoDevice => {
                    TransportError::PortNotFound(self.config.port.clone())
                }
                serialport::ErrorKind::Io(io_kind) => match io_kind {
                    std::io::ErrorKind::PermissionDenied => {
                        TransportError::PermissionDenied(self.config.port.clone())
                    }
                    _ => TransportError::ConnectionFailed(e.to_string()),
                },
                _ => TransportError::ConnectionFailed(e.to_string()),
            })?;

        *self.port.lock() = Some(port);
        self.connected_at = Some(Instant::now());
        *self.stats.write() = TransportStats::default();

        self.update_modem_lines();

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        *self.port.lock() = None;
        self.connected_at = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.port.lock().is_some()
    }

    async fn send(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        let mut port_guard = self.port.lock();
        let port = port_guard.as_mut().ok_or(TransportError::Disconnected)?;

        let written = port.write(data).map_err(TransportError::IoError)?;
        port.flush().map_err(TransportError::IoError)?;

        let mut stats = self.stats.write();
        stats.bytes_sent += written as u64;
        stats.packets_sent += 1;

        Ok(written)
    }

    async fn receive(&mut self) -> Result<Bytes, TransportError> {
        let mut port_guard = self.port.lock();
        let port = port_guard.as_mut().ok_or(TransportError::Disconnected)?;

        let mut buffer = vec![0u8; 4096];

        match port.read(&mut buffer) {
            Ok(0) => Err(TransportError::Disconnected),
            Ok(n) => {
                buffer.truncate(n);
                let bytes = Bytes::from(buffer);
                drop(port_guard); // Release lock before stats

                let mut stats = self.stats.write();
                stats.bytes_received += n as u64;
                stats.packets_received += 1;

                // Broadcast to subscribers
                let _ = self.tx.send(bytes.clone());

                self.update_modem_lines();

                Ok(bytes)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // No data available, return empty
                Ok(Bytes::new())
            }
            Err(e) => Err(TransportError::IoError(e)),
        }
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Serial
    }

    fn connection_info(&self) -> String {
        format!(
            "{} @ {} baud ({}{}{} {})",
            self.config.port,
            self.config.baud_rate,
            self.config.data_bits,
            match self.config.parity {
                SerialParity::None => "N",
                SerialParity::Odd => "O",
                SerialParity::Even => "E",
            },
            self.config.stop_bits,
            match self.config.flow_control {
                SerialFlowControl::None => "No FC",
                SerialFlowControl::Hardware => "HW FC",
                SerialFlowControl::Software => "SW FC",
            }
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

    async fn send_break(&mut self) -> Result<(), TransportError> {
        {
            let mut port_guard = self.port.lock();
            if let Some(ref mut port) = *port_guard {
                port.set_break()
                    .map_err(|e| TransportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
        {
            let mut port_guard = self.port.lock();
            if let Some(ref mut port) = *port_guard {
                port.clear_break()
                    .map_err(|e| TransportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            }
        }
        Ok(())
    }

    async fn set_dtr(&mut self, state: bool) -> Result<(), TransportError> {
        let mut port_guard = self.port.lock();
        if let Some(ref mut port) = *port_guard {
            port.write_data_terminal_ready(state)
                .map_err(|e| TransportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            self.modem_lines.write().dtr = state;
        }
        Ok(())
    }

    async fn set_rts(&mut self, state: bool) -> Result<(), TransportError> {
        let mut port_guard = self.port.lock();
        if let Some(ref mut port) = *port_guard {
            port.write_request_to_send(state)
                .map_err(|e| TransportError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            self.modem_lines.write().rts = state;
        }
        Ok(())
    }

    fn modem_lines(&self) -> Option<ModemLines> {
        Some(*self.modem_lines.read())
    }
}

/// List available serial ports
pub fn list_ports() -> Result<Vec<serialport::SerialPortInfo>, TransportError> {
    serialport::available_ports().map_err(|e| TransportError::IoError(e.into()))
}


