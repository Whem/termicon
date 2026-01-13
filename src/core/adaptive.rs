//! Adaptive Automation / Feedback Control
//! 
//! Closed-loop automation that:
//! - Measures → Decides → Modifies → Retries
//! - Auto-adjusts parameters based on feedback
//! - Self-optimizing communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Metric being measured
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Metric {
    /// Error rate (0.0 - 1.0)
    ErrorRate,
    /// Latency in milliseconds
    Latency,
    /// Throughput in bytes/second
    Throughput,
    /// Packet loss rate
    PacketLoss,
    /// CRC error count
    CrcErrors,
    /// Timeout count
    Timeouts,
    /// Queue depth
    QueueDepth,
    /// Custom metric
    Custom(String),
}

/// Condition for triggering adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveCondition {
    /// Metric to monitor
    pub metric: Metric,
    /// Comparison operator
    pub operator: ComparisonOp,
    /// Threshold value
    pub threshold: f64,
    /// Window size for averaging (in samples or time)
    pub window_size: usize,
    /// Hysteresis to prevent oscillation
    pub hysteresis: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    Equal,
    GreaterOrEqual,
    LessOrEqual,
}

impl ComparisonOp {
    pub fn compare(&self, value: f64, threshold: f64) -> bool {
        match self {
            ComparisonOp::GreaterThan => value > threshold,
            ComparisonOp::LessThan => value < threshold,
            ComparisonOp::Equal => (value - threshold).abs() < f64::EPSILON,
            ComparisonOp::GreaterOrEqual => value >= threshold,
            ComparisonOp::LessOrEqual => value <= threshold,
        }
    }
}

/// Action to take when condition is met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptiveAction {
    /// Adjust baud rate
    AdjustBaudRate { delta: i32 },
    /// Set specific baud rate
    SetBaudRate { rate: u32 },
    /// Adjust packet size
    AdjustPacketSize { delta: i32 },
    /// Set specific packet size
    SetPacketSize { size: usize },
    /// Adjust timeout
    AdjustTimeout { delta_ms: i32 },
    /// Adjust retry count
    AdjustRetries { delta: i32 },
    /// Enable/disable flow control
    SetFlowControl { enabled: bool },
    /// Renegotiate MTU (for BLE)
    RenegotiateMtu { preferred_mtu: u16 },
    /// Add delay between packets
    SetInterPacketDelay { delay_ms: u64 },
    /// Switch protocol variant
    SwitchProtocol { protocol: String },
    /// Send notification
    Notify { message: String },
    /// Run script
    RunScript { script: String },
    /// Reset connection
    ResetConnection,
    /// Do nothing (for logging only)
    LogOnly { message: String },
}

/// Feedback control rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRule {
    /// Rule name
    pub name: String,
    /// Rule is enabled
    pub enabled: bool,
    /// Conditions (all must be met)
    pub conditions: Vec<AdaptiveCondition>,
    /// Actions to take
    pub actions: Vec<AdaptiveAction>,
    /// Cooldown between activations (ms)
    pub cooldown_ms: u64,
    /// Last activation time
    #[serde(skip)]
    pub last_activation: Option<Instant>,
    /// Activation count
    pub activation_count: u64,
    /// Priority (higher = evaluated first)
    pub priority: i32,
}

impl FeedbackRule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            conditions: Vec::new(),
            actions: Vec::new(),
            cooldown_ms: 1000,
            last_activation: None,
            activation_count: 0,
            priority: 0,
        }
    }

    /// Add a condition
    pub fn when(mut self, metric: Metric, op: ComparisonOp, threshold: f64) -> Self {
        self.conditions.push(AdaptiveCondition {
            metric,
            operator: op,
            threshold,
            window_size: 10,
            hysteresis: 0.1,
        });
        self
    }

    /// Add an action
    pub fn then(mut self, action: AdaptiveAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set cooldown
    pub fn with_cooldown(mut self, ms: u64) -> Self {
        self.cooldown_ms = ms;
        self
    }

    /// Check if rule can activate (cooldown elapsed)
    pub fn can_activate(&self) -> bool {
        match self.last_activation {
            Some(time) => time.elapsed().as_millis() as u64 >= self.cooldown_ms,
            None => true,
        }
    }

    /// Activate the rule
    pub fn activate(&mut self) {
        self.last_activation = Some(Instant::now());
        self.activation_count += 1;
    }
}

/// Metric tracker with history
#[derive(Debug, Clone)]
pub struct MetricTracker {
    pub metric: Metric,
    pub values: Vec<(Instant, f64)>,
    pub window_size: usize,
    pub current_average: f64,
}

impl MetricTracker {
    pub fn new(metric: Metric, window_size: usize) -> Self {
        Self {
            metric,
            values: Vec::new(),
            window_size,
            current_average: 0.0,
        }
    }

