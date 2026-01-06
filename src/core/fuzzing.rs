//! Fuzzing / Robustness Testing
//! 
//! Provides:
//! - Packet Fuzzer
//! - Timing Fuzzer  
//! - Boundary value testing
//! - Protocol stress testing

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use crate::core::deterministic::DeterministicSeed;

/// Fuzzing strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FuzzStrategy {
    /// Random byte mutations
    RandomMutation {
        mutation_rate: f64,
        max_mutations: usize,
    },
    /// Bit flipping
    BitFlip {
        bits_to_flip: usize,
    },
    /// Boundary values (0, 1, 127, 128, 255, etc.)
    BoundaryValue,
    /// Length fuzzing (truncate, extend)
    LengthFuzz {
        min_length: usize,
        max_length: usize,
    },
    /// Repeat sequences
    Repeat {
        min_repeats: usize,
        max_repeats: usize,
    },
    /// Dictionary-based fuzzing
    Dictionary {
        patterns: Vec<Vec<u8>>,
    },
    /// Protocol-aware fuzzing
    ProtocolAware {
        field_offsets: Vec<(usize, usize)>, // (offset, length) pairs
    },
}

/// Timing fuzz configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingFuzzConfig {
    /// Enable timing fuzzing
    pub enabled: bool,
    /// Minimum delay (ms)
    pub min_delay_ms: u64,
    /// Maximum delay (ms)
    pub max_delay_ms: u64,
    /// Jitter percentage (0-100)
    pub jitter_percent: u8,
    /// Burst mode - send multiple packets quickly then pause
    pub burst_mode: bool,
    /// Burst size
    pub burst_size: usize,
}

impl Default for TimingFuzzConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_delay_ms: 0,
            max_delay_ms: 100,
            jitter_percent: 20,
            burst_mode: false,
            burst_size: 5,
        }
    }
}

/// Fuzz test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzResult {
    /// Test iteration number
    pub iteration: usize,
    /// Original data
    pub original: Vec<u8>,
    /// Fuzzed data
    pub fuzzed: Vec<u8>,
    /// Strategy used
    pub strategy: String,
    /// Response received (if any)
    pub response: Option<Vec<u8>>,
    /// Response time (ms)
    pub response_time_ms: Option<u64>,
    /// Did it cause an error/crash?
    pub caused_error: bool,
    /// Error message
    pub error_message: Option<String>,
    /// Interesting finding
    pub interesting: bool,
    /// Notes
    pub notes: String,
}

/// Packet fuzzer
#[derive(Debug, Clone)]
pub struct PacketFuzzer {
    /// Random seed
    pub seed: DeterministicSeed,
    /// Active strategies
    pub strategies: Vec<FuzzStrategy>,
    /// Timing config
    pub timing: TimingFuzzConfig,
    /// Results
    pub results: Vec<FuzzResult>,
    /// Max iterations
    pub max_iterations: usize,
    /// Current iteration
    pub current_iteration: usize,
    /// Stop on first error
    pub stop_on_error: bool,
    /// Boundary values to test
    pub boundary_values: Vec<u8>,
}

impl Default for PacketFuzzer {
    fn default() -> Self {
        Self {
            seed: DeterministicSeed::default(),
            strategies: vec![
                FuzzStrategy::RandomMutation {
                    mutation_rate: 0.1,
                    max_mutations: 5,
                },
                FuzzStrategy::BitFlip { bits_to_flip: 1 },
                FuzzStrategy::BoundaryValue,
            ],
            timing: TimingFuzzConfig::default(),
            results: Vec::new(),
            max_iterations: 1000,
            current_iteration: 0,
            stop_on_error: false,
            boundary_values: vec![0, 1, 127, 128, 255, 0x00, 0xFF, 0x7F, 0x80],
        }
    }
}

