//! Protocol Definition DSL
//!
//! Declarative protocol schema for auto-generating parsers, builders, and UI.
//! Define protocols in YAML/JSON and get automatic decoding/encoding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Byte order (endianness)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ByteOrder {
    /// Little endian (LSB first)
    #[default]
    Little,
    /// Big endian (MSB first)
    Big,
}

/// Field data type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// Unsigned 8-bit integer
    U8,
    /// Signed 8-bit integer
    I8,
    /// Unsigned 16-bit integer
    U16,
    /// Signed 16-bit integer
    I16,
    /// Unsigned 32-bit integer
    U32,
    /// Signed 32-bit integer
    I32,
    /// Unsigned 64-bit integer
    U64,
    /// Signed 64-bit integer
    I64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Fixed-length byte array
    Bytes(usize),
    /// Variable-length byte array (length from another field)
    VarBytes(String),
    /// Fixed-length string
    String(usize),
    /// Null-terminated string
    CString,
    /// Boolean (1 byte)
    Bool,
    /// Bit field (within a byte)
    Bits { offset: u8, length: u8 },
    /// Enum (u8 with named values)
    Enum(HashMap<u8, String>),
    /// Nested structure
    Struct(String),
    /// Array of fixed count
    Array { element_type: Box<FieldType>, count: usize },
    /// Variable-length array (count from another field)
    VarArray { element_type: Box<FieldType>, count_field: String },
}

/// Value transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Transform {
    /// No transformation
    None,
    /// Linear scaling: value * scale + offset
    Scale {
        scale: f64,
        #[serde(default)]
        offset: f64,
    },
    /// Value mapping
    Map { map: HashMap<String, String> },
    /// Bit mask
    Mask { mask: u64 },
    /// Custom formula (simple expression)
    Formula { formula: String },
}

impl Default for Transform {
    fn default() -> Self {
        Self::None
    }
}

/// Checksum configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChecksumConfig {
    /// No checksum
    None,
    /// XOR of all bytes
    Xor8,
    /// LRC (Longitudinal Redundancy Check)
    Lrc8,
    /// CRC-16 Modbus
    Crc16Modbus,
    /// CRC-16 CCITT
    Crc16Ccitt,
    /// CRC-32
    Crc32,
    /// Custom checksum with range specification
    Custom {
        algorithm: String,
        start_field: String,
        end_field: String,
    },
}

impl Default for ChecksumConfig {
    fn default() -> Self {
        Self::None
    }
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Field type
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Byte order override
    #[serde(default)]
    pub byte_order: Option<ByteOrder>,
    /// Value transformation
    #[serde(default)]
    pub transform: Transform,
    /// Unit (e.g., "°C", "mV")
    #[serde(default)]
    pub unit: Option<String>,
    /// Expected/constant value
    #[serde(default)]
    pub constant: Option<serde_json::Value>,
    /// Validation expression
    #[serde(default)]
    pub validate: Option<String>,
    /// Optional field (may not be present)
    #[serde(default)]
    pub optional: bool,
    /// Condition for presence
    #[serde(default)]
    pub condition: Option<String>,
}

/// Message/packet definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDef {
    /// Message name
    pub name: String,
    /// Message ID (if applicable)
    #[serde(default)]
    pub id: Option<u8>,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Fields in order
    pub fields: Vec<FieldDef>,
    /// Checksum configuration
    #[serde(default)]
    pub checksum: ChecksumConfig,
    /// Is request message
    #[serde(default)]
    pub is_request: bool,
    /// Corresponding response message name
    #[serde(default)]
    pub response: Option<String>,
}

