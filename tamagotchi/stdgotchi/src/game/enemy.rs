//! Enemy system
//!
//! Manages enemy stats, HP, and combat properties

use serde::{Deserialize, Serialize};

/// Enemy type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EnemyType {
    Hornet,
    Poring,
    Fabre,
}

impl EnemyType {
    /// Get enemy ID string
    pub fn id(&self) -> &'static str {
        match self {
            EnemyType::Hornet => "hornet",
            EnemyType::Poring => "poring",
            EnemyType::Fabre => "fabre",
        }
    }

    /// Get enemy display name
    pub fn name(&self) -> &'static str {
        match self {
            EnemyType::Hornet => "Hornet",
            EnemyType::Poring => "Poring",
            EnemyType::Fabre => "Fabre",
        }
    }
}

/// Enemy instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub level: u32,
    pub current_hp: u32,
    pub max_hp: u32,
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub exp_reward: u64,
}

impl Enemy {
    /// Create enemy from type with scaled stats
    pub fn new(enemy_type: EnemyType, hero_level: u32) -> Self {
        let (base_hp, base_atk, base_def, base_exp) = match enemy_type {
            EnemyType::Hornet => (100, 15, 5, 25),
            EnemyType::Poring => (150, 10, 8, 15),
            EnemyType::Fabre => (80, 12, 3, 20),
        };

        // Scale enemy stats based on hero level
        let level = (hero_level + 2).min(50);  // Enemy level slightly higher than hero
        let level_modifier = 1.0 + (level as f32 * 0.1);
        
        let max_hp = (base_hp as f32 * level_modifier) as u32;
        let atk = (base_atk as f32 * level_modifier) as u32;
        let def = (base_def as f32 * level_modifier) as u32;
        let exp_reward = (base_exp as f32 * level_modifier) as u64;

        Self {
            enemy_type,
            level,
            current_hp: max_hp,
            max_hp,
            atk,
            def,
            hit: 90 + level * 2,
            flee: 10 + level,
            exp_reward,
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
        3000  // 3 seconds fixed for now
    }
}
