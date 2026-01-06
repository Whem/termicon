//! Knowledge Base Integration
//!
//! Device knowledge base for storing device information, firmware notes,
//! known issues, and inline hints for better debugging experience.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Device entry in the knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEntry {
    /// Device ID (unique)
    pub id: String,
    /// Device name
    pub name: String,
    /// Manufacturer
    pub manufacturer: Option<String>,
    /// Device model/part number
    pub model: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Firmware versions known
    pub firmware_versions: Vec<FirmwareVersion>,
    /// Communication settings
    pub comm_settings: CommSettings,
    /// Known issues
    pub known_issues: Vec<KnownIssue>,
    /// Protocol hints
    pub protocol_hints: Vec<ProtocolHint>,
    /// Related documentation links
    pub documentation: Vec<DocLink>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom notes
    pub notes: String,
}

impl DeviceEntry {
    /// Create new device entry
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            manufacturer: None,
            model: None,
            description: None,
            firmware_versions: Vec::new(),
            comm_settings: CommSettings::default(),
            known_issues: Vec::new(),
            protocol_hints: Vec::new(),
            documentation: Vec::new(),
            tags: Vec::new(),
            notes: String::new(),
        }
    }
}

/// Firmware version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareVersion {
    /// Version string
    pub version: String,
    /// Release date
    pub release_date: Option<String>,
    /// Changelog/notes
    pub notes: Option<String>,
    /// Known issues specific to this version
    pub issues: Vec<String>,
}

/// Communication settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommSettings {
    /// Preferred baud rates
    pub baud_rates: Vec<u32>,
    /// Default baud rate
    pub default_baud: Option<u32>,
    /// Preferred data format (e.g., "8N1")
    pub data_format: Option<String>,
    /// Flow control preference
    pub flow_control: Option<String>,
    /// Line ending preference
    pub line_ending: Option<String>,
    /// Protocol type (if applicable)
    pub protocol: Option<String>,
    /// Bootloader baud rate (if different)
    pub bootloader_baud: Option<u32>,
}

/// Known issue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownIssue {
    /// Issue ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Symptoms (patterns to match)
    pub symptoms: Vec<String>,
    /// Workaround if any
    pub workaround: Option<String>,
    /// Fixed in version
    pub fixed_in: Option<String>,
    /// Severity (info, warning, error)
    pub severity: IssueSeverity,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Protocol hint for inline assistance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolHint {
    /// Pattern to match (regex or hex)
    pub pattern: String,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Hint message
    pub message: String,
    /// Suggested action
    pub action: Option<String>,
    /// Link to documentation
    pub doc_link: Option<String>,
}

/// Pattern type for matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Regex pattern
    Regex,
    /// Hex pattern (with wildcards)
    Hex,
    /// Text pattern
    Text,
}

/// Documentation link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocLink {
    /// Title
    pub title: String,
    /// URL
    pub url: String,
    /// Link type
    pub link_type: DocLinkType,
}

/// Documentation link type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocLinkType {
    Datasheet,
    UserManual,
    ApplicationNote,
    Tutorial,
    Forum,
    Other,
}

/// Communication diagnostics
#[derive(Debug, Clone)]
pub struct CommDiagnostic {
    /// Issue detected
    pub issue: DiagnosticIssue,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Suggestion
    pub suggestion: String,
}

/// Diagnostic issue types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticIssue {
    /// Baud rate mismatch
    BaudMismatch,
    /// Wrong parity
    ParityError,
    /// Flow control issue
    FlowControlIssue,
    /// Line ending mismatch
    LineEndingMismatch,
    /// No response
    NoResponse,
    /// Garbage data
    GarbageData,
    /// Framing error
    FramingError,
    /// Checksum error
    ChecksumError,
}

impl DiagnosticIssue {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::BaudMismatch => "Baud Rate Mismatch",
            Self::ParityError => "Parity Error",
            Self::FlowControlIssue => "Flow Control Issue",
            Self::LineEndingMismatch => "Line Ending Mismatch",
            Self::NoResponse => "No Response",
            Self::GarbageData => "Garbage Data",
            Self::FramingError => "Framing Error",
            Self::ChecksumError => "Checksum Error",
        }
    }
}

