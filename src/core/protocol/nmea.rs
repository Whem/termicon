//! NMEA 0183 Protocol Parser
//!
//! Parses standard NMEA sentences used in GPS and marine equipment.
//!
//! Supported sentences:
//! - GGA: Global Positioning System Fix Data
//! - RMC: Recommended Minimum Navigation Information
//! - GSV: Satellites in View
//! - GSA: GPS DOP and Active Satellites
//! - VTG: Track Made Good and Ground Speed
//! - GLL: Geographic Position - Latitude/Longitude
//! - ZDA: Time & Date
//! - HDT: Heading True
//! - DBT: Depth Below Transducer

use std::collections::HashMap;
use chrono::{NaiveTime, NaiveDate};

/// NMEA sentence types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NmeaSentenceType {
    GGA,  // Fix data
    RMC,  // Recommended minimum
    GSV,  // Satellites in view
    GSA,  // DOP and active satellites
    VTG,  // Track and ground speed
    GLL,  // Geographic position
    ZDA,  // Time and date
    HDT,  // Heading true
    DBT,  // Depth
    Unknown(String),
}

impl NmeaSentenceType {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GGA" | "GPGGA" | "GNGGA" => Self::GGA,
            "RMC" | "GPRMC" | "GNRMC" => Self::RMC,
            "GSV" | "GPGSV" | "GLGSV" | "GAGSV" => Self::GSV,
            "GSA" | "GPGSA" | "GNGSA" => Self::GSA,
            "VTG" | "GPVTG" | "GNVTG" => Self::VTG,
            "GLL" | "GPGLL" | "GNGLL" => Self::GLL,
            "ZDA" | "GPZDA" | "GNZDA" => Self::ZDA,
            "HDT" | "HEHDT" => Self::HDT,
            "DBT" | "SDDBT" => Self::DBT,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// GPS fix quality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpsFixQuality {
    #[default]
    Invalid = 0,
    GpsFix = 1,
    DgpsFix = 2,
    PpsFix = 3,
    Rtk = 4,
    FloatRtk = 5,
    Estimated = 6,
    Manual = 7,
    Simulation = 8,
}

impl From<u8> for GpsFixQuality {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Invalid,
            1 => Self::GpsFix,
            2 => Self::DgpsFix,
            3 => Self::PpsFix,
            4 => Self::Rtk,
            5 => Self::FloatRtk,
            6 => Self::Estimated,
            7 => Self::Manual,
            8 => Self::Simulation,
            _ => Self::Invalid,
        }
    }
}

/// GPS fix mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GpsFixMode {
    #[default]
    NotAvailable,
    Fix2D,
    Fix3D,
}

/// Geographic coordinate
#[derive(Debug, Clone, Copy, Default)]
pub struct Coordinate {
    pub degrees: f64,
    pub direction: char,  // N/S for lat, E/W for lon
}

impl Coordinate {
    /// Parse NMEA coordinate format (DDDMM.MMMM)
    pub fn parse(value: &str, direction: &str) -> Option<Self> {
        if value.is_empty() || direction.is_empty() {
            return None;
        }
        
        let value: f64 = value.parse().ok()?;
        let dir = direction.chars().next()?;
        
        // NMEA format: DDDMM.MMMM
        let degrees = (value / 100.0).floor();
        let minutes = value - (degrees * 100.0);
        let decimal_degrees = degrees + (minutes / 60.0);
        
        Some(Self {
            degrees: decimal_degrees,
            direction: dir,
        })
    }
    
    /// Get signed decimal degrees
    pub fn to_decimal(&self) -> f64 {
        match self.direction {
            'S' | 'W' => -self.degrees,
            _ => self.degrees,
        }
    }
}

/// Satellite information
#[derive(Debug, Clone, Default)]
pub struct SatelliteInfo {
    pub prn: u8,           // Satellite PRN number
    pub elevation: Option<u8>,  // Elevation in degrees
    pub azimuth: Option<u16>,   // Azimuth in degrees
    pub snr: Option<u8>,        // Signal-to-noise ratio
}

