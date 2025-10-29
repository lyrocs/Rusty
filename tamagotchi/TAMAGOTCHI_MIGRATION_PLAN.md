# Tamagotchi Folder Migration Plan

## Executive Summary
Complete migration of `src/tamagotchi/` folder to clean Domain-Driven Design architecture, removing the legacy folder entirely.

## Current State Analysis

### Remaining Files in src/tamagotchi/
1. **models.rs** (974 lines) - Mixed concerns: GameState impl, MapHelper, re-exports
2. **quest_system.rs** (402 lines) - Quest game logic
3. **systems.rs** (1,789 lines) - Update, render, save systems
4. **ui.rs** (3,189 lines) - Monolithic UI rendering
5. **game_data.rs** (7 lines) - Just re-exports (can be deleted)
6. **mod.rs** (11 lines) - Module file
7. **data/** folder - JSON data files
8. **images/** folder - GIF assets

**Total to migrate: ~6,372 lines of code**

### Current Clean Architecture
```
src/
├── core/          ✅ Game state and types (200 lines)
├── hero/          ✅ Character domain (706 lines)
├── combat/        ✅ Combat domain (920 lines)
├── quest/         ✅ Quest domain (108 lines)
├── systems/       ⚠️  Input only (980 lines)
├── data/          ✅ Static data (392 lines)
├── ui/            ⚠️  Components only (470 lines)
├── display/       ✅ Display drivers
├── drivers/       ✅ Hardware drivers
├── ecs/           ✅ ECS resources
└── utils/         ✅ Utilities
```

## Migration Plan

### Phase 1: World/Map Domain Creation
**Target: Extract map-related code from models.rs**

#### 1.1 Create `src/world/` module
```
src/world/
├── mod.rs
├── location.rs      - LocationType enum
├── navigation.rs    - MapHelper, MapExit
└── constants.rs     - Map-related constants
```

**Files to modify:**
- Move `LocationType`, `MapHelper`, `MapExit` from models.rs → world/
- Update imports across codebase

### Phase 2: Game Logic Extraction
**Target: Extract GameState implementation from models.rs**

#### 2.1 Organize GameState methods by domain
```
src/core/
├── game_state.rs       - Core GameState struct (existing)
└── game_logic/
    ├── mod.rs
    ├── initialization.rs   - init_* methods
    ├── battle_logic.rs     - battle-related methods
    ├── farming_logic.rs    - farming methods
    ├── jrpg_logic.rs       - JRPG battle methods
    └── progression.rs      - level up, rewards
```

**Methods to move:**
- `init_rest_state()`, `reset_*()` → initialization.rs
- `start_battle()`, `click_battle_circle()` → battle_logic.rs
- `start_farming()`, `reset_farming()` → farming_logic.rs
- `start_jrpg_battle()`, `jrpg_*()` → jrpg_logic.rs
- `apply_battle_rewards()`, `check_level_up()` → progression.rs

### Phase 3: Quest System Migration
**Target: Move quest_system.rs to quest domain**

#### 3.1 Enhance quest module
```
src/quest/
├── mod.rs
├── models.rs          - (existing)
├── system.rs          - Quest game logic (from quest_system.rs)
├── daily.rs           - Daily quest logic
└── achievements.rs    - Achievement tracking
```

### Phase 4: Systems Module Completion
**Target: Extract remaining systems from systems.rs**

#### 4.1 Complete systems module
```
src/systems/
├── mod.rs
├── input.rs           - ✅ Already extracted (980 lines)
├── update.rs          - Game state updates (~400 lines)
├── render.rs          - Rendering system (~300 lines)
├── save.rs            - Save/load system (~200 lines)
└── animations.rs      - Animation helpers (~200 lines)
```

**Functions to extract:**
- `tamagotchi_update_system()` → update.rs
- `tamagotchi_render_system()` → render.rs
- `tamagotchi_save_system()` → save.rs
- `update_*_animation()` functions → animations.rs

### Phase 5: UI Page Extraction
**Target: Break down 3,189-line ui.rs into pages**

#### 5.1 Extract page modules
```
src/ui/pages/
├── mod.rs
├── overview.rs        - draw_overview_page (~200 lines)
├── stats.rs           - draw_stats_page (~150 lines)
├── equipment.rs       - draw_equipment_page (~500 lines)
├── farm.rs            - draw_farm_page (~300 lines)
├── rest.rs            - draw_rest_page (~150 lines)
├── battle.rs          - draw_battle_page (~400 lines)
├── jrpg_battle.rs     - draw_jrpg_battle_page (~600 lines)
├── map.rs             - draw_map_page (~250 lines)
├── menu.rs            - draw_menu (~100 lines)
├── inventory.rs       - draw_inventory (~100 lines)
├── quests.rs          - draw_quests_page (~300 lines)
└── settings.rs        - draw_settings_page (~150 lines)
```

#### 5.2 Extract UI helper functions
```
src/ui/components/
├── equipment_ui.rs    - Equipment selection/refine popups
├── quest_cards.rs     - Quest card rendering
└── menus.rs           - Generic menu helpers
```

### Phase 6: Asset Organization
**Target: Move data/ and images/ to appropriate locations**

#### 6.1 Move assets
```
src/
├── assets/
│   ├── data/
│   │   ├── enemies.json
│   │   ├── maps.json
│   │   └── quests.json
│   └── images/
│       ├── heroes/
│       ├── monsters/
│       └── backgrounds/
```

### Phase 7: Backward Compatibility Layer
**Target: Create temporary compatibility module**

#### 7.1 Create `src/compat/` for migration period
```
src/compat/
├── mod.rs
└── legacy.rs          - Re-exports for gradual migration
```

### Phase 8: Final Cleanup
**Target: Remove src/tamagotchi/ entirely**

1. Update all imports to use new modules
2. Remove all re-exports from models.rs
3. Delete src/tamagotchi/ folder
4. Update main.rs imports
5. Run full test suite

## Architectural Improvements

### 1. State Management Pattern
**Problem:** GameState has 100+ fields, becoming unwieldy
**Solution:** Split into sub-states

```rust
pub struct GameState {
    pub core: CoreState,        // Page, location, etc.
    pub hero: HeroState,         // Embedded Hero
    pub battle: BattleState,     // All battle-related
    pub ui: UIState,             // UI-specific state
    pub farm: FarmingState,      // Farming state
    pub jrpg: JrpgBattleState,   // JRPG battle state
}
```

### 2. Event System
**Problem:** Direct state mutation scattered everywhere
**Solution:** Command pattern for state changes

```rust
pub enum GameCommand {
    StartBattle(Enemy),
    NavigateTo(MapId),
    UseSkill(SkillId),
    // etc.
}
```

### 3. Resource Management
**Problem:** Static data loaded multiple times
**Solution:** Centralized resource manager

```rust
pub struct Resources {
    enemies: &'static EnemyDatabase,
    maps: &'static MapDatabase,
    items: &'static ItemDatabase,
}
```

### 4. Trait-Based Systems
**Problem:** Monolithic system functions
**Solution:** System traits

```rust
trait System {
    fn update(&mut self, state: &mut GameState, delta: u32);
}
```

## Implementation Order

1. **Week 1:** Phases 1-2 (World domain + GameState logic)
2. **Week 2:** Phases 3-4 (Quest + Systems completion)
3. **Week 3:** Phase 5 (UI pages extraction)
4. **Week 4:** Phases 6-8 (Assets + Cleanup)

## Success Metrics

- ✅ No files > 500 lines
- ✅ Clear domain boundaries
- ✅ No circular dependencies
- ✅ All tests passing
- ✅ Compilation time improved
- ✅ src/tamagotchi/ deleted

## Risk Mitigation

1. **Incremental Migration:** Each phase independently compilable
2. **Backward Compatibility:** Temporary re-exports during migration
3. **Version Control:** Git branch for each phase
4. **Testing:** Verify functionality after each phase
5. **Rollback Plan:** Keep original structure tagged

## Benefits

### Immediate
- Better code organization
- Easier to find code
- Reduced compilation times
- Parallel development enabled

### Long-term
- Easier testing
- Better maintainability
- New feature addition simplified
- Code reuse improved
- Documentation clearer

## Conclusion

This migration will transform the codebase from a legacy monolithic structure to a modern, maintainable architecture. The phased approach ensures minimal disruption while delivering immediate benefits at each step.

**Estimated Total Effort:** 4 weeks (1 developer)
**Lines to Migrate:** ~6,372
**Final Module Count:** ~40 focused modules
**Average Module Size:** <200 lines