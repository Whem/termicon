//! Chart Panel GUI Component
//!
//! Real-time data visualization panel

use eframe::egui::{self, Color32, RichText, Vec2};
use egui_plot::{Line, Plot, PlotPoints};
use std::collections::VecDeque;
use chrono::{DateTime, Local};

/// Data point for chart
#[derive(Debug, Clone)]
pub struct ChartDataPoint {
    pub timestamp: f64,
    pub value: f64,
}

/// Chart channel
#[derive(Debug, Clone)]
pub struct ChartChannel {
    pub name: String,
    pub data: VecDeque<ChartDataPoint>,
    pub color: Color32,
    pub visible: bool,
    pub max_points: usize,
}

impl ChartChannel {
    pub fn new(name: &str, color: Color32) -> Self {
        Self {
            name: name.to_string(),
            data: VecDeque::with_capacity(1000),
            color,
            visible: true,
            max_points: 1000,
        }
    }

    pub fn add_point(&mut self, value: f64) {
        let timestamp = Local::now().timestamp_millis() as f64 / 1000.0;
        self.data.push_back(ChartDataPoint { timestamp, value });
        
        while self.data.len() > self.max_points {
            self.data.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn last_value(&self) -> Option<f64> {
        self.data.back().map(|p| p.value)
    }

    pub fn min_value(&self) -> Option<f64> {
        self.data.iter().map(|p| p.value).fold(None, |min, v| {
            Some(min.map_or(v, |m: f64| m.min(v)))
        })
    }

    pub fn max_value(&self) -> Option<f64> {
        self.data.iter().map(|p| p.value).fold(None, |max, v| {
            Some(max.map_or(v, |m: f64| m.max(v)))
        })
    }

    pub fn avg_value(&self) -> Option<f64> {
        if self.data.is_empty() {
            None
        } else {
            let sum: f64 = self.data.iter().map(|p| p.value).sum();
            Some(sum / self.data.len() as f64)
        }
    }
}

/// Chart parser type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartParserType {
    /// Comma-separated values
    Csv,
    /// Key=value pairs
    KeyValue,
    /// JSON object
    Json,
    /// Single number per line
    Number,
}

/// Chart panel state
pub struct ChartPanel {
    pub channels: Vec<ChartChannel>,
    pub parser_type: ChartParserType,
    pub csv_column: usize,
    pub key_name: String,
    pub auto_scale: bool,
    pub show_legend: bool,
    pub show_stats: bool,
    pub paused: bool,
    start_time: f64,
}

impl Default for ChartPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartPanel {
    pub fn new() -> Self {
        Self {
            channels: vec![
                ChartChannel::new("Channel 1", Color32::from_rgb(66, 165, 245)),
            ],
            parser_type: ChartParserType::Number,
            csv_column: 0,
            key_name: "value".to_string(),
            auto_scale: true,
            show_legend: true,
            show_stats: true,
            paused: false,
            start_time: Local::now().timestamp_millis() as f64 / 1000.0,
        }
    }

