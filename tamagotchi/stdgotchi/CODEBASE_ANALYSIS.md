# Rustymon Game Codebase Structure Analysis

## Executive Summary

The Rustymon game is an ESP32-S3 embedded RPG system built with Rust and Bevy ECS. It's transitioning from a hero-based system to a Pokemon-like Rustymon collection battle system. The codebase is well-structured with clear separation between game logic, UI, and systems.

---

## 1. Current Battle System Implementation

### Damage Calculation Logic
**Location**: `/src/game/battle.rs` (lines 44-89)

```rust
pub fn calculate_damage(attacker_atk: u32, attacker_hit: u32, attacker_crit_rate: f32,
                       defender_def: u32, defender_flee: u32) -> DamageResult
```

**Key Features**:
- **Base Damage**: `attacker_atk - defender_def` (minimum 1)
- **Variance**: Random 80-120% of base damage
- **Hit/Miss System**: Hit chance = `80% + (attacker_hit - defender_flee) / 2`, clamped 20-95%
- **Critical Hits**: 2x damage multiplier when triggered
- **Element Advantage**: Applied as damage multiplier (1.5x, 1.0x, 0.5x, etc.)

**Attack Functions**:
1. `hero_attack()` - Hero vs Enemy
2. `enemy_attack()` - Enemy vs Hero
3. `rustymon_attack_enemy()` - Rustymon vs Enemy with element advantage
4. `enemy_attack_rustymon()` - Enemy vs Rustymon with element advantage

### Battle State Management
**Location**: `/src/game/battle.rs` (lines 28-42)

```rust
pub struct BattleState {
    pub hero_last_attack: f64,      // Timestamp of last hero attack
    pub enemy_last_attack: f64,     // Timestamp of last enemy attack
}
```

### Battle System Integration
**Location**: `/src/systems/battle.rs`

- Handles input events during battle mode
- Processes team switching via touch input
- Boot button opens menu from battle
- `battle_page.handle_touch()` returns `BattleAction::SwitchRustymon(slot)`

---

## 2. Rustymon & Enemy Data Structure

### Rustymon Model
**Location**: `/src/game/rustymon.rs` (lines 59-119)

```rust
pub struct Rustymon {
    // Identity
    pub id: String,                  // Unique UUID
    pub species_id: u32,             // Monster type ID (1002=Poring, etc.)
    pub name: String,                // Species name
    
    // Level & Experience
    pub level: u32,                  // 1-99
    pub exp: u32,                    // Current EXP
    pub exp_to_next: u32,            // EXP required for next level
    
    // Element
    pub element: Element,
    
    // Base Stats (randomly generated on capture)
    pub str: u32,    // Strength - affects ATK
    pub dex: u32,    // Dexterity - affects HIT/FLEE
    pub vit: u32,    // Vitality - affects HP/DEF
    pub int: u32,    // Intelligence - for future magic
    pub luk: u32,    // Luck - affects CRIT
    
    // Derived Combat Stats
    pub current_hp: u32,
    pub max_hp: u32,
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub crit_rate: f32,
}
```

### Stat Calculation (Rustymon)
**Location**: `/src/game/rustymon.rs` (lines 169-191)

```
max_hp = 40 + (vit * 10) + (level * 5)
atk    = 5 + (str * 2) + level
def    = 2 + vit + (level / 2)
hit    = 175 + dex + level
flee   = 100 + (dex / 2) + level
crit   = 5.0 + (luk * 0.3)
```

### Enemy Data JSON Structure
**Location**: `/assets/data/enemies.json`

```json
{
  "name": "Poring",
  "id": 1002,
  "level": 1,
  "hp": 50,
  "attack": 7,
  "defense": 0,
  "hit": 22,
  "flee": 82,
  "base_exp": 150,
  "element": "water",
  "fragment_drop_rate": 0.3,
  "fragments_required": 3,
  "str": 1, "agi": 1, "int": 0, "dex": 6, "vit": 1, "luk": 30,
  "drops": [
    {"item_id": 901, "name": "Jellopy", "drop_rate": 150, "min_quantity": 1, "max_quantity": 2}
  ]
}
```

### Enemy Model
**Location**: `/src/game/enemy.rs` (lines 10-23)

