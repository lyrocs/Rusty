# ⚔️ Equipment System Design Plan
## Ragnarok Online-Inspired Equipment & Refinement

---

## 📋 Design Philosophy

**Keep It Simple:**
- No equipment drops from monsters (crafting/quest only)
- No complex inventory management for equipment
- Equipment slots are always filled (starter gear provided)
- Focus on progression through upgrading and refinement
- Clear visual feedback for equipment changes

**Ragnarok Online Inspiration:**
- Equipment slots system (Weapon, Armor, Accessory)
- Refinement system (+1, +2, ..., +10)
- Equipment evolution/upgrade paths
- Level requirements for equipment
- Stat bonuses from equipment

---

## 🎯 Core Equipment Mechanics

### **Equipment Slots (3 Slots)**

```
┌─────────────────────────────────┐
│  [WEAPON]   [ARMOR]   [ACCESSORY]│
└─────────────────────────────────┘
```

**1. Weapon Slot** 🗡️
- Provides: ATK, sometimes DEX/CRIT
- Job-specific (Sword for Swordsman, Staff for Mage, etc.)
- Affects damage output significantly

**2. Armor Slot** 🛡️
- Provides: DEF, HP, sometimes VIT
- Reduces incoming damage
- Universal or job-specific

**3. Accessory Slot** 💍
- Provides: Various stat bonuses (AGI, INT, LUK, etc.)
- Special effects (SP regen, crit bonus, etc.)
- Universal across all jobs

---

## 📊 Equipment Data Structure

### **Equipment Base Stats**

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EquipmentType {
    // Weapons
    Sword,
    Staff,
    Bow,
    Dagger,
    Axe,
    Mace,

    // Armor
    ClothArmor,
    LeatherArmor,
    PlateArmor,
    Robe,

    // Accessories
    Ring,
    Necklace,
    Earring,
    Gloves,
}

#[derive(Debug, Clone)]
pub struct Equipment {
    pub id: u16,
    pub name: &'static str,
    pub equipment_type: EquipmentType,
    pub slot: EquipmentSlot,

    // Level requirement
    pub level_req: u16,
    pub job_req: Option<&'static str>, // None = all jobs

    // Base stats (before refinement)
    pub atk_bonus: u16,
    pub def_bonus: u16,
    pub hp_bonus: u16,
    pub sp_bonus: u16,

    // Stat bonuses
    pub str_bonus: u16,
    pub agi_bonus: u16,
    pub vit_bonus: u16,
    pub int_bonus: u16,
    pub dex_bonus: u16,
    pub luk_bonus: u16,

    // Special bonuses
    pub crit_rate_bonus: u16, // +X% crit rate
    pub aspd_bonus: u16,      // +X% double attack chance

    // Refinement data
    pub refine_level: u8,     // 0 to 10 (+0 to +10)
    pub max_refine: u8,       // Usually 10

    // Upgrade path (evolution)
    pub can_upgrade: bool,
    pub upgrade_level_req: u16, // Level needed to upgrade
    pub upgrade_cost: u32,      // Zeny cost
    pub upgrades_to: Option<u16>, // Equipment ID it upgrades to
}
```

### **Equipped Items (Hero)**

Add to Hero struct:
```rust
pub equipped_weapon: Equipment,
pub equipped_armor: Equipment,
pub equipped_accessory: Equipment,
```

---

## 🔨 Refinement System

### **How Refinement Works**

**Refinement Levels:** +0 → +1 → +2 → ... → +10

**Stat Bonuses Per Refine:**
- **Weapon:** +2 ATK per refine level
- **Armor:** +1 DEF per refine level
- **Accessory:** +1 to primary stat per refine level

**Example:**
```
Dagger [+0]  → ATK: 15
Dagger [+5]  → ATK: 15 + (5 × 2) = 25 ATK
Dagger [+10] → ATK: 15 + (10 × 2) = 35 ATK
```

### **Refinement Cost**

**Zeny Cost Formula:**
```
Cost = Base_Cost × (Refine_Level + 1)

