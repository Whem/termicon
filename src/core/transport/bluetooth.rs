//! Bluetooth Transport Module
//!
//! Provides BLE (Bluetooth Low Energy) and SPP (Serial Port Profile) connectivity

use super::{TransportError, TransportStats, TransportTrait, TransportType};
use async_trait::async_trait;
use btleplug::api::{
    Central, CentralEvent, Manager as _, Characteristic, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use bytes::Bytes;
use futures::stream::StreamExt;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

/// Bluetooth device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BluetoothType {
    /// BLE (Bluetooth Low Energy)
    Ble,
    /// Classic Bluetooth SPP (Serial Port Profile)
    Spp,
}

/// BLE Service/Characteristic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleServiceConfig {
    /// Service UUID
    pub service_uuid: String,
    /// TX Characteristic UUID (for writing)
    pub tx_characteristic: String,
    /// RX Characteristic UUID (for reading/notifications)
    pub rx_characteristic: String,
}

impl Default for BleServiceConfig {
    fn default() -> Self {
        // Nordic UART Service (NUS) - common for BLE serial
        Self {
            service_uuid: "6e400001-b5a3-f393-e0a9-e50e24dcca9e".to_string(),
            tx_characteristic: "6e400002-b5a3-f393-e0a9-e50e24dcca9e".to_string(),
            rx_characteristic: "6e400003-b5a3-f393-e0a9-e50e24dcca9e".to_string(),
        }
    }
}

/// Bluetooth connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothConfig {
    /// Device name or address
    pub device: String,
    /// Bluetooth type
    pub bt_type: BluetoothType,
    /// BLE service configuration
    pub ble_service: BleServiceConfig,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// MTU size (for BLE)
    pub mtu: u16,
}

impl Default for BluetoothConfig {
    fn default() -> Self {
        Self {
            device: String::new(),
            bt_type: BluetoothType::Ble,
            ble_service: BleServiceConfig::default(),
            timeout_secs: 10,
            auto_reconnect: false,
            mtu: 512,
        }
    }
}

/// Discovered Bluetooth device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    /// Device name
    pub name: String,
    /// Device address
    pub address: String,
    /// RSSI (signal strength)
    pub rssi: Option<i16>,
    /// Is connectable
    pub connectable: bool,
    /// Device type
    pub bt_type: BluetoothType,
    /// Advertised services
    pub services: Vec<String>,
}

/// Bluetooth scanner for discovering devices
pub struct BluetoothScanner {
    manager: Option<Manager>,
    adapter: Option<Adapter>,
    devices: Arc<RwLock<Vec<BluetoothDevice>>>,
    scanning: bool,
}

impl BluetoothScanner {
    /// Create a new scanner
    pub async fn new() -> Result<Self, TransportError> {
        let manager = Manager::new().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create Bluetooth manager: {}", e)))?;
        
        let adapters = manager.adapters().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get adapters: {}", e)))?;
        
        let adapter = adapters.into_iter().next()
            .ok_or_else(|| TransportError::ConnectionFailed("No Bluetooth adapter found".to_string()))?;
        
        Ok(Self {
            manager: Some(manager),
            adapter: Some(adapter),
            devices: Arc::new(RwLock::new(Vec::new())),
            scanning: false,
        })
    }

