# Bluetooth Module

## Overview

The Bluetooth module provides support for both Bluetooth Low Energy (BLE) and Classic Bluetooth (SPP/RFCOMM) communication.

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| BLE Scanning | ✅ | Discover BLE devices |
| BLE Connect | ✅ | Connect to BLE devices |
| GATT Services | ✅ | Service discovery |
| GATT Characteristics | ✅ | Read/Write/Notify |
| Nordic UART (NUS) | ✅ | Nordic UART Service |
| Battery Service | ✅ | Read battery level |
| Device Information | ✅ | Read device info |
| SPP/RFCOMM | 🔄 | Classic Bluetooth serial |
| Device Pairing | 🔄 | Pair with devices |

## BLE Configuration

```rust
pub struct BleConfig {
    pub device_address: String,      // Device MAC address
    pub service_uuid: Uuid,          // Service UUID
    pub tx_characteristic: Uuid,     // TX characteristic UUID
    pub rx_characteristic: Uuid,     // RX characteristic UUID (notify)
    pub scan_timeout: Duration,      // Scan timeout
}
```

## Common UUIDs

### Nordic UART Service (NUS)

```rust
pub const NUS_SERVICE: &str = "6e400001-b5a3-f393-e0a9-e50e24dcca9e";
pub const NUS_RX_CHAR: &str = "6e400002-b5a3-f393-e0a9-e50e24dcca9e";
pub const NUS_TX_CHAR: &str = "6e400003-b5a3-f393-e0a9-e50e24dcca9e";
```

### Standard Services

| Service | UUID |
|---------|------|
| Generic Access | 0x1800 |
| Generic Attribute | 0x1801 |
| Device Information | 0x180A |
| Battery Service | 0x180F |
| Heart Rate | 0x180D |

## GUI Usage

### BLE Connection

1. Click **BLE** button in toolbar
2. Click **Scan** to discover devices
3. Select device from list
4. Configure service/characteristic UUIDs
5. Click **Connect**

### BLE Inspector

The BLE Inspector panel shows:
- Connected device information
- Discovered services
- Characteristics with properties
- Read/Write/Notify operations

### Device Scanning

During scanning:
- Device name (if available)
- MAC address
- RSSI signal strength
- Advertising data

## CLI Usage

```bash
# Scan for devices
termicon-cli ble scan --timeout 10

# Connect to device
termicon-cli ble connect AA:BB:CC:DD:EE:FF

# Connect with Nordic UART
termicon-cli ble connect AA:BB:CC:DD:EE:FF --profile nus

# List services
termicon-cli ble services AA:BB:CC:DD:EE:FF

# Read characteristic
termicon-cli ble read AA:BB:CC:DD:EE:FF --service 180F --char 2A19
```

## Code Examples

### Scanning for Devices

```rust
use termicon_core::bluetooth::{BleScanner, ScanFilter};

let scanner = BleScanner::new()?;

// Scan with filter
let filter = ScanFilter {
    name_contains: Some("Nordic".to_string()),
    services: Some(vec![NUS_SERVICE_UUID]),
    rssi_threshold: Some(-80),
};

let devices = scanner.scan(Duration::from_secs(10), Some(filter)).await?;

for device in devices {
    println!("{}: {} (RSSI: {})", 
        device.name.unwrap_or("Unknown".to_string()),
        device.address,
        device.rssi);
}
```

### Connecting to Device

```rust
use termicon_core::bluetooth::{BleTransport, BleConfig};

let config = BleConfig {
    device_address: "AA:BB:CC:DD:EE:FF".to_string(),
    service_uuid: NUS_SERVICE_UUID,
    tx_characteristic: NUS_TX_UUID,
    rx_characteristic: NUS_RX_UUID,
    scan_timeout: Duration::from_secs(10),
};

let mut transport = BleTransport::new(config)?;
transport.connect().await?;
```

### Reading/Writing Data

```rust
// Send data (write to TX characteristic)
transport.send(b"Hello BLE\r\n").await?;

// Receive data (from RX notifications)
let data = transport.receive().await?;
println!("Received: {:?}", data);
```

### GATT Operations

```rust
// Discover services
let services = transport.discover_services().await?;

for service in services {
    println!("Service: {}", service.uuid);
    
    for char in service.characteristics {
        println!("  Characteristic: {}", char.uuid);
        println!("    Properties: {:?}", char.properties);
        
        // Read if readable
        if char.properties.read {
            let value = transport.read_characteristic(&char.uuid).await?;
            println!("    Value: {:?}", value);
        }
    }
}

// Write to characteristic
transport.write_characteristic(&char_uuid, b"data", false).await?;

// Write with response
transport.write_characteristic(&char_uuid, b"data", true).await?;

// Subscribe to notifications
transport.subscribe(&char_uuid).await?;
```

## Nordic UART Service

The Nordic UART Service (NUS) is commonly used for serial-over-BLE:

### Auto-Detection

```rust
// Connect with automatic NUS detection
let config = BleConfig::nordic_uart("AA:BB:CC:DD:EE:FF");
let transport = BleTransport::new(config)?;
```

### NUS Protocol

```
┌─────────────────────────────────────┐
│         Nordic UART Service          │
│   UUID: 6e400001-b5a3-f393-e0a9-...  │
├─────────────────────────────────────┤
│ RX Characteristic (Write)            │
│ UUID: 6e400002-b5a3-f393-e0a9-...   │
│ → Data FROM phone TO device          │
├─────────────────────────────────────┤
│ TX Characteristic (Notify)           │
│ UUID: 6e400003-b5a3-f393-e0a9-...   │
│ ← Data FROM device TO phone          │
└─────────────────────────────────────┘
```

## Profile Support

BLE connections can be saved as profiles:

1. Connect to BLE device
2. Click **Save Profile** in toolbar
3. Profile saves:
   - Device address
   - Service UUID
   - Characteristic UUIDs
   - Custom settings

## Platform Support

### Windows

- Windows 10 1803+ required
- Uses Windows.Devices.Bluetooth API
- Bluetooth adapter must be enabled

### Linux

- BlueZ 5.48+ required
- D-Bus interface used
- May require `bluetoothctl` permissions

### macOS

- CoreBluetooth framework
- May require Bluetooth permissions
- Privacy settings check

## Troubleshooting

### Device Not Found

- Ensure device is advertising
- Check device is not already connected
- Try moving closer to device
- Verify Bluetooth is enabled

### Connection Failed

- Device may have connection limit
- Try power cycling device
- Check pairing requirements
- Verify device is BLE (not Classic only)

### No Data Received

- Check characteristic UUID is correct
- Verify notifications are enabled
- Check MTU size (default 20 bytes)
- Verify device is sending data

### MTU Issues

```rust
// Request larger MTU
transport.request_mtu(512).await?;

// Check negotiated MTU
let mtu = transport.get_mtu();
println!("MTU: {} bytes", mtu);
```

## Classic Bluetooth (SPP)

For traditional serial over Bluetooth:

```rust
use termicon_core::bluetooth::{SppConfig, SppTransport};

let config = SppConfig {
    device_address: "AA:BB:CC:DD:EE:FF".to_string(),
    channel: 1,  // RFCOMM channel
};

let mut transport = SppTransport::new(config)?;
transport.connect().await?;

// Use like serial port
transport.send(b"AT\r\n").await?;
```

**Note:** SPP support is currently in progress (🔄)

## Security Notes

- BLE connections may not be encrypted by default
- Use pairing for sensitive applications
- Consider application-level encryption
- Be aware of BLE sniffing risks
