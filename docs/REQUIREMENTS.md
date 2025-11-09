# FPV Bridge - Project Requirements

## Document Information

- **Project Name**: FPV Bridge
- **Version**: 1.0.0
- **Last Updated**: 2025-11-09
- **Status**: Initial Requirements

---

## 1. Project Overview

### 1.1 Purpose
The FPV Bridge is a software application that enables control of a Meteor75 Pro ELRS 2.4GHz drone using a PlayStation 5 DualSense controller via a Raspberry Pi Zero 2 W. The bridge translates controller inputs into CRSF (Crossfire) protocol packets and transmits them through an ExpressLRS transmitter module.

### 1.2 Goals
- Provide an affordable alternative to traditional RC transmitters
- Enable intuitive drone control using familiar gaming controller
- Support telemetry logging for flight analysis and debugging
- Create an extensible platform for future enhancements
- Deliver a reliable, low-latency control system

### 1.3 Use Case
**Primary Use**: Casual and freestyle FPV flying (non-competitive)
- **NOT** designed for racing (latency requirements are relaxed)
- Suitable for line-of-sight and short-range FPV
- Educational and hobbyist applications
- Flight testing and tuning

### 1.4 Target Audience
- DIY drone enthusiasts
- FPV hobbyists
- Software developers interested in drone control
- Users seeking cost-effective RC solutions

---

## 2. Hardware Requirements

### 2.1 Computing Platform

**Component**: Raspberry Pi Zero 2 W

**Specifications**:
- **CPU**: 1GHz quad-core ARM Cortex-A53 (64-bit)
- **RAM**: 512MB LPDDR2
- **Connectivity**:
  - Bluetooth 4.2 BLE (for PS5 controller)
  - 802.11 b/g/n WiFi (for remote access/updates)
  - 1x USB 2.0 (for ELRS module)
  - 40-pin GPIO header (optional UART)
- **Power**: 5V/2A via micro-USB
- **OS**: Raspberry Pi OS (32-bit recommended)

**Rationale**:
- Compact form factor for portable builds
- Built-in Bluetooth eliminates need for USB dongle
- Sufficient performance for non-real-time control (vs. racing)
- Low power consumption (~300mA typical)
- Cost-effective (~$15 USD)

### 2.2 ExpressLRS Transmitter Module

**Component**: BetaFPV ELRS Nano 2.4GHz USB Adapter

**Specifications**:
- **Frequency**: 2.4GHz ISM band
- **Power Output**: 100mW (configurable, some models 250mW)
- **Interface**: USB 2.0 (appears as serial device)
- **Protocol**: CRSF over serial (420,000 baud)
- **Packet Rate**: 50Hz, 150Hz, 250Hz, 500Hz (default: 250Hz)
- **Range**: ~500m (varies with power, antennas, environment)
- **Dimensions**: ~30mm x 15mm x 5mm

**Connection**:
- USB Type-A to Pi Zero 2 W via micro-USB OTG adapter
- Device path: `/dev/ttyACM0` or `/dev/ttyUSB0`

**Alternative**: ESP32-based ELRS TX module via UART (not covered in v1.0)

### 2.3 Controller

**Component**: Sony PlayStation 5 DualSense Controller

**Specifications**:
- **Connectivity**: Bluetooth 5.1 (compatible with BT 4.2)
- **Inputs**:
  - 2x analog sticks (4 axes, 8-bit resolution each)
  - 2x analog triggers (L2/R2)
  - 4x shoulder buttons (L1/R1/L2/R2)
  - 4x face buttons (Cross/Circle/Square/Triangle)
  - 1x D-pad (4 directions)
  - 3x system buttons (PS/Share/Options)
  - 1x touchpad (clickable)
- **Sensors**: 6-axis IMU (gyroscope + accelerometer) - optional use
- **Battery**: Built-in rechargeable Li-ion
- **Range**: ~10m typical (Bluetooth)

**Linux Support**: Requires kernel 5.12+ for full support (Pi OS includes drivers)

### 2.4 Drone

**Component**: Meteor75 Pro (or compatible ELRS 2.4GHz receiver)

