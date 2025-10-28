# 🎮 JRPG Battle System Improvement Plan
## Ragnarok Online-Inspired Gameplay Enhancement

---

## 📊 Current State Analysis

### ✅ What Works Well
- Clean turn-based combat flow
- Proper state management (Start → PlayerTurn → PlayerAction → EnemyTurn → EnemyAction)
- Visual feedback with damage numbers and HP bars
- Basic damage formula with ATK/DEF calculations
- Enemy data structure ready for expansion

### ❌ Current Issues

**1. Missing Core Mechanics:**
- No skills system (SP exists but not used)
- No damage variance (combat feels repetitive)
- Enemies always attack (no AI strategy)
- No status effects or buffs/debuffs

**2. Shallow Combat:**
- Only 2 viable actions: Attack or Run
- No combo system
- No critical hits
- No element system
- No job-specific abilities

---

## 🎯 Improvement Plan: Ragnarok Online Style

### **Phase 1: Add Combat Depth** ⚔️

#### 1.1 Add Damage Variance & Critical Hits
**Current:** Damage is fixed (ATK - DEF/2)

**Ragnarok Online Style:**
- **Base Damage:** (ATK × 0.8 to ATK × 1.2) - (DEF/2) [±20% variance]
- **Critical Hit:** 5% chance to deal 140% damage (ignores DEF)
- **Lucky Strike:** 2% chance to deal 200% damage
- **Display:** Show "CRITICAL!" or "LUCKY!" text in battle

**Implementation:**
```rust
// Add to JrpgCombatant
pub crit_rate: u16,  // Base 5%, can be increased by stats
pub luck: u16,       // Affects crit and lucky strike rate
```

#### 1.2 Add ASPD (Attack Speed) System
**Ragnarok Online Core Mechanic:** AGI affects attack speed

**Implementation:**
- Add `agility` stat to JrpgCombatant
- High AGI = chance for double attack (20% at AGI 50+)
- Add "AGI" stat display in battle UI
- Formula: `double_attack_chance = agility / 250` (capped at 30%)

---

### **Phase 2: Skill System** 🔥

#### 2.1 Job-Based Skill Trees (RO Style)
**Current Jobs (from your code):** Swordsman, Mage, Archer, Merchant, Thief, Acolyte

**Example Skills Per Job:**

**Swordsman:**
- **Bash** (SP: 8) - Deal 150% ATK damage, 10% stun chance
- **Magnum Break** (SP: 15) - Deal 120% ATK to enemy, splash damage concept
- **Provoke** (SP: 5) - Reduce enemy DEF by 30%, increase your ATK by 20%, 3 turns

**Mage:**
- **Fire Bolt** (SP: 12) - Deal (INT × 2) magic damage, ignores DEF
- **Cold Bolt** (SP: 12) - Deal (INT × 1.8) magic damage, 15% slow (enemy loses turn)
- **Lightning Bolt** (SP: 12) - Deal (INT × 2.2) magic damage, 10% stun chance

**Archer:**
- **Double Strafe** (SP: 10) - Attack twice in one turn
- **Arrow Shower** (SP: 15) - Deal 80% ATK damage (area concept)
- **Improve Concentration** (SP: 8) - Increase AGI and DEX by 30%, 3 turns

**Thief:**
- **Steal** (SP: 10) - Steal Zeny (10-50z) from enemy, 60% success
- **Hiding** (SP: 12) - Dodge next enemy attack 100%, counterattack for 80% ATK
- **Envenom** (SP: 15) - Deal 120% ATK + poison (5 dmg/turn for 3 turns)

**Acolyte:**
- **Heal** (SP: 13) - Restore (INT × 3) HP to self
- **Blessing** (SP: 10) - Increase ATK/DEF by 20%, 4 turns
- **Divine Protection** (SP: 12) - Reduce damage taken by 40%, 2 turns

**Merchant:**
- **Mammonite** (SP: 8) - Spend 50z to deal 180% ATK damage
- **Discount** (SP: 5) - Steal item from enemy, 30% success
- **Enlarge Weight** (SP: 10) - Increase max HP by 20%, 3 turns

