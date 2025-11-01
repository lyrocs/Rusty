# IDLE Game Transformation Plan

## Executive Summary
Transform the current tamagotchi game into an IDLE-style game where heroes automatically farm monsters in the background, accumulating resources and experience continuously while allowing players to navigate through other game screens.

### Key Modifications from Current System
- **Map System**: Keep existing map structure, only replace 3 difficulty buttons (Easy/Normal/Hard) with single "Start Farming" button
- **Monster Info**: Add difficulty indicators and farming estimates to each map
- **Inventory System**: Replace Quest button with Inventory button on overview screen
- **Background Farming**: Combat continues while navigating to other screens
- **Risk Management**: Heroes take damage but auto-regenerate HP; death ends session with cooldown

---

## 1. Core Gameplay Loop

### 1.1 Auto-Farming System
- **Continuous Combat**: Hero automatically engages monsters in selected map
- **Background Processing**: Combat continues even when viewing other screens
- **Kill Rate Calculation**: Monsters killed per minute based on:
  - Hero's attack power
  - Hero's attack speed
  - Hero's critical rate
  - Monster's defense
  - Monster's HP
  - Equipment bonuses
  - Skill modifiers (if applicable)

### 1.2 Resource Generation
- **Zeny Generation**: Based on monster's base drop rate
- **Item Drops**:
  - Common items (high frequency)
  - Rare items (low frequency)
  - Equipment (very low frequency)
- **Experience Points**: Continuous XP gain based on monsters killed

### 1.3 Risk/Reward Balance
- **Damage System**:
  - Hero takes damage from monsters
  - Damage rate calculated per minute
  - Higher level maps = more damage taken
- **Auto-Regeneration**:
  - Base HP regen per minute
  - Can be enhanced with equipment/skills
  - Food items provide regen buffs
- **Death Mechanics**:
  - Hero dies when HP reaches 0
  - Session ends immediately
  - All accumulated rewards are kept
  - 1-minute cooldown before next session
  - Manual restart required

---

## 2. User Interface Changes

### 2.1 Map Selection Screen (Modified from Current)
The existing map selection screen remains mostly unchanged, with these modifications:
- Replace the 3 battle buttons (Easy/Normal/Hard) with a single **[START FARMING]** button
- Add monster difficulty indicators and farming statistics to each map
- Keep the existing map navigation system

```
┌─────────────────────────────────┐
│ PORING FIELD                    │
├─────────────────────────────────┤
│ [Map Image/Preview]             │
│                                 │
│ Monsters:                       │
│  • Poring (Lv 1) - Difficulty ★ │
│  • Drops (Lv 2) - Difficulty ★  │
│                                 │
│ Farming Estimates:              │
│  Kills/min: ~45                 │
│  Zeny/min: ~120                 │
│  Damage/min: ~5 HP              │
│  Recommended Level: 1-10        │
│                                 │
│ Drop Highlights:                │
│  Common: Jellopy, Apple         │
│  Rare: Poring Card (0.1%)       │
│                                 │
│ [START FARMING]                 │
│                                 │
│ [← Previous Map] [Next Map →]   │
└─────────────────────────────────┘
```

### 2.2 Hero Overview (Main Screen - Modified)
The overview screen is updated to show farming status and replace Quest button with Inventory:

```
┌─────────────────────────────────┐
│ HERO OVERVIEW                   │
├─────────────────────────────────┤
│ [Hero Sprite]                   │
│ Name: Hero123                   │
│ Level: 15 (3,450/10,000 XP)     │
│                                 │
│ === Farming Status ===          │
│ Status: [ACTIVE] ⚔️              │
│ Map: Poring Field               │
│ Time: 12:34 (ongoing)           │
│ HP: [████████░░] 80/100         │
│ Regen: +5/min | Damage: -3/min  │
│                                 │
│ === Current Session ===         │
│ Monsters Killed: 234            │
│ Zeny Earned: 1,456              │
│ Items Found: 12                 │
│                                 │
│ [INVENTORY] [STATS] [MAP]       │
│ [STOP FARMING]                  │
└─────────────────────────────────┘
```

When not farming:
```
┌─────────────────────────────────┐
│ HERO OVERVIEW                   │
├─────────────────────────────────┤
│ [Hero Sprite]                   │
│ Name: Hero123                   │
│ Level: 15 (3,450/10,000 XP)     │
│                                 │
│ HP: [██████████] 100/100        │
│ Base Regen: +5 HP/min           │
│                                 │
│ === Statistics ===              │
│ Total Monsters Killed: 5,234    │
│ Total Zeny Earned: 45,678       │
│ Total Deaths: 3                 │
│ Favorite Map: Poring Field      │
│                                 │
│ [INVENTORY] [STATS] [MAP]       │
│ [START FARMING]                 │
└─────────────────────────────────┘
```

