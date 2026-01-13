//! Accessibility Features
//!
//! Provides high contrast mode, font scaling, and accessibility helpers.

use egui::{Color32, Visuals, Style, Stroke};
use serde::{Deserialize, Serialize};

/// Accessibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// High contrast mode
    pub high_contrast: bool,
    /// Font scale factor (1.0 = normal)
    pub font_scale: f32,
    /// Reduce motion (disable animations)
    pub reduce_motion: bool,
    /// Large cursor
    pub large_cursor: bool,
    /// Focus indicators
    pub focus_indicators: bool,
    /// Screen reader friendly mode
    pub screen_reader_mode: bool,
    /// Underline links
    pub underline_links: bool,
    /// Minimum contrast ratio
    pub min_contrast_ratio: f32,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            high_contrast: false,
            font_scale: 1.0,
            reduce_motion: false,
            large_cursor: false,
            focus_indicators: true,
            screen_reader_mode: false,
            underline_links: true,
            min_contrast_ratio: 4.5,
        }
    }
}

impl AccessibilitySettings {
    /// Apply settings to egui context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        
        // Apply font scaling
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size *= self.font_scale;
        }
        
        // Apply high contrast if enabled
        if self.high_contrast {
            style.visuals = high_contrast_visuals();
        }
        
        // Reduce animations
        if self.reduce_motion {
            style.animation_time = 0.0;
        }
        
        ctx.set_style(style);
    }
    
    /// Get scaled size
    pub fn scaled(&self, size: f32) -> f32 {
        size * self.font_scale
    }
    
    /// Check if color pair has sufficient contrast
    pub fn has_sufficient_contrast(&self, fg: Color32, bg: Color32) -> bool {
        contrast_ratio(fg, bg) >= self.min_contrast_ratio
    }
}

/// High contrast visuals for egui
pub fn high_contrast_visuals() -> Visuals {
    let mut visuals = Visuals::dark();
    
    // Pure black background
    visuals.panel_fill = Color32::BLACK;
    visuals.window_fill = Color32::BLACK;
    visuals.extreme_bg_color = Color32::BLACK;
    visuals.faint_bg_color = Color32::from_gray(20);
    
    // Pure white text
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.fg_stroke = Stroke::new(2.0, Color32::YELLOW);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::YELLOW);
    
    // High contrast borders
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::YELLOW);
    visuals.widgets.active.bg_stroke = Stroke::new(2.0, Color32::YELLOW);
    
    // Button backgrounds
    visuals.widgets.inactive.bg_fill = Color32::from_gray(30);
    visuals.widgets.hovered.bg_fill = Color32::from_gray(50);
    visuals.widgets.active.bg_fill = Color32::from_gray(70);
    
    // Selection
    visuals.selection.bg_fill = Color32::from_rgb(0, 100, 200);
    visuals.selection.stroke = Stroke::new(2.0, Color32::WHITE);
    
    // Hyperlinks
    visuals.hyperlink_color = Color32::from_rgb(100, 200, 255);
    
    // Warnings/errors
    visuals.warn_fg_color = Color32::YELLOW;
    visuals.error_fg_color = Color32::from_rgb(255, 100, 100);
    
    visuals
}

/// High contrast terminal palette
#[derive(Debug, Clone)]
pub struct HighContrastPalette {
    pub background: Color32,
    pub foreground: Color32,
    pub cursor: Color32,
    pub selection_bg: Color32,
    pub selection_fg: Color32,
    
    // ANSI colors (high contrast versions)
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
}

impl Default for HighContrastPalette {
    fn default() -> Self {
        Self {
            background: Color32::BLACK,
            foreground: Color32::WHITE,
            cursor: Color32::WHITE,
            selection_bg: Color32::from_rgb(255, 255, 0),
            selection_fg: Color32::BLACK,
            
            black: Color32::BLACK,
            red: Color32::from_rgb(255, 0, 0),
            green: Color32::from_rgb(0, 255, 0),
            yellow: Color32::from_rgb(255, 255, 0),
            blue: Color32::from_rgb(0, 128, 255),
            magenta: Color32::from_rgb(255, 0, 255),
            cyan: Color32::from_rgb(0, 255, 255),
            white: Color32::WHITE,
            bright_black: Color32::from_gray(128),
            bright_red: Color32::from_rgb(255, 128, 128),
            bright_green: Color32::from_rgb(128, 255, 128),
            bright_yellow: Color32::from_rgb(255, 255, 128),
            bright_blue: Color32::from_rgb(128, 192, 255),
            bright_magenta: Color32::from_rgb(255, 128, 255),
            bright_cyan: Color32::from_rgb(128, 255, 255),
            bright_white: Color32::WHITE,
        }
    }
}

