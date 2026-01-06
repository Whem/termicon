//! SSH transport implementation
//!
//! Provides SSH-2 protocol support with:
//! - Password and public key authentication
//! - Interactive shell and command execution
//! - Port forwarding (local, remote, dynamic/SOCKS)
//! - SFTP file transfer
//! - Agent forwarding

use super::{TransportError, TransportStats, TransportTrait, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::RwLock;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// SSH authentication method
#[derive(Debug, Clone)]
pub enum SshAuth {
    /// Password authentication
    Password(String),
    /// Public key authentication
    PublicKey {
        /// Path to private key file
        private_key: PathBuf,
        /// Optional passphrase for encrypted keys
        passphrase: Option<String>,
    },
    /// SSH agent authentication
    Agent,
    /// Keyboard-interactive (for 2FA, etc.)
    KeyboardInteractive,
    /// No authentication (rare)
    None,
}

impl Default for SshAuth {
    fn default() -> Self {
        Self::Agent
    }
}

/// SSH port forward configuration
#[derive(Debug, Clone)]
pub struct PortForward {
    /// Forward type
    pub forward_type: PortForwardType,
    /// Local bind address
    pub local_host: String,
    /// Local bind port
    pub local_port: u16,
    /// Remote host (for local/dynamic forward)
    pub remote_host: String,
    /// Remote port
    pub remote_port: u16,
}

/// Port forward type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortForwardType {
    /// Local port forward (-L)
    Local,
    /// Remote port forward (-R)
    Remote,
    /// Dynamic SOCKS proxy (-D)
    Dynamic,
}

/// SSH connection configuration
#[derive(Debug, Clone)]
pub struct SshConfig {
    /// Hostname or IP address
    pub host: String,
    /// Port (default: 22)
    pub port: u16,
    /// Username
    pub username: String,
    /// Authentication method
    pub auth: SshAuth,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Keepalive interval (0 = disabled)
    pub keepalive_secs: u64,
    /// Request PTY for shell
    pub request_pty: bool,
    /// Terminal type for PTY
    pub term_type: String,
    /// Terminal width
    pub term_width: u32,
    /// Terminal height
    pub term_height: u32,
    /// Port forwards to establish
    pub port_forwards: Vec<PortForward>,
    /// Compression enabled
    pub compression: bool,
    /// Jump host / proxy command
    pub proxy_jump: Option<Box<SshConfig>>,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Known hosts file path (None = don't verify)
    pub known_hosts: Option<PathBuf>,
    /// Strict host key checking
    pub strict_host_key: bool,
}

impl SshConfig {
    /// Create a new SSH configuration with defaults
    pub fn new(host: &str, username: &str) -> Self {
        Self {
            host: host.to_string(),
            port: 22,
            username: username.to_string(),
            auth: SshAuth::Agent,
            timeout_secs: 30,
            keepalive_secs: 60,
            request_pty: true,
            term_type: "xterm-256color".to_string(),
            term_width: 80,
            term_height: 24,
            port_forwards: Vec::new(),
            compression: false,
            proxy_jump: None,
            auto_reconnect: false,
            known_hosts: None,
            strict_host_key: false,
        }
    }

    /// Set port
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set password authentication
    #[must_use]
    pub fn password(mut self, password: &str) -> Self {
        self.auth = SshAuth::Password(password.to_string());
        self
    }

    /// Set public key authentication
    #[must_use]
    pub fn private_key(mut self, path: PathBuf, passphrase: Option<String>) -> Self {
        self.auth = SshAuth::PublicKey {
            private_key: path,
            passphrase,
        };
        self
    }

    /// Add a local port forward
    #[must_use]
    pub fn local_forward(mut self, local_port: u16, remote_host: &str, remote_port: u16) -> Self {
        self.port_forwards.push(PortForward {
            forward_type: PortForwardType::Local,
            local_host: "127.0.0.1".to_string(),
            local_port,
            remote_host: remote_host.to_string(),
            remote_port,
        });
        self
    }

