//! Bluetooth device representation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Bluetooth device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BluetoothDeviceType {
    /// BLE only device
    Ble,
    /// Classic Bluetooth only
    Classic,
    /// Dual-mode (BLE + Classic)
    DualMode,
    /// Unknown type
    Unknown,
}

/// Bluetooth device class (Classic)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BluetoothDeviceClass {
    /// Computer (desktop, laptop, etc.)
    Computer,
    /// Phone
    Phone,
    /// Audio device (headphones, speaker)
    Audio,
    /// Peripheral (keyboard, mouse, etc.)
    Peripheral,
    /// Imaging device (printer, scanner)
    Imaging,
    /// Wearable (watch, etc.)
    Wearable,
    /// Health device
    Health,
    /// Toy
    Toy,
    /// Unknown class
    Unknown,
}

impl BluetoothDeviceClass {
    /// Parse from Class of Device (CoD) value
    pub fn from_cod(cod: u32) -> Self {
        let major = (cod >> 8) & 0x1F;
        match major {
            1 => Self::Computer,
            2 => Self::Phone,
            4 => Self::Audio,
            5 => Self::Peripheral,
            6 => Self::Imaging,
            7 => Self::Wearable,
            9 => Self::Health,
            8 => Self::Toy,
            _ => Self::Unknown,
        }
    }
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Connecting in progress
    Connecting,
    /// Connected
    Connected,
    /// Disconnecting
    Disconnecting,
}

/// Bluetooth device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    /// Device address (MAC or platform-specific ID)
    pub address: String,
    /// Device name (may be empty if not yet discovered)
    pub name: Option<String>,
    /// Device type (BLE/Classic/Dual)
    pub device_type: BluetoothDeviceType,
    /// Device class (for Classic)
    pub device_class: BluetoothDeviceClass,
    /// RSSI (signal strength)
    pub rssi: Option<i16>,
    /// Is paired/bonded
    pub paired: bool,
    /// Is trusted (auto-connect)
    pub trusted: bool,
    /// Is connected
    pub connected: bool,
    /// Connection state
    pub connection_state: ConnectionState,
    /// Advertised service UUIDs (for BLE)
    pub service_uuids: Vec<Uuid>,
    /// Manufacturer data (for BLE advertising)
    pub manufacturer_data: HashMap<u16, Vec<u8>>,
    /// Service data (for BLE advertising)
    pub service_data: HashMap<Uuid, Vec<u8>>,
    /// TX power level (if advertised)
    pub tx_power: Option<i8>,
    /// Last seen timestamp
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    /// User-defined label
    pub label: Option<String>,
    /// User notes
    pub notes: Option<String>,
    /// Battery level (if available)
    pub battery_level: Option<u8>,
    /// Firmware version (if available)
    pub firmware_version: Option<String>,
}

impl BluetoothDevice {
    /// Create a new device with address
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            name: None,
            device_type: BluetoothDeviceType::Unknown,
            device_class: BluetoothDeviceClass::Unknown,
            rssi: None,
            paired: false,
            trusted: false,
            connected: false,
            connection_state: ConnectionState::Disconnected,
            service_uuids: Vec::new(),
            manufacturer_data: HashMap::new(),
            service_data: HashMap::new(),
            tx_power: None,
            last_seen: Some(chrono::Utc::now()),
            label: None,
            notes: None,
            battery_level: None,
            firmware_version: None,
        }
    }

    /// Get display name (name or address if name unknown)
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.address)
    }

    /// Check if device supports BLE
    pub fn supports_ble(&self) -> bool {
        matches!(self.device_type, BluetoothDeviceType::Ble | BluetoothDeviceType::DualMode)
    }

    /// Check if device supports Classic Bluetooth
    pub fn supports_classic(&self) -> bool {
        matches!(self.device_type, BluetoothDeviceType::Classic | BluetoothDeviceType::DualMode)
    }

    /// Check if device has a specific service
    pub fn has_service(&self, uuid: &Uuid) -> bool {
        self.service_uuids.contains(uuid)
    }

    /// Get manufacturer name from manufacturer data
    pub fn manufacturer_name(&self) -> Option<&'static str> {
        // Check common manufacturer IDs
        for &company_id in self.manufacturer_data.keys() {
            match company_id {
                0x004C => return Some("Apple"),
                0x0006 => return Some("Microsoft"),
                0x000D => return Some("Texas Instruments"),
                0x000F => return Some("Broadcom"),
                0x0059 => return Some("Nordic Semiconductor"),
                0x00E0 => return Some("Google"),
                0x0075 => return Some("Samsung"),
                0x02D5 => return Some("Xiaomi"),
                _ => {}
            }
        }
        None
    }

    /// Update from another device (merge data)
    pub fn update_from(&mut self, other: &BluetoothDevice) {
        if other.name.is_some() {
            self.name = other.name.clone();
        }
        if other.device_type != BluetoothDeviceType::Unknown {
            self.device_type = other.device_type;
        }
        if other.device_class != BluetoothDeviceClass::Unknown {
            self.device_class = other.device_class;
        }
        if other.rssi.is_some() {
            self.rssi = other.rssi;
        }
        self.paired = other.paired;
        self.trusted = other.trusted;
        self.connected = other.connected;
        self.connection_state = other.connection_state;
        
        // Merge service UUIDs
        for uuid in &other.service_uuids {
            if !self.service_uuids.contains(uuid) {
                self.service_uuids.push(*uuid);
            }
        }
        
        // Merge manufacturer data
        self.manufacturer_data.extend(other.manufacturer_data.clone());
        self.service_data.extend(other.service_data.clone());
        
        if other.tx_power.is_some() {
            self.tx_power = other.tx_power;
        }
        
        self.last_seen = Some(chrono::Utc::now());
        
        if other.battery_level.is_some() {
            self.battery_level = other.battery_level;
        }
        if other.firmware_version.is_some() {
            self.firmware_version = other.firmware_version.clone();
        }
    }
}

impl Default for BluetoothDevice {
    fn default() -> Self {
        Self::new("00:00:00:00:00:00")
    }
}





