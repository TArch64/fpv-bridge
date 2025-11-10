# FPV Bridge - Building and Installation

This guide covers building FPV Bridge from source, cross-compiling for Raspberry Pi, and installing on your system.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building on Raspberry Pi](#building-on-raspberry-pi)
3. [Cross-Compiling from PC](#cross-compiling-from-pc)
4. [Installation](#installation)
5. [Running as a Service](#running-as-a-service)
6. [Development Build](#development-build)
7. [Testing](#testing)

---

## Prerequisites

### System Requirements

**For Raspberry Pi Zero 2 W**:
- Raspberry Pi OS (32-bit) Lite or Desktop
- Kernel 5.15+ (for PS5 controller support)
- 512MB RAM minimum
- 1GB free disk space

**For Development PC** (cross-compilation):
- Linux, macOS, or Windows (WSL2)
- 2GB free disk space
- Internet connection (for dependencies)

### Install Rust

**On Raspberry Pi or Linux/macOS**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts and select default installation.

**Activate Rust**:
```bash
source $HOME/.cargo/env
```

**Verify Installation**:
```bash
rustc --version
cargo --version
```

Expected output:

```text
rustc 1.75.0 (or newer)
cargo 1.75.0 (or newer)
```

---

## Building on Raspberry Pi

### Step 1: Install System Dependencies

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libudev-dev \
    libevdev-dev \
    libbluetooth-dev
```

### Step 2: Clone Repository

```bash
git clone https://github.com/TArch64/fpv-bridge.git
cd fpv-bridge
```

### Step 3: Build Release Binary

```bash
cargo build --release
```

**Build time**: ~10-15 minutes on Pi Zero 2 W

**Output**: `target/release/fpv-bridge`

### Step 4: Verify Build

```bash
./target/release/fpv-bridge --version
```

---

## Cross-Compiling from PC

Cross-compiling is **much faster** than building on the Pi (minutes vs. 10+ minutes).

### Step 1: Install Cross-Compilation Tools

**On Linux (Ubuntu/Debian)**:

```bash
# Install ARM cross-compiler
sudo apt-get install -y gcc-arm-linux-gnueabihf

# Add Rust target
rustup target add armv7-unknown-linux-gnueabihf
```

**On macOS**:

```bash
# Install cross-compilation tool
brew install messense/macos-cross-toolchains/armv7-unknown-linux-gnueabihf

# Add Rust target
rustup target add armv7-unknown-linux-gnueabihf
```

**On Windows (WSL2)**:

Use the Linux instructions above within WSL2.

### Step 2: Configure Cargo for Cross-Compilation

Create `.cargo/config.toml` in the project root:

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

**For macOS** (using brew cross-compiler):

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "armv7-unknown-linux-gnueabihf-gcc"
```

### Step 3: Install ARM System Libraries

**Option A: Use Docker (Recommended)**

Create `Dockerfile.cross`:

```dockerfile
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    gcc-arm-linux-gnueabihf \
    libc6-dev-armhf-cross \
    curl \
    pkg-config

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add armv7-unknown-linux-gnueabihf

WORKDIR /build
```

Build and use:

```bash
docker build -f Dockerfile.cross -t fpv-bridge-cross .
docker run --rm -v $(pwd):/build fpv-bridge-cross \
    cargo build --release --target armv7-unknown-linux-gnueabihf
```

**Option B: Manual Setup**

Download ARM libraries from Raspberry Pi (one-time):

```bash
# On your Pi
tar -czf pi-libs.tar.gz /usr/lib/arm-linux-gnueabihf /lib/arm-linux-gnueabihf

# Transfer to PC
scp pi@raspberrypi.local:~/pi-libs.tar.gz .
```

### Step 4: Cross-Compile

```bash
cargo build --release --target armv7-unknown-linux-gnueabihf
```

**Build time**: ~2-5 minutes (vs 10-15 on Pi)

**Output**: `target/armv7-unknown-linux-gnueabihf/release/fpv-bridge`

### Step 5: Transfer to Raspberry Pi

```bash
# Copy binary to Pi
scp target/armv7-unknown-linux-gnueabihf/release/fpv-bridge \
    pi@raspberrypi.local:~/fpv-bridge-app/

# SSH to Pi and make executable
ssh pi@raspberrypi.local
chmod +x ~/fpv-bridge-app/fpv-bridge
```

---

## Installation

### Option 1: Manual Installation

```bash
# On Raspberry Pi

# Create directory structure
mkdir -p ~/fpv-bridge-app/{config,logs}

# Copy binary (if cross-compiled, already done above)
cp target/release/fpv-bridge ~/fpv-bridge-app/

# Copy example config
cp config/default.toml ~/fpv-bridge-app/config/

# Make executable
chmod +x ~/fpv-bridge-app/fpv-bridge
```

### Option 2: System-Wide Installation

```bash
# Install to /usr/local/bin
sudo install -m 755 target/release/fpv-bridge /usr/local/bin/

# Create config directory
sudo mkdir -p /etc/fpv-bridge

# Copy default config
sudo cp config/default.toml /etc/fpv-bridge/

# Create log directory
sudo mkdir -p /var/log/fpv-bridge
sudo chown $USER:$USER /var/log/fpv-bridge
```

Update config to use system paths:

```toml
[telemetry]
log_dir = "/var/log/fpv-bridge"
```

---

## Running as a Service

### Create systemd Service

Create `/etc/systemd/system/fpv-bridge.service`:

```ini
[Unit]
Description=FPV Bridge - PS5 Controller to ELRS
After=network.target bluetooth.target

[Service]
Type=simple
User=pi
Group=pi
WorkingDirectory=/home/pi/fpv-bridge-app
ExecStart=/home/pi/fpv-bridge-app/fpv-bridge --config /home/pi/fpv-bridge-app/config/default.toml
Restart=on-failure
RestartSec=5s

# Environment
Environment="RUST_LOG=info"

# Permissions (important!)
SupplementaryGroups=dialout input bluetooth

[Install]
WantedBy=multi-user.target
```

### Enable and Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service (start on boot)
sudo systemctl enable fpv-bridge

# Start service now
sudo systemctl start fpv-bridge

# Check status
sudo systemctl status fpv-bridge

# View logs
journalctl -u fpv-bridge -f
```

### Service Management

```bash
# Stop service
sudo systemctl stop fpv-bridge

# Restart service
sudo systemctl restart fpv-bridge

# Disable service (don't start on boot)
sudo systemctl disable fpv-bridge

# View recent logs
journalctl -u fpv-bridge -n 50
```

---

## Development Build

### Debug Build (Faster Compilation)

```bash
cargo build
```

**Output**: `target/debug/fpv-bridge`

**Differences from release**:
- Faster compilation
- Larger binary size
- Debug symbols included
- Slower runtime performance

### Running in Development Mode

```bash
# With logging
RUST_LOG=debug cargo run

# With specific config
cargo run -- --config config/default.toml

# With trace-level logging
RUST_LOG=trace cargo run
```

### Watch Mode (Auto-Rebuild on Changes)

Install `cargo-watch`:
```bash
cargo install cargo-watch
```

Use:
```bash
cargo watch -x run
```

Now code changes trigger automatic rebuilds.

---

## Testing

### Running Tests

**All tests**:
```bash
cargo test
```

**Specific module**:
```bash
cargo test crsf
cargo test controller
cargo test serial
```

**With output**:
```bash
cargo test -- --nocapture
```

**Single test**:
```bash
cargo test test_crc8_calculation
```

### Code Coverage

Install `cargo-tarpaulin`:
```bash
cargo install cargo-tarpaulin
```

Generate coverage report:
```bash
cargo tarpaulin --out Html
```

Open `tarpaulin-report.html` in browser.

### Benchmarks (Optional)

Run performance benchmarks:
```bash
cargo bench
```

### Linting

**Clippy** (linter):
```bash
cargo clippy -- -D warnings
```

**Format check**:
```bash
cargo fmt --check
```

**Format code**:
```bash
cargo fmt
```

---

## Optimization Tips

### Release Builds with LTO

For maximum performance, edit `Cargo.toml`:

```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization
opt-level = 3           # Maximum optimization
strip = true            # Remove debug symbols
panic = "abort"         # Smaller binary
```

**Trade-off**: Longer compile time, smaller and faster binary.

### Size Optimization

For embedded systems with limited space:

```toml
[profile.release]
opt-level = "z"         # Optimize for size
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

**Result**: Binary size reduced by ~30-50%.

---

## Troubleshooting Build Issues

### Issue: "linker `cc` not found"

**Solution**:
```bash
sudo apt-get install build-essential
```

### Issue: "could not find libudev"

**Solution**:
```bash
sudo apt-get install libudev-dev
```

### Issue: Cross-compilation fails with "cannot find -ludev"

**Solution**:
```bash
# Install ARM libudev
sudo apt-get install libudev-dev:armhf
```

Or use Docker-based cross-compilation (recommended).

### Issue: "error: linking with `arm-linux-gnueabihf-gcc` failed"

**Solution**: Install ARM cross-compiler:
```bash
sudo apt-get install gcc-arm-linux-gnueabihf
```

### Issue: Out of memory during compilation on Pi

**Solution**: Add swap space:
```bash
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile
# Change CONF_SWAPSIZE=100 to CONF_SWAPSIZE=1024
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

Or use cross-compilation from PC.

### Issue: "failed to run custom build command for `pkg-config`"

**Solution**:
```bash
sudo apt-get install pkg-config
```

---

## Build Scripts

### Automated Cross-Compilation Script

Create `scripts/cross_compile.sh`:

```bash
#!/bin/bash
set -e

echo "=== FPV Bridge Cross-Compilation Script ==="

# Configuration
TARGET="armv7-unknown-linux-gnueabihf"
PI_USER="pi"
PI_HOST="raspberrypi.local"
PI_DIR="/home/pi/fpv-bridge-app"

# Build
echo "Building for $TARGET..."
cargo build --release --target $TARGET

# Transfer
echo "Transferring to Raspberry Pi..."
scp target/$TARGET/release/fpv-bridge $PI_USER@$PI_HOST:$PI_DIR/

# Make executable
echo "Setting permissions..."
ssh $PI_USER@$PI_HOST "chmod +x $PI_DIR/fpv-bridge"

echo "✓ Build and transfer complete!"
echo "Run on Pi: cd $PI_DIR && ./fpv-bridge"
```

**Usage**:

```bash
chmod +x scripts/cross_compile.sh
./scripts/cross_compile.sh
```

### Automated Pi Setup Script

Create `scripts/setup_pi.sh`:

```bash
#!/bin/bash
set -e

echo "=== FPV Bridge Raspberry Pi Setup ==="

# Update system
echo "Updating system..."
sudo apt-get update
sudo apt-get upgrade -y

# Install dependencies
echo "Installing dependencies..."
sudo apt-get install -y \
    bluetooth bluez libbluetooth-dev \
    build-essential pkg-config \
    libevdev-dev libudev-dev \
    usbutils htop

# Install Rust
echo "Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Add user to groups
echo "Configuring permissions..."
sudo usermod -a -G dialout $USER
sudo usermod -a -G input $USER
sudo usermod -a -G bluetooth $USER

# Create directories
echo "Creating directories..."
mkdir -p ~/fpv-bridge-app/{config,logs}

echo "✓ Setup complete!"
echo "Please log out and back in for group changes to take effect."
```

**Usage** (run on Pi):

```bash
wget https://raw.githubusercontent.com/TArch64/fpv-bridge/main/scripts/setup_pi.sh
chmod +x setup_pi.sh
./setup_pi.sh
```

---

## Documentation Generation

### Generate Rust Documentation

```bash
cargo doc --no-deps --open
```

This generates HTML documentation from rustdoc comments and opens it in your browser.

### Build Documentation for Offline Use

```bash
cargo doc --no-deps
cd target/doc
python3 -m http.server 8000
```

Access documentation at `http://localhost:8000/fpv_bridge/`

---

## Continuous Integration (Optional)

### GitHub Actions Example

Create `.github/workflows/build.yml`:

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: armv7-unknown-linux-gnueabihf

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-arm-linux-gnueabihf

      - name: Build
        run: cargo build --release --target armv7-unknown-linux-gnueabihf

      - name: Test
        run: cargo test

      - name: Lint
        run: cargo clippy -- -D warnings
```

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
