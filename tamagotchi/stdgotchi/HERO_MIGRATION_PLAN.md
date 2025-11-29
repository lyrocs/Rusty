# Hero Job System Migration Plan

## Overview
This document outlines the complete transformation from the current Rustymon team-based game to a hero job evolution system, similar to Ragnarok Online's job progression.

## 🔴 Phase 1: Remove Old Systems

### 1.1 Remove Rustymon Core System
**Files to Delete:**
- `/src/game/rustymon.rs` - Complete monster class
- `/src/game/rustymon_factory.rs` - Monster creation factory
- `/src/game/rustymon_team.rs` - Team management system

**Files to Modify:**
- `/src/ecs/resources.rs` - Remove `rustymon_team` field from `GameManager`
- `/src/game/mod.rs` - Remove rustymon module exports

### 1.2 Remove Fragment & Summon System
**Files to Delete:**
- `/src/game/fragment_collection.rs` - Fragment collection logic
- `/src/ui/pages/fragment_collection_page.rs` - Fragment UI
- `/src/ui/pages/rustymon_summon.rs` - Summon UI

**Database Changes:**
- Remove fragment data from `/assets/data/rustymons.json`
- Remove fragment references from save system

### 1.3 Remove Skill System
**Files to Delete:**
- `/src/game/skill.rs` - Complete skill system (active & passive)
- `/src/ui/pages/rustymon_skills.rs` - Skill management UI

**Battle System Modifications:**
- Remove skill casting logic from `/src/game/battle.rs`
- Remove cooldown tracking
- Remove team passive calculations

### 1.4 Remove Team-Related UI
**Files to Delete:**
- `/src/ui/pages/rustymon_list.rs` - Team list page
- `/src/ui/pages/rustymon_detail.rs` - Individual monster details
- `/src/systems/rustymon_navigation.rs` - Team navigation system

### 1.5 Clean Up Battle System
**Modify `/src/game/battle.rs`:**
- Remove team switching logic
- Remove 3v3 battle mode (delete `/src/ui/pages/battle_3v3.rs`)
- Simplify to 1v1 hero vs enemy
- Remove skill-based damage calculations
- Keep basic attack formulas using stats

### 1.6 Data Files Cleanup
**Delete from `/assets/data/`:**
- `rustymons.json` - Monster definitions
- `skills.json` - Skill definitions
- Any team-related configuration

## 🟢 Phase 2: Add Hero Job System

### 2.1 Create Hero Core System
**New Files to Create:**

#### `/src/game/hero.rs`
```rust
// Hero class with stats and job
pub struct Hero {
    // Identity
    pub name: String,
    pub job: JobClass,
    pub job_level: u8,

    // Core Stats (KEEP EXISTING)
    pub level: u32,
    pub experience: u32,
    pub health: i32,
    pub max_health: i32,

    // Base Stats (KEEP EXISTING)
    pub strength: u16,
    pub dexterity: u16,
    pub intelligence: u16,
    pub vitality: u16,
    pub agility: u16,

    // Derived Stats (KEEP EXISTING)
    pub attack: u16,
    pub defense: u16,
    pub magic_attack: u16,
    pub magic_defense: u16,
    pub speed: u16,
    pub hit: u16,
    pub flee: u16,
    pub critical: u16,
    pub aspd: f32,
}
```

#### `/src/game/job_system.rs`
```rust
// Job evolution tree
pub enum JobClass {
    // First Class
    Novice,

    // Second Class
    Swordsman,
    Mage,
    Archer,
    Thief,
    Merchant,
    Acolyte,

    // Third Class (Examples)
    Knight,
    Wizard,
    Hunter,
    Assassin,
    Blacksmith,
    Priest,

    // Advanced Classes
    LordKnight,
    HighWizard,
    Sniper,
    AssassinCross,
    Whitesmith,
    HighPriest,
}

pub struct JobEvolution {
    pub from: JobClass,
    pub to: Vec<JobClass>,
    pub level_requirement: u32,
    pub stat_bonuses: StatBonus,
}
```

### 2.2 Create Job Data Files
**New File: `/assets/data/jobs.json`**
```json
{
  "jobs": [
    {
      "id": "novice",
      "name": "Novice",
      "tier": 1,
      "base_stats": {
        "str": 5,
        "dex": 5,
        "int": 5,
        "vit": 5,
        "agi": 5
      },
      "stat_growth": {
        "str": 1,
        "dex": 1,
        "int": 1,
        "vit": 1,
        "agi": 1
      },
      "evolutions": ["swordsman", "mage", "archer", "thief", "merchant", "acolyte"]
    }
  ]
}
```

