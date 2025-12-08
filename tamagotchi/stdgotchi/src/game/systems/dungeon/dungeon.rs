//! Dungeon State
//!
//! Manages dungeon run state and progression.

use serde::{Deserialize, Serialize};

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
        }
    }

    /// Advance to next floor after winning combat
    pub fn advance_floor(&mut self, crystals: u32, xp: u32) {
        self.current_floor += 1;
        self.crystals_earned += crystals;
        self.xp_earned += xp;
    }

    /// End the run (either by abandoning or death)
    pub fn end_run(&mut self) {
        self.is_active = false;
    }

    /// Get floors cleared this run
    pub fn floors_cleared(&self) -> u16 {
        self.current_floor.saturating_sub(self.start_floor)
    }
}

/// Dungeon record (highest floor reached)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DungeonRecord {
    pub highest_floor: u16,
}
