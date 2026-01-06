//! Session management for handling connections
//!
//! A Session represents an active connection that can be controlled,
//! monitored, and logged.

use super::transport::{create_transport, Transport, TransportError, TransportStats, TransportTrait, ModemLines};
use crate::core::logger::Logger;
use crate::core::trigger::{Trigger, TriggerAction};
use bytes::Bytes;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Not connected
    Disconnected,
    /// Connecting in progress
    Connecting,
    /// Connected and active
    Connected,
    /// Connection error occurred
    Error,
    /// Reconnecting after disconnect
    Reconnecting,
}

/// Session events
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// Data received from the connection
    DataReceived(Bytes),
    /// Data sent through the connection
    DataSent(Bytes),
    /// State changed
    StateChanged(SessionState),
    /// Error occurred
    Error(String),
    /// Trigger matched
    TriggerMatched {
        /// Trigger ID
        trigger_id: Uuid,
        /// Matched pattern
        pattern: String,
    },
    /// Connection statistics updated
    StatsUpdated(TransportStats),
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session name (for display)
    pub name: String,
    /// Transport configuration
    pub transport: Transport,
    /// Enable logging
    pub logging_enabled: bool,
    /// Log file path (if logging enabled)
    pub log_path: Option<String>,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Reconnect delay in seconds
    pub reconnect_delay_secs: u64,
    /// Maximum reconnect attempts (0 = infinite)
    pub max_reconnect_attempts: u32,
}

impl SessionConfig {
    /// Create a new session configuration
    pub fn new(name: &str, transport: Transport) -> Self {
        Self {
            name: name.to_string(),
            transport,
            logging_enabled: false,
            log_path: None,
            auto_reconnect: false,
            reconnect_delay_secs: 5,
            max_reconnect_attempts: 0,
        }
    }
}

/// Active session
pub struct Session {
    /// Unique session ID
    id: Uuid,
    /// Session name
    name: String,
    /// Current state
    state: Arc<RwLock<SessionState>>,
    /// Transport instance
    transport: Arc<tokio::sync::Mutex<Box<dyn TransportTrait>>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<SessionEvent>,
    /// Command sender
    cmd_tx: mpsc::Sender<SessionCommand>,
    /// Logger instance
    logger: Option<Logger>,
    /// Triggers
    triggers: Arc<RwLock<Vec<Trigger>>>,
    /// Receive buffer (for trigger matching)
    receive_buffer: Arc<RwLock<Vec<u8>>>,
}

/// Internal commands for session control
enum SessionCommand {
    Send(Bytes),
    Disconnect,
    SetDtr(bool),
    SetRts(bool),
    SendBreak,
}

impl Session {
    /// Connect with the given transport configuration
    pub async fn connect(transport: Transport) -> Result<Self, TransportError> {
        let config = SessionConfig::new("Session", transport);
        Self::connect_with_config(config).await
    }

    /// Connect with full configuration
    pub async fn connect_with_config(config: SessionConfig) -> Result<Self, TransportError> {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(SessionState::Connecting));
        let (event_tx, _) = broadcast::channel(1024);
        let (cmd_tx, cmd_rx) = mpsc::channel(256);

        // Create and connect transport
        let mut transport = create_transport(config.transport).await?;
        transport.connect().await?;

        *state.write() = SessionState::Connected;
        let _ = event_tx.send(SessionEvent::StateChanged(SessionState::Connected));

        // Create logger if enabled
        let logger: Option<Logger> = if config.logging_enabled {
            config.log_path.map(|_path| {
                Arc::new(parking_lot::Mutex::new(crate::core::logger::SessionLogger::new()))
            })
        } else {
            None
        };

        let transport = Arc::new(tokio::sync::Mutex::new(transport));
        let triggers = Arc::new(RwLock::new(Vec::new()));
        let receive_buffer = Arc::new(RwLock::new(Vec::with_capacity(8192)));

        let session = Self {
            id,
            name: config.name,
            state: state.clone(),
            transport: transport.clone(),
            event_tx: event_tx.clone(),
            cmd_tx,
            logger,
            triggers: triggers.clone(),
            receive_buffer: receive_buffer.clone(),
        };

        // Spawn receive loop
        let rx_state = state.clone();
        let rx_transport = transport.clone();
        let rx_event_tx = event_tx.clone();
        let rx_triggers = triggers;
        let rx_buffer = receive_buffer;