**Key Requirements**:
- ELRS 2.4GHz receiver (CRSF protocol compatible)
- Flight controller running Betaflight/INAV/similar
- Configured for CRSF serial RX protocol
- Minimum 4 channels (AETR), recommended 8+ for switches

### 2.5 Power Supply

**Stationary Use**:
- 5V/2.5A USB power adapter
- Micro-USB cable with data support

**Portable Use**:
- USB power bank (10,000mAh+ recommended)
- Provides 3-5 hours of continuous operation

### 2.6 Optional Accessories

| Component | Purpose | Cost |
|-----------|---------|------|
| Heatsink/Fan | Thermal management | ~$5 |
| Protective Case | Physical protection | ~$10 |
| USB OTG Adapter | Cleaner USB connection | ~$3 |
| Better Antenna | Increased range (ELRS module) | ~$10 |
| SD Card | 16GB+ (for OS + logs) | ~$10 |

---

## 3. Software Requirements

### 3.1 Operating System

**Required**: Linux-based OS with kernel 5.4+

**Recommended**: Raspberry Pi OS (32-bit) Lite or Desktop
- **Version**: Bullseye (11.x) or Bookworm (12.x)
- **Architecture**: armhf (32-bit ARMv7)
- **Kernel**: 5.15+ (includes evdev, Bluetooth stack)

**Rationale**: Official Pi OS provides best hardware support and compatibility

### 3.2 Programming Language

**Required**: Rust 1.70+ (stable channel)

**Toolchain**:
```bash
# On Raspberry Pi
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Cross-compilation target (for development PC)
rustup target add armv7-unknown-linux-gnueabihf
```

**Rationale**:
- Memory safety without garbage collection
- Excellent async/await support (Tokio)
- Strong type system prevents common bugs
- Growing ecosystem for embedded/hardware
- Performance comparable to C/C++

### 3.3 System Dependencies

**Required Packages**:
```bash
# Bluetooth support
sudo apt-get install bluetooth bluez libbluetooth-dev

# USB/Serial support
sudo apt-get install usbutils

# Input device support
sudo apt-get install libevdev-dev libudev-dev

# Development tools (if compiling on Pi)
sudo apt-get install build-essential pkg-config
```

### 3.4 Runtime Permissions

**User Permissions** (avoid running as root):
```bash
# Serial port access
sudo usermod -a -G dialout $USER

# Input device access
sudo usermod -a -G input $USER

# Bluetooth access
sudo usermod -a -G bluetooth $USER

# Log out and back in for changes to take effect
```

**udev Rules** (optional, for non-root serial access):
```bash
# /etc/udev/rules.d/99-elrs.rules
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", ATTRS{idProduct}=="ea60", MODE="0666"
```

---

## 4. Functional Requirements

### 4.1 Controller Input (FR-1)

**FR-1.1**: The system SHALL support PlayStation 5 DualSense controllers
- **Priority**: MUST HAVE
- **Verification**: Connect PS5 controller, verify device detection

**FR-1.2**: The system SHALL read all analog stick positions
- **Inputs**: Left stick X/Y, Right stick X/Y
- **Resolution**: 8-bit (0-255) from evdev, mapped to 11-bit CRSF
- **Update Rate**: Minimum 100Hz
- **Priority**: MUST HAVE

**FR-1.3**: The system SHALL read all digital button states
- **Inputs**: All face buttons, shoulder buttons, D-pad, system buttons
- **States**: Pressed (1) or Released (0)
- **Priority**: MUST HAVE

**FR-1.4**: The system SHALL apply configurable deadzones to analog inputs
- **Range**: 0-25% configurable per axis
- **Default**: 5% for sticks, 10% for triggers
- **Algorithm**: Scaled deadzone (smooth transition)
- **Priority**: MUST HAVE

**FR-1.5**: The system SHALL support exponential curves for stick inputs
- **Range**: 0.0 (linear) to 1.0 (maximum expo)
- **Default**: 0.3 for roll/pitch, 0.2 for yaw
- **Priority**: SHOULD HAVE

**FR-1.6**: The system SHALL provide stick calibration routine
- **Trigger**: Touchpad button press
- **Action**: Store current stick positions as center points
- **Priority**: SHOULD HAVE

