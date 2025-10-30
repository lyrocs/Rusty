# Equipment System Implementation Plan

## Overview
Complete redesign of the equipment system to create meaningful progression through crafting, cards, and build diversity.

## Core Design Principles
- **No equipment drops** - Only materials drop from monsters
- **Crafting-based progression** - All equipment is crafted using materials
- **Card collection** - Rare cards (0.2%) provide build customization
- **Build diversity** - Multiple viable builds (AGI, STR, VIT, DEF)
- **City-based progression** - Each city offers tier-appropriate crafting

## 1. Equipment Slots (6 Total)

### Layout (Left to Right)
```
[Weapon] - [Armor]
[Shoes]  - [Garment]
[Access1] - [Access2]
```

### Slot Types & Primary Stats
- **Weapon**: Primary ATK, secondary stats vary by build
- **Armor**: Primary DEF, HP bonuses
- **Shoes**: Primary AGI, movement/dodge bonuses
- **Garment**: Secondary DEF, elemental resists (future)
- **Accessory 1**: Utility stats (crit, ASPD, etc)
- **Accessory 2**: Utility stats (LUK, hit rate, etc)

## 2. Tier System

### Level & Tier Breakdown
| Tier | Level Range | City | Materials | Base Stats |
|------|-------------|------|-----------|------------|
| T1 | 1-19 | Prontera | Common/Uncommon from Poring, Fabre | Low |
| T2 | 20-39 | Payon (future) | Common/Uncommon from Hornet, Wolf | Medium |
| T3 | 40-59 | Morroc (future) | Uncommon/Rare from Desert mobs | High |
| T4 | 60-79 | Aldebaran (future) | Rare from Clock Tower mobs | Very High |
| T5 | 80-99 | Juno (future) | Boss materials + Rare | Legendary |

### Equipment Per Tier (2 Options Each)
Each slot has exactly 2 crafting options per tier:
- **Build A**: AGI/Crit/ASPD focused (farming, DPS)
- **Build B**: STR/VIT/DEF focused (tanking, bossing)

## 3. Material System

### T1 Materials (Prontera Region)
From existing monsters near Prontera:

#### Poring (Level 1)
- **Common (70%)**: Jellopy - Basic crafting material
- **Uncommon (30%)**: Sticky Mucus - Enhanced T1 crafts
- **Rare (5%)**: Poring Essence - Card slot upgrades
- **Card (0.2%)**: Poring Card - +10% EXP (Accessory only)

#### Fabre (Level 6)
- **Common (70%)**: Fluff - Cloth/light armor material
- **Uncommon (30%)**: Spider Silk - AGI equipment material
- **Rare (5%)**: Fabre Essence - Card slot upgrades
- **Card (0.2%)**: Fabre Card - +5 SP/sec (Armor only)

#### Hornet (Level 11)
- **Common (70%)**: Bee Sting - Weapon crafting
- **Uncommon (30%)**: Royal Jelly - HP/SP bonuses
- **Rare (5%)**: Hornet Essence - Card slot upgrades
- **Card (0.2%)**: Hornet Card - +10% ASPD (Weapon only)

#### Thief Bug (Level 21) - T2 Preview
- **Common (70%)**: Worm Peeling - T2 material
- **Uncommon (30%)**: Chitin Shell - T2 armor material
- **Rare (5%)**: Bug Essence - T2 card slots
- **Card (0.2%)**: Thief Bug Card - +100 HP (Garment only)

## 4. T1 Equipment Recipes (Prontera Blacksmith)

### Weapons (T1, Level 1-19)
```json
{
  "t1_agility_dagger": {
    "name": "Swift Knife",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Bee Sting": 5,
      "Jellopy": 10
    },
    "zeny_cost": 500,
    "stats": {
      "atk": 12,
      "agi": 2,
      "aspd": 5
    },
    "card_slots": 1
  },
  "t1_strength_sword": {
    "name": "Iron Blade",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Bee Sting": 8,
      "Fluff": 5
    },
    "zeny_cost": 500,
    "stats": {
      "atk": 18,
      "str": 2,
      "crit": 2
    },
    "card_slots": 1
  }
}
```

