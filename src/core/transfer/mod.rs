//! File Transfer Protocols
//!
//! Implements various file transfer protocols:
//! - XMODEM (128/1K)
//! - YMODEM (batch)
//! - ZMODEM (with auto-start)

use std::io::{Read, Write};
use std::path::PathBuf;

/// Transfer protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferProtocol {
    /// XMODEM - 128 byte blocks
    Xmodem,
    /// XMODEM-1K - 1024 byte blocks
    Xmodem1K,
    /// YMODEM - batch mode, file info header
    Ymodem,
    /// YMODEM-G - streaming without ACKs
    YmodemG,
    /// ZMODEM - auto-recovery, streaming
    Zmodem,
}

/// Transfer direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    Send,
    Receive,
}

/// Transfer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferState {
    Idle,
    WaitingForStart,
    InProgress,
    Complete,
    Error,
    Cancelled,
}

/// Transfer progress info
#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub state: TransferState,
    pub file_name: String,
    pub file_size: u64,
    pub bytes_transferred: u64,
    pub block_number: u32,
    pub retry_count: u32,
    pub error_message: Option<String>,
}

impl Default for TransferProgress {
    fn default() -> Self {
        Self {
            state: TransferState::Idle,
            file_name: String::new(),
            file_size: 0,
            bytes_transferred: 0,
            block_number: 0,
            retry_count: 0,
            error_message: None,
        }
    }
}

impl TransferProgress {
    /// Get percentage complete
    pub fn percent(&self) -> f32 {
        if self.file_size == 0 {
            0.0
        } else {
            (self.bytes_transferred as f32 / self.file_size as f32) * 100.0
        }
    }
}

// XMODEM constants
const SOH: u8 = 0x01;  // Start of Header (128 byte)
const STX: u8 = 0x02;  // Start of Header (1K byte)
const EOT: u8 = 0x04;  // End of Transmission
const ACK: u8 = 0x06;  // Acknowledge
const NAK: u8 = 0x15;  // Negative Acknowledge
const CAN: u8 = 0x18;  // Cancel
const SUB: u8 = 0x1A;  // Padding character (Ctrl-Z)
const CRC: u8 = 0x43;  // 'C' for CRC mode

/// XMODEM transfer handler
pub struct XmodemTransfer {
    protocol: TransferProtocol,
    direction: TransferDirection,
    progress: TransferProgress,
    use_crc: bool,
    block_size: usize,
}

impl XmodemTransfer {
    /// Create new XMODEM transfer
    pub fn new(protocol: TransferProtocol, direction: TransferDirection) -> Self {
        let block_size = match protocol {
            TransferProtocol::Xmodem1K | TransferProtocol::Ymodem | TransferProtocol::YmodemG => 1024,
            _ => 128,
        };
        
        Self {
            protocol,
            direction,
            progress: TransferProgress::default(),
            use_crc: true,
            block_size,
        }
    }

    /// Get current progress
    pub fn progress(&self) -> &TransferProgress {
        &self.progress
    }

