//! # Termicon Core Library
//!
//! A professional multi-platform serial terminal library with support for:
//! - Serial ports (RS-232, RS-485, USB-Serial adapters)
//! - TCP/IP connections
//! - Telnet protocol
//! - SSH-2 (libssh2)
//! - Bluetooth (BLE/SPP)
//!
//! ## Features
//!
//! - Multiple simultaneous connections
//! - Hex/Text/Chart data views
//! - File transfer protocols (XMODEM, YMODEM, ZMODEM, Kermit)
//! - Logging with timestamps
//! - Scripting support (Lua)
//! - Plugin architecture
//! - Internationalization (i18n)
//! - CLI with exit codes and pipe support
//!
//! ## Example
//!
//! ```rust,no_run
//! use termicon_core::{Session, Transport, SerialConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = SerialConfig::new("COM3", 115200);
//!     let session = Session::connect(Transport::Serial(config)).await?;
//!     
//!     session.send(b"AT\r\n").await?;
//!     
//!     let mut rx = session.subscribe();
//!     while let Ok(event) = rx.recv().await {
//!         if let termicon_core::SessionEvent::DataReceived(data) = event {
//!             println!("Received: {:?}", data);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// Initialize i18n - load translations from i18n folder (TOML)
rust_i18n::i18n!("i18n", fallback = "en");

pub mod cli;
pub mod config;
pub mod core;
pub mod i18n;
pub mod utils;

// Re-exports for convenience
pub use crate::cli::{CliResult, ExitCodes, PipeMode, OutputFormat};
pub use crate::config::{AppConfig, ConnectionProfile};
pub use crate::core::codec::{Codec, CodecType};
pub use crate::core::logger::{LogEntry, Logger, LogFormat};
pub use crate::core::session::{Session, SessionEvent, SessionState};
pub use crate::core::transport::{
    SerialConfig, SerialFlowControl, SshAuth, SshConfig, TcpConfig, TelnetConfig, Transport,
    TransportType,
};
pub use crate::core::trigger::{Trigger, TriggerAction, TriggerCondition, TriggerManager, TriggerScope};
pub use crate::i18n::{get_locale, set_locale, t, Locale};
pub use crate::utils::autoconnect::{AutoConnect, AutoConnectConfig};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

