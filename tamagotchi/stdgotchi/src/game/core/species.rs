//! Species Data
//!
//! Defines monster species templates loaded from species.json.
//! Each species has base stats, element, learnable skills, and capture zones.

use serde::{Deserialize, Serialize};
use super::Element;

/// A skill that can be learned at a specific level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnableSkill {
    /// Reference to skill ID in tamer_skills.json
    pub skill_id: String,
    /// Level required to learn this skill
    pub level_required: u8,
}

/// Monster species data (loaded from JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    /// Unique species ID (e.g., "poring", "wolf")
    pub id: String,
    /// Display name
    pub name: String,
    /// Element type
    pub element: Element,

    /// Base level (from RO database) - starting level when captured
    pub base_level: u8,

    // Base stats at level 1
    pub base_hp: u16,
    pub base_atk: u16,
    pub base_def: u16,
    pub base_spd: u16,

    /// Base XP reward when defeated (from RO database)
    pub base_exp: u32,

    /// Skills this species can learn at specific levels (Pokemon-style)
    pub learnable_skills: Vec<LearnableSkill>,

    /// Zones where this species can be captured
    pub zones: Vec<String>,
}

impl Species {
    /// Calculate base power rating
    pub fn base_power(&self) -> u16 {
        self.base_atk + self.base_def + self.base_spd + (self.base_hp / 5)
    }
}
