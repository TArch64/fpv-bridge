# CRSF Protocol Specification

This document describes the Crossfire (CRSF) protocol used by ExpressLRS and implemented in FPV Bridge.

---

## Table of Contents

1. [Overview](#overview)
2. [Frame Structure](#frame-structure)
3. [RC Channels Packet](#rc-channels-packet)
4. [Telemetry Packets](#telemetry-packets)
5. [CRC8 Checksum](#crc8-checksum)
6. [Implementation Details](#implementation-details)
7. [Examples](#examples)

---

## Overview

**CRSF (Crossfire)** is a serial communication protocol developed by Team BlackSheep (TBS) for RC control and telemetry. ExpressLRS adopted CRSF as its primary protocol.

### Key Characteristics

- **Serial Interface**: UART at 420,000 baud (8N1)
- **Bidirectional**: TX sends RC channels, RX sends telemetry
- **Packet-Based**: Variable-length frames with CRC
- **Efficient**: 11-bit channel resolution, packed encoding
- **Extensible**: Multiple packet types for different data

### Packet Types (Common)

| Type | Hex  | Name | Direction | Description |
|------|------|------|-----------|-------------|
| 0x02 | 0x02 | GPS | RX → TX | GPS coordinates, speed, sats |
| 0x08 | 0x08 | Battery Sensor | RX → TX | Voltage, current, capacity |
| 0x14 | 0x14 | Link Statistics | RX → TX | RSSI, LQ, SNR |
| 0x16 | 0x16 | RC Channels Packed | TX → RX | 16 RC channels (11-bit) |
| 0x1E | 0x1E | Attitude | RX → TX | Pitch, roll, yaw |

---

## Frame Structure

All CRSF frames follow this structure:

```text
┌──────────┬──────────┬──────────┬─────────────────┬──────────┐
│   SYNC   │  LENGTH  │   TYPE   │     PAYLOAD     │   CRC8   │
│  (0xC8)  │  (N+2)   │  (0xXX)  │   (N bytes)     │          │
└──────────┴──────────┴──────────┴─────────────────┴──────────┘
    1 byte     1 byte     1 byte      N bytes        1 byte
```

### Field Descriptions

**1. Sync Byte (0xC8)**
- **Value**: Always `0xC8`
- **Purpose**: Frame synchronization marker
- **Note**: Not included in CRC calculation

**2. Length**
- **Value**: `N + 2` (payload size + type + CRC)
- **Range**: 0x03 to 0x40 (3 to 64 bytes)
- **Note**: Does NOT include sync byte and length byte itself
- **Included in CRC**: Yes (starting from this byte)

**3. Type**
- **Value**: Packet type identifier (see table above)
- **Included in CRC**: Yes

**4. Payload**
- **Value**: Packet-specific data
- **Length**: Variable (0 to 62 bytes typically)
- **Included in CRC**: Yes

**5. CRC8**
- **Algorithm**: CRC-8-DVB-S2 (polynomial 0xD5)
- **Range**: Length + Type + Payload
- **Purpose**: Error detection

---

## RC Channels Packet

**Type**: `0x16` (RC Channels Packed)

### Purpose

Transmits 16 RC channel values from controller to flight controller.

### Frame Structure

```text
Total: 26 bytes

┌──────┬──────┬──────┬────────────────────────────┬──────┐
│ 0xC8 │ 0x18 │ 0x16 │   22 bytes (payload)       │ CRC8 │
└──────┴──────┴──────┴────────────────────────────┴──────┘
 Sync   Len    Type         Packed Channels         CRC
```

**Length**: `0x18` = 24 bytes = 1 (type) + 22 (payload) + 1 (CRC)

### Payload Encoding (22 bytes)

Each channel is **11 bits** (0-2047), representing 988-2012μs:
- **Min Value**: 0 → 988μs
- **Center Value**: 1024 → 1500μs
- **Max Value**: 2047 → 2012μs

16 channels × 11 bits = 176 bits = 22 bytes

**Packing Algorithm**:

Channels are packed as a continuous bitstream, LSB first:

```text
Byte 0: Ch1[0:7]
Byte 1: Ch1[8:10] | Ch2[0:4]
Byte 2: Ch2[5:10] | Ch3[0:1]
Byte 3: Ch3[2:9]
Byte 4: Ch3[10] | Ch4[0:6]
...and so on
```

### Encoding Example (Rust)

```rust
fn encode_rc_channels(channels: &[u16; 16]) -> Vec<u8> {
    let mut payload = vec![0u8; 22];
    let mut bit_index = 0;

    for &channel in channels {
        let value = channel & 0x7FF; // 11-bit mask

        for bit in 0..11 {
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;

            if (value >> bit) & 1 == 1 {
                payload[byte_index] |= 1 << bit_offset;
            }

            bit_index += 1;
        }
    }

    payload
}
```

### Channel Mapping (Standard)

| Channel | Function | Typical Input | Range (μs) |
|---------|----------|---------------|------------|
| CH1 | Aileron (Roll) | Right Stick X | 1000-2000 |
| CH2 | Elevator (Pitch) | Right Stick Y | 1000-2000 |
| CH3 | Throttle | Left Stick Y | 1000-2000 |
| CH4 | Rudder (Yaw) | Left Stick X | 1000-2000 |
| CH5 | ARM | Switch/Button | 1000 or 2000 |
| CH6 | Flight Mode | Switch | 1000/1500/2000 |
| CH7-16 | Aux Channels | Buttons/Switches | Varies |

---

## Telemetry Packets

### Link Statistics (0x14)

**Purpose**: Signal quality metrics

**Payload Structure** (10 bytes):

```text
Offset | Size | Field            | Unit/Range
-------|------|------------------|------------------
   0   |  1   | Uplink RSSI 1    | dBm (0-255, 0xFF = invalid)
   1   |  1   | Uplink RSSI 2    | dBm (diversity antenna)
   2   |  1   | Uplink LQ        | % (0-100)
   3   |  1   | Uplink SNR       | dB (-128 to 127)
   4   |  1   | Active Antenna   | 0 or 1
   5   |  1   | RF Mode          | Packet rate
   6   |  1   | Uplink TX Power  | mW (encoded)
   7   |  1   | Downlink RSSI    | dBm
   8   |  1   | Downlink LQ      | %
   9   |  1   | Downlink SNR     | dB
```

**Example Decoding**:

```rust
struct LinkStats {
    uplink_rssi: i8,      // dBm (negative value)
    uplink_lq: u8,        // % (0-100)
    uplink_snr: i8,       // dB
    downlink_rssi: i8,    // dBm
    downlink_lq: u8,      // %
}

fn decode_link_stats(payload: &[u8]) -> LinkStats {
    LinkStats {
        uplink_rssi: -(payload[0] as i8),  // Negate to get -dBm
        uplink_lq: payload[2],
        uplink_snr: payload[3] as i8,
        downlink_rssi: -(payload[7] as i8),
        downlink_lq: payload[8],
    }
}
```

### Battery Sensor (0x08)

**Payload Structure** (8 bytes):

```
Offset | Size | Field            | Unit
-------|------|------------------|------------------
   0   |  2   | Voltage          | cV (centi-volts, /100 for V)
   2   |  2   | Current          | dA (deci-amps, /10 for A)
   4   |  3   | Capacity Used    | mAh
   7   |  1   | Remaining %      | % (0-100)
```

**Example**:

```text
Voltage: 0x0419 = 1049 cV = 10.49V
Current: 0x007D = 125 dA = 12.5A
Capacity: 0x0003E8 = 1000 mAh
Remaining: 0x4B = 75%
```

### GPS (0x02)

**Payload Structure** (15 bytes):

```
Offset | Size | Field            | Unit
-------|------|------------------|------------------
   0   |  4   | Latitude         | degrees × 10^7
   4   |  4   | Longitude        | degrees × 10^7
   8   |  2   | Ground Speed     | km/h × 10
  10   |  2   | Heading          | degrees × 100
  12   |  2   | Altitude         | meters + 1000
  14   |  1   | Satellites       | count
```

**Example**:

```text
Latitude: 0x164F7B88 = 375432072 → 37.5432072° N
Longitude: 0xF8B72F00 = -122419200 → -122.4192° W
Altitude: 0x046C = 1132 → 132m (subtract 1000)
Satellites: 0x0C = 12
```

---

## CRC8 Checksum

### Algorithm: CRC-8-DVB-S2

**Polynomial**: `0xD5` (x^8 + x^7 + x^6 + x^4 + x^2 + 1)

**Initial Value**: `0x00`

**Input**: Length + Type + Payload (everything except Sync and CRC itself)

### Implementation (Rust)

```rust
const CRC8_POLY: u8 = 0xD5;

fn crc8_dvb_s2(data: &[u8]) -> u8 {
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
```

### Lookup Table Optimization

```rust
const CRC8_TABLE: [u8; 256] = [
    0x00, 0xD5, 0x7F, 0xAA, 0xFE, 0x2B, 0x81, 0x54,
    // ... (pre-computed table)
];

fn crc8_dvb_s2_fast(data: &[u8]) -> u8 {
    let mut crc: u8 = 0;
    for &byte in data {
        crc = CRC8_TABLE[(crc ^ byte) as usize];
    }
    crc
}
```

### Verification

Test vector:

```text
Input:  [0x18, 0x16, 0x00, 0x04, 0x00, ...(22 bytes)...]
CRC8:   0x?? (calculated)
```

**Validation**:
- Calculate CRC8 on received data (Length through Payload)
- Compare with received CRC byte
- If match → valid packet
- If mismatch → discard packet

---

## Implementation Details

### Baud Rate: 420,000

**Why 420,000?**
- Fast enough for 250Hz packet rate
- Low enough for reliable serial communication
- Standard for CRSF/ELRS

**Serial Settings**:

```text
Baud Rate: 420,000
Data Bits: 8
Parity:    None
Stop Bits: 1
Flow Control: None
```

### Packet Rate: 250Hz

**Interval**: 4ms (250 packets/second)

**Timing**:
```rust
use tokio::time::{interval, Duration};

let mut tick = interval(Duration::from_millis(4));

loop {
    tick.tick().await;
    send_rc_channels_packet().await?;
}
```

### Buffer Sizes

**TX Buffer**: 64 bytes (single packet max)
**RX Buffer**: 256 bytes (multiple telemetry packets)

### Frame Synchronization

**Challenge**: Detecting frame boundaries in byte stream

**Strategy**:
1. Search for sync byte (`0xC8`)
2. Read length byte
3. Validate length (0x03 to 0x40)
4. Read remaining bytes
5. Verify CRC
6. If CRC fails, continue searching for next sync byte

```rust
async fn read_frame(reader: &mut SerialPort) -> Result<Vec<u8>> {
    let mut buffer = [0u8; 256];
    let mut frame = Vec::new();

    loop {
        // Read byte by byte until sync
        reader.read_exact(&mut buffer[..1]).await?;
        if buffer[0] != 0xC8 {
            continue;
        }

        // Read length
        reader.read_exact(&mut buffer[..1]).await?;
        let length = buffer[0] as usize;

        if length < 3 || length > 64 {
            continue; // Invalid length
        }

        // Read rest of frame
        reader.read_exact(&mut buffer[..length]).await?;

        // Verify CRC
        let crc_calculated = crc8_dvb_s2(&buffer[..length - 1]);
        let crc_received = buffer[length - 1];

        if crc_calculated == crc_received {
            frame.extend_from_slice(&buffer[..length]);
            return Ok(frame);
        }

        // CRC failed, keep searching
    }
}
```

---

## Examples

### Example 1: RC Channels Packet (All Centered)

All 16 channels at center (1500μs → 1024 in 11-bit):

**Channel Values** (11-bit):
```
CH1-16: 1024 (0x400)
```

**Encoded Payload** (22 bytes):

```text
0x00 0x04 0x00 0x04 0x00 0x04 0x00 0x04
0x00 0x04 0x00 0x04 0x00 0x04 0x00 0x04
0x00 0x04 0x00 0x04 0x00 0x04
```

**Full Frame**:

```text
Sync:    0xC8
Length:  0x18 (24)
Type:    0x16
Payload: 0x00 0x04 0x00 0x04... (22 bytes)
CRC8:    0x?? (calculated)

Complete: C8 18 16 00 04 00 04 00 04 00 04 00 04 00 04 00 04 00 04 00 04 00 04 00 04 [CRC]
```

### Example 2: RC Channels with ARM

```text
CH1 (Roll):     1500μs → 1024 (0x400)
CH2 (Pitch):    1500μs → 1024 (0x400)
CH3 (Throttle): 1000μs → 0    (0x000)
CH4 (Yaw):      1500μs → 1024 (0x400)
CH5 (ARM):      2000μs → 2047 (0x7FF)
CH6-16:         1500μs → 1024 (0x400)
```

**Bit Packing**:

```text
CH1: 00000000100 (0x400)
CH2: 00000000100 (0x400)
CH3: 00000000000 (0x000)
CH4: 00000000100 (0x400)
CH5: 11111111111 (0x7FF)
...
```

### Example 3: Link Statistics Packet

**Received Bytes**:

```text
C8 0C 14 5A 5A 64 0A 00 02 32 5C 62 08 [CRC]
```

**Decoding**:

```text
Sync:    0xC8
Length:  0x0C (12 bytes)
Type:    0x14 (Link Stats)

Payload:
  RSSI 1:       0x5A (90) → -90 dBm
  RSSI 2:       0x5A (90) → -90 dBm
  Uplink LQ:    0x64 (100) → 100%
  SNR:          0x0A (10) → 10 dB
  Active Ant:   0x00 → Antenna 1
  RF Mode:      0x02 → 250Hz
  TX Power:     0x32 (50) → 100mW
  Downlink RSSI: 0x5C (92) → -92 dBm
  Downlink LQ:  0x62 (98) → 98%
  Downlink SNR: 0x08 (8) → 8 dB

CRC8: (verify)
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc8() {
        let data = [0x18, 0x16, 0x00, 0x04];
        let crc = crc8_dvb_s2(&data);
        assert_eq!(crc, /* expected value */);
    }

    #[test]
    fn test_channel_encoding() {
        let channels = [1024; 16]; // All centered
        let payload = encode_rc_channels(&channels);
        assert_eq!(payload.len(), 22);

        // Decode and verify
        let decoded = decode_rc_channels(&payload);
        assert_eq!(decoded, channels);
    }

    #[test]
    fn test_frame_building() {
        let channels = [1024; 16];
        let frame = build_rc_channels_frame(&channels);

        assert_eq!(frame[0], 0xC8); // Sync
        assert_eq!(frame[1], 0x18); // Length
        assert_eq!(frame[2], 0x16); // Type
        assert_eq!(frame.len(), 26); // Total length
    }
}
```

---

## References

- [CRSF Protocol Wiki](https://github.com/crsf-wg/crsf/wiki)
- [ExpressLRS Documentation](https://www.expresslrs.org/3.0/info/signal-health/)
- [Betaflight CRSF Implementation](https://github.com/betaflight/betaflight/blob/master/src/main/rx/crsf.c)
- [CRC-8-DVB-S2 Spec](https://en.wikipedia.org/wiki/Cyclic_redundancy_check)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
