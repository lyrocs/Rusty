# ESP32-S3 Tamagotchi - STD Version with Multithreading

This is a complete rewrite of the ESP32-S3 AMOLED Tamagotchi project, migrating from `no_std` to `std` environment with multithreading support.

## Project Status: Phase 1 - Foundation

This is the **Phase 1 implementation** focusing on establishing the architecture and proving the concept works.

### Goals Achieved

- ✅ ESP-IDF std environment setup
- ✅ Bevy ECS with std features
- ✅ Hardware abstraction layer
- ✅ Multithreading architecture (input + render threads)
- ✅ Thread-safe communication via channels
- ✅ Graceful shutdown mechanism

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Main Thread (Core 0)                   │
│  - Bevy ECS coordination                                 │
│  - Game state management at 60 FPS                       │
└─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼
┌──────────────┐   ┌──────────────┐
│ Input Thread │   │Render Thread │
│   (Core 1)   │   │   (Core 0)   │
├──────────────┤   ├──────────────┤
│ - Touch poll │   │ - Display    │
│ - 120Hz rate │   │ - Double buf │
│              │   │ - GIF decode │
└──────────────┘   └──────────────┘
```

## Hardware Support

### ESP32-S3 Waveshare AMOLED 1.8"

- **Display**: SH8601 AMOLED (240x280, QSPI)
- **Touch**: FT3168 capacitive touch controller
- **CPU**: Dual-core Xtensa LX7 @ 240MHz
- **RAM**: PSRAM support @ 80MHz
- **Storage**: SD card via SPI
- **Power**: AXP2101 PMIC for battery management

### Pin Configuration

See `src/hal/pins.rs` for complete pin mapping.

## Building

### Prerequisites

1. **ESP-IDF Toolchain**:
   ```bash
   # Install espup
   cargo install espup
   espup install
   ```

2. **ESP Rust Toolchain**:
   ```bash
   # Source the environment
   . $HOME/export-esp.sh
   ```

3. **Flash Tool**:
   ```bash
   cargo install espflash
   ```

### Build Commands

```bash
# Build the project
cargo build --release

# Flash to device
cargo run --release

# Monitor serial output
espflash monitor
```

## Project Structure

```
src/
├── main.rs              # Application entry point
├── hal/                 # Hardware abstraction layer
│   ├── mod.rs          # Trait definitions
│   ├── pins.rs         # Pin configuration
│   └── config.rs       # System configuration
├── drivers/            # Hardware driver implementations
│   ├── display.rs      # SH8601 display driver
│   ├── touch.rs        # FT3168 touch driver
│   ├── storage.rs      # SD card storage
│   ├── button.rs       # Button input
│   └── power.rs        # AXP2101 power management
├── types/              # Common types and events
│   └── mod.rs
├── systems/            # Bevy ECS systems
│   ├── input.rs        # Input processing
│   ├── render.rs       # Render commands
│   └── game.rs         # Game logic
└── threads/            # Worker thread implementations
    ├── input.rs        # Input polling thread
    └── render.rs       # Rendering thread
```

## Key Features

### Thread-Safe Hardware Access

All hardware drivers implement traits with `Send` bound, wrapped in `Arc<Mutex<>>`:
- `DisplayDriver` - Thread-safe display operations
- `TouchDriver` - Thread-safe input reading
- `StorageDriver` - Thread-safe file operations
- `PowerDriver` - Thread-safe battery monitoring

### Inter-Thread Communication

Using `crossbeam-channel` for lock-free communication:
- `InputEvent` - From input thread to game logic
- `RenderCommand` - From game logic to render thread

### Double Buffering

Render thread maintains two frame buffers to eliminate tearing:
1. Back buffer receives draw commands
2. Front buffer transfers to display
3. Buffers swap on `Present` command

## Performance Targets

- **Game Logic**: 60 FPS (16.67ms per frame)
- **Input Polling**: 120 Hz (8ms interval)
- **Render Thread**: As fast as possible, non-blocking
- **Touch Latency**: < 16ms from touch to response

## Next Steps (Phase 2)

- [ ] Implement actual hardware initialization
- [ ] Port SH8601 display driver to ESP-IDF HAL
- [ ] Port FT3168 touch driver to ESP-IDF HAL
- [ ] Implement GIF frame caching
- [ ] Port JSON data loading
- [ ] Migrate game logic from no_std version
- [ ] Add SD card filesystem support
- [ ] Implement sprite rendering
- [ ] Add battery monitoring UI

## Differences from no_std Version

### Removed Constraints

- ❌ No more `embedded-hal` 0.x limitations
- ❌ No more `heapless` fixed-size collections
- ❌ No more blocking single-threaded loop
- ❌ No more GIF parsing on every frame

### Added Capabilities

- ✅ Full `std` library access
- ✅ Dynamic allocations with `Vec`, `HashMap`, etc.
- ✅ True multithreading with `std::thread`
- ✅ Proper `Mutex` and `RwLock` synchronization
- ✅ Channel-based communication
- ✅ Better error handling with `anyhow`

## Performance Comparison

| Metric | no_std Version | std Version (Target) |
|--------|----------------|---------------------|
| FPS during GIF | 10 | 60 |
| Touch Latency | 200-600ms | < 16ms |
| CPU Cores Used | 1 | 2 |
| Input Event Loss | 30-40% | < 1% |

## License

Same as original project.

## References

- [Migration Plan](../waveshare-esp32-s3-touch-amoled-1_8/STD_MULTITHREADING_MIGRATION_PLAN.md)
- [ESP-IDF Rust Book](https://esp-rs.github.io/book/)
- [Bevy ECS Documentation](https://bevyengine.org/learn/book/)
- [ESP32-S3 Technical Reference](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