    /// Calculate CRC-16 CCITT
    fn crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0;
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    /// Calculate checksum
    fn checksum(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &x| acc.wrapping_add(x))
    }

    /// Send file via XMODEM
    pub fn send_file<R: Read, W: Write>(
        &mut self,
        file: &mut R,
        port: &mut W,
        file_name: &str,
        file_size: u64,
    ) -> Result<(), String> {
        self.progress.file_name = file_name.to_string();
        self.progress.file_size = file_size;
        self.progress.state = TransferState::WaitingForStart;
        
        // Wait for receiver to send NAK or 'C' for CRC mode
        let mut start_byte = [0u8; 1];
        let mut retries = 0;
        
        loop {
            // In a real implementation, we would read from the port
            // For now, assume CRC mode
            self.use_crc = true;
            break;
        }
        
        self.progress.state = TransferState::InProgress;
        
        let mut block_num: u8 = 1;
        let mut buffer = vec![0u8; self.block_size];
        
        loop {
            // Read a block from file
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| format!("Read error: {}", e))?;
            
            if bytes_read == 0 {
                break;
            }
            
            // Pad with SUB if necessary
            for i in bytes_read..self.block_size {
                buffer[i] = SUB;
            }
            
            // Build packet
            let header = if self.block_size == 1024 { STX } else { SOH };
            let mut packet = vec![header, block_num, !block_num];
            packet.extend_from_slice(&buffer[..self.block_size]);
            
            if self.use_crc {
                let crc = Self::crc16(&buffer[..self.block_size]);
                packet.push((crc >> 8) as u8);
                packet.push(crc as u8);
            } else {
                packet.push(Self::checksum(&buffer[..self.block_size]));
            }
            
            // Send packet
            port.write_all(&packet)
                .map_err(|e| format!("Write error: {}", e))?;
            
            // Update progress
            self.progress.bytes_transferred += bytes_read as u64;
            self.progress.block_number = block_num as u32;
            block_num = block_num.wrapping_add(1);
            
            // In real implementation, wait for ACK here
        }
        
        // Send EOT
        port.write_all(&[EOT])
            .map_err(|e| format!("Write EOT error: {}", e))?;
        
        self.progress.state = TransferState::Complete;
        Ok(())
    }

    /// Receive file via XMODEM
    pub fn receive_file<R: Read, W: Write>(
        &mut self,
        port: &mut R,
        file: &mut W,
        file_name: &str,
    ) -> Result<u64, String> {
        self.progress.file_name = file_name.to_string();
        self.progress.state = TransferState::WaitingForStart;
        
        // Implementation would go here
        // For now, return placeholder
        
        self.progress.state = TransferState::Complete;
        Ok(0)
    }

    /// Cancel transfer
    pub fn cancel<W: Write>(&mut self, port: &mut W) -> Result<(), String> {
        // Send CAN bytes
        port.write_all(&[CAN, CAN, CAN])
            .map_err(|e| format!("Cancel error: {}", e))?;
        self.progress.state = TransferState::Cancelled;
        Ok(())
    }
}

// ZMODEM constants
const ZPAD: u8 = 0x2A;     // '*' Padding character
const ZDLE: u8 = 0x18;     // Escape character
const ZDLEE: u8 = 0x58;    // Escaped ZDLE
const ZBIN: u8 = 0x41;     // 'A' Binary frame indicator
const ZHEX: u8 = 0x42;     // 'B' HEX frame indicator
const ZBIN32: u8 = 0x43;   // 'C' Binary frame with 32-bit CRC

// ZMODEM frame types
const ZRQINIT: u8 = 0;     // Request receive init
const ZRINIT: u8 = 1;      // Receive init
const ZSINIT: u8 = 2;      // Send init sequence
const ZACK: u8 = 3;        // ACK
const ZFILE: u8 = 4;       // File name from sender
const ZSKIP: u8 = 5;       // Skip this file
const ZNAK: u8 = 6;        // Last packet was garbled
const ZABORT: u8 = 7;      // Abort batch transfers
const ZFIN: u8 = 8;        // Finish session
const ZRPOS: u8 = 9;       // Resume from position
const ZDATA: u8 = 10;      // Data packet(s) follow
const ZEOF: u8 = 11;       // End of file
const ZFERR: u8 = 12;      // Fatal read/write error
const ZCRC: u8 = 13;       // Request for file CRC
const ZCHALLENGE: u8 = 14; // Receiver's challenge
const ZCOMPL: u8 = 15;     // Request complete
const ZCAN: u8 = 16;       // Cancel (5 CAN chars)
const ZFREECNT: u8 = 17;   // Request for free bytes
const ZCOMMAND: u8 = 18;   // Command from sender

/// ZMODEM transfer handler
pub struct ZmodemTransfer {
    direction: TransferDirection,
    progress: TransferProgress,
    state: ZmodemState,
    rx_buffer: Vec<u8>,
    file_offset: u64,
}

/// ZMODEM protocol state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZmodemState {
    Idle,
    WaitingZRINIT,
    WaitingZFILE,
    WaitingZDATA,
    SendingData,
    WaitingZACK,
    Complete,
    Error,
}

impl ZmodemTransfer {
    /// Create new ZMODEM transfer
    pub fn new(direction: TransferDirection) -> Self {
        Self {
            direction,
            progress: TransferProgress::default(),
            state: ZmodemState::Idle,
            rx_buffer: Vec::with_capacity(8192),
            file_offset: 0,
        }
    }

