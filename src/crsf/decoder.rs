//! # CRSF Packet Decoder
//!
//! Decodes CRSF telemetry packets (Link Statistics, Battery, GPS).

use super::crc::crc8_dvb_s2;
use super::protocol::*;
use crate::error::{FpvBridgeError, Result};

/// Decode a complete CRSF frame
///
/// # Arguments
///
/// * `frame` - Complete CRSF frame bytes (including sync, length, type, payload, crc)
///
/// # Returns
///
/// * `Result<CrsfFrame>` - Decoded frame, or error if invalid
///
/// # Errors
///
/// Returns error if:
/// - Frame is too short
/// - Sync byte is incorrect
/// - CRC check fails
pub fn decode_frame(frame: &[u8]) -> Result<CrsfFrame> {
    // Minimum frame size: sync(1) + length(1) + type(1) + crc(1) = 4 bytes
    if frame.len() < 4 {
        return Err(FpvBridgeError::CrsfProtocol(
            "Frame too short".to_string()
        ));
    }

    // Check sync byte
    if frame[0] != CRSF_SYNC_BYTE {
        return Err(FpvBridgeError::CrsfProtocol(
            format!("Invalid sync byte: 0x{:02X}", frame[0])
        ));
    }

    let length = frame[1] as usize;

    // Verify frame size matches length field
    // Frame should be: sync(1) + length(1) + [length bytes]
    // where [length bytes] = type(1) + payload(N) + crc(1)
    if frame.len() < 2 + length {
        return Err(FpvBridgeError::CrsfProtocol(
            format!("Frame too short: expected {} bytes, got {}", 2 + length, frame.len())
        ));
    }

    // Extract CRC (last byte after length field)
    let received_crc = frame[1 + length];

    // Calculate CRC over Length + Type + Payload (everything except Sync and CRC)
    let data_for_crc = &frame[1..1 + length];
    let calculated_crc = crc8_dvb_s2(data_for_crc);

    // Verify CRC
    if calculated_crc != received_crc {
        return Err(FpvBridgeError::CrsfProtocol(
            format!("CRC mismatch: expected 0x{:02X}, got 0x{:02X}", calculated_crc, received_crc)
        ));
    }

    // Extract type and payload
    let frame_type = frame[2]; // After sync and length
    let payload = frame[3..1 + length].to_vec(); // Between type and CRC

    Ok(CrsfFrame::new(frame_type, payload))
}

/// Decode Link Statistics telemetry packet
///
/// # Arguments
///
/// * `payload` - Link Statistics payload (10 bytes)
///
/// # Returns
///
/// * `Result<LinkStatistics>` - Decoded link statistics
pub fn decode_link_statistics(payload: &[u8]) -> Result<LinkStatistics> {
    if payload.len() < CRSF_LINK_STATS_PAYLOAD_SIZE {
        return Err(FpvBridgeError::CrsfProtocol(
            format!("Link stats payload too short: {} bytes", payload.len())
        ));
    }

    Ok(LinkStatistics {
        uplink_rssi_1: payload[0],
        uplink_rssi_2: payload[1],
        uplink_lq: payload[2],
        uplink_snr: payload[3] as i8,
        active_antenna: payload[4],
        rf_mode: payload[5],
        uplink_tx_power: payload[6],
        downlink_rssi: payload[7],
        downlink_lq: payload[8],
        downlink_snr: payload[9] as i8,
    })
}

/// Decode Battery Sensor telemetry packet
///
/// # Arguments
///
/// * `payload` - Battery Sensor payload (8 bytes)
///
/// # Returns
///
/// * `Result<BatterySensor>` - Decoded battery sensor data
pub fn decode_battery_sensor(payload: &[u8]) -> Result<BatterySensor> {
    if payload.len() < CRSF_BATTERY_SENSOR_PAYLOAD_SIZE {
        return Err(FpvBridgeError::CrsfProtocol(
            format!("Battery sensor payload too short: {} bytes", payload.len())
        ));
    }

    // Voltage: 2 bytes, little-endian, in centi-volts
    let voltage_cv = u16::from_be_bytes([payload[0], payload[1]]);
    let voltage = voltage_cv as f32 / 100.0;

    // Current: 2 bytes, little-endian, in deci-amps
    let current_da = u16::from_be_bytes([payload[2], payload[3]]);
    let current = current_da as f32 / 10.0;

    // Capacity: 3 bytes, big-endian, in mAh
    let capacity_used = u32::from_be_bytes([0, payload[4], payload[5], payload[6]]);

    // Remaining: 1 byte, percentage
    let remaining_percent = payload[7];

    Ok(BatterySensor {
        voltage,
        current,
        capacity_used,
        remaining_percent,
    })
}

