# Rustymon Codebase Exploration - Complete Summary

## What Was Explored

This exploration provided a comprehensive analysis of the Rustymon game codebase to support implementation of a skill system. Three detailed documents were created:

### 1. **CODEBASE_ANALYSIS.md** (17 sections)
Main reference document covering:
- Battle system implementation & damage calculations
- Rustymon and enemy data structures
- Element system with advantage multipliers
- Battle page UI components
- Stats and modifiers system
- Level system mechanics
- Team management
- Rustymon factory & creation
- UI display components
- Data loading pipeline
- Fragment system
- Save data serialization
- Current skill system status
- GameManager integration
- File structure summary
- Performance considerations
- Architecture overview
- Skill system recommendations

### 2. **ARCHITECTURE_DIAGRAMS.md** (14 diagrams + tables)
Visual quick-reference guide with:
- Damage calculation flow diagram
- Battle state machine
- Rustymon stat derivation tree
- Element advantage matchup matrix
- Collection & battle flow
- Data loading pipeline
- Save/load serialization structure
- UI page hierarchy
- Skill system integration points (proposed)
- Performance profile breakdown
- EXP to level-up flow
- Battle UI touch zones layout
- Element advantages quick lookup
- Fragment drop system flow
- Quick reference code location table

## Key Findings

### Battle System
- **Damage Formula**: `base_damage = ATK - DEF` with 80-120% variance
- **Accuracy System**: Hit chance = 80% + (hit - flee)/2, clamped 20-95%
- **Critical Hits**: 5% base + (LUK * 0.3)%, deals 2x damage
- **Element System**: 10 elements with 1.0x-2.0x damage multipliers
- **Attack Flow**: Turn-based with animation states (Idle, Attack, Attacked, Death)

### Rustymon Structure
- **5 Base Stats**: STR, DEX, VIT, INT, LUK (generated on capture)
- **5 Derived Combat Stats**: ATK, DEF, HIT, FLEE, CRIT (calculated from base + level)
- **2 Resource Stats**: Current HP, Current EXP
- **Level Range**: 1-99
- **Unique ID**: UUID for each instance
- **Species ID**: Links to enemy data (1002=Poring, 1007=Fabre, etc.)

### Team System
- **Active Team**: 4 Rustymon slots (max capacity)
- **Bank Storage**: Unlimited additional Rustymon
- **Switching**: Can swap active Rustymon during battle
- **Flexibility**: Move between team and bank freely

### Fragment Collection
- **Drop Rate**: Per-enemy percentage (0.3 for Poring, 0.02 for Thief Bug)
- **Summoning Cost**: 3-20 fragments per species
- **Stat Generation**: Random within species range
- **Starting Level**: Always level 1

### UI Architecture
- **Page-Based System**: Each view is a separate page struct
- **Touch Input**: Handled via touch areas with bounding boxes
- **Battle Page**: Sprite animations with 100ms frame delay
- **Damage Numbers**: Float upward with fade-out animation
- **Element Colors**: Distinct RGB values per element

### Data Storage
- **Format**: JSON embedded in binary (include_str!)
- **Enemies.json**: 4 enemies defined with stats and drops
- **Skills.json**: 16 skills defined but not integrated
- **Save Size**: ~30-40KB for average game (scalable)
- **Storage**: SD card with plenty of capacity

## Critical Integration Points for Skills

### 1. Damage Calculation
**File**: `/src/game/battle.rs` (lines 44-89)
- Current: `calculate_damage()` uses ATK vs DEF
- Enhancement: Add skill power multiplier and status effects
- Function signatures ready to support skill data

### 2. Rustymon Structure
**File**: `/src/game/rustymon.rs` (lines 59-119)
- Add: `skills: Vec<RustymonSkill>`
- Add: `current_sp: u32`, `max_sp: u32` for skill points
- Modify: `level_up()` to check for learnable skills

### 3. Battle System
**File**: `/src/systems/battle.rs`
- Modify: Input handling for skill selection
- Add: Skill availability checks (SP cost, learn requirements)
- Update: Turn logic to apply skill effects

### 4. Battle UI
**File**: `/src/ui/pages/battle.rs` (lines 1-500)
- Add: Skill menu or hotbar display
- Add: SP bar alongside HP bar
- Modify: Touch zones for skill selection
- Status effects visualization

### 5. Rustymon Detail Page
**File**: `/src/ui/pages/rustymon_detail.rs`
- Add: Skill list section
- Show: Learned and learnable skills
- Display: Skill requirements (level, species)

### 6. Data Loading
**File**: `/src/game/data_loader.rs` (lines 165-223)
- Modify: Load skills.json (currently skipped)
- Add: Link skills to Rustymon species
- Create: `SkillData` structure

### 7. Level-Up System
**File**: `/src/game/rustymon.rs` (lines 205-237)
- Check: `level_up_skills` at each level
- Grant: New skills when requirements met
- Notify: Player of new skill learned

## Implementation Strategy