**FR-1.7**: The system SHALL handle controller disconnection gracefully
- **Behavior**: Trigger failsafe, attempt reconnection
- **Timeout**: 500ms before failsafe
- **Priority**: MUST HAVE

### 4.2 CRSF Protocol (FR-2)

**FR-2.1**: The system SHALL generate valid CRSF RC Channels packets (Type 0x16)
- **Format**: 16 channels, 11-bit resolution (0-2047, representing 1000-2000μs)
- **Frame Rate**: 250Hz (4ms intervals)
- **Structure**: `[Sync][Len][Type][Payload][CRC8]`
- **Priority**: MUST HAVE

**FR-2.2**: The system SHALL calculate correct CRC8-DVB-S2 checksums
- **Polynomial**: 0xD5
- **Initial Value**: 0x00
- **Verification**: All packets validated before transmission
- **Priority**: MUST HAVE

**FR-2.3**: The system SHALL map controller inputs to RC channels
- **CH1 (Roll)**: Right stick X (1000-2000μs)
- **CH2 (Pitch)**: Right stick Y (1000-2000μs)
- **CH3 (Throttle)**: Left stick Y (1000-2000μs)
- **CH4 (Yaw)**: Left stick X (1000-2000μs)
- **CH5-16 (Aux)**: Buttons and switches
- **Priority**: MUST HAVE

**FR-2.4**: The system SHALL support 16 RC channels
- **Channels**: Full 16-channel support (Betaflight standard)
- **Unused Channels**: Set to center (1500μs)
- **Priority**: MUST HAVE

**FR-2.5**: The system SHALL parse incoming CRSF telemetry packets
- **Types**: Link Statistics (0x14), Battery (0x08), GPS (0x02), Attitude (0x1E)
- **Priority**: SHOULD HAVE

### 4.3 ELRS Communication (FR-3)

**FR-3.1**: The system SHALL communicate with BetaFPV ELRS USB module via serial
- **Port**: Auto-detect or configurable (default: `/dev/ttyACM0`)
- **Baud Rate**: 420,000 (CRSF standard)
- **Data Bits**: 8
- **Stop Bits**: 1
- **Parity**: None
- **Priority**: MUST HAVE

**FR-3.2**: The system SHALL support 250Hz packet transmission rate
- **Interval**: 4ms (±0.5ms tolerance)
- **Jitter**: <1ms
- **Priority**: MUST HAVE

**FR-3.3**: The system SHALL handle bidirectional communication
- **TX**: RC channels packets (outbound)
- **RX**: Telemetry packets (inbound)
- **Buffer**: Asynchronous read/write
- **Priority**: SHOULD HAVE

**FR-3.4**: The system SHALL detect and recover from serial disconnection
- **Detection**: Read/write timeout (100ms)
- **Recovery**: Automatic reconnection attempts (every 1s)
- **Behavior**: Trigger failsafe during disconnection
- **Priority**: MUST HAVE

### 4.4 Telemetry Logging (FR-4)

**FR-4.1**: The system SHALL log telemetry data in JSONL (JSON Lines) format
- **Format**: One JSON object per line
- **Encoding**: UTF-8
- **Priority**: MUST HAVE

**FR-4.2**: The system SHALL implement rotating log files
- **Trigger**: Configurable max records per file (default: 10,000)
- **Naming**: `telemetry_YYYYMMDD_HHMMSS.jsonl`
- **Location**: Configurable directory (default: `./logs/`)
- **Priority**: MUST HAVE

**FR-4.3**: The system SHALL retain only the last N log files
- **Retention**: Configurable (default: 10 files)
- **Deletion**: Automatic removal of oldest files
- **Priority**: MUST HAVE

**FR-4.4**: The system SHALL timestamp all log entries
- **Format**: ISO 8601 with microsecond precision (e.g., `2025-11-09T15:30:45.123456Z`)
- **Timezone**: UTC
- **Priority**: MUST HAVE

