//! Data parsing for chart visualization
//!
//! Supports multiple input formats: CSV, JSON, key=value, regex

use std::collections::HashMap;
use regex::Regex;

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Parser mode
    pub mode: ParserMode,
    /// Field delimiter for CSV
    pub delimiter: char,
    /// Column names (for CSV with headers)
    pub columns: Vec<String>,
    /// Skip first N values (e.g., timestamp column)
    pub skip_columns: usize,
    /// Key-value separator
    pub kv_separator: char,
    /// Pair separator for key-value
    pub pair_separator: char,
    /// Regex pattern for extraction
    pub regex_pattern: Option<String>,
    /// Channel name prefix
    pub channel_prefix: String,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            mode: ParserMode::Auto,
            delimiter: ',',
            columns: Vec::new(),
            skip_columns: 0,
            kv_separator: '=',
            pair_separator: ',',
            regex_pattern: None,
            channel_prefix: String::new(),
        }
    }
}

/// Parser mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserMode {
    /// Automatic detection
    Auto,
    /// CSV/comma-separated values
    Csv,
    /// JSON object
    Json,
    /// Key=value pairs
    KeyValue,
    /// Arduino Serial Plotter format (space-separated)
    ArduinoPlotter,
    /// Regex extraction
    Regex,
    /// Single numeric value
    SingleValue,
}

impl ParserMode {
    /// Get all modes
    pub fn all() -> &'static [ParserMode] {
        &[
            ParserMode::Auto,
            ParserMode::Csv,
            ParserMode::Json,
            ParserMode::KeyValue,
            ParserMode::ArduinoPlotter,
            ParserMode::Regex,
            ParserMode::SingleValue,
        ]
    }

    /// Get name
    pub fn name(&self) -> &'static str {
        match self {
            ParserMode::Auto => "Auto-detect",
            ParserMode::Csv => "CSV",
            ParserMode::Json => "JSON",
            ParserMode::KeyValue => "Key=Value",
            ParserMode::ArduinoPlotter => "Arduino Plotter",
            ParserMode::Regex => "Regex",
            ParserMode::SingleValue => "Single Value",
        }
    }
}

/// Data parser
pub struct DataParser {
    config: ParserConfig,
    compiled_regex: Option<Regex>,
    value_counter: usize,
}

