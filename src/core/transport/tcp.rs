//! TCP transport implementation

use super::{TransportError, TransportStats, TransportTrait, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

/// TCP connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    /// Host address
    pub host: String,
    /// Port number
    pub port: u16,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
}

impl TcpConfig {
    /// Create a new TCP configuration
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            timeout_secs: 10,
        }
    }

    /// Set timeout
    #[must_use]
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self::new("localhost", 23)
    }
}

/// TCP transport
pub struct TcpTransport {
    config: TcpConfig,
    stream: Option<TcpStream>,
    stats: Arc<RwLock<TransportStats>>,
    connected_at: Option<Instant>,
    tx: broadcast::Sender<Bytes>,
}

impl TcpTransport {
    /// Create a new TCP transport
    pub fn new(config: TcpConfig) -> Self {
        let (tx, _) = broadcast::channel(1024);

        Self {
            config,
            stream: None,
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connected_at: None,
            tx,
        }
    }
}

#[async_trait]
impl TransportTrait for TcpTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        
        let stream = tokio::time::timeout(
            Duration::from_secs(self.config.timeout_secs),
            TcpStream::connect(&addr),
        )
        .await
        .map_err(|_| TransportError::Timeout(self.config.timeout_secs))?
        .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

        // Set TCP_NODELAY for lower latency
        stream
            .set_nodelay(true)
            .map_err(TransportError::IoError)?;

        self.stream = Some(stream);
        self.connected_at = Some(Instant::now());
        *self.stats.write() = TransportStats::default();

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await.ok();
        }
        self.connected_at = None;
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

        stream
            .write_all(data)
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

        // Use try_read for non-blocking read
        match stream.try_read(&mut buffer) {
            Ok(0) => Err(TransportError::Disconnected),
            Ok(n) => {
                buffer.truncate(n);
                let bytes = Bytes::from(buffer);

                let mut stats = self.stats.write();
                stats.bytes_received += n as u64;
                stats.packets_received += 1;

                let _ = self.tx.send(bytes.clone());

                Ok(bytes)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(Bytes::new())
            }
            Err(e) => Err(TransportError::IoError(e)),
        }
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }

    fn connection_info(&self) -> String {
        format!("{}:{}", self.config.host, self.config.port)
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


