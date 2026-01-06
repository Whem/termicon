# Termicon - Full Feature Matrix

**Last Updated:** 2026-01-06

## Legend
- ✅ Complete and working
- 🔄 Partially implemented
- ❌ Not implemented

---

## 1. TRANSPORT LAYER (Connections)

### 1.1 Serial Port
| Feature | Status | File |
|---------|--------|------|
| Port listing | ✅ | `transport/serial.rs` |
| Connect/disconnect | ✅ | `transport/serial.rs` |
| Baud rate (300-921600+) | ✅ | `transport/serial.rs` |
| Data bits (5-8) | ✅ | `transport/serial.rs` |
| Stop bits (1-2) | ✅ | `transport/serial.rs` |
| Parity (None/Odd/Even) | ✅ | `transport/serial.rs` |
| Flow control (None/HW/SW) | ✅ | `transport/serial.rs` |
| DTR/RTS manual control | ✅ | `transport/serial.rs` |
| Modem lines | ✅ | `transport/serial.rs` |
| Break signal | ✅ | `transport/serial.rs` |
| Auto-reconnect | ✅ | `transport/serial.rs` |

### 1.2 TCP
| Feature | Status | File |
|---------|--------|------|
| TCP client | ✅ | `transport/tcp.rs` |
| Timeout | ✅ | `transport/tcp.rs` |
| TCP server | ✅ | `bridge/mod.rs` |

### 1.3 Telnet
| Feature | Status | File |
|---------|--------|------|
| Telnet client | ✅ | `transport/telnet.rs` |
| IAC commands | ✅ | `transport/telnet.rs` |
| Terminal type | ✅ | `transport/telnet.rs` |
| Echo negotiation | ✅ | `transport/telnet.rs` |

### 1.4 SSH
| Feature | Status | File |
|---------|--------|------|
| SSH-2 connection | ✅ | `transport/ssh.rs` |
| Password auth | ✅ | `transport/ssh.rs` |
| Key-based auth | ✅ | `transport/ssh.rs` |
| Key passphrase | ✅ | `transport/ssh.rs` |
| SSH Agent | 🔄 | `transport/ssh.rs` |
| PTY allocation | ✅ | `transport/ssh.rs` |
| PTY resize | ✅ | `transport/ssh.rs` |
| Exec command | ✅ | `transport/ssh.rs` |
| Jump Host (ProxyJump) | ✅ | `gui/app.rs` |
| Local port forward (-L) | ✅ | `gui/app.rs` |
| Remote port forward (-R) | ✅ | `gui/app.rs` |
| SFTP | ✅ | `transport/ssh.rs` |
| Compression | ✅ | `gui/app.rs` |
| Keepalive | ✅ | `gui/app.rs` |
| Connection timeout | ✅ | `gui/app.rs` |
| Terminal type selection | ✅ | `gui/app.rs` |
| X11 forwarding | ✅ | `gui/app.rs` |
| Agent forwarding | ✅ | `gui/app.rs` |
| Password save option | ✅ | `gui/profiles.rs` |
| Auto-connect | ✅ | `gui/app.rs` |

### 1.5 Bluetooth
| Feature | Status | File |
|---------|--------|------|
| BLE Scan/Connect | ✅ | `transport/bluetooth.rs` |
| GATT Services | ✅ | `transport/bluetooth.rs` |
| BLE Notifications | ✅ | `transport/bluetooth.rs` |
| Nordic UART Service | ✅ | `transport/bluetooth.rs` |
| RFCOMM/SPP | 🔄 | - |

---

## 2. DISPLAY AND UI

### 2.1 Terminal Emulation
| Feature | Status | File |
|---------|--------|------|
| VT100/VT102/VT220 | ✅ | `terminal/parser.rs` |
| ANSI colors (16) | ✅ | `terminal/color.rs` |
| 256 colors | ✅ | `terminal/color.rs` |
| True color (24-bit) | ✅ | `terminal/color.rs` |
| Cursor movement | ✅ | `terminal/screen.rs` |
| Scroll region | ✅ | `terminal/screen.rs` |
| Screen buffer | ✅ | `terminal/screen.rs` |
| Mouse reporting | ✅ | `terminal/mod.rs` |
| Unicode/UTF-8 | ✅ | - |