impl HighContrastPalette {
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

/// Calculate relative luminance of a color
fn relative_luminance(color: Color32) -> f32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;
    
    let r = if r <= 0.03928 { r / 12.92 } else { ((r + 0.055) / 1.055).powf(2.4) };
    let g = if g <= 0.03928 { g / 12.92 } else { ((g + 0.055) / 1.055).powf(2.4) };
    let b = if b <= 0.03928 { b / 12.92 } else { ((b + 0.055) / 1.055).powf(2.4) };
    
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculate contrast ratio between two colors (WCAG formula)
pub fn contrast_ratio(color1: Color32, color2: Color32) -> f32 {
    let l1 = relative_luminance(color1);
    let l2 = relative_luminance(color2);
    
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    
    (lighter + 0.05) / (darker + 0.05)
}

/// Ensure minimum contrast by adjusting color
pub fn ensure_contrast(fg: Color32, bg: Color32, min_ratio: f32) -> Color32 {
    if contrast_ratio(fg, bg) >= min_ratio {
        return fg;
    }
    
    let bg_lum = relative_luminance(bg);
    
    // Determine if we need lighter or darker foreground
    if bg_lum > 0.5 {
        // Dark foreground needed
        Color32::BLACK
    } else {
        // Light foreground needed
        Color32::WHITE
    }
}

/// Font scale presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontScalePreset {
    Tiny,    // 75%
    Small,   // 87.5%
    Normal,  // 100%
    Large,   // 125%
    Larger,  // 150%
    Huge,    // 200%
}

impl FontScalePreset {
    pub fn factor(&self) -> f32 {
        match self {
            Self::Tiny => 0.75,
            Self::Small => 0.875,
            Self::Normal => 1.0,
            Self::Large => 1.25,
            Self::Larger => 1.5,
            Self::Huge => 2.0,
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            Self::Tiny => "Tiny (75%)",
            Self::Small => "Small (87.5%)",
            Self::Normal => "Normal (100%)",
            Self::Large => "Large (125%)",
            Self::Larger => "Larger (150%)",
            Self::Huge => "Huge (200%)",
        }
    }
    
    pub fn all() -> Vec<Self> {
        vec![
            Self::Tiny,
            Self::Small,
            Self::Normal,
            Self::Large,
            Self::Larger,
            Self::Huge,
        ]
    }
}

impl Default for FontScalePreset {
    fn default() -> Self {
        Self::Normal
    }
}

/// Focus ring style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusRing {
    pub color: Color32,
    pub width: f32,
    pub offset: f32,
    pub style: FocusRingStyle,
}

impl Default for FocusRing {
    fn default() -> Self {
        Self {
            color: Color32::from_rgb(0, 120, 215),
            width: 2.0,
            offset: 2.0,
            style: FocusRingStyle::Solid,
        }
    }
}

/// Focus ring style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusRingStyle {
    Solid,
    Dashed,
    Dotted,
    Double,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contrast_ratio() {
        // White on black should have maximum contrast
        let ratio = contrast_ratio(Color32::WHITE, Color32::BLACK);
        assert!(ratio > 20.0);
        
        // Same color should have minimum contrast
        let ratio = contrast_ratio(Color32::WHITE, Color32::WHITE);
        assert!(ratio < 1.1);
    }
    
    #[test]
    fn test_sufficient_contrast() {
        let settings = AccessibilitySettings::default();
        
        // White on black should pass
        assert!(settings.has_sufficient_contrast(Color32::WHITE, Color32::BLACK));
        
        // Similar grays should fail
        assert!(!settings.has_sufficient_contrast(
            Color32::from_gray(128),
            Color32::from_gray(140)
        ));
    }
    
    #[test]
    fn test_high_contrast_palette() {
        let palette = HighContrastPalette::default();
        
        // All colors should have sufficient contrast against background
        for i in 1..16 {
            let color = palette.ansi_color(i);
            let ratio = contrast_ratio(color, palette.background);
            assert!(ratio >= 4.5, "Color {} has insufficient contrast: {}", i, ratio);
        }
    }
    
    #[test]
    fn test_font_scale() {
        let mut settings = AccessibilitySettings::default();
        settings.font_scale = 1.5;
        
        assert_eq!(settings.scaled(12.0), 18.0);
        assert_eq!(settings.scaled(16.0), 24.0);
    }
}




