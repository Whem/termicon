//! Advanced Trigger Features
//!
//! Provides multi-pattern groups, conditional triggers, and trigger chains.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Condition operator for conditional triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    And,
    Or,
    Not,
    Xor,
}

/// Condition type for triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Always true
    Always,
    /// Previous trigger was matched
    AfterTrigger(String),
    /// Variable equals value
    VariableEquals { name: String, value: String },
    /// Variable matches regex
    VariableMatches { name: String, pattern: String },
    /// Counter reached value
    CounterReached { name: String, value: u32 },
    /// Time elapsed since event
    TimeSince { event: String, duration_ms: u64 },
    /// Connection state
    ConnectionState { connected: bool },
    /// Compound condition
    Compound {
        operator: ConditionOperator,
        conditions: Vec<TriggerCondition>,
    },
}

impl TriggerCondition {
    /// Evaluate condition against context
    pub fn evaluate(&self, context: &TriggerContext) -> bool {
        match self {
            Self::Always => true,
            Self::AfterTrigger(name) => context.last_triggered.as_ref() == Some(name),
            Self::VariableEquals { name, value } => {
                context.variables.get(name).map(|v| v == value).unwrap_or(false)
            }
            Self::VariableMatches { name, pattern } => {
                if let Some(value) = context.variables.get(name) {
                    Regex::new(pattern).map(|re| re.is_match(value)).unwrap_or(false)
                } else {
                    false
                }
            }
            Self::CounterReached { name, value } => {
                context.counters.get(name).map(|&v| v >= *value).unwrap_or(false)
            }
            Self::TimeSince { event, duration_ms } => {
                if let Some(time) = context.events.get(event) {
                    time.elapsed() >= Duration::from_millis(*duration_ms)
                } else {
                    false
                }
            }
            Self::ConnectionState { connected } => context.is_connected == *connected,
            Self::Compound { operator, conditions } => {
                match operator {
                    ConditionOperator::And => conditions.iter().all(|c| c.evaluate(context)),
                    ConditionOperator::Or => conditions.iter().any(|c| c.evaluate(context)),
                    ConditionOperator::Not => {
                        !conditions.first().map(|c| c.evaluate(context)).unwrap_or(true)
                    }
                    ConditionOperator::Xor => {
                        conditions.iter().filter(|c| c.evaluate(context)).count() == 1
                    }
                }
            }
        }
    }
}

/// Pattern matching mode for multi-pattern groups
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PatternMatchMode {
    /// All patterns must match
    All,
    /// Any pattern can match
    Any,
    /// Patterns must match in sequence
    Sequence,
    /// First N patterns must match
    AtLeast(usize),
}

/// Multi-pattern trigger group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternGroup {
    /// Group name
    pub name: String,
    /// Patterns in this group
    pub patterns: Vec<PatternDefinition>,
    /// Match mode
    pub mode: PatternMatchMode,
    /// Timeout for sequence mode (ms)
    pub sequence_timeout_ms: u64,
    /// Actions to execute on match
    pub actions: Vec<TriggerAction>,
    /// Condition for execution
    pub condition: TriggerCondition,
    /// Enabled state
    pub enabled: bool,
}

/// Pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDefinition {
    /// Pattern name/label
    pub name: String,
    /// Pattern type
    pub pattern_type: PatternType,
    /// Pattern data
    pub pattern: String,
    /// Case sensitive (for text patterns)
    pub case_sensitive: bool,
}

/// Pattern type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Exact,
    Contains,
    Regex,
    Hex,
    StartsWith,
    EndsWith,
}

/// Trigger action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerAction {
    /// Send data
    Send(String),
    /// Send hex data
    SendHex(Vec<u8>),
    /// Log message
    Log(String),
    /// Set variable
    SetVariable { name: String, value: String },
    /// Increment counter
    IncrementCounter(String),
    /// Reset counter
    ResetCounter(String),
    /// Record event time
    RecordEvent(String),
    /// Execute trigger chain
    ExecuteChain(String),
    /// Delay (ms)
    Delay(u64),
    /// Notify user
    Notify { title: String, message: String },
    /// Play sound
    PlaySound(String),
    /// Execute external command
    ExecuteCommand(String),
    /// Disconnect
    Disconnect,
    /// Reconnect
    Reconnect,
}

/// Trigger chain - sequence of triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerChain {
    /// Chain name
    pub name: String,
    /// Chain description
    pub description: String,
    /// Ordered list of trigger/action pairs
    pub steps: Vec<ChainStep>,
    /// Loop the chain
    pub loop_chain: bool,
    /// Enabled state
    pub enabled: bool,
}

