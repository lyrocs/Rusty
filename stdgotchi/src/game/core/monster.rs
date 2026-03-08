//! Monster Instance
//!
//! A Monster is an instance of a Species owned by the player.
//! It has stats, level, XP, skills (Pokemon-style), and status.

use serde::{Deserialize, Serialize};
use super::{Element, Skill};

/// Maximum number of equipped skills for battle
pub const MAX_EQUIPPED_SKILLS: usize = 3;

/// Maximum stat bonus per stat (EV-like cap)
pub const MAX_STAT_BONUS: u8 = 50;

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

    // Stat bonuses (EV-like, 0-50 each, added on top of calculated stats)
    #[serde(default)]
    pub hp_bonus: u8,
    #[serde(default)]
    pub atk_bonus: u8,
    #[serde(default)]
    pub def_bonus: u8,
    #[serde(default)]
    pub spd_bonus: u8,

    /// Equipped skills for battle (up to 3)
    pub equipped_skills: Vec<Skill>,

    /// All skill IDs this monster has learned (for skill management)
    pub learned_skill_ids: Vec<String>,

    /// Cooldown timers for each equipped skill slot (turns remaining)
    #[serde(default)]
    pub skill_cooldowns: [u8; MAX_EQUIPPED_SKILLS],

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

    /// Calculate power rating (includes bonuses)
    pub fn power(&self) -> u16 {
        self.total_atk() + self.total_def() + self.total_spd() + (self.total_hp_max() / 5)
    }

    /// Get total ATK (base + bonus)
    pub fn total_atk(&self) -> u16 {
        self.atk + self.atk_bonus as u16
    }

    /// Get total DEF (base + bonus)
    pub fn total_def(&self) -> u16 {
        self.def + self.def_bonus as u16
    }

    /// Get total SPD (base + bonus)
    pub fn total_spd(&self) -> u16 {
        self.spd + self.spd_bonus as u16
    }

    /// Get total HP max (base + bonus * 10 since HP is bigger)
    pub fn total_hp_max(&self) -> u16 {
        self.hp_max + (self.hp_bonus as u16 * 10)
    }

    /// Add bonus to ATK (capped at MAX_STAT_BONUS)
    pub fn add_atk_bonus(&mut self, amount: u8) -> bool {
        if self.atk_bonus >= MAX_STAT_BONUS {
            return false;
        }
        self.atk_bonus = (self.atk_bonus + amount).min(MAX_STAT_BONUS);
        true
    }

    /// Add bonus to DEF (capped at MAX_STAT_BONUS)
    pub fn add_def_bonus(&mut self, amount: u8) -> bool {
        if self.def_bonus >= MAX_STAT_BONUS {
            return false;
        }
        self.def_bonus = (self.def_bonus + amount).min(MAX_STAT_BONUS);
        true
    }

    /// Add bonus to SPD (capped at MAX_STAT_BONUS)
    pub fn add_spd_bonus(&mut self, amount: u8) -> bool {
        if self.spd_bonus >= MAX_STAT_BONUS {
            return false;
        }
        self.spd_bonus = (self.spd_bonus + amount).min(MAX_STAT_BONUS);
        true
    }

    /// Add bonus to HP (capped at MAX_STAT_BONUS)
    pub fn add_hp_bonus(&mut self, amount: u8) -> bool {
        if self.hp_bonus >= MAX_STAT_BONUS {
            return false;
        }
        self.hp_bonus = (self.hp_bonus + amount).min(MAX_STAT_BONUS);
        true
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

    /// Get equipped skill by slot index (0-2)
    pub fn get_skill(&self, slot: usize) -> Option<&Skill> {
        self.equipped_skills.get(slot)
    }

    /// Check if a skill slot is on cooldown
    pub fn is_skill_on_cooldown(&self, slot: usize) -> bool {
        slot < MAX_EQUIPPED_SKILLS && self.skill_cooldowns[slot] > 0
    }

    /// Get remaining cooldown for a skill slot
    pub fn get_skill_cooldown(&self, slot: usize) -> u8 {
        if slot < MAX_EQUIPPED_SKILLS {
            self.skill_cooldowns[slot]
        } else {
            0
        }
    }

    /// Start cooldown for a skill slot after using it
    pub fn start_skill_cooldown(&mut self, slot: usize) {
        if slot < MAX_EQUIPPED_SKILLS {
            if let Some(skill) = self.equipped_skills.get(slot) {
                self.skill_cooldowns[slot] = skill.cooldown;
            }
        }
    }

    /// Decrement all skill cooldowns by 1 (call at end of turn)
    pub fn tick_cooldowns(&mut self) {
        for cd in &mut self.skill_cooldowns {
            *cd = cd.saturating_sub(1);
        }
    }

    /// Reset all cooldowns to 0 (for start of combat or after boss)
    pub fn reset_cooldowns(&mut self) {
        self.skill_cooldowns = [0; MAX_EQUIPPED_SKILLS];
    }

    /// Check if monster has any usable skills (not on cooldown)
    pub fn has_usable_skill(&self) -> bool {
        for (i, _skill) in self.equipped_skills.iter().enumerate() {
            if !self.is_skill_on_cooldown(i) {
                return true;
            }
        }
        false
    }

    /// Get the first usable skill index (for AI/enemy use)
    pub fn get_first_usable_skill_index(&self) -> Option<usize> {
        for (i, _skill) in self.equipped_skills.iter().enumerate() {
            if !self.is_skill_on_cooldown(i) {
                return Some(i);
            }
        }
        None
    }
}
