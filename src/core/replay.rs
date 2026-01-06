//! Session Replay System
//!
//! Records sessions for later playback with timing preservation.
//! Enables offline debugging, CI testing, and bug reproduction.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use super::packet::{Packet, PacketDirection};

/// Replay event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayEvent {
    /// Data transmitted
    Tx(Vec<u8>),
    /// Data received
    Rx(Vec<u8>),
    /// Connection established
    Connected,
    /// Connection lost
    Disconnected,
    /// Error occurred
    Error(String),
    /// User note/marker
    Marker(String),
    /// Configuration change
    ConfigChange(String),
    /// Bookmark with label
    Bookmark { label: String, color: Option<String> },
    /// Named checkpoint for navigation
    Checkpoint { name: String, description: Option<String> },
    /// Protocol-specific event
    Protocol { name: String, data: serde_json::Value },
}

/// Event marker for session replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMarker {
    /// Marker ID
    pub id: String,
    /// Marker type
    pub marker_type: MarkerType,
    /// Event index this marker is attached to
    pub event_index: usize,
    /// Offset in microseconds
    pub offset_us: u64,
    /// Label/description
    pub label: String,
    /// Color (hex)
    pub color: String,
    /// User notes
    pub notes: Option<String>,
}

/// Marker types for replay
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarkerType {
    /// Generic marker
    Generic,
    /// Error/issue marker
    Error,
    /// Warning marker  
    Warning,
    /// Success/checkpoint marker
    Success,
    /// Information marker
    Info,
    /// Start of a region
    RegionStart,
    /// End of a region
    RegionEnd,
    /// Protocol event
    Protocol,
    /// User-defined
    Custom,
}

/// A single recorded event with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    /// Event timestamp
    pub timestamp: DateTime<Local>,
    /// Time since session start (microseconds)
    pub offset_us: u64,
    /// Time since previous event (microseconds)
    pub delta_us: u64,
    /// The event
    pub event: ReplayEvent,
}

/// Session recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecording {
    /// Recording version
    pub version: u32,
    /// Session start time
    pub start_time: DateTime<Local>,
    /// Session end time
    pub end_time: Option<DateTime<Local>>,
    /// Connection type
    pub connection_type: String,
    /// Connection info
    pub connection_info: String,
    /// Recorded events
    pub events: Vec<RecordedEvent>,
    /// Metadata
    pub metadata: RecordingMetadata,
}

/// Recording metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// Recording name/title
    pub name: Option<String>,
    /// Description
    pub description: String,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Device/target info
    pub device_info: Option<String>,
    /// Firmware version
    pub firmware_version: Option<String>,
    /// User notes
    pub notes: String,
}

/// Export format for session recordings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// Human-readable text
    Text,
    /// Hex dump format
    Hex,
    /// Wireshark PCAP format
    Wireshark,
}

impl ExportFormat {
    /// Get file extension
    pub fn extension(&self) -> &str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Text => "txt",
            Self::Hex => "hex",
            Self::Wireshark => "pcap",
        }
    }
    
    /// Get MIME type
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::Text => "text/plain",
            Self::Hex => "text/plain",
            Self::Wireshark => "application/vnd.tcpdump.pcap",
        }
    }
}

/// Format data as hex dump
fn format_hex_dump(data: &[u8]) -> String {
    let mut result = String::new();
    
    for (i, chunk) in data.chunks(16).enumerate() {
        // Address
        result.push_str(&format!("{:08X}  ", i * 16));
        
        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                result.push(' ');
            }
            result.push_str(&format!("{:02X} ", byte));
        }
        
        // Padding
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                if j == 8 {
                    result.push(' ');
                }
                result.push_str("   ");
            }
        }
        
        // ASCII
        result.push_str(" |");
        for byte in chunk {
            if *byte >= 0x20 && *byte < 0x7F {
                result.push(*byte as char);
            } else {
                result.push('.');
            }
        }
        result.push_str("|\n");
    }
    
    result
}

impl SessionRecording {
    /// Create new recording
    pub fn new(connection_type: &str, connection_info: &str) -> Self {
        Self {
            version: 1,
            start_time: Local::now(),
            end_time: None,
            connection_type: connection_type.to_string(),
            connection_info: connection_info.to_string(),
            events: Vec::new(),
            metadata: RecordingMetadata::default(),
        }
    }
    
