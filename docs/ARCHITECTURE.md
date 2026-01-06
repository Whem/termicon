# Termicon Architecture

## Overview

Termicon is designed as a modular, layered architecture that separates concerns and allows for easy extension.

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interface                            │
│  ┌─────────────────────────┐  ┌─────────────────────────────┐   │
│  │         GUI             │  │           CLI               │   │
│  │    (egui/eframe)        │  │         (clap)              │   │
│  └─────────────────────────┘  └─────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────────┐
│                      Application Layer                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │ Session  │  │ Profiles │  │ Triggers │  │   Macros     │    │
│  │ Manager  │  │  Config  │  │  Engine  │  │   System     │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │ Command  │  │ Knowledge│  │ Explain  │  │  Experiment  │    │
│  │ Palette  │  │   Base   │  │   Mode   │  │    Runner    │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────────┐
│                       Protocol Layer                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │  Telnet  │  │   SSH    │  │  Modbus  │  │   Protocol   │    │
│  │ Protocol │  │ Protocol │  │ RTU/TCP  │  │     DSL      │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │   SLIP   │  │   COBS   │  │  STX/ETX │  │    Packet    │    │
│  │ Framing  │  │ Framing  │  │ Framing  │  │  Abstraction │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────────┐
│                      Transport Layer                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │  Serial  │  │   TCP    │  │   SSH    │  │  Bluetooth   │    │
│  │Transport │  │Transport │  │Transport │  │  (BLE/SPP)   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │  Bridge  │  │  Router  │  │ Virtual  │  │  Capability  │    │
│  │  Module  │  │  Graph   │  │   Port   │  │  Registry    │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                               │
┌─────────────────────────────────────────────────────────────────┐
│                        Core Layer                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │  Codec   │  │  Logger  │  │ Terminal │  │    Utils     │    │
│  │ (Hex/Txt)│  │(File/Mem)│  │ Emulator │  │              │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │
│  │  Vault   │  │  Replay  │  │Simulator │  │   Adaptive   │    │
│  │(Secrets) │  │  System  │  │ (Virtual)│  │  Automation  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Core Concepts

### Transport Trait

All communication channels implement a unified `TransportTrait`:

```rust
#[async_trait]
pub trait TransportTrait: Send + Sync {
    async fn connect(&mut self) -> Result<(), TransportError>;
    async fn disconnect(&mut self) -> Result<(), TransportError>;
    fn is_connected(&self) -> bool;
    async fn send(&mut self, data: &[u8]) -> Result<usize, TransportError>;
    async fn receive(&mut self) -> Result<Bytes, TransportError>;
    fn transport_type(&self) -> TransportType;
    fn connection_info(&self) -> String;
    fn stats(&self) -> TransportStats;
    fn subscribe(&self) -> broadcast::Receiver<Bytes>;
}
```

This allows uniform handling of:
- Serial ports
- TCP sockets
- Telnet connections
- SSH sessions
- Bluetooth (BLE/SPP)

### Session Management

A `Session` wraps a transport and adds:
- Receive buffer management
- Trigger matching
- Event broadcasting
- Statistics tracking
- State machine lifecycle

```rust
pub struct Session {
    id: Uuid,
    transport: Box<dyn TransportTrait>,
    rx_buffer: Arc<RwLock<Vec<u8>>>,
    triggers: Arc<RwLock<Vec<Trigger>>>,
    event_tx: broadcast::Sender<SessionEvent>,
    state: SessionState,
    logger: Option<SessionLogger>,
}
```

### Transport Capability Registry

Each transport declares its capabilities:

```rust
pub struct TransportCapabilities {
    pub can_send: bool,
    pub can_receive: bool,
    pub supports_flow_control: bool,
    pub supports_modem_lines: bool,
    pub supports_break: bool,
    pub max_baud_rate: Option<u32>,
    pub supports_file_transfer: bool,
}
```

### Codec Pipeline

Data flows through codecs for transformation:

```
Raw Bytes → Decoder → Display Format
User Input → Encoder → Raw Bytes
```

Supported codecs:
- **Text**: ASCII/UTF-8 with line ending conversion
- **Hex**: Hexadecimal display with grouping
- **Mixed**: Automatic detection (printable vs binary)
- **Binary**: Bit-level display

### Protocol Decoders

Protocol decoders parse structured data:

```rust
pub trait ProtocolDecoder: Send + Sync {
    fn name(&self) -> &str;
    fn decode(&self, data: &[u8]) -> Result<DecodedPacket, DecodeError>;
    fn can_decode(&self, data: &[u8]) -> bool;
}
```

