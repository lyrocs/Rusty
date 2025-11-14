# Rustymon System Implementation Plan

## Overview
Transform the current hero-based RPG into a Rustymon collection and battle system, similar to Pokemon. Players collect monster fragments to unlock Rustymon, build teams of 4, and battle using their collected creatures.

---

## Phase 1: Core Data Structures & Models

### 1.1 New Files to Create

#### `/src/game/rustymon.rs`
```rust
// Core Rustymon structure
pub struct Rustymon {
    pub id: String,              // Unique instance ID (UUID)
    pub species_id: u32,         // Monster type (1002=Poring, 1007=Fabre, etc.)
    pub name: String,             // Species name
    pub level: u32,
    pub exp: u32,
    pub exp_to_next: u32,
    pub element: Element,

    // Base stats (randomly generated on capture)
    pub str: u32,
    pub dex: u32,
    pub vit: u32,
    pub int: u32,
    pub luk: u32,

    // Current battle stats
    pub current_hp: u32,
    pub max_hp: u32,
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub crit_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Element {
    Neutral, Water, Earth, Fire, Wind,
    Poison, Holy, Shadow, Ghost, Undead
}
```

#### `/src/game/rustymon_team.rs`
```rust
pub struct RustymonTeam {
    pub active_slots: [Option<String>; 4],  // Rustymon IDs in active team
    pub active_index: usize,                // Current battling Rustymon (0-3)
    pub bank: Vec<String>,                   // All other Rustymon IDs
}
```

#### `/src/game/fragment_collection.rs`
```rust
pub struct FragmentCollection {
    pub fragments: HashMap<u32, u32>,  // monster_id -> fragment_count
}
```

### 1.2 Modifications to Existing Files

#### `/assets/data/enemies.json`
Add to each enemy (example for Poring):
```json
{
    "name": "Poring",
    "id": 1002,
    "level": 1,
    "hp": 60,
    "attack": 13,
    "defense": 2,
    "base_exp": 150,
    "element": "water",
    "fragment_drop_rate": 0.05,
    "fragments_required": 5,
    // ... rest of existing data
}
```

Elements assignment:
- Poring: Water
- Fabre: Earth
- Hornet: Wind
- Thief Bug: Shadow

#### `/src/game/enemy.rs`
Add fields to EnemyData:
```rust
pub struct EnemyData {
    // ... existing fields ...
    pub element: Element,
    pub fragment_drop_rate: f32,
    pub fragments_required: u32,
}
```

#### `/src/game/data_loader.rs`
- Add Element enum parsing in load_enemies()
- Add fragment_drop_rate and fragments_required parsing
- Create helper function for element string to enum conversion

#### `/src/game/save.rs` (lines 14-32)
Update SaveData struct:
```rust
pub struct SaveData {
    pub version: u32,
    pub hero: Hero,
    pub kill_tracker: KillTracker,
    pub current_location_id: u32,
    pub play_time_seconds: u64,
    pub save_timestamp: u64,
    // New fields:
    pub rustymon_collection: Vec<Rustymon>,
    pub rustymon_team: RustymonTeam,
    pub fragment_collection: FragmentCollection,
}
```

#### `/src/ecs/resources.rs` (lines 145-152)
Update GameManager struct:
```rust
pub struct GameManager {
    // ... existing pages ...
    pub rustymon_page: Option<RustymonListPage>,        // Replaces equipment_page
    pub fragment_page: Option<FragmentCollectionPage>,  // Replaces inventory_page
    pub rustymon_detail_page: Option<RustymonDetailPage>,
    pub rustymon_summon_page: Option<RustymonSummonPage>,
    // ... existing fields ...
    pub rustymon_collection: Vec<Rustymon>,
    pub rustymon_team: RustymonTeam,
    pub fragment_collection: FragmentCollection,
}
```

---

## Phase 2: Fragment & Collection System

### 2.1 Battle Rewards Update

#### `/src/game/battle.rs` (modify lines 88-120)
Add fragment drop logic after enemy defeat:
```rust
pub fn handle_enemy_defeat(enemy: &Enemy, fragment_collection: &mut FragmentCollection) -> FragmentDropResult {
    let mut rng = rand::thread_rng();

    // Check for fragment drop
    if rng.gen::<f32>() < enemy.fragment_drop_rate {
        fragment_collection.add_fragment(enemy.id, 1);
        return FragmentDropResult::Dropped(enemy.id, enemy.name.clone());
    }

    FragmentDropResult::None
}
```

