//! Protocol implementations
//!
//! Provides parsers, encoders, and helpers for industrial and embedded protocols:
//! - Modbus RTU/TCP
//! - Checksum algorithms (CRC-16, CRC-32, etc.)
//! - Framing (SLIP, COBS, STX/ETX, length-prefixed)

pub mod checksum;
pub mod framing;
pub mod modbus;

pub use checksum::{calculate as calc_checksum, ChecksumType};
pub use framing::{encode as frame_encode, decode as frame_decode, FramingType, FrameDecoder};
pub use modbus::{
    ModbusMode, FunctionCode, ExceptionCode, ModbusFrame,
    ModbusRequest, ModbusResponse, ModbusException,
    build_rtu_request, parse_rtu_frame,
    build_tcp_request, parse_tcp_frame,
};
