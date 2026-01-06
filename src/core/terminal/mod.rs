//! Terminal emulation module
//!
//! Provides VT100/VT220/ANSI terminal emulation with:
//! - CSI (Control Sequence Introducer) parsing
//! - SGR (Select Graphic Rendition) for colors/styles
//! - Cursor movement and positioning
//! - Screen buffer management
//! - Scrolling regions
//! - Mouse reporting
//! - Sixel graphics

mod parser;
mod screen;
mod cell;
mod color;
pub mod sixel;

pub use parser::{AnsiParser, AnsiEvent};
pub use screen::{Screen, ScreenMode};
pub use cell::{Cell, CellStyle};
pub use color::{Color, NamedColor};
pub use sixel::{SixelEncoder, SixelImage, SixelParser, SixelColor};

/// Terminal size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    /// Columns (width)
    pub cols: u16,
    /// Rows (height)
    pub rows: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self { cols: 80, rows: 24 }
    }
}

impl TerminalSize {
    /// Create new terminal size
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }
}

/// Terminal emulator state
pub struct Terminal {
    /// Screen buffer
    screen: Screen,
    /// Parser state
    parser: AnsiParser,
    /// Current size
    size: TerminalSize,
    /// Alternate screen buffer
    alt_screen: Option<Screen>,
    /// Use alternate screen
    use_alt_screen: bool,
    /// Application cursor keys mode
    app_cursor_keys: bool,
    /// Application keypad mode
    app_keypad: bool,
    /// Bracketed paste mode
    bracketed_paste: bool,
    /// Mouse reporting mode
    mouse_mode: MouseMode,
    /// Title
    title: String,
}

/// Mouse reporting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseMode {
    /// No mouse reporting
    #[default]
    None,
    /// X10 mouse reporting (button press only)
    X10,
    /// Normal tracking (button press and release)
    Normal,
    /// Button event tracking (motion while pressed)
    ButtonEvent,
    /// Any event tracking (all motion)
    AnyEvent,
}

impl Terminal {
    /// Create a new terminal with default size
    pub fn new() -> Self {
        Self::with_size(TerminalSize::default())
    }

    /// Create a terminal with specific size
    pub fn with_size(size: TerminalSize) -> Self {
        Self {
            screen: Screen::new(size.cols, size.rows),
            parser: AnsiParser::new(),
            size,
            alt_screen: None,
            use_alt_screen: false,
            app_cursor_keys: false,
            app_keypad: false,
            bracketed_paste: false,
            mouse_mode: MouseMode::None,
            title: String::new(),
        }
    }

    /// Process input bytes
    pub fn process(&mut self, data: &[u8]) {
        for event in self.parser.parse(data) {
            self.handle_event(event);
        }
    }

    /// Handle a parsed ANSI event
    fn handle_event(&mut self, event: AnsiEvent) {
        let screen = if self.use_alt_screen {
            self.alt_screen.as_mut().unwrap_or(&mut self.screen)
        } else {
            &mut self.screen
        };

        match event {
            AnsiEvent::Print(c) => {
                screen.put_char(c);
            }
            AnsiEvent::Execute(byte) => {
                self.handle_control(byte);
            }
            AnsiEvent::CsiDispatch { params, intermediates, action } => {
                self.handle_csi(params, intermediates, action);
            }
            AnsiEvent::EscDispatch { intermediates, action } => {
                self.handle_esc(intermediates, action);
            }
            AnsiEvent::OscDispatch { params } => {
                self.handle_osc(params);
            }
        }
    }

    /// Handle control character (C0)
    fn handle_control(&mut self, byte: u8) {
        let screen = self.current_screen_mut();
        
        match byte {
            0x07 => {
                // BEL - Bell
                // TODO: Trigger bell notification
            }
            0x08 => {
                // BS - Backspace
                screen.move_cursor_left(1);
            }
            0x09 => {
                // HT - Horizontal Tab
                screen.move_to_next_tab();
            }
            0x0A | 0x0B | 0x0C => {
                // LF, VT, FF - Line feed
                screen.linefeed();
            }
            0x0D => {
                // CR - Carriage return
                screen.carriage_return();
            }
            0x0E => {
                // SO - Shift Out (G1 character set)
                screen.set_charset(1);
            }
            0x0F => {
                // SI - Shift In (G0 character set)
                screen.set_charset(0);
            }
            _ => {}
        }
    }