/// Complete protocol definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDef {
    /// Protocol name
    pub name: String,
    /// Protocol version
    #[serde(default)]
    pub version: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Default byte order
    #[serde(default)]
    pub byte_order: ByteOrder,
    /// Message definitions
    pub messages: Vec<MessageDef>,
    /// Shared structure definitions
    #[serde(default)]
    pub structs: HashMap<String, Vec<FieldDef>>,
    /// Common header (prepended to all messages)
    #[serde(default)]
    pub header: Option<Vec<FieldDef>>,
    /// Common footer (appended to all messages)
    #[serde(default)]
    pub footer: Option<Vec<FieldDef>>,
}

impl ProtocolDef {
    /// Load protocol from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Load protocol from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Export to YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get message by name
    pub fn get_message(&self, name: &str) -> Option<&MessageDef> {
        self.messages.iter().find(|m| m.name == name)
    }

    /// Get message by ID
    pub fn get_message_by_id(&self, id: u8) -> Option<&MessageDef> {
        self.messages.iter().find(|m| m.id == Some(id))
    }
}

/// Decoded field value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecodedValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    String(String),
    Enum { value: u8, name: String },
    Array(Vec<DecodedValue>),
    Struct(HashMap<String, DecodedValue>),
    /// Scaled/transformed value
    Scaled { raw: Box<DecodedValue>, scaled: f64, unit: Option<String> },
}

impl DecodedValue {
    /// Get as f64 (for charting, etc.)
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::U8(v) => Some(*v as f64),
            Self::I8(v) => Some(*v as f64),
            Self::U16(v) => Some(*v as f64),
            Self::I16(v) => Some(*v as f64),
            Self::U32(v) => Some(*v as f64),
            Self::I32(v) => Some(*v as f64),
            Self::U64(v) => Some(*v as f64),
            Self::I64(v) => Some(*v as f64),
            Self::F32(v) => Some(*v as f64),
            Self::F64(v) => Some(*v),
            Self::Scaled { scaled, .. } => Some(*scaled),
            _ => None,
        }
    }

    /// Get display string
    pub fn display(&self) -> String {
        match self {
            Self::U8(v) => format!("{}", v),
            Self::I8(v) => format!("{}", v),
            Self::U16(v) => format!("{}", v),
            Self::I16(v) => format!("{}", v),
            Self::U32(v) => format!("{}", v),
            Self::I32(v) => format!("{}", v),
            Self::U64(v) => format!("{}", v),
            Self::I64(v) => format!("{}", v),
            Self::F32(v) => format!("{:.3}", v),
            Self::F64(v) => format!("{:.6}", v),
            Self::Bool(v) => format!("{}", v),
            Self::Bytes(v) => hex::encode(v),
            Self::String(v) => v.clone(),
            Self::Enum { value, name } => format!("{} (0x{:02X})", name, value),
            Self::Array(v) => format!("[{} items]", v.len()),
            Self::Struct(v) => format!("{{ {} fields }}", v.len()),
            Self::Scaled { scaled, unit, .. } => {
                if let Some(u) = unit {
                    format!("{:.3} {}", scaled, u)
                } else {
                    format!("{:.3}", scaled)
                }
            }
        }
    }
}

/// Decoded message result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedMessage {
    /// Message name
    pub name: String,
    /// Decoded fields
    pub fields: HashMap<String, DecodedValue>,
    /// Raw bytes
    pub raw: Vec<u8>,
    /// Checksum valid
    pub checksum_valid: Option<bool>,
    /// Parse errors/warnings
    pub warnings: Vec<String>,
}

/// Protocol decoder
pub struct ProtocolDecoder {
    protocol: ProtocolDef,
}

impl ProtocolDecoder {
    /// Create decoder from protocol definition
    pub fn new(protocol: ProtocolDef) -> Self {
        Self { protocol }
    }

    /// Decode bytes into a message
    pub fn decode(&self, data: &[u8]) -> Result<DecodedMessage, String> {
        // Try to identify message by ID or pattern
        for msg in &self.protocol.messages {
            if let Ok(decoded) = self.try_decode_message(data, msg) {
                return Ok(decoded);
            }
        }
        
        Err("Unable to decode message - no matching protocol message".to_string())
    }