#### 2.2 Skill Selection UI
**Change Battle Menu from:**
```
[Attack] [Skill]  [Item]
[Defend] [Run]    [Empty]
```

**To Skill Submenu:**
```
When tapping "Skill":
[Skill 1: Bash]     SP: 8
[Skill 2: Provoke]  SP: 5
[Skill 3: Magnum]   SP: 15
[Back]
```

**Updated Battle Menu (Simplified):**
```
[Attack] [Skill]  [Run]
```

#### 2.3 Skill Data Structure
```rust
pub struct JrpgSkill {
    pub id: u16,
    pub name: &'static str,
    pub sp_cost: u16,
    pub skill_type: SkillType,  // Physical, Magic, Buff, Debuff
    pub power: u16,             // Damage multiplier (150 = 150%)
    pub effect: Option<SkillEffect>,
    pub duration: u8,           // For buffs/debuffs
}

pub enum SkillType {
    Physical,
    Magic,
    Buff,
    Debuff,
    Healing,
    Utility,
}

pub enum SkillEffect {
    Damage(u16),
    Heal(u16),
    Stun(u8),              // Stun for X turns
    Poison(u16, u8),       // Damage per turn, duration
    BuffAtk(u16, u8),      // Increase ATK by X%, duration
    BuffDef(u16, u8),
    BuffAgi(u16, u8),
    DebuffAtk(u16, u8),
    DebuffDef(u16, u8),
    Steal(u16, u16),       // Min/max zeny
    MultiHit(u8),          // Number of hits
    DodgeNext,             // Dodge next attack (like Hiding)
}
```

---

### **Phase 3: Active Status Effects & Buffs** 💫