### Armor (T1, Level 1-19)
```json
{
  "t1_defense_armor": {
    "name": "Padded Shirt",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Fluff": 15,
      "Jellopy": 5
    },
    "zeny_cost": 400,
    "stats": {
      "def": 8,
      "hp": 50,
      "vit": 1
    },
    "card_slots": 1
  },
  "t1_vitality_armor": {
    "name": "Vital Vest",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Fluff": 10,
      "Royal Jelly": 3
    },
    "zeny_cost": 600,
    "stats": {
      "def": 5,
      "hp": 100,
      "vit": 3
    },
    "card_slots": 1
  }
}
```

### Shoes (T1, Level 1-19)
```json
{
  "t1_agility_shoes": {
    "name": "Light Boots",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Jellopy": 8,
      "Spider Silk": 2
    },
    "zeny_cost": 350,
    "stats": {
      "agi": 3,
      "flee": 5,
      "move_speed": 5
    },
    "card_slots": 1
  },
  "t1_defense_shoes": {
    "name": "Heavy Boots",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Fluff": 8,
      "Sticky Mucus": 3
    },
    "zeny_cost": 350,
    "stats": {
      "def": 3,
      "hp": 30,
      "vit": 1
    },
    "card_slots": 1
  }
}
```

### Garment (T1, Level 1-19)
```json
{
  "t1_agility_garment": {
    "name": "Wind Cape",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Spider Silk": 4,
      "Fluff": 6
    },
    "zeny_cost": 400,
    "stats": {
      "agi": 2,
      "flee": 8,
      "def": 2
    },
    "card_slots": 1
  },
  "t1_defense_garment": {
    "name": "Guard Mantle",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Sticky Mucus": 5,
      "Fluff": 8
    },
    "zeny_cost": 400,
    "stats": {
      "def": 5,
      "hp": 40,
      "damage_reduction": 2
    },
    "card_slots": 1
  }
}
```

### Accessories (T1, Level 1-19)
```json
{
  "t1_critical_ring": {
    "name": "Lucky Ring",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Jellopy": 10,
      "Bee Sting": 2
    },
    "zeny_cost": 300,
    "stats": {
      "luk": 2,
      "crit": 5,
      "hit": 3
    },
    "card_slots": 1
  },
  "t1_strength_ring": {
    "name": "Power Ring",
    "tier": 1,
    "level_req": 1,
    "materials": {
      "Sticky Mucus": 3,
      "Jellopy": 8
    },
    "zeny_cost": 300,
    "stats": {
      "str": 2,
      "atk": 3,
      "hp": 20
    },
    "card_slots": 1
  }
}
```

## 5. Card System

### Card Slot Restrictions
Each card can only be equipped in specific slot types:

| Card | Allowed Slots | Effect |
|------|--------------|--------|
| Poring Card | Accessory Only | +10% EXP gain |
| Fabre Card | Armor Only | +5 SP regen/sec |
| Hornet Card | Weapon Only | +10% ASPD |
| Thief Bug Card | Garment Only | +100 HP, +5 VIT |

### Card Slot Upgrade System
At the Refinery NPC:
- **Add 2nd slot**: 3x [Monster] Essence + 2000z
- **Add 3rd slot**: 5x [Monster] Essence + 5000z (T3+ equipment only)
- **Add 4th slot**: 10x [Monster] Essence + 10000z (T5 equipment only)

Maximum slots by tier:
- T1: 1-2 slots
- T2: 1-2 slots
- T3: 2-3 slots
- T4: 2-3 slots
- T5: 3-4 slots

## 6. Build Preset System

### Three Preset Slots
- **Preset 1**: Quick-swap equipment set
- **Preset 2**: Quick-swap equipment set
- **Preset 3**: Quick-swap equipment set

