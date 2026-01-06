//! Data framing protocols
//!
//! Supports: SLIP, COBS, STX/ETX, Length-prefixed, Line-based

/// Framing protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramingType {
    /// No framing - raw bytes
    None,
    /// Line-based with LF delimiter
    LineLf,
    /// Line-based with CRLF delimiter
    LineCrLf,
    /// STX/ETX framing (0x02 start, 0x03 end)
    StxEtx,
    /// SLIP (Serial Line Internet Protocol) - RFC 1055
    Slip,
    /// COBS (Consistent Overhead Byte Stuffing)
    Cobs,
    /// Length-prefixed (1 byte length)
    LengthPrefix8,
    /// Length-prefixed (2 bytes length, big-endian)
    LengthPrefix16Be,
    /// Length-prefixed (2 bytes length, little-endian)
    LengthPrefix16Le,
    /// Length-prefixed (4 bytes length, big-endian)
    LengthPrefix32Be,
    /// Length-prefixed (4 bytes length, little-endian)
    LengthPrefix32Le,
}

impl FramingType {
    /// Get all available framing types
    pub fn all() -> &'static [FramingType] {
        &[
            FramingType::None,
            FramingType::LineLf,
            FramingType::LineCrLf,
            FramingType::StxEtx,
            FramingType::Slip,
            FramingType::Cobs,
            FramingType::LengthPrefix8,
            FramingType::LengthPrefix16Be,
            FramingType::LengthPrefix16Le,
            FramingType::LengthPrefix32Be,
            FramingType::LengthPrefix32Le,
        ]
    }

    /// Get name of framing type
    pub fn name(&self) -> &'static str {
        match self {
            FramingType::None => "None (Raw)",
            FramingType::LineLf => "Line (LF)",
            FramingType::LineCrLf => "Line (CRLF)",
            FramingType::StxEtx => "STX/ETX",
            FramingType::Slip => "SLIP",
            FramingType::Cobs => "COBS",
            FramingType::LengthPrefix8 => "Length-8",
            FramingType::LengthPrefix16Be => "Length-16 BE",
            FramingType::LengthPrefix16Le => "Length-16 LE",
            FramingType::LengthPrefix32Be => "Length-32 BE",
            FramingType::LengthPrefix32Le => "Length-32 LE",
        }
    }
}

// ============ SLIP Constants ============
const SLIP_END: u8 = 0xC0;
const SLIP_ESC: u8 = 0xDB;
const SLIP_ESC_END: u8 = 0xDC;
const SLIP_ESC_ESC: u8 = 0xDD;

// ============ STX/ETX Constants ============
const STX: u8 = 0x02;
const ETX: u8 = 0x03;
const DLE: u8 = 0x10; // Data Link Escape for stuffing

/// Encode data with framing
pub fn encode(data: &[u8], framing: FramingType) -> Vec<u8> {
    match framing {
        FramingType::None => data.to_vec(),
        FramingType::LineLf => encode_line(data, b"\n"),
        FramingType::LineCrLf => encode_line(data, b"\r\n"),
        FramingType::StxEtx => encode_stx_etx(data),
        FramingType::Slip => encode_slip(data),
        FramingType::Cobs => encode_cobs(data),
        FramingType::LengthPrefix8 => encode_length_prefix_8(data),
        FramingType::LengthPrefix16Be => encode_length_prefix_16(data, true),
        FramingType::LengthPrefix16Le => encode_length_prefix_16(data, false),
        FramingType::LengthPrefix32Be => encode_length_prefix_32(data, true),
        FramingType::LengthPrefix32Le => encode_length_prefix_32(data, false),
    }
}

/// Decode framed data, returning extracted packets
pub fn decode(data: &[u8], framing: FramingType) -> Vec<Vec<u8>> {
    match framing {
        FramingType::None => vec![data.to_vec()],
        FramingType::LineLf => decode_line(data, b'\n'),
        FramingType::LineCrLf => decode_line_crlf(data),
        FramingType::StxEtx => decode_stx_etx(data),
        FramingType::Slip => decode_slip(data),
        FramingType::Cobs => decode_cobs(data),
        FramingType::LengthPrefix8 => decode_length_prefix_8(data),
        FramingType::LengthPrefix16Be => decode_length_prefix_16(data, true),
        FramingType::LengthPrefix16Le => decode_length_prefix_16(data, false),
        FramingType::LengthPrefix32Be => decode_length_prefix_32(data, true),
        FramingType::LengthPrefix32Le => decode_length_prefix_32(data, false),
    }
}

