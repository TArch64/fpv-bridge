//! # CRSF Packet Encoder
//!
//! Encodes RC channels into CRSF protocol packets.

use super::crc::crc8_dvb_s2;
use super::protocol::*;

/// Encode RC channels into a complete CRSF frame
///
/// # Arguments
///
/// * `channels` - Array of 16 channel values (11-bit: 0-2047)
///
/// # Returns
///
/// * `Vec<u8>` - Complete CRSF frame (26 bytes: sync + length + type + 22-byte payload + crc)
///
/// # Examples
///
/// ```no_run
/// use fpv_bridge::crsf::encoder::encode_rc_channels_frame;
///
/// let channels = [1024u16; 16]; // All channels at center
/// let frame = encode_rc_channels_frame(&channels);
/// assert_eq!(frame.len(), 26);
/// ```
pub fn encode_rc_channels_frame(channels: &RcChannels) -> Vec<u8> {
    let payload = encode_rc_channels_payload(channels);

    // Build frame: Length + Type + Payload
    let mut frame_data = Vec::with_capacity(1 + 1 + payload.len());
    frame_data.push(CRSF_RC_CHANNELS_FRAME_LENGTH); // Length
    frame_data.push(CRSF_FRAMETYPE_RC_CHANNELS_PACKED); // Type
    frame_data.extend_from_slice(&payload); // Payload

    // Calculate CRC over Length + Type + Payload
    let crc = crc8_dvb_s2(&frame_data);

    // Build complete frame: Sync + Length + Type + Payload + CRC
    let mut complete_frame = Vec::with_capacity(26);
    complete_frame.push(CRSF_SYNC_BYTE); // Sync byte
    complete_frame.extend_from_slice(&frame_data); // Length + Type + Payload
    complete_frame.push(crc); // CRC

    complete_frame
}

/// Encode RC channels into payload (22 bytes)
///
/// Packs 16 channels (11 bits each) into 22 bytes using bit packing.
/// Channels are packed as a continuous bitstream, LSB first.
///
/// # Arguments
///
/// * `channels` - Array of 16 channel values (11-bit: 0-2047)
///
/// # Returns
///
/// * `Vec<u8>` - 22-byte payload
///
/// # Algorithm
///
/// Each channel is 11 bits (0-2047). Channels are packed LSB-first:
/// ```text
/// Byte 0: Ch1[0:7]
/// Byte 1: Ch1[8:10] | Ch2[0:4]
/// Byte 2: Ch2[5:10] | Ch3[0:1]
/// ...
/// ```
pub fn encode_rc_channels_payload(channels: &RcChannels) -> Vec<u8> {
    let mut payload = vec![0u8; CRSF_RC_CHANNELS_PAYLOAD_SIZE];
    let mut bit_index = 0;

    for &channel in channels.iter() {
        // Clamp channel value to 11-bit range
        let value = channel.min(CRSF_CHANNEL_VALUE_MAX);

        // Pack 11 bits
        for bit in 0..11 {
            if (value >> bit) & 1 == 1 {
                let byte_index = bit_index / 8;
                let bit_offset = bit_index % 8;
                payload[byte_index] |= 1 << bit_offset;
            }
            bit_index += 1;
        }
    }

    payload
}

