# FPV Bridge - System Architecture

## Overview

The FPV Bridge is an asynchronous, event-driven system built with Rust and the Tokio async runtime. It translates PlayStation 5 DualSense controller inputs into CRSF (Crossfire) protocol packets for controlling an ExpressLRS-enabled drone.

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         FPV Bridge Application                   │
│                                                                   │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐  │
│  │   Config     │─────▶│     Main     │─────▶│   Logging    │  │
│  │   Loader     │      │   Runtime    │      │   System     │  │
│  └──────────────┘      └──────┬───────┘      └──────────────┘  │
│                                │                                 │
│                                │ Spawn Tasks                     │
│                                │                                 │
│         ┌──────────────────────┼──────────────────────┐         │
│         │                      │                       │         │
│         ▼                      ▼                       ▼         │
│  ┌─────────────┐      ┌─────────────┐        ┌─────────────┐   │
│  │ Controller  │      │   Serial    │        │ Telemetry   │   │
│  │   Handler   │──┐   │   Handler   │◀───────│   Logger    │   │
│  │   (Async)   │  │   │   (Async)   │        │   (Async)   │   │
│  └─────────────┘  │   └─────────────┘        └─────────────┘   │
│                    │          │                                  │
│                    │          │                                  │
│                    │   ┌──────▼──────┐                          │
│                    │   │    CRSF     │                          │
│                    └──▶│   Encoder/  │                          │
│                        │   Decoder   │                          │
│                        └─────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Architecture

### 1. Main Runtime (`src/main.rs`)

**Responsibilities:**
- Initialize Tokio async runtime
- Load configuration from TOML file
- Set up tracing/logging infrastructure
- Spawn async tasks for each subsystem
- Handle graceful shutdown (SIGINT, SIGTERM)

**Key Operations:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load configuration
    let config = Config::load("config/default.toml")?;

    // 2. Initialize logging
    setup_logging(&config.log_level)?;

    // 3. Create channels for inter-task communication
    let (rc_tx, rc_rx) = mpsc::channel(100);
    let (telem_tx, telem_rx) = mpsc::channel(100);

    // 4. Spawn async tasks
    let controller_task = tokio::spawn(controller::run(config.controller, rc_tx));
    let serial_task = tokio::spawn(serial::run(config.serial, rc_rx, telem_tx));
    let telemetry_task = tokio::spawn(telemetry::run(config.telemetry, telem_rx));

    // 5. Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    // 6. Graceful shutdown
    shutdown_tasks().await
}
```

---

### 2. Controller Handler (`src/controller/`)

**Module Structure:**
```
controller/
├── mod.rs           # Module exports and high-level coordination
├── ps5.rs           # PS5 DualSense input handling via evdev
├── mapper.rs        # Input → RC channel mapping logic
├── calibration.rs   # Deadzone, expo curve calculations
└── tests.rs         # Unit tests
```

**Data Flow:**
```
evdev Device
    ↓ (Raw Input Events)
PS5InputHandler
    ↓ (Parsed Stick/Button State)
Calibration Layer (deadzones, expo)
    ↓ (Calibrated Values)
ChannelMapper
    ↓ (16 RC Channels: [u16; 16])
mpsc::Sender → Serial Handler
```

**Architecture Pattern:**
- **Async Event Loop**: Continuously reads evdev events
- **State Machine**: Tracks button hold times (e.g., arming sequence)
- **Channel Communication**: Sends RC channel arrays via MPSC channel

**Pseudo-code:**
```rust
pub async fn run(config: ControllerConfig, rc_tx: Sender<RcChannels>) -> Result<()> {
    let device = evdev::Device::open(&config.device_path)?;
    let mut mapper = ChannelMapper::new(config);

    loop {
        // Read input events (async)
        let events = device.fetch_events().await?;

        for event in events {
            // Update internal state
            mapper.process_event(event);

            // Generate RC channels
            let channels = mapper.to_rc_channels();

            // Send to serial handler
            rc_tx.send(channels).await?;
        }
    }
}
```

---

### 3. CRSF Protocol (`src/crsf/`)

**Module Structure:**
```
crsf/
├── mod.rs           # Module exports
├── protocol.rs      # Packet structures and constants
├── encoder.rs       # RC channels → CRSF packet encoding
├── decoder.rs       # Telemetry packet parsing
├── crc.rs           # CRC8 DVB-S2 checksum
└── tests.rs         # Protocol validation tests
```

**Packet Structure:**

```
CRSF Frame Format:
┌────────┬────────┬────────┬─────────────┬────────┐
│ SYNC   │ LENGTH │  TYPE  │   PAYLOAD   │  CRC8  │
│ (0xC8) │ (N+2)  │ (0x16) │   (N bytes) │        │
└────────┴────────┴────────┴─────────────┴────────┘
   1B        1B       1B          N B        1B

