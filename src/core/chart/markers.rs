//! Chart Data Markers
//!
//! Provides markers for annotating data points on charts.
//! Markers can be used to highlight specific events, thresholds, or annotations.

use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Marker shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerShape {
    Circle,
    Square,
    Diamond,
    Triangle,
    TriangleDown,
    Cross,
    Plus,
    Star,
    Asterisk,
}

impl MarkerShape {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Circle,
            Self::Square,
            Self::Diamond,
            Self::Triangle,
            Self::TriangleDown,
            Self::Cross,
            Self::Plus,
            Self::Star,
            Self::Asterisk,
        ]
    }
    
    pub fn name(&self) -> &str {
        match self {
            Self::Circle => "Circle",
            Self::Square => "Square",
            Self::Diamond => "Diamond",
            Self::Triangle => "Triangle",
            Self::TriangleDown => "Triangle Down",
            Self::Cross => "Cross",
            Self::Plus => "Plus",
            Self::Star => "Star",
            Self::Asterisk => "Asterisk",
        }
    }
}

impl Default for MarkerShape {
    fn default() -> Self {
        Self::Circle
    }
}

/// Marker type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerType {
    /// Single point marker
    Point,
    /// Vertical line spanning chart height
    VerticalLine,
    /// Horizontal line spanning chart width
    HorizontalLine,
    /// Region (filled area between two values)
    Region,
    /// Text annotation
    Annotation,
}

impl Default for MarkerType {
    fn default() -> Self {
        Self::Point
    }
}

/// Color definition
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MarkerColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl MarkerColor {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn red() -> Self { Self::rgb(255, 0, 0) }
    pub fn green() -> Self { Self::rgb(0, 255, 0) }
    pub fn blue() -> Self { Self::rgb(0, 0, 255) }
    pub fn yellow() -> Self { Self::rgb(255, 255, 0) }
    pub fn orange() -> Self { Self::rgb(255, 165, 0) }
    pub fn purple() -> Self { Self::rgb(128, 0, 128) }
    pub fn cyan() -> Self { Self::rgb(0, 255, 255) }
    pub fn white() -> Self { Self::rgb(255, 255, 255) }
    
    pub fn to_rgba32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }
}

impl Default for MarkerColor {
    fn default() -> Self {
        Self::red()
    }
}

/// Data marker definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMarker {
    /// Unique marker ID
    pub id: String,
    /// Marker name/label
    pub name: String,
    /// Marker type
    pub marker_type: MarkerType,
    /// Shape (for point markers)
    pub shape: MarkerShape,
    /// Color
    pub color: MarkerColor,
    /// Size in pixels
    pub size: f32,
    /// X position (time in milliseconds from start, or data index)
    pub x: f64,
    /// Y position (data value, for point/horizontal markers)
    pub y: Option<f64>,
    /// End X position (for regions)
    pub x_end: Option<f64>,
    /// End Y position (for regions)
    pub y_end: Option<f64>,
    /// Channel/series index (-1 for all)
    pub channel: i32,
    /// Annotation text
    pub text: String,
    /// Visible
    pub visible: bool,
    /// Creation timestamp
    #[serde(skip)]
    pub created_at: Option<Instant>,
}

