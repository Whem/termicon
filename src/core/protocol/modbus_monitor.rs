//! Modbus Register Monitoring and Polling System
//!
//! Provides automatic polling of Modbus registers with configurable intervals,
//! data type conversion, and value change detection.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tokio::sync::mpsc;

/// Data types for register interpretation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModbusDataType {
    /// Unsigned 16-bit integer (single register)
    U16,
    /// Signed 16-bit integer (single register)
    I16,
    /// Unsigned 32-bit integer (2 registers, big-endian)
    U32BE,
    /// Unsigned 32-bit integer (2 registers, little-endian)
    U32LE,
    /// Signed 32-bit integer (2 registers, big-endian)
    I32BE,
    /// Signed 32-bit integer (2 registers, little-endian)
    I32LE,
    /// 32-bit float (2 registers, big-endian)
    F32BE,
    /// 32-bit float (2 registers, little-endian)
    F32LE,
    /// Unsigned 64-bit integer (4 registers)
    U64BE,
    /// Signed 64-bit integer (4 registers)
    I64BE,
    /// 64-bit float (4 registers)
    F64BE,
    /// Binary/Bit field
    Binary,
    /// ASCII string (multiple registers)
    Ascii,
}

impl ModbusDataType {
    /// Get number of registers needed for this type
    pub fn register_count(&self) -> u16 {
        match self {
            Self::U16 | Self::I16 | Self::Binary => 1,
            Self::U32BE | Self::U32LE | Self::I32BE | Self::I32LE | Self::F32BE | Self::F32LE => 2,
            Self::U64BE | Self::I64BE | Self::F64BE => 4,
            Self::Ascii => 1, // Variable, will be multiplied by string length
        }
    }
    
    /// Convert register values to displayable value
    pub fn convert(&self, registers: &[u16]) -> Option<ModbusValue> {
        match self {
            Self::U16 => registers.first().map(|&v| ModbusValue::U64(v as u64)),
            Self::I16 => registers.first().map(|&v| ModbusValue::I64(v as i16 as i64)),
            
            Self::U32BE => {
                if registers.len() >= 2 {
                    let value = ((registers[0] as u32) << 16) | (registers[1] as u32);
                    Some(ModbusValue::U64(value as u64))
                } else {
                    None
                }
            }
            Self::U32LE => {
                if registers.len() >= 2 {
                    let value = ((registers[1] as u32) << 16) | (registers[0] as u32);
                    Some(ModbusValue::U64(value as u64))
                } else {
                    None
                }
            }
            Self::I32BE => {
                if registers.len() >= 2 {
                    let value = ((registers[0] as u32) << 16) | (registers[1] as u32);
                    Some(ModbusValue::I64(value as i32 as i64))
                } else {
                    None
                }
            }
            Self::I32LE => {
                if registers.len() >= 2 {
                    let value = ((registers[1] as u32) << 16) | (registers[0] as u32);
                    Some(ModbusValue::I64(value as i32 as i64))
                } else {
                    None
                }
            }
            Self::F32BE => {
                if registers.len() >= 2 {
                    let bits = ((registers[0] as u32) << 16) | (registers[1] as u32);
                    Some(ModbusValue::F64(f32::from_bits(bits) as f64))
                } else {
                    None
                }
            }
            Self::F32LE => {
                if registers.len() >= 2 {
                    let bits = ((registers[1] as u32) << 16) | (registers[0] as u32);
                    Some(ModbusValue::F64(f32::from_bits(bits) as f64))
                } else {
                    None
                }
            }
            Self::U64BE => {
                if registers.len() >= 4 {
                    let value = ((registers[0] as u64) << 48)
                        | ((registers[1] as u64) << 32)
                        | ((registers[2] as u64) << 16)
                        | (registers[3] as u64);
                    Some(ModbusValue::U64(value))
                } else {
                    None
                }
            }
            Self::I64BE => {
                if registers.len() >= 4 {
                    let value = ((registers[0] as u64) << 48)
                        | ((registers[1] as u64) << 32)
                        | ((registers[2] as u64) << 16)
                        | (registers[3] as u64);
                    Some(ModbusValue::I64(value as i64))
                } else {
                    None
                }
            }
            Self::F64BE => {
                if registers.len() >= 4 {
                    let bits = ((registers[0] as u64) << 48)
                        | ((registers[1] as u64) << 32)
                        | ((registers[2] as u64) << 16)
                        | (registers[3] as u64);
                    Some(ModbusValue::F64(f64::from_bits(bits)))
                } else {
                    None
                }
            }
            Self::Binary => {
                registers.first().map(|&v| ModbusValue::Binary(v))
            }
            Self::Ascii => {
                let mut s = String::new();
                for &reg in registers {
                    let hi = (reg >> 8) as u8;
                    let lo = (reg & 0xFF) as u8;
                    if hi != 0 {
                        s.push(hi as char);
                    }
                    if lo != 0 {
                        s.push(lo as char);
                    }
                }
                Some(ModbusValue::String(s))
            }
        }
    }
}