#### `/src/ui/pages/battle.rs`
Add fragment drop notification display:
- Show "Fragment obtained!" message
- Display fragment icon with +1 animation
- 2-second display duration

### 2.2 Rustymon Creation System

#### `/src/game/rustymon_factory.rs` (New)
```rust
use rand::Rng;
use uuid::Uuid;

pub struct RustymonFactory;

impl RustymonFactory {
    /// Create a new Rustymon from enemy data with random stats
    pub fn create_from_enemy(enemy_data: &EnemyData) -> Rustymon {
        let mut rng = rand::thread_rng();

        // Random stat ranges based on enemy level
        let base_stat = 5 + enemy_data.level;
        let variance = 5;

        let str = rng.gen_range(base_stat..base_stat + variance);
        let dex = rng.gen_range(base_stat..base_stat + variance);
        let vit = rng.gen_range(base_stat..base_stat + variance);
        let int = rng.gen_range(base_stat..base_stat + variance);
        let luk = rng.gen_range(base_stat..base_stat + variance);

        // Calculate derived stats
        let max_hp = 40 + (vit * 10) + (enemy_data.level * 5);
        let atk = 5 + (str * 2) + enemy_data.level;
        let def = 2 + vit + (enemy_data.level / 2);
        let hit = 175 + dex + enemy_data.level;
        let flee = 100 + (dex / 2) + enemy_data.level;
        let crit_rate = 5.0 + (luk as f32 * 0.3);

        Rustymon {
            id: Uuid::new_v4().to_string(),
            species_id: enemy_data.id,
            name: enemy_data.name.clone(),
            level: 1,
            exp: 0,
            exp_to_next: 100,
            element: enemy_data.element,
            str, dex, vit, int, luk,
            current_hp: max_hp,
            max_hp, atk, def, hit, flee, crit_rate,
        }
    }

    /// Calculate stats for level up
    pub fn recalculate_stats(rustymon: &mut Rustymon) {
        rustymon.max_hp = 40 + (rustymon.vit * 10) + (rustymon.level * 5);
        rustymon.atk = 5 + (rustymon.str * 2) + rustymon.level;
        rustymon.def = 2 + rustymon.vit + (rustymon.level / 2);
        rustymon.hit = 175 + rustymon.dex + rustymon.level;
        rustymon.flee = 100 + (rustymon.dex / 2) + rustymon.level;
        rustymon.crit_rate = 5.0 + (rustymon.luk as f32 * 0.3);
        rustymon.exp_to_next = rustymon.level.pow(2) * 100;
    }
}
```

---

## Phase 3: UI Pages Replacement/Modification

### 3.1 Replace Equipment Page with Rustymon Page

#### `/src/ui/pages/rustymon_list.rs` (New)
Replace `/src/ui/pages/equipment.rs`
- Display all owned Rustymon
- Show which are in active team (highlight/indicator)
- Click to open detail page
- Sort by level/element/name

#### `/src/ui/pages/rustymon_detail.rs` (New)
- Display individual Rustymon stats
- Show level, EXP bar
- Element indicator
- "Add to Team" / "Remove from Team" button
- Stats breakdown

### 3.2 Replace Inventory Page with Fragment Collection

#### `/src/ui/pages/fragment_collection.rs` (New)
Replace `/src/ui/pages/inventory.rs`
- List all monsters with fragment counts
- Show progress bars (X/Y fragments)
- Clickable "Summon" button when enough fragments
- Opens Rustymon creation preview

#### `/src/ui/pages/rustymon_summon.rs` (New)
- Preview of new Rustymon with rolled stats
- "Confirm" to add to bank
- "Cancel" to re-roll later

### 3.3 Modify Battle Page

#### `/src/ui/pages/battle.rs`
Key modifications (specific line changes):
```rust
pub struct BattlePage {
    // Replace hero with rustymon_team
    pub rustymon_team: RustymonTeam,
    pub rustymon_collection: Vec<Rustymon>,
    // Add element advantage indicator
    pub element_advantage: f32,
    // ... existing fields
}

// In draw() method:
// Line ~200-250: Replace hero sprite with rustymon sprite
// Line ~300-350: Replace hero HP bar with rustymon HP/name/level
// Line ~400: Add "Switch" button (120x40) at bottom right
// Line ~450: Add element indicator (advantage/disadvantage/neutral)

// New method for switching:
pub fn switch_rustymon(&mut self) {
    self.rustymon_team.active_index =
        (self.rustymon_team.active_index + 1) % 4;
}
```

