//! Snippet (Command Macro) Management
//!
//! Supports quick command execution, macros, and command sequences

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Snippet type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnippetType {
    /// Single command
    Command,
    /// Multi-line script
    Script,
    /// Key sequence (special keys)
    KeySequence,
    /// Binary data (hex)
    Binary,
}

/// Line ending type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    /// No line ending
    None,
    /// Carriage Return (\r)
    Cr,
    /// Line Feed (\n)
    Lf,
    /// CR+LF (\r\n)
    CrLf,
}

impl LineEnding {
    /// Get line ending bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            LineEnding::None => &[],
            LineEnding::Cr => &[b'\r'],
            LineEnding::Lf => &[b'\n'],
            LineEnding::CrLf => &[b'\r', b'\n'],
        }
    }
}

impl Default for LineEnding {
    fn default() -> Self {
        LineEnding::CrLf
    }
}

/// A snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub description: String,
    pub snippet_type: SnippetType,
    pub content: String,
    pub folder: Option<String>,
    pub shortcut: Option<String>, // e.g., "Ctrl+1"
    pub line_ending: LineEnding,
    #[serde(default)]
    pub delay_ms: u64, // Delay between lines for Script type
    #[serde(default)]
    pub color: Option<String>,
}

impl Snippet {
    /// Create new command snippet
    pub fn new_command(name: &str, command: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: String::new(),
            snippet_type: SnippetType::Command,
            content: command.to_string(),
            folder: None,
            shortcut: None,
            line_ending: LineEnding::CrLf,
            delay_ms: 0,
            color: None,
        }
    }

    /// Create new script snippet
    pub fn new_script(name: &str, script: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: String::new(),
            snippet_type: SnippetType::Script,
            content: script.to_string(),
            folder: None,
            shortcut: None,
            line_ending: LineEnding::CrLf,
            delay_ms: 100,
            color: None,
        }
    }

    /// Create new binary snippet from hex
    pub fn new_binary(name: &str, hex: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: String::new(),
            snippet_type: SnippetType::Binary,
            content: hex.to_string(),
            folder: None,
            shortcut: None,
            line_ending: LineEnding::None,
            delay_ms: 0,
            color: None,
        }
    }

    /// Get content as bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        match self.snippet_type {
            SnippetType::Binary => {
                // Parse hex string
                hex::decode(self.content.replace(" ", "").replace("\n", ""))
                    .unwrap_or_default()
            }
            SnippetType::KeySequence => {
                // Parse key sequences like {ENTER}, {TAB}, etc.
                self.parse_key_sequence()
            }
            _ => {
                let mut bytes = self.content.as_bytes().to_vec();
                bytes.extend_from_slice(self.line_ending.as_bytes());
                bytes
            }
        }
    }

    /// Get lines for script type
    pub fn lines(&self) -> Vec<String> {
        self.content.lines().map(|s| s.to_string()).collect()
    }

    /// Parse key sequence
    fn parse_key_sequence(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let mut chars = self.content.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                // Parse key name
                let mut key_name = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '}' {
                        chars.next();
                        break;
                    }
                    key_name.push(chars.next().unwrap());
                }

                match key_name.to_uppercase().as_str() {
                    "ENTER" | "CR" => result.push(b'\r'),
                    "LF" => result.push(b'\n'),
                    "TAB" => result.push(b'\t'),
                    "ESC" | "ESCAPE" => result.push(0x1B),
                    "BS" | "BACKSPACE" => result.push(0x08),
                    "DEL" | "DELETE" => result.push(0x7F),
                    "NULL" | "NUL" => result.push(0x00),
                    "SPACE" => result.push(b' '),
                    "CTRL-C" | "ETX" => result.push(0x03),
                    "CTRL-D" | "EOT" => result.push(0x04),
                    "CTRL-Z" | "SUB" => result.push(0x1A),
                    _ => {
                        // Try hex format: {0x1B}
                        if key_name.starts_with("0x") || key_name.starts_with("0X") {
                            if let Ok(b) = u8::from_str_radix(&key_name[2..], 16) {
                                result.push(b);
                            }
                        }
                    }
                }
            } else {
                result.push(c as u8);
            }
        }

        result
    }
}

/// Snippet manager
pub struct SnippetManager {
    snippets: HashMap<String, Snippet>,
    folders: Vec<String>,
    config_path: PathBuf,
}

impl SnippetManager {
    /// Create new snippet manager
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let mut manager = Self {
            snippets: HashMap::new(),
            folders: Vec::new(),
            config_path,
        };
        manager.load().ok();
        
        // Add some default snippets if empty
        if manager.snippets.is_empty() {
            manager.add_defaults();
        }
        