    /// Start scanning for devices
    pub async fn start_scan(&mut self, duration_secs: u64) -> Result<(), TransportError> {
        let adapter = self.adapter.as_ref()
            .ok_or_else(|| TransportError::NotConnected)?;
        
        self.devices.write().clear();
        self.scanning = true;

        adapter.start_scan(ScanFilter::default()).await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to start scan: {}", e)))?;

        // Scan for the specified duration
        tokio::time::sleep(Duration::from_secs(duration_secs)).await;

        // Get discovered peripherals
        let peripherals = adapter.peripherals().await
            .map_err(|e| TransportError::ReceiveError(format!("Failed to get peripherals: {}", e)))?;

        let mut devices = Vec::new();
        for peripheral in peripherals {
            if let Ok(Some(props)) = peripheral.properties().await {
                let name = props.local_name.unwrap_or_else(|| "Unknown".to_string());
                let address = peripheral.id().to_string();
                let rssi = props.rssi;
                let services: Vec<String> = props.services.iter().map(|u| u.to_string()).collect();
                
                devices.push(BluetoothDevice {
                    name,
                    address,
                    rssi,
                    connectable: true,
                    bt_type: BluetoothType::Ble,
                    services,
                });
            }
        }

        *self.devices.write() = devices;
        
        adapter.stop_scan().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to stop scan: {}", e)))?;
        
        self.scanning = false;
        Ok(())
    }

    /// Stop scanning
    pub async fn stop_scan(&mut self) -> Result<(), TransportError> {
        if let Some(adapter) = &self.adapter {
            adapter.stop_scan().await
                .map_err(|e| TransportError::ConnectionFailed(format!("Failed to stop scan: {}", e)))?;
        }
        self.scanning = false;
        Ok(())
    }

    /// Get discovered devices
    pub fn get_devices(&self) -> Vec<BluetoothDevice> {
        self.devices.read().clone()
    }

    /// Is scanning
    pub fn is_scanning(&self) -> bool {
        self.scanning
    }
}

/// Bluetooth BLE transport
pub struct BluetoothTransport {
    config: BluetoothConfig,
    manager: Option<Manager>,
    adapter: Option<Adapter>,
    peripheral: Option<Peripheral>,
    tx_char: Option<Characteristic>,
    rx_char: Option<Characteristic>,
    stats: Arc<RwLock<TransportStats>>,
    connected_at: Option<Instant>,
    tx: broadcast::Sender<Bytes>,
    notification_task: Option<tokio::task::JoinHandle<()>>,
}

impl BluetoothTransport {
    /// Create a new Bluetooth transport
    pub async fn new(config: BluetoothConfig) -> Result<Self, TransportError> {
        let (tx, _) = broadcast::channel(1024);
        
        Ok(Self {
            config,
            manager: None,
            adapter: None,
            peripheral: None,
            tx_char: None,
            rx_char: None,
            stats: Arc::new(RwLock::new(TransportStats::default())),
            connected_at: None,
            tx,
            notification_task: None,
        })
    }

    /// Find peripheral by name or address
    async fn find_peripheral(&self, adapter: &Adapter) -> Result<Peripheral, TransportError> {
        let peripherals = adapter.peripherals().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get peripherals: {}", e)))?;

        for peripheral in peripherals {
            if let Ok(Some(props)) = peripheral.properties().await {
                let name = props.local_name.unwrap_or_default();
                let address = peripheral.id().to_string();
                
                if name == self.config.device || address == self.config.device {
                    return Ok(peripheral);
                }
            }
        }

        Err(TransportError::ConnectionFailed(format!("Device '{}' not found", self.config.device)))
    }

    /// Find characteristics after connection
    async fn discover_characteristics(&mut self) -> Result<(), TransportError> {
        let peripheral = self.peripheral.as_ref()
            .ok_or_else(|| TransportError::NotConnected)?;

        peripheral.discover_services().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to discover services: {}", e)))?;

        let service_uuid = Uuid::parse_str(&self.config.ble_service.service_uuid)
            .map_err(|e| TransportError::InvalidConfiguration(format!("Invalid service UUID: {}", e)))?;
        let tx_uuid = Uuid::parse_str(&self.config.ble_service.tx_characteristic)
            .map_err(|e| TransportError::InvalidConfiguration(format!("Invalid TX UUID: {}", e)))?;
        let rx_uuid = Uuid::parse_str(&self.config.ble_service.rx_characteristic)
            .map_err(|e| TransportError::InvalidConfiguration(format!("Invalid RX UUID: {}", e)))?;

        let characteristics = peripheral.characteristics();
        
        for char in characteristics {
            if char.uuid == tx_uuid {
                self.tx_char = Some(char.clone());
            }
            if char.uuid == rx_uuid {
                self.rx_char = Some(char.clone());
            }
        }

        if self.tx_char.is_none() {
            return Err(TransportError::ConnectionFailed("TX characteristic not found".to_string()));
        }
        if self.rx_char.is_none() {
            return Err(TransportError::ConnectionFailed("RX characteristic not found".to_string()));
        }

        Ok(())
    }

