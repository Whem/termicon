//! CLI Exit Codes
//!
//! Standard exit codes for CLI operations and automation.

use std::process::ExitCode;

/// Exit code constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCodes;

impl ExitCodes {
    /// Success
    pub const SUCCESS: u8 = 0;
    
    /// General error
    pub const ERROR: u8 = 1;
    
    /// Invalid arguments
    pub const INVALID_ARGS: u8 = 2;
    
    /// Connection failed
    pub const CONNECTION_FAILED: u8 = 3;
    
    /// Connection timeout
    pub const TIMEOUT: u8 = 4;
    
    /// Authentication failed
    pub const AUTH_FAILED: u8 = 5;
    
    /// File not found
    pub const FILE_NOT_FOUND: u8 = 6;
    
    /// Permission denied
    pub const PERMISSION_DENIED: u8 = 7;
    
    /// Configuration error
    pub const CONFIG_ERROR: u8 = 8;
    
    /// Protocol error
    pub const PROTOCOL_ERROR: u8 = 9;
    
    /// Transfer failed
    pub const TRANSFER_FAILED: u8 = 10;
    
    /// User cancelled
    pub const CANCELLED: u8 = 11;
    
    /// Device not found
    pub const DEVICE_NOT_FOUND: u8 = 12;
    
    /// Device busy
    pub const DEVICE_BUSY: u8 = 13;
    
    /// Port not found
    pub const PORT_NOT_FOUND: u8 = 14;
    
    /// Script error
    pub const SCRIPT_ERROR: u8 = 15;
    
    /// Pattern match failed
    pub const PATTERN_NOT_FOUND: u8 = 16;
    
    /// Data validation failed
    pub const VALIDATION_FAILED: u8 = 17;
    
    /// Internal error
    pub const INTERNAL_ERROR: u8 = 127;
}

/// CLI operation result
#[derive(Debug)]
pub enum CliResult {
    /// Success with optional message
    Success(Option<String>),
    
    /// Error with code and message
    Error(u8, String),
}

impl CliResult {
    pub fn success() -> Self {
        Self::Success(None)
    }
    
    pub fn success_with_message(msg: impl Into<String>) -> Self {
        Self::Success(Some(msg.into()))
    }
    
    pub fn error(code: u8, msg: impl Into<String>) -> Self {
        Self::Error(code, msg.into())
    }
    
    pub fn connection_failed(msg: impl Into<String>) -> Self {
        Self::Error(ExitCodes::CONNECTION_FAILED, msg.into())
    }
    
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Error(ExitCodes::TIMEOUT, msg.into())
    }
    
    pub fn auth_failed(msg: impl Into<String>) -> Self {
        Self::Error(ExitCodes::AUTH_FAILED, msg.into())
    }
    
    pub fn file_not_found(path: &str) -> Self {
        Self::Error(ExitCodes::FILE_NOT_FOUND, format!("File not found: {}", path))
    }
    
    pub fn port_not_found(port: &str) -> Self {
        Self::Error(ExitCodes::PORT_NOT_FOUND, format!("Port not found: {}", port))
    }
    
    /// Get exit code
    pub fn code(&self) -> u8 {
        match self {
            Self::Success(_) => ExitCodes::SUCCESS,
            Self::Error(code, _) => *code,
        }
    }
    
    /// Get message
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Success(Some(msg)) => Some(msg),
            Self::Error(_, msg) => Some(msg),
            _ => None,
        }
    }
    
    /// Convert to ExitCode
    pub fn to_exit_code(&self) -> ExitCode {
        ExitCode::from(self.code())
    }
    
    /// Is success?
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }
}

impl From<std::io::Error> for CliResult {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind;
        
        let code = match err.kind() {
            ErrorKind::NotFound => ExitCodes::FILE_NOT_FOUND,
            ErrorKind::PermissionDenied => ExitCodes::PERMISSION_DENIED,
            ErrorKind::ConnectionRefused => ExitCodes::CONNECTION_FAILED,
            ErrorKind::TimedOut => ExitCodes::TIMEOUT,
            _ => ExitCodes::ERROR,
        };
        
        Self::Error(code, err.to_string())
    }
}

/// Exit code description
pub fn exit_code_description(code: u8) -> &'static str {
    match code {
        0 => "Success",
        1 => "General error",
        2 => "Invalid arguments",
        3 => "Connection failed",
        4 => "Connection timeout",
        5 => "Authentication failed",
        6 => "File not found",
        7 => "Permission denied",
        8 => "Configuration error",
        9 => "Protocol error",
        10 => "Transfer failed",
        11 => "Operation cancelled",
        12 => "Device not found",
        13 => "Device busy",
        14 => "Port not found",
        15 => "Script error",
        16 => "Pattern not found",
        17 => "Validation failed",
        127 => "Internal error",
        _ => "Unknown error",
    }
}

/// Print exit code table
pub fn print_exit_codes() {
    println!("Exit Codes:");
    println!("  {:>3}  {}", 0, exit_code_description(0));
    for code in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 127] {
        println!("  {:>3}  {}", code, exit_code_description(code));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_result() {
        let success = CliResult::success();
        assert!(success.is_success());
        assert_eq!(success.code(), 0);
        
        let error = CliResult::error(3, "Connection failed");
        assert!(!error.is_success());
        assert_eq!(error.code(), 3);
        assert_eq!(error.message(), Some("Connection failed"));
    }
    
    #[test]
    fn test_from_io_error() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let result = CliResult::from(err);
        assert_eq!(result.code(), ExitCodes::FILE_NOT_FOUND);
    }
}