    /// Get all markers
    pub fn markers(&self) -> Vec<EventMarker> {
        let mut markers = Vec::new();
        for (idx, event) in self.events.iter().enumerate() {
            match &event.event {
                ReplayEvent::Marker(label) => {
                    markers.push(EventMarker {
                        id: format!("marker_{}", idx),
                        marker_type: MarkerType::Generic,
                        event_index: idx,
                        offset_us: event.offset_us,
                        label: label.clone(),
                        color: "#4CAF50".to_string(),
                        notes: None,
                    });
                }
                ReplayEvent::Bookmark { label, color } => {
                    markers.push(EventMarker {
                        id: format!("bookmark_{}", idx),
                        marker_type: MarkerType::Info,
                        event_index: idx,
                        offset_us: event.offset_us,
                        label: label.clone(),
                        color: color.clone().unwrap_or_else(|| "#2196F3".to_string()),
                        notes: None,
                    });
                }
                ReplayEvent::Checkpoint { name, description } => {
                    markers.push(EventMarker {
                        id: format!("checkpoint_{}", idx),
                        marker_type: MarkerType::Success,
                        event_index: idx,
                        offset_us: event.offset_us,
                        label: name.clone(),
                        color: "#8BC34A".to_string(),
                        notes: description.clone(),
                    });
                }
                ReplayEvent::Error(msg) => {
                    markers.push(EventMarker {
                        id: format!("error_{}", idx),
                        marker_type: MarkerType::Error,
                        event_index: idx,
                        offset_us: event.offset_us,
                        label: msg.clone(),
                        color: "#F44336".to_string(),
                        notes: None,
                    });
                }
                _ => {}
            }
        }
        markers
    }
    
