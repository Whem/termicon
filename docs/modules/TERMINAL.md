# Terminal Module

## Overview

The Terminal module provides complete VT100/VT220/ANSI terminal emulation with support for colors, cursor control, and advanced features.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| VT100 Emulation | ✅ | Basic terminal |
| VT220 Emulation | ✅ | Extended features |
| ANSI Colors (16) | ✅ | Standard colors |
| 256 Colors | ✅ | Extended palette |
| True Color (24-bit) | ✅ | RGB colors |
| Cursor Control | ✅ | Movement, visibility |
| Screen Buffer | ✅ | Normal + alternate |
| Scroll Regions | ✅ | Scroll areas |
| Character Sets | ✅ | G0/G1 sets |
| Line Drawing | ✅ | Box characters |
| Mouse Reporting | ✅ | X10, X11, SGR |
| Bracketed Paste | ✅ | Paste detection |
| OSC Sequences | ✅ | Title, colors |
| Unicode/UTF-8 | ✅ | Full support |
| Sixel Graphics | ❌ | Future |

## Escape Sequences

### CSI Sequences (Control Sequence Introducer)

Format: `ESC [ <params> <command>`

| Sequence | Description |
|----------|-------------|
| `ESC[nA` | Cursor up n lines |
| `ESC[nB` | Cursor down n lines |
| `ESC[nC` | Cursor forward n cols |
| `ESC[nD` | Cursor back n cols |
| `ESC[n;mH` | Cursor to row n, col m |
| `ESC[nJ` | Erase display |
| `ESC[nK` | Erase line |
| `ESC[n;mf` | Cursor position |
| `ESC[s` | Save cursor position |
| `ESC[u` | Restore cursor position |

### SGR Sequences (Select Graphic Rendition)

Format: `ESC [ <params> m`

| Code | Description |
|------|-------------|
| 0 | Reset all attributes |
| 1 | Bold |
| 2 | Dim |
| 3 | Italic |
| 4 | Underline |
| 5 | Blink (slow) |
| 7 | Reverse video |
| 8 | Hidden |
| 9 | Strikethrough |
| 30-37 | Foreground color |
| 40-47 | Background color |
| 90-97 | Bright foreground |
| 100-107 | Bright background |

### Color Codes

**Standard Colors (30-37 / 40-47):**
- 0: Black
- 1: Red
- 2: Green
- 3: Yellow
- 4: Blue
- 5: Magenta
- 6: Cyan
- 7: White

**256 Colors:**
```
ESC[38;5;nm  - Foreground
ESC[48;5;nm  - Background
```

**True Color (24-bit):**
```
ESC[38;2;r;g;bm  - Foreground RGB
ESC[48;2;r;g;bm  - Background RGB
```

## Configuration

```rust
pub struct TerminalConfig {
    pub rows: usize,
    pub cols: usize,
    pub scrollback: usize,
    pub word_wrap: bool,
    pub mouse_reporting: bool,
    pub bracketed_paste: bool,
}
```

## GUI Usage

### Terminal View

The terminal view in each session tab provides:
- Full terminal emulation
- Scrollback buffer
- Text selection
- Copy/paste support

### Display Options

- **Timestamps**: Show time for each line
- **Hex View**: Show hex alongside text
- **Local Echo**: Echo typed characters

### Mouse Support

When enabled, mouse events are reported to the remote:
- Left/middle/right click
- Scroll wheel
- Motion tracking (if enabled)

## Code Examples

### Terminal Parser

```rust
use termicon_core::terminal::{Terminal, TerminalConfig};

let config = TerminalConfig {
    rows: 24,
    cols: 80,
    scrollback: 10000,
    word_wrap: true,
    mouse_reporting: true,
    bracketed_paste: true,
};

let mut terminal = Terminal::new(config);

// Process incoming data
terminal.process(b"\x1b[31mRed Text\x1b[0m");

// Get screen buffer
let screen = terminal.screen();
for row in screen.iter() {
    for cell in row.iter() {
        print!("{}", cell.character);
    }
    println!();
}
```

### ANSI Parser (GUI)

```rust
use termicon_gui::ansi_parser::parse_ansi;

let text = "\x1b[32mGreen\x1b[0m Normal";
let spans = parse_ansi(text);

for span in spans {
    // Render with span.color and span.text
}
```

### Screen Buffer

```rust
// Get current screen state
let screen = terminal.screen();

// Get cell at position
let cell = screen.get(row, col);
println!("Char: {}, FG: {:?}, BG: {:?}", 
    cell.character, cell.fg_color, cell.bg_color);

// Get cursor position
let (row, col) = terminal.cursor_position();

// Resize terminal
terminal.resize(30, 120);
```

### Mouse Reporting

```rust
// Enable mouse reporting
terminal.set_mouse_mode(MouseMode::Sgr);

// Process mouse event
let event = MouseEvent {
    button: MouseButton::Left,
    action: MouseAction::Press,
    row: 10,
    col: 20,
    modifiers: Modifiers::empty(),
};

let sequence = terminal.encode_mouse_event(&event);
transport.send(&sequence).await?;
```

## Screen Buffer Modes

### Normal Buffer

The default screen buffer for regular terminal output.

### Alternate Buffer

Used by full-screen applications (vim, less, htop):

```
ESC[?1049h  - Switch to alternate buffer
ESC[?1049l  - Switch to normal buffer
```

## Character Sets

### G0/G1 Sets

```
ESC(0  - Select DEC Special Graphics (G0)
ESC(B  - Select ASCII (G0)
SO     - Switch to G1
SI     - Switch to G0
```

### Line Drawing Characters

When DEC Special Graphics is active:

| Char | Display |
|------|---------|
| j | ┘ |
| k | ┐ |
| l | ┌ |
| m | └ |
| n | ┼ |
| q | ─ |
| t | ├ |
| u | ┤ |
| v | ┴ |
| w | ┬ |
| x | │ |

## OSC Sequences

Format: `ESC ] <command> ; <data> BEL`

| Sequence | Description |
|----------|-------------|
| `ESC]0;title\x07` | Set window title |
| `ESC]4;n;color\x07` | Set palette color |
| `ESC]10;color\x07` | Set foreground |
| `ESC]11;color\x07` | Set background |
| `ESC]52;c;base64\x07` | Clipboard access |

## CLI Integration

The terminal emulation is also used in CLI mode:

```bash
# Interactive terminal session
termicon-cli ssh user@host

# With specific terminal type
termicon-cli ssh user@host --term xterm-256color
```

## Troubleshooting

### Colors Not Working

- Check TERM environment variable
- Verify 256-color support: `echo $TERM`
- Test with: `echo -e "\e[38;5;196mRed\e[0m"`

### Special Keys Not Working

- Check terminal type matches remote
- Verify key encoding
- Some applications expect specific terminals

### Garbled Output

- Check character encoding (UTF-8)
- Verify line ending settings
- Check for binary data in text mode

### Mouse Not Working

- Verify mouse reporting is enabled
- Check application supports mouse
- Some terminals need focus

## Performance

The terminal module is optimized for:
- Minimal memory allocation
- Efficient screen updates
- Fast escape sequence parsing
- Large scrollback buffers