    /// Decode specific message type
    pub fn decode_as(&self, data: &[u8], message_name: &str) -> Result<DecodedMessage, String> {
        let msg = self.protocol.get_message(message_name)
            .ok_or_else(|| format!("Unknown message: {}", message_name))?;
        
        self.try_decode_message(data, msg)
    }

    fn try_decode_message(&self, data: &[u8], msg: &MessageDef) -> Result<DecodedMessage, String> {
        let mut offset = 0;
        let mut fields = HashMap::new();
        let mut warnings = Vec::new();
        let byte_order = self.protocol.byte_order;

        // Decode header if present
        if let Some(ref header) = self.protocol.header {
            for field_def in header {
                match self.decode_field(data, &mut offset, field_def, byte_order) {
                    Ok(value) => { fields.insert(field_def.name.clone(), value); }
                    Err(e) => warnings.push(format!("Header field {}: {}", field_def.name, e)),
                }
            }
        }

        // Decode message fields
        for field_def in &msg.fields {
            match self.decode_field(data, &mut offset, field_def, byte_order) {
                Ok(value) => { fields.insert(field_def.name.clone(), value); }
                Err(e) => {
                    if field_def.optional {
                        warnings.push(format!("Optional field {}: {}", field_def.name, e));
                    } else {
                        return Err(format!("Field {}: {}", field_def.name, e));
                    }
                }
            }
        }

        Ok(DecodedMessage {
            name: msg.name.clone(),
            fields,
            raw: data.to_vec(),
            checksum_valid: None, // TODO: verify checksum
            warnings,
        })
    }

    fn decode_field(
        &self,
        data: &[u8],
        offset: &mut usize,
        field: &FieldDef,
        default_order: ByteOrder,
    ) -> Result<DecodedValue, String> {
        let order = field.byte_order.unwrap_or(default_order);
        
        let value = match &field.field_type {
            FieldType::U8 => {
                if *offset >= data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let v = data[*offset];
                *offset += 1;
                DecodedValue::U8(v)
            }
            FieldType::I8 => {
                if *offset >= data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let v = data[*offset] as i8;
                *offset += 1;
                DecodedValue::I8(v)
            }
            FieldType::U16 => {
                if *offset + 2 > data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let bytes = [data[*offset], data[*offset + 1]];
                let v = match order {
                    ByteOrder::Little => u16::from_le_bytes(bytes),
                    ByteOrder::Big => u16::from_be_bytes(bytes),
                };
                *offset += 2;
                DecodedValue::U16(v)
            }
            FieldType::I16 => {
                if *offset + 2 > data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let bytes = [data[*offset], data[*offset + 1]];
                let v = match order {
                    ByteOrder::Little => i16::from_le_bytes(bytes),
                    ByteOrder::Big => i16::from_be_bytes(bytes),
                };
                *offset += 2;
                DecodedValue::I16(v)
            }
            FieldType::U32 => {
                if *offset + 4 > data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let bytes: [u8; 4] = data[*offset..*offset + 4].try_into().unwrap();
                let v = match order {
                    ByteOrder::Little => u32::from_le_bytes(bytes),
                    ByteOrder::Big => u32::from_be_bytes(bytes),
                };
                *offset += 4;
                DecodedValue::U32(v)
            }
            FieldType::F32 => {
                if *offset + 4 > data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let bytes: [u8; 4] = data[*offset..*offset + 4].try_into().unwrap();
                let v = match order {
                    ByteOrder::Little => f32::from_le_bytes(bytes),
                    ByteOrder::Big => f32::from_be_bytes(bytes),
                };
                *offset += 4;
                DecodedValue::F32(v)
            }
            FieldType::Bytes(len) => {
                if *offset + len > data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let v = data[*offset..*offset + len].to_vec();
                *offset += len;
                DecodedValue::Bytes(v)
            }
            FieldType::String(len) => {
                if *offset + len > data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let v = String::from_utf8_lossy(&data[*offset..*offset + len]).to_string();
                *offset += len;
                DecodedValue::String(v.trim_end_matches('\0').to_string())
            }
            FieldType::Bool => {
                if *offset >= data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let v = data[*offset] != 0;
                *offset += 1;
                DecodedValue::Bool(v)
            }
            FieldType::Enum(mapping) => {
                if *offset >= data.len() {
                    return Err("Buffer underflow".to_string());
                }
                let v = data[*offset];
                *offset += 1;
                let name = mapping.get(&v).cloned().unwrap_or_else(|| format!("Unknown({})", v));
                DecodedValue::Enum { value: v, name }
            }
            _ => {
                // TODO: Implement remaining types
                return Err(format!("Unsupported field type: {:?}", field.field_type));
            }
        };

        // Apply transformation
        let final_value = match &field.transform {
            Transform::Scale { scale, offset: trans_offset } => {
                if let Some(raw) = value.as_f64() {
                    DecodedValue::Scaled {
                        raw: Box::new(value),
                        scaled: raw * scale + trans_offset,
                        unit: field.unit.clone(),
                    }
                } else {
                    value
                }
            }
            _ => value,
        };

        Ok(final_value)
    }
}

