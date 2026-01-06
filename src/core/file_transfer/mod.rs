//! File Transfer Protocols
//!
//! Provides implementations for various file transfer protocols:
//! - XMODEM (128-byte blocks with checksum)
//! - XMODEM-CRC (128-byte blocks with CRC-16)
//! - XMODEM-1K (1024-byte blocks)
//! - YMODEM (batch file transfer)
//! - ZMODEM (streaming with auto-resume)
//! - Kermit (robust, extensible)

pub mod kermit;

pub use kermit::{
    Kermit, KermitConfig, KermitPacket, KermitState, KermitError, PacketType,
};


