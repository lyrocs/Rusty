# Skill System Implementation Plan for Rustymon

## Overview
This document outlines a comprehensive implementation plan for adding a skill system to the Rustymon game. The system will support both active and passive skills, with level-based learning, cooldowns, and team-wide passive effects.

## System Requirements Summary
- **Skill Storage**: JSON-based skill definitions (dedicated skills.json file)
- **Skill Limits**: 3 skills enabled per Rustymon, max 6 learnable skills
- **Skill Types**: Active (combat actions) and Passive (team buffs)
- **Active Effects**: Damage, DOT, stat buffs/debuffs, element-based attacks
- **Passive Effects**: Team-wide stat modifiers
- **UI Features**: Skill management in details, battle skill icons with cooldowns

## Phase 1: Data Structure & Models

### Step 1.1: Enhance Skill Data Structure
**File**: `data/skills.json` (currently exists with 16 skills)
**Actions**:
- Restructure existing skills.json to support new requirements
- Add fields for:
  ```json
  {
    "id": 1,
    "name": "Fireball",
    "type": "active",
    "element": "fire",
    "cooldown": 3,
    "duration": 0,
    "level_required": 5,
    "effect_type": "damage",
    "effect_value": 150,
    "effect_target": "enemy",
    "description": "Deals 150% fire damage",
    "icon": "fireball_icon"
  }
  ```
- Define effect types: "damage", "dot", "buff_self", "debuff_enemy", "passive_team"
- Add stat modifiers: "atk_percent", "def_percent", "crit_percent", "regen", "flee"

### Step 1.2: Update Enemies Data
**File**: `data/enemies.json`
**Actions**:
- Add skill learning configuration to each Rustymon:
  ```json
  "learnable_skills": [
    {"skill_id": 1, "learn_level": 5},
    {"skill_id": 2, "learn_level": 10},
    {"skill_id": 3, "learn_level": 15},
    {"skill_id": 4, "learn_level": 20},
    {"skill_id": 5, "learn_level": 30},
    {"skill_id": 6, "learn_level": 40}
  ]
  ```

### Step 1.3: Create Rust Skill Models
**File**: `src/game/skill.rs` (new file)
**Actions**:
- Define Skill struct matching JSON structure
- Create SkillEffect enum for different effect types
- Implement SkillTarget enum (self, enemy, team)
- Add SkillState struct for tracking cooldowns and active effects

### Step 1.4: Extend Rustymon Model
**File**: `src/game/rustymon.rs` (lines 59-237)
**Actions**:
- Add fields to Rustymon struct:
  ```rust
  pub learned_skills: Vec<u32>,      // Skill IDs learned
  pub enabled_skills: [Option<u32>; 3], // 3 enabled skill slots
  pub skill_cooldowns: HashMap<u32, u32>, // Skill ID -> turns remaining
  ```
- Update `from_enemy_data()` method to initialize skill fields
- Add methods: `learn_skill()`, `enable_skill()`, `disable_skill()`

## Phase 2: Skill Learning & Management System

### Step 2.1: Implement Level-Based Skill Learning
**File**: `src/game/rustymon.rs`
**Actions**:
- Modify `level_up()` method (lines 187-210) to check for new skills
- Create `check_learnable_skills()` method
- Auto-learn skills when reaching required level
- Store learned skills in rustymon data

### Step 2.2: Create Skill Manager
**File**: `src/game/skill_manager.rs` (new file)
**Actions**:
- Implement SkillManager struct
- Methods for:
  - Loading skills from JSON
  - Getting skill details by ID
  - Validating skill enablement (max 3)
  - Managing skill swapping

### Step 2.3: Update Data Loader
**File**: `src/game/data_loader.rs` (lines 165-223)
**Actions**:
- Add `load_skills()` method
- Parse skills.json into Skill structs
- Create global skill registry
- Link skills to enemies data

## Phase 3: Battle System Integration

### Step 3.1: Active Skill System
**File**: `src/game/battle.rs` (lines 44-89)
**Actions**:
- Create `use_skill()` method alongside existing `attack()` method
- Implement skill damage calculation:
  ```rust
  // Damage skills: base_damage * (skill_power / 100) * element_multiplier
  // Apply existing variance (80-120%)
  ```
