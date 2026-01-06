//! XMODEM protocol implementation
//!
//! Supports:
//! - XMODEM Checksum
//! - XMODEM CRC
//! - XMODEM-1K

use super::{FileTransferProtocol, ProtocolError, TransferStatus};
use async_trait::async_trait;
use bytes::Bytes;
use std::path::Path;
use tokio::sync::mpsc;

// XMODEM constants
const SOH: u8 = 0x01; // Start of Header (128 byte block)
const STX: u8 = 0x02; // Start of Text (1K block)
const EOT: u8 = 0x04; // End of Transmission
const ACK: u8 = 0x06; // Acknowledge
const NAK: u8 = 0x15; // Negative Acknowledge
const CAN: u8 = 0x18; // Cancel
const SUB: u8 = 0x1A; // Padding character (Ctrl-Z)
const C: u8 = 0x43;   // 'C' for CRC mode

/// XMODEM variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XModemVariant {
    /// Original XMODEM with checksum
    Checksum,
    /// XMODEM with CRC-16
    CRC,
    /// XMODEM-1K (1024 byte blocks)
    OneK,
}

/// XMODEM protocol implementation
pub struct XModem {
    variant: XModemVariant,
    block_size: usize,
    use_crc: bool,
    cancelled: bool,
}

impl XModem {
    /// Create a new XMODEM instance
    pub fn new(variant: XModemVariant) -> Self {
        let (block_size, use_crc) = match variant {
            XModemVariant::Checksum => (128, false),
            XModemVariant::CRC => (128, true),
            XModemVariant::OneK => (1024, true),
        };

        Self {
            variant,
            block_size,
            use_crc,
            cancelled: false,
        }
    }

    /// Calculate checksum
    fn checksum(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
    }

    /// Calculate CRC-16-CCITT
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

    /// Create a packet
    fn create_packet(&self, block_num: u8, data: &[u8]) -> Vec<u8> {
        let mut packet = Vec::with_capacity(self.block_size + 5);

        // Header
        if self.block_size == 1024 {
            packet.push(STX);
        } else {
            packet.push(SOH);
        }

        packet.push(block_num);
        packet.push(!block_num);

        // Data (pad with SUB if needed)
        let mut block = data.to_vec();
        block.resize(self.block_size, SUB);
        packet.extend_from_slice(&block);

        // Checksum or CRC
        if self.use_crc {
            let crc = Self::crc16(&block);
            packet.push((crc >> 8) as u8);
            packet.push((crc & 0xFF) as u8);
        } else {
            packet.push(Self::checksum(&block));
        }

        packet
    }
}

