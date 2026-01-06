//! Modbus protocol implementation
//!
//! Supports RTU and TCP framing, common function codes

use super::checksum;

/// Modbus protocol variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusMode {
    /// Modbus RTU (binary, CRC-16)
    Rtu,
    /// Modbus TCP (with MBAP header)
    Tcp,
    /// Modbus ASCII (hex text, LRC)
    Ascii,
}

/// Modbus function codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FunctionCode {
    /// Read Coils (0x01)
    ReadCoils = 0x01,
    /// Read Discrete Inputs (0x02)
    ReadDiscreteInputs = 0x02,
    /// Read Holding Registers (0x03)
    ReadHoldingRegisters = 0x03,
    /// Read Input Registers (0x04)
    ReadInputRegisters = 0x04,
    /// Write Single Coil (0x05)
    WriteSingleCoil = 0x05,
    /// Write Single Register (0x06)
    WriteSingleRegister = 0x06,
    /// Write Multiple Coils (0x0F)
    WriteMultipleCoils = 0x0F,
    /// Write Multiple Registers (0x10)
    WriteMultipleRegisters = 0x10,
    /// Read/Write Multiple Registers (0x17)
    ReadWriteMultipleRegisters = 0x17,
    /// Mask Write Register (0x16)
    MaskWriteRegister = 0x16,
    /// Read FIFO Queue (0x18)
    ReadFifoQueue = 0x18,
    /// Read Device Identification (0x2B)
    ReadDeviceIdentification = 0x2B,
}

impl FunctionCode {
    /// Get function code from u8
    pub fn from_u8(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(FunctionCode::ReadCoils),
            0x02 => Some(FunctionCode::ReadDiscreteInputs),
            0x03 => Some(FunctionCode::ReadHoldingRegisters),
            0x04 => Some(FunctionCode::ReadInputRegisters),
            0x05 => Some(FunctionCode::WriteSingleCoil),
            0x06 => Some(FunctionCode::WriteSingleRegister),
            0x0F => Some(FunctionCode::WriteMultipleCoils),
            0x10 => Some(FunctionCode::WriteMultipleRegisters),
            0x16 => Some(FunctionCode::MaskWriteRegister),
            0x17 => Some(FunctionCode::ReadWriteMultipleRegisters),
            0x18 => Some(FunctionCode::ReadFifoQueue),
            0x2B => Some(FunctionCode::ReadDeviceIdentification),
            _ => None,
        }
    }

    /// Get name of function code
    pub fn name(&self) -> &'static str {
        match self {
            FunctionCode::ReadCoils => "Read Coils",
            FunctionCode::ReadDiscreteInputs => "Read Discrete Inputs",
            FunctionCode::ReadHoldingRegisters => "Read Holding Registers",
            FunctionCode::ReadInputRegisters => "Read Input Registers",
            FunctionCode::WriteSingleCoil => "Write Single Coil",
            FunctionCode::WriteSingleRegister => "Write Single Register",
            FunctionCode::WriteMultipleCoils => "Write Multiple Coils",
            FunctionCode::WriteMultipleRegisters => "Write Multiple Registers",
            FunctionCode::MaskWriteRegister => "Mask Write Register",
            FunctionCode::ReadWriteMultipleRegisters => "Read/Write Multiple Registers",
            FunctionCode::ReadFifoQueue => "Read FIFO Queue",
            FunctionCode::ReadDeviceIdentification => "Read Device Identification",
        }
    }
}

/// Modbus exception codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExceptionCode {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
    SlaveDeviceFailure = 0x04,
    Acknowledge = 0x05,
    SlaveDeviceBusy = 0x06,
    MemoryParityError = 0x08,
    GatewayPathUnavailable = 0x0A,
    GatewayTargetDeviceFailedToRespond = 0x0B,
}

impl ExceptionCode {
    /// Get exception from u8
    pub fn from_u8(code: u8) -> Option<Self> {
        match code {
            0x01 => Some(ExceptionCode::IllegalFunction),
            0x02 => Some(ExceptionCode::IllegalDataAddress),
            0x03 => Some(ExceptionCode::IllegalDataValue),
            0x04 => Some(ExceptionCode::SlaveDeviceFailure),
            0x05 => Some(ExceptionCode::Acknowledge),
            0x06 => Some(ExceptionCode::SlaveDeviceBusy),
            0x08 => Some(ExceptionCode::MemoryParityError),
            0x0A => Some(ExceptionCode::GatewayPathUnavailable),
            0x0B => Some(ExceptionCode::GatewayTargetDeviceFailedToRespond),
            _ => None,
        }
    }

