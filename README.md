# FPV Bridge

Control your FPV drone with a PS5 DualSense controller via ExpressLRS.

## Overview

FPV Bridge is a Rust application that bridges PlayStation 5 DualSense controller inputs to CRSF (Crossfire) protocol for controlling ExpressLRS-enabled drones. It runs on a Raspberry Pi Zero 2 W and provides low-latency, reliable control for casual and freestyle FPV flying.

## Features

- ✅ **PS5 Controller Support**: Native support for DualSense via Bluetooth
- ✅ **CRSF/ExpressLRS Protocol**: Full 16-channel support at 250Hz
- ✅ **Telemetry Logging**: JSONL format with rotating log files
- ✅ **Safety Features**: Arming sequences, failsafe, emergency disarm
- ✅ **Configurable**: TOML configuration with sensible defaults
- ✅ **Low Latency**: <30ms end-to-end latency for responsive control
- ✅ **Well Documented**: Comprehensive documentation and examples

## Hardware Requirements

- **Raspberry Pi Zero 2 W** (1GHz quad-core, 512MB RAM, Bluetooth 4.2)
- **BetaFPV ELRS Nano 2.4GHz USB Adapter** (or compatible ELRS TX module)
- **PS5 DualSense Controller** (Bluetooth connection)
- **Meteor75 Pro** (or any drone with ELRS 2.4GHz receiver)
- **5V/2.5A USB Power Supply**

## Quick Start

### 1. Hardware Setup

See [docs/HARDWARE_SETUP.md](docs/HARDWARE_SETUP.md) for detailed instructions.

```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y bluetooth bluez build-essential pkg-config \
    libevdev-dev libudev-dev

# Add user to required groups
sudo usermod -a -G dialout,input,bluetooth $USER
```

### 2. Build and Install

See [docs/BUILDING.md](docs/BUILDING.md) for build instructions.

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/TArch64/fpv-bridge.git
cd fpv-bridge

# Build release binary
cargo build --release

# Copy binary
cp target/release/fpv-bridge ~/fpv-bridge-app/
```

### 3. Configuration

```bash
# Copy default config
cp config/default.toml ~/fpv-bridge-app/config/

# Edit configuration
nano ~/fpv-bridge-app/config/default.toml
```

See [docs/CONFIGURATION.md](docs/CONFIGURATION.md) for complete configuration reference.

### 4. Run

```bash
cd ~/fpv-bridge-app
./fpv-bridge --config config/default.toml
```

Or install as a systemd service (see [docs/BUILDING.md](docs/BUILDING.md#running-as-a-service)).

## Documentation

- [REQUIREMENTS.md](docs/REQUIREMENTS.md) - Project requirements and specifications
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - System design and architecture
- [HARDWARE_SETUP.md](docs/HARDWARE_SETUP.md) - Hardware setup guide
- [CRSF_PROTOCOL.md](docs/CRSF_PROTOCOL.md) - CRSF protocol specification
- [CONFIGURATION.md](docs/CONFIGURATION.md) - Configuration reference
- [BUTTON_MAPPING.md](docs/BUTTON_MAPPING.md) - Controller button mapping
- [TELEMETRY.md](docs/TELEMETRY.md) - Telemetry logging format
- [BUILDING.md](docs/BUILDING.md) - Build and installation guide
- [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) - Troubleshooting guide

## Button Mapping

| Input | Channel | Function |
|-------|---------|----------|
| Right Stick X | CH1 | Roll |
| Right Stick Y | CH2 | Pitch |
| Left Stick Y | CH3 | Throttle |
| Left Stick X | CH4 | Yaw |
| L1 | CH5 | ARM Switch |
| R1 | CH6 | Flight Mode |
| L2 | CH7 | Beeper |
| R2 | CH8 | Turtle Mode |
| PS Button | - | Emergency Disarm |

See [docs/BUTTON_MAPPING.md](docs/BUTTON_MAPPING.md) for complete mapping.

## Performance

- **Latency**: <30ms end-to-end (target: <50ms)
- **Packet Rate**: 250Hz (4ms intervals)
- **CPU Usage**: <50% on Pi Zero 2 W
- **Memory Usage**: <100MB

## Safety

- ✅ Hold L1 for 1 second to arm
- ✅ PS button for emergency disarm
- ✅ Auto-disarm on controller disconnect
- ✅ Throttle must be low to arm
- ✅ Failsafe on communication loss

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ExpressLRS](https://www.expresslrs.org/) - Open-source RC link
- [TBS Crossfire](https://www.team-blacksheep.com/) - CRSF protocol
- [Betaflight](https://betaflight.com/) - Flight controller firmware
- [Tokio](https://tokio.rs/) - Async runtime for Rust

## Support

- **Issues**: https://github.com/TArch64/fpv-bridge/issues
- **Documentation**: See [docs/](docs/) directory
- **Troubleshooting**: See [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)

## Disclaimer

**This software is for educational and hobbyist use only. Flying FPV drones carries inherent risks. Always:**

- Follow local regulations and laws
- Maintain line of sight when required
- Fly in safe, open areas
- Use a spotter when appropriate
- Ensure your equipment is properly configured and tested
- Never fly over people or property without permission

**The authors are not responsible for any damages, injuries, or legal issues resulting from the use of this software.**

---

**Version**: 0.1.0
**Status**: Development
**Last Updated**: 2025-11-09