    /// Start notification subscription
    async fn start_notifications(&mut self) -> Result<(), TransportError> {
        let peripheral = self.peripheral.clone()
            .ok_or_else(|| TransportError::NotConnected)?;
        let rx_char = self.rx_char.clone()
            .ok_or_else(|| TransportError::NotConnected)?;
        
        peripheral.subscribe(&rx_char).await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to subscribe: {}", e)))?;

        let tx = self.tx.clone();
        let stats = self.stats.clone();

        let task = tokio::spawn(async move {
            let mut notification_stream = peripheral.notifications().await.unwrap();
            
            while let Some(data) = notification_stream.next().await {
                let bytes = Bytes::from(data.value);
                let len = bytes.len();
                
                // Update stats
                {
                    let mut s = stats.write();
                    s.bytes_received += len as u64;
                    s.packets_received += 1;
                }
                
                let _ = tx.send(bytes);
            }
        });

        self.notification_task = Some(task);
        Ok(())
    }
}

#[async_trait]
impl TransportTrait for BluetoothTransport {
    fn transport_type(&self) -> TransportType {
        TransportType::Bluetooth
    }

    async fn connect(&mut self) -> Result<(), TransportError> {
        let manager = Manager::new().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to create manager: {}", e)))?;
        
        let adapters = manager.adapters().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to get adapters: {}", e)))?;
        
        let adapter = adapters.into_iter().next()
            .ok_or_else(|| TransportError::ConnectionFailed("No Bluetooth adapter found".to_string()))?;

        // Start scanning to find device
        adapter.start_scan(ScanFilter::default()).await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to start scan: {}", e)))?;
        
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Find the device
        let peripheral = self.find_peripheral(&adapter).await?;
        
        adapter.stop_scan().await
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to stop scan: {}", e)))?;

        // Connect to device
        let timeout = Duration::from_secs(self.config.timeout_secs);
        tokio::time::timeout(timeout, peripheral.connect()).await
            .map_err(|_| TransportError::ConnectionFailed("Connection timeout".to_string()))?
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to connect: {}", e)))?;

        self.manager = Some(manager);
        self.adapter = Some(adapter);
        self.peripheral = Some(peripheral);
        self.connected_at = Some(Instant::now());

        // Discover services and characteristics
        self.discover_characteristics().await?;

        // Start notifications
        self.start_notifications().await?;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        // Stop notification task
        if let Some(task) = self.notification_task.take() {
            task.abort();
        }

        // Disconnect from peripheral
        if let Some(ref peripheral) = self.peripheral {
            peripheral.disconnect().await
                .map_err(|e| TransportError::ConnectionFailed(format!("Failed to disconnect: {}", e)))?;
        }

        self.peripheral = None;
        self.tx_char = None;
        self.rx_char = None;
        self.connected_at = None;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.peripheral.is_some() && self.connected_at.is_some()
    }

    async fn send(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        let peripheral = self.peripheral.as_ref()
            .ok_or_else(|| TransportError::NotConnected)?;
        let tx_char = self.tx_char.as_ref()
            .ok_or_else(|| TransportError::NotConnected)?;

        let len = data.len();

        // Write data to TX characteristic
        peripheral.write(tx_char, data, WriteType::WithoutResponse).await
            .map_err(|e| TransportError::SendError(format!("Write failed: {}", e)))?;

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.bytes_sent += len as u64;
            stats.packets_sent += 1;
        }

