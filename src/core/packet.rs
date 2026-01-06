//! Packet Abstraction Layer
//!
//! Provides packet-level abstractions instead of raw byte streams.
//! Enables packet list view, timeline, replay, and protocol analysis.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Packet direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketDirection {
    /// Sent from host
    Tx,
    /// Received from device
    Rx,
    /// Internal/system message
    Internal,
}

/// Packet type classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PacketType {
    /// Raw data (unclassified)
    Raw,
    /// Text/ASCII data
    Text,
    /// Binary data
    Binary,
    /// Command packet
    Command,
    /// Response packet
    Response,
    /// Acknowledgment
    Ack,
    /// Negative acknowledgment
    Nak,
    /// Error packet
    Error,
    /// Keepalive/heartbeat
    Heartbeat,
    /// Protocol-specific type
    Protocol(String),
    /// Custom type
    Custom(String),
}

/// Packet metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketMetadata {
    /// Protocol name
    pub protocol: Option<String>,
    /// Decoded fields
    pub fields: Vec<PacketField>,
    /// Checksum valid
    pub checksum_valid: Option<bool>,
    /// Framing info
    pub framing: Option<String>,
    /// Notes/annotations
    pub notes: Vec<String>,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Linked packet ID (e.g., request-response pair)
    pub linked_packet: Option<Uuid>,
}

impl Default for PacketMetadata {
    fn default() -> Self {
        Self {
            protocol: None,
            fields: Vec::new(),
            checksum_valid: None,
            framing: None,
            notes: Vec::new(),
            tags: Vec::new(),
            linked_packet: None,
        }
    }
}

/// Decoded packet field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketField {
    /// Field name
    pub name: String,
    /// Field value (as string)
    pub value: String,
    /// Raw bytes for this field
    pub raw: Option<Vec<u8>>,
    /// Byte offset in packet
    pub offset: usize,
    /// Field length
    pub length: usize,
    /// Field description
    pub description: Option<String>,
}

/// A single packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    /// Unique packet ID
    pub id: Uuid,
    /// Packet sequence number
    pub seq: u64,
    /// Timestamp
    pub timestamp: DateTime<Local>,
    /// Direction
    pub direction: PacketDirection,
    /// Packet type
    pub packet_type: PacketType,
    /// Raw data
    pub data: Vec<u8>,
    /// Metadata
    pub metadata: PacketMetadata,
    /// Time delta from previous packet (microseconds)
    pub delta_us: u64,
}

impl Packet {
    /// Create a new packet
    pub fn new(direction: PacketDirection, data: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            seq: 0,
            timestamp: Local::now(),
            direction,
            packet_type: PacketType::Raw,
            data,
            metadata: PacketMetadata::default(),
            delta_us: 0,
        }
    }

    /// Create TX packet
    pub fn tx(data: Vec<u8>) -> Self {
        Self::new(PacketDirection::Tx, data)
    }

    /// Create RX packet
    pub fn rx(data: Vec<u8>) -> Self {
        Self::new(PacketDirection::Rx, data)
    }

    /// Set packet type
    pub fn with_type(mut self, ptype: PacketType) -> Self {
        self.packet_type = ptype;
        self
    }

    /// Set protocol
    pub fn with_protocol(mut self, protocol: &str) -> Self {
        self.metadata.protocol = Some(protocol.to_string());
        self
    }

    /// Add a decoded field
    pub fn add_field(&mut self, field: PacketField) {
        self.metadata.fields.push(field);
    }

    /// Add a note
    pub fn add_note(&mut self, note: &str) {
        self.metadata.notes.push(note.to_string());
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: &str) {
        self.metadata.tags.push(tag.to_string());
    }

    /// Get data as hex string
    pub fn hex(&self) -> String {
        hex::encode(&self.data)
    }

    /// Get data as ASCII (with escapes for non-printable)
    pub fn ascii(&self) -> String {
        self.data
            .iter()
            .map(|&b| {
                if b >= 32 && b < 127 {
                    (b as char).to_string()
                } else {
                    format!("\\x{:02x}", b)
                }
            })
            .collect()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Link to another packet
    pub fn link_to(&mut self, other_id: Uuid) {
        self.metadata.linked_packet = Some(other_id);
    }
}

