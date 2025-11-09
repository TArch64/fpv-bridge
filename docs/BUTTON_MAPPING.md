# FPV Bridge - Button Mapping Reference

This document describes the PS5 DualSense controller button mappings and how they control your drone.

---

## Table of Contents

1. [Controller Layout](#controller-layout)
2. [Flight Controls](#flight-controls)
3. [Aux Channels (Switches)](#aux-channels-switches)
4. [Special Functions](#special-functions)
5. [Customization](#customization)
6. [Tips for Flying](#tips-for-flying)

---

## Controller Layout

### PS5 DualSense Visual Guide

```
                    ┌─────────────────────────────────┐
                    │         TOUCHPAD (Click)        │
                    │      (Calibrate Sticks)         │
                    └─────────────────────────────────┘

      ┌──────┐                                    ┌──────┐
      │  L1  │  (ARM Switch)                      │  R1  │  (Flight Mode)
      └──────┘                                    └──────┘
      ┌──────┐                                    ┌──────┐
      │  L2  │  (Beeper/Find)                     │  R2  │  (Turtle Mode)
      └──────┘                                    └──────┘


  ┌─────────┐                                  ┌─────────┐
  │    ↑    │  D-Pad Up (Rate +)                   △      │  (Flip Mode)
  │  ←   →  │  D-Pad L/R (Reserved)          ○          □ │  (Reserved)
  │    ↓    │  D-Pad Down (Rate -)                 ×      │  (Reserved)
  └─────────┘                                  └─────────┘

      ╔═════╗                                    ╔═════╗
      ║  ○  ║  Left Stick                        ║  ○  ║  Right Stick
      ║     ║  L3 Press: (Reserved)              ║     ║  R3 Press: (Reserved)
      ╚═════╝                                    ╚═════╝
      Y: Throttle (CH3)                          Y: Pitch (CH2)
      X: Yaw (CH4)                               X: Roll (CH1)


           [SHARE]  [PS]  [OPTIONS]
          (Toggle Log) (EMERGENCY DISARM) (Cycle Flight Mode)
```

---

## Flight Controls

### Primary Flight Axes

These are the main controls for flying your drone:

| Control | Input | Channel | Function | Range | Notes |
|---------|-------|---------|----------|-------|-------|
| **Roll** | Right Stick X (Left/Right) | CH1 | Bank left/right | 1000-2000μs | Left = 1000, Right = 2000 |
| **Pitch** | Right Stick Y (Up/Down) | CH2 | Nose up/down | 1000-2000μs | Down = 1000, Up = 2000 |
| **Throttle** | Left Stick Y (Up/Down) | CH3 | Motor speed | 1000-2000μs | Down = 1000, Up = 2000 |
| **Yaw** | Left Stick X (Left/Right) | CH4 | Rotate left/right | 1000-2000μs | Left = 1000, Right = 2000 |

### Stick Positions (Standard Mode 2)

```
    LEFT STICK                  RIGHT STICK

    Throttle Up                 Pitch Forward
         ▲                           ▲
         │                           │
Yaw Left ◄─┼─► Yaw Right   Roll Left ◄─┼─► Roll Right
         │                           │
         ▼                           ▼
   Throttle Down               Pitch Back

   (Mode 2 - Most Common)
```

### Stick Calibration

**Center Point**: 1500μs (no movement)

**Deadzones** (configurable in config.toml):
- Default: 5% (prevents drift)
- Adjustable: 0% to 25%

**Expo Curves** (configurable):
- Roll: 0.3 (default)
- Pitch: 0.3 (default)
- Yaw: 0.2 (default)
- Throttle: 0.0 (linear)

**To Recalibrate**:
1. Press Touchpad (click)
2. Center all sticks
3. Release touchpad
4. New center points saved

---

## Aux Channels (Switches)

### Critical Functions

| Button | Channel | Function | Behavior | Values |
|--------|---------|----------|----------|--------|
| **L1** | CH5 | **ARM Switch** | Hold 1s to arm, release to disarm | 1000 (disarmed) / 2000 (armed) |
| **R1** | CH6 | **Flight Mode** | Toggle modes | 1000 (Angle) / 1500 (Horizon) / 2000 (Acro) |

### Utility Functions

| Button | Channel | Function | Behavior | Notes |
|--------|---------|----------|----------|-------|
| **L2** | CH7 | **Beeper/Find Drone** | Trigger beeper | 2000 when pressed, 1000 released |
| **R2** | CH8 | **Turtle Mode** | Flip after crash | 2000 when pressed, 1000 released |
| **Triangle (△)** | CH9 | **Flip Mode** | Enable flips | 2000 when pressed |
| **Circle (○)** | CH10 | **Reserved** | Available for custom | - |
| **Cross (×)** | CH11 | **Reserved** | Available for custom | - |
| **Square (□)** | CH12 | **Reserved** | Available for custom | - |

### D-Pad Functions

| Button | Channel | Function | Behavior |
|--------|---------|----------|----------|
| **D-Pad Up (↑)** | CH13 | **Increase Rates** | Increment PID profile |
| **D-Pad Down (↓)** | CH14 | **Decrease Rates** | Decrement PID profile |
| **D-Pad Left (←)** | CH15 | **Reserved** | Available for custom |
| **D-Pad Right (→)** | CH16 | **Reserved** | Available for custom |

---

## Special Functions

### Emergency Controls

#### PS Button (Home)
**Function**: **EMERGENCY DISARM**

**Behavior**:
- Immediately sets CH5 (ARM) to 1000 (disarmed)
- Bypasses arm button hold timer
- Use only in emergency situations
- Motors cut instantly

**When to use**:
- Drone out of control
- Crash imminent
- Need to stop motors immediately

**Warning**: Drone will fall from sky if disarmed while flying!

---

#### Share Button
**Function**: **Toggle Telemetry Logging**

**Behavior**:
- ON → OFF: Stops logging to file
- OFF → ON: Resumes logging (new file)
- LED feedback (if available)

**Use cases**:
- Save disk space during practice
- Enable logging only for flights you want to analyze

---

#### Options Button
**Function**: **Cycle Flight Modes**

**Behavior**:
- Press to cycle: Angle → Horizon → Acro → Angle
- Same as pressing R1 multiple times
- Visual feedback in Betaflight OSD (if configured)

---

### Flight Mode Details

#### Angle Mode (CH6 = 1000μs)
**Best for**: Beginners, GPS hold

**Characteristics**:
- Self-leveling enabled
- Stick centered = level flight
- Cannot flip or roll 360°
- Very stable

**When to use**:
- First flights
- Windy conditions
- Line-of-sight flying

---

#### Horizon Mode (CH6 = 1500μs)
**Best for**: Intermediate pilots

**Characteristics**:
- Self-leveling near center
- Acro mode at full stick deflection
- Can flip and roll
- Moderate stability

**When to use**:
- Learning flips/rolls
- Freestyle practice
- Transitioning to acro

---

#### Acro Mode (CH6 = 2000μs)
**Best for**: Advanced pilots, racing, freestyle

**Characteristics**:
- No self-leveling
- Full manual control
- Can flip/roll continuously
- Requires constant correction

**When to use**:
- Racing
- Advanced freestyle
- Maximum control authority

---

## Customization

### Modifying Button Mappings

Edit `src/controller/mapper.rs` to customize button mappings:

```rust
// Example: Swap L1 and R1 functions
pub fn map_buttons(&self, state: &ControllerState) -> [u16; 16] {
    let mut channels = [1500; 16];

    // Swap ARM to R1 instead of L1
    channels[4] = if state.button_r1 { 2000 } else { 1000 };  // ARM on R1

    // Swap Flight Mode to L1 instead of R1
    channels[5] = if state.button_l1 { 2000 } else { 1000 };  // Mode on L1

    // ... rest of mappings
}
```

### Adding New Functions

To map a button to a specific channel:

```rust
// Example: Add GPS Rescue to Circle button (CH10)
channels[9] = if state.button_circle { 2000 } else { 1000 };
```

### Creating Toggle Switches

```rust
// Example: Toggle LED strip with Square button
if state.button_square && !self.prev_button_square {
    self.led_toggle = !self.led_toggle;
}
channels[10] = if self.led_toggle { 2000 } else { 1000 };
self.prev_button_square = state.button_square;
```

---

## Tips for Flying

### Pre-Flight Checklist

1. **Controller Check**:
   - ✅ Controller fully charged
   - ✅ Bluetooth connected (solid blue light)
   - ✅ Sticks centered (calibrate if needed)

2. **Drone Check**:
   - ✅ Battery charged and connected
   - ✅ Props installed correctly
   - ✅ ELRS module powered
   - ✅ Telemetry shows good link (RSSI, LQ)

3. **Safety Check**:
   - ✅ Throttle stick at minimum
   - ✅ Flight mode set to Angle (for beginners)
   - ✅ Clear area around drone
   - ✅ Know where PS button is (emergency disarm)

### Arming Procedure

```
1. Throttle stick to minimum (all the way down)
   └─> Verify: CH3 < 1050μs

2. Hold L1 button for 1 second
   └─> Watch for motor beep/LED change

3. Motors should spin slowly (armed)
   └─> Telemetry shows Armed: true

4. Slowly increase throttle to take off
   └─> Keep movements smooth
```

### Disarming Procedure

```
1. Land drone gently (throttle to minimum)

2. Reduce throttle to idle

3. Release L1 button (or press again)
   └─> Motors stop immediately
   └─> Telemetry shows Armed: false
```

### Common Mistakes

❌ **Arming with throttle up**
- System will reject arming if throttle > 1050μs
- Lower throttle fully before arming

❌ **Panic stick inputs**
- Make small, smooth corrections
- Use expo curves to reduce sensitivity

❌ **Wrong flight mode**
- Beginners: Start in Angle mode
- Don't jump to Acro too early

❌ **Forgetting to check battery**
- Always check voltage before flight
- Auto-land if voltage drops too low

---

### Stick Sensitivity

**Too Sensitive?**
- Increase expo values (0.4-0.6)
- Increase deadzones (0.08-0.12)
- Reduce rates in Betaflight

**Not Sensitive Enough?**
- Decrease expo values (0.1-0.2)
- Decrease deadzones (0.02-0.05)
- Increase rates in Betaflight

---

### Advanced Techniques

#### Flips and Rolls

1. Switch to Horizon or Acro mode (R1 or Options)
2. Gain altitude (~10m minimum)
3. Full stick deflection (left/right for roll, forward/back for flip)
4. Release stick when upright
5. Correct throttle to maintain altitude

#### Turtle Mode (After Crash)

If drone lands upside down:
1. **Hold R2** (Turtle Mode)
2. Use throttle to spin props
3. Drone should flip itself over
4. **Release R2**
5. Disarm and check for damage

---

## Button Mapping Summary Table

| Input | Channel | Default Function | Value When Pressed | Value Released |
|-------|---------|------------------|-------------------|----------------|
| Right Stick X | CH1 | Roll | 1000-2000 (analog) | 1500 (center) |
| Right Stick Y | CH2 | Pitch | 1000-2000 (analog) | 1500 (center) |
| Left Stick Y | CH3 | Throttle | 1000-2000 (analog) | 1000 (min) |
| Left Stick X | CH4 | Yaw | 1000-2000 (analog) | 1500 (center) |
| L1 | CH5 | ARM | 2000 (armed) | 1000 (disarmed) |
| R1 | CH6 | Flight Mode | Cycle modes | - |
| L2 | CH7 | Beeper | 2000 | 1000 |
| R2 | CH8 | Turtle | 2000 | 1000 |
| Triangle | CH9 | Flip Mode | 2000 | 1000 |
| Circle | CH10 | Reserved | - | 1500 |
| Cross | CH11 | Reserved | - | 1500 |
| Square | CH12 | Reserved | - | 1500 |
| D-Pad Up | CH13 | Rate + | 2000 | 1500 |
| D-Pad Down | CH14 | Rate - | 2000 | 1500 |
| D-Pad Left | CH15 | Reserved | - | 1500 |
| D-Pad Right | CH16 | Reserved | - | 1500 |
| **PS Button** | - | **Emergency Disarm** | Force CH5=1000 | - |
| **Share** | - | Toggle Logging | - | - |
| **Options** | - | Cycle Modes | Same as R1 | - |
| **Touchpad Click** | - | Calibrate Sticks | - | - |

---

## Betaflight Configuration

### Required Receiver Settings

In Betaflight Configurator:

```
Configuration → Receiver:
- Receiver Type: Serial-based receiver
- Serial Receiver Provider: CRSF
- RSSI Channel: Disabled (or AUX12 if needed)
```

### Channel Map Verification

In Betaflight Configurator → Receiver tab:

1. Move sticks and verify:
   - Roll: CH1 moves
   - Pitch: CH2 moves
   - Throttle: CH3 moves
   - Yaw: CH4 moves

2. Press buttons and verify:
   - L1: CH5 toggles
   - R1: CH6 changes
   - etc.

3. If reversed, use `channel_reverse` in config.toml:
   ```toml
   [channels]
   channel_reverse = [1, 2]  # Reverse roll and pitch
   ```

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-09
