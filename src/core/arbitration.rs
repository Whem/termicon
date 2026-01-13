//! Resource Arbitration / Fairness
//! 
//! Manages:
//! - Session priorities
//! - Bandwidth/rate limiting
//! - Fairness policies
//! - Resource allocation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Session priority level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Background - lowest priority, can be preempted
    Background = 0,
    /// Normal - default priority
    Normal = 1,
    /// High - preferential treatment
    High = 2,
    /// RealTime - guaranteed bandwidth/latency
    RealTime = 3,
    /// Critical - highest priority, emergency
    Critical = 4,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Rate limiter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiter {
    /// Maximum bytes per second
    pub bytes_per_second: u64,
    /// Maximum packets per second
    pub packets_per_second: u64,
    /// Burst allowance (bytes)
    pub burst_size: u64,
    /// Current tokens available
    #[serde(skip)]
    tokens: u64,
    /// Last refill time
    #[serde(skip)]
    last_refill: Option<Instant>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            bytes_per_second: 1_000_000, // 1 MB/s
            packets_per_second: 1000,
            burst_size: 65536,
            tokens: 65536,
            last_refill: None,
        }
    }
}

impl RateLimiter {
    pub fn new(bytes_per_second: u64) -> Self {
        Self {
            bytes_per_second,
            burst_size: bytes_per_second / 10, // 100ms worth
            tokens: bytes_per_second / 10,
            ..Default::default()
        }
    }

    /// Refill tokens based on time elapsed
    pub fn refill(&mut self) {
        let now = Instant::now();
        
        if let Some(last) = self.last_refill {
            let elapsed_ms = last.elapsed().as_millis() as u64;
            let new_tokens = (self.bytes_per_second * elapsed_ms) / 1000;
            self.tokens = (self.tokens + new_tokens).min(self.burst_size);
        } else {
            self.tokens = self.burst_size;
        }
        
        self.last_refill = Some(now);
    }

    /// Try to consume tokens, returns true if allowed
    pub fn try_consume(&mut self, bytes: u64) -> bool {
        self.refill();
        
        if self.tokens >= bytes {
            self.tokens -= bytes;
            true
        } else {
            false
        }
    }

    /// Wait duration until enough tokens available
    pub fn wait_time(&mut self, bytes: u64) -> Duration {
        self.refill();
        
        if self.tokens >= bytes {
            Duration::ZERO
        } else {
            let needed = bytes - self.tokens;
            let ms = (needed * 1000) / self.bytes_per_second;
            Duration::from_millis(ms.max(1))
        }
    }
}

/// Fairness policy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FairnessPolicy {
    /// First come, first served
    FCFS,
    /// Round robin between sessions
    RoundRobin,
    /// Priority-based (higher priority first)
    Priority,
    /// Weighted fair queuing
    WeightedFair { weights: HashMap<String, f64> },
    /// Deadline-based (earliest deadline first)
    EDF,
}

impl Default for FairnessPolicy {
    fn default() -> Self {
        FairnessPolicy::RoundRobin
    }
}

/// Resource allocation for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAllocation {
    /// Session ID
    pub session_id: String,
    /// Priority
    pub priority: Priority,
    /// Rate limiter
    pub rate_limiter: RateLimiter,
    /// Maximum CPU time percentage
    pub cpu_percent: u8,
    /// Maximum memory (bytes)
    pub max_memory: u64,
    /// Is this session active?
    pub active: bool,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Created time
    pub created: String,
}

impl SessionAllocation {
    pub fn new(session_id: &str, priority: Priority) -> Self {
        Self {
            session_id: session_id.to_string(),
            priority,
            rate_limiter: RateLimiter::default(),
            cpu_percent: 25,
            max_memory: 100 * 1024 * 1024, // 100 MB
            active: true,
            bytes_sent: 0,
            bytes_received: 0,
            created: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Resource arbiter
#[derive(Debug)]
pub struct ResourceArbiter {
    /// Session allocations
    pub sessions: HashMap<String, SessionAllocation>,
    /// Global rate limiter
    pub global_rate_limiter: RateLimiter,
    /// Fairness policy
    pub policy: FairnessPolicy,
    /// Total bandwidth available
    pub total_bandwidth: u64,
    /// Current round-robin index
    rr_index: usize,
}

impl Default for ResourceArbiter {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            global_rate_limiter: RateLimiter::new(10_000_000), // 10 MB/s
            policy: FairnessPolicy::RoundRobin,
            total_bandwidth: 10_000_000,
            rr_index: 0,
        }
    }
}

impl ResourceArbiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a session
    pub fn register_session(&mut self, session_id: &str, priority: Priority) {
        let allocation = SessionAllocation::new(session_id, priority);
        self.sessions.insert(session_id.to_string(), allocation);
        self.rebalance();
    }

