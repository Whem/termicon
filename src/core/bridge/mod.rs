//! Network Bridge - Serial ↔ TCP bidirectional data forwarding
//!
//! Supports:
//! - Serial to TCP client/server
//! - TCP to Serial forwarding
//! - RFC 2217 (Telnet Com Port Control)
//! - Data transformation/filtering

use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::io::{Read, Write};
use parking_lot::Mutex;

/// Bridge mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMode {
    /// Serial to TCP client (connect to remote)
    SerialToTcpClient,
    /// Serial to TCP server (listen for connections)
    SerialToTcpServer,
    /// TCP client to Serial
    TcpClientToSerial,
    /// Bidirectional (most common)
    Bidirectional,
}

/// Bridge configuration
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Bridge mode
    pub mode: BridgeMode,
    /// Serial port name
    pub serial_port: String,
    /// Serial baud rate
    pub baud_rate: u32,
    /// TCP host (for client mode)
    pub tcp_host: String,
    /// TCP port
    pub tcp_port: u16,
    /// Buffer size
    pub buffer_size: usize,
    /// Enable RFC 2217
    pub rfc2217: bool,
    /// Enable local echo
    pub local_echo: bool,
    /// Log traffic
    pub log_traffic: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            mode: BridgeMode::Bidirectional,
            serial_port: String::new(),
            baud_rate: 115200,
            tcp_host: "localhost".to_string(),
            tcp_port: 2217,
            buffer_size: 4096,
            rfc2217: false,
            local_echo: false,
            log_traffic: false,
        }
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Default)]
pub struct BridgeStats {
    pub bytes_serial_to_tcp: u64,
    pub bytes_tcp_to_serial: u64,
    pub packets_serial_to_tcp: u64,
    pub packets_tcp_to_serial: u64,
    pub errors: u64,
    pub connections: u64,
}

/// Bridge state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeState {
    Stopped,
    Starting,
    Running,
    Error,
}

