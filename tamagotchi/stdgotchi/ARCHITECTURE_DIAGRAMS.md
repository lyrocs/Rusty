# Rustymon Architecture Diagrams & Quick Reference

## 1. Damage Calculation Flow

```
Battle Attack
    ↓
calculate_damage()
    ├─→ Check Hit/Miss
    │   ├─→ attacker_hit vs defender_flee
    │   └─→ Result: 20-95% chance
    │
    ├─→ If Hit:
    │   ├─→ Base Damage = ATK - DEF (min 1)
    │   ├─→ Variance: 80-120%
    │   ├─→ Check Critical (crit_rate %)
    │   │   └─→ If Crit: × 2.0
    │   └─→ Element Advantage (if applicable)
    │       └─→ × 1.5 / 1.0 / 0.5 / 0.1
    │
    └─→ Result: DamageResult {
        damage: u32,
        is_critical: bool,
        is_miss: bool,
    }
```

## 2. Battle System State Machine

```
Battle State:

START
  ↓
RUSTYMON_ATTACKS
  ├─→ Calculate damage
  ├─→ Enemy takes damage
  ├─→ Check if enemy dead
  │   └─→ If yes → VICTORY
  └─→ ENEMY_ATTACKS
      ↓
      ├─→ Calculate damage
      ├─→ Rustymon takes damage
      ├─→ Check if rustymon dead
      │   └─→ If yes → DEFEAT
      └─→ Allow team switch or next turn
          └─→ RUSTYMON_ATTACKS

VICTORY
  ├─→ Award EXP
  ├─→ Award gold
  ├─→ Check fragment drop
  └─→ Return to map

DEFEAT
  ├─→ Switch to next Rustymon
  │   (if available)
  ├─→ Or game over
  └─→ Return to map / death sequence
```

## 3. Rustymon Stat Derivation

```
BASE STATS (from level-up or capture)
    ├─ STR (Strength)
    ├─ DEX (Dexterity)
    ├─ VIT (Vitality)
    ├─ INT (Intelligence)
    └─ LUK (Luck)
        ↓
    STAT FORMULAS (derived from base stats + level)
        ├─ max_hp = 40 + (vit × 10) + (level × 5)
        ├─ atk = 5 + (str × 2) + level
        ├─ def = 2 + vit + (level / 2)
        ├─ hit = 175 + dex + level
        ├─ flee = 100 + (dex / 2) + level
        └─ crit = 5.0 + (luk × 0.3)
        ↓
    COMBAT STATS
        ├─ HP (with current_hp)
        ├─ ATK
        ├─ DEF
        ├─ HIT (accuracy)
        ├─ FLEE (evasion)
        └─ CRIT% (critical chance)
```

## 4. Element Advantage Matchup Matrix

```
        Fire  Wind  Earth Water Poison Holy  Shadow Ghost Undead
Fire      —    0.5   1.5   0.5   —     —     —     —     —
Wind      1.5   —    0.5   1.5   —     —     —     —     —
Earth     0.5   1.5   —    0.5   —     —     —     —     —
Water     1.5   0.5   1.5   —     —     —     —     —     —
Poison    —     —     —     —     —     1.5   —     —     0.1
Holy      —     —     —     —     0.5   —     1.5   —     2.0★
Shadow    —     —     —     —     —     1.5   —     —     —
Ghost     —     —     —     —     —     —     —     0.75  —
Neutral   —     —     —     —     —     —     —     0.5   —
Undead    —     —     —     —     1.2   0.5   —     —     0.75

Key: 1.5 = Effective (★2.0 = Super Effective), 0.5 = Not Effective, 0.1 = Resisted
```

## 5. Rustymon Collection & Battle Flow

```
EXPLORATION
    ↓
ENCOUNTER ENEMY
    ├─→ Load from GameData
    ├─→ Scale stats based on player level
    └─→ Enter Battle
        ↓
    BATTLE
        ├─→ Draw Rustymon (sprite animations)
        ├─→ Draw Enemy (sprite animations)
        ├─→ Turn-based combat
        │   ├─→ Check attack timing
        │   ├─→ Display damage numbers
        │   └─→ Update HP bars
        │
        └─→ ENEMY DEFEATED
            ├─→ Award EXP (all team members)
            ├─→ Award Gold
            ├─→ Check Fragment Drop
            │   ├─→ roll < drop_rate
            │   └─→ Add to FragmentCollection
            ├─→ Update Kill Tracker
            └─→ Return to map

FRAGMENT COLLECTION
    ├─→ View fragments for each enemy
    ├─→ Progress bar (X/Y fragments)
    └─→ When complete:
        ↓
    SUMMONING
        ├─→ Create Rustymon with random stats
        ├─→ Set to level 1
        ├─→ Add to collection
        └─→ Add to team (or bank if full)

TEAM MANAGEMENT
    ├─→ View all Rustymon
    ├─→ Select individual for detail view
    ├─→ Add/Remove from active team
    ├─→ Switch between team and bank
    └─→ Max 4 in active team
```

## 6. Data Loading Pipeline

