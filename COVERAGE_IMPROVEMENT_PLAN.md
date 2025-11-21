# Code Coverage Improvement Plan

## Current Coverage Status
```
TOTAL: 85.07% line coverage (441/2953 lines uncovered)
```

## Files with Lower Coverage (Improvement Opportunities)

### 1. **main.rs: 29.80% coverage** ⚠️ HIGH PRIORITY
- **Uncovered**: 245/349 lines (70.20% uncovered)
- **Functions missing**: 4/16 functions

#### Mockable Components:
1. **controller_task() function** (lines 70-158)
   - **What**: Async task that reads PS5 controller and sends channels
   - **Blocker**: Requires `DualSenseController::open()` (hardware)
   - **Solution**: Create trait abstraction for controller

2. **main() function** (lines 182-370)
   - **What**: Main event loop with tokio::select!
   - **Blocker**: Requires real serial port and controller
   - **Solution**: Trait-based dependency injection

#### Recommended Mocking Strategy:
```rust
// Create trait in controller/ps5.rs
pub trait ControllerDevice {
    fn device_path(&self) -> &str;
    fn fetch_events(&mut self) -> Result<Vec<InputEvent>>;
}

impl ControllerDevice for DualSenseController {
    // Existing implementation
}

#[cfg(test)]
pub struct MockController {
    events: Vec<InputEvent>,
    event_index: usize,
}
```

**Impact**: Could improve main.rs coverage to ~60-70%

---

### 2. **controller/ps5.rs: 44.64% coverage** ⚠️ HIGH PRIORITY  
- **Uncovered**: 93/168 lines (55.36% uncovered)
- **Functions missing**: 9/20 functions

#### Uncovered Code:
1. **DualSenseController::open()** (lines 86-150)
   - Scans `/dev/input/*` for DualSense
   - Requires filesystem access
   - Error paths not tested

2. **DualSenseController::fetch_events()** (lines 193-238)
   - Reads evdev events
   - Requires hardware

#### Recommended Tests:
```rust
#[test]
fn test_open_controller_with_mock_device() {
    // Mock evdev::Device::open()
    // Test device detection logic
    // Test vendor/product ID matching
}

#[test]
fn test_fetch_events_returns_correct_format() {
    // Mock evdev event stream
    // Test event buffering
    // Test error handling
}

#[test]
fn test_multiple_controllers_selects_first() {
    // Mock multiple devices
    // Verify selection logic
}
```

**Current blockers**: 
- `evdev::Device` is not a trait (external crate)
- Need wrapper trait around evdev

**Solution**:
```rust
trait EvdevDevice {
    fn input_id(&self) -> InputId;
    fn fetch_events(&mut self) -> io::Result<Vec<InputEvent>>;
}

// Wrap real device
struct RealEvdevDevice(Device);

// Create mock
struct MockEvdevDevice {
    vendor: u16,
    product: u16,
    events: Vec<InputEvent>,
}
```

**Impact**: Could improve ps5.rs coverage to ~75-85%

---

### 3. **serial/mod.rs: 80.80% coverage** ✅ GOOD (Could be better)
- **Uncovered**: 62/323 lines (19.20% uncovered)
- **Functions missing**: 4/41 functions

#### Already has good mocking via `SerialPortIO` trait! ✅

#### Missing test scenarios:
1. **Reconnection logic** (lines 120-140)
   - Multiple retry attempts
   - Exponential backoff behavior
   - Recovery after long disconnect

2. **Timeout scenarios**
   - Write timeouts
   - Flush timeouts
   - Concurrent operations

#### Recommended Additional Tests:
```rust
#[tokio::test]
async fn test_reconnect_after_multiple_failures() {
    // Simulate 3 failures then success
}

#[tokio::test]
async fn test_concurrent_send_packets() {
    // Test thread safety
    // Test buffer management
}

#[tokio::test]  
async fn test_send_packet_timeout_recovery() {
    // Test timeout handling
}
```

**Impact**: Could improve serial coverage to ~90-95%

---

### 4. **crsf/crc.rs: 85.83% coverage** ✅ GOOD
- **Uncovered**: 18/127 lines (14.17% uncovered)
- **Functions missing**: 1/16 functions

#### Uncovered Code:
1. **crc8_dvb_s2_slow()** - Marked as `#[allow(dead_code)]`
   - Used for verification but not in tests
   - Should be tested in lookup table validation

