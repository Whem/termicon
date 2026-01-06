//! Protocol implementations
//!
//! Provides parsers, encoders, and helpers for industrial and embedded protocols:
//! - Modbus RTU/TCP
//! - Checksum algorithms (CRC-16, CRC-32, etc.)
//! - Framing (SLIP, COBS, STX/ETX, length-prefixed)
//! - NMEA 0183 (GPS and marine)

pub mod checksum;
pub mod framing;
pub mod modbus;
pub mod modbus_monitor;
pub mod nmea;

// Note: File transfer protocols (XMODEM, YMODEM, ZMODEM) are in core::transfer module
// Kermit is in core::file_transfer module

pub use checksum::{calculate as calc_checksum, ChecksumType};
pub use framing::{encode as frame_encode, decode as frame_decode, FramingType, FrameDecoder};
pub use modbus::{
    ModbusMode, FunctionCode, ExceptionCode, ModbusFrame,
    ModbusRequest, ModbusResponse, ModbusException,
    build_rtu_request, parse_rtu_frame,
    build_tcp_request, parse_tcp_frame,
};
pub use modbus_monitor::{
    ModbusPoller, ModbusDataType, ModbusValue, RegisterDefinition,
    RegisterType, PollGroup, RegisterReading, PollingEvent,
};
pub use nmea::{
    NmeaParser, NmeaSentence, NmeaSentenceType, NmeaError,
    GgaData, RmcData, GsvData, GsaData, VtgData,
    Coordinate, SatelliteInfo, GpsFixQuality,
};
