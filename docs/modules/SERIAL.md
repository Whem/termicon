# Serial Port Module

## Overview

The Serial Port module provides comprehensive support for RS-232, RS-485, and USB-Serial communication.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| Port Enumeration | ✅ | Auto-detect available ports |
| Connect/Disconnect | ✅ | Full connection lifecycle |
| Baud Rates | ✅ | 300 to 4,000,000 bps |
| Data Bits | ✅ | 5, 6, 7, 8 bits |
| Stop Bits | ✅ | 1, 1.5, 2 bits |
| Parity | ✅ | None, Odd, Even, Mark, Space |
| Flow Control | ✅ | None, Hardware (RTS/CTS), Software (XON/XOFF) |
| Modem Lines | ✅ | DTR, RTS, CTS, DSR, DCD, RI |
| Break Signal | ✅ | Send break signal |
| Auto-reconnect | ✅ | Automatic reconnection on disconnect |
| Virtual Ports | ✅ | PTY (Unix), Named Pipes (Windows) |

## Configuration

```rust
pub struct SerialConfig {
    pub port: String,           // "COM1", "/dev/ttyUSB0"
    pub baud_rate: u32,         // 300 - 4000000
    pub data_bits: DataBits,    // Five, Six, Seven, Eight
    pub stop_bits: StopBits,    // One, OnePointFive, Two
    pub parity: Parity,         // None, Odd, Even, Mark, Space
    pub flow_control: FlowControl, // None, Hardware, Software
    pub auto_reconnect: bool,   // Enable auto-reconnect
}
```

## GUI Usage

### Connection Dialog

1. Click **Serial** button in toolbar
2. Select port from dropdown (auto-detected)
3. Configure baud rate and parameters
4. Click **Connect**

### Modem Line Control

When connected to a serial port:
- **DTR** button: Toggle Data Terminal Ready
- **RTS** button: Toggle Request To Send
- **Break** button: Send break signal

### Quick Settings

The bottom status bar shows:
- Connection state (Connected/Disconnected)
- Baud rate
- TX/RX byte counters
- Modem line status (when connected)

## CLI Usage

```bash
# List available ports
termicon-cli list-ports

# Connect to serial port
termicon-cli serial --port COM3 --baud 115200

# With full configuration
termicon-cli serial \
  --port /dev/ttyUSB0 \
  --baud 9600 \
  --data-bits 8 \
  --stop-bits 1 \
  --parity none \
  --flow-control hardware
```

## Profile Support

Serial connections can be saved as profiles:

1. Connect to a port
2. Click **Save Profile** in toolbar
3. Enter a profile name
4. Profile will appear in the Profiles panel

Profile-specific commands are automatically saved and sorted by usage frequency.

## Code Examples

### Basic Connection

```rust
use termicon_core::{SerialConfig, Transport};

let config = SerialConfig::new("COM3", 115200);
let mut transport = Transport::Serial(config);
transport.connect().await?;

// Send data
transport.send(b"AT\r\n").await?;

// Receive data
let data = transport.receive().await?;

transport.disconnect().await?;
```

### With All Options

```rust
use termicon_core::{SerialConfig, Parity, FlowControl, DataBits, StopBits};

let config = SerialConfig {
    port: "COM3".to_string(),
    baud_rate: 9600,
    data_bits: DataBits::Eight,
    stop_bits: StopBits::One,
    parity: Parity::Even,
    flow_control: FlowControl::Hardware,
    auto_reconnect: true,
};
```

### Modem Line Control

```rust
// Set DTR high
transport.set_dtr(true).await?;

// Set RTS high
transport.set_rts(true).await?;

// Read modem status
let status = transport.modem_status().await?;
println!("CTS: {}, DSR: {}, DCD: {}, RI: {}", 
    status.cts, status.dsr, status.dcd, status.ri);

// Send break signal
transport.send_break(250).await?; // 250ms break
```

## File Transfer

Serial ports support file transfer protocols:

| Protocol | Status | Description |
|----------|--------|-------------|
| XMODEM | ✅ | Basic 128-byte blocks |
| XMODEM-CRC | ✅ | CRC-16 error detection |
| XMODEM-1K | ✅ | 1KB blocks |
| YMODEM | ✅ | Batch file transfer |
| ZMODEM | ✅ | Streaming with resume |

### Using File Transfer

1. Connect to serial port
2. Click **XMODEM**, **YMODEM**, or **ZMODEM** in toolbar
3. Select file to send or save location for receive
4. Progress shown in status bar

## Virtual COM Ports

Create virtual serial port pairs for testing:

### Linux/macOS (PTY)

```rust
use termicon_core::virtual_port::VirtualPort;

let (master, slave) = VirtualPort::create_pair()?;
println!("Slave port: {}", slave.path());
```

### Windows (Named Pipes)

```rust
use termicon_core::virtual_port::VirtualPort;

let port = VirtualPort::create_named_pipe("\\\\.\\pipe\\termicon_virtual")?;
```

## Troubleshooting

### Port Not Found

- Check if port exists: `ls /dev/tty*` (Linux/macOS)
- Check Device Manager (Windows)
- Try running with elevated privileges

### Permission Denied

**Linux:**
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

**Windows:**
- Run as Administrator
- Check COM port in use by another application

### No Data Received

- Verify baud rate matches device
- Check flow control settings
- Verify TX/RX are not swapped
- Check DTR/RTS requirements

### Garbled Data

- Baud rate mismatch
- Parity mismatch
- Data bits mismatch
- Electrical interference

## Platform Notes

### Windows
- Uses Windows API for serial communication
- Named pipes for virtual ports
- COM1-COM256 supported

### Linux
- Uses termios for serial configuration
- PTY for virtual ports
- Supports USB-Serial adapters

### macOS
- Similar to Linux
- USB-Serial drivers may be required
- Check System Information for port names
