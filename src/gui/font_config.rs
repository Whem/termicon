//! Font Configuration UI
//!
//! Provides UI for configuring terminal fonts, sizes, and rendering options.

use egui::{self, Context, Window, RichText, Color32};
use serde::{Deserialize, Serialize};

/// Font family options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontFamily {
    JetBrainsMono,
    FiraCode,
    SourceCodePro,
    CascadiaCode,
    Consolas,
    Monaco,
    Menlo,
    UbuntuMono,
    RobotoMono,
    Hack,
    Inconsolata,
    Custom(String),
}

impl FontFamily {
    pub fn name(&self) -> &str {
        match self {
            Self::JetBrainsMono => "JetBrains Mono",
            Self::FiraCode => "Fira Code",
            Self::SourceCodePro => "Source Code Pro",
            Self::CascadiaCode => "Cascadia Code",
            Self::Consolas => "Consolas",
            Self::Monaco => "Monaco",
            Self::Menlo => "Menlo",
            Self::UbuntuMono => "Ubuntu Mono",
            Self::RobotoMono => "Roboto Mono",
            Self::Hack => "Hack",
            Self::Inconsolata => "Inconsolata",
            Self::Custom(name) => name,
        }
    }
    
    pub fn all() -> Vec<Self> {
        vec![
            Self::JetBrainsMono,
            Self::FiraCode,
            Self::SourceCodePro,
            Self::CascadiaCode,
            Self::Consolas,
            Self::Monaco,
            Self::Menlo,
            Self::UbuntuMono,
            Self::RobotoMono,
            Self::Hack,
            Self::Inconsolata,
        ]
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        Self::JetBrainsMono
    }
}

/// Font rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
}

impl FontWeight {
    pub fn name(&self) -> &str {
        match self {
            Self::Thin => "Thin",
            Self::Light => "Light",
            Self::Regular => "Regular",
            Self::Medium => "Medium",
            Self::SemiBold => "SemiBold",
            Self::Bold => "Bold",
        }
    }
    
    pub fn all() -> Vec<Self> {
        vec![
            Self::Thin,
            Self::Light,
            Self::Regular,
            Self::Medium,
            Self::SemiBold,
            Self::Bold,
        ]
    }
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Regular
    }
}

/// Font configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSettings {
    /// Font family
    pub family: FontFamily,
    /// Font size in points
    pub size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Letter spacing (em units)
    pub letter_spacing: f32,
    /// Font weight
    pub weight: FontWeight,
    /// Enable font ligatures
    pub ligatures: bool,
    /// Enable anti-aliasing
    pub antialiasing: bool,
    /// Subpixel rendering (for LCD)
    pub subpixel_rendering: bool,
    /// Bold text weight
    pub bold_weight: FontWeight,
    /// Italic style
    pub italic_enabled: bool,
    /// Underline style
    pub underline_enabled: bool,
    /// Cursor blink rate (ms, 0 = no blink)
    pub cursor_blink_ms: u32,
    /// Cell width adjustment (percentage)
    pub cell_width_adjust: f32,
    /// Cell height adjustment (percentage)
    pub cell_height_adjust: f32,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: FontFamily::default(),
            size: 13.0,
            line_height: 1.2,
            letter_spacing: 0.0,
            weight: FontWeight::Regular,
            ligatures: true,
            antialiasing: true,
            subpixel_rendering: true,
            bold_weight: FontWeight::Bold,
            italic_enabled: true,
            underline_enabled: true,
            cursor_blink_ms: 530,
            cell_width_adjust: 100.0,
            cell_height_adjust: 100.0,
        }
    }
}

impl FontSettings {
    /// Calculate cell width in pixels
    pub fn cell_width(&self) -> f32 {
        self.size * 0.6 * (self.cell_width_adjust / 100.0)
    }
    
    /// Calculate cell height in pixels
    pub fn cell_height(&self) -> f32 {
        self.size * self.line_height * (self.cell_height_adjust / 100.0)
    }
    
    /// Reset to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Font configuration dialog
pub struct FontConfigDialog {
    /// Current settings (being edited)
    pub settings: FontSettings,
    /// Original settings (for cancel)
    original: FontSettings,
    /// Dialog open state
    pub open: bool,
    /// Preview text
    preview_text: String,
}

impl Default for FontConfigDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl FontConfigDialog {
    pub fn new() -> Self {
        Self {
            settings: FontSettings::default(),
            original: FontSettings::default(),
            open: false,
            preview_text: "The quick brown fox jumps over the lazy dog.\nABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz\n0123456789 !@#$%^&*()_+-=[]{}|;':\",./<>?".to_string(),
        }
    }
    
    /// Open dialog with current settings
    pub fn open(&mut self, current: FontSettings) {
        self.settings = current.clone();
        self.original = current;
        self.open = true;
    }
    