        Ok(len)
    }

    async fn receive(&mut self) -> Result<Bytes, TransportError> {
        // Receive is handled through notifications/subscribe
        Err(TransportError::ReceiveError("Use subscribe() for BLE notifications".to_string()))
    }

    fn subscribe(&self) -> broadcast::Receiver<Bytes> {
        self.tx.subscribe()
    }

    fn stats(&self) -> TransportStats {
        self.stats.read().clone()
    }

    fn connection_info(&self) -> String {
        if let Some(ref peripheral) = self.peripheral {
            format!("BLE: {} ({})", self.config.device, peripheral.id())
        } else {
            format!("BLE: {} (disconnected)", self.config.device)
        }
    }
}

/// GATT Service browser
pub struct GattBrowser {
    peripheral: Peripheral,
}

impl GattBrowser {
    /// Create a new GATT browser for a connected peripheral
    pub fn new(peripheral: Peripheral) -> Self {
        Self { peripheral }
    }

    /// Get all services
    pub async fn get_services(&self) -> Result<Vec<GattService>, TransportError> {
        self.peripheral.discover_services().await
            .map_err(|e| TransportError::ReceiveError(format!("Failed to discover services: {}", e)))?;

        let chars = self.peripheral.characteristics();
        let mut services: std::collections::HashMap<Uuid, GattService> = std::collections::HashMap::new();

        for char in chars {
            let service_uuid = char.service_uuid;
            let entry = services.entry(service_uuid).or_insert_with(|| GattService {
                uuid: service_uuid.to_string(),
                characteristics: Vec::new(),
            });

            entry.characteristics.push(GattCharacteristic {
                uuid: char.uuid.to_string(),
                properties: format!("{:?}", char.properties),
            });
        }

        Ok(services.into_values().collect())
    }

    /// Read a characteristic value
    pub async fn read_characteristic(&self, char_uuid: &str) -> Result<Vec<u8>, TransportError> {
        let uuid = Uuid::parse_str(char_uuid)
            .map_err(|e| TransportError::InvalidConfiguration(format!("Invalid UUID: {}", e)))?;

        let chars = self.peripheral.characteristics();
        let char = chars.iter().find(|c| c.uuid == uuid)
            .ok_or_else(|| TransportError::ReceiveError("Characteristic not found".to_string()))?;

        self.peripheral.read(char).await
            .map_err(|e| TransportError::ReceiveError(format!("Read failed: {}", e)))
    }

    /// Write a characteristic value
    pub async fn write_characteristic(&self, char_uuid: &str, data: &[u8]) -> Result<(), TransportError> {
        let uuid = Uuid::parse_str(char_uuid)
            .map_err(|e| TransportError::InvalidConfiguration(format!("Invalid UUID: {}", e)))?;

        let chars = self.peripheral.characteristics();
        let char = chars.iter().find(|c| c.uuid == uuid)
            .ok_or_else(|| TransportError::SendError("Characteristic not found".to_string()))?;

        self.peripheral.write(char, data, WriteType::WithResponse).await
            .map_err(|e| TransportError::SendError(format!("Write failed: {}", e)))
    }
}

/// GATT Service info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GattService {
    /// Service UUID
    pub uuid: String,
    /// Characteristics
    pub characteristics: Vec<GattCharacteristic>,
}

/// GATT Characteristic info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GattCharacteristic {
    /// Characteristic UUID
    pub uuid: String,
    /// Properties (Read, Write, Notify, etc.)
    pub properties: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluetooth_config_default() {
        let config = BluetoothConfig::default();
        assert_eq!(config.bt_type, BluetoothType::Ble);
        assert_eq!(config.timeout_secs, 10);
        assert_eq!(config.mtu, 512);
    }

    #[test]
    fn test_ble_service_config_default() {
        let config = BleServiceConfig::default();
        // Nordic UART Service
        assert!(config.service_uuid.contains("6e400001"));
    }
}

