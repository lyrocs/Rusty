# Quest System Implementation Plan

## Overview
A flexible, data-driven quest system that supports multiple quest types, daily quests based on hero level, and achievement tracking where a single action can progress multiple quests simultaneously.

## Architecture Goals
- **Data-Driven**: Quest definitions in JSON files, loaded at compile time
- **Flexible Objectives**: Support various quest types (kill monsters, collect items, level up, etc.)
- **Progress Tracking**: Single action can update multiple quests
- **Daily Quests**: Level-scaled daily quests that refresh
- **Reward System**: Multiple reward types (EXP, Zeny, Items)
- **UI Integration**: Quest list page and claim rewards interface

---

## Phase 1: Core Data Structures

### Quest Data Model
```rust
// Quest objective types
pub enum QuestObjectiveType {
    KillMonster { enemy_id: u32, count: u16 },
    CollectItem { item_id: u32, count: u16 },
    ReachLevel { level: u16 },
    EarnZeny { amount: u32 },
    RefineEquipment { count: u16 },
    CompleteBattles { count: u16 },
}

// Quest rewards
pub struct QuestReward {
    pub base_exp: u32,
    pub job_exp: u32,
    pub zeny: u32,
    pub items: HeaplessVec<(u32, u16), 4>, // (item_id, quantity)
}

// Quest definition (from JSON)
pub struct QuestData {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub quest_type: QuestType, // Story, Daily, Achievement
    pub min_level: u16,
    pub max_level: u16, // 0 = no max
    pub objectives: HeaplessVec<QuestObjectiveType, 4>,
    pub rewards: QuestReward,
}

// Active quest progress (in GameState)
pub struct ActiveQuest {
    pub quest_id: u32,
    pub started_at: u32, // timestamp
    pub progress: HeaplessVec<u16, 4>, // progress per objective
    pub completed: bool,
    pub claimed: bool,
}

pub enum QuestType {
    Story,      // Manual quests from NPCs
    Daily,      // Auto-assigned daily quests
    Achievement, // Long-term achievements
}
```

### Quest State in GameState
```rust
pub struct GameState {
    // ... existing fields ...

    // Quest system
    pub active_quests: HeaplessVec<ActiveQuest, 16>,
    pub completed_quest_ids: HeaplessVec<u32, 64>, // Track completed quests
    pub daily_quest_refresh_time: u32, // When daily quests refresh
    pub quest_page_scroll: u8, // Scroll position in quest list
}
```

---

## Phase 2: JSON Data Structure

### File: `src/tamagotchi/data/quests.json`
```json
[
  {
    "id": 1,
    "name": "Novice Hunter",
    "description": "Defeat 5 Porings to prove your strength",
    "quest_type": "Daily",
    "min_level": 1,
    "max_level": 10,
    "objectives": [
      {
        "type": "KillMonster",
        "enemy_id": 1002,
        "count": 5
      }
    ],
    "rewards": {
      "base_exp": 100,
      "job_exp": 50,
      "zeny": 500,
      "items": []
    }
  },
  {
    "id": 2,
    "name": "Training Montage",
    "description": "Complete 10 battles",
    "quest_type": "Daily",
    "min_level": 1,
    "max_level": 15,
    "objectives": [
      {
        "type": "CompleteBattles",
        "count": 10
      }
    ],
    "rewards": {
      "base_exp": 150,
      "job_exp": 75,
      "zeny": 750,
      "items": []
    }
  },
  {
    "id": 3,
    "name": "Monster Slayer I",
    "description": "Defeat 100 monsters",
    "quest_type": "Achievement",
    "min_level": 1,
    "max_level": 0,
    "objectives": [
      {
        "type": "KillMonster",
        "enemy_id": 0,
        "count": 100
      }
    ],
    "rewards": {
      "base_exp": 1000,
      "job_exp": 500,
      "zeny": 5000,
      "items": []
    }
  },
  {
    "id": 10,
    "name": "Poring Exterminator",
    "description": "Defeat 50 Porings",
    "quest_type": "Achievement",
    "min_level": 1,
    "max_level": 0,
    "objectives": [
      {
        "type": "KillMonster",
        "enemy_id": 1002,
        "count": 50
      }
    ],
    "rewards": {
      "base_exp": 500,
      "job_exp": 250,
      "zeny": 2500,
      "items": [[909, 10]]
    }
  }
]
```

---

## Phase 3: Quest System Logic

### File: `src/tamagotchi/quest_system.rs`

