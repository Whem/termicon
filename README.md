# Termicon

**Professional Multi-Protocol Terminal Application**

Termicon is a comprehensive serial port and network terminal application built in Rust with a modern GUI. It supports multiple connection types, advanced data processing, and automation features.

## Features

### Connection Types
- **Serial Port (UART)** - Full RS-232/RS-485 support with all baud rates, DTR/RTS control
- **TCP/IP** - Raw TCP socket connections
- **Telnet** - Full Telnet protocol with option negotiation
- **SSH** - Secure Shell with password/key auth, SFTP, jump proxy
- **Bluetooth LE** - BLE GATT client with Nordic UART Service (NUS) support

### Terminal Emulation
- VT100/VT102/VT220 terminal emulation
- ANSI escape sequence support
- 256-color and true color support
- Full cursor control and screen manipulation
- Mouse reporting (SGR mode)

### Data Processing
- **Hex/ASCII/Mixed views** - Multiple data display formats
- **Timestamps** - Automatic timestamp injection
- **Framing** - SLIP, COBS, STX/ETX, length-prefixed protocols
- **Checksums** - CRC-16 (Modbus/CCITT/XMODEM), CRC-32, XOR, LRC, Fletcher

### Industrial Protocols
- **Modbus RTU/TCP** - Full function code support (1-16)
- **Protocol DSL** - YAML/JSON protocol definitions
- **Packet abstraction** - Field-level packet handling

### File Transfer
- **XMODEM** - 128 byte and 1K variants with CRC
- **YMODEM** - Batch mode with file info
- **ZMODEM** - Auto-start, streaming transfer
- **SFTP** - Secure file transfer over SSH

### Advanced Features
- **Network Bridge** - Serial to TCP bidirectional forwarding
- **Virtual COM Ports** - PTY (Unix) / Named Pipes (Windows)
- **Profiles** - Save and load connection configurations
- **Macros (M1-M24)** - Quick macro buttons like classic terminals
- **Commands** - Profile-specific command history sorted by usage
- **Triggers** - Pattern matching with auto-response
- **Session Logging** - Configurable log formats
- **Real-time Charts** - Data visualization for sensor values
- **Deterministic Mode** - Reproducible test runs
- **Fuzzing/Testing** - Protocol robustness testing
- **Adaptive Automation** - Feedback control rules
- **External API** - REST/WebSocket control

### User Interface
- Modern dark/light themes
- Tab-based multi-session support
- Side panel with profiles, commands, history, charts
- Keyboard shortcuts (F1-F12 for macros)
- Multi-language support (English, Hungarian)
- SFTP file browser
- Real-time search in output
- Macro recording and playback

## 🚀 Installation

### Prerequisites
- Rust 1.70 or later
- Windows/Linux/macOS

### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/termicon.git
cd termicon

# Build release version
cargo build --release

# Run
./target/release/termicon
```

### Dependencies

Key dependencies include:
- `eframe/egui` - Modern Rust GUI framework
- `tokio` - Async runtime
- `serialport` - Cross-platform serial port library
- `ssh2` - SSH client library
- `serde` - Serialization framework

## 📖 Usage

### Quick Start

1. Launch Termicon
2. Click a connection button (Serial, TCP, Telnet, SSH)
3. Configure connection parameters
4. Click Connect

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+D | Disconnect |
| Ctrl+Shift+C | Copy |
| Ctrl+Shift+V | Paste |
| F1-F12 | Execute snippets |

### CLI Mode

```bash
# List serial ports
termicon list-ports

# Connect to serial port
termicon serial --port COM3 --baud 115200

# Connect via TCP
termicon tcp --host 192.168.1.100 --port 23

# Connect via SSH
termicon ssh --host example.com --user admin
```

## 📁 Project Structure

```
termicon/
├── src/
│   ├── core/
│   │   ├── bridge/         # Network bridge (Serial↔TCP)
│   │   ├── chart/          # Real-time data charting
│   │   ├── codec/          # Data encoding/decoding
│   │   ├── logger.rs       # Session logging
│   │   ├── macro_recorder.rs # Macro recording/playback
│   │   ├── plugin/         # Plugin system
│   │   ├── profile/        # Connection profiles
│   │   ├── protocol/       # Industrial protocols (Modbus, framing, checksums)
│   │   ├── session.rs      # Session management
│   │   ├── snippet/        # Command snippets/macros
│   │   ├── terminal/       # VT100/VT220 terminal emulation
│   │   ├── transfer/       # File transfer (X/Y/ZMODEM)
│   │   ├── transport/      # Connection transports (Serial/TCP/Telnet/SSH)
│   │   ├── trigger.rs      # Auto-response triggers
│   │   └── virtual_port/   # Virtual COM ports (PTY/Named Pipes)
│   ├── gui/
│   │   ├── app.rs          # Main application
│   │   ├── chart_panel.rs  # Chart UI component
│   │   ├── session_tab.rs  # Tab management
│   │   └── sftp_panel.rs   # SFTP file browser
│   ├── config/             # Configuration management
│   ├── i18n/               # Internationalization
│   └── utils/              # Utility functions
├── locales/                # Translation files (EN/HU)
├── assets/                 # Icons and resources
├── docs/                   # Documentation
└── benches/                # Performance benchmarks
```

## 🔧 Configuration

Configuration files are stored in:
- Windows: `%APPDATA%\termicon\Termicon\`
- Linux: `~/.config/termicon/`
- macOS: `~/Library/Application Support/com.termicon.Termicon/`

### Files
- `config.toml` - Main configuration
- `profiles.json` - Saved connection profiles
- `snippets.json` - Saved command snippets
- `triggers.json` - Auto-response triggers

## 🔌 Plugin System

Termicon supports plugins for:
- Protocol decoders
- Custom views
- Data processors

Create a plugin directory in the plugins folder with:
- `plugin.json` - Plugin manifest
- Plugin source files

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- [egui](https://github.com/emilk/egui) - Immediate mode GUI
- [serialport-rs](https://github.com/serialport/serialport-rs) - Serial port library
- [ssh2-rs](https://github.com/alexcrichton/ssh2-rs) - SSH bindings

## 📊 Feature Status

| Feature | Status |
|---------|--------|
| Serial Port | ✅ Complete |
| TCP/IP | ✅ Complete |
| Telnet | ✅ Complete |
| SSH | ✅ Complete |
| Terminal Emulation | ✅ Complete (VT100/VT220, 256+true color, mouse) |
| Modbus | ✅ Complete (RTU/TCP) |
| XMODEM/YMODEM | ✅ Complete |
| ZMODEM | ✅ Complete |
| Bridge | ✅ Complete |
| Virtual Ports | ✅ Complete |
| Profiles | ✅ Complete |
| Snippets/Macros | ✅ Complete |
| Triggers | ✅ Complete |
| Charts | ✅ Complete |
| SFTP Browser | ✅ Complete |
| Macro Recording | ✅ Complete |
| Dark/Light Themes | ✅ Complete |
| Search in Output | ✅ Complete |
| Bluetooth LE | ✅ Complete (GATT, NUS) |
