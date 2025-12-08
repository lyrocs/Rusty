# Ragnarok Monster Tamer - Implementation Plan
## Clean Architecture Migration from Rustymon to GDD System

**Document Version**: 1.0
**Last Updated**: 2025-12-06
**Based on**: GDD_Ragnarok_Tamagotchi.md

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Principles](#architecture-principles)
3. [Current System Analysis](#current-system-analysis)
4. [Target Architecture](#target-architecture)
5. [Data Structure Design](#data-structure-design)
6. [JSON Data Files](#json-data-files)
7. [Implementation Phases](#implementation-phases)
8. [Detailed Steps](#detailed-steps)

---

## Overview

### Mission
Transform the current Rustymon-based system into a clean, professional Monster Tamer game following the Ragnarok Online GDD specifications.

### Core Objectives
- **Remove ALL unused systems**: Rustymon battles, 3v3 modes, skills, fragments, job systems
- **Implement GDD systems**: Monsters, Expeditions, Dungeons, Real-time Combat
- **Clean Architecture**: DRY, KISS, reusable functions, dedicated modules
- **JSON-driven**: Zero hardcoded data, all game data in .json files
- **Separation of Concerns**: Game logic isolated in dedicated calculation modules

---

## Architecture Principles

### 1. **DRY (Don't Repeat Yourself)**
- Single source of truth for all calculations
- Reusable utility functions for common operations
- Shared data structures across systems

### 2. **KISS (Keep It Simple, Stupid)**
- Simple, focused modules
- Clear function signatures
- Minimal abstraction layers

### 3. **Separation of Concerns**
```
src/
├── game/
│   ├── core/           # Core game structures (Monster, Player, etc.)
│   ├── calculations/   # All game math (damage, XP, stats, elements)
│   ├── systems/        # Game systems (expedition, combat, progression)
│   ├── data/           # Data loading and management
│   └── save/           # Save/load functionality
├── ui/                 # UI rendering and pages
├── ecs/                # ECS resources and systems
└── assets/data/        # JSON data files
```

### 4. **JSON Data Files**
All game configuration, content, and balancing in JSON:
- Monster species data
- Zone and map definitions
- Element relationships
- Skill definitions
- Dungeon configurations
- Expedition rewards tables

---

## Current System Analysis

### Files to REMOVE (Rustymon System)
```
src/game/
├── rustymon.rs                     ❌ Remove - Monster system redesign
├── rustymon_team.rs                ❌ Remove - New team structure
├── rustymon_factory.rs             ❌ Remove - New monster creation
├── skill.rs                        ❌ Remove - New skill system (1 per species)
├── fragment_collection.rs          ❌ Remove - Not in GDD
└── battle.rs                       ⚠️  Refactor - Real-time combat needed

src/ui/pages/
├── battle_3v3.rs                   ❌ Remove - Not in GDD
├── rustymon_list.rs                ⚠️  Refactor - Becomes monster_list.rs
├── rustymon_detail.rs              ⚠️  Refactor - Becomes monster_detail.rs
├── rustymon_skills.rs              ⚠️  Refactor - Simplified skill view
├── fragment_collection_page.rs     ❌ Remove - Not in GDD
├── rustymon_summon.rs              ❌ Remove - Different capture system
└── afk_farm.rs                     ⚠️  Refactor - Becomes expedition system

src/systems/
├── battle_3v3_loading.rs           ❌ Remove - Not in GDD
├── rustymon_navigation.rs          ⚠️  Refactor - Monster navigation
└── afk.rs                          ⚠️  Refactor - Expedition system

assets/data/
├── jobs.json                       ❌ Remove - No job system in GDD
```

### Files to KEEP and REFACTOR
```
src/game/
├── mod.rs                          ✅ Update exports
├── element_system.rs               ⚠️  Extend for reactions
├── enemy.rs                        ⚠️  Becomes monster base
├── map.rs                          ⚠️  Extend for expeditions
├── save.rs                         ⚠️  Update save structure
├── data_loader.rs                  ⚠️  Update for new JSON files
├── kill_tracker.rs                 ✅ Keep (useful for quests)
└── quest.rs                        ✅ Keep (quest system valid)

src/ui/
├── mod.rs                          ⚠️  Update page imports
└── pages/                          ⚠️  Refactor existing pages
```

---

## Target Architecture

### Module Structure

```
src/game/
├── mod.rs                          # Module exports
│
├── core/                           # Core data structures
│   ├── mod.rs
│   ├── monster.rs                  # Monster structure
│   ├── species.rs                  # Species data (from JSON)
│   ├── skill.rs                    # Skill structure
│   ├── player.rs                   # Player inventory/resources
│   ├── team.rs                     # Monster team (3 max)
│   └── element.rs                  # Element enum
│
├── calculations/                   # Pure calculation functions
│   ├── mod.rs
│   ├── stats.rs                    # Stat calculations (base, fusion, level)
│   ├── damage.rs                   # Damage formulas
│   ├── elements.rs                 # Element advantages/reactions
│   ├── xp.rs                       # XP and leveling
│   └── combat.rs                   # Combat timing, bars, etc.
│
├── systems/                        # Game systems
│   ├── mod.rs
│   ├── expedition/
│   │   ├── mod.rs
│   │   ├── expedition.rs           # Expedition state
│   │   ├── rewards.rs              # Reward calculation
│   │   └── capture.rs              # Monster capture logic
│   ├── dungeon/
│   │   ├── mod.rs
│   │   ├── dungeon.rs              # Dungeon state
│   │   ├── floor_gen.rs            # Floor enemy generation
│   │   └── checkpoints.rs          # Checkpoint system
│   ├── combat/
│   │   ├── mod.rs
│   │   ├── combat_state.rs         # Real-time combat state
│   │   ├── reactions.rs            # Elemental reactions
│   │   ├── auras.rs                # Aura system
│   │   └── swap.rs                 # Monster swapping
│   └── progression/
│       ├── mod.rs
│       ├── leveling.rs             # Level up logic
│       ├── fusion.rs               # Duplicate fusion
│       └── upgrade.rs              # Stat upgrades (crystals)
│
├── data/                           # Data management
│   ├── mod.rs
│   ├── loader.rs                   # JSON data loading
│   ├── game_data.rs                # Central game data store
│   └── validation.rs               # Data validation
│
└── save/                           # Save/load
    ├── mod.rs
    ├── save_data.rs                # Save data structure
    └── migration.rs                # Save version migration
```

### Calculation Module Design (Example: damage.rs)

```rust
// src/game/calculations/damage.rs
// ALL damage calculations in one place

use crate::game::core::Element;

/// Calculate base damage
pub fn calculate_base_damage(atk: u16, def: u16) -> f32 {
    let base = atk as f32 - (def as f32 * 0.5);
    let min = (atk as f32) * 0.1;
    base.max(min)
}

/// Get element multiplier
pub fn get_element_multiplier(attacker: Element, defender: Element) -> f32 {
    // Load from JSON config or use hardcoded table
    // This is the ONLY place element advantages are calculated
    match (attacker, defender) {
        (Element::Fire, Element::Earth) => 1.5,
        (Element::Fire, Element::Wind) => 1.5,
        (Element::Fire, Element::Water) => 0.5,
        // ... etc
        _ => 1.0,
    }
}

/// Calculate final damage with all modifiers
pub fn calculate_final_damage(
    atk: u16,
    def: u16,
    attacker_element: Element,
    defender_element: Element,
    reaction_multiplier: f32
) -> u16 {
    let base = calculate_base_damage(atk, def);
    let element_mult = get_element_multiplier(attacker_element, defender_element);
    let final_damage = base * element_mult * reaction_multiplier;
    final_damage.round() as u16
}
```

---

## Data Structure Design

### Core Structures

#### Monster
```rust
// src/game/core/monster.rs
use serde::{Deserialize, Serialize};
use crate::game::core::{Element, Skill};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    pub id: String,              // Unique instance ID (UUID)
    pub species_id: String,      // Species type (e.g., "poring", "wolf")
    pub name: String,            // Display name
    pub level: u8,               // 1-99
    pub xp: u32,                 // Current XP
    pub xp_to_next: u32,         // XP needed for next level
    pub element: Element,        // Monster element
    pub fusion_count: u8,        // 0-9 (+5% stats per fusion)

    // Stats (affected by level, base stats, fusion)
    pub hp_current: u16,
    pub hp_max: u16,
    pub atk: u16,
    pub def: u16,
    pub spd: u16,

    // Skill (one per species)
    pub skill: Skill,

    // State
    pub status: MonsterStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MonsterStatus {
    Available,
    InExpedition,
    InDungeon,
}
```

#### Species (JSON-loaded)
```rust
// src/game/core/species.rs
use serde::{Deserialize, Serialize};
use crate::game::core::Element;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: String,              // e.g., "poring"
    pub name: String,            // Display name
    pub element: Element,

    // Base stats (at level 1)
    pub base_hp: u16,
    pub base_atk: u16,
    pub base_def: u16,
    pub base_spd: u16,

    // Skill
    pub skill_id: String,        // References skills.json

    // Capture info
    pub zones: Vec<String>,      // Zones where this appears
}
```

#### Expedition
```rust
// src/game/systems/expedition/expedition.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expedition {
    pub id: String,
    pub map_id: String,
    pub monster_ids: Vec<String>,    // 1-3 monsters
    pub duration_minutes: u32,       // 20, 60, 240, 480
    pub started_at: u64,             // Unix timestamp
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct ExpeditionResult {
    pub xp_per_monster: u32,
    pub crystals: u16,
    pub essences: Vec<(Element, u8)>,
    pub captured_monster: Option<Species>,
}
```

#### Combat State
```rust
// src/game/systems/combat/combat_state.rs
use crate::game::core::{Monster, Element};

pub struct CombatState {
    // Player team
    pub player_team: [Monster; 3],
    pub active_index: u8,            // 0-2
    pub swap_cooldowns: [f32; 3],    // Seconds remaining

    // Enemy
    pub enemy: Monster,
    pub enemy_aura: Option<(Element, f32)>, // Element + time remaining

    // Combat jauges (0.0 to 1.0)
    pub player_atk_bar: f32,
    pub player_skl_bar: f32,
    pub enemy_atk_bar: f32,
    pub enemy_skl_bar: f32,

    // Run info
    pub current_floor: u16,
    pub crystals_earned: u16,
    pub xp_earned: u32,
}
```

### Player Resources
```rust
// src/game/core/player.rs
use std::collections::HashMap;
use crate::game::core::Element;

pub struct Player {
    pub crystals: u32,
    pub essences: HashMap<Element, u16>,
    pub monsters: Vec<Monster>,           // Max 6
    pub active_team: [Option<String>; 3], // Monster IDs
}
```

---

## JSON Data Files

### 1. species.json
```json
{
  "species": [
    {
      "id": "poring",
      "name": "Poring",
      "element": "Water",
      "base_hp": 80,
      "base_atk": 15,
      "base_def": 10,
      "base_spd": 20,
      "skill_id": "heal",
      "zones": ["prontera"]
    },
    {
      "id": "wolf",
      "name": "Wolf",
      "element": "Earth",
      "base_hp": 100,
      "base_atk": 35,
      "base_def": 20,
      "base_spd": 30,
      "skill_id": "fang",
      "zones": ["payon"]
    }
  ]
}
```

### 2. skills.json
```json
{
  "skills": [
    {
      "id": "heal",
      "name": "Heal",
      "element": "Holy",
      "description": "Soigne le monstre actif",
      "effect_type": "heal",
      "effect_value": 0.3
    },
    {
      "id": "meteor",
      "name": "Meteor",
      "element": "Fire",
      "description": "Gros dégâts + applique Burn (DoT)",
      "effect_type": "damage_dot",
      "effect_value": 2.0,
      "dot_duration": 5
    }
  ]
}
```

### 3. zones.json
```json
{
  "zones": [
    {
      "id": "prontera",
      "name": "Prontera",
      "maps": ["plains_south", "forest_west", "sewers", "hills_north"],
      "dungeon_id": "culvert",
      "unlock_condition": null
    },
    {
      "id": "payon",
      "name": "Payon",
      "maps": ["forest", "cave", "temple", "summit"],
      "dungeon_id": "payon_cave",
      "unlock_condition": {
        "type": "dungeon_floor",
        "dungeon_id": "culvert",
        "floor": 20
      }
    }
  ]
}
```

### 4. maps.json
```json
{
  "maps": [
    {
      "id": "plains_south",
      "name": "Plaine Sud",
      "zone_id": "prontera",
      "level_range": [1, 5],
      "required_elements": ["Fire"],
      "capturable_species": ["poring", "lunatic", "fabre"],
      "base_rewards": {
        "crystals": 15,
        "essences": [
          { "element": "Earth", "amount": 3 }
        ]
      }
    },
    {
      "id": "forest_west",
      "name": "Forêt Ouest",
      "zone_id": "prontera",
      "level_range": [5, 10],
      "required_elements": ["Water", "Earth"],
      "capturable_species": ["lunatic", "fabre", "pupa"],
      "base_rewards": {
        "crystals": 15,
        "essences": [
          { "element": "Earth", "amount": 3 },
          { "element": "Water", "amount": 2 }
        ]
      }
    }
  ]
}
```

### 5. dungeons.json
```json
{
  "dungeons": [
    {
      "id": "culvert",
      "name": "Culvert",
      "zone_id": "prontera",
      "checkpoints": [5, 10, 15, 20, 25, 30, 35, 40, 45, 50],
      "dominant_elements": ["Water", "Shadow"],
      "enemy_pool": [
        {
          "floor_min": 1,
          "floor_max": 10,
          "species": ["thief_bug", "familiar", "poring"],
          "count_per_floor": 1
        },
        {
          "floor_min": 11,
          "floor_max": 20,
          "species": ["thief_bug", "familiar"],
          "count_per_floor": 2
        }
      ],
      "boss_floors": [10, 20, 30, 40, 50],
      "boss_species": "golden_thief_bug"
    }
  ]
}
```

### 6. element_config.json
```json
{
  "advantages": {
    "Fire": {
      "strong_against": ["Earth", "Wind"],
      "weak_against": ["Water"],
      "multiplier_strong": 1.5,
      "multiplier_weak": 0.5
    },
    "Water": {
      "strong_against": ["Fire"],
      "weak_against": ["Earth", "Wind"],
      "multiplier_strong": 1.5,
      "multiplier_weak": 0.5
    }
  },
  "reactions": [
    {
      "aura": "Water",
      "trigger": "Fire",
      "name": "VAPORIZE",
      "effect": "damage_multiplier",
      "value": 2.0
    },
    {
      "aura": "Water",
      "trigger": "Thunder",
      "name": "ELECTROCUTE",
      "effect": "damage_stun",
      "value": 1.5,
      "stun_duration": 1.0
    }
  ]
}
```

### 7. expedition_rewards.json
```json
{
  "durations": {
    "20": {
      "minutes": 20,
      "rating": 3,
      "xp_base": 50,
      "crystals_base": 15,
      "capture_chance": 0.15
    },
    "60": {
      "minutes": 60,
      "rating": 2,
      "xp_base": 120,
      "crystals_base": 35,
      "capture_chance": 0.25
    },
    "240": {
      "minutes": 240,
      "rating": 1,
      "xp_base": 350,
      "crystals_base": 90,
      "capture_chance": 0.40
    },
    "480": {
      "minutes": 480,
      "rating": 1,
      "xp_base": 600,
      "crystals_base": 150,
      "capture_chance": 0.50
    }
  }
}
```

---

## Implementation Phases

### Phase 1: Foundation & Data (1-2 weeks)
**Goal**: Clean slate, new core structures, JSON data files

### Phase 2: Monster System (1 week)
**Goal**: Monster creation, stats, leveling, fusion

### Phase 3: Expedition System (1 week)
**Goal**: Passive exploration with rewards and captures

### Phase 4: Combat Foundation (2 weeks)
**Goal**: Real-time combat, bars, auto-attacks

### Phase 5: Combat Advanced (2 weeks)
**Goal**: Swapping, skills, elemental auras, reactions

### Phase 6: Dungeon System (1 week)
**Goal**: Infinite floors, checkpoints, progression

### Phase 7: Progression & Polish (2 weeks)
**Goal**: Upgrades, zone unlocking, UI polish, save/load

---

## Detailed Steps

### PHASE 1: Foundation & Data

#### Step 1.1: Create New Module Structure
**Time**: 2 hours
**Files**: Module organization

**Tasks**:
1. Create new directory structure:
```bash
mkdir -p src/game/core
mkdir -p src/game/calculations
mkdir -p src/game/systems/expedition
mkdir -p src/game/systems/dungeon
mkdir -p src/game/systems/combat
mkdir -p src/game/systems/progression
mkdir -p src/game/data
mkdir -p src/game/save
```

2. Create mod.rs files for each module with proper exports
3. Update src/game/mod.rs to export new modules

**Validation**:
- `cargo check` passes
- All new modules are accessible from main

---

#### Step 1.2: Define Element System
**Time**: 1 hour
**Files**:
- `src/game/core/element.rs`
- `assets/data/element_config.json`

**Tasks**:
1. Create Element enum (Fire, Water, Earth, Wind, Thunder, Shadow, Holy, Ghost)
2. Implement element advantage calculation from JSON config
3. Create element_config.json with advantages table
4. Add unit tests for element calculations

**Code Example**:
```rust
// src/game/core/element.rs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Element {
    Fire,
    Water,
    Earth,
    Wind,
    Thunder,
    Shadow,
    Holy,
    Ghost,
}
```

**Validation**:
- Unit tests pass
- Element advantages calculated correctly from JSON

---

#### Step 1.3: Create Species Data Structure
**Time**: 2 hours
**Files**:
- `src/game/core/species.rs`
- `assets/data/species.json`

**Tasks**:
1. Define Species struct
2. Create species.json with all monsters from GDD section 4.1
3. Implement Species loader in data module
4. Add validation for required fields

**Validation**:
- All 17+ species from GDD loaded successfully
- Validation catches missing fields

---

#### Step 1.4: Create Skill Data Structure
**Time**: 2 hours
**Files**:
- `src/game/core/skill.rs`
- `assets/data/skills.json`

**Tasks**:
1. Define Skill struct
2. Create skills.json with all skills from GDD
3. Implement Skill loader
4. Link skills to species

**Validation**:
- All skills from GDD table 2.1.6 loaded
- Skills correctly associated with species

---

#### Step 1.5: Create Monster Core Structure
**Time**: 3 hours
**Files**: `src/game/core/monster.rs`

**Tasks**:
1. Define Monster struct with all fields from GDD
2. Implement Monster creation from Species
3. Add methods: check_level_up, heal, take_damage, etc.
4. Add serialization support

**Validation**:
- Monster instances can be created from species
- Stats calculated correctly with fusion bonus

---

#### Step 1.6: Create Calculation Modules
**Time**: 4 hours
**Files**:
- `src/game/calculations/stats.rs`
- `src/game/calculations/xp.rs`
- `src/game/calculations/damage.rs`

**Tasks**:

1. **stats.rs**: Implement stat calculations
   - Base stats + level scaling
   - Fusion bonus: `stat_final = stat_base * (1 + fusion_count * 0.05)`
   - Power rating: `power = ATK + DEF + SPD + (HP / 5)`

2. **xp.rs**: Implement XP system
   - Formula: `xp_to_next = level * 100`
   - Level up handling
   - XP reward calculation

3. **damage.rs**: Implement damage formulas
   - Base damage: `max(ATK - DEF*0.5, ATK*0.1)`
   - Element multipliers
   - Reaction bonuses

**Code Example**:
```rust
// src/game/calculations/xp.rs

/// Calculate XP needed for next level
pub fn xp_for_next_level(current_level: u8) -> u32 {
    current_level as u32 * 100
}

/// Check if monster should level up and return new level
pub fn check_level_up(current_level: u8, current_xp: u32, xp_to_next: u32) -> Option<u8> {
    if current_xp >= xp_to_next && current_level < 99 {
        Some(current_level + 1)
    } else {
        None
    }
}
```

**Validation**:
- Unit tests for all formulas match GDD specs
- Edge cases handled (max level, min damage, etc.)

---

#### Step 1.7: Remove Old Rustymon Files
**Time**: 2 hours
**Files**: Delete/archive old system

**Tasks**:
1. Create backup branch: `git checkout -b backup/old-rustymon-system`
2. Delete files:
   ```bash
   rm src/game/rustymon.rs
   rm src/game/rustymon_team.rs
   rm src/game/rustymon_factory.rs
   rm src/game/fragment_collection.rs
   rm src/ui/pages/rustymon_summon.rs
   rm src/ui/pages/fragment_collection_page.rs
   rm src/ui/pages/battle_3v3.rs
   rm src/systems/battle_3v3_loading.rs
   rm assets/data/jobs.json
   ```
3. Update imports and remove references
4. Fix compilation errors

**Validation**:
- `cargo check` passes after deletions
- No dead code warnings

---

### PHASE 2: Monster System

#### Step 2.1: Implement Team Management
**Time**: 3 hours
**Files**: `src/game/core/team.rs`

**Tasks**:
1. Create Team struct (3 monster slots max)
2. Implement add/remove monster
3. Implement swap active monster
4. Track monster status (Available/InExpedition/InDungeon)

**Validation**:
- Cannot add more than 3 monsters
- Status updates correctly

---

#### Step 2.2: Implement Fusion System
**Time**: 2 hours
**Files**: `src/game/systems/progression/fusion.rs`

**Tasks**:
1. Implement duplicate detection (same species_id)
2. Calculate fusion bonus (+5% stats per fusion, max +9)
3. Update monster stats after fusion

**Formula**:
```rust
pub fn apply_fusion_bonus(base_stat: u16, fusion_count: u8) -> u16 {
    let multiplier = 1.0 + (fusion_count as f32 * 0.05);
    (base_stat as f32 * multiplier).round() as u16
}
```

**Validation**:
- Stats increase by 5% per fusion
- Max fusion is +9 (45% bonus)

---

#### Step 2.3: Implement Leveling System
**Time**: 3 hours
**Files**: `src/game/systems/progression/leveling.rs`

**Tasks**:
1. Implement XP gain
2. Handle level up
3. Calculate stat increases on level up
4. Update monster stats

**Validation**:
- XP formula matches GDD (level * 100)
- Stats increase properly on level up

---

#### Step 2.4: Create Monster List UI
**Time**: 4 hours
**Files**: `src/ui/pages/monster_list.rs`

**Tasks**:
1. Refactor rustymon_list.rs → monster_list.rs
2. Display monsters with status (Available/Expedition/Dungeon)
3. Show fusion count (+0 to +9)
4. Show power rating

**Validation**:
- UI displays all monsters
- Status icons show correctly
- Tap opens detail view

---

#### Step 2.5: Create Monster Detail UI
**Time**: 4 hours
**Files**: `src/ui/pages/monster_detail.rs`

**Tasks**:
1. Refactor rustymon_detail.rs → monster_detail.rs
2. Show stats with fusion bonus
3. Show single skill (not a list)
4. Show XP progress bar
5. Add "Améliorer" button

**Validation**:
- All monster info displays correctly
- XP bar shows progress accurately

---

### PHASE 3: Expedition System

#### Step 3.1: Create Zone & Map Data
**Time**: 3 hours
**Files**:
- `assets/data/zones.json`
- `assets/data/maps.json`

**Tasks**:
1. Create zones.json with all zones from GDD section 4.2
2. Create maps.json with all maps from GDD
3. Implement Zone and Map loaders
4. Add unlock conditions

**Validation**:
- All 5 zones loaded
- All maps loaded with correct requirements

---

#### Step 3.2: Implement Expedition Core
**Time**: 4 hours
**Files**: `src/game/systems/expedition/expedition.rs`

**Tasks**:
1. Create Expedition struct
2. Implement expedition start (validate elements)
3. Implement expedition completion check (timestamp)
4. Handle 2 concurrent expedition slots

**Validation**:
- Element requirements validated
- Timer works correctly
- Max 2 expeditions

---

#### Step 3.3: Implement Expedition Rewards
**Time**: 3 hours
**Files**:
- `src/game/systems/expedition/rewards.rs`
- `assets/data/expedition_rewards.json`

**Tasks**:
1. Create expedition_rewards.json with GDD table 2.2.3
2. Implement reward calculation based on duration
3. Implement essence drops based on map
4. Add XP distribution to team

**Validation**:
- Rewards match GDD tables
- XP distributed to all monsters in expedition

---

#### Step 3.4: Implement Capture System
**Time**: 3 hours
**Files**: `src/game/systems/expedition/capture.rs`

**Tasks**:
1. Implement capture chance based on duration
2. Roll for capture on expedition complete
3. Handle duplicate (fusion)
4. Handle max 6 monsters limit

**Capture Chances** (from GDD):
- 20 min: 15%
- 1 hour: 25%
- 4 hours: 40%
- 8 hours: 50%

**Validation**:
- Capture rates match GDD
- Duplicates fuse correctly
- Cannot exceed 6 monsters

---

#### Step 3.5: Create Expedition UI
**Time**: 6 hours
**Files**:
- `src/ui/pages/expedition_map.rs`
- `src/ui/pages/expedition_team_select.rs`
- `src/ui/pages/expedition_result.rs`

**Tasks**:
1. Create map selection UI (GDD 3.3.4)
2. Create team selection UI (GDD 3.3.5)
   - Show element requirements
   - Lock monsters of wrong element until requirements met
3. Create expedition result UI (GDD 3.3.7)
4. Add timer display for active expeditions

**Validation**:
- UI matches GDD mockups
- Element requirements enforced
- Results show all rewards

---

### PHASE 4: Combat Foundation

#### Step 4.1: Create Combat State
**Time**: 4 hours
**Files**: `src/game/systems/combat/combat_state.rs`

**Tasks**:
1. Create CombatState struct
2. Implement ATK bar filling based on SPD
3. Implement SKL bar filling (+20% per attack)
4. Add HP tracking

**ATK Bar Formula**:
```rust
// Fill rate per second = SPD / 30.0
// SPD 30 = 1 attack/second
// SPD 60 = 2 attacks/second
pub fn update_atk_bar(current: f32, spd: u16, delta_time: f32) -> f32 {
    let fill_rate = spd as f32 / 30.0;
    (current + fill_rate * delta_time).min(1.0)
}
```

**Validation**:
- ATK bar timing matches GDD specs
- SKL bar fills correctly

---

#### Step 4.2: Implement Auto-Attack System
**Time**: 3 hours
**Files**: `src/game/systems/combat/combat_state.rs`

**Tasks**:
1. When ATK bar reaches 100%, trigger attack
2. Calculate damage using damage.rs
3. Apply damage to target
4. Reset ATK bar to 0
5. Increase SKL bar by 20%

**Validation**:
- Auto-attacks happen at correct intervals
- Damage calculations correct
- SKL bar increases

---

#### Step 4.3: Create Basic Combat UI
**Time**: 6 hours
**Files**: `src/ui/pages/combat.rs`

**Tasks**:
1. Display enemy (sprite, element, HP bar, SKL bar)
2. Display player monster (sprite, HP bar, SKL bar, ATK bar)
3. Show damage numbers as feedback
4. Add 3 buttons (SWAP 1, SWAP 2, SKILL) - greyed out for now

**UI Layout** (GDD 3.3.9):
```
╔═══════════════════════════════╗
║  CULVERT Ét.12      +45 💎    ║
╠═══════════════════════════════╣
║                               ║
║  🐸 Thief Bug 💧              ║
║  HP ████████░░  SKL ██░░      ║
║                               ║
║  🔥 Flame                     ║
║  HP ██████░░░░  SKL ████      ║
║                               ║
╠═══════════════════════════════╣
║   [🌿]    [💧]    [🔥 SKILL]  ║
║   --      --      --          ║
╚═══════════════════════════════╝
```

**Validation**:
- Real-time bar updates smooth
- Damage feedback visible
- UI matches GDD mockup

---

### PHASE 5: Combat Advanced

#### Step 5.1: Implement Monster Swapping
**Time**: 4 hours
**Files**: `src/game/systems/combat/swap.rs`

**Tasks**:
1. Implement swap functionality
2. Add 3-second cooldown
3. Preserve jauges on swap out
4. Update combat UI with swap buttons

**Validation**:
- Swap works correctly
- Cooldown enforced
- Jauges preserved

---

#### Step 5.2: Implement Skills
**Time**: 4 hours
**Files**: `src/game/systems/combat/combat_state.rs`

**Tasks**:
1. When SKILL button tapped and SKL bar = 100%:
   - Execute skill effect
   - Reset SKL bar to 0
2. Implement skill effects (damage, heal, DoT)

**Validation**:
- Skills can only be used at 100% SKL
- Skill effects work correctly

---

#### Step 5.3: Implement Aura System
**Time**: 3 hours
**Files**: `src/game/systems/combat/auras.rs`

**Tasks**:
1. When monster attacks, apply its element as aura on target
2. Aura duration: 2 seconds (auto-attack), 4 seconds (skill)
3. Display aura icon on affected target
4. Aura expires after duration

**Validation**:
- Auras apply correctly
- Timer counts down accurately

---

#### Step 5.4: Implement Elemental Reactions
**Time**: 6 hours
**Files**:
- `src/game/systems/combat/reactions.rs`
- `assets/data/element_config.json`

**Tasks**:
1. Load reactions from element_config.json (GDD Appendix A)
2. When attack hits target with different element aura:
   - Trigger reaction
   - Apply reaction effect
   - Remove aura
3. Display reaction name and effect

**Reactions to Implement**:
- VAPORIZE (x2 damage)
- ELECTROCUTE (damage + stun 1s)
- BLOOM (heal team 15%)
- SWIRL (propagate aura)
- MELT (x1.5 damage)
- BURNING (DoT 5s)
- SUPERCONDUCT (DEF -30% for 5s)

**Validation**:
- All reactions from GDD work correctly
- Visual feedback shows reaction name

---

### PHASE 6: Dungeon System

#### Step 6.1: Create Dungeon Data
**Time**: 3 hours
**Files**: `assets/data/dungeons.json`

**Tasks**:
1. Create dungeons.json with all dungeons from GDD
2. Define enemy pools per floor range
3. Define checkpoints every 5 floors
4. Define boss floors (every 10 floors)

**Validation**:
- All 5 dungeons from GDD loaded
- Enemy pools correct per floor

---

#### Step 6.2: Implement Dungeon Floor Generation
**Time**: 4 hours
**Files**: `src/game/systems/dungeon/floor_gen.rs`

**Tasks**:
1. Generate enemy based on current floor
2. Select from appropriate enemy pool
3. Scale enemy stats based on floor
4. Generate boss on boss floors

**Validation**:
- Enemies scale correctly
- Boss appears on correct floors

---

#### Step 6.3: Implement Checkpoint System
**Time**: 3 hours
**Files**: `src/game/systems/dungeon/checkpoints.rs`

**Tasks**:
1. Track highest floor reached
2. Save checkpoints every 5 floors
3. Allow starting from any reached checkpoint
4. Apply reward multipliers based on starting floor

**Reward Multipliers** (GDD 2.3.3):
- Floor 1: x1.0
- Floor 10: x1.5
- Floor 20: x2.0
- Floor 30+: x2.5

**Validation**:
- Checkpoints save correctly
- Reward multipliers apply

---

#### Step 6.4: Implement Dungeon Run Flow
**Time**: 5 hours
**Files**:
- `src/game/systems/dungeon/dungeon.rs`
- `src/ui/pages/dungeon_between_floors.rs`

**Tasks**:
1. Start dungeon run from selected checkpoint
2. After each combat:
   - Show between-floors screen (GDD 3.3.10)
   - Display rewards earned
   - Show team HP
   - Preview next floor
   - Offer CONTINUE or ABANDON
3. On death or abandon:
   - Keep all rewards
   - Update checkpoint if new record

**Validation**:
- Full dungeon run works end-to-end
- Rewards accumulate correctly
- HP persists between floors

---

#### Step 6.5: Implement Zone Unlocking
**Time**: 2 hours
**Files**: `src/game/systems/progression/zones.rs`

**Tasks**:
1. Track dungeon records (highest floor per dungeon)
2. Check unlock conditions for zones
3. Unlock zones when conditions met

**Unlock Conditions** (GDD 2.6.2):
- Payon: Culvert Floor 20
- Geffen: Payon Cave Floor 15
- Morroc: Geffen Tower Floor 15
- Aldebaran: Pyramides Floor 15

**Validation**:
- Zones unlock at correct milestones
- Unlocked zones persist in save data

---

### PHASE 7: Progression & Polish

#### Step 7.1: Implement Stat Upgrades
**Time**: 4 hours
**Files**:
- `src/game/systems/progression/upgrade.rs`
- `src/ui/pages/monster_upgrade.rs`

**Tasks**:
1. Implement crystal cost formula: `cost = (stat / 10) * 5`
2. Implement +1 stat upgrade
3. Implement major upgrade (+10 stats, costs crystals + essences)
4. Create upgrade UI (GDD 3.3.14)

**Validation**:
- Cost calculations match GDD
- Upgrades persist
- Resources consumed correctly

---

#### Step 7.2: Implement Player Resources
**Time**: 3 hours
**Files**:
- `src/game/core/player.rs`
- `src/ui/pages/inventory.rs`

**Tasks**:
1. Track crystals and essences
2. Gain resources from expeditions and dungeons
3. Consume resources for upgrades
4. Create inventory UI (GDD 3.3.16)

**Validation**:
- Resources track correctly
- Inventory UI shows all resources

---

#### Step 7.3: Create Home Screen
**Time**: 4 hours
**Files**: `src/ui/pages/home.rs`

**Tasks**:
1. Show active expeditions with timers
2. Show active team preview
3. Show crystal count
4. Navigation buttons: [Carte] [Monstres] [Inventaire]

**UI** (GDD 3.3.1):
```
╔═══════════════════════════════╗
║  ☀️ 14:32            💎 1250  ║
╠═══════════════════════════════╣
║                               ║
║  Expéditions:                 ║
║  1. 🗺️ Payon ████░░ 23min    ║
║  2. 🗺️ Disponible             ║
║                               ║
║  Équipe active:               ║
║  🔥 Niv.24  💧 Niv.22  🌿 20  ║
║                               ║
╠═══════════════════════════════╣
║ [📍Carte][👹Monstres][📦Inv.] ║
╚═══════════════════════════════╝
```

**Validation**:
- Timers count down in real-time
- Navigation works

---

#### Step 7.4: Implement Collection Tracker
**Time**: 3 hours
**Files**: `src/ui/pages/collection.rs`

**Tasks**:
1. Track which species have been captured
2. Show collection progress per zone
3. Create collection UI (GDD 3.3.15)

**Validation**:
- Collection updates on capture
- UI shows progress correctly

---

#### Step 7.5: Update Save System
**Time**: 4 hours
**Files**: `src/game/save/save_data.rs`

**Tasks**:
1. Update SaveData struct for new system:
   ```rust
   pub struct SaveData {
       pub version: u32,
       pub monsters: Vec<Monster>,
       pub team: Team,
       pub player: Player,
       pub expeditions: [Option<Expedition>; 2],
       pub dungeon_records: HashMap<String, u16>,
       pub unlocked_zones: Vec<String>,
       pub collection: HashSet<String>,
       pub play_time_seconds: u64,
   }
   ```
2. Implement save/load
3. Add migration from old save format

**Validation**:
- Save and load work correctly
- Old saves migrate successfully

---

#### Step 7.6: UI Polish
**Time**: 6 hours
**Files**: All UI pages

**Tasks**:
1. Add swipe gestures (swipe → = back)
2. Add scroll for lists
3. Add visual feedback for taps
4. Add loading states
5. Polish all pages to match GDD mockups

**Validation**:
- All gestures work
- UI feels responsive
- Matches GDD design

---

#### Step 7.7: Tutorial/First Run
**Time**: 4 hours
**Files**: `src/ui/pages/tutorial.rs`

**Tasks**:
1. Create first-run flow:
   - Welcome screen
   - Starter monster selection
   - Basic controls explanation
2. Show tutorial on first launch only

**Validation**:
- Tutorial shows on first run only
- Controls explained clearly

---

#### Step 7.8: Final Testing & Balancing
**Time**: 8 hours

**Tasks**:
1. Play through full game loop:
   - Start new game
   - Complete expeditions
   - Capture monsters
   - Run dungeons
   - Upgrade monsters
   - Unlock zones
2. Tune balancing:
   - XP rates
   - Resource rewards
   - Difficulty scaling
3. Fix bugs
4. Optimize performance

**Validation**:
- Full game loop works
- Game feels balanced
- No crashes or major bugs

---

## Summary

### Total Estimated Time
- **Phase 1**: 16 hours (Foundation)
- **Phase 2**: 16 hours (Monster System)
- **Phase 3**: 19 hours (Expeditions)
- **Phase 4**: 13 hours (Combat Foundation)
- **Phase 5**: 17 hours (Combat Advanced)
- **Phase 6**: 17 hours (Dungeons)
- **Phase 7**: 28 hours (Progression & Polish)

**Total**: ~126 hours (~16 working days)

### Key Architecture Benefits
1. **Zero Hardcoded Data**: All game content in JSON
2. **Single Source of Truth**: Calculations in dedicated modules
3. **Clean Separation**: Game logic ≠ UI rendering
4. **Easy Balancing**: Tune JSON files without code changes
5. **Maintainable**: Clear module boundaries, DRY principles
6. **Testable**: Pure calculation functions easy to unit test

### Success Criteria
- ✅ All old Rustymon system removed
- ✅ All GDD features implemented
- ✅ All game data in JSON files
- ✅ Clean, professional architecture
- ✅ No hardcoded game values
- ✅ Reusable calculation functions
- ✅ Save/load working
- ✅ Full game loop playable

---

## Next Steps

1. **Review this plan** with the team
2. **Create Phase 1 branch**: `git checkout -b feature/phase1-foundation`
3. **Start with Step 1.1**: Create new module structure
4. **Commit frequently**: Small, atomic commits per step
5. **Test continuously**: Run `cargo check` and tests after each step

---

**Document End**