#[async_trait]
impl FileTransferProtocol for XModem {
    fn name(&self) -> &'static str {
        match self.variant {
            XModemVariant::Checksum => "XMODEM",
            XModemVariant::CRC => "XMODEM-CRC",
            XModemVariant::OneK => "XMODEM-1K",
        }
    }

    async fn send_file(
        &mut self,
        path: &Path,
        tx: mpsc::Sender<Bytes>,
        mut rx: mpsc::Receiver<Bytes>,
        status_tx: mpsc::Sender<TransferStatus>,
    ) -> Result<(), ProtocolError> {
        let file_data = tokio::fs::read(path)
            .await
            .map_err(|e| ProtocolError::FileError(e.to_string()))?;

        let filename = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        let mut status = TransferStatus::new(&filename);
        status.file_size = Some(file_data.len() as u64);
        status.total_packets = Some((file_data.len() / self.block_size + 1) as u32);

        // Wait for receiver ready signal
        let mut retries = 0;
        loop {
            if self.cancelled {
                return Err(ProtocolError::Cancelled);
            }

            match tokio::time::timeout(
                std::time::Duration::from_secs(60),
                rx.recv(),
            ).await {
                Ok(Some(data)) if !data.is_empty() => {
                    if data[0] == C {
                        // CRC mode
                        self.use_crc = true;
                        break;
                    } else if data[0] == NAK {
                        // Checksum mode
                        self.use_crc = false;
                        break;
                    }
                }
                Ok(_) => {
                    retries += 1;
                    if retries > 10 {
                        return Err(ProtocolError::TooManyRetries);
                    }
                }
                Err(_) => return Err(ProtocolError::Timeout),
            }
        }

        // Send file data
        let mut block_num: u8 = 1;
        for chunk in file_data.chunks(self.block_size) {
            let packet = self.create_packet(block_num, chunk);
            let mut retries = 0;

            loop {
                if self.cancelled {
                    tx.send(Bytes::from(vec![CAN, CAN, CAN])).await.ok();
                    return Err(ProtocolError::Cancelled);
                }

                tx.send(Bytes::from(packet.clone())).await
                    .map_err(|_| ProtocolError::Disconnected)?;

                match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    rx.recv(),
                ).await {
                    Ok(Some(data)) if !data.is_empty() => {
                        if data[0] == ACK {
                            break;
                        } else if data[0] == NAK {
                            retries += 1;
                            status.retries += 1;
                        } else if data[0] == CAN {
                            return Err(ProtocolError::Cancelled);
                        }
                    }
                    _ => {
                        retries += 1;
                        status.retries += 1;
                    }
                }

                if retries > 10 {
                    return Err(ProtocolError::TooManyRetries);
                }
            }

            block_num = block_num.wrapping_add(1);
            status.packet_number = block_num as u32;
            status.bytes_transferred += chunk.len() as u64;
            status_tx.send(status.clone()).await.ok();
        }

        // Send EOT
        let mut retries = 0;
        loop {
            tx.send(Bytes::from(vec![EOT])).await
                .map_err(|_| ProtocolError::Disconnected)?;

            match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                rx.recv(),
            ).await {
                Ok(Some(data)) if !data.is_empty() && data[0] == ACK => break,
                _ => {
                    retries += 1;
                    if retries > 10 {
                        return Err(ProtocolError::TooManyRetries);
                    }
                }
            }
        }

        status.complete = true;
        status_tx.send(status).await.ok();

        Ok(())
    }

    async fn receive_file(
        &mut self,
        save_dir: &Path,
        tx: mpsc::Sender<Bytes>,
        mut rx: mpsc::Receiver<Bytes>,
        status_tx: mpsc::Sender<TransferStatus>,
    ) -> Result<String, ProtocolError> {
        let mut status = TransferStatus::new("receiving");
        let mut file_data = Vec::new();

        // Send ready signal
        let ready_char = if self.use_crc { C } else { NAK };
        tx.send(Bytes::from(vec![ready_char])).await
            .map_err(|_| ProtocolError::Disconnected)?;

        let mut expected_block: u8 = 1;

        loop {
            if self.cancelled {
                tx.send(Bytes::from(vec![CAN, CAN, CAN])).await.ok();
                return Err(ProtocolError::Cancelled);
            }

            match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                rx.recv(),
            ).await {
                Ok(Some(data)) if !data.is_empty() => {
                    if data[0] == EOT {
                        // End of transmission
                        tx.send(Bytes::from(vec![ACK])).await.ok();
                        break;
                    } else if data[0] == CAN {
                        return Err(ProtocolError::Cancelled);
                    } else if data[0] == SOH || data[0] == STX {
                        let block_size = if data[0] == STX { 1024 } else { 128 };
                        let expected_len = 3 + block_size + if self.use_crc { 2 } else { 1 };

                        if data.len() < expected_len {
                            tx.send(Bytes::from(vec![NAK])).await.ok();
                            continue;
                        }

                        let block_num = data[1];
                        let block_comp = data[2];

                        // Verify block number
                        if block_num != !block_comp {
                            tx.send(Bytes::from(vec![NAK])).await.ok();
                            continue;
                        }

                        let block_data = &data[3..3 + block_size];

                        // Verify checksum/CRC
                        let valid = if self.use_crc {
                            let received_crc = ((data[3 + block_size] as u16) << 8)
                                | (data[4 + block_size] as u16);
                            Self::crc16(block_data) == received_crc
                        } else {
                            Self::checksum(block_data) == data[3 + block_size]
                        };

                        if !valid {
                            status.retries += 1;
                            tx.send(Bytes::from(vec![NAK])).await.ok();
                            continue;
                        }

                        if block_num == expected_block {
                            file_data.extend_from_slice(block_data);
                            expected_block = expected_block.wrapping_add(1);
                            status.packet_number = expected_block as u32;
                            status.bytes_transferred = file_data.len() as u64;
                            status_tx.send(status.clone()).await.ok();
                        }

                        tx.send(Bytes::from(vec![ACK])).await.ok();
                    }
                }
                _ => {
                    tx.send(Bytes::from(vec![NAK])).await.ok();
                }
            }
        }

        // Save file
        let filename = format!("received_{}.bin", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let file_path = save_dir.join(&filename);

        // Remove padding
        while file_data.last() == Some(&SUB) {
            file_data.pop();
        }

        tokio::fs::write(&file_path, &file_data)
            .await
            .map_err(|e| ProtocolError::FileError(e.to_string()))?;

        status.filename = filename.clone();
        status.complete = true;
        status_tx.send(status).await.ok();

        Ok(filename)
    }

    async fn cancel(&mut self) -> Result<(), ProtocolError> {
        self.cancelled = true;
        Ok(())
    }
}





