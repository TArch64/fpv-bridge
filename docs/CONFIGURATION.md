# FPV Bridge - Configuration Reference

This document provides a complete reference for all configuration options in FPV Bridge.

---

## Table of Contents

1. [Configuration File Format](#configuration-file-format)
2. [Configuration Sections](#configuration-sections)
3. [Complete Example](#complete-example)
4. [Environment Variables](#environment-variables)
5. [Command-Line Options](#command-line-options)
6. [Validation](#validation)

---

## Configuration File Format

FPV Bridge uses **TOML** (Tom's Obvious, Minimal Language) for configuration files.

### File Location

**Default Search Paths** (in order):
1. `./config/default.toml` (current directory)
2. `~/.config/fpv-bridge/config.toml` (user config)
3. `/etc/fpv-bridge/config.toml` (system-wide)

**Override with command-line**:
```bash
fpv-bridge --config /path/to/custom.toml
```

### File Structure

```toml
[section]
key = value
nested_key = { subkey = value }

[another_section]
# Comments start with #
option = "string value"
number = 42
boolean = true
array = [1, 2, 3]
```

---

## Configuration Sections

### 1. Serial Port Configuration

```toml
[serial]
```

#### `port` (String)
**Description**: Path to the serial device for ELRS module

**Default**: `"/dev/ttyACM0"`

**Examples**:

```toml
port = "/dev/ttyACM0"  # Most common for BetaFPV ELRS USB
port = "/dev/ttyUSB0"  # Alternative USB serial
port = "/dev/elrs_tx"  # Custom udev symlink
```

**Notes**:
- Auto-detection not yet supported
- Device must exist and be readable
- User must have `dialout` group membership

#### `baud_rate` (Integer)
**Description**: Serial communication speed in bits per second

**Default**: `420000`

**Valid Values**: `420000` (CRSF standard)

**Examples**:

```toml
baud_rate = 420000  # CRSF standard, DO NOT CHANGE
```

**Notes**:
- CRSF protocol requires exactly 420,000 baud
- Changing this will break communication

#### `timeout_ms` (Integer)
**Description**: Read/write timeout in milliseconds

**Default**: `100`

**Range**: `10` to `5000`

**Examples**:

```toml
timeout_ms = 100   # Default
timeout_ms = 200   # More tolerant of delays
```

#### `reconnect_interval_ms` (Integer)
**Description**: Time between reconnection attempts on disconnect

**Default**: `1000`

**Range**: `100` to `10000`

**Examples**:
```toml
reconnect_interval_ms = 1000  # Retry every 1 second
reconnect_interval_ms = 5000  # Retry every 5 seconds
```

---

### 2. Controller Configuration

```toml
[controller]
```

#### `device_path` (String, Optional)
**Description**: Path to the PS5 controller input device

**Default**: Auto-detect first available PS5 controller

**Examples**:

```toml
device_path = "/dev/input/event0"  # Explicit device
# device_path = ""  # Auto-detect (default)
```

**Notes**:
- Leave empty for auto-detection
- Use `evtest` to find correct device
- User must have `input` group membership

#### `deadzone_stick` (Float)
**Description**: Deadzone for analog sticks (percentage)

**Default**: `0.05` (5%)

**Range**: `0.0` to `0.25` (0% to 25%)

**Examples**:

```toml
deadzone_stick = 0.05  # 5% deadzone
deadzone_stick = 0.10  # 10% for worn sticks
deadzone_stick = 0.0   # No deadzone (not recommended)
```

**Notes**:
- Prevents stick drift
- Too high = less responsive
- Too low = drift near center

#### `deadzone_trigger` (Float)
**Description**: Deadzone for L2/R2 triggers (percentage)

**Default**: `0.10` (10%)

**Range**: `0.0` to `0.25`

**Examples**:

```toml
deadzone_trigger = 0.10  # Default
deadzone_trigger = 0.15  # If triggers are sensitive
```

#### `expo_roll` (Float)
**Description**: Exponential curve for roll axis

**Default**: `0.3`

**Range**: `0.0` (linear) to `1.0` (maximum expo)

**Examples**:

```toml
expo_roll = 0.0   # Linear response
expo_roll = 0.3   # Gentle curve (default)
expo_roll = 0.7   # Aggressive curve
```

**Notes**:
- Higher values = less sensitive near center
- Useful for smoother control
- Formula: `output = sign(input) * |input|^(1 + expo)`

#### `expo_pitch` (Float)
**Description**: Exponential curve for pitch axis

**Default**: `0.3`

**Range**: `0.0` to `1.0`

#### `expo_yaw` (Float)
**Description**: Exponential curve for yaw axis

**Default**: `0.2`

**Range**: `0.0` to `1.0`

**Notes**:
- Yaw typically needs less expo than roll/pitch

#### `expo_throttle` (Float)
**Description**: Exponential curve for throttle axis

**Default**: `0.0` (linear)

**Range**: `0.0` to `1.0`

**Notes**:
- Throttle usually kept linear for precise control

---

### 3. Channel Configuration

```toml
[channels]
```

#### `throttle_min` (Integer)
**Description**: Minimum throttle value in microseconds

**Default**: `1000`

**Range**: `988` to `1500`

**Examples**:

```toml
throttle_min = 1000  # Standard minimum
throttle_min = 1100  # Higher minimum (safety margin)
```

#### `throttle_max` (Integer)
**Description**: Maximum throttle value in microseconds

**Default**: `2000`

**Range**: `1500` to `2012`

#### `center` (Integer)
**Description**: Center point for roll/pitch/yaw in microseconds

**Default**: `1500`

**Range**: `1400` to `1600`

**Notes**:
- Standard RC center is 1500μs
- Adjust if FC expects different center

#### `channel_reverse` (Array of Integers)
**Description**: List of channels to reverse

**Default**: `[]` (none reversed)

**Examples**:

```toml
channel_reverse = [2]       # Reverse pitch only
channel_reverse = [1, 2]    # Reverse roll and pitch
channel_reverse = []        # No reversing
```

**Notes**:
- Useful if drone responds in wrong direction
- Channels numbered 1-16

---

### 4. Telemetry Configuration

```toml
[telemetry]
```

#### `enabled` (Boolean)
**Description**: Enable/disable telemetry logging

**Default**: `true`

**Examples**:

```toml
enabled = true   # Logging on
enabled = false  # Logging off
```

#### `log_dir` (String)
**Description**: Directory for telemetry log files

**Default**: `"./logs"`

**Examples**:

```toml
log_dir = "./logs"
log_dir = "/var/log/fpv-bridge"
log_dir = "/home/pi/telemetry"
```

**Notes**:
- Directory created automatically if doesn't exist
- Requires write permissions

#### `max_records_per_file` (Integer)
**Description**: Maximum records before rotating to new file

**Default**: `10000`

**Range**: `100` to `1000000`

**Examples**:

```toml
max_records_per_file = 10000   # Default (~1MB file)
max_records_per_file = 50000   # Larger files
max_records_per_file = 1000    # Small files for testing
```

#### `max_files_to_keep` (Integer)
**Description**: Number of log files to retain (oldest deleted)

**Default**: `10`

**Range**: `1` to `100`

**Examples**:

```toml
max_files_to_keep = 10   # Keep last 10 files
max_files_to_keep = 50   # Keep more history
max_files_to_keep = 1    # Keep only current file
```

#### `log_interval_ms` (Integer)
**Description**: Time between log entries in milliseconds

**Default**: `100` (10Hz)

**Range**: `10` to `10000`

**Examples**:

```toml
log_interval_ms = 100   # 10Hz logging (default)
log_interval_ms = 50    # 20Hz logging (more data)
log_interval_ms = 1000  # 1Hz logging (less data)
```

**Notes**:
- Lower values = more disk usage
- Higher values = less detail

#### `format` (String)
**Description**: Log file format

**Default**: `"jsonl"`

**Valid Values**: `"jsonl"` (JSON Lines)

**Examples**:

```toml
format = "jsonl"  # JSON Lines (one JSON object per line)
```

**Notes**:
- Currently only JSONL supported
- Future: CSV, binary formats

---

### 5. Safety Configuration

```toml
[safety]
```

#### `arm_button_hold_ms` (Integer)
**Description**: Duration to hold ARM button before arming (milliseconds)

**Default**: `1000` (1 second)

**Range**: `0` to `5000`

**Examples**:

```toml
arm_button_hold_ms = 1000  # Default
arm_button_hold_ms = 2000  # Extra safety (2s hold)
arm_button_hold_ms = 0     # Instant arm (not recommended)
```

**Notes**:
- Prevents accidental arming
- 0 = instant arm (dangerous)

#### `auto_disarm_timeout_s` (Integer)
**Description**: Auto-disarm after no stick movement (seconds)

**Default**: `300` (5 minutes)

**Range**: `0` (disabled) to `3600` (1 hour)

**Examples**:

```toml
auto_disarm_timeout_s = 300   # 5 minutes
auto_disarm_timeout_s = 60    # 1 minute (aggressive)
auto_disarm_timeout_s = 0     # Disabled (not recommended)
```

#### `failsafe_timeout_ms` (Integer)
**Description**: Trigger failsafe if no controller input for this duration

**Default**: `500` (0.5 seconds)

**Range**: `100` to `5000`

**Examples**:

```toml
failsafe_timeout_ms = 500    # Default
failsafe_timeout_ms = 1000   # More tolerant
failsafe_timeout_ms = 100    # Very aggressive
```

#### `min_throttle_to_arm` (Integer)
**Description**: Maximum throttle value allowed when arming (microseconds)

**Default**: `1050`

**Range**: `1000` to `1200`

**Examples**:

```toml
min_throttle_to_arm = 1050  # Default
min_throttle_to_arm = 1100  # Stricter safety check
```

**Notes**:
- Prevents arming with throttle up
- Critical safety feature

---

### 6. CRSF Protocol Configuration

```toml
[crsf]
```

#### `packet_rate_hz` (Integer)
**Description**: RC channels packet transmission rate

**Default**: `250` (250Hz)

**Valid Values**: `50`, `150`, `250`, `500`

**Examples**:

```toml
packet_rate_hz = 250  # Default, good balance
packet_rate_hz = 500  # Lower latency (if ELRS module supports)
packet_rate_hz = 150  # Longer range, higher latency
```

**Notes**:
- Must match ELRS module capabilities
- Higher rate = lower latency, shorter range
- 250Hz recommended for casual flying

#### `link_stats_interval_ms` (Integer)
**Description**: Request link statistics from ELRS every N milliseconds

**Default**: `1000` (1 second)

**Range**: `100` to `10000`

**Examples**:
```toml
link_stats_interval_ms = 1000  # Default
link_stats_interval_ms = 500   # More frequent updates
link_stats_interval_ms = 5000  # Less network traffic
```

---

## Complete Example

### Default Configuration

```toml
# FPV Bridge - Default Configuration
# Copy to config/default.toml and customize

[serial]
port = "/dev/ttyACM0"
baud_rate = 420000
timeout_ms = 100
reconnect_interval_ms = 1000

[controller]
# Leave empty for auto-detection
device_path = ""

# Deadzones (0.0 to 0.25)
deadzone_stick = 0.05     # 5%
deadzone_trigger = 0.10   # 10%

# Exponential curves (0.0 = linear, 1.0 = max)
expo_roll = 0.3
expo_pitch = 0.3
expo_yaw = 0.2
expo_throttle = 0.0

[channels]
throttle_min = 1000
throttle_max = 2000
center = 1500

# Reverse channels (empty = none)
channel_reverse = []

[telemetry]
enabled = true
log_dir = "./logs"
max_records_per_file = 10000
max_files_to_keep = 10
log_interval_ms = 100
format = "jsonl"

[safety]
arm_button_hold_ms = 1000
auto_disarm_timeout_s = 300
failsafe_timeout_ms = 500
min_throttle_to_arm = 1050

[crsf]
packet_rate_hz = 250
link_stats_interval_ms = 1000
```

### Example: High-Performance Setup

```toml
# Optimized for low latency

[serial]
port = "/dev/ttyACM0"
baud_rate = 420000
timeout_ms = 50               # Lower timeout

[controller]
device_path = ""
deadzone_stick = 0.03         # Smaller deadzone
deadzone_trigger = 0.08
expo_roll = 0.2               # Less expo
expo_pitch = 0.2
expo_yaw = 0.1
expo_throttle = 0.0

[channels]
throttle_min = 1000
throttle_max = 2000
center = 1500
channel_reverse = []

[telemetry]
enabled = true
log_dir = "./logs"
max_records_per_file = 50000  # Larger files
max_files_to_keep = 5
log_interval_ms = 50          # 20Hz logging
format = "jsonl"

[safety]
arm_button_hold_ms = 500      # Faster arming
auto_disarm_timeout_s = 600   # 10 minutes
failsafe_timeout_ms = 300     # Faster failsafe
min_throttle_to_arm = 1050

[crsf]
packet_rate_hz = 500          # Higher rate
link_stats_interval_ms = 500
```

### Example: Conservative/Safe Setup

```toml
# Optimized for safety and beginners

[serial]
port = "/dev/ttyACM0"
baud_rate = 420000
timeout_ms = 200              # More tolerant
reconnect_interval_ms = 2000

[controller]
device_path = ""
deadzone_stick = 0.10         # Larger deadzone
deadzone_trigger = 0.15
expo_roll = 0.5               # More expo (smoother)
expo_pitch = 0.5
expo_yaw = 0.4
expo_throttle = 0.2

[channels]
throttle_min = 1000
throttle_max = 1900           # Lower max throttle
center = 1500
channel_reverse = []

[telemetry]
enabled = true
log_dir = "./logs"
max_records_per_file = 10000
max_files_to_keep = 20        # More history
log_interval_ms = 200         # 5Hz (less data)
format = "jsonl"

[safety]
arm_button_hold_ms = 2000     # 2 second hold
auto_disarm_timeout_s = 120   # 2 minutes
failsafe_timeout_ms = 500
min_throttle_to_arm = 1100    # Stricter check

[crsf]
packet_rate_hz = 150          # Lower rate, longer range
link_stats_interval_ms = 1000
```

---

## Environment Variables

Override configuration via environment variables:

```bash
# Serial port
export FPV_BRIDGE_SERIAL_PORT="/dev/ttyUSB0"

# Log directory
export FPV_BRIDGE_LOG_DIR="/var/log/fpv-bridge"

# Enable/disable telemetry
export FPV_BRIDGE_TELEMETRY_ENABLED="false"

# Run application
fpv-bridge
```

**Naming Convention**: `FPV_BRIDGE_<SECTION>_<KEY>` (uppercase, underscores)

**Priority** (highest to lowest):
1. Environment variables
2. Command-line config file (`--config`)
3. Default search paths
4. Built-in defaults

---

## Command-Line Options

```bash
fpv-bridge [OPTIONS]
```

### Options

#### `--config <FILE>`
**Description**: Path to configuration file

**Example**:

```bash
fpv-bridge --config /etc/fpv-bridge/custom.toml
```

#### `--log-level <LEVEL>`
**Description**: Logging verbosity

**Values**: `error`, `warn`, `info`, `debug`, `trace`

**Default**: `info`

**Example**:

```bash
fpv-bridge --log-level debug
```

#### `--dry-run`
**Description**: Validate configuration without running

**Example**:

```bash
fpv-bridge --config myconfig.toml --dry-run
```

#### `--version`
**Description**: Print version and exit

**Example**:

```bash
fpv-bridge --version
```

#### `--help`
**Description**: Print help message

**Example**:

```bash
fpv-bridge --help
```

---

## Validation

### Configuration Validation

On startup, FPV Bridge validates all configuration values:

**Validation Checks**:
- Required fields present
- Value ranges correct
- File paths exist and accessible
- Devices available
- Permissions correct

### Example Errors

**Missing serial port**:

```text
Error: Serial port /dev/ttyACM0 not found
Hint: Check USB connection and run 'ls /dev/ttyACM*'
```

**Invalid range**:

```text
Error: deadzone_stick must be between 0.0 and 0.25, got 0.5
```

**Permission denied**:

```text
Error: Cannot open /dev/ttyACM0: Permission denied
Hint: Add user to dialout group: sudo usermod -a -G dialout $USER
```

### Validation Command

```bash
# Validate config without running
fpv-bridge --config myconfig.toml --dry-run
```

**Output**:

```text
✓ Configuration loaded: myconfig.toml
✓ Serial port accessible: /dev/ttyACM0
✓ Log directory writable: ./logs
✓ Controller auto-detection enabled
✓ All values within valid ranges

Configuration is valid!
```

---

## Tips and Best Practices

### 1. Start with Defaults
- Use the default configuration as a starting point
- Make small incremental changes
- Test after each change

### 2. Tune Deadzones
- Too small: Stick drift
- Too large: Less responsive
- Test by hovering drone in place

### 3. Adjust Expo Curves
- Start with defaults (0.2-0.3)
- Increase for smoother control
- Decrease for more direct response

### 4. Safety First
- Keep arm button hold time ≥1 second
- Enable auto-disarm
- Set conservative throttle limits when learning

### 5. Telemetry Logging
- Enable for troubleshooting
- Disable for long flights (saves disk space)
- Adjust interval based on needs (100ms default is good)

### 6. Performance Tuning
- 250Hz packet rate is the sweet spot
- 500Hz for racing (if supported)
- 150Hz for maximum range

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