/// Parsed GGA sentence (Fix Data)
#[derive(Debug, Clone, Default)]
pub struct GgaData {
    pub time: Option<NaiveTime>,
    pub latitude: Option<Coordinate>,
    pub longitude: Option<Coordinate>,
    pub fix_quality: GpsFixQuality,
    pub satellites_used: u8,
    pub hdop: Option<f32>,
    pub altitude: Option<f32>,
    pub altitude_unit: char,
    pub geoid_separation: Option<f32>,
    pub geoid_unit: char,
    pub dgps_age: Option<f32>,
    pub dgps_station_id: Option<u16>,
}

/// Parsed RMC sentence (Recommended Minimum)
#[derive(Debug, Clone, Default)]
pub struct RmcData {
    pub time: Option<NaiveTime>,
    pub status: char,  // A=Active, V=Void
    pub latitude: Option<Coordinate>,
    pub longitude: Option<Coordinate>,
    pub speed_knots: Option<f32>,
    pub course: Option<f32>,
    pub date: Option<NaiveDate>,
    pub magnetic_variation: Option<f32>,
    pub magnetic_direction: char,
    pub mode: char,
}

/// Parsed GSV sentence (Satellites in View)
#[derive(Debug, Clone, Default)]
pub struct GsvData {
    pub total_messages: u8,
    pub message_number: u8,
    pub satellites_in_view: u8,
    pub satellites: Vec<SatelliteInfo>,
}

/// Parsed GSA sentence (DOP and Active Satellites)
#[derive(Debug, Clone, Default)]
pub struct GsaData {
    pub mode: char,  // M=Manual, A=Automatic
    pub fix_mode: GpsFixMode,
    pub satellite_prns: Vec<u8>,
    pub pdop: Option<f32>,
    pub hdop: Option<f32>,
    pub vdop: Option<f32>,
}

/// Parsed VTG sentence (Track and Ground Speed)
#[derive(Debug, Clone, Default)]
pub struct VtgData {
    pub track_true: Option<f32>,
    pub track_magnetic: Option<f32>,
    pub speed_knots: Option<f32>,
    pub speed_kmh: Option<f32>,
    pub mode: char,
}

/// Parsed GLL sentence (Geographic Position)
#[derive(Debug, Clone, Default)]
pub struct GllData {
    pub latitude: Option<Coordinate>,
    pub longitude: Option<Coordinate>,
    pub time: Option<NaiveTime>,
    pub status: char,
    pub mode: char,
}

/// Parsed ZDA sentence (Time and Date)
#[derive(Debug, Clone, Default)]
pub struct ZdaData {
    pub time: Option<NaiveTime>,
    pub day: Option<u8>,
    pub month: Option<u8>,
    pub year: Option<u16>,
    pub local_zone_hours: Option<i8>,
    pub local_zone_minutes: Option<u8>,
}

/// Parsed HDT sentence (Heading True)
#[derive(Debug, Clone, Default)]
pub struct HdtData {
    pub heading: Option<f32>,
}

/// Parsed DBT sentence (Depth Below Transducer)
#[derive(Debug, Clone, Default)]
pub struct DbtData {
    pub depth_feet: Option<f32>,
    pub depth_meters: Option<f32>,
    pub depth_fathoms: Option<f32>,
}

/// Generic NMEA sentence
#[derive(Debug, Clone)]
pub enum NmeaSentence {
    Gga(GgaData),
    Rmc(RmcData),
    Gsv(GsvData),
    Gsa(GsaData),
    Vtg(VtgData),
    Gll(GllData),
    Zda(ZdaData),
    Hdt(HdtData),
    Dbt(DbtData),
    Unknown { sentence_type: String, fields: Vec<String> },
}

/// NMEA parser errors
#[derive(Debug, Clone)]
pub enum NmeaError {
    InvalidFormat,
    ChecksumMismatch { expected: u8, got: u8 },
    ParseError(String),
}