Core quest system functions:
- `load_quests()` - Parse JSON at compile time
- `get_daily_quests_for_level(level: u16)` - Get applicable daily quests
- `start_quest(quest_id: u32)` - Add quest to active quests
- `update_quest_progress(action: QuestAction)` - Update all matching quests
- `check_quest_completion(quest_id: u32)` - Check if objectives met
- `claim_quest_reward(quest_id: u32)` - Give rewards to hero
- `refresh_daily_quests()` - Reset and reassign daily quests

### Quest Action Events
```rust
pub enum QuestAction {
    MonsterKilled { enemy_id: u32 },
    ItemCollected { item_id: u32, count: u16 },
    LevelReached { level: u16 },
    ZenyEarned { amount: u32 },
    EquipmentRefined,
    BattleCompleted,
}
```

### Update Flow
When a monster is killed:
1. Create `QuestAction::MonsterKilled { enemy_id }`
2. Call `update_quest_progress(action)`
3. System iterates all active quests
4. For each quest, check objectives:
   - `KillMonster { enemy_id: 1002, count: 5 }` matches if enemy_id == 1002 OR enemy_id == 0 (any)
   - Increment progress[objective_index]
5. Check if quest completed, set flag
6. Mark GameState needs_redraw

---

## Phase 4: UI Implementation

### Quest Menu Page (Replace Inventory)

**Layout:**
- Title: "=== QUESTS ==="
- Quest list (scrollable if >4 quests):
  - Quest name
  - Progress bar per objective
  - Status: "In Progress" / "Completed" / "Ready to Claim"
  - Claim button if completed
- Back button

**Quest Card Design:**
```
┌────────────────────────────────┐
│ [Quest Name]                    │
│ [Description]                   │
│ ━━━━━━━━━━━━━━━━ 3/5           │
│ [ CLAIM REWARD ] or [In Progress]│
└────────────────────────────────┘
```

### Quest Details (on click)
- Full description
- All objectives with progress
- Reward preview
- Claim button (if completed)

### Menu Integration
Replace "Inventory" button with "Quests" button in Menu overlay.

---

## Phase 5: Daily Quest System

### Daily Quest Mechanics
- Refresh every 24 hours (based on in-game time or real time)
- Select 3 daily quests based on hero level
- Only quests with min_level <= hero_level <= max_level (or max_level == 0)
- Remove old daily quests on refresh
- Auto-start new daily quests

### Refresh Logic
```rust
fn should_refresh_daily_quests(game_state: &GameState) -> bool {
    let current_time = game_state.last_update_ms;
    let time_since_refresh = current_time - game_state.daily_quest_refresh_time;
    time_since_refresh >= 86400000 // 24 hours in milliseconds
}

fn refresh_daily_quests(game_state: &mut GameState) {
    // Remove old unclaimed daily quests
    game_state.active_quests.retain(|q| {
        let quest_data = get_quest_data(q.quest_id);
        quest_data.quest_type != QuestType::Daily || q.claimed
    });

    // Get new daily quests for hero level
    let daily_quests = get_daily_quests_for_level(game_state.hero.level);
    for quest_id in daily_quests {
        start_quest(game_state, quest_id);
    }

    game_state.daily_quest_refresh_time = game_state.last_update_ms;
}
```

---

## Phase 6: Achievement System

### Achievement Quests
- Always active (auto-started on first login)
- Long-term goals (kill 100 monsters, reach level 50, etc.)
- Can be claimed when completed
- Don't refresh

### Achievement Auto-Start
On game initialization, check all achievement quests:
- If not in completed_quest_ids, start them

---

## Phase 7: Integration Points

### Hook into Existing Systems

**1. Battle Victory** (systems.rs - battle system)
```rust
// After battle victory
quest_system::update_quest_progress(
    &mut game_state,
    QuestAction::MonsterKilled { enemy_id: enemy.id }
);
quest_system::update_quest_progress(
    &mut game_state,
    QuestAction::BattleCompleted
);
```

**2. Farm Victory** (systems.rs - farm system)
```rust
// After farm victory
quest_system::update_quest_progress(
    &mut game_state,
    QuestAction::MonsterKilled { enemy_id: enemy.id }
);
```

**3. Level Up** (models.rs - Hero::gain_exp)
```rust
// After level increase
quest_system::update_quest_progress(
    &mut game_state,
    QuestAction::LevelReached { level: self.level }
);
```