// ============ Line-based ============

fn encode_line(data: &[u8], delimiter: &[u8]) -> Vec<u8> {
    let mut result = data.to_vec();
    result.extend_from_slice(delimiter);
    result
}

fn decode_line(data: &[u8], delimiter: u8) -> Vec<Vec<u8>> {
    data.split(|&b| b == delimiter)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_vec())
        .collect()
}

fn decode_line_crlf(data: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if i + 1 < data.len() && data[i] == b'\r' && data[i + 1] == b'\n' {
            if !current.is_empty() {
                result.push(current.clone());
                current.clear();
            }
            i += 2;
        } else {
            current.push(data[i]);
            i += 1;
        }
    }
    
    if !current.is_empty() {
        result.push(current);
    }
    
    result
}

// ============ STX/ETX ============

fn encode_stx_etx(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() * 2 + 2);
    result.push(STX);
    
    for &byte in data {
        match byte {
            STX | ETX | DLE => {
                result.push(DLE);
                result.push(byte);
            }
            _ => result.push(byte),
        }
    }
    
    result.push(ETX);
    result
}

fn decode_stx_etx(data: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut in_frame = false;
    let mut escape = false;
    
    for &byte in data {
        if escape {
            current.push(byte);
            escape = false;
            continue;
        }
        
        match byte {
            STX => {
                in_frame = true;
                current.clear();
            }
            ETX if in_frame => {
                if !current.is_empty() {
                    result.push(current.clone());
                }
                current.clear();
                in_frame = false;
            }
            DLE if in_frame => {
                escape = true;
            }
            _ if in_frame => {
                current.push(byte);
            }
            _ => {}
        }
    }
    
    result
}

// ============ SLIP (RFC 1055) ============

fn encode_slip(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() * 2 + 2);
    
    // Optional: start with END to flush any garbage
    result.push(SLIP_END);
    
    for &byte in data {
        match byte {
            SLIP_END => {
                result.push(SLIP_ESC);
                result.push(SLIP_ESC_END);
            }
            SLIP_ESC => {
                result.push(SLIP_ESC);
                result.push(SLIP_ESC_ESC);
            }
            _ => result.push(byte),
        }
    }
    
    result.push(SLIP_END);
    result
}

fn decode_slip(data: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut escape = false;
    
    for &byte in data {
        if escape {
            match byte {
                SLIP_ESC_END => current.push(SLIP_END),
                SLIP_ESC_ESC => current.push(SLIP_ESC),
                _ => current.push(byte), // Error recovery
            }
            escape = false;
            continue;
        }
        
        match byte {
            SLIP_END => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            SLIP_ESC => {
                escape = true;
            }
            _ => {
                current.push(byte);
            }
        }
    }
    
    // Don't include incomplete packet
    result
}

// ============ COBS (Consistent Overhead Byte Stuffing) ============

fn encode_cobs(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() + data.len() / 254 + 2);
    let mut code_index = 0;
    let mut code: u8 = 1;
    
    result.push(0); // Placeholder for first code
    
    for &byte in data {
        if byte == 0 {
            result[code_index] = code;
            code_index = result.len();
            result.push(0); // Placeholder
            code = 1;
        } else {
            result.push(byte);
            code += 1;
            if code == 0xFF {
                result[code_index] = code;
                code_index = result.len();
                result.push(0); // Placeholder
                code = 1;
            }
        }
    }
    
    result[code_index] = code;
    result.push(0); // Delimiter
    result
}

fn decode_cobs(data: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    
    // Split by zero delimiter
    for chunk in data.split(|&b| b == 0) {
        if chunk.is_empty() {
            continue;
        }
        
        if let Some(decoded) = decode_cobs_packet(chunk) {
            result.push(decoded);
        }
    }
    
    result
}

fn decode_cobs_packet(data: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len());
    let mut i = 0;
    
    while i < data.len() {
        let code = data[i] as usize;
        if code == 0 {
            return None; // Invalid
        }
        
        i += 1;
        
        for _ in 1..code {
            if i >= data.len() {
                break;
            }
            result.push(data[i]);
            i += 1;
        }
        
        if code < 0xFF && i < data.len() {
            result.push(0);
        }
    }
    
    // Remove trailing zero if present
    if result.last() == Some(&0) {
        result.pop();
    }
    
    Some(result)
}

