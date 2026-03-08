//! Zone Data Structure
//!
//! Zones are geographical regions containing multiple maps.
//! Each zone has a dungeon and unlock conditions.

use serde::{Deserialize, Serialize};

/// Unlock condition for a zone
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnlockCondition {
    /// Requires reaching a floor in a dungeon
    DungeonFloor {
        dungeon_id: String,
        floor: u16,
    },
}

/// A geographical zone in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    /// Unique zone ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Zone description
    pub description: String,
    /// Map IDs in this zone
    pub maps: Vec<String>,
    /// Associated dungeon ID
    pub dungeon_id: String,
    /// Unlock condition (None = unlocked by default)
    pub unlock_condition: Option<UnlockCondition>,
    /// Recommended level range
    pub level_range: (u8, u8),
}

impl Zone {
    /// Check if zone is unlocked based on dungeon progress
    pub fn is_unlocked(&self, dungeon_progress: &std::collections::HashMap<String, u16>) -> bool {
        match &self.unlock_condition {
            None => true,
            Some(UnlockCondition::DungeonFloor { dungeon_id, floor }) => {
                dungeon_progress.get(dungeon_id).map_or(false, |&reached| reached >= *floor)
            }
        }
    }
}
