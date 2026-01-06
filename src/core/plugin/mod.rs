//! Plugin system for extending Termicon
//!
//! Supports:
//! - Native plugins (dynamic libraries)
//! - Protocol decoders
//! - Custom views

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Plugin error types
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Failed to load plugin: {0}")]
    LoadError(String),
    #[error("Plugin initialization failed: {0}")]
    InitError(String),
    #[error("Plugin API error: {0}")]
    ApiError(String),
}

/// Plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// Plugin type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    /// Protocol decoder
    Decoder,
    /// Custom view/panel
    View,
    /// Data processor
    Processor,
    /// Connection handler
    Connection,
}

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Loaded,
    Active,
    Disabled,
    Error,
}

/// A loaded plugin
pub struct Plugin {
    pub info: PluginInfo,
    pub plugin_type: PluginType,
    pub state: PluginState,
    pub path: PathBuf,
}

impl Plugin {
    /// Create new plugin
    pub fn new(info: PluginInfo, plugin_type: PluginType, path: PathBuf) -> Self {
        Self {
            info,
            plugin_type,
            state: PluginState::Loaded,
            path,
        }
    }

    /// Enable plugin
    pub fn enable(&mut self) {
        self.state = PluginState::Active;
    }

    /// Disable plugin
    pub fn disable(&mut self) {
        self.state = PluginState::Disabled;
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.state == PluginState::Active
    }
}

/// Plugin manager
pub struct PluginManager {
    plugins: HashMap<String, Plugin>,
    plugin_dir: PathBuf,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> Self {
        let plugin_dir = Self::get_plugin_dir();
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }

    /// Get plugin directory
    fn get_plugin_dir() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "termicon", "Termicon") {
            proj_dirs.data_dir().join("plugins")
        } else {
            PathBuf::from("plugins")
        }
    }

    /// Scan and load plugins from directory
    pub fn scan(&mut self) -> Result<usize, PluginError> {
        if !self.plugin_dir.exists() {
            let _ = std::fs::create_dir_all(&self.plugin_dir);
            return Ok(0);
        }

        let mut count = 0;

        if let Ok(entries) = std::fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(_plugin) = self.load_plugin(&path) {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load a plugin from path
    pub fn load_plugin(&mut self, path: &Path) -> Result<String, PluginError> {
        // Look for plugin.json manifest
        let manifest_path = path.join("plugin.json");
        if !manifest_path.exists() {
            return Err(PluginError::NotFound("plugin.json not found".to_string()));
        }

        let manifest_str = std::fs::read_to_string(&manifest_path)
            .map_err(|e| PluginError::LoadError(e.to_string()))?;

        let manifest: serde_json::Value = serde_json::from_str(&manifest_str)
            .map_err(|e| PluginError::LoadError(e.to_string()))?;

        let info = PluginInfo {
            id: manifest["id"].as_str().unwrap_or("unknown").to_string(),
            name: manifest["name"].as_str().unwrap_or("Unknown").to_string(),
            version: manifest["version"].as_str().unwrap_or("0.0.0").to_string(),
            description: manifest["description"].as_str().unwrap_or("").to_string(),
            author: manifest["author"].as_str().unwrap_or("").to_string(),
        };

        let plugin_type = match manifest["type"].as_str().unwrap_or("decoder") {
            "decoder" => PluginType::Decoder,
            "view" => PluginType::View,
            "processor" => PluginType::Processor,
            "connection" => PluginType::Connection,
            _ => PluginType::Decoder,
        };

        let id = info.id.clone();
        let plugin = Plugin::new(info, plugin_type, path.to_path_buf());
        self.plugins.insert(id.clone(), plugin);

        Ok(id)
    }

    /// Unload a plugin
    pub fn unload(&mut self, id: &str) -> Option<Plugin> {
        self.plugins.remove(id)
    }

    /// Get plugin by ID
    pub fn get(&self, id: &str) -> Option<&Plugin> {
        self.plugins.get(id)
    }

    /// Get mutable plugin by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Plugin> {
        self.plugins.get_mut(id)
    }

    /// Get all plugins
    pub fn all(&self) -> Vec<&Plugin> {
        self.plugins.values().collect()
    }

    /// Get active plugins
    pub fn active(&self) -> Vec<&Plugin> {
        self.plugins
            .values()
            .filter(|p| p.is_active())
            .collect()
    }

    /// Enable plugin
    pub fn enable(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self.plugins.get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.enable();
        Ok(())
    }

    /// Disable plugin
    pub fn disable(&mut self, id: &str) -> Result<(), PluginError> {
        let plugin = self.plugins.get_mut(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        plugin.disable();
        Ok(())
    }

    /// Plugin count
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Get plugin directory
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }
}

/// Protocol decoder trait
pub trait ProtocolDecoder: Send + Sync {
    /// Decoder name
    fn name(&self) -> &str;
    
    /// Decode data
    fn decode(&self, data: &[u8]) -> Option<DecodedData>;
    
    /// Can decode this data?
    fn can_decode(&self, data: &[u8]) -> bool;
}

/// Decoded data result
#[derive(Debug, Clone)]
pub struct DecodedData {
    pub protocol: String,
    pub fields: Vec<DecodedField>,
    pub raw: Vec<u8>,
    pub summary: String,
}

/// A decoded field
#[derive(Debug, Clone)]
pub struct DecodedField {
    pub name: String,
    pub value: String,
    pub field_type: String,
    pub offset: usize,
    pub length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager() {
        let manager = PluginManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_plugin_info() {
        let info = PluginInfo {
            id: "test".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test".to_string(),
        };
        let plugin = Plugin::new(info, PluginType::Decoder, PathBuf::from("."));
        assert!(!plugin.is_active());
    }
}
