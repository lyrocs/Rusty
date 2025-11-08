//! Kill tracking system
//!
//! Tracks number of kills per enemy type

use super::enemy::EnemyType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks kills for each enemy type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillTracker {
    kills: HashMap<EnemyType, u32>,
    total_kills: u32,
}

impl Default for KillTracker {
    fn default() -> Self {
        Self {
            kills: HashMap::new(),
            total_kills: 0,
        }
    }
}

impl KillTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a kill for an enemy type
    pub fn record_kill(&mut self, enemy_type: EnemyType) {
        *self.kills.entry(enemy_type).or_insert(0) += 1;
        self.total_kills += 1;
        
        log::info!("{} killed! Total kills: {}", enemy_type.name(), self.get_kills(enemy_type));
    }

    /// Get kill count for specific enemy type
    pub fn get_kills(&self, enemy_type: EnemyType) -> u32 {
        *self.kills.get(&enemy_type).unwrap_or(&0)
    }

    /// Get total kills across all enemies
    pub fn get_total_kills(&self) -> u32 {
        self.total_kills
    }

    /// Get all kill counts
    pub fn get_all_kills(&self) -> &HashMap<EnemyType, u32> {
        &self.kills
    }
}