/// Packet buffer with indexing and search
pub struct PacketBuffer {
    /// Packets storage
    packets: VecDeque<Packet>,
    /// Maximum buffer size
    max_size: usize,
    /// Sequence counter
    seq_counter: u64,
    /// Last packet timestamp for delta calculation
    last_timestamp: Option<DateTime<Local>>,
    /// Index by packet ID
    id_index: std::collections::HashMap<Uuid, usize>,
}

impl Default for PacketBuffer {
    fn default() -> Self {
        Self::new(10000)
    }
}

impl PacketBuffer {
    /// Create a new packet buffer
    pub fn new(max_size: usize) -> Self {
        Self {
            packets: VecDeque::with_capacity(max_size.min(1000)),
            max_size,
            seq_counter: 0,
            last_timestamp: None,
            id_index: std::collections::HashMap::new(),
        }
    }

    /// Add a packet
    pub fn push(&mut self, mut packet: Packet) {
        // Calculate delta
        if let Some(last_ts) = self.last_timestamp {
            let delta = packet.timestamp.signed_duration_since(last_ts);
            packet.delta_us = delta.num_microseconds().unwrap_or(0).max(0) as u64;
        }
        
        // Set sequence
        packet.seq = self.seq_counter;
        self.seq_counter += 1;
        
        // Update last timestamp
        self.last_timestamp = Some(packet.timestamp);
        
        // Remove old packets if at capacity
        while self.packets.len() >= self.max_size {
            if let Some(old) = self.packets.pop_front() {
                self.id_index.remove(&old.id);
            }
        }
        
        // Add to index
        self.id_index.insert(packet.id, self.packets.len());
        
        // Add packet
        self.packets.push_back(packet);
    }

    /// Get packet by ID
    pub fn get(&self, id: &Uuid) -> Option<&Packet> {
        self.id_index.get(id).and_then(|&idx| self.packets.get(idx))
    }

    /// Get packet by sequence number
    pub fn get_by_seq(&self, seq: u64) -> Option<&Packet> {
        if self.packets.is_empty() {
            return None;
        }
        let first_seq = self.packets.front()?.seq;
        if seq < first_seq {
            return None;
        }
        let idx = (seq - first_seq) as usize;
        self.packets.get(idx)
    }

    /// Get all packets
    pub fn all(&self) -> impl Iterator<Item = &Packet> {
        self.packets.iter()
    }

    /// Get packets by direction
    pub fn by_direction(&self, dir: PacketDirection) -> impl Iterator<Item = &Packet> {
        self.packets.iter().filter(move |p| p.direction == dir)
    }

    /// Get TX packets
    pub fn tx_packets(&self) -> impl Iterator<Item = &Packet> {
        self.by_direction(PacketDirection::Tx)
    }

    /// Get RX packets
    pub fn rx_packets(&self) -> impl Iterator<Item = &Packet> {
        self.by_direction(PacketDirection::Rx)
    }

    /// Search packets by hex pattern
    pub fn search_hex(&self, pattern: &str) -> Vec<&Packet> {
        let pattern_bytes = match hex::decode(pattern.replace(' ', "")) {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };
        
        self.packets
            .iter()
            .filter(|p| {
                p.data.windows(pattern_bytes.len()).any(|w| w == pattern_bytes.as_slice())
            })
            .collect()
    }

    /// Search packets by text
    pub fn search_text(&self, text: &str) -> Vec<&Packet> {
        let lower = text.to_lowercase();
        self.packets
            .iter()
            .filter(|p| {
                String::from_utf8_lossy(&p.data).to_lowercase().contains(&lower)
            })
            .collect()
    }

    /// Search packets by regex
    pub fn search_regex(&self, pattern: &str) -> Vec<&Packet> {
        let re = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        
        self.packets
            .iter()
            .filter(|p| {
                let text = String::from_utf8_lossy(&p.data);
                re.is_match(&text)
            })
            .collect()
    }

    /// Search packets by tag
    pub fn by_tag(&self, tag: &str) -> impl Iterator<Item = &Packet> {
        let tag = tag.to_string();
        self.packets.iter().filter(move |p| p.metadata.tags.contains(&tag))
    }

    /// Get packets in time range
    pub fn in_range(&self, start: DateTime<Local>, end: DateTime<Local>) -> Vec<&Packet> {
        self.packets
            .iter()
            .filter(|p| p.timestamp >= start && p.timestamp <= end)
            .collect()
    }