/// Parsed value from Modbus registers
#[derive(Debug, Clone)]
pub enum ModbusValue {
    U64(u64),
    I64(i64),
    F64(f64),
    Binary(u16),
    String(String),
}

impl ModbusValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::U64(v) => v.to_string(),
            Self::I64(v) => v.to_string(),
            Self::F64(v) => format!("{:.6}", v),
            Self::Binary(v) => format!("{:016b}", v),
            Self::String(s) => s.clone(),
        }
    }
    
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::U64(v) => Some(*v as f64),
            Self::I64(v) => Some(*v as f64),
            Self::F64(v) => Some(*v),
            _ => None,
        }
    }
}

/// Register definition for monitoring
#[derive(Debug, Clone)]
pub struct RegisterDefinition {
    /// Register address
    pub address: u16,
    /// Number of registers to read
    pub count: u16,
    /// Register type (holding, input, coil, discrete)
    pub register_type: RegisterType,
    /// Data type for interpretation
    pub data_type: ModbusDataType,
    /// Human-readable name
    pub name: String,
    /// Unit (e.g., "°C", "bar", "%")
    pub unit: String,
    /// Scale factor (multiply raw value)
    pub scale: f64,
    /// Offset (add after scaling)
    pub offset: f64,
    /// Description
    pub description: String,
    /// Poll interval override (None = use group default)
    pub poll_interval: Option<Duration>,
    /// Enable change detection
    pub detect_change: bool,
}

impl RegisterDefinition {
    pub fn new(address: u16, name: impl Into<String>) -> Self {
        Self {
            address,
            count: 1,
            register_type: RegisterType::Holding,
            data_type: ModbusDataType::U16,
            name: name.into(),
            unit: String::new(),
            scale: 1.0,
            offset: 0.0,
            description: String::new(),
            poll_interval: None,
            detect_change: false,
        }
    }
    
    /// Builder: set register type
    pub fn register_type(mut self, t: RegisterType) -> Self {
        self.register_type = t;
        self
    }
    
    /// Builder: set data type
    pub fn data_type(mut self, t: ModbusDataType) -> Self {
        self.data_type = t;
        self.count = t.register_count();
        self
    }
    
    /// Builder: set unit
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    
    /// Builder: set scale and offset
    pub fn scale_offset(mut self, scale: f64, offset: f64) -> Self {
        self.scale = scale;
        self.offset = offset;
        self
    }
    
    /// Builder: enable change detection
    pub fn with_change_detection(mut self) -> Self {
        self.detect_change = true;
        self
    }
    
    /// Apply scaling to a value
    pub fn apply_scaling(&self, value: &ModbusValue) -> Option<f64> {
        value.to_f64().map(|v| v * self.scale + self.offset)
    }
}

/// Register type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegisterType {
    Coil,           // Read/write bit (function 1, 5, 15)
    DiscreteInput,  // Read-only bit (function 2)
    Holding,        // Read/write 16-bit (function 3, 6, 16)
    Input,          // Read-only 16-bit (function 4)
}

impl RegisterType {
    pub fn read_function_code(&self) -> u8 {
        match self {
            Self::Coil => 1,
            Self::DiscreteInput => 2,
            Self::Holding => 3,
            Self::Input => 4,
        }
    }
}

/// Poll group - registers that are polled together
#[derive(Debug, Clone)]
pub struct PollGroup {
    /// Group name
    pub name: String,
    /// Slave ID
    pub slave_id: u8,
    /// Poll interval
    pub interval: Duration,
    /// Registers in this group
    pub registers: Vec<RegisterDefinition>,
    /// Enabled
    pub enabled: bool,
}

