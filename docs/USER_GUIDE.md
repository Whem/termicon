# Termicon User Guide

## Introduction

Termicon is a professional multi-protocol terminal application that supports multiple connection types and provides powerful features for embedded development, system administration, and device configuration.

## Quick Start

### Launching the Application

**GUI Mode:**
```bash
termicon
# or
.\target\release\termicon.exe  (Windows)
```

**CLI Mode:**
```bash
termicon-cli --help
termicon-cli serial --port COM3 --baud 115200
termicon-cli ssh --host example.com --user admin
```

### Creating Your First Connection

1. Launch Termicon
2. Click one of the connection buttons in the toolbar:
   - **Serial** - For COM port connections
   - **TCP** - For raw TCP socket connections
   - **Telnet** - For Telnet protocol
   - **SSH** - For secure shell connections
   - **BLE** - For Bluetooth Low Energy
3. Configure the connection settings
4. Click "Connect"
5. When prompted, save the connection as a profile for quick access later

### Using Profiles

**Connecting from Profile:**
- **Double-click** a profile in the side panel to connect immediately
- Or click the **Connect** button next to the profile

**Profile Features:**
- Filter profiles by type (All, Serial, TCP, SSH, etc.)
- Search profiles by name
- Profiles are sorted by usage frequency
- Toggle favorite status with star button

**Commands (Profile-specific):**
- Any command you type while connected via a profile is saved automatically
- Commands are sorted by frequency of use
- **Double-click** a command to insert it into the input field
- Delete commands with the X button

## Connection Types

### Serial Port (S/)

Configure serial connections with:

| Setting | Description | Common Values |
|---------|-------------|---------------|
| Port | Serial port name | COM3, /dev/ttyUSB0 |
| Baud Rate | Data speed | 9600, 115200, 921600 |
| Data Bits | Bits per character | 5, 6, 7, 8 |
| Stop Bits | Stop bits | 1, 1.5, 2 |
| Parity | Error checking | None, Odd, Even |
| Flow Control | Data flow management | None, Hardware (RTS/CTS), Software (XON/XOFF) |

**Serial-specific controls:**
- **DTR** - Data Terminal Ready signal toggle
- **RTS** - Request To Send signal toggle
- **BRK** - Send break signal

### TCP/IP (@)

Direct TCP socket connection:

| Setting | Description |
|---------|-------------|
| Host | IP address or hostname |
| Port | TCP port number |
| Timeout | Connection timeout (seconds) |

### Telnet (T>)

Telnet with protocol negotiation:

| Setting | Description |
|---------|-------------|
| Host | Server address |
| Port | Default: 23 |

### SSH (#)

Secure Shell connection:

| Setting | Description |
|---------|-------------|
| Host | Server address |
| Port | Default: 22 |
| Username | Login username |
| Password | Login password |
| Key File | SSH private key (optional) |
| Jump Host | Proxy/bastion host (optional) |

**SSH Features:**
- SFTP file browser (click SFTP button when connected)
- Port forwarding
- Jump proxy support

### Bluetooth LE (B*)

Bluetooth Low Energy connections:

| Setting | Description |
|---------|-------------|
| Device | Scan and select device |
| Service UUID | GATT service UUID |
| TX Characteristic | Write characteristic |
| RX Characteristic | Notify characteristic |

**BLE Features:**
- Device scanning
- GATT service browser
- Nordic UART Service (NUS) support

## User Interface

### Main Window Layout

```
+----------------------------------------------------------+
| Menu Bar: File | Edit | View | Help                      |
+----------------------------------------------------------+
| Toolbar: [Serial] [TCP] [Telnet] [SSH] [BLE] [Stop]      |
|          [DTR] [RTS] [BRK] [Transfer] | [Light] [HU] [>] |
+----------------------------------------------------------+
| Tab Bar: [Welcome] [COM8 @ 115200] [+]                   |
+----------------------------------------------------------+
|                                        | Side Panel      |
|  Terminal Output                       | [P] [C] [H] [G] |
|                                        |                 |
|  19:29:05.896 > Hello World            | Profiles        |
|  19:29:05.899 < Response               | - My Device     |
|                                        | - SSH Server    |
|                                        |                 |
+----------------------------------------------------------+
| Macros: [AT] [AT+GMR] [Help] [ls-la] [pwd] [clear] ...   |
+----------------------------------------------------------+
| > _____________________________ [Send]                   |
+----------------------------------------------------------+
| [Connected] COM8 @ 115200 baud         Termicon v0.1.0   |
+----------------------------------------------------------+
```

