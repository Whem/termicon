//! Profile management system for saving and loading connection profiles
//!
//! Profiles store:
//! - Connection settings (type, host, port, credentials)
//! - Profile-specific snippets with usage counts
//! - Last used timestamp for sorting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Connection type for profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ProfileType {
    Serial,
    Tcp,
    Telnet,
    Ssh,
    Bluetooth,
}

impl ProfileType {
    pub fn icon(&self) -> &'static str {
        match self {
            ProfileType::Serial => "S/",
            ProfileType::Tcp => "@",
            ProfileType::Telnet => "T>",
            ProfileType::Ssh => "#",
            ProfileType::Bluetooth => "B*",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ProfileType::Serial => "Serial",
            ProfileType::Tcp => "TCP",
            ProfileType::Telnet => "Telnet",
            ProfileType::Ssh => "SSH",
            ProfileType::Bluetooth => "Bluetooth",
        }
    }

    pub fn all() -> &'static [ProfileType] {
        &[
            ProfileType::Serial,
            ProfileType::Tcp,
            ProfileType::Telnet,
            ProfileType::Ssh,
            ProfileType::Bluetooth,
        ]
    }
}

/// Snippet with usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSnippet {
    pub name: String,
    pub content: String,
    pub description: String,
    pub usage_count: u32,
    pub last_used: Option<DateTime<Utc>>,
}

impl ProfileSnippet {
    pub fn new(name: String, content: String, description: String) -> Self {
        Self {
            name,
            content,
            description,
            usage_count: 0,
            last_used: None,
        }
    }

    pub fn record_use(&mut self) {
        self.usage_count += 1;
        self.last_used = Some(Utc::now());
    }
}

/// Serial-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SerialProfileSettings {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: String,
    pub flow_control: String,
}

/// TCP-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TcpProfileSettings {
    pub host: String,
    pub port: u16,
}

/// SSH-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SshProfileSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub use_key: bool,
    pub key_path: String,
    // Password - only saved if user explicitly opts in
    #[serde(default)]
    pub saved_password: Option<String>,
    #[serde(default)]
    pub save_password: bool,
    #[serde(default)]
    pub auto_connect: bool,
    // Advanced settings
    #[serde(default)]
    pub jump_host: Option<String>,
    #[serde(default)]
    pub jump_port: Option<u16>,
    #[serde(default)]
    pub jump_username: Option<String>,
    #[serde(default)]
    pub compression: bool,
    #[serde(default)]
    pub keepalive_interval: Option<u32>,
    #[serde(default)]
    pub connection_timeout: Option<u32>,
    #[serde(default)]
    pub terminal_type: Option<String>,
    #[serde(default)]
    pub x11_forwarding: bool,
    #[serde(default)]
    pub agent_forwarding: bool,
    #[serde(default)]
    pub local_port_forward: Option<String>,
    #[serde(default)]
    pub remote_port_forward: Option<String>,
}

/// Bluetooth-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BluetoothProfileSettings {
    pub device: String,
    pub service_uuid: String,
    pub tx_uuid: String,
    pub rx_uuid: String,
}

/// Connection profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub profile_type: ProfileType,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub use_count: u32,
    pub snippets: Vec<ProfileSnippet>,
    pub favorite: bool,
    // Type-specific settings
    pub serial: Option<SerialProfileSettings>,
    pub tcp: Option<TcpProfileSettings>,
    pub ssh: Option<SshProfileSettings>,
    pub bluetooth: Option<BluetoothProfileSettings>,
}

