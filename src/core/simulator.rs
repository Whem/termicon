//! Virtual Device Simulator
//!
//! Create scriptable mock devices for testing and development.
//! Define response rules based on patterns, regex, or packet matching.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use parking_lot::RwLock;

/// Response rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MatchCondition {
    /// Match exact bytes
    Exact { bytes: Vec<u8> },
    /// Match hex pattern (wildcards with *)
    HexPattern { pattern: String },
    /// Match text (substring)
    Text { text: String, case_sensitive: bool },
    /// Match regex
    Regex { pattern: String },
    /// Match any input
    Any,
    /// Match by length range
    LengthRange { min: usize, max: usize },
    /// Combined conditions (all must match)
    All(Vec<MatchCondition>),
    /// Combined conditions (any must match)
    OneOf(Vec<MatchCondition>),
}

impl MatchCondition {
    /// Check if data matches this condition
    pub fn matches(&self, data: &[u8]) -> bool {
        match self {
            Self::Exact { bytes } => data == bytes.as_slice(),
            Self::HexPattern { pattern } => self.match_hex_pattern(data, pattern),
            Self::Text { text, case_sensitive } => {
                let data_str = String::from_utf8_lossy(data);
                if *case_sensitive {
                    data_str.contains(text)
                } else {
                    data_str.to_lowercase().contains(&text.to_lowercase())
                }
            }
            Self::Regex { pattern } => {
                if let Ok(re) = Regex::new(pattern) {
                    let data_str = String::from_utf8_lossy(data);
                    re.is_match(&data_str)
                } else {
                    false
                }
            }
            Self::Any => true,
            Self::LengthRange { min, max } => data.len() >= *min && data.len() <= *max,
            Self::All(conditions) => conditions.iter().all(|c| c.matches(data)),
            Self::OneOf(conditions) => conditions.iter().any(|c| c.matches(data)),
        }
    }

    fn match_hex_pattern(&self, data: &[u8], pattern: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split_whitespace().collect();
        
        if data.len() < pattern_parts.len() {
            return false;
        }

        for (i, part) in pattern_parts.iter().enumerate() {
            if *part == "*" || *part == "??" {
                continue; // Wildcard
            }
            if let Ok(expected) = u8::from_str_radix(part, 16) {
                if data.get(i) != Some(&expected) {
                    return false;
                }
            }
        }
        true
    }
}

/// Response action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ResponseAction {
    /// Send fixed bytes
    Send { data: Vec<u8> },
    /// Send hex string
    SendHex { hex: String },
    /// Send text
    SendText { text: String },
    /// Echo input back
    Echo,
    /// Echo with modification
    EchoModified { prefix: Vec<u8>, suffix: Vec<u8> },
    /// Delay before next action
    Delay { ms: u64 },
    /// Execute multiple actions in sequence
    Sequence(Vec<ResponseAction>),
    /// Random delay range
    RandomDelay { min_ms: u64, max_ms: u64 },
    /// Increment counter and include in response
    Counter { name: String, format: String },
    /// Call Lua script (if enabled)
    Script { code: String },
    /// No response
    None,
}

impl ResponseAction {
    /// Convert SendHex to actual bytes
    pub fn get_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::Send { data } => Some(data.clone()),
            Self::SendHex { hex } => {
                hex::decode(hex.replace(' ', "")).ok()
            }
            Self::SendText { text } => Some(text.as_bytes().to_vec()),
            _ => None,
        }
    }
}

/// Response rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRule {
    /// Rule name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Match condition
    pub condition: MatchCondition,
    /// Response action
    pub action: ResponseAction,
    /// Priority (higher = checked first)
    pub priority: i32,
    /// Enabled
    pub enabled: bool,
    /// One-shot (disable after first match)
    pub one_shot: bool,
    /// Match count
    #[serde(default)]
    pub match_count: u64,
}

