//! Configuration module
//!
//! Handles application settings and connection profiles

mod settings;

pub use settings::{AppConfig, ConnectionProfile, ProfileType};

use directories::ProjectDirs;
use std::path::PathBuf;

/// Get the application configuration directory
pub fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "termicon", "Termicon")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

/// Get the application data directory
pub fn data_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "termicon", "Termicon")
        .map(|dirs| dirs.data_dir().to_path_buf())
}

/// Get the plugin directory
pub fn plugin_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("plugins"))
}

/// Get the log directory
pub fn log_dir() -> Option<PathBuf> {
    data_dir().map(|d| d.join("logs"))
}

/// Initialize application directories
pub fn init_directories() -> std::io::Result<()> {
    if let Some(dir) = config_dir() {
        std::fs::create_dir_all(&dir)?;
    }
    if let Some(dir) = data_dir() {
        std::fs::create_dir_all(&dir)?;
    }
    if let Some(dir) = plugin_dir() {
        std::fs::create_dir_all(&dir)?;
    }
    if let Some(dir) = log_dir() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}






