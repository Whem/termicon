//! Session tab management for multiple connections

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use uuid::Uuid;

use super::app::{ConnectionCommand, ConnectionMessage, ConnectionState, ConnectionType};

/// A single session/tab
pub struct SessionTab {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Connection type
    pub conn_type: ConnectionType,
    /// Connection state
    pub state: ConnectionState,
    /// Terminal output lines
    pub output: Vec<TerminalLine>,
    /// Input history
    pub input_history: Vec<String>,
    /// Current input
    pub current_input: String,
    /// History index
    pub history_index: Option<usize>,
    /// Channel to send commands
    pub tx: Option<Sender<ConnectionCommand>>,
    /// Channel to receive messages
    pub rx: Option<Receiver<ConnectionMessage>>,
    /// Connection info string
    pub connection_info: String,
    /// Scroll offset
    pub scroll_offset: f32,
    /// Has unread data
    pub has_unread: bool,
    /// Local echo enabled
    pub local_echo: bool,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Show hex
    pub show_hex: bool,
    /// Search query
    pub search_query: String,
    /// Search results (line indices)
    pub search_results: Vec<usize>,
    /// Current search result index
    pub search_index: usize,
    /// Search visible
    pub search_visible: bool,
    /// Case sensitive search
    pub search_case_sensitive: bool,
    /// Just connected (trigger save profile prompt)
    pub just_connected: bool,
    /// Associated profile ID (if connected from a profile)
    pub profile_id: Option<String>,
}

/// Terminal line with metadata
#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub text: String,
    pub timestamp: String,
    pub is_input: bool,
    pub raw_bytes: Option<Vec<u8>>,
}

impl Default for SessionTab {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "New Tab".to_string(),
            conn_type: ConnectionType::Serial,
            state: ConnectionState::Disconnected,
            output: Vec::new(),
            input_history: Vec::new(),
            current_input: String::new(),
            history_index: None,
            tx: None,
            rx: None,
            connection_info: String::new(),
            scroll_offset: 0.0,
            has_unread: false,
            local_echo: true,
            show_timestamps: true,
            show_hex: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_index: 0,
            search_visible: false,
            search_case_sensitive: false,
            just_connected: false,
            profile_id: None,
        }
    }
}

impl SessionTab {
    /// Create a new session tab
    pub fn new(name: &str, conn_type: ConnectionType) -> Self {
        Self {
            name: name.to_string(),
            conn_type,
            ..Default::default()
        }
    }

    /// Add a line to output
    pub fn add_line(&mut self, text: &str, is_input: bool) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.output.push(TerminalLine {
            text: text.to_string(),
            timestamp,
            is_input,
            raw_bytes: None,
        });
        
