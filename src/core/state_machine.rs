//! Session State Machine
//!
//! Provides a formal state machine for session lifecycle management.
//! Enables proper error recovery, reconnection policies, and UI state binding.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionState {
    /// Initial state, not connected
    Idle,
    /// Attempting to establish connection
    Connecting,
    /// Connection established, authenticating
    Authenticating,
    /// Fully connected and operational
    Active,
    /// Connected but experiencing issues (high latency, packet loss)
    Degraded,
    /// Connection lost, attempting to reconnect
    Reconnecting,
    /// Connection intentionally paused
    Suspended,
    /// Disconnecting gracefully
    Disconnecting,
    /// Connection terminated (error or user action)
    Disconnected,
    /// Fatal error, cannot recover
    Error,
}

impl SessionState {
    /// Check if state is a connected state
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Active | Self::Degraded | Self::Authenticating)
    }

    /// Check if state is a transitional state
    pub fn is_transitional(&self) -> bool {
        matches!(self, Self::Connecting | Self::Reconnecting | Self::Disconnecting | Self::Authenticating)
    }

    /// Check if state is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Disconnected | Self::Error)
    }

    /// Check if state allows sending data
    pub fn can_send(&self) -> bool {
        matches!(self, Self::Active | Self::Degraded)
    }

    /// Check if state allows receiving data
    pub fn can_receive(&self) -> bool {
        matches!(self, Self::Active | Self::Degraded | Self::Authenticating)
    }
}

/// Disconnect reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    /// User initiated disconnect
    User,
    /// Connection timeout
    Timeout,
    /// Remote end closed connection
    RemoteClosed,
    /// Network error
    NetworkError(String),
    /// Authentication failed
    AuthenticationFailed(String),
    /// Protocol error
    ProtocolError(String),
    /// Application shutdown
    Shutdown,
    /// Resource exhausted (e.g., too many connections)
    ResourceExhausted,
    /// Unknown reason
    Unknown,
}

/// State transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Previous state
    pub from: SessionState,
    /// New state
    pub to: SessionState,
    /// Timestamp
    pub timestamp: DateTime<Local>,
    /// Reason for transition
    pub reason: Option<String>,
    /// Associated error if any
    pub error: Option<String>,
}

/// Error recovery policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPolicy {
    /// Maximum reconnection attempts
    pub max_attempts: u32,
    /// Initial delay between attempts
    pub initial_delay: Duration,
    /// Maximum delay between attempts
    pub max_delay: Duration,
    /// Delay multiplier (exponential backoff)
    pub backoff_multiplier: f32,
    /// Whether to attempt recovery at all
    pub enabled: bool,
    /// Timeout for connection attempts
    pub connection_timeout: Duration,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            enabled: true,
            connection_timeout: Duration::from_secs(10),
        }
    }
}

impl RecoveryPolicy {
    /// Calculate delay for a given attempt number
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }
        
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let delay_secs = self.initial_delay.as_secs_f32() * multiplier;
        let capped = delay_secs.min(self.max_delay.as_secs_f32());
        
        Duration::from_secs_f32(capped)
    }

    /// Check if should attempt recovery
    pub fn should_attempt(&self, current_attempt: u32) -> bool {
        self.enabled && current_attempt < self.max_attempts
    }
}