    /// Show the dialog
    pub fn show(&mut self, ctx: &Context) -> Option<FontSettings> {
        let mut result = None;
        
        if !self.open {
            return None;
        }
        
        Window::new("Font Configuration")
            .collapsible(false)
            .resizable(true)
            .min_width(450.0)
            .show(ctx, |ui| {
                ui.heading("Font Settings");
                ui.separator();
                
                egui::Grid::new("font_settings_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        // Font family
                        ui.label("Font Family:");
                        egui::ComboBox::from_id_salt("font_family")
                            .selected_text(self.settings.family.name())
                            .show_ui(ui, |ui| {
                                for family in FontFamily::all() {
                                    ui.selectable_value(
                                        &mut self.settings.family,
                                        family.clone(),
                                        family.name()
                                    );
                                }
                            });
                        ui.end_row();
                        
                        // Font size
                        ui.label("Font Size:");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut self.settings.size)
                                .range(6.0..=72.0)
                                .speed(0.5)
                                .suffix(" pt"));
                            if ui.button("-").clicked() && self.settings.size > 6.0 {
                                self.settings.size -= 1.0;
                            }
                            if ui.button("+").clicked() && self.settings.size < 72.0 {
                                self.settings.size += 1.0;
                            }
                        });
                        ui.end_row();
                        
                        // Line height
                        ui.label("Line Height:");
                        ui.add(egui::Slider::new(&mut self.settings.line_height, 0.8..=2.0)
                            .show_value(true));
                        ui.end_row();
                        
                        // Letter spacing
                        ui.label("Letter Spacing:");
                        ui.add(egui::Slider::new(&mut self.settings.letter_spacing, -0.2..=0.5)
                            .show_value(true)
                            .suffix(" em"));
                        ui.end_row();
                        
                        // Font weight
                        ui.label("Font Weight:");
                        egui::ComboBox::from_id_salt("font_weight")
                            .selected_text(self.settings.weight.name())
                            .show_ui(ui, |ui| {
                                for weight in FontWeight::all() {
                                    ui.selectable_value(
                                        &mut self.settings.weight,
                                        weight,
                                        weight.name()
                                    );
                                }
                            });
                        ui.end_row();
                        
                        // Bold weight
                        ui.label("Bold Weight:");
                        egui::ComboBox::from_id_salt("bold_weight")
                            .selected_text(self.settings.bold_weight.name())
                            .show_ui(ui, |ui| {
                                for weight in FontWeight::all() {
                                    ui.selectable_value(
                                        &mut self.settings.bold_weight,
                                        weight,
                                        weight.name()
                                    );
                                }
                            });
                        ui.end_row();
                    });
                
                ui.separator();
                ui.heading("Rendering Options");
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.settings.ligatures, "Ligatures");
                    ui.checkbox(&mut self.settings.antialiasing, "Anti-aliasing");
                    ui.checkbox(&mut self.settings.subpixel_rendering, "Subpixel");
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.settings.italic_enabled, "Italic");
                    ui.checkbox(&mut self.settings.underline_enabled, "Underline");
                });
                
                ui.separator();
                ui.heading("Cell Adjustments");
                
                egui::Grid::new("cell_adjust_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Cell Width:");
                        ui.add(egui::Slider::new(&mut self.settings.cell_width_adjust, 80.0..=120.0)
                            .show_value(true)
                            .suffix("%"));
                        ui.end_row();
                        
                        ui.label("Cell Height:");
                        ui.add(egui::Slider::new(&mut self.settings.cell_height_adjust, 80.0..=120.0)
                            .show_value(true)
                            .suffix("%"));
                        ui.end_row();
                        
                        ui.label("Cursor Blink:");
                        ui.add(egui::Slider::new(&mut self.settings.cursor_blink_ms, 0..=1000)
                            .show_value(true)
                            .suffix(" ms"));
                        ui.end_row();
                    });
                
                ui.separator();
                ui.heading("Preview");
                
                // Preview area
                egui::Frame::canvas(ui.style())
                    .fill(Color32::from_rgb(30, 30, 30))
                    .show(ui, |ui| {
                        ui.set_min_height(100.0);
                        ui.label(RichText::new(&self.preview_text)
                            .size(self.settings.size)
                            .color(Color32::from_rgb(200, 200, 200)));
                    });
                
                // Calculated values
                ui.label(format!(
                    "Cell size: {:.1} x {:.1} px",
                    self.settings.cell_width(),
                    self.settings.cell_height()
                ));
                
                ui.separator();
                
                // Buttons
                ui.horizontal(|ui| {
                    if ui.button("Reset to Defaults").clicked() {
                        self.settings.reset();
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.settings = self.original.clone();
                            self.open = false;
                        }
                        if ui.button("Apply").clicked() {
                            result = Some(self.settings.clone());
                            self.open = false;
                        }
                    });
                });
            });
        
        result
    }
}

/// Font scaling for accessibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontScale {
    Tiny,    // 75%
    Small,   // 87.5%
    Normal,  // 100%
    Large,   // 125%
    Larger,  // 150%
    Huge,    // 200%
}

impl FontScale {
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

impl Default for FontScale {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_font_settings_default() {
        let settings = FontSettings::default();
        assert_eq!(settings.size, 13.0);
        assert!(settings.ligatures);
        assert!(settings.antialiasing);
    }
    
    #[test]
    fn test_cell_dimensions() {
        let settings = FontSettings::default();
        let width = settings.cell_width();
        let height = settings.cell_height();
        
        assert!(width > 0.0);
        assert!(height > 0.0);
        assert!(height > width); // Cells are typically taller than wide
    }
    
    #[test]
    fn test_font_scale() {
        assert_eq!(FontScale::Normal.factor(), 1.0);
        assert!(FontScale::Large.factor() > FontScale::Normal.factor());
        assert!(FontScale::Small.factor() < FontScale::Normal.factor());
    }
}


