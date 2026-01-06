//! Workspace Save/Restore
//!
//! Provides functionality to save and restore the complete application state,
//! including open sessions, window layout, and settings.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Workspace version for compatibility
pub const WORKSPACE_VERSION: u32 = 1;

/// Complete workspace state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Version number
    pub version: u32,
    /// Workspace name
    pub name: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Open sessions
    pub sessions: Vec<SessionState>,
    /// Window layout
    pub layout: WindowLayout,
    /// Active tab index
    pub active_tab: usize,
    /// Side panel state
    pub side_panel: SidePanelState,
    /// Terminal settings per session
    pub terminal_settings: HashMap<String, TerminalSettings>,
    /// Custom variables
    pub variables: HashMap<String, String>,
    /// Notes/comments
    pub notes: String,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new("Default")
    }
}

impl Workspace {
    pub fn new(name: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            version: WORKSPACE_VERSION,
            name: name.into(),
            created_at: now,
            modified_at: now,
            sessions: Vec::new(),
            layout: WindowLayout::default(),
            active_tab: 0,
            side_panel: SidePanelState::default(),
            terminal_settings: HashMap::new(),
            variables: HashMap::new(),
            notes: String::new(),
        }
    }
    
    /// Touch modified timestamp
    pub fn touch(&mut self) {
        self.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Add a session
    pub fn add_session(&mut self, session: SessionState) {
        self.sessions.push(session);
        self.touch();
    }
    
    /// Remove a session by ID
    pub fn remove_session(&mut self, id: &str) -> bool {
        let initial_len = self.sessions.len();
        self.sessions.retain(|s| s.id != id);
        if self.sessions.len() != initial_len {
            self.touch();
            true
        } else {
            false
        }
    }
    
    /// Save to file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }
    
    /// Load from file
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
    
    /// Export to portable format (base64 encoded)
    pub fn export(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        base64::encode(json.as_bytes())
    }
    
    /// Import from portable format
    pub fn import(data: &str) -> Option<Self> {
        let bytes = base64::decode(data).ok()?;
        let json = String::from_utf8(bytes).ok()?;
        serde_json::from_str(&json).ok()
    }
}

/// Saved session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Session ID
    pub id: String,
    /// Session name/title
    pub name: String,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Connection parameters
    pub connection_params: ConnectionParams,
    /// Was connected when saved
    pub was_connected: bool,
    /// Auto-reconnect on restore
    pub auto_reconnect: bool,
    /// Scroll buffer content (optional, can be large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_buffer: Option<String>,
    /// Command history
    pub command_history: Vec<String>,
    /// Tab index
    pub tab_index: usize,
}

/// Connection type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Serial,
    Tcp,
    Telnet,
    Ssh,
    Bluetooth,
}

/// Connection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectionParams {
    Serial {
        port: String,
        baud_rate: u32,
        data_bits: u8,
        stop_bits: u8,
        parity: String,
        flow_control: String,
    },
    Tcp {
        host: String,
        port: u16,
    },
    Telnet {
        host: String,
        port: u16,
    },
    Ssh {
        host: String,
        port: u16,
        username: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        key_file: Option<String>,
        #[serde(skip)]
        password: Option<String>, // Never save passwords
    },
    Bluetooth {
        device_id: String,
        device_name: String,
        service_uuid: String,
    },
}

/// Window layout state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowLayout {
    /// Window position (x, y)
    pub position: Option<(i32, i32)>,
    /// Window size (width, height)
    pub size: (u32, u32),
    /// Maximized state
    pub maximized: bool,
    /// Split layout
    pub split: Option<SplitLayout>,
    /// Side panel width
    pub side_panel_width: f32,
    /// Side panel visible
    pub side_panel_visible: bool,
}

impl Default for WindowLayout {
    fn default() -> Self {
        Self {
            position: None,
            size: (1280, 800),
            maximized: false,
            split: None,
            side_panel_width: 250.0,
            side_panel_visible: true,
        }
    }
}

/// Split layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitLayout {
    /// Split orientation (horizontal or vertical)
    pub orientation: SplitOrientation,
    /// Split ratio (0.0 - 1.0)
    pub ratio: f32,
    /// First pane content
    pub first: SplitContent,
    /// Second pane content
    pub second: SplitContent,
}

/// Split orientation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SplitOrientation {
    Horizontal,
    Vertical,
}

/// Content of a split pane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitContent {
    /// Session by ID
    Session(String),
    /// Nested split
    Split(Box<SplitLayout>),
    /// Empty pane
    Empty,
}

/// Side panel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidePanelState {
    /// Active tab
    pub active_tab: SidePanelTab,
    /// Collapsed groups
    pub collapsed_groups: Vec<String>,
    /// Profile filter
    pub profile_filter: Option<String>,
    /// Search query
    pub search_query: String,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            active_tab: SidePanelTab::Profiles,
            collapsed_groups: Vec::new(),
            profile_filter: None,
            search_query: String::new(),
        }
    }
}

/// Side panel tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidePanelTab {
    Profiles,
    Commands,
    History,
    Chart,
    Settings,
}

/// Terminal settings per session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    /// Local echo
    pub local_echo: bool,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Hex view mode
    pub hex_view: bool,
    /// Line ending
    pub line_ending: String,
    /// Font size override
    pub font_size: Option<f32>,
    /// Theme override
    pub theme: Option<String>,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            local_echo: false,
            show_timestamps: false,
            hex_view: false,
            line_ending: "CRLF".to_string(),
            font_size: None,
            theme: None,
        }
    }
}

