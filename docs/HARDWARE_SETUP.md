# FPV Bridge - Hardware Setup Guide

This guide provides step-by-step instructions for setting up the hardware components of the FPV Bridge system.

---

## Table of Contents

1. [Bill of Materials](#bill-of-materials)
2. [Raspberry Pi Zero 2 W Setup](#raspberry-pi-zero-2-w-setup)
3. [ELRS USB Module Connection](#elrs-usb-module-connection)
4. [PS5 Controller Pairing](#ps5-controller-pairing)
5. [Power Supply](#power-supply)
6. [Physical Assembly](#physical-assembly)
7. [Verification](#verification)
8. [Troubleshooting](#troubleshooting)

---

## Bill of Materials

### Required Components

| Item | Specification | Quantity | Est. Cost |
|------|---------------|----------|-----------|
| **Raspberry Pi Zero 2 W** | 1GHz quad-core, 512MB RAM, BT 4.2, WiFi | 1 | $15 |
| **BetaFPV ELRS Nano 2.4GHz USB** | 100mW TX module | 1 | $25-30 |
| **PS5 DualSense Controller** | Bluetooth 5.1 compatible | 1 | $70 (or use existing) |
| **MicroSD Card** | 16GB+ Class 10 | 1 | $8 |
| **USB OTG Adapter** | Micro-USB male to USB-A female | 1 | $3 |
| **USB Power Supply** | 5V/2.5A minimum | 1 | $8 |
| **Micro-USB Cable** | Data-capable (not charge-only) | 1 | $5 |

**Total Cost**: ~$134 (or ~$64 if you already have a PS5 controller)

### Optional Components

| Item | Purpose | Cost |
|------|---------|------|
| Heatsink + Small Fan | Thermal management | $5 |
| Protective Case | Physical protection, mounting | $10 |
| USB Power Bank (10,000mAh+) | Portable power | $20 |
| Better Antenna for ELRS | Increased range | $10 |
| USB Hub (powered) | Multiple USB devices | $15 |

---

## Raspberry Pi Zero 2 W Setup

### Step 1: Install Raspberry Pi OS

**Using Raspberry Pi Imager (Recommended):**

1. Download [Raspberry Pi Imager](https://www.raspberrypi.com/software/)
2. Insert microSD card into your computer
3. Launch Imager:
   - **OS**: Raspberry Pi OS (32-bit) Lite (or Desktop if you want GUI)
   - **Storage**: Select your microSD card
4. Click **Advanced Options** (gear icon):
   - ✅ Enable SSH
   - ✅ Set username: `pi` (or your choice)
   - ✅ Set password
   - ✅ Configure WiFi (SSID and password)
   - ✅ Set locale settings
5. Click **Write** and wait for completion
6. Eject the microSD card

### Step 2: First Boot

1. Insert microSD card into Raspberry Pi Zero 2 W
2. Connect power (5V/2.5A via micro-USB port marked "PWR")
3. Wait ~60 seconds for first boot
4. Find the Pi's IP address:
   - Check your router's DHCP client list, or
   - Use `ping raspberrypi.local` (if mDNS works), or
   - Use a network scanner app

### Step 3: SSH Connection

```bash
# From your computer
ssh pi@raspberrypi.local
# or
ssh pi@<IP_ADDRESS>

# Enter password when prompted
```

### Step 4: Update System

```bash
# Update package lists
sudo apt-get update

# Upgrade installed packages
sudo apt-get upgrade -y

# Reboot
sudo reboot
```

### Step 5: Install Dependencies

After reboot, SSH back in and install required packages:

```bash
# Bluetooth support
sudo apt-get install -y bluetooth bluez libbluetooth-dev

# Development tools
sudo apt-get install -y build-essential pkg-config

# USB/Serial support
sudo apt-get install -y usbutils

# Input device support
sudo apt-get install -y libevdev-dev libudev-dev

# Optional: System monitoring
sudo apt-get install -y htop
```

### Step 6: Configure User Permissions

Add your user to required groups:

```bash
# Serial port access
sudo usermod -a -G dialout $USER

# Input device access
sudo usermod -a -G input $USER

# Bluetooth access
sudo usermod -a -G bluetooth $USER

# Log out and back in for changes to take effect
exit
```

SSH back in after logging out.

---

## ELRS USB Module Connection

### Step 1: Physical Connection

1. Connect the BetaFPV ELRS Nano USB module:
   - If using **USB OTG adapter**:
     ```
     Pi Zero 2 W (USB port) → Micro-USB OTG adapter → ELRS USB module
     ```
   - If using **USB hub**:
     ```
     Pi Zero 2 W (USB port) → USB hub → ELRS USB module
     ```

2. Ensure the ELRS module's LED lights up (indicates power)

### Step 2: Verify Detection

```bash
# List USB devices
lsusb
```

**Expected output** (look for Silicon Labs CP210x or similar):
```
Bus 001 Device 003: ID 10c4:ea60 Silicon Labs CP210x UART Bridge
```

```bash
# List serial ports
ls -l /dev/ttyACM* /dev/ttyUSB*
```

**Expected output**:
```
crw-rw---- 1 root dialout 166, 0 Nov  9 14:30 /dev/ttyACM0
```

or

```
crw-rw---- 1 root dialout 188, 0 Nov  9 14:30 /dev/ttyUSB0
```

**Note the device path** (`/dev/ttyACM0` or `/dev/ttyUSB0`) for configuration.

### Step 3: Test Serial Communication

```bash
# Install minicom (serial terminal)
sudo apt-get install -y minicom

# Test connection (replace /dev/ttyACM0 with your device)
minicom -D /dev/ttyACM0 -b 420000

# Press Ctrl+A then X to exit
```

You should see no errors when opening the port.

### Step 4: Optional - Create udev Rule

To ensure the device is always accessible without root:

```bash
# Create udev rule
sudo nano /etc/udev/rules.d/99-elrs.rules
```

Add this line:
```
SUBSYSTEM=="tty", ATTRS{idVendor}=="10c4", ATTRS{idProduct}=="ea60", MODE="0666", SYMLINK+="elrs_tx"
```

Save and reload udev rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Now you can use `/dev/elrs_tx` as a stable device path.

---

## PS5 Controller Pairing

### Step 1: Enable Bluetooth

```bash
# Start Bluetooth service
sudo systemctl start bluetooth
sudo systemctl enable bluetooth

# Check Bluetooth status
sudo systemctl status bluetooth
```

### Step 2: Pair PS5 DualSense Controller

```bash
# Enter Bluetooth pairing mode
sudo bluetoothctl
```

You'll enter the `bluetoothctl` interactive shell:

```
[bluetooth]# power on
[bluetooth]# agent on
[bluetooth]# default-agent
[bluetooth]# scan on
```

### Step 3: Put Controller in Pairing Mode

On the PS5 controller:
1. Hold **PS button** + **Share button** simultaneously for 3-5 seconds
2. The light bar will start flashing rapidly (pairing mode)

### Step 4: Complete Pairing

Back in `bluetoothctl`:

```
# You should see the controller appear (MAC address like XX:XX:XX:XX:XX:XX)
[bluetooth]# pair XX:XX:XX:XX:XX:XX
[bluetooth]# trust XX:XX:XX:XX:XX:XX
[bluetooth]# connect XX:XX:XX:XX:XX:XX

# Stop scanning
[bluetooth]# scan off

# Exit
[bluetooth]# exit
```

**Note**: Replace `XX:XX:XX:XX:XX:XX` with your controller's actual MAC address.

### Step 5: Verify Controller Input

```bash
# List input devices
ls -l /dev/input/event*
```

**Expected output** (multiple event devices):
```
crw-rw---- 1 root input 13, 64 Nov  9 14:35 /dev/input/event0
crw-rw---- 1 root input 13, 65 Nov  9 14:35 /dev/input/event1
```

Find the PS5 controller:
```bash
# Install evtest
sudo apt-get install -y evtest

# Test controller input
sudo evtest
```

Select the DualSense controller from the list and press buttons to verify.

### Step 6: Auto-Reconnect Configuration

To make the controller auto-connect on startup:

```bash
# Edit Bluetooth configuration
sudo nano /etc/bluetooth/main.conf
```

Ensure these lines are set:
```ini
[Policy]
AutoEnable=true
ReconnectAttempts=7
ReconnectIntervals=1,2,4,8,16,32,64
```

Save and restart Bluetooth:
```bash
sudo systemctl restart bluetooth
```

---

## Power Supply

### Stationary Setup

1. Use a **5V/2.5A (or higher) USB power adapter**
2. Connect to the **PWR** micro-USB port on the Pi (not the USB port)
3. Ensure the cable is **data-capable** (not charge-only)

**Recommended adapters**:
- Official Raspberry Pi Power Supply (5V/3A)
- Quality phone chargers (2.5A minimum)

### Portable Setup

For field use with a battery:

1. Use a **USB power bank** (10,000mAh or larger)
2. Ensure output is **5V/2A minimum**
3. Use a **short, quality micro-USB cable** to minimize voltage drop

**Estimated runtime**:
- 10,000mAh bank: ~3-5 hours
- 20,000mAh bank: ~6-10 hours

### Power Consumption

Typical power draw:
- **Raspberry Pi Zero 2 W**: ~200-300mA
- **ELRS USB Module (100mW)**: ~100-150mA
- **PS5 Controller (via Bluetooth)**: ~0mA (self-powered)
- **Total**: ~300-450mA (1.5-2.25W)

---

## Physical Assembly

### Basic Setup (Stationary)

```
┌─────────────────────────────────────┐
│                                     │
│        Desk/Table Surface           │
│                                     │
│  ┌──────────────┐   ┌──────────┐   │
│  │ Raspberry Pi │   │  Power   │   │
│  │  Zero 2 W    │───│ Adapter  │───┼──→ Wall Outlet
│  └──────┬───────┘   └──────────┘   │
│         │                           │
│         │ USB OTG                   │
│         │                           │
│  ┌──────▼───────┐                  │
│  │  ELRS USB    │                  │
│  │   Module     │                  │
│  └──────────────┘                  │
│                                     │
│          ∿∿∿ Bluetooth ∿∿∿         │
│                                     │
│      ┌──────────────┐              │
│      │     PS5      │              │
│      │  Controller  │ (in hand)    │
│      └──────────────┘              │
└─────────────────────────────────────┘
```

### Portable Setup (Recommended)

**Materials**:
- Small project box or 3D-printed case
- Velcro straps or zip ties
- Optional: cooling fan

**Assembly**:

1. Mount Pi Zero 2 W inside case
2. Attach ELRS module via short USB cable
3. Mount power bank externally or in separate compartment
4. Add ventilation holes for cooling
5. Label ports and LED indicators

**Example layout**:
```
┌─────────────────────────────┐
│     Portable FPV Bridge     │
│                             │
│  ┌─────────────────────┐   │
│  │   Power Bank        │   │
│  │   (10,000mAh)       │   │
│  └──────────┬──────────┘   │
│             │ USB cable     │
│  ┌──────────▼──────────┐   │
│  │  Raspberry Pi       │   │
│  │  Zero 2 W + Heatsink│   │
│  └──────────┬──────────┘   │
│             │ USB OTG       │
│  ┌──────────▼──────────┐   │
│  │  ELRS USB Module    │   │
│  │  (antenna outside)  │───┼──→ Antenna
│  └─────────────────────┘   │
│                             │
│  [LED Status Indicators]    │
└─────────────────────────────┘
```

### Cooling Considerations

**When to add cooling**:
- Continuous operation >30 minutes
- Ambient temperature >25°C (77°F)
- CPU-intensive tasks running

**Cooling options**:
1. **Passive**: Aluminum heatsink on CPU
2. **Active**: Small 30mm fan (5V) via GPIO pins
3. **Case**: Ventilated case with airflow

**GPIO Fan Connection** (optional):
```
Pi GPIO:
Pin 4 (5V)  ────→ Fan Red Wire
Pin 6 (GND) ────→ Fan Black Wire
```

---

## Verification

### System Check Checklist

After completing setup, verify all components:

```bash
# 1. Check Bluetooth controller
ls /dev/input/event* | wc -l
# Should show at least 2-3 event devices

# 2. Check serial port
ls -l /dev/ttyACM0
# Should exist with dialout group

# 3. Check USB device
lsusb | grep "CP210x\|Silicon Labs"
# Should show ELRS module

# 4. Check user permissions
groups
# Should include: dialout, input, bluetooth

# 5. Check Bluetooth status
systemctl status bluetooth
# Should show "active (running)"

# 6. Check PS5 controller connection
bluetoothctl devices
# Should list your controller

# 7. Check system resources
htop
# CPU should be <10% idle, memory <200MB used
```

### LED Indicators

**Raspberry Pi Zero 2 W**:
- **Green LED (ACT)**: Blinks on SD card activity
- **Solid green**: Normal operation

**BetaFPV ELRS Module**:
- **Solid LED**: Powered, no connection
- **Blinking LED**: Transmitting data
- **Fast blink**: Binding mode

**PS5 Controller**:
- **Solid blue bar**: Connected
- **Flashing white**: Pairing mode
- **Orange pulse**: Charging

---

## Troubleshooting

### Issue: Pi Won't Boot

**Symptoms**: No green LED activity, no network connection

**Solutions**:
1. Check power supply (minimum 2A)
2. Re-flash SD card with Raspberry Pi Imager
3. Try a different microSD card
4. Verify power cable is data-capable

### Issue: ELRS Module Not Detected

**Symptoms**: `lsusb` doesn't show module, no `/dev/ttyACM0`

**Solutions**:
```bash
# 1. Check USB connection
lsusb
# Should see "Silicon Labs" or similar

# 2. Try different USB port (if using hub)

# 3. Check kernel messages
dmesg | tail -n 20
# Look for USB device detection

# 4. Manual driver load (if needed)
sudo modprobe cp210x
```

### Issue: PS5 Controller Won't Pair

**Symptoms**: Controller not visible in Bluetooth scan

**Solutions**:
1. **Reset controller**:
   - Insert paperclip in small hole on back near L2 button
   - Hold for 5 seconds
   - Try pairing again

2. **Forget previous pairings**:
   ```bash
   sudo bluetoothctl
   [bluetooth]# remove XX:XX:XX:XX:XX:XX
   ```

3. **Restart Bluetooth**:
   ```bash
   sudo systemctl restart bluetooth
   ```

4. **Check Bluetooth adapter**:
   ```bash
   hciconfig
   # Should show hci0 UP RUNNING
   ```

### Issue: Permission Denied on /dev/ttyACM0

**Symptoms**: `Permission denied` when accessing serial port

**Solutions**:
```bash
# 1. Add user to dialout group
sudo usermod -a -G dialout $USER

# 2. Log out and back in
exit
# Then SSH back in

# 3. Verify group membership
groups
# Should include "dialout"

# 4. Check device permissions
ls -l /dev/ttyACM0
# Should show "crw-rw---- 1 root dialout"
```

### Issue: Controller Input Not Working

**Symptoms**: Controller paired but no input events

**Solutions**:
```bash
# 1. Check input devices
ls /dev/input/event*

# 2. Test with evtest
sudo evtest
# Select DualSense and press buttons

# 3. Check kernel support
uname -r
# Should be 5.12+ for native PS5 support

# 4. Update kernel if needed
sudo apt-get update
sudo apt-get dist-upgrade
```

### Issue: High CPU Temperature

**Symptoms**: Pi throttling, slow performance

**Solutions**:
```bash
# 1. Check temperature
vcgencmd measure_temp
# Should be <70°C ideally

# 2. Add heatsink to CPU

# 3. Enable fan or improve airflow

# 4. Reduce CPU governor
echo "powersave" | sudo tee /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
```

---

## Next Steps

Once hardware setup is complete:

1. **Install Rust**: See [BUILDING.md](BUILDING.md)
2. **Compile FPV Bridge**: Build the application
3. **Configure**: Edit `config/default.toml`
4. **Test**: Run `fpv-bridge` and verify operation
5. **Fly**: Pair with drone and test flight

---

## Appendix: Hardware Pinouts

### Raspberry Pi Zero 2 W GPIO (Reference)

```
     3V3  (1) (2)  5V
   GPIO2  (3) (4)  5V
   GPIO3  (5) (6)  GND
   GPIO4  (7) (8)  GPIO14 (UART TX)
     GND  (9) (10) GPIO15 (UART RX)
  GPIO17 (11) (12) GPIO18
  GPIO27 (13) (14) GND
  GPIO22 (15) (16) GPIO23
     3V3 (17) (18) GPIO24
  GPIO10 (19) (20) GND
   GPIO9 (21) (22) GPIO25
  GPIO11 (23) (24) GPIO8
     GND (25) (26) GPIO7
   GPIO0 (27) (28) GPIO1
   GPIO5 (29) (30) GND
   GPIO6 (31) (32) GPIO12
  GPIO13 (33) (34) GND
  GPIO19 (35) (36) GPIO16
  GPIO26 (37) (38) GPIO20
     GND (39) (40) GPIO21
```

**Ports**:
- **PWR**: Micro-USB (power input only)
- **USB**: Micro-USB OTG (data + power)
- **HDMI**: Mini HDMI (video output)
- **SD**: MicroSD card slot

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