Built-in decoders:
- Modbus RTU/TCP
- SLIP/COBS framing
- Protocol DSL (custom YAML/JSON definitions)

### Packet Abstraction

Generic packet handling with metadata:

```rust
pub struct Packet {
    pub timestamp: SystemTime,
    pub direction: Direction,
    pub data: Vec<u8>,
    pub protocol: Option<String>,
    pub metadata: HashMap<String, Value>,
}
```

## Module Details

### Core Modules

| Module | Path | Description |
|--------|------|-------------|
| transport | `src/core/transport/` | Serial, TCP, Telnet, SSH, Bluetooth |
| terminal | `src/core/terminal/` | VT100/VT220/ANSI emulation |
| chart | `src/core/chart/` | Real-time data visualization |
| protocol | `src/core/protocol/` | Modbus, CRC, framing |
| bridge | `src/core/bridge/` | Network bridging |
| virtual_port | `src/core/virtual_port/` | Virtual COM ports |
| session | `src/core/session.rs` | Connection management |
| codec | `src/core/codec/` | Data encoding |
| logger | `src/core/logger.rs` | Session logging |
| trigger | `src/core/trigger.rs` | Pattern matching |
| macros | `src/core/macros.rs` | Quick macros M1-M24 |
| macro_recorder | `src/core/macro_recorder.rs` | Recording/playback |
| capability | `src/core/capability.rs` | Transport capabilities |
| state_machine | `src/core/state_machine.rs` | Session lifecycle |
| packet | `src/core/packet.rs` | Packet abstraction |
| protocol_dsl | `src/core/protocol_dsl.rs` | Protocol definitions |
| replay | `src/core/replay.rs` | Session replay |
| simulator | `src/core/simulator.rs` | Virtual devices |
| vault | `src/core/vault.rs` | Credential storage |
| knowledge | `src/core/knowledge.rs` | Device knowledge base |
| deterministic | `src/core/deterministic.rs` | Reproducible runs |
| fuzzing | `src/core/fuzzing.rs` | Protocol fuzzing |
| routing | `src/core/routing.rs` | Transport routing |
| adaptive | `src/core/adaptive.rs` | Adaptive automation |
| arbitration | `src/core/arbitration.rs` | Resource management |
| experiment | `src/core/experiment.rs` | Parameter sweeps |
| explain | `src/core/explain.rs` | Root cause analysis |
| collaborative | `src/core/collaborative.rs` | Team features |
| external_api | `src/core/external_api.rs` | REST/WebSocket API |

### GUI Modules

| Module | Path | Description |
|--------|------|-------------|
| app | `src/gui/app.rs` | Main application |
| session_tab | `src/gui/session_tab.rs` | Tab management |
| chart_panel | `src/gui/chart_panel.rs` | Chart view |
| sftp_panel | `src/gui/sftp_panel.rs` | SFTP browser |
| macros_panel | `src/gui/macros_panel.rs` | M1-M24 macros |
| profiles | `src/gui/profiles.rs` | Profile management |
| command_palette | `src/gui/command_palette.rs` | Command palette |
| ansi_parser | `src/gui/ansi_parser.rs` | ANSI color parsing |

### Transport: Serial

```
SerialTransport
├── Configuration
│   ├── Port name (COM1, /dev/ttyUSB0)
│   ├── Baud rate (300 - 4000000)
│   ├── Data bits (5-8)
│   ├── Parity (None, Odd, Even)
│   ├── Stop bits (1, 2)
│   └── Flow control (None, HW, SW)
├── Modem Lines
│   ├── DTR, RTS (output)
│   └── CTS, DSR, DCD, RI (input)
└── Features
    ├── Break signal
    ├── RS-485 mode
    └── Virtual COM support
```

### Transport: SSH

```
SshTransport
├── Authentication
│   ├── Password ✅
│   ├── Password save (optional) ✅
│   ├── Public key (RSA, ECDSA, Ed25519) ✅
│   ├── Key passphrase ✅
│   ├── SSH Agent 🔄
│   ├── Keyboard-interactive ✅
│   └── Auto-connect ✅
├── Channels
│   ├── Shell (PTY) ✅
│   ├── Exec (command) ✅
│   └── SFTP (file transfer) ✅
├── Port Forwarding
│   ├── Local (-L) ✅
│   ├── Remote (-R) ✅
│   └── Dynamic/SOCKS (-D) 🔄
├── Jump Host (ProxyJump)
│   ├── Jump host/port ✅
│   ├── Jump credentials ✅
│   └── Multi-hop 🔄
└── Features
    ├── Compression ✅
    ├── Keepalive ✅
    ├── Connection timeout ✅
    ├── Terminal type selection ✅
    ├── X11 forwarding ✅
    └── Agent forwarding ✅
```