    /// Get packet count
    pub fn len(&self) -> usize {
        self.packets.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// Clear all packets
    pub fn clear(&mut self) {
        self.packets.clear();
        self.id_index.clear();
        self.seq_counter = 0;
        self.last_timestamp = None;
    }

    /// Get total bytes TX
    pub fn total_tx_bytes(&self) -> usize {
        self.tx_packets().map(|p| p.len()).sum()
    }

    /// Get total bytes RX
    pub fn total_rx_bytes(&self) -> usize {
        self.rx_packets().map(|p| p.len()).sum()
    }

    /// Get statistics
    pub fn stats(&self) -> PacketStats {
        PacketStats {
            total_packets: self.len(),
            tx_packets: self.tx_packets().count(),
            rx_packets: self.rx_packets().count(),
            tx_bytes: self.total_tx_bytes(),
            rx_bytes: self.total_rx_bytes(),
            first_packet_time: self.packets.front().map(|p| p.timestamp),
            last_packet_time: self.packets.back().map(|p| p.timestamp),
        }
    }

    /// Export to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let packets: Vec<_> = self.packets.iter().collect();
        serde_json::to_string_pretty(&packets)
    }

    /// Export to PCAP-like format (simplified)
    pub fn export_raw(&self) -> Vec<u8> {
        let mut output = Vec::new();
        
        for packet in &self.packets {
            // Simple format: timestamp (8 bytes) + direction (1 byte) + length (4 bytes) + data
            let ts = packet.timestamp.timestamp_micros();
            output.extend_from_slice(&ts.to_le_bytes());
            output.push(match packet.direction {
                PacketDirection::Tx => 0,
                PacketDirection::Rx => 1,
                PacketDirection::Internal => 2,
            });
            output.extend_from_slice(&(packet.data.len() as u32).to_le_bytes());
            output.extend_from_slice(&packet.data);
        }
        
        output
    }
}

/// Packet buffer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketStats {
    pub total_packets: usize,
    pub tx_packets: usize,
    pub rx_packets: usize,
    pub tx_bytes: usize,
    pub rx_bytes: usize,
    pub first_packet_time: Option<DateTime<Local>>,
    pub last_packet_time: Option<DateTime<Local>>,
}

/// Packet timeline entry for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub packet_id: Uuid,
    pub seq: u64,
    pub timestamp_us: i64,
    pub direction: PacketDirection,
    pub length: usize,
    pub summary: String,
}

impl From<&Packet> for TimelineEntry {
    fn from(p: &Packet) -> Self {
        Self {
            packet_id: p.id,
            seq: p.seq,
            timestamp_us: p.timestamp.timestamp_micros(),
            direction: p.direction,
            length: p.len(),
            summary: if p.len() <= 32 {
                p.ascii()
            } else {
                format!("{}... ({} bytes)", &p.ascii()[..32.min(p.len())], p.len())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let packet = Packet::tx(vec![0x01, 0x02, 0x03])
            .with_type(PacketType::Command)
            .with_protocol("Modbus");
        
        assert_eq!(packet.direction, PacketDirection::Tx);
        assert_eq!(packet.len(), 3);
        assert_eq!(packet.hex(), "010203");
    }

    #[test]
    fn test_packet_buffer() {
        let mut buffer = PacketBuffer::new(100);
        
        buffer.push(Packet::tx(vec![0x01, 0x02]));
        buffer.push(Packet::rx(vec![0x03, 0x04]));
        buffer.push(Packet::tx(vec![0x05, 0x06]));
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.tx_packets().count(), 2);
        assert_eq!(buffer.rx_packets().count(), 1);
    }

    #[test]
    fn test_packet_search() {
        let mut buffer = PacketBuffer::new(100);
        
        buffer.push(Packet::tx(b"Hello World".to_vec()));
        buffer.push(Packet::rx(b"Goodbye World".to_vec()));
        buffer.push(Packet::tx(b"Hello Again".to_vec()));
        
        let results = buffer.search_text("Hello");
        assert_eq!(results.len(), 2);
        
        let results = buffer.search_text("Goodbye");
        assert_eq!(results.len(), 1);
    }
}



