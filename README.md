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
- Sixel graphics support

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
- **Kermit** - Full protocol with quoting and checksums
- **SFTP** - Secure file transfer over SSH

### Advanced Features
- **Network Bridge** - Serial to TCP bidirectional forwarding
- **Virtual COM Ports** - PTY (Unix) / Named Pipes (Windows)
- **Profiles** - Save and load connection configurations
- **Macros (M1-M24)** - Quick macro buttons like classic terminals
- **Commands** - Profile-specific command history sorted by usage
- **Triggers** - Pattern matching with auto-response, multi-pattern groups, trigger chains
- **Session Logging** - Configurable log formats
- **Real-time Charts** - Data visualization with markers and PNG/SVG export
- **Deterministic Mode** - Reproducible test runs
- **Fuzzing/Testing** - Packet/timing fuzzing for robustness testing
- **Adaptive Automation** - Feedback control rules
- **External API** - REST/WebSocket control
- **Batch Operations** - Multi-session commands with parallel execution
- **Workspace** - Save and restore complete session states

### User Interface
- Modern dark/light themes with 12+ color schemes
- Tab-based multi-session support with split views
- Side panel with profiles, commands, history, charts
- Comprehensive keyboard shortcuts (F1-F24 for macros)
- Command palette (Ctrl+K) for quick access
- Multi-language support (English, Hungarian)
- SFTP file browser
- Real-time search in output
- Macro recording and playback
- High contrast mode and font scaling for accessibility
- Font configuration UI

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
| Ctrl+K | Command palette |
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+D | Disconnect |
| Ctrl+L | Clear screen |
| Ctrl+F | Search |
| F1-F12 | Execute M1-M12 macros |
| Shift+F1-F12 | Execute M13-M24 macros |

### CLI Mode

```bash
# List serial ports
termicon-cli list-ports

# Connect to serial port
termicon-cli serial --port COM3 --baud 115200

# Connect via TCP
termicon-cli tcp --host 192.168.1.100 --port 23

# Connect via SSH
termicon-cli ssh --host example.com --user admin

# Pipe support (stdin/stdout)
echo "AT" | termicon-cli serial --port COM3

# JSON output for scripting
termicon-cli serial --port COM3 --output-format json
```

Exit codes are standardized for scripting (0=success, 3=connection failed, etc.).

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
| Terminal Emulation | ✅ Complete (VT100/VT220, 256+true color, mouse, sixel) |
| Modbus | ✅ Complete (RTU/TCP with monitoring) |
| NMEA 0183 | ✅ Complete (GPS/navigation sentences) |
| XMODEM/YMODEM/ZMODEM | ✅ Complete |
| Kermit | ✅ Complete |
| Bridge | ✅ Complete |
| Virtual Ports | ✅ Complete |
| Profiles | ✅ Complete |
| Snippets/Macros | ✅ Complete (M1-M24) |
| Triggers | ✅ Complete (chains, multi-pattern) |
| Charts | ✅ Complete (markers, PNG/SVG export) |
| SFTP Browser | ✅ Complete |
| Macro Recording | ✅ Complete |
| Dark/Light Themes | ✅ Complete (12+ schemes) |
| Split Views | ✅ Complete |
| Command Palette | ✅ Complete |
| Keyboard Navigation | ✅ Complete |
| Accessibility | ✅ Complete (high contrast, font scaling) |
| Batch Operations | ✅ Complete |
| Workspace Save/Restore | ✅ Complete |
| CLI Pipe Support | ✅ Complete |
| Bluetooth LE | ✅ Complete (GATT, NUS) |

**Overall Completion: ~85%**

### Recent Fixes (v0.1.1)
- Language switching now works properly
- Fixed emoji characters showing as squares
- Profile double-click connects directly
- Command double-click inserts into input field

See [ROADMAP.md](docs/ROADMAP.md) for detailed progress.