/// Workspace manager
#[derive(Debug)]
pub struct WorkspaceManager {
    /// Current workspace
    pub current: Option<Workspace>,
    /// Workspace file path
    pub file_path: Option<PathBuf>,
    /// Auto-save enabled
    pub auto_save: bool,
    /// Auto-save interval (seconds)
    pub auto_save_interval: u64,
    /// Recent workspaces
    pub recent: Vec<PathBuf>,
    /// Maximum recent count
    pub max_recent: usize,
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            current: None,
            file_path: None,
            auto_save: true,
            auto_save_interval: 60,
            recent: Vec::new(),
            max_recent: 10,
        }
    }
    
    /// Create new workspace
    pub fn new_workspace(&mut self, name: &str) {
        self.current = Some(Workspace::new(name));
        self.file_path = None;
    }
    
    /// Save current workspace
    pub fn save(&mut self) -> std::io::Result<bool> {
        if let (Some(workspace), Some(path)) = (&self.current, &self.file_path) {
            workspace.save(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Save workspace to new path
    pub fn save_as(&mut self, path: PathBuf) -> std::io::Result<()> {
        if let Some(workspace) = &mut self.current {
            workspace.touch();
            workspace.save(&path)?;
            self.file_path = Some(path.clone());
            self.add_recent(path);
        }
        Ok(())
    }
    
    /// Load workspace from path
    pub fn load(&mut self, path: PathBuf) -> std::io::Result<()> {
        let workspace = Workspace::load(&path)?;
        self.current = Some(workspace);
        self.file_path = Some(path.clone());
        self.add_recent(path);
        Ok(())
    }
    
    /// Add to recent list
    fn add_recent(&mut self, path: PathBuf) {
        self.recent.retain(|p| p != &path);
        self.recent.insert(0, path);
        self.recent.truncate(self.max_recent);
    }
    
    /// Get current workspace
    pub fn workspace(&self) -> Option<&Workspace> {
        self.current.as_ref()
    }
    
    /// Get mutable current workspace
    pub fn workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.current.as_mut()
    }
    
    /// Check if workspace is modified
    pub fn is_modified(&self) -> bool {
        if let Some(workspace) = &self.current {
            workspace.modified_at > workspace.created_at
        } else {
            false
        }
    }
    
    /// Clear current workspace
    pub fn close(&mut self) {
        self.current = None;
        self.file_path = None;
    }
}

/// Simple base64 implementation (for portable export)
mod base64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    pub fn encode(data: &[u8]) -> String {
        let mut result = String::new();
        
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).map(|&b| b as u32).unwrap_or(0);
            let b2 = chunk.get(2).map(|&b| b as u32).unwrap_or(0);
            
            let n = (b0 << 16) | (b1 << 8) | b2;
            
            result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
            result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
            
            if chunk.len() > 1 {
                result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            
            if chunk.len() > 2 {
                result.push(ALPHABET[(n & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        
        result
    }
    
    pub fn decode(data: &str) -> Result<Vec<u8>, ()> {
        let mut result = Vec::new();
        let data = data.trim_end_matches('=');
        
        let decode_char = |c: char| -> Option<u32> {
            ALPHABET.iter().position(|&b| b == c as u8).map(|p| p as u32)
        };
        
        let chars: Vec<char> = data.chars().collect();
        
        for chunk in chars.chunks(4) {
            if chunk.len() < 2 {
                break;
            }
            
            let n0 = decode_char(chunk[0]).ok_or(())?;
            let n1 = decode_char(chunk[1]).ok_or(())?;
            let n2 = chunk.get(2).and_then(|&c| decode_char(c)).unwrap_or(0);
            let n3 = chunk.get(3).and_then(|&c| decode_char(c)).unwrap_or(0);
            
            let n = (n0 << 18) | (n1 << 12) | (n2 << 6) | n3;
            
            result.push(((n >> 16) & 0xFF) as u8);
            if chunk.len() > 2 {
                result.push(((n >> 8) & 0xFF) as u8);
            }
            if chunk.len() > 3 {
                result.push((n & 0xFF) as u8);
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workspace_creation() {
        let workspace = Workspace::new("Test");
        assert_eq!(workspace.name, "Test");
        assert_eq!(workspace.version, WORKSPACE_VERSION);
    }
    
    #[test]
    fn test_workspace_export_import() {
        let mut workspace = Workspace::new("Test");
        workspace.notes = "Test notes".to_string();
        workspace.variables.insert("key".to_string(), "value".to_string());
        
        let exported = workspace.export();
        let imported = Workspace::import(&exported).unwrap();
        
        assert_eq!(imported.name, "Test");
        assert_eq!(imported.notes, "Test notes");
        assert_eq!(imported.variables.get("key"), Some(&"value".to_string()));
    }
    
    #[test]
    fn test_base64() {
        let data = b"Hello, World!";
        let encoded = base64::encode(data);
        let decoded = base64::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
    
    #[test]
    fn test_workspace_manager() {
        let mut manager = WorkspaceManager::new();
        manager.new_workspace("Test");
        
        assert!(manager.workspace().is_some());
        assert_eq!(manager.workspace().unwrap().name, "Test");
    }
}


