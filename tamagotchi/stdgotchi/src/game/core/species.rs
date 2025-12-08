//! Species Data
//!
//! Defines monster species templates loaded from species.json.
//! Each species has base stats, element, skill, and capture zones.

use serde::{Deserialize, Serialize};
use super::Element;

/// Monster species data (loaded from JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    /// Unique species ID (e.g., "poring", "wolf")
    pub id: String,
    /// Display name
    pub name: String,
    /// Element type
    pub element: Element,

    // Base stats at level 1
    pub base_hp: u16,
    pub base_atk: u16,
    pub base_def: u16,
    pub base_spd: u16,

    /// Reference to skill in skills.json
    pub skill_id: String,

    /// Zones where this species can be captured
    pub zones: Vec<String>,
}

impl Species {
    /// Calculate base power rating
    pub fn base_power(&self) -> u16 {
        self.base_atk + self.base_def + self.base_spd + (self.base_hp / 5)
    }
}