/// Clamp a channel value to valid CRSF range (0-2047)
///
/// # Arguments
///
/// * `value` - Channel value to clamp
///
/// # Returns
///
/// * `u16` - Clamped value
pub fn clamp_channel_value(value: u16) -> u16 {
    value.min(CRSF_CHANNEL_VALUE_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_rc_channels_frame_length() {
        let channels = [CRSF_CHANNEL_VALUE_CENTER; CRSF_NUM_CHANNELS];
        let frame = encode_rc_channels_frame(&channels);

        // Frame should be 26 bytes: sync(1) + length(1) + type(1) + payload(22) + crc(1)
        assert_eq!(frame.len(), 26);
    }

    #[test]
    fn test_encode_rc_channels_frame_structure() {
        let channels = [CRSF_CHANNEL_VALUE_CENTER; CRSF_NUM_CHANNELS];
        let frame = encode_rc_channels_frame(&channels);

        // Check frame structure
        assert_eq!(frame[0], CRSF_SYNC_BYTE); // Sync byte
        assert_eq!(frame[1], CRSF_RC_CHANNELS_FRAME_LENGTH); // Length
        assert_eq!(frame[2], CRSF_FRAMETYPE_RC_CHANNELS_PACKED); // Type

        // Verify CRC is calculated (last byte should not be zero for non-zero data)
        assert_ne!(frame[25], 0x00);
    }

    #[test]
    fn test_encode_rc_channels_payload_length() {
        let channels = [0u16; CRSF_NUM_CHANNELS];
        let payload = encode_rc_channels_payload(&channels);

        // Payload should be exactly 22 bytes
        assert_eq!(payload.len(), CRSF_RC_CHANNELS_PAYLOAD_SIZE);
    }

    #[test]
    fn test_encode_rc_channels_all_zeros() {
        let channels = [0u16; CRSF_NUM_CHANNELS];
        let payload = encode_rc_channels_payload(&channels);

        // All zeros should produce all-zero payload
        assert_eq!(payload, vec![0u8; 22]);
    }

    #[test]
    fn test_encode_rc_channels_all_max() {
        let channels = [CRSF_CHANNEL_VALUE_MAX; CRSF_NUM_CHANNELS];
        let payload = encode_rc_channels_payload(&channels);

        // All max values should produce all-ones payload
        // 16 channels × 11 bits = 176 bits = 22 bytes
        assert_eq!(payload, vec![0xFFu8; 22]);
    }

    #[test]
    fn test_encode_rc_channels_center_values() {
        let channels = [CRSF_CHANNEL_VALUE_CENTER; CRSF_NUM_CHANNELS];
        let payload = encode_rc_channels_payload(&channels);

        // Center value is 1024 = 0b10000000000 (11 bits)
        // Should produce a specific pattern
        assert_eq!(payload.len(), 22);
        assert_ne!(payload, vec![0u8; 22]);
        assert_ne!(payload, vec![0xFFu8; 22]);
    }

    #[test]
    fn test_encode_rc_channels_single_channel() {
        let mut channels = [0u16; CRSF_NUM_CHANNELS];
        channels[0] = 0x7FF; // Max value (2047)

        let payload = encode_rc_channels_payload(&channels);

        // First 11 bits should be set
        // Byte 0: 0xFF (bits 0-7)
        // Byte 1: 0x07 (bits 8-10)
        assert_eq!(payload[0], 0xFF);
        assert_eq!(payload[1] & 0x07, 0x07);
    }

    #[test]
    fn test_encode_rc_channels_alternating() {
        let mut channels = [0u16; CRSF_NUM_CHANNELS];
        channels[0] = CRSF_CHANNEL_VALUE_MAX;
        channels[1] = 0;
        channels[2] = CRSF_CHANNEL_VALUE_MAX;
        channels[3] = 0;

        let payload = encode_rc_channels_payload(&channels);

        // Should produce alternating pattern
        assert_ne!(payload, vec![0u8; 22]);
        assert_ne!(payload, vec![0xFFu8; 22]);
    }

    #[test]
    fn test_clamp_channel_value() {
        assert_eq!(clamp_channel_value(0), 0);
        assert_eq!(clamp_channel_value(1024), 1024);
        assert_eq!(clamp_channel_value(2047), 2047);
        assert_eq!(clamp_channel_value(2048), 2047); // Should clamp
        assert_eq!(clamp_channel_value(5000), 2047); // Should clamp
        assert_eq!(clamp_channel_value(u16::MAX), 2047); // Should clamp
    }

    #[test]
    fn test_encode_rc_channels_clamping() {
        let mut channels = [0u16; CRSF_NUM_CHANNELS];
        channels[0] = 5000; // Over max

        let payload = encode_rc_channels_payload(&channels);

        // Should clamp to 2047 (0x7FF)
        assert_eq!(payload[0], 0xFF);
        assert_eq!(payload[1] & 0x07, 0x07);
    }

    #[test]
    fn test_encode_frame_different_data_different_crc() {
        let channels1 = [1000u16; CRSF_NUM_CHANNELS];
        let channels2 = [1500u16; CRSF_NUM_CHANNELS];

        let frame1 = encode_rc_channels_frame(&channels1);
        let frame2 = encode_rc_channels_frame(&channels2);

        // Frames should have different CRCs
        assert_ne!(frame1[25], frame2[25]);
    }
}