### 2.2 GUI (egui/eframe)
| Feature | Status | File |
|---------|--------|------|
| Main window | ✅ | `gui/app.rs` |
| Toolbar | ✅ | `gui/app.rs` |
| Status bar | ✅ | `gui/app.rs` |
| Connection dialogs | ✅ | `gui/app.rs` |
| Terminal view | ✅ | `gui/app.rs` |
| Input field + history | ✅ | `gui/app.rs` |
| Tabs | ✅ | `gui/session_tab.rs` |
| Hex view | ✅ | `gui/session_tab.rs` |
| Timestamps | ✅ | `gui/session_tab.rs` |
| Local echo | ✅ | `gui/session_tab.rs` |
| Dark theme | ✅ | `gui/app.rs` |
| Light theme | ✅ | `gui/app.rs` |
| Search in output | ✅ | `gui/session_tab.rs` |
| Chart Panel | ✅ | `gui/chart_panel.rs` |
| SFTP Panel | ✅ | `gui/sftp_panel.rs` |
| Side panel | ✅ | `gui/app.rs` |
| Profiles panel | ✅ | `gui/profiles.rs` |
| Macros panel (M1-M24) | ✅ | `gui/macros_panel.rs` |
| Command palette | ✅ | `gui/command_palette.rs` |
| BLE Inspector | ✅ | `gui/ble_panel.rs` |
| Application icon | ✅ | `main.rs` |

### 2.3 CLI
| Feature | Status | File |
|---------|--------|------|
| Command-line args | ✅ | `bin/cli.rs` |
| list-ports | ✅ | `bin/cli.rs` |
| Subcommands | ✅ | `bin/cli.rs` |
| Headless mode | ✅ | `bin/cli.rs` |

---

## 3. PROTOCOLS AND DECODING

### 3.1 Framing
| Feature | Status | File |
|---------|--------|------|
| Line-based | ✅ | `codec/mod.rs` |
| SLIP (RFC 1055) | ✅ | `protocol/framing.rs` |
| COBS | ✅ | `protocol/framing.rs` |
| STX/ETX | ✅ | `protocol/framing.rs` |
| Length-prefixed | ✅ | `protocol/framing.rs` |

### 3.2 Checksum
| Feature | Status | File |
|---------|--------|------|
| CRC-16 Modbus | ✅ | `protocol/checksum.rs` |
| CRC-16 CCITT | ✅ | `protocol/checksum.rs` |
| CRC-16 XMODEM | ✅ | `protocol/checksum.rs` |
| CRC-32 | ✅ | `protocol/checksum.rs` |
| XOR | ✅ | `protocol/checksum.rs` |
| LRC | ✅ | `protocol/checksum.rs` |
| Fletcher 16/32 | ✅ | `protocol/checksum.rs` |

### 3.3 Modbus
| Feature | Status | File |
|---------|--------|------|
| RTU framing | ✅ | `protocol/modbus.rs` |
| TCP framing | ✅ | `protocol/modbus.rs` |
| FC 1-16 | ✅ | `protocol/modbus.rs` |
| Exception handling | ✅ | `protocol/modbus.rs` |

### 3.4 Protocol DSL
| Feature | Status | File |
|---------|--------|------|
| YAML/JSON definitions | ✅ | `protocol_dsl.rs` |
| Field definitions | ✅ | `protocol_dsl.rs` |
| Packet abstraction | ✅ | `packet.rs` |

---

## 4. FILE TRANSFER

### 4.1 Serial Protocols
| Feature | Status | File |
|---------|--------|------|
| XMODEM | ✅ | `transfer/mod.rs` |
| XMODEM-CRC | ✅ | `transfer/mod.rs` |
| XMODEM-1K | ✅ | `transfer/mod.rs` |
| YMODEM | ✅ | `transfer/mod.rs` |
| ZMODEM | ✅ | `transfer/mod.rs` |

### 4.2 SSH File Transfer
| Feature | Status | File |
|---------|--------|------|
| SFTP list | ✅ | `transport/ssh.rs` |
| SFTP read/write | ✅ | `transport/ssh.rs` |
| SFTP delete/mkdir | ✅ | `transport/ssh.rs` |
| SFTP GUI | ✅ | `gui/sftp_panel.rs` |

---

## 5. LOGGING AND TRIGGERS

