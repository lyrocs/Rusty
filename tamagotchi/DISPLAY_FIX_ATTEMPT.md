# Display Fix Attempts

## What We Just Tried

### 1. Removed QSPI Protocol Prefixes ❌
**Problem**: We were adding `0x02` and `0x32` prefixes assuming QSPI
**Fix**: Using standard SPI commands directly

### 2. Added Brightness/Display Control Commands ✅
```rust
// Command 0x51 - Write Display Brightness
send_command_with_data(0x51, &[0xFF])?; // Max brightness

// Command 0x53 - Write CTRL Display
send_command_with_data(0x53, &[0x2C])?; // Enable display

// Command 0x21 - Display Inversion ON
send_command(0x21)?; // Some AMOLEDs need this
```

### 3. Extended Reset Timing ✅
- HIGH: 20ms (idle)
- LOW: 50ms (reset pulse - was 10ms)
- HIGH: 200ms (boot time - was 120ms)

### 4. Fixed Display Dimensions ✅
Changed from 240x280 → **368x448** (correct size)

## Test This Now

```bash
cargo run --release
```

**Look for:**
- Any flicker or partial image on screen
- New logs about brightness and display control
- Extended reset timing logs

## Root Cause: No QSPI Hardware Support

**The fundamental issue**: esp-idf-svc's SPI driver doesn't support QSPI

Our current setup:
```rust
// Only 2 data lines configured:
SpiDriver::new(
    peripherals.spi2,
    peripherals.pins.gpio11, // SCK
    peripherals.pins.gpio4,  // MOSI (SIO0)  ✓
    Some(peripherals.pins.gpio5), // MISO (SIO1) ✓
    // GPIO6 (SIO2) - NOT CONFIGURED ❌
    // GPIO7 (SIO3) - NOT CONFIGURED ❌
)
```

**QSPI needs 4 data lines** working in parallel, which esp-idf-svc doesn't support.

## If Screen Is Still Blank

We need **Option: Hybrid esp-hal + esp-idf-svc**

### Solution Architecture

```
┌─────────────────────────────────────┐
│         Your Application            │
│         (std environment)           │
├─────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐│
│  │   Display    │  │  Everything  ││
│  │  (esp-hal)   │  │    Else      ││
│  │    QSPI      │  │ (esp-idf-svc)││
│  └──────────────┘  └──────────────┘│
│       4-line           I2C, etc    │
│       SPI              Threading   │
│                        Heap/std    │
└─────────────────────────────────────┘
```

### Implementation Plan

1. **Add Dependencies**
```toml
[dependencies]
esp-hal = { version = "1.0.0-rc.0", features = ["esp32s3"] }
sh8601-rs = { version = "0.1.6", features = ["waveshare_18_amoled"] }
```

2. **Create Hybrid Display Module**
- Use `esp-hal` for SPI initialization (QSPI mode)
- Wrap in `Arc<Mutex<>>` for thread safety
- Keep I2C on esp-idf-svc for touch/GPIO

3. **Peripheral Management**
- Initialize esp-hal SPI first (for display)
- Then initialize esp-idf-svc I2C (for touch/expander)
- Carefully manage peripheral ownership

### Estimated Complexity
- ⏱️ **Time**: 1-2 hours
- 🔧 **Difficulty**: Medium (lifetime/ownership management)
- ✅ **Success Rate**: High (proven in working example)

## Alternative: Full esp-hal Migration

If hybrid approach has issues, convert entire project to esp-hal (no_std).

**Pros**:
- ✅ Native QSPI support
- ✅ Cleaner architecture
- ✅ Best performance

**Cons**:
- ❌ Lose std library (threading, heap, etc.)
- ❌ Major rewrite required
- ❌ More complex resource management

## Current Status

Touch Controller: ✅ Working at 0x38
Display: ❌ Blank (no QSPI hardware support)

If current fixes don't show anything on screen, we'll implement the hybrid esp-hal solution next.