RC Channels Packet (Type 0x16):
- Payload: 22 bytes (16 channels × 11 bits = 176 bits = 22 bytes)
- Channel encoding: 11-bit values (0-2047) representing 988-2012μs
- Packed bitfield for efficiency
```

**Encoder Responsibilities:**
- Convert RC channel values (1000-2000μs) to 11-bit CRSF values (0-2047)
- Pack 16 channels into 22-byte payload
- Calculate CRC8 checksum
- Frame with sync byte and length

**Decoder Responsibilities:**
- Parse incoming telemetry packets
- Validate CRC8 checksums
- Extract battery, link stats, GPS data
- Convert to structured Rust types

---

### 4. Serial Handler (`src/serial/`)

**Module Structure:**
```
serial/
├── mod.rs           # Module exports
├── port.rs          # Serial port management
└── tests.rs         # Serial communication tests
```

**Responsibilities:**
- Open serial port at 420,000 baud
- Async read/write operations
- Bidirectional communication:
  - **TX**: Send CRSF RC channels packets (250Hz)
  - **RX**: Receive telemetry packets (variable rate)
- Error handling and reconnection logic

**Architecture Pattern:**
- **Split I/O**: Separate read and write tasks
- **Rate Limiting**: Ensure 250Hz (4ms) packet rate
- **Buffer Management**: Ring buffers for telemetry data

**Pseudo-code:**
```rust
pub async fn run(
    config: SerialConfig,
    mut rc_rx: Receiver<RcChannels>,
    telem_tx: Sender<TelemetryPacket>
) -> Result<()> {
    let mut port = tokio_serial::SerialStream::open(&config.port)?;

    // Split into read/write halves
    let (reader, writer) = tokio::io::split(port);

    // Spawn write task (TX)
    let write_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(4)); // 250Hz
        loop {
            interval.tick().await;
            if let Ok(channels) = rc_rx.try_recv() {
                let packet = crsf::encode_rc_channels(channels);
                writer.write_all(&packet).await?;
            }
        }
    });

    // Spawn read task (RX)
    let read_task = tokio::spawn(async move {
        let mut buffer = [0u8; 256];
        loop {
            let n = reader.read(&mut buffer).await?;
            if let Some(telem) = crsf::decode_telemetry(&buffer[..n]) {
                telem_tx.send(telem).await?;
            }
        }
    });

    tokio::try_join!(write_task, read_task)?;
    Ok(())
}
```

---

### 5. Telemetry Logger (`src/telemetry/`)

**Module Structure:**
```
telemetry/
├── mod.rs           # Module exports
├── logger.rs        # Rotating log file writer
├── types.rs         # Telemetry data structures
└── tests.rs         # Logging tests
```

**Responsibilities:**
- Receive telemetry data from serial handler
- Format as JSONL (JSON Lines)
- Write to rotating log files
- Manage file rotation (max N records)
- Retain only last M files

**Rotating Log Strategy:**

```
logs/
├── telemetry_20251109_143045.jsonl  (active, 8,234 records)
├── telemetry_20251109_135012.jsonl  (10,000 records)
├── telemetry_20251109_131508.jsonl  (10,000 records)
└── ... (keep last 10 files)

When active file reaches 10,000 records:
1. Close current file
2. Create new file with timestamp
3. Delete oldest file if >10 total files
```

**Implementation:**
```rust
pub struct RotatingLogger {
    current_file: File,
    record_count: usize,
    max_records_per_file: usize,
    max_files_to_keep: usize,
    log_dir: PathBuf,
}

impl RotatingLogger {
    pub async fn write(&mut self, entry: TelemetryEntry) -> Result<()> {
        // 1. Serialize to JSON
        let json = serde_json::to_string(&entry)?;

        // 2. Write to file
        self.current_file.write_all(json.as_bytes()).await?;
        self.current_file.write_all(b"\n").await?;
        self.record_count += 1;

        // 3. Check rotation
        if self.record_count >= self.max_records_per_file {
            self.rotate().await?;
        }

        Ok(())
    }