+0 → +1 = 100z
+1 → +2 = 200z
+2 → +3 = 300z
+3 → +4 = 400z
+4 → +5 = 500z
+5 → +6 = 600z
+6 → +7 = 700z
+7 → +8 = 800z
+8 → +9 = 900z
+9 → +10 = 1000z
```

**Success Rate:**
- +0 to +4: 100% success (safe refine)
- +5 to +7: 80% success
- +8 to +9: 60% success
- +9 to +10: 40% success

**Failure Penalty:**
- Safe refine (+0 to +4): No penalty on failure
- Risky refine (+5+): Refine level drops by 1 on failure
- Equipment is **never destroyed** (simplified from RO)

### **Refinement Materials**

**Simplest Approach (Zeny Only):**
- Only costs Zeny (no special materials needed)
- Easy to understand and implement

**Alternative (With Materials):**
- **Phracon** (for Weapon refine) - obtained from quests
- **Emveretarcon** (for Armor refine) - obtained from quests
- **Oridecon** (for Accessory refine) - obtained from quests

**Recommendation:** Start with Zeny-only system, add materials later if needed.

---

## 🔄 Equipment Upgrade/Evolution System

### **How Equipment Upgrades Work**

Equipment can **evolve** into a better version once level requirements are met.

**Example Upgrade Path:**

```
Rusty Dagger [Lv1]
    ↓ (Upgrade at Lv10, costs 500z)
Iron Dagger [Lv10]
    ↓ (Upgrade at Lv20, costs 2000z)
Steel Dagger [Lv20]
    ↓ (Upgrade at Lv30, costs 5000z)
Mithril Dagger [Lv30]
    ↓ (Upgrade at Lv40, costs 10000z)
