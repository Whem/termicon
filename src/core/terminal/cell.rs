//! Terminal cell representation

use super::color::Color;
use serde::{Deserialize, Serialize};

/// A single cell in the terminal grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// Character displayed
    pub c: char,
    /// Cell style
    pub style: CellStyle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            style: CellStyle::default(),
        }
    }
}

impl Cell {
    /// Create a new cell with a character
    pub fn new(c: char, style: CellStyle) -> Self {
        Self { c, style }
    }

    /// Check if cell is empty (space with default style)
    pub fn is_empty(&self) -> bool {
        self.c == ' ' && self.style == CellStyle::default()
    }
}

/// Cell styling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CellStyle {
    /// Foreground color
    pub fg: Color,
    /// Background color
    pub bg: Color,
    /// Bold
    pub bold: bool,
    /// Dim/faint
    pub dim: bool,
    /// Italic
    pub italic: bool,
    /// Underline
    pub underline: bool,
    /// Blink
    pub blink: bool,
    /// Inverse/reverse video
    pub inverse: bool,
    /// Hidden/invisible
    pub hidden: bool,
    /// Strikethrough
    pub strikethrough: bool,
}

impl CellStyle {
    /// Create default style
    pub fn new() -> Self {
        Self::default()
    }

    /// Set foreground color
    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = color;
        self
    }

    /// Set background color
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = color;
        self
    }

    /// Set bold
    #[must_use]
    pub fn bold(mut self, v: bool) -> Self {
        self.bold = v;
        self
    }

    /// Set italic
    #[must_use]
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = v;
        self
    }

    /// Set underline
    #[must_use]
    pub fn underline(mut self, v: bool) -> Self {
        self.underline = v;
        self
    }

    /// Get effective foreground (accounting for inverse)
    pub fn effective_fg(&self) -> Color {
        if self.inverse {
            self.bg
        } else {
            self.fg
        }
    }

    /// Get effective background (accounting for inverse)
    pub fn effective_bg(&self) -> Color {
        if self.inverse {
            self.fg
        } else {
            self.bg
        }
    }
}







