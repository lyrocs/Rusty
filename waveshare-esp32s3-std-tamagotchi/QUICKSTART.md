# Quick Start Guide

## Phase 1: Getting Started with the STD Version

This guide will help you get the Phase 1 proof-of-concept running.

## Prerequisites

### 1. Install ESP Rust Toolchain

```bash
# Install espup
cargo install espup

# Install ESP toolchain
espup install

# Source the environment (add to your shell profile)
. $HOME/export-esp.sh
```

### 2. Install Flash Tool

```bash
cargo install espflash cargo-espflash
```

### 3. Verify Installation

```bash
# Check ESP toolchain
rustup show

# Should show something like:
# Default host: x86_64-apple-darwin
# rustup home:  /Users/your-name/.rustup
# installed targets:
# ...
# xtensa-esp32s3-espidf
```

## Building the Project

### Check Dependencies

```bash
cd waveshare-esp32s3-std-tamagotchi
cargo check
```

This will download all dependencies and verify the project compiles.

### Build for Release

```bash
cargo build --release
```

## Flashing to Device

### 1. Connect ESP32-S3

Connect your Waveshare ESP32-S3 AMOLED board via USB.

### 2. Find Serial Port

```bash
# macOS
ls /dev/cu.usbserial-*

# Linux
ls /dev/ttyUSB*
```

### 3. Flash and Monitor

```bash
# Flash and open serial monitor
cargo run --release

# Or specify port explicitly
cargo run --release -- --port /dev/cu.usbserial-XXXX
```

## Expected Behavior (Phase 1)

The proof-of-concept will:

1. Initialize ESP-IDF
2. Create shared hardware resources (display, touch)
3. Spawn worker threads:
   - Input thread on Core 1
   - Render thread on Core 0
4. Initialize Bevy ECS
5. Run game loop for 10 seconds at 60 FPS
6. Gracefully shutdown

### Console Output

You should see:
```
I (xxx) esp_image: segment 0: ...
I (xxx) main: === ESP32-S3 Tamagotchi STD Version ===
I (xxx) main: Phase 1: Proof of Concept
I (xxx) main: Initializing hardware drivers...
I (xxx) main: Spawning worker threads...
I (xxx) input: Input thread started
I (xxx) render: Render thread started
I (xxx) main: Worker threads spawned successfully
I (xxx) main: Initializing Bevy ECS...
I (xxx) main: Bevy ECS initialized
I (xxx) main: Starting main game loop...
I (xxx) game: Frame: 60
I (xxx) game: Frame: 120
...
I (xxx) main: Shutting down...
I (xxx) input: Input thread stopped
I (xxx) render: Render thread stopped
I (xxx) main: Shutdown complete
```

## Troubleshooting

### Build Errors

**Problem**: `error: failed to run custom build command for esp-idf-sys`

**Solution**:
```bash
# Make sure ESP-IDF environment is sourced
. $HOME/export-esp.sh

# Clean and rebuild
cargo clean
cargo build --release
```

**Problem**: `linker 'ldproxy' not found`

**Solution**:
```bash
cargo install ldproxy
```

### Flash Errors

**Problem**: `Failed to connect to the device`

**Solution**:
- Hold the BOOT button while connecting USB
- Try a different USB cable
- Check permissions: `sudo chmod 666 /dev/ttyUSB0` (Linux)

**Problem**: `Device not found`

**Solution**:
```bash
# Install USB drivers (macOS)
brew install libusb

# Install USB drivers (Linux)
sudo apt-get install libudev-dev
```

### Runtime Issues

**Problem**: Device resets immediately after boot

**Solution**:
- Check power supply (use good USB cable)
- Verify PSRAM is properly initialized
- Check serial monitor for panic messages

## Next Steps

Once Phase 1 is working:

1. **Verify hardware access**: Check that threads are spawning
2. **Monitor performance**: Look for 60 FPS game loop messages
3. **Test stability**: Run for extended periods

Then proceed to Phase 2:
- Implement actual hardware drivers
- Port game logic from no_std version
- Add GIF rendering with caching

## Development Tips

### Fast Iteration

```bash
# Use dev profile for faster builds (slower runtime)
cargo run

# Use release for testing performance
cargo run --release
```

### Logging Levels

Edit `sdkconfig.defaults` to change log verbosity:
```
CONFIG_LOG_DEFAULT_LEVEL_DEBUG=y  # More verbose
CONFIG_LOG_DEFAULT_LEVEL_INFO=y   # Balanced
CONFIG_LOG_DEFAULT_LEVEL_WARN=y   # Less verbose
```

### Serial Monitor Only

```bash
# Just monitor, don't flash
espflash monitor /dev/cu.usbserial-XXXX
```

## Useful Commands

```bash
# Check code without building
cargo check

# Run tests (on host, not ESP32)
cargo test

# Generate documentation
cargo doc --open

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Getting Help

- Check the [Migration Plan](../waveshare-esp32-s3-touch-amoled-1_8/STD_MULTITHREADING_MIGRATION_PLAN.md)
- Read the [ESP-IDF Rust Book](https://esp-rs.github.io/book/)
- Join the ESP Rust community on Matrix or Discord