#### 3.1 Status Effect System (RO Style)
**Add StatusEffect enum:**
- **Poison:** Lose 5% max HP per turn, 3-5 turns
- **Stun:** Skip next turn (can't act)
- **Slow:** AGI reduced by 50%, 2 turns
- **Burn:** Lose 3% max HP per turn, 2 turns
- **Freeze:** Skip next turn + take 50% more damage
- **Blind:** 40% chance to miss attacks, 3 turns

**Add Buff/Debuff System:**
- **ATK Up/Down:** ±20-40% ATK modification
- **DEF Up/Down:** ±20-40% DEF modification
- **AGI Up/Down:** Affects double attack chance
- **Blessing:** +20% all stats, 4 turns
- **Curse:** -20% all stats, 3 turns

**Visual Indicators:**
- Show status icons above HP bar (poison skull, flame icon, etc.)
- Display buff duration countdown
- Show "POISONED!" message when damage triggers

**Status Effect Structure:**
```rust
pub struct ActiveStatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: u8,           // Turns remaining
    pub power: u16,             // Effect strength (%)
}

pub enum StatusEffectType {
    Poison,
    Stun,
    Slow,
    Burn,
    Freeze,
    Blind,
    AtkBuff,
    DefBuff,
    AgiBuff,
    AtkDebuff,
    DefDebuff,
    AgiDebuff,
    Blessing,
    DodgeNext,
}
```

#### 3.2 Enemy Skills & AI
**Give Enemies Skills Too!**

**Example (Hornet - Level 11):**
- 70% chance: Normal Attack
- 20% chance: Poison Sting (deal 80% ATK + poison)
- 10% chance: Defend

**Example (Thief Bug - Level 21):**
- 60% chance: Normal Attack
- 25% chance: Steal (steal 20-100z from player)
- 10% chance: Hide (dodge next attack)
- 5% chance: Envenom

**Add to enemies.json:**
```json
{
    "id": 1004,
    "name": "Hornet",
    "skills": [
        {
            "name": "Poison Sting",
            "sp_cost": 0,
            "power": 80,
            "effect": "poison",
            "effect_duration": 3
        }
    ],
    "ai_pattern": [
        {"action": "attack", "weight": 70},
        {"action": "skill:0", "weight": 20},
        {"action": "defend", "weight": 10}
    ]
}
```

---

### **Phase 4: Advanced Combat Features** 🎲

#### 4.1 Combo System
**Ragnarok-Style Combo Chain:**
- Land 3 attacks in a row without missing → Unlock combo
- Combo attack: Deal 175% damage
- Resets if you miss or use skill

**Display:**
- Show "COMBO x2" → "COMBO x3" → "COMBO READY!" text
- Glowing attack button when combo available

**Implementation:**
```rust
// Add to GameState
pub jrpg_combo_count: u8,
pub jrpg_combo_ready: bool,
```

#### 4.2 Element System (Later Expansion)
**RO Element Wheel:**
- Fire > Earth > Wind > Water > Fire
- Neutral (no weakness/resistance)
- Holy vs Undead/Demon

**Add to enemies.json:**
```json
{
    "element": "earth",
    "element_level": 1
}
```

**Damage Modifiers:**
- Strong Against: 125% damage
- Weak Against: 75% damage
- Neutral: 100% damage

#### 4.3 Experience & Leveling in Battle
**Display EXP Gained:**
- Show "Base EXP: +158" and "Job EXP: +90" after victory
- Animate EXP bars filling up
- Show "LEVEL UP!" if level increases
- Display stat increases

---

## 📋 Recommended Implementation Order

### **Priority 1 (Fix Core Issues):**
1. ✅ Add damage variance (±20% randomness)
2. ✅ Add critical hit system (5% chance, 140% damage)
3. ✅ Add AGI stat and double attack chance
4. ✅ Add LUCK stat for lucky strikes

### **Priority 2 (Add Depth):**
5. ✅ Implement 2-3 skills per job class
6. ✅ Create skill selection UI
7. ✅ Add SP consumption and skill effects
8. ✅ Add status effects (poison, stun, buffs/debuffs)

### **Priority 3 (Polish & Strategy):**
9. ✅ Implement enemy AI with skill usage
10. ✅ Add combo system
11. ✅ Visual improvements (status icons, buff timers, critical hit text)
12. ✅ Add skill animations

### **Priority 4 (Optional Enhancements):**
13. ⏳ Element system
14. ⏳ Equipment system affecting battle stats
15. ⏳ Party system (multiple heroes)
16. ⏳ Boss battles with phases

---

## 🎯 Key Design Principles (Ragnarok Online Style)

1. **Every Action Should Feel Impactful**
   - Randomness keeps combat exciting
   - Critical hits feel rewarding
   - Skills have visible, powerful effects

2. **Strategic Decision-Making**
   - SP management matters (use skills wisely)
   - Status effects create tactical opportunities
   - Timing buffs/debuffs is crucial

3. **Job Identity**
   - Each job has unique playstyle
   - Mage = high SP cost, high damage magic
   - Thief = steal, dodge, poison
   - Swordsman = tanky, stunning, high ATK

4. **Risk vs Reward**
   - Use expensive skill for big damage or save SP?
   - Go aggressive or apply buffs first?
   - Chain attacks for combo or use skill?

5. **Visual Feedback**
   - Show CRITICAL!, COMBO!, POISON! messages
   - Display status effect icons
   - Animate damage numbers with variance

---

## 📝 Files That Need Changes

| File | Changes Needed |
|------|----------------|
| `src/tamagotchi/models.rs` | Add: AGI/LUCK stats, Skill struct, StatusEffect enum, damage variance, combo counter |
| `src/tamagotchi/systems.rs` | Add: Skill selection logic, status effect processing, enemy AI, combo system |
| `src/tamagotchi/ui.rs` | Add: Skill menu UI, status icons, buff timers, critical hit text, combo display |
| `src/tamagotchi/game_data.rs` | Add: Load skills.json, enemy AI patterns |
| `src/tamagotchi/data/skills.json` | **CREATE NEW:** Skill database |
| `src/tamagotchi/data/enemies.json` | **MODIFY:** Add enemy skills and AI patterns |

---

## 🗂️ New Data Structures Summary

### JrpgCombatant (Extended)
```rust
pub struct JrpgCombatant {
    // Existing
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub sp: u16,
    pub max_sp: u16,
    pub attack: u16,
    pub defense: u16,

    // New Stats
    pub agility: u16,          // For double attack chance
    pub luck: u16,             // For critical/lucky hits
    pub intelligence: u16,     // For magic damage
    pub dexterity: u16,        // For accuracy (future)

    // New Combat State
    pub active_effects: Vec<ActiveStatusEffect>,
    pub available_skills: Vec<JrpgSkill>,
}
```

### GameState (Extended)
```rust
// Add to GameState
pub jrpg_combo_count: u8,
pub jrpg_combo_ready: bool,
pub jrpg_selected_skill: Option<u16>,
pub jrpg_skill_menu_open: bool,
pub jrpg_skill_selected_index: Option<usize>,
```

### Battle Menu States
```rust
pub enum JrpgBattleMenuState {
    Main,           // Show Attack, Skill, Run
    SkillSelect,    // Show skill list
}
```

---

## 📐 Updated Battle Flow

```
Start Battle
    ↓
PlayerTurn (Main Menu: Attack/Skill/Run)
    ↓
    ├─→ Attack Selected
    │       ↓
    │   Calculate Damage (variance + crit + combo)
    │       ↓
    │   PlayerAction (animation)
    │       ↓
    │   Check Victory → Yes: Victory State
    │       ↓ No
    │   Process Status Effects (poison damage, buff countdown)
    │       ↓
    │   EnemyTurn
    │
    ├─→ Skill Selected
    │       ↓
    │   Open Skill Menu
    │       ↓
    │   Select Skill → Check SP
    │       ↓
    │   Execute Skill (damage/heal/buff/debuff)
    │       ↓
    │   PlayerAction (animation)
    │       ↓
    │   Apply Skill Effects (status, buffs)
    │       ↓
    │   Check Victory → Yes: Victory State
    │       ↓ No
    │   Process Status Effects
    │       ↓
    │   EnemyTurn
    │
    └─→ Run Selected
            ↓
        Try Escape (50% success)
            ↓
        Escaped or Failed
            ↓
        If Failed: EnemyTurn

EnemyTurn
    ↓
AI Decision (based on ai_pattern)
    ↓
    ├─→ Enemy Attack
    ├─→ Enemy Skill
    └─→ Enemy Defend
    ↓
Check if Player Stunned (skip turn)
    ↓
EnemyAction (animation)
    ↓
Check Defeat → Yes: Defeat State
    ↓ No
Process Status Effects
    ↓
PlayerTurn
```

---

## 🎨 UI Layout Changes

### Battle Menu (Simplified)
```
Bottom section (3 buttons in a row):
┌─────────────────────────────────────┐
│  [Attack]   [Skill]   [Run]         │
└─────────────────────────────────────┘
```

### Skill Menu (When Skill is tapped)
```
┌─────────────────────────────────────┐
│  Bash            SP: 8              │
│  Provoke         SP: 5              │
│  Magnum Break    SP: 15             │
│  [Back]                             │
└─────────────────────────────────────┘
```

### Status Effect Display (Above HP Bar)
```
┌─────────────────────────────────────┐
│  [💀 Poison: 2] [⚔️ ATK+: 3]        │
│  Hero HP: ████████░░  85/100       │
└─────────────────────────────────────┘
```

### Combat Messages
```
"CRITICAL HIT!"      (Red, large font)
"LUCKY STRIKE!"      (Gold, large font)
"COMBO x3!"          (Orange, animated)
"POISONED!"          (Purple)
"STUNNED!"           (Yellow)
"ATK UP!"            (Green)
```

---

## 🚀 Next Steps

1. **Start with Priority 1:** Damage variance and critical hits
2. **Then Priority 2:** Implement skill system for 1-2 jobs as proof of concept
3. **Test and balance:** Make sure skills feel powerful but balanced
4. **Add status effects:** Implement buff/debuff/status system
5. **Enemy AI:** Give enemies skills and decision-making
6. **Polish:** Add visual effects, animations, and feedback

---

## ❓ Implementation Questions

1. **Which job should we implement skills for first?** (Recommend: Swordsman + Mage as they're most different)
2. **Should AGI affect turn order later?** (Like in real RO)
3. **Max number of skills per job?** (Recommend: 3-5 skills)
4. **Should skills have levels?** (Bash Lv1, Bash Lv2, etc.)
5. **Should we add equipment that affects battle stats?** (Weapons, armor)

---

**Document Version:** 1.0
**Last Updated:** 2025-10-27
**Status:** Ready for Implementation
