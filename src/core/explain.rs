//! Explain Mode / Root Cause Hint Engine
//! 
//! Provides:
//! - "Why this failed" explanations
//! - Likely root cause hints
//! - Troubleshooting suggestions
//! - Rule-based diagnostic engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Symptom that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Symptom {
    /// Connection timeout
    ConnectionTimeout,
    /// Connection refused
    ConnectionRefused,
    /// CRC/checksum error
    ChecksumError,
    /// Frame error
    FrameError,
    /// Parity error
    ParityError,
    /// Overrun error
    OverrunError,
    /// No response
    NoResponse,
    /// Partial response
    PartialResponse,
    /// Garbled data
    GarbledData,
    /// Protocol error
    ProtocolError,
    /// Authentication failed
    AuthenticationFailed,
    /// Permission denied
    PermissionDenied,
    /// Resource busy
    ResourceBusy,
    /// Buffer overflow
    BufferOverflow,
    /// Unexpected disconnect
    UnexpectedDisconnect,
    /// Slow response
    SlowResponse,
    /// High latency
    HighLatency,
    /// Packet loss
    PacketLoss,
    /// Custom symptom
    Custom(String),
}

/// Possible root cause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    /// Cause description
    pub description: String,
    /// Likelihood (0.0 - 1.0)
    pub likelihood: f64,
    /// Category
    pub category: CauseCategory,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Related documentation
    pub documentation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CauseCategory {
    Configuration,
    Hardware,
    Software,
    Network,
    Protocol,
    Security,
    Resource,
    Environmental,
    User,
}

/// Diagnostic rule
#[derive(Debug, Clone)]
pub struct DiagnosticRule {
    /// Rule name
    pub name: String,
    /// Symptoms that trigger this rule
    pub symptoms: Vec<Symptom>,
    /// Additional context requirements
    pub context_requirements: HashMap<String, String>,
    /// Root cause if rule matches
    pub root_cause: RootCause,
}

impl DiagnosticRule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            symptoms: Vec::new(),
            context_requirements: HashMap::new(),
            root_cause: RootCause {
                description: String::new(),
                likelihood: 0.5,
                category: CauseCategory::Configuration,
                suggestions: Vec::new(),
                documentation: Vec::new(),
            },
        }
    }

    pub fn when_symptom(mut self, symptom: Symptom) -> Self {
        self.symptoms.push(symptom);
        self
    }

    pub fn when_context(mut self, key: &str, value: &str) -> Self {
        self.context_requirements.insert(key.to_string(), value.to_string());
        self
    }

    pub fn cause(mut self, description: &str, likelihood: f64, category: CauseCategory) -> Self {
        self.root_cause.description = description.to_string();
        self.root_cause.likelihood = likelihood;
        self.root_cause.category = category;
        self
    }

    pub fn suggest(mut self, suggestion: &str) -> Self {
        self.root_cause.suggestions.push(suggestion.to_string());
        self
    }

    pub fn doc(mut self, url: &str) -> Self {
        self.root_cause.documentation.push(url.to_string());
        self
    }
}

/// Diagnostic context
#[derive(Debug, Clone, Default)]
pub struct DiagnosticContext {
    /// Current symptoms
    pub symptoms: Vec<Symptom>,
    /// Context information
    pub context: HashMap<String, String>,
    /// Error messages
    pub error_messages: Vec<String>,
    /// Recent log entries
    pub recent_logs: Vec<String>,
}

impl DiagnosticContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_symptom(&mut self, symptom: Symptom) {
        if !self.symptoms.contains(&symptom) {
            self.symptoms.push(symptom);
        }
    }

    pub fn set_context(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
    }

    pub fn add_error(&mut self, message: &str) {
        self.error_messages.push(message.to_string());
    }
}

/// Diagnostic result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticResult {
    /// Detected symptoms
    pub symptoms: Vec<String>,
    /// Possible root causes (sorted by likelihood)
    pub root_causes: Vec<RootCause>,
    /// Summary explanation
    pub summary: String,
    /// Recommended actions
    pub recommended_actions: Vec<String>,
    /// Timestamp
    pub timestamp: String,
}

/// Root Cause Hint Engine
#[derive(Debug)]
pub struct ExplainEngine {
    /// Diagnostic rules
    rules: Vec<DiagnosticRule>,
}

