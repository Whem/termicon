//! Chart data structures

use std::collections::VecDeque;
use eframe::egui::Color32;

/// Single data point
#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f64,  // Timestamp
    pub y: f64,  // Value
}

impl DataPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Chart configuration
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// Maximum points to keep
    pub max_points: usize,
    /// Time window in seconds (0 = show all)
    pub time_window: f64,
    /// Auto-scale Y axis
    pub auto_scale_y: bool,
    /// Y axis min (when not auto-scaling)
    pub y_min: f64,
    /// Y axis max (when not auto-scaling)
    pub y_max: f64,
    /// Show grid
    pub show_grid: bool,
    /// Show legend
    pub show_legend: bool,
    /// Show points
    pub show_points: bool,
    /// Line width
    pub line_width: f32,
    /// Downsample threshold
    pub downsample_threshold: usize,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            max_points: 10000,
            time_window: 60.0,  // 60 seconds
            auto_scale_y: true,
            y_min: 0.0,
            y_max: 100.0,
            show_grid: true,
            show_legend: true,
            show_points: false,
            line_width: 1.5,
            downsample_threshold: 1000,
        }
    }
}

/// A data channel for charting
#[derive(Debug, Clone)]
pub struct ChartChannel {
    /// Channel name
    pub name: String,
    /// Display color
    pub color: Color32,
    /// Data points
    points: VecDeque<DataPoint>,
    /// Maximum points
    max_points: usize,
    /// Is visible
    pub visible: bool,
    /// Unit label
    pub unit: String,
}

impl ChartChannel {
    /// Create new channel
    pub fn new(name: &str, color: Color32) -> Self {
        Self {
            name: name.to_string(),
            color,
            points: VecDeque::with_capacity(1000),
            max_points: 10000,
            visible: true,
            unit: String::new(),
        }
    }

    /// Add a data point
    pub fn add_point(&mut self, x: f64, y: f64) {
        self.points.push_back(DataPoint::new(x, y));
        
        // Limit size
        while self.points.len() > self.max_points {
            self.points.pop_front();
        }
    }

    /// Get all points
    pub fn points(&self) -> &VecDeque<DataPoint> {
        &self.points
    }

    /// Get points in time window
    pub fn points_in_window(&self, min_time: f64, max_time: f64) -> Vec<&DataPoint> {
        self.points
            .iter()
            .filter(|p| p.x >= min_time && p.x <= max_time)
            .collect()
    }

    /// Get downsampled points for display
    pub fn downsampled(&self, target_points: usize) -> Vec<DataPoint> {
        if self.points.len() <= target_points {
            return self.points.iter().copied().collect();
        }

        let step = self.points.len() / target_points;
        self.points
            .iter()
            .step_by(step.max(1))
            .copied()
            .collect()
    }

    /// Get last N points
    pub fn last_n(&self, n: usize) -> Vec<&DataPoint> {
        let skip = self.points.len().saturating_sub(n);
        self.points.iter().skip(skip).collect()
    }

    /// Get latest value
    pub fn last_value(&self) -> Option<f64> {
        self.points.back().map(|p| p.y)
    }

    /// Get min/max Y values
    pub fn y_range(&self) -> Option<(f64, f64)> {
        if self.points.is_empty() {
            return None;
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for point in &self.points {
            min = min.min(point.y);
            max = max.max(point.y);
        }

        Some((min, max))
    }

    /// Get time range
    pub fn time_range(&self) -> Option<(f64, f64)> {
        if self.points.is_empty() {
            return None;
        }

        let first = self.points.front()?.x;
        let last = self.points.back()?.x;
        Some((first, last))
    }

    /// Clear all points
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Set max points
    pub fn set_max_points(&mut self, max: usize) {
        self.max_points = max;
        while self.points.len() > self.max_points {
            self.points.pop_front();
        }
    }

    /// Point count
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Historical data for export/replay
#[derive(Debug, Clone)]
pub struct ChartData {
    pub channels: Vec<(String, Vec<DataPoint>)>,
    pub start_time: f64,
    pub end_time: f64,
}

impl ChartData {
    /// Create from channels
    pub fn from_channels(channels: &[(String, &ChartChannel)]) -> Self {
        let mut start_time = f64::MAX;
        let mut end_time = f64::MIN;

        let channel_data: Vec<(String, Vec<DataPoint>)> = channels
            .iter()
            .map(|(name, channel)| {
                if let Some((min, max)) = channel.time_range() {
                    start_time = start_time.min(min);
                    end_time = end_time.max(max);
                }
                (name.clone(), channel.points().iter().copied().collect())
            })
            .collect();

        Self {
            channels: channel_data,
            start_time: if start_time == f64::MAX { 0.0 } else { start_time },
            end_time: if end_time == f64::MIN { 0.0 } else { end_time },
        }
    }

    /// Get duration
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Get total point count
    pub fn total_points(&self) -> usize {
        self.channels.iter().map(|(_, points)| points.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_add_points() {
        let mut channel = ChartChannel::new("test", Color32::RED);
        channel.add_point(1.0, 10.0);
        channel.add_point(2.0, 20.0);
        channel.add_point(3.0, 30.0);

        assert_eq!(channel.len(), 3);
        assert_eq!(channel.last_value(), Some(30.0));
    }

    #[test]
    fn test_channel_y_range() {
        let mut channel = ChartChannel::new("test", Color32::RED);
        channel.add_point(1.0, 10.0);
        channel.add_point(2.0, 50.0);
        channel.add_point(3.0, 30.0);

        let (min, max) = channel.y_range().unwrap();
        assert_eq!(min, 10.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_channel_max_points() {
        let mut channel = ChartChannel::new("test", Color32::RED);
        channel.set_max_points(3);

        for i in 0..10 {
            channel.add_point(i as f64, i as f64);
        }

        assert_eq!(channel.len(), 3);
    }
}
