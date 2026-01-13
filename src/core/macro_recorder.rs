//! Macro Recording System
//!
//! Records user actions and input for playback automation

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Recorded action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroAction {
    /// Send data
    Send(Vec<u8>),
    /// Send text (converted to bytes with line ending)
    SendText(String),
    /// Wait for specific response pattern
    WaitFor(String),
    /// Delay for milliseconds
    Delay(u64),
    /// Send special key
    SendKey(SpecialKey),
    /// Comment/note
    Comment(String),
}

/// Special keys that can be recorded
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpecialKey {
    Enter,
    Tab,
    Escape,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    CtrlC,
    CtrlD,
    CtrlZ,
}

impl SpecialKey {
    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            SpecialKey::Enter => vec![b'\r'],
            SpecialKey::Tab => vec![b'\t'],
            SpecialKey::Escape => vec![0x1b],
            SpecialKey::Backspace => vec![0x08],
            SpecialKey::Delete => vec![0x7f],
            SpecialKey::Up => vec![0x1b, b'[', b'A'],
            SpecialKey::Down => vec![0x1b, b'[', b'B'],
            SpecialKey::Right => vec![0x1b, b'[', b'C'],
            SpecialKey::Left => vec![0x1b, b'[', b'D'],
            SpecialKey::Home => vec![0x1b, b'[', b'H'],
            SpecialKey::End => vec![0x1b, b'[', b'F'],
            SpecialKey::PageUp => vec![0x1b, b'[', b'5', b'~'],
            SpecialKey::PageDown => vec![0x1b, b'[', b'6', b'~'],
            SpecialKey::F1 => vec![0x1b, b'O', b'P'],
            SpecialKey::F2 => vec![0x1b, b'O', b'Q'],
            SpecialKey::F3 => vec![0x1b, b'O', b'R'],
            SpecialKey::F4 => vec![0x1b, b'O', b'S'],
            SpecialKey::F5 => vec![0x1b, b'[', b'1', b'5', b'~'],
            SpecialKey::F6 => vec![0x1b, b'[', b'1', b'7', b'~'],
            SpecialKey::F7 => vec![0x1b, b'[', b'1', b'8', b'~'],
            SpecialKey::F8 => vec![0x1b, b'[', b'1', b'9', b'~'],
            SpecialKey::F9 => vec![0x1b, b'[', b'2', b'0', b'~'],
            SpecialKey::F10 => vec![0x1b, b'[', b'2', b'1', b'~'],
            SpecialKey::F11 => vec![0x1b, b'[', b'2', b'3', b'~'],
            SpecialKey::F12 => vec![0x1b, b'[', b'2', b'4', b'~'],
            SpecialKey::CtrlC => vec![0x03],
            SpecialKey::CtrlD => vec![0x04],
            SpecialKey::CtrlZ => vec![0x1a],
        }
    }
}

/// A recorded macro step with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroStep {
    /// The action
    pub action: MacroAction,
    /// Delay before this action (from previous)
    pub delay_ms: u64,
    /// Optional label
    pub label: Option<String>,
}

/// A complete recorded macro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Recorded steps
    pub steps: Vec<MacroStep>,
    /// Created timestamp
    pub created: DateTime<Local>,
    /// Last modified
    pub modified: DateTime<Local>,
    /// Playback speed multiplier (1.0 = real time)
    pub speed: f32,
    /// Loop count (0 = infinite)
    pub loop_count: u32,
}

impl Macro {
    /// Create new empty macro
    pub fn new(name: &str) -> Self {
        let now = Local::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: String::new(),
            steps: Vec::new(),
            created: now,
            modified: now,
            speed: 1.0,
            loop_count: 1,
        }
    }

    /// Add a step
    pub fn add_step(&mut self, action: MacroAction, delay_ms: u64) {
        self.steps.push(MacroStep {
            action,
            delay_ms,
            label: None,
        });
        self.modified = Local::now();
    }

    /// Get all bytes to send for playback
    pub fn to_bytes(&self) -> Vec<Vec<u8>> {
        self.steps.iter().filter_map(|step| {
            match &step.action {
                MacroAction::Send(data) => Some(data.clone()),
                MacroAction::SendText(text) => {
                    let mut bytes = text.as_bytes().to_vec();
                    bytes.push(b'\r');
                    Some(bytes)
                }
                MacroAction::SendKey(key) => Some(key.to_bytes()),
                _ => None,
            }
        }).collect()
    }

    /// Total duration in milliseconds
    pub fn total_duration_ms(&self) -> u64 {
        self.steps.iter().map(|s| s.delay_ms).sum()
    }
}

/// Macro recorder state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderState {
    Idle,
    Recording,
    Paused,
}

/// Macro recorder
pub struct MacroRecorder {
    /// Current state
    state: RecorderState,
    /// Recording in progress
    recording: Option<Macro>,
    /// Last action timestamp
    last_action: Option<Instant>,
    /// Capture delays
    capture_delays: bool,
    /// Minimum delay to record (ms)
    min_delay_ms: u64,
    /// Maximum delay to record (ms)
    max_delay_ms: u64,
}

impl Default for MacroRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroRecorder {
    /// Create new recorder
    pub fn new() -> Self {
        Self {
            state: RecorderState::Idle,
            recording: None,
            last_action: None,
            capture_delays: true,
            min_delay_ms: 50,
            max_delay_ms: 5000,
        }
    }

    /// Start recording
    pub fn start(&mut self, name: &str) {
        self.recording = Some(Macro::new(name));
        self.last_action = Some(Instant::now());
        self.state = RecorderState::Recording;
    }

