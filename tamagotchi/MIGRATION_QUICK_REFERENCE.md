# Migration Quick Reference

## File-by-File Migration Map

### src/tamagotchi/models.rs (974 lines)
**Current Contents:**
- Lines 1-27: Re-exports for backward compatibility → **DELETE** (use new modules)
- Lines 28-33: `LocationType` enum → **world/location.rs**
- Lines 34-103: `MapHelper` struct + impl → **world/navigation.rs**
- Lines 104-110: `MapExit` struct → **world/navigation.rs**
- Lines 111-974: `GameState` impl block → **Split across core/game_logic/**
  - init methods → **core/game_logic/initialization.rs**
  - battle methods → **core/game_logic/battle_logic.rs**
  - farming methods → **core/game_logic/farming_logic.rs**
  - JRPG methods → **core/game_logic/jrpg_logic.rs**
  - reward methods → **core/game_logic/progression.rs**

### src/tamagotchi/quest_system.rs (402 lines)
**Current Contents:**
- Quest management functions → **quest/system.rs**
- Daily quest logic → **quest/daily.rs**
- Achievement logic → **quest/achievements.rs**

### src/tamagotchi/systems.rs (1,789 lines)
**Current Contents:**
- Lines 1-20: Re-exports from systems/input.rs → **DELETE**
- Lines 21-918: Already extracted to systems/input.rs → **DELETE**
- Lines 919-1050: Animation helpers → **systems/animations.rs**
- Lines 1051-1556: `tamagotchi_update_system` → **systems/update.rs**
- Lines 1557-1689: `tamagotchi_render_system` → **systems/render.rs**
- Lines 1690-1789: `tamagotchi_save_system` → **systems/save.rs**

### src/tamagotchi/ui.rs (3,189 lines)
**Current Contents:**
- Lines 1-30: Imports and colors → **Already in ui/colors.rs**
- Lines 31-227: `draw_overview_page` → **ui/pages/overview.rs**
- Lines 228-369: `draw_stats_page` → **ui/pages/stats.rs**
- Lines 370-890: `draw_equipment_page` + helpers → **ui/pages/equipment.rs**
- Lines 891-1211: `draw_farm_page` → **ui/pages/farm.rs**
- Lines 1212-1359: `draw_rest_page` → **ui/pages/rest.rs**
- Lines 1360-1753: `draw_battle_page` → **ui/pages/battle.rs**
- Lines 1754-1978: `draw_map_page` → **ui/pages/map.rs**
- Lines 1979-2071: `draw_menu` → **ui/pages/menu.rs**
- Lines 2072-2156: `draw_inventory` → **ui/pages/inventory.rs**
- Lines 2157-2434: `draw_quests_page` → **ui/pages/quests.rs**
- Lines 2435-2561: `draw_settings_page` → **ui/pages/settings.rs**
- Lines 2562-2727: GIF helpers → **Already in ui/components/gif.rs**
- Lines 2728-2862: Component helpers → **Already in ui/components/**
- Lines 2863-3189: `draw_jrpg_battle_page` → **ui/pages/jrpg_battle.rs**

### src/tamagotchi/game_data.rs (7 lines)
- Just re-exports from data module → **DELETE**

### src/tamagotchi/mod.rs (11 lines)
- Module exports → **DELETE** after migration

### src/tamagotchi/data/ (folder)
- JSON files → **src/assets/data/**

### src/tamagotchi/images/ (folder)
- GIF files → **src/assets/images/**

## Import Update Guide

### Old → New Import Mappings

```rust
// Old
use crate::tamagotchi::models::{GameState, LocationType, MapHelper};
// New
use crate::core::GameState;
use crate::world::{LocationType, MapHelper};

// Old
use crate::tamagotchi::quest_system::*;
// New
use crate::quest::system::*;

// Old
use crate::tamagotchi::systems::*;
// New
use crate::systems::*;

// Old
use crate::tamagotchi::ui::*;
// New
use crate::ui::pages::*;

// Old
use crate::tamagotchi::game_data::*;
// New
use crate::data::*;
```

## Dependencies to Update

### Files that import from tamagotchi:
1. **src/main.rs** - Update all imports
2. **src/lib.rs** - Remove tamagotchi module
3. **src/systems/input.rs** - Update GameState, MapHelper imports
4. **src/ecs/resources.rs** - May need GameState import updates

## Testing Checklist

After each phase:
- [ ] `cargo check` passes
- [ ] `cargo build` succeeds
- [ ] No unused imports warnings
- [ ] No dead code warnings
- [ ] File sizes all < 500 lines
- [ ] Module boundaries clear

## Git Commands for Safety

```bash
# Before starting
git checkout -b tamagotchi-migration
git tag pre-migration

# After each phase
git add -A
git commit -m "Migration Phase X: Description"

# If rollback needed
git checkout main
git branch -D tamagotchi-migration
```

## Module Creation Commands

```bash
# Phase 1: World domain
mkdir -p src/world
touch src/world/{mod.rs,location.rs,navigation.rs,constants.rs}

# Phase 2: Game logic
mkdir -p src/core/game_logic
touch src/core/game_logic/{mod.rs,initialization.rs,battle_logic.rs,farming_logic.rs,jrpg_logic.rs,progression.rs}

# Phase 3: Quest enhancement
touch src/quest/{system.rs,daily.rs,achievements.rs}

# Phase 4: Systems completion
touch src/systems/{update.rs,render.rs,save.rs,animations.rs}

# Phase 5: UI pages
mkdir -p src/ui/pages
touch src/ui/pages/{overview.rs,stats.rs,equipment.rs,farm.rs,rest.rs,battle.rs,jrpg_battle.rs,map.rs,menu.rs,inventory.rs,quests.rs,settings.rs}

# Phase 6: Assets
mkdir -p src/assets/{data,images/{heroes,monsters,backgrounds}}
mv src/tamagotchi/data/*.json src/assets/data/
mv src/tamagotchi/images/*.gif src/assets/images/

# Phase 8: Cleanup
rm -rf src/tamagotchi
```

## Final Verification

```bash
# Ensure no tamagotchi references remain
grep -r "tamagotchi::" src/
grep -r "src/tamagotchi" Cargo.toml build.rs

# Check for broken imports
cargo check --all-targets

# Verify no large files
find src -name "*.rs" -exec wc -l {} \; | sort -rn | head -20
```