//! Telnet transport implementation
//!
//! Implements the Telnet protocol (RFC 854) with option negotiation

use super::{TransportError, TransportStats, TransportTrait, TransportType};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

// Telnet protocol constants
const IAC: u8 = 255; // Interpret As Command
const DONT: u8 = 254;
const DO: u8 = 253;
const WONT: u8 = 252;
const WILL: u8 = 251;
const SB: u8 = 250; // Subnegotiation Begin
const SE: u8 = 240; // Subnegotiation End

// Common Telnet options
#[allow(dead_code)]
const OPT_ECHO: u8 = 1;
const OPT_SUPPRESS_GO_AHEAD: u8 = 3;
const OPT_TERMINAL_TYPE: u8 = 24;
const OPT_NAWS: u8 = 31; // Negotiate About Window Size

/// Telnet connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelnetConfig {
    /// Host address
    pub host: String,
    /// Port number (default: 23)
    pub port: u16,
    /// Terminal type to announce
    pub terminal_type: String,
}

impl TelnetConfig {
    /// Create a new Telnet configuration
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
            port: 23,
            terminal_type: "xterm".to_string(),
        }
    }

    /// Set port
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set terminal type
    #[must_use]
    pub fn terminal_type(mut self, term_type: &str) -> Self {
        self.terminal_type = term_type.to_string();
        self
    }
}

impl Default for TelnetConfig {
    fn default() -> Self {
        Self::new("localhost")
    }
}

/// Telnet transport
pub struct TelnetTransport {
    config: TelnetConfig,
    stream: Option<TcpStream>,
    stats: Arc<RwLock<TransportStats>>,
    connected_at: Option<Instant>,
    tx: broadcast::Sender<Bytes>,
    /// Buffer for incomplete Telnet sequences
    pending_data: BytesMut,
}

impl TelnetTransport {
    /// Create a new Telnet transport
    pub fn new(config: TelnetConfig) -> Self {
        let (tx, _) = broadcast::channel(1024);

        Self {
            config,
            stream: None,
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connected_at: None,
            tx,
            pending_data: BytesMut::new(),
        }
    }

    /// Handle Telnet option negotiation
    async fn handle_telnet_option(
        stream: &mut TcpStream,
        command: u8,
        option: u8,
        terminal_type: &str,
    ) -> Result<(), TransportError> {
        let response = match command {
            DO => match option {
                OPT_TERMINAL_TYPE | OPT_NAWS | OPT_SUPPRESS_GO_AHEAD => {
                    vec![IAC, WILL, option]
                }
                _ => vec![IAC, WONT, option],
            },
            WILL => match option {
                OPT_ECHO | OPT_SUPPRESS_GO_AHEAD => {
                    vec![IAC, DO, option]
                }
                _ => vec![IAC, DONT, option],
            },
            _ => return Ok(()),
        };

        stream
            .write_all(&response)
            .await
            .map_err(TransportError::IoError)?;

        // Handle terminal type subnegotiation
        if command == DO && option == OPT_TERMINAL_TYPE {
            // Wait for subnegotiation request and respond with terminal type
            let term_bytes = terminal_type.as_bytes();
            let mut sub_response = vec![IAC, SB, OPT_TERMINAL_TYPE, 0]; // 0 = IS
            sub_response.extend_from_slice(term_bytes);
            sub_response.extend_from_slice(&[IAC, SE]);
            stream
                .write_all(&sub_response)
                .await
                .map_err(TransportError::IoError)?;
        }

        Ok(())
    }