/// Step in a trigger chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    /// Step name
    pub name: String,
    /// Pattern to match (optional - if None, executes immediately)
    pub pattern: Option<PatternDefinition>,
    /// Timeout for this step (ms, 0 = no timeout)
    pub timeout_ms: u64,
    /// Actions to execute
    pub actions: Vec<TriggerAction>,
    /// Action on timeout
    pub on_timeout: ChainTimeoutAction,
}

/// Action when chain step times out
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainTimeoutAction {
    /// Continue to next step
    Continue,
    /// Abort chain
    Abort,
    /// Retry step
    Retry { max_attempts: u32 },
    /// Execute actions and continue
    ExecuteAndContinue(Vec<TriggerAction>),
}

/// Trigger execution context
#[derive(Debug, Default)]
pub struct TriggerContext {
    /// Variables
    pub variables: HashMap<String, String>,
    /// Counters
    pub counters: HashMap<String, u32>,
    /// Event timestamps
    pub events: HashMap<String, Instant>,
    /// Last triggered pattern/group name
    pub last_triggered: Option<String>,
    /// Connection state
    pub is_connected: bool,
    /// Sequence match state (for multi-pattern sequence mode)
    pub sequence_state: HashMap<String, SequenceState>,
}

/// State for sequence pattern matching
#[derive(Debug, Clone)]
pub struct SequenceState {
    /// Current position in sequence
    pub position: usize,
    /// Time of first match
    pub started_at: Instant,
}

impl TriggerContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set a variable
    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }
    
    /// Get a variable
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }
    
    /// Increment a counter
    pub fn increment_counter(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }
    
    /// Reset a counter
    pub fn reset_counter(&mut self, name: &str) {
        self.counters.insert(name.to_string(), 0);
    }
    
    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u32 {
        *self.counters.get(name).unwrap_or(&0)
    }
    
    /// Record an event
    pub fn record_event(&mut self, name: &str) {
        self.events.insert(name.to_string(), Instant::now());
    }
}

/// Advanced trigger manager
#[derive(Debug)]
pub struct AdvancedTriggerManager {
    /// Pattern groups
    pub groups: Vec<PatternGroup>,
    /// Trigger chains
    pub chains: Vec<TriggerChain>,
    /// Active chain executions
    active_chains: HashMap<String, ChainExecution>,
    /// Context
    pub context: TriggerContext,
}

/// Active chain execution state
#[derive(Debug)]
struct ChainExecution {
    chain_name: String,
    current_step: usize,
    step_started: Instant,
    retry_count: u32,
}