/// Decode GPS telemetry packet
///
/// # Arguments
///
/// * `payload` - GPS payload (15 bytes)
///
/// # Returns
///
/// * `Result<GpsData>` - Decoded GPS data
pub fn decode_gps(payload: &[u8]) -> Result<GpsData> {
    if payload.len() < CRSF_GPS_PAYLOAD_SIZE {
        return Err(FpvBridgeError::CrsfProtocol(
            format!("GPS payload too short: {} bytes", payload.len())
        ));
    }

    // Latitude: 4 bytes, big-endian, degrees × 10^7
    let lat_raw = i32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
    let latitude = lat_raw as f64 / 10_000_000.0;

    // Longitude: 4 bytes, big-endian, degrees × 10^7
    let lon_raw = i32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
    let longitude = lon_raw as f64 / 10_000_000.0;

    // Ground speed: 2 bytes, big-endian, km/h × 10
    let speed_raw = u16::from_be_bytes([payload[8], payload[9]]);
    let ground_speed = speed_raw as f32 / 10.0;

    // Heading: 2 bytes, big-endian, degrees × 100
    let heading_raw = u16::from_be_bytes([payload[10], payload[11]]);
    let heading = heading_raw as f32 / 100.0;

    // Altitude: 2 bytes, big-endian, meters + 1000
    let altitude_raw = u16::from_be_bytes([payload[12], payload[13]]);
    let altitude = altitude_raw as i16 - 1000;

    // Satellites: 1 byte
    let satellites = payload[14];

    Ok(GpsData {
        latitude,
        longitude,
        ground_speed,
        heading,
        altitude,
        satellites,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crsf::encoder::encode_rc_channels_frame;

    #[test]
    fn test_decode_frame_too_short() {
        let frame = [CRSF_SYNC_BYTE, 0x03];
        let result = decode_frame(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_frame_invalid_sync() {
        let frame = [0xFF, 0x03, 0x16, 0x00];
        let result = decode_frame(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_valid_rc_channels_frame() {
        let channels = [CRSF_CHANNEL_VALUE_CENTER; CRSF_NUM_CHANNELS];
        let frame = encode_rc_channels_frame(&channels);

        let result = decode_frame(&frame);
        if let Err(ref e) = result {
            eprintln!("Decode error: {:?}", e);
            eprintln!("Frame bytes: {:02X?}", &frame);
        }
        assert!(result.is_ok(), "Decode failed: {:?}", result.err());

        let decoded = result.unwrap();
        assert_eq!(decoded.frame_type, CRSF_FRAMETYPE_RC_CHANNELS_PACKED);
        assert_eq!(decoded.payload.len(), 22);
    }

    #[test]
    fn test_decode_frame_crc_error() {
        let channels = [CRSF_CHANNEL_VALUE_CENTER; CRSF_NUM_CHANNELS];
        let mut frame = encode_rc_channels_frame(&channels);

        // Corrupt CRC
        frame[25] ^= 0xFF;

        let result = decode_frame(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_link_statistics() {
        let payload = vec![
            100u8, // uplink_rssi_1
            95,    // uplink_rssi_2
            80,    // uplink_lq (80%)
            10,    // uplink_snr
            0,     // active_antenna
            0,     // rf_mode
            20,    // uplink_tx_power
            90,    // downlink_rssi
            85,    // downlink_lq
            12,    // downlink_snr
        ];

        let result = decode_link_statistics(&payload);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.uplink_rssi_1, 100);
        assert_eq!(stats.uplink_lq, 80);
        assert_eq!(stats.uplink_snr, 10);
        assert_eq!(stats.downlink_rssi, 90);
    }

    #[test]
    fn test_decode_link_statistics_too_short() {
        let payload = vec![100u8; 5]; // Only 5 bytes
        let result = decode_link_statistics(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_battery_sensor() {
        // Voltage: 0x0419 = 1049 cV = 10.49V
        // Current: 0x007D = 125 dA = 12.5A
        // Capacity: 0x0003E8 = 1000 mAh
        // Remaining: 0x4B = 75%
        let payload = vec![
            0x04, 0x19, // Voltage: 1049 cV
            0x00, 0x7D, // Current: 125 dA
            0x00, 0x03, 0xE8, // Capacity: 1000 mAh
            0x4B, // Remaining: 75%
        ];

        let result = decode_battery_sensor(&payload);
        assert!(result.is_ok());

        let battery = result.unwrap();
        assert!((battery.voltage - 10.49).abs() < 0.01);
        assert!((battery.current - 12.5).abs() < 0.01);
        assert_eq!(battery.capacity_used, 1000);
        assert_eq!(battery.remaining_percent, 75);
    }

    #[test]
    fn test_decode_battery_sensor_too_short() {
        let payload = vec![0u8; 4]; // Only 4 bytes
        let result = decode_battery_sensor(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_gps() {
        // Latitude: 37.7749° N (San Francisco)
        // Longitude: -122.4194° W
        let lat_raw: i32 = 377_749_000; // 37.7749 × 10^7
        let lon_raw: i32 = -1_224_194_000; // -122.4194 × 10^7

        let payload = vec![
            // Latitude (4 bytes, big-endian)
            (lat_raw >> 24) as u8,
            (lat_raw >> 16) as u8,
            (lat_raw >> 8) as u8,
            lat_raw as u8,
            // Longitude (4 bytes, big-endian)
            (lon_raw >> 24) as u8,
            (lon_raw >> 16) as u8,
            (lon_raw >> 8) as u8,
            lon_raw as u8,
            // Ground speed: 25.5 km/h = 255 (× 10)
            0x00,
            0xFF,
            // Heading: 90.0° = 9000 (× 100)
            0x23,
            0x28,
            // Altitude: 100m = 1100 (+ 1000)
            0x04,
            0x4C,
            // Satellites: 12
            12,
        ];

        let result = decode_gps(&payload);
        assert!(result.is_ok());

        let gps = result.unwrap();
        assert!((gps.latitude - 37.7749).abs() < 0.0001);
        assert!((gps.longitude - (-122.4194)).abs() < 0.0001);
        assert!((gps.ground_speed - 25.5).abs() < 0.1);
        assert!((gps.heading - 90.0).abs() < 0.1);
        assert_eq!(gps.altitude, 100);
        assert_eq!(gps.satellites, 12);
    }

    #[test]
    fn test_decode_gps_too_short() {
        let payload = vec![0u8; 10]; // Only 10 bytes
        let result = decode_gps(&payload);
        assert!(result.is_err());
    }
}