**FR-4.5**: The system SHALL log the following telemetry data:
- **Timestamp**: Entry time
- **Battery**: Voltage, current, capacity used
- **Link**: RSSI, LQ, SNR
- **GPS**: Latitude, longitude, altitude, satellites
- **Flight State**: Armed status, flight mode
- **RC Channels**: All 16 channel values
- **Priority**: SHOULD HAVE (battery/link MUST, GPS optional)

**FR-4.6**: The system SHALL support configurable logging intervals
- **Range**: 10ms to 10,000ms
- **Default**: 100ms (10Hz)
- **Priority**: SHOULD HAVE

**FR-4.7**: The system SHALL allow enabling/disabling telemetry logging
- **Runtime Toggle**: Share button on controller
- **Configuration**: Startup default in config file
- **Priority**: SHOULD HAVE

### 4.5 Safety Features (FR-5)

**FR-5.1**: The system SHALL implement arming sequence with hold timer
- **Trigger**: L1 button (ARM switch)
- **Hold Duration**: 1000ms (configurable)
- **Indication**: Console message or LED (future)
- **Priority**: MUST HAVE

**FR-5.2**: The system SHALL disarm on controller disconnect
- **Detection**: No input events for 500ms
- **Action**: Set CH5 (ARM) to 1000μs (disarmed)
- **Priority**: MUST HAVE

**FR-5.3**: The system SHALL implement emergency disarm
- **Trigger**: PS button (home button)
- **Action**: Immediate disarm (CH5 = 1000μs)
- **Bypass**: No hold timer required
- **Priority**: MUST HAVE

**FR-5.4**: The system SHALL prevent arming with high throttle
- **Threshold**: Throttle must be below 1050μs to arm
- **Behavior**: Reject arming attempts if throttle too high
- **Priority**: MUST HAVE

**FR-5.5**: The system SHALL implement failsafe on communication loss
- **Triggers**:
  - Controller disconnection (>500ms no input)
  - Serial port error
  - ELRS module unresponsive
- **Action**: Send disarm signal, log error
- **Priority**: MUST HAVE

**FR-5.6**: The system SHALL implement auto-disarm timeout
- **Timeout**: Configurable (default: 300s / 5 minutes)
- **Condition**: No stick movement detected
- **Behavior**: Automatic disarm with warning
- **Priority**: SHOULD HAVE

### 4.6 Configuration (FR-6)

**FR-6.1**: The system SHALL support TOML configuration files
- **Format**: TOML 1.0.0 specification
- **Location**: `./config/default.toml` or via `--config` flag
- **Priority**: MUST HAVE

**FR-6.2**: The system SHALL validate configuration on startup
- **Checks**: Required fields, value ranges, file paths
- **Behavior**: Exit with clear error message if invalid
- **Priority**: MUST HAVE

**FR-6.3**: The system SHALL provide sensible default values
- **Behavior**: All configuration optional, defaults provided
- **Documentation**: Defaults documented in example config
- **Priority**: MUST HAVE

**FR-6.4**: The system SHALL support the following configuration sections:
- **[serial]**: Port, baud rate, timeout
- **[controller]**: Device path, deadzones, expo
- **[channels]**: Min/max values, center point
- **[telemetry]**: Logging settings, intervals
- **[safety]**: Arming timers, failsafe timeouts
- **[crsf]**: Packet rate, protocol settings
- **Priority**: MUST HAVE

**FR-6.5**: The system SHOULD support runtime configuration reload
- **Trigger**: SIGHUP signal or command
- **Scope**: Non-critical parameters only (not serial port)
- **Priority**: NICE TO HAVE

---

## 5. Non-Functional Requirements

### 5.1 Performance (NFR-1)

**NFR-1.1**: End-to-end latency SHALL be less than 50ms (target: <30ms)
- **Measurement**: Controller input to CRSF packet transmission
- **Condition**: 95th percentile under normal load
- **Priority**: MUST HAVE

**NFR-1.2**: The system SHALL maintain 250Hz packet rate with zero dropped packets
- **Measurement**: Packet interval consistency
- **Condition**: Over 1-hour continuous operation
- **Priority**: MUST HAVE

**NFR-1.3**: CPU usage SHALL be less than 50% on Raspberry Pi Zero 2 W
- **Measurement**: Average CPU utilization (all cores)
- **Condition**: During active flight control
- **Priority**: SHOULD HAVE