    /// Process incoming data and strip Telnet commands
    async fn process_incoming(
        &mut self,
        raw_data: &[u8],
    ) -> Result<Bytes, TransportError> {
        let mut output = BytesMut::new();
        let mut i = 0;

        while i < raw_data.len() {
            if raw_data[i] == IAC {
                if i + 1 >= raw_data.len() {
                    // Incomplete command, save for later
                    self.pending_data.extend_from_slice(&raw_data[i..]);
                    break;
                }

                match raw_data[i + 1] {
                    IAC => {
                        // Escaped IAC (255 255 -> 255)
                        output.extend_from_slice(&[IAC]);
                        i += 2;
                    }
                    DO | DONT | WILL | WONT => {
                        if i + 2 >= raw_data.len() {
                            self.pending_data.extend_from_slice(&raw_data[i..]);
                            break;
                        }
                        let command = raw_data[i + 1];
                        let option = raw_data[i + 2];
                        
                        // Handle option negotiation
                        if let Some(ref mut stream) = self.stream {
                            Self::handle_telnet_option(
                                stream,
                                command,
                                option,
                                &self.config.terminal_type,
                            )
                            .await?;
                        }
                        i += 3;
                    }
                    SB => {
                        // Subnegotiation - find SE
                        let mut j = i + 2;
                        while j < raw_data.len() - 1 {
                            if raw_data[j] == IAC && raw_data[j + 1] == SE {
                                break;
                            }
                            j += 1;
                        }
                        if j >= raw_data.len() - 1 {
                            self.pending_data.extend_from_slice(&raw_data[i..]);
                            break;
                        }
                        i = j + 2;
                    }
                    _ => {
                        // Other commands, skip
                        i += 2;
                    }
                }
            } else {
                output.extend_from_slice(&[raw_data[i]]);
                i += 1;
            }
        }

        Ok(output.freeze())
    }
}

#[async_trait]
impl TransportTrait for TelnetTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        
        let stream = tokio::time::timeout(
            Duration::from_secs(10),
            TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| TransportError::Timeout(10))?
        .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

        stream.set_nodelay(true).map_err(TransportError::IoError)?;

        self.stream = Some(stream);
        self.connected_at = Some(Instant::now());
        self.pending_data.clear();
        *self.stats.write() = TransportStats::default();

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await.ok();
        }
        self.connected_at = None;
        self.pending_data.clear();
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    async fn send(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        let stream = self
            .stream
            .as_mut()
            .ok_or(TransportError::Disconnected)?;

        // Escape any IAC bytes in the data
        let mut escaped = Vec::with_capacity(data.len());
        for &byte in data {
            if byte == IAC {
                escaped.push(IAC);
                escaped.push(IAC);
            } else {
                escaped.push(byte);
            }
        }

        stream
            .write_all(&escaped)
            .await
            .map_err(TransportError::IoError)?;

        stream.flush().await.map_err(TransportError::IoError)?;

        let mut stats = self.stats.write();
        stats.bytes_sent += data.len() as u64;
        stats.packets_sent += 1;

        Ok(data.len())
    }

    async fn receive(&mut self) -> Result<Bytes, TransportError> {
        let stream = self
            .stream
            .as_mut()
            .ok_or(TransportError::Disconnected)?;

        let mut buffer = vec![0u8; 4096];

        match stream.try_read(&mut buffer) {
            Ok(0) => Err(TransportError::Disconnected),
            Ok(n) => {
                buffer.truncate(n);
                
                // Prepend any pending data
                if !self.pending_data.is_empty() {
                    let mut combined = BytesMut::new();
                    combined.extend_from_slice(&self.pending_data);
                    combined.extend_from_slice(&buffer);
                    self.pending_data.clear();
                    buffer = combined.to_vec();
                }

                // Process Telnet commands and extract data
                let processed = self.process_incoming(&buffer).await?;

                if !processed.is_empty() {
                    let mut stats = self.stats.write();
                    stats.bytes_received += processed.len() as u64;
                    stats.packets_received += 1;

                    let _ = self.tx.send(processed.clone());
                }

                Ok(processed)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(Bytes::new())
            }
            Err(e) => Err(TransportError::IoError(e)),
        }
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Telnet
    }

    fn connection_info(&self) -> String {
        format!("telnet://{}:{}", self.config.host, self.config.port)
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


