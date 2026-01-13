//! Bluetooth module for BLE and Classic Bluetooth support
//!
//! Provides:
//! - BLE GATT client (scan, connect, services/characteristics)
//! - RFCOMM/SPP serial terminal
//! - Device management (pairing, bonding, trust)
//! - HCI logging and debugging

pub mod ble;
pub mod device;
pub mod rfcomm;

pub use ble::{BleClient, BleConfig, GattCharacteristic, GattService};
pub use device::{BluetoothDevice, BluetoothDeviceClass, BluetoothDeviceType};
pub use rfcomm::{RfcommConfig, RfcommTransport};

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;
use uuid::Uuid;

/// Bluetooth errors
#[derive(Error, Debug)]
pub enum BluetoothError {
    /// Adapter not found
    #[error("Bluetooth adapter not found")]
    AdapterNotFound,

    /// Adapter error
    #[error("Adapter error: {0}")]
    AdapterError(String),

    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Not connected
    #[error("Not connected")]
    NotConnected,

    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Characteristic not found
    #[error("Characteristic not found: {0}")]
    CharacteristicNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Pairing failed
    #[error("Pairing failed: {0}")]
    PairingFailed(String),

    /// Read/Write error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Platform not supported
    #[error("Platform not supported for this operation")]
    PlatformNotSupported,

    /// Timeout
    #[error("Operation timed out")]
    Timeout,
}

/// Bluetooth adapter information
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    /// Adapter ID/name
    pub id: String,
    /// Adapter address (MAC)
    pub address: String,
    /// Adapter name (friendly)
    pub name: String,
    /// Is powered on
    pub powered: bool,
    /// Is discoverable
    pub discoverable: bool,
    /// Is pairable
    pub pairable: bool,
    /// Is discovering
    pub discovering: bool,
}

/// Bluetooth manager for adapter and device management
pub struct BluetoothManager {
    /// Known devices
    devices: Arc<RwLock<HashMap<String, BluetoothDevice>>>,
    /// Active adapter info
    adapter: Arc<RwLock<Option<AdapterInfo>>>,
}

impl BluetoothManager {
    /// Create a new Bluetooth manager
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            adapter: Arc::new(RwLock::new(None)),
        }
    }

    /// Get available Bluetooth adapters
    pub async fn get_adapters(&self) -> Result<Vec<AdapterInfo>, BluetoothError> {
        // Platform-specific implementation
        #[cfg(target_os = "linux")]
        {
            // On Linux, use BlueZ D-Bus API via bluer crate
            // For now, return a placeholder
            Ok(vec![AdapterInfo {
                id: "hci0".to_string(),
                address: "00:00:00:00:00:00".to_string(),
                name: "Default Adapter".to_string(),
                powered: true,
                discoverable: false,
                pairable: true,
                discovering: false,
            }])
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, use Windows.Devices.Bluetooth API
            Ok(vec![AdapterInfo {
                id: "local".to_string(),
                address: "00:00:00:00:00:00".to_string(),
                name: "Windows Bluetooth".to_string(),
                powered: true,
                discoverable: false,
                pairable: true,
                discovering: false,
            }])
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, use IOBluetooth/CoreBluetooth
            Ok(vec![AdapterInfo {
                id: "default".to_string(),
                address: "00:00:00:00:00:00".to_string(),
                name: "macOS Bluetooth".to_string(),
                powered: true,
                discoverable: false,
                pairable: true,
                discovering: false,
            }])
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(BluetoothError::PlatformNotSupported)
        }
    }

    /// Start device discovery
    pub async fn start_discovery(&self) -> Result<(), BluetoothError> {
        // Platform-specific implementation using btleplug
        tracing::info!("Starting Bluetooth discovery...");
        Ok(())
    }

    /// Stop device discovery
    pub async fn stop_discovery(&self) -> Result<(), BluetoothError> {
        tracing::info!("Stopping Bluetooth discovery...");
        Ok(())
    }

    /// Get discovered devices
    pub fn get_devices(&self) -> Vec<BluetoothDevice> {
        self.devices.read().values().cloned().collect()
    }

    /// Get device by address
    pub fn get_device(&self, address: &str) -> Option<BluetoothDevice> {
        self.devices.read().get(address).cloned()
    }

    /// Add or update a device
    pub fn update_device(&self, device: BluetoothDevice) {
        self.devices.write().insert(device.address.clone(), device);
    }

    /// Remove a device
    pub fn remove_device(&self, address: &str) {
        self.devices.write().remove(address);
    }

    /// Pair with a device
    pub async fn pair(&self, address: &str) -> Result<(), BluetoothError> {
        tracing::info!("Pairing with device: {}", address);
        // Platform-specific pairing
        Ok(())
    }

    /// Trust a device (auto-connect)
    pub async fn trust(&self, address: &str, trusted: bool) -> Result<(), BluetoothError> {
        tracing::info!("Setting trust for device {}: {}", address, trusted);
        Ok(())
    }

    /// Remove pairing
    pub async fn unpair(&self, address: &str) -> Result<(), BluetoothError> {
        tracing::info!("Removing pairing for device: {}", address);
        Ok(())
    }
}

