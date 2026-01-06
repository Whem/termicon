//! Text codec for plain text display

use super::{Codec, CodecError, CodecType};
use bytes::Bytes;

/// Text codec configuration
#[derive(Debug, Clone)]
pub struct TextCodecConfig {
    /// Character to display for non-printable characters
    pub non_printable_char: char,
    /// Show escape sequences (e.g., \r\n instead of actual newlines)
    pub show_escape_sequences: bool,
    /// Character encoding
    pub encoding: TextEncoding,
}

impl Default for TextCodecConfig {
    fn default() -> Self {
        Self {
            non_printable_char: '·',
            show_escape_sequences: false,
            encoding: TextEncoding::Utf8,
        }
    }
}

/// Text encoding type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextEncoding {
    /// UTF-8 encoding
    #[default]
    Utf8,
    /// ASCII (7-bit)
    Ascii,
    /// Latin-1 (ISO-8859-1)
    Latin1,
}

/// Text codec for plain text display
pub struct TextCodec {
    config: TextCodecConfig,
}

impl TextCodec {
    /// Create a new text codec with default config
    pub fn new() -> Self {
        Self {
            config: TextCodecConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: TextCodecConfig) -> Self {
        Self { config }
    }

    /// Set non-printable character
    #[must_use]
    pub fn non_printable_char(mut self, c: char) -> Self {
        self.config.non_printable_char = c;
        self
    }

    /// Enable escape sequence display
    #[must_use]
    pub fn show_escape_sequences(mut self, show: bool) -> Self {
        self.config.show_escape_sequences = show;
        self
    }
}

impl Default for TextCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec for TextCodec {
    fn encode(&self, data: &[u8]) -> String {
        let mut output = String::with_capacity(data.len() * 2);

        for &byte in data {
            if self.config.show_escape_sequences {
                match byte {
                    b'\r' => output.push_str("\\r"),
                    b'\n' => output.push_str("\\n"),
                    b'\t' => output.push_str("\\t"),
                    b'\0' => output.push_str("\\0"),
                    0x1b => output.push_str("\\e"), // ESC
                    b if b.is_ascii_graphic() || b == b' ' => output.push(b as char),
                    b => output.push_str(&format!("\\x{:02x}", b)),
                }
            } else {
                match byte {
                    b'\r' | b'\n' | b'\t' => output.push(byte as char),
                    b if b.is_ascii_graphic() || b == b' ' => output.push(b as char),
                    _ => output.push(self.config.non_printable_char),
                }
            }
        }

        output
    }

    fn decode(&self, text: &str) -> Result<Bytes, CodecError> {
        let mut output = Vec::with_capacity(text.len());
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('r') => output.push(b'\r'),
                    Some('n') => output.push(b'\n'),
                    Some('t') => output.push(b'\t'),
                    Some('0') => output.push(0),
                    Some('e') => output.push(0x1b),
                    Some('x') => {
                        let hex: String = chars.by_ref().take(2).collect();
                        if hex.len() == 2 {
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                output.push(byte);
                            } else {
                                return Err(CodecError::InvalidFormat(format!(
                                    "Invalid hex sequence: \\x{}",
                                    hex
                                )));
                            }
                        } else {
                            return Err(CodecError::InvalidFormat(
                                "Incomplete hex sequence".to_string(),
                            ));
                        }
                    }
                    Some('\\') => output.push(b'\\'),
                    Some(other) => {
                        output.push(b'\\');
                        output.extend(other.to_string().bytes());
                    }
                    None => output.push(b'\\'),
                }
            } else {
                output.extend(c.to_string().bytes());
            }
        }

        Ok(Bytes::from(output))
    }

    fn codec_type(&self) -> CodecType {
        CodecType::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_ascii() {
        let codec = TextCodec::new();
        assert_eq!(codec.encode(b"Hello"), "Hello");
    }

    #[test]
    fn test_encode_escape_sequences() {
        let codec = TextCodec::new().show_escape_sequences(true);
        assert_eq!(codec.encode(b"Hello\r\n"), "Hello\\r\\n");
    }

    #[test]
    fn test_decode_escape_sequences() {
        let codec = TextCodec::new();
        let result = codec.decode("Hello\\r\\n").unwrap();
        assert_eq!(&result[..], b"Hello\r\n");
    }

    #[test]
    fn test_decode_hex() {
        let codec = TextCodec::new();
        let result = codec.decode("\\x00\\xff").unwrap();
        assert_eq!(&result[..], &[0x00, 0xff]);
    }
}