impl PacketFuzzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed: DeterministicSeed::new(seed),
            ..Default::default()
        }
    }

    /// Add a fuzzing strategy
    pub fn add_strategy(&mut self, strategy: FuzzStrategy) {
        self.strategies.push(strategy);
    }

    /// Fuzz a packet
    pub fn fuzz_packet(&mut self, original: &[u8]) -> Vec<u8> {
        if self.strategies.is_empty() {
            return original.to_vec();
        }

        // Pick a random strategy
        let strategy_idx = self.seed.next_range(self.strategies.len() as u64) as usize;
        let strategy = self.strategies[strategy_idx].clone();

        self.apply_strategy(original, &strategy)
    }

    /// Apply a specific fuzzing strategy
    pub fn apply_strategy(&mut self, data: &[u8], strategy: &FuzzStrategy) -> Vec<u8> {
        match strategy {
            FuzzStrategy::RandomMutation { mutation_rate, max_mutations } => {
                self.random_mutation(data, *mutation_rate, *max_mutations)
            }
            FuzzStrategy::BitFlip { bits_to_flip } => {
                self.bit_flip(data, *bits_to_flip)
            }
            FuzzStrategy::BoundaryValue => {
                self.boundary_value_fuzz(data)
            }
            FuzzStrategy::LengthFuzz { min_length, max_length } => {
                self.length_fuzz(data, *min_length, *max_length)
            }
            FuzzStrategy::Repeat { min_repeats, max_repeats } => {
                self.repeat_fuzz(data, *min_repeats, *max_repeats)
            }
            FuzzStrategy::Dictionary { patterns } => {
                self.dictionary_fuzz(data, patterns)
            }
            FuzzStrategy::ProtocolAware { field_offsets } => {
                self.protocol_aware_fuzz(data, field_offsets)
            }
        }
    }

    /// Random byte mutations
    fn random_mutation(&mut self, data: &[u8], rate: f64, max: usize) -> Vec<u8> {
        let mut result = data.to_vec();
        let mut mutations = 0;

        for byte in result.iter_mut() {
            if mutations >= max {
                break;
            }
            if self.seed.next_float() < rate {
                *byte = self.seed.next() as u8;
                mutations += 1;
            }
        }

        result
    }

    /// Bit flipping
    fn bit_flip(&mut self, data: &[u8], bits: usize) -> Vec<u8> {
        let mut result = data.to_vec();
        if result.is_empty() {
            return result;
        }

        for _ in 0..bits {
            let byte_idx = self.seed.next_range(result.len() as u64) as usize;
            let bit_idx = self.seed.next_range(8) as u8;
            result[byte_idx] ^= 1 << bit_idx;
        }

        result
    }

    /// Boundary value fuzzing
    fn boundary_value_fuzz(&mut self, data: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        if result.is_empty() {
            return result;
        }

        // Replace random bytes with boundary values
        let num_replacements = std::cmp::min(3, result.len());
        for _ in 0..num_replacements {
            let byte_idx = self.seed.next_range(result.len() as u64) as usize;
            let value_idx = self.seed.next_range(self.boundary_values.len() as u64) as usize;
            result[byte_idx] = self.boundary_values[value_idx];
        }

        result
    }

    /// Length fuzzing
    fn length_fuzz(&mut self, data: &[u8], min: usize, max: usize) -> Vec<u8> {
        let target_len = min + self.seed.next_range((max - min + 1) as u64) as usize;
        
        if target_len < data.len() {
            // Truncate
            data[..target_len].to_vec()
        } else if target_len > data.len() {
            // Extend with random/pattern bytes
            let mut result = data.to_vec();
            while result.len() < target_len {
                result.push(self.seed.next() as u8);
            }
            result
        } else {
            data.to_vec()
        }
    }

    /// Repeat sequence fuzzing
    fn repeat_fuzz(&mut self, data: &[u8], min: usize, max: usize) -> Vec<u8> {
        if data.is_empty() {
            return data.to_vec();
        }

        let repeats = min + self.seed.next_range((max - min + 1) as u64) as usize;
        
        // Repeat entire packet or a portion
        if self.seed.next_float() > 0.5 {
            // Repeat entire packet
            data.repeat(repeats)
        } else {
            // Repeat a portion
            let start = self.seed.next_range(data.len() as u64) as usize;
            let end = start + self.seed.next_range((data.len() - start) as u64 + 1) as usize;
            let portion = &data[start..end];
            
            let mut result = data[..start].to_vec();
            for _ in 0..repeats {
                result.extend_from_slice(portion);
            }
            result.extend_from_slice(&data[end..]);
            result
        }
    }

    /// Dictionary-based fuzzing
    fn dictionary_fuzz(&mut self, data: &[u8], patterns: &[Vec<u8>]) -> Vec<u8> {
        if patterns.is_empty() || data.is_empty() {
            return data.to_vec();
        }

        let mut result = data.to_vec();
        let pattern_idx = self.seed.next_range(patterns.len() as u64) as usize;
        let pattern = &patterns[pattern_idx];

        // Insert pattern at random position
        let pos = self.seed.next_range((result.len() + 1) as u64) as usize;
        result.splice(pos..pos, pattern.iter().cloned());

        result
    }

    /// Protocol-aware fuzzing
    fn protocol_aware_fuzz(&mut self, data: &[u8], fields: &[(usize, usize)]) -> Vec<u8> {
        let mut result = data.to_vec();
        if fields.is_empty() {
            return result;
        }

        // Pick a random field to fuzz
        let field_idx = self.seed.next_range(fields.len() as u64) as usize;
        let (offset, length) = fields[field_idx];

        if offset + length <= result.len() {
            // Fuzz the field
            for i in 0..length {
                if self.seed.next_float() < 0.5 {
                    result[offset + i] = self.seed.next() as u8;
                }
            }
        }

        result
    }

    /// Get timing delay for next packet
    pub fn get_timing_delay(&mut self) -> Duration {
        if !self.timing.enabled {
            return Duration::ZERO;
        }

        let base = self.timing.min_delay_ms + 
            self.seed.next_range(self.timing.max_delay_ms - self.timing.min_delay_ms + 1);
        
        // Add jitter
        let jitter_range = (base as f64 * self.timing.jitter_percent as f64 / 100.0) as u64;
        let jitter = if jitter_range > 0 {
            self.seed.next_range(jitter_range * 2) as i64 - jitter_range as i64
        } else {
            0
        };

        let final_ms = (base as i64 + jitter).max(0) as u64;
        Duration::from_millis(final_ms)
    }

    /// Record a test result
    pub fn record_result(&mut self, result: FuzzResult) {
        self.results.push(result);
        self.current_iteration += 1;
    }

    /// Get interesting results (errors, crashes, unexpected behavior)
    pub fn interesting_results(&self) -> Vec<&FuzzResult> {
        self.results.iter()
            .filter(|r| r.interesting || r.caused_error)
            .collect()
    }

    /// Generate summary report
    pub fn summary(&self) -> FuzzSummary {
        FuzzSummary {
            total_iterations: self.current_iteration,
            errors_found: self.results.iter().filter(|r| r.caused_error).count(),
            interesting_findings: self.results.iter().filter(|r| r.interesting).count(),
            avg_response_time_ms: self.results.iter()
                .filter_map(|r| r.response_time_ms)
                .sum::<u64>() as f64 / self.results.len().max(1) as f64,
            strategies_used: self.strategies.len(),
        }
    }
}

