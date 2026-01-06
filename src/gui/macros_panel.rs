//! Macros Panel - Quick macro buttons M1-M24 like classic terminal programs

use eframe::egui::{self, Color32, RichText, Ui};
use termicon_core::core::macros::{MacroManager, MacroSlot, MacroContent, parse_hex_string, format_hex_bytes};

/// Macros panel state
pub struct MacrosPanel {
    /// Macro manager
    pub manager: MacroManager,
    /// Show edit dialog
    show_edit_dialog: bool,
    /// Currently editing macro index (1-24)
    editing_index: usize,
    /// Edit dialog fields
    edit_name: String,
    edit_content: String,
    edit_description: String,
    edit_hex_mode: bool,
    edit_append_crlf: bool,
    /// Is panel expanded
    pub expanded: bool,
}

impl Default for MacrosPanel {
    fn default() -> Self {
        Self {
            manager: MacroManager::new(),
            show_edit_dialog: false,
            editing_index: 0,
            edit_name: String::new(),
            edit_content: String::new(),
            edit_description: String::new(),
            edit_hex_mode: false,
            edit_append_crlf: true,
            expanded: false,
        }
    }
}

/// Macro display data (to avoid borrow issues)
#[derive(Clone)]
struct MacroDisplayData {
    index: usize,
    name: String,
    is_empty: bool,
    description: String,
    content: MacroContent,
    usage_count: u64,
    append_crlf: bool,
}