    /// Get name of exception
    pub fn name(&self) -> &'static str {
        match self {
            ExceptionCode::IllegalFunction => "Illegal Function",
            ExceptionCode::IllegalDataAddress => "Illegal Data Address",
            ExceptionCode::IllegalDataValue => "Illegal Data Value",
            ExceptionCode::SlaveDeviceFailure => "Slave Device Failure",
            ExceptionCode::Acknowledge => "Acknowledge",
            ExceptionCode::SlaveDeviceBusy => "Slave Device Busy",
            ExceptionCode::MemoryParityError => "Memory Parity Error",
            ExceptionCode::GatewayPathUnavailable => "Gateway Path Unavailable",
            ExceptionCode::GatewayTargetDeviceFailedToRespond => "Gateway Target Failed to Respond",
        }
    }
}

/// Modbus request/response PDU
#[derive(Debug, Clone)]
pub struct ModbusPdu {
    pub function_code: u8,
    pub data: Vec<u8>,
}

/// Modbus ADU (Application Data Unit)
#[derive(Debug, Clone)]
pub struct ModbusAdu {
    pub slave_id: u8,
    pub pdu: ModbusPdu,
    pub transaction_id: u16, // For TCP
}

/// Parsed Modbus frame
#[derive(Debug, Clone)]
pub enum ModbusFrame {
    Request(ModbusRequest),
    Response(ModbusResponse),
    Exception(ModbusException),
}

/// Modbus request
#[derive(Debug, Clone)]
pub struct ModbusRequest {
    pub slave_id: u8,
    pub function: FunctionCode,
    pub start_address: u16,
    pub quantity: u16,
    pub data: Vec<u8>,
}

/// Modbus response
#[derive(Debug, Clone)]
pub struct ModbusResponse {
    pub slave_id: u8,
    pub function: FunctionCode,
    pub data: Vec<u8>,
}

/// Modbus exception response
#[derive(Debug, Clone)]
pub struct ModbusException {
    pub slave_id: u8,
    pub function: u8,
    pub exception: ExceptionCode,
}

// ============ RTU Encoding/Decoding ============

/// Build Modbus RTU request frame
pub fn build_rtu_request(slave_id: u8, function: FunctionCode, start_address: u16, quantity: u16) -> Vec<u8> {
    let mut frame = Vec::with_capacity(8);
    frame.push(slave_id);
    frame.push(function as u8);
    frame.extend_from_slice(&start_address.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());
    
    // Add CRC-16 Modbus
    let crc = checksum::crc16_modbus(&frame);
    frame.extend_from_slice(&crc.to_le_bytes());
    
    frame
}

/// Build Modbus RTU write single register request
pub fn build_rtu_write_single_register(slave_id: u8, address: u16, value: u16) -> Vec<u8> {
    let mut frame = Vec::with_capacity(8);
    frame.push(slave_id);
    frame.push(FunctionCode::WriteSingleRegister as u8);
    frame.extend_from_slice(&address.to_be_bytes());
    frame.extend_from_slice(&value.to_be_bytes());
    
    let crc = checksum::crc16_modbus(&frame);
    frame.extend_from_slice(&crc.to_le_bytes());
    
    frame
}

/// Build Modbus RTU write multiple registers request
pub fn build_rtu_write_multiple_registers(slave_id: u8, start_address: u16, values: &[u16]) -> Vec<u8> {
    let quantity = values.len() as u16;
    let byte_count = (values.len() * 2) as u8;
    
    let mut frame = Vec::with_capacity(9 + values.len() * 2);
    frame.push(slave_id);
    frame.push(FunctionCode::WriteMultipleRegisters as u8);
    frame.extend_from_slice(&start_address.to_be_bytes());
    frame.extend_from_slice(&quantity.to_be_bytes());
    frame.push(byte_count);
    
    for value in values {
        frame.extend_from_slice(&value.to_be_bytes());
    }
    
    let crc = checksum::crc16_modbus(&frame);
    frame.extend_from_slice(&crc.to_le_bytes());
    
    frame
}

