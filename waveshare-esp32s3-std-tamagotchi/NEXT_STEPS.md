# Next Steps: Phase 2 Implementation Guide

## Overview

Now that Phase 1 is complete with a solid foundation, Phase 2 focuses on implementing actual hardware drivers and migrating game logic from the no_std version.

## Priority Order

### High Priority (Week 1-2)

#### 1. Display Driver Implementation

**File**: `src/drivers/display.rs`

**Tasks**:
- [ ] Implement SPI initialization for SH8601
- [ ] Port reset pin control via I2C GPIO expander
- [ ] Implement display initialization sequence
- [ ] Add DMA buffer management
- [ ] Implement `draw_buffer()` with actual SPI transfer
- [ ] Test with simple color fills

**Reference**:
- Original: `waveshare-esp32-s3-touch-amoled-1_8/src/bin/tamagotchi.rs:71-138`
- Crate: `sh8601-rs`

**Code Stub**:
```rust
// In display.rs
pub fn initialize_spi(
    spi: SpiDriver,
    pins: DisplayPins,
) -> Result<Sh8601DisplayDriver> {
    // 1. Configure QSPI mode
    // 2. Initialize reset pin
    // 3. Send init sequence
    // 4. Configure color mode RGB888
    todo!()
}
```

#### 2. Touch Driver Implementation

**File**: `src/drivers/touch.rs`

**Tasks**:
- [ ] Implement I2C initialization for FT3168
- [ ] Port reset pin control via GPIO expander
- [ ] Implement touch data reading
- [ ] Add gesture mode configuration
- [ ] Test touch coordinate mapping

**Reference**:
- Original: `waveshare-esp32-s3-touch-amoled-1_8/src/bin/tamagotchi.rs:101-117`
- Crate: `ft3x68-rs`

**Code Stub**:
```rust
// In touch.rs
pub fn initialize_i2c(
    i2c: I2cDriver,
    address: u8,
) -> Result<Ft3168TouchDriver> {
    // 1. Configure I2C bus
    // 2. Initialize reset pin
    // 3. Configure registers
    // 4. Enable gesture mode
    todo!()
}
```

#### 3. Basic Hardware Test

**Create**: `src/bin/hardware_test.rs`

**Purpose**: Verify hardware without full game logic

```rust
fn main() -> Result<()> {
    // Initialize display
    let display = initialize_display()?;

    // Fill screen with colors
    display.clear()?;
    display.fill_rect(0, 0, 100, 100, Color::RED)?;

    // Initialize touch
    let touch = initialize_touch()?;

    // Print touch events
    loop {
        if let Some((x, y)) = touch.read_touch() {
            println!("Touch at: ({}, {})", x, y);
        }
        thread::sleep(Duration::from_millis(10));
    }
}
```

### Medium Priority (Week 2-3)

#### 4. GIF Frame Cache Implementation

**File**: `src/graphics/gif_cache.rs` (create new)

**Tasks**:
- [ ] Create `GifCache` struct with HashMap storage
- [ ] Implement `load_gif()` to parse and cache all frames
- [ ] Store decoded frame data in memory
- [ ] Implement `get_frame()` for fast frame access
- [ ] Preload all game GIFs at startup

**Expected Performance**:
- Current: 50-150ms per frame (full parse each time)
- Target: < 5ms per frame (direct memory access)

**Code Outline**:
```rust
pub struct GifCache {
    frames: HashMap<String, Arc<Vec<FrameData>>>,
}

pub struct FrameData {
    pixels: Vec<u8>,  // Pre-decoded RGB888
    width: u16,
    height: u16,
    delay: u16,
}

impl GifCache {
    pub fn load_from_sd(&mut self, sd: &mut SdCard) -> Result<()> {
        // Load all GIFs from /sprites/ directory
        for file in sd.list_dir("/sprites")? {
            if file.ends_with(".gif") {
                self.load_gif(&file, &sd.read_file(&file)?)?;
            }
        }
        Ok(())
    }
}
```

#### 5. SD Card Implementation

**File**: `src/drivers/storage.rs`

**Tasks**:
- [ ] Implement SPI initialization for SD card
- [ ] Port CS pin control via GPIO expander (EXIO7)
- [ ] Mount filesystem using `embedded-sdmmc`
- [ ] Implement file read/write operations
- [ ] Test with JSON file loading

**Reference**:
- Original: `waveshare-esp32-s3-touch-amoled-1_8/src/bin/tamagotchi.rs:179-220`

#### 6. JSON Data Loading

**File**: `src/data/loader.rs` (create new)

**Tasks**:
- [ ] Define data structures (monsters, items, maps, quests)
- [ ] Implement JSON deserializer
- [ ] Load all game data from SD card
- [ ] Create resource caches

**Data Files to Port**:
```
/sd/
├── monsters.json
├── items.json
├── maps.json
├── quests.json
├── skills.json
└── sprites/
    ├── hero_*.gif
    ├── monster_*.gif
    └── map_*.gif
```

### Lower Priority (Week 3-4)

#### 7. GameState Migration

**File**: `src/game/state.rs` (create new)

**Tasks**:
- [ ] Port `GameState` struct from no_std version
- [ ] Convert to thread-safe version with Arc<Mutex<>>
- [ ] Migrate all game logic methods
- [ ] Update to use std collections (Vec, HashMap)

**Original**: `waveshare-esp32-s3-touch-amoled-1_8/src/tamagotchi/mod.rs`

#### 8. Combat System

**File**: `src/game/combat.rs` (create new)

**Tasks**:
- [ ] Port turn-based combat logic
- [ ] Implement auto-battle mode
- [ ] Implement manual battle mode
- [ ] Add skill system
- [ ] Add loot drops