    /// Handle CSI sequence
    fn handle_csi(&mut self, params: Vec<u16>, intermediates: Vec<u8>, action: u8) {
        // Get parameters with defaults
        let param = |idx: usize, default: u16| -> u16 {
            params.get(idx).copied().filter(|&p| p != 0).unwrap_or(default)
        };

        // Pre-compute values that need self before borrowing screen
        let default_rows = self.size.rows;

        match action {
            // Cursor movement
            b'A' => self.current_screen_mut().move_cursor_up(param(0, 1)),
            b'B' => self.current_screen_mut().move_cursor_down(param(0, 1)),
            b'C' => self.current_screen_mut().move_cursor_right(param(0, 1)),
            b'D' => self.current_screen_mut().move_cursor_left(param(0, 1)),
            b'E' => {
                // CNL - Cursor Next Line
                let screen = self.current_screen_mut();
                screen.move_cursor_down(param(0, 1));
                screen.carriage_return();
            }
            b'F' => {
                // CPL - Cursor Previous Line
                let screen = self.current_screen_mut();
                screen.move_cursor_up(param(0, 1));
                screen.carriage_return();
            }
            b'G' => self.current_screen_mut().set_cursor_col(param(0, 1).saturating_sub(1)),
            b'H' | b'f' => {
                // CUP/HVP - Cursor Position
                self.current_screen_mut().set_cursor_pos(param(0, 1).saturating_sub(1), param(1, 1).saturating_sub(1));
            }
            b'J' => {
                // ED - Erase in Display
                match param(0, 0) {
                    0 => self.current_screen_mut().erase_below(),
                    1 => self.current_screen_mut().erase_above(),
                    2 => self.current_screen_mut().erase_all(),
                    3 => self.current_screen_mut().erase_scrollback(),
                    _ => {}
                }
            }
            b'K' => {
                // EL - Erase in Line
                match param(0, 0) {
                    0 => self.current_screen_mut().erase_line_right(),
                    1 => self.current_screen_mut().erase_line_left(),
                    2 => self.current_screen_mut().erase_line(),
                    _ => {}
                }
            }
            b'L' => self.current_screen_mut().insert_lines(param(0, 1)),
            b'M' => self.current_screen_mut().delete_lines(param(0, 1)),
            b'P' => self.current_screen_mut().delete_chars(param(0, 1)),
            b'S' => self.current_screen_mut().scroll_up(param(0, 1)),
            b'T' => self.current_screen_mut().scroll_down(param(0, 1)),
            b'X' => self.current_screen_mut().erase_chars(param(0, 1)),
            b'@' => self.current_screen_mut().insert_chars(param(0, 1)),
            b'd' => self.current_screen_mut().set_cursor_row(param(0, 1).saturating_sub(1)),
            b'm' => {
                // SGR - Select Graphic Rendition
                self.handle_sgr(&params);
            }
            b'r' => {
                // DECSTBM - Set Scrolling Region
                let top = param(0, 1);
                let bottom = param(1, default_rows);
                self.current_screen_mut().set_scroll_region(top.saturating_sub(1), bottom.saturating_sub(1));
            }
            b's' => self.current_screen_mut().save_cursor(),
            b'u' => self.current_screen_mut().restore_cursor(),
            b'h' => {
                // SM - Set Mode
                self.handle_mode(&params, &intermediates, true);
            }
            b'l' => {
                // RM - Reset Mode
                self.handle_mode(&params, &intermediates, false);
            }
            b'n' => {
                // DSR - Device Status Report
                self.handle_dsr(&params);
            }
            b'c' => {
                // DA - Device Attributes
                // TODO: Send response
            }
            _ => {
                // Unknown CSI sequence
                tracing::debug!("Unknown CSI: {:?} {:?} {}", params, intermediates, action as char);
            }
        }
    }

    /// Handle SGR (Select Graphic Rendition)
    fn handle_sgr(&mut self, params: &[u16]) {
        let screen = self.current_screen_mut();
        
        if params.is_empty() {
            screen.reset_style();
            return;
        }

        let mut iter = params.iter().copied().peekable();
        
        while let Some(param) = iter.next() {
            match param {
                0 => screen.reset_style(),
                1 => screen.set_bold(true),
                2 => screen.set_dim(true),
                3 => screen.set_italic(true),
                4 => screen.set_underline(true),
                5 => screen.set_blink(true),
                7 => screen.set_inverse(true),
                8 => screen.set_hidden(true),
                9 => screen.set_strikethrough(true),
                21 => screen.set_bold(false),
                22 => {
                    screen.set_bold(false);
                    screen.set_dim(false);
                }
                23 => screen.set_italic(false),
                24 => screen.set_underline(false),
                25 => screen.set_blink(false),
                27 => screen.set_inverse(false),
                28 => screen.set_hidden(false),
                29 => screen.set_strikethrough(false),
                30..=37 => screen.set_fg_color(Color::Named(NamedColor::from_ansi(param - 30))),
                38 => {
                    // Extended foreground color
                    if let Some(color) = Self::parse_extended_color(&mut iter) {
                        screen.set_fg_color(color);
                    }
                }
                39 => screen.set_fg_color(Color::Default),
                40..=47 => screen.set_bg_color(Color::Named(NamedColor::from_ansi(param - 40))),
                48 => {
                    // Extended background color
                    if let Some(color) = Self::parse_extended_color(&mut iter) {
                        screen.set_bg_color(color);
                    }
                }
                49 => screen.set_bg_color(Color::Default),
                90..=97 => screen.set_fg_color(Color::Named(NamedColor::from_ansi(param - 90 + 8))),
                100..=107 => screen.set_bg_color(Color::Named(NamedColor::from_ansi(param - 100 + 8))),
                _ => {}
            }
        }
    }