/// NMEA 0183 Parser
#[derive(Debug, Default)]
pub struct NmeaParser {
    /// Accumulated satellite data from GSV messages
    pub satellites: Vec<SatelliteInfo>,
    /// Last parsed sentences by type
    pub last_data: HashMap<NmeaSentenceType, NmeaSentence>,
}

impl NmeaParser {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Calculate NMEA checksum
    pub fn calculate_checksum(data: &str) -> u8 {
        data.bytes().fold(0u8, |acc, b| acc ^ b)
    }
    
    /// Verify NMEA sentence checksum
    pub fn verify_checksum(sentence: &str) -> Result<bool, NmeaError> {
        if !sentence.starts_with('$') && !sentence.starts_with('!') {
            return Err(NmeaError::InvalidFormat);
        }
        
        let sentence = sentence.trim();
        
        if let Some(star_pos) = sentence.rfind('*') {
            let data = &sentence[1..star_pos];
            let checksum_str = &sentence[star_pos + 1..];
            
            let expected = u8::from_str_radix(checksum_str.trim(), 16)
                .map_err(|_| NmeaError::ParseError("Invalid checksum format".to_string()))?;
            
            let calculated = Self::calculate_checksum(data);
            
            if calculated != expected {
                return Err(NmeaError::ChecksumMismatch {
                    expected,
                    got: calculated,
                });
            }
            
            Ok(true)
        } else {
            // No checksum present
            Ok(false)
        }
    }
    
    /// Parse a single NMEA sentence
    pub fn parse(&mut self, sentence: &str) -> Result<NmeaSentence, NmeaError> {
        let sentence = sentence.trim();
        
        // Verify checksum if present
        let _ = Self::verify_checksum(sentence)?;
        
        // Remove checksum and leading $
        let data = if let Some(star_pos) = sentence.rfind('*') {
            &sentence[1..star_pos]
        } else {
            &sentence[1..]
        };
        
        let fields: Vec<&str> = data.split(',').collect();
        
        if fields.is_empty() {
            return Err(NmeaError::InvalidFormat);
        }
        
        let sentence_type = NmeaSentenceType::from_str(fields[0]);
        
        let parsed = match &sentence_type {
            NmeaSentenceType::GGA => self.parse_gga(&fields)?,
            NmeaSentenceType::RMC => self.parse_rmc(&fields)?,
            NmeaSentenceType::GSV => self.parse_gsv(&fields)?,
            NmeaSentenceType::GSA => self.parse_gsa(&fields)?,
            NmeaSentenceType::VTG => self.parse_vtg(&fields)?,
            NmeaSentenceType::GLL => self.parse_gll(&fields)?,
            NmeaSentenceType::ZDA => self.parse_zda(&fields)?,
            NmeaSentenceType::HDT => self.parse_hdt(&fields)?,
            NmeaSentenceType::DBT => self.parse_dbt(&fields)?,
            NmeaSentenceType::Unknown(t) => NmeaSentence::Unknown {
                sentence_type: t.clone(),
                fields: fields.iter().map(|s| s.to_string()).collect(),
            },
        };
        
        self.last_data.insert(sentence_type, parsed.clone());
        
        Ok(parsed)
    }
    
    /// Parse time from HHMMSS.sss format
    fn parse_time(s: &str) -> Option<NaiveTime> {
        if s.len() < 6 {
            return None;
        }
        
        let hours: u32 = s[0..2].parse().ok()?;
        let minutes: u32 = s[2..4].parse().ok()?;
        let seconds: f64 = s[4..].parse().ok()?;
        
        let secs = seconds.floor() as u32;
        let nanos = ((seconds - seconds.floor()) * 1_000_000_000.0) as u32;
        
        NaiveTime::from_hms_nano_opt(hours, minutes, secs, nanos)
    }
    