impl Default for AdvancedTriggerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedTriggerManager {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            chains: Vec::new(),
            active_chains: HashMap::new(),
            context: TriggerContext::new(),
        }
    }
    
    /// Add a pattern group
    pub fn add_group(&mut self, group: PatternGroup) {
        self.groups.push(group);
    }
    
    /// Add a trigger chain
    pub fn add_chain(&mut self, chain: TriggerChain) {
        self.chains.push(chain);
    }
    
    /// Process incoming data
    pub fn process(&mut self, data: &[u8]) -> Vec<TriggerAction> {
        let mut actions = Vec::new();
        let data_str = String::from_utf8_lossy(data);
        
        // Collect matching groups first to avoid borrow conflicts
        let mut matched_groups: Vec<(String, Vec<TriggerAction>)> = Vec::new();
        
        for group in &self.groups {
            if !group.enabled {
                continue;
            }
            
            if !group.condition.evaluate(&self.context) {
                continue;
            }
            
            let matches = Self::check_pattern_group_static(
                group,
                &data_str,
                data,
                self.context.sequence_state.get(&group.name),
            );
            
            if matches {
                matched_groups.push((group.name.clone(), group.actions.clone()));
            }
        }
        
        // Now update state for matched groups
        for (name, group_actions) in matched_groups {
            self.context.last_triggered = Some(name.clone());
            
            // Update sequence state if needed
            if let Some(state) = self.context.sequence_state.get_mut(&name) {
                state.position = 0;
            }
            
            actions.extend(group_actions);
        }
        
        // Check active chains
        let chain_actions = self.process_chains(&data_str, data);
        actions.extend(chain_actions);
        
        actions
    }
    
    fn check_pattern_group_static(
        group: &PatternGroup,
        text: &str,
        raw: &[u8],
        seq_state: Option<&SequenceState>,
    ) -> bool {
        match group.mode {
            PatternMatchMode::All => {
                group.patterns.iter().all(|p| Self::match_pattern_static(p, text, raw))
            }
            PatternMatchMode::Any => {
                group.patterns.iter().any(|p| Self::match_pattern_static(p, text, raw))
            }
            PatternMatchMode::AtLeast(n) => {
                group.patterns.iter().filter(|p| Self::match_pattern_static(p, text, raw)).count() >= n
            }
            PatternMatchMode::Sequence => {
                Self::check_sequence_static(group, text, raw, seq_state)
            }
        }
    }
    
    fn check_sequence_static(
        group: &PatternGroup,
        text: &str,
        raw: &[u8],
        seq_state: Option<&SequenceState>,
    ) -> bool {
        let position = seq_state.map(|s| {
            // Check timeout
            if s.position > 0 && s.started_at.elapsed().as_millis() as u64 > group.sequence_timeout_ms {
                0
            } else {
                s.position
            }
        }).unwrap_or(0);
        
        // Check current pattern
        if position < group.patterns.len() {
            if Self::match_pattern_static(&group.patterns[position], text, raw) {
                return position + 1 >= group.patterns.len();
            }
        }
        
        false
    }
    
    /// Update sequence state after processing
    pub fn update_sequence_state(&mut self, group_name: &str, group: &PatternGroup, text: &str, raw: &[u8]) {
        let state = self.context.sequence_state.entry(group_name.to_string())
            .or_insert(SequenceState {
                position: 0,
                started_at: Instant::now(),
            });
        
        // Check timeout
        if state.position > 0 && state.started_at.elapsed().as_millis() as u64 > group.sequence_timeout_ms {
            state.position = 0;
            state.started_at = Instant::now();
        }
        
        // Check current pattern
        if state.position < group.patterns.len() {
            if Self::match_pattern_static(&group.patterns[state.position], text, raw) {
                if state.position == 0 {
                    state.started_at = Instant::now();
                }
                state.position += 1;
                
                if state.position >= group.patterns.len() {
                    state.position = 0;
                }
            }
        }
    }
    
    fn match_pattern_static(pattern: &PatternDefinition, text: &str, raw: &[u8]) -> bool {
        let text = if pattern.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };
        
        let pattern_text = if pattern.case_sensitive {
            pattern.pattern.clone()
        } else {
            pattern.pattern.to_lowercase()
        };
        
        match pattern.pattern_type {
            PatternType::Exact => text == pattern_text,
            PatternType::Contains => text.contains(&pattern_text),
            PatternType::StartsWith => text.starts_with(&pattern_text),
            PatternType::EndsWith => text.ends_with(&pattern_text),
            PatternType::Regex => {
                Regex::new(&pattern.pattern).map(|re| re.is_match(&text)).unwrap_or(false)
            }
            PatternType::Hex => {
                if let Some(hex_pattern) = Self::parse_hex_pattern(&pattern.pattern) {
                    raw.windows(hex_pattern.len()).any(|w| w == hex_pattern.as_slice())
                } else {
                    false
                }
            }
        }
    }
    
    /// Public method for pattern matching (for tests)
    pub fn match_pattern(&self, pattern: &PatternDefinition, text: &str, raw: &[u8]) -> bool {
        Self::match_pattern_static(pattern, text, raw)
    }
    
    fn parse_hex_pattern(s: &str) -> Option<Vec<u8>> {
        let s = s.replace(' ', "");
        if s.len() % 2 != 0 {
            return None;
        }
        
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i+2], 16).ok())
            .collect()
    }
    
    fn process_chains(&mut self, text: &str, raw: &[u8]) -> Vec<TriggerAction> {
        let mut actions = Vec::new();
        let mut completed = Vec::new();
        
        // Collect chain names and their steps first
        let chain_data: Vec<(String, Option<PatternDefinition>, Vec<TriggerAction>, u64, ChainTimeoutAction, bool, usize)> = 
            self.active_chains.iter().filter_map(|(name, exec)| {
                self.chains.iter().find(|c| c.name == *name).and_then(|chain| {
                    if exec.current_step < chain.steps.len() {
                        let step = &chain.steps[exec.current_step];
                        Some((
                            name.clone(),
                            step.pattern.clone(),
                            step.actions.clone(),
                            step.timeout_ms,
                            step.on_timeout.clone(),
                            chain.loop_chain,
                            chain.steps.len(),
                        ))
                    } else {
                        None
                    }
                })
            }).collect();
        
        for (name, pattern, step_actions, timeout_ms, on_timeout, loop_chain, total_steps) in chain_data {
            let exec = self.active_chains.get_mut(&name).unwrap();
            
            // Check pattern if present
            let matched = if let Some(ref pattern) = pattern {
                Self::match_pattern_static(pattern, text, raw)
            } else {
                true
            };
            
            if matched {
                actions.extend(step_actions);
                exec.current_step += 1;
                exec.step_started = Instant::now();
                exec.retry_count = 0;
                
                if exec.current_step >= total_steps {
                    if loop_chain {
                        exec.current_step = 0;
                    } else {
                        completed.push(name.clone());
                    }
                }
            } else if timeout_ms > 0 && exec.step_started.elapsed().as_millis() as u64 > timeout_ms {
                // Handle timeout
                match on_timeout {
                    ChainTimeoutAction::Continue => {
                        exec.current_step += 1;
                    }
                    ChainTimeoutAction::Abort => {
                        completed.push(name.clone());
                    }
                    ChainTimeoutAction::Retry { max_attempts } => {
                        exec.retry_count += 1;
                        if exec.retry_count >= max_attempts {
                            completed.push(name.clone());
                        }
                        exec.step_started = Instant::now();
                    }
                    ChainTimeoutAction::ExecuteAndContinue(timeout_actions) => {
                        actions.extend(timeout_actions);
                        exec.current_step += 1;
                    }
                }
            }
        }
        
        for name in completed {
            self.active_chains.remove(&name);
        }
        
        actions
    }
    
    /// Start a trigger chain
    pub fn start_chain(&mut self, name: &str) -> bool {
        if self.chains.iter().any(|c| c.name == name && c.enabled) {
            self.active_chains.insert(name.to_string(), ChainExecution {
                chain_name: name.to_string(),
                current_step: 0,
                step_started: Instant::now(),
                retry_count: 0,
            });
            true
        } else {
            false
        }
    }
    
    /// Stop a trigger chain
    pub fn stop_chain(&mut self, name: &str) -> bool {
        self.active_chains.remove(name).is_some()
    }
    
    /// Check if chain is active
    pub fn is_chain_active(&self, name: &str) -> bool {
        self.active_chains.contains_key(name)
    }
    
    /// Execute an action
    pub fn execute_action(&mut self, action: &TriggerAction) -> Option<Vec<u8>> {
        match action {
            TriggerAction::Send(text) => Some(text.as_bytes().to_vec()),
            TriggerAction::SendHex(data) => Some(data.clone()),
            TriggerAction::SetVariable { name, value } => {
                self.context.set_variable(name, value);
                None
            }
            TriggerAction::IncrementCounter(name) => {
                self.context.increment_counter(name);
                None
            }
            TriggerAction::ResetCounter(name) => {
                self.context.reset_counter(name);
                None
            }
            TriggerAction::RecordEvent(name) => {
                self.context.record_event(name);
                None
            }
            TriggerAction::ExecuteChain(name) => {
                self.start_chain(name);
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_condition_evaluation() {
        let mut context = TriggerContext::new();
        context.set_variable("status", "ok");
        context.counters.insert("retries".to_string(), 3);
        
        let cond1 = TriggerCondition::VariableEquals {
            name: "status".to_string(),
            value: "ok".to_string(),
        };
        assert!(cond1.evaluate(&context));
        
        let cond2 = TriggerCondition::CounterReached {
            name: "retries".to_string(),
            value: 3,
        };
        assert!(cond2.evaluate(&context));
        
        let compound = TriggerCondition::Compound {
            operator: ConditionOperator::And,
            conditions: vec![cond1.clone(), cond2.clone()],
        };
        assert!(compound.evaluate(&context));
    }
    
    #[test]
    fn test_pattern_matching() {
        let manager = AdvancedTriggerManager::new();
        
        let pattern = PatternDefinition {
            name: "test".to_string(),
            pattern_type: PatternType::Contains,
            pattern: "hello".to_string(),
            case_sensitive: false,
        };
        
        assert!(manager.match_pattern(&pattern, "Hello World", b"Hello World"));
        assert!(!manager.match_pattern(&pattern, "Goodbye", b"Goodbye"));
    }
    
    #[test]
    fn test_pattern_group() {
        let mut manager = AdvancedTriggerManager::new();
        
        let group = PatternGroup {
            name: "test_group".to_string(),
            patterns: vec![
                PatternDefinition {
                    name: "p1".to_string(),
                    pattern_type: PatternType::Contains,
                    pattern: "OK".to_string(),
                    case_sensitive: true,
                },
            ],
            mode: PatternMatchMode::All,
            sequence_timeout_ms: 5000,
            actions: vec![TriggerAction::Log("Matched!".to_string())],
            condition: TriggerCondition::Always,
            enabled: true,
        };
        
        manager.add_group(group);
        
        let actions = manager.process(b"Response: OK");
        assert_eq!(actions.len(), 1);
    }
}


