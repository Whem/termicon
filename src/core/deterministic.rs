//! Deterministic Session Mode
//! 
//! Provides reproducible test runs with:
//! - Fixed random seeds
//! - Timing jitter normalization
//! - "Same input → Same output" guarantee
//! 
//! Critical for CI/audit/safety environments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Random seed for deterministic operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DeterministicSeed(pub u64);

impl Default for DeterministicSeed {
    fn default() -> Self {
        Self(42) // Default seed
    }
}

impl DeterministicSeed {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn from_timestamp() -> Self {
        Self(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs())
    }

    /// Simple deterministic random number generator
    pub fn next(&mut self) -> u64 {
        // LCG parameters
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    /// Get random in range [0, max)
    pub fn next_range(&mut self, max: u64) -> u64 {
        self.next() % max
    }

    /// Get random float [0.0, 1.0)
    pub fn next_float(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
}

/// Timing normalization mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimingMode {
    /// Real-time - use actual timestamps
    RealTime,
    /// Normalized - remove jitter, use fixed intervals
    Normalized {
        /// Base interval in milliseconds
        interval_ms: u64,
    },
    /// Instant - no delays at all
    Instant,
    /// Scaled - multiply all timings by factor
    Scaled {
        factor: f64,
    },
}

impl Default for TimingMode {
    fn default() -> Self {
        TimingMode::RealTime
    }
}

/// Deterministic execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicContext {
    /// Random seed
    pub seed: DeterministicSeed,
    /// Timing mode
    pub timing_mode: TimingMode,
    /// Enable deterministic mode
    pub enabled: bool,
    /// Input sequence hash (for verification)
    pub input_hash: Option<String>,
    /// Expected output hash (for verification)
    pub expected_output_hash: Option<String>,
    /// Recorded inputs (for replay)
    #[serde(skip)]
    pub recorded_inputs: Vec<DeterministicInput>,
    /// Recorded outputs (for verification)
    #[serde(skip)]
    pub recorded_outputs: Vec<DeterministicOutput>,
    /// Virtual clock (for normalized timing)
    #[serde(skip)]
    pub virtual_clock: u64,
}

impl Default for DeterministicContext {
    fn default() -> Self {
        Self {
            seed: DeterministicSeed::default(),
            timing_mode: TimingMode::RealTime,
            enabled: false,
            input_hash: None,
            expected_output_hash: None,
            recorded_inputs: Vec::new(),
            recorded_outputs: Vec::new(),
            virtual_clock: 0,
        }
    }
}

/// Recorded input event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicInput {
    /// Virtual timestamp (microseconds)
    pub timestamp_us: u64,
    /// Input data
    pub data: Vec<u8>,
    /// Source identifier
    pub source: String,
}

/// Recorded output event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicOutput {
    /// Virtual timestamp (microseconds)
    pub timestamp_us: u64,
    /// Output data
    pub data: Vec<u8>,
    /// Destination identifier
    pub destination: String,
}

impl DeterministicContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed: DeterministicSeed::new(seed),
            enabled: true,
            ..Default::default()
        }
    }

    /// Enable deterministic mode
    pub fn enable(&mut self, seed: u64) {
        self.seed = DeterministicSeed::new(seed);
        self.enabled = true;
        self.recorded_inputs.clear();
        self.recorded_outputs.clear();
        self.virtual_clock = 0;
    }

    /// Disable deterministic mode
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Get normalized timestamp
    pub fn get_timestamp(&mut self) -> u64 {
        if !self.enabled {
            return std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64;
        }

        match self.timing_mode {
            TimingMode::RealTime => {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros() as u64
            }
            TimingMode::Normalized { interval_ms } => {
                self.virtual_clock += interval_ms * 1000;
                self.virtual_clock
            }
            TimingMode::Instant => {
                self.virtual_clock += 1;
                self.virtual_clock
            }
            TimingMode::Scaled { factor } => {
                let real = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros() as u64;
                (real as f64 * factor) as u64
            }
        }
    }

    /// Record an input event
    pub fn record_input(&mut self, data: &[u8], source: &str) {
        if self.enabled {
            let timestamp_us = self.get_timestamp();
            self.recorded_inputs.push(DeterministicInput {
                timestamp_us,
                data: data.to_vec(),
                source: source.to_string(),
            });
        }
    }

    /// Record an output event
    pub fn record_output(&mut self, data: &[u8], destination: &str) {
        if self.enabled {
            let timestamp_us = self.get_timestamp();
            self.recorded_outputs.push(DeterministicOutput {
                timestamp_us,
                data: data.to_vec(),
                destination: destination.to_string(),
            });
        }
    }

    /// Compute hash of recorded inputs
    pub fn compute_input_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for input in &self.recorded_inputs {
            input.data.hash(&mut hasher);
            input.source.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    /// Compute hash of recorded outputs
    pub fn compute_output_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for output in &self.recorded_outputs {
            output.data.hash(&mut hasher);
            output.destination.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    /// Verify output matches expected
    pub fn verify_output(&self) -> bool {
        match &self.expected_output_hash {
            Some(expected) => &self.compute_output_hash() == expected,
            None => true, // No expectation set
        }
    }

    /// Export session for reproducibility
    pub fn export_session(&self) -> DeterministicSession {
        DeterministicSession {
            seed: self.seed,
            timing_mode: self.timing_mode,
            inputs: self.recorded_inputs.clone(),
            outputs: self.recorded_outputs.clone(),
            input_hash: self.compute_input_hash(),
            output_hash: self.compute_output_hash(),
        }
    }
}

/// Exportable deterministic session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicSession {
    pub seed: DeterministicSeed,
    pub timing_mode: TimingMode,
    pub inputs: Vec<DeterministicInput>,
    pub outputs: Vec<DeterministicOutput>,
    pub input_hash: String,
    pub output_hash: String,
}

impl DeterministicSession {
    /// Save to file
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    /// Load from file
    pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// Reproducible test runner
#[derive(Debug)]
pub struct TestRunner {
    pub context: DeterministicContext,
    pub results: Vec<TestResult>,
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub expected_hash: String,
    pub actual_hash: String,
    pub duration_ms: u64,
    pub error: Option<String>,
}

impl TestRunner {
    pub fn new(seed: u64) -> Self {
        Self {
            context: DeterministicContext::with_seed(seed),
            results: Vec::new(),
        }
    }

    /// Run a reproducible test
    pub fn run_test<F>(&mut self, name: &str, expected_hash: Option<&str>, test_fn: F) -> bool
    where
        F: FnOnce(&mut DeterministicContext) -> Result<(), String>,
    {
        let start = Instant::now();
        
        // Reset context
        self.context.recorded_inputs.clear();
        self.context.recorded_outputs.clear();
        self.context.virtual_clock = 0;

        // Run test
        let error = match test_fn(&mut self.context) {
            Ok(()) => None,
            Err(e) => Some(e),
        };

        let actual_hash = self.context.compute_output_hash();
        let passed = error.is_none() && match expected_hash {
            Some(expected) => actual_hash == expected,
            None => true,
        };

        let result = TestResult {
            name: name.to_string(),
            passed,
            expected_hash: expected_hash.unwrap_or("").to_string(),
            actual_hash,
            duration_ms: start.elapsed().as_millis() as u64,
            error,
        };

        self.results.push(result.clone());
        passed
    }

    /// Get summary
    pub fn summary(&self) -> String {
        let passed = self.results.iter().filter(|r| r.passed).count();
        let total = self.results.len();
        format!(
            "Test Results: {}/{} passed ({:.1}%)",
            passed,
            total,
            (passed as f64 / total as f64) * 100.0
        )
    }
}