- Add skill cooldown tracking
- Implement skill miss chance based on accuracy

### Step 3.2: Damage Over Time (DOT) Effects
**File**: `src/game/battle.rs`
**Actions**:
- Add ActiveEffects struct to track DOT and temporary effects:
  ```rust
  pub struct ActiveEffect {
      skill_id: u32,
      effect_type: EffectType,
      remaining_turns: u32,
      value: f32,
      target: SkillTarget,
  }
  ```
- Process DOT damage at turn start
- Track effect durations

### Step 3.3: Buff/Debuff System
**File**: `src/game/battle.rs`
**Actions**:
- Create temporary stat modifier system
- Track active buffs/debuffs with duration
- Apply modifiers in damage calculation:
  ```rust
  let modified_atk = base_atk * (1.0 + buff_percentage / 100.0);
  let modified_def = base_def * (1.0 - debuff_percentage / 100.0);
  ```

### Step 3.4: Passive Skill Integration
**File**: `src/game/battle.rs`
**Actions**:
- Collect all passive skills from team at battle start
- Create TeamPassives struct:
  ```rust
  pub struct TeamPassives {
      damage_bonus: f32,
      defense_bonus: f32,
      crit_bonus: f32,
      // etc...
  }
  ```
- Apply team passives to all damage calculations

### Step 3.5: Element System Integration
**File**: `src/game/element_system.rs` (lines 15-58)
**Actions**:
- Extend `get_element_multiplier()` to work with skill elements
- Allow skills to override attacker's natural element
- Apply element advantages to skill damage

## Phase 4: UI Implementation - Rustymon Details

### Step 4.1: Skill Display in Details Page
**File**: `src/ui/pages/rustymon_detail.rs`
**Actions**:
- Add new "Skills" section after stats display
- Show all learned skills (max 6)
- Display skill details: name, type, effect, cooldown
- Indicate which 3 skills are enabled

### Step 4.2: Skill Management UI
**File**: `src/ui/pages/rustymon_detail.rs`
**Actions**:
- Add skill slot indicators (3 slots)
- Implement toggle buttons for enable/disable
- Add visual feedback for enabled skills (highlight/checkmark)
- Show passive skills with special indicator
- Prevent enabling more than 3 skills

### Step 4.3: Skill Information Panel
**File**: `src/ui/pages/rustymon_detail.rs`
**Actions**:
- Create expandable skill descriptions
- Show skill requirements (level)
- Display cooldown information
- Show damage/effect calculations

## Phase 5: Battle UI Integration

### Step 5.1: Battle UI Layout Update
**File**: `src/ui/pages/battle.rs` (lines 500+)
**Actions**:
- Add skill bar below action buttons
- Display 3 enabled active skills as icons
- Show cooldown overlay on skills (grayed out + number)
- Add skill tooltip on hover/tap

### Step 5.2: Skill Usage Flow
**File**: `src/systems/battle.rs`
**Actions**:
- Add skill button input handling
- Create skill selection state
- Implement skill execution:
  1. Player selects skill
  2. Validate cooldown
  3. Execute skill effect
  4. Apply cooldown
  5. Update UI

### Step 5.3: Visual Feedback
**File**: `src/ui/pages/battle.rs`
**Actions**:
- Add skill animation system
- Show skill name when used
- Display damage numbers with skill color
- Show buff/debuff icons on affected Rustymon
- Add DOT indicator with remaining turns

### Step 5.4: Cooldown Management
**File**: `src/ui/pages/battle.rs`
**Actions**:
- Update cooldowns each turn
- Show remaining cooldown on skill icons
- Disable skill buttons when on cooldown
- Add cooldown reduction mechanics (optional)

## Phase 6: State & Save System

### Step 6.1: Update Save Structure
**File**: `src/game/save_system.rs`
**Actions**:
- Add skill data to save format:
  - Learned skills per Rustymon
  - Enabled skill configuration
  - Current cooldowns (for battle saves)
- Ensure backward compatibility

### Step 6.2: Battle State Management
**File**: `src/game/game_state.rs`
**Actions**:
- Track skill cooldowns in battle state
- Save active effects (DOT, buffs, debuffs)
- Persist team passive bonuses

## Phase 7: Testing & Balancing

