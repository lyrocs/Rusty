# ESP32 Tamagotchi Game - Clean Architecture

## 🎯 Overview

This ESP32 Tamagotchi game follows **Domain-Driven Design (DDD)** principles with a clean, modular architecture. All code is organized by business domain rather than technical layers, resulting in a highly maintainable and testable codebase.

## 📁 Project Structure

```
tamagotchi/
├── assets/                      # Game assets (separate from code)
│   ├── data/                    # JSON game data
│   │   ├── enemies.json         # Enemy definitions
│   │   ├── maps.json            # Map data
│   │   ├── quests.json          # Quest definitions
│   │   └── equipments.json      # Equipment items
│   └── images/                  # GIF animations
│       ├── poring/              # Monster animations
│       ├── fabre/               # Monster animations
│       ├── swordman/            # Hero animations
│       └── map/                 # Map backgrounds
│
└── src/
    ├── lib.rs                   # Module exports + backward compat layer
    │
    ├── combat/                  # Combat domain (1,457 lines)
    │   ├── models.rs            # Combat types (Enemy, BattleState, etc.)
    │   ├── battle.rs            # Base battle logic
    │   ├── battle_manual.rs     # Whac-A-Mole style battles
    │   ├── battle_jrpg.rs       # Turn-based JRPG battles
    │   ├── jrpg.rs              # JRPG combat system
    │   ├── skills.rs            # Skill system
    │   ├── skills_db.rs         # Skill definitions
    │   ├── damage.rs            # Damage calculation
    │   └── animations.rs        # Animation data (GIFs)
    │
    ├── core/                    # Core domain (398 lines)
    │   ├── game_state.rs        # GameState struct
    │   ├── types.rs             # Core types (GamePage, MapId)
    │   ├── constants.rs         # Game constants
    │   ├── farming.rs           # Farming mechanics (extension methods)
    │   └── rest.rs              # Rest/recovery (extension methods)
    │
    ├── hero/                    # Hero domain (752 lines)
    │   ├── models.rs            # Hero struct and types
    │   ├── stats.rs             # Stat management
    │   ├── level.rs             # Leveling system
    │   ├── inventory.rs         # Inventory management
    │   └── equipment.rs         # Equipment system
    │
    ├── quest/                   # Quest domain (510 lines)
    │   ├── models.rs            # Quest types
    │   └── system.rs            # Quest logic and progression
    │
    ├── world/                   # World domain (105 lines)
    │   ├── location.rs          # LocationType enum
    │   └── navigation.rs        # Map navigation (MapHelper, MapExit)
    │
    ├── data/                    # Data domain (392 lines)
    │   ├── common.rs            # LazyData pattern
    │   ├── enemies.rs           # Enemy data loading
    │   ├── maps.rs              # Map data loading
    │   ├── drops.rs             # Item drop tables
    │   ├── items.rs             # Item definitions
    │   └── npcs.rs              # NPC data
    │
    ├── systems/                 # ECS Systems (1,900 lines)
    │   ├── input.rs             # Button & touch input
    │   ├── update.rs            # Game logic update loop
    │   ├── render.rs            # Display rendering
    │   ├── save.rs              # SD card persistence
    │   └── animations.rs        # Animation helpers
    │
    ├── ui/                      # UI domain (4,137 lines)
    │   ├── colors.rs            # Color palette
    │   ├── helpers.rs           # Drawing utilities
    │   ├── components/          # Reusable UI components
    │   │   ├── bars.rs          # Progress bars
    │   │   ├── buttons.rs       # Button rendering
    │   │   ├── battery.rs       # Battery indicator
    │   │   └── ...
    │   └── pages/               # Page rendering (12 modules)
    │       ├── overview.rs      # Hero overview
    │       ├── stats.rs         # Stat allocation
    │       ├── equipment.rs     # Equipment management
    │       ├── farm.rs          # Farming page
    │       ├── rest.rs          # Rest/recovery
    │       ├── battle.rs        # Manual battle
    │       ├── jrpg_battle.rs   # JRPG battle
    │       ├── map.rs           # Map navigation
    │       ├── menu.rs          # Menu overlay
    │       ├── inventory.rs     # Item inventory
    │       ├── quests.rs        # Quest tracking
    │       └── settings.rs      # Game settings
    │
    ├── drivers/                 # Hardware drivers
    ├── display/                 # Display management
    ├── ecs/                     # ECS infrastructure (Bevy)
    ├── utils/                   # Utilities
    └── main.rs                  # Entry point
```

## 🏗️ Architecture Patterns

### 1. Domain-Driven Design (DDD)

Code is organized by **business domain** rather than technical layers:

- **Combat domain** - All battle-related code (manual battles, JRPG battles, skills, damage)
- **Hero domain** - Character management (stats, inventory, equipment, leveling)
- **Quest domain** - Quest system and progression
- **World domain** - Map navigation and locations
- **UI domain** - All rendering code organized by page

### 2. Extension Trait Pattern

GameState methods are distributed across domains using Rust's extension trait pattern:

```rust
// In src/core/farming.rs
impl GameState {
    pub fn start_farming(&mut self, enemy: Enemy) { ... }
    pub fn update_farm_progress(&mut self, delta_ms: u32) { ... }
}

// In src/combat/battle_jrpg.rs
impl GameState {
    pub fn start_jrpg_battle(&mut self, enemy: Enemy) { ... }
    pub fn jrpg_player_attack(&mut self) { ... }
}
```

