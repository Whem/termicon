//! Macros Panel - Quick macro buttons (M1-M24) like classic terminal programs
//! 
//! Each macro can be:
//! - A simple command string
//! - A hex sequence
//! - A file path to send
//! - A script reference

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Maximum number of macro slots
pub const MAX_MACROS: usize = 24;

/// Macro content type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MacroContent {
    /// Plain text command
    Text(String),
    /// Hex bytes (e.g., "FF 00 A5")
    Hex(Vec<u8>),
    /// File to send
    File(PathBuf),
    /// Script reference
    Script(String),
    /// Empty slot
    Empty,
}

impl Default for MacroContent {
    fn default() -> Self {
        MacroContent::Empty
    }
}

/// A single macro definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroSlot {
    /// Macro number (1-24)
    pub number: usize,
    /// Display name
    pub name: String,
    /// Content to send
    pub content: MacroContent,
    /// Description/tooltip
    pub description: String,
    /// Append CR+LF after sending
    pub append_crlf: bool,
    /// Keyboard shortcut (e.g., "Ctrl+1")
    pub shortcut: Option<String>,
    /// Usage count
    pub usage_count: u64,
}

impl MacroSlot {
    pub fn new(number: usize) -> Self {
        Self {
            number,
            name: format!("M{}", number),
            content: MacroContent::Empty,
            description: String::new(),
            append_crlf: true,
            shortcut: if number <= 12 {
                Some(format!("F{}", number))
            } else {
                None
            },
            usage_count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.content, MacroContent::Empty)
    }

    /// Get the bytes to send
    pub fn get_bytes(&self) -> Vec<u8> {
        let mut bytes = match &self.content {
            MacroContent::Text(s) => s.as_bytes().to_vec(),
            MacroContent::Hex(b) => b.clone(),
            MacroContent::File(path) => {
                std::fs::read(path).unwrap_or_default()
            }
            MacroContent::Script(_) => Vec::new(), // Scripts handled separately
            MacroContent::Empty => Vec::new(),
        };

        if self.append_crlf && !bytes.is_empty() {
            bytes.extend_from_slice(b"\r\n");
        }

        bytes
    }

    pub fn record_use(&mut self) {
        self.usage_count += 1;
    }
}

/// Macro set - collection of 24 macros
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroSet {
    /// Set name
    pub name: String,
    /// Macros (indexed 0-23 for M1-M24)
    pub macros: Vec<MacroSlot>,
    /// Profile-specific or global
    pub profile_id: Option<String>,
}

impl Default for MacroSet {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            macros: (1..=MAX_MACROS).map(MacroSlot::new).collect(),
            profile_id: None,
        }
    }
}

impl MacroSet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    pub fn get(&self, index: usize) -> Option<&MacroSlot> {
        if index > 0 && index <= MAX_MACROS {
            self.macros.get(index - 1)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut MacroSlot> {
        if index > 0 && index <= MAX_MACROS {
            self.macros.get_mut(index - 1)
        } else {
            None
        }
    }

    pub fn set_macro(&mut self, index: usize, name: &str, content: MacroContent, description: &str) {
        if let Some(slot) = self.get_mut(index) {
            slot.name = name.to_string();
            slot.content = content;
            slot.description = description.to_string();
        }
    }
}

/// Macro manager - handles multiple macro sets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroManager {
    /// Global macro set
    pub global: MacroSet,
    /// Profile-specific macro sets
    pub profile_sets: HashMap<String, MacroSet>,
    /// Config file path
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

impl Default for MacroManager {
    fn default() -> Self {
        Self {
            global: MacroSet::default(),
            profile_sets: HashMap::new(),
            config_path: None,
        }
    }
}

impl MacroManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        
        // Try to load from config
        if let Some(path) = Self::config_file_path() {
            manager.config_path = Some(path.clone());
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(loaded) = serde_json::from_str::<MacroManager>(&data) {
                    manager.global = loaded.global;
                    manager.profile_sets = loaded.profile_sets;
                }
            }
        }

        // Setup some default macros
        if manager.global.macros.iter().all(|m| m.is_empty()) {
            manager.setup_defaults();
        }

        manager
    }

    fn setup_defaults(&mut self) {
        // Common AT commands
        self.global.set_macro(1, "AT", MacroContent::Text("AT".to_string()), "Basic AT command");
        self.global.set_macro(2, "AT+GMR", MacroContent::Text("AT+GMR".to_string()), "Get firmware version");
        self.global.set_macro(3, "AT+RST", MacroContent::Text("AT+RST".to_string()), "Reset device");
        self.global.set_macro(4, "Help", MacroContent::Text("help".to_string()), "Show help");
        
        // Common Linux commands
        self.global.set_macro(5, "ls -la", MacroContent::Text("ls -la".to_string()), "List files");
        self.global.set_macro(6, "pwd", MacroContent::Text("pwd".to_string()), "Print working directory");
        self.global.set_macro(7, "clear", MacroContent::Text("clear".to_string()), "Clear screen");
        
        // Hex sequences
        self.global.set_macro(8, "Ping", MacroContent::Hex(vec![0x00, 0x01, 0x02, 0x03]), "Ping bytes");
        
        // ANSI clear
        self.global.set_macro(9, "CLS", MacroContent::Text("\x1b[2J\x1b[H".to_string()), "Clear terminal (ANSI)");
    }

    fn config_file_path() -> Option<PathBuf> {
        std::env::var("APPDATA").ok()
            .map(|p| PathBuf::from(p).join("termicon").join("macros.json"))
    }

    pub fn save(&self) {
        if let Some(ref path) = self.config_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }

    /// Get macro set for a profile (or global if no profile)
    pub fn get_set(&self, profile_id: Option<&str>) -> &MacroSet {
        match profile_id {
            Some(id) => self.profile_sets.get(id).unwrap_or(&self.global),
            None => &self.global,
        }
    }

    /// Get mutable macro set for a profile
    pub fn get_set_mut(&mut self, profile_id: Option<&str>) -> &mut MacroSet {
        match profile_id {
            Some(id) => {
                if !self.profile_sets.contains_key(id) {
                    let mut set = MacroSet::default();
                    set.profile_id = Some(id.to_string());
                    self.profile_sets.insert(id.to_string(), set);
                }
                self.profile_sets.get_mut(id).unwrap()
            }
            None => &mut self.global,
        }
    }

    /// Record macro usage
    pub fn record_use(&mut self, profile_id: Option<&str>, macro_number: usize) {
        let set = self.get_set_mut(profile_id);
        if let Some(slot) = set.get_mut(macro_number) {
            slot.record_use();
        }
        self.save();
    }
}

/// Parse hex string to bytes
pub fn parse_hex_string(s: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = s.chars()
        .filter(|c| c.is_ascii_hexdigit() || c.is_whitespace())
        .collect();
    
    let hex_chars: String = cleaned.split_whitespace().collect();
    
    if hex_chars.len() % 2 != 0 {
        return Err("Hex string must have even number of characters".to_string());
    }

    (0..hex_chars.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_chars[i..i+2], 16)
                .map_err(|e| format!("Invalid hex: {}", e))
        })
        .collect()
}

/// Format bytes as hex string
pub fn format_hex_bytes(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

