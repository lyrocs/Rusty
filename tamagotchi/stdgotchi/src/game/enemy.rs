//! Enemy system
//!
//! Manages enemy instances in battle

use serde::{Deserialize, Serialize};

use super::element_system::Element;
use super::calculations::xp::calculate_exp_reward;

/// Enemy instance in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub id: u32,
    pub name: String,
    pub level: u32,
    pub current_hp: u32,
    pub max_hp: u32,
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub exp_reward: u64,
    pub element: Element,
}

impl Enemy {
    /// Create enemy from loaded data
    /// Applies XP_MULTIPLIER to base_exp for actual reward
    pub fn from_data(id: u32, name: String, level: u32, hp: u32, attack: u32, defense: u32, hit: u32, flee: u32, base_exp: u64, element: Element) -> Self {
        Self {
            id,
            name,
            level,
            current_hp: hp,
            max_hp: hp,
            atk: attack,
            def: defense,
            hit,
            flee,
            exp_reward: calculate_exp_reward(base_exp),
            element,
        }
    }

    /// Create enemy with level scaling based on hero level
    /// Applies XP_MULTIPLIER to base_exp, then scales by level modifier
    pub fn from_data_scaled(
        id: u32,
        name: String,
        base_level: u32,
        base_hp: u32,
        base_attack: u32,
        base_defense: u32,
        base_hit: u32,
        base_flee: u32,
        base_exp: u64,
        element: Element,
        hero_level: u32,
    ) -> Self {
        // Scale enemy stats based on hero level, but keep base level for display
        let scaling_level = (hero_level + 2).min(50); // For stat scaling
        let level_modifier = 1.0 + ((scaling_level as f32 - base_level as f32) * 0.1);

        let max_hp = (base_hp as f32 * level_modifier).max(1.0) as u32;
        let atk = (base_attack as f32 * level_modifier).max(1.0) as u32;
        let def = (base_defense as f32 * level_modifier).max(0.0) as u32;
        let hit = (base_hit as f32 * level_modifier).max(1.0) as u32;
        let flee = (base_flee as f32 * level_modifier).max(1.0) as u32;
        // Apply multiplier first, then scale
        let exp_reward = (calculate_exp_reward(base_exp) as f32 * level_modifier).max(1.0) as u64;

        Self {
            id,
            name,
            level: base_level, // Display base level from data, not scaled level
            current_hp: max_hp,
            max_hp,
            atk,
            def,
            hit,
            flee,
            exp_reward,
            element,
        }
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: u32) {
        if damage >= self.current_hp {
            self.current_hp = 0;
        } else {
            self.current_hp -= damage;
        }
    }

    /// Check if enemy is alive
    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    /// Get HP percentage
    pub fn hp_percentage(&self) -> f32 {
        (self.current_hp as f32 / self.max_hp as f32) * 100.0
    }

    /// Get attack interval in milliseconds (enemies attack slower)
    pub fn get_attack_interval(&self) -> u64 {
        3000 // 3 seconds fixed for now
    }

    /// Get enemy name for display
    pub fn display_name(&self) -> &str {
        &self.name
    }
}
