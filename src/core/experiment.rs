//! Experiment / Parameter Sweep Mode
//! 
//! Systematic testing with:
//! - Parameter sweeps
//! - Result analysis
//! - Heatmap generation
//! - Automated optimization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Current value
    pub current_value: f64,
    /// Unit (e.g., "baud", "ms", "bytes")
    pub unit: String,
}

/// Parameter type with range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// Continuous range
    Range {
        min: f64,
        max: f64,
        step: f64,
    },
    /// Discrete values
    Discrete {
        values: Vec<f64>,
    },
    /// Boolean
    Boolean,
    /// Logarithmic scale
    Logarithmic {
        min: f64,
        max: f64,
        base: f64,
    },
}

impl Parameter {
    pub fn range(name: &str, min: f64, max: f64, step: f64, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            param_type: ParameterType::Range { min, max, step },
            current_value: min,
            unit: unit.to_string(),
        }
    }

    pub fn discrete(name: &str, values: Vec<f64>, unit: &str) -> Self {
        Self {
            name: name.to_string(),
            param_type: ParameterType::Discrete { values: values.clone() },
            current_value: values.first().copied().unwrap_or(0.0),
            unit: unit.to_string(),
        }
    }

    /// Get all values to sweep
    pub fn all_values(&self) -> Vec<f64> {
        match &self.param_type {
            ParameterType::Range { min, max, step } => {
                let mut values = Vec::new();
                let mut v = *min;
                while v <= *max {
                    values.push(v);
                    v += step;
                }
                values
            }
            ParameterType::Discrete { values } => values.clone(),
            ParameterType::Boolean => vec![0.0, 1.0],
            ParameterType::Logarithmic { min, max, base } => {
                let mut values = Vec::new();
                let mut v = *min;
                while v <= *max {
                    values.push(v);
                    v *= base;
                }
                values
            }
        }
    }
}

/// Metric to measure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub name: String,
    pub unit: String,
    pub higher_is_better: bool,
}

/// Single experiment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// Parameter values for this run
    pub parameters: HashMap<String, f64>,
    /// Measured metrics
    pub metrics: HashMap<String, f64>,
    /// Run duration
    pub duration_ms: u64,
    /// Success/failure
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Notes
    pub notes: String,
}

/// Experiment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Experiment name
    pub name: String,
    /// Description
    pub description: String,
    /// Parameters to sweep
    pub parameters: Vec<Parameter>,
    /// Metrics to measure
    pub metrics: Vec<MetricDefinition>,
    /// Results
    pub results: Vec<ExperimentResult>,
    /// Number of repetitions per configuration
    pub repetitions: usize,
    /// Timeout per run (ms)
    pub timeout_ms: u64,
    /// Status
    pub status: ExperimentStatus,
    /// Created timestamp
    pub created: String,
    /// Completed timestamp
    pub completed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl Experiment {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            parameters: Vec::new(),
            metrics: Vec::new(),
            results: Vec::new(),
            repetitions: 3,
            timeout_ms: 10000,
            status: ExperimentStatus::Pending,
            created: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            completed: None,
        }
    }

    /// Add a parameter
    pub fn add_parameter(&mut self, param: Parameter) {
        self.parameters.push(param);
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: &str, unit: &str, higher_is_better: bool) {
        self.metrics.push(MetricDefinition {
            name: name.to_string(),
            unit: unit.to_string(),
            higher_is_better,
        });
    }

    /// Get total number of configurations
    pub fn total_configurations(&self) -> usize {
        self.parameters.iter()
            .map(|p| p.all_values().len())
            .product::<usize>()
            * self.repetitions
    }

    /// Generate all parameter combinations
    pub fn all_configurations(&self) -> Vec<HashMap<String, f64>> {
        let mut configs = vec![HashMap::new()];

        for param in &self.parameters {
            let mut new_configs = Vec::new();
            for value in param.all_values() {
                for config in &configs {
                    let mut new_config = config.clone();
                    new_config.insert(param.name.clone(), value);
                    new_configs.push(new_config);
                }
            }
            configs = new_configs;
        }

        configs
    }

    /// Add a result
    pub fn add_result(&mut self, result: ExperimentResult) {
        self.results.push(result);
    }

    /// Get best configuration for a metric
    pub fn best_configuration(&self, metric: &str) -> Option<&ExperimentResult> {
        let metric_def = self.metrics.iter().find(|m| m.name == metric)?;
        
        self.results.iter()
            .filter(|r| r.success && r.metrics.contains_key(metric))
            .max_by(|a, b| {
                let va = a.metrics.get(metric).unwrap_or(&0.0);
                let vb = b.metrics.get(metric).unwrap_or(&0.0);
                if metric_def.higher_is_better {
                    va.partial_cmp(vb).unwrap()
                } else {
                    vb.partial_cmp(va).unwrap()
                }
            })
    }

    /// Generate summary statistics
    pub fn summary(&self) -> ExperimentSummary {
        let successful = self.results.iter().filter(|r| r.success).count();
        
        let mut metric_stats = HashMap::new();
        for metric in &self.metrics {
            let values: Vec<f64> = self.results.iter()
                .filter(|r| r.success)
                .filter_map(|r| r.metrics.get(&metric.name))
                .copied()
                .collect();

            if !values.is_empty() {
                let sum: f64 = values.iter().sum();
                let mean = sum / values.len() as f64;
                let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let variance = values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / values.len() as f64;
                let std_dev = variance.sqrt();

                metric_stats.insert(metric.name.clone(), MetricStats {
                    min,
                    max,
                    mean,
                    std_dev,
                    count: values.len(),
                });
            }
        }

        ExperimentSummary {
            total_runs: self.results.len(),
            successful_runs: successful,
            failed_runs: self.results.len() - successful,
            metric_stats,
        }
    }

    /// Generate 2D heatmap data for two parameters
    pub fn heatmap(&self, param_x: &str, param_y: &str, metric: &str) -> Option<HeatmapData> {
        let x_values: Vec<f64> = self.parameters.iter()
            .find(|p| p.name == param_x)?
            .all_values();
        let y_values: Vec<f64> = self.parameters.iter()
            .find(|p| p.name == param_y)?
            .all_values();

        let mut grid = vec![vec![f64::NAN; x_values.len()]; y_values.len()];

        for result in &self.results {
            if !result.success {
                continue;
            }
            
            let x_val = result.parameters.get(param_x)?;
            let y_val = result.parameters.get(param_y)?;
            let metric_val = result.metrics.get(metric)?;

            let x_idx = x_values.iter().position(|v| (v - x_val).abs() < f64::EPSILON)?;
            let y_idx = y_values.iter().position(|v| (v - y_val).abs() < f64::EPSILON)?;

            grid[y_idx][x_idx] = *metric_val;
        }

        Some(HeatmapData {
            x_label: param_x.to_string(),
            y_label: param_y.to_string(),
            metric_label: metric.to_string(),
            x_values,
            y_values,
            grid,
        })
    }
}