impl ResponseRule {
    /// Create a simple echo rule
    pub fn echo(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: Some("Echo all input".to_string()),
            condition: MatchCondition::Any,
            action: ResponseAction::Echo,
            priority: 0,
            enabled: true,
            one_shot: false,
            match_count: 0,
        }
    }

    /// Create a pattern-response rule
    pub fn pattern(name: &str, pattern: &str, response: Vec<u8>) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            condition: MatchCondition::HexPattern { pattern: pattern.to_string() },
            action: ResponseAction::Send { data: response },
            priority: 10,
            enabled: true,
            one_shot: false,
            match_count: 0,
        }
    }
}

/// Device state for stateful simulations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceState {
    /// Current state name
    pub current_state: String,
    /// Variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Counters
    pub counters: HashMap<String, u64>,
}

impl DeviceState {
    /// Get variable
    pub fn get(&self, name: &str) -> Option<&serde_json::Value> {
        self.variables.get(name)
    }

    /// Set variable
    pub fn set(&mut self, name: &str, value: serde_json::Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// Increment counter
    pub fn increment(&mut self, name: &str) -> u64 {
        let counter = self.counters.entry(name.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Get counter
    pub fn counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }
}

/// Virtual device simulator
pub struct VirtualDevice {
    /// Device name
    name: String,
    /// Response rules
    rules: Vec<ResponseRule>,
    /// Device state
    state: Arc<RwLock<DeviceState>>,
    /// Running flag
    running: Arc<RwLock<bool>>,
    /// Response channel
    response_tx: Option<mpsc::Sender<Vec<u8>>>,
}

impl VirtualDevice {
    /// Create new virtual device
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Vec::new(),
            state: Arc::new(RwLock::new(DeviceState::default())),
            running: Arc::new(RwLock::new(false)),
            response_tx: None,
        }
    }