    /// Record a value
    pub fn record(&mut self, value: f64) {
        self.values.push((Instant::now(), value));
        
        // Keep only window_size values
        if self.values.len() > self.window_size {
            self.values.remove(0);
        }

        // Update average
        self.current_average = self.values.iter()
            .map(|(_, v)| v)
            .sum::<f64>() / self.values.len() as f64;
    }

    /// Get current average
    pub fn average(&self) -> f64 {
        self.current_average
    }

    /// Get min value in window
    pub fn min(&self) -> f64 {
        self.values.iter()
            .map(|(_, v)| *v)
            .fold(f64::INFINITY, f64::min)
    }

    /// Get max value in window
    pub fn max(&self) -> f64 {
        self.values.iter()
            .map(|(_, v)| *v)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Get trend (positive = increasing)
    pub fn trend(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }

        let n = self.values.len() as f64;
        let (sum_x, sum_y, sum_xy, sum_xx) = self.values.iter()
            .enumerate()
            .fold((0.0, 0.0, 0.0, 0.0), |(sx, sy, sxy, sxx), (i, (_, v))| {
                let x = i as f64;
                (sx + x, sy + v, sxy + x * v, sxx + x * x)
            });

        // Linear regression slope
        (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x)
    }
}

/// Adaptive controller
#[derive(Debug)]
pub struct AdaptiveController {
    /// Rules
    pub rules: Vec<FeedbackRule>,
    /// Metric trackers
    pub metrics: HashMap<String, MetricTracker>,
    /// Pending actions
    pub pending_actions: Vec<AdaptiveAction>,
    /// Event log
    pub event_log: Vec<AdaptiveEvent>,
    /// Controller enabled
    pub enabled: bool,
}

/// Event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveEvent {
    pub timestamp: String,
    pub rule_name: String,
    pub metric_values: HashMap<String, f64>,
    pub actions_taken: Vec<String>,
}

impl Default for AdaptiveController {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveController {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            metrics: HashMap::new(),
            pending_actions: Vec::new(),
            event_log: Vec::new(),
            enabled: true,
        }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: FeedbackRule) {
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Record a metric value
    pub fn record_metric(&mut self, metric: Metric, value: f64) {
        let key = format!("{:?}", metric);
        
        if !self.metrics.contains_key(&key) {
            self.metrics.insert(key.clone(), MetricTracker::new(metric.clone(), 100));
        }

        if let Some(tracker) = self.metrics.get_mut(&key) {
            tracker.record(value);
        }
    }

    /// Evaluate all rules and collect actions
    pub fn evaluate(&mut self) -> Vec<AdaptiveAction> {
        if !self.enabled {
            return Vec::new();
        }

        let mut actions = Vec::new();

        for rule in &mut self.rules {
            if !rule.enabled || !rule.can_activate() {
                continue;
            }

            // Check all conditions
            let all_met = rule.conditions.iter().all(|cond| {
                let key = format!("{:?}", cond.metric);
                if let Some(tracker) = self.metrics.get(&key) {
                    cond.operator.compare(tracker.average(), cond.threshold)
                } else {
                    false
                }
            });

            if all_met {
                rule.activate();
                actions.extend(rule.actions.clone());

                // Log event
                let mut metric_values = HashMap::new();
                for (key, tracker) in &self.metrics {
                    metric_values.insert(key.clone(), tracker.average());
                }

                self.event_log.push(AdaptiveEvent {
                    timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    rule_name: rule.name.clone(),
                    metric_values,
                    actions_taken: rule.actions.iter().map(|a| format!("{:?}", a)).collect(),
                });
            }
        }

        actions
    }

    /// Create common rules
    pub fn setup_default_rules(&mut self) {
        // High error rate → reduce baud rate
        self.add_rule(
            FeedbackRule::new("High Error Rate")
                .when(Metric::ErrorRate, ComparisonOp::GreaterThan, 0.1)
                .then(AdaptiveAction::AdjustBaudRate { delta: -9600 })
                .with_cooldown(5000)
        );

        // High latency → reduce packet size
        self.add_rule(
            FeedbackRule::new("High Latency")
                .when(Metric::Latency, ComparisonOp::GreaterThan, 500.0)
                .then(AdaptiveAction::AdjustPacketSize { delta: -64 })
                .with_cooldown(3000)
        );

        // Many timeouts → increase timeout
        self.add_rule(
            FeedbackRule::new("Frequent Timeouts")
                .when(Metric::Timeouts, ComparisonOp::GreaterThan, 5.0)
                .then(AdaptiveAction::AdjustTimeout { delta_ms: 100 })
                .with_cooldown(2000)
        );

        // CRC errors → enable flow control
        self.add_rule(
            FeedbackRule::new("CRC Errors")
                .when(Metric::CrcErrors, ComparisonOp::GreaterThan, 3.0)
                .then(AdaptiveAction::SetFlowControl { enabled: true })
                .with_cooldown(10000)
        );
    }
}




