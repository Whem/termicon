//! Connection Profile Management
//!
//! Supports saving and loading connection profiles with all settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Profile type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileType {
    Serial,
    Tcp,
    Telnet,
    Ssh,
}

/// Serial profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialProfile {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: u8,
    pub flow_control: String,
}

impl Default for SerialProfile {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 115200,
            data_bits: 8,
            parity: "None".to_string(),
            stop_bits: 1,
            flow_control: "None".to_string(),
        }
    }
}

/// TCP profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpProfile {
    pub host: String,
    pub port: u16,
    pub timeout_secs: u64,
}

impl Default for TcpProfile {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 23,
            timeout_secs: 10,
        }
    }
}

/// SSH profile settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshProfile {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String, // "password" or "key"
    pub key_path: Option<String>,
    pub term_type: String,
}

impl Default for SshProfile {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            username: String::new(),
            auth_type: "password".to_string(),
            key_path: None,
            term_type: "xterm-256color".to_string(),
        }
    }
}

/// Connection profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub profile_type: ProfileType,
    pub folder: Option<String>,
    pub color: Option<String>,
    pub notes: String,
    #[serde(default)]
    pub serial: Option<SerialProfile>,
    #[serde(default)]
    pub tcp: Option<TcpProfile>,
    #[serde(default)]
    pub ssh: Option<SshProfile>,
    #[serde(default)]
    pub snippets: Vec<String>, // IDs of associated snippets
    #[serde(default)]
    pub auto_connect: bool,
    #[serde(default)]
    pub local_echo: bool,
    #[serde(default)]
    pub log_session: bool,
}

impl Profile {
    /// Create new serial profile
    pub fn new_serial(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            profile_type: ProfileType::Serial,
            folder: None,
            color: None,
            notes: String::new(),
            serial: Some(SerialProfile::default()),
            tcp: None,
            ssh: None,
            snippets: Vec::new(),
            auto_connect: false,
            local_echo: true,
            log_session: false,
        }
    }

    /// Create new TCP profile
    pub fn new_tcp(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            profile_type: ProfileType::Tcp,
            folder: None,
            color: None,
            notes: String::new(),
            serial: None,
            tcp: Some(TcpProfile::default()),
            ssh: None,
            snippets: Vec::new(),
            auto_connect: false,
            local_echo: true,
            log_session: false,
        }
    }

    /// Create new SSH profile
    pub fn new_ssh(name: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            profile_type: ProfileType::Ssh,
            folder: None,
            color: None,
            notes: String::new(),
            serial: None,
            tcp: None,
            ssh: Some(SshProfile::default()),
            snippets: Vec::new(),
            auto_connect: false,
            local_echo: false,
            log_session: false,
        }
    }
}

/// Profile manager
pub struct ProfileManager {
    profiles: HashMap<String, Profile>,
    folders: Vec<String>,
    config_path: PathBuf,
}

impl ProfileManager {
    /// Create new profile manager
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let mut manager = Self {
            profiles: HashMap::new(),
            folders: Vec::new(),
            config_path,
        };
        manager.load().ok();
        manager
    }

    /// Get config path
    fn get_config_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "termicon", "Termicon") {
            let config_dir = proj_dirs.config_dir();
            let _ = fs::create_dir_all(config_dir);
            config_dir.join("profiles.json")
        } else {
            PathBuf::from("profiles.json")
        }
    }

    /// Load profiles from disk
    pub fn load(&mut self) -> Result<(), String> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read profiles: {}", e))?;

        let data: ProfileData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse profiles: {}", e))?;

        self.profiles = data.profiles.into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();
        self.folders = data.folders;

        Ok(())
    }

    /// Save profiles to disk
    pub fn save(&self) -> Result<(), String> {
        let data = ProfileData {
            profiles: self.profiles.values().cloned().collect(),
            folders: self.folders.clone(),
        };

        let content = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize profiles: {}", e))?;

        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write profiles: {}", e))?;

        Ok(())
    }

    /// Add a profile
    pub fn add(&mut self, profile: Profile) {
        self.profiles.insert(profile.id.clone(), profile);
        let _ = self.save();
    }

    /// Remove a profile
    pub fn remove(&mut self, id: &str) -> Option<Profile> {
        let profile = self.profiles.remove(id);
        if profile.is_some() {
            let _ = self.save();
        }
        profile
    }

    /// Get a profile
    pub fn get(&self, id: &str) -> Option<&Profile> {
        self.profiles.get(id)
    }

    /// Get mutable profile
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Profile> {
        self.profiles.get_mut(id)
    }

    /// Get all profiles
    pub fn all(&self) -> Vec<&Profile> {
        self.profiles.values().collect()
    }

    /// Get profiles by folder
    pub fn by_folder(&self, folder: Option<&str>) -> Vec<&Profile> {
        self.profiles.values()
            .filter(|p| p.folder.as_deref() == folder)
            .collect()
    }

    /// Get profiles by type
    pub fn by_type(&self, profile_type: ProfileType) -> Vec<&Profile> {
        self.profiles.values()
            .filter(|p| p.profile_type == profile_type)
            .collect()
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

    /// Count profiles
    pub fn count(&self) -> usize {
        self.profiles.len()
    }

    /// Update a profile
    pub fn update(&mut self, profile: Profile) {
        self.profiles.insert(profile.id.clone(), profile);
        let _ = self.save();
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialized profile data
#[derive(Debug, Serialize, Deserialize)]
struct ProfileData {
    profiles: Vec<Profile>,
    folders: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new_serial("Test Serial");
        assert_eq!(profile.name, "Test Serial");
        assert_eq!(profile.profile_type, ProfileType::Serial);
        assert!(profile.serial.is_some());
    }

    #[test]
    fn test_profile_manager() {
        let manager = ProfileManager::new();
        assert_eq!(manager.count(), 0);
    }
}