    /// Parse extended color (256-color or RGB)
    fn parse_extended_color<I>(iter: &mut std::iter::Peekable<I>) -> Option<Color>
    where
        I: Iterator<Item = u16>,
    {
        match iter.next()? {
            5 => {
                // 256-color
                let idx = iter.next()?;
                Some(Color::Indexed(idx as u8))
            }
            2 => {
                // RGB
                let r = iter.next()? as u8;
                let g = iter.next()? as u8;
                let b = iter.next()? as u8;
                Some(Color::Rgb(r, g, b))
            }
            _ => None,
        }
    }

    /// Handle mode changes
    fn handle_mode(&mut self, params: &[u16], intermediates: &[u8], set: bool) {
        let is_dec = intermediates.first() == Some(&b'?');
        
        for &param in params {
            if is_dec {
                // DEC private modes
                match param {
                    1 => self.app_cursor_keys = set,
                    7 => self.current_screen_mut().set_auto_wrap(set),
                    12 => {
                        // Start/stop cursor blink
                    }
                    25 => self.current_screen_mut().set_cursor_visible(set),
                    47 | 1047 => {
                        // Alternate screen buffer
                        if set && self.alt_screen.is_none() {
                            self.alt_screen = Some(Screen::new(self.size.cols, self.size.rows));
                        }
                        self.use_alt_screen = set;
                    }
                    1000 => self.mouse_mode = if set { MouseMode::Normal } else { MouseMode::None },
                    1002 => self.mouse_mode = if set { MouseMode::ButtonEvent } else { MouseMode::None },
                    1003 => self.mouse_mode = if set { MouseMode::AnyEvent } else { MouseMode::None },
                    1049 => {
                        // Alternate screen with save/restore cursor
                        if set {
                            self.screen.save_cursor();
                            if self.alt_screen.is_none() {
                                self.alt_screen = Some(Screen::new(self.size.cols, self.size.rows));
                            }
                            self.use_alt_screen = true;
                        } else {
                            self.use_alt_screen = false;
                            self.screen.restore_cursor();
                        }
                    }
                    2004 => self.bracketed_paste = set,
                    _ => {}
                }
            } else {
                // ANSI modes
                match param {
                    4 => self.current_screen_mut().set_insert_mode(set),
                    20 => self.current_screen_mut().set_newline_mode(set),
                    _ => {}
                }
            }
        }
    }

    /// Handle DSR (Device Status Report)
    fn handle_dsr(&self, params: &[u16]) {
        // TODO: Send responses through callback
        match params.first() {
            Some(5) => {
                // Status report - device OK
                // Response: ESC [ 0 n
            }
            Some(6) => {
                // Cursor position report
                // Response: ESC [ row ; col R
            }
            _ => {}
        }
    }

    /// Handle ESC sequence
    fn handle_esc(&mut self, intermediates: Vec<u8>, action: u8) {
        match action {
            b'7' => self.current_screen_mut().save_cursor(),
            b'8' => self.current_screen_mut().restore_cursor(),
            b'D' => self.current_screen_mut().linefeed(),
            b'E' => {
                self.current_screen_mut().carriage_return();
                self.current_screen_mut().linefeed();
            }
            b'M' => self.current_screen_mut().reverse_linefeed(),
            b'c' => {
                // RIS - Reset to Initial State
                self.reset();
            }
            _ => {
                if !intermediates.is_empty() {
                    // Character set designation
                    // ( ) * + for G0-G3
                }
            }
        }
    }