    async fn rotate(&mut self) -> Result<()> {
        // Close current file
        self.current_file.flush().await?;

        // Create new file
        let filename = format!("telemetry_{}.jsonl", Utc::now().format("%Y%m%d_%H%M%S"));
        self.current_file = File::create(self.log_dir.join(filename)).await?;
        self.record_count = 0;

        // Delete old files
        self.cleanup_old_files().await?;

        Ok(())
    }
}
```

---

### 6. Configuration (`src/config.rs`)

**Responsibilities:**
- Load TOML configuration file
- Validate all settings
- Provide defaults for missing values
- Expose strongly-typed config structs

**Configuration Schema:**
```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub serial: SerialConfig,
    pub controller: ControllerConfig,
    pub channels: ChannelConfig,
    pub telemetry: TelemetryConfig,
    pub safety: SafetyConfig,
    pub crsf: CrsfConfig,
}

#[derive(Debug, Deserialize)]
pub struct SerialConfig {
    #[serde(default = "default_serial_port")]
    pub port: String,
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    // ...
}

fn default_serial_port() -> String {
    "/dev/ttyACM0".to_string()
}
```

---

## Data Flow Diagram

### Control Path (Controller → Drone)

```
┌─────────────┐
│ PS5 Sticks  │
│ & Buttons   │
└──────┬──────┘
       │ Bluetooth
       ▼
┌─────────────┐
│   evdev     │ (Linux Input Subsystem)
│ /dev/input/ │
└──────┬──────┘
       │ Event Stream
       ▼
┌─────────────┐
│ Controller  │
│  Handler    │ Apply deadzones, expo
└──────┬──────┘
       │ mpsc::channel
       ▼
┌─────────────┐
│   Channel   │
│   Mapper    │ Map to 16 RC channels
└──────┬──────┘
       │ [u16; 16]
       ▼
┌─────────────┐
│    CRSF     │
│  Encoder    │ Build packet, CRC8
└──────┬──────┘
       │ Byte array
       ▼
┌─────────────┐
│   Serial    │
│    Port     │ 420,000 baud
└──────┬──────┘
       │ UART/USB
       ▼
┌─────────────┐
│ ELRS Module │ 2.4GHz RF transmission
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Drone     │
│     RX      │
└─────────────┘
```

**Latency Breakdown:**
- Controller input: ~5-10ms (Bluetooth)
- Event processing: ~1-2ms (Rust processing)
- CRSF encoding: <1ms
- Serial transmission: ~1ms
- ELRS air time: ~4ms (250Hz)
- **Total**: ~12-18ms (well under 50ms target)

### Telemetry Path (Drone → Logger)

```
┌─────────────┐
│   Drone FC  │ (Battery sensor, GPS, etc.)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ ELRS Module │ Send telemetry packets
└──────┬──────┘
       │ 2.4GHz RF
       ▼
┌─────────────┐
│ ELRS TX USB │
│   Module    │
└──────┬──────┘
       │ Serial RX
       ▼
┌─────────────┐
│   Serial    │
│  Read Task  │ Parse incoming bytes
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    CRSF     │
│  Decoder    │ Validate CRC, parse payload
└──────┬──────┘
       │ mpsc::channel
       ▼
┌─────────────┐
│ Telemetry   │
│   Logger    │ Write JSONL, rotate files
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Log Files  │ ./logs/telemetry_*.jsonl
└─────────────┘
```

---

## Concurrency Model

### Tokio Async Runtime

The application uses Tokio's multi-threaded runtime with work-stealing scheduler:

```rust
#[tokio::main]
async fn main() {
    // Uses default runtime: multi-threaded with N worker threads (N = CPU cores)
}
```

### Task Isolation

Each subsystem runs as an independent async task:

```
┌────────────────────────────────────────┐
│         Tokio Runtime (4 threads)      │
│                                        │
│  ┌──────────┐  ┌──────────┐           │
│  │ Thread 1 │  │ Thread 2 │  ...      │
│  └────┬─────┘  └────┬─────┘           │
│       │             │                  │
│  ┌────▼─────────────▼──────┐          │
│  │   Work-Stealing Queue   │          │
│  └────┬────────┬────────┬──┘          │
│       │        │        │             │
│  ┌────▼───┐ ┌─▼─────┐ ┌▼────────┐    │
│  │Control │ │Serial │ │Telemetry│    │
│  │ Task   │ │ Task  │ │  Task   │    │
│  └────────┘ └───────┘ └─────────┘    │
└────────────────────────────────────────┘
```

### Communication Channels

**MPSC (Multi-Producer, Single-Consumer):**
- Controller → Serial: `mpsc::channel<RcChannels>(100)`
- Serial → Telemetry: `mpsc::channel<TelemetryPacket>(100)`

**Why MPSC?**
- Lock-free for single consumer
- Bounded capacity (backpressure)
- Async-friendly (`.await` on send/recv)

---

## Error Handling Strategy

### Error Types

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum FpvBridgeError {
    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),

    #[error("Controller not found: {0}")]
    ControllerNotFound(String),

    #[error("CRSF protocol error: {0}")]
    CrsfProtocol(String),

    #[error("Configuration error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("Telemetry log error: {0}")]
    Telemetry(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, FpvBridgeError>;
```