### 5.1 Session Logging
| Feature | Status | File |
|---------|--------|------|
| SessionLogger | ✅ | `logger.rs` |
| Raw/Text/Hex log | ✅ | `logger.rs` |
| Timestamps | ✅ | `logger.rs` |
| Direction (TX/RX) | ✅ | `logger.rs` |
| Log to file | ✅ | `logger.rs` |

### 5.2 Trigger System
| Feature | Status | File |
|---------|--------|------|
| Exact pattern | ✅ | `trigger.rs` |
| Text match | ✅ | `trigger.rs` |
| Regex trigger | ✅ | `trigger.rs` |
| Hex pattern | ✅ | `trigger.rs` |
| Auto-response | ✅ | `trigger.rs` |
| TriggerManager | ✅ | `trigger.rs` |

---

## 6. AUTOMATION

### 6.1 Snippets/Macros
| Feature | Status | File |
|---------|--------|------|
| Command snippet | ✅ | `snippet/mod.rs` |
| Script (multi-line) | ✅ | `snippet/mod.rs` |
| KeySequence | ✅ | `snippet/mod.rs` |
| Binary (hex) | ✅ | `snippet/mod.rs` |
| SnippetManager | ✅ | `snippet/mod.rs` |
| Quick Macros (M1-M24) | ✅ | `macros.rs` |

### 6.2 Macro Recording
| Feature | Status | File |
|---------|--------|------|
| MacroRecorder | ✅ | `macro_recorder.rs` |
| MacroPlayer | ✅ | `macro_recorder.rs` |
| MacroAction types | ✅ | `macro_recorder.rs` |
| SpecialKey | ✅ | `macro_recorder.rs` |
| Timing capture | ✅ | `macro_recorder.rs` |
| Loop playback | ✅ | `macro_recorder.rs` |

### 6.3 Plugin/Scripting
| Feature | Status | File |
|---------|--------|------|
| PluginManager | ✅ | `plugin/mod.rs` |
| Plugin scan/load | ✅ | `plugin/mod.rs` |
| ProtocolDecoder | ✅ | `plugin/mod.rs` |
| Lua scripting | ❌ | - |

---

## 7. CONFIGURATION

### 7.1 Session Profiles
| Feature | Status | File |
|---------|--------|------|
| Profile struct | ✅ | `profile/mod.rs` |
| SerialProfile | ✅ | `profile/mod.rs` |
| TcpProfile | ✅ | `profile/mod.rs` |
| SshProfile | ✅ | `profile/mod.rs` |
| ProfileManager | ✅ | `profile/mod.rs` |
| Profile-specific commands | ✅ | `gui/profiles.rs` |
| Usage tracking | ✅ | `gui/profiles.rs` |

### 7.2 Global Settings
| Feature | Status | File |
|---------|--------|------|
| AppConfig | ✅ | `config/mod.rs` |
| Config persist | ✅ | `config/mod.rs` |
| i18n (EN/HU) | ✅ | `i18n/` |
| Theme switching | ✅ | `gui/app.rs` |

---

## 8. ADVANCED FEATURES

### 8.1 Bridge/Router
| Feature | Status | File |
|---------|--------|------|
| Serial → TCP Server | ✅ | `bridge/mod.rs` |
| Serial → TCP Client | ✅ | `bridge/mod.rs` |
| Bidirectional | ✅ | `bridge/mod.rs` |
| BridgeStats | ✅ | `bridge/mod.rs` |

### 8.2 Virtual Ports
| Feature | Status | File |
|---------|--------|------|
| PTY (Unix) | ✅ | `virtual_port/mod.rs` |
| Named Pipe (Windows) | ✅ | `virtual_port/mod.rs` |
| Loopback | ✅ | `virtual_port/mod.rs` |

### 8.3 Chart/Graph
| Feature | Status | File |
|---------|--------|------|
| Real-time plot | ✅ | `chart/mod.rs` |
| Multi-channel | ✅ | `chart/mod.rs` |
| CSV parser | ✅ | `chart/parser.rs` |
| JSON parser | ✅ | `chart/parser.rs` |
| Key-Value parser | ✅ | `chart/parser.rs` |
| Regex parser | ✅ | `chart/parser.rs` |
| Export CSV | ✅ | `chart/mod.rs` |
| Chart GUI | ✅ | `gui/chart_panel.rs` |