```rust
pub struct Enemy {
    pub id: u32,
    pub name: String,
    pub level: u32,
    pub current_hp: u32,
    pub max_hp: u32,
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub exp_reward: u64,
    pub element: Element,
}
```

### Level Scaling
**Location**: `/src/game/enemy.rs` (lines 44-81)

Enemies scale based on hero level:
```rust
scaling_level = (hero_level + 2).min(50)
level_modifier = 1.0 + ((scaling_level - base_level) * 0.1)
scaled_stats = base_stats * level_modifier
```

### Element Enum
**Location**: `/src/game/rustymon.rs` (lines 9-21)

```rust
pub enum Element {
    Neutral, Water, Earth, Fire, Wind,
    Poison, Holy, Shadow, Ghost, Undead
}
```

---

## 3. Element System & Advantages

**Location**: `/src/game/element_system.rs`

### Element Advantage Multipliers

| Attacker | Defender | Multiplier | Advantage Text |
|----------|----------|------------|-----------------|
| Fire | Wind | 1.5x | "Effective" |
| Wind | Earth | 1.5x | "Effective" |
| Earth | Water | 1.5x | "Effective" |
| Water | Fire | 1.5x | "Effective" |
| Holy | Undead | 2.0x | "SUPER EFFECTIVE!" |
| Holy | Shadow | 1.5x | "Effective" |
| Poison | Holy | 1.5x | "Effective" |
| Ghost | Neutral | 1.5x | "Effective" |
| Reverse matchups | | 0.5x | "Not Very Effective" |
| Poison | Undead | 0.1x | "Resisted!" |

### Element Colors (UI Display)
```rust
Neutral → Gray (200,200,200)
Water   → Blue (100,150,255)
Earth   → Brown (139,90,43)
Fire    → Red (255,100,100)
Wind    → Light Green (150,255,150)
Poison  → Purple (150,50,200)
Holy    → Light Yellow (255,255,150)
Shadow  → Dark Purple (100,50,150)
Ghost   → Lavender (200,150,255)
Undead  → Dark Gray (100,100,100)
```

### Element Icons
```
Neutral: ○
Water:   ≈
Earth:   ▲
Fire:    ※
Wind:    ~
Poison:  ☠
Holy:    ☼
Shadow:  ◆
Ghost:   ♦
Undead:  †
```

---

## 4. Battle Page Implementation

**Location**: `/src/ui/pages/battle.rs`

### BattlePage Structure
```rust
pub struct BattlePage {
    background: Option<Background>,
    background_color: Rgb888,
    hero: Option<BattleEntity>,        // Hero sprite animation
    enemy: Option<BattleEntity>,       // Enemy sprite animation
    fps: f32,
    first_draw: bool,
    
    // Game state
    game_hero: Hero,
    game_enemy: Option<GameEnemy>,
    kill_tracker: KillTracker,
    game_data: GameData,
    
    // Rustymon system
    rustymon_collection: Vec<Rustymon>,
    rustymon_team: RustymonTeam,
    fragment_collection: FragmentCollection,
    
    // Animations
    damage_numbers: Vec<DamageNumber>,  // Floating damage numbers
    last_hp_regen: Instant,
    
    // UI
    touch_areas: Vec<TouchArea>,
    asset_loader: Option<AssetLoader<SdCardWrapper>>,
    fragment_drops: Vec<(u32, String)>,
    fragment_notification: Option<(String, Instant)>,
    
    hero_died: bool,
}
```

### BattleEntity Animation States
**Location**: `/src/ui/pages/battle.rs` (lines 26-56)

```rust
pub enum AnimationType {
    Idle,      // Standing/waiting
    Attack,    // Attacking animation
    Attacked,  // Recoil from hit
    Death,     // Death animation
}

pub struct BattleEntity {
    idle_sprite: AnimatedSprite,
    attack_sprite: AnimatedSprite,
    attacked_sprite: AnimatedSprite,
    death_sprite: Option<AnimatedSprite>,
    current_animation: AnimationType,
    role: EntityRole,                 // Hero or Enemy
    last_attack_time: Instant,
    attack_interval: Duration,
    is_dead: bool,
    attack_damage_dealt: bool,        // Track damage timing
}
```