### Error Recovery

| Error Type | Recovery Strategy |
|------------|-------------------|
| Serial disconnection | Retry connection every 1s, send failsafe |
| Controller disconnection | Disarm drone, wait for reconnect |
| Invalid CRSF packet | Log warning, discard packet, continue |
| Config parse error | Exit with error message (no recovery) |
| Log write failure | Log to stderr, continue operation |

---

## Safety Architecture

### Failsafe Conditions

```rust
pub enum FailsafeCondition {
    ControllerDisconnected,  // No input for 500ms
    SerialPortLost,          // Cannot write to ELRS module
    EmergencyButton,         // PS button pressed
    AutoTimeout,             // No stick movement for 5 minutes
}

impl FailsafeCondition {
    pub fn trigger(&self) -> RcChannels {
        // Always disarm (CH5 = 1000)
        // Throttle to low (CH3 = 1000)
        // Other channels to center (1500)
        [1500, 1500, 1000, 1500, 1000, 1500, ...]
    }
}
```

### Arming State Machine

```
┌──────────┐
│ Disarmed │◀─────────────────────────┐
└────┬─────┘                          │
     │ L1 pressed & throttle low      │
     │                                 │
     ▼                                 │
┌──────────┐                          │
│ Arming   │ Hold L1 for 1000ms       │
│ (Hold)   │──────────────┐           │
└────┬─────┘               │           │
     │ Timer expires        │ Release  │
     │                      │ L1       │
     ▼                      ▼          │
┌──────────┐          ┌──────────┐    │
│  Armed   │          │ Disarmed │────┘
└────┬─────┘          └──────────┘
     │
     │ PS button OR failsafe
     │
     ▼
┌──────────┐
│Emergency │
│ Disarm   │
└──────────┘
```

---

## Performance Considerations

### Memory Allocation

- **Minimize allocations in hot path**: Pre-allocate buffers
- **Zero-copy where possible**: Use byte slices, avoid cloning
- **Stack allocation**: Small structs (RC channels, etc.)

### CPU Efficiency

- **Avoid blocking**: All I/O is async
- **Efficient polling**: Use Tokio's epoll/kqueue (not busy-wait)
- **Lazy evaluation**: Process only when data available

### Latency Optimization

- **Direct paths**: Minimal layers between input and output
- **No locks in hot path**: Use message passing instead
- **Tuned buffer sizes**: Avoid queueing delays

---

## Testing Strategy

### Unit Tests
- Each module has `tests.rs` with comprehensive unit tests
- Test coverage: >80% overall, >90% for CRSF/mapper

### Integration Tests
- Mock serial port (loopback test)
- Mock controller input (event injection)
- End-to-end packet validation

### Property-Based Testing
- CRSF encoding/decoding round-trip
- Channel value mapping (1000-2000 ↔ 0-2047)

---

## Deployment Architecture

### Development Workflow

```
┌─────────────┐
│   Dev PC    │
│  (x86_64)   │
└──────┬──────┘
       │ cargo build --target armv7-unknown-linux-gnueabihf
       ▼
┌─────────────┐
│ Cross-      │
│ Compiler    │
└──────┬──────┘
       │ ARM binary
       ▼
┌─────────────┐
│     SCP     │ Transfer to Pi
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Pi Zero 2 W │
│  (armv7)    │
└─────────────┘
```

### Production Deployment

```
Pi Zero 2 W:
├── /home/pi/fpv-bridge/
│   ├── fpv-bridge           (executable)
│   ├── config/
│   │   └── default.toml
│   └── logs/                (auto-created)
│       └── telemetry_*.jsonl
└── /etc/systemd/system/
    └── fpv-bridge.service   (systemd service)
```

---

## Future Architecture Enhancements

### Potential Improvements (Out of Scope for v1.0)

1. **Plugin System**: Dynamic controller support
2. **Web UI**: Real-time telemetry dashboard
3. **OSD Integration**: Display stats on FPV feed
4. **Multi-protocol**: Support MAVLink, MSP
5. **Distributed Mode**: Run components on different devices

---

## References

- [Tokio Async Runtime](https://tokio.rs/)
- [CRSF Protocol Spec](https://github.com/crsf-wg/crsf/wiki)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Linux evdev](https://www.kernel.org/doc/html/latest/input/input.html)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