impl Profile {
    pub fn new(name: String, profile_type: ProfileType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            profile_type,
            created_at: Utc::now(),
            last_used: None,
            use_count: 0,
            snippets: Vec::new(),
            favorite: false,
            serial: None,
            tcp: None,
            ssh: None,
            bluetooth: None,
        }
    }

    pub fn new_serial(name: String, settings: SerialProfileSettings) -> Self {
        let mut p = Self::new(name, ProfileType::Serial);
        p.serial = Some(settings);
        p
    }

    pub fn new_tcp(name: String, settings: TcpProfileSettings) -> Self {
        let mut p = Self::new(name, ProfileType::Tcp);
        p.tcp = Some(settings);
        p
    }

    pub fn new_ssh(name: String, settings: SshProfileSettings) -> Self {
        let mut p = Self::new(name, ProfileType::Ssh);
        p.ssh = Some(settings);
        p
    }

    pub fn new_bluetooth(name: String, settings: BluetoothProfileSettings) -> Self {
        let mut p = Self::new(name, ProfileType::Bluetooth);
        p.bluetooth = Some(settings);
        p
    }

    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used = Some(Utc::now());
    }

    pub fn add_snippet(&mut self, snippet: ProfileSnippet) {
        // Check if snippet with same content exists
        if let Some(existing) = self.snippets.iter_mut().find(|s| s.content == snippet.content) {
            existing.record_use();
        } else {
            self.snippets.push(snippet);
        }
    }

    pub fn record_command(&mut self, command: &str) {
        let cmd = command.trim();
        if cmd.is_empty() || cmd.len() < 2 {
            return;
        }

        // Check if this command already exists
        if let Some(snippet) = self.snippets.iter_mut().find(|s| s.content.trim() == cmd) {
            snippet.record_use();
        } else {
            // Create new snippet from command
            let snippet = ProfileSnippet::new(
                cmd.chars().take(20).collect::<String>() + if cmd.len() > 20 { "..." } else { "" },
                format!("{}\r\n", cmd),
                "Auto-saved command".to_string(),
            );
            self.snippets.push(snippet);
        }
    }

    /// Get snippets sorted by usage count (most used first)
    pub fn sorted_snippets(&self) -> Vec<&ProfileSnippet> {
        let mut snippets: Vec<_> = self.snippets.iter().collect();
        snippets.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        snippets
    }

    pub fn connection_summary(&self) -> String {
        match self.profile_type {
            ProfileType::Serial => {
                if let Some(ref s) = self.serial {
                    format!("{} @ {}", s.port, s.baud_rate)
                } else {
                    "Serial".to_string()
                }
            }
            ProfileType::Tcp => {
                if let Some(ref s) = self.tcp {
                    format!("{}:{}", s.host, s.port)
                } else {
                    "TCP".to_string()
                }
            }
            ProfileType::Telnet => {
                if let Some(ref s) = self.tcp {
                    format!("{}:{}", s.host, s.port)
                } else {
                    "Telnet".to_string()
                }
            }
            ProfileType::Ssh => {
                if let Some(ref s) = self.ssh {
                    format!("{}@{}:{}", s.username, s.host, s.port)
                } else {
                    "SSH".to_string()
                }
            }
            ProfileType::Bluetooth => {
                if let Some(ref s) = self.bluetooth {
                    s.device.clone()
                } else {
                    "Bluetooth".to_string()
                }
            }
        }
    }
}

/// Profile manager - handles saving/loading profiles
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileManager {
    pub profiles: Vec<Profile>,
    #[serde(skip)]
    pub filter: Option<ProfileType>,
    #[serde(skip)]
    pub search_query: String,
}

impl ProfileManager {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            filter: None,
            search_query: String::new(),
        }
    }

    /// Get config directory
    fn config_dir() -> Option<PathBuf> {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "termicon", "Termicon") {
            let config_dir = proj_dirs.config_dir().to_path_buf();
            if !config_dir.exists() {
                let _ = fs::create_dir_all(&config_dir);
            }
            Some(config_dir)
        } else {
            None
        }
    }

    /// Get profiles file path
    fn profiles_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("profiles.json"))
    }

    /// Load profiles from disk
    pub fn load() -> Self {
        if let Some(path) = Self::profiles_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(manager) = serde_json::from_str::<ProfileManager>(&content) {
                        return manager;
                    }
                }
            }
        }
        Self::new()
    }

    /// Save profiles to disk
    pub fn save(&self) {
        if let Some(path) = Self::profiles_path() {
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, content);
            }
        }
    }

    /// Add a new profile
    pub fn add(&mut self, profile: Profile) {
        self.profiles.push(profile);
        self.save();
    }

    /// Remove a profile by ID
    pub fn remove(&mut self, id: &str) {
        self.profiles.retain(|p| p.id != id);
        self.save();
    }

    /// Get profile by ID
    pub fn get(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    /// Get mutable profile by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|p| p.id == id)
    }

    /// Get filtered and sorted profiles
    pub fn filtered_profiles(&self) -> Vec<&Profile> {
        let mut profiles: Vec<_> = self.profiles.iter()
            .filter(|p| {
                // Type filter
                if let Some(filter_type) = self.filter {
                    if p.profile_type != filter_type {
                        return false;
                    }
                }
                // Search filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    if !p.name.to_lowercase().contains(&query) &&
                       !p.connection_summary().to_lowercase().contains(&query) {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort: favorites first, then by use count, then by name
        profiles.sort_by(|a, b| {
            match (a.favorite, b.favorite) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.use_count.cmp(&a.use_count).then(a.name.cmp(&b.name)),
            }
        });

        profiles
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self, id: &str) {
        if let Some(profile) = self.get_mut(id) {
            profile.favorite = !profile.favorite;
            self.save();
        }
    }

    /// Record profile usage
    pub fn record_use(&mut self, id: &str) {
        if let Some(profile) = self.get_mut(id) {
            profile.record_use();
            self.save();
        }
    }

    /// Record command for a profile
    pub fn record_command(&mut self, id: &str, command: &str) {
        if let Some(profile) = self.get_mut(id) {
            profile.record_command(command);
            self.save();
        }
    }

    /// Get count by type
    pub fn count_by_type(&self, profile_type: ProfileType) -> usize {
        self.profiles.iter().filter(|p| p.profile_type == profile_type).count()
    }
}


