//! Auto-connect and auto-reconnect functionality
//!
//! Monitors connection state and automatically reconnects when disconnected.
//! Also handles USB device hot-plug detection.

use crate::core::session::{Session, SessionState};
use crate::core::transport::Transport;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Auto-connect configuration
#[derive(Debug, Clone)]
pub struct AutoConnectConfig {
    /// Enable auto-reconnect
    pub enabled: bool,
    /// Delay between reconnect attempts
    pub delay: Duration,
    /// Maximum reconnect attempts (0 = unlimited)
    pub max_attempts: u32,
    /// Monitor USB device changes
    pub monitor_usb: bool,
}

impl Default for AutoConnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            delay: Duration::from_secs(5),
            max_attempts: 0,
            monitor_usb: true,
        }
    }
}

/// Auto-connect events
#[derive(Debug, Clone)]
pub enum AutoConnectEvent {
    /// Attempting to reconnect
    Reconnecting { attempt: u32, max: u32 },
    /// Successfully reconnected
    Reconnected,
    /// Reconnect failed
    ReconnectFailed { error: String },
    /// Max attempts reached, giving up
    GaveUp,
    /// USB device detected
    DeviceDetected { port: String },
    /// USB device removed
    DeviceRemoved { port: String },
}

/// Auto-connect manager
pub struct AutoConnect {
    config: AutoConnectConfig,
    transport: Transport,
    state: Arc<RwLock<AutoConnectState>>,
    event_tx: mpsc::Sender<AutoConnectEvent>,
    cancel_tx: Option<mpsc::Sender<()>>,
}

#[derive(Debug, Clone, Default)]
struct AutoConnectState {
    attempts: u32,
    is_reconnecting: bool,
    last_error: Option<String>,
}

impl AutoConnect {
    /// Create a new auto-connect manager
    pub fn new(
        config: AutoConnectConfig,
        transport: Transport,
        event_tx: mpsc::Sender<AutoConnectEvent>,
    ) -> Self {
        Self {
            config,
            transport,
            state: Arc::new(RwLock::new(AutoConnectState::default())),
            event_tx,
            cancel_tx: None,
        }
    }

    /// Start monitoring for reconnection
    pub fn start(&mut self) {
        if !self.config.enabled {
            return;
        }

        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
        self.cancel_tx = Some(cancel_tx);

        let config = self.config.clone();
        let transport = self.transport.clone();
        let state = self.state.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                // Wait for cancellation or delay
                tokio::select! {
                    _ = cancel_rx.recv() => {
                        info!("Auto-connect cancelled");
                        break;
                    }
                    _ = tokio::time::sleep(config.delay) => {
                        // Continue with reconnect attempt
                    }
                }

                let current_attempts = state.read().attempts;

                // Check if we've exceeded max attempts
                if config.max_attempts > 0 && current_attempts >= config.max_attempts {
                    let _ = event_tx.send(AutoConnectEvent::GaveUp).await;
                    break;
                }

                // Attempt reconnection
                state.write().is_reconnecting = true;
                state.write().attempts += 1;

                let attempt = state.read().attempts;
                let _ = event_tx
                    .send(AutoConnectEvent::Reconnecting {
                        attempt,
                        max: config.max_attempts,
                    })
                    .await;

                match Session::connect(transport.clone()).await {
                    Ok(_session) => {
                        state.write().is_reconnecting = false;
                        state.write().attempts = 0;
                        let _ = event_tx.send(AutoConnectEvent::Reconnected).await;
                        info!("Auto-reconnect successful");
                        break;
                    }
                    Err(e) => {
                        let error = e.to_string();
                        state.write().last_error = Some(error.clone());
                        state.write().is_reconnecting = false;
                        let _ = event_tx
                            .send(AutoConnectEvent::ReconnectFailed { error })
                            .await;
                        warn!("Auto-reconnect failed (attempt {}): {}", attempt, e);
                    }
                }
            }
        });
    }

    /// Stop auto-reconnect
    pub fn stop(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.try_send(());
        }
    }

    /// Get current state
    pub fn state(&self) -> (u32, bool, Option<String>) {
        let state = self.state.read();
        (state.attempts, state.is_reconnecting, state.last_error.clone())
    }

    /// Reset attempts counter
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.attempts = 0;
        state.last_error = None;
    }

    /// Check if reconnecting
    pub fn is_reconnecting(&self) -> bool {
        self.state.read().is_reconnecting
    }
}

impl Drop for AutoConnect {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Monitor for USB serial device changes
pub struct UsbMonitor {
    event_tx: mpsc::Sender<AutoConnectEvent>,
    cancel_tx: Option<mpsc::Sender<()>>,
    known_ports: Arc<RwLock<Vec<String>>>,
}

impl UsbMonitor {
    /// Create a new USB monitor
    pub fn new(event_tx: mpsc::Sender<AutoConnectEvent>) -> Self {
        Self {
            event_tx,
            cancel_tx: None,
            known_ports: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start monitoring for USB device changes
    pub fn start(&mut self) {
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
        self.cancel_tx = Some(cancel_tx);

        // Initialize known ports
        if let Ok(ports) = serialport::available_ports() {
            *self.known_ports.write() = ports.iter().map(|p| p.port_name.clone()).collect();
        }

        let known_ports = self.known_ports.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let poll_interval = Duration::from_secs(2);

            loop {
                tokio::select! {
                    _ = cancel_rx.recv() => {
                        break;
                    }
                    _ = tokio::time::sleep(poll_interval) => {
                        // Check for port changes
                        if let Ok(current_ports) = serialport::available_ports() {
                            let current: Vec<String> = current_ports.iter()
                                .map(|p| p.port_name.clone())
                                .collect();

                            let known = known_ports.read().clone();

                            // Check for new ports
                            for port in &current {
                                if !known.contains(port) {
                                    info!("USB device detected: {}", port);
                                    let _ = event_tx.send(AutoConnectEvent::DeviceDetected {
                                        port: port.clone(),
                                    }).await;
                                }
                            }

                            // Check for removed ports
                            for port in &known {
                                if !current.contains(port) {
                                    info!("USB device removed: {}", port);
                                    let _ = event_tx.send(AutoConnectEvent::DeviceRemoved {
                                        port: port.clone(),
                                    }).await;
                                }
                            }

                            // Update known ports
                            *known_ports.write() = current;
                        }
                    }
                }
            }
        });
    }

    /// Stop monitoring
    pub fn stop(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.try_send(());
        }
    }

    /// Get current known ports
    pub fn known_ports(&self) -> Vec<String> {
        self.known_ports.read().clone()
    }
}

impl Drop for UsbMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}