### Side Panel Tabs

- **[P] Profiles** - Saved connection profiles
- **[C] Commands** - Saved commands (per-profile)
- **[H] History** - Command history
- **[G] Chart** - Real-time data visualization
- **[S] Settings** - Application settings

### Profiles

Profiles save your connection settings for quick access:

1. Connect to a device
2. You'll be prompted to save as a profile
3. Enter a name and click Save
4. Click on a profile to connect instantly

**Profile Features:**
- Filter by type (Serial, TCP, SSH, etc.)
- Search profiles
- Favorites (click the star)
- Usage counter
- Profile-specific commands

### Commands (Snippets)

Commands are saved per-profile and sorted by usage:

1. Connect from a saved profile
2. Type commands - they're auto-saved
3. Double-click a command to insert it
4. Most-used commands appear first

## Macros (M1-M24)

Quick macro buttons at the bottom (Serial connections only):

- **Left-click**: Execute macro
- **Right-click**: Edit macro

Each macro can contain:
- Plain text commands
- Hex data (e.g., "FF 00 A5")
- Optional CR+LF suffix

**Function Keys**: F1-F12 trigger M1-M12

## Display Modes

### Text View

Standard terminal display with:
- ANSI color support (256 colors + true color)
- VT100/VT220 emulation
- Scrollable output buffer
- Timestamps (optional)
- Line wrapping

### Hex View

Toggle with View menu or toolbar:
```
00000000  48 65 6C 6C 6F 20 57 6F  72 6C 64 21 0D 0A     |Hello World!..|
```

## File Transfer

### Serial Protocols

Access via **Transfer** menu button:

| Protocol | Description |
|----------|-------------|
| XMODEM | 128-byte blocks with checksum/CRC |
| XMODEM-1K | 1024-byte blocks |
| YMODEM | Batch mode with file info |
| ZMODEM | Streaming, auto-start |
| Kermit | Full protocol with quoting and checksums |

### SFTP (SSH only)

Click **SFTP** button when connected via SSH:
- Two-panel file browser
- Upload/download files
- Create directories
- Delete files

## Automation

### Triggers

Create automatic responses:

1. Go to Tools menu
2. Create trigger with:
   - Pattern (text, regex, hex)
   - Action (send response, log, notify)

### Macro Recording

Record and replay actions:
1. Start recording
2. Perform actions
3. Stop recording
4. Save and replay

### Trigger Chains

Create multi-step triggers:
1. Define a sequence of patterns
2. Each pattern triggers the next step
3. Configure timeout and retry behavior

### Batch Operations

Execute commands on multiple sessions:
1. Select sessions
2. Define command sequence
3. Run in parallel or sequential mode

## Menu System

### File Menu
- **New Tab (Ctrl+T)** - Create a new session tab
- **Close Tab (Ctrl+W)** - Close current tab
- **Exit** - Close application

### Edit Menu
- **Clear Terminal** - Clear the terminal output

### View Menu
- **Show Timestamps** - Toggle timestamp display
- **Hex View** - Toggle hex data display
- **Local Echo** - Toggle local echo
- **Side Panel** - Toggle side panel visibility
- **Macros Bar (M1-M24)** - Toggle macros toolbar

### Connection Menu
- **Serial Port** - Serial connection options and bridge
- **Network** - TCP Client and Telnet
- **SSH** - SSH connect, SFTP, key generation
- **Bluetooth LE** - BLE connections

### Tools Menu
- **File Transfer** - XMODEM, YMODEM, ZMODEM, Kermit
- **Protocols** - Modbus Monitor, NMEA Viewer, Protocol DSL
- **Triggers & Auto-response** - Pattern matching triggers
- **Macro Recorder** - Record and playback macros
- **Advanced** - Device Simulator, Session Replay, Fuzzing, Experiment Mode

