//! Quest System
//!
//! Handles quest definitions, progress tracking, and rewards.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quest type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestType {
    Daily,
    Weekly,
    Achievement,
    Story,
    Event,
}

impl QuestType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Daily" => Some(QuestType::Daily),
            "Weekly" => Some(QuestType::Weekly),
            "Achievement" => Some(QuestType::Achievement),
            "Story" => Some(QuestType::Story),
            "Event" => Some(QuestType::Event),
            _ => None,
        }
    }
}

/// Quest status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    /// Quest is locked (prerequisites not met)
    Locked,
    /// Quest is available to start
    Available,
    /// Quest is in progress
    InProgress,
    /// Quest objectives completed, rewards not claimed
    Completed,
    /// Rewards claimed (for non-repeatable quests)
    Claimed,
}

/// Quest objective types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "objective_type")]
pub enum QuestObjective {
    /// Kill any monsters
    KillAny {
        target_count: u32,
    },
    /// Kill specific monster type
    KillSpecific {
        monster_id: u32,
        target_count: u32,
    },
    /// Win battles
    WinBattles {
        target_count: u32,
    },
    /// Collect fragments (any type)
    CollectFragments {
        target_count: u32,
    },
    /// Collect specific monster fragments
    CollectSpecificFragments {
        monster_id: u32,
        target_count: u32,
    },
    /// Reach a level with any Rustymon
    ReachLevel {
        target_level: u32,
    },
    /// Summon a Rustymon
    SummonRustymon {
        target_count: u32,
    },
    /// Explore maps
    ExploreMaps {
        target_count: u32,
    },
}

impl QuestObjective {
    /// Get the target count for this objective
    pub fn target_count(&self) -> u32 {
        match self {
            QuestObjective::KillAny { target_count } => *target_count,
            QuestObjective::KillSpecific { target_count, .. } => *target_count,
            QuestObjective::WinBattles { target_count } => *target_count,
            QuestObjective::CollectFragments { target_count } => *target_count,
            QuestObjective::CollectSpecificFragments { target_count, .. } => *target_count,
            QuestObjective::ReachLevel { target_level } => *target_level,
            QuestObjective::SummonRustymon { target_count } => *target_count,
            QuestObjective::ExploreMaps { target_count } => *target_count,
        }
    }

    /// Get description of the objective
    pub fn description(&self) -> String {
        match self {
            QuestObjective::KillAny { target_count } => {
                format!("Defeat {} monsters", target_count)
            }
            QuestObjective::KillSpecific {
                monster_id,
                target_count,
            } => format!("Defeat {} of monster #{}", target_count, monster_id),
            QuestObjective::WinBattles { target_count } => {
                format!("Win {} battles", target_count)
            }
            QuestObjective::CollectFragments { target_count } => {
                format!("Collect {} fragments", target_count)
            }
            QuestObjective::CollectSpecificFragments {
                monster_id,
                target_count,
            } => format!("Collect {} fragments of monster #{}", target_count, monster_id),
            QuestObjective::ReachLevel { target_level } => {
                format!("Reach level {}", target_level)
            }
            QuestObjective::SummonRustymon { target_count } => {
                format!("Summon {} Rustymon", target_count)
            }
            QuestObjective::ExploreMaps { target_count } => {
                format!("Explore {} maps", target_count)
            }
        }
    }
}

/// Fragment reward structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentReward {
    pub monster_id: u32,
    pub amount: u32,
}

/// Quest rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestRewards {
    pub exp: u64,
    #[serde(default)]
    pub fragments: Vec<FragmentReward>,
}

/// Quest definition loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestData {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub quest_type: String,
    pub category: String,
    pub level_requirement: u32,
    pub max_level: u32,
    pub objectives: Vec<QuestObjective>,
    pub rewards: QuestRewards,
    #[serde(default)]
    pub prerequisites: Vec<u32>,
    pub repeatable: bool,
}

impl QuestData {
    /// Get the quest type as enum
    pub fn get_quest_type(&self) -> QuestType {
        QuestType::from_str(&self.quest_type).unwrap_or(QuestType::Achievement)
    }

    /// Check if quest is daily
    pub fn is_daily(&self) -> bool {
        self.get_quest_type() == QuestType::Daily
    }

    /// Check if quest is weekly
    pub fn is_weekly(&self) -> bool {
        self.get_quest_type() == QuestType::Weekly
    }
}

/// Quest progress for a single objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveProgress {
    pub current: u32,
    pub target: u32,
}

impl ObjectiveProgress {
    pub fn new(target: u32) -> Self {
        Self { current: 0, target }
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.target
    }