### Phase 1: Data Structure (Foundation)
1. Create `RustymonSkill` struct in new `/src/game/skills.rs`
2. Add skills field to Rustymon
3. Add SP tracking to Rustymon
4. Create `Skill` and `RustymonSkillTable` from JSON

### Phase 2: Battle Integration (Core Mechanics)
1. Modify damage calculation to accept skill data
2. Implement skill effect application
3. Add SP cost checking
4. Update level-up to grant skills

### Phase 3: UI Implementation (Player Interaction)
1. Add skill display to Rustymon detail page
2. Create skill selection UI in battle
3. Show SP bars and cooldowns
4. Display skill effects in battle log

### Phase 4: Polish & Balance (Polish)
1. Test skill combinations with elements
2. Balance damage multipliers
3. Tune SP costs and cooldowns
4. Add visual effects for skills

## Recommended Skills Structure

Based on existing skills.json:

```rust
pub struct Skill {
    pub id: u32,
    pub name: String,
    pub sp_cost: u32,
    pub skill_type: SkillType,  // Physical, Magic, Healing, Buff, Debuff, Utility
    pub power: u32,              // Damage multiplier (100 = 100% ATK)
    pub description: String,
}

pub enum SkillType {
    Physical,    // Damage = ATK * (power / 100)
    Magic,       // Damage = INT * (power / 100)
    Healing,     // Heal = INT * (power / 100)
    Buff,        // Stat increase for duration
    Debuff,      // Reduce enemy stats
    Utility,     // Special effects
}

pub struct RustymonSkill {
    pub skill: Skill,
    pub level_learned: u32,
}
```

## Performance Impact

### Memory
- Each RustymonSkill: ~200 bytes
- Per Rustymon: 4-6 skills = ~1-1.2KB extra
- Collection of 100: ~100-120KB additional
- Still within ESP32-S3 budget

### CPU
- Skill lookup: O(1) with hashmap
- Damage calculation: +1 float multiplication
- No noticeable performance impact

### Storage
- Skills.json already in assets
- Just needs parsing and linking
- No additional save file size

## Testing Checklist for Implementation

- [ ] Skill struct creates and serializes
- [ ] Skills load from JSON correctly
- [ ] Rustymon learn skills on level-up
- [ ] Skill selection works in UI
- [ ] Damage calculation with skills
- [ ] SP cost deducted correctly
- [ ] Physical skills scale with STR
- [ ] Magic skills scale with INT
- [ ] Healing skills work properly
- [ ] Status effects apply/wear off
- [ ] Element advantage + skill combo
- [ ] Save/load preserves learned skills
- [ ] Memory usage acceptable
- [ ] Battle performance maintained
- [ ] UI responsive with skill display

## Known Limitations & Constraints

1. **Memory**: Max ~100 Rustymon in collection
2. **Display**: 390x450 AMOLED limits UI elements
3. **Touch**: 240x240 active area for touch input
4. **Animation**: Frame-based (100ms per frame)
5. **Save Size**: JSON format (could switch to binary for compression)
6. **Performance**: Target 60 FPS, accept 30 FPS minimum

## Advantages of Current Architecture

1. **Clean Separation**: Game logic separate from UI
2. **Modular Design**: Easy to add new systems
3. **ECS Architecture**: Bevy makes systems composable
4. **Trait-Based Pages**: Pages implement common interface
5. **Data-Driven**: JSON assets easy to modify without recompile
6. **Type Safety**: Rust prevents many bugs at compile time

## Documentation Files Created

1. **CODEBASE_ANALYSIS.md** (17 sections, ~700 lines)
   - Comprehensive reference with code locations
   - Suitable for detailed implementation planning

2. **ARCHITECTURE_DIAGRAMS.md** (14 diagrams, quick lookup)
   - Visual flows and state machines
   - Quick reference tables
   - Perfect for understanding system interactions

3. **EXPLORATION_SUMMARY.md** (this file)
   - High-level overview of findings
   - Implementation strategy
   - Testing checklist
   - Recommendations

## Next Steps for Skill Implementation

1. Read CODEBASE_ANALYSIS.md sections 1-6 for foundational understanding
2. Review ARCHITECTURE_DIAGRAMS.md sections 1, 3, 9 for data flow
3. Create `/src/game/skills.rs` with Skill structures
4. Extend Rustymon struct with skills and SP fields
5. Modify damage calculation in `/src/game/battle.rs`
6. Update level-up logic in `/src/game/rustymon.rs`
7. Implement skill UI in `/src/ui/pages/battle.rs`
8. Test with Poring (easiest Rustymon to work with)
9. Balance and polish

## Conclusion

The Rustymon codebase is well-structured and ready for skill system implementation. The existing damage calculation, element system, and level-up mechanics provide solid foundations. The modular page-based UI makes it straightforward to add skill selection. With the comprehensive analysis provided, you have all the information needed to implement a robust skill system that integrates seamlessly with the existing game architecture.