### Help Menu
- **User Guide** - Open documentation
- **Keyboard Shortcuts** - Show shortcuts
- **About** - Application info

## Settings

### Theme
- **Dark** - Dark background
- **Light** - Light background

### Language
- **EN** - English
- **HU** - Magyar (Hungarian)
- Click the language button in toolbar to switch instantly

### Terminal Options
- Local Echo
- Show Timestamps
- Hex View
- Auto-scroll
- Line Ending (CR, LF, CRLF)

### Accessibility
- **High Contrast Mode** - Maximum contrast colors
- **Font Scaling** - Adjust text size (75%-200%)
- **Focus Indicators** - Clear focus highlighting

### Custom Themes
12+ built-in color schemes:
- Dracula, Monokai, Solarized (Dark/Light)
- Nord, One Dark, Gruvbox, Tokyo Night
- And more...

## Keyboard Shortcuts

### General
| Shortcut | Action |
|----------|--------|
| Ctrl+K | Command Palette |
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Ctrl+D | Disconnect |
| Ctrl+L | Clear screen |
| Ctrl+S | Save session log |
| Ctrl+F | Search in output |
| Ctrl+G | Toggle chart panel |
| Ctrl+P | Toggle profiles panel |
| Escape | Close dialogs/panels |

### Terminal
| Shortcut | Action |
|----------|--------|
| Enter | Send command |
| Up/Down | Command history |
| Ctrl+C | Copy selection |
| Ctrl+V | Paste |
| Page Up/Down | Scroll output |
| Home/End | Jump to start/end |

### Macros
| Shortcut | Action |
|----------|--------|
| F1-F12 | Execute M1-M12 macros |
| Shift+F1-F12 | Execute M13-M24 macros |

### Navigation
| Shortcut | Action |
|----------|--------|
| Tab | Focus next element |
| Shift+Tab | Focus previous element |
| Alt+1-9 | Switch to tab 1-9 |
| Ctrl+Tab | Next tab |
| Ctrl+Shift+Tab | Previous tab |

## Command Palette (Ctrl+K)

Quick access to all commands:
1. Press Ctrl+K to open
2. Type to search commands
3. Use arrow keys to navigate
4. Enter to execute

Command categories:
- **Connection** - New connections, disconnect
- **View** - Toggle panels, themes
- **File** - Save, export, transfer
- **Tools** - Macros, triggers, automation
- **Help** - Documentation, about

## CLI Usage

```bash
# List serial ports
termicon-cli list-ports

# Connect to serial port
termicon-cli serial --port COM3 --baud 115200

# Connect via TCP
termicon-cli tcp --host 192.168.1.100 --port 23

# Connect via SSH
termicon-cli ssh --host example.com --user admin

# Headless mode (for automation)
termicon-cli serial --port COM3 --headless --script myscript.txt

# Pipe support
echo "AT" | termicon-cli serial --port COM3 --baud 115200
termicon-cli serial --port COM3 | tee output.log

# Output format (raw, hex, json)
termicon-cli serial --port COM3 --output-format json

# Exit codes for scripting
termicon-cli serial --port COM3 --command "AT" && echo "Success"
```

### Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Connection failed |
| 4 | Connection timeout |
| 5 | Authentication failed |
| 6 | File not found |
| 14 | Port not found |

## Troubleshooting

### Port Not Found
- Check device is connected
- Verify drivers are installed
- Click refresh button

### Permission Denied (Linux)
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

### Connection Timeout
- Verify device is powered
- Check cable connections
- Try different baud rate

### Garbled Output
- Check baud rate matches device
- Verify data bits and parity
- Try different line ending

### SSH ANSI Colors Not Working
- Terminal emulation handles ANSI codes
- Some escape sequences may not render

## Configuration Files

Location:
- **Windows**: `%APPDATA%\termicon\`
- **Linux**: `~/.config/termicon/`
- **macOS**: `~/Library/Application Support/termicon/`

Files:
- `profiles.json` - Saved profiles
- `macros.json` - M1-M24 macros
- `config.toml` - General settings

## Getting Help

- Check [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
- See [FULL_FEATURE_MATRIX.md](FULL_FEATURE_MATRIX.md) for complete feature list
- Report issues on GitHub
