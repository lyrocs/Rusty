# SH8601 AMOLED Display - ESP-IDF STD Implementation Findings

## Summary

The SH8601 display controller on the Waveshare ESP32-S3 1.8" AMOLED requires **QSPI (Quad SPI) mode** specifically for pixel data transmission. While ESP-IDF supports SPI with command/address phases, it has limitations with per-phase line mode configuration that prevent proper QSPI operation.

## Hardware Configuration

**Display:** Waveshare ESP32-S3 1.8" AMOLED Touch (368x448)
**Controller:** SH8601
**Interface:** QSPI with I2C reset control

### Pin Configuration
- **SPI Pins:**
  - CLK: GPIO11
  - CS: GPIO12
  - SIO0 (MOSI): GPIO4
  - SIO1 (MISO): GPIO5
  - SIO2 (WP): GPIO6
  - SIO3 (HD): GPIO7

- **I2C Pins (for TCA9554 GPIO expander):**
  - SDA: GPIO15
  - SCL: GPIO14
  - TCA9554 Address: 0x20
  - Display Reset: EXIO3 (pin 3 on TCA9554)

## What We Attempted

### 1. Initial Standard SPI Implementation
**Status:** ❌ Failed - No display output
- Used simple `SpiDeviceDriver` with `write()` calls
- Commands and data sent sequentially without phases
- Display initialized but showed nothing

### 2. SPI Half-Duplex with Command/Address Phases
**Status:** ❌ Failed - No display output
- Used `spi_transaction_t` with proper command/address/data phases
- Command: 8-bit opcode
- Address: 24-bit (display command << 8)
- Data: Pixel data
- All phases in single-line mode
- Display initialized successfully but no visible output

### 3. Attempted QSPI Mode with QIO Flag
**Status:** ❌ Failed - ESP-IDF Error
```
E (1472) spi_master: check_trans_valid(1060): Incompatible when setting to both multi-line mode and half duplex mode
```
- Tried using `SPI_TRANS_MODE_QIO` flag
- ESP-IDF doesn't allow quad mode flags in half-duplex transactions

### 4. Extended Transaction Type
**Status:** ❌ Failed - No display output
- Used `spi_transaction_ext_t` for per-phase configuration
- Attempted to set data phase to quad mode
- No ESP-IDF errors but display remained blank

## Root Cause Analysis

### Display Requirements (from working no_std version)

Looking at the working implementation in `waveshare-esp32-s3-touch-amoled-1_8`:

```rust
// From sh8601-rs/src/displays/waveshare_18_amoled.rs
fn send_pixels(&mut self, pixels: &[u8]) -> Result<(), Self::Error> {
    // ...
    self.qspi.half_duplex_write(
        DataMode::Quad,  // <-- DATA MUST BE QUAD MODE
        Command::_8Bit(QSPI_PIXEL_OPCODE as u16, DataMode::Single),
        Address::_24Bit(ramwr_addr_val, DataMode::Single),
        0,
        chunk,
    )?;
}

fn send_command(&mut self, cmd: u8) -> Result<(), Self::Error> {
    self.qspi.half_duplex_write(
        DataMode::Single,  // <-- Commands use single mode
        Command::_8Bit(QSPI_CONTROL_OPCODE as u16, DataMode::Single),
        Address::_24Bit(address_value, DataMode::Single),
        0,
        &[],
    )?;
}
```

**Key requirement:** Per-phase line mode configuration:
- **Command phase:** Single-line mode (1 wire)
- **Address phase:** Single-line mode (1 wire)
- **Data phase:** Quad mode for pixels (4 wires), single mode for commands

### ESP-IDF Limitations

ESP-IDF's SPI master driver:
1. **Half-duplex mode** - Supports command/address/data phases ✅
2. **QSPI hardware** - ESP32-S3 has QSPI capability ✅
3. **Per-phase mode control** - **Limited/undocumented** ❌

The `spi_transaction_t` structure doesn't provide a clear way to specify:
- "Use single-line for command/address"
- "Use quad-line for data"

Flags like `SPI_TRANS_MODE_QIO` apply to the entire transaction and conflict with half-duplex mode.

### esp-hal Advantage

The working no_std version uses `esp-hal` which provides:

```rust
pub fn half_duplex_write(
    &mut self,
    data_mode: DataMode,      // Can be Quad
    command: Command,         // Independent mode control
    address: Address,         // Independent mode control
    dummy: u8,
    buffer: &[u8],
) -> Result<(), Error>
```

**esp-hal directly controls the SPI peripheral registers**, allowing proper per-phase mode configuration that ESP-IDF abstracts away.

## What Works

### ✅ Successfully Implemented