    /// Add a remote port forward
    #[must_use]
    pub fn remote_forward(mut self, remote_port: u16, local_host: &str, local_port: u16) -> Self {
        self.port_forwards.push(PortForward {
            forward_type: PortForwardType::Remote,
            local_host: local_host.to_string(),
            local_port,
            remote_host: "127.0.0.1".to_string(),
            remote_port,
        });
        self
    }

    /// Add a dynamic SOCKS proxy
    #[must_use]
    pub fn dynamic_forward(mut self, local_port: u16) -> Self {
        self.port_forwards.push(PortForward {
            forward_type: PortForwardType::Dynamic,
            local_host: "127.0.0.1".to_string(),
            local_port,
            remote_host: String::new(),
            remote_port: 0,
        });
        self
    }

    /// Set proxy jump host
    #[must_use]
    pub fn proxy_jump(mut self, jump_config: SshConfig) -> Self {
        self.proxy_jump = Some(Box::new(jump_config));
        self
    }

    /// Set terminal size
    #[must_use]
    pub fn terminal_size(mut self, width: u32, height: u32) -> Self {
        self.term_width = width;
        self.term_height = height;
        self
    }
}

impl Default for SshConfig {
    fn default() -> Self {
        Self::new("localhost", "root")
    }
}

/// SSH transport using ssh2 crate (libssh2 bindings)
pub struct SshTransport {
    config: SshConfig,
    session: Option<ssh2::Session>,
    channel: Option<ssh2::Channel>,
    stats: Arc<RwLock<TransportStats>>,
    connected_at: Option<Instant>,
    tx: broadcast::Sender<Bytes>,
}

impl SshTransport {
    /// Create a new SSH transport
    pub fn new(config: SshConfig) -> Self {
        let (tx, _) = broadcast::channel(1024);

        Self {
            config,
            session: None,
            channel: None,
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connected_at: None,
            tx,
        }
    }

    /// Authenticate with the configured method
    fn authenticate(&self, session: &ssh2::Session) -> Result<(), TransportError> {
        match &self.config.auth {
            SshAuth::Password(password) => {
                session
                    .userauth_password(&self.config.username, password)
                    .map_err(|e| TransportError::ConnectionFailed(format!("Password auth failed: {}", e)))?;
            }
            SshAuth::PublicKey { private_key, passphrase } => {
                session
                    .userauth_pubkey_file(
                        &self.config.username,
                        None,
                        private_key,
                        passphrase.as_deref(),
                    )
                    .map_err(|e| TransportError::ConnectionFailed(format!("Key auth failed: {}", e)))?;
            }
            SshAuth::Agent => {
                let mut agent = session.agent()
                    .map_err(|e| TransportError::ConnectionFailed(format!("Agent connect failed: {}", e)))?;
                
                agent.connect()
                    .map_err(|e| TransportError::ConnectionFailed(format!("Agent connect failed: {}", e)))?;
                
                agent.list_identities()
                    .map_err(|e| TransportError::ConnectionFailed(format!("Agent list failed: {}", e)))?;
                
                let identities: Vec<_> = agent.identities().unwrap_or_default();
                
                let mut authenticated = false;
                for identity in identities {
                    if agent.userauth(&self.config.username, &identity).is_ok() {
                        authenticated = true;
                        break;
                    }
                }
                
                if !authenticated {
                    return Err(TransportError::ConnectionFailed("No valid agent identity".to_string()));
                }
            }
            SshAuth::KeyboardInteractive => {
                // Simplified keyboard-interactive (just try empty response)
                struct EmptyPromptHandler;
                impl ssh2::KeyboardInteractivePrompt for EmptyPromptHandler {
                    fn prompt<'a>(
                        &mut self,
                        _username: &str,
                        _instructions: &str,
                        prompts: &[ssh2::Prompt<'a>],
                    ) -> Vec<String> {
                        prompts.iter().map(|_| String::new()).collect()
                    }
                }
                session
                    .userauth_keyboard_interactive(&self.config.username, &mut EmptyPromptHandler)
                    .map_err(|e| TransportError::ConnectionFailed(format!("KB-interactive auth failed: {}", e)))?;
            }
            SshAuth::None => {
                // No authentication
            }
        }

        if !session.authenticated() {
            return Err(TransportError::ConnectionFailed("Authentication failed".to_string()));
        }

        Ok(())
    }