/// Fuzzing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzSummary {
    pub total_iterations: usize,
    pub errors_found: usize,
    pub interesting_findings: usize,
    pub avg_response_time_ms: f64,
    pub strategies_used: usize,
}

/// Robustness test suite
#[derive(Debug)]
pub struct RobustnessTest {
    pub name: String,
    pub fuzzer: PacketFuzzer,
    pub base_packet: Vec<u8>,
    pub expected_responses: Vec<Vec<u8>>,
    pub timeout_ms: u64,
}

impl RobustnessTest {
    pub fn new(name: &str, base_packet: Vec<u8>) -> Self {
        Self {
            name: name.to_string(),
            fuzzer: PacketFuzzer::new(),
            base_packet,
            expected_responses: Vec::new(),
            timeout_ms: 1000,
        }
    }

    /// Add expected valid response
    pub fn expect_response(&mut self, response: Vec<u8>) {
        self.expected_responses.push(response);
    }

    /// Check if response is expected
    pub fn is_expected_response(&self, response: &[u8]) -> bool {
        self.expected_responses.iter().any(|r| r == response)
    }
}

/// Common fuzz patterns for protocols
pub mod patterns {
    pub fn overflow_strings() -> Vec<Vec<u8>> {
        vec![
            vec![0x41; 256],    // 256 'A's
            vec![0x41; 1024],   // 1KB
            vec![0x41; 4096],   // 4KB
            vec![0x00; 256],    // Null bytes
            vec![0xFF; 256],    // 0xFF bytes
        ]
    }

    pub fn format_strings() -> Vec<Vec<u8>> {
        vec![
            b"%s%s%s%s%s".to_vec(),
            b"%n%n%n%n".to_vec(),
            b"%x%x%x%x".to_vec(),
            b"${7*7}".to_vec(),
            b"{{7*7}}".to_vec(),
        ]
    }

    pub fn sql_injection() -> Vec<Vec<u8>> {
        vec![
            b"' OR '1'='1".to_vec(),
            b"'; DROP TABLE--".to_vec(),
            b"1; SELECT * FROM".to_vec(),
        ]
    }

    pub fn modbus_fuzz() -> Vec<Vec<u8>> {
        vec![
            vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF], // Invalid unit ID
            vec![0x01, 0xFF, 0x00, 0x00, 0x00, 0x00], // Invalid function code
            vec![0x01, 0x03, 0xFF, 0xFF, 0xFF, 0xFF], // Max registers
        ]
    }

    pub fn slip_fuzz() -> Vec<Vec<u8>> {
        vec![
            vec![0xC0],                          // Empty SLIP frame
            vec![0xC0, 0xC0, 0xC0],              // Multiple END markers
            vec![0xC0, 0xDB, 0xDC, 0xC0],        // Escaped END
            vec![0xC0, 0xDB, 0xDD, 0xC0],        // Escaped ESC
            vec![0xC0, 0xDB, 0x00, 0xC0],        // Invalid escape
        ]
    }
}