impl PollGroup {
    pub fn new(name: impl Into<String>, slave_id: u8, interval: Duration) -> Self {
        Self {
            name: name.into(),
            slave_id,
            interval,
            registers: Vec::new(),
            enabled: true,
        }
    }
    
    /// Add a register to the group
    pub fn add_register(&mut self, reg: RegisterDefinition) {
        self.registers.push(reg);
    }
    
    /// Optimize reads by grouping consecutive registers
    pub fn optimize_reads(&self) -> Vec<OptimizedRead> {
        let mut reads = Vec::new();
        
        // Group by register type
        let mut by_type: HashMap<RegisterType, Vec<&RegisterDefinition>> = HashMap::new();
        for reg in &self.registers {
            by_type.entry(reg.register_type).or_default().push(reg);
        }
        
        for (reg_type, regs) in by_type {
            // Sort by address
            let mut sorted: Vec<_> = regs.into_iter().collect();
            sorted.sort_by_key(|r| r.address);
            
            // Group consecutive addresses
            let mut current_start = 0u16;
            let mut current_end = 0u16;
            let mut current_regs = Vec::new();
            
            for reg in sorted {
                if current_regs.is_empty() {
                    current_start = reg.address;
                    current_end = reg.address + reg.count;
                    current_regs.push(reg.clone());
                } else if reg.address <= current_end + 5 {
                    // Allow small gaps (5 registers) to reduce number of reads
                    current_end = current_end.max(reg.address + reg.count);
                    current_regs.push(reg.clone());
                } else {
                    // Start new group
                    reads.push(OptimizedRead {
                        register_type: reg_type,
                        start_address: current_start,
                        count: current_end - current_start,
                        registers: std::mem::take(&mut current_regs),
                    });
                    current_start = reg.address;
                    current_end = reg.address + reg.count;
                    current_regs.push(reg.clone());
                }
            }
            
            if !current_regs.is_empty() {
                reads.push(OptimizedRead {
                    register_type: reg_type,
                    start_address: current_start,
                    count: current_end - current_start,
                    registers: current_regs,
                });
            }
        }
        
        reads
    }
}

/// Optimized read operation
#[derive(Debug, Clone)]
pub struct OptimizedRead {
    pub register_type: RegisterType,
    pub start_address: u16,
    pub count: u16,
    pub registers: Vec<RegisterDefinition>,
}

/// Register value with metadata
#[derive(Debug, Clone)]
pub struct RegisterReading {
    /// Register definition
    pub definition: RegisterDefinition,
    /// Raw register values
    pub raw_values: Vec<u16>,
    /// Converted value
    pub value: Option<ModbusValue>,
    /// Scaled value
    pub scaled_value: Option<f64>,
    /// Timestamp
    pub timestamp: Instant,
    /// Error (if read failed)
    pub error: Option<String>,
    /// Value changed since last read
    pub changed: bool,
}

/// Polling event
#[derive(Debug, Clone)]
pub enum PollingEvent {
    /// New readings available
    Readings(Vec<RegisterReading>),
    /// Value changed
    ValueChanged {
        register: String,
        old_value: Option<ModbusValue>,
        new_value: ModbusValue,
    },
    /// Polling error
    Error(String),
    /// Polling started
    Started,
    /// Polling stopped
    Stopped,
}

/// Modbus polling scheduler
pub struct ModbusPoller {
    /// Poll groups
    groups: Arc<RwLock<Vec<PollGroup>>>,
    /// Last readings
    last_readings: Arc<RwLock<HashMap<String, RegisterReading>>>,
    /// Event sender
    event_tx: mpsc::Sender<PollingEvent>,
    /// Running flag
    running: Arc<RwLock<bool>>,
}

