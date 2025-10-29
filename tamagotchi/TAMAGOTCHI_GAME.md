# Ragnarok Online Tamagotchi Game

A Ragnarok Online-inspired Tamagotchi game for ESP32-S3 with touch AMOLED display.

## Game Overview

Raise your own Ragnarok Online character! Start as a Novice and level up through farming monsters, managing your SP (Skill Points), and watching your hero grow.

## Features

### 📊 **Overview Page**
- View hero statistics:
  - Level and Job
  - HP (Health Points) with percentage bar
  - SP (Skill Points) with percentage bar
  - EXP (Experience) with progress to next level
  - Zeny (Currency)
- Clean Ragnarok-inspired UI with colored bars

### ⚔️ **Auto Farm Page**
- Farm monsters to gain EXP and Zeny
- Fight against classic RO monsters:
  - Poring
  - Lunatic
  - Spore
- Combat features:
  - Costs 20 SP per farming session
  - 60-second (1 minute) farming duration
  - Real-time progress bar
  - Shows both hero and enemy HP
  - Displays potential rewards
- Victory screen with rewards earned

### 🧘 **Rest/Sit Page**
- Regenerate SP while your hero rests
- Visual sitting animation
- SP regeneration: +5 SP per second
- Progress bar showing SP recovery
- Auto-notification when SP is full

### 📱 **Menu System**
- Press BOOT button to open/close menu
- Touch to navigate between:
  - Overview
  - Auto Farm
  - Rest
  - Save Game (framework ready, SD card pending)
- Visual selection indicator
- Larger menu items with 55px spacing for easy touch selection

## Controls

- **BOOT Button (GPIO0)**: Toggle menu on/off
- **Touch Screen**:
  - Start farming (on Farm page)
  - Navigate menu (on Menu overlay)
  - Confirm actions (Victory/Rest complete)

## Game Mechanics

### Experience & Leveling
- Gain EXP by defeating enemies
- Level up increases HP, SP, and stats
- At Level 10, job advances from Novice to Swordsman
- EXP requirement increases by 20% per level

### SP Management
- Farming costs 20 SP
- Rest to regenerate SP at 5 SP/second
- Maximum SP increases with level

### Enemy System
- Enemies scale with hero level
- Random enemy selection (Poring, Lunatic, or Spore)
- Each enemy has unique rewards:
  - EXP rewards
  - Zeny (currency) rewards

## Technical Details

### Architecture
```
src/tamagotchi/
├── models.rs      # Game state, Hero, Enemy data structures
├── ui.rs          # All UI rendering functions
└── systems.rs     # ECS systems (button, touch, update, render)
```

### Game States
- `GamePage`: Overview, Farm, Rest, Menu
- `FarmState`: Idle, Fighting, Victory, Defeat
- `RestState`: Resting, FullSP

### Color Palette
Inspired by Ragnarok Online:
- HP Bar: Red (`220, 50, 50`)
- SP Bar: Blue (`50, 120, 220`)
- EXP Bar: Golden (`255, 200, 50`)
- Background: Dark blue (`40, 40, 60`)

## Building & Running

### Build the Tamagotchi game:
```bash
cargo build --bin tamagotchi
```

### Flash to ESP32-S3:
```bash
cargo run --bin tamagotchi --release
```

Or with espflash:
```bash
cargo espflash flash --bin tamagotchi --release --monitor
```

## Game Flow

1. **Start**: Game opens on Overview page
2. **Farm**: Press BOOT → Select "Auto Farm" → Touch to start
3. **Wait**: Watch 1-minute progress bar
4. **Victory**: Collect EXP and Zeny rewards
5. **Rest**: When SP is low, go to Rest page
6. **Level Up**: Gain EXP to level up and grow stronger!

## Recent Updates

### v0.7.0 - UI and Save System Improvements
- **Increased text sizes**: All text now uses 12px+ fonts (FONT_9X18_BOLD, FONT_9X15) for better readability
- **Battery monitoring**: Added battery voltage and percentage display on all pages
- **Enhanced menu**:
  - Increased spacing to 55px between items
  - Larger menu panel (288x328 vs 248x248)
  - Added "Save Game" button (4th menu item)
- **Save/Load framework**: Hero data serialization ready (CSV format)
  - ⚠️ **SD card support pending** due to esp-hal/embedded-sdmmc SPI trait compatibility
  - See `SD_CARD_STATUS.md` for details and solutions

## Future Enhancements

Potential features to add:
- **Enable SD card saves** (pending SPI trait compatibility fix)
- Multiple job classes (Mage, Archer, etc.)
- Item/Equipment system
- More monster types
- Skills and abilities
- Pet system
- Day/night cycle
- Quest system
- Boss battles
- Sound effects and music

## Credits

Inspired by:
- **Ragnarok Online** by Gravity Corp.
- Classic Tamagotchi virtual pets
- Embedded Rust community

## License

This is a fan project created for educational purposes.