    /// Unregister a session
    pub fn unregister_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
        self.rebalance();
    }

    /// Set session priority
    pub fn set_priority(&mut self, session_id: &str, priority: Priority) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.priority = priority;
        }
        self.rebalance();
    }

    /// Rebalance allocations based on policy
    pub fn rebalance(&mut self) {
        let active_count = self.sessions.values().filter(|s| s.active).count();
        if active_count == 0 {
            return;
        }

        match &self.policy {
            FairnessPolicy::FCFS | FairnessPolicy::RoundRobin => {
                // Equal share
                let share = self.total_bandwidth / active_count as u64;
                for session in self.sessions.values_mut() {
                    if session.active {
                        session.rate_limiter.bytes_per_second = share;
                    }
                }
            }
            FairnessPolicy::Priority => {
                // Priority-weighted
                let total_priority: u64 = self.sessions.values()
                    .filter(|s| s.active)
                    .map(|s| (s.priority as u64 + 1) * 10)
                    .sum();
                
                for session in self.sessions.values_mut() {
                    if session.active {
                        let weight = (session.priority as u64 + 1) * 10;
                        session.rate_limiter.bytes_per_second = 
                            (self.total_bandwidth * weight) / total_priority;
                    }
                }
            }
            FairnessPolicy::WeightedFair { weights } => {
                let total_weight: f64 = self.sessions.values()
                    .filter(|s| s.active)
                    .map(|s| weights.get(&s.session_id).unwrap_or(&1.0))
                    .sum();
                
                for session in self.sessions.values_mut() {
                    if session.active {
                        let weight = weights.get(&session.session_id).unwrap_or(&1.0);
                        session.rate_limiter.bytes_per_second = 
                            ((self.total_bandwidth as f64 * weight) / total_weight) as u64;
                    }
                }
            }
            FairnessPolicy::EDF => {
                // Would need deadline information
                // For now, treat as priority-based
            }
        }
    }

    /// Request to send data
    pub fn request_send(&mut self, session_id: &str, bytes: u64) -> SendPermission {
        // Check global limit
        if !self.global_rate_limiter.try_consume(bytes) {
            return SendPermission::Wait(self.global_rate_limiter.wait_time(bytes));
        }

        // Check session limit
        if let Some(session) = self.sessions.get_mut(session_id) {
            if !session.active {
                return SendPermission::Denied("Session inactive".to_string());
            }

            if session.rate_limiter.try_consume(bytes) {
                session.bytes_sent += bytes;
                SendPermission::Allowed
            } else {
                SendPermission::Wait(session.rate_limiter.wait_time(bytes))
            }
        } else {
            SendPermission::Denied("Session not registered".to_string())
        }
    }

    /// Get next session to serve (round-robin)
    pub fn next_session(&mut self) -> Option<&str> {
        let active_sessions: Vec<_> = self.sessions.values()
            .filter(|s| s.active)
            .map(|s| s.session_id.as_str())
            .collect();

        if active_sessions.is_empty() {
            return None;
        }

        self.rr_index = (self.rr_index + 1) % active_sessions.len();
        Some(active_sessions[self.rr_index])
    }

    /// Get statistics
    pub fn stats(&self) -> ArbiterStats {
        ArbiterStats {
            total_sessions: self.sessions.len(),
            active_sessions: self.sessions.values().filter(|s| s.active).count(),
            total_bandwidth: self.total_bandwidth,
            total_bytes_sent: self.sessions.values().map(|s| s.bytes_sent).sum(),
            total_bytes_received: self.sessions.values().map(|s| s.bytes_received).sum(),
        }
    }
}

/// Permission result
#[derive(Debug, Clone)]
pub enum SendPermission {
    /// Allowed to send immediately
    Allowed,
    /// Must wait before sending
    Wait(Duration),
    /// Denied (with reason)
    Denied(String),
}

/// Arbiter statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbiterStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub total_bandwidth: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}




