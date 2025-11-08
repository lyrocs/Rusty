//! Stats and stat calculation module
//!
//! Handles hero and enemy stats, and calculations for HP, SP, damage, etc.

use serde::{Deserialize, Serialize};

/// Core stats structure (Ragnarok Online style)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    pub str: u32,  // Strength - Physical attack, weight capacity
    pub agi: u32,  // Agility - Attack speed, flee rate
    pub vit: u32,  // Vitality - Max HP, HP regen, defense
    pub int: u32,  // Intelligence - Max SP, magic attack, magic defense
    pub dex: u32,  // Dexterity - Hit rate, attack accuracy
    pub luk: u32,  // Luck - Critical rate, perfect dodge
}

impl Stats {
    pub fn new() -> Self {
        Self {
            str: 5,
            agi: 5,
            vit: 5,
            int: 5,
            dex: 5,
            luk: 5,
        }
    }

    /// Calculate max HP based on VIT and level
    pub fn calculate_max_hp(&self, base_hp: u32, level: u32) -> u32 {
        base_hp + (self.vit * 10) + (level * 5)
    }

    /// Calculate max SP based on INT and level
    pub fn calculate_max_sp(&self, base_sp: u32, level: u32) -> u32 {
        base_sp + (self.int * 5) + (level * 2)
    }

    /// Calculate physical attack power
    pub fn calculate_atk(&self) -> u32 {
        // Increased multiplier for better damage output
        // Base damage + (STR * 5) + DEX bonus
        let base_atk = 20; // Base attack power
        let str_bonus = self.str * 5;
        let dex_bonus = self.dex / 2;
        base_atk + str_bonus + dex_bonus
    }

    /// Calculate physical defense
    pub fn calculate_def(&self) -> u32 {
        self.vit + (self.agi / 4)
    }

    /// Calculate hit rate
    pub fn calculate_hit(&self, level: u32) -> u32 {
        (self.dex * 2) + (self.luk / 2) + level
    }

    /// Calculate flee/dodge rate
    pub fn calculate_flee(&self, level: u32) -> u32 {
        (self.agi * 2) + (self.luk / 3) + level
    }

    /// Calculate critical rate (percentage)
    pub fn calculate_crit_rate(&self) -> f32 {
        (self.luk as f32 / 10.0).min(30.0)  // Cap at 30%
    }

    /// Calculate attack speed interval (milliseconds)
    pub fn calculate_attack_interval(&self) -> u64 {
        let base_interval = 2000.0;  // 2 seconds base
        let modifier = 1.0 + (self.agi as f32 / 50.0);
        (base_interval / modifier) as u64
    }

    /// Calculate HP regeneration per second
    pub fn calculate_hp_regen(&self) -> u32 {
        (self.vit / 5).max(1)
    }

    /// Calculate SP regeneration per second
    pub fn calculate_sp_regen(&self) -> u32 {
        (self.int / 10).max(1)
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