### Transport: Bluetooth

```
BluetoothModule
├── BLE (Low Energy)
│   ├── Scanning (filters, RSSI)
│   ├── GATT Client
│   │   ├── Service discovery
│   │   ├── Characteristic R/W
│   │   └── Notifications
│   └── Common profiles
│       ├── Nordic UART (NUS)
│       ├── Battery Service
│       └── Device Information
└── Classic
    ├── SPP (Serial Port Profile)
    ├── RFCOMM channels
    └── Device pairing
```

## Data Flow

### Receive Path

```
Hardware → Transport.receive() → Session.rx_buffer
                                     │
                                     ├─→ Trigger matching
                                     ├─→ Protocol decoding
                                     ├─→ Codec transformation
                                     ├─→ ANSI parsing
                                     └─→ UI display
```

### Send Path

```
User Input → Codec.encode() → Session.send()
                                  │
                                  ├─→ Macro recording
                                  ├─→ Profile command save
                                  └─→ Transport.send() → Hardware
```

## Async Architecture

Termicon uses Tokio for async I/O:

```rust
// Each session runs in its own task
tokio::spawn(async move {
    loop {
        tokio::select! {
            // Receive from transport
            result = transport.receive() => { ... }
            
            // Handle commands from UI
            cmd = command_rx.recv() => { ... }
            
            // Periodic tasks
            _ = interval.tick() => { ... }
        }
    }
});
```

## Configuration System

```
AppConfig
├── General Settings
│   ├── Language (EN/HU)
│   ├── Theme (Dark/Light)
│   └── Window state
├── Terminal Settings
│   ├── Font
│   ├── Colors
│   └── Buffer size
├── Logging Settings
│   ├── Directory
│   ├── Format
│   └── Rotation
└── Connection Profiles[]
    ├── Name, Description
    ├── Transport config
    ├── Display settings
    ├── Triggers
    └── Profile-specific commands
```

## Profile System

```
ProfileManager
├── Profile
│   ├── id (UUID)
│   ├── name
│   ├── profile_type (Serial/TCP/Telnet/SSH/Bluetooth)
│   ├── favorite (bool)
│   ├── use_count
│   ├── last_used
│   ├── connection settings
│   └── snippets[] (profile-specific commands)
├── Filter by type
├── Search
├── Usage tracking
└── Persistence (JSON)
```

## Plugin Architecture (Future)

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, api: &PluginApi);
    fn on_unload(&mut self);
}

pub trait PluginApi {
    fn register_decoder(&self, decoder: Box<dyn ProtocolDecoder>);
    fn register_command(&self, name: &str, handler: CommandHandler);
    fn get_session(&self, id: Uuid) -> Option<&Session>;
}
```

## Security Considerations

1. **Credentials**: Vault system with encryption support
2. **SSH Keys**: Key-based authentication supported
3. **Audit Log**: Session logging with timestamps and direction
4. **Plugin Sandbox**: Restricted API access for plugins (future)

## Performance

- **Zero-copy**: Where possible, data is not copied
- **Streaming**: Large files streamed, not loaded into memory
- **Lazy UI**: Only visible elements rendered
- **Background I/O**: All I/O operations are async

## Current Status

### Implemented ✅

- **Transport**: Serial, TCP, Telnet, SSH-2, Bluetooth LE
- **Terminal**: VT100/VT220/ANSI emulation with true color
- **Chart**: Real-time data visualization
- **Protocol**: Modbus RTU/TCP, CRC, SLIP, COBS, Protocol DSL
- **Bridge**: Serial ↔ TCP bridging
- **Virtual Port**: PTY (Unix), Named Pipes (Windows)
- **Profiles**: Full profile management with usage tracking
- **Macros**: M1-M24 quick macros, macro recording
- **Advanced**: Fuzzing, Routing, Adaptive automation, Experiment mode
- **i18n**: English and Hungarian translations

### In Progress 🔄

- Bluetooth SPP (Classic)
- Lua scripting
- REST API completion

### Planned 📋

- Plugin system
- Protocol IDE
- Real-time collaboration
