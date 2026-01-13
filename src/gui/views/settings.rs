//! Settings view component

use crate::config::{AppConfig, LineEnding};
use crate::core::codec::CodecType;
use crate::core::logger::LogFormat;
use crate::i18n::{t, Locale};
use egui::{ComboBox, Grid, Slider, Ui};

/// Settings view tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Terminal,
    Logging,
    AutoConnect,
}

/// Settings view state
pub struct SettingsView {
    /// Active tab
    active_tab: SettingsTab,
}

impl SettingsView {
    /// Create a new settings view
    pub fn new() -> Self {
        Self {
            active_tab: SettingsTab::General,
        }
    }

    /// Show the settings view
    pub fn show(&mut self, ui: &mut Ui, config: &mut AppConfig) {
        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, SettingsTab::General, "General");
            ui.selectable_value(&mut self.active_tab, SettingsTab::Terminal, "Terminal");
            ui.selectable_value(&mut self.active_tab, SettingsTab::Logging, "Logging");
            ui.selectable_value(&mut self.active_tab, SettingsTab::AutoConnect, "Auto-Connect");
        });

        ui.separator();

        // Tab content
        match self.active_tab {
            SettingsTab::General => self.show_general(ui, config),
            SettingsTab::Terminal => self.show_terminal(ui, config),
            SettingsTab::Logging => self.show_logging(ui, config),
            SettingsTab::AutoConnect => self.show_autoconnect(ui, config),
        }
    }

    /// Show general settings
    fn show_general(&mut self, ui: &mut Ui, config: &mut AppConfig) {
        Grid::new("general_settings")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                // Language
                ui.label("Language");
                let current_locale = config.locale();
                ComboBox::from_id_salt("language")
                    .selected_text(current_locale.display_name())
                    .show_ui(ui, |ui| {
                        for locale in Locale::available() {
                            if ui
                                .selectable_label(*locale == current_locale, locale.display_name())
                                .clicked()
                            {
                                config.set_locale(*locale);
                            }
                        }
                    });
                ui.end_row();

                // Theme
                ui.label("Theme");
                ComboBox::from_id_salt("theme")
                    .selected_text(&config.window.theme)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.window.theme, "dark".to_string(), "Dark");
                        ui.selectable_value(&mut config.window.theme, "light".to_string(), "Light");
                        ui.selectable_value(
                            &mut config.window.theme,
                            "high_contrast".to_string(),
                            "High Contrast",
                        );
                    });
                ui.end_row();

                // Show toolbar
                ui.label("Show Toolbar");
                ui.checkbox(&mut config.window.show_toolbar, "");
                ui.end_row();

                // Show status bar
                ui.label("Show Status Bar");
                ui.checkbox(&mut config.window.show_status_bar, "");
                ui.end_row();
            });
    }

    /// Show terminal settings
    fn show_terminal(&mut self, ui: &mut Ui, config: &mut AppConfig) {
        Grid::new("terminal_settings")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                // Font family
                ui.label("Font Family");
                ui.text_edit_singleline(&mut config.terminal.font_family);
                ui.end_row();

                // Font size
                ui.label("Font Size");
                ui.add(Slider::new(&mut config.terminal.font_size, 8.0..=24.0));
                ui.end_row();

                // Local echo
                ui.label(t("terminal.local_echo"));
                ui.checkbox(&mut config.terminal.local_echo, "");
                ui.end_row();

                // Line ending
                ui.label(t("terminal.line_ending"));
                ComboBox::from_id_salt("line_ending")
                    .selected_text(match config.terminal.line_ending {
                        LineEnding::Cr => t("terminal.line_ending_cr"),
                        LineEnding::Lf => t("terminal.line_ending_lf"),
                        LineEnding::CrLf => t("terminal.line_ending_crlf"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut config.terminal.line_ending,
                            LineEnding::Cr,
                            t("terminal.line_ending_cr"),
                        );
                        ui.selectable_value(
                            &mut config.terminal.line_ending,
                            LineEnding::Lf,
                            t("terminal.line_ending_lf"),
                        );
                        ui.selectable_value(
                            &mut config.terminal.line_ending,
                            LineEnding::CrLf,
                            t("terminal.line_ending_crlf"),
                        );
                    });
                ui.end_row();

                // Display mode
                ui.label("Default View");
                ComboBox::from_id_salt("display_mode")
                    .selected_text(match config.terminal.display_mode {
                        CodecType::Text => "Text",
                        CodecType::Hex => "Hex",
                        CodecType::Mixed => "Mixed",
                        CodecType::Binary => "Binary",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut config.terminal.display_mode,
                            CodecType::Text,
                            "Text",
                        );
                        ui.selectable_value(
                            &mut config.terminal.display_mode,
                            CodecType::Hex,
                            "Hex",
                        );
                        ui.selectable_value(
                            &mut config.terminal.display_mode,
                            CodecType::Mixed,
                            "Mixed",
                        );
                    });
                ui.end_row();

                // Scroll buffer
                ui.label("Scroll Buffer (lines)");
                ui.add(Slider::new(&mut config.terminal.scroll_buffer, 1000..=100000));
                ui.end_row();

                // Scroll on output
                ui.label(t("terminal.scroll_on_output"));
                ui.checkbox(&mut config.terminal.scroll_on_output, "");
                ui.end_row();

                // Show timestamps
                ui.label(t("log.timestamps"));
                ui.checkbox(&mut config.terminal.show_timestamps, "");
                ui.end_row();
            });
    }

    /// Show logging settings
    fn show_logging(&mut self, ui: &mut Ui, config: &mut AppConfig) {
        Grid::new("logging_settings")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                // Enable logging
                ui.label(t("log.enabled"));
                ui.checkbox(&mut config.logging.enabled, "");
                ui.end_row();

                // Log format
                ui.label(t("log.format"));
                ComboBox::from_id_salt("log_format")
                    .selected_text(match config.logging.format {
                        LogFormat::Text => "Text",
                        LogFormat::Hex => "Hex",
                        LogFormat::Csv => "CSV",
                        LogFormat::Raw => "Raw",
                        LogFormat::Json => "JSON",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut config.logging.format, LogFormat::Text, "Text");
                        ui.selectable_value(&mut config.logging.format, LogFormat::Hex, "Hex");
                        ui.selectable_value(&mut config.logging.format, LogFormat::Csv, "CSV");
                        ui.selectable_value(&mut config.logging.format, LogFormat::Raw, "Raw");
                        ui.selectable_value(&mut config.logging.format, LogFormat::Json, "JSON");
                    });
                ui.end_row();

                // Timestamps
                ui.label(t("log.timestamps"));
                ui.checkbox(&mut config.logging.timestamps, "");
                ui.end_row();

                // Auto-rotate
                ui.label("Auto-rotate Logs");
                ui.checkbox(&mut config.logging.auto_rotate, "");
                ui.end_row();

                // Max size
                ui.label("Max Log Size (MB)");
                ui.add(Slider::new(&mut config.logging.max_size_mb, 1..=100));
                ui.end_row();
            });
    }

    /// Show auto-connect settings
    fn show_autoconnect(&mut self, ui: &mut Ui, config: &mut AppConfig) {
        Grid::new("autoconnect_settings")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                // Enable auto-reconnect
                ui.label(t("autoconnect.enabled"));
                ui.checkbox(&mut config.autoconnect.enabled, "");
                ui.end_row();

                // Delay
                ui.label(t("autoconnect.delay"));
                ui.add(Slider::new(&mut config.autoconnect.delay_secs, 1..=60));
                ui.end_row();

                // Max attempts
                ui.label(t("autoconnect.max_retries"));
                ui.add(Slider::new(&mut config.autoconnect.max_attempts, 0..=100));
                ui.end_row();

                // Notify
                ui.label("Show Notification");
                ui.checkbox(&mut config.autoconnect.notify, "");
                ui.end_row();
            });
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}