/// Knowledge base
pub struct KnowledgeBase {
    /// Device entries
    devices: HashMap<String, DeviceEntry>,
    /// File path (if file-backed)
    path: Option<PathBuf>,
    /// Inline hints enabled
    hints_enabled: bool,
    /// Cached pattern matchers
    hint_matchers: Vec<(regex::Regex, String)>,
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeBase {
    /// Create new knowledge base
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            path: None,
            hints_enabled: true,
            hint_matchers: Vec::new(),
        }
    }

    /// Load from file
    pub fn load(path: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let devices: HashMap<String, DeviceEntry> = serde_json::from_str(&content)?;
        
        let mut kb = Self {
            devices,
            path: Some(path),
            hints_enabled: true,
            hint_matchers: Vec::new(),
        };
        
        kb.rebuild_matchers();
        Ok(kb)
    }

    /// Save to file
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(ref path) = self.path {
            let content = serde_json::to_string_pretty(&self.devices)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }

    /// Add device entry
    pub fn add_device(&mut self, device: DeviceEntry) {
        self.devices.insert(device.id.clone(), device);
        self.rebuild_matchers();
    }

    /// Get device by ID
    pub fn get_device(&self, id: &str) -> Option<&DeviceEntry> {
        self.devices.get(id)
    }

    /// Search devices by name or tag
    pub fn search(&self, query: &str) -> Vec<&DeviceEntry> {
        let query_lower = query.to_lowercase();
        self.devices.values()
            .filter(|d| {
                d.name.to_lowercase().contains(&query_lower) ||
                d.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)) ||
                d.manufacturer.as_ref().map(|m| m.to_lowercase().contains(&query_lower)).unwrap_or(false)
            })
            .collect()
    }

    /// Rebuild hint pattern matchers
    fn rebuild_matchers(&mut self) {
        self.hint_matchers.clear();
        
        for device in self.devices.values() {
            for hint in &device.protocol_hints {
                if hint.pattern_type == PatternType::Regex {
                    if let Ok(re) = regex::Regex::new(&hint.pattern) {
                        self.hint_matchers.push((re, hint.message.clone()));
                    }
                }
            }
        }
    }

    /// Check data for hints
    pub fn check_hints(&self, data: &[u8]) -> Vec<String> {
        if !self.hints_enabled {
            return Vec::new();
        }

        let text = String::from_utf8_lossy(data);
        let mut hints = Vec::new();

        for (re, message) in &self.hint_matchers {
            if re.is_match(&text) {
                hints.push(message.clone());
            }
        }

        hints
    }

    /// Analyze communication for issues
    pub fn analyze_communication(&self, data: &[u8], expected_baud: u32) -> Vec<CommDiagnostic> {
        let mut diagnostics = Vec::new();

        // Check for garbage data (non-printable characters in expected text)
        let printable_ratio = data.iter()
            .filter(|&&b| b >= 32 && b < 127 || b == b'\r' || b == b'\n' || b == b'\t')
            .count() as f32 / data.len().max(1) as f32;

        if printable_ratio < 0.5 && !data.is_empty() {
            diagnostics.push(CommDiagnostic {
                issue: DiagnosticIssue::GarbageData,
                confidence: ((1.0 - printable_ratio) * 100.0) as u8,
                suggestion: format!(
                    "Received data contains {}% non-printable characters. \
                     This often indicates a baud rate mismatch. \
                     Current setting: {} baud. Try common rates: 9600, 19200, 38400, 57600, 115200",
                    ((1.0 - printable_ratio) * 100.0) as u8,
                    expected_baud
                ),
            });
        }

        // Check for framing patterns (0xFF, 0x00 sequences)
        let framing_pattern = data.windows(2)
            .filter(|w| (w[0] == 0xFF && w[1] == 0xFF) || (w[0] == 0x00 && w[1] == 0x00))
            .count();
        
        if framing_pattern > data.len() / 4 {
            diagnostics.push(CommDiagnostic {
                issue: DiagnosticIssue::FramingError,
                confidence: 70,
                suggestion: "Many consecutive 0xFF or 0x00 bytes detected. \
                            Check data bits and stop bits configuration.".to_string(),
            });
        }

        diagnostics
    }

    /// Get communication suggestions for a device
    pub fn get_comm_suggestions(&self, device_id: &str) -> Option<Vec<String>> {
        let device = self.devices.get(device_id)?;
        let mut suggestions = Vec::new();

        if let Some(baud) = device.comm_settings.default_baud {
            suggestions.push(format!("Recommended baud rate: {}", baud));
        }

        if let Some(ref format) = device.comm_settings.data_format {
            suggestions.push(format!("Data format: {}", format));
        }

        if let Some(ref ending) = device.comm_settings.line_ending {
            suggestions.push(format!("Line ending: {}", ending));
        }

        if !device.known_issues.is_empty() {
            suggestions.push(format!("Note: {} known issues for this device", device.known_issues.len()));
        }

        Some(suggestions)
    }

    /// Enable/disable hints
    pub fn set_hints_enabled(&mut self, enabled: bool) {
        self.hints_enabled = enabled;
    }

    /// Get all devices
    pub fn all_devices(&self) -> impl Iterator<Item = &DeviceEntry> {
        self.devices.values()
    }
}

