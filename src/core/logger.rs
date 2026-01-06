//! Session logging functionality
//!
//! Supports multiple output formats and real-time logging

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{DateTime, Local};
use parking_lot::Mutex;

/// Type alias for backwards compatibility
pub type Logger = Arc<Mutex<SessionLogger>>;

/// Log format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum LogFormat {
    /// Plain text
    #[default]
    Text,
    /// Hex dump
    Hex,
    /// Mixed (text + hex for non-printable)
    Mixed,
    /// CSV with timestamp
    Csv,
    /// Raw binary
    Raw,
    /// JSON lines
    JsonLines,
}

impl LogFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            LogFormat::Text => "txt",
            LogFormat::Hex => "hex",
            LogFormat::Mixed => "txt",
            LogFormat::Csv => "csv",
            LogFormat::Raw => "bin",
            LogFormat::JsonLines => "jsonl",
        }
    }

    /// Get all formats
    pub fn all() -> &'static [LogFormat] {
        &[
            LogFormat::Text,
            LogFormat::Hex,
            LogFormat::Mixed,
            LogFormat::Csv,
            LogFormat::Raw,
            LogFormat::JsonLines,
        ]
    }

    /// Get name
    pub fn name(&self) -> &'static str {
        match self {
            LogFormat::Text => "Text",
            LogFormat::Hex => "Hex Dump",
            LogFormat::Mixed => "Mixed",
            LogFormat::Csv => "CSV",
            LogFormat::Raw => "Raw Binary",
            LogFormat::JsonLines => "JSON Lines",
        }
    }
}

/// Data direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    Received,
    Sent,
    Info,
}

