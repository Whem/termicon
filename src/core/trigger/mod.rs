//! Trigger system for automated responses and alerts
//!
//! Provides:
//! - Single pattern triggers
//! - Multi-pattern groups
//! - Conditional triggers
//! - Trigger chains

pub mod advanced;

use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export advanced types
pub use advanced::{
    TriggerContext, AdvancedTriggerManager,
    PatternGroup, PatternDefinition, PatternType, PatternMatchMode,
    TriggerChain, ChainStep, ChainTimeoutAction, SequenceState,
};

/// Trigger condition type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Match exact bytes
    Exact(Vec<u8>),
    /// Match text (case-sensitive)
    Text(String),
    /// Match text (case-insensitive)
    TextIgnoreCase(String),
    /// Match regex pattern
    Regex(String),
    /// Match hex pattern (e.g., "FF 00 *" where * is wildcard)
    HexPattern(String),
}

impl TriggerCondition {
    /// Check if the condition matches the data
    pub fn matches(&self, data: &[u8]) -> Option<String> {
        match self {
            Self::Exact(pattern) => {
                if data.windows(pattern.len()).any(|w| w == pattern.as_slice()) {
                    Some(hex::encode(pattern))
                } else {
                    None
                }
            }
            Self::Text(text) => {
                let data_str = String::from_utf8_lossy(data);
                if data_str.contains(text) {
                    Some(text.clone())
                } else {
                    None
                }
            }
            Self::TextIgnoreCase(text) => {
                let data_str = String::from_utf8_lossy(data).to_lowercase();
                let text_lower = text.to_lowercase();
                if data_str.contains(&text_lower) {
                    Some(text.clone())
                } else {
                    None
                }
            }
            Self::Regex(pattern) => {
                if let Ok(re) = Regex::new(pattern) {
                    let data_str = String::from_utf8_lossy(data);
                    re.find(&data_str).map(|m| m.as_str().to_string())
                } else {
                    None
                }
            }
            Self::HexPattern(pattern) => {
                // Parse pattern: "FF 00 * 01" where * matches any byte
                let parts: Vec<&str> = pattern.split_whitespace().collect();
                let mut pattern_bytes: Vec<Option<u8>> = Vec::new();

                for part in parts {
                    if part == "*" {
                        pattern_bytes.push(None);
                    } else if let Ok(byte) = u8::from_str_radix(part, 16) {
                        pattern_bytes.push(Some(byte));
                    }
                }

                if pattern_bytes.is_empty() {
                    return None;
                }

                // Search for pattern in data
                'outer: for window in data.windows(pattern_bytes.len()) {
                    for (i, &expected) in pattern_bytes.iter().enumerate() {
                        if let Some(exp) = expected {
                            if window[i] != exp {
                                continue 'outer;
                            }
                        }
                        // None (wildcard) matches anything
                    }
                    return Some(hex::encode(window));
                }

                None
            }
        }
    }
}

/// Action to perform when trigger matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerAction {
    /// Play a sound
    PlaySound(String),
    /// Show notification
    Notify(String),
    /// Send data response
    SendResponse(Vec<u8>),
    /// Execute command
    ExecuteCommand(String),
    /// Log message
    LogMessage(String),
    /// Highlight in output (color name)
    Highlight(String),
    /// Stop/pause connection
    StopConnection,
}

/// Trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    /// Unique trigger ID
    pub id: Uuid,
    /// Trigger name
    pub name: String,
    /// Condition to match
    pub condition: TriggerCondition,
    /// Actions to perform
    pub actions: Vec<TriggerAction>,
    /// Is trigger enabled
    pub enabled: bool,
    /// Fire only once
    pub one_shot: bool,
    /// Has fired (for one-shot triggers)
    #[serde(skip)]
    pub fired: bool,
}

impl Trigger {
    /// Create a new trigger
    pub fn new(name: &str, condition: TriggerCondition) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            condition,
            actions: Vec::new(),
            enabled: true,
            one_shot: false,
            fired: false,
        }
    }

    /// Add an action
    #[must_use]
    pub fn with_action(mut self, action: TriggerAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set one-shot mode
    #[must_use]
    pub fn one_shot(mut self, value: bool) -> Self {
        self.one_shot = value;
        self
    }

    /// Check if trigger matches and return matched string
    pub fn check(&self, data: &[u8]) -> Option<String> {
        if !self.enabled {
            return None;
        }

        if self.one_shot && self.fired {
            return None;
        }

        self.condition.matches(data)
    }

    /// Mark trigger as fired
    pub fn mark_fired(&mut self) {
        self.fired = true;
    }

    /// Reset trigger state
    pub fn reset(&mut self) {
        self.fired = false;
    }
}

/// Trigger manager for storing and managing triggers
pub struct TriggerManager {
    triggers: std::collections::HashMap<Uuid, Trigger>,
    config_path: std::path::PathBuf,
}

