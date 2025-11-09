# FPV Bridge - Telemetry Logging

This document describes the telemetry logging system, log file formats, and how to analyze flight data.

---

## Table of Contents

1. [Overview](#overview)
2. [Log File Format](#log-file-format)
3. [Data Fields](#data-fields)
4. [File Rotation](#file-rotation)
5. [Analyzing Logs](#analyzing-logs)
6. [Tools and Scripts](#tools-and-scripts)

---

## Overview

FPV Bridge logs telemetry data received from the drone via ExpressLRS in real-time. This data is useful for:

- **Flight Analysis**: Review flight performance
- **Debugging**: Troubleshoot issues with drone or controller
- **Statistics**: Track battery usage, flight time, range
- **Safety**: Review incidents and crashes

### Key Features

- **JSONL Format**: One JSON object per line (easy to parse)
- **Rotating Logs**: Automatic file rotation based on record count
- **Configurable Retention**: Keep only last N files
- **Timestamped**: Microsecond precision for accurate timelines
- **Efficient**: Low overhead, doesn't impact flight performance

---

## Log File Format

### File Naming

```
telemetry_YYYYMMDD_HHMMSS.jsonl
```

**Examples**:
```
telemetry_20251109_143045.jsonl
telemetry_20251109_150312.jsonl
```

### File Location

**Default**: `./logs/` (relative to executable)

**Configurable** in `config/default.toml`:
```toml
[telemetry]
log_dir = "./logs"  # Change to your preferred path
```

### Format: JSONL (JSON Lines)

Each line is a complete JSON object (no commas between lines):

```jsonl
{"timestamp":"2025-11-09T15:30:45.123456Z","battery_voltage":4.15,"current":12.5,"rssi":-85,"link_quality":98,"armed":true}
{"timestamp":"2025-11-09T15:30:45.223456Z","battery_voltage":4.14,"current":13.2,"rssi":-86,"link_quality":97,"armed":true}
{"timestamp":"2025-11-09T15:30:45.323456Z","battery_voltage":4.13,"current":14.1,"rssi":-84,"link_quality":99,"armed":true}
```

**Why JSONL?**
- Easy to stream (append-only)
- Each line is independently parseable
- Standard format with many tools
- Human-readable

---

## Data Fields

### Timestamp

```json
"timestamp": "2025-11-09T15:30:45.123456Z"
```

**Format**: ISO 8601 with microsecond precision

**Timezone**: UTC (Z suffix)

**Example**: `2025-11-09T15:30:45.123456Z` = Nov 9, 2025 at 15:30:45.123456 UTC

---

### Battery Data

#### `battery_voltage` (Float)
**Description**: Battery voltage in volts

**Unit**: Volts (V)

**Range**: 0.0 to 50.0 (typical LiPo: 3.0-4.2V per cell)

**Example**:
```json
"battery_voltage": 4.15  // 4.15V (1S LiPo)
"battery_voltage": 12.45 // 12.45V (3S LiPo)
```

#### `current` (Float)
**Description**: Current draw in amperes

**Unit**: Amperes (A)

**Range**: 0.0 to 200.0

**Example**:
```json
"current": 12.5  // 12.5A
```

#### `capacity_used` (Integer)
**Description**: Battery capacity consumed in milliamp-hours

**Unit**: mAh

**Range**: 0 to 50,000+

**Example**:
```json
"capacity_used": 850  // 850mAh used
```

#### `battery_remaining` (Integer)
**Description**: Remaining battery percentage

**Unit**: Percent (%)

**Range**: 0 to 100

**Example**:
```json
"battery_remaining": 75  // 75% remaining
```

---

### Link Quality Data

#### `rssi` (Integer)
**Description**: Received Signal Strength Indicator (uplink)

**Unit**: dBm (decibel-milliwatts)

**Range**: -120 to 0 (higher is better)

**Interpretation**:
- `-50 to -60 dBm`: Excellent
- `-60 to -70 dBm`: Very good
- `-70 to -80 dBm`: Good
- `-80 to -90 dBm`: Fair
- `-90 to -100 dBm`: Poor
- `< -100 dBm`: Very poor

**Example**:
```json
"rssi": -85  // -85 dBm (good signal)
```

#### `link_quality` (Integer)
**Description**: Link quality percentage

**Unit**: Percent (%)

**Range**: 0 to 100 (higher is better)

**Interpretation**:
- `90-100%`: Excellent
- `70-89%`: Good
- `50-69%`: Fair
- `< 50%`: Poor (may have issues)

**Example**:
```json
"link_quality": 98  // 98% LQ (excellent)
```

#### `snr` (Integer)
**Description**: Signal-to-Noise Ratio

**Unit**: dB (decibels)

**Range**: -20 to +20 (higher is better)

**Interpretation**:
- `> 5 dB`: Excellent
- `0 to 5 dB`: Good
- `-5 to 0 dB`: Fair
- `< -5 dB`: Poor

**Example**:
```json
"snr": 10  // 10 dB SNR (excellent)
```

---

### GPS Data

#### `gps` (Object)
**Description**: GPS location and status

**Fields**:
```json
"gps": {
  "lat": 37.7749,      // Latitude (degrees)
  "lon": -122.4194,    // Longitude (degrees)
  "alt": 125,          // Altitude (meters above sea level)
  "speed": 15.5,       // Ground speed (km/h)
  "heading": 180,      // Heading (degrees, 0=North)
  "sats": 12           // Number of satellites
}
```

**Example**:
```json
"gps": {
  "lat": 37.7749,
  "lon": -122.4194,
  "alt": 125,
  "speed": 0.0,
  "heading": 0,
  "sats": 12
}
```

**Note**: Only present if drone has GPS module

---

### Flight State

#### `armed` (Boolean)
**Description**: Drone armed status

**Values**: `true` (armed) or `false` (disarmed)

**Example**:
```json
"armed": true   // Drone is armed
"armed": false  // Drone is disarmed
```

#### `flight_mode` (String)
**Description**: Current flight mode

**Values**: `"ANGLE"`, `"HORIZON"`, `"ACRO"`, `"GPS_RESCUE"`, etc.

**Example**:
```json
"flight_mode": "ACRO"
```

---

### RC Channels

#### `channels` (Array of Integers)
**Description**: All 16 RC channel values

**Unit**: Microseconds (μs)

**Range**: 1000 to 2000 per channel

**Example**:
```json
"channels": [1500, 1500, 1000, 1500, 2000, 2000, 1500, 1000, 1500, 1500, 1500, 1500, 1500, 1500, 1500, 1500]
```

**Interpretation**:
```
Index  Channel  Typical Function   Example Value
  0      CH1    Roll                1520 (slight right)
  1      CH2    Pitch               1480 (slight back)
  2      CH3    Throttle            1200 (20% throttle)
  3      CH4    Yaw                 1505 (centered)
  4      CH5    ARM                 2000 (armed)
  5      CH6    Flight Mode         2000 (acro)
  6      CH7    Beeper              1000 (off)
  7      CH8    Turtle              1000 (off)
  ...
```

---

## File Rotation

### Rotation Triggers

**Primary**: Record count reaches limit

**Configuration**:
```toml
[telemetry]
max_records_per_file = 10000  # Rotate after 10,000 records
```

**Example**:
- Logging at 10Hz (100ms intervals)
- 10,000 records = ~16.7 minutes per file
- Logging at 20Hz = ~8.3 minutes per file

### Rotation Process

```
1. Current file reaches max_records_per_file
   └─> telemetry_20251109_143045.jsonl (10,000 records)

2. Close current file

3. Create new file with current timestamp
   └─> telemetry_20251109_145712.jsonl (new file)

4. Check total file count

5. If > max_files_to_keep, delete oldest
   └─> Delete telemetry_20251109_120530.jsonl (oldest)

6. Continue logging to new file
```

### Retention Policy

**Configuration**:
```toml
[telemetry]
max_files_to_keep = 10  # Keep only last 10 files
```

**Example Directory**:
```
logs/
├── telemetry_20251109_143045.jsonl  (10,000 records, 16 min)
├── telemetry_20251109_145712.jsonl  (10,000 records, 16 min)
├── telemetry_20251109_152033.jsonl  (10,000 records, 16 min)
├── ...
└── telemetry_20251109_183520.jsonl  (5,234 records, active)

Total: 10 files (9 complete + 1 active)
Oldest files automatically deleted
```

### Disk Space Management

**Estimated File Sizes**:
```
Average log entry: ~150 bytes (compact JSON)
10,000 records × 150 bytes = ~1.5 MB per file
10 files × 1.5 MB = ~15 MB total
```

**Calculation**:
```python
records_per_file = 10000
files_to_keep = 10
avg_entry_size = 150  # bytes

total_disk_usage = records_per_file * files_to_keep * avg_entry_size
# = 10,000 × 10 × 150 = 15,000,000 bytes = ~15 MB
```

---

## Analyzing Logs

### Reading JSONL Files

#### Python

```python
import json

with open('logs/telemetry_20251109_143045.jsonl', 'r') as f:
    for line in f:
        entry = json.loads(line)
        print(f"Time: {entry['timestamp']}, Battery: {entry['battery_voltage']}V")
```

#### JavaScript/Node.js

```javascript
const fs = require('fs');
const readline = require('readline');

const rl = readline.createInterface({
  input: fs.createReadStream('logs/telemetry_20251109_143045.jsonl')
});

rl.on('line', (line) => {
  const entry = JSON.parse(line);
  console.log(`Time: ${entry.timestamp}, Battery: ${entry.battery_voltage}V`);
});
```

#### Command Line (jq)

```bash
# Extract all battery voltages
cat logs/telemetry_*.jsonl | jq '.battery_voltage'

# Find minimum battery voltage
cat logs/telemetry_*.jsonl | jq '.battery_voltage' | sort -n | head -1

# Filter entries where LQ < 80%
cat logs/telemetry_*.jsonl | jq 'select(.link_quality < 80)'

# Count armed vs disarmed records
cat logs/telemetry_*.jsonl | jq '.armed' | sort | uniq -c
```

---

### Common Queries

#### 1. Find Low Battery Events

```bash
# Battery voltage below 3.5V per cell (3S = 10.5V)
cat logs/telemetry_*.jsonl | jq 'select(.battery_voltage < 10.5)'
```

#### 2. Find Link Quality Issues

```bash
# Link quality below 80%
cat logs/telemetry_*.jsonl | jq 'select(.link_quality < 80)'
```

#### 3. Calculate Flight Time

```python
import json
from datetime import datetime

with open('logs/telemetry_20251109_143045.jsonl', 'r') as f:
    lines = f.readlines()

first = json.loads(lines[0])
last = json.loads(lines[-1])

t1 = datetime.fromisoformat(first['timestamp'].replace('Z', '+00:00'))
t2 = datetime.fromisoformat(last['timestamp'].replace('Z', '+00:00'))

flight_time = (t2 - t1).total_seconds()
print(f"Flight time: {flight_time:.1f} seconds ({flight_time/60:.1f} minutes)")
```

#### 4. Calculate Average Battery Drain

```python
import json

voltages = []
with open('logs/telemetry_20251109_143045.jsonl', 'r') as f:
    for line in f:
        entry = json.loads(line)
        if 'battery_voltage' in entry:
            voltages.append(entry['battery_voltage'])

if voltages:
    avg_voltage = sum(voltages) / len(voltages)
    min_voltage = min(voltages)
    max_voltage = max(voltages)

    print(f"Battery Stats:")
    print(f"  Average: {avg_voltage:.2f}V")
    print(f"  Min: {min_voltage:.2f}V")
    print(f"  Max: {max_voltage:.2f}V")
    print(f"  Voltage drop: {max_voltage - min_voltage:.2f}V")
```

#### 5. Plot Battery Voltage Over Time

```python
import json
import matplotlib.pyplot as plt
from datetime import datetime

timestamps = []
voltages = []

with open('logs/telemetry_20251109_143045.jsonl', 'r') as f:
    for line in f:
        entry = json.loads(line)
        if 'battery_voltage' in entry and 'timestamp' in entry:
            t = datetime.fromisoformat(entry['timestamp'].replace('Z', '+00:00'))
            timestamps.append(t)
            voltages.append(entry['battery_voltage'])

plt.figure(figsize=(12, 6))
plt.plot(timestamps, voltages, label='Battery Voltage')
plt.xlabel('Time')
plt.ylabel('Voltage (V)')
plt.title('Battery Voltage Over Time')
plt.grid(True)
plt.legend()
plt.xticks(rotation=45)
plt.tight_layout()
plt.show()
```

---

## Tools and Scripts

### Log Viewer (Python)

```python
#!/usr/bin/env python3
"""Simple telemetry log viewer"""

import json
import sys
from datetime import datetime

def format_timestamp(ts_str):
    dt = datetime.fromisoformat(ts_str.replace('Z', '+00:00'))
    return dt.strftime('%H:%M:%S.%f')[:-3]  # HH:MM:SS.mmm

def main(log_file):
    with open(log_file, 'r') as f:
        for line in f:
            entry = json.loads(line)

            ts = format_timestamp(entry.get('timestamp', 'N/A'))
            bat = entry.get('battery_voltage', 'N/A')
            rssi = entry.get('rssi', 'N/A')
            lq = entry.get('link_quality', 'N/A')
            armed = '✓' if entry.get('armed', False) else '✗'

            print(f"{ts} | Bat: {bat:>5}V | RSSI: {rssi:>4}dBm | LQ: {lq:>3}% | Armed: {armed}")

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <log_file.jsonl>")
        sys.exit(1)

    main(sys.argv[1])
```

**Usage**:
```bash
python3 log_viewer.py logs/telemetry_20251109_143045.jsonl
```

---

### Log Statistics (Bash)

```bash
#!/bin/bash
# Flight statistics from telemetry log

LOG_FILE="$1"

if [ -z "$LOG_FILE" ]; then
    echo "Usage: $0 <log_file.jsonl>"
    exit 1
fi

echo "=== Telemetry Log Statistics ==="
echo

# Record count
total_records=$(wc -l < "$LOG_FILE")
echo "Total Records: $total_records"

# Time range
first_ts=$(head -1 "$LOG_FILE" | jq -r '.timestamp')
last_ts=$(tail -1 "$LOG_FILE" | jq -r '.timestamp')
echo "First Entry: $first_ts"
echo "Last Entry:  $last_ts"

echo

# Battery stats
echo "Battery Voltage:"
cat "$LOG_FILE" | jq '.battery_voltage' | awk '
    BEGIN { min=999; max=0; sum=0; count=0 }
    {
        if ($1 < min) min = $1;
        if ($1 > max) max = $1;
        sum += $1;
        count++;
    }
    END {
        print "  Min: " min "V";
        print "  Max: " max "V";
        print "  Avg: " sum/count "V";
        print "  Drop: " (max - min) "V";
    }
'

echo

# Link quality
echo "Link Quality:"
cat "$LOG_FILE" | jq '.link_quality' | awk '
    BEGIN { min=100; max=0; sum=0; count=0 }
    {
        if ($1 < min) min = $1;
        if ($1 > max) max = $1;
        sum += $1;
        count++;
    }
    END {
        print "  Min: " min "%";
        print "  Max: " max "%";
        print "  Avg: " sum/count "%";
    }
'

echo

# Armed time
armed_count=$(cat "$LOG_FILE" | jq 'select(.armed == true)' | wc -l)
armed_pct=$(echo "scale=1; $armed_count * 100 / $total_records" | bc)
echo "Armed Time: $armed_pct% of log ($armed_count records)"
```

**Usage**:
```bash
bash log_stats.sh logs/telemetry_20251109_143045.jsonl
```

---

## Best Practices

### 1. Enable Logging During Important Flights

Toggle logging with **Share button** to conserve disk space:
- Practice flights: Logging OFF
- Test flights: Logging ON
- Long flights: Logging ON (for safety)

### 2. Monitor Disk Space

```bash
# Check disk usage
du -h logs/

# Clean old logs manually (if needed)
rm logs/telemetry_202511{01,02,03}*.jsonl
```

### 3. Backup Important Logs

```bash
# Copy logs to backup location
cp logs/*.jsonl /mnt/backup/fpv-logs/

# Or compress
tar -czf fpv-logs-backup-$(date +%Y%m%d).tar.gz logs/
```

### 4. Analyze After Crashes

After a crash:
1. Review last 10 seconds of log
2. Check battery voltage (low voltage?)
3. Check link quality (signal loss?)
4. Check RC channels (stick input issue?)

### 5. Tune Based on Logs

Use logs to optimize:
- Battery capacity (when voltage drops)
- Flight time estimates
- Range testing (RSSI vs distance)
- Link quality in different environments

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