    /// Parse date from DDMMYY format
    fn parse_date(s: &str) -> Option<NaiveDate> {
        if s.len() < 6 {
            return None;
        }
        
        let day: u32 = s[0..2].parse().ok()?;
        let month: u32 = s[2..4].parse().ok()?;
        let year: i32 = s[4..6].parse().ok()?;
        
        // Assume 2000s for 2-digit year
        let full_year = if year > 80 { 1900 + year } else { 2000 + year };
        
        NaiveDate::from_ymd_opt(full_year, month, day)
    }
    
    fn parse_gga(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = GgaData::default();
        
        if fields.len() > 1 {
            data.time = Self::parse_time(fields[1]);
        }
        if fields.len() > 4 {
            data.latitude = Coordinate::parse(fields[2], fields[3]);
        }
        if fields.len() > 6 {
            data.longitude = Coordinate::parse(fields[4], fields[5]);
        }
        if fields.len() > 7 {
            data.fix_quality = fields[6].parse::<u8>().unwrap_or(0).into();
        }
        if fields.len() > 8 {
            data.satellites_used = fields[7].parse().unwrap_or(0);
        }
        if fields.len() > 9 {
            data.hdop = fields[8].parse().ok();
        }
        if fields.len() > 10 {
            data.altitude = fields[9].parse().ok();
        }
        if fields.len() > 11 {
            data.altitude_unit = fields[10].chars().next().unwrap_or('M');
        }
        if fields.len() > 12 {
            data.geoid_separation = fields[11].parse().ok();
        }
        if fields.len() > 13 {
            data.geoid_unit = fields[12].chars().next().unwrap_or('M');
        }
        if fields.len() > 14 {
            data.dgps_age = fields[13].parse().ok();
        }
        if fields.len() > 15 {
            data.dgps_station_id = fields[14].parse().ok();
        }
        
        Ok(NmeaSentence::Gga(data))
    }
    
    fn parse_rmc(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = RmcData::default();
        
        if fields.len() > 1 {
            data.time = Self::parse_time(fields[1]);
        }
        if fields.len() > 2 {
            data.status = fields[2].chars().next().unwrap_or('V');
        }
        if fields.len() > 5 {
            data.latitude = Coordinate::parse(fields[3], fields[4]);
        }
        if fields.len() > 7 {
            data.longitude = Coordinate::parse(fields[5], fields[6]);
        }
        if fields.len() > 8 {
            data.speed_knots = fields[7].parse().ok();
        }
        if fields.len() > 9 {
            data.course = fields[8].parse().ok();
        }
        if fields.len() > 10 {
            data.date = Self::parse_date(fields[9]);
        }
        if fields.len() > 11 {
            data.magnetic_variation = fields[10].parse().ok();
        }
        if fields.len() > 12 {
            data.magnetic_direction = fields[11].chars().next().unwrap_or(' ');
        }
        if fields.len() > 13 {
            data.mode = fields[12].chars().next().unwrap_or(' ');
        }
        
        Ok(NmeaSentence::Rmc(data))
    }
    
    fn parse_gsv(&mut self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = GsvData::default();
        
        if fields.len() > 1 {
            data.total_messages = fields[1].parse().unwrap_or(1);
        }
        if fields.len() > 2 {
            data.message_number = fields[2].parse().unwrap_or(1);
        }
        if fields.len() > 3 {
            data.satellites_in_view = fields[3].parse().unwrap_or(0);
        }
        
        // Parse satellite data (4 satellites per message max)
        let mut i = 4;
        while i + 3 < fields.len() {
            let sat = SatelliteInfo {
                prn: fields[i].parse().unwrap_or(0),
                elevation: fields.get(i + 1).and_then(|s| s.parse().ok()),
                azimuth: fields.get(i + 2).and_then(|s| s.parse().ok()),
                snr: fields.get(i + 3).and_then(|s| s.parse().ok()),
            };
            data.satellites.push(sat);
            i += 4;
        }
        
        // Accumulate satellites
        if data.message_number == 1 {
            self.satellites.clear();
        }
        self.satellites.extend(data.satellites.clone());
        
        Ok(NmeaSentence::Gsv(data))
    }
    