impl DataMarker {
    /// Create a point marker
    pub fn point(id: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            marker_type: MarkerType::Point,
            shape: MarkerShape::Circle,
            color: MarkerColor::red(),
            size: 8.0,
            x,
            y: Some(y),
            x_end: None,
            y_end: None,
            channel: -1,
            text: String::new(),
            visible: true,
            created_at: Some(Instant::now()),
        }
    }
    
    /// Create a vertical line marker
    pub fn vertical_line(id: impl Into<String>, x: f64) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            marker_type: MarkerType::VerticalLine,
            shape: MarkerShape::Circle,
            color: MarkerColor::blue(),
            size: 2.0,
            x,
            y: None,
            x_end: None,
            y_end: None,
            channel: -1,
            text: String::new(),
            visible: true,
            created_at: Some(Instant::now()),
        }
    }
    
    /// Create a horizontal line marker
    pub fn horizontal_line(id: impl Into<String>, y: f64) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            marker_type: MarkerType::HorizontalLine,
            shape: MarkerShape::Circle,
            color: MarkerColor::green(),
            size: 2.0,
            x: 0.0,
            y: Some(y),
            x_end: None,
            y_end: None,
            channel: -1,
            text: String::new(),
            visible: true,
            created_at: Some(Instant::now()),
        }
    }
    
    /// Create a region marker
    pub fn region(id: impl Into<String>, x_start: f64, x_end: f64, y_start: f64, y_end: f64) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            marker_type: MarkerType::Region,
            shape: MarkerShape::Square,
            color: MarkerColor::rgba(255, 255, 0, 64),
            size: 0.0,
            x: x_start,
            y: Some(y_start),
            x_end: Some(x_end),
            y_end: Some(y_end),
            channel: -1,
            text: String::new(),
            visible: true,
            created_at: Some(Instant::now()),
        }
    }
    
    /// Create an annotation marker
    pub fn annotation(id: impl Into<String>, x: f64, y: f64, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            marker_type: MarkerType::Annotation,
            shape: MarkerShape::Circle,
            color: MarkerColor::white(),
            size: 12.0,
            x,
            y: Some(y),
            x_end: None,
            y_end: None,
            channel: -1,
            text: text.into(),
            visible: true,
            created_at: Some(Instant::now()),
        }
    }
    
    /// Set marker name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    
    /// Set marker color
    pub fn with_color(mut self, color: MarkerColor) -> Self {
        self.color = color;
        self
    }
    
    /// Set marker shape
    pub fn with_shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }
    
    /// Set marker size
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    
    /// Set channel
    pub fn for_channel(mut self, channel: i32) -> Self {
        self.channel = channel;
        self
    }
}

/// Marker manager
#[derive(Debug, Default)]
pub struct MarkerManager {
    markers: Vec<DataMarker>,
    next_id: usize,
}

impl MarkerManager {
    pub fn new() -> Self {
        Self {
            markers: Vec::new(),
            next_id: 0,
        }
    }
    
    /// Generate unique marker ID
    fn generate_id(&mut self) -> String {
        let id = format!("marker_{}", self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Add a marker
    pub fn add(&mut self, mut marker: DataMarker) {
        if marker.id.is_empty() {
            marker.id = self.generate_id();
        }
        self.markers.push(marker);
    }
    
    /// Add point marker
    pub fn add_point(&mut self, x: f64, y: f64) -> &DataMarker {
        let id = self.generate_id();
        let marker = DataMarker::point(&id, x, y);
        self.markers.push(marker);
        self.markers.last().unwrap()
    }
    
    /// Add vertical line marker
    pub fn add_vertical_line(&mut self, x: f64) -> &DataMarker {
        let id = self.generate_id();
        let marker = DataMarker::vertical_line(&id, x);
        self.markers.push(marker);
        self.markers.last().unwrap()
    }
    
    /// Add horizontal line (threshold) marker
    pub fn add_threshold(&mut self, y: f64, name: &str) -> &DataMarker {
        let id = self.generate_id();
        let marker = DataMarker::horizontal_line(&id, y).with_name(name);
        self.markers.push(marker);
        self.markers.last().unwrap()
    }
    
    /// Add annotation marker
    pub fn add_annotation(&mut self, x: f64, y: f64, text: &str) -> &DataMarker {
        let id = self.generate_id();
        let marker = DataMarker::annotation(&id, x, y, text);
        self.markers.push(marker);
        self.markers.last().unwrap()
    }
    
    /// Remove marker by ID
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.markers.iter().position(|m| m.id == id) {
            self.markers.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Clear all markers
    pub fn clear(&mut self) {
        self.markers.clear();
    }
    
    /// Get all markers
    pub fn all(&self) -> &[DataMarker] {
        &self.markers
    }
    
    /// Get visible markers
    pub fn visible(&self) -> Vec<&DataMarker> {
        self.markers.iter().filter(|m| m.visible).collect()
    }
    
    /// Get markers in time range
    pub fn in_range(&self, x_min: f64, x_max: f64) -> Vec<&DataMarker> {
        self.markers.iter().filter(|m| {
            m.visible && m.x >= x_min && m.x <= x_max
        }).collect()
    }
    
    /// Get marker by ID
    pub fn get(&self, id: &str) -> Option<&DataMarker> {
        self.markers.iter().find(|m| m.id == id)
    }
    
    /// Get mutable marker by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut DataMarker> {
        self.markers.iter_mut().find(|m| m.id == id)
    }
    
    /// Toggle marker visibility
    pub fn toggle_visibility(&mut self, id: &str) -> bool {
        if let Some(marker) = self.get_mut(id) {
            marker.visible = !marker.visible;
            marker.visible
        } else {
            false
        }
    }
    
    /// Count markers
    pub fn count(&self) -> usize {
        self.markers.len()
    }
    
    /// Export markers to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.markers)
    }
    
    /// Import markers from JSON
    pub fn import_json(&mut self, json: &str) -> Result<usize, serde_json::Error> {
        let markers: Vec<DataMarker> = serde_json::from_str(json)?;
        let count = markers.len();
        for marker in markers {
            self.add(marker);
        }
        Ok(count)
    }
}

/// Predefined marker presets
pub mod presets {
    use super::*;
    
