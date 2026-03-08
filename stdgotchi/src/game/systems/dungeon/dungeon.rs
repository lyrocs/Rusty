//! Dungeon State
//!
//! Manages dungeon run state and progression.

use serde::{Deserialize, Serialize};
use crate::game::core::bonus::{ActiveBonus, ActiveBonusType, StatBoostType};

/// A dungeon run in progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonRun {
    /// Dungeon ID
    pub dungeon_id: String,
    /// Current floor (1-indexed)
    pub current_floor: u16,
    /// Floor where the run started (checkpoint)
    pub start_floor: u16,
    /// Total crystals earned this run
    pub crystals_earned: u32,
    /// Total XP earned this run
    pub xp_earned: u32,
    /// Whether the run is active
    pub is_active: bool,
    /// Skill bar progress preserved between fights (0.0 to 1.0)
    #[serde(default)]
    pub persistent_skill_bar: f32,
    /// Active bonuses (stat boosts, capture boosts)
    #[serde(default)]
    pub active_bonuses: Vec<ActiveBonus>,
}

impl DungeonRun {
    /// Create a new dungeon run starting from a checkpoint
    pub fn new(dungeon_id: String, start_floor: u16) -> Self {
        Self {
            dungeon_id,
            current_floor: start_floor,
            start_floor,
            crystals_earned: 0,
            xp_earned: 0,
            is_active: true,
            persistent_skill_bar: 0.0,
            active_bonuses: Vec::new(),
        }
    }

    /// Advance to next floor after winning combat
    pub fn advance_floor(&mut self, crystals: u32, xp: u32) {
        self.current_floor += 1;
        self.crystals_earned += crystals;
        self.xp_earned += xp;

        // Tick down bonus durations
        self.active_bonuses.retain_mut(|bonus| bonus.tick_floor());
    }

    /// End the run (either by abandoning or death)
    pub fn end_run(&mut self) {
        self.is_active = false;
    }

    /// Get floors cleared this run
    pub fn floors_cleared(&self) -> u16 {
        self.current_floor.saturating_sub(self.start_floor)
    }

    /// Add an active bonus
    pub fn add_bonus(&mut self, bonus: ActiveBonus) {
        log::info!("Added bonus: {:?}, {} floors remaining", bonus.bonus_type, bonus.floors_remaining);
        self.active_bonuses.push(bonus);
    }

    /// Get total stat boost percentage for a stat type
    pub fn get_stat_boost(&self, stat: StatBoostType) -> f32 {
        let mut total = 0.0;
        for bonus in &self.active_bonuses {
            if let ActiveBonusType::StatBoost { stat: bonus_stat, percent } = &bonus.bonus_type {
                if *bonus_stat == stat || *bonus_stat == StatBoostType::AllStats {
                    total += percent;
                }
            }
        }
        total
    }

    /// Get capture multiplier from active bonuses
    pub fn get_capture_multiplier(&self) -> f32 {
        let mut multiplier = 1.0;
        for bonus in &self.active_bonuses {
            if let ActiveBonusType::CaptureBoost { multiplier: m } = &bonus.bonus_type {
                multiplier *= m;
            }
        }
        multiplier
    }
}

/// Dungeon record (highest floor reached)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DungeonRecord {
    pub highest_floor: u16,
}