Damascus Dagger [Lv40]
```

**Upgrade Rules:**
1. Player must meet level requirement
2. Pay Zeny cost
3. **Refinement level is preserved** (if +7 Iron Dagger → becomes +7 Steel Dagger)
4. Cannot downgrade equipment
5. Upgrade is instant (no materials needed for simplicity)

### **Starter Equipment**

**Every new character starts with:**

| Job | Weapon | Armor | Accessory |
|-----|--------|-------|-----------|
| **Novice** | Rusty Knife (+0) | Cotton Shirt (+0) | Wooden Ring (+0) |
| **Swordsman** | Training Sword (+0) | Padded Armor (+0) | Strength Ring (+0) |
| **Mage** | Apprentice Staff (+0) | Mage Robe (+0) | Magic Ring (+0) |
| **Archer** | Practice Bow (+0) | Leather Vest (+0) | Dexterity Gloves (+0) |
| **Thief** | Rusty Dagger (+0) | Thief Suit (+0) | Lucky Coin (+0) |
| **Acolyte** | Wooden Mace (+0) | Priest Robe (+0) | Holy Ring (+0) |
| **Merchant** | Merchant Axe (+0) | Merchant Vest (+0) | Zeny Bag (+0) |

---

## 🎨 Equipment Examples (Full Data)

### **Weapon Examples**

**1. Rusty Knife (Novice Starter)**
```rust
Equipment {
    id: 1000,
    name: "Rusty Knife",
    equipment_type: EquipmentType::Dagger,
    slot: EquipmentSlot::Weapon,
    level_req: 1,
    job_req: None, // All jobs can use
    atk_bonus: 8,
    def_bonus: 0,
    hp_bonus: 0,
    sp_bonus: 0,
    str_bonus: 0,
    agi_bonus: 0,
    vit_bonus: 0,
    int_bonus: 0,
    dex_bonus: 0,
    luk_bonus: 0,
    crit_rate_bonus: 0,
    aspd_bonus: 0,
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 10,
    upgrade_cost: 500,
    upgrades_to: Some(1001), // Iron Knife
}
```

**2. Iron Dagger (Thief Weapon, Lv10)**
```rust
Equipment {
    id: 1010,
    name: "Iron Dagger",
    equipment_type: EquipmentType::Dagger,
    slot: EquipmentSlot::Weapon,
    level_req: 10,
    job_req: Some("Thief"),
    atk_bonus: 15,
    def_bonus: 0,
    hp_bonus: 0,
    sp_bonus: 0,
    str_bonus: 0,
    agi_bonus: 2,
    vit_bonus: 0,
    int_bonus: 0,
    dex_bonus: 0,
    luk_bonus: 1,
    crit_rate_bonus: 2, // +2% crit
    aspd_bonus: 5, // +5% double attack
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 20,
    upgrade_cost: 2000,
    upgrades_to: Some(1011), // Steel Dagger
}
```

**3. Flame Sword (Swordsman Weapon, Lv25)**
```rust
Equipment {
    id: 1020,
    name: "Flame Sword",
    equipment_type: EquipmentType::Sword,
    slot: EquipmentSlot::Weapon,
    level_req: 25,
    job_req: Some("Swordsman"),
    atk_bonus: 35,
    def_bonus: 0,
    hp_bonus: 0,
    sp_bonus: 0,
    str_bonus: 5,
    agi_bonus: 0,
    vit_bonus: 2,
    int_bonus: 0,
    dex_bonus: 0,
    luk_bonus: 0,
    crit_rate_bonus: 0,
    aspd_bonus: 0,
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 35,
    upgrade_cost: 8000,
    upgrades_to: Some(1021), // Inferno Sword
}
```

**4. Mage Staff (Mage Weapon, Lv15)**
```rust
Equipment {
    id: 1030,
    name: "Mage Staff",
    equipment_type: EquipmentType::Staff,
    slot: EquipmentSlot::Weapon,
    level_req: 15,
    job_req: Some("Mage"),
    atk_bonus: 10,
    def_bonus: 0,
    hp_bonus: 0,
    sp_bonus: 20,
    str_bonus: 0,
    agi_bonus: 0,
    vit_bonus: 0,
    int_bonus: 8,
    dex_bonus: 3,
    luk_bonus: 0,
    crit_rate_bonus: 0,
    aspd_bonus: 0,
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 25,
    upgrade_cost: 4000,
    upgrades_to: Some(1031), // Wizard Staff
}
```

### **Armor Examples**

**5. Cotton Shirt (Novice Starter)**
```rust
Equipment {
    id: 2000,
    name: "Cotton Shirt",
    equipment_type: EquipmentType::ClothArmor,
    slot: EquipmentSlot::Armor,
    level_req: 1,
    job_req: None,
    atk_bonus: 0,
    def_bonus: 5,
    hp_bonus: 10,
    sp_bonus: 0,
    str_bonus: 0,
    agi_bonus: 0,
    vit_bonus: 1,
    int_bonus: 0,
    dex_bonus: 0,
    luk_bonus: 0,
    crit_rate_bonus: 0,
    aspd_bonus: 0,
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 10,
    upgrade_cost: 500,
    upgrades_to: Some(2001), // Padded Armor
}
```

**6. Iron Plate (Swordsman Armor, Lv20)**
```rust
Equipment {
    id: 2010,
    name: "Iron Plate",
    equipment_type: EquipmentType::PlateArmor,
    slot: EquipmentSlot::Armor,
    level_req: 20,
    job_req: Some("Swordsman"),
    atk_bonus: 0,
    def_bonus: 25,
    hp_bonus: 100,
    sp_bonus: 0,
    str_bonus: 2,
    agi_bonus: -2, // Heavy armor reduces AGI
    vit_bonus: 5,
    int_bonus: 0,
    dex_bonus: 0,
    luk_bonus: 0,
    crit_rate_bonus: 0,
    aspd_bonus: 0,
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 30,
    upgrade_cost: 5000,
    upgrades_to: Some(2011), // Steel Plate
}
```

### **Accessory Examples**

**7. Wooden Ring (Novice Starter)**
```rust
Equipment {
    id: 3000,
    name: "Wooden Ring",
    equipment_type: EquipmentType::Ring,
    slot: EquipmentSlot::Accessory,
    level_req: 1,
    job_req: None,
    atk_bonus: 0,
    def_bonus: 0,
    hp_bonus: 5,
    sp_bonus: 5,
    str_bonus: 1,
    agi_bonus: 0,
    vit_bonus: 0,
    int_bonus: 0,
    dex_bonus: 0,
    luk_bonus: 0,
    crit_rate_bonus: 0,
    aspd_bonus: 0,
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 10,
    upgrade_cost: 500,
    upgrades_to: Some(3001), // Bronze Ring
}
```

**8. Lucky Coin (Accessory, Lv15)**
```rust
Equipment {
    id: 3010,
    name: "Lucky Coin",
    equipment_type: EquipmentType::Ring,
    slot: EquipmentSlot::Accessory,
    level_req: 15,
    job_req: None,
    atk_bonus: 0,
    def_bonus: 0,
    hp_bonus: 0,
    sp_bonus: 0,
    str_bonus: 0,
    agi_bonus: 0,
    vit_bonus: 0,
    int_bonus: 0,
    dex_bonus: 0,
    luk_bonus: 8,
    crit_rate_bonus: 5, // +5% crit
    aspd_bonus: 0,
    refine_level: 0,
    max_refine: 10,
    can_upgrade: true,
    upgrade_level_req: 25,
    upgrade_cost: 3000,
    upgrades_to: Some(3011), // Fortune Coin
}
```

---

## 🎮 Equipment UI Design

### **Equipment Page Layout**

```
┌─────────────────────────────────────┐
│       === EQUIPMENT ===             │
│                                     │
│  [🗡️ Weapon]    [Iron Dagger +5]   │
│   ATK: +25 (15+10)  AGI+2  LUK+1   │
│   [Upgrade] [Refine]                │
│                                     │
│  [🛡️ Armor]     [Leather Vest +3]   │
│   DEF: +18 (15+3)  HP+50  VIT+3    │
│   [Upgrade] [Refine]                │
│                                     │
│  [💍 Accessory] [Lucky Coin +2]     │
│   LUK+10 (8+2)  CRIT+5%            │
│   [Upgrade] [Refine]                │
│                                     │
│  Zeny: 12,450                       │
│  [Back]                             │
└─────────────────────────────────────┘
```

### **Refine Popup**

```
┌─────────────────────────────────────┐
│     === REFINE EQUIPMENT ===        │
│                                     │
│  Iron Dagger [+5]                   │
│                                     │
│  Next Level: [+6]                   │
│  ATK: 25 → 27 (+2)                  │
│                                     │
│  Cost: 600 Zeny                     │
│  Success Rate: 80%                  │
│                                     │
│  ⚠️ Failure drops to +4             │
│                                     │
│  [Refine] [Cancel]                  │
└─────────────────────────────────────┘
```

### **Upgrade Popup**

```
┌─────────────────────────────────────┐
│     === UPGRADE EQUIPMENT ===       │
│                                     │
│  Iron Dagger [+5]                   │
│         ↓                           │
│  Steel Dagger [+5]                  │
│                                     │
│  ATK: 15 → 22 (+7 base ATK)         │
│  AGI+2 → AGI+4                      │
│  New: CRIT+3%                       │
│                                     │
│  Level Required: 20                 │
│  Your Level: 22 ✓                   │
│                                     │
│  Cost: 2,000 Zeny                   │
│                                     │
│  [Upgrade] [Cancel]                 │
└─────────────────────────────────────┘
```

---

## 📐 Stat Calculation with Equipment

### **Total Stats Formula**

```rust
// Example: Calculate total ATK
total_atk = base_atk + (STR × 2) + weapon_atk + weapon_refine_bonus

