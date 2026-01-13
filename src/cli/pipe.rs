//! CLI Pipe Support
//!
//! Enables stdin/stdout piping for automation and scripting.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Pipe mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeMode {
    /// No piping, interactive mode
    Interactive,
    /// Read from stdin
    StdinOnly,
    /// Write to stdout
    StdoutOnly,
    /// Full pipe mode (stdin -> process -> stdout)
    Full,
}

impl PipeMode {
    /// Detect pipe mode from environment
    pub fn detect() -> Self {
        let stdin_is_tty = atty::is(atty::Stream::Stdin);
        let stdout_is_tty = atty::is(atty::Stream::Stdout);
        
        match (stdin_is_tty, stdout_is_tty) {
            (true, true) => Self::Interactive,
            (false, true) => Self::StdinOnly,
            (true, false) => Self::StdoutOnly,
            (false, false) => Self::Full,
        }
    }
    
    /// Is receiving from stdin?
    pub fn has_stdin(&self) -> bool {
        matches!(self, Self::StdinOnly | Self::Full)
    }
    
    /// Is sending to stdout?
    pub fn has_stdout(&self) -> bool {
        matches!(self, Self::StdoutOnly | Self::Full)
    }
    
    /// Is interactive?
    pub fn is_interactive(&self) -> bool {
        matches!(self, Self::Interactive)
    }
}

/// Pipe handler for stdin
pub struct StdinPipe {
    receiver: Receiver<Vec<u8>>,
    _thread: thread::JoinHandle<()>,
}

impl StdinPipe {
    /// Create new stdin pipe reader
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        let thread = thread::spawn(move || {
            Self::reader_thread(sender);
        });
        
        Self {
            receiver,
            _thread: thread,
        }
    }
    
    fn reader_thread(sender: Sender<Vec<u8>>) {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut buffer = [0u8; 4096];
        
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if sender.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    /// Try to receive data with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Option<Vec<u8>> {
        self.receiver.recv_timeout(timeout).ok()
    }
    
    /// Try to receive data (non-blocking)
    pub fn try_recv(&self) -> Option<Vec<u8>> {
        self.receiver.try_recv().ok()
    }
}

impl Default for StdinPipe {
    fn default() -> Self {
        Self::new()
    }
}

/// Line-based stdin reader
pub struct StdinLineReader {
    receiver: Receiver<String>,
    _thread: thread::JoinHandle<()>,
}

impl StdinLineReader {
    /// Create new line reader
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        let thread = thread::spawn(move || {
            Self::reader_thread(sender);
        });
        
        Self {
            receiver,
            _thread: thread,
        }
    }
    
    fn reader_thread(sender: Sender<String>) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    if sender.send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
    
    /// Try to receive line with timeout
    pub fn recv_line_timeout(&self, timeout: Duration) -> Option<String> {
        self.receiver.recv_timeout(timeout).ok()
    }
    
    /// Try to receive line (non-blocking)
    pub fn try_recv_line(&self) -> Option<String> {
        self.receiver.try_recv().ok()
    }
}

impl Default for StdinLineReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Stdout writer for pipe output
pub struct StdoutPipe {
    buffer: Vec<u8>,
    line_buffered: bool,
}

impl StdoutPipe {
    /// Create new stdout pipe
    pub fn new(line_buffered: bool) -> Self {
        Self {
            buffer: Vec::new(),
            line_buffered,
        }
    }
    
    /// Write data to stdout
    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if self.line_buffered {
            self.buffer.extend_from_slice(data);
            
            // Find newlines and flush
            while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
                let line = self.buffer.drain(..=pos).collect::<Vec<_>>();
                io::stdout().write_all(&line)?;
            }
        } else {
            io::stdout().write_all(data)?;
        }
        Ok(())
    }
    
    /// Write line to stdout
    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        writeln!(io::stdout(), "{}", line)
    }
    
    /// Flush output
    pub fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            io::stdout().write_all(&self.buffer)?;
            self.buffer.clear();
        }
        io::stdout().flush()
    }
}