// ============ Length-prefixed ============

fn encode_length_prefix_8(data: &[u8]) -> Vec<u8> {
    let len = std::cmp::min(data.len(), 255) as u8;
    let mut result = Vec::with_capacity(1 + len as usize);
    result.push(len);
    result.extend_from_slice(&data[..len as usize]);
    result
}

fn decode_length_prefix_8(data: &[u8]) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        let len = data[i] as usize;
        i += 1;
        
        if i + len <= data.len() {
            result.push(data[i..i + len].to_vec());
            i += len;
        } else {
            break;
        }
    }
    
    result
}

fn encode_length_prefix_16(data: &[u8], big_endian: bool) -> Vec<u8> {
    let len = std::cmp::min(data.len(), 65535) as u16;
    let mut result = Vec::with_capacity(2 + len as usize);
    
    if big_endian {
        result.extend_from_slice(&len.to_be_bytes());
    } else {
        result.extend_from_slice(&len.to_le_bytes());
    }
    result.extend_from_slice(&data[..len as usize]);
    result
}

fn decode_length_prefix_16(data: &[u8], big_endian: bool) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i + 2 <= data.len() {
        let len = if big_endian {
            u16::from_be_bytes([data[i], data[i + 1]]) as usize
        } else {
            u16::from_le_bytes([data[i], data[i + 1]]) as usize
        };
        i += 2;
        
        if i + len <= data.len() {
            result.push(data[i..i + len].to_vec());
            i += len;
        } else {
            break;
        }
    }
    
    result
}

fn encode_length_prefix_32(data: &[u8], big_endian: bool) -> Vec<u8> {
    let len = data.len() as u32;
    let mut result = Vec::with_capacity(4 + data.len());
    
    if big_endian {
        result.extend_from_slice(&len.to_be_bytes());
    } else {
        result.extend_from_slice(&len.to_le_bytes());
    }
    result.extend_from_slice(data);
    result
}

fn decode_length_prefix_32(data: &[u8], big_endian: bool) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i + 4 <= data.len() {
        let len = if big_endian {
            u32::from_be_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize
        } else {
            u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize
        };
        i += 4;
        
        if i + len <= data.len() {
            result.push(data[i..i + len].to_vec());
            i += len;
        } else {
            break;
        }
    }
    
    result
}

/// Streaming frame decoder that handles partial data
pub struct FrameDecoder {
    framing: FramingType,
    buffer: Vec<u8>,
}

impl FrameDecoder {
    /// Create new decoder
    pub fn new(framing: FramingType) -> Self {
        Self {
            framing,
            buffer: Vec::new(),
        }
    }

    /// Add data and return complete frames
    pub fn push(&mut self, data: &[u8]) -> Vec<Vec<u8>> {
        self.buffer.extend_from_slice(data);
        
        let frames = decode(&self.buffer, self.framing);
        
        // Keep incomplete data in buffer
        // This is a simplified version - real implementation would track boundaries
        if !frames.is_empty() {
            self.buffer.clear();
        }
        
        frames
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slip_roundtrip() {
        let original = vec![0xC0, 0xDB, 0x01, 0x02, 0x03];
        let encoded = encode_slip(&original);
        let decoded = decode_slip(&encoded);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], original);
    }

    #[test]
    fn test_cobs_roundtrip() {
        let original = vec![0x00, 0x01, 0x02, 0x00, 0x03];
        let encoded = encode_cobs(&original);
        let decoded = decode_cobs(&encoded);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], original);
    }

    #[test]
    fn test_stx_etx_roundtrip() {
        let original = vec![0x02, 0x03, 0x10, 0x41, 0x42];
        let encoded = encode_stx_etx(&original);
        let decoded = decode_stx_etx(&encoded);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], original);
    }

    #[test]
    fn test_length_prefix_roundtrip() {
        let original = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        
        let encoded = encode_length_prefix_8(&original);
        let decoded = decode_length_prefix_8(&encoded);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], original);
        
        let encoded = encode_length_prefix_16(&original, true);
        let decoded = decode_length_prefix_16(&encoded, true);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0], original);
    }
}