// Example: Calculate total DEF
total_def = base_def + VIT + armor_def + armor_refine_bonus

// Example: Calculate total HP
total_hp = base_hp + (level × 10) + (VIT × 10) + armor_hp_bonus

// Example: Calculate crit rate
total_crit_rate = base_crit_rate + (LUK / 20) + weapon_crit_bonus + accessory_crit_bonus
```

### **Equipment Impact on Battle Stats**

When initializing battle combatant:
```rust
pub fn start_jrpg_battle(&mut self, enemy: Enemy) {
    // Calculate stats with equipment bonuses
    let weapon = &self.hero.equipped_weapon;
    let armor = &self.hero.equipped_armor;
    let accessory = &self.hero.equipped_accessory;

    // Total ATK
    let weapon_atk = weapon.atk_bonus + (weapon.refine_level as u16 * 2);
    let total_atk = 10 + (self.hero.base_str * 2) + weapon_atk + weapon.str_bonus;

    // Total DEF
    let armor_def = armor.def_bonus + (armor.refine_level as u16 * 1);
    let total_def = 5 + self.hero.base_vit + armor_def + armor.vit_bonus;

    // Total stats with equipment bonuses
    let total_str = self.hero.base_str + weapon.str_bonus + armor.str_bonus + accessory.str_bonus;
    let total_agi = self.hero.base_agi + weapon.agi_bonus + armor.agi_bonus + accessory.agi_bonus;
    let total_vit = self.hero.base_vit + weapon.vit_bonus + armor.vit_bonus + accessory.vit_bonus;
    let total_int = self.hero.base_int + weapon.int_bonus + armor.int_bonus + accessory.int_bonus;
    let total_dex = self.hero.base_dex + weapon.dex_bonus + armor.dex_bonus + accessory.dex_bonus;
    let total_luk = self.hero.base_luk + weapon.luk_bonus + armor.luk_bonus + accessory.luk_bonus;

    // Create combatant with equipment-modified stats
    self.jrpg_hero_combatant = Some(JrpgCombatant {
        name: self.hero.name,
        level: self.hero.level,
        hp: self.hero.hp,
        max_hp: self.hero.max_hp + armor.hp_bonus,
        sp: self.hero.sp,
        max_sp: self.hero.max_sp + weapon.sp_bonus,
        attack: total_atk,
        defense: total_def,
        agility: total_agi,
        luck: total_luk,
        intelligence: total_int,
        dexterity: total_dex,
        active_effects: heapless::Vec::new(),
        available_skills: get_skills_for_job(self.hero.job),
    });
}
```

---

## 🗂️ Equipment Database Structure

### **equipment.json (Data File)**

Store equipment data in JSON format (similar to enemies.json):

```json
{
  "weapons": [
    {
      "id": 1000,
      "name": "Rusty Knife",
      "type": "Dagger",
      "level_req": 1,
      "job_req": null,
      "atk": 8,
      "def": 0,
      "hp": 0,
      "sp": 0,
      "str": 0,
      "agi": 0,
      "vit": 0,
      "int": 0,
      "dex": 0,
      "luk": 0,
      "crit_rate": 0,
      "aspd": 0,
      "can_upgrade": true,
      "upgrade_level_req": 10,
      "upgrade_cost": 500,
      "upgrades_to": 1001
    },
    {
      "id": 1010,
      "name": "Iron Dagger",
      "type": "Dagger",
      "level_req": 10,
      "job_req": "Thief",
      "atk": 15,
      "def": 0,
      "hp": 0,
      "sp": 0,
      "str": 0,
      "agi": 2,
      "vit": 0,
      "int": 0,
      "dex": 0,
      "luk": 1,
      "crit_rate": 2,
      "aspd": 5,
      "can_upgrade": true,
      "upgrade_level_req": 20,
      "upgrade_cost": 2000,
      "upgrades_to": 1011
    }
  ],
  "armors": [
    {
      "id": 2000,
      "name": "Cotton Shirt",
      "type": "ClothArmor",
      "level_req": 1,
      "job_req": null,
      "atk": 0,
      "def": 5,
      "hp": 10,
      "sp": 0,
      "str": 0,
      "agi": 0,
      "vit": 1,
      "int": 0,
      "dex": 0,
      "luk": 0,
      "crit_rate": 0,
      "aspd": 0,
      "can_upgrade": true,
      "upgrade_level_req": 10,
      "upgrade_cost": 500,
      "upgrades_to": 2001
    }
  ],
  "accessories": [
    {
      "id": 3000,
      "name": "Wooden Ring",
      "type": "Ring",
      "level_req": 1,
      "job_req": null,
      "atk": 0,
      "def": 0,
      "hp": 5,
      "sp": 5,
      "str": 1,
      "agi": 0,
      "vit": 0,
      "int": 0,
      "dex": 0,
      "luk": 0,
      "crit_rate": 0,
      "aspd": 0,
      "can_upgrade": true,
      "upgrade_level_req": 10,
      "upgrade_cost": 500,
      "upgrades_to": 3001
    }
  ]
}
```

**Alternative: Hardcode in Rust**
For embedded systems with limited storage, hardcode equipment as const arrays in Rust instead of JSON files.

---

## 📋 Implementation Plan

### **Phase 1: Core Equipment System** ⚔️

**Priority 1.1: Data Structures**
- [ ] Create `Equipment` struct
- [ ] Create `EquipmentSlot` and `EquipmentType` enums
- [ ] Add equipment fields to `Hero` struct
- [ ] Create starter equipment for each job

**Priority 1.2: Stat Calculations**
- [ ] Update battle initialization to include equipment bonuses
- [ ] Create helper function: `calculate_total_stats_with_equipment()`
- [ ] Update damage calculation to use equipment stats
- [ ] Test stat bonuses in battle

**Priority 1.3: Equipment UI**
- [ ] Create Equipment page (GamePage::Equipment)
- [ ] Display 3 equipped items with stats
- [ ] Show total stat bonuses
- [ ] Add navigation from Overview/Menu to Equipment page

---

### **Phase 2: Refinement System** 🔨

**Priority 2.1: Refinement Logic**
- [ ] Create `refine_equipment()` method
- [ ] Implement refine cost calculation
- [ ] Implement success rate calculation
- [ ] Implement refine level up/down logic
- [ ] Add Zeny deduction on refine attempt

**Priority 2.2: Refinement UI**
- [ ] Create refine popup/modal
- [ ] Display current stats and next level stats
- [ ] Show cost and success rate
- [ ] Show warning for risky refines (+5+)
- [ ] Add [Refine] and [Cancel] buttons
- [ ] Show refine result (success/failure animation)

**Priority 2.3: Visual Feedback**
- [ ] Display equipment with refine level (+0, +5, +10)
- [ ] Show refined stat bonuses separately (15 + 10 = 25)
- [ ] Color code by refine level (green +1-4, yellow +5-7, orange +8-9, red +10)

---

### **Phase 3: Equipment Upgrade/Evolution** 🔄

**Priority 3.1: Upgrade Logic**
- [ ] Create `upgrade_equipment()` method
- [ ] Check level requirement
- [ ] Check Zeny cost
- [ ] Preserve refine level on upgrade
- [ ] Replace old equipment with new equipment
- [ ] Update total stats

**Priority 3.2: Upgrade UI**
- [ ] Create upgrade popup/modal
- [ ] Display current → new equipment comparison
- [ ] Show stat changes (ATK: 15 → 22)
- [ ] Show level requirement (check or X)
- [ ] Show Zeny cost
- [ ] Add [Upgrade] and [Cancel] buttons

**Priority 3.3: Equipment Progression Paths**
- [ ] Define upgrade paths for all starter equipment
- [ ] Create 3-4 tiers per equipment type (Lv1, Lv10, Lv20, Lv30)
- [ ] Balance stat growth per tier
- [ ] Test progression flow

---

### **Phase 4: Advanced Features** ✨ (Optional)

**Priority 4.1: Equipment Crafting (Future)**
- [ ] Add crafting system (use materials from quests)
- [ ] Create crafting recipes
- [ ] Add crafting UI

**Priority 4.2: Quest Rewards (Future)**
- [ ] Add quest system
- [ ] Reward equipment or materials from quests
- [ ] Special unique equipment from quests

**Priority 4.3: Equipment Sets (Future)**
- [ ] Define equipment sets (2-3 pieces)
- [ ] Add set bonuses when wearing full set
- [ ] Display set bonus in UI

**Priority 4.4: Special Effects (Future)**
- [ ] Add equipment special effects (lifesteal, reflect, etc.)
- [ ] Add elemental weapons (fire, ice, holy, etc.)
- [ ] Add status resist equipment (poison resist, stun resist, etc.)

---

## 📝 Files That Need Changes

| File | Changes Needed |
|------|----------------|
| `src/tamagotchi/models.rs` | Add Equipment struct, add equipment fields to Hero, update battle initialization with equipment stats |
| `src/tamagotchi/ui.rs` | Create Equipment page UI, create Refine popup, create Upgrade popup |
| `src/tamagotchi/systems.rs` | Add Equipment page input handling, add refine/upgrade button handlers |
| `src/tamagotchi/game_data.rs` | Load equipment.json or define equipment as const arrays |
| `data/equipment.json` | **CREATE NEW:** Equipment database (optional, can hardcode) |

---

## 🎯 Starter Equipment Table (All Jobs)

| Job | Weapon | Stats | Armor | Stats | Accessory | Stats |
|-----|--------|-------|-------|-------|-----------|-------|
| **Novice** | Rusty Knife | ATK+8 | Cotton Shirt | DEF+5, HP+10, VIT+1 | Wooden Ring | HP+5, SP+5, STR+1 |
| **Swordsman** | Training Sword | ATK+12, STR+1 | Padded Armor | DEF+10, HP+30, VIT+2 | Strength Ring | STR+3, ATK+5 |
| **Mage** | Apprentice Staff | ATK+5, INT+5, SP+15 | Mage Robe | DEF+5, SP+30, INT+3 | Magic Ring | INT+5, SP+10 |
| **Archer** | Practice Bow | ATK+10, DEX+3 | Leather Vest | DEF+8, HP+20, AGI+2 | Dexterity Gloves | DEX+4, AGI+1 |
| **Thief** | Rusty Dagger | ATK+10, AGI+2, CRIT+2% | Thief Suit | DEF+7, HP+15, AGI+3 | Lucky Coin | LUK+5, CRIT+3% |
| **Acolyte** | Wooden Mace | ATK+8, INT+2, SP+10 | Priest Robe | DEF+6, HP+25, INT+2, SP+15 | Holy Ring | INT+3, DEX+2 |
| **Merchant** | Merchant Axe | ATK+11, STR+2 | Merchant Vest | DEF+9, HP+35, VIT+2 | Zeny Bag | STR+2, VIT+2 |

---

## 💡 Design Notes

### **Why No Equipment Drops?**
- Keeps inventory management simple
- No need for complex loot tables
- Focuses progression on upgrading existing gear
- Easier to balance (controlled progression)

### **Why 3 Slots Only?**
- Ragnarok Online has 10+ slots (too complex for small screen)
- 3 slots covers core stats (ATK, DEF, Special)
- Easy to display on 368x448 screen
- Keeps UI simple and clear

### **Why Refinement Preserves on Upgrade?**
- Rewards player investment in equipment
- Encourages refining even at low levels
- Makes progression feel continuous
- Prevents "wasted refines" frustration

### **Balance Considerations**
- Refine bonuses should be significant (+2 ATK per level = +20 ATK at +10)
- Upgrade cost should scale with level (early cheap, late expensive)
- Success rates force risk/reward decisions (+4 safe → +10 risky)
- Equipment evolution provides clear power spikes at levels 10, 20, 30, 40

---

## 🔢 Example Progression (Thief Player)

```
Level 1:  Rusty Dagger [+0]   ATK: 10 (base 8 + 2 refine)
          → Refine to +4 (safe, costs 1000z total)

