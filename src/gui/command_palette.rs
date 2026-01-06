//! Command Palette
//!
//! Quick command execution via fuzzy search (Ctrl+K / Ctrl+P style)

use eframe::egui::{self, Color32, Key, Modifiers, RichText, Vec2};
use std::collections::HashMap;

/// Command category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Connection,
    View,
    Edit,
    Tools,
    Help,
    Navigation,
    Settings,
}

impl CommandCategory {
    fn label(&self) -> &'static str {
        match self {
            Self::Connection => "Connection",
            Self::View => "View",
            Self::Edit => "Edit",
            Self::Tools => "Tools",
            Self::Help => "Help",
            Self::Navigation => "Navigation",
            Self::Settings => "Settings",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Connection => "[C]",
            Self::View => "[V]",
            Self::Edit => "[E]",
            Self::Tools => "[T]",
            Self::Help => "[?]",
            Self::Navigation => "[N]",
            Self::Settings => "[S]",
        }
    }
}

/// A command that can be executed
#[derive(Debug, Clone)]
pub struct Command {
    /// Unique command ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Category
    pub category: CommandCategory,
    /// Search keywords
    pub keywords: Vec<String>,
    /// Is enabled
    pub enabled: bool,
}

impl Command {
    pub fn new(id: &str, name: &str, category: CommandCategory) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            shortcut: None,
            category,
            keywords: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Calculate match score for fuzzy search
    pub fn match_score(&self, query: &str) -> Option<i32> {
        if query.is_empty() {
            return Some(0);
        }

        let query_lower = query.to_lowercase();
        let name_lower = self.name.to_lowercase();

        // Exact match
        if name_lower == query_lower {
            return Some(1000);
        }

        // Starts with
        if name_lower.starts_with(&query_lower) {
            return Some(500 + (100 - query.len() as i32).max(0));
        }

        // Contains
        if name_lower.contains(&query_lower) {
            return Some(200 + (100 - query.len() as i32).max(0));
        }

        // Check keywords
        for keyword in &self.keywords {
            if keyword.to_lowercase().contains(&query_lower) {
                return Some(100);
            }
        }

        // Check description
        if let Some(ref desc) = self.description {
            if desc.to_lowercase().contains(&query_lower) {
                return Some(50);
            }
        }

        // Fuzzy matching (simple)
        let mut score = 0i32;
        let mut name_chars = name_lower.chars().peekable();
        for qc in query_lower.chars() {
            while let Some(&nc) = name_chars.peek() {
                name_chars.next();
                if nc == qc {
                    score += 10;
                    break;
                }
            }
        }

        if score > 0 { Some(score) } else { None }
    }
}

/// Command action result
#[derive(Debug, Clone)]
pub enum CommandAction {
    /// No action
    None,
    /// Navigate to a view
    NavigateTo(String),
    /// Open dialog
    OpenDialog(String),
    /// Toggle setting
    ToggleSetting(String),
    /// Execute function
    Execute(String),
    /// Open URL
    OpenUrl(String),
    /// Show submenu
    ShowSubmenu(Vec<Command>),
}

