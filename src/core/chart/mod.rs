//! Real-time charting and data visualization
//!
//! Provides:
//! - Real-time line plots
//! - Multi-channel support
//! - Data parsing from various formats
//! - Export capabilities

pub mod data;
pub mod parser;

pub use data::{ChartData, DataPoint, ChartConfig, ChartChannel};
pub use parser::{DataParser, ParserConfig};

use std::collections::HashMap;
use eframe::egui::Color32;

/// Chart manager for handling multiple data channels
pub struct ChartManager {
    /// Data channels by name
    channels: HashMap<String, ChartChannel>,
    /// Configuration
    pub config: ChartConfig,
    /// Parser for incoming data
    parser: DataParser,
    /// Is recording
    recording: bool,
}

impl Default for ChartManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartManager {
    /// Create new chart manager
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            config: ChartConfig::default(),
            parser: DataParser::new(),
            recording: true,
        }
    }

    /// Add or get channel
    pub fn channel(&mut self, name: &str) -> &mut ChartChannel {
        let len = self.channels.len();
        self.channels.entry(name.to_string()).or_insert_with(|| {
            let color = Self::next_color(len);
            ChartChannel::new(name, color)
        })
    }

    /// Get channel by name
    pub fn get_channel(&self, name: &str) -> Option<&ChartChannel> {
        self.channels.get(name)
    }

    /// Get all channels
    pub fn channels(&self) -> &HashMap<String, ChartChannel> {
        &self.channels
    }

    /// Get all channel names
    pub fn channel_names(&self) -> Vec<&str> {
        self.channels.keys().map(|s| s.as_str()).collect()
    }

    /// Process incoming data line
    pub fn process_line(&mut self, line: &str) {
        if !self.recording {
            return;
        }

        if let Some(values) = self.parser.parse_line(line) {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);

            for (name, value) in values {
                self.channel(&name).add_point(timestamp, value);
            }
        }
    }

    /// Process raw bytes
    pub fn process_bytes(&mut self, data: &[u8]) {
        if !self.recording {
            return;
        }

        // Convert to string and process lines
        if let Ok(text) = std::str::from_utf8(data) {
            for line in text.lines() {
                self.process_line(line);
            }
        }
    }

    /// Add single value to channel
    pub fn add_value(&mut self, channel: &str, value: f64) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        self.channel(channel).add_point(timestamp, value);
    }

    /// Clear all data
    pub fn clear(&mut self) {
        for channel in self.channels.values_mut() {
            channel.clear();
        }
    }

    /// Set recording state
    pub fn set_recording(&mut self, recording: bool) {
        self.recording = recording;
    }

    /// Is recording
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Set parser config
    pub fn set_parser_config(&mut self, config: ParserConfig) {
        self.parser.set_config(config);
    }

    /// Get parser config
    pub fn parser_config(&self) -> &ParserConfig {
        self.parser.config()
    }

    /// Export to CSV
    pub fn export_csv(&self) -> String {
        let mut csv = String::new();

        // Header
        csv.push_str("Timestamp");
        for name in self.channels.keys() {
            csv.push(',');
            csv.push_str(name);
        }
        csv.push('\n');

        // Find all unique timestamps
        let mut all_points: Vec<(f64, HashMap<&str, f64>)> = Vec::new();

        for (name, channel) in &self.channels {
            for point in channel.points() {
                // Find or create entry for this timestamp
                let entry = all_points.iter_mut().find(|(t, _)| (*t - point.x).abs() < 0.001);
                if let Some((_, values)) = entry {
                    values.insert(name, point.y);
                } else {
                    let mut values = HashMap::new();
                    values.insert(name.as_str(), point.y);
                    all_points.push((point.x, values));
                }
            }
        }

        // Sort by timestamp
        all_points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Write rows
        for (timestamp, values) in all_points {
            csv.push_str(&format!("{:.3}", timestamp));
            for name in self.channels.keys() {
                csv.push(',');
                if let Some(value) = values.get(name.as_str()) {
                    csv.push_str(&format!("{:.6}", value));
                }
            }
            csv.push('\n');
        }

        csv
    }

    /// Get next color for new channel
    fn next_color(index: usize) -> Color32 {
        const COLORS: [Color32; 10] = [
            Color32::from_rgb(31, 119, 180),   // Blue
            Color32::from_rgb(255, 127, 14),   // Orange
            Color32::from_rgb(44, 160, 44),    // Green
            Color32::from_rgb(214, 39, 40),    // Red
            Color32::from_rgb(148, 103, 189),  // Purple
            Color32::from_rgb(140, 86, 75),    // Brown
            Color32::from_rgb(227, 119, 194),  // Pink
            Color32::from_rgb(127, 127, 127),  // Gray
            Color32::from_rgb(188, 189, 34),   // Yellow-green
            Color32::from_rgb(23, 190, 207),   // Cyan
        ];
        COLORS[index % COLORS.len()]
    }

    /// Get statistics for channel
    pub fn channel_stats(&self, name: &str) -> Option<ChannelStats> {
        self.channels.get(name).map(|c| c.stats())
    }
}

/// Channel statistics
#[derive(Debug, Clone, Default)]
pub struct ChannelStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub last: f64,
}

impl ChartChannel {
    /// Calculate statistics
    pub fn stats(&self) -> ChannelStats {
        let points = self.points();
        if points.is_empty() {
            return ChannelStats::default();
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut sum = 0.0;

        for point in points {
            min = min.min(point.y);
            max = max.max(point.y);
            sum += point.y;
        }

        ChannelStats {
            count: points.len(),
            min,
            max,
            avg: sum / points.len() as f64,
            last: points.back().map(|p| p.y).unwrap_or(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_manager() {
        let mut manager = ChartManager::new();
        manager.add_value("temp", 25.5);
        manager.add_value("temp", 26.0);
        manager.add_value("humidity", 60.0);

        assert_eq!(manager.channels().len(), 2);
        assert_eq!(manager.get_channel("temp").unwrap().points().len(), 2);
    }

    #[test]
    fn test_export_csv() {
        let mut manager = ChartManager::new();
        manager.add_value("sensor1", 100.0);
        manager.add_value("sensor2", 200.0);

        let csv = manager.export_csv();
        assert!(csv.contains("sensor1"));
        assert!(csv.contains("sensor2"));
    }
}