impl ModbusPoller {
    /// Create new poller
    pub fn new(event_tx: mpsc::Sender<PollingEvent>) -> Self {
        Self {
            groups: Arc::new(RwLock::new(Vec::new())),
            last_readings: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Add a poll group
    pub fn add_group(&self, group: PollGroup) {
        self.groups.write().push(group);
    }
    
    /// Remove a poll group by name
    pub fn remove_group(&self, name: &str) {
        self.groups.write().retain(|g| g.name != name);
    }
    
    /// Get all groups
    pub fn groups(&self) -> Vec<PollGroup> {
        self.groups.read().clone()
    }
    
    /// Enable/disable a group
    pub fn set_group_enabled(&self, name: &str, enabled: bool) {
        let mut groups = self.groups.write();
        if let Some(group) = groups.iter_mut().find(|g| g.name == name) {
            group.enabled = enabled;
        }
    }
    
    /// Get last readings
    pub fn get_readings(&self) -> HashMap<String, RegisterReading> {
        self.last_readings.read().clone()
    }
    
    /// Get reading for specific register
    pub fn get_reading(&self, name: &str) -> Option<RegisterReading> {
        self.last_readings.read().get(name).cloned()
    }
    
    /// Check if poller is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
    
    /// Stop polling
    pub fn stop(&self) {
        *self.running.write() = false;
    }
    
    /// Start polling (call in async context)
    /// 
    /// The `read_fn` should perform the actual Modbus read and return register values
    pub async fn start<F, Fut>(&self, mut read_fn: F)
    where
        F: FnMut(u8, RegisterType, u16, u16) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<u16>, String>>,
    {
        *self.running.write() = true;
        let _ = self.event_tx.send(PollingEvent::Started).await;
        
        let mut last_poll: HashMap<String, Instant> = HashMap::new();
        
        while *self.running.read() {
            let now = Instant::now();
            let groups = self.groups.read().clone();
            
            for group in groups {
                if !group.enabled {
                    continue;
                }
                
                // Check if it's time to poll this group
                let should_poll = last_poll
                    .get(&group.name)
                    .map(|&t| now.duration_since(t) >= group.interval)
                    .unwrap_or(true);
                
                if !should_poll {
                    continue;
                }
                
                last_poll.insert(group.name.clone(), now);
                
                // Optimize reads
                let optimized = group.optimize_reads();
                let mut readings = Vec::new();
                
                for read in optimized {
                    // Perform read
                    let result = read_fn(
                        group.slave_id,
                        read.register_type,
                        read.start_address,
                        read.count,
                    ).await;
                    
                    match result {
                        Ok(values) => {
                            // Extract individual register values
                            for reg_def in &read.registers {
                                let offset = (reg_def.address - read.start_address) as usize;
                                let count = reg_def.count as usize;
                                
                                if offset + count <= values.len() {
                                    let raw_values: Vec<u16> = values[offset..offset + count].to_vec();
                                    let value = reg_def.data_type.convert(&raw_values);
                                    let scaled_value = value.as_ref().and_then(|v| reg_def.apply_scaling(v));
                                    
                                    // Check for change
                                    let changed = if reg_def.detect_change {
                                        let last = self.last_readings.read();
                                        if let Some(prev) = last.get(&reg_def.name) {
                                            match (&prev.value, &value) {
                                                (Some(ModbusValue::U64(a)), Some(ModbusValue::U64(b))) => a != b,
                                                (Some(ModbusValue::I64(a)), Some(ModbusValue::I64(b))) => a != b,
                                                (Some(ModbusValue::F64(a)), Some(ModbusValue::F64(b))) => (a - b).abs() > 0.0001,
                                                _ => true,
                                            }
                                        } else {
                                            true
                                        }
                                    } else {
                                        false
                                    };
                                    
                                    let reading = RegisterReading {
                                        definition: reg_def.clone(),
                                        raw_values,
                                        value: value.clone(),
                                        scaled_value,
                                        timestamp: now,
                                        error: None,
                                        changed,
                                    };
                                    
                                    // Send change event
                                    if changed {
                                        if let Some(v) = value {
                                            let _ = self.event_tx.send(PollingEvent::ValueChanged {
                                                register: reg_def.name.clone(),
                                                old_value: self.last_readings.read()
                                                    .get(&reg_def.name)
                                                    .and_then(|r| r.value.clone()),
                                                new_value: v,
                                            }).await;
                                        }
                                    }
                                    
                                    readings.push(reading);
                                }
                            }
                        }
                        Err(e) => {
                            for reg_def in &read.registers {
                                readings.push(RegisterReading {
                                    definition: reg_def.clone(),
                                    raw_values: Vec::new(),
                                    value: None,
                                    scaled_value: None,
                                    timestamp: now,
                                    error: Some(e.clone()),
                                    changed: false,
                                });
                            }
                            let _ = self.event_tx.send(PollingEvent::Error(e)).await;
                        }
                    }
                }
                
                // Update last readings
                {
                    let mut last = self.last_readings.write();
                    for reading in &readings {
                        last.insert(reading.definition.name.clone(), reading.clone());
                    }
                }
                
                // Send readings event
                if !readings.is_empty() {
                    let _ = self.event_tx.send(PollingEvent::Readings(readings)).await;
                }
            }
            
            // Small sleep to prevent busy loop
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        let _ = self.event_tx.send(PollingEvent::Stopped).await;
    }
}

/// Common Modbus register templates
pub mod templates {
    use super::*;
    
    /// Temperature sensor (0.1°C resolution)
    pub fn temperature(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::I16)
            .unit("°C")
            .scale_offset(0.1, 0.0)
            .with_change_detection()
    }
    
    /// Humidity sensor (0.1% resolution)
    pub fn humidity(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::U16)
            .unit("%")
            .scale_offset(0.1, 0.0)
    }
    
    /// Pressure sensor (32-bit float)
    pub fn pressure_f32(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::F32BE)
            .unit("bar")
    }
    