#### 9. UI System

**File**: `src/ui/` (create directory)

**Tasks**:
- [ ] Create button abstraction layer
- [ ] Implement page navigation
- [ ] Port all UI pages (stats, inventory, skills, etc.)
- [ ] Add touch coordinate mapping

**Pages to Port**:
1. Main/Map view
2. Battle view
3. Stats page
4. Inventory page
5. Skills page
6. Quest page
7. Equipment page
8. Rest page

#### 10. Save System

**Tasks**:
- [ ] Implement save data serialization
- [ ] Add periodic auto-save
- [ ] Implement load game
- [ ] Add save integrity checks

## Implementation Strategy

### Week 1: Hardware Foundation
```
Day 1-2: Display driver
Day 3-4: Touch driver
Day 5: Hardware test program
```

### Week 2: Graphics & Storage
```
Day 1-2: GIF cache system
Day 3-4: SD card driver
Day 5: JSON data loading
```

### Week 3: Game Logic
```
Day 1-2: GameState migration
Day 3-4: Combat system
Day 5: Basic UI
```

### Week 4: Polish & Testing
```
Day 1-2: Complete UI migration
Day 3: Save system
Day 4-5: Testing and optimization
```

## Testing Checklist

### Hardware Tests
- [ ] Display shows colors correctly
- [ ] Touch responds to all areas of screen
- [ ] SD card mounts and reads files
- [ ] Battery voltage reads correctly
- [ ] Buttons work (BOOT, PWR)

### Performance Tests
- [ ] Game loop runs at 60 FPS
- [ ] Touch latency < 16ms
- [ ] GIF animations smooth
- [ ] No frame drops during combat
- [ ] Memory usage stable

### Game Logic Tests
- [ ] Character movement on map
- [ ] Combat encounters trigger
- [ ] Skills work correctly
- [ ] Inventory management
- [ ] Quest tracking
- [ ] Equipment system
- [ ] Save/load works

## Migration Tips

### From no_std to std

**Replace**:
```rust
// Old (no_std)
use heapless::Vec as HeaplessVec;
let mut items = HeaplessVec::<Item, 32>::new();

// New (std)
let mut items = Vec::new();
```

**Replace**:
```rust
// Old (no_std)
use core::cell::RefCell;
static SHARED: StaticCell<RefCell<Device>> = StaticCell::new();

// New (std)
use parking_lot::Mutex;
let shared = Arc::new(Mutex::new(Device::new()));
```

**Replace**:
```rust
// Old (no_std)
loop {
    update_game();
    render();
    Timer::after_millis(16).await;
}

// New (std)
loop {
    app.update();  // Bevy handles game logic
    thread::sleep(Duration::from_millis(16));
}
```

### Working with Threads

**Shared State**:
```rust
// Create shared resource
let game_state = Arc::new(Mutex::new(GameState::new()));

// Clone for thread
let game_state_clone = game_state.clone();
thread::spawn(move || {
    // Use in thread
    let mut state = game_state_clone.lock();
    state.update();
});
```

**Channel Communication**:
```rust
// Send from one thread
let (tx, rx) = bounded(100);
tx.send(InputEvent::Touch(x, y)).ok();

// Receive in another
while let Ok(event) = rx.try_recv() {
    handle_event(event);
}
```

## Useful Commands

### Development
```bash
# Check code
cargo check

# Build
cargo build --release

# Flash and monitor
cargo run --release

# Just monitor
espflash monitor /dev/cu.usbserial-XXX
```

### Debugging
```bash
# Increase log level
export RUST_LOG=debug
cargo run --release

# View full errors
cargo build --release --verbose

# Check specific module
cargo check --package tamagotchi-std --bin tamagotchi-std
```

### Testing on Host (before ESP32)
```bash
# Run tests on host machine
cargo test --lib

# This won't work for ESP32-specific code but good for logic
```

## Common Issues

### Issue: SPI not working
**Solution**: Check DMA buffer sizes, verify pin configuration, ensure correct SPI mode

### Issue: I2C devices not responding
**Solution**: Verify pull-up resistors, check I2C addresses, scan bus with i2cdetect equivalent

### Issue: Out of memory
**Solution**: Check PSRAM is enabled, verify allocations are freed, use frame allocator

### Issue: Thread synchronization deadlock
**Solution**: Always use `try_lock()`, never nest locks, prefer message passing

## Resources

### Documentation
- [ESP32-S3 Technical Reference](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [ESP-IDF Programming Guide](https://docs.espressif.com/projects/esp-idf/en/latest/)
- [ESP Rust Book](https://esp-rs.github.io/book/)
- [Bevy ECS Guide](https://bevyengine.org/learn/book/)

### Original Code
- no_std version: `../waveshare-esp32-s3-touch-amoled-1_8/`
- Migration plan: `../waveshare-esp32-s3-touch-amoled-1_8/STD_MULTITHREADING_MIGRATION_PLAN.md`

### Community
- ESP-RS Matrix: https://matrix.to/#/#esp-rs:matrix.org
- Rust Embedded: https://github.com/rust-embedded

## Success Criteria

Phase 2 is complete when:
- [ ] Hardware test runs successfully on actual device
- [ ] Display shows sprites from SD card
- [ ] Touch input moves character on map
- [ ] Basic combat works
- [ ] Game saves and loads
- [ ] Runs at 60 FPS consistently

## Go Build! 🚀

You now have everything needed to implement Phase 2. Start with the display driver, verify it works, then move to touch, and gradually build up the game logic.

The architecture from Phase 1 is solid - now it's time to bring it to life!
