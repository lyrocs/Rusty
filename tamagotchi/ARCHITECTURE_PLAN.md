# Tamagotchi Architecture Improvement Plan

## Current State Analysis

### Problems Identified
1. **Monolithic Module Structure**: The `src/tamagotchi/` folder contains 8,259 lines of code with files that are too large:
   - `ui.rs`: 3,180 lines (massive UI rendering code)
   - `models.rs`: 2,547 lines (all game entities and data structures)
   - `systems.rs`: 1,790 lines (all game logic and state management)

2. **Poor Separation of Concerns**: Everything is mixed together in the tamagotchi module
   - UI rendering mixed with game logic
   - Data models mixed with business logic
   - No clear domain boundaries

3. **Underutilized Existing Architecture**: The src/ folder has good structure but it's not being used:
   - `ui/` - only has generic UI utilities, not game-specific UI
   - `ecs/` - only has resources, not properly utilizing ECS pattern
   - `utils/` - underutilized

## Proposed Architecture

### Core Design Principles
1. **Domain-Driven Design**: Organize code by domain/feature
2. **Separation of Concerns**: Clear boundaries between UI, logic, and data
3. **Single Responsibility**: Each module should have one clear purpose
4. **Scalability**: Easy to add new features without touching existing code

### New Directory Structure

```
src/
├── main.rs                    # Entry point and system initialization
├── lib.rs                      # Library exports
│
├── core/                       # Core game domain
│   ├── mod.rs
│   ├── game_state.rs          # GameState and core state management
│   ├── constants.rs           # Game constants and configuration
│   └── types.rs               # Common type aliases and enums
│
├── hero/                       # Hero domain
│   ├── mod.rs
│   ├── models.rs              # Hero struct and related enums
│   ├── stats.rs               # Stat calculations and management
│   ├── inventory.rs           # Inventory management
│   ├── equipment.rs           # Equipment system
│   └── skills.rs              # Skill system
│
├── combat/                     # Combat domain
│   ├── mod.rs
│   ├── models.rs              # Enemy, CombatResult, etc.
│   ├── battle_system.rs       # Battle logic and state machine
│   ├── jrpg_battle.rs         # JRPG battle system
│   ├── animations.rs          # Combat animations
│   └── damage.rs              # Damage calculations
│
├── quest/                      # Quest system domain
│   ├── mod.rs
│   ├── models.rs              # Quest, QuestObjective, QuestReward
│   ├── quest_manager.rs       # Quest lifecycle management
│   ├── daily_quests.rs        # Daily quest system
│   ├── achievements.rs        # Achievement system
│   └── data.rs                # Quest definitions
│
├── world/                      # World/Map domain
│   ├── mod.rs
│   ├── models.rs              # MapData, Location, MapExit
│   ├── navigation.rs          # Map navigation logic
│   ├── locations.rs           # Location definitions
│   └── npcs.rs                # NPC definitions and interactions
│
├── items/                      # Item system domain
│   ├── mod.rs
│   ├── models.rs              # Item, Equipment, etc.
│   ├── drops.rs               # Drop system and loot tables
│   ├── crafting.rs            # Crafting/refinement system
│   └── data.rs                # Item definitions
│
├── farming/                    # Farming/Idle game domain
│   ├── mod.rs
│   ├── models.rs              # FarmState and related
│   └── farming_system.rs      # Farming logic
│
├── ui/                         # UI rendering (expanded)
│   ├── mod.rs
│   ├── components/             # Reusable UI components
│   │   ├── mod.rs
│   │   ├── text.rs            # Text rendering utilities
│   │   ├── gif.rs             # GIF rendering utilities
│   │   ├── battery.rs         # Battery display
│   │   ├── bars.rs            # Health/mana bars
│   │   ├── buttons.rs         # Button components
│   │   └── menus.rs           # Menu components
│   │
│   ├── pages/                 # Page-specific rendering
│   │   ├── mod.rs
│   │   ├── overview.rs        # Overview page (from ui.rs)
│   │   ├── stats.rs           # Stats page
│   │   ├── equipment.rs       # Equipment page
│   │   ├── inventory.rs       # Inventory page
│   │   ├── farm.rs            # Farm page
│   │   ├── rest.rs            # Rest page
│   │   ├── battle.rs          # Battle page
│   │   ├── jrpg_battle.rs     # JRPG battle page
│   │   ├── map.rs             # Map page
│   │   ├── quests.rs          # Quests page
│   │   ├── settings.rs        # Settings page
│   │   └── menu.rs            # Main menu
│   │
│   └── renderer.rs            # Main rendering system
│
├── systems/                    # ECS Systems (game logic)
│   ├── mod.rs
│   ├── input.rs               # Button and touch input handling
│   ├── update.rs              # Game state updates
│   ├── render.rs              # Rendering system
│   ├── save.rs                # Save/Load system
│   └── battery.rs             # Battery monitoring
│
├── data/                       # Static game data
│   ├── mod.rs
│   ├── enemies.rs             # Enemy definitions
│   ├── maps.rs                # Map definitions
│   ├── items.rs               # Item definitions
│   ├── quests.rs              # Quest definitions
│   └── skills.rs              # Skill definitions
│
├── ecs/                        # ECS resources (keep existing)
│   ├── mod.rs
│   └── resources.rs           # Hardware and system resources
│
├── drivers/                    # Hardware drivers (keep existing)
├── display/                    # Display configuration (keep existing)
└── utils/                      # Utilities (keep existing)
```