#### Missing Tests:
```rust
#[test]
fn test_crc8_lookup_table_coverage() {
    // Test all 256 table entries are correct
    for i in 0..=255u8 {
        let data = [i];
        assert_eq!(crc8_dvb_s2(&data), crc8_dvb_s2_slow(&data));
    }
}

#[test]
fn test_crc8_polynomial_edge_cases() {
    // Test specific polynomial behavior
    // Test carry bits
    // Test XOR patterns
}
```

**Impact**: Could achieve 95-100% coverage

---

### 5. **serial/port_trait.rs: 74.07% coverage** ✅ ACCEPTABLE
- **Uncovered**: 7/27 lines (25.93% uncovered)  
- **Functions missing**: 3/9 functions

#### Uncovered:
- TokioSerialPort wrapper (requires real serial port)
- Integration between trait and tokio_serial

This is acceptable as it's thin integration code.

---

## Prioritized Improvement Plan

### Phase 1: High-Value, Low-Effort (Recommended) ⭐
1. **Add CRC edge case tests** (1-2 hours)
   - Test all lookup table entries
   - Test polynomial edge cases
   - **Gain**: +10% crc.rs coverage → ~96%

2. **Add serial reconnection tests** (2-3 hours)
   - Test retry logic
   - Test exponential backoff
   - **Gain**: +5-10% serial.rs coverage → ~90%

### Phase 2: Medium Effort, High Value
3. **Create ControllerDevice trait for ps5.rs** (4-6 hours)
   - Define trait
   - Create mock implementation
   - Add 10-15 tests for device detection and event handling
   - **Gain**: +30-40% ps5.rs coverage → ~75-85%

### Phase 3: High Effort, High Value (Long-term)
4. **Refactor main.rs for testability** (8-12 hours)
   - Extract controller_task logic into testable functions
   - Create trait abstractions for dependencies
   - Add integration tests with mocks
   - **Gain**: +30-40% main.rs coverage → ~60-70%
   - **Benefit**: Better architecture, easier future testing

---

## Overall Impact

| Phase | Effort | Coverage Gain | New Total Coverage |
|-------|--------|---------------|-------------------|
| Current | - | - | 85.07% |
| Phase 1 | Low | +1-2% | ~86-87% |
| Phase 2 | Medium | +3-5% | ~89-92% |
| Phase 3 | High | +4-6% | ~93-97% |

---

## Implementation Notes

### Testing Best Practices Applied
1. ✅ Using trait abstractions (SerialPortIO)
2. ✅ Mock implementations in test modules
3. ✅ Comprehensive error path testing
4. ✅ Arc<Mutex<>> for stateful mocks
5. ✅ async_trait for async methods

### What Works Well
- **serial/mod.rs**: Excellent trait-based mocking ⭐
- **All controller modules**: 98-100% coverage ⭐  
- **CRSF protocol**: 96-100% coverage ⭐
- **Config**: 99.27% coverage ⭐

### Architectural Recommendation
The main blocker for testing `main.rs` and `ps5.rs` is **tight coupling to hardware**. Consider:

1. **Dependency Injection Pattern**
   ```rust
   struct AppDependencies {
       controller: Box<dyn ControllerDevice>,
       serial: Box<dyn SerialPortIO>,
       config: Config,
   }
   ```

2. **Factory Pattern for Testing**
   ```rust
   #[cfg(test)]
   fn create_test_dependencies() -> AppDependencies {
       AppDependencies {
           controller: Box::new(MockController::new()),
           serial: Box::new(MockSerialPort::new()),
           config: test_config(),
       }
   }
   ```

---

## Quick Wins for Next Commit

**Add these 3 tests to get quick coverage boost (~2%)**:

```rust
// In crsf/crc.rs
#[test]
fn test_crc8_all_256_table_entries() {
    for i in 0..=255u8 {
        let data = [i];
        assert_eq!(crc8_dvb_s2(&data), crc8_dvb_s2_slow(&data));
    }
}

// In serial/mod.rs  
#[tokio::test]
async fn test_multiple_reconnect_attempts() {
    // Test reconnection with mock failures
}

#[tokio::test]
async fn test_send_during_reconnect() {
    // Test queueing behavior during reconnect
}
```

