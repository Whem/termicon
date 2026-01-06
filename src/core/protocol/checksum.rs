//! Checksum calculation algorithms
//!
//! Supports: CRC-16 (Modbus, CCITT, XMODEM), CRC-32, XOR, LRC, Fletcher

/// Checksum algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumType {
    /// No checksum
    None,
    /// XOR of all bytes
    Xor,
    /// Longitudinal Redundancy Check (sum mod 256, negated)
    Lrc,
    /// Simple sum mod 256
    Sum8,
    /// Simple sum mod 65536
    Sum16,
    /// CRC-16 Modbus (polynomial 0x8005, init 0xFFFF, reflect)
    Crc16Modbus,
    /// CRC-16 CCITT (polynomial 0x1021, init 0xFFFF)
    Crc16Ccitt,
    /// CRC-16 XMODEM (polynomial 0x1021, init 0x0000)
    Crc16Xmodem,
    /// CRC-16 IBM (polynomial 0x8005, init 0x0000)
    Crc16Ibm,
    /// CRC-32 (IEEE 802.3)
    Crc32,
    /// Fletcher-16
    Fletcher16,
    /// Fletcher-32
    Fletcher32,
}

impl ChecksumType {
    /// Get all available checksum types
    pub fn all() -> &'static [ChecksumType] {
        &[
            ChecksumType::None,
            ChecksumType::Xor,
            ChecksumType::Lrc,
            ChecksumType::Sum8,
            ChecksumType::Sum16,
            ChecksumType::Crc16Modbus,
            ChecksumType::Crc16Ccitt,
            ChecksumType::Crc16Xmodem,
            ChecksumType::Crc16Ibm,
            ChecksumType::Crc32,
            ChecksumType::Fletcher16,
            ChecksumType::Fletcher32,
        ]
    }

    /// Get name of checksum type
    pub fn name(&self) -> &'static str {
        match self {
            ChecksumType::None => "None",
            ChecksumType::Xor => "XOR",
            ChecksumType::Lrc => "LRC",
            ChecksumType::Sum8 => "Sum-8",
            ChecksumType::Sum16 => "Sum-16",
            ChecksumType::Crc16Modbus => "CRC-16/Modbus",
            ChecksumType::Crc16Ccitt => "CRC-16/CCITT",
            ChecksumType::Crc16Xmodem => "CRC-16/XMODEM",
            ChecksumType::Crc16Ibm => "CRC-16/IBM",
            ChecksumType::Crc32 => "CRC-32",
            ChecksumType::Fletcher16 => "Fletcher-16",
            ChecksumType::Fletcher32 => "Fletcher-32",
        }
    }

    /// Get output size in bytes
    pub fn size(&self) -> usize {
        match self {
            ChecksumType::None => 0,
            ChecksumType::Xor | ChecksumType::Lrc | ChecksumType::Sum8 => 1,
            ChecksumType::Sum16 | ChecksumType::Crc16Modbus | ChecksumType::Crc16Ccitt |
            ChecksumType::Crc16Xmodem | ChecksumType::Crc16Ibm | ChecksumType::Fletcher16 => 2,
            ChecksumType::Crc32 | ChecksumType::Fletcher32 => 4,
        }
    }
}

/// Calculate checksum for data
pub fn calculate(data: &[u8], algorithm: ChecksumType) -> Vec<u8> {
    match algorithm {
        ChecksumType::None => Vec::new(),
        ChecksumType::Xor => vec![xor_checksum(data)],
        ChecksumType::Lrc => vec![lrc_checksum(data)],
        ChecksumType::Sum8 => vec![sum8_checksum(data)],
        ChecksumType::Sum16 => sum16_checksum(data).to_le_bytes().to_vec(),
        ChecksumType::Crc16Modbus => crc16_modbus(data).to_le_bytes().to_vec(),
        ChecksumType::Crc16Ccitt => crc16_ccitt(data).to_be_bytes().to_vec(),
        ChecksumType::Crc16Xmodem => crc16_xmodem(data).to_be_bytes().to_vec(),
        ChecksumType::Crc16Ibm => crc16_ibm(data).to_le_bytes().to_vec(),
        ChecksumType::Crc32 => crc32(data).to_le_bytes().to_vec(),
        ChecksumType::Fletcher16 => fletcher16(data).to_le_bytes().to_vec(),
        ChecksumType::Fletcher32 => fletcher32(data).to_le_bytes().to_vec(),
    }
}