    /// Create min value marker
    pub fn min_value(x: f64, y: f64) -> DataMarker {
        DataMarker::point("min", x, y)
            .with_name("Min")
            .with_color(MarkerColor::blue())
            .with_shape(MarkerShape::TriangleDown)
    }
    
    /// Create max value marker
    pub fn max_value(x: f64, y: f64) -> DataMarker {
        DataMarker::point("max", x, y)
            .with_name("Max")
            .with_color(MarkerColor::red())
            .with_shape(MarkerShape::Triangle)
    }
    
    /// Create threshold marker
    pub fn threshold(value: f64, name: &str, color: MarkerColor) -> DataMarker {
        DataMarker::horizontal_line("threshold", value)
            .with_name(name)
            .with_color(color)
    }
    
    /// Create event marker
    pub fn event(x: f64, name: &str) -> DataMarker {
        DataMarker::vertical_line("event", x)
            .with_name(name)
            .with_color(MarkerColor::orange())
    }
    
    /// Create error region marker
    pub fn error_region(x_start: f64, x_end: f64) -> DataMarker {
        DataMarker::region("error", x_start, x_end, f64::MIN, f64::MAX)
            .with_name("Error")
            .with_color(MarkerColor::rgba(255, 0, 0, 32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_marker_creation() {
        let marker = DataMarker::point("test", 10.0, 20.0)
            .with_name("Test Point")
            .with_color(MarkerColor::green());
        
        assert_eq!(marker.id, "test");
        assert_eq!(marker.name, "Test Point");
        assert_eq!(marker.x, 10.0);
        assert_eq!(marker.y, Some(20.0));
    }
    
    #[test]
    fn test_marker_manager() {
        let mut manager = MarkerManager::new();
        
        manager.add_point(1.0, 2.0);
        manager.add_threshold(50.0, "Upper Limit");
        manager.add_annotation(3.0, 4.0, "Note");
        
        assert_eq!(manager.count(), 3);
        
        let visible = manager.visible();
        assert_eq!(visible.len(), 3);
    }
    
    #[test]
    fn test_marker_json() {
        let mut manager = MarkerManager::new();
        manager.add_point(1.0, 2.0);
        manager.add_threshold(50.0, "Limit");
        
        let json = manager.export_json().unwrap();
        assert!(json.contains("marker_0"));
        
        let mut new_manager = MarkerManager::new();
        let count = new_manager.import_json(&json).unwrap();
        assert_eq!(count, 2);
    }
}