        tokio::spawn(async move {
            loop {
                if *rx_state.read() != SessionState::Connected {
                    break;
                }

                let data = {
                    let mut transport = rx_transport.lock().await;
                    transport.receive().await
                };

                match data {
                    Ok(bytes) if !bytes.is_empty() => {
                        // Add to receive buffer for trigger matching
                        {
                            let mut buffer = rx_buffer.write();
                            buffer.extend_from_slice(&bytes);
                            
                            // Limit buffer size
                            if buffer.len() > 65536 {
                                let drain_len = buffer.len() - 32768;
                                buffer.drain(0..drain_len);
                            }
                        }

                        // Check triggers
                        let triggers = rx_triggers.read().clone();
                        let buffer = rx_buffer.read().clone();
                        for trigger in &triggers {
                            if let Some(matched) = trigger.check(&buffer) {
                                let _ = rx_event_tx.send(SessionEvent::TriggerMatched {
                                    trigger_id: trigger.id,
                                    pattern: matched,
                                });
                            }
                        }

                        let _ = rx_event_tx.send(SessionEvent::DataReceived(bytes));
                    }
                    Ok(_) => {
                        // No data, continue
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                    Err(TransportError::Disconnected) => {
                        *rx_state.write() = SessionState::Disconnected;
                        let _ = rx_event_tx.send(SessionEvent::StateChanged(SessionState::Disconnected));
                        break;
                    }
                    Err(e) => {
                        *rx_state.write() = SessionState::Error;
                        let _ = rx_event_tx.send(SessionEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
        });

        // Spawn command handler
        let cmd_state = state;
        let cmd_transport = transport;
        let cmd_event_tx = event_tx;

        tokio::spawn(async move {
            let mut cmd_rx = cmd_rx;
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    SessionCommand::Send(data) => {
                        let mut transport = cmd_transport.lock().await;
                        match transport.send(&data).await {
                            Ok(_) => {
                                let _ = cmd_event_tx.send(SessionEvent::DataSent(data));
                            }
                            Err(e) => {
                                let _ = cmd_event_tx.send(SessionEvent::Error(e.to_string()));
                            }
                        }
                    }
                    SessionCommand::Disconnect => {
                        let mut transport = cmd_transport.lock().await;
                        let _ = transport.disconnect().await;
                        *cmd_state.write() = SessionState::Disconnected;
                        let _ = cmd_event_tx.send(SessionEvent::StateChanged(SessionState::Disconnected));
                        break;
                    }
                    SessionCommand::SetDtr(state) => {
                        let mut transport = cmd_transport.lock().await;
                        let _ = transport.set_dtr(state).await;
                    }
                    SessionCommand::SetRts(state) => {
                        let mut transport = cmd_transport.lock().await;
                        let _ = transport.set_rts(state).await;
                    }
                    SessionCommand::SendBreak => {
                        let mut transport = cmd_transport.lock().await;
                        let _ = transport.send_break().await;
                    }
                }
            }
        });

        Ok(session)
    }

    /// Get session ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get session name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current state
    pub fn state(&self) -> SessionState {
        *self.state.read()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        *self.state.read() == SessionState::Connected
    }

    /// Send data
    pub async fn send(&self, data: &[u8]) -> Result<(), TransportError> {
        if !self.is_connected() {
            return Err(TransportError::Disconnected);
        }

        self.cmd_tx
            .send(SessionCommand::Send(Bytes::copy_from_slice(data)))
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;

        // Log if enabled
        if let Some(ref logger) = self.logger {
            logger.lock().log_tx(data);
        }

        Ok(())
    }

    /// Disconnect the session
    pub async fn disconnect(&self) -> Result<(), TransportError> {
        self.cmd_tx
            .send(SessionCommand::Disconnect)
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        Ok(())
    }

    /// Subscribe to session events
    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_tx.subscribe()
    }

    /// Set DTR line state
    pub async fn set_dtr(&self, state: bool) -> Result<(), TransportError> {
        self.cmd_tx
            .send(SessionCommand::SetDtr(state))
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        Ok(())
    }

    /// Set RTS line state
    pub async fn set_rts(&self, state: bool) -> Result<(), TransportError> {
        self.cmd_tx
            .send(SessionCommand::SetRts(state))
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        Ok(())
    }

    /// Send break signal
    pub async fn send_break(&self) -> Result<(), TransportError> {
        self.cmd_tx
            .send(SessionCommand::SendBreak)
            .await
            .map_err(|e| TransportError::SendError(e.to_string()))?;
        Ok(())
    }

    /// Get modem lines state
    pub async fn modem_lines(&self) -> Option<ModemLines> {
        let transport = self.transport.lock().await;
        transport.modem_lines()
    }

    /// Get connection statistics
    pub async fn stats(&self) -> TransportStats {
        let transport = self.transport.lock().await;
        transport.stats()
    }

    /// Get connection info string
    pub async fn connection_info(&self) -> String {
        let transport = self.transport.lock().await;
        transport.connection_info()
    }

    /// Add a trigger
    pub fn add_trigger(&self, trigger: Trigger) {
        self.triggers.write().push(trigger);
    }

    /// Remove a trigger by ID
    pub fn remove_trigger(&self, id: Uuid) {
        self.triggers.write().retain(|t| t.id != id);
    }

    /// Get all triggers
    pub fn triggers(&self) -> Vec<Trigger> {
        self.triggers.read().clone()
    }

    /// Clear receive buffer
    pub fn clear_buffer(&self) {
        self.receive_buffer.write().clear();
    }
}