### 2.3 Inventory Screen (New)
**Note**: Item icons available at `/assets/images/items/{item_id}.gif` (e.g., Jellopy ID 909 → `909.gif`)

```
┌─────────────────────────────────┐
│ INVENTORY                       │
├─────────────────────────────────┤
│ Zeny: 45,678                    │
│ Capacity: 42/100                │
│                                 │
│ === Equipment ===               │
│ Weapon: [Icon][Novice Sword]    │
│ Armor: [Icon][Cotton Shirt]     │
│ Accessory: [Empty]              │
│                                 │
│ === Consumables ===             │
│ [🧪] Red Potion x15             │
│ [🍎] Apple x8                   │
│ [🥩] Meat x3                    │
│                                 │
│ === Materials ===               │
│ [909.gif] Jellopy x234          │
│ [938.gif] Sticky Mucus x45      │
│ [705.gif] Clover x12            │
│                                 │
│ === Cards ===                   │
│ [4001.gif] Poring Card x1       │
│                                 │
│ [USE] [EQUIP] [SELL] [BACK]     │
└─────────────────────────────────┘
```

**Icon Display**:
- Each item shows its icon from `/assets/images/items/`
- Icon filenames match item IDs (e.g., ID 909 = 909.gif)
- Format: GIF files
- Display size: 24x24 or 32x32 pixels (to be determined)

### 2.4 Session Results Screen (After Death/Stop)
```
┌─────────────────────────────────┐
│ SESSION COMPLETE                │
├─────────────────────────────────┤
│ Result: [HERO DIED] 💀          │
│ Duration: 45:23                 │
│                                 │
│ === Final Statistics ===        │
│ Total Kills: 2,045              │
│ Total Zeny: +12,456             │
│ Total XP: +34,500               │
│                                 │
│ === Items Obtained ===          │
│ [909.gif] Jellopy x45           │
│ [501.gif] Red Potion x12        │
│ [4001.gif] Poring Card x1 ⭐    │
│                                 │
│ Cooldown: 0:59 remaining        │
│                                 │
│ [CLOSE]                         │
└─────────────────────────────────┘
```

### 2.5 Navigation Bar Update
- Add status indicator showing if farming is active
- Show mini HP bar during active sessions
- Quick access to stop farming from any screen

---

## 3. Technical Architecture

### 3.1 Existing Assets
The project already includes item icon assets:
- **Location**: `/assets/images/items/`
- **Format**: GIF files
- **Naming Convention**: `{item_id}.gif` (e.g., Jellopy with ID 909 is stored as `909.gif`)
- **Usage**: These icons will be used in inventory UI, session results, and item notifications
- **Examples**:
  - 909.gif - Jellopy
  - 501.gif - Red Potion
  - 4001.gif - Poring Card

### 3.2 State Management
```rust
struct FarmingSession {
    is_active: bool,
    map_id: MapId,
    start_time: Instant,
    last_update: Instant,
    monsters_killed: u32,
    zeny_earned: u32,
    items_collected: Vec<ItemDrop>,
    xp_gained: u32,
    current_hp: i32,
    max_hp: i32,
}

struct FarmingCalculator {
    kills_per_minute: f32,
    zeny_per_minute: f32,
    damage_per_minute: f32,
    regen_per_minute: f32,
    drop_rates: HashMap<ItemId, f32>,
}

struct Inventory {
    capacity: u32,
    used_slots: u32,
    items: HashMap<ItemId, ItemStack>,
    equipped: EquippedItems,
    auto_sell_filters: Vec<ItemFilter>,
}

struct ItemStack {
    item_id: ItemId,
    quantity: u32,
    max_stack: u32,
    item_type: ItemType,
    is_locked: bool,
}

enum ItemType {
    Equipment,
    Consumable,
    Material,
    Card,
}

struct ItemDefinition {
    id: u32,
    name: String,
    description: String,
    item_type: ItemType,
    max_stack: u32,
    sell_price: u32,
    // Icon path: /assets/images/items/{id}.gif
    // e.g., ID 909 → /assets/images/items/909.gif
}

// Helper function to get icon path
fn get_item_icon_path(item_id: u32) -> String {
    format!("/assets/images/items/{}.gif", item_id)
}
```

### 3.3 Background Processing Strategy

#### Option A: Real-time Ticker (Recommended)
- Use a separate thread/task that updates every second
- Calculate incremental progress
- Update state atomically
- Pros: Real-time feel, accurate calculations
- Cons: More resource intensive

#### Option B: Lazy Evaluation
- Calculate progress only when UI needs update
- Use timestamp differences
- Pros: Lower resource usage
- Cons: Less responsive, potential calculation spikes

