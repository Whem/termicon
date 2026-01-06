//! BLE Inspector Panel
//!
//! Provides a dedicated UI for Bluetooth Low Energy device inspection:
//! - Device scanning
//! - GATT service/characteristic browser
//! - RSSI monitoring
//! - Connection management

use eframe::egui::{self, Color32, RichText, Vec2};
use std::collections::HashMap;

/// BLE device entry
#[derive(Debug, Clone)]
pub struct BleDeviceEntry {
    pub name: String,
    pub address: String,
    pub rssi: Option<i16>,
    pub connectable: bool,
    pub services: Vec<String>,
    pub last_seen: std::time::Instant,
}

/// GATT service display
#[derive(Debug, Clone)]
pub struct GattServiceDisplay {
    pub uuid: String,
    pub name: Option<String>,
    pub characteristics: Vec<GattCharacteristicDisplay>,
    pub expanded: bool,
}

/// GATT characteristic display
#[derive(Debug, Clone)]
pub struct GattCharacteristicDisplay {
    pub uuid: String,
    pub name: Option<String>,
    pub properties: String,
    pub value: Option<Vec<u8>>,
    pub notifications_enabled: bool,
}

/// RSSI history entry
#[derive(Debug, Clone, Copy)]
pub struct RssiEntry {
    pub timestamp: f64,
    pub rssi: i16,
}

/// BLE Inspector panel state
pub struct BleInspectorPanel {
    /// Discovered devices
    devices: HashMap<String, BleDeviceEntry>,
    /// Selected device address
    selected_device: Option<String>,
    /// Connected device address
    connected_device: Option<String>,
    /// GATT services of connected device
    services: Vec<GattServiceDisplay>,
    /// Scanning state
    scanning: bool,
    /// Scan duration (seconds)
    scan_duration: u32,
    /// RSSI history
    rssi_history: Vec<RssiEntry>,
    /// Max RSSI history size
    max_rssi_history: usize,
    /// Show unknown services
    show_unknown_services: bool,
    /// Auto-refresh characteristics
    auto_refresh: bool,
    /// Filter text
    filter_text: String,
}

