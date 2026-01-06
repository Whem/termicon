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
    pub name: String,
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