impl Default for DataParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DataParser {
    /// Create new parser
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
            compiled_regex: None,
            value_counter: 0,
        }
    }

    /// Set configuration
    pub fn set_config(&mut self, config: ParserConfig) {
        // Compile regex if needed
        if let Some(ref pattern) = config.regex_pattern {
            self.compiled_regex = Regex::new(pattern).ok();
        } else {
            self.compiled_regex = None;
        }
        self.config = config;
    }

    /// Get configuration
    pub fn config(&self) -> &ParserConfig {
        &self.config
    }

    /// Parse a line of data
    pub fn parse_line(&mut self, line: &str) -> Option<Vec<(String, f64)>> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        let mode = if self.config.mode == ParserMode::Auto {
            Self::detect_mode(line)
        } else {
            self.config.mode
        };

        let values = match mode {
            ParserMode::Auto => self.parse_auto(line),
            ParserMode::Csv => self.parse_csv(line),
            ParserMode::Json => self.parse_json(line),
            ParserMode::KeyValue => self.parse_key_value(line),
            ParserMode::ArduinoPlotter => self.parse_arduino(line),
            ParserMode::Regex => self.parse_regex(line),
            ParserMode::SingleValue => self.parse_single(line),
        };

        // Apply prefix
        if !self.config.channel_prefix.is_empty() {
            values.map(|v| {
                v.into_iter()
                    .map(|(name, val)| (format!("{}{}", self.config.channel_prefix, name), val))
                    .collect()
            })
        } else {
            values
        }
    }

    /// Detect input format
    fn detect_mode(line: &str) -> ParserMode {
        let line = line.trim();

        // Check for JSON
        if line.starts_with('{') && line.ends_with('}') {
            return ParserMode::Json;
        }

        // Check for key=value
        if line.contains('=') && !line.contains(',') {
            return ParserMode::KeyValue;
        }

        // Check for key=value pairs with comma
        if line.contains('=') && line.contains(',') {
            return ParserMode::KeyValue;
        }

        // Check for CSV
        if line.contains(',') {
            return ParserMode::Csv;
        }

        // Check for space-separated (Arduino Plotter style)
        if line.contains(' ') || line.contains('\t') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.iter().all(|p| p.parse::<f64>().is_ok()) {
                return ParserMode::ArduinoPlotter;
            }
        }

        // Default to single value
        ParserMode::SingleValue
    }

    /// Parse with auto-detection
    fn parse_auto(&mut self, line: &str) -> Option<Vec<(String, f64)>> {
        let mode = Self::detect_mode(line);
        match mode {
            ParserMode::Csv => self.parse_csv(line),
            ParserMode::Json => self.parse_json(line),
            ParserMode::KeyValue => self.parse_key_value(line),
            ParserMode::ArduinoPlotter => self.parse_arduino(line),
            ParserMode::SingleValue => self.parse_single(line),
            _ => None,
        }
    }

    /// Parse CSV line
    fn parse_csv(&self, line: &str) -> Option<Vec<(String, f64)>> {
        let parts: Vec<&str> = line.split(self.config.delimiter).collect();
        let mut result = Vec::new();

        for (i, part) in parts.iter().enumerate().skip(self.config.skip_columns) {
            if let Ok(value) = part.trim().parse::<f64>() {
                let name = if i < self.config.columns.len() {
                    self.config.columns[i].clone()
                } else {
                    format!("col{}", i)
                };
                result.push((name, value));
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Parse JSON object
    fn parse_json(&self, line: &str) -> Option<Vec<(String, f64)>> {
        let obj: serde_json::Value = serde_json::from_str(line).ok()?;
        let map = obj.as_object()?;

        let mut result = Vec::new();
        for (key, value) in map {
            if let Some(num) = value.as_f64() {
                result.push((key.clone(), num));
            } else if let Some(num) = value.as_i64() {
                result.push((key.clone(), num as f64));
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Parse key=value pairs
    fn parse_key_value(&self, line: &str) -> Option<Vec<(String, f64)>> {
        let mut result = Vec::new();

        for pair in line.split(self.config.pair_separator) {
            let parts: Vec<&str> = pair.split(self.config.kv_separator).collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                if let Ok(value) = parts[1].trim().parse::<f64>() {
                    result.push((key.to_string(), value));
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Parse Arduino plotter format (space-separated numbers)
    fn parse_arduino(&self, line: &str) -> Option<Vec<(String, f64)>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut result = Vec::new();

        for (i, part) in parts.iter().enumerate().skip(self.config.skip_columns) {
            if let Ok(value) = part.parse::<f64>() {
                let name = if i < self.config.columns.len() {
                    self.config.columns[i].clone()
                } else {
                    format!("ch{}", i)
                };
                result.push((name, value));
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Parse using regex
    fn parse_regex(&self, line: &str) -> Option<Vec<(String, f64)>> {
        let regex = self.compiled_regex.as_ref()?;
        let caps = regex.captures(line)?;

        let mut result = Vec::new();

        for (i, name) in regex.capture_names().enumerate() {
            if i == 0 {
                continue; // Skip full match
            }

            if let Some(m) = caps.get(i) {
                if let Ok(value) = m.as_str().parse::<f64>() {
                    let channel_name = name
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| format!("group{}", i));
                    result.push((channel_name, value));
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Parse single numeric value
    fn parse_single(&mut self, line: &str) -> Option<Vec<(String, f64)>> {
        // Try to extract number from line
        let value: f64 = line
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == '+')
            .collect::<String>()
            .parse()
            .ok()?;

        self.value_counter += 1;
        Some(vec![("value".to_string(), value)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let mut parser = DataParser::new();
        let result = parser.parse_line("10.5,20.3,30.1").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].1, 10.5);
    }

    #[test]
    fn test_parse_json() {
        let mut parser = DataParser::new();
        let result = parser.parse_line(r#"{"temp": 25.5, "humidity": 60}"#).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_key_value() {
        let mut parser = DataParser::new();
        let result = parser.parse_line("temp=25.5,humidity=60").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_arduino() {
        let mut parser = DataParser::new();
        let result = parser.parse_line("100 200 300").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_detect_mode() {
        assert_eq!(DataParser::detect_mode(r#"{"a":1}"#), ParserMode::Json);
        assert_eq!(DataParser::detect_mode("a=1,b=2"), ParserMode::KeyValue);
        assert_eq!(DataParser::detect_mode("1,2,3"), ParserMode::Csv);
        assert_eq!(DataParser::detect_mode("1 2 3"), ParserMode::ArduinoPlotter);
        assert_eq!(DataParser::detect_mode("42"), ParserMode::SingleValue);
    }
}