impl Default for BleInspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl BleInspectorPanel {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            selected_device: None,
            connected_device: None,
            services: Vec::new(),
            scanning: false,
            scan_duration: 5,
            rssi_history: Vec::new(),
            max_rssi_history: 100,
            show_unknown_services: true,
            auto_refresh: false,
            filter_text: String::new(),
        }
    }

    /// Add or update a discovered device
    pub fn add_device(&mut self, device: BleDeviceEntry) {
        self.devices.insert(device.address.clone(), device);
    }

    /// Clear all devices
    pub fn clear_devices(&mut self) {
        self.devices.clear();
    }

    /// Set services for connected device
    pub fn set_services(&mut self, services: Vec<GattServiceDisplay>) {
        self.services = services;
    }

    /// Add RSSI reading
    pub fn add_rssi(&mut self, rssi: i16) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        
        self.rssi_history.push(RssiEntry {
            timestamp: now,
            rssi,
        });

        if self.rssi_history.len() > self.max_rssi_history {
            self.rssi_history.remove(0);
        }
    }

    /// Get service name from UUID
    fn get_service_name(uuid: &str) -> Option<&'static str> {
        match uuid.to_lowercase().as_str() {
            "1800" | "00001800-0000-1000-8000-00805f9b34fb" => Some("Generic Access"),
            "1801" | "00001801-0000-1000-8000-00805f9b34fb" => Some("Generic Attribute"),
            "180a" | "0000180a-0000-1000-8000-00805f9b34fb" => Some("Device Information"),
            "180f" | "0000180f-0000-1000-8000-00805f9b34fb" => Some("Battery Service"),
            "1809" | "00001809-0000-1000-8000-00805f9b34fb" => Some("Health Thermometer"),
            "180d" | "0000180d-0000-1000-8000-00805f9b34fb" => Some("Heart Rate"),
            "6e400001-b5a3-f393-e0a9-e50e24dcca9e" => Some("Nordic UART Service"),
            _ => None,
        }
    }

    /// Get characteristic name from UUID
    fn get_characteristic_name(uuid: &str) -> Option<&'static str> {
        match uuid.to_lowercase().as_str() {
            "2a00" | "00002a00-0000-1000-8000-00805f9b34fb" => Some("Device Name"),
            "2a01" | "00002a01-0000-1000-8000-00805f9b34fb" => Some("Appearance"),
            "2a19" | "00002a19-0000-1000-8000-00805f9b34fb" => Some("Battery Level"),
            "2a29" | "00002a29-0000-1000-8000-00805f9b34fb" => Some("Manufacturer Name"),
            "2a24" | "00002a24-0000-1000-8000-00805f9b34fb" => Some("Model Number"),
            "2a25" | "00002a25-0000-1000-8000-00805f9b34fb" => Some("Serial Number"),
            "2a26" | "00002a26-0000-1000-8000-00805f9b34fb" => Some("Firmware Revision"),
            "2a27" | "00002a27-0000-1000-8000-00805f9b34fb" => Some("Hardware Revision"),
            "2a28" | "00002a28-0000-1000-8000-00805f9b34fb" => Some("Software Revision"),
            "6e400002-b5a3-f393-e0a9-e50e24dcca9e" => Some("Nordic UART TX"),
            "6e400003-b5a3-f393-e0a9-e50e24dcca9e" => Some("Nordic UART RX"),
            _ => None,
        }
    }

    /// Render the panel
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("📶 BLE Inspector");
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.scanning {
                    if ui.button("⏹ Stop Scan").clicked() {
                        self.scanning = false;
                    }
                    ui.spinner();
                } else {
                    if ui.button("🔍 Scan").clicked() {
                        self.scanning = true;
                        // TODO: Trigger actual scan
                    }
                }
                
                ui.add(egui::Slider::new(&mut self.scan_duration, 1..=30).suffix("s").text("Duration"));
            });
        });

        ui.separator();

        // Main content split
        ui.columns(2, |columns| {
            // Left: Device list
            columns[0].group(|ui| {
                ui.heading("Devices");
                
                ui.horizontal(|ui| {
                    ui.label("🔎");
                    ui.add(egui::TextEdit::singleline(&mut self.filter_text)
                        .hint_text("Filter...")
                        .desired_width(150.0));
                });

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        let filter = self.filter_text.to_lowercase();
                        let mut sorted_devices: Vec<_> = self.devices.values()
                            .filter(|d| {
                                filter.is_empty() || 
                                d.name.to_lowercase().contains(&filter) ||
                                d.address.to_lowercase().contains(&filter)
                            })
                            .collect();
                        
                        // Sort by RSSI (strongest first)
                        sorted_devices.sort_by(|a, b| {
                            b.rssi.unwrap_or(-100).cmp(&a.rssi.unwrap_or(-100))
                        });

                        for device in sorted_devices {
                            let is_selected = self.selected_device.as_ref() == Some(&device.address);
                            let is_connected = self.connected_device.as_ref() == Some(&device.address);
                            
                            let bg_color = if is_connected {
                                Color32::from_rgb(46, 160, 67)
                            } else if is_selected {
                                Color32::from_rgb(60, 60, 75)
                            } else {
                                Color32::TRANSPARENT
                            };

                            let frame = egui::Frame::NONE
                                .fill(bg_color)
                                .inner_margin(4.0)
                                .corner_radius(4.0);

                            frame.show(ui, |ui| {
                                if ui.add(
                                    egui::Button::new(
                                        RichText::new(&device.name)
                                            .strong()
                                            .color(if is_connected { Color32::WHITE } else { Color32::LIGHT_GRAY })
                                    )
                                    .frame(false)
                                    .min_size(Vec2::new(180.0, 0.0))
                                ).clicked() {
                                    self.selected_device = Some(device.address.clone());
                                }

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&device.address).small().color(Color32::GRAY));
                                    
                                    if let Some(rssi) = device.rssi {
                                        let rssi_color = if rssi > -50 {
                                            Color32::GREEN
                                        } else if rssi > -70 {
                                            Color32::YELLOW
                                        } else {
                                            Color32::RED
                                        };
                                        ui.label(RichText::new(format!("{} dBm", rssi)).small().color(rssi_color));
                                    }
                                });
                            });
                        }

                        if self.devices.is_empty() {
                            ui.label(RichText::new("No devices found").color(Color32::GRAY).italics());
                            ui.label(RichText::new("Click 'Scan' to search for BLE devices").small().color(Color32::DARK_GRAY));
                        }
                    });

                // Device actions
                if let Some(ref addr) = self.selected_device {
                    ui.separator();
                    ui.horizontal(|ui| {
                        if self.connected_device.is_some() {
                            if ui.button("Disconnect").clicked() {
                                self.connected_device = None;
                                self.services.clear();
                            }
                        } else {
                            if ui.button("Connect").clicked() {
                                self.connected_device = Some(addr.clone());
                                // TODO: Trigger actual connection
                            }
                        }
                    });
                }
            });

            // Right: GATT browser
            columns[1].group(|ui| {
                ui.heading("GATT Services");
                
                ui.checkbox(&mut self.show_unknown_services, "Show unknown services");
                ui.checkbox(&mut self.auto_refresh, "Auto-refresh values");

                if self.connected_device.is_none() {
                    ui.label(RichText::new("Connect to a device to browse services").color(Color32::GRAY).italics());
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(350.0)
                        .show(ui, |ui| {
                            for service in &mut self.services {
                                let service_name = Self::get_service_name(&service.uuid)
                                    .map(|s| s.to_string())
                                    .or_else(|| service.name.clone())
                                    .unwrap_or_else(|| "Unknown Service".to_string());

                                if !self.show_unknown_services && service.name.is_none() && Self::get_service_name(&service.uuid).is_none() {
                                    continue;
                                }

                                let header = egui::CollapsingHeader::new(
                                    RichText::new(format!("📦 {}", service_name)).strong()
                                )
                                .id_salt(&service.uuid);

                                header.show(ui, |ui| {
                                    ui.label(RichText::new(format!("UUID: {}", service.uuid)).small().color(Color32::GRAY));
                                    ui.separator();

                                    for char in &mut service.characteristics {
                                        let char_name = Self::get_characteristic_name(&char.uuid)
                                            .map(|s| s.to_string())
                                            .or_else(|| char.name.clone())
                                            .unwrap_or_else(|| "Unknown".to_string());

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("  📝 {}", char_name)));
                                            ui.label(RichText::new(format!("[{}]", char.properties)).small().color(Color32::YELLOW));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("     {}", char.uuid)).monospace().small().color(Color32::DARK_GRAY));
                                        });

                                        if let Some(ref value) = char.value {
                                            ui.horizontal(|ui| {
                                                ui.label("     Value:");
                                                ui.label(RichText::new(hex::encode(value)).monospace().color(Color32::LIGHT_BLUE));
                                            });
                                        }

                                        ui.horizontal(|ui| {
                                            ui.add_space(20.0);
                                            if ui.small_button("Read").clicked() {
                                                // TODO: Read characteristic
                                            }
                                            if char.properties.contains("Write") {
                                                if ui.small_button("Write").clicked() {
                                                    // TODO: Show write dialog
                                                }
                                            }
                                            if char.properties.contains("Notify") {
                                                let btn_text = if char.notifications_enabled { "Stop Notify" } else { "Notify" };
                                                if ui.small_button(btn_text).clicked() {
                                                    char.notifications_enabled = !char.notifications_enabled;
                                                }
                                            }
                                        });

                                        ui.add_space(4.0);
                                    }
                                });
                            }

                            if self.services.is_empty() {
                                ui.label(RichText::new("Discovering services...").color(Color32::GRAY).italics());
                            }
                        });
                }
            });
        });

        // RSSI chart (if connected)
        if self.connected_device.is_some() && !self.rssi_history.is_empty() {
            ui.separator();
            ui.heading("RSSI Timeline");
            
            let points: egui_plot::PlotPoints = self.rssi_history.iter()
                .map(|e| [e.timestamp, e.rssi as f64])
                .collect();

            let line = egui_plot::Line::new(points)
                .color(Color32::from_rgb(100, 200, 100))
                .name("RSSI");

            egui_plot::Plot::new("rssi_plot")
                .height(100.0)
                .allow_zoom(false)
                .allow_drag(false)
                .show_axes([false, true])
                .y_axis_label("dBm")
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        }
    }
}