/// A single log entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub direction: Direction,
    pub data: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl LogEntry {
    /// Create new entry
    pub fn new(direction: Direction, data: Vec<u8>) -> Self {
        Self {
            timestamp: Local::now(),
            direction,
            data,
            note: None,
        }
    }

    /// Create with note
    pub fn with_note(direction: Direction, data: Vec<u8>, note: &str) -> Self {
        Self {
            timestamp: Local::now(),
            direction,
            data,
            note: Some(note.to_string()),
        }
    }

    /// Format as text
    pub fn to_text(&self, show_timestamp: bool) -> String {
        let dir = match self.direction {
            Direction::Received => "RX",
            Direction::Sent => "TX",
            Direction::Info => "##",
        };

        let text = String::from_utf8_lossy(&self.data);

        if show_timestamp {
            format!(
                "[{}] {} {}",
                self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                dir,
                text
            )
        } else {
            format!("{} {}", dir, text)
        }
    }

    /// Format as hex
    pub fn to_hex(&self, show_timestamp: bool) -> String {
        let dir = match self.direction {
            Direction::Received => "RX",
            Direction::Sent => "TX",
            Direction::Info => "##",
        };

        let hex: String = self.data.iter().map(|b| format!("{:02X} ", b)).collect();

        if show_timestamp {
            format!(
                "[{}] {} {}",
                self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                dir,
                hex
            )
        } else {
            format!("{} {}", dir, hex)
        }
    }

    /// Format as CSV
    pub fn to_csv(&self) -> String {
        let dir = match self.direction {
            Direction::Received => "RX",
            Direction::Sent => "TX",
            Direction::Info => "INFO",
        };

        let hex: String = self.data.iter().map(|b| format!("{:02X}", b)).collect();
        let text = String::from_utf8_lossy(&self.data).replace("\"", "\"\"");

        format!(
            "\"{}\",\"{}\",\"{}\",\"{}\"",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            dir,
            hex,
            text
        )
    }

    /// Format as JSON line
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Session logger
pub struct SessionLogger {
    /// Output file
    file: Option<BufWriter<File>>,
    /// Log format
    format: LogFormat,
    /// Log file path
    path: Option<PathBuf>,
    /// Include timestamps
    timestamps: bool,
    /// Buffer for in-memory log
    buffer: Vec<LogEntry>,
    /// Max buffer size
    max_buffer: usize,
    /// Bytes logged
    bytes_logged: usize,
    /// Lines logged
    lines_logged: usize,
}

impl Default for SessionLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionLogger {
    /// Create new logger (not logging to file yet)
    pub fn new() -> Self {
        Self {
            file: None,
            format: LogFormat::Text,
            path: None,
            timestamps: true,
            buffer: Vec::new(),
            max_buffer: 10000,
            bytes_logged: 0,
            lines_logged: 0,
        }
    }

    /// Start logging to file
    pub fn start(&mut self, path: PathBuf, format: LogFormat) -> Result<(), String> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        let mut writer = BufWriter::new(file);

        // Write header for CSV
        if format == LogFormat::Csv {
            writeln!(writer, "Timestamp,Direction,Hex,Text")
                .map_err(|e| format!("Failed to write header: {}", e))?;
        }

        self.file = Some(writer);
        self.format = format;
        self.path = Some(path);
        self.bytes_logged = 0;
        self.lines_logged = 0;

        Ok(())
    }

    /// Stop logging
    pub fn stop(&mut self) {
        if let Some(ref mut file) = self.file {
            let _ = file.flush();
        }
        self.file = None;
    }

    /// Is currently logging
    pub fn is_logging(&self) -> bool {
        self.file.is_some()
    }

    /// Get log path
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// Log data
    pub fn log(&mut self, direction: Direction, data: &[u8]) {
        let entry = LogEntry::new(direction, data.to_vec());

        // Write to file if logging
        if let Some(ref mut file) = self.file {
            let line = match self.format {
                LogFormat::Text => entry.to_text(self.timestamps),
                LogFormat::Hex => entry.to_hex(self.timestamps),
                LogFormat::Mixed => {
                    if data.iter().all(|&b| b >= 32 && b < 127 || b == b'\n' || b == b'\r' || b == b'\t') {
                        entry.to_text(self.timestamps)
                    } else {
                        entry.to_hex(self.timestamps)
                    }
                }
                LogFormat::Csv => entry.to_csv(),
                LogFormat::Raw => {
                    // Write raw bytes
                    let _ = file.write_all(data);
                    self.bytes_logged += data.len();
                    return;
                }
                LogFormat::JsonLines => entry.to_json(),
            };

            let _ = writeln!(file, "{}", line);
            self.bytes_logged += data.len();
            self.lines_logged += 1;

            // Flush periodically
            if self.lines_logged % 100 == 0 {
                let _ = file.flush();
            }
        }

        // Add to buffer
        self.buffer.push(entry);
        if self.buffer.len() > self.max_buffer {
            self.buffer.remove(0);
        }
    }

    /// Log received data
    pub fn log_rx(&mut self, data: &[u8]) {
        self.log(Direction::Received, data);
    }

    /// Log sent data
    pub fn log_tx(&mut self, data: &[u8]) {
        self.log(Direction::Sent, data);
    }

    /// Log info message
    pub fn log_info(&mut self, message: &str) {
        self.log(Direction::Info, message.as_bytes());
    }

    /// Get buffer
    pub fn buffer(&self) -> &[LogEntry] {
        &self.buffer
    }

    /// Clear buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.bytes_logged, self.lines_logged)
    }

    /// Export buffer to string
    pub fn export_buffer(&self, format: LogFormat) -> String {
        let mut result = String::new();

        if format == LogFormat::Csv {
            result.push_str("Timestamp,Direction,Hex,Text\n");
        }

        for entry in &self.buffer {
            let line = match format {
                LogFormat::Text => entry.to_text(true),
                LogFormat::Hex => entry.to_hex(true),
                LogFormat::Mixed => {
                    if entry.data.iter().all(|&b| b >= 32 && b < 127 || b == b'\n' || b == b'\r' || b == b'\t') {
                        entry.to_text(true)
                    } else {
                        entry.to_hex(true)
                    }
                }
                LogFormat::Csv => entry.to_csv(),
                LogFormat::Raw => hex::encode(&entry.data),
                LogFormat::JsonLines => entry.to_json(),
            };
            result.push_str(&line);
            result.push('\n');
        }

        result
    }

    /// Set timestamp display
    pub fn set_timestamps(&mut self, show: bool) {
        self.timestamps = show;
    }

    /// Set max buffer size
    pub fn set_max_buffer(&mut self, size: usize) {
        self.max_buffer = size;
    }

    /// Flush to disk
    pub fn flush(&mut self) {
        if let Some(ref mut file) = self.file {
            let _ = file.flush();
        }
    }
}

impl Drop for SessionLogger {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Generate log filename with timestamp
pub fn generate_log_filename(prefix: &str, format: LogFormat) -> String {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    format!("{}_{}.{}", prefix, timestamp, format.extension())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_text() {
        let entry = LogEntry::new(Direction::Received, b"Hello".to_vec());
        let text = entry.to_text(false);
        assert!(text.contains("RX"));
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_log_entry_hex() {
        let entry = LogEntry::new(Direction::Sent, vec![0x01, 0x02, 0x03]);
        let hex = entry.to_hex(false);
        assert!(hex.contains("TX"));
        assert!(hex.contains("01 02 03"));
    }

    #[test]
    fn test_buffer_limit() {
        let mut logger = SessionLogger::new();
        logger.set_max_buffer(5);

        for i in 0..10 {
            logger.log_rx(&[i]);
        }

        assert_eq!(logger.buffer().len(), 5);
    }
}