### Damage Floating Numbers
**Location**: `/src/ui/pages/battle.rs` (lines 223-267)

```rust
pub struct DamageNumber {
    value: u32,
    position: (i32, i32),
    start_time: Instant,
    duration: Duration,               // 800ms animation
    is_critical: bool,
    is_miss: bool,
}
```

Animation behavior:
- Floats upward 50 pixels
- Fades out over 800ms
- Color indicates critical/miss state

### Battle Touch Interactions
**Location**: `/src/ui/pages/battle.rs` (lines 269-289)

```rust
pub struct TouchArea {
    bounds: (i32, i32, u32, u32),    // (x, y, width, height)
    action: BattleAction,
}

pub enum BattleAction {
    SwitchRustymon(usize),            // Switch to team slot
}
```

### Key Display Elements
- **HP Bars**: Visual representation with color coding
- **EXP Bars**: Progress toward next level
- **Team Slots**: Display active and available Rustymon
- **Enemy HP**: Shows current/max HP
- **Turn Indicators**: Who's attacking

---

## 5. Existing Stats/Modifiers System

### Hero Stats System
**Location**: `/src/game/stats.rs` (lines 7-92)

```rust
pub struct Stats {
    pub str: u32,  // Strength - Physical attack
    pub agi: u32,  // Agility - Attack speed, flee
    pub vit: u32,  // Vitality - Max HP, defense
    pub int: u32,  // Intelligence - Max SP, magic
    pub dex: u32,  // Dexterity - Hit rate
    pub luk: u32,  // Luck - Critical rate
}

impl Stats {
    pub fn calculate_max_hp(&self, base_hp: u32, level: u32) -> u32 {
        base_hp + (self.vit * 10) + (level * 5)
    }
    
    pub fn calculate_atk(&self) -> u32 {
        20 + (self.str * 5) + (self.dex / 2)
    }
    
    pub fn calculate_def(&self) -> u32 {
        self.vit + (self.agi / 4)
    }
    
    pub fn calculate_hit(&self, level: u32) -> u32 {
        (self.dex * 2) + (self.luk / 2) + level
    }
    
    pub fn calculate_flee(&self, level: u32) -> u32 {
        (self.agi * 2) + (self.luk / 3) + level
    }
    
    pub fn calculate_crit_rate(&self) -> f32 {
        (self.luk as f32 / 10.0).min(30.0)
    }
    
    pub fn calculate_attack_interval(&self) -> u64 {
        let modifier = 1.0 + (self.agi as f32 / 50.0);
        (2000.0 / modifier) as u64
    }
    
    pub fn calculate_hp_regen(&self) -> u32 {
        (self.vit / 5).max(1)
    }
    
    pub fn calculate_sp_regen(&self) -> u32 {
        (self.int / 10).max(1)
    }
}
```

### Hero Combat Stats
**Location**: `/src/game/hero.rs` (lines 78-105)

```rust
pub struct Hero {
    pub name: String,
    pub job: Job,
    pub level: u32,
    pub exp: u64,
    pub exp_to_next_level: u64,
    pub stats: Stats,
    pub stat_points: u32,
    
    // HP/SP
    pub current_hp: u32,
    pub max_hp: u32,
    pub current_sp: u32,
    pub max_sp: u32,
    
    // Combat
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub crit_rate: f32,
    
    // Equipment
    pub inventory: Inventory,
    pub equipped_items: EquippedItems,
    pub gold: u32,
}
```

### Job System
**Location**: `/src/game/hero.rs` (lines 11-75)

Jobs provide base stat modifiers:
```rust
pub enum Job {
    Novice,    // 1, base: 5,5,5,5,5,5
    Swordsman, // 10+, base: 10,7,10,5,7,5
    Knight,    // 40+, base: 15,10,15,7,10,7
}
```

---

## 6. Level System Implementation

### Rustymon Leveling
**Location**: `/src/game/rustymon.rs` (lines 164-237)