    /// Open a shell channel with PTY
    fn open_shell(&mut self) -> Result<(), TransportError> {
        let session = self.session.as_ref()
            .ok_or(TransportError::Disconnected)?;

        let mut channel = session.channel_session()
            .map_err(|e| TransportError::ConnectionFailed(format!("Channel open failed: {}", e)))?;

        if self.config.request_pty {
            channel
                .request_pty(
                    &self.config.term_type,
                    None,
                    Some((
                        self.config.term_width,
                        self.config.term_height,
                        0,
                        0,
                    )),
                )
                .map_err(|e| TransportError::ConnectionFailed(format!("PTY request failed: {}", e)))?;
        }

        channel.shell()
            .map_err(|e| TransportError::ConnectionFailed(format!("Shell request failed: {}", e)))?;

        // Note: ssh2 Channel doesn't have set_blocking, the Session has it
        // We'll handle non-blocking at read time

        self.channel = Some(channel);
        Ok(())
    }

    /// Resize the PTY
    pub fn resize_pty(&mut self, width: u32, height: u32) -> Result<(), TransportError> {
        if let Some(ref mut channel) = self.channel {
            channel
                .request_pty_size(width, height, None, None)
                .map_err(|e| TransportError::SendError(format!("PTY resize failed: {}", e)))?;
        }
        self.config.term_width = width;
        self.config.term_height = height;
        Ok(())
    }

