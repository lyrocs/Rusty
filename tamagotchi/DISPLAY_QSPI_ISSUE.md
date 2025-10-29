# Display QSPI Issue - Status and Solutions

## Current Status ✅ ❌

### What's Working ✅
- **Touch Controller**: Fully initialized at address 0x38 (FT3168, Device ID: 0x03)
- **GPIO Expander**: Working at address 0x20 (TCA9554)
- **I2C Bus**: All devices detected (0x18, 0x20, 0x34, 0x38, 0x51, 0x6B)
- **Display Initialization**: SH8601 accepts commands without errors
- **Multi-threading**: All threads running (main, input, render)
- **Touch Polling**: Now has logging to verify touch detection

### What's Not Working ❌
- **Display Output**: Screen remains blank despite:
  - Proper reset sequence
  - Correct SH8601 initialization commands
  - Data successfully sent (201,600 bytes for 240x280 RGB888)

## Root Cause Analysis 🔍

### The QSPI Protocol Problem

The Waveshare ESP32-S3 1.8" AMOLED uses the SH8601 controller in **QSPI mode**, which requires:

1. **Command Phase**: Sent as 1-line SPI (standard mode)
2. **Data Phase**: Sent as 4-line QSPI (quad mode)
3. **D/C Signaling**: Different wire states for commands vs data

Our current implementation uses **esp-idf-svc's SPI driver**, which:
- ❌ Treats all writes as data (no command/data distinction)
- ❌ Doesn't support QSPI mode switching
- ❌ No API for D/C (Data/Command) pin control

### Why Standard SPI Fails

```rust
// What we're doing (WRONG for SH8601):
self.spi.write(&[0x2C])?;  // Display thinks this is DATA
self.spi.write(pixel_buffer)?;  // Display thinks this is also DATA

// What we NEED (QSPI protocol):
send_command(0x2C);  // 1-line SPI, D/C=LOW
send_data_qspi(pixel_buffer);  // 4-line QSPI, D/C=HIGH
```

## Solution Options

### Option A: Hybrid esp-hal + esp-idf-svc (Recommended) ⭐

Use `esp-hal` QSPI driver for display only, keep `esp-idf-svc` for everything else.

**Pros:**
- ✅ Proper QSPI support via `sh8601-rs` crate
- ✅ Keep std environment benefits (threads, heap, logging)
- ✅ Proven to work (used in reference implementation)

**Cons:**
- ⚠️ Requires careful lifetime/ownership management
- ⚠️ Some code duplication (two SPI systems)
- ⚠️ Need to handle no_std/std boundary

**Implementation:**
```rust
// In Cargo.toml
esp-hal = { version = "1.0.0-rc.0", features = ["esp32s3"] }
sh8601-rs = { version = "0.1.6", features = ["waveshare_18_amoled"] }

// Create QSPI driver with esp-hal
let qspi = /* esp-hal QSPI setup */;
let display = Sh8601Driver::new(qspi, reset_driver)?;

// Use with std threads via Arc/Mutex
let display = Arc::new(Mutex::new(display));
```

### Option B: GPIO Bit-Banging QSPI

Implement QSPI protocol manually using GPIO pins.

**Pros:**
- ✅ Full control over protocol
- ✅ Stays in std environment

**Cons:**
- ❌ Very slow (software timing)
- ❌ Complex implementation
- ❌ High CPU usage
- ❌ Might not be fast enough for smooth rendering

### Option C: Full Migration to no_std/esp-hal

Convert entire project to no_std with esp-hal.

**Pros:**
- ✅ Clean architecture
- ✅ Native QSPI support
- ✅ Best performance

**Cons:**
- ❌ Lose std benefits (threading, heap allocations, etc.)
- ❌ Major rewrite required
- ❌ More complex resource management

### Option D: Use Different Display Driver

If SH8601 supports parallel RGB interface.

**Pros:**
- ✅ Might work with esp-idf-svc

**Cons:**
- ❌ SH8601 on Waveshare doesn't expose RGB interface
- ❌ Would require hardware modification

## Recommended Next Steps 🎯

### Immediate: Test Touch

Run the updated code to verify touch is working:
```bash
cargo run --release
```

**Touch screen and watch logs for:**
```
I (xxxx) input: TOUCH DETECTED at (120, 140)!
I (xxxx) input: Touch released
```

### Short-term: Implement Option A (Hybrid Approach)

1. **Add Dependencies**
```toml
[dependencies]
esp-hal = { version = "1.0.0-rc.0", features = ["esp32s3"] }
sh8601-rs = { version = "0.1.6", features = ["waveshare_18_amoled"] }
embedded-hal = "1.0.0"
```

2. **Create QSPI Display Module**
- Separate module `src/drivers/display_qspi.rs`
- Use esp-hal for SPI initialization
- Wrap in Arc/Mutex for thread safety
- Handle no_std/std boundary carefully

3. **Update main.rs**
- Initialize QSPI display separately
- Keep rest of code unchanged

### Long-term: Consider Full esp-hal Migration

If project grows and needs better performance, migrate to pure esp-hal architecture.

## Testing Checklist

- [x] GPIO Expander at 0x20
- [x] Touch controller at 0x38
- [x] Touch initialization (Device ID: 0x03)
- [ ] Touch events logged when screen touched
- [ ] Display shows something (even if corrupted)
- [ ] Display shows correct colors
- [ ] Smooth 60 FPS rendering

## References

- [sh8601-rs driver](https://github.com/theembeddedrustacean/sh8601-rs)
- [Waveshare working example](../waveshare-esp32-s3-touch-amoled-1_8)
- [SH8601 Datasheet](https://www.waveshare.com/w/upload/5/56/SH8601B.pdf)
- [esp-hal SPI QSPI example](https://github.com/esp-rs/esp-hal/tree/main/examples)
