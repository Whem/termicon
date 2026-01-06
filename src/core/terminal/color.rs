//! Terminal colors

use serde::{Deserialize, Serialize};

/// Terminal color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    /// Default color
    Default,
    /// Named color (0-15)
    Named(NamedColor),
    /// 256-color palette index
    Indexed(u8),
    /// True color RGB
    Rgb(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        Self::Default
    }
}

impl Color {
    /// Convert to RGB
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Default => (255, 255, 255), // White default
            Color::Named(named) => named.to_rgb(),
            Color::Indexed(idx) => index_to_rgb(*idx),
            Color::Rgb(r, g, b) => (*r, *g, *b),
        }
    }

    /// Convert to egui Color32
    #[cfg(feature = "gui")]
    pub fn to_egui(&self) -> egui::Color32 {
        let (r, g, b) = self.to_rgb();
        egui::Color32::from_rgb(r, g, b)
    }
}

/// Named terminal colors (ANSI 0-15)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl NamedColor {
    /// Create from ANSI color index (0-15)
    pub fn from_ansi(idx: u16) -> Self {
        match idx {
            0 => Self::Black,
            1 => Self::Red,
            2 => Self::Green,
            3 => Self::Yellow,
            4 => Self::Blue,
            5 => Self::Magenta,
            6 => Self::Cyan,
            7 => Self::White,
            8 => Self::BrightBlack,
            9 => Self::BrightRed,
            10 => Self::BrightGreen,
            11 => Self::BrightYellow,
            12 => Self::BrightBlue,
            13 => Self::BrightMagenta,
            14 => Self::BrightCyan,
            _ => Self::BrightWhite,
        }
    }

    /// Convert to RGB (default color scheme)
    pub fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Black => (0, 0, 0),
            Self::Red => (205, 49, 49),
            Self::Green => (13, 188, 121),
            Self::Yellow => (229, 229, 16),
            Self::Blue => (36, 114, 200),
            Self::Magenta => (188, 63, 188),
            Self::Cyan => (17, 168, 205),
            Self::White => (229, 229, 229),
            Self::BrightBlack => (102, 102, 102),
            Self::BrightRed => (241, 76, 76),
            Self::BrightGreen => (35, 209, 139),
            Self::BrightYellow => (245, 245, 67),
            Self::BrightBlue => (59, 142, 234),
            Self::BrightMagenta => (214, 112, 214),
            Self::BrightCyan => (41, 184, 219),
            Self::BrightWhite => (255, 255, 255),
        }
    }
}

/// Convert 256-color palette index to RGB
fn index_to_rgb(idx: u8) -> (u8, u8, u8) {
    match idx {
        // Standard colors (0-7)
        0 => (0, 0, 0),
        1 => (128, 0, 0),
        2 => (0, 128, 0),
        3 => (128, 128, 0),
        4 => (0, 0, 128),
        5 => (128, 0, 128),
        6 => (0, 128, 128),
        7 => (192, 192, 192),
        // High intensity colors (8-15)
        8 => (128, 128, 128),
        9 => (255, 0, 0),
        10 => (0, 255, 0),
        11 => (255, 255, 0),
        12 => (0, 0, 255),
        13 => (255, 0, 255),
        14 => (0, 255, 255),
        15 => (255, 255, 255),
        // 216 color cube (16-231)
        16..=231 => {
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            let to_component = |c: u8| if c == 0 { 0 } else { 55 + c * 40 };
            (to_component(r), to_component(g), to_component(b))
        }
        // Grayscale (232-255)
        232..=255 => {
            let gray = 8 + (idx - 232) * 10;
            (gray, gray, gray)
        }
    }
}

/// Color scheme/theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Background color
    pub background: Color,
    /// Foreground color
    pub foreground: Color,
    /// Cursor color
    pub cursor: Color,
    /// Selection color
    pub selection: Color,
    /// Named colors (0-15)
    pub palette: [Color; 16],
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(30, 30, 30),
            foreground: Color::Rgb(229, 229, 229),
            cursor: Color::Rgb(255, 255, 255),
            selection: Color::Rgb(68, 68, 68),
            palette: [
                Color::Named(NamedColor::Black),
                Color::Named(NamedColor::Red),
                Color::Named(NamedColor::Green),
                Color::Named(NamedColor::Yellow),
                Color::Named(NamedColor::Blue),
                Color::Named(NamedColor::Magenta),
                Color::Named(NamedColor::Cyan),
                Color::Named(NamedColor::White),
                Color::Named(NamedColor::BrightBlack),
                Color::Named(NamedColor::BrightRed),
                Color::Named(NamedColor::BrightGreen),
                Color::Named(NamedColor::BrightYellow),
                Color::Named(NamedColor::BrightBlue),
                Color::Named(NamedColor::BrightMagenta),
                Color::Named(NamedColor::BrightCyan),
                Color::Named(NamedColor::BrightWhite),
            ],
        }
    }
}