### 3.4 Modify Map Page

#### `/src/ui/pages/map.rs`
Button replacements (specific coordinates based on current layout):
```rust
// Replace at line ~150-200 (button definitions)
// Old: "Equipment" button at (x: 10, y: 380)
// New: "Rustymon" button at same position

// Old: "Inventory" button at (x: 130, y: 380)
// New: "Fragments" button at same position

// Update handle_touch() method:
MapAction::Equipment => MapAction::Rustymon,
MapAction::Inventory => MapAction::Fragments,
```

---

## Phase 4: Battle System Overhaul

### 4.1 Battle Logic Updates

#### `/src/systems/battle.rs`
System modifications (lines to update):
```rust
// Line ~50-100: Replace hero references
fn battle_system(
    mut game_manager: NonSendMut<GameManager>,
    // ... existing params
) {
    // Use active rustymon instead of hero
    let active_rustymon = game_manager.get_active_rustymon();

    // Handle switch button (new)
    if touch.x > 300 && touch.y > 400 {
        game_manager.switch_rustymon();
    }

    // Update damage calculation to use rustymon stats
    let damage = calculate_damage(
        active_rustymon.atk,
        active_rustymon.hit,
        active_rustymon.crit_rate,
        enemy.def,
        enemy.flee,
    );

    // Apply element advantage
    let element_multiplier = get_element_advantage(
        active_rustymon.element,
        enemy.element
    );
    damage = (damage as f32 * element_multiplier) as u32;
}
```

#### `/src/game/battle.rs` (lines 34-78)
Update calculate_damage to include element system:
```rust
pub fn calculate_damage_with_element(
    attacker_atk: u32,
    attacker_hit: u32,
    attacker_crit_rate: f32,
    attacker_element: Element,
    defender_def: u32,
    defender_flee: u32,
    defender_element: Element,
) -> DamageResult {
    // Existing damage calculation
    let mut result = calculate_damage(/* existing params */);

    // Apply element modifier
    let element_modifier = get_element_advantage(
        attacker_element,
        defender_element
    );

    result.damage = (result.damage as f32 * element_modifier) as u32;
    result.element_advantage = element_modifier;

    result
}
```

### 4.2 Element System

#### `/src/game/element_system.rs` (New)
```rust
use super::Element;

/// Get damage multiplier based on element matchup
pub fn get_element_advantage(attacker: Element, defender: Element) -> f32 {
    use Element::*;

    match (attacker, defender) {
        // Strong advantages (1.5x damage)
        (Fire, Wind) | (Wind, Earth) | (Earth, Water) | (Water, Fire) => 1.5,

        // Weak disadvantages (0.5x damage)
        (Wind, Fire) | (Earth, Wind) | (Water, Earth) | (Fire, Water) => 0.5,

        // Holy vs Shadow (mutual advantage)
        (Holy, Shadow) | (Shadow, Holy) => 1.5,

        // Ghost advantage
        (Ghost, Neutral) => 1.5,
        (Neutral, Ghost) => 0.5,

        // Poison advantage
        (Poison, Holy) => 1.5,
        (Holy, Poison) => 0.5,

        // Undead immunity to poison
        (Poison, Undead) => 0.1,
        (Undead, Poison) => 1.2,

        // Neutral (same element or no advantage)
        _ => 1.0,
    }
}

/// Get element color for UI display
pub fn get_element_color(element: Element) -> Rgb888 {
    use Element::*;

    match element {
        Neutral => Rgb888::new(200, 200, 200),
        Water => Rgb888::new(100, 150, 255),
        Earth => Rgb888::new(139, 90, 43),
        Fire => Rgb888::new(255, 100, 100),
        Wind => Rgb888::new(150, 255, 150),
        Poison => Rgb888::new(150, 50, 200),
        Holy => Rgb888::new(255, 255, 150),
        Shadow => Rgb888::new(100, 50, 150),
        Ghost => Rgb888::new(200, 150, 255),
        Undead => Rgb888::new(100, 100, 100),
    }
}
```

---

## Phase 5: Systems Integration

### 5.1 New Systems to Add

#### `/src/systems/rustymon_management.rs` (New)
- Handle team switching
- Process fragment collection
- Rustymon summoning logic