```rust
// EXP required for next level
fn calculate_exp_to_next(level: u32) -> u32 {
    level.pow(2) * 100
}

// Level up mechanics
pub fn gain_exp(&mut self, exp: u32) -> bool {
    self.exp += exp;
    if self.exp >= self.exp_to_next {
        self.level_up();
        return true;
    }
    false
}

fn level_up(&mut self) {
    if self.level >= 99 {
        return;  // Max level
    }
    
    self.level += 1;
    self.exp = 0;
    
    // Randomly increase one stat by 1
    let stat_choice = rng.gen_range(0..5);
    match stat_choice {
        0 => { self.str += 1; },
        1 => { self.dex += 1; },
        2 => { self.vit += 1; },
        3 => { self.int += 1; },
        4 => { self.luk += 1; },
        _ => {}
    }
    
    self.recalculate_stats();
    
    // Heal HP gained from leveling
    let hp_gained = self.max_hp - old_max_hp;
    self.current_hp += hp_gained;
}
```

### Hero Leveling
**Location**: `/src/game/hero.rs` (lines 140-150+)

```rust
pub fn calculate_exp_for_level(level: u32) -> u64 {
    ((level as u64).pow(3)) * 10  // level^3 * 10
}

pub fn gain_exp(&mut self, exp: u64) {
    self.exp += exp;
    // Check for level up
}
```

---

## 7. Rustymon Team Management

**Location**: `/src/game/rustymon_team.rs`

### Team Structure
```rust
pub struct RustymonTeam {
    pub active_slots: [Option<String>; 4],  // Up to 4 active Rustymon IDs
    pub active_index: usize,                 // Current battling index (0-3)
    pub bank: Vec<String>,                   // Storage for additional Rustymon
}
```

### Key Operations
- `add_rustymon()` - Add to first empty slot, overflow to bank
- `remove_rustymon()` - Remove from team or bank
- `get_active_rustymon_id()` - Get currently battling Rustymon
- `switch_to_next()` - Cycle to next available team member
- `set_active_rustymon()` - Set specific Rustymon as active
- `move_from_bank_to_team()` / `move_from_team_to_bank()` - Reorder
- `is_in_team()` / `is_in_bank()` - Check location
- `team_count()` / `bank_count()` - Get counts

---

## 8. Rustymon Creation & Factory

**Location**: `/src/game/rustymon_factory.rs`

### Creation from Enemy
```rust
pub fn create_from_enemy(
    species_id: u32,
    name: String,
    base_level: u32,
    element: Element,
    str: u32, dex: u32, vit: u32, int: u32, luk: u32,
) -> Rustymon {
    // Creates at level 1 with given stats
    // UUID generated for instance ID
}
```

### Starter Rustymon
```rust
pub fn create_starter(level: u32) -> Rustymon {
    // Creates Poring at specified level
    // All base stats = 1
    // Then recalculated for level
}
```

---

## 9. UI Display Components

### Rustymon Detail Page
**Location**: `/src/ui/pages/rustymon_detail.rs`

Displays:
- **Header**: Element-colored background with name/level
- **Level**: Current level
- **Element**: Element name and color
- **HP**: Current/max with visual bar
- **EXP**: Current/next with progress bar
- **Base Stats**: STR, DEX, VIT, INT, LUK
- **Combat Stats**: ATK, DEF, HIT, FLEE, CRIT
- **Buttons**: "Add to Team" / "Remove from Team" / "Back"

### Touch Areas
- Back button: (10, 420) 100x30
- Add/Remove button: (120, 420) 140x30

### Color Scheme
```rust
background: Dark blue (15, 20, 30)
element_color: Varies by element
stat_labels: Light gray (180, 180, 200)
stat_values: White (255, 255, 255)
buttons: Green (40,80,40) for Add, Red (80,40,40) for Remove
```

---

## 10. Data Loading System

**Location**: `/src/game/data_loader.rs`

### GameData Structure
```rust
pub struct GameData {
    pub maps: HashMap<u32, MapData>,
    pub enemies: HashMap<u32, EnemyData>,
    pub items: HashMap<u32, ItemData>,
    pub recipes_by_city: HashMap<String, Vec<Recipe>>,
    pub upgrade_recipes: HashMap<String, Vec<UpgradeRecipe>>,
}
```