    /// Handle OSC sequence
    fn handle_osc(&mut self, params: Vec<Vec<u8>>) {
        if params.is_empty() {
            return;
        }

        let cmd = params[0].iter().copied().collect::<Vec<_>>();
        let cmd_str = String::from_utf8_lossy(&cmd);
        
        match cmd_str.parse::<u32>() {
            Ok(0) | Ok(2) => {
                // Set title
                if params.len() > 1 {
                    self.title = String::from_utf8_lossy(&params[1]).to_string();
                }
            }
            Ok(4) => {
                // Set/query color palette
            }
            Ok(52) => {
                // Clipboard operations
            }
            _ => {}
        }
    }

    /// Get mutable reference to current screen
    fn current_screen_mut(&mut self) -> &mut Screen {
        if self.use_alt_screen {
            self.alt_screen.as_mut().unwrap_or(&mut self.screen)
        } else {
            &mut self.screen
        }
    }

    /// Reset terminal to initial state
    pub fn reset(&mut self) {
        self.screen = Screen::new(self.size.cols, self.size.rows);
        self.alt_screen = None;
        self.use_alt_screen = false;
        self.app_cursor_keys = false;
        self.app_keypad = false;
        self.bracketed_paste = false;
        self.mouse_mode = MouseMode::None;
        self.title.clear();
    }

    /// Resize the terminal
    pub fn resize(&mut self, size: TerminalSize) {
        self.size = size;
        self.screen.resize(size.cols, size.rows);
        if let Some(ref mut alt) = self.alt_screen {
            alt.resize(size.cols, size.rows);
        }
    }

    /// Get the screen buffer
    pub fn screen(&self) -> &Screen {
        if self.use_alt_screen {
            self.alt_screen.as_ref().unwrap_or(&self.screen)
        } else {
            &self.screen
        }
    }

    /// Get terminal size
    pub fn size(&self) -> TerminalSize {
        self.size
    }

    /// Get title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Check if using alternate screen
    pub fn is_alt_screen(&self) -> bool {
        self.use_alt_screen
    }

    /// Get mouse mode
    pub fn mouse_mode(&self) -> MouseMode {
        self.mouse_mode
    }

    /// Check if bracketed paste is enabled
    pub fn bracketed_paste(&self) -> bool {
        self.bracketed_paste
    }

    /// Check if application cursor keys mode
    pub fn app_cursor_keys(&self) -> bool {
        self.app_cursor_keys
    }

    /// Generate mouse button press event
    /// Returns bytes to send to remote
    pub fn mouse_press(&self, button: u8, col: u16, row: u16, modifiers: MouseModifiers) -> Option<Vec<u8>> {
        self.encode_mouse_event(button, col, row, modifiers, false)
    }

    /// Generate mouse button release event
    pub fn mouse_release(&self, col: u16, row: u16, modifiers: MouseModifiers) -> Option<Vec<u8>> {
        self.encode_mouse_event(3, col, row, modifiers, false) // Button 3 = release
    }

    /// Generate mouse motion event
    pub fn mouse_motion(&self, button: u8, col: u16, row: u16, modifiers: MouseModifiers) -> Option<Vec<u8>> {
        self.encode_mouse_event(button, col, row, modifiers, true)
    }

    /// Generate mouse wheel event
    pub fn mouse_wheel(&self, up: bool, col: u16, row: u16, modifiers: MouseModifiers) -> Option<Vec<u8>> {
        let button = if up { 64 } else { 65 }; // Wheel up = 64, Wheel down = 65
        self.encode_mouse_event(button, col, row, modifiers, false)
    }

    /// Encode mouse event based on current mode
    fn encode_mouse_event(&self, button: u8, col: u16, row: u16, modifiers: MouseModifiers, motion: bool) -> Option<Vec<u8>> {
        if self.mouse_mode == MouseMode::None {
            return None;
        }

        // Only report motion in ButtonEvent or AnyEvent modes
        if motion && self.mouse_mode != MouseMode::ButtonEvent && self.mouse_mode != MouseMode::AnyEvent {
            return None;
        }

        // Build button code
        let mut cb = button;
        if modifiers.shift { cb |= 4; }
        if modifiers.alt { cb |= 8; }
        if modifiers.ctrl { cb |= 16; }
        if motion { cb |= 32; }

        // X10 encoding: ESC [ M Cb Cx Cy
        // Values are 1-based and offset by 32
        let cx = (col.min(222) + 33) as u8;
        let cy = (row.min(222) + 33) as u8;
        
        Some(vec![0x1b, b'[', b'M', cb + 32, cx, cy])
    }

    /// Set mouse mode
    pub fn set_mouse_mode(&mut self, mode: MouseMode) {
        self.mouse_mode = mode;
    }
}

/// Mouse modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct MouseModifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

