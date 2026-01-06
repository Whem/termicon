//! Core module containing the main functionality of Termicon
//!
//! This module provides:
//! - Transport layer for different connection types (Serial, TCP, Telnet, SSH, Bluetooth)
//! - Session management with state machine
//! - Codec for data encoding/decoding
//! - Logger for data logging with timestamps
//! - Trigger system for automated responses
//! - Terminal emulation (VT100/VT220/ANSI)
//! - Real-time charting and data visualization
//! - Industrial protocol support (Modbus)
//! - Network bridging (Serial ↔ TCP)
//! - Virtual COM ports
//! - Session profiles
//! - File transfer (XMODEM/YMODEM/ZMODEM/Kermit)
//! - Macro recording and playback
//! - Transport capability registry
//! - Packet abstraction layer
//! - Protocol DSL (declarative protocol definitions)
//! - Session replay and recording
//! - Virtual device simulation
//! - Credential vault
//! - Deterministic session mode
//! - Fuzzing / robustness testing
//! - Routing graph
//! - Adaptive automation
//! - Resource arbitration
//! - Experiment / parameter sweep
//! - Explain mode / root cause hints
//! - Collaborative features
//! - External API (REST/WebSocket)
//! - Quick macros (M1-M24)
//! - Batch operations
//! - Workspace save/restore

pub mod adaptive;
pub mod arbitration;
pub mod batch;
pub mod bluetooth;
pub mod bridge;
pub mod capability;
pub mod chart;
pub mod codec;
pub mod collaborative;
pub mod deterministic;
pub mod experiment;
pub mod explain;
pub mod external_api;
pub mod file_transfer;
pub mod fuzzing;
pub mod knowledge;
pub mod logger;
pub mod macro_recorder;
pub mod macros;
pub mod packet;
pub mod plugin;
pub mod profile;
pub mod protocol;
pub mod protocol_dsl;
pub mod replay;
pub mod routing;
pub mod session;
pub mod simulator;
pub mod snippet;
pub mod state_machine;
pub mod terminal;
pub mod transfer;
pub mod transport;
pub mod trigger;
pub mod vault;
pub mod virtual_port;
pub mod workspace;