### 3.4 Core Systems to Modify

#### Combat System
- Remove turn-based combat
- Implement DPS (damage per second) calculations
- Add continuous damage/regen mechanics

#### Inventory System
- **Storage Management**:
  - Item capacity limit (e.g., 100 slots)
  - Stack limits for different item types
  - Weight system (optional)
- **Item Categories**:
  - Equipment (weapons, armor, accessories)
  - Consumables (potions, food)
  - Materials (crafting/selling items)
  - Cards (rare drops with special effects)
- **Auto-Collection**: Items automatically collected during farming
- **Item Actions**:
  - Use: Consume items for effects
  - Equip/Unequip: Manage hero equipment
  - Sell: Convert items to zeny
  - Drop: Remove items (with confirmation)
- **Quality of Life Features**:
  - Auto-sell filters for common materials
  - Sort by type/value/quantity
  - Quick-sell all materials button
  - Favorite/lock important items

#### Map System (Minimal Changes)
- **Keep Existing Structure**: Maintain current map navigation system
- **UI Modifications**:
  - Replace 3 difficulty buttons with single "Start Farming" button
  - Add monster difficulty indicators (★ rating system)
  - Display farming estimates (kills/min, zeny/min, damage/min)
- **New Data to Add**:
  - Monster difficulty ratings
  - Farming efficiency calculations
  - Drop rate percentages
  - Recommended level ranges

#### Hero Stats
- Add auto-regen stat
- Balance attack speed importance
- Implement farming efficiency stats

---

## 4. Game Balance Considerations

### 4.1 Progression Curve
- **Early Game**:
  - Low damage maps with guaranteed profit
  - Fast kill rates for dopamine hits
  - Frequent common drops

- **Mid Game**:
  - Risk/reward choices become important
  - Need to balance HP regen vs damage
  - Equipment becomes crucial

- **Late Game**:
  - High-risk maps with rare drops
  - Death becomes a real threat
  - Min-maxing stats for efficiency

### 4.2 Economy Balance
- Prevent inflation through money sinks
- Balance item drop rates
- Consider daily limits or diminishing returns

### 4.3 Player Engagement
- Daily quests/challenges
- Milestone rewards (1000 kills, etc.)
- Rare monster spawn events
- Boss appearances (instant combat?)

---

## 5. Implementation Phases

### Phase 1: Core IDLE System & UI Updates
1. Modify map UI (replace 3 buttons with 1 "Start Farming" button)
2. Add monster difficulty indicators to maps
3. Implement background farming state management
4. Create kill/reward/damage calculations
5. Add HP and regen mechanics
6. Update hero overview to show farming status

### Phase 2: Inventory System
1. Replace Quest button with Inventory button on overview
2. Create inventory UI screen with item icon display
3. Implement icon loading from `/assets/images/items/{id}.gif`
4. Implement item storage and capacity management
5. Add item categories (equipment, consumables, materials, cards)
6. Implement item actions (use, equip, sell)
7. Add auto-collection during farming
8. Create item database with IDs matching icon filenames

### Phase 3: Farming Mechanics & Balance
1. Add farming estimates to each map
2. Implement drop tables and rates
3. Balance kill rates vs damage taken
4. Add death mechanics and cooldown system
5. Test and adjust risk/reward ratios

### Phase 4: Quality of Life & Polish
1. Add auto-sell filters for materials
2. Implement inventory sorting options
3. Add farming session statistics
4. Create notification system for rare drops
5. Add visual feedback (damage numbers, item drops)

### Phase 5: Advanced Features (Future)
1. Skill system for farming bonuses
2. Pet companions that help farming
3. Limited offline progress
4. Events and special farming zones

---

## 6. Data Structures

### 6.1 Map Configuration
```yaml
maps:
  poring_field:
    name: "Poring Field"
    difficulty: 1
    monsters:
      - id: poring
        spawn_weight: 70
        hp: 50
        damage: 2
        xp: 10
        zeny: 5-10
      - id: drops
        spawn_weight: 30
        hp: 30
        damage: 1
        xp: 7
        zeny: 3-7
    drop_table:
      common:
        - jellopy: 0.5
        - apple: 0.1
      rare:
        - poring_card: 0.001
```

### 6.2 Hero Farming Stats
```yaml
farming_stats:
  base_attack_speed: 1.0  # attacks per second
  base_hit_rate: 0.95
  base_crit_rate: 0.05
  base_hp_regen: 2  # per minute

modifiers:
  from_equipment: {}
  from_skills: {}
  from_buffs: {}
```