/// Network bridge
pub struct Bridge {
    config: BridgeConfig,
    state: Arc<Mutex<BridgeState>>,
    stats: Arc<Mutex<BridgeStats>>,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Bridge {
    /// Create new bridge
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(BridgeState::Stopped)),
            stats: Arc::new(Mutex::new(BridgeStats::default())),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Start the bridge
    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("Bridge already running".to_string());
        }

        *self.state.lock() = BridgeState::Starting;
        self.running.store(true, Ordering::Relaxed);

        let config = self.config.clone();
        let running = self.running.clone();
        let state = self.state.clone();
        let stats = self.stats.clone();

        let handle = thread::spawn(move || {
            match config.mode {
                BridgeMode::SerialToTcpServer | BridgeMode::Bidirectional => {
                    Self::run_server_mode(&config, &running, &state, &stats);
                }
                BridgeMode::SerialToTcpClient | BridgeMode::TcpClientToSerial => {
                    Self::run_client_mode(&config, &running, &state, &stats);
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// Stop the bridge
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        *self.state.lock() = BridgeState::Stopped;
    }

    /// Get current state
    pub fn state(&self) -> BridgeState {
        *self.state.lock()
    }

    /// Get statistics
    pub fn stats(&self) -> BridgeStats {
        self.stats.lock().clone()
    }

    /// Is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.lock() = BridgeStats::default();
    }

    /// Run in server mode (listen for TCP connections)
    fn run_server_mode(
        config: &BridgeConfig,
        running: &Arc<AtomicBool>,
        state: &Arc<Mutex<BridgeState>>,
        stats: &Arc<Mutex<BridgeStats>>,
    ) {
        // Open serial port
        let serial: Arc<Mutex<Box<dyn serialport::SerialPort + Send>>> = 
            match serialport::new(&config.serial_port, config.baud_rate)
                .timeout(Duration::from_millis(100))
                .open()
        {
            Ok(s) => Arc::new(Mutex::new(s)),
            Err(e) => {
                *state.lock() = BridgeState::Error;
                eprintln!("Failed to open serial port: {}", e);
                return;
            }
        };

        // Start TCP server
        let addr: SocketAddr = format!("0.0.0.0:{}", config.tcp_port)
            .parse()
            .unwrap();
        
        let listener = match TcpListener::bind(addr) {
            Ok(l) => l,
            Err(e) => {
                *state.lock() = BridgeState::Error;
                eprintln!("Failed to bind TCP: {}", e);
                return;
            }
        };

        listener.set_nonblocking(true).ok();
        *state.lock() = BridgeState::Running;

        while running.load(Ordering::Relaxed) {
            // Accept new connections
            if let Ok((stream, _addr)) = listener.accept() {
                stats.lock().connections += 1;
                stream.set_nonblocking(true).ok();
                
                Self::handle_connection(
                    stream,
                    serial.clone(),
                    config,
                    running,
                    stats,
                );
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Run in client mode (connect to TCP server)
    fn run_client_mode(
        config: &BridgeConfig,
        running: &Arc<AtomicBool>,
        state: &Arc<Mutex<BridgeState>>,
        stats: &Arc<Mutex<BridgeStats>>,
    ) {
        // Open serial port
        let serial: Arc<Mutex<Box<dyn serialport::SerialPort + Send>>> = 
            match serialport::new(&config.serial_port, config.baud_rate)
                .timeout(Duration::from_millis(100))
                .open()
        {
            Ok(s) => Arc::new(Mutex::new(s)),
            Err(e) => {
                *state.lock() = BridgeState::Error;
                eprintln!("Failed to open serial port: {}", e);
                return;
            }
        };

        *state.lock() = BridgeState::Running;

        while running.load(Ordering::Relaxed) {
            // Connect to TCP server
            let addr = format!("{}:{}", config.tcp_host, config.tcp_port);
            match TcpStream::connect_timeout(
                &addr.parse().unwrap(),
                Duration::from_secs(5),
            ) {
                Ok(stream) => {
                    stats.lock().connections += 1;
                    stream.set_nonblocking(true).ok();
                    
                    Self::handle_connection(
                        stream,
                        serial.clone(),
                        config,
                        running,
                        stats,
                    );
                }
                Err(_) => {
                    // Retry after delay
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    /// Handle a single TCP connection
    fn handle_connection(
        mut stream: TcpStream,
        serial: Arc<Mutex<Box<dyn serialport::SerialPort + Send>>>,
        config: &BridgeConfig,
        running: &Arc<AtomicBool>,
        stats: &Arc<Mutex<BridgeStats>>,
    ) {
        let mut tcp_buf = vec![0u8; config.buffer_size];
        let mut serial_buf = vec![0u8; config.buffer_size];

        while running.load(Ordering::Relaxed) {
            // TCP -> Serial
            match stream.read(&mut tcp_buf) {
                Ok(0) => break, // Connection closed
                Ok(n) => {
                    let mut serial_guard = serial.lock();
                    if serial_guard.write_all(&tcp_buf[..n]).is_ok() {
                        let mut stats_guard = stats.lock();
                        stats_guard.bytes_tcp_to_serial += n as u64;
                        stats_guard.packets_tcp_to_serial += 1;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }

            // Serial -> TCP
            {
                let mut serial_guard = serial.lock();
                match serial_guard.read(&mut serial_buf) {
                    Ok(n) if n > 0 => {
                        drop(serial_guard); // Release lock before writing to TCP
                        if stream.write_all(&serial_buf[..n]).is_ok() {
                            let mut stats_guard = stats.lock();
                            stats_guard.bytes_serial_to_tcp += n as u64;
                            stats_guard.packets_serial_to_tcp += 1;
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(_) => {}
                }
            }

            thread::sleep(Duration::from_millis(1));
        }
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Simple TCP server for testing
pub struct TcpServer {
    listener: Option<TcpListener>,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl TcpServer {
    /// Create new TCP server
    pub fn new() -> Self {
        Self {
            listener: None,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Start listening
    pub fn start(&mut self, port: u16) -> Result<(), String> {
        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
        let listener = TcpListener::bind(addr)
            .map_err(|e| format!("Failed to bind: {}", e))?;
        
        self.listener = Some(listener);
        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Stop server
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.listener = None;
    }

    /// Accept a connection
    pub fn accept(&self) -> Option<TcpStream> {
        self.listener.as_ref()?.accept().ok().map(|(s, _)| s)
    }
}

impl Default for TcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_config() {
        let config = BridgeConfig::default();
        assert_eq!(config.mode, BridgeMode::Bidirectional);
        assert_eq!(config.tcp_port, 2217);
    }

    #[test]
    fn test_bridge_stats() {
        let stats = BridgeStats::default();
        assert_eq!(stats.bytes_serial_to_tcp, 0);
    }
}