/// Parse Modbus RTU frame
pub fn parse_rtu_frame(data: &[u8]) -> Result<ModbusFrame, &'static str> {
    if data.len() < 4 {
        return Err("Frame too short");
    }
    
    // Verify CRC
    let frame_len = data.len();
    let crc_received = u16::from_le_bytes([data[frame_len - 2], data[frame_len - 1]]);
    let crc_calculated = checksum::crc16_modbus(&data[..frame_len - 2]);
    
    if crc_received != crc_calculated {
        return Err("CRC mismatch");
    }
    
    let slave_id = data[0];
    let function_code = data[1];
    
    // Check for exception response (bit 7 set)
    if function_code & 0x80 != 0 {
        if data.len() < 5 {
            return Err("Exception frame too short");
        }
        let exception = ExceptionCode::from_u8(data[2]).unwrap_or(ExceptionCode::SlaveDeviceFailure);
        return Ok(ModbusFrame::Exception(ModbusException {
            slave_id,
            function: function_code & 0x7F,
            exception,
        }));
    }
    
    // Parse based on function code
    let function = FunctionCode::from_u8(function_code).ok_or("Unknown function code")?;
    
    match function {
        FunctionCode::ReadHoldingRegisters | FunctionCode::ReadInputRegisters |
        FunctionCode::ReadCoils | FunctionCode::ReadDiscreteInputs => {
            // Response: byte count + data
            if data.len() < 5 {
                return Err("Response too short");
            }
            let byte_count = data[2] as usize;
            if data.len() < 3 + byte_count + 2 {
                return Err("Incomplete data");
            }
            Ok(ModbusFrame::Response(ModbusResponse {
                slave_id,
                function,
                data: data[3..3 + byte_count].to_vec(),
            }))
        }
        FunctionCode::WriteSingleCoil | FunctionCode::WriteSingleRegister => {
            // Echo response
            if data.len() < 8 {
                return Err("Response too short");
            }
            Ok(ModbusFrame::Response(ModbusResponse {
                slave_id,
                function,
                data: data[2..6].to_vec(),
            }))
        }
        FunctionCode::WriteMultipleCoils | FunctionCode::WriteMultipleRegisters => {
            // Response: address + quantity
            if data.len() < 8 {
                return Err("Response too short");
            }
            Ok(ModbusFrame::Response(ModbusResponse {
                slave_id,
                function,
                data: data[2..6].to_vec(),
            }))
        }
        _ => {
            // Generic response
            Ok(ModbusFrame::Response(ModbusResponse {
                slave_id,
                function,
                data: data[2..frame_len - 2].to_vec(),
            }))
        }
    }
}

// ============ TCP Encoding/Decoding ============

/// MBAP Header for Modbus TCP
#[derive(Debug, Clone)]
pub struct MbapHeader {
    pub transaction_id: u16,
    pub protocol_id: u16,  // Always 0 for Modbus
    pub length: u16,
    pub unit_id: u8,
}

/// Build Modbus TCP request frame
pub fn build_tcp_request(
    transaction_id: u16,
    unit_id: u8,
    function: FunctionCode,
    start_address: u16,
    quantity: u16,
) -> Vec<u8> {
    let mut pdu = Vec::with_capacity(5);
    pdu.push(function as u8);
    pdu.extend_from_slice(&start_address.to_be_bytes());
    pdu.extend_from_slice(&quantity.to_be_bytes());
    
    let length = (pdu.len() + 1) as u16; // +1 for unit_id
    
    let mut frame = Vec::with_capacity(7 + pdu.len());
    frame.extend_from_slice(&transaction_id.to_be_bytes());
    frame.extend_from_slice(&0u16.to_be_bytes()); // Protocol ID
    frame.extend_from_slice(&length.to_be_bytes());
    frame.push(unit_id);
    frame.extend_from_slice(&pdu);
    
    frame
}