### UI Flow
```
Equipment Page
├── Current Equipment Display (6 slots)
├── [Preset 1] [Preset 2] [Preset 3] buttons
├── [Save to Preset] [Load Preset] buttons
└── [View All Equipment] button
```

### Preset Features
- One-tap to switch entire equipment loadout
- Saves all 6 equipment pieces + socketed cards
- Visual indicator for active preset
- Cannot load preset if equipment is broken/missing

## 7. NPC System

### Prontera NPCs (T1 City)

#### Blacksmith NPC
Location: Prontera center
Functions:
- **View Recipes**: Filter by slot, see material requirements
- **Craft Equipment**: Create T1 equipment
- **Dismantle**: Break equipment into 50% materials back

#### Refinery NPC (Enhanced)
Location: Prontera east
Functions:
- **Refine**: Upgrade equipment +0 to +10
- **Add Card Slots**: Use essences to add slots
- **Socket Cards**: Insert/remove cards (removal costs 1000z)

### Future Cities & NPCs
Each city will have tier-locked Blacksmiths:
- **Payon** (T2): Forest/nature themed equipment
- **Morroc** (T3): Desert/assassin themed equipment
- **Aldebaran** (T4): Clock/mechanical themed equipment
- **Juno** (T5): Magical/legendary equipment

## 8. Implementation Phases

### Phase 1: Core Systems ✅ COMPLETE
- [x] Extend equipment slots from 3 to 6 ✅
- [x] Update Hero model to support new slots ✅
- [ ] Create T1 equipment data (12 items total) ⏸️ Deferred
- [ ] Update material drop tables for Poring, Fabre, Hornet ⏸️ Deferred
- [ ] Implement crafting system at Blacksmith NPC ⏸️ Deferred
- [x] Add card slot restrictions ✅

### Phase 2: Card & Preset System ✅ COMPLETE
- [x] Implement card socketing UI ✅
- [x] Add card slot upgrade at Refinery ✅
- [x] Create preset save/load system ✅
- [x] Add preset UI to Equipment page ✅
- [x] Implement card effects in combat ✅

#### Phase 2 Implementation Details

**Card System:**
- Cards are loaded from `assets/data/cards.json`
- Each card has specific slot restrictions (Weapon, Armor, Shoes, Garment, Accessory)
- Card effects include: EXP bonus, SP regen, ASPD bonus, HP bonus, VIT bonus
- Players can socket/remove cards via touch UI on equipment page
- Card data is saved/loaded with equipment (`EQUIP.SAV` file)

**Combat Integration:**
- Card bonuses are applied via `Hero::get_total_card_bonuses()` method
- VIT bonus added to total defense calculation
- HP bonus added to max HP during battle initialization
- EXP bonus applied as percentage multiplier when gaining experience
- SP regen bonus added to rest system recovery rate
- ASPD bonus stored but not yet used in turn-based combat (future enhancement)

**Preset System:**
- 3 preset slots for quick-swapping full equipment loadouts
- Each preset saves all 6 equipment pieces with refine levels
- Presets are accessible via buttons on equipment page
- Touch-based menu for Save/Load/Clear preset actions
- Visual indicators show active preset (green) and saved presets (blue)

**Save/Load System:**
- Equipment details (including socketed cards) saved to `EQUIP.SAV`
- Format: `id,refine,card_slots,card0,card1,card2,card3` per equipment, semicolon-separated
- Backward compatible with old saves (gracefully handles missing `EQUIP.SAV`)
- Card slots restore with 0 representing empty slots

**UI Improvements:**
- 2x3 grid layout for 6 equipment slots
- Clickable equipment slots open card management menu
- Card socket menu shows:
  - Current socketed cards with [Remove] buttons
  - Empty slots with [Socket] buttons (placeholder)
  - Add Slot button showing essence and zeny costs
  - Equipment name and slot count (X/Y format)