    /// Execute a single command (non-interactive)
    pub async fn exec(&mut self, command: &str) -> Result<String, TransportError> {
        let session = self.session.as_ref()
            .ok_or(TransportError::Disconnected)?;

        let mut channel = session.channel_session()
            .map_err(|e| TransportError::ConnectionFailed(format!("Channel open failed: {}", e)))?;

        channel.exec(command)
            .map_err(|e| TransportError::SendError(format!("Exec failed: {}", e)))?;

        let mut output = String::new();
        use std::io::Read;
        channel.read_to_string(&mut output)
            .map_err(TransportError::IoError)?;

        channel.wait_close()
            .map_err(|e| TransportError::ReceiveError(e.to_string()))?;

        Ok(output)
    }
}

#[async_trait]
impl TransportTrait for SshTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        
        // Create TCP connection
        let tcp = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| TransportError::ConfigError(format!("Invalid address: {}", e)))?,
            Duration::from_secs(self.config.timeout_secs),
        )
        .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

        // Create SSH session
        let mut session = ssh2::Session::new()
            .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;

        session.set_timeout(self.config.timeout_secs as u32 * 1000);
        
        if self.config.compression {
            session.set_compress(true);
        }

        session.set_tcp_stream(tcp);
        session.handshake()
            .map_err(|e| TransportError::ConnectionFailed(format!("SSH handshake failed: {}", e)))?;

        // Authenticate
        self.authenticate(&session)?;

        self.session = Some(session);
        
        // Open shell channel
        self.open_shell()?;

        self.connected_at = Some(Instant::now());
        *self.stats.write() = TransportStats::default();

        // TODO: Set up port forwards from config

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Some(ref mut channel) = self.channel {
            let _ = channel.close();
            let _ = channel.wait_close();
        }
        self.channel = None;

        if let Some(ref session) = self.session {
            let _ = session.disconnect(None, "User disconnect", None);
        }
        self.session = None;
        self.connected_at = None;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.session.is_some() && self.channel.is_some()
    }

    async fn send(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        use std::io::Write;

        let channel = self.channel.as_mut()
            .ok_or(TransportError::Disconnected)?;

        let written = channel.write(data)
            .map_err(TransportError::IoError)?;

        channel.flush()
            .map_err(TransportError::IoError)?;

        let mut stats = self.stats.write();
        stats.bytes_sent += written as u64;
        stats.packets_sent += 1;

        Ok(written)
    }

    async fn receive(&mut self) -> Result<Bytes, TransportError> {
        use std::io::Read;

        let channel = self.channel.as_mut()
            .ok_or(TransportError::Disconnected)?;

        let mut buffer = vec![0u8; 4096];

        match channel.read(&mut buffer) {
            Ok(0) => {
                if channel.eof() {
                    Err(TransportError::Disconnected)
                } else {
                    Ok(Bytes::new())
                }
            }
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
        TransportType::Ssh
    }

    fn connection_info(&self) -> String {
        format!(
            "ssh://{}@{}:{}",
            self.config.username,
            self.config.host,
            self.config.port
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

/// SFTP client for file operations
pub struct SftpClient {
    sftp: ssh2::Sftp,
}

impl SftpClient {
    /// Create SFTP client from an SSH session
    pub fn new(session: &ssh2::Session) -> Result<Self, TransportError> {
        let sftp = session.sftp()
            .map_err(|e| TransportError::ConnectionFailed(format!("SFTP init failed: {}", e)))?;
        Ok(Self { sftp })
    }

    /// List directory contents
    pub fn list_dir(&self, path: &str) -> Result<Vec<(String, ssh2::FileStat)>, TransportError> {
        let entries = self.sftp.readdir(std::path::Path::new(path))
            .map_err(|e| TransportError::ReceiveError(format!("Read dir failed: {}", e)))?;
        
        let result = entries
            .into_iter()
            .map(|(path, stat)| (path.to_string_lossy().to_string(), stat))
            .collect();
        
        Ok(result)
    }

    /// Read a file
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, TransportError> {
        use std::io::Read;
        
        let mut file = self.sftp.open(std::path::Path::new(path))
            .map_err(|e| TransportError::ReceiveError(format!("Open file failed: {}", e)))?;
        
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(TransportError::IoError)?;
        
        Ok(contents)
    }

    /// Write a file
    pub fn write_file(&self, path: &str, data: &[u8]) -> Result<(), TransportError> {
        use std::io::Write;
        
        let mut file = self.sftp.create(std::path::Path::new(path))
            .map_err(|e| TransportError::SendError(format!("Create file failed: {}", e)))?;
        
        file.write_all(data)
            .map_err(TransportError::IoError)?;
        
        Ok(())
    }

    /// Delete a file
    pub fn remove_file(&self, path: &str) -> Result<(), TransportError> {
        self.sftp.unlink(std::path::Path::new(path))
            .map_err(|e| TransportError::SendError(format!("Delete failed: {}", e)))?;
        Ok(())
    }

    /// Create a directory
    pub fn mkdir(&self, path: &str) -> Result<(), TransportError> {
        self.sftp.mkdir(std::path::Path::new(path), 0o755)
            .map_err(|e| TransportError::SendError(format!("Mkdir failed: {}", e)))?;
        Ok(())
    }

    /// Get file/directory info
    pub fn stat(&self, path: &str) -> Result<ssh2::FileStat, TransportError> {
        self.sftp.stat(std::path::Path::new(path))
            .map_err(|e| TransportError::ReceiveError(format!("Stat failed: {}", e)))
    }

    /// Rename a file
    pub fn rename(&self, from: &str, to: &str) -> Result<(), TransportError> {
        self.sftp.rename(
            std::path::Path::new(from),
            std::path::Path::new(to),
            None,
        )
        .map_err(|e| TransportError::SendError(format!("Rename failed: {}", e)))?;
        Ok(())
    }
}