/// Example protocol definition YAML
pub const EXAMPLE_PROTOCOL_YAML: &str = r#"
name: "Simple Sensor Protocol"
version: "1.0"
description: "Example protocol for temperature/humidity sensor"
byte_order: little

messages:
  - name: "SensorData"
    id: 0x01
    description: "Periodic sensor data report"
    is_request: false
    fields:
      - name: "header"
        type: u8
        constant: 0xAA
        description: "Start byte"
      - name: "msg_id"
        type: u8
        description: "Message ID"
      - name: "temperature_raw"
        type: i16
        transform:
          type: scale
          scale: 0.1
          offset: 0
        unit: "°C"
        description: "Temperature (0.1°C resolution)"
      - name: "humidity_raw"
        type: u16
        transform:
          type: scale
          scale: 0.1
          offset: 0
        unit: "%"
        description: "Humidity (0.1% resolution)"
      - name: "checksum"
        type: u8
        description: "XOR checksum"
    checksum:
      type: xor8

  - name: "ReadCommand"
    id: 0x10
    description: "Read sensor command"
    is_request: true
    response: "SensorData"
    fields:
      - name: "header"
        type: u8
        constant: 0xAA
      - name: "msg_id"
        type: u8
        constant: 0x10
      - name: "checksum"
        type: u8
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_yaml_protocol() {
        let protocol = ProtocolDef::from_yaml(EXAMPLE_PROTOCOL_YAML).unwrap();
        assert_eq!(protocol.name, "Simple Sensor Protocol");
        assert_eq!(protocol.messages.len(), 2);
        
        let sensor_data = protocol.get_message("SensorData").unwrap();
        assert_eq!(sensor_data.fields.len(), 5);
    }

    #[test]
    fn test_decode_message() {
        let protocol = ProtocolDef::from_yaml(EXAMPLE_PROTOCOL_YAML).unwrap();
        let decoder = ProtocolDecoder::new(protocol);
        
        // Example: AA 01 E803 9001 XX (temperature=100.0, humidity=40.0)
        let data = vec![0xAA, 0x01, 0xE8, 0x03, 0x90, 0x01, 0x00];
        
        let result = decoder.decode_as(&data, "SensorData").unwrap();
        assert_eq!(result.name, "SensorData");
        
        if let Some(DecodedValue::Scaled { scaled, unit, .. }) = result.fields.get("temperature_raw") {
            assert!((scaled - 100.0).abs() < 0.01);
            assert_eq!(unit.as_ref().unwrap(), "°C");
        } else {
            panic!("Expected scaled temperature");
        }
    }
}






