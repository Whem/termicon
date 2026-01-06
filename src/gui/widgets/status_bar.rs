//! Status bar widget

use crate::gui::theme::StatusLedColors;
use crate::i18n::t;
use crate::utils::{format_bytes, format_duration};
use egui::{Color32, RichText, Ui};

/// Status bar widget
pub struct StatusBar;

impl StatusBar {
    /// Create a new status bar
    pub fn new() -> Self {
        Self
    }

    /// Show the status bar
    pub fn show(&self, ui: &mut Ui, session: Option<&super::super::app::SessionData>) {
        ui.horizontal(|ui| {
            if let Some(session) = session {
                // Connection status
                let (status_text, status_color) = if session.session.is_some() {
                    (t("status.connected"), Color32::from_rgb(50, 205, 50))
                } else {
                    (t("status.disconnected"), Color32::from_rgb(150, 150, 150))
                };

                // Status indicator
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(10.0, 10.0),
                    egui::Sense::hover(),
                );
                ui.painter().circle_filled(rect.center(), 5.0, status_color);

                ui.label(status_text);

                ui.separator();

                // Connection info
                ui.label(&session.connection_info);

                ui.separator();

                // Stats (placeholder - would need real stats from session)
                ui.label(format!("{}: {}", t("status.bytes_received"), format_bytes(session.rx_buffer.len() as u64)));

                // Spacer
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Right-aligned items
                    ui.label(format!("Buffer: {}", format_bytes(session.rx_buffer.len() as u64)));
                });
            } else {
                ui.label(t("status.disconnected"));
            }
        });
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}