Level 5:  Rusty Dagger [+4]   ATK: 18 (base 8 + 8 refine)

Level 10: Upgrade to Iron Dagger [+4]
          Iron Dagger [+4]     ATK: 23 (base 15 + 8 refine)
                               AGI+2, LUK+1, CRIT+2%, ASPD+5%

Level 15: Refine to +7 (risky, but worth it)
          Iron Dagger [+7]     ATK: 29 (base 15 + 14 refine)

Level 20: Upgrade to Steel Dagger [+7]
          Steel Dagger [+7]    ATK: 36 (base 22 + 14 refine)
                               AGI+4, LUK+2, CRIT+4%, ASPD+8%

Level 25: Push to +10 (very risky)
          Steel Dagger [+10]   ATK: 42 (base 22 + 20 refine)

Level 30: Upgrade to Mithril Dagger [+10]
          Mithril Dagger [+10] ATK: 50 (base 30 + 20 refine)
                               AGI+6, LUK+4, CRIT+6%, ASPD+10%
```

**Total Power Progression:**
- Level 1: ATK 10
- Level 30: ATK 50 (5x power increase)

---

## 🚀 Recommended Implementation Order

1. **Start with Phase 1:** Equipment data structures and stat integration
2. **Add Equipment UI:** Display equipped items on Equipment page
3. **Test in battle:** Verify equipment bonuses work correctly
4. **Implement Refinement:** Add refine system (simple version first)
5. **Implement Upgrades:** Add equipment evolution
6. **Polish UI:** Add animations, color coding, visual feedback
7. **Balance:** Adjust costs, success rates, and stat bonuses
8. **Optional:** Add crafting/quests/sets later

---

**Document Version:** 1.0
**Last Updated:** 2025-10-27
**Status:** Ready for Implementation
**Next Step:** Phase 1 - Core Equipment System
