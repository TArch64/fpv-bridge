# FPV Bridge - Troubleshooting Guide

This guide helps you diagnose and resolve common issues with FPV Bridge.

---

## Table of Contents

1. [General Debugging](#general-debugging)
2. [Controller Issues](#controller-issues)
3. [Serial/ELRS Issues](#serialelrs-issues)
4. [Connectivity Issues](#connectivity-issues)
5. [Performance Issues](#performance-issues)
6. [Configuration Issues](#configuration-issues)
7. [Logging and Diagnostics](#logging-and-diagnostics)

---

## General Debugging

### Enable Debug Logging

**Temporary** (for current session):

```bash
RUST_LOG=debug ./fpv-bridge
```

**Persistent** (in systemd service):

Edit `/etc/systemd/system/fpv-bridge.service`:

```ini
Environment="RUST_LOG=debug"
```

Then restart:

```bash
sudo systemctl restart fpv-bridge
```

### Log Levels

| Level | Use Case | Output Volume |
|-------|----------|---------------|
| `error` | Production | Minimal (errors only) |
| `warn` | Production | Low (errors + warnings) |
| `info` | Default | Medium (key events) |
| `debug` | Troubleshooting | High (detailed info) |
| `trace` | Development | Very high (all events) |

**Example**:

```bash
RUST_LOG=trace ./fpv-bridge
```

### Check System Status

```bash
# Check if process is running
ps aux | grep fpv-bridge

# Check system resources
htop

# Check USB devices
lsusb

# Check serial ports
ls -l /dev/tty*

# Check Bluetooth
hciconfig
bluetoothctl devices

# Check input devices
ls -l /dev/input/event*
```

---

## Controller Issues

### Problem: PS5 Controller Not Detected

**Symptoms**:
- `Controller not found` error
- No `/dev/input/event*` device for controller

**Diagnosis**:
```bash
# Check Bluetooth status
sudo systemctl status bluetooth

# Check paired devices
bluetoothctl devices

# Check input devices
ls -l /dev/input/event*

# Test with evtest
sudo evtest
```

**Solutions**:

**1. Controller not paired**:

```bash
sudo bluetoothctl
[bluetooth]# power on
[bluetooth]# agent on
[bluetooth]# scan on
# Put controller in pairing mode (PS + Share for 3s)
[bluetooth]# pair XX:XX:XX:XX:XX:XX
[bluetooth]# trust XX:XX:XX:XX:XX:XX
[bluetooth]# connect XX:XX:XX:XX:XX:XX
```

**2. Bluetooth service not running**:
```bash
sudo systemctl start bluetooth
sudo systemctl enable bluetooth
```

**3. Controller paired but disconnected**:

```bash
bluetoothctl
[bluetooth]# connect XX:XX:XX:XX:XX:XX
```

**4. Kernel too old (missing PS5 support)**:

```bash
uname -r  # Should be 5.12+

# If older, update
sudo apt-get update
sudo apt-get dist-upgrade
sudo reboot
```

---

### Problem: Controller Input Lag

**Symptoms**:
- Delayed response to stick movements
- Choppy control

**Diagnosis**:
```bash
# Check CPU usage
htop

# Check Bluetooth signal
hciconfig hci0

# Check system load
uptime
```

**Solutions**:

**1. High CPU usage**:
- Close unnecessary processes
- Use release build (not debug)
- Add cooling (heatsink/fan)

**2. Bluetooth interference**:
- Move closer to Pi (<5m)
- Avoid WiFi 2.4GHz interference
- Use USB Bluetooth dongle (better antenna)

**3. Power issues**:
- Use 5V/2.5A power supply
- Check `vcgencmd get_throttled` for undervoltage

---

### Problem: Stick Drift

**Symptoms**:
- Drone moves when sticks centered
- Channels not at 1500μs when centered

**Diagnosis**:
```bash
# Monitor channel values in Betaflight Configurator
# Or check telemetry logs
cat logs/telemetry_*.jsonl | jq '.channels[0:4]'
```

**Solutions**:

**1. Increase deadzone**:

Edit `config/default.toml`:

```toml
[controller]
deadzone_stick = 0.10  # Increase from 0.05 to 0.10
```

**2. Calibrate sticks**:
- Press Touchpad on controller
- Center all sticks
- Release touchpad

**3. Controller hardware issue**:
- Test controller on PS5/PC
- Clean analog sticks
- Replace controller if worn

---

## Serial/ELRS Issues

### Problem: Serial Port Not Found

**Symptoms**:
- `Serial port /dev/ttyACM0 not found`
- Cannot open serial port

**Diagnosis**:
```bash
# Check USB devices
lsusb

# Check serial ports
ls -l /dev/ttyACM* /dev/ttyUSB*

# Check kernel messages
dmesg | grep tty
dmesg | grep USB
```

**Solutions**:

**1. ELRS module not connected**:
- Check USB connection
- Try different USB port
- Check module LED (should be lit)

**2. Wrong device path**:

```bash
# Find correct device
ls -l /dev/ttyACM* /dev/ttyUSB*

# Update config
[serial]
port = "/dev/ttyUSB0"  # or whatever you found
```

**3. Permission denied**:

```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Log out and back in
exit

# Verify group membership
groups  # Should include "dialout"
```

**4. Device node doesn't exist**:

```bash
# Create udev rule
sudo nano /etc/udev/rules.d/99-elrs.rules

# Add:
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", ATTRS{idProduct}=="ea60", MODE="0666", SYMLINK+="elrs_tx"

# Reload udev
sudo udevadm control --reload-rules
sudo udevadm trigger

# Use symlink
[serial]
port = "/dev/elrs_tx"
```

---

### Problem: ELRS Module Not Responding

**Symptoms**:
- Cannot write to serial port
- Timeout errors
- No telemetry received

**Diagnosis**:
```bash
# Test serial port manually
sudo apt-get install minicom
minicom -D /dev/ttyACM0 -b 420000

# Check for errors in logs
RUST_LOG=debug ./fpv-bridge 2>&1 | grep -i error
```

**Solutions**:

**1. Wrong baud rate**:

Verify config:

```toml
[serial]
baud_rate = 420000  # Must be exactly this for CRSF
```

**2. Module firmware issue**:
- Reflash ELRS module firmware
- Use ExpressLRS Configurator
- Ensure CRSF protocol enabled in firmware

**3. Bad USB cable/connection**:
- Try different USB cable
- Check for loose connections
- Test with different USB port

**4. Module in bootloader mode**:
- Power cycle the module
- Re-flash firmware if needed

---

### Problem: No Telemetry Data

**Symptoms**:
- Logs show no telemetry entries
- RSSI/LQ always `null` or `0`

**Diagnosis**:
```bash
# Check if serial RX is working
RUST_LOG=debug ./fpv-bridge 2>&1 | grep -i telemetry

# Check Betaflight settings
# In Betaflight Configurator:
# - Configuration → Telemetry → CRSF should be enabled
```

**Solutions**:

**1. Telemetry disabled in flight controller**:
- Enable CRSF telemetry in Betaflight
- Configuration → Telemetry → Enable all sensors

**2. Bidirectional communication not working**:
- Check serial TX pin connected to FC RX
- Check serial RX pin connected to FC TX
- (For USB modules, this is automatic)

**3. Telemetry packets not parsed**:
- Check debug logs for parse errors
- Update to latest ELRS firmware

---

## Connectivity Issues

### Problem: Drone Not Responding to Inputs

**Symptoms**:
- Sticks move but drone doesn't respond
- Motors don't spin when armed

**Diagnosis**:
```bash
# Check in Betaflight Configurator → Receiver tab
# - Do channels move when you move sticks?
# - Is receiver connected?

# Check logs
RUST_LOG=debug ./fpv-bridge 2>&1 | grep -i channel
```

**Solutions**:

**1. FC not configured for CRSF**:

In Betaflight Configurator:

```text
Configuration → Receiver:
- Receiver Type: Serial-based receiver
- Serial Receiver Provider: CRSF
```
Save and reboot FC.

**2. Wrong UART port**:
- Check which UART is connected to ELRS RX
- Configure FC to use that UART for CRSF

**3. Channel mapping wrong**:
```bash
# In Betaflight Configurator → Receiver tab
# Verify channel order: AETR (Aileron, Elevator, Throttle, Rudder)
```

If reversed, edit config:

```toml
[channels]
channel_reverse = [1, 2]  # Example: reverse roll and pitch
```

**4. Not armed**:
- Check ARM channel (CH5) = 2000 when L1 held
- Verify throttle at minimum before arming
- Check Betaflight arming prevention flags

---

### Problem: Link Quality Poor

**Symptoms**:
- LQ < 80%
- Frequent disconnections
- RSSI very low (<-100 dBm)

**Diagnosis**:
```bash
# Check telemetry logs
cat logs/telemetry_*.jsonl | jq '.rssi, .link_quality'

# Check distance from drone
# Check for obstacles/interference
```

**Solutions**:

**1. Out of range**:
- Reduce distance to drone
- Use higher power ELRS module (250mW vs 100mW)
- Upgrade antennas

**2. Interference**:
- Avoid 2.4GHz WiFi congestion
- Move away from WiFi routers
- Fly in open areas

**3. Antenna damaged/loose**:
- Check ELRS module antenna connection
- Check drone receiver antenna
- Replace damaged antennas

**4. Wrong packet rate**:

Edit config:

```toml
[crsf]
packet_rate_hz = 150  # Lower rate = longer range
```

---

## Performance Issues

### Problem: High Latency

**Symptoms**:
- Noticeable delay between stick input and drone response
- Latency >100ms

**Diagnosis**:
```bash
# Check CPU usage
htop

# Check system load
uptime

# Profile application
RUST_LOG=trace ./fpv-bridge 2>&1 | grep -i latency
```

**Solutions**:

**1. CPU throttling**:

```bash
# Check temperature
vcgencmd measure_temp

# Check throttling
vcgencmd get_throttled
# 0x0 = no throttling
# Other values = thermal/voltage throttling
```

Add cooling (heatsink/fan).

**2. Background processes**:
```bash
# Stop unnecessary services
sudo systemctl stop <service_name>

# Check what's running
ps aux --sort=-%cpu | head -10
```

**3. Debug build instead of release**:

```bash
# Use release build
cargo build --release
./target/release/fpv-bridge
```

**4. Bluetooth latency**:
- Move controller closer to Pi
- Reduce Bluetooth interference
- Use higher packet rate:

  ```toml
  [crsf]
  packet_rate_hz = 250  # or 500 if supported
  ```

---

### Problem: Packet Loss

**Symptoms**:
- Jerky control
- Dropped inputs
- Warnings in logs

**Diagnosis**:
```bash
# Check serial errors
RUST_LOG=debug ./fpv-bridge 2>&1 | grep -i error

# Check system messages
dmesg | tail -50
```

**Solutions**:

**1. Serial buffer overflow**:
- Reduce logging verbosity
- Use release build (faster processing)

**2. Serial cable issues**:
- Use high-quality USB cable
- Shorten cable length if possible
- Avoid USB hubs (direct connection)

**3. System overload**:
- Close other applications
- Reduce telemetry logging frequency:

  ```toml
  [telemetry]
  log_interval_ms = 200  # Reduce from 100
  ```

---

## Configuration Issues

### Problem: Config File Not Found

**Symptoms**:
- `Configuration file not found` error

**Solutions**:

**1. Create default config**:

```bash
mkdir -p config
cp docs/examples/default.toml config/
```

**2. Specify config path**:

```bash
./fpv-bridge --config /path/to/config.toml
```

**3. Use system config**:

```bash
sudo mkdir -p /etc/fpv-bridge
sudo cp config/default.toml /etc/fpv-bridge/
```

---

### Problem: Invalid Configuration

**Symptoms**:
- `Configuration error: ...`
- Parsing errors

**Diagnosis**:
```bash
# Validate config
./fpv-bridge --config config/default.toml --dry-run
```

**Solutions**:

**1. TOML syntax error**:
- Check for typos
- Validate with TOML linter: https://www.toml-lint.com/

**2. Invalid value range**:

```text
Error: deadzone_stick must be between 0.0 and 0.25, got 0.5
```

Fix: Correct the value in config file.

**3. Missing required field**:

Add missing field with default value:

```toml
[serial]
port = "/dev/ttyACM0"  # This field is required
```

---

## Logging and Diagnostics

### Collect Diagnostic Information

Run this script to collect debug info:

```bash
#!/bin/bash
# Save as: collect_diagnostics.sh

echo "=== FPV Bridge Diagnostics ==="
echo

echo "System Info:"
uname -a
echo

echo "USB Devices:"
lsusb
echo

echo "Serial Ports:"
ls -l /dev/ttyACM* /dev/ttyUSB* 2>/dev/null
echo

echo "Input Devices:"
ls -l /dev/input/event* 2>/dev/null
echo

echo "Bluetooth:"
hciconfig
bluetoothctl devices
echo

echo "Groups:"
groups
echo

echo "Processes:"
ps aux | grep fpv-bridge
echo

echo "Recent Logs:"
journalctl -u fpv-bridge -n 20 --no-pager
echo

echo "Config File:"
cat ~/fpv-bridge-app/config/default.toml
echo

echo "=== End Diagnostics ==="
```

**Usage**:

```bash
bash collect_diagnostics.sh > diagnostics.txt
```

---

### Enable Verbose Logging

Create `config/debug.toml`:

```toml
# Same as default.toml but with debug settings

[serial]
port = "/dev/ttyACM0"
baud_rate = 420000
timeout_ms = 200  # Increased timeout

[controller]
device_path = ""  # Auto-detect

[telemetry]
enabled = true
log_dir = "./logs"
log_interval_ms = 50  # More frequent logging
```

Run with debug logging:

```bash
RUST_LOG=debug ./fpv-bridge --config config/debug.toml 2>&1 | tee debug.log
```

---

### Common Error Messages

| Error Message | Cause | Solution |
|---------------|-------|----------|
| `Permission denied` | User not in required group | `sudo usermod -a -G dialout,input $USER` |
| `Device not found` | Hardware not connected | Check USB/Bluetooth connection |
| `Failed to open serial port` | Port in use or wrong path | Check `lsof /dev/ttyACM0`, fix path |
| `Controller not detected` | Not paired or connected | Re-pair controller via `bluetoothctl` |
| `CRC mismatch` | Corrupted CRSF packet | Check serial cable, reduce interference |
| `Configuration error` | Invalid config syntax | Validate TOML file |
| `Failsafe triggered` | Connection lost | Check controller battery, Bluetooth range |

---

### Getting Help

If you're still stuck after trying these solutions:

1. **Check GitHub Issues**: https://github.com/TArch64/fpv-bridge/issues
2. **Collect diagnostics**: Run `collect_diagnostics.sh`
3. **Enable debug logging**: `RUST_LOG=debug ./fpv-bridge 2>&1 > debug.log`
4. **Create issue** with:
   - Hardware setup (Pi model, ELRS module)
   - Software versions (`fpv-bridge --version`, `uname -a`)
   - Config file
   - Debug log (last 50 lines)
   - Steps to reproduce

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
