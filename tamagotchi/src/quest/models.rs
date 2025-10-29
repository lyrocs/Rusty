/// Quest system models
///
/// Defines quest types, objectives, rewards, and active quest tracking.
use heapless::Vec as HeaplessVec;
use serde::Deserialize;

/// Quest types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum QuestType {
    Story,
    Daily,
    Achievement,
}

/// Quest objective (flat structure for no-std JSON parsing)
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct QuestObjective {
    #[serde(rename = "type")]
    pub objective_type: &'static str,
    #[serde(default)]
    pub enemy_id: u32, // For KillMonster (0 = any)
    #[serde(default)]
    pub item_id: u32, // For CollectItem
    #[serde(default)]
    pub count: u16, // For KillMonster, CollectItem, RefineEquipment, CompleteBattles
    #[serde(default)]
    pub amount: u32, // For EarnZeny
    #[serde(default)]
    pub level: u16, // For ReachLevel
}

/// Quest reward data
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct QuestReward {
    #[serde(default)]
    pub base_exp: u32,
    #[serde(default)]
    pub zeny: u32,
    #[serde(default)]
    pub items: [(u32, u16); 4], // (Item ID, Quantity) pairs (0, 0 = empty slot)
}

/// Quest data from JSON
#[derive(Debug, Clone, Deserialize)]
pub struct QuestData {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub quest_type: QuestType,
    pub min_level: u16,                             // Min level to accept
    pub max_level: u16,                             // Max level for quest (0 = no limit)
    #[serde(default)]
    pub priority: u16,                              // Sort order (lower = higher priority)
    pub objectives: HeaplessVec<QuestObjective, 4>, // Up to 4 objectives per quest
    pub rewards: QuestReward,
}

/// Active quest (tracking player progress)
#[derive(Debug, Clone, Copy)]
pub struct ActiveQuest {
    pub quest_id: u32,
    pub progress: [u16; 4], // Progress for each objective (up to 4)
    pub completed: bool,    // Objectives done, ready to claim
    pub claimed: bool,      // Rewards claimed
}

impl ActiveQuest {
    pub fn new(quest_id: u32, _objective_count: usize, _timestamp: u32) -> Self {
        Self {
            quest_id,
            progress: [0, 0, 0, 0],
            completed: false,
            claimed: false,
        }
    }

    /// Check if all objectives are complete
    pub fn check_completion(&mut self, quest_data: &QuestData) {
        let mut all_complete = true;
        for (i, objective) in quest_data.objectives.iter().enumerate() {
            if self.progress[i] < objective.count {
                all_complete = false;
                break;
            }
        }
        self.completed = all_complete;
    }
}

/// Quest actions that trigger progress updates
#[derive(Debug, Clone, Copy)]
pub enum QuestAction {
    MonsterKilled { enemy_id: u32 },
    BattleCompleted,
    ItemCollected { item_id: u32, quantity: u16 },
    ZenyEarned { amount: u32 },
    EquipmentRefined,
    LevelReached { level: u16 },
}