/// Command palette state
pub struct CommandPalette {
    /// Is open
    pub is_open: bool,
    /// Search query
    pub query: String,
    /// Selected index
    pub selected_index: usize,
    /// All commands
    commands: Vec<Command>,
    /// Filtered commands
    filtered: Vec<usize>,
    /// Recent commands (IDs)
    recent: Vec<String>,
    /// Max recent count
    max_recent: usize,
    /// Pending action
    pub pending_action: Option<CommandAction>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut palette = Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            commands: Vec::new(),
            filtered: Vec::new(),
            recent: Vec::new(),
            max_recent: 10,
            pending_action: None,
        };

        palette.register_default_commands();
        palette.update_filter();
        palette
    }

    /// Register default commands
    fn register_default_commands(&mut self) {
        // Connection commands
        self.add_command(Command::new("conn.serial", "Connect Serial Port", CommandCategory::Connection)
            .with_shortcut("Ctrl+Shift+S")
            .with_description("Open serial port connection dialog")
            .with_keywords(&["uart", "com", "rs232", "rs485"]));

        self.add_command(Command::new("conn.tcp", "Connect TCP", CommandCategory::Connection)
            .with_description("Open TCP connection dialog")
            .with_keywords(&["socket", "network"]));

        self.add_command(Command::new("conn.telnet", "Connect Telnet", CommandCategory::Connection)
            .with_description("Open Telnet connection dialog"));

        self.add_command(Command::new("conn.ssh", "Connect SSH", CommandCategory::Connection)
            .with_shortcut("Ctrl+Shift+H")
            .with_description("Open SSH connection dialog")
            .with_keywords(&["secure", "shell", "remote"]));

        self.add_command(Command::new("conn.ble", "Connect Bluetooth", CommandCategory::Connection)
            .with_description("Open BLE connection dialog")
            .with_keywords(&["bluetooth", "wireless", "ble", "gatt"]));

        self.add_command(Command::new("conn.disconnect", "Disconnect", CommandCategory::Connection)
            .with_shortcut("Ctrl+D")
            .with_description("Disconnect current session"));

        // View commands
        self.add_command(Command::new("view.hex", "Toggle Hex View", CommandCategory::View)
            .with_shortcut("Ctrl+H")
            .with_description("Toggle hex display mode"));

        self.add_command(Command::new("view.timestamps", "Toggle Timestamps", CommandCategory::View)
            .with_description("Show/hide timestamps"));

        self.add_command(Command::new("view.chart", "Open Chart Panel", CommandCategory::View)
            .with_description("Open real-time chart view"));

        self.add_command(Command::new("view.sftp", "Open SFTP Browser", CommandCategory::View)
            .with_description("Open SFTP file browser"));

        self.add_command(Command::new("view.ble", "Open BLE Inspector", CommandCategory::View)
            .with_description("Open Bluetooth LE inspector"));

        self.add_command(Command::new("view.clear", "Clear Output", CommandCategory::View)
            .with_shortcut("Ctrl+L")
            .with_description("Clear terminal output"));

        // Edit commands
        self.add_command(Command::new("edit.copy", "Copy", CommandCategory::Edit)
            .with_shortcut("Ctrl+Shift+C")
            .with_description("Copy selected text"));

        self.add_command(Command::new("edit.paste", "Paste", CommandCategory::Edit)
            .with_shortcut("Ctrl+Shift+V")
            .with_description("Paste from clipboard"));

        self.add_command(Command::new("edit.selectall", "Select All", CommandCategory::Edit)
            .with_shortcut("Ctrl+A")
            .with_description("Select all output"));

        self.add_command(Command::new("edit.find", "Find in Output", CommandCategory::Edit)
            .with_shortcut("Ctrl+F")
            .with_description("Search in terminal output"));

        // Tools commands
        self.add_command(Command::new("tools.snippets", "Manage Snippets", CommandCategory::Tools)
            .with_description("Open snippet manager")
            .with_keywords(&["macro", "command", "quick"]));

        self.add_command(Command::new("tools.profiles", "Manage Profiles", CommandCategory::Tools)
            .with_description("Open profile manager")
            .with_keywords(&["save", "load", "connection"]));

        self.add_command(Command::new("tools.triggers", "Manage Triggers", CommandCategory::Tools)
            .with_description("Open trigger manager")
            .with_keywords(&["auto", "response", "pattern"]));

        self.add_command(Command::new("tools.record", "Start Macro Recording", CommandCategory::Tools)
            .with_description("Start recording commands")
            .with_keywords(&["macro", "automation"]));

        self.add_command(Command::new("tools.bridge", "Start Bridge", CommandCategory::Tools)
            .with_description("Start Serial↔TCP bridge")
            .with_keywords(&["forward", "proxy"]));

        // Navigation
        self.add_command(Command::new("nav.newtab", "New Tab", CommandCategory::Navigation)
            .with_shortcut("Ctrl+T")
            .with_description("Open a new tab"));

        self.add_command(Command::new("nav.closetab", "Close Tab", CommandCategory::Navigation)
            .with_shortcut("Ctrl+W")
            .with_description("Close current tab"));

        self.add_command(Command::new("nav.nexttab", "Next Tab", CommandCategory::Navigation)
            .with_shortcut("Ctrl+Tab")
            .with_description("Switch to next tab"));

        self.add_command(Command::new("nav.prevtab", "Previous Tab", CommandCategory::Navigation)
            .with_shortcut("Ctrl+Shift+Tab")
            .with_description("Switch to previous tab"));

        // Settings
        self.add_command(Command::new("settings.theme.dark", "Dark Theme", CommandCategory::Settings)
            .with_description("Switch to dark theme"));

        self.add_command(Command::new("settings.theme.light", "Light Theme", CommandCategory::Settings)
            .with_description("Switch to light theme"));

        self.add_command(Command::new("settings.localecho", "Toggle Local Echo", CommandCategory::Settings)
            .with_description("Enable/disable local echo"));

        self.add_command(Command::new("settings.preferences", "Open Preferences", CommandCategory::Settings)
            .with_shortcut("Ctrl+,")
            .with_description("Open settings dialog"));

        // Help
        self.add_command(Command::new("help.about", "About Termicon", CommandCategory::Help)
            .with_description("Show about dialog"));

        self.add_command(Command::new("help.shortcuts", "Keyboard Shortcuts", CommandCategory::Help)
            .with_description("Show keyboard shortcut list"));

        self.add_command(Command::new("help.docs", "Documentation", CommandCategory::Help)
            .with_description("Open documentation"));
    }

    /// Add a command
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    /// Open the palette
    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
        self.update_filter();
    }

    /// Close the palette
    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
    }

    /// Update filtered list based on query
    pub fn update_filter(&mut self) {
        let mut scored: Vec<(usize, i32)> = self.commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| cmd.enabled)
            .filter_map(|(i, cmd)| {
                cmd.match_score(&self.query).map(|score| {
                    // Boost recent commands
                    let recent_boost = if self.recent.contains(&cmd.id) { 50 } else { 0 };
                    (i, score + recent_boost)
                })
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        
        self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        self.selected_index = 0;
    }

    /// Execute selected command
    pub fn execute_selected(&mut self) -> Option<String> {
        if let Some(&idx) = self.filtered.get(self.selected_index) {
            let cmd_id = self.commands[idx].id.clone();
            
            // Add to recent
            self.recent.retain(|id| id != &cmd_id);
            self.recent.insert(0, cmd_id.clone());
            if self.recent.len() > self.max_recent {
                self.recent.pop();
            }

            self.close();
            return Some(cmd_id);
        }
        None
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        if self.selected_index + 1 < self.filtered.len() {
            self.selected_index += 1;
        }
    }

    /// Handle keyboard input
    pub fn handle_input(&mut self, ctx: &egui::Context) -> Option<String> {
        // Check for Ctrl+K or Ctrl+P to open
        if ctx.input(|i| i.key_pressed(Key::K) && i.modifiers.ctrl) ||
           ctx.input(|i| i.key_pressed(Key::P) && i.modifiers.ctrl) {
            if !self.is_open {
                self.open();
            }
            return None;
        }

        if !self.is_open {
            return None;
        }

        // Escape to close
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.close();
            return None;
        }

        // Navigation
        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            self.select_up();
        }
        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            self.select_down();
        }

        // Execute on Enter
        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            return self.execute_selected();
        }

        None
    }

    /// Render the palette
    pub fn render(&mut self, ctx: &egui::Context) -> Option<String> {
        if !self.is_open {
            return None;
        }

        let mut result = None;

        // Center modal
        egui::Area::new(egui::Id::new("command_palette"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(Color32::from_rgb(30, 30, 35))
                    .shadow(egui::epaint::Shadow {
                        spread: 8,
                        blur: 16,
                        color: Color32::from_black_alpha(100),
                        offset: [0, 4],
                    })
                    .corner_radius(8.0)
                    .show(ui, |ui| {
                        ui.set_width(500.0);
                        
                        // Search input
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(">").size(18.0));
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.query)
                                    .hint_text("Type a command...")
                                    .frame(false)
                                    .font(egui::FontId::proportional(16.0))
                                    .desired_width(f32::INFINITY)
                            );
                            
                            if response.changed() {
                                self.update_filter();
                            }

                            // Auto-focus
                            response.request_focus();
                        });

                        ui.separator();

                        // Results
                        let filtered_snapshot: Vec<(usize, usize)> = self.filtered.iter()
                            .enumerate()
                            .take(15)
                            .map(|(i, &idx)| (i, idx))
                            .collect();
                        
                        let mut clicked_item: Option<usize> = None;
                        let mut hovered_item: Option<usize> = None;

                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                for (i, cmd_idx) in &filtered_snapshot {
                                    let cmd = &self.commands[*cmd_idx];
                                    let is_selected = *i == self.selected_index;

                                    let bg_color = if is_selected {
                                        Color32::from_rgb(60, 60, 80)
                                    } else {
                                        Color32::TRANSPARENT
                                    };

                                    let frame = egui::Frame::new()
                                        .fill(bg_color)
                                        .inner_margin(egui::Margin::symmetric(8, 4))
                                        .corner_radius(4.0);

                                    frame.show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // Category icon
                                            ui.label(RichText::new(cmd.category.icon()).size(14.0));

                                            // Command name
                                            ui.label(RichText::new(&cmd.name).strong());

                                            // Spacer
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                // Shortcut
                                                if let Some(ref shortcut) = cmd.shortcut {
                                                    ui.label(
                                                        RichText::new(shortcut)
                                                            .small()
                                                            .color(Color32::GRAY)
                                                    );
                                                }
                                            });
                                        });

                                        // Description
                                        if let Some(ref desc) = cmd.description {
                                            ui.label(
                                                RichText::new(desc)
                                                    .small()
                                                    .color(Color32::DARK_GRAY)
                                            );
                                        }
                                    });

                                    // Click to execute
                                    let resp = ui.interact(
                                        ui.min_rect(),
                                        ui.id().with(*cmd_idx),
                                        egui::Sense::click()
                                    );

                                    if resp.clicked() {
                                        clicked_item = Some(*i);
                                    }

                                    if resp.hovered() && !is_selected {
                                        hovered_item = Some(*i);
                                    }
                                }

                                if filtered_snapshot.is_empty() {
                                    ui.label(
                                        RichText::new("No commands found")
                                            .color(Color32::GRAY)
                                            .italics()
                                    );
                                }
                            });
                        
                        // Handle click/hover after the borrow ends
                        if let Some(i) = clicked_item {
                            self.selected_index = i;
                            result = self.execute_selected();
                        } else if let Some(i) = hovered_item {
                            self.selected_index = i;
                        }

                        // Footer
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("↑↓ Navigate").small().color(Color32::DARK_GRAY));
                            ui.label(RichText::new("↵ Execute").small().color(Color32::DARK_GRAY));
                            ui.label(RichText::new("Esc Close").small().color(Color32::DARK_GRAY));
                        });
                    });
            });

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_matching() {
        let cmd = Command::new("test", "Connect Serial Port", CommandCategory::Connection)
            .with_keywords(&["uart", "com"]);

        assert!(cmd.match_score("serial").is_some());
        assert!(cmd.match_score("uart").is_some());
        assert!(cmd.match_score("xyz").is_none());
        
        // Fuzzy match
        assert!(cmd.match_score("csp").is_some());
    }

    #[test]
    fn test_palette_filter() {
        let mut palette = CommandPalette::new();
        
        palette.query = "ssh".to_string();
        palette.update_filter();
        
        assert!(!palette.filtered.is_empty());
    }
}