**NFR-1.4**: Memory usage SHALL be less than 100MB
- **Measurement**: Resident Set Size (RSS)
- **Condition**: Including all buffers and logs
- **Priority**: SHOULD HAVE

**NFR-1.5**: The system SHALL start up in less than 5 seconds
- **Measurement**: Launch to first packet transmission
- **Condition**: Cold start with controller paired
- **Priority**: SHOULD HAVE

### 5.2 Reliability (NFR-2)

**NFR-2.1**: The system SHALL recover from serial disconnections
- **Recovery Time**: <3 seconds to resume operation
- **Data Loss**: No data corruption allowed
- **Priority**: MUST HAVE

**NFR-2.2**: The system SHALL handle controller reconnection
- **Behavior**: Seamless resume of control
- **Timeout**: Reconnect within 30 seconds without restart
- **Priority**: MUST HAVE

**NFR-2.3**: The system SHALL NOT crash on invalid telemetry data
- **Behavior**: Log error, continue operation
- **Recovery**: Discard malformed packets gracefully
- **Priority**: MUST HAVE

**NFR-2.4**: The system SHALL achieve >99% uptime under normal conditions
- **Measurement**: Operational time / total time
- **Condition**: Excluding intentional shutdowns
- **Priority**: SHOULD HAVE

**NFR-2.5**: The system SHALL handle Bluetooth interference gracefully
- **Behavior**: Detect packet loss, maintain control with degraded input rate
- **Priority**: SHOULD HAVE

### 5.3 Code Quality (NFR-3)

**NFR-3.1**: Overall test coverage SHALL be >80%
- **Measurement**: Line coverage via `cargo-tCoverage`
- **Scope**: All modules in `src/`
- **Priority**: MUST HAVE

**NFR-3.2**: Core module test coverage SHALL be >90%
- **Modules**: `crsf/`, `serial/`, `controller/mapper.rs`
- **Priority**: MUST HAVE

**NFR-3.3**: All public APIs SHALL be documented with rustdoc
- **Requirements**: Module docs, function docs, examples
- **Verification**: `cargo doc --no-deps` generates complete docs
- **Priority**: MUST HAVE

**NFR-3.4**: Code SHALL pass all clippy lints
- **Command**: `cargo clippy -- -D warnings`
- **Exceptions**: Documented via `#[allow(...)]` with justification
- **Priority**: MUST HAVE

**NFR-3.5**: Code SHALL be formatted with rustfmt
- **Command**: `cargo fmt --check`
- **Configuration**: Default rustfmt settings
- **Priority**: MUST HAVE

**NFR-3.6**: No unsafe code blocks allowed (except documented exceptions)
- **Verification**: `cargo geiger` scan
- **Exceptions**: Must be justified and reviewed
- **Priority**: SHOULD HAVE

### 5.4 Documentation (NFR-4)

**NFR-4.1**: All public functions SHALL have rustdoc comments
- **Sections**: Summary, Arguments, Returns, Errors, Examples
- **Priority**: MUST HAVE

**NFR-4.2**: All modules SHALL have module-level documentation
- **Content**: Purpose, usage overview, examples
- **Priority**: MUST HAVE

**NFR-4.3**: The project SHALL include comprehensive markdown documentation
- **Required Docs**:
  - REQUIREMENTS.md (this document)
  - ARCHITECTURE.md
  - HARDWARE_SETUP.md
  - CRSF_PROTOCOL.md
  - CONFIGURATION.md
  - BUTTON_MAPPING.md
  - TELEMETRY.md
  - BUILDING.md
  - TROUBLESHOOTING.md
- **Priority**: MUST HAVE

**NFR-4.4**: README.md SHALL include quick start guide
- **Sections**: Introduction, requirements, installation, usage, troubleshooting
- **Priority**: MUST HAVE

**NFR-4.5**: Configuration file SHALL include inline comments
- **Format**: TOML comments explaining each option
- **Priority**: MUST HAVE

### 5.5 Maintainability (NFR-5)

**NFR-5.1**: Architecture SHALL be modular with clear separation of concerns
- **Modules**: CRSF, Serial, Controller, Telemetry, Config (isolated)
- **Coupling**: Minimal cross-module dependencies
- **Priority**: MUST HAVE