## Implementation Plan

### Phase 1: Core Extraction (Priority: High)
**Goal**: Extract core game state and types

1. Create `src/core/` directory
2. Move `GameState` from `models.rs` to `core/game_state.rs`
3. Extract common enums (GamePage, etc.) to `core/types.rs`
4. Create `core/constants.rs` for game constants

### Phase 2: Domain Module Creation (Priority: High)
**Goal**: Create domain modules with clear boundaries

1. **Hero Domain**
   - Extract Hero struct and related code to `hero/models.rs`
   - Move inventory logic to `hero/inventory.rs`
   - Move equipment system to `hero/equipment.rs`
   - Extract stat calculations to `hero/stats.rs`

2. **Combat Domain**
   - Extract Enemy and combat-related structs to `combat/models.rs`
   - Move battle state machine to `combat/battle_system.rs`
   - Move JRPG battle system to `combat/jrpg_battle.rs`
   - Extract animation logic to `combat/animations.rs`

3. **Quest Domain**
   - Move quest models to `quest/models.rs`
   - Extract quest management to `quest/quest_manager.rs`
   - Separate daily quests logic to `quest/daily_quests.rs`

### Phase 3: UI Refactoring (Priority: Medium)
**Goal**: Break down the 3,180-line ui.rs file

1. Create `ui/components/` for reusable UI components
2. Create `ui/pages/` for page-specific rendering
3. Each page function becomes its own module:
   - `draw_overview_page` → `ui/pages/overview.rs`
   - `draw_stats_page` → `ui/pages/stats.rs`
   - etc.
4. Extract common UI utilities to components

### Phase 4: Systems Refactoring (Priority: Medium)
**Goal**: Properly utilize ECS pattern

1. Split `systems.rs` into focused system modules:
   - Input handling → `systems/input.rs`
   - Game updates → `systems/update.rs`
   - Rendering → `systems/render.rs`
   - Save/Load → `systems/save.rs`

### Phase 5: Data Extraction (Priority: Low)
**Goal**: Centralize static game data

1. Create `data/` directory for all static game data
2. Move enemy definitions to `data/enemies.rs`
3. Move map definitions to `data/maps.rs`
4. Move item definitions to `data/items.rs`

## Benefits of New Architecture

### 1. Maintainability
- **Smaller Files**: No more 3,000+ line files
- **Clear Boundaries**: Each module has a single responsibility
- **Easy Navigation**: Find code by domain, not by file type

### 2. Scalability
- **Feature Addition**: New features go in new modules
- **Minimal Impact**: Changes don't ripple through the entire codebase
- **Parallel Development**: Multiple developers can work on different domains

### 3. Testing
- **Unit Testing**: Each module can be tested independently
- **Mock Dependencies**: Clear interfaces make mocking easier
- **Integration Testing**: Domain boundaries make integration points clear

### 4. Performance
- **Compile Times**: Smaller modules compile faster
- **Code Splitting**: Only compile what changed
- **Memory Usage**: Better organization can lead to better memory layouts

## Migration Strategy

### Step-by-Step Migration
1. **Start Small**: Begin with one domain (e.g., Quest system)
2. **Test Continuously**: Ensure each step maintains functionality
3. **Incremental Changes**: Migrate one module at a time
4. **Update Imports**: Fix imports as modules move
5. **Document Changes**: Update module documentation

### Migration Order (Recommended)
1. Core extraction (GameState, types)
2. Quest system (smallest, well-contained)
3. Combat system (clear boundaries)
4. Hero system (interconnected but manageable)
5. UI refactoring (largest change)
6. Systems split (affects main.rs)

## Code Quality Guidelines

### Module Organization
```rust
// Each module should follow this structure:
pub mod models;      // Data structures
pub mod logic;       // Business logic
pub mod systems;     // ECS systems if applicable
pub mod ui;          // UI rendering if applicable

// mod.rs should only re-export public API
pub use models::*;
pub use logic::{specific_functions};
```

### Naming Conventions
- **Modules**: snake_case, domain-specific names
- **Files**: snake_case, descriptive names
- **Structs**: PascalCase
- **Functions**: snake_case, verb_noun pattern

### Documentation
- Each module must have a module-level doc comment
- Public APIs must be documented
- Complex algorithms need inline comments

## Future Considerations

### Potential Extensions
1. **Multiplayer Support**: Network module can be added
2. **Modding Support**: Data files can be externalized
3. **Save System**: Can be extended with cloud saves
4. **Analytics**: Can add analytics module

### Technology Upgrades
1. **Async/Await**: Systems can be made async
2. **WASM Support**: UI can be compiled to WASM
3. **Plugin System**: Domains can become plugins

## Conclusion

This architecture plan transforms the monolithic tamagotchi module into a well-organized, domain-driven architecture. The new structure will:
- Reduce file sizes from 3,000+ lines to <500 lines
- Create clear domain boundaries
- Enable parallel development
- Improve maintainability and testability
- Prepare the codebase for future features

The migration can be done incrementally, ensuring the game remains functional throughout the process. Start with Phase 1 (Core Extraction) as it provides the foundation for all other changes.