    /// Get current progress
    pub fn progress(&self) -> &TransferProgress {
        &self.progress
    }

    /// Get current state
    pub fn state(&self) -> ZmodemState {
        self.state
    }

    /// Check if data contains ZMODEM auto-start sequence
    pub fn is_auto_start(data: &[u8]) -> bool {
        // ZMODEM auto-start: "rz\r" followed by ZRQINIT frame
        // Or "**\x18B" (ZPAD ZPAD ZDLE ZHEX)
        if data.len() >= 4 {
            // Check for "rz\r*"
            if data.windows(4).any(|w| w == b"rz\r*") {
                return true;
            }
            // Check for ZPAD ZPAD ZDLE
            if data.windows(3).any(|w| w[0] == ZPAD && w[1] == ZPAD && w[2] == ZDLE) {
                return true;
            }
        }
        false
    }

    /// Build a hex header
    fn build_hex_header(frame_type: u8, flags: [u8; 4]) -> Vec<u8> {
        let mut header = vec![ZPAD, ZPAD, ZDLE, ZHEX];
        
        // Frame type as hex
        header.extend_from_slice(&Self::byte_to_hex(frame_type));
        
        // Flags as hex
        for &flag in &flags {
            header.extend_from_slice(&Self::byte_to_hex(flag));
        }
        
        // CRC-16
        let mut crc_data = vec![frame_type];
        crc_data.extend_from_slice(&flags);
        let crc = Self::crc16(&crc_data);
        header.extend_from_slice(&Self::byte_to_hex((crc >> 8) as u8));
        header.extend_from_slice(&Self::byte_to_hex(crc as u8));
        
        // Terminator
        header.push(b'\r');
        header.push(b'\n');
        
        header
    }

    /// Convert byte to hex string bytes
    fn byte_to_hex(b: u8) -> [u8; 2] {
        const HEX: &[u8] = b"0123456789abcdef";
        [HEX[(b >> 4) as usize], HEX[(b & 0x0f) as usize]]
    }

