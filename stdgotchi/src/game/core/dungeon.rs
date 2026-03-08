//! Dungeon Data Structure
//!
//! Dungeons are infinite (practically capped) combat gauntlets with checkpoints.
//! Based on GDD section 2.3

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::game::core::Element;

/// Enemy pool for a floor range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyPool {
    /// Minimum floor for this pool
    pub floor_min: u16,
    /// Maximum floor for this pool
    pub floor_max: u16,
    /// Species IDs that can spawn
    pub species: Vec<String>,
    /// Number of enemies per floor
    pub enemies_per_floor: u8,
}

/// Dungeon definition (loaded from JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    /// Unique dungeon ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Associated zone ID
    pub zone_id: String,
    /// Description
    pub description: String,
    /// Checkpoint floors (e.g., [5, 10, 15, 20, ...])
    pub checkpoints: Vec<u16>,
    /// Dominant elements in this dungeon
    pub dominant_elements: Vec<Element>,
    /// Enemy pools by floor range
    pub enemy_pools: Vec<EnemyPool>,
    /// Boss floors (e.g., [10, 20, 30, ...])
    pub boss_floors: Vec<u16>,
    /// Boss species by floor (key is floor number as string)
    pub bosses: HashMap<String, String>,
    /// Base crystal reward per floor
    pub base_crystal_reward: u32,
    /// Base XP reward per floor
    pub base_xp_reward: u32,
}

impl Dungeon {
    /// Get enemy pool for a specific floor
    pub fn get_enemy_pool(&self, floor: u16) -> Option<&EnemyPool> {
        self.enemy_pools.iter().find(|pool| {
            floor >= pool.floor_min && floor <= pool.floor_max
        })
    }

    /// Check if floor is a boss floor
    pub fn is_boss_floor(&self, floor: u16) -> bool {
        self.boss_floors.contains(&floor)
    }

    /// Get boss species for a floor (if it's a boss floor)
    pub fn get_boss_species(&self, floor: u16) -> Option<&str> {
        let key = floor.to_string();
        self.bosses.get(&key).map(|s| s.as_str())
    }

    /// Get available checkpoints based on highest floor reached
    pub fn available_checkpoints(&self, highest_floor: u16) -> Vec<u16> {
        let mut result = vec![1]; // Always start from floor 1
        for &checkpoint in &self.checkpoints {
            if checkpoint <= highest_floor {
                result.push(checkpoint);
            }
        }
        result
    }

    /// Calculate rewards for a floor (with checkpoint multiplier)
    pub fn calculate_rewards(&self, floor: u16, start_floor: u16) -> (u32, u32) {
        let multiplier = match start_floor {
            0..=9 => 1.0,
            10..=19 => 1.5,
            20..=29 => 2.0,
            _ => 2.5,
        };

        let crystals = ((self.base_crystal_reward as f32) * multiplier) as u32;
        let xp = ((self.base_xp_reward as f32) * multiplier) as u32;

        // Add floor-based bonus
        let floor_bonus = 1.0 + (floor as f32 * 0.02);
        let crystals = (crystals as f32 * floor_bonus) as u32;
        let xp = (xp as f32 * floor_bonus) as u32;

        (crystals, xp)
    }
}