### 2.3 Update Game Manager
**Modify `/src/ecs/resources.rs`:**
```rust
pub struct GameManager {
    pub hero: Hero,  // Replace rustymon_team with single hero
    pub current_area: Area,
    pub battle_state: Option<Battle>,
    // Keep other fields
}
```

### 2.4 Create New UI Pages
**New Files:**
- `/src/ui/pages/hero_status.rs` - Display hero stats and job
- `/src/ui/pages/job_change.rs` - Job evolution selection UI

### 2.5 Simplify Battle System
**Modify `/src/game/battle.rs`:**
- Single hero vs single enemy
- Basic attack only (no skills)
- Damage calculation based on stats:
  ```rust
  damage = (hero.attack * 2) - enemy.defense
  ```

## 🔧 Phase 3: Implementation Steps

### Step 1: Create Feature Branch
```bash
git checkout -b feature/hero-job-system
```

### Step 2: Backup Current System
1. Create backup branch: `git checkout -b backup/rustymon-system`
2. Tag current version: `git tag v1.0-rustymon`

### Step 3: Remove Old Systems (Week 1)
1. Start with UI removal (least dependencies)
2. Remove skill system
3. Remove fragment/summon system
4. Remove team management
5. Finally remove core Rustymon classes

### Step 4: Implement Hero System (Week 2)
1. Create Hero struct and basic stats
2. Implement job class enum
3. Create job evolution logic
4. Update save/load system

### Step 5: Update Battle System (Week 3)
1. Simplify to 1v1 combat
2. Remove skill-based damage
3. Implement stat-based combat
4. Update battle UI

### Step 6: Create New UI (Week 4)
1. Hero status page
2. Job change interface
3. Update main menu
4. Polish and test

## 📊 Data Migration

### Save File Changes
**Old Format (v3):**
```json
{
  "version": 3,
  "rustymon_team": {...},
  "fragment_collection": {...}
}
```

**New Format (v4):**
```json
{
  "version": 4,
  "hero": {
    "name": "Player",
    "job": "novice",
    "level": 1,
    "stats": {...}
  }
}
```

### Migration Script
Create `/src/game/save_migration.rs` to convert old saves:
- Extract first Rustymon's stats
- Convert to Hero with Novice job
- Preserve level and experience

## 🎯 Key Considerations

### What to Keep
- ✅ Stats system (str, dex, int, vit, agi)
- ✅ Derived stats (atk, def, matk, mdef, etc.)
- ✅ Level and experience system
- ✅ Element system (for damage calculation)
- ✅ Area progression
- ✅ Quest system (modify objectives)
- ✅ Equipment system (if exists)
- ✅ WiFi features

### What to Remove
- ❌ All Rustymon classes and data
- ❌ Team management (4 active + bank)
- ❌ Fragment collection
- ❌ Summon/evolution via fragments
- ❌ Active skills (cooldowns, effects)
- ❌ Passive skills (team bonuses)
- ❌ 3v3 battle mode

### What to Add
- ➕ Hero class with single character
- ➕ Job class system
- ➕ Job evolution tree
- ➕ Job-specific stat bonuses
- ➕ Simplified combat (stat-based only)

## 🚀 Testing Plan

### Phase 1 Tests
- Ensure all Rustymon references are removed
- Verify no skill system remnants
- Check save system doesn't reference old data

### Phase 2 Tests
- Hero creation and stat calculation
- Job evolution triggers
- Battle damage formulas
- Save/load functionality

### Phase 3 Tests
- Complete gameplay loop
- UI navigation
- Performance testing
- Edge cases (death, level up, job change)

## 📝 Notes

### Potential Issues
1. **Heavy coupling**: Battle system may be deeply integrated with team/skill logic
2. **UI dependencies**: Many UI components expect team data
3. **Save compatibility**: Need migration for existing players
4. **Quest objectives**: Many quests may reference Rustymon catching

### Recommendations
1. Consider keeping some monster elements as "enemies only"
2. Add basic combat abilities per job (not full skill system)
3. Implement job-specific equipment later
4. Keep WiFi battle system but adapt for hero vs hero

## Timeline Estimate
- **Total Duration**: 4-6 weeks
- **Phase 1**: 1-2 weeks (removal)
- **Phase 2**: 2-3 weeks (implementation)
- **Phase 3**: 1 week (testing & polish)

## Next Steps
1. Review this plan with team
2. Create detailed task list
3. Set up CI/CD for new branch
4. Begin Phase 1 implementation

---
*Document created: 2025-11-29*
*Target completion: End of Q1 2025*