    /// Get checkpoints for navigation
    pub fn checkpoints(&self) -> Vec<(usize, String, Duration)> {
        self.events.iter().enumerate()
            .filter_map(|(idx, e)| {
                if let ReplayEvent::Checkpoint { name, .. } = &e.event {
                    Some((idx, name.clone(), Duration::from_micros(e.offset_us)))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Export to different formats
    pub fn export(&self, format: ExportFormat) -> Result<Vec<u8>, String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_vec_pretty(self)
                    .map_err(|e| format!("JSON export error: {}", e))
            }
            ExportFormat::Csv => {
                Ok(self.export_csv().into_bytes())
            }
            ExportFormat::Text => {
                Ok(self.export_text().into_bytes())
            }
            ExportFormat::Hex => {
                Ok(self.export_hex().into_bytes())
            }
            ExportFormat::Wireshark => {
                Ok(self.export_pcap())
            }
        }
    }
    
    /// Export as CSV
    fn export_csv(&self) -> String {
        let mut csv = String::from("timestamp,offset_us,delta_us,direction,type,data_hex,data_text\n");
        
        for event in &self.events {
            let (direction, event_type, data_hex, data_text) = match &event.event {
                ReplayEvent::Tx(data) => ("TX", "data", hex::encode(data), String::from_utf8_lossy(data).to_string()),
                ReplayEvent::Rx(data) => ("RX", "data", hex::encode(data), String::from_utf8_lossy(data).to_string()),
                ReplayEvent::Connected => ("", "connected", String::new(), String::new()),
                ReplayEvent::Disconnected => ("", "disconnected", String::new(), String::new()),
                ReplayEvent::Error(msg) => ("", "error", String::new(), msg.clone()),
                ReplayEvent::Marker(msg) => ("", "marker", String::new(), msg.clone()),
                ReplayEvent::ConfigChange(cfg) => ("", "config", String::new(), cfg.clone()),
                ReplayEvent::Bookmark { label, .. } => ("", "bookmark", String::new(), label.clone()),
                ReplayEvent::Checkpoint { name, .. } => ("", "checkpoint", String::new(), name.clone()),
                ReplayEvent::Protocol { name, data } => ("", "protocol", String::new(), format!("{}: {}", name, data)),
            };
            
            csv.push_str(&format!(
                "{},{},{},{},{},\"{}\",\"{}\"\n",
                event.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                event.offset_us,
                event.delta_us,
                direction,
                event_type,
                data_hex,
                data_text.replace('"', "\"\"").replace('\n', "\\n").replace('\r', "\\r")
            ));
        }
        
        csv
    }
    
    /// Export as readable text
    fn export_text(&self) -> String {
        let mut text = format!(
            "Session Recording: {}\n",
            self.metadata.name.clone().unwrap_or_else(|| "Untitled".to_string())
        );
        text.push_str(&format!("Connection: {} - {}\n", self.connection_type, self.connection_info));
        text.push_str(&format!("Started: {}\n", self.start_time.format("%Y-%m-%d %H:%M:%S")));
        if let Some(end) = self.end_time {
            text.push_str(&format!("Ended: {}\n", end.format("%Y-%m-%d %H:%M:%S")));
        }
        text.push_str(&format!("Events: {}\n", self.events.len()));
        text.push_str(&format!("TX: {} bytes, RX: {} bytes\n\n", self.tx_bytes(), self.rx_bytes()));
        text.push_str("=" .repeat(80).as_str());
        text.push('\n');
        
        for event in &self.events {
            let time_str = format!("[{:.3}s]", event.offset_us as f64 / 1_000_000.0);
            match &event.event {
                ReplayEvent::Tx(data) => {
                    text.push_str(&format!("{} TX: {}\n", time_str, String::from_utf8_lossy(data)));
                }
                ReplayEvent::Rx(data) => {
                    text.push_str(&format!("{} RX: {}\n", time_str, String::from_utf8_lossy(data)));
                }
                ReplayEvent::Connected => {
                    text.push_str(&format!("{} --- CONNECTED ---\n", time_str));
                }
                ReplayEvent::Disconnected => {
                    text.push_str(&format!("{} --- DISCONNECTED ---\n", time_str));
                }
                ReplayEvent::Error(msg) => {
                    text.push_str(&format!("{} ERROR: {}\n", time_str, msg));
                }
                ReplayEvent::Marker(msg) => {
                    text.push_str(&format!("{} [MARKER] {}\n", time_str, msg));
                }
                ReplayEvent::Checkpoint { name, description } => {
                    text.push_str(&format!("{} [CHECKPOINT: {}] {}\n", time_str, name, description.as_deref().unwrap_or("")));
                }
                _ => {}
            }
        }
        
        text
    }
    
    /// Export as hex dump
    fn export_hex(&self) -> String {
        let mut hex = String::new();
        
        for event in &self.events {
            let time_str = format!("[{:.3}s]", event.offset_us as f64 / 1_000_000.0);
            match &event.event {
                ReplayEvent::Tx(data) | ReplayEvent::Rx(data) => {
                    let dir = if matches!(&event.event, ReplayEvent::Tx(_)) { "TX" } else { "RX" };
                    hex.push_str(&format!("{} {} ({} bytes):\n", time_str, dir, data.len()));
                    hex.push_str(&format_hex_dump(data));
                    hex.push('\n');
                }
                _ => {}
            }
        }
        
        hex
    }
    
    /// Export as PCAP format (Wireshark compatible)
    fn export_pcap(&self) -> Vec<u8> {
        let mut pcap = Vec::new();
        
        // PCAP global header
        pcap.extend_from_slice(&0xA1B2C3D4u32.to_le_bytes()); // Magic
        pcap.extend_from_slice(&2u16.to_le_bytes()); // Major version
        pcap.extend_from_slice(&4u16.to_le_bytes()); // Minor version
        pcap.extend_from_slice(&0i32.to_le_bytes()); // Timezone
        pcap.extend_from_slice(&0u32.to_le_bytes()); // Sigfigs
        pcap.extend_from_slice(&65535u32.to_le_bytes()); // Snaplen
        pcap.extend_from_slice(&147u32.to_le_bytes()); // Network (user-defined)
        
        // Packets
        for event in &self.events {
            if let ReplayEvent::Tx(data) | ReplayEvent::Rx(data) = &event.event {
                let ts_sec = event.offset_us / 1_000_000;
                let ts_usec = event.offset_us % 1_000_000;
                
                // Packet header
                pcap.extend_from_slice(&(ts_sec as u32).to_le_bytes());
                pcap.extend_from_slice(&(ts_usec as u32).to_le_bytes());
                pcap.extend_from_slice(&(data.len() as u32).to_le_bytes());
                pcap.extend_from_slice(&(data.len() as u32).to_le_bytes());
                
                // Packet data
                pcap.extend_from_slice(data);
            }
        }
        
        pcap
    }

    /// Get total duration
    pub fn duration(&self) -> Option<Duration> {
        if let Some(end) = self.end_time {
            let duration = end.signed_duration_since(self.start_time);
            Some(Duration::from_micros(duration.num_microseconds()? as u64))
        } else if let Some(last) = self.events.last() {
            Some(Duration::from_micros(last.offset_us))
        } else {
            None
        }
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get total TX bytes
    pub fn tx_bytes(&self) -> usize {
        self.events
            .iter()
            .filter_map(|e| match &e.event {
                ReplayEvent::Tx(data) => Some(data.len()),
                _ => None,
            })
            .sum()
    }

    /// Get total RX bytes
    pub fn rx_bytes(&self) -> usize {
        self.events
            .iter()
            .filter_map(|e| match &e.event {
                ReplayEvent::Rx(data) => Some(data.len()),
                _ => None,
            })
            .sum()
    }

    /// Save to file (JSON)
    pub fn save_json(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    /// Load from file (JSON)
    pub fn load_json(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    /// Save to binary format (more compact)
    pub fn save_binary(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        // Simple binary format: header + events
        writer.write_all(b"TREC")?; // Magic
        writer.write_all(&self.version.to_le_bytes())?;
        
        let json = serde_json::to_vec(self)?;
        writer.write_all(&(json.len() as u32).to_le_bytes())?;
        writer.write_all(&json)?;
        
        Ok(())
    }

    /// Load from binary format
    pub fn load_binary(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"TREC" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid recording file magic",
            ));
        }
        
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let _version = u32::from_le_bytes(version_bytes);
        
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;
        
        let mut json = vec![0u8; len];
        reader.read_exact(&mut json)?;
        
        Ok(serde_json::from_slice(&json)?)
    }
}

/// Session recorder
pub struct SessionRecorder {
    recording: SessionRecording,
    start_instant: Instant,
    last_event: Instant,
    enabled: bool,
}

impl SessionRecorder {
    /// Create new recorder
    pub fn new(connection_type: &str, connection_info: &str) -> Self {
        let now = Instant::now();
        Self {
            recording: SessionRecording::new(connection_type, connection_info),
            start_instant: now,
            last_event: now,
            enabled: true,
        }
    }

