//! Connection dialog component

use crate::core::transport::{
    SerialConfig, SerialFlowControl, SerialParity, TcpConfig, TelnetConfig, Transport,
};
use crate::i18n::t;
use egui::{ComboBox, Grid, Ui};

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionType {
    #[default]
    Serial,
    Tcp,
    Telnet,
}

/// Connection dialog state
pub struct ConnectionDialog {
    /// Selected connection type
    connection_type: ConnectionType,
    /// Serial port settings
    serial: SerialDialogState,
    /// TCP settings
    tcp: TcpDialogState,
    /// Telnet settings
    telnet: TelnetDialogState,
    /// Available serial ports
    available_ports: Vec<String>,
}

/// Serial port dialog state
struct SerialDialogState {
    port: String,
    baud_rate: String,
    data_bits: u8,
    stop_bits: u8,
    parity: SerialParity,
    flow_control: SerialFlowControl,
    auto_reconnect: bool,
}

/// TCP dialog state
struct TcpDialogState {
    host: String,
    port: String,
    timeout: String,
}

/// Telnet dialog state
struct TelnetDialogState {
    host: String,
    port: String,
}

impl ConnectionDialog {
    /// Create a new connection dialog
    pub fn new() -> Self {
        // Get available serial ports
        let available_ports = serialport::available_ports()
            .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
            .unwrap_or_default();

        let default_port = available_ports.first().cloned().unwrap_or_default();

        Self {
            connection_type: ConnectionType::Serial,
            serial: SerialDialogState {
                port: default_port,
                baud_rate: "115200".to_string(),
                data_bits: 8,
                stop_bits: 1,
                parity: SerialParity::None,
                flow_control: SerialFlowControl::None,
                auto_reconnect: true,
            },
            tcp: TcpDialogState {
                host: "localhost".to_string(),
                port: "23".to_string(),
                timeout: "10".to_string(),
            },
            telnet: TelnetDialogState {
                host: "localhost".to_string(),
                port: "23".to_string(),
            },
            available_ports,
        }
    }

    /// Refresh available serial ports
    pub fn refresh_ports(&mut self) {
        self.available_ports = serialport::available_ports()
            .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
            .unwrap_or_default();
    }

    /// Show the dialog UI
    pub fn show(&mut self, ui: &mut Ui) {
        // Connection type selector
        ui.horizontal(|ui| {
            ui.label(t("dialog.connection_type"));

            ComboBox::from_id_salt("connection_type")
                .selected_text(match self.connection_type {
                    ConnectionType::Serial => t("dialog.serial"),
                    ConnectionType::Tcp => t("dialog.tcp"),
                    ConnectionType::Telnet => t("dialog.telnet"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.connection_type,
                        ConnectionType::Serial,
                        t("dialog.serial"),
                    );
                    ui.selectable_value(
                        &mut self.connection_type,
                        ConnectionType::Tcp,
                        t("dialog.tcp"),
                    );
                    ui.selectable_value(
                        &mut self.connection_type,
                        ConnectionType::Telnet,
                        t("dialog.telnet"),
                    );
                });
        });

        ui.separator();

        // Type-specific settings
        match self.connection_type {
            ConnectionType::Serial => self.show_serial_settings(ui),
            ConnectionType::Tcp => self.show_tcp_settings(ui),
            ConnectionType::Telnet => self.show_telnet_settings(ui),
        }
    }