    pub fn add_progress(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.target);
    }

    pub fn set_progress(&mut self, value: u32) {
        self.current = value.min(self.target);
    }

    pub fn percentage(&self) -> f32 {
        if self.target == 0 {
            100.0
        } else {
            (self.current as f32 / self.target as f32) * 100.0
        }
    }
}

/// Active quest with progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuest {
    pub quest_id: u32,
    pub status: QuestStatus,
    pub progress: Vec<ObjectiveProgress>,
    pub started_at: u64,
    pub completed_at: Option<u64>,
}

impl ActiveQuest {
    /// Create a new active quest from quest data
    pub fn new(quest_data: &QuestData) -> Self {
        let progress = quest_data
            .objectives
            .iter()
            .map(|obj| ObjectiveProgress::new(obj.target_count()))
            .collect();

        Self {
            quest_id: quest_data.id,
            status: QuestStatus::InProgress,
            progress,
            started_at: Self::current_timestamp(),
            completed_at: None,
        }
    }

    /// Check if all objectives are complete
    pub fn is_complete(&self) -> bool {
        self.progress.iter().all(|p| p.is_complete())
    }

    /// Update status based on progress
    pub fn update_status(&mut self) {
        if self.is_complete() && self.status == QuestStatus::InProgress {
            self.status = QuestStatus::Completed;
            self.completed_at = Some(Self::current_timestamp());
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Quest event for progress tracking
#[derive(Debug, Clone)]
pub enum QuestEvent {
    /// Monster was killed
    MonsterKilled { monster_id: u32 },
    /// Battle was won
    BattleWon,
    /// Fragment was collected
    FragmentCollected { monster_id: u32, amount: u32 },
    /// Level was reached
    LevelReached { level: u32 },
    /// Rustymon was summoned
    RustymonSummoned { monster_id: u32 },
    /// Map was explored
    MapExplored { map_id: u32 },
}

/// Quest manager handles all quest logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestManager {
    /// All active quests (quest_id -> ActiveQuest)
    pub active_quests: HashMap<u32, ActiveQuest>,
    /// Completed quest IDs (for non-repeatable quests)
    pub completed_quest_ids: Vec<u32>,
    /// Last daily reset timestamp
    pub last_daily_reset: u64,
    /// Last weekly reset timestamp
    pub last_weekly_reset: u64,
    /// Total kills for tracking (for daily resets)
    pub session_kills: HashMap<u32, u32>,
    /// Total battles won this session
    pub session_battles_won: u32,
    /// Total fragments collected this session
    pub session_fragments_collected: u32,
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestManager {
    /// Create a new quest manager
    pub fn new() -> Self {
        Self {
            active_quests: HashMap::new(),
            completed_quest_ids: Vec::new(),
            last_daily_reset: Self::current_timestamp(),
            last_weekly_reset: Self::current_timestamp(),
            session_kills: HashMap::new(),
            session_battles_won: 0,
            session_fragments_collected: 0,
        }
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Check if daily reset is needed (24 hours passed)
    pub fn should_reset_daily(&self) -> bool {
        let now = Self::current_timestamp();
        let day_seconds = 24 * 60 * 60;
        now - self.last_daily_reset >= day_seconds
    }

    /// Check if weekly reset is needed (7 days passed)
    pub fn should_reset_weekly(&self) -> bool {
        let now = Self::current_timestamp();
        let week_seconds = 7 * 24 * 60 * 60;
        now - self.last_weekly_reset >= week_seconds
    }

    /// Reset daily quests
    pub fn reset_daily_quests(&mut self, quest_data: &HashMap<u32, QuestData>) {
        log::info!("Resetting daily quests");

        // Remove all daily quests from active quests
        self.active_quests.retain(|_, quest| {
            if let Some(data) = quest_data.get(&quest.quest_id) {
                !data.is_daily()
            } else {
                true
            }
        });

        // Reset session tracking
        self.session_kills.clear();
        self.session_battles_won = 0;
        self.session_fragments_collected = 0;

        // Update reset timestamp
        self.last_daily_reset = Self::current_timestamp();
    }

    /// Reset weekly quests
    pub fn reset_weekly_quests(&mut self, quest_data: &HashMap<u32, QuestData>) {
        log::info!("Resetting weekly quests");

        // Remove all weekly quests from active quests
        self.active_quests.retain(|_, quest| {
            if let Some(data) = quest_data.get(&quest.quest_id) {
                !data.is_weekly()
            } else {
                true
            }
        });

        // Update reset timestamp
        self.last_weekly_reset = Self::current_timestamp();
    }

    /// Start a quest
    pub fn start_quest(&mut self, quest_data: &QuestData) -> bool {
        // Check if quest is already active
        if self.active_quests.contains_key(&quest_data.id) {
            return false;
        }

        // Check if non-repeatable quest was already completed
        if !quest_data.repeatable && self.completed_quest_ids.contains(&quest_data.id) {
            return false;
        }

        // Create and add active quest
        let active_quest = ActiveQuest::new(quest_data);
        self.active_quests.insert(quest_data.id, active_quest);

        log::info!("Started quest: {} (ID: {})", quest_data.name, quest_data.id);
        true
    }

    /// Process a quest event and update progress
    pub fn process_event(&mut self, event: &QuestEvent, quest_data: &HashMap<u32, QuestData>) {
        // Update session tracking
        match event {
            QuestEvent::MonsterKilled { monster_id } => {
                *self.session_kills.entry(*monster_id).or_insert(0) += 1;
            }
            QuestEvent::BattleWon => {
                self.session_battles_won += 1;
            }
            QuestEvent::FragmentCollected { amount, .. } => {
                self.session_fragments_collected += amount;
            }
            _ => {}
        }

        // Update all active quests
        let quest_ids: Vec<u32> = self.active_quests.keys().copied().collect();
        for quest_id in quest_ids {
            if let Some(data) = quest_data.get(&quest_id) {
                if let Some(active_quest) = self.active_quests.get_mut(&quest_id) {
                    Self::update_quest_progress(active_quest, data, event);
                }
            }
        }
    }

    /// Update progress for a single quest based on event
    fn update_quest_progress(
        active_quest: &mut ActiveQuest,
        quest_data: &QuestData,
        event: &QuestEvent,
    ) {
        if active_quest.status != QuestStatus::InProgress {
            return;
        }

        for (i, objective) in quest_data.objectives.iter().enumerate() {
            if i >= active_quest.progress.len() {
                continue;
            }

            let progress = &mut active_quest.progress[i];

            match (objective, event) {
                (QuestObjective::KillAny { .. }, QuestEvent::MonsterKilled { .. }) => {
                    progress.add_progress(1);
                }
                (
                    QuestObjective::KillSpecific {
                        monster_id: target_id,
                        ..
                    },
                    QuestEvent::MonsterKilled { monster_id },
                ) => {
                    if target_id == monster_id {
                        progress.add_progress(1);
                    }
                }
                (QuestObjective::WinBattles { .. }, QuestEvent::BattleWon) => {
                    progress.add_progress(1);
                }
                (
                    QuestObjective::CollectFragments { .. },
                    QuestEvent::FragmentCollected { amount, .. },
                ) => {
                    progress.add_progress(*amount);
                }
                (
                    QuestObjective::CollectSpecificFragments {
                        monster_id: target_id,
                        ..
                    },
                    QuestEvent::FragmentCollected { monster_id, amount },
                ) => {
                    if target_id == monster_id {
                        progress.add_progress(*amount);
                    }
                }
                (
                    QuestObjective::ReachLevel { target_level },
                    QuestEvent::LevelReached { level },
                ) => {
                    if *level >= *target_level {
                        progress.set_progress(*target_level);
                    }
                }
                (QuestObjective::SummonRustymon { .. }, QuestEvent::RustymonSummoned { .. }) => {
                    progress.add_progress(1);
                }
                (QuestObjective::ExploreMaps { .. }, QuestEvent::MapExplored { .. }) => {
                    progress.add_progress(1);
                }
                _ => {}
            }
        }

        // Update quest status
        active_quest.update_status();
    }

    /// Claim rewards for a completed quest
    pub fn claim_rewards(
        &mut self,
        quest_id: u32,
        quest_data: &QuestData,
    ) -> Option<QuestRewards> {
        let active_quest = self.active_quests.get_mut(&quest_id)?;

        if active_quest.status != QuestStatus::Completed {
            return None;
        }

        // Mark as claimed
        active_quest.status = QuestStatus::Claimed;

        // For non-repeatable quests, add to completed list
        if !quest_data.repeatable {
            if !self.completed_quest_ids.contains(&quest_id) {
                self.completed_quest_ids.push(quest_id);
            }
        }

        // Remove from active quests
        self.active_quests.remove(&quest_id);

        log::info!(
            "Claimed rewards for quest: {} (ID: {})",
            quest_data.name,
            quest_id
        );

        Some(quest_data.rewards.clone())
    }

    /// Get all active quests
    pub fn get_active_quests(&self) -> Vec<&ActiveQuest> {
        self.active_quests.values().collect()
    }

    /// Get quests available for a given player level
    pub fn get_available_quests<'a>(
        &self,
        quest_data: &'a HashMap<u32, QuestData>,
        player_level: u32,
    ) -> Vec<&'a QuestData> {
        quest_data
            .values()
            .filter(|quest| {
                // Check level requirements
                player_level >= quest.level_requirement && player_level <= quest.max_level
            })
            .filter(|quest| {
                // Check if not already active
                !self.active_quests.contains_key(&quest.id)
            })
            .filter(|quest| {
                // Check if not already completed (for non-repeatable)
                quest.repeatable || !self.completed_quest_ids.contains(&quest.id)
            })
            .filter(|quest| {
                // Check prerequisites
                quest
                    .prerequisites
                    .iter()
                    .all(|prereq| self.completed_quest_ids.contains(prereq))
            })
            .collect()
    }

    /// Get completed quests that can be claimed
    pub fn get_claimable_quests(&self) -> Vec<u32> {
        self.active_quests
            .iter()
            .filter(|(_, quest)| quest.status == QuestStatus::Completed)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Check if a quest is active
    pub fn is_quest_active(&self, quest_id: u32) -> bool {
        self.active_quests.contains_key(&quest_id)
    }

    /// Get quest progress if active
    pub fn get_quest_progress(&self, quest_id: u32) -> Option<&ActiveQuest> {
        self.active_quests.get(&quest_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quest() -> QuestData {
        QuestData {
            id: 1,
            name: "Test Quest".to_string(),
            description: "Kill 5 monsters".to_string(),
            quest_type: "Daily".to_string(),
            category: "combat".to_string(),
            level_requirement: 1,
            max_level: 99,
            objectives: vec![QuestObjective::KillAny { target_count: 5 }],
            rewards: QuestRewards {
                exp: 100,
                fragments: vec![],
            },
            prerequisites: vec![],
            repeatable: true,
        }
    }

    #[test]
    fn test_quest_progress() {
        let quest = create_test_quest();
        let mut manager = QuestManager::new();
        let mut quest_data = HashMap::new();
        quest_data.insert(quest.id, quest.clone());

        // Start quest
        assert!(manager.start_quest(&quest));
        assert!(manager.is_quest_active(1));

        // Process events
        for _ in 0..5 {
            manager.process_event(&QuestEvent::MonsterKilled { monster_id: 1002 }, &quest_data);
        }

        // Check completion
        let active = manager.get_quest_progress(1).unwrap();
        assert!(active.is_complete());
        assert_eq!(active.status, QuestStatus::Completed);
    }

    #[test]
    fn test_claim_rewards() {
        let quest = create_test_quest();
        let mut manager = QuestManager::new();
        let mut quest_data = HashMap::new();
        quest_data.insert(quest.id, quest.clone());

        manager.start_quest(&quest);

        // Complete the quest
        for _ in 0..5 {
            manager.process_event(&QuestEvent::MonsterKilled { monster_id: 1002 }, &quest_data);
        }

        // Claim rewards
        let rewards = manager.claim_rewards(1, &quest).unwrap();
        assert_eq!(rewards.exp, 100);
        assert!(!manager.is_quest_active(1));
    }

    #[test]
    fn test_specific_monster_kill() {
        let quest = QuestData {
            id: 2,
            name: "Poring Hunt".to_string(),
            description: "Kill 3 Porings".to_string(),
            quest_type: "Daily".to_string(),
            category: "hunting".to_string(),
            level_requirement: 1,
            max_level: 99,
            objectives: vec![QuestObjective::KillSpecific {
                monster_id: 1002,
                target_count: 3,
            }],
            rewards: QuestRewards {
                exp: 50,
                fragments: vec![FragmentReward {
                    monster_id: 1002,
                    amount: 1,
                }],
            },
            prerequisites: vec![],
            repeatable: true,
        };

        let mut manager = QuestManager::new();
        let mut quest_data = HashMap::new();
        quest_data.insert(quest.id, quest.clone());

        manager.start_quest(&quest);

        // Kill wrong monster - no progress
        manager.process_event(&QuestEvent::MonsterKilled { monster_id: 1007 }, &quest_data);
        let active = manager.get_quest_progress(2).unwrap();
        assert_eq!(active.progress[0].current, 0);

        // Kill correct monster
        manager.process_event(&QuestEvent::MonsterKilled { monster_id: 1002 }, &quest_data);
        let active = manager.get_quest_progress(2).unwrap();
        assert_eq!(active.progress[0].current, 1);
    }
}
