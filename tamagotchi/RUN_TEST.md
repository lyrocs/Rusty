# Display Test with QSPI Driver

## What We Fixed

1. ✅ **Display Dimensions**: Changed from 240x280 → **368x448** (correct size)
2. ✅ **QSPI Protocol**: Implemented proper command/data distinction
   - Commands prefixed with `0x02` (QSPI write command)
   - Pixel data uses `0x32` (quad write mode)
3. ✅ **Initialization Sequence**: Matches sh8601-rs reference
4. ✅ **Touch Logging**: Added to verify touch events

## Test Commands

```bash
# Flash and run
cargo run --release

# Or just monitor if already flashed
espflash monitor
```

## What to Look For

### Display Test (Should see RED screen!)

Look for these logs:
```
I (xxxx) display_qspi: Creating QSPI display driver for SH8601
I (xxxx) display_qspi: Initializing SH8601 display in QSPI mode...
I (xxxx) display_qspi: Software reset...
I (xxxx) display_qspi: Sleep out...
I (xxxx) display_qspi: Setting color mode to RGB888...
I (xxxx) display_qspi: Display on...
I (xxxx) display_qspi: SH8601 display initialized successfully!
I (xxxx) render: Sending red buffer to display using QSPI...
I (xxxx) render: 🔴 RED SCREEN sent successfully via QSPI
```

### Touch Test

Touch the screen and look for:
```
I (xxxx) input: TOUCH DETECTED at (x, y)!
I (xxxx) input: Touch released
```

## Expected Results

1. **Display**: Should show **solid RED color** across entire 368x448 screen
2. **Touch**: Should log coordinates when touched
3. **Performance**: Should maintain 60 FPS (see frame counter logs)

## If Display Still Blank

The QSPI protocol might need fine-tuning. Possible issues:

1. **Command Prefix**: Try changing `QSPI_WRITE_CMD` from `0x02` to `0x00`
2. **Data Format**: The display might expect different byte ordering
3. **Timing**: Add longer delays between commands
4. **Reset Sequence**: Try longer reset delays

## Debug Tips

Enable trace logging to see all SPI transactions:
```bash
RUST_LOG=trace cargo run --release
```

Check I2C devices:
```
I (xxxx) Found I2C device at address: 0x20 (GPIO expander)
I (xxxx) Found I2C device at address: 0x38 (Touch controller)
```

## Summary

We've created a proper QSPI driver that:
- Uses correct display size (368x448)
- Implements QSPI command/data protocol
- Follows SH8601 initialization sequence
- Should display a red test pattern

The key difference from before is the **QSPI protocol prefixes** that tell the display whether we're sending commands or data.