/// Calculate and return checksum as u32
pub fn calculate_u32(data: &[u8], algorithm: ChecksumType) -> u32 {
    match algorithm {
        ChecksumType::None => 0,
        ChecksumType::Xor => xor_checksum(data) as u32,
        ChecksumType::Lrc => lrc_checksum(data) as u32,
        ChecksumType::Sum8 => sum8_checksum(data) as u32,
        ChecksumType::Sum16 => sum16_checksum(data) as u32,
        ChecksumType::Crc16Modbus => crc16_modbus(data) as u32,
        ChecksumType::Crc16Ccitt => crc16_ccitt(data) as u32,
        ChecksumType::Crc16Xmodem => crc16_xmodem(data) as u32,
        ChecksumType::Crc16Ibm => crc16_ibm(data) as u32,
        ChecksumType::Crc32 => crc32(data),
        ChecksumType::Fletcher16 => fletcher16(data) as u32,
        ChecksumType::Fletcher32 => fletcher32(data),
    }
}

/// Verify checksum
pub fn verify(data: &[u8], checksum: &[u8], algorithm: ChecksumType) -> bool {
    let calculated = calculate(data, algorithm);
    calculated == checksum
}

// ============ Individual checksum implementations ============

/// XOR checksum - XOR of all bytes
pub fn xor_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc ^ b)
}

/// LRC (Longitudinal Redundancy Check)
/// Sum of all bytes, then two's complement
pub fn lrc_checksum(data: &[u8]) -> u8 {
    let sum: u8 = data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    (!sum).wrapping_add(1)
}

/// Simple 8-bit sum
pub fn sum8_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/// Simple 16-bit sum
pub fn sum16_checksum(data: &[u8]) -> u16 {
    data.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16))
}

/// CRC-16/Modbus
/// Polynomial: 0x8005, Init: 0xFFFF, RefIn: true, RefOut: true, XorOut: 0x0000
pub fn crc16_modbus(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    
    crc
}

/// CRC-16/CCITT (Kermit)
/// Polynomial: 0x1021, Init: 0xFFFF, RefIn: false, RefOut: false
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    
    crc
}

/// CRC-16/XMODEM
/// Polynomial: 0x1021, Init: 0x0000, RefIn: false, RefOut: false
pub fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
    
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    
    crc
}

/// CRC-16/IBM (CRC-16/ARC)
/// Polynomial: 0x8005, Init: 0x0000, RefIn: true, RefOut: true
pub fn crc16_ibm(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
    
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    
    crc
}