        // Limit buffer
        while self.output.len() > 50000 {
            self.output.remove(0);
        }
    }

    /// Add raw bytes to output
    pub fn add_bytes(&mut self, data: &[u8], is_input: bool) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        let text = String::from_utf8_lossy(data).to_string();
        
        // Split by newlines
        for line in text.lines() {
            if !line.is_empty() {
                self.output.push(TerminalLine {
                    text: line.to_string(),
                    timestamp: timestamp.clone(),
                    is_input,
                    raw_bytes: Some(line.as_bytes().to_vec()),
                });
            }
        }
        
        // Limit buffer
        while self.output.len() > 50000 {
            self.output.remove(0);
        }
    }

    /// Send data to connection
    pub fn send(&mut self, data: &[u8]) {
        if let Some(ref tx) = self.tx {
            let _ = tx.send(ConnectionCommand::Send(data.to_vec()));
        }
    }

    /// Send input with newline
    pub fn send_input(&mut self) {
        if self.current_input.is_empty() {
            return;
        }

        let text = self.current_input.clone();
        self.current_input.clear();

        // Add to history
        if self.input_history.last() != Some(&text) {
            self.input_history.push(text.clone());
        }
        self.history_index = None;

        // Local echo
        if self.local_echo {
            self.add_line(&format!("> {}", text), true);
        }

        // Send with newline
        let mut data = text.into_bytes();
        data.push(b'\n');
        self.send(&data);
    }

    /// Navigate history up
    pub fn history_up(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        let idx = match self.history_index {
            None => self.input_history.len().saturating_sub(1),
            Some(i) => i.saturating_sub(1),
        };

        self.history_index = Some(idx);
        self.current_input = self.input_history[idx].clone();
    }

    /// Navigate history down
    pub fn history_down(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        match self.history_index {
            None => {}
            Some(i) => {
                if i + 1 < self.input_history.len() {
                    self.history_index = Some(i + 1);
                    self.current_input = self.input_history[i + 1].clone();
                } else {
                    self.history_index = None;
                    self.current_input.clear();
                }
            }
        }
    }

    /// Process incoming messages
    pub fn process_messages(&mut self) {
        let messages: Vec<ConnectionMessage> = if let Some(ref rx) = self.rx {
            let mut msgs = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                msgs.push(msg);
            }
            msgs
        } else {
            Vec::new()
        };

        let mut should_clear = false;

        for msg in messages {
            match msg {
                ConnectionMessage::Connected => {
                    self.state = ConnectionState::Connected;
                    self.add_line("Connected!", false);
                    // Only set just_connected if not already from a profile
                    if self.profile_id.is_none() {
                        self.just_connected = true;
                    }
                }
                ConnectionMessage::Disconnected => {
                    self.state = ConnectionState::Disconnected;
                    self.add_line("Disconnected.", false);
                    should_clear = true;
                }
                ConnectionMessage::Error(e) => {
                    self.add_line(&format!("Error: {}", e), false);
                    self.state = ConnectionState::Disconnected;
                    should_clear = true;
                }
                ConnectionMessage::Data(data) => {
                    self.add_bytes(&data, false);
                    self.has_unread = true;
                }
            }
        }

        if should_clear {
            self.tx = None;
            self.rx = None;
        }
    }

    /// Disconnect
    pub fn disconnect(&mut self) {
        if let Some(ref tx) = self.tx {
            let _ = tx.send(ConnectionCommand::Disconnect);
        }
        self.add_line("Disconnecting...", false);
    }

    /// Clear output
    pub fn clear(&mut self) {
        self.output.clear();
    }

    /// Get hex view of a line
    pub fn get_hex_view(bytes: &[u8]) -> String {
        let mut result = String::new();
        
        for (i, chunk) in bytes.chunks(16).enumerate() {
            // Offset
            result.push_str(&format!("{:08X}  ", i * 16));
            
            // Hex bytes
            for (j, byte) in chunk.iter().enumerate() {
                result.push_str(&format!("{:02X} ", byte));
                if j == 7 {
                    result.push(' ');
                }
            }
            
            // Padding
            for _ in chunk.len()..16 {
                result.push_str("   ");
            }
            if chunk.len() <= 8 {
                result.push(' ');
            }
            
            // ASCII
            result.push_str(" |");
            for byte in chunk {
                if *byte >= 32 && *byte < 127 {
                    result.push(*byte as char);
                } else {
                    result.push('.');
                }
            }
            result.push_str("|\n");
        }
        
        result
    }

    /// Toggle search visibility
    pub fn toggle_search(&mut self) {
        self.search_visible = !self.search_visible;
        if !self.search_visible {
            self.search_query.clear();
            self.search_results.clear();
        }
    }

    /// Perform search
    pub fn search(&mut self) {
        self.search_results.clear();
        self.search_index = 0;

        if self.search_query.is_empty() {
            return;
        }

        let query = if self.search_case_sensitive {
            self.search_query.clone()
        } else {
            self.search_query.to_lowercase()
        };

        for (i, line) in self.output.iter().enumerate() {
            let text = if self.search_case_sensitive {
                line.text.clone()
            } else {
                line.text.to_lowercase()
            };

            if text.contains(&query) {
                self.search_results.push(i);
            }
        }
    }

    /// Go to next search result
    pub fn search_next(&mut self) -> Option<usize> {
        if self.search_results.is_empty() {
            return None;
        }
        
        self.search_index = (self.search_index + 1) % self.search_results.len();
        Some(self.search_results[self.search_index])
    }

    /// Go to previous search result
    pub fn search_prev(&mut self) -> Option<usize> {
        if self.search_results.is_empty() {
            return None;
        }
        
        if self.search_index == 0 {
            self.search_index = self.search_results.len() - 1;
        } else {
            self.search_index -= 1;
        }
        Some(self.search_results[self.search_index])
    }

    /// Get current search result line index
    pub fn current_search_result(&self) -> Option<usize> {
        if self.search_results.is_empty() {
            None
        } else {
            Some(self.search_results[self.search_index])
        }
    }

    /// Check if line matches search
    pub fn line_matches_search(&self, line_index: usize) -> bool {
        self.search_results.contains(&line_index)
    }

    /// Is current search result
    pub fn is_current_search_result(&self, line_index: usize) -> bool {
        self.current_search_result() == Some(line_index)
    }
}

/// Tab manager for multiple sessions
pub struct TabManager {
    /// All tabs
    pub tabs: Vec<SessionTab>,
    /// Active tab index
    pub active_index: usize,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: 0,
        }
    }
}

impl TabManager {
    /// Create new tab manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new tab
    pub fn add_tab(&mut self, tab: SessionTab) -> usize {
        self.tabs.push(tab);
        self.tabs.len() - 1
    }

    /// Remove tab by index
    pub fn remove_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            // Disconnect if connected
            if self.tabs[index].state == ConnectionState::Connected {
                self.tabs[index].disconnect();
            }
            self.tabs.remove(index);
            
            // Adjust active index
            if self.active_index >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_index = self.tabs.len() - 1;
            }
        }
    }

    /// Get active tab
    pub fn active_tab(&self) -> Option<&SessionTab> {
        self.tabs.get(self.active_index)
    }

    /// Get active tab mutable
    pub fn active_tab_mut(&mut self) -> Option<&mut SessionTab> {
        self.tabs.get_mut(self.active_index)
    }

    /// Set active tab
    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            // Clear unread on previously active
            if let Some(tab) = self.tabs.get_mut(index) {
                tab.has_unread = false;
            }
            self.active_index = index;
        }
    }

    /// Process all tabs
    pub fn process_all(&mut self) {
        for tab in &mut self.tabs {
            tab.process_messages();
        }
    }

    /// Check if any tab has unread
    pub fn has_any_unread(&self) -> bool {
        self.tabs.iter().any(|t| t.has_unread)
    }

    /// Get tab count
    pub fn count(&self) -> usize {
        self.tabs.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