    /// Add a response rule
    pub fn add_rule(&mut self, rule: ResponseRule) {
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove rule by name
    pub fn remove_rule(&mut self, name: &str) {
        self.rules.retain(|r| r.name != name);
    }

    /// Get all rules
    pub fn rules(&self) -> &[ResponseRule] {
        &self.rules
    }

    /// Set response channel
    pub fn set_response_channel(&mut self, tx: mpsc::Sender<Vec<u8>>) {
        self.response_tx = Some(tx);
    }

    /// Process input and generate response
    pub async fn process(&mut self, input: &[u8]) -> Vec<Vec<u8>> {
        let mut responses = Vec::new();

        for rule in &mut self.rules {
            if !rule.enabled {
                continue;
            }

            if rule.condition.matches(input) {
                rule.match_count += 1;

                match &rule.action {
                    ResponseAction::Send { data } => {
                        responses.push(data.clone());
                    }
                    ResponseAction::SendHex { hex } => {
                        if let Ok(data) = hex::decode(hex.replace(' ', "")) {
                            responses.push(data);
                        }
                    }
                    ResponseAction::SendText { text } => {
                        responses.push(text.as_bytes().to_vec());
                    }
                    ResponseAction::Echo => {
                        responses.push(input.to_vec());
                    }
                    ResponseAction::EchoModified { prefix, suffix } => {
                        let mut data = prefix.clone();
                        data.extend_from_slice(input);
                        data.extend_from_slice(suffix);
                        responses.push(data);
                    }
                    ResponseAction::Delay { ms } => {
                        tokio::time::sleep(Duration::from_millis(*ms)).await;
                    }
                    ResponseAction::RandomDelay { min_ms, max_ms } => {
                        use rand::Rng;
                        let delay = rand::thread_rng().gen_range(*min_ms..=*max_ms);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                    }
                    ResponseAction::Counter { name, format } => {
                        let count = self.state.write().increment(name);
                        let text = format.replace("{}", &count.to_string());
                        responses.push(text.into_bytes());
                    }
                    ResponseAction::Sequence(actions) => {
                        for action in actions {
                            if let Some(data) = action.get_bytes() {
                                responses.push(data);
                            }
                            if let ResponseAction::Delay { ms } = action {
                                tokio::time::sleep(Duration::from_millis(*ms)).await;
                            }
                        }
                    }
                    ResponseAction::Script { code: _ } => {
                        // TODO: Lua scripting
                    }
                    ResponseAction::None => {}
                }

                if rule.one_shot {
                    rule.enabled = false;
                }

                // Only match first rule
                break;
            }
        }

        // Send through channel if configured
        if let Some(ref tx) = self.response_tx {
            for response in &responses {
                let _ = tx.send(response.clone()).await;
            }
        }

        responses
    }

    /// Get device state
    pub fn state(&self) -> DeviceState {
        self.state.read().clone()
    }

    /// Set device state
    pub fn set_state(&self, state: DeviceState) {
        *self.state.write() = state;
    }

    /// Reset state
    pub fn reset(&self) {
        *self.state.write() = DeviceState::default();
    }

    /// Get device name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Preset device templates
pub struct DeviceTemplates;

impl DeviceTemplates {
    /// Simple echo device
    pub fn echo() -> VirtualDevice {
        let mut device = VirtualDevice::new("Echo Device");
        device.add_rule(ResponseRule::echo("echo_all"));
        device
    }

    /// AT command responder
    pub fn at_modem() -> VirtualDevice {
        let mut device = VirtualDevice::new("AT Modem");
        
        device.add_rule(ResponseRule {
            name: "AT".to_string(),
            description: Some("Basic AT command".to_string()),
            condition: MatchCondition::Text { text: "AT\r".to_string(), case_sensitive: false },
            action: ResponseAction::SendText { text: "OK\r\n".to_string() },
            priority: 10,
            enabled: true,
            one_shot: false,
            match_count: 0,
        });

        device.add_rule(ResponseRule {
            name: "ATI".to_string(),
            description: Some("Device info".to_string()),
            condition: MatchCondition::Text { text: "ATI\r".to_string(), case_sensitive: false },
            action: ResponseAction::SendText { text: "Termicon Virtual Modem v1.0\r\nOK\r\n".to_string() },
            priority: 10,
            enabled: true,
            one_shot: false,
            match_count: 0,
        });

        device.add_rule(ResponseRule {
            name: "unknown".to_string(),
            description: Some("Unknown command".to_string()),
            condition: MatchCondition::Text { text: "AT".to_string(), case_sensitive: false },
            action: ResponseAction::SendText { text: "ERROR\r\n".to_string() },
            priority: 1,
            enabled: true,
            one_shot: false,
            match_count: 0,
        });

        device
    }

    /// Modbus RTU slave
    pub fn modbus_slave(address: u8) -> VirtualDevice {
        let mut device = VirtualDevice::new(&format!("Modbus Slave {}", address));
        
        // Read Holding Registers (FC 03)
        device.add_rule(ResponseRule {
            name: "read_holding".to_string(),
            description: Some("Read Holding Registers".to_string()),
            condition: MatchCondition::HexPattern { 
                pattern: format!("{:02X} 03 * * * *", address) 
            },
            action: ResponseAction::Sequence(vec![
                ResponseAction::Delay { ms: 10 },
                // Simplified response
                ResponseAction::Send { data: vec![address, 0x03, 0x02, 0x00, 0x64, 0x00, 0x00] },
            ]),
            priority: 10,
            enabled: true,
            one_shot: false,
            match_count: 0,
        });

        device
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_device() {
        let mut device = DeviceTemplates::echo();
        
        let responses = device.process(b"Hello").await;
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], b"Hello");
    }

    #[tokio::test]
    async fn test_at_modem() {
        let mut device = DeviceTemplates::at_modem();
        
        let responses = device.process(b"AT\r").await;
        assert_eq!(responses.len(), 1);
        assert_eq!(String::from_utf8_lossy(&responses[0]), "OK\r\n");
    }

    #[test]
    fn test_match_conditions() {
        let exact = MatchCondition::Exact { bytes: vec![0x01, 0x02, 0x03] };
        assert!(exact.matches(&[0x01, 0x02, 0x03]));
        assert!(!exact.matches(&[0x01, 0x02]));

        let text = MatchCondition::Text { text: "hello".to_string(), case_sensitive: false };
        assert!(text.matches(b"Hello World"));
        assert!(text.matches(b"HELLO"));

        let hex = MatchCondition::HexPattern { pattern: "AA * 03".to_string() };
        assert!(hex.matches(&[0xAA, 0xFF, 0x03]));
        assert!(!hex.matches(&[0xAA, 0xFF, 0x04]));
    }
}



