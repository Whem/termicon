//! Status LED widget for modem control lines

use crate::core::transport::ModemLines;
use crate::gui::theme::StatusLedColors;
use egui::{Color32, Ui, Vec2};

/// Status LED widget
pub struct StatusLeds {
    /// LED size
    size: f32,
    /// Spacing between LEDs
    spacing: f32,
}

impl StatusLeds {
    /// Create a new status LEDs widget
    pub fn new() -> Self {
        Self {
            size: 8.0,
            spacing: 4.0,
        }
    }

    /// Set LED size
    #[must_use]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Show the LEDs
    pub fn show(&self, ui: &mut Ui, lines: &ModemLines, tx_active: bool, rx_active: bool) {
        ui.horizontal(|ui| {
            // TX LED
            self.draw_led(ui, "TX", tx_active, StatusLedColors::green());

            // RX LED
            self.draw_led(ui, "RX", rx_active, StatusLedColors::green());

            ui.add_space(self.spacing * 2.0);

            // Control lines
            self.draw_led(ui, "RTS", lines.rts, StatusLedColors::yellow());
            self.draw_led(ui, "CTS", lines.cts, StatusLedColors::yellow());
            self.draw_led(ui, "DTR", lines.dtr, StatusLedColors::yellow());
            self.draw_led(ui, "DSR", lines.dsr, StatusLedColors::blue());
            self.draw_led(ui, "DCD", lines.dcd, StatusLedColors::red());
            self.draw_led(ui, "RI", lines.ri, StatusLedColors::blue());
        });
    }

    /// Draw a single LED
    fn draw_led(&self, ui: &mut Ui, label: &str, state: bool, colors: StatusLedColors) {
        ui.vertical(|ui| {
            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(self.size, self.size),
                egui::Sense::hover(),
            );

            let color = if state { colors.on } else { colors.off };

            // Draw LED circle
            ui.painter().circle_filled(rect.center(), self.size / 2.0, color);

            // Draw highlight for "on" state
            if state {
                ui.painter().circle_filled(
                    rect.center() - Vec2::new(self.size * 0.2, self.size * 0.2),
                    self.size / 6.0,
                    Color32::from_rgba_premultiplied(255, 255, 255, 100),
                );
            }

            // Label
            ui.label(
                egui::RichText::new(label)
                    .size(9.0)
                    .color(Color32::from_rgb(150, 150, 150)),
            );
        });

        ui.add_space(self.spacing);
    }
}

impl Default for StatusLeds {
    fn default() -> Self {
        Self::new()
    }
}