### 8.4 Deterministic Mode
| Feature | Status | File |
|---------|--------|------|
| Fixed random seed | ✅ | `deterministic.rs` |
| Timing normalization | ✅ | `deterministic.rs` |
| Reproducible runs | ✅ | `deterministic.rs` |
| Session export | ✅ | `deterministic.rs` |

### 8.5 Fuzzing/Testing
| Feature | Status | File |
|---------|--------|------|
| Packet fuzzer | ✅ | `fuzzing.rs` |
| Timing fuzzer | ✅ | `fuzzing.rs` |
| Boundary values | ✅ | `fuzzing.rs` |
| Protocol patterns | ✅ | `fuzzing.rs` |

### 8.6 Routing Graph
| Feature | Status | File |
|---------|--------|------|
| Node definitions | ✅ | `routing.rs` |
| Edge connections | ✅ | `routing.rs` |
| DOT export | ✅ | `routing.rs` |
| Path finding | ✅ | `routing.rs` |

### 8.7 Adaptive Automation
| Feature | Status | File |
|---------|--------|------|
| Metric tracking | ✅ | `adaptive.rs` |
| Feedback rules | ✅ | `adaptive.rs` |
| Auto-adjustment | ✅ | `adaptive.rs` |

### 8.8 Resource Arbitration
| Feature | Status | File |
|---------|--------|------|
| Session priority | ✅ | `arbitration.rs` |
| Rate limiter | ✅ | `arbitration.rs` |
| Fairness policy | ✅ | `arbitration.rs` |

### 8.9 Experiment Mode
| Feature | Status | File |
|---------|--------|------|
| Parameter sweep | ✅ | `experiment.rs` |
| Result analysis | ✅ | `experiment.rs` |
| Heatmap data | ✅ | `experiment.rs` |

### 8.10 Explain Mode
| Feature | Status | File |
|---------|--------|------|
| Root cause hints | ✅ | `explain.rs` |
| Diagnostic rules | ✅ | `explain.rs` |
| Troubleshooting | ✅ | `explain.rs` |

### 8.11 Collaborative Features
| Feature | Status | File |
|---------|--------|------|
| Workspace model | ✅ | `collaborative.rs` |
| Profile sharing | ✅ | `collaborative.rs` |
| User roles | ✅ | `collaborative.rs` |

### 8.12 External API
| Feature | Status | File |
|---------|--------|------|
| REST endpoints | ✅ | `external_api.rs` |
| WebSocket messages | ✅ | `external_api.rs` |
| Trigger outputs | ✅ | `external_api.rs` |
| OpenAPI spec | ✅ | `external_api.rs` |

### 8.13 Session Replay
| Feature | Status | File |
|---------|--------|------|
| Event recording | ✅ | `replay.rs` |
| Playback control | ✅ | `replay.rs` |
| Speed control | ✅ | `replay.rs` |

### 8.14 Virtual Device
| Feature | Status | File |
|---------|--------|------|
| Device simulator | ✅ | `simulator.rs` |
| Script-based | ✅ | `simulator.rs` |
| State machine | ✅ | `simulator.rs` |

### 8.15 Credential Vault
| Feature | Status | File |
|---------|--------|------|
| Secure storage | ✅ | `vault.rs` |
| Encryption | ✅ | `vault.rs` |
| Key management | ✅ | `vault.rs` |

### 8.16 Knowledge Base
| Feature | Status | File |
|---------|--------|------|
| Device database | ✅ | `knowledge.rs` |
| Inline hints | ✅ | `knowledge.rs` |
| Documentation links | ✅ | `knowledge.rs` |

---

## SUMMARY

| Category | Complete | Partial | Missing |
|----------|----------|---------|---------|
| Transport | 46 | 2 | 0 |
| UI/Display | 41 | 0 | 0 |
| Protocols | 24 | 0 | 0 |
| File Transfer | 9 | 0 | 0 |
| Logging | 11 | 0 | 0 |
| Automation | 20 | 0 | 1 |
| Configuration | 14 | 0 | 0 |
| Advanced | 52 | 0 | 0 |
| **TOTAL** | **217** | **2** | **1** |

### Completion: ~99% complete

---

## PROJECT STRUCTURE

