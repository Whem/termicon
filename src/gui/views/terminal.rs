//! Terminal view component

use crate::gui::theme::TerminalColors;
use egui::{Color32, FontId, RichText, ScrollArea, TextStyle, Ui};

/// Terminal view state
pub struct TerminalView {
    /// Terminal colors
    colors: TerminalColors,
    /// Font size
    font_size: f32,
    /// Show line numbers
    show_line_numbers: bool,
    /// Scroll position
    scroll_to_bottom: bool,
}

impl TerminalView {
    /// Create a new terminal view
    pub fn new() -> Self {
        Self {
            colors: TerminalColors::dark(),
            font_size: 13.0,
            show_line_numbers: false,
            scroll_to_bottom: true,
        }
    }

    /// Show the terminal view
    pub fn show(&mut self, ui: &mut Ui, data: &[u8], ui_state: &super::super::app::UiState) {
        let font_id = FontId::new(self.font_size, egui::FontFamily::Monospace);

        // Terminal background
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, self.colors.background);

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(ui_state.auto_scroll)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                if data.is_empty() {
                    ui.label(
                        RichText::new("Waiting for data...")
                            .color(self.colors.timestamp_color)
                            .font(font_id.clone()),
                    );
                    return;
                }

                // Convert data to string for display
                let text = String::from_utf8_lossy(data);

                // Split into lines and display
                let lines: Vec<&str> = text.lines().collect();

                for (i, line) in lines.iter().enumerate() {
                    ui.horizontal(|ui| {
                        // Line number
                        if self.show_line_numbers {
                            ui.label(
                                RichText::new(format!("{:5} ", i + 1))
                                    .color(self.colors.timestamp_color)
                                    .font(font_id.clone()),
                            );
                        }

                        // Timestamp
                        if ui_state.show_timestamps {
                            let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
                            ui.label(
                                RichText::new(format!("[{}] ", timestamp))
                                    .color(self.colors.timestamp_color)
                                    .font(font_id.clone()),
                            );
                        }

                        // Line content
                        ui.label(
                            RichText::new(*line)
                                .color(self.colors.rx_color)
                                .font(font_id.clone()),
                        );
                    });
                }

                // Scroll to bottom if needed
                if self.scroll_to_bottom {
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                    self.scroll_to_bottom = false;
                }
            });
    }

    /// Show hex view
    pub fn show_hex(&mut self, ui: &mut Ui, data: &[u8]) {
        let font_id = FontId::new(self.font_size, egui::FontFamily::Monospace);

        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let bytes_per_line = 16;

                for (offset, chunk) in data.chunks(bytes_per_line).enumerate() {
                    ui.horizontal(|ui| {
                        // Offset
                        ui.label(
                            RichText::new(format!("{:08X}  ", offset * bytes_per_line))
                                .color(self.colors.timestamp_color)
                                .font(font_id.clone()),
                        );

                        // Hex bytes
                        let mut hex_str = String::new();
                        for (i, byte) in chunk.iter().enumerate() {
                            hex_str.push_str(&format!("{:02X} ", byte));
                            if i == 7 {
                                hex_str.push(' ');
                            }
                        }
                        // Padding
                        if chunk.len() < bytes_per_line {
                            let missing = bytes_per_line - chunk.len();
                            for i in 0..missing {
                                hex_str.push_str("   ");
                                if chunk.len() + i == 7 {
                                    hex_str.push(' ');
                                }
                            }
                        }

                        ui.label(
                            RichText::new(hex_str)
                                .color(self.colors.foreground)
                                .font(font_id.clone()),
                        );

                        // ASCII
                        ui.label(
                            RichText::new(" |")
                                .color(self.colors.timestamp_color)
                                .font(font_id.clone()),
                        );

                        let ascii: String = chunk
                            .iter()
                            .map(|&b| {
                                if b.is_ascii_graphic() || b == b' ' {
                                    b as char
                                } else {
                                    '.'
                                }
                            })
                            .collect();

                        ui.label(
                            RichText::new(ascii)
                                .color(self.colors.foreground)
                                .font(font_id.clone()),
                        );

                        ui.label(
                            RichText::new("|")
                                .color(self.colors.timestamp_color)
                                .font(font_id.clone()),
                        );
                    });
                }
            });
    }

    /// Set theme
    pub fn set_dark_theme(&mut self, dark: bool) {
        self.colors = if dark {
            TerminalColors::dark()
        } else {
            TerminalColors::light()
        };
    }

    /// Set font size
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    /// Toggle line numbers
    pub fn toggle_line_numbers(&mut self) {
        self.show_line_numbers = !self.show_line_numbers;
    }

    /// Trigger scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_to_bottom = true;
    }
}

impl Default for TerminalView {
    fn default() -> Self {
        Self::new()
    }
}