/// CRC-32 (IEEE 802.3, used in Ethernet, ZIP, etc.)
/// Polynomial: 0x04C11DB7, Init: 0xFFFFFFFF, RefIn: true, RefOut: true, XorOut: 0xFFFFFFFF
pub fn crc32(data: &[u8]) -> u32 {
    // Pre-computed table for polynomial 0xEDB88320 (reflected 0x04C11DB7)
    const TABLE: [u32; 256] = [
        0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA, 0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
        0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988, 0x09B64C2B, 0x7EB17CBD, 0xE7B82D07, 0x90BF1D91,
        0x1DB71064, 0x6AB020F2, 0xF3B97148, 0x84BE41DE, 0x1ADAD47D, 0x6DDDE4EB, 0xF4D4B551, 0x83D385C7,
        0x136C9856, 0x646BA8C0, 0xFD62F97A, 0x8A65C9EC, 0x14015C4F, 0x63066CD9, 0xFA0F3D63, 0x8D080DF5,
        0x3B6E20C8, 0x4C69105E, 0xD56041E4, 0xA2677172, 0x3C03E4D1, 0x4B04D447, 0xD20D85FD, 0xA50AB56B,
        0x35B5A8FA, 0x42B2986C, 0xDBBBC9D6, 0xACBCF940, 0x32D86CE3, 0x45DF5C75, 0xDCD60DCF, 0xABD13D59,
        0x26D930AC, 0x51DE003A, 0xC8D75180, 0xBFD06116, 0x21B4F4B5, 0x56B3C423, 0xCFBA9599, 0xB8BDA50F,
        0x2802B89E, 0x5F058808, 0xC60CD9B2, 0xB10BE924, 0x2F6F7C87, 0x58684C11, 0xC1611DAB, 0xB6662D3D,
        0x76DC4190, 0x01DB7106, 0x98D220BC, 0xEFD5102A, 0x71B18589, 0x06B6B51F, 0x9FBFE4A5, 0xE8B8D433,
        0x7807C9A2, 0x0F00F934, 0x9609A88E, 0xE10E9818, 0x7F6A0DBB, 0x086D3D2D, 0x91646C97, 0xE6635C01,
        0x6B6B51F4, 0x1C6C6162, 0x856530D8, 0xF262004E, 0x6C0695ED, 0x1B01A57B, 0x8208F4C1, 0xF50FC457,
        0x65B0D9C6, 0x12B7E950, 0x8BBEB8EA, 0xFCB9887C, 0x62DD1DDF, 0x15DA2D49, 0x8CD37CF3, 0xFBD44C65,
        0x4DB26158, 0x3AB551CE, 0xA3BC0074, 0xD4BB30E2, 0x4ADFA541, 0x3DD895D7, 0xA4D1C46D, 0xD3D6F4FB,
        0x4369E96A, 0x346ED9FC, 0xAD678846, 0xDA60B8D0, 0x44042D73, 0x33031DE5, 0xAA0A4C5F, 0xDD0D7CC9,
        0x5005713C, 0x270241AA, 0xBE0B1010, 0xC90C2086, 0x5768B525, 0x206F85B3, 0xB966D409, 0xCE61E49F,
        0x5EDEF90E, 0x29D9C998, 0xB0D09822, 0xC7D7A8B4, 0x59B33D17, 0x2EB40D81, 0xB7BD5C3B, 0xC0BA6CAD,
        0xEDB88320, 0x9ABFB3B6, 0x03B6E20C, 0x74B1D29A, 0xEAD54739, 0x9DD277AF, 0x04DB2615, 0x73DC1683,
        0xE3630B12, 0x94643B84, 0x0D6D6A3E, 0x7A6A5AA8, 0xE40ECF0B, 0x9309FF9D, 0x0A00AE27, 0x7D079EB1,
        0xF00F9344, 0x8708A3D2, 0x1E01F268, 0x6906C2FE, 0xF762575D, 0x806567CB, 0x196C3671, 0x6E6B06E7,
        0xFED41B76, 0x89D32BE0, 0x10DA7A5A, 0x67DD4ACC, 0xF9B9DF6F, 0x8EBEEFF9, 0x17B7BE43, 0x60B08ED5,
        0xD6D6A3E8, 0xA1D1937E, 0x38D8C2C4, 0x4FDFF252, 0xD1BB67F1, 0xA6BC5767, 0x3FB506DD, 0x48B2364B,
        0xD80D2BDA, 0xAF0A1B4C, 0x36034AF6, 0x41047A60, 0xDF60EFC3, 0xA867DF55, 0x316E8EEF, 0x4669BE79,
        0xCB61B38C, 0xBC66831A, 0x256FD2A0, 0x5268E236, 0xCC0C7795, 0xBB0B4703, 0x220216B9, 0x5505262F,
        0xC5BA3BBE, 0xB2BD0B28, 0x2BB45A92, 0x5CB36A04, 0xC2D7FFA7, 0xB5D0CF31, 0x2CD99E8B, 0x5BDEAE1D,
        0x9B64C2B0, 0xEC63F226, 0x756AA39C, 0x026D930A, 0x9C0906A9, 0xEB0E363F, 0x72076785, 0x05005713,
        0x95BF4A82, 0xE2B87A14, 0x7BB12BAE, 0x0CB61B38, 0x92D28E9B, 0xE5D5BE0D, 0x7CDCEFB7, 0x0BDBDF21,
        0x86D3D2D4, 0xF1D4E242, 0x68DDB3F8, 0x1FDA836E, 0x81BE16CD, 0xF6B9265B, 0x6FB077E1, 0x18B74777,
        0x88085AE6, 0xFF0F6A70, 0x66063BCA, 0x11010B5C, 0x8F659EFF, 0xF862AE69, 0x616BFFD3, 0x166CCF45,
        0xA00AE278, 0xD70DD2EE, 0x4E048354, 0x3903B3C2, 0xA7672661, 0xD06016F7, 0x4969474D, 0x3E6E77DB,
        0xAED16A4A, 0xD9D65ADC, 0x40DF0B66, 0x37D83BF0, 0xA9BCAE53, 0xDEBB9EC5, 0x47B2CF7F, 0x30B5FFE9,
        0xBDBDF21C, 0xCABAC28A, 0x53B39330, 0x24B4A3A6, 0xBAD03605, 0xCDD706B3, 0x54DE5729, 0x23D967BF,
        0xB3667A2E, 0xC4614AB8, 0x5D681B02, 0x2A6F2B94, 0xB40BBE37, 0xC30C8EA1, 0x5A05DF1B, 0x2D02EF8D,
    ];

    let mut crc: u32 = 0xFFFFFFFF;
    
    for &byte in data {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = TABLE[index] ^ (crc >> 8);
    }
    
    crc ^ 0xFFFFFFFF
}

