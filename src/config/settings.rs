//! Application settings and connection profiles

use crate::core::codec::CodecType;
use crate::core::logger::LogFormat;
use crate::core::transport::{SerialConfig, SerialFlowControl, SerialParity, TcpConfig, TelnetConfig};
use crate::i18n::Locale;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application language
    pub locale: String,
    /// Window state
    pub window: WindowConfig,
    /// Terminal settings
    pub terminal: TerminalConfig,
    /// Logging settings
    pub logging: LoggingConfig,
    /// Auto-connect settings
    pub autoconnect: AutoConnectSettings,
    /// Saved connection profiles
    pub profiles: Vec<ConnectionProfile>,
    /// Recently used connections
    pub recent_connections: Vec<String>,
    /// Custom macros
    pub macros: Vec<MacroDefinition>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: "en".to_string(),
            window: WindowConfig::default(),
            terminal: TerminalConfig::default(),
            logging: LoggingConfig::default(),
            autoconnect: AutoConnectSettings::default(),
            profiles: Vec::new(),
            recent_connections: Vec::new(),
            macros: default_macros(),
        }
    }
}

impl AppConfig {
    /// Load config from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = super::config_dir()
            .ok_or("Could not determine config directory")?
            .join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = super::config_dir()
            .ok_or("Could not determine config directory")?
            .join("config.toml");

        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// Get locale
    pub fn locale(&self) -> Locale {
        Locale::from_code(&self.locale).unwrap_or_default()
    }

    /// Set locale
    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale.code().to_string();
    }
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window width
    pub width: f32,
    /// Window height
    pub height: f32,
    /// Window X position
    pub x: Option<f32>,
    /// Window Y position
    pub y: Option<f32>,
    /// Maximized state
    pub maximized: bool,
    /// Show toolbar
    pub show_toolbar: bool,
    /// Show status bar
    pub show_status_bar: bool,
    /// Theme (light/dark)
    pub theme: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 800.0,
            x: None,
            y: None,
            maximized: false,
            show_toolbar: true,
            show_status_bar: true,
            theme: "dark".to_string(),
        }
    }
}

/// Terminal settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Font family
    pub font_family: String,
    /// Font size
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Local echo
    pub local_echo: bool,
    /// Line ending type
    pub line_ending: LineEnding,
    /// Default display mode
    pub display_mode: CodecType,
    /// Scroll buffer size (lines)
    pub scroll_buffer: usize,
    /// Scroll on output
    pub scroll_on_output: bool,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Highlight patterns (pattern -> color)
    pub highlights: HashMap<String, String>,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_family: "JetBrains Mono".to_string(),
            font_size: 13.0,
            line_height: 1.2,
            local_echo: false,
            line_ending: LineEnding::CrLf,
            display_mode: CodecType::Text,
            scroll_buffer: 10000,
            scroll_on_output: true,
            show_timestamps: false,
            highlights: HashMap::new(),
        }
    }
}

/// Line ending type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LineEnding {
    /// Carriage Return only
    Cr,
    /// Line Feed only
    Lf,
    /// Both CR and LF
    #[default]
    CrLf,
}

impl LineEnding {
    /// Get the byte sequence for this line ending
    pub fn bytes(&self) -> &'static [u8] {
        match self {
            Self::Cr => b"\r",
            Self::Lf => b"\n",
            Self::CrLf => b"\r\n",
        }
    }
}

/// Logging settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable logging by default
    pub enabled: bool,
    /// Default log directory
    pub directory: Option<PathBuf>,
    /// Default log format
    pub format: LogFormat,
    /// Add timestamps
    pub timestamps: bool,
    /// Auto-rotate logs
    pub auto_rotate: bool,
    /// Max log file size in MB
    pub max_size_mb: u64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: super::log_dir(),
            format: LogFormat::Text,
            timestamps: true,
            auto_rotate: true,
            max_size_mb: 10,
        }
    }
}

/// Auto-connect settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConnectSettings {
    /// Enable auto-reconnect
    pub enabled: bool,
    /// Delay between reconnect attempts (seconds)
    pub delay_secs: u64,
    /// Maximum reconnect attempts (0 = unlimited)
    pub max_attempts: u32,
    /// Show notification on reconnect
    pub notify: bool,
}

impl Default for AutoConnectSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            delay_secs: 5,
            max_attempts: 0,
            notify: true,
        }
    }
}

/// Connection profile type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileType {
    /// Serial port connection
    Serial(SerialConfig),
    /// TCP connection
    Tcp(TcpConfig),
    /// Telnet connection
    Telnet(TelnetConfig),
}

/// Connection profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    /// Profile name
    pub name: String,
    /// Profile description
    pub description: Option<String>,
    /// Connection type and settings
    pub connection: ProfileType,
    /// Terminal settings override
    pub terminal: Option<TerminalConfig>,
    /// Auto-connect on startup
    pub auto_connect: bool,
    /// Startup macro (run after connect)
    pub startup_macro: Option<String>,
    /// Profile icon/color
    pub color: Option<String>,
}

impl ConnectionProfile {
    /// Create a new serial profile
    pub fn serial(name: &str, port: &str, baud_rate: u32) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            connection: ProfileType::Serial(SerialConfig::new(port, baud_rate)),
            terminal: None,
            auto_connect: false,
            startup_macro: None,
            color: None,
        }
    }

    /// Create a new TCP profile
    pub fn tcp(name: &str, host: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            connection: ProfileType::Tcp(TcpConfig::new(host, port)),
            terminal: None,
            auto_connect: false,
            startup_macro: None,
            color: None,
        }
    }

    /// Create a new Telnet profile
    pub fn telnet(name: &str, host: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            connection: ProfileType::Telnet(TelnetConfig::new(host)),
            terminal: None,
            auto_connect: false,
            startup_macro: None,
            color: None,
        }
    }
}

/// Macro definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDefinition {
    /// Macro name
    pub name: String,
    /// Data to send (supports escape sequences)
    pub data: String,
    /// Keyboard shortcut (e.g., "F1", "Ctrl+M")
    pub shortcut: Option<String>,
    /// Send as hex
    pub is_hex: bool,
}

impl MacroDefinition {
    /// Create a new macro
    pub fn new(name: &str, data: &str) -> Self {
        Self {
            name: name.to_string(),
            data: data.to_string(),
            shortcut: None,
            is_hex: false,
        }
    }
}

/// Default macros
fn default_macros() -> Vec<MacroDefinition> {
    vec![
        MacroDefinition {
            name: "CR".to_string(),
            data: "\\r".to_string(),
            shortcut: Some("Ctrl+Enter".to_string()),
            is_hex: false,
        },
        MacroDefinition {
            name: "LF".to_string(),
            data: "\\n".to_string(),
            shortcut: None,
            is_hex: false,
        },
        MacroDefinition {
            name: "CRLF".to_string(),
            data: "\\r\\n".to_string(),
            shortcut: None,
            is_hex: false,
        },
        MacroDefinition {
            name: "ESC".to_string(),
            data: "\\x1b".to_string(),
            shortcut: None,
            is_hex: false,
        },
        MacroDefinition {
            name: "Break".to_string(),
            data: "__BREAK__".to_string(),
            shortcut: Some("Ctrl+Break".to_string()),
            is_hex: false,
        },
    ]
}