/// Parse Modbus TCP frame
pub fn parse_tcp_frame(data: &[u8]) -> Result<(MbapHeader, ModbusFrame), &'static str> {
    if data.len() < 8 {
        return Err("Frame too short");
    }
    
    let header = MbapHeader {
        transaction_id: u16::from_be_bytes([data[0], data[1]]),
        protocol_id: u16::from_be_bytes([data[2], data[3]]),
        length: u16::from_be_bytes([data[4], data[5]]),
        unit_id: data[6],
    };
    
    if header.protocol_id != 0 {
        return Err("Invalid protocol ID");
    }
    
    let expected_len = 6 + header.length as usize;
    if data.len() < expected_len {
        return Err("Incomplete frame");
    }
    
    let slave_id = header.unit_id;
    let function_code = data[7];
    
    // Check for exception
    if function_code & 0x80 != 0 {
        if data.len() < 9 {
            return Err("Exception frame too short");
        }
        let exception = ExceptionCode::from_u8(data[8]).unwrap_or(ExceptionCode::SlaveDeviceFailure);
        return Ok((header, ModbusFrame::Exception(ModbusException {
            slave_id,
            function: function_code & 0x7F,
            exception,
        })));
    }
    
    let function = FunctionCode::from_u8(function_code).ok_or("Unknown function code")?;
    
    Ok((header, ModbusFrame::Response(ModbusResponse {
        slave_id,
        function,
        data: data[8..expected_len].to_vec(),
    })))
}

// ============ Helper functions ============

/// Extract register values from response data
pub fn parse_registers(data: &[u8]) -> Vec<u16> {
    data.chunks(2)
        .map(|chunk| {
            if chunk.len() == 2 {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                0
            }
        })
        .collect()
}

/// Extract coil/discrete values from response data
pub fn parse_coils(data: &[u8], count: usize) -> Vec<bool> {
    let mut result = Vec::with_capacity(count);
    for (i, &byte) in data.iter().enumerate() {
        for bit in 0..8 {
            if i * 8 + bit >= count {
                break;
            }
            result.push((byte >> bit) & 1 == 1);
        }
    }
    result
}

/// Pack coil values into bytes
pub fn pack_coils(coils: &[bool]) -> Vec<u8> {
    let mut result = Vec::with_capacity((coils.len() + 7) / 8);
    for chunk in coils.chunks(8) {
        let mut byte = 0u8;
        for (bit, &coil) in chunk.iter().enumerate() {
            if coil {
                byte |= 1 << bit;
            }
        }
        result.push(byte);
    }
    result
}

/// Format Modbus frame for display
pub fn format_frame(data: &[u8], mode: ModbusMode) -> String {
    match mode {
        ModbusMode::Rtu => {
            if data.len() < 4 {
                return "Invalid RTU frame".to_string();
            }
            format!(
                "RTU: Slave={:02X} Func={:02X} Data={} CRC={:04X}",
                data[0],
                data[1],
                hex::encode(&data[2..data.len() - 2]),
                u16::from_le_bytes([data[data.len() - 2], data[data.len() - 1]])
            )
        }
        ModbusMode::Tcp => {
            if data.len() < 8 {
                return "Invalid TCP frame".to_string();
            }
            format!(
                "TCP: Trans={:04X} Unit={:02X} Func={:02X} Data={}",
                u16::from_be_bytes([data[0], data[1]]),
                data[6],
                data[7],
                hex::encode(&data[8..])
            )
        }
        ModbusMode::Ascii => {
            format!("ASCII: {}", String::from_utf8_lossy(data))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_rtu_read_holding_registers() {
        let frame = build_rtu_request(1, FunctionCode::ReadHoldingRegisters, 0, 10);
        assert_eq!(frame.len(), 8);
        assert_eq!(frame[0], 1); // Slave ID
        assert_eq!(frame[1], 3); // Function code
    }

    #[test]
    fn test_parse_registers() {
        let data = vec![0x00, 0x64, 0x01, 0x2C]; // 100, 300
        let registers = parse_registers(&data);
        assert_eq!(registers, vec![100, 300]);
    }

    #[test]
    fn test_parse_coils() {
        let data = vec![0b00000101]; // Coils 0 and 2 are ON
        let coils = parse_coils(&data, 8);
        assert_eq!(coils[0], true);
        assert_eq!(coils[1], false);
        assert_eq!(coils[2], true);
    }

    #[test]
    fn test_pack_coils() {
        let coils = vec![true, false, true, false, false, false, false, false];
        let packed = pack_coils(&coils);
        assert_eq!(packed, vec![0b00000101]);
    }
}