#### `/src/systems/fragment_collection.rs` (New)
- Handle fragment UI interactions
- Process summoning requests

### 5.2 Modified Systems

#### `/src/systems/menu.rs`
- Update menu navigation for new pages
- Remove equipment/inventory references

#### `/src/systems/autosave.rs`
- Include Rustymon data in saves

---

## Phase 6: Migration & Cleanup

### 6.1 Data Migration
Create migration function to:
- Convert existing hero level → starter Rustymon
- Grant bonus fragments based on kill counts
- Preserve play time and progress

### 6.2 Features to Remove (After Testing)
Mark for removal but keep temporarily:
- `/src/game/hero.rs` - Hero stats system
- `/src/systems/stats_allocation.rs` - Manual stat allocation
- `/src/ui/pages/stats_allocation.rs` - Stats UI
- `/src/ui/pages/hero_overview.rs` - Hero overview
- `/src/systems/crafting.rs` - Crafting system
- `/src/ui/pages/crafting.rs` - Crafting UI
- All equipment-related code
- All crafting recipes

---

## Implementation Steps

### Step 1: Foundation (Week 1)
1. Create Rustymon data structures
2. Update enemies.json with elements and fragment data
3. Implement fragment drop system
4. Create save/load for new data

### Step 2: Collection System (Week 1-2)
1. Implement fragment collection tracking
2. Create Rustymon generation from fragments
3. Build team management system
4. Test fragment drops and collection

### Step 3: UI Replacement - Part 1 (Week 2)
1. Create Rustymon list page
2. Create Rustymon detail page
3. Replace Equipment button with Rustymon button
4. Test navigation and display

### Step 4: UI Replacement - Part 2 (Week 2-3)
1. Create fragment collection page
2. Create summoning preview page
3. Replace Inventory button with Fragments button
4. Integrate with fragment system

### Step 5: Battle System Update (Week 3)
1. Modify battle to use Rustymon stats
2. Implement switching mechanism
3. Add element advantage system
4. Update battle UI components

### Step 6: Polish & Testing (Week 4)
1. Balance stat generation
2. Tune fragment drop rates
3. Test full gameplay loop
4. Create starter Rustymon for new players

### Step 7: Migration & Cleanup (Week 4)
1. Create data migration for existing saves
2. Test migration thoroughly
3. Remove deprecated systems (keep backup)
4. Final testing

---

## Technical Considerations

### Performance (ESP32-S3 Specific)
- **Memory Constraints**: ESP32-S3 has 512KB SRAM
  - Each Rustymon struct ~120 bytes in memory
  - Limit active collection to 100 Rustymon (~12KB)
  - Use pagination (10 items per page) in UI lists
  - Consider using SD card for overflow storage

- **Display Refresh**: AMOLED 390x450 @ 60 FPS
  - Partial redraws for UI updates
  - Cache sprite data in memory
  - Minimize full screen redraws

### Save File Size
- Each Rustymon ~200 bytes serialized JSON
- 100 Rustymon = ~20KB additional save data
- SD card has plenty of space (typical 8GB+)
- Implement save file versioning for future migrations
- Consider binary format if JSON becomes too large

### Sprite Management
- **Existing Assets**: Reuse enemy sprites (40x40 pixels)
- **Memory Usage**: Each sprite ~6KB uncompressed
- **Element Variations**: Add color tint overlay for elements
- **Storage**: Embed sprites in binary (include_bytes!)

### Input Handling
- **Touch Screen**: 240x240 active area on 390x450 display
- **Boot Button**: Reserved for menu/back navigation
- **Power Button**: Reserved for system functions
- **Debounce**: 50ms for touch, 100ms for buttons

### Balance Considerations
| Enemy | Drop Rate | Fragments Required | Base Stats Range |
|-------|-----------|-------------------|-----------------|
| Poring | 5% | 5 | 6-10 |
| Fabre | 4% | 8 | 11-15 |
| Hornet | 3% | 12 | 16-20 |
| Thief Bug | 2% | 20 | 26-30 |

### EXP & Leveling
- **EXP Formula**: `level^2 * 100`
- **Stat Growth**: +1 to random stats every 5 levels
- **Max Level**: 99 (matching original hero system)
- **EXP Share**: All team members get 25% of battle EXP

---

## Testing Checklist