/// Session state machine
pub struct SessionStateMachine {
    /// Current state
    state: SessionState,
    /// Previous state
    previous_state: Option<SessionState>,
    /// State history
    history: Vec<StateTransition>,
    /// Maximum history size
    max_history: usize,
    /// Recovery policy
    recovery_policy: RecoveryPolicy,
    /// Current reconnection attempt
    reconnect_attempt: u32,
    /// Last state change time
    last_transition: Option<Instant>,
    /// Total time in each state (for stats)
    state_durations: std::collections::HashMap<SessionState, Duration>,
    /// Disconnect reason
    disconnect_reason: Option<DisconnectReason>,
    /// State change callback
    on_state_change: Option<Box<dyn Fn(SessionState, SessionState) + Send + Sync>>,
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStateMachine {
    /// Create a new state machine
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
            previous_state: None,
            history: Vec::new(),
            max_history: 100,
            recovery_policy: RecoveryPolicy::default(),
            reconnect_attempt: 0,
            last_transition: None,
            state_durations: std::collections::HashMap::new(),
            disconnect_reason: None,
            on_state_change: None,
        }
    }

    /// Create with custom recovery policy
    pub fn with_policy(policy: RecoveryPolicy) -> Self {
        Self {
            recovery_policy: policy,
            ..Self::new()
        }
    }

    /// Get current state
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Get previous state
    pub fn previous_state(&self) -> Option<SessionState> {
        self.previous_state
    }

    /// Get disconnect reason
    pub fn disconnect_reason(&self) -> Option<&DisconnectReason> {
        self.disconnect_reason.as_ref()
    }

    /// Get current reconnection attempt
    pub fn reconnect_attempt(&self) -> u32 {
        self.reconnect_attempt
    }

    /// Get state history
    pub fn history(&self) -> &[StateTransition] {
        &self.history
    }

    /// Get time in current state
    pub fn time_in_state(&self) -> Option<Duration> {
        self.last_transition.map(|t| t.elapsed())
    }

    /// Set state change callback
    pub fn on_state_change<F>(&mut self, callback: F)
    where
        F: Fn(SessionState, SessionState) + Send + Sync + 'static,
    {
        self.on_state_change = Some(Box::new(callback));
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: SessionState, reason: Option<&str>) -> Result<(), String> {
        self.transition_with_error(new_state, reason, None)
    }

    /// Transition with error information
    pub fn transition_with_error(
        &mut self,
        new_state: SessionState,
        reason: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), String> {
        // Validate transition
        if !self.is_valid_transition(new_state) {
            return Err(format!(
                "Invalid transition from {:?} to {:?}",
                self.state, new_state
            ));
        }

        // Update duration tracking
        if let Some(last) = self.last_transition {
            let duration = last.elapsed();
            *self.state_durations.entry(self.state).or_default() += duration;
        }

        // Record transition
        let transition = StateTransition {
            from: self.state,
            to: new_state,
            timestamp: Local::now(),
            reason: reason.map(String::from),
            error: error.map(String::from),
        };

        self.history.push(transition);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Update state
        self.previous_state = Some(self.state);
        let old_state = self.state;
        self.state = new_state;
        self.last_transition = Some(Instant::now());

        // Handle reconnection counting
        match new_state {
            SessionState::Reconnecting => {
                self.reconnect_attempt += 1;
            }
            SessionState::Active => {
                self.reconnect_attempt = 0;
            }
            _ => {}
        }

        // Call callback
        if let Some(ref callback) = self.on_state_change {
            callback(old_state, new_state);
        }

        Ok(())
    }

    /// Check if transition is valid
    fn is_valid_transition(&self, new_state: SessionState) -> bool {
        use SessionState::*;
        
        match (self.state, new_state) {
            // From Idle
            (Idle, Connecting) => true,
            (Idle, Error) => true,
            
            // From Connecting
            (Connecting, Authenticating) => true,
            (Connecting, Active) => true, // For transports without auth
            (Connecting, Disconnected) => true,
            (Connecting, Error) => true,
            
            // From Authenticating
            (Authenticating, Active) => true,
            (Authenticating, Disconnected) => true,
            (Authenticating, Error) => true,
            
            // From Active
            (Active, Degraded) => true,
            (Active, Disconnecting) => true,
            (Active, Reconnecting) => true,
            (Active, Disconnected) => true,
            (Active, Error) => true,
            (Active, Suspended) => true,
            
            // From Degraded
            (Degraded, Active) => true,
            (Degraded, Disconnecting) => true,
            (Degraded, Reconnecting) => true,
            (Degraded, Disconnected) => true,
            (Degraded, Error) => true,
            
            // From Reconnecting
            (Reconnecting, Connecting) => true,
            (Reconnecting, Active) => true,
            (Reconnecting, Disconnected) => true,
            (Reconnecting, Error) => true,
            
            // From Suspended
            (Suspended, Active) => true,
            (Suspended, Disconnecting) => true,
            (Suspended, Disconnected) => true,
            
            // From Disconnecting
            (Disconnecting, Disconnected) => true,
            (Disconnecting, Error) => true,
            
            // From Disconnected
            (Disconnected, Connecting) => true,
            (Disconnected, Idle) => true,
            
            // From Error
            (Error, Idle) => true,
            (Error, Connecting) => true,
            
            // Same state (no-op)
            (a, b) if a == b => true,
            
            _ => false,
        }
    }

    /// Set disconnect reason
    pub fn set_disconnect_reason(&mut self, reason: DisconnectReason) {
        self.disconnect_reason = Some(reason);
    }

    /// Check if should attempt reconnection
    pub fn should_reconnect(&self) -> bool {
        self.recovery_policy.should_attempt(self.reconnect_attempt)
    }

    /// Get delay before next reconnection attempt
    pub fn reconnect_delay(&self) -> Duration {
        self.recovery_policy.delay_for_attempt(self.reconnect_attempt)
    }

    /// Get recovery policy
    pub fn recovery_policy(&self) -> &RecoveryPolicy {
        &self.recovery_policy
    }

    /// Set recovery policy
    pub fn set_recovery_policy(&mut self, policy: RecoveryPolicy) {
        self.recovery_policy = policy;
    }

    /// Get total time spent in a state
    pub fn total_time_in_state(&self, state: SessionState) -> Duration {
        self.state_durations.get(&state).copied().unwrap_or_default()
    }

    /// Reset the state machine
    pub fn reset(&mut self) {
        self.state = SessionState::Idle;
        self.previous_state = None;
        self.reconnect_attempt = 0;
        self.last_transition = None;
        self.disconnect_reason = None;
    }

    /// Get stats summary
    pub fn stats(&self) -> SessionStats {
        SessionStats {
            current_state: self.state,
            total_transitions: self.history.len(),
            reconnect_attempts: self.reconnect_attempt,
            time_in_current_state: self.time_in_state(),
            disconnect_reason: self.disconnect_reason.clone(),
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub current_state: SessionState,
    pub total_transitions: usize,
    pub reconnect_attempts: u32,
    pub time_in_current_state: Option<Duration>,
    pub disconnect_reason: Option<DisconnectReason>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_transitions() {
        let mut sm = SessionStateMachine::new();
        
        assert_eq!(sm.state(), SessionState::Idle);
        
        sm.transition(SessionState::Connecting, Some("User initiated")).unwrap();
        assert_eq!(sm.state(), SessionState::Connecting);
        
        sm.transition(SessionState::Active, Some("Connected")).unwrap();
        assert_eq!(sm.state(), SessionState::Active);
        assert!(sm.state().can_send());
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = SessionStateMachine::new();
        
        // Can't go directly from Idle to Active
        let result = sm.transition(SessionState::Active, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_reconnection_counting() {
        let mut sm = SessionStateMachine::new();
        
        sm.transition(SessionState::Connecting, None).unwrap();
        sm.transition(SessionState::Active, None).unwrap();
        
        assert_eq!(sm.reconnect_attempt(), 0);
        
        sm.transition(SessionState::Reconnecting, None).unwrap();
        assert_eq!(sm.reconnect_attempt(), 1);
        
        sm.transition(SessionState::Connecting, None).unwrap();
        sm.transition(SessionState::Active, None).unwrap();
        assert_eq!(sm.reconnect_attempt(), 0);
    }

    #[test]
    fn test_recovery_policy() {
        let policy = RecoveryPolicy::default();
        
        assert!(policy.should_attempt(0));
        assert!(policy.should_attempt(4));
        assert!(!policy.should_attempt(5));
        
        // Check exponential backoff
        assert_eq!(policy.delay_for_attempt(0), Duration::from_secs(1));
        assert_eq!(policy.delay_for_attempt(1), Duration::from_secs(2));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(4));
    }
}






