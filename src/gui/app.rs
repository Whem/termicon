//! Main GUI application with tab support for multiple connections

use eframe::egui::{self, Color32, CornerRadius, FontId, RichText, Vec2, Margin, Stroke};
use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use rust_i18n::t;

use super::ansi_parser::parse_ansi;
use super::profiles::{Profile, ProfileManager, ProfileType, ProfileSnippet, SerialProfileSettings, TcpProfileSettings, SshProfileSettings, BluetoothProfileSettings};
use super::session_tab::{SessionTab, TabManager};
use termicon_core::i18n::{set_locale, Locale};

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Serial,
    Tcp,
    Telnet,
    Ssh,
    Bluetooth,
}

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// Dialog type
#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    None,
    Serial,
    Tcp,
    Telnet,
    Ssh,
    Bluetooth,
    About,
    Settings,
    SaveProfile,
    KeyboardShortcuts,
    UserGuide,
    DeviceSimulator,
    SessionReplay,
    Fuzzing,
    ExperimentMode,
    DeterministicMode,
    ExplainDebug,
    ModbusMonitor,
    Bridge,
    VirtualCom,
    SshKeyGen,
    FileTransferSend(String),
    FileTransferReceive(String),
    ProtocolDsl,
    Triggers,
}

/// Messages from connection thread to GUI
#[derive(Debug, Clone)]
pub enum ConnectionMessage {
    Connected,
    Disconnected,
    Error(String),
    Data(Vec<u8>),
}

/// Commands from GUI to connection thread
#[derive(Debug, Clone)]
pub enum ConnectionCommand {
    Send(Vec<u8>),
    Disconnect,
}

/// Serial connection settings
#[derive(Debug, Clone)]
pub struct SerialSettings {
    pub port: String,
    pub baud_rate: String,
    pub data_bits: String,
    pub parity: String,
    pub stop_bits: String,
    pub flow_control: String,
}

impl Default for SerialSettings {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: "115200".to_string(),
            data_bits: "8".to_string(),
            parity: "None".to_string(),
            stop_bits: "1".to_string(),
            flow_control: "None".to_string(),
        }
    }
}

/// TCP connection settings
#[derive(Debug, Clone)]
pub struct TcpSettings {
    pub host: String,
    pub port: String,
}

impl Default for TcpSettings {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: "23".to_string(),
        }
    }
}

/// SSH connection settings
#[derive(Debug, Clone)]
pub struct SshSettings {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub use_key: bool,
    pub key_path: String,
    pub key_passphrase: String,
    // Advanced settings
    pub show_advanced: bool,
    pub jump_host: String,
    pub jump_port: String,
    pub jump_username: String,
    pub jump_password: String,
    pub compression: bool,
    pub keepalive_interval: String,
    pub connection_timeout: String,
    pub save_password: bool,
    pub auto_connect: bool,
    pub terminal_type: String,
    pub x11_forwarding: bool,
    pub agent_forwarding: bool,
    pub local_port_forward: String,
    pub remote_port_forward: String,
}

impl Default for SshSettings {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: "22".to_string(),
            username: String::new(),
            password: String::new(),
            use_key: false,
            key_path: String::new(),
            key_passphrase: String::new(),
            show_advanced: false,
            jump_host: String::new(),
            jump_port: "22".to_string(),
            jump_username: String::new(),
            jump_password: String::new(),
            compression: false,
            keepalive_interval: "30".to_string(),
            connection_timeout: "10".to_string(),
            save_password: false,
            auto_connect: false,
            terminal_type: "xterm-256color".to_string(),
            x11_forwarding: false,
            agent_forwarding: false,
            local_port_forward: String::new(),
            remote_port_forward: String::new(),
        }
    }
}

/// Bluetooth connection settings
#[derive(Debug, Clone)]
pub struct BluetoothSettings {
    pub device: String,
    pub service_uuid: String,
    pub tx_uuid: String,
    pub rx_uuid: String,
}

impl Default for BluetoothSettings {
    fn default() -> Self {
        Self {
            device: String::new(),
            // Nordic UART Service (NUS) defaults
            service_uuid: "6e400001-b5a3-f393-e0a9-e50e24dcca9e".to_string(),
            tx_uuid: "6e400002-b5a3-f393-e0a9-e50e24dcca9e".to_string(),
            rx_uuid: "6e400003-b5a3-f393-e0a9-e50e24dcca9e".to_string(),
        }
    }
}

/// View mode for terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Text,
    Hex,
    Mixed,
}

/// Theme type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    Dark,
    Light,
}

/// Snippet entry
#[derive(Debug, Clone)]
pub struct Snippet {
    pub name: String,
    pub content: String,
    pub description: String,
}

/// Trigger entry
#[derive(Debug, Clone)]
pub struct TriggerEntry {
    pub name: String,
    pub pattern: String,
    pub response: String,
    pub enabled: bool,
}

impl Default for Snippet {
    fn default() -> Self {
        Self {
            name: String::new(),
            content: String::new(),
            description: String::new(),
        }
    }
}

/// Side panel mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidePanelMode {
    Profiles,
    Snippets,
    History,
    Settings,
    Chart,
    Sftp,
}

/// Language
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Hungarian,
}

/// Main application state
pub struct TermiconApp {
    /// Tab manager
    tabs: TabManager,
    /// Current dialog
    current_dialog: DialogType,
    /// Serial settings (for dialog)
    serial_settings: SerialSettings,
    /// TCP settings (for dialog)
    tcp_settings: TcpSettings,
    /// SSH settings (for dialog)
    ssh_settings: SshSettings,
    /// Bluetooth settings (for dialog)
    bluetooth_settings: BluetoothSettings,
    /// Status message
    status_message: String,
    /// Available serial ports
    available_ports: Vec<String>,
    /// View mode
    view_mode: ViewMode,
    /// Show side panel
    show_side_panel: bool,
    /// Current theme
    theme: AppTheme,
    /// Side panel mode
    side_panel_mode: SidePanelMode,
    /// Snippets list (global, for non-profile use)
    snippets: Vec<Snippet>,
    /// New snippet being edited
    new_snippet: Snippet,
    /// Current language
    language: Language,
    /// Show add snippet dialog
    show_add_snippet: bool,
    /// Chart data points for demo
    chart_data: Vec<f64>,
    /// Profile manager
    profile_manager: ProfileManager,
    /// New profile name (for save dialog)
    new_profile_name: String,
    /// Current active profile ID (for snippets)
    active_profile_id: Option<String>,
    /// Pending connection type for save profile dialog
    pending_profile_type: Option<ProfileType>,
    /// Ask to save profile after connect
    prompt_save_profile: bool,
    /// Macros panel
    macros_panel: super::macros_panel::MacrosPanel,
    /// Show macros bar
    show_macros_bar: bool,
    /// Pending command to insert into input field
    pending_insert_command: Option<String>,
    /// Language just changed flag (for UI refresh)
    language_changed: bool,
    /// Virtual COM pipe name (Windows)
    virtual_com_pipe_name: String,
    /// SSH keygen settings
    ssh_keygen_type: String,
    ssh_keygen_comment: String,
    ssh_keygen_passphrase: String,
    ssh_keygen_path: String,
    /// File transfer settings
    file_transfer_path: String,
    file_transfer_save_path: String,
    file_transfer_progress: f32,
    file_transfer_status: String,
    /// Protocol DSL content
    protocol_dsl_content: String,
    /// Triggers
    triggers: Vec<TriggerEntry>,
    new_trigger_name: String,
    new_trigger_pattern: String,
    new_trigger_response: String,
    /// SFTP remote path
    sftp_remote_path: String,
}

impl Default for TermiconApp {
    fn default() -> Self {
        // Default snippets
        let default_snippets = vec![
            Snippet {
                name: "AT OK".to_string(),
                content: "AT\r\n".to_string(),
                description: "Basic AT command".to_string(),
            },
            Snippet {
                name: "AT+GMR".to_string(),
                content: "AT+GMR\r\n".to_string(),
                description: "Get firmware version".to_string(),
            },
            Snippet {
                name: "Ping".to_string(),
                content: "ping\r\n".to_string(),
                description: "Send ping".to_string(),
            },
            Snippet {
                name: "Help".to_string(),
                content: "help\r\n".to_string(),
                description: "Show help".to_string(),
            },
            Snippet {
                name: "Clear Screen".to_string(),
                content: "\x1b[2J\x1b[H".to_string(),
                description: "Clear terminal (ANSI)".to_string(),
            },
        ];

        Self {
            tabs: TabManager::new(),
            current_dialog: DialogType::None,
            serial_settings: SerialSettings::default(),
            tcp_settings: TcpSettings::default(),
            ssh_settings: SshSettings::default(),
            bluetooth_settings: BluetoothSettings::default(),
            status_message: "Ready".to_string(),
            available_ports: Vec::new(),
            view_mode: ViewMode::Text,
            show_side_panel: true,
            theme: AppTheme::Dark,
            side_panel_mode: SidePanelMode::Profiles,  // Start with profiles view
            snippets: default_snippets,
            new_snippet: Snippet::default(),
            language: Language::English,
            show_add_snippet: false,
            chart_data: Vec::new(),
            profile_manager: ProfileManager::load(),  // Load saved profiles
            new_profile_name: String::new(),
            active_profile_id: None,
            pending_profile_type: None,
            prompt_save_profile: false,
            macros_panel: super::macros_panel::MacrosPanel::new(),
            show_macros_bar: true,
            pending_insert_command: None,
            language_changed: false,
            virtual_com_pipe_name: "\\\\.\\pipe\\termicon_vcom".to_string(),
            ssh_keygen_type: "ed25519".to_string(),
            ssh_keygen_comment: String::new(),
            ssh_keygen_passphrase: String::new(),
            ssh_keygen_path: String::new(),
            file_transfer_path: String::new(),
            file_transfer_save_path: String::new(),
            file_transfer_progress: 0.0,
            file_transfer_status: "Ready".to_string(),
            protocol_dsl_content: String::new(),
            triggers: Vec::new(),
            new_trigger_name: String::new(),
            new_trigger_pattern: String::new(),
            new_trigger_response: String::new(),
            sftp_remote_path: "/".to_string(),
        }
    }
}

