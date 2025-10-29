/// Combat models
///
/// Core combat entities including Enemy and basic combat states.

use crate::tamagotchi::{get_enemy_data};

/// Enemy data (based on data/enemies.json)
/// Note: JSON files in data/ folder serve as source of truth
/// This struct contains runtime enemy data used in battles
#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: u32, // Enemy ID from JSON
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub attack: u16,      // Added from JSON
    pub defense: u16,     // Added from JSON
    pub base_exp: u32,    // Renamed from exp_reward
    pub job_exp: u32,     // Added from JSON
    pub zeny_reward: u32, // Calculated zeny (base_exp / 10)
}

impl Enemy {
    /// Get a random enemy based on hero level (from enemies.json)
    /// Uses generated data from build.rs
    pub fn random_for_level(_hero_level: u16, rng_value: u8) -> Self {
        // Enemy IDs from enemies.json: Poring=1002, Fabre=1007, Hornet=1004, Thief Bug=1051
        let enemy_id = match rng_value % 4 {
            0 => 1002, // Poring
            1 => 1007, // Fabre
            2 => 1004, // Hornet
            _ => 1051, // Thief Bug
        };

        // Use generated function to get enemy data from JSON
        get_enemy_data(enemy_id).expect("Enemy ID should exist in enemies.json")
    }

    /// Get enemy by ID from JSON data (convenience function)
    pub fn from_id(id: u32) -> Option<Self> {
        get_enemy_data(id)
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn hp_percent(&self) -> u8 {
        ((self.hp as u32 * 100) / self.max_hp as u32) as u8
    }
}

/// Farming state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarmState {
    Idle,
    Fighting,
    Victory,
    Defeat,
}

/// Rest state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestState {
    Resting,
    FullSP,
    Complete,
}

/// Combat result type (for JRPG battles)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResult {
    Normal,
    Critical,
    Lucky,
    Miss,
}