impl Default for ExplainEngine {
    fn default() -> Self {
        let mut engine = Self { rules: Vec::new() };
        engine.setup_default_rules();
        engine
    }
}

impl ExplainEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic rule
    pub fn add_rule(&mut self, rule: DiagnosticRule) {
        self.rules.push(rule);
    }

    /// Setup default diagnostic rules
    fn setup_default_rules(&mut self) {
        // Serial connection rules
        self.add_rule(
            DiagnosticRule::new("Baud Rate Mismatch")
                .when_symptom(Symptom::GarbledData)
                .when_context("transport", "serial")
                .cause(
                    "Baud rate mismatch between sender and receiver",
                    0.85,
                    CauseCategory::Configuration
                )
                .suggest("Verify baud rate matches on both ends")
                .suggest("Try common baud rates: 9600, 115200")
                .suggest("Check device documentation for correct baud rate")
        );

        self.add_rule(
            DiagnosticRule::new("Wrong Parity/Stop Bits")
                .when_symptom(Symptom::FrameError)
                .when_context("transport", "serial")
                .cause(
                    "Parity or stop bit settings don't match",
                    0.75,
                    CauseCategory::Configuration
                )
                .suggest("Check parity setting (None, Even, Odd)")
                .suggest("Check stop bits (1, 1.5, 2)")
                .suggest("Common setting: 8N1 (8 data, No parity, 1 stop)")
        );

        self.add_rule(
            DiagnosticRule::new("Cable/Connection Issue")
                .when_symptom(Symptom::NoResponse)
                .when_context("transport", "serial")
                .cause(
                    "Physical connection problem - cable disconnected or faulty",
                    0.70,
                    CauseCategory::Hardware
                )
                .suggest("Check cable connections")
                .suggest("Try a different cable")
                .suggest("Verify correct COM port selected")
                .suggest("Check TX/RX wiring (may need crossover)")
        );

        self.add_rule(
            DiagnosticRule::new("Flow Control Mismatch")
                .when_symptom(Symptom::BufferOverflow)
                .when_context("transport", "serial")
                .cause(
                    "Flow control not enabled or mismatched",
                    0.65,
                    CauseCategory::Configuration
                )
                .suggest("Enable hardware flow control (RTS/CTS)")
                .suggest("Or enable software flow control (XON/XOFF)")
                .suggest("Reduce transmission speed")
        );

        // TCP/Network rules
        self.add_rule(
            DiagnosticRule::new("Firewall Blocking")
                .when_symptom(Symptom::ConnectionTimeout)
                .when_context("transport", "tcp")
                .cause(
                    "Firewall blocking the connection",
                    0.80,
                    CauseCategory::Network
                )
                .suggest("Check firewall rules")
                .suggest("Temporarily disable firewall for testing")
                .suggest("Add exception for the port")
        );

        self.add_rule(
            DiagnosticRule::new("Wrong Port/Address")
                .when_symptom(Symptom::ConnectionRefused)
                .when_context("transport", "tcp")
                .cause(
                    "No service listening on the specified port",
                    0.85,
                    CauseCategory::Configuration
                )
                .suggest("Verify IP address and port number")
                .suggest("Check if server is running")
                .suggest("Use netstat to verify listening ports")
        );

        // SSH rules
        self.add_rule(
            DiagnosticRule::new("SSH Auth Failed")
                .when_symptom(Symptom::AuthenticationFailed)
                .when_context("transport", "ssh")
                .cause(
                    "SSH authentication credentials invalid",
                    0.90,
                    CauseCategory::Security
                )
                .suggest("Verify username and password")
                .suggest("Check if SSH key is correctly configured")
                .suggest("Verify user has SSH access")
                .suggest("Check /var/log/auth.log on server")
        );

        // Protocol rules
        self.add_rule(
            DiagnosticRule::new("Modbus CRC Error")
                .when_symptom(Symptom::ChecksumError)
                .when_context("protocol", "modbus")
                .cause(
                    "Data corruption during transmission",
                    0.70,
                    CauseCategory::Hardware
                )
                .suggest("Check cable shielding")
                .suggest("Reduce cable length")
                .suggest("Lower baud rate")
                .suggest("Check for electrical interference")
        );

        self.add_rule(
            DiagnosticRule::new("Modbus No Response")
                .when_symptom(Symptom::NoResponse)
                .when_context("protocol", "modbus")
                .cause(
                    "Device not responding - wrong unit ID or offline",
                    0.75,
                    CauseCategory::Configuration
                )
                .suggest("Verify slave/unit ID")
                .suggest("Check device is powered and online")
                .suggest("Increase response timeout")
                .suggest("Check RS-485 termination")
        );

        // BLE rules
        self.add_rule(
            DiagnosticRule::new("BLE Connection Lost")
                .when_symptom(Symptom::UnexpectedDisconnect)
                .when_context("transport", "ble")
                .cause(
                    "BLE connection unstable - device moved out of range",
                    0.65,
                    CauseCategory::Environmental
                )
                .suggest("Move device closer")
                .suggest("Remove sources of interference")
                .suggest("Check battery level")
                .suggest("Try reconnecting")
        );

        // Generic rules
        self.add_rule(
            DiagnosticRule::new("Resource Busy")
                .when_symptom(Symptom::ResourceBusy)
                .cause(
                    "Port/resource in use by another application",
                    0.90,
                    CauseCategory::Resource
                )
                .suggest("Close other applications using this port")
                .suggest("Check for zombie processes")
                .suggest("Restart the device")
        );

        self.add_rule(
            DiagnosticRule::new("Slow Response")
                .when_symptom(Symptom::SlowResponse)
                .cause(
                    "Device overloaded or network congestion",
                    0.60,
                    CauseCategory::Resource
                )
                .suggest("Reduce request frequency")
                .suggest("Check device CPU/memory usage")
                .suggest("Check network congestion")
        );
    }

    /// Diagnose based on context
    pub fn diagnose(&self, context: &DiagnosticContext) -> DiagnosticResult {
        let mut causes: Vec<RootCause> = Vec::new();

        for rule in &self.rules {
            // Check if all symptoms match
            let symptoms_match = rule.symptoms.iter()
                .all(|s| context.symptoms.contains(s));

            if !symptoms_match {
                continue;
            }

            // Check context requirements
            let context_match = rule.context_requirements.iter()
                .all(|(k, v)| context.context.get(k).map(|cv| cv == v).unwrap_or(false));

            if !context_match && !rule.context_requirements.is_empty() {
                continue;
            }

            causes.push(rule.root_cause.clone());
        }

        // Sort by likelihood
        causes.sort_by(|a, b| b.likelihood.partial_cmp(&a.likelihood).unwrap());

        // Generate summary
        let summary = if causes.is_empty() {
            "Unable to determine root cause. Please provide more information.".to_string()
        } else {
            format!(
                "Most likely cause: {} ({:.0}% confidence)",
                causes[0].description,
                causes[0].likelihood * 100.0
            )
        };

        // Collect all recommended actions
        let recommended_actions: Vec<String> = causes.iter()
            .take(3)
            .flat_map(|c| c.suggestions.clone())
            .collect();

        DiagnosticResult {
            symptoms: context.symptoms.iter().map(|s| format!("{:?}", s)).collect(),
            root_causes: causes,
            summary,
            recommended_actions,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    /// Quick explain for a simple error
    pub fn explain_error(&self, error: &str, transport: &str) -> DiagnosticResult {
        let mut context = DiagnosticContext::new();
        context.set_context("transport", transport);
        context.add_error(error);

        // Parse error message to detect symptoms
        let error_lower = error.to_lowercase();
        
        if error_lower.contains("timeout") {
            context.add_symptom(Symptom::ConnectionTimeout);
        }
        if error_lower.contains("refused") {
            context.add_symptom(Symptom::ConnectionRefused);
        }
        if error_lower.contains("crc") || error_lower.contains("checksum") {
            context.add_symptom(Symptom::ChecksumError);
        }
        if error_lower.contains("auth") || error_lower.contains("password") {
            context.add_symptom(Symptom::AuthenticationFailed);
        }
        if error_lower.contains("permission") || error_lower.contains("denied") {
            context.add_symptom(Symptom::PermissionDenied);
        }
        if error_lower.contains("busy") || error_lower.contains("in use") {
            context.add_symptom(Symptom::ResourceBusy);
        }
        if error_lower.contains("disconnect") {
            context.add_symptom(Symptom::UnexpectedDisconnect);
        }

        self.diagnose(&context)
    }
}