impl TermiconApp {
    /// Apply dark theme
    fn apply_dark_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.panel_fill = Color32::from_rgb(24, 24, 28);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(32, 32, 38);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 55);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 75);
        visuals.widgets.active.bg_fill = Color32::from_rgb(80, 80, 100);
        visuals.selection.bg_fill = Color32::from_rgb(70, 130, 180);
        ctx.set_visuals(visuals);
    }

    /// Apply light theme
    fn apply_light_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::light();
        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.panel_fill = Color32::from_rgb(245, 245, 248);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(235, 235, 240);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(225, 225, 230);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(210, 210, 220);
        visuals.widgets.active.bg_fill = Color32::from_rgb(190, 190, 210);
        visuals.selection.bg_fill = Color32::from_rgb(100, 150, 200);
        ctx.set_visuals(visuals);
    }

    /// Toggle theme
    pub fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.theme = match self.theme {
            AppTheme::Dark => AppTheme::Light,
            AppTheme::Light => AppTheme::Dark,
        };
        self.apply_theme(ctx);
    }

    /// Apply current theme
    pub fn apply_theme(&self, ctx: &egui::Context) {
        match self.theme {
            AppTheme::Dark => Self::apply_dark_theme(ctx),
            AppTheme::Light => Self::apply_light_theme(ctx),
        }
    }

    /// Create a new application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure fonts - use default egui fonts which have good unicode support
        // No custom font configuration needed - egui includes Ubuntu font with good coverage
        
        // Set dark theme with custom colors
        Self::apply_dark_theme(&cc.egui_ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.panel_fill = Color32::from_rgb(24, 24, 28);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(32, 32, 38);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 55);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 75);
        visuals.widgets.active.bg_fill = Color32::from_rgb(80, 80, 100);
        visuals.selection.bg_fill = Color32::from_rgb(70, 130, 180);
        cc.egui_ctx.set_visuals(visuals);

        let mut app = Self::default();
        app.refresh_serial_ports();

        // Create welcome tab
        let mut welcome_tab = SessionTab::new("Welcome", ConnectionType::Serial);
        welcome_tab.add_line("╔════════════════════════════════════════════════════════════╗", false);
        welcome_tab.add_line("║           Welcome to Termicon v0.1.0                       ║", false);
        welcome_tab.add_line("║   Professional Multi-Protocol Terminal Application         ║", false);
        welcome_tab.add_line("╚════════════════════════════════════════════════════════════╝", false);
        welcome_tab.add_line("", false);
        welcome_tab.add_line("Use the toolbar buttons to create new connections.", false);
        welcome_tab.add_line("Supported protocols: Serial, TCP, Telnet, SSH", false);
        welcome_tab.add_line("", false);
        welcome_tab.add_line("Tips:", false);
        welcome_tab.add_line("  • Press Ctrl+T to open a new tab", false);
        welcome_tab.add_line("  • Press Ctrl+W to close current tab", false);
        welcome_tab.add_line("  • Use Up/Down arrows in input for command history", false);
        app.tabs.add_tab(welcome_tab);

        app
    }

    /// Refresh available serial ports
    fn refresh_serial_ports(&mut self) {
        self.available_ports = serialport::available_ports()
            .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
            .unwrap_or_default();

        if !self.available_ports.is_empty() && self.serial_settings.port.is_empty() {
            self.serial_settings.port = self.available_ports[0].clone();
        }
    }

    /// Show connection button
    fn connection_button(&self, ui: &mut egui::Ui, label: &str, icon: &str, conn_type: ConnectionType) -> bool {
        let is_active = self.tabs.active_tab()
            .map(|t| t.conn_type == conn_type && t.state == ConnectionState::Connected)
            .unwrap_or(false);

        let button_color = if is_active {
            Color32::from_rgb(46, 160, 67)
        } else {
            Color32::from_rgb(55, 55, 65)
        };

        ui.add(
            egui::Button::new(
                RichText::new(format!("{} {}", icon, label))
                    .size(13.0)
                    .color(Color32::WHITE)
            )
            .fill(button_color)
            .corner_radius(CornerRadius::same(4))
            .min_size(Vec2::new(90.0, 28.0))
        ).clicked()
    }

    /// Render tab bar
    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let tab_count = self.tabs.count();
            let mut tab_to_close: Option<usize> = None;
            let mut tab_to_activate: Option<usize> = None;

            for i in 0..tab_count {
                let tab = &self.tabs.tabs[i];
                let is_active = i == self.tabs.active_index;
                let has_unread = tab.has_unread && !is_active;

                // Tab colors
                let bg_color = if is_active {
                    Color32::from_rgb(45, 45, 55)
                } else if has_unread {
                    Color32::from_rgb(60, 80, 60)
                } else {
                    Color32::from_rgb(32, 32, 38)
                };

                let text_color = if is_active {
                    Color32::WHITE
                } else if has_unread {
                    Color32::from_rgb(150, 255, 150)
                } else {
                    Color32::from_rgb(180, 180, 180)
                };

                // Connection indicator
                let indicator = match tab.state {
                    ConnectionState::Connected => "●",
                    ConnectionState::Connecting => "○",
                    ConnectionState::Disconnected => "○",
                };
                let indicator_color = match tab.state {
                    ConnectionState::Connected => Color32::from_rgb(46, 160, 67),
                    ConnectionState::Connecting => Color32::YELLOW,
                    ConnectionState::Disconnected => Color32::GRAY,
                };

                // Tab frame
                let response = ui.horizontal(|ui| {
                    egui::Frame::NONE
                        .fill(bg_color)
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(indicator).size(10.0).color(indicator_color));
                                ui.label(RichText::new(&tab.name).size(12.0).color(text_color));

                                // Close button
                                if ui.small_button("✕").clicked() {
                                    tab_to_close = Some(i);
                                }
                            });
                        });
                }).response;

                if response.interact(egui::Sense::click()).clicked() {
                    tab_to_activate = Some(i);
                }

                ui.add_space(2.0);
            }

            // New tab button
            if ui.add(
                egui::Button::new(RichText::new("+").size(14.0))
                    .min_size(Vec2::new(24.0, 24.0))
            ).on_hover_text("New Tab (Ctrl+T)").clicked() {
                let new_tab = SessionTab::new("New Tab", ConnectionType::Serial);
                let idx = self.tabs.add_tab(new_tab);
                self.tabs.set_active(idx);
            }

            // Process tab actions
            if let Some(idx) = tab_to_close {
                self.tabs.remove_tab(idx);
            }
            if let Some(idx) = tab_to_activate {
                self.tabs.set_active(idx);
            }
        });
    }

    /// Render terminal output with ANSI color support
    fn render_terminal(&mut self, ui: &mut egui::Ui) {
        let Some(tab) = self.tabs.active_tab() else {
            ui.centered_and_justified(|ui| {
                ui.label("No active tab");
            });
            return;
        };

        let show_timestamps = tab.show_timestamps;
        let show_hex = tab.show_hex;
        let is_dark = self.theme == AppTheme::Dark;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add_space(8.0);

                for line in &tab.output {
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(8.0);

                        // Timestamp
                        if show_timestamps {
                            let ts_color = if is_dark {
                                Color32::from_rgb(100, 100, 100)
                            } else {
                                Color32::from_rgb(120, 120, 120)
                            };
                            ui.label(RichText::new(&line.timestamp)
                                .monospace()
                                .size(11.0)
                                .color(ts_color));
                            ui.add_space(8.0);
                        }

                        // Hex view
                        if show_hex {
                            if let Some(ref bytes) = line.raw_bytes {
                                let hex: String = bytes.iter()
                                    .map(|b| format!("{:02X} ", b))
                                    .collect();
                                let hex_color = if line.is_input {
                                    Color32::from_rgb(100, 200, 255)
                                } else {
                                    Color32::from_rgb(200, 200, 200)
                                };
                                ui.label(RichText::new(hex)
                                    .monospace()
                                    .size(12.0)
                                    .color(hex_color));
                                return;
                            }
                        }

                        // Parse ANSI codes and render styled text
                        if line.is_input {
                            // Input lines - show in cyan
                            ui.label(RichText::new(&line.text)
                                .monospace()
                                .size(12.0)
                                .color(Color32::from_rgb(100, 200, 255)));
                        } else {
                            // Output lines - parse ANSI codes
                            let spans = parse_ansi(&line.text);
                            for span in spans {
                                let mut text = RichText::new(&span.text)
                                    .monospace()
                                    .size(12.0)
                                    .color(span.style.get_fg());

                                if span.style.bold {
                                    text = text.strong();
                                }
                                if span.style.italic {
                                    text = text.italics();
                                }
                                if span.style.underline {
                                    text = text.underline();
                                }
                                if span.style.strikethrough {
                                    text = text.strikethrough();
                                }

                                // Background color if set
                                if span.style.bg_color.is_some() {
                                    ui.label(text.background_color(span.style.get_bg()));
                                } else {
                                    ui.label(text);
                                }
                            }
                        }
                    });
                }

                ui.add_space(8.0);
            });
    }

    /// Render side panel with snippets, history, settings
    fn render_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel")
            .default_width(280.0)
            .min_width(200.0)
            .max_width(400.0)
            .frame(egui::Frame::NONE
                .fill(if self.theme == AppTheme::Dark {
                    Color32::from_rgb(28, 28, 32)
                } else {
                    Color32::from_rgb(240, 240, 245)
                })
                .inner_margin(Margin::same(10)))
            .show(ctx, |ui| {
                // Mode selector tabs
                ui.horizontal(|ui| {
                    let btn_color = |active: bool| {
                        if active {
                            Color32::from_rgb(70, 130, 180)
                        } else if self.theme == AppTheme::Dark {
                            Color32::from_rgb(45, 45, 55)
                        } else {
                            Color32::from_rgb(200, 200, 210)
                        }
                    };

                    if ui.add(egui::Button::new("[P]").fill(btn_color(self.side_panel_mode == SidePanelMode::Profiles)))
                        .on_hover_text("Profiles")
                        .clicked() {
                        self.side_panel_mode = SidePanelMode::Profiles;
                    }
                    if ui.add(egui::Button::new("[C]").fill(btn_color(self.side_panel_mode == SidePanelMode::Snippets)))
                        .on_hover_text("Commands")
                        .clicked() {
                        self.side_panel_mode = SidePanelMode::Snippets;
                    }
                    if ui.add(egui::Button::new("[H]").fill(btn_color(self.side_panel_mode == SidePanelMode::History)))
                        .on_hover_text("History")
                        .clicked() {
                        self.side_panel_mode = SidePanelMode::History;
                    }
                    if ui.add(egui::Button::new("[G]").fill(btn_color(self.side_panel_mode == SidePanelMode::Chart)))
                        .on_hover_text("Chart")
                        .clicked() {
                        self.side_panel_mode = SidePanelMode::Chart;
                    }
                    if ui.add(egui::Button::new("[S]").fill(btn_color(self.side_panel_mode == SidePanelMode::Settings)))
                        .on_hover_text("Settings")
                        .clicked() {
                        self.side_panel_mode = SidePanelMode::Settings;
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                match self.side_panel_mode {
                    SidePanelMode::Profiles => self.render_profiles_panel(ui),
                    SidePanelMode::Snippets => self.render_snippets_panel(ui),
                    SidePanelMode::History => self.render_history_panel(ui),
                    SidePanelMode::Chart => self.render_chart_panel(ui),
                    SidePanelMode::Settings => self.render_settings_panel(ui),
                    SidePanelMode::Sftp => self.render_sftp_panel(ui),
                }
            });
    }

    /// Render SFTP panel
    fn render_sftp_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new(t!("menu.sftp_browser")).size(14.0));
        ui.add_space(8.0);
        
        // Check if we have an SSH connection
        let has_ssh = self.tabs.active_tab()
            .map(|t| t.conn_type == ConnectionType::Ssh && t.state == ConnectionState::Connected)
            .unwrap_or(false);
        
        if !has_ssh {
            ui.label(RichText::new("Connect via SSH first to use SFTP.").color(Color32::YELLOW));
            ui.add_space(10.0);
            if ui.button("Open SSH Connection...").clicked() {
                self.current_dialog = DialogType::Ssh;
            }
            return;
        }
        
        ui.group(|ui| {
            ui.label(RichText::new("Remote Files").strong());
            ui.add_space(5.0);
            
            // Remote path
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.add(egui::TextEdit::singleline(&mut self.sftp_remote_path).desired_width(200.0));
                if ui.button("Go").clicked() {
                    self.status_message = format!("Navigating to {}", self.sftp_remote_path);
                }
            });
            
            ui.add_space(5.0);
            
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                // Placeholder file list
                ui.label(".. (parent)");
                ui.label("bin/");
                ui.label("etc/");
                ui.label("home/");
                ui.label("tmp/");
                ui.label("var/");
            });
        });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            if ui.button("Upload...").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.status_message = format!("Uploading {}", path.display());
                }
            }
            if ui.button("Download...").clicked() {
                self.status_message = "Select remote file first".to_string();
            }
        });
    }

    /// Render snippets panel
    fn render_snippets_panel(&mut self, ui: &mut egui::Ui) {
        // Check if we have a profile-specific snippet list
        let profile_id = self.tabs.active_tab()
            .and_then(|tab| tab.profile_id.clone());
        
        // Only show if we have a profile
        if profile_id.is_none() {
            ui.heading(RichText::new("Commands").size(14.0));
            ui.add_space(10.0);
            ui.label(RichText::new("No profile active").color(Color32::GRAY));
            ui.add_space(5.0);
            ui.label(RichText::new("Connect from a saved profile to see your saved commands here.").size(11.0).color(Color32::GRAY));
            ui.add_space(10.0);
            ui.label(RichText::new("Tip: Save your connection as a profile, and all commands you type will be remembered and sorted by usage!").size(10.0).color(Color32::DARK_GRAY));
            return;
        }
        
        let pid = profile_id.unwrap();
        
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Commands").size(14.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("➕").on_hover_text("Add new command").clicked() {
                    self.show_add_snippet = true;
                }
            });
        });
        
        ui.label(RichText::new("Sorted by usage (most used first)").size(10.0).color(Color32::GRAY));
        ui.add_space(8.0);

        // Add snippet dialog
        if self.show_add_snippet {
            ui.group(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_snippet.name);
                ui.label("Command:");
                ui.text_edit_singleline(&mut self.new_snippet.content);
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() && !self.new_snippet.name.is_empty() {
                        let snippet = ProfileSnippet::new(
                            self.new_snippet.name.clone(),
                            self.new_snippet.content.trim().to_string(),
                            String::new(),
                        );
                        if let Some(profile) = self.profile_manager.get_mut(&pid) {
                            profile.add_snippet(snippet);
                            self.profile_manager.save();
                        }
                        self.new_snippet = Snippet::default();
                        self.show_add_snippet = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.new_snippet = Snippet::default();
                        self.show_add_snippet = false;
                    }
                });
            });
            ui.add_space(8.0);
        }

        // Profile-specific snippets, sorted by usage
        let snippets: Vec<ProfileSnippet> = self.profile_manager.get(&pid)
            .map(|p| p.sorted_snippets().into_iter().cloned().collect())
            .unwrap_or_default();
        
        if snippets.is_empty() {
            ui.label(RichText::new("No commands yet.").size(11.0).color(Color32::GRAY));
            ui.label(RichText::new("Any command you type will appear here automatically!").size(10.0).color(Color32::DARK_GRAY));
        } else {
            let mut snippet_to_insert: Option<String> = None;
            let mut snippet_to_delete: Option<usize> = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (idx, snippet) in snippets.iter().enumerate() {
                        let response = ui.group(|ui| {
                            ui.horizontal(|ui| {
                                // Usage count badge
                                if snippet.usage_count > 0 {
                                    ui.label(RichText::new(format!("x{}", snippet.usage_count))
                                        .size(9.0)
                                        .color(Color32::from_rgb(100, 180, 100)));
                                }
                                ui.label(RichText::new(&snippet.name).strong().size(11.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("X").on_hover_text("Delete").clicked() {
                                        snippet_to_delete = Some(idx);
                                    }
                                });
                            });
                            ui.label(RichText::new(snippet.content.trim()).monospace().size(10.0).color(
                                if self.theme == AppTheme::Dark {
                                    Color32::from_rgb(150, 200, 150)
                                } else {
                                    Color32::from_rgb(50, 120, 50)
                                }
                            ));
                        });

                        // Double-click to INSERT into command line (not send!)
                        if response.response.double_clicked() {
                            snippet_to_insert = Some(snippet.content.trim().to_string());
                        }
                        if response.response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                    }
                });

            // Insert into command line (NOT send)
            // We store it as pending and process it at the start of next frame
            if let Some(content) = snippet_to_insert {
                self.pending_insert_command = Some(content);
                ui.ctx().request_repaint();
            }

            // Delete snippet
            if let Some(idx) = snippet_to_delete {
                if let Some(profile) = self.profile_manager.get_mut(&pid) {
                    // Find the actual index in the unsorted list
                    let sorted_snippets: Vec<_> = profile.sorted_snippets().iter().map(|s| s.content.clone()).collect();
                    if idx < sorted_snippets.len() {
                        let content_to_delete = &sorted_snippets[idx];
                        profile.snippets.retain(|s| &s.content != content_to_delete);
                        self.profile_manager.save();
                    }
                }
            }
        }

        ui.add_space(10.0);
        ui.label(RichText::new("Double-click to insert into command line").size(10.0).color(Color32::GRAY));
    }

    /// Render history panel
    fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("History").size(14.0));
        ui.add_space(8.0);

        // First collect the history items
        let history: Vec<String> = self.tabs.active_tab()
            .map(|tab| tab.input_history.iter().rev().take(50).cloned().collect())
            .unwrap_or_default();

        if history.is_empty() {
            ui.label("No command history");
        } else {
            let mut cmd_to_send: Option<String> = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for cmd in &history {
                        let response = ui.selectable_label(false, RichText::new(cmd).monospace().size(11.0));
                        if response.double_clicked() {
                            cmd_to_send = Some(cmd.clone());
                        }
                        if response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                    }
                });

            // Send from history
            if let Some(content) = cmd_to_send {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    if let Some(ref tx) = tab.tx {
                        let data = format!("{}\r\n", content);
                        let _ = tx.send(ConnectionCommand::Send(data.as_bytes().to_vec()));
                        if tab.local_echo {
                            tab.add_line(&content, true);
                        }
                    }
                }
            }
        }

        ui.add_space(10.0);
        ui.label(RichText::new("Double-click to resend").size(10.0).color(Color32::GRAY));
    }

    /// Render chart panel
    fn render_chart_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("[Chart] Real-time Data").size(14.0));
        ui.add_space(8.0);

        ui.label("Chart visualization for numeric data streams.");
        ui.label("Parse patterns: CSV, key=value, JSON");
        ui.add_space(10.0);

        // Simple demo chart using egui plot
        use egui_plot::{Line, Plot, PlotPoints};

        // Add random data for demo
        if self.chart_data.len() < 100 {
            self.chart_data.push(
                50.0 + 30.0 * (self.chart_data.len() as f64 * 0.1).sin() + 
                (rand_simple() * 10.0 - 5.0)
            );
        } else {
            self.chart_data.remove(0);
            self.chart_data.push(
                50.0 + 30.0 * ((self.chart_data.len() + 100) as f64 * 0.1).sin() + 
                (rand_simple() * 10.0 - 5.0)
            );
        }

        let points: PlotPoints = self.chart_data.iter()
            .enumerate()
            .map(|(i, &v)| [i as f64, v])
            .collect();
        let line = Line::new(points)
            .color(Color32::from_rgb(100, 200, 255))
            .name("Data");

        Plot::new("demo_chart")
            .height(200.0)
            .show_axes(true)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });

        ui.add_space(10.0);
        ui.label(RichText::new("Connect to see real data").size(10.0).color(Color32::GRAY));
    }

    /// Render settings panel
    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Settings").size(14.0));
        ui.add_space(15.0);

        // Theme
        ui.group(|ui| {
            ui.label(RichText::new("Theme").strong());
            ui.horizontal(|ui| {
                if ui.selectable_label(self.theme == AppTheme::Dark, "Dark").clicked() {
                    self.theme = AppTheme::Dark;
                    Self::apply_dark_theme(ui.ctx());
                }
                if ui.selectable_label(self.theme == AppTheme::Light, "Light").clicked() {
                    self.theme = AppTheme::Light;
                    Self::apply_light_theme(ui.ctx());
                }
            });
        });

        ui.add_space(10.0);

        // Language
        ui.group(|ui| {
            ui.label(RichText::new("Language").strong());
            ui.horizontal(|ui| {
                if ui.selectable_label(self.language == Language::English, "English").clicked() {
                    self.language = Language::English;
                    set_locale(Locale::English);
                    self.language_changed = true;
                }
                if ui.selectable_label(self.language == Language::Hungarian, "Magyar").clicked() {
                    self.language = Language::Hungarian;
                    set_locale(Locale::Hungarian);
                    self.language_changed = true;
                }
            });
        });

        ui.add_space(10.0);

        // Terminal settings
        if let Some(tab) = self.tabs.active_tab_mut() {
            ui.group(|ui| {
                ui.label(RichText::new("Terminal").strong());
                ui.checkbox(&mut tab.show_timestamps, "Show Timestamps");
                ui.checkbox(&mut tab.show_hex, "Hex View");
                ui.checkbox(&mut tab.local_echo, "Local Echo");
            });
        }

        ui.add_space(10.0);

        // About info
        ui.group(|ui| {
            ui.label(RichText::new("[i] About").strong());
            ui.label("Termicon v0.1.0");
            ui.label("Multi-protocol Terminal");
            ui.label(RichText::new("Serial • TCP • Telnet • SSH • BLE").size(10.0));
        });
    }

    /// Render profiles panel
    fn render_profiles_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(RichText::new("Profiles").size(14.0));
        ui.add_space(8.0);

        // Type filter buttons
        ui.horizontal_wrapped(|ui| {
            let filter_btn = |ui: &mut egui::Ui, manager: &mut ProfileManager, filter: Option<ProfileType>, label: &str, theme: AppTheme| {
                let is_active = manager.filter == filter;
                let btn_color = if is_active {
                    Color32::from_rgb(70, 130, 180)
                } else if theme == AppTheme::Dark {
                    Color32::from_rgb(45, 45, 55)
                } else {
                    Color32::from_rgb(200, 200, 210)
                };
                if ui.add(egui::Button::new(label).fill(btn_color).min_size(Vec2::new(40.0, 24.0))).clicked() {
                    manager.filter = filter;
                }
            };

            filter_btn(ui, &mut self.profile_manager, None, "All", self.theme);
            for pt in ProfileType::all() {
                let count = self.profile_manager.count_by_type(*pt);
                let label = format!("{} {}", pt.icon(), count);
                filter_btn(ui, &mut self.profile_manager, Some(*pt), &label, self.theme);
            }
        });

        ui.add_space(8.0);

        // Search
        ui.horizontal(|ui| {
            ui.label("[S]");
            ui.add(egui::TextEdit::singleline(&mut self.profile_manager.search_query)
                .hint_text("Search profiles...")
                .desired_width(ui.available_width()));
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Profile list
        let filtered: Vec<_> = self.profile_manager.filtered_profiles().iter().map(|p| (*p).clone()).collect();

        if filtered.is_empty() {
            ui.label(RichText::new("No profiles yet").color(Color32::GRAY));
            ui.label(RichText::new("Connect to a device and save it as a profile!").size(11.0).color(Color32::GRAY));
        } else {
            let mut profile_to_connect: Option<String> = None;
            let mut profile_to_delete: Option<String> = None;
            let mut profile_to_favorite: Option<String> = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for profile in &filtered {
                        let is_active = self.active_profile_id.as_ref() == Some(&profile.id);
                        let bg_color = if is_active {
                            if self.theme == AppTheme::Dark {
                                Color32::from_rgb(50, 70, 50)
                            } else {
                                Color32::from_rgb(200, 230, 200)
                            }
                        } else if self.theme == AppTheme::Dark {
                            Color32::from_rgb(38, 38, 45)
                        } else {
                            Color32::from_rgb(250, 250, 252)
                        };

                        let profile_id_clone = profile.id.clone();
                        
                        let frame_response = egui::Frame::NONE
                            .fill(bg_color)
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Type icon
                                    ui.label(RichText::new(profile.profile_type.icon()).size(16.0));
                                    
                                    ui.vertical(|ui| {
                                        // Name
                                        ui.label(RichText::new(&profile.name).strong());
                                        // Connection summary
                                        ui.label(RichText::new(profile.connection_summary()).size(10.0).color(Color32::GRAY));
                                        // Use count
                                        if profile.use_count > 0 {
                                            ui.label(RichText::new(format!("Used {} times", profile.use_count)).size(9.0).color(Color32::DARK_GRAY));
                                        }
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Delete button
                                        if ui.small_button("X").on_hover_text("Delete profile").clicked() {
                                            profile_to_delete = Some(profile.id.clone());
                                        }
                                        
                                        // Favorite toggle
                                        let star = if profile.favorite { "[*]" } else { "[ ]" };
                                        if ui.small_button(star).on_hover_text("Toggle favorite").clicked() {
                                            profile_to_favorite = Some(profile.id.clone());
                                        }
                                    });
                                });
                            });
                        
                        // Make the entire frame clickable - double click to connect
                        let interact = ui.interact(
                            frame_response.response.rect,
                            egui::Id::new(format!("profile_{}", profile_id_clone)),
                            egui::Sense::click()
                        );
                        
                        if interact.double_clicked() {
                            profile_to_connect = Some(profile_id_clone);
                        }
                        
                        if interact.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }

                        ui.add_space(4.0);
                    }
                });

            // Handle actions
            if let Some(id) = profile_to_favorite {
                self.profile_manager.toggle_favorite(&id);
            }
            if let Some(id) = profile_to_delete {
                self.profile_manager.remove(&id);
            }
            if let Some(id) = profile_to_connect {
                self.connect_from_profile(&id);
            }
        }

        ui.add_space(10.0);
        ui.label(RichText::new("Double-click profile to connect").size(10.0).color(Color32::GRAY));
    }

    /// Connect from a saved profile
    fn connect_from_profile(&mut self, profile_id: &str) {
        let profile = self.profile_manager.get(profile_id).cloned();
        if let Some(profile) = profile {
            self.active_profile_id = Some(profile_id.to_string());
            self.profile_manager.record_use(profile_id);

            match profile.profile_type {
                ProfileType::Serial => {
                    if let Some(ref settings) = profile.serial {
                        self.serial_settings = SerialSettings {
                            port: settings.port.clone(),
                            baud_rate: settings.baud_rate.to_string(),
                            data_bits: settings.data_bits.to_string(),
                            parity: settings.parity.clone(),
                            stop_bits: settings.stop_bits.clone(),
                            flow_control: settings.flow_control.clone(),
                        };
                        self.connect_serial();
                    }
                }
                ProfileType::Tcp => {
                    if let Some(ref settings) = profile.tcp {
                        self.tcp_settings = TcpSettings {
                            host: settings.host.clone(),
                            port: settings.port.to_string(),
                        };
                        self.connect_tcp();
                    }
                }
                ProfileType::Telnet => {
                    if let Some(ref settings) = profile.tcp {
                        self.tcp_settings = TcpSettings {
                            host: settings.host.clone(),
                            port: settings.port.to_string(),
                        };
                        self.connect_telnet();
                    }
                }
                ProfileType::Ssh => {
                    if let Some(ref settings) = profile.ssh {
                        self.ssh_settings = SshSettings {
                            host: settings.host.clone(),
                            port: settings.port.to_string(),
                            username: settings.username.clone(),
                            password: settings.saved_password.clone().unwrap_or_default(),
                            use_key: settings.use_key,
                            key_path: settings.key_path.clone(),
                            key_passphrase: String::new(),
                            show_advanced: false,
                            jump_host: settings.jump_host.clone().unwrap_or_default(),
                            jump_port: settings.jump_port.map(|p| p.to_string()).unwrap_or_else(|| "22".to_string()),
                            jump_username: settings.jump_username.clone().unwrap_or_default(),
                            jump_password: String::new(),
                            compression: settings.compression,
                            keepalive_interval: settings.keepalive_interval.map(|k| k.to_string()).unwrap_or_else(|| "30".to_string()),
                            connection_timeout: settings.connection_timeout.map(|t| t.to_string()).unwrap_or_else(|| "10".to_string()),
                            save_password: settings.save_password,
                            auto_connect: settings.auto_connect,
                            terminal_type: settings.terminal_type.clone().unwrap_or_else(|| "xterm-256color".to_string()),
                            x11_forwarding: settings.x11_forwarding,
                            agent_forwarding: settings.agent_forwarding,
                            local_port_forward: settings.local_port_forward.clone().unwrap_or_default(),
                            remote_port_forward: settings.remote_port_forward.clone().unwrap_or_default(),
                        };
                        
                        // If password is saved and auto_connect is enabled, connect directly
                        if settings.save_password && settings.saved_password.is_some() && settings.auto_connect {
                            self.connect_ssh();
                        } else {
                            // Show SSH dialog to enter password
                            self.current_dialog = DialogType::Ssh;
                        }
                    }
                }
                ProfileType::Bluetooth => {
                    if let Some(ref settings) = profile.bluetooth {
                        self.bluetooth_settings = BluetoothSettings {
                            device: settings.device.clone(),
                            service_uuid: settings.service_uuid.clone(),
                            tx_uuid: settings.tx_uuid.clone(),
                            rx_uuid: settings.rx_uuid.clone(),
                        };
                        self.connect_bluetooth();
                    }
                }
            }
        }
    }

    /// Show save profile dialog
    fn show_save_profile_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Save Profile")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(300.0);
                ui.add_space(10.0);

                ui.label("Save this connection as a profile for quick access.");
                ui.add_space(10.0);

                ui.label("Profile name:");
                ui.add(egui::TextEdit::singleline(&mut self.new_profile_name)
                    .hint_text("e.g., My SSH Server, Arduino, ESP32...")
                    .desired_width(280.0));

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() && !self.new_profile_name.is_empty() {
                        self.save_current_as_profile();
                        self.current_dialog = DialogType::None;
                        self.prompt_save_profile = false;
                    }
                    if ui.button("Skip").clicked() {
                        self.current_dialog = DialogType::None;
                        self.prompt_save_profile = false;
                    }
                    if ui.button("Don't ask again").clicked() {
                        self.current_dialog = DialogType::None;
                        self.prompt_save_profile = false;
                        // Could save preference to config
                    }
                });
            });
    }

    /// Save current connection settings as a profile
    fn save_current_as_profile(&mut self) {
        let name = self.new_profile_name.clone();
        if name.is_empty() {
            return;
        }

        let profile = match self.pending_profile_type {
            Some(ProfileType::Serial) => {
                Profile::new_serial(name, SerialProfileSettings {
                    port: self.serial_settings.port.clone(),
                    baud_rate: self.serial_settings.baud_rate.parse().unwrap_or(115200),
                    data_bits: self.serial_settings.data_bits.parse().unwrap_or(8),
                    parity: self.serial_settings.parity.clone(),
                    stop_bits: self.serial_settings.stop_bits.clone(),
                    flow_control: self.serial_settings.flow_control.clone(),
                })
            }
            Some(ProfileType::Tcp) | Some(ProfileType::Telnet) => {
                let pt = self.pending_profile_type.unwrap();
                let mut p = Profile::new(name, pt);
                p.tcp = Some(TcpProfileSettings {
                    host: self.tcp_settings.host.clone(),
                    port: self.tcp_settings.port.parse().unwrap_or(23),
                });
                p
            }
            Some(ProfileType::Ssh) => {
                Profile::new_ssh(name, SshProfileSettings {
                    host: self.ssh_settings.host.clone(),
                    port: self.ssh_settings.port.parse().unwrap_or(22),
                    username: self.ssh_settings.username.clone(),
                    use_key: self.ssh_settings.use_key,
                    key_path: self.ssh_settings.key_path.clone(),
                    saved_password: if self.ssh_settings.save_password { 
                        Some(self.ssh_settings.password.clone()) 
                    } else { 
                        None 
                    },
                    save_password: self.ssh_settings.save_password,
                    auto_connect: self.ssh_settings.auto_connect,
                    jump_host: if self.ssh_settings.jump_host.is_empty() { None } else { Some(self.ssh_settings.jump_host.clone()) },
                    jump_port: self.ssh_settings.jump_port.parse().ok(),
                    jump_username: if self.ssh_settings.jump_username.is_empty() { None } else { Some(self.ssh_settings.jump_username.clone()) },
                    compression: self.ssh_settings.compression,
                    keepalive_interval: self.ssh_settings.keepalive_interval.parse().ok(),
                    connection_timeout: self.ssh_settings.connection_timeout.parse().ok(),
                    terminal_type: Some(self.ssh_settings.terminal_type.clone()),
                    x11_forwarding: self.ssh_settings.x11_forwarding,
                    agent_forwarding: self.ssh_settings.agent_forwarding,
                    local_port_forward: if self.ssh_settings.local_port_forward.is_empty() { None } else { Some(self.ssh_settings.local_port_forward.clone()) },
                    remote_port_forward: if self.ssh_settings.remote_port_forward.is_empty() { None } else { Some(self.ssh_settings.remote_port_forward.clone()) },
                })
            }
            Some(ProfileType::Bluetooth) => {
                Profile::new_bluetooth(name, BluetoothProfileSettings {
                    device: self.bluetooth_settings.device.clone(),
                    service_uuid: self.bluetooth_settings.service_uuid.clone(),
                    tx_uuid: self.bluetooth_settings.tx_uuid.clone(),
                    rx_uuid: self.bluetooth_settings.rx_uuid.clone(),
                })
            }
            None => return,
        };

        self.active_profile_id = Some(profile.id.clone());
        self.profile_manager.add(profile);
        self.new_profile_name.clear();
        self.pending_profile_type = None;
    }

    /// Render input area
    fn render_input(&mut self, ui: &mut egui::Ui) {
        // Capture info we need before taking mutable borrow
        let profile_id = self.tabs.active_tab().and_then(|t| t.profile_id.clone());
        let mut command_sent: Option<String> = None;
        
        {
            let Some(tab) = self.tabs.active_tab_mut() else {
                return;
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(">").monospace().size(14.0).color(Color32::from_rgb(100, 200, 255)));

                let response = ui.add(
                    egui::TextEdit::singleline(&mut tab.current_input)
                        .font(FontId::monospace(13.0))
                        .desired_width(ui.available_width() - 80.0)
                        .frame(false)
                );

                // Handle Enter key
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // Capture command before sending
                    if !tab.current_input.is_empty() {
                        command_sent = Some(tab.current_input.clone());
                    }
                    tab.send_input();
                    response.request_focus();
                }

                // Handle Up/Down for history
                if response.has_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        tab.history_up();
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        tab.history_down();
                    }
                }

                if ui.button(RichText::new("Send").size(12.0)).clicked() {
                    if !tab.current_input.is_empty() {
                        command_sent = Some(tab.current_input.clone());
                    }
                    tab.send_input();
                }
            });
        }
        
        // Record command to profile if connected from one
        if let (Some(pid), Some(cmd)) = (profile_id, command_sent) {
            self.profile_manager.record_command(&pid, &cmd);
        }
    }

    /// Show serial dialog
    fn show_serial_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Serial Port Connection")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(350.0);
                ui.add_space(10.0);

                egui::Grid::new("serial_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Port:");
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("serial_port")
                                .selected_text(&self.serial_settings.port)
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    for port in &self.available_ports {
                                        ui.selectable_value(&mut self.serial_settings.port, port.clone(), port);
                                    }
                                });
                            if ui.small_button("R").on_hover_text("Refresh ports").clicked() {
                                self.refresh_serial_ports();
                            }
                        });
                        ui.end_row();

                        ui.label("Baud Rate:");
                        egui::ComboBox::from_id_salt("baud_rate")
                            .selected_text(&self.serial_settings.baud_rate)
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for rate in &["9600", "19200", "38400", "57600", "115200", "230400", "460800", "921600"] {
                                    ui.selectable_value(&mut self.serial_settings.baud_rate, rate.to_string(), *rate);
                                }
                            });
                        ui.end_row();

                        ui.label("Data Bits:");
                        egui::ComboBox::from_id_salt("data_bits")
                            .selected_text(&self.serial_settings.data_bits)
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for bits in &["7", "8"] {
                                    ui.selectable_value(&mut self.serial_settings.data_bits, bits.to_string(), *bits);
                                }
                            });
                        ui.end_row();

                        ui.label("Parity:");
                        egui::ComboBox::from_id_salt("parity")
                            .selected_text(&self.serial_settings.parity)
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for p in &["None", "Odd", "Even"] {
                                    ui.selectable_value(&mut self.serial_settings.parity, p.to_string(), *p);
                                }
                            });
                        ui.end_row();

                        ui.label("Stop Bits:");
                        egui::ComboBox::from_id_salt("stop_bits")
                            .selected_text(&self.serial_settings.stop_bits)
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for bits in &["1", "2"] {
                                    ui.selectable_value(&mut self.serial_settings.stop_bits, bits.to_string(), *bits);
                                }
                            });
                        ui.end_row();

                        ui.label("Flow Control:");
                        egui::ComboBox::from_id_salt("flow_control")
                            .selected_text(&self.serial_settings.flow_control)
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for fc in &["None", "Hardware", "Software"] {
                                    ui.selectable_value(&mut self.serial_settings.flow_control, fc.to_string(), *fc);
                                }
                            });
                        ui.end_row();
                    });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect_serial();
                        self.current_dialog = DialogType::None;
                    }
                    if ui.button("Cancel").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
    }

    /// Show TCP dialog
    fn show_tcp_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("🌐 TCP Connection")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(300.0);
                ui.add_space(10.0);

                egui::Grid::new("tcp_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Host:");
                        ui.add(egui::TextEdit::singleline(&mut self.tcp_settings.host).desired_width(180.0));
                        ui.end_row();

                        ui.label("Port:");
                        ui.add(egui::TextEdit::singleline(&mut self.tcp_settings.port).desired_width(180.0));
                        ui.end_row();
                    });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect_tcp();
                        self.current_dialog = DialogType::None;
                    }
                    if ui.button("Cancel").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
    }

    /// Show Telnet dialog
    fn show_telnet_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("📡 Telnet Connection")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(300.0);
                ui.add_space(10.0);

                egui::Grid::new("telnet_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Host:");
                        ui.add(egui::TextEdit::singleline(&mut self.tcp_settings.host).desired_width(180.0));
                        ui.end_row();

                        ui.label("Port:");
                        ui.add(egui::TextEdit::singleline(&mut self.tcp_settings.port).desired_width(180.0));
                        ui.end_row();
                    });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect_telnet();
                        self.current_dialog = DialogType::None;
                    }
                    if ui.button("Cancel").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
    }

    /// Show SSH dialog
    fn show_ssh_dialog(&mut self, ctx: &egui::Context) {
        let window_height = if self.ssh_settings.show_advanced { 580.0 } else { 320.0 };
        
        egui::Window::new("SSH Connection")
            .collapsible(false)
            .resizable(true)
            .default_size([450.0, window_height])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Basic settings section
                    ui.group(|ui| {
                        ui.label(RichText::new("Basic Connection").strong());
                        ui.add_space(5.0);
                        
                        egui::Grid::new("ssh_basic_grid")
                            .num_columns(2)
                            .spacing([20.0, 6.0])
                            .show(ui, |ui| {
                                ui.label("Host:");
                                ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.host)
                                    .desired_width(250.0)
                                    .hint_text("hostname or IP"));
                                ui.end_row();

                                ui.label("Port:");
                                ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.port)
                                    .desired_width(80.0));
                                ui.end_row();

                                ui.label("Username:");
                                ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.username)
                                    .desired_width(250.0));
                                ui.end_row();

                                ui.label("Password:");
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.password)
                                        .password(true)
                                        .desired_width(180.0));
                                    ui.checkbox(&mut self.ssh_settings.save_password, "Save");
                                });
                                ui.end_row();
                            });
                    });

                    ui.add_space(5.0);

                    // Key Authentication
                    ui.group(|ui| {
                        ui.checkbox(&mut self.ssh_settings.use_key, RichText::new("Use SSH Key Authentication").strong());
                        
                        if self.ssh_settings.use_key {
                            ui.add_space(5.0);
                            egui::Grid::new("ssh_key_grid")
                                .num_columns(2)
                                .spacing([20.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Key file:");
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.key_path)
                                            .desired_width(200.0)
                                            .hint_text("~/.ssh/id_rsa"));
                                        if ui.button("...").clicked() {
                                            // File browser would go here
                                        }
                                    });
                                    ui.end_row();

                                    ui.label("Passphrase:");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.key_passphrase)
                                        .password(true)
                                        .desired_width(200.0));
                                    ui.end_row();
                                });
                        }
                    });

                    ui.add_space(5.0);

                    // Advanced settings toggle
                    ui.horizontal(|ui| {
                        let arrow = if self.ssh_settings.show_advanced { "v" } else { ">" };
                        if ui.button(format!("{} Advanced Settings", arrow)).clicked() {
                            self.ssh_settings.show_advanced = !self.ssh_settings.show_advanced;
                        }
                    });

                    if self.ssh_settings.show_advanced {
                        ui.add_space(5.0);
                        
                        // Jump Host / Proxy
                        ui.group(|ui| {
                            ui.label(RichText::new("Jump Host (ProxyJump)").strong());
                            ui.add_space(5.0);
                            
                            egui::Grid::new("ssh_jump_grid")
                                .num_columns(2)
                                .spacing([20.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Jump Host:");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.jump_host)
                                        .desired_width(200.0)
                                        .hint_text("bastion.example.com"));
                                    ui.end_row();

                                    ui.label("Jump Port:");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.jump_port)
                                        .desired_width(80.0));
                                    ui.end_row();

                                    ui.label("Jump User:");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.jump_username)
                                        .desired_width(200.0));
                                    ui.end_row();

                                    ui.label("Jump Password:");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.jump_password)
                                        .password(true)
                                        .desired_width(200.0));
                                    ui.end_row();
                                });
                        });

                        ui.add_space(5.0);

                        // Port Forwarding
                        ui.group(|ui| {
                            ui.label(RichText::new("Port Forwarding").strong());
                            ui.add_space(5.0);
                            
                            egui::Grid::new("ssh_forward_grid")
                                .num_columns(2)
                                .spacing([20.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Local (-L):");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.local_port_forward)
                                        .desired_width(200.0)
                                        .hint_text("8080:localhost:80"));
                                    ui.end_row();

                                    ui.label("Remote (-R):");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.remote_port_forward)
                                        .desired_width(200.0)
                                        .hint_text("9090:localhost:80"));
                                    ui.end_row();
                                });
                        });

                        ui.add_space(5.0);

                        // Connection Options
                        ui.group(|ui| {
                            ui.label(RichText::new("Connection Options").strong());
                            ui.add_space(5.0);
                            
                            egui::Grid::new("ssh_options_grid")
                                .num_columns(2)
                                .spacing([20.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label("Terminal Type:");
                                    egui::ComboBox::from_id_salt("term_type")
                                        .selected_text(&self.ssh_settings.terminal_type)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut self.ssh_settings.terminal_type, "xterm-256color".to_string(), "xterm-256color");
                                            ui.selectable_value(&mut self.ssh_settings.terminal_type, "xterm".to_string(), "xterm");
                                            ui.selectable_value(&mut self.ssh_settings.terminal_type, "vt100".to_string(), "vt100");
                                            ui.selectable_value(&mut self.ssh_settings.terminal_type, "vt220".to_string(), "vt220");
                                            ui.selectable_value(&mut self.ssh_settings.terminal_type, "linux".to_string(), "linux");
                                        });
                                    ui.end_row();

                                    ui.label("Keepalive (sec):");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.keepalive_interval)
                                        .desired_width(60.0));
                                    ui.end_row();

                                    ui.label("Timeout (sec):");
                                    ui.add(egui::TextEdit::singleline(&mut self.ssh_settings.connection_timeout)
                                        .desired_width(60.0));
                                    ui.end_row();
                                });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.ssh_settings.compression, "Compression");
                                ui.checkbox(&mut self.ssh_settings.x11_forwarding, "X11 Forward");
                                ui.checkbox(&mut self.ssh_settings.agent_forwarding, "Agent Forward");
                            });
                        });
                    }

                    ui.add_space(10.0);

                    // Auto connect option
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.ssh_settings.auto_connect, "Auto-connect on startup");
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        let connect_btn = egui::Button::new("Connect").fill(Color32::from_rgb(60, 130, 80));
                        if ui.add(connect_btn).clicked() {
                            self.connect_ssh();
                            self.current_dialog = DialogType::None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.current_dialog = DialogType::None;
                        }
                    });
                });
            });
    }

    /// Show Bluetooth dialog
    fn show_bluetooth_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("BLE Connection")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(400.0);
                ui.add_space(10.0);

                egui::Grid::new("ble_grid")
                    .num_columns(2)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Device Name/Address:");
                        ui.add(egui::TextEdit::singleline(&mut self.bluetooth_settings.device).desired_width(250.0));
                        ui.end_row();

                        ui.label("Service UUID:");
                        ui.add(egui::TextEdit::singleline(&mut self.bluetooth_settings.service_uuid).desired_width(250.0));
                        ui.end_row();

                        ui.label("TX Characteristic:");
                        ui.add(egui::TextEdit::singleline(&mut self.bluetooth_settings.tx_uuid).desired_width(250.0));
                        ui.end_row();

                        ui.label("RX Characteristic:");
                        ui.add(egui::TextEdit::singleline(&mut self.bluetooth_settings.rx_uuid).desired_width(250.0));
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.label(RichText::new("Note: Default UUIDs are for Nordic UART Service (NUS)").small().color(Color32::GRAY));

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect_bluetooth();
                        self.current_dialog = DialogType::None;
                    }
                    if ui.button("Scan").clicked() {
                        // TODO: Implement BLE scanning
                        self.status_message = "Scanning for BLE devices...".to_string();
                    }
                    if ui.button("Cancel").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
    }

    /// Show about dialog
    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("About Termicon")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(300.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading(RichText::new("Termicon").size(24.0).strong());
                    ui.label(RichText::new("v0.1.0").color(Color32::GRAY));
                    ui.add_space(10.0);
                    ui.label("Professional Multi-Protocol Terminal");
                    ui.add_space(15.0);
                    ui.label(RichText::new("Supported Protocols:").strong());
                    ui.label("- Serial (RS-232/RS-485/USB)");
                    ui.label("- TCP/IP");
                    ui.label("- Telnet");
                    ui.label("- SSH-2");
                    ui.label("- Bluetooth LE");
                    ui.add_space(15.0);

                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    /// Show keyboard shortcuts dialog
    fn show_keyboard_shortcuts_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Keyboard Shortcuts")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([450.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                    ui.heading("General");
                    ui.add_space(5.0);
                    egui::Grid::new("shortcuts_general").num_columns(2).spacing([40.0, 4.0]).show(ui, |ui| {
                        ui.label(RichText::new("Ctrl+T").monospace()); ui.label("New Tab"); ui.end_row();
                        ui.label(RichText::new("Ctrl+W").monospace()); ui.label("Close Tab"); ui.end_row();
                        ui.label(RichText::new("Ctrl+K").monospace()); ui.label("Command Palette"); ui.end_row();
                        ui.label(RichText::new("Ctrl+D").monospace()); ui.label("Disconnect"); ui.end_row();
                        ui.label(RichText::new("Ctrl+L").monospace()); ui.label("Clear Terminal"); ui.end_row();
                        ui.label(RichText::new("Ctrl+F").monospace()); ui.label("Search"); ui.end_row();
                        ui.label(RichText::new("Escape").monospace()); ui.label("Close Dialog"); ui.end_row();
                    });
                    
                    ui.add_space(15.0);
                    ui.heading("Navigation");
                    ui.add_space(5.0);
                    egui::Grid::new("shortcuts_nav").num_columns(2).spacing([40.0, 4.0]).show(ui, |ui| {
                        ui.label(RichText::new("Ctrl+Tab").monospace()); ui.label("Next Tab"); ui.end_row();
                        ui.label(RichText::new("Ctrl+Shift+Tab").monospace()); ui.label("Previous Tab"); ui.end_row();
                        ui.label(RichText::new("Alt+1-9").monospace()); ui.label("Switch to Tab 1-9"); ui.end_row();
                        ui.label(RichText::new("Page Up/Down").monospace()); ui.label("Scroll Output"); ui.end_row();
                    });
                    
                    ui.add_space(15.0);
                    ui.heading("Terminal");
                    ui.add_space(5.0);
                    egui::Grid::new("shortcuts_term").num_columns(2).spacing([40.0, 4.0]).show(ui, |ui| {
                        ui.label(RichText::new("Enter").monospace()); ui.label("Send Command"); ui.end_row();
                        ui.label(RichText::new("Up/Down").monospace()); ui.label("Command History"); ui.end_row();
                        ui.label(RichText::new("Ctrl+C").monospace()); ui.label("Copy Selection"); ui.end_row();
                        ui.label(RichText::new("Ctrl+V").monospace()); ui.label("Paste"); ui.end_row();
                    });
                    
                    ui.add_space(15.0);
                    ui.heading("Macros");
                    ui.add_space(5.0);
                    egui::Grid::new("shortcuts_macros").num_columns(2).spacing([40.0, 4.0]).show(ui, |ui| {
                        ui.label(RichText::new("F1-F12").monospace()); ui.label("Execute M1-M12"); ui.end_row();
                        ui.label(RichText::new("Shift+F1-F12").monospace()); ui.label("Execute M13-M24"); ui.end_row();
                    });
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    /// Show user guide dialog
    fn show_user_guide_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("User Guide")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    ui.heading("Termicon User Guide");
                    ui.add_space(10.0);
                    
                    ui.heading("Quick Start");
                    ui.add_space(5.0);
                    ui.label("1. Click a connection button (Serial, TCP, SSH, etc.)");
                    ui.label("2. Configure connection settings");
                    ui.label("3. Click Connect");
                    ui.label("4. Save as Profile for quick access later");
                    
                    ui.add_space(15.0);
                    ui.heading("Connection Types");
                    ui.add_space(5.0);
                    
                    ui.label(RichText::new("Serial (S/)").strong());
                    ui.label("  Connect to COM ports with configurable baud rate,");
                    ui.label("  data bits, parity, stop bits, and flow control.");
                    ui.add_space(5.0);
                    
                    ui.label(RichText::new("TCP (@)").strong());
                    ui.label("  Raw TCP socket connection to host:port.");
                    ui.add_space(5.0);
                    
                    ui.label(RichText::new("Telnet (T>)").strong());
                    ui.label("  Telnet protocol with option negotiation.");
                    ui.add_space(5.0);
                    
                    ui.label(RichText::new("SSH (#)").strong());
                    ui.label("  Secure Shell with password or key authentication.");
                    ui.label("  Supports SFTP, jump proxy, port forwarding.");
                    ui.add_space(5.0);
                    
                    ui.label(RichText::new("Bluetooth LE (B*)").strong());
                    ui.label("  BLE GATT connections, Nordic UART Service (NUS).");
                    
                    ui.add_space(15.0);
                    ui.heading("Profiles");
                    ui.add_space(5.0);
                    ui.label("- Double-click a profile to connect");
                    ui.label("- Filter profiles by type (All, Serial, SSH, etc.)");
                    ui.label("- Profiles are sorted by usage frequency");
                    ui.label("- Star icon marks favorites");
                    
                    ui.add_space(15.0);
                    ui.heading("Commands");
                    ui.add_space(5.0);
                    ui.label("- Commands you type are saved per profile");
                    ui.label("- Double-click to insert into command line");
                    ui.label("- Most used commands appear first");
                    
                    ui.add_space(15.0);
                    ui.heading("Macros (M1-M24)");
                    ui.add_space(5.0);
                    ui.label("- Click macro buttons to send commands");
                    ui.label("- Right-click to edit macro content");
                    ui.label("- Use F1-F12 / Shift+F1-F12 as shortcuts");
                    
                    ui.add_space(15.0);
                    ui.heading("File Transfer");
                    ui.add_space(5.0);
                    ui.label("- Tools > File Transfer for XMODEM/YMODEM/ZMODEM");
                    ui.label("- SFTP available for SSH connections");
                    
                    ui.add_space(15.0);
                    ui.heading("More Information");
                    ui.add_space(5.0);
                    ui.label("Full documentation: docs/USER_GUIDE.md");
                    ui.label("Feature roadmap: docs/ROADMAP.md");
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show settings dialog
    fn show_settings_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Application Settings");
                ui.add_space(15.0);
                
                egui::Grid::new("settings_grid").num_columns(2).spacing([40.0, 10.0]).show(ui, |ui| {
                    // Theme
                    ui.label("Theme:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.theme == AppTheme::Dark, "Dark").clicked() {
                            self.theme = AppTheme::Dark;
                        }
                        if ui.selectable_label(self.theme == AppTheme::Light, "Light").clicked() {
                            self.theme = AppTheme::Light;
                        }
                    });
                    ui.end_row();
                    
                    // Language
                    ui.label("Language:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.language == Language::English, "English").clicked() {
                            self.language = Language::English;
                            set_locale(Locale::English);
                            self.language_changed = true;
                        }
                        if ui.selectable_label(self.language == Language::Hungarian, "Magyar").clicked() {
                            self.language = Language::Hungarian;
                            set_locale(Locale::Hungarian);
                            self.language_changed = true;
                        }
                    });
                    ui.end_row();
                    
                    // Show side panel
                    ui.label("Side Panel:");
                    ui.checkbox(&mut self.show_side_panel, "Show");
                    ui.end_row();
                    
                    // Show macros bar
                    ui.label("Macros Bar:");
                    ui.checkbox(&mut self.show_macros_bar, "Show");
                    ui.end_row();
                });
                
                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);
                
                ui.heading("Default Connection Settings");
                ui.add_space(10.0);
                
                egui::Grid::new("default_conn_grid").num_columns(2).spacing([40.0, 8.0]).show(ui, |ui| {
                    ui.label("Default Baud Rate:");
                    ui.text_edit_singleline(&mut self.serial_settings.baud_rate);
                    ui.end_row();
                    
                    ui.label("Default SSH Port:");
                    ui.text_edit_singleline(&mut self.ssh_settings.port);
                    ui.end_row();
                    
                    ui.label("Default TCP Port:");
                    ui.text_edit_singleline(&mut self.tcp_settings.port);
                    ui.end_row();
                });
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show device simulator dialog
    fn show_device_simulator_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Device Simulator")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([550.0, 450.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Virtual Device Simulator");
                ui.add_space(10.0);
                ui.label("Create virtual devices for testing communication protocols.");
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("New Virtual Device").strong());
                    ui.add_space(5.0);
                    
                    egui::Grid::new("sim_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.label("Device Type:");
                        egui::ComboBox::from_id_salt("sim_type")
                            .selected_text("Echo Server")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(true, "Echo Server");
                                ui.selectable_label(false, "Modbus Slave");
                                ui.selectable_label(false, "Custom Script");
                            });
                        ui.end_row();
                        
                        ui.label("Transport:");
                        egui::ComboBox::from_id_salt("sim_transport")
                            .selected_text("TCP Server")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(true, "TCP Server");
                                ui.selectable_label(false, "Serial (PTY)");
                                ui.selectable_label(false, "Named Pipe");
                            });
                        ui.end_row();
                        
                        ui.label("Port:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("9999")).desired_width(100.0));
                        ui.end_row();
                    });
                    
                    ui.add_space(10.0);
                    if ui.button("Start Simulator").clicked() {
                        self.status_message = "Simulator started on port 9999".to_string();
                    }
                });
                
                ui.add_space(15.0);
                ui.heading("Running Simulators");
                ui.label("(None running)");
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show session replay dialog
    fn show_session_replay_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Session Replay")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Session Recording & Replay");
                ui.add_space(10.0);
                ui.label("Record communication sessions and replay them for testing/debugging.");
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Recording").strong());
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("Start Recording").clicked() {
                            self.status_message = "Recording started...".to_string();
                        }
                        if ui.button("Stop Recording").clicked() {
                            self.status_message = "Recording stopped".to_string();
                        }
                    });
                });
                
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Playback").strong());
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        egui::ComboBox::from_id_salt("replay_speed")
                            .selected_text("1x")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(false, "0.5x");
                                ui.selectable_label(true, "1x");
                                ui.selectable_label(false, "2x");
                                ui.selectable_label(false, "5x");
                                ui.selectable_label(false, "10x");
                            });
                    });
                    
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("[<<]").clicked() {}
                        if ui.button("Play").clicked() {
                            self.status_message = "Playing session...".to_string();
                        }
                        if ui.button("Pause").clicked() {}
                        if ui.button("[>>]").clicked() {}
                    });
                });
                
                ui.add_space(15.0);
                ui.heading("Saved Sessions");
                ui.label("(No saved sessions)");
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show fuzzing dialog
    fn show_fuzzing_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Fuzzing / Testing")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([550.0, 450.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Protocol Fuzzing & Robustness Testing");
                ui.add_space(10.0);
                ui.label("Test protocol implementations with randomized/boundary value inputs.");
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Fuzzing Options").strong());
                    ui.add_space(5.0);
                    
                    egui::Grid::new("fuzz_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.label("Target:");
                        egui::ComboBox::from_id_salt("fuzz_target")
                            .selected_text("Current Session")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(true, "Current Session");
                                ui.selectable_label(false, "All Sessions");
                            });
                        ui.end_row();
                        
                        ui.label("Fuzz Type:");
                        egui::ComboBox::from_id_salt("fuzz_type")
                            .selected_text("Packet Fuzzing")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(true, "Packet Fuzzing");
                                ui.selectable_label(false, "Timing Fuzzing");
                                ui.selectable_label(false, "Boundary Values");
                                ui.selectable_label(false, "Random Data");
                            });
                        ui.end_row();
                        
                        ui.label("Iterations:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("1000")).desired_width(100.0));
                        ui.end_row();
                        
                        ui.label("Timeout (ms):");
                        ui.add(egui::TextEdit::singleline(&mut String::from("5000")).desired_width(100.0));
                        ui.end_row();
                    });
                    
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Start Fuzzing").clicked() {
                            self.status_message = "Fuzzing started...".to_string();
                        }
                        if ui.button("Stop").clicked() {
                            self.status_message = "Fuzzing stopped".to_string();
                        }
                    });
                });
                
                ui.add_space(15.0);
                ui.heading("Results");
                ui.label("(No results yet)");
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show experiment mode dialog
    fn show_experiment_mode_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Experiment Mode")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([550.0, 450.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Parameter Sweep & Analysis");
                ui.add_space(10.0);
                ui.label("Run experiments with varying parameters and analyze results.");
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Parameter Configuration").strong());
                    ui.add_space(5.0);
                    
                    egui::Grid::new("exp_grid").num_columns(4).spacing([10.0, 8.0]).show(ui, |ui| {
                        ui.label("Parameter");
                        ui.label("Start");
                        ui.label("End");
                        ui.label("Step");
                        ui.end_row();
                        
                        ui.label("Baud Rate:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("9600")).desired_width(60.0));
                        ui.add(egui::TextEdit::singleline(&mut String::from("115200")).desired_width(60.0));
                        ui.add(egui::TextEdit::singleline(&mut String::from("9600")).desired_width(60.0));
                        ui.end_row();
                        
                        ui.label("Packet Size:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("8")).desired_width(60.0));
                        ui.add(egui::TextEdit::singleline(&mut String::from("256")).desired_width(60.0));
                        ui.add(egui::TextEdit::singleline(&mut String::from("8")).desired_width(60.0));
                        ui.end_row();
                    });
                    
                    ui.add_space(10.0);
                    if ui.button("Run Experiment").clicked() {
                        self.status_message = "Experiment running...".to_string();
                    }
                });
                
                ui.add_space(15.0);
                ui.heading("Results");
                ui.label("(No experiment results yet)");
                ui.label("Results will show a heatmap of parameter combinations.");
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show deterministic mode dialog
    fn show_deterministic_mode_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Deterministic Mode")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 350.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Reproducible Test Runs");
                ui.add_space(10.0);
                ui.label("Configure deterministic execution for reproducible testing.");
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Deterministic Settings").strong());
                    ui.add_space(5.0);
                    
                    egui::Grid::new("det_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.label("Random Seed:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("12345")).desired_width(100.0));
                        ui.end_row();
                        
                        ui.label("Timing Mode:");
                        egui::ComboBox::from_id_salt("timing_mode")
                            .selected_text("Normalized")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(true, "Normalized");
                                ui.selectable_label(false, "Fixed Delay");
                                ui.selectable_label(false, "Real-time");
                            });
                        ui.end_row();
                        
                        ui.label("Fixed Delay (ms):");
                        ui.add(egui::TextEdit::singleline(&mut String::from("10")).desired_width(100.0));
                        ui.end_row();
                    });
                    
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Enable Deterministic Mode").clicked() {
                            self.status_message = "Deterministic mode enabled".to_string();
                        }
                        if ui.button("Disable").clicked() {
                            self.status_message = "Deterministic mode disabled".to_string();
                        }
                    });
                });
                
                ui.add_space(15.0);
                ui.label("With deterministic mode enabled:");
                ui.label("- Same input always produces same output");
                ui.label("- Timing jitter is normalized");
                ui.label("- Random operations use fixed seed");
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show explain/debug dialog
    fn show_explain_debug_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Explain / Debug")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([550.0, 450.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Root Cause Analysis");
                ui.add_space(10.0);
                ui.label("Get hints about why communication might be failing.");
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Recent Issues").strong());
                    ui.add_space(5.0);
                    
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        ui.label("(No issues detected)");
                    });
                });
                
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Diagnostic Rules").strong());
                    ui.add_space(5.0);
                    
                    ui.label("The following rules are checked:");
                    ui.label("  - Baud rate mismatch detection");
                    ui.label("  - Parity/stop bit errors");
                    ui.label("  - Flow control issues");
                    ui.label("  - Protocol framing errors");
                    ui.label("  - Timeout patterns");
                    ui.label("  - CRC/checksum failures");
                });
                
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Suggestions").strong());
                    ui.add_space(5.0);
                    ui.label("No suggestions at this time.");
                    ui.label("Connect to a device to enable diagnostics.");
                });
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Run Diagnostics").clicked() {
                        self.status_message = "Running diagnostics...".to_string();
                    }
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show Modbus monitor dialog
    fn show_modbus_monitor_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Modbus Monitor")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Modbus RTU/TCP Monitor");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Mode:");
                    egui::ComboBox::from_id_salt("modbus_mode")
                        .selected_text("RTU")
                        .show_ui(ui, |ui| {
                            ui.selectable_label(true, "RTU");
                            ui.selectable_label(false, "TCP");
                            ui.selectable_label(false, "ASCII");
                        });
                    
                    ui.add_space(20.0);
                    ui.label("Slave ID:");
                    ui.add(egui::TextEdit::singleline(&mut String::from("1")).desired_width(40.0));
                });
                
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Register Read").strong());
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Function:");
                        egui::ComboBox::from_id_salt("modbus_func")
                            .selected_text("03 - Read Holding Registers")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(false, "01 - Read Coils");
                                ui.selectable_label(false, "02 - Read Discrete Inputs");
                                ui.selectable_label(true, "03 - Read Holding Registers");
                                ui.selectable_label(false, "04 - Read Input Registers");
                            });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Start Addr:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("0")).desired_width(60.0));
                        ui.label("Count:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("10")).desired_width(60.0));
                        if ui.button("Read").clicked() {
                            self.status_message = "Modbus read sent".to_string();
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Register Values").strong());
                    ui.add_space(5.0);
                    
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        egui::Grid::new("modbus_regs").num_columns(5).striped(true).show(ui, |ui| {
                            ui.label(RichText::new("Addr").strong());
                            ui.label(RichText::new("Hex").strong());
                            ui.label(RichText::new("Uint16").strong());
                            ui.label(RichText::new("Int16").strong());
                            ui.label(RichText::new("Binary").strong());
                            ui.end_row();
                            
                            for i in 0..10 {
                                ui.label(format!("{}", i));
                                ui.label("0x0000");
                                ui.label("0");
                                ui.label("0");
                                ui.label("0000000000000000");
                                ui.end_row();
                            }
                        });
                    });
                });
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }
    
    /// Show bridge dialog
    fn show_bridge_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new("Serial-TCP Bridge")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("Serial to TCP Bridge");
                ui.add_space(10.0);
                ui.label("Create a bridge between serial port and TCP connection.");
                ui.add_space(15.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Serial Side").strong());
                    ui.add_space(5.0);
                    
                    egui::Grid::new("bridge_serial").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.label("Port:");
                        egui::ComboBox::from_id_salt("bridge_port")
                            .selected_text("COM1")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(true, "COM1");
                                ui.selectable_label(false, "COM2");
                                ui.selectable_label(false, "COM3");
                            });
                        ui.end_row();
                        
                        ui.label("Baud Rate:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("115200")).desired_width(100.0));
                        ui.end_row();
                    });
                });
                
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("TCP Side").strong());
                    ui.add_space(5.0);
                    
                    egui::Grid::new("bridge_tcp").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.label("Mode:");
                        egui::ComboBox::from_id_salt("bridge_mode")
                            .selected_text("TCP Server")
                            .show_ui(ui, |ui| {
                                ui.selectable_label(true, "TCP Server");
                                ui.selectable_label(false, "TCP Client");
                            });
                        ui.end_row();
                        
                        ui.label("Port:");
                        ui.add(egui::TextEdit::singleline(&mut String::from("9999")).desired_width(100.0));
                        ui.end_row();
                    });
                });
                
                ui.add_space(15.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Start Bridge").clicked() {
                        self.status_message = "Bridge started on port 9999".to_string();
                    }
                    if ui.button("Stop Bridge").clicked() {
                        self.status_message = "Bridge stopped".to_string();
                    }
                });
                
                ui.add_space(10.0);
                ui.label("Status: Not running");
                ui.label("Bytes: TX: 0  RX: 0");
                
                ui.add_space(20.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("   Close   ").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    fn show_virtual_com_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new(t!("menu.virtual_com"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([450.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(t!("menu.virtual_com"));
                ui.add_space(10.0);
                
                #[cfg(windows)]
                {
                    ui.label("Create a virtual COM port pair using Named Pipes.");
                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.label(RichText::new("Virtual Port Settings").strong());
                        egui::Grid::new("vcom_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                            ui.label("Pipe Name:");
                            ui.add(egui::TextEdit::singleline(&mut self.virtual_com_pipe_name).desired_width(200.0));
                            ui.end_row();
                        });
                    });
                }
                
                #[cfg(unix)]
                {
                    ui.label("Create a virtual serial port using PTY pairs.");
                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.label(RichText::new("PTY Settings").strong());
                        ui.label("A PTY pair will be created automatically.");
                        ui.label("Master: /dev/pts/X");
                        ui.label("Slave: /dev/pts/Y");
                    });
                }
                
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        self.status_message = "Virtual COM port created".to_string();
                    }
                    if ui.button("Close").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    fn show_ssh_keygen_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new(t!("menu.generate_ssh_key"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(t!("menu.generate_ssh_key"));
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Key Type").strong());
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.ssh_keygen_type, "ed25519".to_string(), "Ed25519 (recommended)");
                        ui.selectable_value(&mut self.ssh_keygen_type, "rsa".to_string(), "RSA 4096");
                        ui.selectable_value(&mut self.ssh_keygen_type, "ecdsa".to_string(), "ECDSA");
                    });
                });
                
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Key Settings").strong());
                    egui::Grid::new("keygen_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.label("Comment:");
                        ui.add(egui::TextEdit::singleline(&mut self.ssh_keygen_comment).desired_width(250.0));
                        ui.end_row();
                        
                        ui.label("Passphrase:");
                        ui.add(egui::TextEdit::singleline(&mut self.ssh_keygen_passphrase).password(true).desired_width(250.0));
                        ui.end_row();
                        
                        ui.label("Save Location:");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.ssh_keygen_path).desired_width(200.0));
                            if ui.button("Browse...").clicked() {
                                // Would open file dialog
                            }
                        });
                        ui.end_row();
                    });
                });
                
                ui.add_space(15.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Generate Key").clicked() {
                        self.status_message = format!("Generating {} key...", self.ssh_keygen_type);
                        // TODO: Actually generate key using vault module
                    }
                    if ui.button("Close").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    fn show_file_transfer_send_dialog(&mut self, ctx: &egui::Context, protocol: String) {
        let mut open = true;
        let title = format!("{} - {}", protocol.to_uppercase(), t!("menu.send_file"));
        egui::Window::new(&title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(&title);
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new(t!("menu.select_file")).strong());
                    ui.horizontal(|ui| {
                        let file_display = if self.file_transfer_path.is_empty() {
                            t!("menu.no_file_selected").to_string()
                        } else {
                            self.file_transfer_path.clone()
                        };
                        ui.add(egui::TextEdit::singleline(&mut self.file_transfer_path).desired_width(350.0));
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.file_transfer_path = path.display().to_string();
                            }
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                if !self.file_transfer_path.is_empty() {
                    if let Ok(metadata) = std::fs::metadata(&self.file_transfer_path) {
                        ui.label(format!("File size: {} bytes", metadata.len()));
                    }
                }
                
                ui.add_space(15.0);
                
                // Progress bar
                ui.add(egui::ProgressBar::new(self.file_transfer_progress).show_percentage());
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    let can_send = !self.file_transfer_path.is_empty() && 
                        self.tabs.active_tab().map(|t| t.state == ConnectionState::Connected).unwrap_or(false);
                    
                    ui.add_enabled_ui(can_send, |ui| {
                        if ui.button(format!("Send via {}", protocol.to_uppercase())).clicked() {
                            self.status_message = format!("Sending file via {}...", protocol.to_uppercase());
                            // TODO: Start actual file transfer
                        }
                    });
                    
                    if ui.button(t!("btn.cancel")).clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
                
                if !self.tabs.active_tab().map(|t| t.state == ConnectionState::Connected).unwrap_or(false) {
                    ui.add_space(5.0);
                    ui.label(RichText::new("Note: Connect to a device first to send files.").color(Color32::YELLOW));
                }
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    fn show_file_transfer_receive_dialog(&mut self, ctx: &egui::Context, protocol: String) {
        let mut open = true;
        let title = format!("{} - {}", protocol.to_uppercase(), t!("menu.receive_file"));
        egui::Window::new(&title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(&title);
                ui.add_space(10.0);
                
                ui.group(|ui| {
                    ui.label(RichText::new("Save Location").strong());
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.file_transfer_save_path).desired_width(350.0));
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                self.file_transfer_save_path = path.display().to_string();
                            }
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                // Progress bar
                ui.add(egui::ProgressBar::new(self.file_transfer_progress).show_percentage());
                ui.label(format!("Status: {}", self.file_transfer_status));
                
                ui.add_space(15.0);
                
                ui.horizontal(|ui| {
                    let is_connected = self.tabs.active_tab().map(|t| t.state == ConnectionState::Connected).unwrap_or(false);
                    
                    ui.add_enabled_ui(is_connected, |ui| {
                        if ui.button(format!("Start {} Receive", protocol.to_uppercase())).clicked() {
                            self.file_transfer_status = format!("Waiting for {} transfer...", protocol.to_uppercase());
                            // TODO: Start actual file receive
                        }
                    });
                    
                    if ui.button(t!("btn.cancel")).clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
                
                if !self.tabs.active_tab().map(|t| t.state == ConnectionState::Connected).unwrap_or(false) {
                    ui.add_space(5.0);
                    ui.label(RichText::new("Note: Connect to a device first to receive files.").color(Color32::YELLOW));
                }
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    fn show_protocol_dsl_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new(t!("menu.protocol_dsl"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([700.0, 500.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(t!("menu.protocol_dsl"));
                ui.add_space(10.0);
                ui.label("Define custom protocols using YAML or JSON syntax.");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("New Protocol").clicked() {
                        self.protocol_dsl_content = r#"# Example Protocol Definition
name: MyProtocol
version: 1.0
framing: length_prefixed

fields:
  - name: header
    type: u8
    value: 0xAA
  - name: length
    type: u16
    endian: little
  - name: command
    type: u8
  - name: payload
    type: bytes
    length_field: length
  - name: checksum
    type: u16
    algorithm: crc16_modbus
"#.to_string();
                    }
                    if ui.button("Load...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Protocol files", &["yaml", "yml", "json"])
                            .pick_file() 
                        {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                self.protocol_dsl_content = content;
                            }
                        }
                    }
                    if ui.button("Save...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("YAML", &["yaml", "yml"])
                            .save_file() 
                        {
                            let _ = std::fs::write(&path, &self.protocol_dsl_content);
                            self.status_message = format!("Saved to {}", path.display());
                        }
                    }
                });
                
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.protocol_dsl_content)
                            .font(FontId::monospace(12.0))
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                    );
                });
                
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Validate").clicked() {
                        self.status_message = "Protocol definition validated successfully".to_string();
                    }
                    if ui.button("Close").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    fn show_triggers_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Window::new(t!("menu.triggers_auto"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 450.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(t!("menu.triggers_auto"));
                ui.add_space(10.0);
                ui.label("Configure pattern-based triggers with automatic responses.");
                ui.add_space(10.0);
                
                // Trigger list
                ui.group(|ui| {
                    ui.label(RichText::new("Active Triggers").strong());
                    ui.add_space(5.0);
                    
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for (i, trigger) in self.triggers.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut true, "");
                                ui.label(&trigger.name);
                                ui.label(format!("Pattern: {}", trigger.pattern));
                                if !trigger.response.is_empty() {
                                    ui.label(format!("-> {}", trigger.response));
                                }
                            });
                        }
                        if self.triggers.is_empty() {
                            ui.label("No triggers defined. Add one below.");
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                // Add new trigger
                ui.group(|ui| {
                    ui.label(RichText::new("Add New Trigger").strong());
                    ui.add_space(5.0);
                    
                    egui::Grid::new("trigger_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.label("Name:");
                        ui.add(egui::TextEdit::singleline(&mut self.new_trigger_name).desired_width(200.0));
                        ui.end_row();
                        
                        ui.label("Pattern (regex):");
                        ui.add(egui::TextEdit::singleline(&mut self.new_trigger_pattern).desired_width(200.0));
                        ui.end_row();
                        
                        ui.label("Auto-response:");
                        ui.add(egui::TextEdit::singleline(&mut self.new_trigger_response).desired_width(200.0));
                        ui.end_row();
                    });
                    
                    ui.add_space(5.0);
                    if ui.button("Add Trigger").clicked() {
                        if !self.new_trigger_name.is_empty() && !self.new_trigger_pattern.is_empty() {
                            self.triggers.push(TriggerEntry {
                                name: self.new_trigger_name.clone(),
                                pattern: self.new_trigger_pattern.clone(),
                                response: self.new_trigger_response.clone(),
                                enabled: true,
                            });
                            self.new_trigger_name.clear();
                            self.new_trigger_pattern.clear();
                            self.new_trigger_response.clear();
                        }
                    }
                });
                
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        self.status_message = "Triggers updated".to_string();
                    }
                    if ui.button("Close").clicked() {
                        self.current_dialog = DialogType::None;
                    }
                });
            });
        if !open {
            self.current_dialog = DialogType::None;
        }
    }

    /// Connect serial port
    fn connect_serial(&mut self) {
        let port = self.serial_settings.port.clone();
        let baud: u32 = self.serial_settings.baud_rate.parse().unwrap_or(115200);

        // Create new tab
        let mut tab = SessionTab::new(&format!("{} @ {}", port, baud), ConnectionType::Serial);
        tab.state = ConnectionState::Connecting;
        tab.connection_info = format!("{} @ {} baud", port, baud);
        tab.add_line(&format!("Connecting to {} @ {} baud...", port, baud), false);
        // Set profile_id if connecting from a profile
        tab.profile_id = self.active_profile_id.clone();

        let (tx_to_gui, rx_from_conn) = mpsc::channel::<ConnectionMessage>();
        let (tx_to_conn, rx_from_gui) = mpsc::channel::<ConnectionCommand>();

        tab.rx = Some(rx_from_conn);
        tab.tx = Some(tx_to_conn);

        let idx = self.tabs.add_tab(tab);
        self.tabs.set_active(idx);
        // Clear active profile ID after setting it on tab
        self.active_profile_id = None;

        self.status_message = format!("Connecting: {}", port);

        let port_clone = port;
        // Spawn connection thread
        thread::spawn(move || {
            let port = port_clone;
            match serialport::new(&port, baud)
                .timeout(std::time::Duration::from_millis(100))
                .open()
            {
                Ok(mut serial_port) => {
                    let _ = tx_to_gui.send(ConnectionMessage::Connected);

                    loop {
                        match rx_from_gui.try_recv() {
                            Ok(ConnectionCommand::Send(data)) => {
                                use std::io::Write;
                                if let Err(e) = serial_port.write_all(&data) {
                                    let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                                    break;
                                }
                            }
                            Ok(ConnectionCommand::Disconnect) => break,
                            Err(mpsc::TryRecvError::Disconnected) => break,
                            Err(mpsc::TryRecvError::Empty) => {}
                        }

                        let mut buf = [0u8; 1024];
                        use std::io::Read;
                        match serial_port.read(&mut buf) {
                            Ok(n) if n > 0 => {
                                let _ = tx_to_gui.send(ConnectionMessage::Data(buf[..n].to_vec()));
                            }
                            Ok(_) => {}
                            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                            Err(e) => {
                                let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                                break;
                            }
                        }

                        thread::sleep(std::time::Duration::from_millis(10));
                    }

                    let _ = tx_to_gui.send(ConnectionMessage::Disconnected);
                }
                Err(e) => {
                    let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                }
            }
        });
    }

    /// Connect TCP
    fn connect_tcp(&mut self) {
        let host = self.tcp_settings.host.clone();
        let port: u16 = self.tcp_settings.port.parse().unwrap_or(23);

        let mut tab = SessionTab::new(&format!("{}:{}", host, port), ConnectionType::Tcp);
        tab.state = ConnectionState::Connecting;
        tab.connection_info = format!("{}:{}", host, port);
        tab.add_line(&format!("Connecting to {}:{}...", host, port), false);
        tab.profile_id = self.active_profile_id.clone();

        let (tx_to_gui, rx_from_conn) = mpsc::channel::<ConnectionMessage>();
        let (tx_to_conn, rx_from_gui) = mpsc::channel::<ConnectionCommand>();

        tab.rx = Some(rx_from_conn);
        tab.tx = Some(tx_to_conn);

        let idx = self.tabs.add_tab(tab);
        self.tabs.set_active(idx);
        self.active_profile_id = None;

        self.status_message = format!("Connecting: {}:{}", host, port);

        let host_clone = host;
        thread::spawn(move || {
            let host = host_clone;
            match std::net::TcpStream::connect_timeout(
                &format!("{}:{}", host, port).parse().unwrap(),
                std::time::Duration::from_secs(10)
            ) {
                Ok(stream) => {
                    let _ = stream.set_nonblocking(true);
                    let mut stream = stream;
                    let _ = tx_to_gui.send(ConnectionMessage::Connected);

                    loop {
                        match rx_from_gui.try_recv() {
                            Ok(ConnectionCommand::Send(data)) => {
                                use std::io::Write;
                                if let Err(e) = stream.write_all(&data) {
                                    let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                                    break;
                                }
                            }
                            Ok(ConnectionCommand::Disconnect) => break,
                            Err(mpsc::TryRecvError::Disconnected) => break,
                            Err(mpsc::TryRecvError::Empty) => {}
                        }

                        let mut buf = [0u8; 4096];
                        use std::io::Read;
                        match stream.read(&mut buf) {
                            Ok(0) => {
                                let _ = tx_to_gui.send(ConnectionMessage::Disconnected);
                                break;
                            }
                            Ok(n) => {
                                let _ = tx_to_gui.send(ConnectionMessage::Data(buf[..n].to_vec()));
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                            Err(e) => {
                                let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                                break;
                            }
                        }

                        thread::sleep(std::time::Duration::from_millis(10));
                    }

                    let _ = tx_to_gui.send(ConnectionMessage::Disconnected);
                }
                Err(e) => {
                    let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                }
            }
        });
    }

    /// Connect Telnet
    fn connect_telnet(&mut self) {
        if self.tcp_settings.port.is_empty() {
            self.tcp_settings.port = "23".to_string();
        }
        self.connect_tcp();
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.conn_type = ConnectionType::Telnet;
        }
    }

    /// Connect SSH
    fn connect_ssh(&mut self) {
        let host = self.ssh_settings.host.clone();
        let port: u16 = self.ssh_settings.port.parse().unwrap_or(22);
        let username = self.ssh_settings.username.clone();
        let password = self.ssh_settings.password.clone();
        let use_key = self.ssh_settings.use_key;
        let key_path = self.ssh_settings.key_path.clone();

        let mut tab = SessionTab::new(&format!("{}@{}", username, host), ConnectionType::Ssh);
        tab.state = ConnectionState::Connecting;
        tab.connection_info = format!("ssh://{}@{}:{}", username, host, port);
        tab.add_line(&format!("Connecting to {}@{}:{} (SSH)...", username, host, port), false);
        tab.profile_id = self.active_profile_id.clone();

        let (tx_to_gui, rx_from_conn) = mpsc::channel::<ConnectionMessage>();
        let (tx_to_conn, rx_from_gui) = mpsc::channel::<ConnectionCommand>();

        tab.rx = Some(rx_from_conn);
        tab.tx = Some(tx_to_conn);

        let idx = self.tabs.add_tab(tab);
        self.tabs.set_active(idx);
        self.active_profile_id = None;

        self.status_message = format!("Connecting: {}@{}", username, host);

        let host_clone = host;
        let username_clone = username;
        thread::spawn(move || {
            let host = host_clone;
            let username = username_clone;
            let tcp = match std::net::TcpStream::connect_timeout(
                &format!("{}:{}", host, port).parse().unwrap(),
                std::time::Duration::from_secs(10)
            ) {
                Ok(tcp) => tcp,
                Err(e) => {
                    let _ = tx_to_gui.send(ConnectionMessage::Error(format!("TCP: {}", e)));
                    return;
                }
            };

            let mut session = match ssh2::Session::new() {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx_to_gui.send(ConnectionMessage::Error(format!("SSH: {}", e)));
                    return;
                }
            };

            session.set_timeout(30000);
            session.set_tcp_stream(tcp);

            if let Err(e) = session.handshake() {
                let _ = tx_to_gui.send(ConnectionMessage::Error(format!("Handshake: {}", e)));
                return;
            }

            // Authenticate
            let auth_result = if use_key && !key_path.is_empty() {
                session.userauth_pubkey_file(&username, None, std::path::Path::new(&key_path), None)
            } else if !password.is_empty() {
                session.userauth_password(&username, &password)
            } else {
                match session.agent() {
                    Ok(mut agent) => {
                        let _ = agent.connect();
                        let _ = agent.list_identities();
                        let identities: Vec<_> = agent.identities().unwrap_or_default();
                        let mut authed = false;
                        for identity in identities {
                            if agent.userauth(&username, &identity).is_ok() {
                                authed = true;
                                break;
                            }
                        }
                        if authed { Ok(()) } else { Err(ssh2::Error::from_errno(ssh2::ErrorCode::Session(-1))) }
                    }
                    Err(e) => Err(e)
                }
            };

            if let Err(e) = auth_result {
                let _ = tx_to_gui.send(ConnectionMessage::Error(format!("Auth: {}", e)));
                return;
            }

            if !session.authenticated() {
                let _ = tx_to_gui.send(ConnectionMessage::Error("Authentication failed".to_string()));
                return;
            }

            let mut channel = match session.channel_session() {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx_to_gui.send(ConnectionMessage::Error(format!("Channel: {}", e)));
                    return;
                }
            };

            if let Err(e) = channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0))) {
                let _ = tx_to_gui.send(ConnectionMessage::Error(format!("PTY: {}", e)));
                return;
            }

            if let Err(e) = channel.shell() {
                let _ = tx_to_gui.send(ConnectionMessage::Error(format!("Shell: {}", e)));
                return;
            }

            session.set_blocking(false);
            let _ = tx_to_gui.send(ConnectionMessage::Connected);

            loop {
                match rx_from_gui.try_recv() {
                    Ok(ConnectionCommand::Send(data)) => {
                        use std::io::Write;
                        session.set_blocking(true);
                        if let Err(e) = channel.write_all(&data) {
                            let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                            break;
                        }
                        let _ = channel.flush();
                        session.set_blocking(false);
                    }
                    Ok(ConnectionCommand::Disconnect) => break,
                    Err(mpsc::TryRecvError::Disconnected) => break,
                    Err(mpsc::TryRecvError::Empty) => {}
                }

                let mut buf = [0u8; 4096];
                use std::io::Read;
                match channel.read(&mut buf) {
                    Ok(0) => {
                        if channel.eof() {
                            let _ = tx_to_gui.send(ConnectionMessage::Disconnected);
                            break;
                        }
                    }
                    Ok(n) => {
                        let _ = tx_to_gui.send(ConnectionMessage::Data(buf[..n].to_vec()));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        if !e.to_string().contains("EAGAIN") {
                            let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                            break;
                        }
                    }
                }

                thread::sleep(std::time::Duration::from_millis(10));
            }

            let _ = channel.close();
            let _ = channel.wait_close();
            let _ = session.disconnect(None, "Goodbye", None);
            let _ = tx_to_gui.send(ConnectionMessage::Disconnected);
        });
    }

    /// Connect to Bluetooth device
    fn connect_bluetooth(&mut self) {
        let device = self.bluetooth_settings.device.clone();
        let service_uuid = self.bluetooth_settings.service_uuid.clone();
        let tx_uuid = self.bluetooth_settings.tx_uuid.clone();
        let rx_uuid = self.bluetooth_settings.rx_uuid.clone();

        let mut tab = SessionTab::new(&format!("BLE: {}", device), ConnectionType::Bluetooth);
        tab.state = ConnectionState::Connecting;
        tab.connection_info = format!("ble://{}", device);
        tab.add_line(&format!("Connecting to BLE device: {}...", device), false);
        tab.add_line(&format!("Service: {}", service_uuid), false);
        tab.profile_id = self.active_profile_id.clone();

        let (tx_to_gui, rx_from_conn) = mpsc::channel::<ConnectionMessage>();
        let (tx_to_conn, rx_from_gui) = mpsc::channel::<ConnectionCommand>();

        tab.rx = Some(rx_from_conn);
        tab.tx = Some(tx_to_conn);

        let idx = self.tabs.add_tab(tab);
        self.tabs.set_active(idx);
        self.active_profile_id = None;

        self.status_message = format!("Connecting: BLE {}", device);

        // Spawn async Bluetooth connection
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                use termicon_core::core::transport::{BluetoothConfig, BleServiceConfig, BluetoothType, BluetoothTransport};
                use termicon_core::core::transport::TransportTrait;

                let config = BluetoothConfig {
                    device: device.clone(),
                    bt_type: BluetoothType::Ble,
                    ble_service: BleServiceConfig {
                        service_uuid,
                        tx_characteristic: tx_uuid,
                        rx_characteristic: rx_uuid,
                    },
                    timeout_secs: 10,
                    auto_reconnect: false,
                    mtu: 512,
                };

                let mut transport = match BluetoothTransport::new(config).await {
                    Ok(t) => t,
                    Err(e) => {
                        let _ = tx_to_gui.send(ConnectionMessage::Error(format!("BLE init: {}", e)));
                        return;
                    }
                };

                if let Err(e) = transport.connect().await {
                    let _ = tx_to_gui.send(ConnectionMessage::Error(format!("BLE connect: {}", e)));
                    return;
                }

                let _ = tx_to_gui.send(ConnectionMessage::Connected);

                let mut rx = transport.subscribe();

                loop {
                    // Check for commands from GUI (non-blocking)
                    match rx_from_gui.try_recv() {
                        Ok(ConnectionCommand::Send(data)) => {
                            if let Err(e) = transport.send(&data).await {
                                let _ = tx_to_gui.send(ConnectionMessage::Error(e.to_string()));
                                break;
                            }
                        }
                        Ok(ConnectionCommand::Disconnect) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                    }

                    // Check for data from BLE device (with timeout)
                    match tokio::time::timeout(
                        tokio::time::Duration::from_millis(10),
                        rx.recv()
                    ).await {
                        Ok(Ok(data)) => {
                            let _ = tx_to_gui.send(ConnectionMessage::Data(data.to_vec()));
                        }
                        Ok(Err(_)) => {
                            // Channel closed
                            break;
                        }
                        Err(_) => {
                            // Timeout, continue
                        }
                    }
                }

                let _ = transport.disconnect().await;
                let _ = tx_to_gui.send(ConnectionMessage::Disconnected);
            });
        });
    }
}

