//! CLI Module
//!
//! Provides command-line interface functionality including:
//! - Exit codes for automation
//! - Pipe support for stdin/stdout

pub mod exit_codes;
pub mod pipe;

pub use exit_codes::{ExitCodes, CliResult, exit_code_description, print_exit_codes};
pub use pipe::{PipeMode, StdinPipe, StdinLineReader, StdoutPipe, PipeProcessor, OutputFormat, format_output};