impl MacrosPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get macro data for display
    fn get_display_data(&self, profile_id: Option<&str>) -> Vec<MacroDisplayData> {
        let macro_set = self.manager.get_set(profile_id);
        macro_set.macros.iter().enumerate().map(|(i, slot)| {
            MacroDisplayData {
                index: i + 1,
                name: slot.name.clone(),
                is_empty: slot.is_empty(),
                description: slot.description.clone(),
                content: slot.content.clone(),
                usage_count: slot.usage_count,
                append_crlf: slot.append_crlf,
            }
        }).collect()
    }

    /// Get bytes for a macro
    fn get_macro_bytes(&self, profile_id: Option<&str>, index: usize) -> Vec<u8> {
        let macro_set = self.manager.get_set(profile_id);
        if let Some(slot) = macro_set.get(index) {
            slot.get_bytes()
        } else {
            Vec::new()
        }
    }

    /// Render the macros panel (bottom bar with M1-M24 buttons)
    pub fn render(&mut self, ui: &mut Ui, profile_id: Option<&str>) -> Option<Vec<u8>> {
        let mut data_to_send: Option<Vec<u8>> = None;
        let mut edit_index: Option<usize> = None;

        // Get display data
        let macros_data = self.get_display_data(profile_id);

        // Compact mode - just show buttons
        ui.horizontal(|ui| {
            ui.label(RichText::new("Macros").size(10.0).color(Color32::GRAY));
            ui.separator();

            // First row: M1-M12
            for data in macros_data.iter().take(12) {
                let btn_text = if data.is_empty {
                    format!("M{}", data.index)
                } else {
                    data.name.chars().take(6).collect::<String>()
                };

                let btn = egui::Button::new(
                    RichText::new(&btn_text).size(10.0)
                ).min_size(egui::vec2(40.0, 20.0));

                let response = ui.add(btn);

                // Tooltip
                if !data.is_empty {
                    let desc = data.description.clone();
                    let content = data.content.clone();
                    let name = data.name.clone();
                    let usage = data.usage_count;
                    response.clone().on_hover_ui(|ui| {
                        ui.label(RichText::new(&name).strong());
                        if !desc.is_empty() {
                            ui.label(&desc);
                        }
                        match &content {
                            MacroContent::Text(s) => {
                                ui.label(RichText::new(s).monospace().size(10.0));
                            }
                            MacroContent::Hex(b) => {
                                ui.label(RichText::new(format_hex_bytes(b)).monospace().size(10.0));
                            }
                            _ => {}
                        }
                        if usage > 0 {
                            ui.label(RichText::new(format!("Used {} times", usage)).size(9.0).color(Color32::GRAY));
                        }
                    });
                }

                // Left click = execute
                if response.clicked() && !data.is_empty {
                    data_to_send = Some(self.get_macro_bytes(profile_id, data.index));
                    self.manager.record_use(profile_id, data.index);
                }

                // Right click = edit
                if response.secondary_clicked() {
                    edit_index = Some(data.index);
                }
            }
        });

        // Second row: M13-M24
        ui.horizontal(|ui| {
            ui.add_space(55.0); // Align with first row

            for data in macros_data.iter().skip(12) {
                let btn_text = if data.is_empty {
                    format!("M{}", data.index)
                } else {
                    data.name.chars().take(6).collect::<String>()
                };

                let btn = egui::Button::new(
                    RichText::new(&btn_text).size(10.0)
                ).min_size(egui::vec2(40.0, 20.0));

                let response = ui.add(btn);

                // Tooltip
                if !data.is_empty {
                    let desc = data.description.clone();
                    let content = data.content.clone();
                    let name = data.name.clone();
                    response.clone().on_hover_ui(|ui| {
                        ui.label(RichText::new(&name).strong());
                        if !desc.is_empty() {
                            ui.label(&desc);
                        }
                        match &content {
                            MacroContent::Text(s) => {
                                ui.label(RichText::new(s).monospace().size(10.0));
                            }
                            MacroContent::Hex(b) => {
                                ui.label(RichText::new(format_hex_bytes(b)).monospace().size(10.0));
                            }
                            _ => {}
                        }
                    });
                }

                // Left click = execute
                if response.clicked() && !data.is_empty {
                    data_to_send = Some(self.get_macro_bytes(profile_id, data.index));
                    self.manager.record_use(profile_id, data.index);
                }

                // Right click = edit
                if response.secondary_clicked() {
                    edit_index = Some(data.index);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("⚙").on_hover_text("Set Macros").clicked() {
                    self.expanded = !self.expanded;
                }
            });
        });

        // Handle edit
        if let Some(idx) = edit_index {
            if let Some(data) = macros_data.iter().find(|d| d.index == idx) {
                self.start_edit_from_data(data);
            }
        }

        // Edit dialog
        if self.show_edit_dialog {
            self.render_edit_dialog(ui, profile_id);
        }

        data_to_send
    }

    fn start_edit_from_data(&mut self, data: &MacroDisplayData) {
        self.editing_index = data.index;
        self.edit_name = data.name.clone();
        self.edit_description = data.description.clone();
        self.edit_append_crlf = data.append_crlf;
        
        match &data.content {
            MacroContent::Text(s) => {
                self.edit_content = s.clone();
                self.edit_hex_mode = false;
            }
            MacroContent::Hex(b) => {
                self.edit_content = format_hex_bytes(b);
                self.edit_hex_mode = true;
            }
            _ => {
                self.edit_content = String::new();
                self.edit_hex_mode = false;
            }
        }
        
        self.show_edit_dialog = true;
    }

    fn render_edit_dialog(&mut self, ui: &mut Ui, profile_id: Option<&str>) {
        egui::Window::new(format!("Edit Macro M{}", self.editing_index))
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.edit_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Description:");
                    ui.text_edit_singleline(&mut self.edit_description);
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.edit_hex_mode, "Hex mode");
                    ui.checkbox(&mut self.edit_append_crlf, "Append CR+LF");
                });

                ui.label(if self.edit_hex_mode { "Hex data:" } else { "Command:" });
                ui.add(
                    egui::TextEdit::multiline(&mut self.edit_content)
                        .desired_width(300.0)
                        .desired_rows(3)
                        .font(egui::TextStyle::Monospace)
                );

                if self.edit_hex_mode {
                    ui.label(RichText::new("Format: FF 00 A5 or FF00A5").size(10.0).color(Color32::GRAY));
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_macro(profile_id);
                        self.show_edit_dialog = false;
                    }
                    if ui.button("Clear").clicked() {
                        self.clear_macro(profile_id);
                        self.show_edit_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_edit_dialog = false;
                    }
                });
            });
    }

    fn save_macro(&mut self, profile_id: Option<&str>) {
        let content = if self.edit_hex_mode {
            match parse_hex_string(&self.edit_content) {
                Ok(bytes) => MacroContent::Hex(bytes),
                Err(_) => MacroContent::Text(self.edit_content.clone()),
            }
        } else {
            MacroContent::Text(self.edit_content.clone())
        };

        let set = self.manager.get_set_mut(profile_id);
        set.set_macro(self.editing_index, &self.edit_name, content, &self.edit_description);
        
        if let Some(slot) = set.get_mut(self.editing_index) {
            slot.append_crlf = self.edit_append_crlf;
        }

        self.manager.save();
    }

    fn clear_macro(&mut self, profile_id: Option<&str>) {
        let set = self.manager.get_set_mut(profile_id);
        set.set_macro(self.editing_index, &format!("M{}", self.editing_index), MacroContent::Empty, "");
        self.manager.save();
    }

    /// Get bytes to send for a macro (by function key)
    pub fn get_macro_for_key(&self, key: egui::Key, profile_id: Option<&str>) -> Option<Vec<u8>> {
        let index = match key {
            egui::Key::F1 => 1,
            egui::Key::F2 => 2,
            egui::Key::F3 => 3,
            egui::Key::F4 => 4,
            egui::Key::F5 => 5,
            egui::Key::F6 => 6,
            egui::Key::F7 => 7,
            egui::Key::F8 => 8,
            egui::Key::F9 => 9,
            egui::Key::F10 => 10,
            egui::Key::F11 => 11,
            egui::Key::F12 => 12,
            _ => return None,
        };

        let set = self.manager.get_set(profile_id);
        set.get(index).map(|slot: &MacroSlot| slot.get_bytes())
    }
}
