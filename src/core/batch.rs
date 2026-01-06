//! Batch Operations
//!
//! Provides multi-session commands with sequential and parallel execution,
//! error handling, and result aggregation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tokio::sync::mpsc;

/// Batch operation type
#[derive(Debug, Clone)]
pub enum BatchOperation {
    /// Send data to session
    Send { data: Vec<u8> },
    /// Send text with line ending
    SendLine { text: String },
    /// Wait for pattern
    WaitFor { pattern: String, timeout_ms: u64 },
    /// Delay
    Delay { duration_ms: u64 },
    /// Execute command
    Execute { command: String },
    /// Disconnect session
    Disconnect,
    /// Reconnect session
    Reconnect,
    /// Set session variable
    SetVariable { name: String, value: String },
    /// Log message
    Log { message: String },
}

/// Batch execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Execute on sessions one at a time
    Sequential,
    /// Execute on all sessions simultaneously
    Parallel,
    /// Execute on sessions in round-robin fashion
    RoundRobin,
}

/// Error handling strategy
#[derive(Debug, Clone)]
pub enum ErrorStrategy {
    /// Stop on first error
    StopOnError,
    /// Continue and collect errors
    ContinueOnError,
    /// Retry with limit
    Retry { max_attempts: u32, delay_ms: u64 },
    /// Skip failed session and continue
    SkipFailed,
}

/// Batch task definition
#[derive(Debug, Clone)]
pub struct BatchTask {
    /// Task name
    pub name: String,
    /// Target session IDs (empty = all sessions)
    pub sessions: Vec<String>,
    /// Operations to execute
    pub operations: Vec<BatchOperation>,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Error handling
    pub error_strategy: ErrorStrategy,
    /// Timeout for entire batch (ms, 0 = no timeout)
    pub timeout_ms: u64,
}

impl BatchTask {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sessions: Vec::new(),
            operations: Vec::new(),
            mode: ExecutionMode::Sequential,
            error_strategy: ErrorStrategy::StopOnError,
            timeout_ms: 0,
        }
    }
    
    /// Add target session
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.sessions.push(session_id.into());
        self
    }
    
    /// Add operation
    pub fn with_operation(mut self, op: BatchOperation) -> Self {
        self.operations.push(op);
        self
    }
    
    /// Set execution mode
    pub fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }
    
    /// Set error strategy
    pub fn with_error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.error_strategy = strategy;
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// Result from a single operation
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Operation index
    pub index: usize,
    /// Session ID
    pub session_id: String,
    /// Success
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Captured output (if applicable)
    pub output: Option<String>,
    /// Execution time
    pub duration: Duration,
}

/// Result from batch execution
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Task name
    pub task_name: String,
    /// Overall success
    pub success: bool,
    /// Individual results
    pub results: Vec<OperationResult>,
    /// Sessions that completed successfully
    pub successful_sessions: Vec<String>,
    /// Sessions that failed
    pub failed_sessions: Vec<String>,
    /// Total execution time
    pub total_duration: Duration,
    /// Error summary
    pub errors: Vec<String>,
}

impl BatchResult {
    pub fn new(task_name: String) -> Self {
        Self {
            task_name,
            success: true,
            results: Vec::new(),
            successful_sessions: Vec::new(),
            failed_sessions: Vec::new(),
            total_duration: Duration::ZERO,
            errors: Vec::new(),
        }
    }
    
    /// Success rate (0.0 - 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        self.results.iter().filter(|r| r.success).count() as f64 / self.results.len() as f64
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Task '{}': {} successful, {} failed, {:.1}% success rate, took {:?}",
            self.task_name,
            self.successful_sessions.len(),
            self.failed_sessions.len(),
            self.success_rate() * 100.0,
            self.total_duration
        )
    }
}

/// Batch event for progress reporting
#[derive(Debug, Clone)]
pub enum BatchEvent {
    /// Batch started
    Started { task_name: String, total_sessions: usize },
    /// Session started
    SessionStarted { session_id: String },
    /// Operation started
    OperationStarted { session_id: String, operation_index: usize },
    /// Operation completed
    OperationCompleted { result: OperationResult },
    /// Session completed
    SessionCompleted { session_id: String, success: bool },
    /// Batch completed
    Completed { result: BatchResult },
    /// Error occurred
    Error { session_id: String, error: String },
}