```
ASSETS (embedded in binary)
    ├─ /assets/data/enemies.json
    ├─ /assets/data/items.json
    ├─ /assets/data/maps.json
    ├─ /assets/data/recipes.json
    ├─ /assets/data/upgrade_recipes.json
    └─ /assets/data/skills.json
        ↓
    GameData::load_from_assets()
        ↓
    PARSED DATA
        ├─ HashMap<u32, EnemyData>
        ├─ HashMap<u32, ItemData>
        ├─ HashMap<u32, MapData>
        ├─ HashMap<String, Vec<Recipe>>
        ├─ HashMap<String, Vec<UpgradeRecipe>>
        └─ (Skills not yet loaded)
        ↓
    GameManager (available during gameplay)
        ↓
    Used by:
        ├─ Battle system (enemy spawning)
        ├─ Rustymon factory (creature creation)
        ├─ UI pages (item/recipe display)
        └─ Fragment system (drop rate lookup)
```

## 7. Save/Load Serialization

```
SaveData (JSON serialized)
├─ version: u32                          [for future migrations]
├─ hero: Hero                            [legacy, kept for now]
├─ kill_tracker: KillTracker             [enemy defeats]
├─ current_location_id: u32              [player position]
├─ play_time_seconds: u64                [play duration]
├─ save_timestamp: u64                   [when saved]
├─ rustymon_collection: Vec<Rustymon>    [all owned creatures]
│   └─ Each Rustymon ~200 bytes JSON
├─ rustymon_team: RustymonTeam           [active team + bank]
│   ├─ active_slots: [Option<String>; 4]
│   ├─ active_index: usize
│   └─ bank: Vec<String>
└─ fragment_collection: FragmentCollection
    └─ HashMap<u32, u32>                 [enemy_id -> count]

Save File Size Estimate:
- 50 Rustymon: ~10KB
- 100 Rustymon: ~20KB
- Full save: ~30-40KB (plenty of space on SD card)
```

## 8. UI Page Hierarchy

```
MAIN MENU
├─ New Game
├─ Continue
└─ Settings
    ↓
MAP PAGE (gameplay hub)
├─ Button: North/South/East/West (navigation)
├─ Button: "Rustymon" → RUSTYMON_LIST_PAGE
├─ Button: "Fragments" → FRAGMENT_COLLECTION_PAGE
├─ Button: "Menu" → MENU_PAGE
└─ Encounters enemy → BATTLE_PAGE
    ├─ Button: "Switch" (in battle)
    └─ Button: "Boot" → MENU_PAGE (from battle)

RUSTYMON_LIST_PAGE
└─ List all owned Rustymon
    ├─ Click → RUSTYMON_DETAIL_PAGE
    │   ├─ Button: "Add to Team"
    │   ├─ Button: "Remove from Team"
    │   └─ Button: "Back"
    └─ Show active team indicator

FRAGMENT_COLLECTION_PAGE
├─ List all enemies
├─ Show fragment count progress
├─ Click when complete → RUSTYMON_SUMMON_PAGE
│   ├─ Preview new Rustymon with rolled stats
│   ├─ Button: "Confirm"
│   └─ Button: "Cancel"
└─ Button: "Back"

BATTLE_PAGE
├─ Display Rustymon sprite (left)
├─ Display Enemy sprite (right)
├─ Show HP bars
├─ Show damage numbers
├─ Show team slots
├─ Touch to switch Rustymon
└─ Boot button → Menu
```

## 9. Skill System Integration Points (Future)

```
CURRENT FLOW (without skills):
Enemy Defeated
    ├─→ rustymon_attack_enemy()
    │   ├─→ Base ATK stat
    │   ├─→ Element advantage
    │   └─→ calculate_damage()
    └─→ Result

PROPOSED FLOW (with skills):
Enemy Defeated
    ├─→ Select skill (UI)
    │   ├─→ Check SP cost
    │   ├─→ Check if learnable
    │   └─→ Confirm selection
    │
    ├─→ rustymon_skill_attack()
    │   ├─→ Get skill data
    │   ├─→ Base damage = ATK × (skill_power / 100)
    │   ├─→ Apply skill modifiers
    │   ├─→ Element advantage
    │   ├─→ Status effects (if any)
    │   └─→ calculate_damage_with_skill()
    │
    └─→ Result (with special effects)

SKILL LEARNING:
Level Up
    ├─→ Check level_up_skills
    ├─→ Add learnable skill to Rustymon
    └─→ Grant new move

Or:

Crafting
    ├─→ Use materials
    ├─→ Teach skill to Rustymon
    └─→ Consume SP
```

## 10. Performance Profile (ESP32-S3)

```
MEMORY ALLOCATION:
┌──────────────────────────────────┐
│ ESP32-S3 SRAM: 512KB             │
├──────────────────────────────────┤
│ Rustymon struct:  120 bytes      │
│ × 100 collection: 12KB           │
│                                  │
│ GameData (cached): ~50KB         │
│ Text/UI buffers:   ~20KB         │
│ Sprite animation:  ~30KB         │
│ Remaining:        ~400KB         │
└──────────────────────────────────┘

DISPLAY:
├─ Resolution: 390×450 AMOLED
├─ FPS target: 60 (min 30)
├─ Partial redraws: Per page
└─ Full redraws: ~10-20ms per frame

SAVE FILE:
├─ Average game: 30-40KB JSON
├─ SD card storage: 8GB+ available
└─ Load time: <500ms
```

