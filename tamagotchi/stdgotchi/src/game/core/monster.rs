//! Monster Instance
//!
//! A Monster is an instance of a Species owned by the player.
//! It has stats, level, XP, and status.

use serde::{Deserialize, Serialize};
use super::{Element, Skill};

/// Monster status - where the monster currently is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonsterStatus {
    /// Available for selection
    Available,
    /// Currently on an expedition
    InExpedition,
    /// Currently in a dungeon run
    InDungeon,
}

impl Default for MonsterStatus {
    fn default() -> Self {
        MonsterStatus::Available
    }
}

/// A monster instance owned by the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    /// Unique instance ID (UUID)
    pub id: String,
    /// Species type ID (e.g., "poring")
    pub species_id: String,
    /// Display name
    pub name: String,
    /// Current level (1-99)
    pub level: u8,
    /// Current XP
    pub xp: u32,
    /// XP needed for next level
    pub xp_to_next: u32,
    /// Monster element
    pub element: Element,
    /// Fusion count (0-9), each gives +5% stats
    pub fusion_count: u8,

    // Stats (calculated from base + level + fusion)
    pub hp_current: u16,
    pub hp_max: u16,
    pub atk: u16,
    pub def: u16,
    pub spd: u16,

    /// The monster's unique skill
    pub skill: Skill,

    /// Current status
    pub status: MonsterStatus,
}

impl Monster {
    /// Check if monster is alive
    pub fn is_alive(&self) -> bool {
        self.hp_current > 0
    }

    /// Take damage, returns actual damage taken
    pub fn take_damage(&mut self, damage: u16) -> u16 {
        let actual_damage = damage.min(self.hp_current);
        self.hp_current = self.hp_current.saturating_sub(damage);
        actual_damage
    }

    /// Heal, returns actual amount healed
    pub fn heal(&mut self, amount: u16) -> u16 {
        let actual_heal = amount.min(self.hp_max - self.hp_current);
        self.hp_current = (self.hp_current + amount).min(self.hp_max);
        actual_heal
    }

    /// Fully heal the monster
    pub fn full_heal(&mut self) {
        self.hp_current = self.hp_max;
    }

    /// Calculate power rating
    pub fn power(&self) -> u16 {
        self.atk + self.def + self.spd + (self.hp_max / 5)
    }

    /// Get HP percentage (0.0 to 1.0)
    pub fn hp_percentage(&self) -> f32 {
        if self.hp_max == 0 {
            0.0
        } else {
            self.hp_current as f32 / self.hp_max as f32
        }
    }

    /// Get XP percentage (0.0 to 1.0)
    pub fn xp_percentage(&self) -> f32 {
        if self.xp_to_next == 0 {
            1.0
        } else {
            self.xp as f32 / self.xp_to_next as f32
        }
    }
}