    /// Counter (32-bit unsigned)
    pub fn counter_u32(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::U32BE)
            .with_change_detection()
    }
    
    /// Status word (16 bits)
    pub fn status_word(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::Binary)
            .with_change_detection()
    }
    
    /// Energy meter (kWh, 32-bit float)
    pub fn energy_kwh(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::F32BE)
            .unit("kWh")
    }
    
    /// Voltage (V, 16-bit with 0.1 resolution)
    pub fn voltage(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::U16)
            .unit("V")
            .scale_offset(0.1, 0.0)
    }
    
    /// Current (A, 16-bit with 0.01 resolution)
    pub fn current(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::U16)
            .unit("A")
            .scale_offset(0.01, 0.0)
    }
    
    /// Frequency (Hz, 16-bit with 0.01 resolution)
    pub fn frequency(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::U16)
            .unit("Hz")
            .scale_offset(0.01, 0.0)
    }
    
    /// Power (W, 32-bit signed)
    pub fn power_w(address: u16, name: &str) -> RegisterDefinition {
        RegisterDefinition::new(address, name)
            .data_type(ModbusDataType::I32BE)
            .unit("W")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_type_conversion() {
        // U16
        let dt = ModbusDataType::U16;
        let value = dt.convert(&[0x1234]).unwrap();
        assert!(matches!(value, ModbusValue::U64(0x1234)));
        
        // I16
        let dt = ModbusDataType::I16;
        let value = dt.convert(&[0xFFFF]).unwrap();
        assert!(matches!(value, ModbusValue::I64(-1)));
        
        // U32BE
        let dt = ModbusDataType::U32BE;
        let value = dt.convert(&[0x0001, 0x0002]).unwrap();
        assert!(matches!(value, ModbusValue::U64(0x00010002)));
        
        // F32BE
        let dt = ModbusDataType::F32BE;
        let bits = 3.14f32.to_bits();
        let hi = ((bits >> 16) & 0xFFFF) as u16;
        let lo = (bits & 0xFFFF) as u16;
        let value = dt.convert(&[hi, lo]).unwrap();
        if let ModbusValue::F64(f) = value {
            assert!((f - 3.14f64).abs() < 0.01);
        }
    }
    
    #[test]
    fn test_poll_group_optimization() {
        let mut group = PollGroup::new("Test", 1, Duration::from_secs(1));
        
        group.add_register(RegisterDefinition::new(0, "Reg0"));
        group.add_register(RegisterDefinition::new(1, "Reg1"));
        group.add_register(RegisterDefinition::new(2, "Reg2"));
        group.add_register(RegisterDefinition::new(100, "Reg100"));
        
        let optimized = group.optimize_reads();
        
        // Should create 2 read groups: 0-2 and 100
        assert_eq!(optimized.len(), 2);
    }
    
    #[test]
    fn test_templates() {
        let temp = templates::temperature(0, "Room Temp");
        assert_eq!(temp.scale, 0.1);
        assert_eq!(temp.unit, "°C");
        
        let pressure = templates::pressure_f32(10, "Pressure");
        assert_eq!(pressure.count, 2); // F32 uses 2 registers
    }
}