    /// Pause recording
    pub fn pause(&mut self) {
        if self.state == RecorderState::Recording {
            self.state = RecorderState::Paused;
        }
    }

    /// Resume recording
    pub fn resume(&mut self) {
        if self.state == RecorderState::Paused {
            self.last_action = Some(Instant::now());
            self.state = RecorderState::Recording;
        }
    }

    /// Stop recording and return the macro
    pub fn stop(&mut self) -> Option<Macro> {
        self.state = RecorderState::Idle;
        self.last_action = None;
        self.recording.take()
    }

    /// Record an action
    pub fn record(&mut self, action: MacroAction) {
        if self.state != RecorderState::Recording {
            return;
        }

        let delay_ms = if self.capture_delays {
            self.last_action
                .map(|t| {
                    let elapsed = t.elapsed().as_millis() as u64;
                    elapsed.clamp(self.min_delay_ms, self.max_delay_ms)
                })
                .unwrap_or(0)
        } else {
            0
        };

        if let Some(ref mut macro_rec) = self.recording {
            macro_rec.add_step(action, delay_ms);
        }

        self.last_action = Some(Instant::now());
    }

    /// Record sent text
    pub fn record_text(&mut self, text: &str) {
        self.record(MacroAction::SendText(text.to_string()));
    }

    /// Record sent bytes
    pub fn record_bytes(&mut self, data: &[u8]) {
        self.record(MacroAction::Send(data.to_vec()));
    }

    /// Record special key
    pub fn record_key(&mut self, key: SpecialKey) {
        self.record(MacroAction::SendKey(key));
    }

    /// Get current state
    pub fn state(&self) -> RecorderState {
        self.state
    }

    /// Is recording
    pub fn is_recording(&self) -> bool {
        self.state == RecorderState::Recording
    }

    /// Get current recording
    pub fn current(&self) -> Option<&Macro> {
        self.recording.as_ref()
    }

    /// Set capture delays
    pub fn set_capture_delays(&mut self, capture: bool) {
        self.capture_delays = capture;
    }
}

/// Macro player for playback
pub struct MacroPlayer {
    /// Macro to play
    macro_ref: Option<Macro>,
    /// Current step index
    current_step: usize,
    /// Current loop
    current_loop: u32,
    /// Playing state
    playing: bool,
    /// Last step time
    last_step_time: Option<Instant>,
    /// Speed multiplier
    speed: f32,
}

impl Default for MacroPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroPlayer {
    /// Create new player
    pub fn new() -> Self {
        Self {
            macro_ref: None,
            current_step: 0,
            current_loop: 0,
            playing: false,
            last_step_time: None,
            speed: 1.0,
        }
    }

    /// Load a macro for playback
    pub fn load(&mut self, macro_to_play: Macro) {
        self.speed = macro_to_play.speed;
        self.macro_ref = Some(macro_to_play);
        self.current_step = 0;
        self.current_loop = 0;
    }

    /// Start playback
    pub fn play(&mut self) {
        if self.macro_ref.is_some() {
            self.playing = true;
            self.current_step = 0;
            self.current_loop = 0;
            self.last_step_time = Some(Instant::now());
        }
    }

    /// Stop playback
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_step = 0;
        self.current_loop = 0;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Is playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Get next action if ready
    /// Returns (action, is_complete)
    pub fn next_action(&mut self) -> Option<(MacroAction, bool)> {
        if !self.playing {
            return None;
        }

        let macro_ref = self.macro_ref.as_ref()?;
        
        if self.current_step >= macro_ref.steps.len() {
            // End of macro, check loop
            self.current_loop += 1;
            if macro_ref.loop_count > 0 && self.current_loop >= macro_ref.loop_count {
                self.playing = false;
                return Some((MacroAction::Comment("Playback complete".to_string()), true));
            }
            self.current_step = 0;
        }

        let step = &macro_ref.steps[self.current_step];
        
        // Check if delay has elapsed
        let delay = (step.delay_ms as f32 / self.speed) as u64;
        if let Some(last_time) = self.last_step_time {
            if last_time.elapsed() < Duration::from_millis(delay) {
                return None;
            }
        }

        let action = step.action.clone();
        self.current_step += 1;
        self.last_step_time = Some(Instant::now());

        let is_complete = self.current_step >= macro_ref.steps.len() 
            && (macro_ref.loop_count > 0 && self.current_loop + 1 >= macro_ref.loop_count);

        Some((action, is_complete))
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.1, 10.0);
    }

    /// Get progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if let Some(ref m) = self.macro_ref {
            if m.steps.is_empty() {
                return 0.0;
            }
            self.current_step as f32 / m.steps.len() as f32
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_recording() {
        let mut recorder = MacroRecorder::new();
        recorder.set_capture_delays(false);
        
        recorder.start("Test Macro");
        recorder.record_text("hello");
        recorder.record_key(SpecialKey::Enter);
        recorder.record_text("world");
        
        let macro_rec = recorder.stop().unwrap();
        assert_eq!(macro_rec.name, "Test Macro");
        assert_eq!(macro_rec.steps.len(), 3);
    }

    #[test]
    fn test_special_keys() {
        assert_eq!(SpecialKey::Enter.to_bytes(), vec![b'\r']);
        assert_eq!(SpecialKey::CtrlC.to_bytes(), vec![0x03]);
        assert_eq!(SpecialKey::Up.to_bytes(), vec![0x1b, b'[', b'A']);
    }
}






