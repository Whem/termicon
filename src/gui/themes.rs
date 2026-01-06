//! Custom Color Schemes and Themes
//!
//! Provides a variety of color schemes for the terminal and UI.

use egui::{Color32, Visuals, Stroke};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Terminal color palette (16 ANSI colors + extras)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPalette {
    pub name: String,
    pub description: String,
    
    // Standard 16 ANSI colors
    pub black: Color32,
    pub red: Color32,
    pub green: Color32,
    pub yellow: Color32,
    pub blue: Color32,
    pub magenta: Color32,
    pub cyan: Color32,
    pub white: Color32,
    pub bright_black: Color32,
    pub bright_red: Color32,
    pub bright_green: Color32,
    pub bright_yellow: Color32,
    pub bright_blue: Color32,
    pub bright_magenta: Color32,
    pub bright_cyan: Color32,
    pub bright_white: Color32,
    
    // Terminal colors
    pub foreground: Color32,
    pub background: Color32,
    pub cursor: Color32,
    pub selection: Color32,
}

impl Default for TerminalPalette {
    fn default() -> Self {
        Self::solarized_dark()
    }
}

impl TerminalPalette {
    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            description: "Precision colors for machines and people".to_string(),
            black: Color32::from_rgb(7, 54, 66),
            red: Color32::from_rgb(220, 50, 47),
            green: Color32::from_rgb(133, 153, 0),
            yellow: Color32::from_rgb(181, 137, 0),
            blue: Color32::from_rgb(38, 139, 210),
            magenta: Color32::from_rgb(211, 54, 130),
            cyan: Color32::from_rgb(42, 161, 152),
            white: Color32::from_rgb(238, 232, 213),
            bright_black: Color32::from_rgb(0, 43, 54),
            bright_red: Color32::from_rgb(203, 75, 22),
            bright_green: Color32::from_rgb(88, 110, 117),
            bright_yellow: Color32::from_rgb(101, 123, 131),
            bright_blue: Color32::from_rgb(131, 148, 150),
            bright_magenta: Color32::from_rgb(108, 113, 196),
            bright_cyan: Color32::from_rgb(147, 161, 161),
            bright_white: Color32::from_rgb(253, 246, 227),
            foreground: Color32::from_rgb(131, 148, 150),
            background: Color32::from_rgb(0, 43, 54),
            cursor: Color32::from_rgb(133, 153, 0),
            selection: Color32::from_rgba_unmultiplied(7, 54, 66, 180),
        }
    }
    
    /// Solarized Light theme
    pub fn solarized_light() -> Self {
        Self {
            name: "Solarized Light".to_string(),
            description: "Light variant of Solarized".to_string(),
            black: Color32::from_rgb(238, 232, 213),
            red: Color32::from_rgb(220, 50, 47),
            green: Color32::from_rgb(133, 153, 0),
            yellow: Color32::from_rgb(181, 137, 0),
            blue: Color32::from_rgb(38, 139, 210),
            magenta: Color32::from_rgb(211, 54, 130),
            cyan: Color32::from_rgb(42, 161, 152),
            white: Color32::from_rgb(7, 54, 66),
            bright_black: Color32::from_rgb(253, 246, 227),
            bright_red: Color32::from_rgb(203, 75, 22),
            bright_green: Color32::from_rgb(147, 161, 161),
            bright_yellow: Color32::from_rgb(131, 148, 150),
            bright_blue: Color32::from_rgb(101, 123, 131),
            bright_magenta: Color32::from_rgb(108, 113, 196),
            bright_cyan: Color32::from_rgb(88, 110, 117),
            bright_white: Color32::from_rgb(0, 43, 54),
            foreground: Color32::from_rgb(101, 123, 131),
            background: Color32::from_rgb(253, 246, 227),
            cursor: Color32::from_rgb(211, 54, 130),
            selection: Color32::from_rgba_unmultiplied(238, 232, 213, 180),
        }
    }
    
    /// Monokai theme
    pub fn monokai() -> Self {
        Self {
            name: "Monokai".to_string(),
            description: "Iconic dark color scheme".to_string(),
            black: Color32::from_rgb(39, 40, 34),
            red: Color32::from_rgb(249, 38, 114),
            green: Color32::from_rgb(166, 226, 46),
            yellow: Color32::from_rgb(244, 191, 117),
            blue: Color32::from_rgb(102, 217, 239),
            magenta: Color32::from_rgb(174, 129, 255),
            cyan: Color32::from_rgb(161, 239, 228),
            white: Color32::from_rgb(248, 248, 242),
            bright_black: Color32::from_rgb(117, 113, 94),
            bright_red: Color32::from_rgb(249, 38, 114),
            bright_green: Color32::from_rgb(166, 226, 46),
            bright_yellow: Color32::from_rgb(244, 191, 117),
            bright_blue: Color32::from_rgb(102, 217, 239),
            bright_magenta: Color32::from_rgb(174, 129, 255),
            bright_cyan: Color32::from_rgb(161, 239, 228),
            bright_white: Color32::from_rgb(248, 248, 242),
            foreground: Color32::from_rgb(248, 248, 242),
            background: Color32::from_rgb(39, 40, 34),
            cursor: Color32::from_rgb(248, 248, 242),
            selection: Color32::from_rgba_unmultiplied(73, 72, 62, 200),
        }
    }
    
    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            description: "Dark theme for vampires".to_string(),
            black: Color32::from_rgb(40, 42, 54),
            red: Color32::from_rgb(255, 85, 85),
            green: Color32::from_rgb(80, 250, 123),
            yellow: Color32::from_rgb(241, 250, 140),
            blue: Color32::from_rgb(189, 147, 249),
            magenta: Color32::from_rgb(255, 121, 198),
            cyan: Color32::from_rgb(139, 233, 253),
            white: Color32::from_rgb(248, 248, 242),
            bright_black: Color32::from_rgb(68, 71, 90),
            bright_red: Color32::from_rgb(255, 110, 103),
            bright_green: Color32::from_rgb(90, 247, 142),
            bright_yellow: Color32::from_rgb(244, 249, 157),
            bright_blue: Color32::from_rgb(202, 169, 250),
            bright_magenta: Color32::from_rgb(255, 146, 208),
            bright_cyan: Color32::from_rgb(154, 237, 254),
            bright_white: Color32::from_rgb(255, 255, 255),
            foreground: Color32::from_rgb(248, 248, 242),
            background: Color32::from_rgb(40, 42, 54),
            cursor: Color32::from_rgb(248, 248, 242),
            selection: Color32::from_rgba_unmultiplied(68, 71, 90, 200),
        }
    }
    
    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            description: "Arctic, north-bluish color palette".to_string(),
            black: Color32::from_rgb(46, 52, 64),
            red: Color32::from_rgb(191, 97, 106),
            green: Color32::from_rgb(163, 190, 140),
            yellow: Color32::from_rgb(235, 203, 139),
            blue: Color32::from_rgb(129, 161, 193),
            magenta: Color32::from_rgb(180, 142, 173),
            cyan: Color32::from_rgb(136, 192, 208),
            white: Color32::from_rgb(229, 233, 240),
            bright_black: Color32::from_rgb(76, 86, 106),
            bright_red: Color32::from_rgb(191, 97, 106),
            bright_green: Color32::from_rgb(163, 190, 140),
            bright_yellow: Color32::from_rgb(235, 203, 139),
            bright_blue: Color32::from_rgb(129, 161, 193),
            bright_magenta: Color32::from_rgb(180, 142, 173),
            bright_cyan: Color32::from_rgb(143, 188, 187),
            bright_white: Color32::from_rgb(236, 239, 244),
            foreground: Color32::from_rgb(216, 222, 233),
            background: Color32::from_rgb(46, 52, 64),
            cursor: Color32::from_rgb(216, 222, 233),
            selection: Color32::from_rgba_unmultiplied(67, 76, 94, 200),
        }
    }
    
    /// Gruvbox Dark theme
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "Gruvbox Dark".to_string(),
            description: "Retro groove color scheme".to_string(),
            black: Color32::from_rgb(40, 40, 40),
            red: Color32::from_rgb(204, 36, 29),
            green: Color32::from_rgb(152, 151, 26),
            yellow: Color32::from_rgb(215, 153, 33),
            blue: Color32::from_rgb(69, 133, 136),
            magenta: Color32::from_rgb(177, 98, 134),
            cyan: Color32::from_rgb(104, 157, 106),
            white: Color32::from_rgb(168, 153, 132),
            bright_black: Color32::from_rgb(146, 131, 116),
            bright_red: Color32::from_rgb(251, 73, 52),
            bright_green: Color32::from_rgb(184, 187, 38),
            bright_yellow: Color32::from_rgb(250, 189, 47),
            bright_blue: Color32::from_rgb(131, 165, 152),
            bright_magenta: Color32::from_rgb(211, 134, 155),
            bright_cyan: Color32::from_rgb(142, 192, 124),
            bright_white: Color32::from_rgb(235, 219, 178),
            foreground: Color32::from_rgb(235, 219, 178),
            background: Color32::from_rgb(40, 40, 40),
            cursor: Color32::from_rgb(235, 219, 178),
            selection: Color32::from_rgba_unmultiplied(80, 73, 69, 200),
        }
    }
    
    /// Gruvbox Light theme
    pub fn gruvbox_light() -> Self {
        Self {
            name: "Gruvbox Light".to_string(),
            description: "Light variant of Gruvbox".to_string(),
            black: Color32::from_rgb(251, 241, 199),
            red: Color32::from_rgb(204, 36, 29),
            green: Color32::from_rgb(152, 151, 26),
            yellow: Color32::from_rgb(215, 153, 33),
            blue: Color32::from_rgb(69, 133, 136),
            magenta: Color32::from_rgb(177, 98, 134),
            cyan: Color32::from_rgb(104, 157, 106),
            white: Color32::from_rgb(60, 56, 54),
            bright_black: Color32::from_rgb(189, 174, 147),
            bright_red: Color32::from_rgb(157, 0, 6),
            bright_green: Color32::from_rgb(121, 116, 14),
            bright_yellow: Color32::from_rgb(181, 118, 20),
            bright_blue: Color32::from_rgb(7, 102, 120),
            bright_magenta: Color32::from_rgb(143, 63, 113),
            bright_cyan: Color32::from_rgb(66, 123, 88),
            bright_white: Color32::from_rgb(40, 40, 40),
            foreground: Color32::from_rgb(60, 56, 54),
            background: Color32::from_rgb(251, 241, 199),
            cursor: Color32::from_rgb(60, 56, 54),
            selection: Color32::from_rgba_unmultiplied(213, 196, 161, 200),
        }
    }
    
    /// One Dark theme (Atom)
    pub fn one_dark() -> Self {
        Self {
            name: "One Dark".to_string(),
            description: "Atom's iconic One Dark theme".to_string(),
            black: Color32::from_rgb(40, 44, 52),
            red: Color32::from_rgb(224, 108, 117),
            green: Color32::from_rgb(152, 195, 121),
            yellow: Color32::from_rgb(229, 192, 123),
            blue: Color32::from_rgb(97, 175, 239),
            magenta: Color32::from_rgb(198, 120, 221),
            cyan: Color32::from_rgb(86, 182, 194),
            white: Color32::from_rgb(171, 178, 191),
            bright_black: Color32::from_rgb(92, 99, 112),
            bright_red: Color32::from_rgb(224, 108, 117),
            bright_green: Color32::from_rgb(152, 195, 121),
            bright_yellow: Color32::from_rgb(229, 192, 123),
            bright_blue: Color32::from_rgb(97, 175, 239),
            bright_magenta: Color32::from_rgb(198, 120, 221),
            bright_cyan: Color32::from_rgb(86, 182, 194),
            bright_white: Color32::from_rgb(255, 255, 255),
            foreground: Color32::from_rgb(171, 178, 191),
            background: Color32::from_rgb(40, 44, 52),
            cursor: Color32::from_rgb(97, 175, 239),
            selection: Color32::from_rgba_unmultiplied(62, 68, 81, 200),
        }
    }
    
    /// Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            description: "Soothing pastel theme".to_string(),
            black: Color32::from_rgb(69, 71, 90),
            red: Color32::from_rgb(243, 139, 168),
            green: Color32::from_rgb(166, 227, 161),
            yellow: Color32::from_rgb(249, 226, 175),
            blue: Color32::from_rgb(137, 180, 250),
            magenta: Color32::from_rgb(245, 194, 231),
            cyan: Color32::from_rgb(148, 226, 213),
            white: Color32::from_rgb(186, 194, 222),
            bright_black: Color32::from_rgb(88, 91, 112),
            bright_red: Color32::from_rgb(243, 139, 168),
            bright_green: Color32::from_rgb(166, 227, 161),
            bright_yellow: Color32::from_rgb(249, 226, 175),
            bright_blue: Color32::from_rgb(137, 180, 250),
            bright_magenta: Color32::from_rgb(245, 194, 231),
            bright_cyan: Color32::from_rgb(148, 226, 213),
            bright_white: Color32::from_rgb(205, 214, 244),
            foreground: Color32::from_rgb(205, 214, 244),
            background: Color32::from_rgb(30, 30, 46),
            cursor: Color32::from_rgb(245, 224, 220),
            selection: Color32::from_rgba_unmultiplied(88, 91, 112, 200),
        }
    }
    
    /// Tokyo Night theme
    pub fn tokyo_night() -> Self {
        Self {
            name: "Tokyo Night".to_string(),
            description: "Clean dark theme with vivid colors".to_string(),
            black: Color32::from_rgb(31, 35, 53),
            red: Color32::from_rgb(247, 118, 142),
            green: Color32::from_rgb(158, 206, 106),
            yellow: Color32::from_rgb(224, 175, 104),
            blue: Color32::from_rgb(122, 162, 247),
            magenta: Color32::from_rgb(187, 154, 247),
            cyan: Color32::from_rgb(125, 207, 255),
            white: Color32::from_rgb(192, 202, 245),
            bright_black: Color32::from_rgb(68, 75, 106),
            bright_red: Color32::from_rgb(247, 118, 142),
            bright_green: Color32::from_rgb(158, 206, 106),
            bright_yellow: Color32::from_rgb(224, 175, 104),
            bright_blue: Color32::from_rgb(122, 162, 247),
            bright_magenta: Color32::from_rgb(187, 154, 247),
            bright_cyan: Color32::from_rgb(125, 207, 255),
            bright_white: Color32::from_rgb(220, 228, 253),
            foreground: Color32::from_rgb(192, 202, 245),
            background: Color32::from_rgb(26, 27, 38),
            cursor: Color32::from_rgb(192, 202, 245),
            selection: Color32::from_rgba_unmultiplied(51, 59, 91, 200),
        }
    }
    
    /// Retro green terminal
    pub fn retro_green() -> Self {
        Self {
            name: "Retro Green".to_string(),
            description: "Classic green phosphor CRT".to_string(),
            black: Color32::from_rgb(0, 0, 0),
            red: Color32::from_rgb(0, 128, 0),
            green: Color32::from_rgb(0, 255, 0),
            yellow: Color32::from_rgb(128, 255, 128),
            blue: Color32::from_rgb(0, 192, 0),
            magenta: Color32::from_rgb(0, 160, 0),
            cyan: Color32::from_rgb(64, 255, 64),
            white: Color32::from_rgb(0, 255, 0),
            bright_black: Color32::from_rgb(0, 64, 0),
            bright_red: Color32::from_rgb(0, 192, 0),
            bright_green: Color32::from_rgb(128, 255, 128),
            bright_yellow: Color32::from_rgb(192, 255, 192),
            bright_blue: Color32::from_rgb(0, 224, 0),
            bright_magenta: Color32::from_rgb(0, 192, 0),
            bright_cyan: Color32::from_rgb(128, 255, 128),
            bright_white: Color32::from_rgb(192, 255, 192),
            foreground: Color32::from_rgb(0, 255, 0),
            background: Color32::from_rgb(0, 0, 0),
            cursor: Color32::from_rgb(0, 255, 0),
            selection: Color32::from_rgba_unmultiplied(0, 128, 0, 150),
        }
    }
    
    /// Retro amber terminal
    pub fn retro_amber() -> Self {
        Self {
            name: "Retro Amber".to_string(),
            description: "Classic amber phosphor CRT".to_string(),
            black: Color32::from_rgb(0, 0, 0),
            red: Color32::from_rgb(255, 176, 0),
            green: Color32::from_rgb(255, 191, 0),
            yellow: Color32::from_rgb(255, 223, 128),
            blue: Color32::from_rgb(255, 160, 0),
            magenta: Color32::from_rgb(255, 144, 0),
            cyan: Color32::from_rgb(255, 207, 64),
            white: Color32::from_rgb(255, 191, 0),
            bright_black: Color32::from_rgb(128, 96, 0),
            bright_red: Color32::from_rgb(255, 191, 0),
            bright_green: Color32::from_rgb(255, 223, 128),
            bright_yellow: Color32::from_rgb(255, 239, 192),
            bright_blue: Color32::from_rgb(255, 176, 0),
            bright_magenta: Color32::from_rgb(255, 160, 0),
            bright_cyan: Color32::from_rgb(255, 223, 128),
            bright_white: Color32::from_rgb(255, 239, 192),
            foreground: Color32::from_rgb(255, 191, 0),
            background: Color32::from_rgb(0, 0, 0),
            cursor: Color32::from_rgb(255, 191, 0),
            selection: Color32::from_rgba_unmultiplied(128, 96, 0, 150),
        }
    }
    
    /// Get all available palettes
    pub fn all_palettes() -> Vec<Self> {
        vec![
            Self::solarized_dark(),
            Self::solarized_light(),
            Self::monokai(),
            Self::dracula(),
            Self::nord(),
            Self::gruvbox_dark(),
            Self::gruvbox_light(),
            Self::one_dark(),
            Self::catppuccin_mocha(),
            Self::tokyo_night(),
            Self::retro_green(),
            Self::retro_amber(),
        ]
    }
    
    /// Get ANSI color by index (0-15)
    pub fn ansi_color(&self, index: u8) -> Color32 {
        match index {
            0 => self.black,
            1 => self.red,
            2 => self.green,
            3 => self.yellow,
            4 => self.blue,
            5 => self.magenta,
            6 => self.cyan,
            7 => self.white,
            8 => self.bright_black,
            9 => self.bright_red,
            10 => self.bright_green,
            11 => self.bright_yellow,
            12 => self.bright_blue,
            13 => self.bright_magenta,
            14 => self.bright_cyan,
            15 => self.bright_white,
            _ => self.white,
        }
    }
}