/// Batch executor
#[derive(Debug)]
pub struct BatchExecutor {
    /// Event channel sender
    event_tx: Option<mpsc::Sender<BatchEvent>>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Session variables
    variables: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for BatchExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchExecutor {
    pub fn new() -> Self {
        Self {
            event_tx: None,
            running: Arc::new(RwLock::new(false)),
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Set event channel
    pub fn with_events(mut self, tx: mpsc::Sender<BatchEvent>) -> Self {
        self.event_tx = Some(tx);
        self
    }
    
    /// Check if running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
    
    /// Cancel execution
    pub fn cancel(&self) {
        *self.running.write() = false;
    }
    
    /// Set a variable
    pub fn set_variable(&self, name: &str, value: &str) {
        self.variables.write().insert(name.to_string(), value.to_string());
    }
    
    /// Get a variable
    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.variables.read().get(name).cloned()
    }
    
    /// Execute batch task (synchronous simulation - real impl would be async)
    pub fn execute<F>(&self, task: &BatchTask, mut session_handler: F) -> BatchResult
    where
        F: FnMut(&str, &BatchOperation) -> Result<Option<String>, String>,
    {
        *self.running.write() = true;
        let start = Instant::now();
        let mut result = BatchResult::new(task.name.clone());
        
        // Send started event
        if let Some(tx) = &self.event_tx {
            let _ = tx.blocking_send(BatchEvent::Started {
                task_name: task.name.clone(),
                total_sessions: task.sessions.len(),
            });
        }
        
        match task.mode {
            ExecutionMode::Sequential => {
                result = self.execute_sequential(task, &mut session_handler);
            }
            ExecutionMode::Parallel => {
                // For true parallel, would need async runtime
                // Simulating sequential for now
                result = self.execute_sequential(task, &mut session_handler);
            }
            ExecutionMode::RoundRobin => {
                result = self.execute_round_robin(task, &mut session_handler);
            }
        }
        
        result.total_duration = start.elapsed();
        *self.running.write() = false;
        
        // Send completed event
        if let Some(tx) = &self.event_tx {
            let _ = tx.blocking_send(BatchEvent::Completed { result: result.clone() });
        }
        
        result
    }
    
    fn execute_sequential<F>(&self, task: &BatchTask, session_handler: &mut F) -> BatchResult
    where
        F: FnMut(&str, &BatchOperation) -> Result<Option<String>, String>,
    {
        let mut result = BatchResult::new(task.name.clone());
        
        for session_id in &task.sessions {
            if !*self.running.read() {
                break;
            }
            
            let session_success = self.execute_session_ops(
                session_id,
                &task.operations,
                &task.error_strategy,
                session_handler,
                &mut result,
            );
            
            if session_success {
                result.successful_sessions.push(session_id.clone());
            } else {
                result.failed_sessions.push(session_id.clone());
                result.success = false;
                
                if matches!(task.error_strategy, ErrorStrategy::StopOnError) {
                    break;
                }
            }
        }
        
        result
    }
    
    fn execute_round_robin<F>(&self, task: &BatchTask, session_handler: &mut F) -> BatchResult
    where
        F: FnMut(&str, &BatchOperation) -> Result<Option<String>, String>,
    {
        let mut result = BatchResult::new(task.name.clone());
        let mut session_indices: Vec<usize> = (0..task.sessions.len()).collect();
        
        for (op_idx, op) in task.operations.iter().enumerate() {
            if !*self.running.read() {
                break;
            }
            
            for session_idx in &session_indices {
                let session_id = &task.sessions[*session_idx];
                let start = Instant::now();
                
                let op_result = session_handler(session_id, op);
                
                let success = op_result.is_ok();
                result.results.push(OperationResult {
                    index: op_idx,
                    session_id: session_id.clone(),
                    success,
                    error: op_result.as_ref().err().cloned(),
                    output: op_result.ok().flatten(),
                    duration: start.elapsed(),
                });
                
                if !success {
                    result.success = false;
                    if matches!(task.error_strategy, ErrorStrategy::StopOnError) {
                        return result;
                    }
                }
            }
        }
        
        // Determine success per session
        for session_id in &task.sessions {
            let session_results: Vec<_> = result.results.iter()
                .filter(|r| r.session_id == *session_id)
                .collect();
            
            if session_results.iter().all(|r| r.success) {
                result.successful_sessions.push(session_id.clone());
            } else {
                result.failed_sessions.push(session_id.clone());
            }
        }
        
        result
    }
    
    fn execute_session_ops<F>(
        &self,
        session_id: &str,
        operations: &[BatchOperation],
        error_strategy: &ErrorStrategy,
        session_handler: &mut F,
        result: &mut BatchResult,
    ) -> bool
    where
        F: FnMut(&str, &BatchOperation) -> Result<Option<String>, String>,
    {
        let mut all_success = true;
        
        for (idx, op) in operations.iter().enumerate() {
            if !*self.running.read() {
                return false;
            }
            
            let start = Instant::now();
            let mut attempts = 0;
            let max_attempts = match error_strategy {
                ErrorStrategy::Retry { max_attempts, .. } => *max_attempts,
                _ => 1,
            };
            
            loop {
                attempts += 1;
                let op_result = session_handler(session_id, op);
                
                match &op_result {
                    Ok(output) => {
                        result.results.push(OperationResult {
                            index: idx,
                            session_id: session_id.to_string(),
                            success: true,
                            error: None,
                            output: output.clone(),
                            duration: start.elapsed(),
                        });
                        break;
                    }
                    Err(e) => {
                        if attempts >= max_attempts {
                            result.results.push(OperationResult {
                                index: idx,
                                session_id: session_id.to_string(),
                                success: false,
                                error: Some(e.clone()),
                                output: None,
                                duration: start.elapsed(),
                            });
                            result.errors.push(format!("{}[{}]: {}", session_id, idx, e));
                            all_success = false;
                            
                            match error_strategy {
                                ErrorStrategy::StopOnError => return false,
                                ErrorStrategy::SkipFailed => break,
                                _ => break,
                            }
                        }
                        
                        // Retry delay
                        if let ErrorStrategy::Retry { delay_ms, .. } = error_strategy {
                            std::thread::sleep(Duration::from_millis(*delay_ms));
                        }
                    }
                }
            }
        }
        
        all_success
    }
}

/// Batch task builder with fluent API
pub struct BatchBuilder {
    task: BatchTask,
}

impl BatchBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            task: BatchTask::new(name),
        }
    }
    
    /// Add target sessions
    pub fn sessions(mut self, sessions: Vec<String>) -> Self {
        self.task.sessions = sessions;
        self
    }
    
    /// Send data
    pub fn send(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.task.operations.push(BatchOperation::Send { data: data.into() });
        self
    }
    
    /// Send text line
    pub fn send_line(mut self, text: impl Into<String>) -> Self {
        self.task.operations.push(BatchOperation::SendLine { text: text.into() });
        self
    }
    
    /// Wait for pattern
    pub fn wait_for(mut self, pattern: impl Into<String>, timeout_ms: u64) -> Self {
        self.task.operations.push(BatchOperation::WaitFor { 
            pattern: pattern.into(),
            timeout_ms,
        });
        self
    }
    
    /// Add delay
    pub fn delay(mut self, duration_ms: u64) -> Self {
        self.task.operations.push(BatchOperation::Delay { duration_ms });
        self
    }
    
    /// Execute command
    pub fn execute(mut self, command: impl Into<String>) -> Self {
        self.task.operations.push(BatchOperation::Execute { command: command.into() });
        self
    }
    
    /// Set execution mode
    pub fn mode(mut self, mode: ExecutionMode) -> Self {
        self.task.mode = mode;
        self
    }
    
    /// Set parallel execution
    pub fn parallel(self) -> Self {
        self.mode(ExecutionMode::Parallel)
    }
    
    /// Set sequential execution
    pub fn sequential(self) -> Self {
        self.mode(ExecutionMode::Sequential)
    }
    
    /// Set error strategy
    pub fn on_error(mut self, strategy: ErrorStrategy) -> Self {
        self.task.error_strategy = strategy;
        self
    }
    
    /// Continue on error
    pub fn continue_on_error(self) -> Self {
        self.on_error(ErrorStrategy::ContinueOnError)
    }
    
    /// Stop on error
    pub fn stop_on_error(self) -> Self {
        self.on_error(ErrorStrategy::StopOnError)
    }
    
    /// Set timeout
    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.task.timeout_ms = timeout_ms;
        self
    }
    
    /// Build task
    pub fn build(self) -> BatchTask {
        self.task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_batch_builder() {
        let task = BatchBuilder::new("test")
            .sessions(vec!["session1".to_string(), "session2".to_string()])
            .send_line("hello")
            .delay(100)
            .wait_for("OK", 5000)
            .parallel()
            .continue_on_error()
            .build();
        
        assert_eq!(task.name, "test");
        assert_eq!(task.sessions.len(), 2);
        assert_eq!(task.operations.len(), 3);
        assert_eq!(task.mode, ExecutionMode::Parallel);
    }
    
    #[test]
    fn test_batch_execution() {
        let executor = BatchExecutor::new();
        
        let task = BatchBuilder::new("test")
            .sessions(vec!["s1".to_string()])
            .send_line("test")
            .build();
        
        let result = executor.execute(&task, |_session, _op| {
            Ok(Some("success".to_string()))
        });
        
        assert!(result.success);
        assert_eq!(result.successful_sessions.len(), 1);
    }
    
    #[test]
    fn test_batch_error_handling() {
        let executor = BatchExecutor::new();
        
        let task = BatchBuilder::new("test")
            .sessions(vec!["s1".to_string()])
            .send_line("test")
            .continue_on_error()
            .build();
        
        let result = executor.execute(&task, |_session, _op| {
            Err("test error".to_string())
        });
        
        assert!(!result.success);
        assert_eq!(result.failed_sessions.len(), 1);
        assert!(!result.errors.is_empty());
    }
}