### Step 7.1: Skill Effect Testing
**Actions**:
- Test each skill type (damage, DOT, buff, debuff)
- Verify cooldown mechanics
- Test passive team effects
- Validate element interactions

### Step 7.2: Balance Testing
**Actions**:
- Adjust skill power values
- Balance cooldown durations
- Test skill combinations
- Ensure no overpowered strategies

### Step 7.3: UI/UX Testing
**Actions**:
- Test skill enabling/disabling flow
- Verify battle skill usage
- Test visual feedback clarity
- Ensure mobile responsiveness

## Implementation Order

### Priority 1 (Core Foundation)
1. Step 1.1-1.4: Data structures and models
2. Step 2.1-2.3: Skill learning and management
3. Step 6.1-6.2: Save system updates

### Priority 2 (Battle Integration)
1. Step 3.1: Active skill system
2. Step 3.4: Passive skill integration
3. Step 3.5: Element system integration

### Priority 3 (Advanced Effects)
1. Step 3.2: DOT effects
2. Step 3.3: Buff/Debuff system

### Priority 4 (UI Implementation)
1. Step 4.1-4.3: Rustymon details UI
2. Step 5.1-5.4: Battle UI integration

### Priority 5 (Polish & Testing)
1. Step 7.1-7.3: Testing and balancing

## Technical Considerations

### Performance
- Cache skill data at game start
- Minimize skill calculations per frame
- Use efficient data structures for cooldown tracking

### Modularity
- Keep skill system separate from core battle logic
- Use trait-based design for extensibility
- Allow easy addition of new skill types

### Data Validation
- Validate skill JSON on load
- Check skill prerequisites
- Prevent invalid skill configurations

### UI Responsiveness
- Ensure skill buttons work on touch devices
- Add keyboard shortcuts for skills (1, 2, 3)
- Provide clear visual feedback for all actions

## Example Skill Definitions

### Active Damage Skill
```json
{
  "id": 1,
  "name": "Flame Strike",
  "type": "active",
  "element": "fire",
  "cooldown": 3,
  "effect_type": "damage",
  "effect_value": 200,
  "description": "Deals 200% fire damage to enemy"
}
```

### DOT Skill
```json
{
  "id": 2,
  "name": "Poison Cloud",
  "type": "active",
  "element": "poison",
  "cooldown": 5,
  "duration": 3,
  "effect_type": "dot",
  "effect_value": 50,
  "description": "Deals 50% poison damage per turn for 3 turns"
}
```

### Buff Skill
```json
{
  "id": 3,
  "name": "Battle Cry",
  "type": "active",
  "cooldown": 4,
  "duration": 3,
  "effect_type": "buff_self",
  "stat": "atk_percent",
  "effect_value": 30,
  "description": "Increases ATK by 30% for 3 turns"
}
```

### Debuff Skill
```json
{
  "id": 4,
  "name": "Weakness",
  "type": "active",
  "cooldown": 4,
  "duration": 2,
  "effect_type": "debuff_enemy",
  "stat": "def_percent",
  "effect_value": -25,
  "description": "Reduces enemy DEF by 25% for 2 turns"
}
```

### Passive Team Skill
```json
{
  "id": 5,
  "name": "Team Spirit",
  "type": "passive",
  "effect_type": "passive_team",
  "stat": "damage_bonus",
  "effect_value": 5,
  "description": "All team members gain +5% damage"
}
```

## Success Metrics
- Skills are properly loaded and assigned to Rustymon
- Players can enable/disable skills in details page
- Active skills can be used in battle with cooldowns
- Passive skills affect entire team
- Damage calculations include all skill modifiers
- UI clearly shows skill states and effects
- Save/load preserves skill configurations

## Risks & Mitigations
- **Risk**: Skill combinations too powerful
  - **Mitigation**: Implement skill stacking limits
- **Risk**: Complex UI becomes cluttered
  - **Mitigation**: Use icons and progressive disclosure
- **Risk**: Performance impact from many effects
  - **Mitigation**: Optimize effect processing, limit active effects

## Future Enhancements
- Skill upgrade system (skill levels)
- Combo skills (requiring multiple Rustymon)
- Environmental skill effects
- Skill synthesis/fusion system
- PvP-specific skill balancing