```
termicon/
├── src/
│   ├── core/
│   │   ├── adaptive.rs       # Adaptive automation
│   │   ├── arbitration.rs    # Resource arbitration
│   │   ├── bridge/           # Serial ↔ TCP bridge
│   │   ├── capability.rs     # Transport capabilities
│   │   ├── chart/            # Real-time charting
│   │   ├── codec/            # Data encoding
│   │   ├── collaborative.rs  # Team features
│   │   ├── deterministic.rs  # Reproducible runs
│   │   ├── experiment.rs     # Parameter sweep
│   │   ├── explain.rs        # Root cause hints
│   │   ├── external_api.rs   # REST/WebSocket API
│   │   ├── fuzzing.rs        # Protocol fuzzing
│   │   ├── knowledge.rs      # Device knowledge base
│   │   ├── logger.rs         # Session logging
│   │   ├── macro_recorder.rs # Macro recording
│   │   ├── macros.rs         # Quick macros M1-M24
│   │   ├── packet.rs         # Packet abstraction
│   │   ├── plugin/           # Plugin system
│   │   ├── profile/          # Session profiles
│   │   ├── protocol/         # Modbus, framing, checksums
│   │   ├── protocol_dsl.rs   # Protocol definitions
│   │   ├── replay.rs         # Session replay
│   │   ├── routing.rs        # Routing graph
│   │   ├── session.rs        # Session management
│   │   ├── simulator.rs      # Virtual device
│   │   ├── snippet/          # Command snippets
│   │   ├── state_machine.rs  # Session state
│   │   ├── terminal/         # VT100/VT220 emulation
│   │   ├── transfer/         # XMODEM/YMODEM/ZMODEM
│   │   ├── transport/        # Serial/TCP/Telnet/SSH/BLE
│   │   ├── trigger.rs        # Pattern triggers
│   │   ├── vault.rs          # Credential vault
│   │   └── virtual_port/     # PTY/Named pipes
│   ├── gui/
│   │   ├── app.rs            # Main application
│   │   ├── ansi_parser.rs    # ANSI color parser
│   │   ├── ble_panel.rs      # BLE inspector
│   │   ├── chart_panel.rs    # Chart view
│   │   ├── command_palette.rs # Command palette
│   │   ├── macros_panel.rs   # M1-M24 macros
│   │   ├── profiles.rs       # Profile management
│   │   ├── session_tab.rs    # Tab management
│   │   └── sftp_panel.rs     # SFTP browser
│   ├── config/               # Configuration
│   ├── i18n/                 # Internationalization
│   └── utils/                # Utilities
├── i18n/                     # Translation files (EN/HU)
├── assets/                   # Icons
├── docs/                     # Documentation
└── benches/                  # Benchmarks
```

---

## VERSION HISTORY

### v0.1.0 (Current)
- All core connection types (Serial, TCP, Telnet, SSH, Bluetooth)
- Modern GUI with tabs, dark/light themes
- Full terminal emulation (VT100/VT220, 256+true color, mouse)
- Framing protocols (SLIP/COBS/STX-ETX/Length-prefix)
- Checksums (CRC-16/32, XOR, LRC, Fletcher)
- Modbus RTU/TCP
- File transfer (XMODEM/YMODEM/ZMODEM)
- SFTP support + GUI browser
- Network Bridge
- Virtual COM ports
- Profiles with usage tracking
- Quick Macros (M1-M24)
- Profile-specific commands
- Triggers with auto-response
- Real-time Charts
- Session logging
- Macro recording and playback
- Search in output
- Plugin system (structure)
- i18n (EN/HU)
- Bluetooth LE (BLE GATT, Nordic UART Service)
- Transport Capability Registry
- Session State Machine
- Packet Abstraction Layer
- Protocol DSL (YAML/JSON)
- Session Replay
- Virtual Device Simulator
- BLE Inspector UI
- Credential Vault
- CLI Parity (full command-line support)
- Command Palette (Ctrl+K)
- Knowledge Base
- Deterministic Session Mode
- Fuzzing/Robustness Testing
- Routing Graph
- Adaptive Automation
- Resource Arbitration
- Experiment/Parameter Sweep Mode
- Explain Mode (Root Cause Hints)
- Collaborative Features
- External API (REST/WebSocket)

### v0.2.0 (Planned)
- Bluetooth Classic SPP (RFCOMM)
- Lua scripting
- Plugin marketplace
- Wireshark export
