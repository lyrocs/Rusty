# RPG Idle Game System Implementation Plan

## Overview
Create a Ragnarok Online-inspired idle RPG system with hero progression, automatic job changes (Novice → Swordsman → Knight), stats-based battle mechanics, and kill tracking.

## Core Systems

### 1. Hero System
#### 1.1 Stats Structure
- **STR (Strength)**: Increases physical attack power and weight capacity
- **AGI (Agility)**: Increases attack speed and flee rate
- **VIT (Vitality)**: Increases max HP, HP recovery, and physical defense
- **INT (Intelligence)**: Increases max SP, magic attack, and magic defense
- **DEX (Dexterity)**: Increases hit rate, reduces cast time, slight attack boost
- **LUK (Luck)**: Increases critical rate, perfect dodge, and drop rates

#### 1.2 Level System
- Experience points (EXP) for leveling
- Base level (1-99)
- Job level (1-50)
- Stat points gained per level
- Job points gained per job level

#### 1.3 Job Progression (Automatic)
```
Novice (Lv 1-10) → Swordsman (Lv 10-40) → Knight (Lv 40+)
```
- Job changes happen **automatically** when reaching the required level
- At level 10: Novice → Swordsman
- At level 40: Swordsman → Knight
- No manual job change required

### 2. Battle System
#### 2.1 Combat Mechanics
- **Damage Calculation**:
  - Physical Damage = (STR × 2) + (DEX × 0.5) + WeaponATK - EnemyDEF
  - Critical Damage = Physical Damage × 2
  - Critical Rate = LUK / 10 + Equipment bonuses
  - Hit Rate = (DEX × 2) + (LUK × 0.5) + BaseLv
  - Flee Rate = (AGI × 2) + (LUK × 0.3) + BaseLv

#### 2.2 HP/SP System
- **HP Calculation**: BaseHP + (VIT × 10) + (BaseLv × 5)
- **SP Calculation**: BaseSP + (INT × 5) + (BaseLv × 2)
- HP Regen: VIT / 5 per second
- SP Regen: INT / 10 per second

#### 2.3 Attack Speed
- Base attack interval: 2000ms
- Modified by AGI: interval = 2000 / (1 + AGI/50)

### 3. Enemy System
#### 3.1 Enemy Stats
- HP, ATK, DEF, EXP reward, Drop rates
- Different enemy types with varying difficulty
- Respawn system after death

#### 3.2 Enemy Progression
- Enemies scale with hero level
- Different zones with appropriate enemies

### 4. Kill Tracking System
- Track total kills per monster type
- Stored persistently for future features
- Display kill count in UI (optional)

### 5. Data Storage
#### 5.1 JSON Files
- `enemies.json`: Enemy definitions (HP, ATK, DEF, EXP, drops)
- `jobs.json`: Job progression data (Novice, Swordsman, Knight)
- `skills.json`: Available skills per job (future feature)

#### 5.2 Save System
- Persistent hero stats (level, EXP, stats)
- Kill counts per monster type
- Current job and progression

## Implementation Phases

### Phase 1: Core Structure (Current)
- [ ] Create data models for Hero, Enemy, Stats
- [ ] Implement JSON loading system
- [ ] Create basic battle calculations
- [ ] Add HP/SP bars to UI

### Phase 2: Battle System
- [ ] Implement damage formulas
- [ ] Add attack timing based on AGI
- [ ] Create battle animation triggers
- [ ] Add death/respawn logic

### Phase 3: Progression System
- [ ] Implement experience gain on enemy kill
- [ ] Add leveling mechanics with stat increases
- [ ] Create automatic job change system (Lv 10 → Swordsman, Lv 40 → Knight)
- [ ] Add stat point allocation (automatic based on job)

### Phase 4: Kill Tracking
- [ ] Implement kill counter per monster type
- [ ] Save kill data persistently
- [ ] Display total kills (optional UI element)

### Phase 5: UI Integration
- [ ] Add HP/SP bars above hero and enemy sprites
- [ ] Implement floating damage numbers near animations (Ragnarok Online style)
- [ ] Add damage number animation (fade out, float up)
- [ ] Display level/exp progress (optional sidebar or top bar)

### Phase 6: Polish
- [ ] Balance combat formulas
- [ ] Add more enemies
- [ ] Implement equipment system
- [ ] Add skills/abilities

## File Structure
```
assets/
├── data/
│   ├── enemies.json      # Enemy definitions (HP, ATK, DEF, EXP, drops)
│   ├── jobs.json         # Job data (Novice, Swordsman, Knight)
│   └── skills.json       # Skills per job (future feature)
src/
├── game/
│   ├── mod.rs           # Module exports and GameState
│   ├── hero.rs          # Hero struct with stats and progression
│   ├── enemy.rs         # Enemy struct and spawning
│   ├── stats.rs         # Stat calculations (damage, HP, SP)
│   ├── battle.rs        # Battle system and damage formulas
│   ├── progression.rs   # Leveling and automatic job changes
│   ├── kill_tracker.rs  # Kill count tracking per monster
│   └── data_loader.rs   # JSON loading utilities
```

## Technical Considerations
- Use Bevy ECS for game state management
- Serialize/deserialize with serde_json
- Store game state in NonSend resources
- Update battle calculations in fixed timestep
- Render HP/SP bars with embedded-graphics

## UI Layout
```
┌─────────────────────────────────────┐
│  Lv: 15 Swordsman     EXP: [████  ] │
├─────────────────────────────────────┤
│                                     │
│         Enemy: Hornet               │
│         HP: [████████░░░░]          │
│            45   ← Floating damage   │
│                                     │
│      [Enemy Animation]              │
│                                     │
│                                     │
│              12  ← Floating damage  │
│      [Hero Animation]               │
│                                     │
│         Hero                        │
│         HP: [█████████░░]           │
│         SP: [██████░░░░░]           │
│                                     │
└─────────────────────────────────────┘
```

**UI Elements:**
- **Top Bar**: Hero level, job name, and EXP bar
- **HP/SP Bars**: Positioned above/below sprites
  - Enemy: HP bar only (above sprite)
  - Hero: HP and SP bars (below sprite)
- **Floating Damage Numbers**:
  - Appear near the sprite that takes damage
  - Animate upward and fade out (0.5-1 second)
  - Color coded: White (normal), Yellow (critical), Red (missed/dodge)
- **No stat panel**: Stats are internal calculations only

## Next Steps
1. Create JSON data files with enemy and job definitions
2. Implement Hero and Enemy structs with stats (STR, AGI, VIT, INT, DEX, LUK)
3. Create battle calculation module with damage formulas
4. Implement automatic job change system (Lv 10 → Swordsman, Lv 40 → Knight)
5. Add kill tracking per monster type
6. Integrate with existing battle animation system
7. Add HP/SP bars and floating damage numbers to UI