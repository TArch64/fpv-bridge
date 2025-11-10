//! # CRC8-DVB-S2 Implementation
//!
//! CRC-8-DVB-S2 checksum calculation for CRSF protocol.
//!
//! **Polynomial**: 0xD5 (x^8 + x^7 + x^6 + x^4 + x^2 + 1)
//! **Initial Value**: 0x00

/// CRC-8-DVB-S2 polynomial
const CRC8_POLY: u8 = 0xD5;

/// Precomputed CRC8 lookup table for fast calculation
const CRC8_TABLE: [u8; 256] = generate_crc8_table();

/// Generate CRC8 lookup table at compile time
const fn generate_crc8_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0;

    while i < 256 {
        let mut crc = i as u8;
        let mut j = 0;

        while j < 8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ CRC8_POLY;
            } else {
                crc <<= 1;
            }
            j += 1;
        }

        table[i] = crc;
        i += 1;
    }

    table
}

/// Calculate CRC8-DVB-S2 checksum using lookup table (fast)
///
/// # Arguments
///
/// * `data` - Byte slice to calculate CRC for (Length + Type + Payload)
///
/// # Returns
///
/// * `u8` - Calculated CRC8 checksum
///
/// # Examples
///
/// ```no_run
/// use fpv_bridge::crsf::crc::crc8_dvb_s2;
///
/// let data = [0x18, 0x16, 0x00, 0x04];
/// let crc = crc8_dvb_s2(&data);
/// ```
pub fn crc8_dvb_s2(data: &[u8]) -> u8 {
    let mut crc: u8 = 0;

    for &byte in data {
        crc = CRC8_TABLE[(crc ^ byte) as usize];
    }

    crc
}

/// Calculate CRC8-DVB-S2 checksum using direct algorithm (slow, for verification)
///
/// This implementation is slower but easier to verify against the specification.
/// Used primarily for testing the lookup table implementation.
///
/// # Arguments
///
/// * `data` - Byte slice to calculate CRC for
///
/// # Returns
///
/// * `u8` - Calculated CRC8 checksum
#[allow(dead_code)]
fn crc8_dvb_s2_slow(data: &[u8]) -> u8 {
    let mut crc: u8 = 0;

    for &byte in data {
        crc ^= byte;

        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ CRC8_POLY;
            } else {
                crc <<= 1;
            }
        }
    }

    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc8_empty() {
        let data = [];
        assert_eq!(crc8_dvb_s2(&data), 0x00);
    }

    #[test]
    fn test_crc8_single_byte() {
        let data = [0x00];
        assert_eq!(crc8_dvb_s2(&data), 0x00);
        assert_eq!(crc8_dvb_s2(&data), crc8_dvb_s2_slow(&data));

        let data = [0xFF];
        let crc = crc8_dvb_s2(&data);
        assert_eq!(crc, crc8_dvb_s2_slow(&data)); // Verify fast matches slow
        assert_ne!(crc, 0x00); // Should produce non-zero CRC
    }

    #[test]
    fn test_crc8_known_vectors() {
        // Test vector from CRSF specification
        let data = [0x18, 0x16];
        let crc = crc8_dvb_s2(&data);
        assert_ne!(crc, 0x00); // Should produce non-zero CRC

        // Verify with slow implementation
        assert_eq!(crc, crc8_dvb_s2_slow(&data));
    }

    #[test]
    fn test_crc8_rc_channels_frame() {
        // Example RC channels frame (Length + Type + 22-byte payload)
        let mut data = vec![0x18, 0x16]; // Length = 24, Type = RC Channels
        data.extend_from_slice(&[0x00; 22]); // Empty channel data

        let crc = crc8_dvb_s2(&data);

        // Verify with slow implementation
        assert_eq!(crc, crc8_dvb_s2_slow(&data));
    }

    #[test]
    fn test_crc8_lookup_table_matches_slow() {
        // Verify lookup table implementation matches slow implementation
        let test_data = [
            vec![0x01, 0x02, 0x03],
            vec![0xFF, 0xFE, 0xFD],
            vec![0x18, 0x16, 0xE0, 0x03],
            vec![0x00; 24],
            vec![0xFF; 10],
        ];

        for data in test_data.iter() {
            assert_eq!(
                crc8_dvb_s2(data),
                crc8_dvb_s2_slow(data),
                "CRC mismatch for data: {:?}",
                data
            );
        }
    }

    #[test]
    fn test_crc8_changes_with_data() {
        let data1 = [0x18, 0x16, 0x00, 0x04];
        let data2 = [0x18, 0x16, 0x00, 0x05];

        let crc1 = crc8_dvb_s2(&data1);
        let crc2 = crc8_dvb_s2(&data2);

        assert_ne!(crc1, crc2, "CRC should change when data changes");
    }
}