/// Experiment summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSummary {
    pub total_runs: usize,
    pub successful_runs: usize,
    pub failed_runs: usize,
    pub metric_stats: HashMap<String, MetricStats>,
}

/// Statistics for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub count: usize,
}

/// Heatmap data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    pub x_label: String,
    pub y_label: String,
    pub metric_label: String,
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
    pub grid: Vec<Vec<f64>>,
}

/// Experiment runner
#[derive(Debug)]
pub struct ExperimentRunner {
    pub experiments: Vec<Experiment>,
    pub current_experiment: Option<usize>,
    pub current_config_index: usize,
    pub current_repetition: usize,
}

impl Default for ExperimentRunner {
    fn default() -> Self {
        Self {
            experiments: Vec::new(),
            current_experiment: None,
            current_config_index: 0,
            current_repetition: 0,
        }
    }
}

impl ExperimentRunner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an experiment
    pub fn add_experiment(&mut self, experiment: Experiment) -> usize {
        self.experiments.push(experiment);
        self.experiments.len() - 1
    }

    /// Start an experiment
    pub fn start(&mut self, index: usize) -> Result<(), String> {
        if index >= self.experiments.len() {
            return Err("Invalid experiment index".to_string());
        }

        self.current_experiment = Some(index);
        self.current_config_index = 0;
        self.current_repetition = 0;
        self.experiments[index].status = ExperimentStatus::Running;

        Ok(())
    }

    /// Get next configuration to test
    pub fn next_configuration(&mut self) -> Option<HashMap<String, f64>> {
        let exp_idx = self.current_experiment?;
        let experiment = self.experiments.get(exp_idx)?;

        let configs = experiment.all_configurations();
        
        if self.current_config_index >= configs.len() {
            // All done
            return None;
        }

        let config = configs.get(self.current_config_index)?.clone();

        self.current_repetition += 1;
        if self.current_repetition >= experiment.repetitions {
            self.current_repetition = 0;
            self.current_config_index += 1;
        }

        Some(config)
    }

    /// Complete current experiment
    pub fn complete(&mut self) {
        if let Some(idx) = self.current_experiment {
            self.experiments[idx].status = ExperimentStatus::Completed;
            self.experiments[idx].completed = Some(
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            );
        }
        self.current_experiment = None;
    }
}