/// Fletcher-16 checksum
pub fn fletcher16(data: &[u8]) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;
    
    for &byte in data {
        sum1 = (sum1 + byte as u16) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    
    (sum2 << 8) | sum1
}

/// Fletcher-32 checksum
pub fn fletcher32(data: &[u8]) -> u32 {
    let mut sum1: u32 = 0;
    let mut sum2: u32 = 0;
    
    // Process as 16-bit words
    for chunk in data.chunks(2) {
        let word = if chunk.len() == 2 {
            (chunk[0] as u32) | ((chunk[1] as u32) << 8)
        } else {
            chunk[0] as u32
        };
        sum1 = (sum1 + word) % 65535;
        sum2 = (sum2 + sum1) % 65535;
    }
    
    (sum2 << 16) | sum1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor() {
        assert_eq!(xor_checksum(&[0x01, 0x02, 0x03]), 0x00);
        assert_eq!(xor_checksum(&[0xFF, 0x00]), 0xFF);
    }

    #[test]
    fn test_crc16_modbus() {
        // Test vector: "123456789" should give 0x4B37
        let data = b"123456789";
        assert_eq!(crc16_modbus(data), 0x4B37);
    }

    #[test]
    fn test_crc16_xmodem() {
        // Test vector: "123456789" should give 0x31C3
        let data = b"123456789";
        assert_eq!(crc16_xmodem(data), 0x31C3);
    }

    #[test]
    fn test_crc32() {
        // Test vector: "123456789" should give 0xCBF43926
        let data = b"123456789";
        assert_eq!(crc32(data), 0xCBF43926);
    }

    #[test]
    fn test_fletcher16() {
        let data = b"abcde";
        let result = fletcher16(data);
        assert!(result != 0); // Basic sanity check
    }
}