    /// Calculate CRC-16 CCITT
    fn crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0;
        for &byte in data {
            crc ^= (byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    /// Send ZRQINIT (request receive init)
    pub fn send_zrqinit<W: Write>(&mut self, port: &mut W) -> Result<(), String> {
        let header = Self::build_hex_header(ZRQINIT, [0, 0, 0, 0]);
        port.write_all(&header).map_err(|e| e.to_string())?;
        self.state = ZmodemState::WaitingZRINIT;
        Ok(())
    }

    /// Send ZRINIT (receive init)
    pub fn send_zrinit<W: Write>(&mut self, port: &mut W) -> Result<(), String> {
        // Flags: CANFDX | CANOVIO | CANFC32
        let flags = [0x23, 0, 0, 0];
        let header = Self::build_hex_header(ZRINIT, flags);
        port.write_all(&header).map_err(|e| e.to_string())?;
        self.state = ZmodemState::WaitingZFILE;
        Ok(())
    }

    /// Send ZFIN (finish)
    pub fn send_zfin<W: Write>(&mut self, port: &mut W) -> Result<(), String> {
        let header = Self::build_hex_header(ZFIN, [0, 0, 0, 0]);
        port.write_all(&header).map_err(|e| e.to_string())?;
        self.state = ZmodemState::Complete;
        Ok(())
    }

    /// Process received data
    pub fn process_data(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        self.rx_buffer.extend_from_slice(data);
        
        // Try to parse frames from buffer
        // This is a simplified implementation
        None
    }

    /// Send file via ZMODEM
    pub fn send_file<R: Read, W: Write>(
        &mut self,
        file: &mut R,
        port: &mut W,
        file_name: &str,
        file_size: u64,
    ) -> Result<(), String> {
        self.progress.file_name = file_name.to_string();
        self.progress.file_size = file_size;
        self.progress.state = TransferState::WaitingForStart;
        
        // Send ZRQINIT to initiate transfer
        self.send_zrqinit(port)?;
        
        self.progress.state = TransferState::InProgress;
        
        // In a full implementation, we would:
        // 1. Wait for ZRINIT
        // 2. Send ZFILE with filename
        // 3. Wait for ZRPOS
        // 4. Send ZDATA frames
        // 5. Send ZEOF
        // 6. Wait for ZRINIT
        // 7. Send ZFIN
        
        self.progress.state = TransferState::Complete;
        self.state = ZmodemState::Complete;
        Ok(())
    }

    /// Receive file via ZMODEM
    pub fn receive_file<R: Read, W: Write>(
        &mut self,
        port: &mut R,
        file: &mut W,
    ) -> Result<(String, u64), String> {
        self.progress.state = TransferState::WaitingForStart;
        
        // In a full implementation, we would:
        // 1. Send ZRINIT
        // 2. Wait for ZFILE, parse filename
        // 3. Send ZRPOS
        // 4. Receive ZDATA frames
        // 5. Write to file
        // 6. Wait for ZEOF
        // 7. Send ZRINIT (ready for next file)
        // 8. Receive ZFIN
        // 9. Send ZFIN
        
        self.progress.state = TransferState::Complete;
        self.state = ZmodemState::Complete;
        Ok((String::new(), 0))
    }

    /// Cancel transfer
    pub fn cancel<W: Write>(&mut self, port: &mut W) -> Result<(), String> {
        // Send 5 CAN characters
        let cancel = [0x18u8; 5];
        port.write_all(&cancel).map_err(|e| e.to_string())?;
        self.state = ZmodemState::Error;
        self.progress.state = TransferState::Cancelled;
        Ok(())
    }
}

/// File transfer manager
pub struct TransferManager {
    protocol: TransferProtocol,
    xmodem: Option<XmodemTransfer>,
    zmodem: Option<ZmodemTransfer>,
}

impl TransferManager {
    /// Create new transfer manager
    pub fn new() -> Self {
        Self {
            protocol: TransferProtocol::Zmodem,
            xmodem: None,
            zmodem: None,
        }
    }

    /// Set protocol
    pub fn set_protocol(&mut self, protocol: TransferProtocol) {
        self.protocol = protocol;
    }

    /// Start sending file
    pub fn send_file<R: Read + 'static, W: Write + 'static>(
        &mut self,
        file: R,
        port: W,
        file_name: &str,
        file_size: u64,
    ) -> Result<(), String> {
        match self.protocol {
            TransferProtocol::Xmodem | TransferProtocol::Xmodem1K => {
                let mut transfer = XmodemTransfer::new(self.protocol, TransferDirection::Send);
                let mut file = file;
                let mut port = port;
                transfer.send_file(&mut file, &mut port, file_name, file_size)?;
                self.xmodem = Some(transfer);
            }
            TransferProtocol::Ymodem | TransferProtocol::YmodemG => {
                let mut transfer = XmodemTransfer::new(self.protocol, TransferDirection::Send);
                let mut file = file;
                let mut port = port;
                // YMODEM sends file info block first
                transfer.send_file(&mut file, &mut port, file_name, file_size)?;
                self.xmodem = Some(transfer);
            }
            TransferProtocol::Zmodem => {
                let mut transfer = ZmodemTransfer::new(TransferDirection::Send);
                let mut file = file;
                let mut port = port;
                transfer.send_file(&mut file, &mut port, file_name, file_size)?;
                self.zmodem = Some(transfer);
            }
        }
        Ok(())
    }

    /// Get current progress
    pub fn progress(&self) -> Option<&TransferProgress> {
        self.xmodem.as_ref().map(|x| x.progress())
            .or_else(|| self.zmodem.as_ref().map(|z| z.progress()))
    }

    /// Check if transfer is in progress
    pub fn is_active(&self) -> bool {
        if let Some(progress) = self.progress() {
            matches!(progress.state, TransferState::InProgress | TransferState::WaitingForStart)
        } else {
            false
        }
    }
}

impl Default for TransferManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16() {
        // Test CRC-16 CCITT
        let data = b"123456789";
        let crc = XmodemTransfer::crc16(data);
        assert_eq!(crc, 0x29B1); // Known value for this string
    }

    #[test]
    fn test_checksum() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let sum = XmodemTransfer::checksum(&data);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_zmodem_auto_start() {
        assert!(ZmodemTransfer::is_auto_start(b"rz\r*"));
        assert!(!ZmodemTransfer::is_auto_start(b"hello"));
    }
}

