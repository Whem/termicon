# TCP/Telnet Module

## Overview

The TCP/Telnet module provides raw TCP socket connections and Telnet protocol support with option negotiation.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| TCP Client | ✅ | Connect to servers |
| TCP Server | ✅ | Accept connections |
| Timeout | ✅ | Connection timeout |
| Keepalive | ✅ | TCP keepalive |
| Telnet Protocol | ✅ | IAC commands |
| Option Negotiation | ✅ | WILL/WONT/DO/DONT |
| Terminal Type | ✅ | TTYPE option |
| Echo | ✅ | Echo negotiation |
| Window Size | ✅ | NAWS option |
| Binary Mode | ✅ | Binary transmission |
| Suppress Go Ahead | ✅ | SGA option |

## TCP Configuration

```rust
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
    pub keepalive: Option<Duration>,
    pub nodelay: bool,
}
```

## Telnet Configuration

```rust
pub struct TelnetConfig {
    pub host: String,
    pub port: u16,              // Default: 23
    pub terminal_type: String,  // Default: "xterm-256color"
    pub window_size: (u16, u16),
    pub echo: bool,
    pub binary: bool,
}
```

## GUI Usage

### TCP Connection

1. Click **TCP** button in toolbar
2. Enter host address
3. Enter port number
4. Click **Connect**

### Telnet Connection

1. Click **Telnet** button in toolbar
2. Enter host address
3. Enter port (default: 23)
4. Click **Connect**

### Connection Features

- Terminal emulation (for Telnet)
- Hex view option
- Timestamps
- Local echo toggle

## CLI Usage

```bash
# Raw TCP connection
termicon-cli tcp 192.168.1.100:5000

# With timeout
termicon-cli tcp 192.168.1.100:5000 --timeout 10

# Telnet connection
termicon-cli telnet router.local

# Telnet with port
termicon-cli telnet server.com:2323
```

## Code Examples

### TCP Client

```rust
use termicon_core::{TcpConfig, Transport};

let config = TcpConfig {
    host: "192.168.1.100".to_string(),
    port: 5000,
    timeout: Duration::from_secs(10),
    keepalive: Some(Duration::from_secs(30)),
    nodelay: true,
};

let mut transport = Transport::Tcp(config);
transport.connect().await?;

// Send data
transport.send(b"Hello TCP\r\n").await?;

// Receive data
let data = transport.receive().await?;

transport.disconnect().await?;
```

### TCP Server (via Bridge)

```rust
use termicon_core::bridge::{Bridge, BridgeConfig};

let config = BridgeConfig {
    source: TransportConfig::TcpServer {
        bind_address: "0.0.0.0:5000".to_string(),
    },
    destination: TransportConfig::Serial {
        port: "COM3".to_string(),
        baud_rate: 115200,
    },
    mode: BridgeMode::Bidirectional,
    logging: true,
    stats_enabled: true,
};

let bridge = Bridge::new(config)?;
bridge.start().await?;
```

### Telnet Client

```rust
use termicon_core::{TelnetConfig, Transport};

let config = TelnetConfig {
    host: "router.local".to_string(),
    port: 23,
    terminal_type: "xterm-256color".to_string(),
    window_size: (80, 24),
    echo: true,
    binary: false,
};

let mut transport = Transport::Telnet(config);
transport.connect().await?;

// Telnet handles IAC sequences automatically
transport.send(b"show version\r\n").await?;
```

## Telnet Protocol

### IAC Commands

| Command | Code | Description |
|---------|------|-------------|
| SE | 240 | End of subnegotiation |
| NOP | 241 | No operation |
| DM | 242 | Data mark |
| BRK | 243 | Break |
| IP | 244 | Interrupt process |
| AO | 245 | Abort output |
| AYT | 246 | Are you there |
| EC | 247 | Erase character |
| EL | 248 | Erase line |
| GA | 249 | Go ahead |
| SB | 250 | Start subnegotiation |
| WILL | 251 | Will do option |
| WONT | 252 | Won't do option |
| DO | 253 | Do option |
| DONT | 254 | Don't do option |
| IAC | 255 | IAC escape |

### Common Options

| Option | Code | Description |
|--------|------|-------------|
| ECHO | 1 | Echo |
| SGA | 3 | Suppress Go Ahead |
| STATUS | 5 | Status |
| TTYPE | 24 | Terminal Type |
| NAWS | 31 | Window Size |
| TSPEED | 32 | Terminal Speed |
| LINEMODE | 34 | Line Mode |
| NEW_ENVIRON | 39 | Environment |

### Option Negotiation

```rust
// Enable option
telnet.send_will(TelnetOption::TerminalType).await?;

// Disable option
telnet.send_wont(TelnetOption::Echo).await?;

// Request option
telnet.send_do(TelnetOption::SuppressGoAhead).await?;

// Refuse option
telnet.send_dont(TelnetOption::Linemode).await?;
```

### Subnegotiation

```rust
// Terminal type subnegotiation
telnet.send_subnegotiation(
    TelnetOption::TerminalType,
    &[0, b'x', b't', b'e', b'r', b'm']
).await?;

// Window size subnegotiation
let width: u16 = 80;
let height: u16 = 24;
telnet.send_subnegotiation(
    TelnetOption::WindowSize,
    &[
        (width >> 8) as u8, (width & 0xFF) as u8,
        (height >> 8) as u8, (height & 0xFF) as u8,
    ]
).await?;
```

## Profile Support

TCP and Telnet connections can be saved as profiles:

1. Connect to server
2. Click **Save Profile** in toolbar
3. Enter profile name
4. Access later from Profiles panel

## Advanced Features

### Connection Pooling

```rust
// Create connection pool for multiple connections
let pool = TcpPool::new(max_connections);

let conn1 = pool.get_connection(&config).await?;
let conn2 = pool.get_connection(&config).await?;
```

### Reconnection

```rust
// Auto-reconnect on disconnect
let config = TcpConfig {
    // ... other config
    auto_reconnect: true,
    reconnect_delay: Duration::from_secs(5),
    max_reconnect_attempts: 10,
};
```

## Troubleshooting

### Connection Refused

- Verify server is running
- Check port number
- Check firewall rules
- Verify network connectivity

### Connection Timeout

- Server may be unreachable
- Firewall blocking connection
- Try increasing timeout value
- Check DNS resolution

### Telnet Negotiation Issues

- Try disabling problematic options
- Check server compatibility
- Verify terminal type
- Enable debug logging

### Data Corruption

- Check for IAC byte escaping in Telnet
- Verify binary mode if needed
- Check line ending conversion
- Verify character encoding

## Security Notes

- TCP connections are unencrypted
- Telnet sends passwords in cleartext
- Consider SSH for secure connections
- Use VPN for sensitive data

## Platform Notes

### Windows

- Windows Firewall may block connections
- Admin rights may be needed for low ports

### Linux

- Ports < 1024 require root
- Check SELinux policies
- iptables/nftables rules

### macOS

- Check firewall settings
- Application sandboxing restrictions