### 6.3 Item Database Structure
```yaml
items:
  # Materials (IDs in 900s)
  909:
    name: "Jellopy"
    type: Material
    max_stack: 999
    sell_price: 3
    icon: "/assets/images/items/909.gif"

  938:
    name: "Sticky Mucus"
    type: Material
    max_stack: 999
    sell_price: 8
    icon: "/assets/images/items/938.gif"

  705:
    name: "Clover"
    type: Material
    max_stack: 999
    sell_price: 5
    icon: "/assets/images/items/705.gif"

  # Consumables (IDs in 500s)
  501:
    name: "Red Potion"
    type: Consumable
    max_stack: 100
    sell_price: 25
    effect: "Restore 45-65 HP"
    icon: "/assets/images/items/501.gif"

  512:
    name: "Apple"
    type: Consumable
    max_stack: 100
    sell_price: 15
    effect: "Restore 16-22 HP"
    icon: "/assets/images/items/512.gif"

  517:
    name: "Meat"
    type: Consumable
    max_stack: 100
    sell_price: 50
    effect: "Restore 70-100 HP"
    icon: "/assets/images/items/517.gif"

  # Cards (IDs in 4000s)
  4001:
    name: "Poring Card"
    type: Card
    max_stack: 1
    sell_price: 1000
    effect: "LUK +2, MDEF +5"
    icon: "/assets/images/items/4001.gif"
    rarity: "Rare"
```

**Asset Convention**:
- All item icons stored in: `/assets/images/items/`
- Filename format: `{item_id}.gif`
- Icon dimensions: Typically 24x24 pixels
- Format: GIF (supports animation if needed)

---

## 7. UI/UX Considerations

### 7.1 Visual Feedback
- Floating damage numbers
- Gold coin animations
- Item drop notifications
- HP bar animations

### 7.2 User Controls
- One-tap farming start/stop
- Quick map switching
- Auto-retreat at low HP (optional)
- Notification settings

### 7.3 Information Display
- Clear efficiency metrics
- Comparative map analysis
- Session history/statistics
- Personal records tracking

---

## 8. Potential Issues & Solutions

### Issue 1: Battery Drain (Mobile)
**Solution**: Add low-power mode with reduced update frequency

### Issue 2: Exploits/Cheating
**Solution**: Server-side validation for important calculations

### Issue 3: Player Burnout
**Solution**: Daily limits, events, varied content

### Issue 4: Balance Complaints
**Solution**: Regular balance patches, A/B testing

---

## 9. Migration Strategy

### From Current System
1. Preserve existing hero stats
2. Convert current items/currency
3. Provide tutorial for new system
4. Offer "classic mode" temporarily

### Database Changes
- Add farming_sessions table
- Extend hero_stats with farming attributes
- Create map_configurations table
- Add session_history for analytics

---

## 10. Success Metrics

### Player Engagement
- Daily active users
- Average session length
- Retention rates (D1, D7, D30)

### Economy Health
- Currency circulation
- Item distribution
- Market stability

### Technical Performance
- Server load
- Client performance
- Bug report frequency

---

## Discussion Points

1. **Offline Progress**: Should the game continue farming while the app is closed? If yes, with limitations?

2. **Energy System**: Should we add an energy/stamina system to limit daily farming?

3. **PvP Elements**: Could we add competitive elements like farming races or territory control?

4. **Monetization**: Premium features? Ad-based bonuses? Cosmetics only?

5. **Skill System**: Active skills that temporarily boost farming? Passive skill trees?

6. **Events**: Seasonal events? Double XP weekends? Special monster invasions?

7. **Social Features**: Friends list? Farming parties? Guild systems?

8. **Prestige System**: Reset progress for permanent bonuses?

---

## Next Steps

1. Review and refine this design document
2. Decide on specific features for MVP
3. Create detailed technical specifications
4. Build prototype of core farming loop
5. Test and iterate on game balance
6. Implement full system
7. Beta testing and refinement

---

## Appendix

### A. Reference Games
- Ragnarok M: Eternal Love (AFK farming)
- Idle Heroes
- AFK Arena
- MapleStory M (auto-battle)

### B. Key Formulas

**Kills Per Minute:**
```
KPM = (60 / time_to_kill) * hit_rate
time_to_kill = monster_hp / (hero_damage * attack_speed)
```

**Net HP Change:**
```
net_hp_per_min = hp_regen_per_min - (damage_taken_per_hit * hits_per_min)
```

**Session Duration (until death):**
```
max_duration = current_hp / abs(net_hp_per_min)  // if net is negative
```

### C. Technology Stack Considerations
- State management: Use event-driven architecture
- Background tasks: Web Workers (web) or separate threads (native)
- Data persistence: Local storage with cloud sync
- Real-time updates: WebSocket or polling