**Benefits:**
- Methods live in relevant domain modules
- GameState doesn't become a monolithic "god object"
- Clear separation of concerns
- Easy to find related functionality

### 3. Entity Component System (ECS)

Uses Bevy ECS for game loop organization:

```rust
// Systems are functions that operate on game state
pub fn tamagotchi_update_system(game_state: ResMut<GameState>) { ... }
pub fn tamagotchi_render_system(game_state: ResMut<GameState>) { ... }
pub fn tamagotchi_input_system(game_state: ResMut<GameState>) { ... }
```

### 4. LazyData Pattern

Static game data is loaded once and cached:

```rust
static ENEMIES: LazyData<HeaplessVec<EnemyData, 32>> = LazyData::new();

pub fn get_enemy_data(id: u32) -> Option<&'static EnemyData> {
    let enemies = ENEMIES.get_or_init(parse_enemies);
    enemies.iter().find(|e| e.id == id)
}
```

### 5. Asset Organization

All game assets (JSON data, GIF animations) are in `assets/` folder:

```rust
// Compile-time embedding with correct relative paths
const ENEMIES_JSON: &str = include_str!("../../assets/data/enemies.json");
const PORING_IDLE: &[u8] = include_bytes!("../../assets/images/poring/0.gif");
```

## 🔄 Backward Compatibility

The legacy `tamagotchi` namespace is maintained for backward compatibility:

```rust
// In lib.rs
pub mod tamagotchi {
    // Re-exports from clean architecture modules
    pub use crate::core::{GamePage, GameState, MapId};
    pub use crate::hero::{Hero, Equipment, Inventory, ...};
    pub use crate::combat::{Enemy, BattleState, ...};
    // ... etc

    pub mod models {
        pub use super::*;
    }
}
```

**This allows existing code to continue working:**

```rust
// Old imports still work
use crate::tamagotchi::models::{GameState, Enemy, Hero};
use crate::tamagotchi::quest_system::update_quest_progress;

// New clean imports also available
use crate::core::GameState;
use crate::combat::Enemy;
use crate::hero::Hero;
use crate::quest::update_quest_progress;
```

## 📊 Module Statistics

| Module     | Lines | Purpose                                    |
|------------|-------|--------------------------------------------|
| ui         | 4,137 | 12 page modules + helpers + components     |
| systems    | 1,900 | ECS systems (input, update, render, save)  |
| combat     | 1,457 | Battle systems (manual + JRPG)             |
| hero       | 752   | Character management                       |
| quest      | 510   | Quest system                               |
| core       | 398   | GameState + farming/rest logic             |
| data       | 392   | Game data loading                          |
| world      | 105   | Map navigation                             |
| **Total**  | **9,651** | **32 domain modules**                  |

## ✨ Key Benefits

### Maintainability
- **Small, focused modules** - Largest module is 512 lines (jrpg_battle.rs)
- **Clear organization** - Easy to find where functionality lives
- **No circular dependencies** - Clean dependency graph

### Testability
- **Isolated domains** - Each domain can be tested independently
- **Extension methods** - GameState methods can be tested in isolation
- **Pure functions** - Data loading and calculations are side-effect free

### Scalability
- **Easy to add features** - New domains can be added without affecting existing code
- **Clear boundaries** - Each domain has well-defined responsibilities
- **Parallel development** - Teams can work on different domains simultaneously

### Performance
- **Compile-time assets** - All data embedded in binary (no runtime loading)
- **Lazy initialization** - Game data parsed once on first access
- **no_std compatible** - Runs on embedded ESP32 hardware

## 🚀 Development Workflow

### Adding a New Feature

1. **Identify the domain** - Which domain does this feature belong to?
2. **Create/modify module** - Add code to appropriate domain module
3. **Add tests** - Test the feature in isolation
4. **Update UI** - Add/modify relevant UI page
5. **Integrate** - Wire up in systems/update.rs if needed

### Example: Adding a New Enemy

```rust
// 1. Add to assets/data/enemies.json
{
  "id": 1008,
  "name": "Spore",
  "level": 5,
  "hp": 150,
  ...
}

// 2. Add animation GIFs to assets/images/spore/
// 0.gif (idle), 16.gif (attacking), 32.gif (dying)

// 3. Update src/combat/animations.rs
("spore", MonsterAnimation::Idle) => include_bytes!("../../assets/images/spore/0.gif"),
("spore", MonsterAnimation::Attacking) => include_bytes!("../../assets/images/spore/16.gif"),
("spore", MonsterAnimation::Dying) => include_bytes!("../../assets/images/spore/32.gif"),

// Done! The enemy is now available in the game.
```

## 📚 Additional Resources

- **MIGRATION_PROGRESS.md** - Detailed migration history and statistics
- **MIGRATION_QUICK_REFERENCE.md** - Quick reference for module locations
- **Cargo.toml** - Dependencies and build configuration
- **README.md** - Project overview and setup instructions

---

**Architecture Version:** 2.0 (Clean Architecture)
**Last Updated:** 2025-01-29
**Status:** ✅ Production Ready