    /// Enable/disable recording
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if recording
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record an event
    pub fn record(&mut self, event: ReplayEvent) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();
        let offset = now.duration_since(self.start_instant);
        let delta = now.duration_since(self.last_event);

        self.recording.events.push(RecordedEvent {
            timestamp: Local::now(),
            offset_us: offset.as_micros() as u64,
            delta_us: delta.as_micros() as u64,
            event,
        });

        self.last_event = now;
    }

    /// Record TX data
    pub fn record_tx(&mut self, data: &[u8]) {
        self.record(ReplayEvent::Tx(data.to_vec()));
    }

    /// Record RX data
    pub fn record_rx(&mut self, data: &[u8]) {
        self.record(ReplayEvent::Rx(data.to_vec()));
    }

    /// Add marker
    pub fn add_marker(&mut self, note: &str) {
        self.record(ReplayEvent::Marker(note.to_string()));
    }
    
    /// Add bookmark with optional color
    pub fn add_bookmark(&mut self, label: &str, color: Option<&str>) {
        self.record(ReplayEvent::Bookmark { 
            label: label.to_string(), 
            color: color.map(|s| s.to_string()) 
        });
    }
    
    /// Add checkpoint for navigation
    pub fn add_checkpoint(&mut self, name: &str, description: Option<&str>) {
        self.record(ReplayEvent::Checkpoint { 
            name: name.to_string(), 
            description: description.map(|s| s.to_string()) 
        });
    }
    
    /// Record protocol event
    pub fn record_protocol_event(&mut self, protocol: &str, data: serde_json::Value) {
        self.record(ReplayEvent::Protocol { 
            name: protocol.to_string(), 
            data 
        });
    }

    /// Finish recording
    pub fn finish(&mut self) -> SessionRecording {
        self.recording.end_time = Some(Local::now());
        self.recording.clone()
    }

    /// Get current recording (without finishing)
    pub fn current(&self) -> &SessionRecording {
        &self.recording
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: RecordingMetadata) {
        self.recording.metadata = metadata;
    }
}

/// Playback speed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackSpeed {
    /// Real-time (1x)
    RealTime,
    /// Fixed multiplier
    Multiplier(f32),
    /// As fast as possible
    Instant,
}