**NFR-5.2**: The system SHALL minimize external dependencies
- **Guideline**: Only include well-maintained crates with >1M downloads
- **Review**: Justify each dependency
- **Priority**: SHOULD HAVE

**NFR-5.3**: The codebase SHALL be easy to extend
- **Examples**: Support new controllers, alternative protocols
- **Design**: Use traits for abstraction
- **Priority**: SHOULD HAVE

**NFR-5.4**: Error messages SHALL be clear and actionable
- **Format**: Context, cause, suggested resolution
- **Example**: "Failed to open serial port /dev/ttyACM0: Permission denied. Try: sudo usermod -a -G dialout $USER"
- **Priority**: MUST HAVE

---

## 6. Constraints

### 6.1 Technical Constraints

**CONST-1**: The system MUST use Rust programming language
- **Rationale**: Project requirement for low-level language with safety

**CONST-2**: The system MUST run on Raspberry Pi Zero 2 W (ARM7 architecture)
- **Rationale**: Hardware constraint

**CONST-3**: The system MUST work with standard Linux kernel (no custom kernel modules)
- **Rationale**: Maintainability and ease of deployment

**CONST-4**: The system MUST NOT require root privileges for normal operation
- **Rationale**: Security best practices
- **Exception**: Initial setup (pairing, permissions) may require sudo

**CONST-5**: The system MUST use open-source dependencies only
- **Rationale**: Licensing and redistribution

### 6.2 Hardware Constraints

**CONST-6**: Serial communication fixed at 420,000 baud (CRSF standard)
- **Rationale**: Protocol specification

**CONST-7**: CRSF packet rate limited to 50-500Hz (hardware dependent)
- **Rationale**: ELRS module capabilities

**CONST-8**: Bluetooth range limited to ~10m (PS5 controller)
- **Rationale**: Hardware limitation

**CONST-9**: CPU/RAM limited by Pi Zero 2 W specifications
- **Rationale**: Hardware constraint

### 6.3 Protocol Constraints

**CONST-10**: CRSF protocol implementation must follow TBS specification
- **Reference**: https://github.com/crsf-wg/crsf/wiki
- **Rationale**: Compatibility with ELRS firmware

**CONST-11**: RC channel values limited to 1000-2000μs (11-bit resolution)
- **Rationale**: CRSF/ExpressLRS standard

---

## 7. Assumptions and Dependencies

### 7.1 Assumptions

**ASSUMP-1**: User has basic Linux command-line knowledge
- **Impact**: Documentation assumes familiarity with SSH, file editing

**ASSUMP-2**: ELRS receiver on drone is already configured for CRSF protocol
- **Impact**: No flight controller configuration covered in this project

**ASSUMP-3**: PS5 controller is charged and functional
- **Impact**: Troubleshooting low battery not covered

**ASSUMP-4**: User has stable power supply for Raspberry Pi
- **Impact**: Power-related issues out of scope

**ASSUMP-5**: Network connectivity available for initial setup
- **Impact**: Rust installation, package updates require internet

### 7.2 External Dependencies

**DEP-1**: Raspberry Pi OS provides working Bluetooth stack (BlueZ)
- **Risk**: OS updates may break Bluetooth compatibility
- **Mitigation**: Document tested OS versions

**DEP-2**: Linux kernel provides evdev interface for controller input
- **Risk**: Kernel updates may change evdev behavior
- **Mitigation**: Use stable kernel versions (5.15 LTS)

**DEP-3**: BetaFPV ELRS module firmware remains CRSF-compatible
- **Risk**: Firmware updates may change protocol
- **Mitigation**: Document tested firmware versions

**DEP-4**: PS5 controller drivers remain stable in Linux kernel
- **Risk**: Sony may change Bluetooth pairing mechanism
- **Mitigation**: Use kernel 5.12+ with native support

**DEP-5**: Rust ecosystem crates remain maintained
- **Risk**: Dependencies may be abandoned
- **Mitigation**: Choose crates with active development

---

## 8. Success Criteria

### 8.1 Minimum Viable Product (MVP)