/// Built-in device definitions
pub fn builtin_devices() -> Vec<DeviceEntry> {
    vec![
        // ESP32
        {
            let mut dev = DeviceEntry::new("esp32", "ESP32");
            dev.manufacturer = Some("Espressif".to_string());
            dev.comm_settings = CommSettings {
                baud_rates: vec![115200, 921600, 1500000],
                default_baud: Some(115200),
                data_format: Some("8N1".to_string()),
                bootloader_baud: Some(115200),
                ..Default::default()
            };
            dev.known_issues.push(KnownIssue {
                id: "esp32-boot-msg".to_string(),
                title: "Boot message garbled".to_string(),
                description: "ESP32 outputs boot messages at 74880 baud before switching to configured rate".to_string(),
                symptoms: vec!["garbage at boot".to_string()],
                workaround: Some("This is normal - wait for application to start".to_string()),
                fixed_in: None,
                severity: IssueSeverity::Info,
            });
            dev.tags = vec!["esp32".to_string(), "iot".to_string(), "wifi".to_string()];
            dev
        },
        // Arduino
        {
            let mut dev = DeviceEntry::new("arduino-uno", "Arduino Uno");
            dev.manufacturer = Some("Arduino".to_string());
            dev.comm_settings = CommSettings {
                baud_rates: vec![9600, 19200, 38400, 57600, 115200],
                default_baud: Some(9600),
                data_format: Some("8N1".to_string()),
                line_ending: Some("NL & CR".to_string()),
                ..Default::default()
            };
            dev.tags = vec!["arduino".to_string(), "atmega".to_string(), "microcontroller".to_string()];
            dev
        },
        // STM32
        {
            let mut dev = DeviceEntry::new("stm32", "STM32 Family");
            dev.manufacturer = Some("STMicroelectronics".to_string());
            dev.comm_settings = CommSettings {
                baud_rates: vec![9600, 115200, 921600],
                default_baud: Some(115200),
                data_format: Some("8N1".to_string()),
                ..Default::default()
            };
            dev.known_issues.push(KnownIssue {
                id: "stm32-dfu".to_string(),
                title: "DFU mode".to_string(),
                description: "STM32 may enter DFU mode instead of normal boot".to_string(),
                symptoms: vec!["no serial output".to_string(), "device not responding".to_string()],
                workaround: Some("Check BOOT0 pin state, should be LOW for normal boot".to_string()),
                fixed_in: None,
                severity: IssueSeverity::Warning,
            });
            dev.tags = vec!["stm32".to_string(), "arm".to_string(), "cortex".to_string()];
            dev
        },
        // Generic Modbus device
        {
            let mut dev = DeviceEntry::new("modbus-rtu", "Modbus RTU Device");
            dev.comm_settings = CommSettings {
                baud_rates: vec![9600, 19200, 38400, 57600, 115200],
                default_baud: Some(9600),
                data_format: Some("8N1".to_string()),
                protocol: Some("Modbus RTU".to_string()),
                ..Default::default()
            };
            dev.protocol_hints.push(ProtocolHint {
                pattern: r"timeout|no response".to_string(),
                pattern_type: PatternType::Regex,
                message: "Modbus timeout - check slave address and communication settings".to_string(),
                action: Some("Verify slave address is correct (1-247)".to_string()),
                doc_link: None,
            });
            dev.tags = vec!["modbus".to_string(), "industrial".to_string(), "plc".to_string()];
            dev
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base() {
        let mut kb = KnowledgeBase::new();
        
        for device in builtin_devices() {
            kb.add_device(device);
        }

        assert!(kb.get_device("esp32").is_some());
        assert!(kb.get_device("nonexistent").is_none());

        let results = kb.search("arduino");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_comm_analysis() {
        let kb = KnowledgeBase::new();
        
        // Garbage data (baud mismatch simulation)
        let garbage = vec![0xFF, 0x00, 0xAB, 0xCD, 0xFF, 0x00, 0x12, 0x34];
        let diagnostics = kb.analyze_communication(&garbage, 115200);
        
        assert!(!diagnostics.is_empty());
        assert!(diagnostics.iter().any(|d| d.issue == DiagnosticIssue::GarbageData));
    }
}