        manager
    }

    /// Get config path
    fn get_config_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "termicon", "Termicon") {
            let config_dir = proj_dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            config_dir.join("snippets.json")
        } else {
            PathBuf::from("snippets.json")
        }
    }

    /// Add default snippets
    fn add_defaults(&mut self) {
        // Common commands
        self.add(Snippet::new_command("Help", "help"));
        self.add(Snippet::new_command("Clear", "clear"));
        self.add(Snippet::new_command("List files", "ls -la"));
        self.add(Snippet::new_command("System info", "uname -a"));
        
        // Embedded/microcontroller commands
        let mut at = Snippet::new_command("AT Check", "AT");
        at.folder = Some("AT Commands".to_string());
        self.add(at);
        
        let mut at_ver = Snippet::new_command("AT Version", "AT+GMR");
        at_ver.folder = Some("AT Commands".to_string());
        self.add(at_ver);
        
        self.folders.push("AT Commands".to_string());
    }

    /// Load snippets from disk
    pub fn load(&mut self) -> Result<(), String> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read snippets: {}", e))?;

        let data: SnippetData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse snippets: {}", e))?;

        self.snippets = data.snippets.into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();
        self.folders = data.folders;

        Ok(())
    }

    /// Save snippets to disk
    pub fn save(&self) -> Result<(), String> {
        let data = SnippetData {
            snippets: self.snippets.values().cloned().collect(),
            folders: self.folders.clone(),
        };

        let content = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize snippets: {}", e))?;

        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write snippets: {}", e))?;

        Ok(())
    }

    /// Add a snippet
    pub fn add(&mut self, snippet: Snippet) {
        self.snippets.insert(snippet.id.clone(), snippet);
        let _ = self.save();
    }

    /// Remove a snippet
    pub fn remove(&mut self, id: &str) -> Option<Snippet> {
        let snippet = self.snippets.remove(id);
        if snippet.is_some() {
            let _ = self.save();
        }
        snippet
    }

    /// Get a snippet
    pub fn get(&self, id: &str) -> Option<&Snippet> {
        self.snippets.get(id)
    }

    /// Get mutable snippet
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Snippet> {
        self.snippets.get_mut(id)
    }

    /// Get all snippets
    pub fn all(&self) -> Vec<&Snippet> {
        self.snippets.values().collect()
    }

    /// Get snippets by folder
    pub fn by_folder(&self, folder: Option<&str>) -> Vec<&Snippet> {
        self.snippets.values()
            .filter(|s| s.folder.as_deref() == folder)
            .collect()
    }

    /// Get snippet by shortcut
    pub fn by_shortcut(&self, shortcut: &str) -> Option<&Snippet> {
        self.snippets.values()
            .find(|s| s.shortcut.as_deref() == Some(shortcut))
    }

    /// Add folder
    pub fn add_folder(&mut self, name: &str) {
        if !self.folders.contains(&name.to_string()) {
            self.folders.push(name.to_string());
            let _ = self.save();
        }
    }

    /// Get folders
    pub fn folders(&self) -> &[String] {
        &self.folders
    }

    /// Count snippets
    pub fn count(&self) -> usize {
        self.snippets.len()
    }

    /// Update a snippet
    pub fn update(&mut self, snippet: Snippet) {
        self.snippets.insert(snippet.id.clone(), snippet);
        let _ = self.save();
    }

    /// Search snippets by name
    pub fn search(&self, query: &str) -> Vec<&Snippet> {
        let query_lower = query.to_lowercase();
        self.snippets.values()
            .filter(|s| s.name.to_lowercase().contains(&query_lower) ||
                       s.content.to_lowercase().contains(&query_lower))
            .collect()
    }
}

impl Default for SnippetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialized snippet data
#[derive(Debug, Serialize, Deserialize)]
struct SnippetData {
    snippets: Vec<Snippet>,
    folders: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_creation() {
        let snippet = Snippet::new_command("Test", "echo hello");
        assert_eq!(snippet.name, "Test");
        assert_eq!(snippet.content, "echo hello");
    }

    #[test]
    fn test_snippet_as_bytes() {
        let snippet = Snippet::new_command("Test", "hello");
        let bytes = snippet.as_bytes();
        assert_eq!(&bytes[..5], b"hello");
        assert_eq!(&bytes[5..], b"\r\n"); // CrLf default
    }

    #[test]
    fn test_binary_snippet() {
        let snippet = Snippet::new_binary("Test", "48 45 4C 4C 4F");
        let bytes = snippet.as_bytes();
        assert_eq!(bytes, b"HELLO");
    }

    #[test]
    fn test_key_sequence() {
        let mut snippet = Snippet::new_command("Test", "hello{ENTER}");
        snippet.snippet_type = SnippetType::KeySequence;
        snippet.line_ending = LineEnding::None;
        let bytes = snippet.parse_key_sequence();
        assert_eq!(bytes, b"hello\r");
    }
}