/// UI theme type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiTheme {
    Dark,
    Light,
    System,
    Custom,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::Dark
    }
}

/// Complete theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub ui_theme: UiTheme,
    pub terminal_palette: TerminalPalette,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    pub custom_colors: HashMap<String, Color32>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            ui_theme: UiTheme::Dark,
            terminal_palette: TerminalPalette::default(),
            font_family: "JetBrains Mono".to_string(),
            font_size: 13.0,
            line_height: 1.2,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            custom_colors: HashMap::new(),
        }
    }
}

/// Cursor style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Block
    }
}

/// Theme manager
#[derive(Debug)]
pub struct ThemeManager {
    pub current: ThemeConfig,
    pub palettes: Vec<TerminalPalette>,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current: ThemeConfig::default(),
            palettes: TerminalPalette::all_palettes(),
        }
    }
    
    /// Apply UI theme to egui context
    pub fn apply_ui_theme(&self, ctx: &egui::Context) {
        match self.current.ui_theme {
            UiTheme::Dark => {
                ctx.set_visuals(Visuals::dark());
            }
            UiTheme::Light => {
                ctx.set_visuals(Visuals::light());
            }
            UiTheme::System => {
                // TODO: Detect system theme
                ctx.set_visuals(Visuals::dark());
            }
            UiTheme::Custom => {
                self.apply_custom_visuals(ctx);
            }
        }
    }
    
    /// Apply custom visuals based on terminal palette
    fn apply_custom_visuals(&self, ctx: &egui::Context) {
        let palette = &self.current.terminal_palette;
        let mut visuals = Visuals::dark();
        
        // Override with palette colors
        visuals.widgets.noninteractive.bg_fill = palette.background;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.foreground);
        visuals.widgets.inactive.bg_fill = palette.bright_black;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.foreground);
        visuals.widgets.hovered.bg_fill = palette.selection;
        visuals.widgets.active.bg_fill = palette.blue;
        visuals.selection.bg_fill = palette.selection;
        visuals.selection.stroke = Stroke::new(1.0, palette.cyan);
        
        ctx.set_visuals(visuals);
    }
    
    /// Set terminal palette by name
    pub fn set_palette(&mut self, name: &str) -> bool {
        if let Some(palette) = self.palettes.iter().find(|p| p.name == name) {
            self.current.terminal_palette = palette.clone();
            true
        } else {
            false
        }
    }
    
    /// Get all palette names
    pub fn palette_names(&self) -> Vec<&str> {
        self.palettes.iter().map(|p| p.name.as_str()).collect()
    }
    
    /// Save theme to file
    pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.current)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }
    
    /// Load theme from file
    pub fn load(path: &std::path::Path) -> std::io::Result<ThemeConfig> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub name: String,
    pub size: f32,
    pub line_height: f32,
    pub ligatures: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            name: "JetBrains Mono".to_string(),
            size: 13.0,
            line_height: 1.2,
            ligatures: true,
        }
    }
}

/// Available monospace fonts
pub fn available_fonts() -> Vec<&'static str> {
    vec![
        "JetBrains Mono",
        "Fira Code",
        "Source Code Pro",
        "Cascadia Code",
        "Consolas",
        "Monaco",
        "Menlo",
        "Ubuntu Mono",
        "Roboto Mono",
        "IBM Plex Mono",
        "Hack",
        "Inconsolata",
        "Droid Sans Mono",
        "DejaVu Sans Mono",
        "Liberation Mono",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_palette_all() {
        let palettes = TerminalPalette::all_palettes();
        assert!(!palettes.is_empty());
        
        for palette in &palettes {
            assert!(!palette.name.is_empty());
        }
    }
    
    #[test]
    fn test_ansi_colors() {
        let palette = TerminalPalette::default();
        
        for i in 0..16 {
            let color = palette.ansi_color(i);
            assert_ne!(color, Color32::TRANSPARENT);
        }
    }
    
    #[test]
    fn test_theme_manager() {
        let mut manager = ThemeManager::new();
        
        assert!(manager.set_palette("Monokai"));
        assert_eq!(manager.current.terminal_palette.name, "Monokai");
        
        assert!(!manager.set_palette("NonExistent"));
    }
}

