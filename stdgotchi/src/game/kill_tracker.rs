//! Kill tracking system
//!
//! Tracks number of kills per enemy ID

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks kills for each enemy ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillTracker {
    kills: HashMap<u32, u32>, // enemy_id -> kill_count
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

    /// Record a kill for an enemy by ID and name
    pub fn record_kill(&mut self, enemy_id: u32, enemy_name: &str) {
        *self.kills.entry(enemy_id).or_insert(0) += 1;
        self.total_kills += 1;

        log::info!(
            "{} killed! Total kills: {}",
            enemy_name,
            self.get_kills(enemy_id)
        );
    }

    /// Get kill count for specific enemy ID
    pub fn get_kills(&self, enemy_id: u32) -> u32 {
        *self.kills.get(&enemy_id).unwrap_or(&0)
    }

    /// Get total kills across all enemies
    pub fn get_total_kills(&self) -> u32 {
        self.total_kills
    }

    /// Get all kill counts
    pub fn get_all_kills(&self) -> &HashMap<u32, u32> {
        &self.kills
    }
}