1. **I2C Communication** - TCA9554 GPIO expander works perfectly
2. **Display Reset** - Reset sequence via I2C GPIO pin works
3. **SPI Bus Initialization** - All 4 data lines configured
4. **Display Initialization** - SH8601 accepts initialization commands
5. **Command Protocol** - Command/address phase structure is correct
6. **PSRAM Support** - Enabled for framebuffer allocation (~494KB)
7. **embedded-graphics Integration** - Drawing primitives work

The display **accepts all commands** and **doesn't error**, but pixel data doesn't appear because it's transmitted on 1 line instead of 4.

## Recommendations

### Option 1: Use no_std with esp-hal (Recommended)

**Advantages:**
- ✅ Proven to work (existing waveshare project)
- ✅ Full QSPI support
- ✅ Direct hardware control
- ✅ Better performance (native DMA, QSPI)
- ✅ Same `sh8601-rs` driver

**Disadvantages:**
- ❌ No standard library
- ❌ Async-only in modern esp-hal
- ❌ Steeper learning curve

**Implementation:**
Use the existing `waveshare-esp32-s3-touch-amoled-1_8` project as reference. It already has:
- Working SH8601 display driver
- Touch support (FT3168)
- Battery monitoring (AXP2101)
- SD card support
- Bevy ECS integration

### Option 2: Hybrid Approach

**For your tamagotchi project:**
- Use **no_std with esp-hal** for display/touch/hardware
- Keep **ESP-IDF (std)** for WiFi/Bluetooth/networking features if needed

**Structure:**
```
tamagotchi/
├── firmware/        # no_std esp-hal (display, input, game logic)
└── network/         # std esp-idf (WiFi, OTA updates, etc.)
```

### Option 3: Wait for ESP-IDF Improvements

Monitor ESP-IDF for better QSPI support, but this may take time and isn't guaranteed.

## Technical Details for Future Reference

### Display Protocol

**Opcodes:**
- `QSPI_PIXEL_OPCODE = 0x32` - Used for pixel data
- `QSPI_CONTROL_OPCODE = 0x02` - Used for commands

**Commands:**
- `CMD_RAMWR = 0x2C` - Write RAM (first chunk)
- `CMD_RAMWRC = 0x3C` - Write RAM Continue (subsequent chunks)

**Transaction Structure:**
```
[Command: 8-bit] [Address: 24-bit] [Data: variable length]
   Single mode      Single mode        Quad mode (for pixels)
```

### Framebuffer Size

For 368x448 display in RGB888:
```rust
FB_SIZE = 368 * 448 * 3 = 494,592 bytes (~482 KB)
```

Requires PSRAM on ESP32-S3.

### SPI Configuration

**Working (esp-hal):**
```rust
let lcd_spi = Spi::new(peripherals.SPI2, ...)
    .with_sio0(GPIO4)
    .with_sio1(GPIO5)
    .with_sio2(GPIO6)
    .with_sio3(GPIO7)
    .with_cs(GPIO12)
    .with_sck(GPIO11)
    .with_dma(DMA_CH0)
    .with_buffers(dma_rx_buf, dma_tx_buf);
```

**ESP-IDF (attempted):**
```rust
spi_bus_config_t {
    mosi_io_num: 4,
    miso_io_num: 5,
    quadwp_io_num: 6,
    quadhd_io_num: 7,
    sclk_io_num: 11,
    // ... but can't configure per-phase modes properly
}
```

## Conclusion

The **no_std esp-hal approach is the correct choice** for this hardware. ESP-IDF's SPI driver, while powerful for many use cases, doesn't provide the fine-grained control needed for the SH8601's QSPI requirements.

The STD implementation in `tamagotchi/amoled/` serves as:
- ✅ Proof of concept for ESP-IDF structure
- ✅ Reference for I2C GPIO expander usage
- ✅ Demonstration of what doesn't work (valuable learning)

For production, use the proven `waveshare-esp32-s3-touch-amoled-1_8` no_std approach.

## Files in This POC

- `src/main.rs` - ESP-IDF implementation with QSPI attempts
- `Cargo.toml` - Dependencies (esp-idf-svc, sh8601-rs, embedded-graphics)
- `sdkconfig.defaults` - ESP32-S3 PSRAM configuration
- `DISPLAY_FINDINGS.md` - This document

## Next Steps

1. **Continue with no_std** - Use `waveshare-esp32-s3-touch-amoled-1_8` as base
2. **Port tamagotchi logic** to no_std environment
3. **Keep this as reference** for ESP-IDF learnings

---

*Created: 2025-10-29*
*Hardware: Waveshare ESP32-S3 1.8" AMOLED Touch*
*Display Controller: SH8601*