impl Drop for StdoutPipe {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Output format for piped data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Raw binary data
    Raw,
    /// Hex dump
    Hex,
    /// One hex byte per line
    HexLines,
    /// Text with escape sequences shown
    Escaped,
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

/// Format data for pipe output
pub fn format_output(data: &[u8], format: OutputFormat) -> String {
    match format {
        OutputFormat::Raw => String::from_utf8_lossy(data).to_string(),
        OutputFormat::Hex => hex_format(data),
        OutputFormat::HexLines => hex_lines_format(data),
        OutputFormat::Escaped => escaped_format(data),
        OutputFormat::Json => json_format(data),
        OutputFormat::Csv => csv_format(data),
    }
}

fn hex_format(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
}

fn hex_lines_format(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("\n")
}

fn escaped_format(data: &[u8]) -> String {
    data.iter()
        .map(|&b| match b {
            0x00 => "\\0".to_string(),
            0x07 => "\\a".to_string(),
            0x08 => "\\b".to_string(),
            0x09 => "\\t".to_string(),
            0x0a => "\\n".to_string(),
            0x0d => "\\r".to_string(),
            0x1b => "\\e".to_string(),
            0x20..=0x7e => (b as char).to_string(),
            _ => format!("\\x{:02x}", b),
        })
        .collect()
}

fn json_format(data: &[u8]) -> String {
    let text = String::from_utf8_lossy(data);
    serde_json::json!({
        "data": text,
        "hex": hex_format(data),
        "length": data.len()
    })
    .to_string()
}

fn csv_format(data: &[u8]) -> String {
    // Output as offset,hex,char per byte
    data.iter()
        .enumerate()
        .map(|(i, &b)| {
            let ch = if b >= 0x20 && b <= 0x7e {
                (b as char).to_string()
            } else {
                ".".to_string()
            };
            format!("{},{:02x},{}", i, b, ch)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Pipe data processor for bidirectional communication
pub struct PipeProcessor {
    mode: PipeMode,
    stdin_pipe: Option<StdinPipe>,
    stdout_pipe: StdoutPipe,
    output_format: OutputFormat,
}

impl PipeProcessor {
    /// Create new pipe processor
    pub fn new(output_format: OutputFormat) -> Self {
        let mode = PipeMode::detect();
        
        let stdin_pipe = if mode.has_stdin() {
            Some(StdinPipe::new())
        } else {
            None
        };
        
        Self {
            mode,
            stdin_pipe,
            stdout_pipe: StdoutPipe::new(true),
            output_format,
        }
    }
    
    /// Get pipe mode
    pub fn mode(&self) -> PipeMode {
        self.mode
    }
    
    /// Try to read from stdin
    pub fn read(&self) -> Option<Vec<u8>> {
        self.stdin_pipe.as_ref()?.try_recv()
    }
    
    /// Read with timeout
    pub fn read_timeout(&self, timeout: Duration) -> Option<Vec<u8>> {
        self.stdin_pipe.as_ref()?.recv_timeout(timeout)
    }
    
    /// Write to stdout
    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if self.output_format == OutputFormat::Raw {
            self.stdout_pipe.write(data)
        } else {
            let formatted = format_output(data, self.output_format);
            self.stdout_pipe.write_line(&formatted)
        }
    }
    
    /// Flush output
    pub fn flush(&mut self) -> io::Result<()> {
        self.stdout_pipe.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_output_format_hex() {
        let data = &[0x48, 0x65, 0x6c, 0x6c, 0x6f];
        let hex = format_output(data, OutputFormat::Hex);
        assert_eq!(hex, "48 65 6c 6c 6f");
    }
    
    #[test]
    fn test_output_format_escaped() {
        let data = &[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x0a];
        let escaped = format_output(data, OutputFormat::Escaped);
        assert_eq!(escaped, "Hello\\n");
    }
    
    #[test]
    fn test_pipe_mode_detection() {
        // In tests, stdio is typically not a TTY
        let mode = PipeMode::detect();
        // Just ensure it returns a valid value
        let _ = mode.is_interactive();
    }
}