**4. Zeny Earned** (models.rs - Hero::add_zeny or wherever zeny is awarded)
```rust
quest_system::update_quest_progress(
    &mut game_state,
    QuestAction::ZenyEarned { amount }
);
```

**5. Equipment Refined** (systems.rs - refine success)
```rust
quest_system::update_quest_progress(
    &mut game_state,
    QuestAction::EquipmentRefined
);
```

---

## Phase 8: Quest NPC (Future)

### Quest Giver NPC in Prontera
- Add NPC ID 1005 "Quest Board" or "Guild Master"
- Shows available story quests
- Click to accept quest
- Shows quest details before accepting

---

## Implementation Order

### Step 1: Data Structures & JSON
1. Create quest data structures in models.rs
2. Create quests.json with 5-10 sample quests
3. Create quest_system.rs with JSON parsing
4. Add quest state fields to GameState

### Step 2: Core Logic
1. Implement quest loading (parse JSON)
2. Implement start_quest()
3. Implement update_quest_progress() with matching logic
4. Implement check_quest_completion()
5. Implement claim_quest_reward()

### Step 3: Daily Quest System
1. Implement get_daily_quests_for_level()
2. Implement refresh_daily_quests()
3. Hook refresh into game update loop
4. Auto-start daily quests on game load

### Step 4: UI
1. Replace "Inventory" with "Quests" in menu
2. Create Quest List page UI
3. Add quest progress rendering
4. Add claim button and handling
5. Add touch handling for quest list

### Step 5: Integration
1. Hook MonsterKilled into battle victory
2. Hook BattleCompleted into battles
3. Hook LevelReached into Hero::gain_exp
4. Hook EquipmentRefined into refine system
5. Test multi-quest progression

### Step 6: Achievement System
1. Add achievement auto-start on game init
2. Create 5-10 achievement quests
3. Test long-term progression

### Step 7: Polish
1. Add quest notifications (popup when quest completed)
2. Add quest counter on Menu button (show active quest count)
3. Add scroll support for quest list if >4 quests
4. Save/load quest state

---

## Example Quests

### Daily Quests (Level 1-10)
- "Novice Hunter" - Kill 5 Porings
- "Training Montage" - Complete 10 battles
- "Treasure Hunter" - Earn 1000 Zeny

### Daily Quests (Level 11-20)
- "Intermediate Hunter" - Kill 10 Hornets
- "Equipment Enthusiast" - Refine equipment 3 times
- "Experience Seeker" - Complete 15 battles

### Achievement Quests (Any Level)
- "Monster Slayer I" - Kill 100 monsters (any)
- "Monster Slayer II" - Kill 500 monsters
- "Poring Exterminator" - Kill 50 Porings
- "Rich Adventurer I" - Earn 10,000 Zeny total
- "Master Refiner" - Refine equipment to +5
- "Level 20 Hero" - Reach level 20

---

## Technical Considerations

### Memory Usage
- Max 16 active quests (HeaplessVec)
- Max 64 completed quest IDs tracked
- Quest data loaded at compile time (no runtime allocation)

### Performance
- Quest progress update: O(active_quests * objectives_per_quest)
- Should be fast enough for embedded (< 16 quests * 4 objectives = 64 checks)

### Save/Load
- Save active_quests state
- Save completed_quest_ids
- Save daily_quest_refresh_time
- Quest definitions always loaded from JSON (no need to save)

### Extensibility
- Easy to add new objective types
- Easy to add new reward types
- JSON-based, no code changes for new quests
- Achievement system can track anything

---

## Success Criteria

✅ Player can view active quests in Quest menu
✅ Quest progress updates automatically when objectives met
✅ Single action (kill monster) updates multiple quests
✅ Daily quests refresh every 24 hours
✅ Achievement quests track long-term progress
✅ Rewards can be claimed when quest completes
✅ Quest system is data-driven (JSON-based)
✅ System is extensible for future quest types

---

## Future Enhancements (Post-MVP)

1. **Quest Chains** - Quest completion unlocks next quest
2. **Quest Requirements** - Require completing quest X before starting quest Y
3. **Repeatable Quests** - Weekly/Monthly quests
4. **Hidden Quests** - Achievement-style secret quests
5. **Quest Notifications** - Popup when quest progresses/completes
6. **Quest Sorting** - Sort by status, type, level
7. **Quest Search/Filter** - Filter by type or status
8. **Quest History** - View all completed quests
9. **Quest Rewards Preview** - Show rewards before starting quest
10. **Story Quest NPCs** - Specific NPCs give specific quests
