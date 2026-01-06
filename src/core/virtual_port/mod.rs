//! Virtual COM Port implementation
//!
//! Supports:
//! - PTY pairs (Linux/macOS)
//! - Named Pipes (Windows)
//! - Loopback for testing

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

/// Virtual port type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualPortType {
    /// PTY pair (Unix)
    Pty,
    /// Named pipe (Windows)
    NamedPipe,
    /// Loopback (all platforms)
    Loopback,
}

/// Virtual port configuration
#[derive(Debug, Clone)]
pub struct VirtualPortConfig {
    /// Port type
    pub port_type: VirtualPortType,
    /// Port name/path
    pub name: String,
    /// Buffer size
    pub buffer_size: usize,
}

impl Default for VirtualPortConfig {
    fn default() -> Self {
        Self {
            port_type: VirtualPortType::Loopback,
            name: "vcom0".to_string(),
            buffer_size: 4096,
        }
    }
}

/// Virtual port handle
#[derive(Debug)]
pub struct VirtualPortHandle {
    /// Master side path/name
    pub master: String,
    /// Slave side path/name
    pub slave: String,
    /// Port type
    pub port_type: VirtualPortType,
}

/// Virtual port state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualPortState {
    Stopped,
    Running,
    Error,
}

/// Virtual COM port pair
pub struct VirtualPort {
    config: VirtualPortConfig,
    state: VirtualPortState,
    handle: Option<VirtualPortHandle>,
    running: Arc<AtomicBool>,
    #[cfg(unix)]
    master_fd: Option<i32>,
}

impl VirtualPort {
    /// Create new virtual port
    pub fn new(config: VirtualPortConfig) -> Self {
        Self {
            config,
            state: VirtualPortState::Stopped,
            handle: None,
            running: Arc::new(AtomicBool::new(false)),
            #[cfg(unix)]
            master_fd: None,
        }
    }

    /// Create a PTY pair (Unix only)
    #[cfg(unix)]
    pub fn create_pty(&mut self) -> Result<VirtualPortHandle, String> {
        use std::os::unix::io::FromRawFd;
        use std::ffi::CStr;

        unsafe {
            let mut master_fd: libc::c_int = 0;
            let mut slave_fd: libc::c_int = 0;
            let mut name_buf = [0i8; 256];

            let result = libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                name_buf.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            if result != 0 {
                return Err("Failed to create PTY pair".to_string());
            }

            let slave_name = CStr::from_ptr(name_buf.as_ptr())
                .to_string_lossy()
                .into_owned();

            self.master_fd = Some(master_fd);
            self.running.store(true, Ordering::Relaxed);
            self.state = VirtualPortState::Running;

            let handle = VirtualPortHandle {
                master: format!("/dev/ptmx (fd={})", master_fd),
                slave: slave_name,
                port_type: VirtualPortType::Pty,
            };

            self.handle = Some(handle.clone());
            Ok(handle)
        }
    }

    /// Create a PTY pair (non-Unix stub)
    #[cfg(not(unix))]
    pub fn create_pty(&mut self) -> Result<VirtualPortHandle, String> {
        Err("PTY not supported on this platform".to_string())
    }

    /// Create a named pipe (Windows only)
    #[cfg(windows)]
    pub fn create_named_pipe(&mut self) -> Result<VirtualPortHandle, String> {
        // For Windows, we would use CreateNamedPipe
        // This is a simplified stub
        let pipe_name = format!(r"\\.\pipe\{}", self.config.name);
        
        self.running.store(true, Ordering::Relaxed);
        self.state = VirtualPortState::Running;

        let handle = VirtualPortHandle {
            master: pipe_name.clone(),
            slave: pipe_name,
            port_type: VirtualPortType::NamedPipe,
        };

        self.handle = Some(handle.clone());
        Ok(handle)
    }

    /// Create a named pipe (non-Windows stub)
    #[cfg(not(windows))]
    pub fn create_named_pipe(&mut self) -> Result<VirtualPortHandle, String> {
        Err("Named pipes not supported on this platform".to_string())
    }

    /// Create a loopback port (all platforms)
    pub fn create_loopback(&mut self) -> Result<VirtualPortHandle, String> {
        self.running.store(true, Ordering::Relaxed);
        self.state = VirtualPortState::Running;

        let handle = VirtualPortHandle {
            master: format!("loopback:{}", self.config.name),
            slave: format!("loopback:{}", self.config.name),
            port_type: VirtualPortType::Loopback,
        };

        self.handle = Some(handle.clone());
        Ok(handle)
    }

    /// Create virtual port based on config
    pub fn create(&mut self) -> Result<VirtualPortHandle, String> {
        match self.config.port_type {
            VirtualPortType::Pty => self.create_pty(),
            VirtualPortType::NamedPipe => self.create_named_pipe(),
            VirtualPortType::Loopback => self.create_loopback(),
        }
    }

    /// Stop and cleanup
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        #[cfg(unix)]
        if let Some(fd) = self.master_fd.take() {
            unsafe {
                libc::close(fd);
            }
        }

        self.handle = None;
        self.state = VirtualPortState::Stopped;
    }

    /// Get current state
    pub fn state(&self) -> VirtualPortState {
        self.state
    }

    /// Get handle
    pub fn handle(&self) -> Option<&VirtualPortHandle> {
        self.handle.as_ref()
    }

    /// Is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Clone for VirtualPortHandle {
    fn clone(&self) -> Self {
        Self {
            master: self.master.clone(),
            slave: self.slave.clone(),
            port_type: self.port_type,
        }
    }
}

impl Drop for VirtualPort {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Loopback buffer for testing
pub struct LoopbackBuffer {
    buffer: std::collections::VecDeque<u8>,
    capacity: usize,
}

impl LoopbackBuffer {
    /// Create new buffer
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: std::collections::VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Write data
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = self.capacity - self.buffer.len();
        let to_write = data.len().min(available);
        
        for &byte in &data[..to_write] {
            self.buffer.push_back(byte);
        }
        
        to_write
    }

    /// Read data
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let to_read = buf.len().min(self.buffer.len());
        
        for i in 0..to_read {
            buf[i] = self.buffer.pop_front().unwrap_or(0);
        }
        
        to_read
    }

    /// Available data
    pub fn available(&self) -> usize {
        self.buffer.len()
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_buffer() {
        let mut buf = LoopbackBuffer::new(1024);
        
        let written = buf.write(b"Hello");
        assert_eq!(written, 5);
        assert_eq!(buf.available(), 5);
        
        let mut read_buf = [0u8; 10];
        let read = buf.read(&mut read_buf);
        assert_eq!(read, 5);
        assert_eq!(&read_buf[..5], b"Hello");
    }

    #[test]
    fn test_virtual_port_loopback() {
        let config = VirtualPortConfig {
            port_type: VirtualPortType::Loopback,
            name: "test".to_string(),
            buffer_size: 1024,
        };
        
        let mut vport = VirtualPort::new(config);
        let handle = vport.create_loopback().unwrap();
        
        assert_eq!(handle.port_type, VirtualPortType::Loopback);
        assert!(vport.is_running());
        
        vport.stop();
        assert!(!vport.is_running());
    }
}