impl eframe::App for TermiconApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process pending insert command BEFORE rendering
        if let Some(cmd) = self.pending_insert_command.take() {
            if let Some(tab) = self.tabs.active_tab_mut() {
                tab.current_input = cmd;
            }
        }
        
        // If language changed, force a complete repaint
        if self.language_changed {
            self.language_changed = false;
            ctx.request_repaint();
        }
        
        // Process all tabs
        self.tabs.process_all();

        // Check for just_connected tabs and prompt to save profile
        if let Some(tab) = self.tabs.active_tab_mut() {
            if tab.just_connected && tab.profile_id.is_none() {
                tab.just_connected = false;
                // Set the pending profile type based on connection type
                self.pending_profile_type = Some(match tab.conn_type {
                    ConnectionType::Serial => ProfileType::Serial,
                    ConnectionType::Tcp => ProfileType::Tcp,
                    ConnectionType::Telnet => ProfileType::Telnet,
                    ConnectionType::Ssh => ProfileType::Ssh,
                    ConnectionType::Bluetooth => ProfileType::Bluetooth,
                });
                // Suggest a name based on connection info
                self.new_profile_name = tab.name.clone();
                // Show the save dialog
                self.current_dialog = DialogType::SaveProfile;
            }
        }

        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) {
            let new_tab = SessionTab::new("New Tab", ConnectionType::Serial);
            let idx = self.tabs.add_tab(new_tab);
            self.tabs.set_active(idx);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::W) && i.modifiers.ctrl) {
            if self.tabs.count() > 1 {
                self.tabs.remove_tab(self.tabs.active_index);
            }
        }

        // Menu bar
        let menu_bar_bg = if self.theme == AppTheme::Dark {
            Color32::from_rgb(28, 28, 32)
        } else {
            Color32::from_rgb(230, 230, 235)
        };
        egui::TopBottomPanel::top("menu_bar")
            .frame(egui::Frame::NONE.fill(menu_bar_bg))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                egui::menu::bar(ui, |ui| {
                    ui.menu_button(t!("menu.file"), |ui| {
                        if ui.button(format!("{} (Ctrl+T)", t!("menu.new_tab"))).clicked() {
                            let new_tab = SessionTab::new("New Tab", ConnectionType::Serial);
                            let idx = self.tabs.add_tab(new_tab);
                            self.tabs.set_active(idx);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(format!("{} (Ctrl+W)", t!("menu.close_tab"))).clicked() {
                            if self.tabs.count() > 1 {
                                self.tabs.remove_tab(self.tabs.active_index);
                            }
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(t!("menu.exit")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button(t!("menu.edit"), |ui| {
                        if ui.button(t!("menu.clear_terminal")).clicked() {
                            if let Some(tab) = self.tabs.active_tab_mut() {
                                tab.clear();
                            }
                            ui.close_menu();
                        }
                    });

                    ui.menu_button(t!("menu.view"), |ui| {
                        if let Some(tab) = self.tabs.active_tab_mut() {
                            ui.checkbox(&mut tab.show_timestamps, t!("menu.show_timestamps"));
                            ui.checkbox(&mut tab.show_hex, t!("menu.hex_view"));
                            ui.checkbox(&mut tab.local_echo, t!("menu.local_echo"));
                        }
                        ui.separator();
                        ui.checkbox(&mut self.show_side_panel, t!("menu.side_panel"));
                        ui.checkbox(&mut self.show_macros_bar, t!("menu.macros_bar"));
                    });

                    ui.menu_button(t!("menu.connection"), |ui| {
                        ui.menu_button(t!("menu.serial_port"), |ui| {
                            if ui.button(format!("{}...", t!("menu.connect"))).clicked() {
                                self.current_dialog = DialogType::Serial;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(t!("menu.bridge_tcp")).clicked() {
                                self.current_dialog = DialogType::Bridge;
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.virtual_com")).clicked() {
                                self.current_dialog = DialogType::VirtualCom;
                                ui.close_menu();
                            }
                        });
                        
                        ui.menu_button(t!("menu.network"), |ui| {
                            if ui.button(t!("menu.tcp_client")).clicked() {
                                self.current_dialog = DialogType::Tcp;
                                ui.close_menu();
                            }
                            if ui.button("Telnet...").clicked() {
                                self.current_dialog = DialogType::Telnet;
                                ui.close_menu();
                            }
                        });
                        
                        ui.menu_button("SSH", |ui| {
                            if ui.button(format!("SSH {}...", t!("menu.connect"))).clicked() {
                                self.current_dialog = DialogType::Ssh;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(t!("menu.sftp_browser")).clicked() {
                                // Switch to SFTP panel mode
                                self.side_panel_mode = SidePanelMode::Sftp;
                                self.show_side_panel = true;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(t!("menu.generate_ssh_key")).clicked() {
                                self.current_dialog = DialogType::SshKeyGen;
                                ui.close_menu();
                            }
                        });
                        
                        if ui.button(t!("menu.bluetooth_le")).clicked() {
                            self.current_dialog = DialogType::Bluetooth;
                            ui.close_menu();
                        }
                    });

                    ui.menu_button(t!("menu.tools"), |ui| {
                        ui.menu_button(t!("menu.file_transfer"), |ui| {
                            if ui.button(t!("menu.xmodem_send")).clicked() {
                                self.current_dialog = DialogType::FileTransferSend("xmodem".to_string());
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.xmodem_receive")).clicked() {
                                self.current_dialog = DialogType::FileTransferReceive("xmodem".to_string());
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(t!("menu.ymodem_send")).clicked() {
                                self.current_dialog = DialogType::FileTransferSend("ymodem".to_string());
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.ymodem_receive")).clicked() {
                                self.current_dialog = DialogType::FileTransferReceive("ymodem".to_string());
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(t!("menu.zmodem_send")).clicked() {
                                self.current_dialog = DialogType::FileTransferSend("zmodem".to_string());
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.zmodem_auto")).clicked() {
                                self.current_dialog = DialogType::FileTransferReceive("zmodem".to_string());
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(t!("menu.kermit_send")).clicked() {
                                self.current_dialog = DialogType::FileTransferSend("kermit".to_string());
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.kermit_receive")).clicked() {
                                self.current_dialog = DialogType::FileTransferReceive("kermit".to_string());
                                ui.close_menu();
                            }
                        });
                        
                        ui.separator();
                        
                        ui.menu_button(t!("menu.protocols"), |ui| {
                            if ui.button(t!("menu.modbus_monitor")).clicked() {
                                self.current_dialog = DialogType::ModbusMonitor;
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.nmea_viewer")).clicked() {
                                self.side_panel_mode = SidePanelMode::Chart;
                                self.show_side_panel = true;
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.protocol_dsl")).clicked() {
                                self.current_dialog = DialogType::ProtocolDsl;
                                ui.close_menu();
                            }
                        });
                        
                        ui.separator();
                        
                        if ui.button(t!("menu.serial_tcp_bridge")).clicked() {
                            self.current_dialog = DialogType::Bridge;
                            ui.close_menu();
                        }
                        
                        ui.separator();
                        
                        if ui.button(t!("menu.triggers_auto")).clicked() {
                            self.current_dialog = DialogType::Triggers;
                            ui.close_menu();
                        }
                        
                        if ui.button(t!("menu.macro_recorder")).clicked() {
                            self.show_macros_bar = true;
                            ui.close_menu();
                        }
                        
                        ui.separator();
                        
                        ui.menu_button(t!("menu.advanced"), |ui| {
                            if ui.button(t!("menu.device_simulator")).clicked() {
                                self.current_dialog = DialogType::DeviceSimulator;
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.session_replay")).clicked() {
                                self.current_dialog = DialogType::SessionReplay;
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.fuzzing_testing")).clicked() {
                                self.current_dialog = DialogType::Fuzzing;
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.experiment_mode")).clicked() {
                                self.current_dialog = DialogType::ExperimentMode;
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui.button(t!("menu.deterministic_mode")).clicked() {
                                self.current_dialog = DialogType::DeterministicMode;
                                ui.close_menu();
                            }
                            if ui.button(t!("menu.explain_debug")).clicked() {
                                self.current_dialog = DialogType::ExplainDebug;
                                ui.close_menu();
                            }
                        });
                        
                        ui.separator();
                        
                        if ui.button(t!("menu.settings")).clicked() {
                            self.current_dialog = DialogType::Settings;
                            ui.close_menu();
                        }
                    });

                    ui.menu_button(t!("menu.help"), |ui| {
                        if ui.button(t!("menu.user_guide")).clicked() {
                            self.current_dialog = DialogType::UserGuide;
                            ui.close_menu();
                        }
                        if ui.button(t!("menu.keyboard_shortcuts")).clicked() {
                            self.current_dialog = DialogType::KeyboardShortcuts;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(t!("menu.about")).clicked() {
                            self.current_dialog = DialogType::About;
                            ui.close_menu();
                        }
                    });
                });
                ui.add_space(4.0);
            });

        // Toolbar
        let toolbar_bg = if self.theme == AppTheme::Dark {
            Color32::from_rgb(32, 32, 38)
        } else {
            Color32::from_rgb(240, 240, 245)
        };
        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::NONE
                .fill(toolbar_bg)
                .inner_margin(Margin::symmetric(10, 6)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.connection_button(ui, "Serial", "S/", ConnectionType::Serial) {
                        self.current_dialog = DialogType::Serial;
                    }
                    ui.add_space(4.0);
                    if self.connection_button(ui, "TCP", "@", ConnectionType::Tcp) {
                        self.current_dialog = DialogType::Tcp;
                    }
                    ui.add_space(4.0);
                    if self.connection_button(ui, "Telnet", "T>", ConnectionType::Telnet) {
                        self.current_dialog = DialogType::Telnet;
                    }
                    ui.add_space(4.0);
                    if self.connection_button(ui, "SSH", "#", ConnectionType::Ssh) {
                        self.current_dialog = DialogType::Ssh;
                    }
                    ui.add_space(4.0);
                    if self.connection_button(ui, "BLE", "B*", ConnectionType::Bluetooth) {
                        self.current_dialog = DialogType::Bluetooth;
                    }

                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(15.0);

                    // Disconnect button
                    let can_disconnect = self.tabs.active_tab()
                        .map(|t| t.state == ConnectionState::Connected)
                        .unwrap_or(false);
                    
                    ui.add_enabled_ui(can_disconnect, |ui| {
                        if ui.add(
                            egui::Button::new(RichText::new("Stop").color(Color32::WHITE))
                                .fill(Color32::from_rgb(180, 60, 60))
                                .corner_radius(CornerRadius::same(4))
                        ).on_hover_text("Disconnect")
                        .clicked() {
                            if let Some(tab) = self.tabs.active_tab_mut() {
                                tab.disconnect();
                            }
                        }
                    });

                    // Serial-specific controls (DTR/RTS)
                    let is_serial_connected = self.tabs.active_tab()
                        .map(|t| t.conn_type == ConnectionType::Serial && t.state == ConnectionState::Connected)
                        .unwrap_or(false);
                    
                    if is_serial_connected {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        // DTR toggle
                        let dtr_color = Color32::from_rgb(60, 140, 60);
                        if ui.add(
                            egui::Button::new("DTR")
                                .fill(dtr_color)
                                .corner_radius(CornerRadius::same(4))
                        ).on_hover_text("Toggle DTR signal")
                        .clicked() {
                            // DTR toggle logic would go here
                        }
                        
                        // RTS toggle
                        let rts_color = Color32::from_rgb(60, 140, 60);
                        if ui.add(
                            egui::Button::new("RTS")
                                .fill(rts_color)
                                .corner_radius(CornerRadius::same(4))
                        ).on_hover_text("Toggle RTS signal")
                        .clicked() {
                            // RTS toggle logic would go here
                        }
                        
                        // Break signal
                        if ui.button("BRK").on_hover_text("Send break signal").clicked() {
                            // Break signal logic
                        }
                    }

                    // File transfer buttons (for all connection types)
                    if can_disconnect {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        
                        // Show SFTP only for SSH
                        let is_ssh = self.tabs.active_tab()
                            .map(|t| t.conn_type == ConnectionType::Ssh)
                            .unwrap_or(false);
                        
                        if is_ssh {
                            if ui.button("SFTP").on_hover_text("Open SFTP file browser").clicked() {
                                self.side_panel_mode = SidePanelMode::Settings; // TODO: Add SFTP panel mode
                            }
                        }
                        
                        // File transfer protocols (for Serial)
                        if is_serial_connected {
                            ui.menu_button("Transfer", |ui| {
                                if ui.button("XMODEM Send").clicked() {
                                    ui.close_menu();
                                }
                                if ui.button("XMODEM Receive").clicked() {
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("YMODEM Send").clicked() {
                                    ui.close_menu();
                                }
                                if ui.button("YMODEM Receive").clicked() {
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("ZMODEM Send").clicked() {
                                    ui.close_menu();
                                }
                                if ui.button("ZMODEM Auto-receive").clicked() {
                                    ui.close_menu();
                                }
                            });
                        }
                    }

                    // Spacer
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Side panel toggle
                        let panel_btn_text = if self.show_side_panel { "<" } else { ">" };
                        if ui.add(egui::Button::new(panel_btn_text)
                            .min_size(Vec2::new(28.0, 28.0)))
                            .on_hover_text(if self.show_side_panel { "Hide panel" } else { "Show panel" })
                            .clicked() {
                            self.show_side_panel = !self.show_side_panel;
                        }
                        
                        ui.add_space(8.0);

                        // Theme toggle
                        let theme_text = if self.theme == AppTheme::Dark { "Light" } else { "Dark" };
                        if ui.add(egui::Button::new(theme_text)
                            .min_size(Vec2::new(40.0, 28.0)))
                            .on_hover_text(format!("Switch to {} theme", theme_text))
                            .clicked() {
                            self.toggle_theme(ui.ctx());
                        }

                        ui.add_space(8.0);

                        // Language toggle  
                        let lang_text = if self.language == Language::English { "HU" } else { "EN" };
                        if ui.add(egui::Button::new(lang_text)
                            .min_size(Vec2::new(28.0, 28.0)))
                            .on_hover_text(if self.language == Language::English { "Switch to Magyar" } else { "Switch to English" })
                            .clicked() {
                            if self.language == Language::English {
                                self.language = Language::Hungarian;
                                set_locale(Locale::Hungarian);
                            } else {
                                self.language = Language::English;
                                set_locale(Locale::English);
                            }
                            // Mark language changed for UI refresh
                            self.language_changed = true;
                            // Force multiple repaints to ensure all text updates
                            ui.ctx().request_repaint();
                        }
                    });
                });
            });

        // Tab bar
        let tab_bar_bg = if self.theme == AppTheme::Dark {
            Color32::from_rgb(28, 28, 32)
        } else {
            Color32::from_rgb(235, 235, 240)
        };
        egui::TopBottomPanel::top("tab_bar")
            .frame(egui::Frame::NONE
                .fill(tab_bar_bg)
                .inner_margin(Margin::symmetric(8, 4)))
            .show(ctx, |ui| {
                self.render_tab_bar(ui);
            });

        // Status bar
        let status_bar_bg = if self.theme == AppTheme::Dark {
            Color32::from_rgb(28, 28, 32)
        } else {
            Color32::from_rgb(235, 235, 240)
        };
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::NONE
                .fill(status_bar_bg)
                .inner_margin(Margin::symmetric(10, 6)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (indicator_color, status_text) = if let Some(tab) = self.tabs.active_tab() {
                        match tab.state {
                            ConnectionState::Connected => (Color32::from_rgb(46, 160, 67), "Connected"),
                            ConnectionState::Connecting => (Color32::YELLOW, "Connecting..."),
                            ConnectionState::Disconnected => (Color32::GRAY, "Disconnected"),
                        }
                    } else {
                        (Color32::GRAY, "No Tab")
                    };

                    ui.label(RichText::new("●").size(12.0).color(indicator_color));
                    ui.label(RichText::new(status_text).size(12.0));
                    ui.separator();

                    if let Some(tab) = self.tabs.active_tab() {
                        if !tab.connection_info.is_empty() {
                            ui.label(RichText::new(&tab.connection_info).size(12.0).color(Color32::GRAY));
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Termicon v0.1.0").size(11.0).color(Color32::DARK_GRAY));
                    });
                });
            });

        // Side panel (right)
        if self.show_side_panel {
            self.render_side_panel(ctx);
        }

        // Main content
        let bg_color = if self.theme == AppTheme::Dark {
            Color32::from_rgb(18, 18, 22)
        } else {
            Color32::from_rgb(250, 250, 252)
        };
        
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE
                .fill(bg_color)
                .inner_margin(Margin::same(0)))
            .show(ctx, |ui| {
                // Macros bar only for Serial connections
                let is_serial = self.tabs.active_tab()
                    .map(|t| t.conn_type == ConnectionType::Serial)
                    .unwrap_or(false);
                let show_macros = self.show_macros_bar && is_serial;
                let macros_height = if show_macros { 58.0 } else { 0.0 };
                
                // Terminal output
                let available_height = ui.available_height() - 45.0 - macros_height;

                egui::Frame::NONE
                    .show(ui, |ui| {
                        ui.set_max_height(available_height);
                        self.render_terminal(ui);
                    });

                // Macros bar (like in classic Terminal programs) - only for Serial
                if show_macros {
                    let macros_bg = if self.theme == AppTheme::Dark {
                        Color32::from_rgb(35, 35, 42)
                    } else {
                        Color32::from_rgb(225, 225, 230)
                    };
                    
                    egui::TopBottomPanel::bottom("macros_panel")
                        .frame(egui::Frame::NONE
                            .fill(macros_bg)
                            .inner_margin(Margin::symmetric(8, 4)))
                        .show_inside(ui, |ui| {
                            let profile_id = self.active_profile_id.clone();
                            if let Some(data) = self.macros_panel.render(ui, profile_id.as_deref()) {
                                // Send macro data
                                if let Some(tab) = self.tabs.active_tab_mut() {
                                    if let Some(ref tx) = tab.tx {
                                        let _ = tx.send(ConnectionCommand::Send(data.clone()));
                                        if tab.local_echo {
                                            if let Ok(text) = String::from_utf8(data) {
                                                tab.add_line(text.trim(), true);
                                            }
                                        }
                                    }
                                }
                            }
                        });
                }

                // Input area
                let input_bg = if self.theme == AppTheme::Dark {
                    Color32::from_rgb(28, 28, 32)
                } else {
                    Color32::from_rgb(235, 235, 240)
                };
                
                egui::TopBottomPanel::bottom("input_panel")
                    .frame(egui::Frame::NONE
                        .fill(input_bg)
                        .inner_margin(Margin::symmetric(10, 8)))
                    .show_inside(ui, |ui| {
                        self.render_input(ui);
                    });
            });

        // Dialogs
        match self.current_dialog {
            DialogType::Serial => self.show_serial_dialog(ctx),
            DialogType::Tcp => self.show_tcp_dialog(ctx),
            DialogType::Telnet => self.show_telnet_dialog(ctx),
            DialogType::Ssh => self.show_ssh_dialog(ctx),
            DialogType::Bluetooth => self.show_bluetooth_dialog(ctx),
            DialogType::About => self.show_about_dialog(ctx),
            DialogType::SaveProfile => self.show_save_profile_dialog(ctx),
            DialogType::KeyboardShortcuts => self.show_keyboard_shortcuts_dialog(ctx),
            DialogType::UserGuide => self.show_user_guide_dialog(ctx),
            DialogType::Settings => self.show_settings_dialog(ctx),
            DialogType::DeviceSimulator => self.show_device_simulator_dialog(ctx),
            DialogType::SessionReplay => self.show_session_replay_dialog(ctx),
            DialogType::Fuzzing => self.show_fuzzing_dialog(ctx),
            DialogType::ExperimentMode => self.show_experiment_mode_dialog(ctx),
            DialogType::DeterministicMode => self.show_deterministic_mode_dialog(ctx),
            DialogType::ExplainDebug => self.show_explain_debug_dialog(ctx),
            DialogType::ModbusMonitor => self.show_modbus_monitor_dialog(ctx),
            DialogType::Bridge => self.show_bridge_dialog(ctx),
            DialogType::VirtualCom => self.show_virtual_com_dialog(ctx),
            DialogType::SshKeyGen => self.show_ssh_keygen_dialog(ctx),
            DialogType::FileTransferSend(ref protocol) => self.show_file_transfer_send_dialog(ctx, protocol.clone()),
            DialogType::FileTransferReceive(ref protocol) => self.show_file_transfer_receive_dialog(ctx, protocol.clone()),
            DialogType::ProtocolDsl => self.show_protocol_dsl_dialog(ctx),
            DialogType::Triggers => self.show_triggers_dialog(ctx),
            DialogType::None => {}
        }

        // Repaint when connected
        let any_connected = self.tabs.tabs.iter().any(|t| 
            t.state == ConnectionState::Connected || t.state == ConnectionState::Connecting
        );
        if any_connected {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
}

// Simple pseudo-random number generator for chart demo
fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    ((seed as f64 * 1103515245.0 + 12345.0) % 2147483648.0) / 2147483648.0
}
