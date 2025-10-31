// Quest system - handles quest loading, progress tracking, and rewards

use heapless::Vec as HeaplessVec;

use crate::core::GameState;
use crate::quest::models::{ActiveQuest, QuestAction, QuestData, QuestObjective, QuestType};

// Embed quests JSON at compile time
const QUESTS_JSON: &str = include_str!("../../assets/data/quests.json");

// Static storage for parsed quests
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

struct LazyData<T> {
    initialized: AtomicBool,
    data: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for LazyData<T> {}

impl<T> LazyData<T> {
    const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            data: UnsafeCell::new(None),
        }
    }

    fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if !self.initialized.load(Ordering::Acquire) {
            unsafe {
                *self.data.get() = Some(init());
            }
            self.initialized.store(true, Ordering::Release);
        }
        unsafe { (*self.data.get()).as_ref().unwrap() }
    }
}

static QUESTS: LazyData<HeaplessVec<QuestData, 32>> = LazyData::new();

/// Parse quests from JSON (done once, cached)
fn parse_quests() -> HeaplessVec<QuestData, 32> {
    esp_println::println!("[QUEST_SYSTEM] Parsing quests.json...");

    match serde_json_core::from_str::<HeaplessVec<QuestData, 32>>(QUESTS_JSON) {
        Ok((quests, _)) => {
            esp_println::println!("[QUEST_SYSTEM] Successfully parsed {} quests", quests.len());
            for quest in &quests {
                esp_println::println!(
                    "  - {} (ID: {}, Type: {:?}, Lvl: {}-{})",
                    quest.name,
                    quest.id,
                    quest.quest_type,
                    quest.min_level,
                    quest.max_level
                );
            }
            quests
        }
        Err(e) => {
            esp_println::println!("[ERROR] Failed to parse quests.json: {:?}", e);
            HeaplessVec::new()
        }
    }
}

/// Get quest data by ID
pub fn get_quest_data(id: u32) -> Option<&'static QuestData> {
    let quests = QUESTS.get_or_init(parse_quests);
    quests.iter().find(|q| q.id == id)
}

/// Get all quests (for debugging/browsing)
pub fn get_all_quests() -> &'static [QuestData] {
    let quests = QUESTS.get_or_init(parse_quests);
    // SAFETY: Data lives in static storage
    unsafe { core::mem::transmute(quests.as_slice()) }
}

