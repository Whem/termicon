//! Application theming

use egui::{Color32, FontFamily, FontId, Style, TextStyle, Visuals};

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Dark theme (default)
    #[default]
    Dark,
    /// Light theme
    Light,
    /// High contrast
    HighContrast,
}

impl Theme {
    /// Apply theme to egui context
    pub fn apply(&self, ctx: &egui::Context) {
        match self {
            Self::Dark => ctx.set_visuals(dark_visuals()),
            Self::Light => ctx.set_visuals(light_visuals()),
            Self::HighContrast => ctx.set_visuals(high_contrast_visuals()),
        }

        // Set custom fonts
        let mut style = (*ctx.style()).clone();
        configure_fonts(&mut style);
        ctx.set_style(style);
    }

    /// Get all available themes
    pub fn all() -> &'static [Self] {
        &[Self::Dark, Self::Light, Self::HighContrast]
    }

    /// Get theme name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::HighContrast => "High Contrast",
        }
    }
}

fn dark_visuals() -> Visuals {
    let mut visuals = Visuals::dark();

    // Customize dark theme colors
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 35);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 48);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 55, 65);
    visuals.widgets.active.bg_fill = Color32::from_rgb(70, 130, 180);

    // Panel colors
    visuals.panel_fill = Color32::from_rgb(25, 25, 30);
    visuals.window_fill = Color32::from_rgb(30, 30, 35);

    // Selection color
    visuals.selection.bg_fill = Color32::from_rgb(70, 130, 180);
    visuals.selection.stroke.color = Color32::from_rgb(100, 160, 210);

    // Accent colors
    visuals.hyperlink_color = Color32::from_rgb(100, 180, 255);
    visuals.warn_fg_color = Color32::from_rgb(255, 200, 100);
    visuals.error_fg_color = Color32::from_rgb(255, 100, 100);

    visuals
}

fn light_visuals() -> Visuals {
    let mut visuals = Visuals::light();

    // Customize light theme colors
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(245, 245, 248);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(235, 235, 240);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(220, 220, 230);
    visuals.widgets.active.bg_fill = Color32::from_rgb(70, 130, 180);

    // Panel colors
    visuals.panel_fill = Color32::from_rgb(250, 250, 252);
    visuals.window_fill = Color32::from_rgb(255, 255, 255);

    // Selection color
    visuals.selection.bg_fill = Color32::from_rgb(70, 130, 180);

    visuals
}

fn high_contrast_visuals() -> Visuals {
    let mut visuals = Visuals::dark();

    // High contrast colors
    visuals.widgets.noninteractive.bg_fill = Color32::BLACK;
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 20, 20);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 40, 40);
    visuals.widgets.active.bg_fill = Color32::from_rgb(255, 255, 0);

    visuals.widgets.noninteractive.fg_stroke.color = Color32::WHITE;
    visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
    visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
    visuals.widgets.active.fg_stroke.color = Color32::BLACK;

    visuals.panel_fill = Color32::BLACK;
    visuals.window_fill = Color32::BLACK;

    visuals.selection.bg_fill = Color32::from_rgb(255, 255, 0);
    visuals.selection.stroke.color = Color32::WHITE;

    visuals
}

fn configure_fonts(style: &mut Style) {
    // Configure text styles
    style.text_styles = [
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
    ]
    .into();
}

/// Terminal colors
pub struct TerminalColors {
    /// Background color
    pub background: Color32,
    /// Default text color
    pub foreground: Color32,
    /// Cursor color
    pub cursor: Color32,
    /// Selection background
    pub selection_bg: Color32,
    /// TX data color
    pub tx_color: Color32,
    /// RX data color
    pub rx_color: Color32,
    /// Error color
    pub error_color: Color32,
    /// Warning color
    pub warning_color: Color32,
    /// Success color
    pub success_color: Color32,
    /// Timestamp color
    pub timestamp_color: Color32,
}

impl Default for TerminalColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl TerminalColors {
    /// Dark terminal colors
    pub fn dark() -> Self {
        Self {
            background: Color32::from_rgb(20, 20, 25),
            foreground: Color32::from_rgb(220, 220, 220),
            cursor: Color32::from_rgb(100, 180, 255),
            selection_bg: Color32::from_rgba_premultiplied(70, 130, 180, 100),
            tx_color: Color32::from_rgb(100, 200, 100),
            rx_color: Color32::from_rgb(220, 220, 220),
            error_color: Color32::from_rgb(255, 100, 100),
            warning_color: Color32::from_rgb(255, 200, 100),
            success_color: Color32::from_rgb(100, 255, 100),
            timestamp_color: Color32::from_rgb(150, 150, 160),
        }
    }

    /// Light terminal colors
    pub fn light() -> Self {
        Self {
            background: Color32::from_rgb(255, 255, 255),
            foreground: Color32::from_rgb(30, 30, 30),
            cursor: Color32::from_rgb(0, 100, 200),
            selection_bg: Color32::from_rgba_premultiplied(70, 130, 180, 80),
            tx_color: Color32::from_rgb(0, 150, 0),
            rx_color: Color32::from_rgb(30, 30, 30),
            error_color: Color32::from_rgb(200, 0, 0),
            warning_color: Color32::from_rgb(200, 150, 0),
            success_color: Color32::from_rgb(0, 180, 0),
            timestamp_color: Color32::from_rgb(120, 120, 130),
        }
    }
}

/// Status LED colors
pub struct StatusLedColors {
    /// LED on color
    pub on: Color32,
    /// LED off color
    pub off: Color32,
    /// LED active/blinking color
    pub active: Color32,
}

impl StatusLedColors {
    /// Green LED (TX, RX, connected)
    pub fn green() -> Self {
        Self {
            on: Color32::from_rgb(50, 205, 50),
            off: Color32::from_rgb(30, 60, 30),
            active: Color32::from_rgb(100, 255, 100),
        }
    }

    /// Red LED (error, CTS, DCD)
    pub fn red() -> Self {
        Self {
            on: Color32::from_rgb(220, 50, 50),
            off: Color32::from_rgb(60, 30, 30),
            active: Color32::from_rgb(255, 100, 100),
        }
    }

    /// Yellow LED (warning, RTS, DTR)
    pub fn yellow() -> Self {
        Self {
            on: Color32::from_rgb(220, 180, 50),
            off: Color32::from_rgb(60, 50, 30),
            active: Color32::from_rgb(255, 220, 100),
        }
    }

    /// Blue LED (DSR, RI)
    pub fn blue() -> Self {
        Self {
            on: Color32::from_rgb(70, 130, 200),
            off: Color32::from_rgb(30, 40, 60),
            active: Color32::from_rgb(100, 180, 255),
        }
    }
}