/// Session player state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
}

/// Session player for replay
pub struct SessionPlayer {
    recording: SessionRecording,
    current_index: usize,
    state: PlayerState,
    speed: PlaybackSpeed,
    last_event_time: Option<Instant>,
    accumulated_delay: Duration,
}

impl SessionPlayer {
    /// Create player from recording
    pub fn new(recording: SessionRecording) -> Self {
        Self {
            recording,
            current_index: 0,
            state: PlayerState::Stopped,
            speed: PlaybackSpeed::RealTime,
            last_event_time: None,
            accumulated_delay: Duration::ZERO,
        }
    }

    /// Start playback
    pub fn play(&mut self) {
        self.state = PlayerState::Playing;
        self.last_event_time = Some(Instant::now());
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlayerState::Paused;
    }

    /// Stop and reset
    pub fn stop(&mut self) {
        self.state = PlayerState::Stopped;
        self.current_index = 0;
        self.last_event_time = None;
        self.accumulated_delay = Duration::ZERO;
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: PlaybackSpeed) {
        self.speed = speed;
    }

    /// Get current state
    pub fn state(&self) -> PlayerState {
        self.state
    }

    /// Get progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.recording.events.is_empty() {
            return 0.0;
        }
        self.current_index as f32 / self.recording.events.len() as f32
    }

    /// Get current position in time
    pub fn current_time(&self) -> Option<Duration> {
        self.recording.events.get(self.current_index).map(|e| {
            Duration::from_micros(e.offset_us)
        })
    }

    /// Seek to position (0.0 - 1.0)
    pub fn seek(&mut self, position: f32) {
        let idx = (position * self.recording.events.len() as f32) as usize;
        self.current_index = idx.min(self.recording.events.len().saturating_sub(1));
        self.accumulated_delay = Duration::ZERO;
    }

    /// Get next event if ready
    /// Returns (event, is_last)
    pub fn next(&mut self) -> Option<(ReplayEvent, bool)> {
        if self.state != PlayerState::Playing {
            return None;
        }

        if self.current_index >= self.recording.events.len() {
            self.state = PlayerState::Stopped;
            return None;
        }

        let event = &self.recording.events[self.current_index];

        // Check timing
        match self.speed {
            PlaybackSpeed::Instant => {
                // Immediate
            }
            PlaybackSpeed::RealTime | PlaybackSpeed::Multiplier(_) => {
                let required_delay = Duration::from_micros(event.delta_us);
                let scaled_delay = match self.speed {
                    PlaybackSpeed::RealTime => required_delay,
                    PlaybackSpeed::Multiplier(m) => Duration::from_secs_f64(required_delay.as_secs_f64() / m as f64),
                    _ => unreachable!(),
                };

                if let Some(last) = self.last_event_time {
                    let elapsed = last.elapsed();
                    if elapsed < scaled_delay {
                        // Not ready yet
                        return None;
                    }
                }
            }
        }

        self.last_event_time = Some(Instant::now());
        let result = event.event.clone();
        let is_last = self.current_index == self.recording.events.len() - 1;
        self.current_index += 1;

        Some((result, is_last))
    }

    /// Get recording info
    pub fn recording(&self) -> &SessionRecording {
        &self.recording
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording() {
        let mut recorder = SessionRecorder::new("Serial", "COM1 @ 115200");
        
        recorder.record_tx(b"Hello");
        std::thread::sleep(Duration::from_millis(10));
        recorder.record_rx(b"World");
        recorder.add_marker("Test marker");
        
        let recording = recorder.finish();
        
        assert_eq!(recording.event_count(), 3);
        assert_eq!(recording.tx_bytes(), 5);
        assert_eq!(recording.rx_bytes(), 5);
    }

    #[test]
    fn test_playback() {
        let mut recorder = SessionRecorder::new("TCP", "localhost:23");
        recorder.record_tx(b"A");
        recorder.record_rx(b"B");
        let recording = recorder.finish();

        let mut player = SessionPlayer::new(recording);
        player.set_speed(PlaybackSpeed::Instant);
        player.play();

        let (event1, _) = player.next().unwrap();
        assert!(matches!(event1, ReplayEvent::Tx(_)));

        let (event2, is_last) = player.next().unwrap();
        assert!(matches!(event2, ReplayEvent::Rx(_)));
        assert!(is_last);
    }
}