## 11. Data Flow: Battle EXP to Level Up

```
Battle Victory
    ↓
Enemy defeated
    ├─→ Get exp_reward from enemy
    ├─→ Award to active Rustymon
    │   └─→ rustymon.gain_exp(exp)
    │       ├─→ exp += amount
    │       ├─→ If exp >= exp_to_next:
    │       │   └─→ level_up()
    │       │       ├─→ level += 1
    │       │       ├─→ random stat +1
    │       │       ├─→ recalculate_stats()
    │       │       │   ├─→ new max_hp
    │       │       │   ├─→ new atk
    │       │       │   ├─→ new def
    │       │       │   └─→ etc.
    │       │       └─→ heal HP gained
    │       └─→ exp = 0 (reset counter)
    │
    └─→ Save to SaveData
        └─→ Write to SD card (autosave)

EXP Formula:
    exp_to_next = level^2 × 100
    
Example:
    Level 1 → 2: need 100 EXP
    Level 2 → 3: need 400 EXP
    Level 5 → 6: need 2500 EXP
    Level 10 → 11: need 10000 EXP
```

## 12. Battle UI Touch Zones

```
BATTLE_PAGE DISPLAY (390×450):

┌────────────────────────────────┐
│ Enemy Info                      │  (0, 0) - (390, 60)
│ Enemy Sprite      │             │  Enemy left side
├──────────────────┬─────────────┤
│                  │ HP/Status    │  Rustymon right side
│ Rustymon Sprite  │ Element icon │  (200, 80) - (390, 300)
│ (left side)      │              │
│ (0, 80)-(150,200)│              │
├──────────────────┴─────────────┤
│ Action Log / Damage Numbers     │  (0, 300) - (390, 380)
├──────────────────┬──────────────┤
│ Team Slots       │ Switch Button │  Team: (0, 390)-(200, 450)
│ □ □ □ □          │ (300, 400)    │  Switch: (300, 400)-(390, 440)
│ (active highlight)└──────────────┤
└────────────────────────────────┘

Touch Input:
├─ Anywhere → Next turn (if auto-attacking)
├─ Team slots → Click to switch Rustymon
├─ Switch button (300-390, 400-440) → Manual switch
└─ Boot button (hardware) → Open menu
```

## 13. Element System Quick Lookup

```
ELEMENT ADVANTAGES:

Fire wins against:  Wind (1.5x)
Wind wins against:  Earth (1.5x)
Earth wins against: Water (1.5x)
Water wins against: Fire (1.5x)
[Elemental Cycle]

Holy wins against:  Undead (2.0x) ★, Shadow (1.5x)
Poison wins against: Holy (1.5x)
Ghost wins against: Neutral (1.5x)

Special Cases:
- Poison vs Undead: 0.1x (heavily resisted)
- Undead vs Poison: 1.2x (benefits)
- Same element: 0.75x (reduced damage)

Neutral element:
- No advantages, no disadvantages (1.0x)
```

## 14. Fragment Drop System

```
BATTLE RESULT:
Enemy Defeated
    ↓
check_fragment_drop()
    ├─→ Generate random: 0.0 - 1.0
    ├─→ Compare with drop_rate
    │   └─→ Example: Poring drop_rate = 0.3 (30%)
    │
    ├─→ If roll < drop_rate:
    │   ├─→ Fragment dropped!
    │   ├─→ fragment_collection.add_fragment(enemy_id, 1)
    │   └─→ Show notification
    │
    └─→ Else:
        └─→ No fragment this time

FRAGMENT REQUIREMENTS (from enemies.json):
Poring:       3 fragments → summon
Fabre:        3 fragments → summon
Hornet:       12 fragments → summon (rarer!)
Thief Bug:    20 fragments → summon (very rare!)

SUMMONING:
Collect X fragments
    ├─→ RustymonFactory::create_from_enemy()
    ├─→ Generate random stats
    ├─→ UUID instance ID
    ├─→ Add to rustymon_collection
    ├─→ Add to team (or bank if full)
    └─→ Reset fragment counter (consume fragments)
```

---

## Quick Reference: Common Code Locations

| Task | File | Lines |
|------|------|-------|
| Calculate damage | `/src/game/battle.rs` | 44-89 |
| Rustymon level up | `/src/game/rustymon.rs` | 205-237 |
| Check element advantage | `/src/game/element_system.rs` | 15-58 |
| Team switching | `/src/game/rustymon_team.rs` | 81-100 |
| Battle display | `/src/ui/pages/battle.rs` | 1-500 |
| Stat calculation | `/src/game/stats.rs` | 30-85 |
| Data loading | `/src/game/data_loader.rs` | 165-223 |
| Fragment drops | `/src/game/battle.rs` | 150-169 |
| Save/Load | `/src/game/save.rs` | All |
| Enemy scaling | `/src/game/enemy.rs` | 44-81 |