    fn parse_gsa(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = GsaData::default();
        
        if fields.len() > 1 {
            data.mode = fields[1].chars().next().unwrap_or('A');
        }
        if fields.len() > 2 {
            data.fix_mode = match fields[2].parse::<u8>().unwrap_or(1) {
                2 => GpsFixMode::Fix2D,
                3 => GpsFixMode::Fix3D,
                _ => GpsFixMode::NotAvailable,
            };
        }
        
        // Satellite PRNs (fields 3-14)
        for i in 3..=14 {
            if let Some(f) = fields.get(i) {
                if let Ok(prn) = f.parse::<u8>() {
                    if prn > 0 {
                        data.satellite_prns.push(prn);
                    }
                }
            }
        }
        
        if fields.len() > 15 {
            data.pdop = fields[15].parse().ok();
        }
        if fields.len() > 16 {
            data.hdop = fields[16].parse().ok();
        }
        if fields.len() > 17 {
            data.vdop = fields[17].parse().ok();
        }
        
        Ok(NmeaSentence::Gsa(data))
    }
    
    fn parse_vtg(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = VtgData::default();
        
        if fields.len() > 1 {
            data.track_true = fields[1].parse().ok();
        }
        if fields.len() > 3 {
            data.track_magnetic = fields[3].parse().ok();
        }
        if fields.len() > 5 {
            data.speed_knots = fields[5].parse().ok();
        }
        if fields.len() > 7 {
            data.speed_kmh = fields[7].parse().ok();
        }
        if fields.len() > 9 {
            data.mode = fields[9].chars().next().unwrap_or(' ');
        }
        
        Ok(NmeaSentence::Vtg(data))
    }
    
    fn parse_gll(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = GllData::default();
        
        if fields.len() > 4 {
            data.latitude = Coordinate::parse(fields[1], fields[2]);
            data.longitude = Coordinate::parse(fields[3], fields[4]);
        }
        if fields.len() > 5 {
            data.time = Self::parse_time(fields[5]);
        }
        if fields.len() > 6 {
            data.status = fields[6].chars().next().unwrap_or('V');
        }
        if fields.len() > 7 {
            data.mode = fields[7].chars().next().unwrap_or(' ');
        }
        
        Ok(NmeaSentence::Gll(data))
    }
    
    fn parse_zda(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = ZdaData::default();
        
        if fields.len() > 1 {
            data.time = Self::parse_time(fields[1]);
        }
        if fields.len() > 2 {
            data.day = fields[2].parse().ok();
        }
        if fields.len() > 3 {
            data.month = fields[3].parse().ok();
        }
        if fields.len() > 4 {
            data.year = fields[4].parse().ok();
        }
        if fields.len() > 5 {
            data.local_zone_hours = fields[5].parse().ok();
        }
        if fields.len() > 6 {
            data.local_zone_minutes = fields[6].parse().ok();
        }
        
        Ok(NmeaSentence::Zda(data))
    }
    
    fn parse_hdt(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = HdtData::default();
        
        if fields.len() > 1 {
            data.heading = fields[1].parse().ok();
        }
        
        Ok(NmeaSentence::Hdt(data))
    }
    
    fn parse_dbt(&self, fields: &[&str]) -> Result<NmeaSentence, NmeaError> {
        let mut data = DbtData::default();
        
        if fields.len() > 1 {
            data.depth_feet = fields[1].parse().ok();
        }
        if fields.len() > 3 {
            data.depth_meters = fields[3].parse().ok();
        }
        if fields.len() > 5 {
            data.depth_fathoms = fields[5].parse().ok();
        }
        
        Ok(NmeaSentence::Dbt(data))
    }
    