    /// Parse incoming data and add to chart
    pub fn parse_data(&mut self, data: &str) {
        if self.paused {
            return;
        }

        match self.parser_type {
            ChartParserType::Number => {
                if let Ok(value) = data.trim().parse::<f64>() {
                    if let Some(channel) = self.channels.first_mut() {
                        channel.add_point(value);
                    }
                }
            }
            ChartParserType::Csv => {
                let parts: Vec<&str> = data.trim().split(',').collect();
                if let Some(part) = parts.get(self.csv_column) {
                    if let Ok(value) = part.trim().parse::<f64>() {
                        if let Some(channel) = self.channels.first_mut() {
                            channel.add_point(value);
                        }
                    }
                }
            }
            ChartParserType::KeyValue => {
                // Parse "key=value" or "key:value"
                for pair in data.split(&[',', ' ', ';'][..]) {
                    let parts: Vec<&str> = pair.split(&['=', ':'][..]).collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        if key == self.key_name {
                            if let Ok(value) = parts[1].trim().parse::<f64>() {
                                if let Some(channel) = self.channels.first_mut() {
                                    channel.add_point(value);
                                }
                            }
                        }
                    }
                }
            }
            ChartParserType::Json => {
                // Simple JSON parsing for {"key": value}
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(value) = json.get(&self.key_name) {
                        if let Some(num) = value.as_f64() {
                            if let Some(channel) = self.channels.first_mut() {
                                channel.add_point(num);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Add a new channel
    pub fn add_channel(&mut self, name: &str) {
        let colors = [
            Color32::from_rgb(66, 165, 245),   // Blue
            Color32::from_rgb(239, 83, 80),    // Red
            Color32::from_rgb(102, 187, 106),  // Green
            Color32::from_rgb(255, 167, 38),   // Orange
            Color32::from_rgb(171, 71, 188),   // Purple
            Color32::from_rgb(255, 241, 118),  // Yellow
        ];
        let color = colors[self.channels.len() % colors.len()];
        self.channels.push(ChartChannel::new(name, color));
    }

    /// Clear all data
    pub fn clear(&mut self) {
        for channel in &mut self.channels {
            channel.clear();
        }
        self.start_time = Local::now().timestamp_millis() as f64 / 1000.0;
    }

    /// Export data to CSV
    pub fn export_csv(&self) -> String {
        let mut csv = String::new();
        
        // Header
        csv.push_str("timestamp");
        for channel in &self.channels {
            csv.push(',');
            csv.push_str(&channel.name);
        }
        csv.push('\n');
        
        // Find max length
        let max_len = self.channels.iter().map(|c| c.data.len()).max().unwrap_or(0);
        
        for i in 0..max_len {
            if let Some(first_channel) = self.channels.first() {
                if let Some(point) = first_channel.data.get(i) {
                    csv.push_str(&format!("{:.3}", point.timestamp - self.start_time));
                }
            }
            
            for channel in &self.channels {
                csv.push(',');
                if let Some(point) = channel.data.get(i) {
                    csv.push_str(&format!("{:.6}", point.value));
                }
            }
            csv.push('\n');
        }
        
        csv
    }

    /// Render the chart panel
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Toolbar
        ui.horizontal(|ui| {
            ui.label(RichText::new("[~]").size(16.0));
            ui.label(RichText::new("Chart View").strong());
            
            ui.separator();
            
            // Parser type
            ui.label("Parser:");
            egui::ComboBox::from_id_salt("parser_type")
                .selected_text(match self.parser_type {
                    ChartParserType::Number => "Number",
                    ChartParserType::Csv => "CSV",
                    ChartParserType::KeyValue => "Key=Value",
                    ChartParserType::Json => "JSON",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.parser_type, ChartParserType::Number, "Number");
                    ui.selectable_value(&mut self.parser_type, ChartParserType::Csv, "CSV");
                    ui.selectable_value(&mut self.parser_type, ChartParserType::KeyValue, "Key=Value");
                    ui.selectable_value(&mut self.parser_type, ChartParserType::Json, "JSON");
                });
            
            // Parser config
            match self.parser_type {
                ChartParserType::Csv => {
                    ui.label("Column:");
                    ui.add(egui::DragValue::new(&mut self.csv_column).range(0..=10));
                }
                ChartParserType::KeyValue | ChartParserType::Json => {
                    ui.label("Key:");
                    ui.add(egui::TextEdit::singleline(&mut self.key_name).desired_width(60.0));
                }
                _ => {}
            }
            
            ui.separator();
            
            // Controls
            if ui.button(if self.paused { "▶ Resume" } else { "⏸ Pause" }).clicked() {
                self.paused = !self.paused;
            }
            
            if ui.button("🗑 Clear").clicked() {
                self.clear();
            }
            
            ui.checkbox(&mut self.auto_scale, "Auto-scale");
            ui.checkbox(&mut self.show_legend, "Legend");
            ui.checkbox(&mut self.show_stats, "Stats");
        });
        
        ui.separator();
        
        // Main content
        ui.horizontal(|ui| {
            // Chart area
            let chart_width = if self.show_stats {
                ui.available_width() - 150.0
            } else {
                ui.available_width()
            };
            
            ui.allocate_ui(Vec2::new(chart_width, ui.available_height()), |ui| {
                let mut plot = Plot::new("data_plot")
                    .height(ui.available_height())
                    .show_axes(true)
                    .show_grid(true)
                    .legend(if self.show_legend {
                        egui_plot::Legend::default()
                    } else {
                        egui_plot::Legend::default().position(egui_plot::Corner::RightTop)
                    });
                
                if !self.auto_scale {
                    plot = plot.auto_bounds(egui::Vec2b::new(true, false));
                }
                
                plot.show(ui, |plot_ui| {
                    for channel in &self.channels {
                        if !channel.visible {
                            continue;
                        }
                        
                        let points: PlotPoints = channel.data.iter()
                            .map(|p| [p.timestamp - self.start_time, p.value])
                            .collect();
                        
                        let line = Line::new(points)
                            .color(channel.color)
                            .name(&channel.name);
                        
                        plot_ui.line(line);
                    }
                });
            });
            
            // Stats panel
            if self.show_stats {
                ui.separator();
                ui.vertical(|ui| {
                    ui.set_min_width(140.0);
                    ui.label(RichText::new("Statistics").strong());
                    ui.separator();
                    
                    for channel in &self.channels {
                        ui.horizontal(|ui| {
                            let color_rect = egui::Rect::from_min_size(
                                ui.cursor().min,
                                Vec2::new(10.0, 10.0),
                            );
                            ui.painter().rect_filled(color_rect, 0.0, channel.color);
                            ui.add_space(12.0);
                            ui.label(&channel.name);
                        });
                        
                        ui.indent(channel.name.as_str(), |ui| {
                            if let Some(last) = channel.last_value() {
                                ui.label(format!("Last: {:.2}", last));
                            }
                            if let Some(min) = channel.min_value() {
                                ui.label(format!("Min: {:.2}", min));
                            }
                            if let Some(max) = channel.max_value() {
                                ui.label(format!("Max: {:.2}", max));
                            }
                            if let Some(avg) = channel.avg_value() {
                                ui.label(format!("Avg: {:.2}", avg));
                            }
                            ui.label(format!("Points: {}", channel.data.len()));
                        });
                        
                        ui.add_space(8.0);
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_channel() {
        let mut channel = ChartChannel::new("Test", Color32::RED);
        channel.add_point(10.0);
        channel.add_point(20.0);
        channel.add_point(15.0);
        
        assert_eq!(channel.data.len(), 3);
        assert_eq!(channel.min_value(), Some(10.0));
        assert_eq!(channel.max_value(), Some(20.0));
        assert_eq!(channel.avg_value(), Some(15.0));
    }

    #[test]
    fn test_chart_parser() {
        let mut panel = ChartPanel::new();
        
        // Number parser
        panel.parse_data("123.45");
        assert!(panel.channels[0].data.len() == 1);
        
        // CSV parser
        panel.parser_type = ChartParserType::Csv;
        panel.csv_column = 1;
        panel.parse_data("timestamp,42.5,other");
        assert!(panel.channels[0].data.len() == 2);
    }
}