    /// Show serial port settings
    fn show_serial_settings(&mut self, ui: &mut Ui) {
        Grid::new("serial_settings")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                // Port selector with refresh button
                ui.label(t("serial.port"));
                ui.horizontal(|ui| {
                    ComboBox::from_id_salt("serial_port")
                        .selected_text(&self.serial.port)
                        .show_ui(ui, |ui| {
                            for port in &self.available_ports {
                                ui.selectable_value(&mut self.serial.port, port.clone(), port);
                            }
                        });

                    if ui.button("🔄").clicked() {
                        self.refresh_ports();
                    }
                });
                ui.end_row();

                // Baud rate
                ui.label(t("serial.baud_rate"));
                ComboBox::from_id_salt("baud_rate")
                    .selected_text(&self.serial.baud_rate)
                    .show_ui(ui, |ui| {
                        for rate in &[
                            "300", "1200", "2400", "4800", "9600", "19200", "38400", "57600",
                            "115200", "230400", "460800", "921600",
                        ] {
                            ui.selectable_value(
                                &mut self.serial.baud_rate,
                                rate.to_string(),
                                *rate,
                            );
                        }
                    });
                ui.end_row();

                // Data bits
                ui.label(t("serial.data_bits"));
                ComboBox::from_id_salt("data_bits")
                    .selected_text(format!("{}", self.serial.data_bits))
                    .show_ui(ui, |ui| {
                        for bits in &[5u8, 6, 7, 8] {
                            ui.selectable_value(
                                &mut self.serial.data_bits,
                                *bits,
                                format!("{}", bits),
                            );
                        }
                    });
                ui.end_row();

                // Stop bits
                ui.label(t("serial.stop_bits"));
                ComboBox::from_id_salt("stop_bits")
                    .selected_text(format!("{}", self.serial.stop_bits))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.serial.stop_bits, 1, "1");
                        ui.selectable_value(&mut self.serial.stop_bits, 2, "2");
                    });
                ui.end_row();

                // Parity
                ui.label(t("serial.parity"));
                ComboBox::from_id_salt("parity")
                    .selected_text(match self.serial.parity {
                        SerialParity::None => t("serial.parity_none"),
                        SerialParity::Odd => t("serial.parity_odd"),
                        SerialParity::Even => t("serial.parity_even"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.serial.parity,
                            SerialParity::None,
                            t("serial.parity_none"),
                        );
                        ui.selectable_value(
                            &mut self.serial.parity,
                            SerialParity::Odd,
                            t("serial.parity_odd"),
                        );
                        ui.selectable_value(
                            &mut self.serial.parity,
                            SerialParity::Even,
                            t("serial.parity_even"),
                        );
                    });
                ui.end_row();

                // Flow control
                ui.label(t("serial.flow_control"));
                ComboBox::from_id_salt("flow_control")
                    .selected_text(match self.serial.flow_control {
                        SerialFlowControl::None => t("serial.flow_none"),
                        SerialFlowControl::Hardware => t("serial.flow_hardware"),
                        SerialFlowControl::Software => t("serial.flow_software"),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.serial.flow_control,
                            SerialFlowControl::None,
                            t("serial.flow_none"),
                        );
                        ui.selectable_value(
                            &mut self.serial.flow_control,
                            SerialFlowControl::Hardware,
                            t("serial.flow_hardware"),
                        );
                        ui.selectable_value(
                            &mut self.serial.flow_control,
                            SerialFlowControl::Software,
                            t("serial.flow_software"),
                        );
                    });
                ui.end_row();

                // Auto-reconnect
                ui.label(t("autoconnect.enabled"));
                ui.checkbox(&mut self.serial.auto_reconnect, "");
                ui.end_row();
            });
    }

    /// Show TCP settings
    fn show_tcp_settings(&mut self, ui: &mut Ui) {
        Grid::new("tcp_settings")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(t("network.host"));
                ui.text_edit_singleline(&mut self.tcp.host);
                ui.end_row();

                ui.label(t("network.port"));
                ui.text_edit_singleline(&mut self.tcp.port);
                ui.end_row();

                ui.label(t("network.timeout"));
                ui.text_edit_singleline(&mut self.tcp.timeout);
                ui.end_row();
            });
    }

    /// Show Telnet settings
    fn show_telnet_settings(&mut self, ui: &mut Ui) {
        Grid::new("telnet_settings")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(t("network.host"));
                ui.text_edit_singleline(&mut self.telnet.host);
                ui.end_row();

                ui.label(t("network.port"));
                ui.text_edit_singleline(&mut self.telnet.port);
                ui.end_row();
            });
    }

    /// Get the configured transport
    pub fn get_transport(&self) -> Option<Transport> {
        match self.connection_type {
            ConnectionType::Serial => {
                let baud_rate: u32 = self.serial.baud_rate.parse().ok()?;

                Some(Transport::Serial(SerialConfig {
                    port: self.serial.port.clone(),
                    baud_rate,
                    data_bits: self.serial.data_bits,
                    stop_bits: self.serial.stop_bits,
                    parity: self.serial.parity,
                    flow_control: self.serial.flow_control,
                    auto_reconnect: self.serial.auto_reconnect,
                }))
            }
            ConnectionType::Tcp => {
                let port: u16 = self.tcp.port.parse().ok()?;
                let timeout: u64 = self.tcp.timeout.parse().unwrap_or(10);

                Some(Transport::Tcp(TcpConfig {
                    host: self.tcp.host.clone(),
                    port,
                    timeout_secs: timeout,
                }))
            }
            ConnectionType::Telnet => {
                let port: u16 = self.telnet.port.parse().unwrap_or(23);

                Some(Transport::Telnet(TelnetConfig {
                    host: self.telnet.host.clone(),
                    port,
                    terminal_type: "xterm".to_string(),
                }))
            }
        }
    }
}

impl Default for ConnectionDialog {
    fn default() -> Self {
        Self::new()
    }
}