### Embedded JSON Loading
```rust
pub fn load_from_assets() -> Result<Self, Box<dyn Error>> {
    let maps_json = include_str!("../../assets/data/maps.json");
    let enemies_json = include_str!("../../assets/data/enemies.json");
    let items_json = include_str!("../../assets/data/items.json");
    let recipes_json = include_str!("../../assets/data/recipes.json");
    let upgrade_json = include_str!("../../assets/data/upgrade_recipes.json");
    
    // Parse and organize into HashMaps
}
```

### Available Methods
- `get_map(id)` - Retrieve map by ID
- `get_enemy(id)` - Retrieve enemy by ID
- `get_item(id)` - Retrieve item by ID
- `get_recipes_for_city(city)` - Get city recipes
- `get_upgrade_recipe(type, level)` - Get upgrade path
- `get_all_items()` - Full item list

---

## 11. Fragment System

**Location**: `/src/game/fragment_collection.rs`

### Fragment Collection
```rust
pub struct FragmentCollection {
    pub fragments: HashMap<u32, u32>,  // enemy_id -> count
}
```

### Battle Fragment Drops
**Location**: `/src/game/battle.rs` (lines 150-169)

```rust
pub fn check_fragment_drop(
    enemy_id: u32,
    enemy_name: &str,
    drop_rate: f32,
    fragment_collection: &mut FragmentCollection,
) -> FragmentDropResult {
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();
    
    if roll < drop_rate {
        fragment_collection.add_fragment(enemy_id, 1);
        return FragmentDropResult::Dropped(enemy_id, enemy_name.to_string());
    }
    
    FragmentDropResult::None
}
```

---

## 12. Save Data Structure

**Location**: `/src/game/save.rs`

```rust
pub struct SaveData {
    pub version: u32,
    pub hero: Hero,
    pub kill_tracker: KillTracker,
    pub current_location_id: u32,
    pub play_time_seconds: u64,
    pub save_timestamp: u64,
    pub rustymon_collection: Vec<Rustymon>,
    pub rustymon_team: RustymonTeam,
    pub fragment_collection: FragmentCollection,
}
```

---

## 13. Skill System Status

### Existing Skills JSON
**Location**: `/assets/data/skills.json`

Currently defines 16 skills with placeholder structure:
```json
{
  "id": 1,
  "name": "Bash",
  "sp_cost": 8,
  "skill_type": "Physical",
  "power": 150,
  "job_req": "Swordman",
  "description": "Powerful strike dealing 150% ATK damage"
}
```

**Skill Types**:
- Physical: Direct ATK-based damage
- Magic: INT-based damage
- Healing: Restore HP
- Buff: Stat increases
- Debuff: Stat decreases
- Utility: Special effects

**Status**: Not yet integrated into Rustymon system
- No skill learning for Rustymon
- No skill selection in battle UI
- No SP (Skill Points) system for Rustymon

---

## 14. Key Game Manager Integration

**Location**: `/src/ecs/resources.rs`

### GameManager Structure
Contains:
- `hero: Hero` - Current hero
- `rustymon_collection: Vec<Rustymon>`
- `rustymon_team: RustymonTeam`
- `fragment_collection: FragmentCollection`
- `battle_page: Option<BattlePage>`
- `rustymon_detail_page: Option<RustymonDetailPage>`
- Various other pages and state

### Key Methods
- `sync_battle_state()` - Sync battle progress to save data
- `get_active_rustymon()` - Get currently battling Rustymon
- `switch_rustymon()` - Switch active team member
- `update_rustymon_exp()` - Award battle EXP

---

## 15. Architecture Diagram

```
Game Flow:
Map Page → Battle → Battle Page (Rustymon vs Enemy)
         ↓
    Team Management → Rustymon List → Rustymon Detail
         ↓
    Fragment Collection → Summoning → New Rustymon

Data Flow:
GameData (JSON) → DataLoader → GameManager → Battle System
                                    ↓
                            Rustymon Collection
                            + Team + Fragments
                                    ↓
                            SaveData (JSON serialized)
```

---

## 16. Performance Considerations (ESP32-S3)

- **Memory**: 512KB SRAM - Each Rustymon ~120 bytes
- **Max Collection**: ~100 Rustymon (~12KB)
- **Display**: 390x450 AMOLED @ 60 FPS target
- **Sprite Cache**: Embedded in binary with include_bytes!
- **Save Size**: ~200 bytes per Rustymon, 100+ = 20KB+
- **Touch Screen**: 240x240 active area, 50ms debounce