impl Default for BluetoothManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Well-known BLE service UUIDs
pub mod service_uuids {
    use uuid::Uuid;

    /// Generic Access
    pub const GENERIC_ACCESS: Uuid = Uuid::from_u128(0x00001800_0000_1000_8000_00805f9b34fb);
    /// Generic Attribute
    pub const GENERIC_ATTRIBUTE: Uuid = Uuid::from_u128(0x00001801_0000_1000_8000_00805f9b34fb);
    /// Device Information
    pub const DEVICE_INFORMATION: Uuid = Uuid::from_u128(0x0000180a_0000_1000_8000_00805f9b34fb);
    /// Battery Service
    pub const BATTERY: Uuid = Uuid::from_u128(0x0000180f_0000_1000_8000_00805f9b34fb);
    /// Heart Rate
    pub const HEART_RATE: Uuid = Uuid::from_u128(0x0000180d_0000_1000_8000_00805f9b34fb);
    /// Nordic UART Service (NUS)
    pub const NORDIC_UART: Uuid = Uuid::from_u128(0x6e400001_b5a3_f393_e0a9_e50e24dcca9e);
}

/// Well-known BLE characteristic UUIDs
pub mod characteristic_uuids {
    use uuid::Uuid;

    /// Device Name
    pub const DEVICE_NAME: Uuid = Uuid::from_u128(0x00002a00_0000_1000_8000_00805f9b34fb);
    /// Appearance
    pub const APPEARANCE: Uuid = Uuid::from_u128(0x00002a01_0000_1000_8000_00805f9b34fb);
    /// Battery Level
    pub const BATTERY_LEVEL: Uuid = Uuid::from_u128(0x00002a19_0000_1000_8000_00805f9b34fb);
    /// Manufacturer Name
    pub const MANUFACTURER_NAME: Uuid = Uuid::from_u128(0x00002a29_0000_1000_8000_00805f9b34fb);
    /// Model Number
    pub const MODEL_NUMBER: Uuid = Uuid::from_u128(0x00002a24_0000_1000_8000_00805f9b34fb);
    /// Firmware Revision
    pub const FIRMWARE_REVISION: Uuid = Uuid::from_u128(0x00002a26_0000_1000_8000_00805f9b34fb);
    /// Nordic UART RX
    pub const NORDIC_UART_RX: Uuid = Uuid::from_u128(0x6e400002_b5a3_f393_e0a9_e50e24dcca9e);
    /// Nordic UART TX
    pub const NORDIC_UART_TX: Uuid = Uuid::from_u128(0x6e400003_b5a3_f393_e0a9_e50e24dcca9e);
}








