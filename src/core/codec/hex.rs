//! Hexadecimal codec for binary data display

use super::{Codec, CodecError, CodecType};
use bytes::Bytes;

/// Hex display format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HexFormat {
    /// Uppercase hex (e.g., "48 65 6C 6C 6F")
    #[default]
    Upper,
    /// Lowercase hex (e.g., "48 65 6c 6c 6f")
    Lower,
    /// No spaces (e.g., "48656C6C6F")
    Compact,
    /// C-style (e.g., "0x48, 0x65, 0x6C")
    CStyle,
}

/// Hex codec configuration
#[derive(Debug, Clone)]
pub struct HexCodecConfig {
    /// Hex format
    pub format: HexFormat,
    /// Bytes per line (0 = no line breaks)
    pub bytes_per_line: usize,
    /// Show ASCII sidebar
    pub show_ascii: bool,
    /// Show line offsets
    pub show_offsets: bool,
    /// Group size (bytes before extra space)
    pub group_size: usize,
}

impl Default for HexCodecConfig {
    fn default() -> Self {
        Self {
            format: HexFormat::Upper,
            bytes_per_line: 16,
            show_ascii: false,
            show_offsets: false,
            group_size: 0,
        }
    }
}

/// Hex codec for binary data display
pub struct HexCodec {
    config: HexCodecConfig,
    codec_type: CodecType,
}

impl HexCodec {
    /// Create a new hex codec with default config
    pub fn new() -> Self {
        Self {
            config: HexCodecConfig::default(),
            codec_type: CodecType::Hex,
        }
    }

    /// Create a mixed (hex + ASCII) codec
    pub fn mixed() -> Self {
        Self {
            config: HexCodecConfig {
                show_ascii: true,
                show_offsets: true,
                ..Default::default()
            },
            codec_type: CodecType::Mixed,
        }
    }

    /// Create a binary display codec
    pub fn binary() -> Self {
        Self {
            config: HexCodecConfig {
                bytes_per_line: 8,
                group_size: 1,
                ..Default::default()
            },
            codec_type: CodecType::Binary,
        }
    }

    /// Create with custom config
    pub fn with_config(config: HexCodecConfig) -> Self {
        Self {
            config,
            codec_type: CodecType::Hex,
        }
    }

    /// Set format
    #[must_use]
    pub fn format(mut self, format: HexFormat) -> Self {
        self.config.format = format;
        self
    }

    /// Set bytes per line
    #[must_use]
    pub fn bytes_per_line(mut self, n: usize) -> Self {
        self.config.bytes_per_line = n;
        self
    }

    /// Enable ASCII sidebar
    #[must_use]
    pub fn show_ascii(mut self, show: bool) -> Self {
        self.config.show_ascii = show;
        self
    }

    /// Enable line offsets
    #[must_use]
    pub fn show_offsets(mut self, show: bool) -> Self {
        self.config.show_offsets = show;
        self
    }

    fn format_byte(&self, byte: u8) -> String {
        match self.config.format {
            HexFormat::Upper => format!("{:02X}", byte),
            HexFormat::Lower => format!("{:02x}", byte),
            HexFormat::Compact => format!("{:02X}", byte),
            HexFormat::CStyle => format!("0x{:02X}", byte),
        }
    }

    fn format_binary(byte: u8) -> String {
        format!("{:08b}", byte)
    }
}

impl Default for HexCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec for HexCodec {
    fn encode(&self, data: &[u8]) -> String {
        if data.is_empty() {
            return String::new();
        }

        let mut output = String::new();

        if self.config.bytes_per_line > 0 {
            // Multi-line format
            for (line_idx, chunk) in data.chunks(self.config.bytes_per_line).enumerate() {
                // Offset
                if self.config.show_offsets {
                    output.push_str(&format!("{:08X}  ", line_idx * self.config.bytes_per_line));
                }

                // Hex bytes
                if self.codec_type == CodecType::Binary {
                    for (i, &byte) in chunk.iter().enumerate() {
                        if i > 0 {
                            output.push(' ');
                        }
                        output.push_str(&Self::format_binary(byte));
                    }
                } else {
                    let separator = match self.config.format {
                        HexFormat::Compact => "",
                        HexFormat::CStyle => ", ",
                        _ => " ",
                    };

                    for (i, &byte) in chunk.iter().enumerate() {
                        if i > 0 {
                            output.push_str(separator);
                            if self.config.group_size > 0 && i % self.config.group_size == 0 {
                                output.push(' ');
                            }
                        }
                        output.push_str(&self.format_byte(byte));
                    }

                    // Padding for ASCII alignment
                    if self.config.show_ascii && chunk.len() < self.config.bytes_per_line {
                        let missing = self.config.bytes_per_line - chunk.len();
                        for _ in 0..missing {
                            output.push_str("   ");
                        }
                    }
                }

                // ASCII sidebar
                if self.config.show_ascii {
                    output.push_str("  |");
                    for &byte in chunk {
                        if byte.is_ascii_graphic() || byte == b' ' {
                            output.push(byte as char);
                        } else {
                            output.push('.');
                        }
                    }
                    output.push('|');
                }

                output.push('\n');
            }
        } else {
            // Single line format
            let separator = match self.config.format {
                HexFormat::Compact => "",
                HexFormat::CStyle => ", ",
                _ => " ",
            };

            for (i, &byte) in data.iter().enumerate() {
                if i > 0 {
                    output.push_str(separator);
                }
                output.push_str(&self.format_byte(byte));
            }
        }

        output
    }

    fn decode(&self, text: &str) -> Result<Bytes, CodecError> {
        let mut output = Vec::new();

        // Remove common separators and prefixes
        let cleaned: String = text
            .replace("0x", "")
            .replace("0X", "")
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();

        if cleaned.len() % 2 != 0 {
            return Err(CodecError::InvalidFormat(
                "Hex string must have even number of digits".to_string(),
            ));
        }

        for chunk in cleaned.as_bytes().chunks(2) {
            let hex_str = std::str::from_utf8(chunk).unwrap();
            match u8::from_str_radix(hex_str, 16) {
                Ok(byte) => output.push(byte),
                Err(_) => {
                    return Err(CodecError::InvalidFormat(format!(
                        "Invalid hex: {}",
                        hex_str
                    )));
                }
            }
        }

        Ok(Bytes::from(output))
    }

    fn codec_type(&self) -> CodecType {
        self.codec_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_upper() {
        let codec = HexCodec::new().bytes_per_line(0);
        assert_eq!(codec.encode(b"Hello"), "48 45 4C 4C 4F");
    }

    #[test]
    fn test_encode_lower() {
        let codec = HexCodec::new().format(HexFormat::Lower).bytes_per_line(0);
        assert_eq!(codec.encode(b"Hello"), "48 45 6c 6c 6f");
    }

    #[test]
    fn test_encode_compact() {
        let codec = HexCodec::new().format(HexFormat::Compact).bytes_per_line(0);
        assert_eq!(codec.encode(b"Hello"), "48454C4C4F");
    }

    #[test]
    fn test_decode() {
        let codec = HexCodec::new();
        let result = codec.decode("48 45 4C 4C 4F").unwrap();
        assert_eq!(&result[..], b"Hello");
    }

    #[test]
    fn test_decode_with_prefix() {
        let codec = HexCodec::new();
        let result = codec.decode("0x48, 0x45, 0x4C").unwrap();
        assert_eq!(&result[..], b"HEL");
    }
}