/// Get daily quests applicable for a specific level
pub fn get_daily_quests_for_level(level: u16) -> HeaplessVec<u32, 8> {
    let quests = QUESTS.get_or_init(parse_quests);
    let mut result = HeaplessVec::new();

    for quest in quests.iter() {
        if quest.quest_type == QuestType::Daily
            && quest.min_level <= level
            && (quest.max_level == 0 || level <= quest.max_level)
        {
            result.push(quest.id).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}

/// Get all achievement quests
pub fn get_achievement_quests() -> HeaplessVec<u32, 16> {
    let quests = QUESTS.get_or_init(parse_quests);
    let mut result = HeaplessVec::new();

    for quest in quests.iter() {
        if quest.quest_type == QuestType::Achievement {
            result.push(quest.id).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}

/// Check if quest prerequisites are met
pub fn are_prerequisites_met(game_state: &GameState, quest_id: u32) -> bool {
    if let Some(quest_data) = get_quest_data(quest_id) {
        // If no prerequisites, quest is available
        if quest_data.requires.is_empty() {
            return true;
        }

        // Check if all required quests are completed
        for required_quest_id in quest_data.requires.iter() {
            if !game_state.completed_quest_ids.contains(required_quest_id) {
                esp_println::println!(
                    "[QUEST] Quest {} requires quest {} to be completed first",
                    quest_id,
                    required_quest_id
                );
                return false;
            }
        }

        true
    } else {
        false
    }
}

/// Start a quest (add to active quests)
pub fn start_quest(game_state: &mut GameState, quest_id: u32) -> bool {
    // Check if already active or completed
    if game_state
        .active_quests
        .iter()
        .any(|q| q.quest_id == quest_id)
    {
        esp_println::println!("[QUEST] Quest {} already active", quest_id);
        return false;
    }

    if game_state.completed_quest_ids.contains(&quest_id) {
        esp_println::println!("[QUEST] Quest {} already completed", quest_id);
        return false;
    }

    // Check prerequisites
    if !are_prerequisites_met(game_state, quest_id) {
        esp_println::println!("[QUEST] Prerequisites not met for quest {}", quest_id);
        return false;
    }

    // Get quest data
    if let Some(quest_data) = get_quest_data(quest_id) {
        let active_quest = ActiveQuest::new(
            quest_id,
            quest_data.objectives.len(),
            game_state.last_update_ms,
        );

        if game_state.active_quests.push(active_quest).is_ok() {
            esp_println::println!(
                "[QUEST] Started quest: {} (ID: {})",
                quest_data.name,
                quest_id
            );
            return true;
        } else {
            esp_println::println!("[QUEST] Failed to start quest (active quests full)");
        }
    } else {
        esp_println::println!("[QUEST] Quest {} not found", quest_id);
    }

    false
}

/// Check if a quest objective matches the given action
fn objective_matches(objective: &QuestObjective, action: &QuestAction) -> Option<u16> {
    match (objective.objective_type, action) {
        (
            "KillMonster",
            QuestAction::MonsterKilled {
                enemy_id: killed_id,
            },
        ) => {
            // enemy_id 0 means "any monster"
            if objective.enemy_id == 0 || objective.enemy_id == *killed_id {
                Some(1) // Increment by 1
            } else {
                None
            }
        }
        (
            "CollectItem",
            QuestAction::ItemCollected {
                item_id: collected_id,
                quantity: count,
            },
        ) => {
            if objective.item_id == *collected_id {
                Some(*count)
            } else {
                None
            }
        }
        ("ReachLevel", QuestAction::LevelReached { level: reached }) => {
            if *reached >= objective.level {
                Some(*reached) // Set progress to level reached
            } else {
                None
            }
        }
        ("EarnZeny", QuestAction::ZenyEarned { amount }) => Some(*amount as u16),
        ("RefineEquipment", QuestAction::EquipmentRefined) => Some(1),
        ("CompleteBattles", QuestAction::BattleCompleted) => Some(1),
        _ => None,
    }
}

/// Update quest progress based on an action
pub fn update_quest_progress(game_state: &mut GameState, action: QuestAction) {
    let mut any_updated = false;

    for active_quest in game_state.active_quests.iter_mut() {
        if active_quest.completed || active_quest.claimed {
            continue;
        }

        // Get quest data
        let quest_data = match get_quest_data(active_quest.quest_id) {
            Some(data) => data,
            None => continue,
        };

        // Check each objective
        for (i, objective) in quest_data.objectives.iter().enumerate() {
            if let Some(increment) = objective_matches(objective, &action) {
                // Get target count for this objective
                let target = match objective.objective_type {
                    "KillMonster" => objective.count,
                    "CollectItem" => objective.count,
                    "ReachLevel" => objective.level,
                    "EarnZeny" => objective.amount as u16,
                    "RefineEquipment" => objective.count,
                    "CompleteBattles" => objective.count,
                    _ => 0,
                };

                // Update progress
                let current = active_quest.progress[i];
                let new_progress = (current + increment).min(target);

                if new_progress != current {
                    active_quest.progress[i] = new_progress;
                    any_updated = true;

                    esp_println::println!(
                        "[QUEST] {} progress: {}/{} (objective {})",
                        quest_data.name,
                        new_progress,
                        target,
                        i
                    );
                }
            }
        }

        // Check if quest completed
        if !active_quest.completed {
            let all_complete = check_quest_completion(active_quest, quest_data);
            if all_complete {
                active_quest.completed = true;
                esp_println::println!("[QUEST] Quest completed: {}", quest_data.name);
                any_updated = true;
            }
        }
    }

    if any_updated {
        game_state.needs_redraw = true;
    }
}

/// Check if all objectives of a quest are completed
fn check_quest_completion(active_quest: &ActiveQuest, quest_data: &QuestData) -> bool {
    for (i, objective) in quest_data.objectives.iter().enumerate() {
        let target = match objective.objective_type {
            "KillMonster" => objective.count,
            "CollectItem" => objective.count,
            "ReachLevel" => objective.level,
            "EarnZeny" => objective.amount as u16,
            "RefineEquipment" => objective.count,
            "CompleteBattles" => objective.count,
            _ => 0,
        };

        if active_quest.progress[i] < target {
            return false;
        }
    }

    true
}

/// Claim quest reward
pub fn claim_quest_reward(game_state: &mut GameState, quest_id: u32) -> bool {
    // Find the active quest
    let quest_index = game_state
        .active_quests
        .iter()
        .position(|q| q.quest_id == quest_id);

    if quest_index.is_none() {
        esp_println::println!("[QUEST] Quest {} not found in active quests", quest_id);
        return false;
    }

    let quest_index = quest_index.unwrap();
    let active_quest = &game_state.active_quests[quest_index];

    if !active_quest.completed {
        esp_println::println!("[QUEST] Quest {} not completed yet", quest_id);
        return false;
    }

    if active_quest.claimed {
        esp_println::println!("[QUEST] Quest {} already claimed", quest_id);
        return false;
    }

    // Get quest data
    let quest_data = match get_quest_data(quest_id) {
        Some(data) => data,
        None => {
            esp_println::println!("[QUEST] Quest data {} not found", quest_id);
            return false;
        }
    };

    // Give rewards
    esp_println::println!("[QUEST] Claiming rewards for: {}", quest_data.name);

    game_state.hero.exp += quest_data.rewards.base_exp;
    game_state.hero.zeny += quest_data.rewards.zeny;

    esp_println::println!(
        "  +{} EXP, +{} Zeny",
        quest_data.rewards.base_exp,
        quest_data.rewards.zeny
    );

    // Give items (if any)
    for (item_id, quantity) in &quest_data.rewards.items {
        esp_println::println!("  +{} x Item {}", quantity, item_id);
        // TODO: Add to inventory when inventory system is implemented
    }

    // Check for level up (simple check)
    while game_state.hero.exp >= game_state.hero.exp_to_next_level {
        game_state.hero.exp -= game_state.hero.exp_to_next_level;
        game_state.hero.level += 1;
        game_state.hero.exp_to_next_level = (game_state.hero.level as u32) * 100; // Simple formula
        esp_println::println!("[HERO] Level up! Now level {}", game_state.hero.level);
    }

    // Mark as claimed and add to completed list
    game_state.active_quests[quest_index].claimed = true;
    game_state.completed_quest_ids.push(quest_id).ok();

    // Remove from active quests after a delay (so player can see completion)
    // For now, we'll keep it in active_quests with claimed=true
    // UI will handle filtering

    game_state.needs_redraw = true;
    true
}

/// Check if daily quests need to be refreshed
pub fn should_refresh_daily_quests(game_state: &GameState) -> bool {
    let current_time = game_state.last_update_ms;
    let time_since_refresh = current_time.saturating_sub(game_state.daily_quest_refresh_time);
    time_since_refresh >= 86400000 // 24 hours in milliseconds
}

/// Refresh daily quests
pub fn refresh_daily_quests(game_state: &mut GameState) {
    esp_println::println!("[QUEST] Refreshing daily quests...");

    // Remove old unclaimed daily quests
    game_state.active_quests.retain(|q| {
        if let Some(quest_data) = get_quest_data(q.quest_id) {
            // Keep if not daily, or if daily and claimed
            quest_data.quest_type != QuestType::Daily || q.claimed
        } else {
            false
        }
    });

    // Get new daily quests for hero level
    let daily_quest_ids = get_daily_quests_for_level(game_state.hero.level);

    // Select up to 3 random daily quests
    let mut selected = 0;
    for quest_id in daily_quest_ids.iter() {
        if selected >= 3 {
            break;
        }
        if start_quest(game_state, *quest_id) {
            selected += 1;
        }
    }

    game_state.daily_quest_refresh_time = game_state.last_update_ms;
    esp_println::println!("[QUEST] Started {} daily quests", selected);
}

/// Initialize quest system (auto-start achievements, refresh daily quests)
pub fn initialize_quest_system(game_state: &mut GameState) {
    esp_println::println!("[QUEST] Initializing quest system...");

    // Auto-start all achievement quests
    let achievement_ids = get_achievement_quests();
    let mut started = 0;
    for quest_id in achievement_ids.iter() {
        if start_quest(game_state, *quest_id) {
            started += 1;
        }
    }
    esp_println::println!("[QUEST] Started {} achievement quests", started);

    // Refresh daily quests if needed (or if first time)
    if game_state.daily_quest_refresh_time == 0 || should_refresh_daily_quests(game_state) {
        refresh_daily_quests(game_state);
    }
}
