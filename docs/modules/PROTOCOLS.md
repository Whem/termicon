# Protocols Module

## Overview

The Protocols module provides industrial protocol support, framing, checksum calculation, and custom protocol definitions.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| Modbus RTU | ✅ | Serial Modbus |
| Modbus TCP | ✅ | Ethernet Modbus |
| Modbus ASCII | ✅ | ASCII framing |
| SLIP Framing | ✅ | RFC 1055 |
| COBS Framing | ✅ | Byte stuffing |
| STX/ETX Framing | ✅ | Start/End markers |
| Length-Prefix | ✅ | Header with length |
| CRC-16 (Multiple) | ✅ | Various CRC-16 |
| CRC-32 | ✅ | Standard CRC-32 |
| XOR Checksum | ✅ | Simple XOR |
| LRC | ✅ | Modbus ASCII |
| Fletcher | ✅ | Fletcher-16/32 |
| Protocol DSL | ✅ | YAML/JSON definitions |
| Packet Abstraction | ✅ | Generic packets |

## Modbus

### RTU Mode

```rust
use termicon_core::protocol::modbus::{ModbusRtu, FunctionCode};

let modbus = ModbusRtu::new();

// Read holding registers
let request = modbus.read_holding_registers(1, 0, 10)?;
transport.send(&request).await?;

let response = transport.receive().await?;
let registers = modbus.parse_read_registers_response(&response)?;

// Write single register
let request = modbus.write_single_register(1, 100, 0x1234)?;
transport.send(&request).await?;
```

### TCP Mode

```rust
use termicon_core::protocol::modbus::{ModbusTcp, FunctionCode};

let modbus = ModbusTcp::new();

// Same function codes, different framing
let request = modbus.read_coils(1, 0, 16)?;
transport.send(&request).await?;
```

### Function Codes

| Code | Name | Description |
|------|------|-------------|
| 0x01 | Read Coils | Read discrete outputs |
| 0x02 | Read Discrete Inputs | Read discrete inputs |
| 0x03 | Read Holding Registers | Read output registers |
| 0x04 | Read Input Registers | Read input registers |
| 0x05 | Write Single Coil | Write single output |
| 0x06 | Write Single Register | Write single register |
| 0x0F | Write Multiple Coils | Write multiple outputs |
| 0x10 | Write Multiple Registers | Write multiple registers |

## Framing Protocols

### SLIP (RFC 1055)

```rust
use termicon_core::protocol::framing::Slip;

let slip = Slip::new();

// Encode frame
let data = b"Hello World";
let frame = slip.encode(data);

// Decode frame
let decoded = slip.decode(&frame)?;
```

### COBS (Consistent Overhead Byte Stuffing)

```rust
use termicon_core::protocol::framing::Cobs;

let cobs = Cobs::new();

// Encode - guarantees no zero bytes in payload
let frame = cobs.encode(data);

// Decode
let decoded = cobs.decode(&frame)?;
```

### STX/ETX Framing

```rust
use termicon_core::protocol::framing::StxEtx;

let framing = StxEtx::new(0x02, 0x03); // STX=0x02, ETX=0x03

let frame = framing.encode(data);
let decoded = framing.decode(&frame)?;
```

### Length-Prefixed

```rust
use termicon_core::protocol::framing::LengthPrefix;

// 2-byte big-endian length prefix
let framing = LengthPrefix::new(2, ByteOrder::BigEndian);

let frame = framing.encode(data);
let decoded = framing.decode(&frame)?;
```

## Checksums

### CRC-16 Variants

```rust
use termicon_core::protocol::checksum::{Crc16, Crc16Algorithm};

// Modbus CRC-16
let crc = Crc16::new(Crc16Algorithm::Modbus);
let checksum = crc.compute(data);

// CCITT (X.25)
let crc = Crc16::new(Crc16Algorithm::Ccitt);
let checksum = crc.compute(data);

// XMODEM
let crc = Crc16::new(Crc16Algorithm::Xmodem);
let checksum = crc.compute(data);

// USB
let crc = Crc16::new(Crc16Algorithm::Usb);
let checksum = crc.compute(data);
```

### CRC-32

```rust
use termicon_core::protocol::checksum::Crc32;

let crc = Crc32::new();
let checksum = crc.compute(data);
```

### XOR Checksum