---

## 17. File Structure Summary

```
/src/
├── game/
│   ├── battle.rs              (Damage calculation, attack functions)
│   ├── rustymon.rs            (Rustymon structure, leveling)
│   ├── rustymon_team.rs       (Team management)
│   ├── rustymon_factory.rs    (Rustymon creation)
│   ├── enemy.rs               (Enemy structure)
│   ├── element_system.rs      (Element advantages, colors)
│   ├── stats.rs               (Hero stats formulas)
│   ├── hero.rs                (Hero structure, jobs)
│   ├── data_loader.rs         (JSON loading)
│   ├── fragment_collection.rs (Fragment tracking)
│   ├── save.rs                (Save serialization)
│   ├── item.rs                (Item structure)
│   ├── inventory.rs           (Inventory management)
│   ├── equipment.rs           (Equipment system)
│   ├── kill_tracker.rs        (Enemy kill tracking)
│   └── mod.rs                 (Module exports)
│
├── ui/
│   ├── pages/
│   │   ├── battle.rs          (Battle UI, animations, damage numbers)
│   │   ├── rustymon_detail.rs (Rustymon stats display)
│   │   ├── rustymon_list.rs   (Team/collection list)
│   │   ├── fragment_collection_page.rs (Fragments tracking)
│   │   ├── rustymon_summon.rs (Summoning preview)
│   │   ├── map.rs             (Map navigation)
│   │   ├── menu.rs            (Main menu)
│   │   ├── equipment.rs       (Equipment UI)
│   │   ├── inventory.rs       (Inventory UI)
│   │   ├── hero_overview.rs   (Hero stats)
│   │   ├── stats_allocation.rs (Stat allocation)
│   │   ├── crafting.rs        (Crafting UI)
│   │   └── mod.rs             (Page registry)
│   ├── page.rs                (Page trait)
│   └── sprite.rs              (Sprite animation)
│
├── systems/
│   ├── battle.rs              (Battle input handling)
│   ├── battle_loading.rs      (Battle setup)
│   ├── map_navigation.rs      (Map movement)
│   ├── rustymon_navigation.rs (Rustymon selection)
│   ├── menu.rs                (Menu navigation)
│   ├── autosave.rs            (Save management)
│   ├── equipment.rs           (Equipment system)
│   ├── crafting.rs            (Crafting system)
│   ├── inventory.rs           (Inventory system)
│   ├── stats_allocation.rs    (Stat system)
│   ├── input.rs               (Input processing)
│   ├── render.rs              (Display rendering)
│   ├── fps.rs                 (FPS tracking)
│   ├── animation.rs           (General animation)
│   ├── death.rs               (Death handling)
│   └── mod.rs                 (System registry)
│
├── ecs/
│   ├── resources.rs           (GameManager, AppState, etc.)
│   └── mod.rs                 (ECS setup)
│
├── display/                   (Display drivers)
├── drivers/                   (Hardware drivers)
└── main.rs                    (Entry point, system setup)

/assets/data/
├── enemies.json               (Enemy definitions)
├── items.json                 (Item definitions)
├── maps.json                  (Map definitions)
├── recipes.json               (Crafting recipes)
├── upgrade_recipes.json       (Equipment upgrades)
├── skills.json                (Skill definitions - not yet used)
├── equipment.json             (Equipment items)
├── jobs.json                  (Job definitions)
├── achievements.json          (Achievement tracking)
├── quests.json                (Quest data)
├── cards.json                 (Card collection?)
├── crafting_npcs.json         (Crafter locations)
└── materials.json             (Material definitions)
```

---

## Recommendations for Skill System Implementation

Based on analysis, here are key integration points for a Rustymon skill system:

1. **Data Structure**: Each Rustymon should have a `skills: Vec<RustymonSkill>` field
2. **Skill Learning**: Via level-up or crafting, not hero jobs
3. **Battle Integration**: Modify `battle.rs` damage calculation to apply skill multipliers
4. **UI Display**: Add skill list to `rustymon_detail.rs`
5. **Battle UI**: Add skill selection during turn-based battles
6. **SP System**: Implement SP tracking in Rustymon struct
7. **Balance**: Skill power should consider element advantages

