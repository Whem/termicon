//! Data codec module for encoding/decoding data
//!
//! Supports different display formats:
//! - Text (ASCII/UTF-8)
//! - Hexadecimal
//! - Binary
//! - Mixed (hex + text)

mod hex;
mod text;

pub use self::hex::HexCodec;
pub use text::TextCodec;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Codec type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CodecType {
    /// Plain text (ASCII/UTF-8)
    #[default]
    Text,
    /// Hexadecimal display
    Hex,
    /// Mixed hex and text
    Mixed,
    /// Binary display
    Binary,
}

/// Codec trait for data transformation
pub trait Codec: Send + Sync {
    /// Encode bytes to display string
    fn encode(&self, data: &[u8]) -> String;

    /// Decode display string to bytes
    fn decode(&self, text: &str) -> Result<Bytes, CodecError>;

    /// Get codec type
    fn codec_type(&self) -> CodecType;
}

/// Codec errors
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// Invalid input format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Invalid character
    #[error("Invalid character at position {0}: {1}")]
    InvalidCharacter(usize, char),

    /// UTF-8 error
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Create a codec from type
pub fn create_codec(codec_type: CodecType) -> Box<dyn Codec> {
    match codec_type {
        CodecType::Text => Box::new(TextCodec::new()),
        CodecType::Hex => Box::new(HexCodec::new()),
        CodecType::Mixed => Box::new(HexCodec::mixed()),
        CodecType::Binary => Box::new(HexCodec::binary()),
    }
}

/// Format bytes as a hexdump (like xxd)
pub fn hexdump(data: &[u8], bytes_per_line: usize) -> String {
    let mut output = String::new();
    
    for (offset, chunk) in data.chunks(bytes_per_line).enumerate() {
        // Offset
        output.push_str(&format!("{:08x}  ", offset * bytes_per_line));
        
        // Hex bytes
        for (i, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02x} ", byte));
            if i == bytes_per_line / 2 - 1 {
                output.push(' ');
            }
        }
        
        // Padding for incomplete lines
        if chunk.len() < bytes_per_line {
            let missing = bytes_per_line - chunk.len();
            for i in 0..missing {
                output.push_str("   ");
                if chunk.len() + i == bytes_per_line / 2 - 1 {
                    output.push(' ');
                }
            }
        }
        
        output.push(' ');
        
        // ASCII representation
        output.push('|');
        for byte in chunk {
            if byte.is_ascii_graphic() || *byte == b' ' {
                output.push(*byte as char);
            } else {
                output.push('.');
            }
        }
        for _ in chunk.len()..bytes_per_line {
            output.push(' ');
        }
        output.push('|');
        output.push('\n');
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hexdump() {
        let data = b"Hello, World!";
        let dump = hexdump(data, 16);
        assert!(dump.contains("48 65 6c 6c 6f"));
        assert!(dump.contains("|Hello, World!"));
    }
}