    /// Get current GPS position (from last GGA or RMC)
    pub fn get_position(&self) -> Option<(f64, f64)> {
        if let Some(NmeaSentence::Gga(gga)) = self.last_data.get(&NmeaSentenceType::GGA) {
            if let (Some(lat), Some(lon)) = (&gga.latitude, &gga.longitude) {
                return Some((lat.to_decimal(), lon.to_decimal()));
            }
        }
        
        if let Some(NmeaSentence::Rmc(rmc)) = self.last_data.get(&NmeaSentenceType::RMC) {
            if let (Some(lat), Some(lon)) = (&rmc.latitude, &rmc.longitude) {
                return Some((lat.to_decimal(), lon.to_decimal()));
            }
        }
        
        None
    }
    
    /// Get current speed in knots
    pub fn get_speed_knots(&self) -> Option<f32> {
        if let Some(NmeaSentence::Rmc(rmc)) = self.last_data.get(&NmeaSentenceType::RMC) {
            return rmc.speed_knots;
        }
        if let Some(NmeaSentence::Vtg(vtg)) = self.last_data.get(&NmeaSentenceType::VTG) {
            return vtg.speed_knots;
        }
        None
    }
    
    /// Get current course/heading
    pub fn get_course(&self) -> Option<f32> {
        if let Some(NmeaSentence::Rmc(rmc)) = self.last_data.get(&NmeaSentenceType::RMC) {
            return rmc.course;
        }
        if let Some(NmeaSentence::Vtg(vtg)) = self.last_data.get(&NmeaSentenceType::VTG) {
            return vtg.track_true;
        }
        if let Some(NmeaSentence::Hdt(hdt)) = self.last_data.get(&NmeaSentenceType::HDT) {
            return hdt.heading;
        }
        None
    }
    
    /// Get altitude in meters
    pub fn get_altitude(&self) -> Option<f32> {
        if let Some(NmeaSentence::Gga(gga)) = self.last_data.get(&NmeaSentenceType::GGA) {
            return gga.altitude;
        }
        None
    }
    
    /// Get fix quality
    pub fn get_fix_quality(&self) -> GpsFixQuality {
        if let Some(NmeaSentence::Gga(gga)) = self.last_data.get(&NmeaSentenceType::GGA) {
            return gga.fix_quality;
        }
        GpsFixQuality::Invalid
    }
    
    /// Get number of satellites used
    pub fn get_satellites_used(&self) -> u8 {
        if let Some(NmeaSentence::Gga(gga)) = self.last_data.get(&NmeaSentenceType::GGA) {
            return gga.satellites_used;
        }
        0
    }
    
    /// Format position as human-readable string
    pub fn format_position(&self) -> Option<String> {
        let (lat, lon) = self.get_position()?;
        
        let lat_dir = if lat >= 0.0 { "N" } else { "S" };
        let lon_dir = if lon >= 0.0 { "E" } else { "W" };
        
        Some(format!(
            "{:.6}° {}, {:.6}° {}",
            lat.abs(), lat_dir,
            lon.abs(), lon_dir
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gga_parse() {
        let mut parser = NmeaParser::new();
        let sentence = "$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,*47";
        
        let result = parser.parse(sentence);
        assert!(result.is_ok());
        
        if let Ok(NmeaSentence::Gga(gga)) = result {
            assert_eq!(gga.satellites_used, 8);
            assert!(gga.latitude.is_some());
            assert!(gga.longitude.is_some());
        }
    }
    
    #[test]
    fn test_rmc_parse() {
        let mut parser = NmeaParser::new();
        let sentence = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        
        let result = parser.parse(sentence);
        assert!(result.is_ok());
        
        if let Ok(NmeaSentence::Rmc(rmc)) = result {
            assert_eq!(rmc.status, 'A');
            assert!(rmc.speed_knots.is_some());
        }
    }
    
    #[test]
    fn test_checksum() {
        let checksum = NmeaParser::calculate_checksum("GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,");
        assert_eq!(checksum, 0x47);
    }
}