The MVP is considered complete when:

1. ✅ PS5 controller connects via Bluetooth and reads all inputs
2. ✅ Sticks control drone (AETR - Aileron, Elevator, Throttle, Rudder)
3. ✅ L1 button arms/disarms the drone
4. ✅ Latency is <50ms end-to-end
5. ✅ System runs for 10 minutes without crashes
6. ✅ Basic telemetry logging works (battery, RSSI)
7. ✅ Configuration file loads successfully
8. ✅ Code coverage >70%

### 8.2 Full Release (v1.0)

Version 1.0 is ready for release when:

1. ✅ All MUST HAVE requirements implemented
2. ✅ >80% test coverage achieved
3. ✅ All documentation written and reviewed
4. ✅ Tested on actual Meteor75 Pro drone
5. ✅ Cross-platform build working (PC → Pi)
6. ✅ Telemetry log rotation working correctly
7. ✅ Safety features (failsafe, emergency disarm) tested
8. ✅ README includes complete setup guide
9. ✅ At least 3 successful test flights completed

### 8.3 Quality Gates

**Gate 1: Protocol Implementation**
- CRSF packets validated against specification
- CRC8 matches reference implementation
- Serial communication stable for 1 hour

**Gate 2: Controller Integration**
- All buttons mapped and responsive
- Deadzones eliminate stick drift
- No input lag noticeable during testing

**Gate 3: Safety Validation**
- Failsafe triggers within 500ms
- Emergency disarm works 100% of time
- Cannot arm with high throttle

**Gate 4: Documentation Complete**
- All markdown docs written
- Rustdoc builds without warnings
- README tested by external user

---

## 9. Out of Scope (Future Enhancements)

The following features are **NOT** included in v1.0:

### 9.1 User Interface
- ❌ Web-based configuration UI
- ❌ Mobile app for telemetry viewing
- ❌ Real-time telemetry dashboard
- ❌ Graphical flight log analyzer

### 9.2 Advanced Features
- ❌ OSD (On-Screen Display) integration
- ❌ Lua script support (OpenTX/EdgeTX)
- ❌ DVR recording integration
- ❌ Multiple controller support (Xbox, Switch Pro, etc.)
- ❌ Voice commands
- ❌ Head tracking support

### 9.3 Protocol Extensions
- ❌ MAVLink support (for autonomous drones)
- ❌ MSP (MultiWii Serial Protocol) for FC configuration
- ❌ Custom protocol implementations
- ❌ 900MHz ELRS support

### 9.4 Platforms
- ❌ macOS/Windows support
- ❌ Raspberry Pi Pico (RP2040) port
- ❌ ESP32 standalone version
- ❌ Android device support

### 9.5 Hardware Integrations
- ❌ External GPS module
- ❌ Barometer for altitude hold
- ❌ External buzzer for audio feedback
- ❌ LED status indicators

---

## 10. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-11-09 | FPV Bridge Team | Initial requirements document |

---

## 11. Approval

**Requirements Approved By**:
- [ ] Project Lead
- [ ] Technical Architect
- [ ] QA Lead
- [ ] Documentation Lead

**Approval Date**: _________________

---

## 12. References

### 12.1 External Specifications
- [CRSF Protocol Specification](https://github.com/crsf-wg/crsf/wiki)
- [ExpressLRS Documentation](https://www.expresslrs.org/)
- [Betaflight Configuration](https://betaflight.com/docs/)
- [Linux evdev Documentation](https://www.kernel.org/doc/html/latest/input/input.html)

### 12.2 Hardware Datasheets
- [Raspberry Pi Zero 2 W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w/)
- [BetaFPV ELRS Nano](https://betafpv.com/products/elrs-nano-tx)
- [PS5 DualSense Technical Info](https://www.playstation.com/en-us/accessories/dualsense-wireless-controller/)

### 12.3 Software Libraries
- [Tokio Async Runtime](https://tokio.rs/)
- [evdev-rs Crate](https://crates.io/crates/evdev)
- [serialport-rs Crate](https://crates.io/crates/serialport)
- [serde TOML](https://crates.io/crates/toml)

---

**End of Requirements Document**