- Preset buttons with color-coded states
- "[Tap]" indicators on equipment with card slots

### Phase 3: Polish & Balance
- [ ] Add dismantle feature
- [ ] Balance material drop rates
- [ ] Add crafting animations/feedback
- [ ] Create equipment comparison UI
- [ ] Add "New!" indicators for recently crafted items

### Phase 4: Future Content (T2-T5)
- [ ] Add new cities with unique Blacksmiths
- [ ] Create T2-T5 equipment recipes
- [ ] Add elemental system
- [ ] Implement mini-boss/MVP materials
- [ ] Create legendary T5 equipment with unique effects

## 9. Data Structure Changes

### Updated Equipment Structure
```json
{
  "id": 1100,
  "name": "Swift Knife",
  "tier": 1,
  "slot": "Weapon",
  "level_req": 1,
  "build_type": "AGI",
  "base_stats": {
    "atk": 12,
    "agi": 2,
    "aspd": 5
  },
  "card_slots": 1,
  "max_card_slots": 2,
  "refine_level": 0,
  "max_refine": 10,
  "craft_materials": {
    "909": 10,  // Jellopy x10
    "939": 5    // Bee Sting x5
  },
  "craft_cost": 500
}
```

### Monster Drop Updates
```json
{
  "id": 1002,
  "name": "Poring",
  "drops": [
    {"item_id": 909, "name": "Jellopy", "type": "material", "quantity": 1, "chance": 70.0},
    {"item_id": 910, "name": "Sticky Mucus", "type": "material", "quantity": 1, "chance": 30.0},
    {"item_id": 911, "name": "Poring Essence", "type": "material", "quantity": 1, "chance": 5.0},
    {"item_id": 4001, "name": "Poring Card", "type": "card", "quantity": 1, "chance": 0.2, "allowed_slots": ["Accessory"]}
  ]
}
```

## 10. UI/UX Considerations

### Equipment Page Layout
```
┌─────────────────────────┐
│     EQUIPMENT           │
├─────────────────────────┤
│ [Weapon] │ [Armor]      │
│  Swift   │  Padded      │
│  Knife   │  Shirt       │
├──────────┼──────────────┤
│ [Shoes]  │ [Garment]    │
│  Light   │  Wind        │
│  Boots   │  Cape        │
├──────────┼──────────────┤
│ [Ring 1] │ [Ring 2]     │
│  Lucky   │  Power       │
│  Ring    │  Ring        │
├─────────────────────────┤
│ Total Stats:            │
│ ATK: 35  DEF: 18        │
│ HP: +220 AGI: +7        │
├─────────────────────────┤
│ [Preset 1] [2] [3]      │
│ [Save] [Load] [Craft]   │
└─────────────────────────┘
```

### Crafting UI at Blacksmith
```
┌─────────────────────────┐
│   BLACKSMITH - T1       │
├─────────────────────────┤
│ Select Category:        │
│ [Weapon] [Armor] [All]  │
├─────────────────────────┤
│ Swift Knife             │
│ Materials:              │
│ - Jellopy x10 (15/10)✓  │
│ - Bee Sting x5 (3/5)✗   │
│ Cost: 500z (800z)✓      │
│                         │
│ [Craft] (disabled)      │
├─────────────────────────┤
│ Iron Blade              │
│ Materials:              │
│ - Bee Sting x8 (3/8)✗   │
│ - Fluff x5 (12/5)✓      │
│ Cost: 500z (800z)✓      │
│                         │
│ [Craft] (disabled)      │
└─────────────────────────┘
```

## Success Metrics
- Players spend 30%+ time farming specific monsters for materials
- Average player crafts 3+ pieces of equipment per tier
- Card collection becomes long-term goal (collect all cards achievement)
- Build diversity: 40% AGI builds, 40% STR/VIT builds, 20% hybrid
- Preset system used by 80%+ of players above level 20