```rust
use termicon_core::protocol::checksum::Xor;

let checksum = Xor::compute(data);
```

### LRC (Longitudinal Redundancy Check)

```rust
use termicon_core::protocol::checksum::Lrc;

// Used in Modbus ASCII
let lrc = Lrc::compute(data);
```

### Fletcher

```rust
use termicon_core::protocol::checksum::Fletcher;

let fletcher16 = Fletcher::fletcher16(data);
let fletcher32 = Fletcher::fletcher32(data);
```

## Protocol DSL

Define custom protocols in YAML or JSON:

### YAML Definition

```yaml
name: MyProtocol
version: "1.0"
endian: big

fields:
  - name: header
    type: u8
    value: 0xAA
  
  - name: length
    type: u16
    compute: payload.length
  
  - name: command
    type: u8
    enum:
      0x01: Read
      0x02: Write
      0x03: Reset
  
  - name: payload
    type: bytes
    length: length
  
  - name: checksum
    type: u16
    compute: crc16_modbus(header..payload)
```

### JSON Definition

```json
{
  "name": "MyProtocol",
  "version": "1.0",
  "endian": "big",
  "fields": [
    {
      "name": "header",
      "type": "u8",
      "value": 170
    },
    {
      "name": "length",
      "type": "u16"
    },
    {
      "name": "payload",
      "type": "bytes",
      "length_field": "length"
    }
  ]
}
```

### Using Protocol DSL

```rust
use termicon_core::protocol::ProtocolDsl;

// Load protocol definition
let protocol = ProtocolDsl::from_yaml(yaml_string)?;
// or
let protocol = ProtocolDsl::from_json(json_string)?;

// Parse incoming data
let packet = protocol.parse(raw_data)?;
println!("Command: {:?}", packet.get("command"));

// Build outgoing packet
let data = protocol.build()
    .set("command", 0x01)
    .set("payload", b"data")
    .build()?;
```

## Packet Abstraction

Generic packet handling:

```rust
use termicon_core::packet::{Packet, Direction};

let packet = Packet {
    timestamp: SystemTime::now(),
    direction: Direction::Received,
    data: raw_bytes,
    protocol: Some("Modbus RTU".to_string()),
    metadata: HashMap::new(),
};

// Add metadata
packet.metadata.insert("slave_id".to_string(), json!(1));
packet.metadata.insert("function".to_string(), json!("Read Holding Registers"));
```

## GUI Integration

### Protocol Decoding

The GUI automatically decodes known protocols:
- Modbus frames show slave ID, function, data
- Framed data shows raw and decoded views
- Checksums are validated and highlighted

### Hex View

In hex view mode:
- Protocol fields are color-coded
- Hover shows field descriptions
- Invalid checksums highlighted in red

## CLI Usage

```bash
# Modbus query
termicon-cli modbus --port COM3 --slave 1 --function 3 --address 0 --count 10

# CRC calculation
termicon-cli crc --algorithm modbus --data "48454C4C4F"

# Frame data
termicon-cli frame --type slip --encode "Hello World"
termicon-cli frame --type slip --decode "C0..."
```

## Extending Protocols

### Custom Decoder

```rust
use termicon_core::protocol::ProtocolDecoder;

pub struct MyDecoder;

impl ProtocolDecoder for MyDecoder {
    fn name(&self) -> &str { "MyProtocol" }
    
    fn can_decode(&self, data: &[u8]) -> bool {
        data.len() >= 4 && data[0] == 0xAA
    }
    
    fn decode(&self, data: &[u8]) -> Result<DecodedPacket, DecodeError> {
        // Parse the packet
        Ok(DecodedPacket {
            protocol: "MyProtocol".to_string(),
            fields: vec![
                ("header", "0xAA"),
                ("data", hex::encode(&data[1..])),
            ],
        })
    }
}
```

### Register Custom Decoder

```rust
use termicon_core::protocol::register_decoder;

register_decoder(Box::new(MyDecoder));
```

## Troubleshooting

### CRC Mismatch

- Verify polynomial matches device
- Check byte order (LSB/MSB first)
- Verify initial value
- Check final XOR value

### Framing Errors

- Verify frame boundaries
- Check escape sequences
- Verify length field byte order
- Check for truncated frames

### Modbus Issues

- Verify slave ID
- Check function code support
- Verify address range
- Check timeout settings