- [ ] Fragment drops work correctly
- [ ] Rustymon generation has proper stat ranges
- [ ] Team switching works in and out of battle
- [ ] Element advantages calculate correctly
- [ ] Save/load preserves all Rustymon data
- [ ] UI navigation flows properly
- [ ] Battle system uses Rustymon stats
- [ ] EXP and leveling work for Rustymon
- [ ] Migration preserves player progress
- [ ] Performance acceptable with 50+ Rustymon

---

## Future Enhancements (Post-Launch)

1. **Evolution System**: Rustymon evolve at certain levels
2. **Abilities**: Each Rustymon gets unique abilities
3. **Trading**: Player-to-player Rustymon trading
4. **Breeding**: Combine Rustymon for better stats
5. **Legendary Rustymon**: Rare spawns with unique mechanics
6. **PvP Battles**: Battle other players' teams
7. **Rustymon Skills**: Active skills beyond basic attacks
8. **Affection System**: Bond with Rustymon for stat boosts

---

## File Modification Summary

### New Files (11 total)
```
/src/game/
├── rustymon.rs              (~300 lines)
├── rustymon_team.rs         (~150 lines)
├── rustymon_factory.rs      (~200 lines)
├── fragment_collection.rs   (~100 lines)
└── element_system.rs        (~150 lines)

/src/ui/pages/
├── rustymon_list.rs         (~400 lines)
├── rustymon_detail.rs       (~350 lines)
├── fragment_collection.rs   (~300 lines)
└── rustymon_summon.rs       (~250 lines)

/src/systems/
├── rustymon_management.rs   (~200 lines)
└── fragment_system.rs       (~150 lines)
```

### Modified Files (15 total)
```
/assets/data/
├── enemies.json             (Add 3 fields per enemy)

/src/game/
├── battle.rs                (Add fragment drop logic)
├── data_loader.rs           (Parse new fields)
├── enemy.rs                 (Add element, fragments)
├── save.rs                  (Add rustymon collections)
├── mod.rs                   (Export new modules)

/src/ui/pages/
├── battle.rs                (Display rustymon, switch button)
├── map.rs                   (Update button labels)
├── mod.rs                   (Export new pages)

/src/systems/
├── battle.rs                (Use rustymon stats)
├── menu.rs                  (Navigate to new pages)
├── autosave.rs              (Save rustymon data)

/src/ecs/
├── resources.rs             (Add rustymon to GameManager)

/src/
└── main.rs                  (Register new systems)
```

### Files to Remove Later (8 total)
```
/src/ui/pages/
├── equipment.rs
├── inventory.rs
├── hero_overview.rs
├── stats_allocation.rs
└── crafting.rs

/src/systems/
├── stats_allocation.rs
└── crafting.rs

/src/game/
└── (Keep hero.rs temporarily for migration)
```

## Migration Strategy

### For Existing Save Files
```rust
impl SaveData {
    pub fn migrate_to_rustymon(&mut self, game_data: &GameData) {
        if self.rustymon_collection.is_empty() {
            // Create starter Rustymon from hero level
            let starter_level = self.hero.level.min(10);
            let poring_data = game_data.get_enemy(1002);

            let mut starter = RustymonFactory::create_from_enemy(poring_data);
            starter.level = starter_level;
            starter.name = "Starter Poring".to_string();
            RustymonFactory::recalculate_stats(&mut starter);

            // Add to team
            self.rustymon_collection.push(starter.clone());
            self.rustymon_team.active_slots[0] = Some(starter.id);

            // Grant bonus fragments based on kills
            for (enemy_id, kill_count) in &self.kill_tracker.kills {
                let bonus_fragments = (kill_count / 10).min(5);
                self.fragment_collection.add_fragment(*enemy_id, bonus_fragments);
            }

            // Update save version
            self.version = 2;
        }
    }
}
```

### Rollback Plan
1. Keep original hero system files (don't delete)
2. Add feature flag in Cargo.toml
3. Conditional compilation for old vs new system
4. Backup saves before migration
5. Provide manual rollback command

## Notes

- **Priority**: Start with Poring as proof of concept
- **Testing**: Use emulator first, then hardware
- **Memory**: Profile with `esp-idf-svc` memory tools
- **Performance**: Target 60 FPS, accept 30 FPS minimum
- **Battery**: Test power consumption changes
- **Backup**: Always backup saves before testing
- **Documentation**: Update README with new gameplay