impl TriggerManager {
    /// Create new trigger manager
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let mut manager = Self {
            triggers: std::collections::HashMap::new(),
            config_path,
        };
        manager.load().ok();
        manager
    }

    /// Get config path
    fn get_config_path() -> std::path::PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "termicon", "Termicon") {
            let config_dir = proj_dirs.config_dir();
            let _ = std::fs::create_dir_all(config_dir);
            config_dir.join("triggers.json")
        } else {
            std::path::PathBuf::from("triggers.json")
        }
    }

    /// Load triggers from disk
    pub fn load(&mut self) -> Result<(), String> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read triggers: {}", e))?;

        let triggers: Vec<Trigger> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse triggers: {}", e))?;

        self.triggers = triggers.into_iter()
            .map(|t| (t.id, t))
            .collect();

        Ok(())
    }

    /// Save triggers to disk
    pub fn save(&self) -> Result<(), String> {
        let triggers: Vec<&Trigger> = self.triggers.values().collect();

        let content = serde_json::to_string_pretty(&triggers)
            .map_err(|e| format!("Failed to serialize triggers: {}", e))?;

        std::fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write triggers: {}", e))?;

        Ok(())
    }

    /// Add a trigger
    pub fn add(&mut self, trigger: Trigger) {
        self.triggers.insert(trigger.id, trigger);
        let _ = self.save();
    }

    /// Remove a trigger
    pub fn remove(&mut self, id: Uuid) -> Option<Trigger> {
        let trigger = self.triggers.remove(&id);
        if trigger.is_some() {
            let _ = self.save();
        }
        trigger
    }

    /// Get a trigger
    pub fn get(&self, id: Uuid) -> Option<&Trigger> {
        self.triggers.get(&id)
    }

    /// Get mutable trigger
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Trigger> {
        self.triggers.get_mut(&id)
    }

    /// Get all triggers
    pub fn all(&self) -> Vec<&Trigger> {
        self.triggers.values().collect()
    }

    /// Get enabled triggers
    pub fn enabled(&self) -> Vec<&Trigger> {
        self.triggers.values()
            .filter(|t| t.enabled)
            .collect()
    }

    /// Check data against all enabled triggers
    pub fn check_all(&self, data: &[u8]) -> Vec<(Uuid, String, Vec<TriggerAction>)> {
        let mut matches = Vec::new();
        
        for trigger in self.triggers.values() {
            if let Some(matched) = trigger.check(data) {
                matches.push((trigger.id, matched, trigger.actions.clone()));
            }
        }
        
        matches
    }

    /// Count triggers
    pub fn count(&self) -> usize {
        self.triggers.len()
    }

    /// Enable trigger
    pub fn enable(&mut self, id: Uuid) {
        if let Some(t) = self.triggers.get_mut(&id) {
            t.enabled = true;
            let _ = self.save();
        }
    }

    /// Disable trigger
    pub fn disable(&mut self, id: Uuid) {
        if let Some(t) = self.triggers.get_mut(&id) {
            t.enabled = false;
            let _ = self.save();
        }
    }

    /// Reset all triggers (clear fired state)
    pub fn reset_all(&mut self) {
        for t in self.triggers.values_mut() {
            t.reset();
        }
    }
}

impl Default for TriggerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Alias for pattern group (trigger group)
pub type TriggerGroup = PatternGroup;

/// Scope for triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerScope {
    /// Global - applies to all sessions
    Global,
    /// Session - applies to specific session
    Session,
    /// Profile - applies to sessions with specific profile
    Profile,
}

/// Conditional trigger from advanced module
pub use advanced::TriggerCondition as AdvancedCondition;
pub use advanced::ConditionOperator as ConditionType;

/// Re-export ConditionalTrigger
pub type ConditionalTrigger = advanced::PatternGroup;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_match() {
        let cond = TriggerCondition::Text("ERROR".to_string());
        assert!(cond.matches(b"An ERROR occurred").is_some());
        assert!(cond.matches(b"All good").is_none());
    }

    #[test]
    fn test_text_ignore_case() {
        let cond = TriggerCondition::TextIgnoreCase("error".to_string());
        assert!(cond.matches(b"An ERROR occurred").is_some());
        assert!(cond.matches(b"An Error occurred").is_some());
    }

    #[test]
    fn test_exact_match() {
        let cond = TriggerCondition::Exact(vec![0xFF, 0x00, 0x01]);
        assert!(cond.matches(&[0x00, 0xFF, 0x00, 0x01, 0x02]).is_some());
        assert!(cond.matches(&[0x00, 0xFF, 0x00, 0x02]).is_none());
    }

    #[test]
    fn test_hex_pattern() {
        let cond = TriggerCondition::HexPattern("FF * 01".to_string());
        assert!(cond.matches(&[0xFF, 0x00, 0x01]).is_some());
        assert!(cond.matches(&[0xFF, 0xAB, 0x01]).is_some());
        assert!(cond.matches(&[0xFF, 0x00, 0x02]).is_none());
    }

    #[test]
    fn test_regex_match() {
        let cond = TriggerCondition::Regex(r"ERROR:\s+\d+".to_string());
        assert!(cond.matches(b"ERROR: 123 occurred").is_some());
        assert!(cond.matches(b"ERROR occurred").is_none());
    }
}
