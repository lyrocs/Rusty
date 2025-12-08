//! Tamer Map Data Structure
//!
//! Maps are expedition destinations within zones.
//! Each map has element requirements and capturable species.

use serde::{Deserialize, Serialize};
use super::Element;

/// Essence reward data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssenceReward {
    /// Element type
    pub element: Element,
    /// Amount of essence
    pub amount: u8,
}

/// Base rewards for a map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapBaseRewards {
    /// Crystal reward
    pub crystals: u16,
    /// Essence rewards
    pub essences: Vec<EssenceReward>,
}

/// A map within a zone for expeditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamerMap {
    /// Unique map ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Parent zone ID
    pub zone_id: String,
    /// Recommended level range
    pub level_range: (u8, u8),
    /// Required elements for expedition (at least one monster must have these)
    pub required_elements: Vec<Element>,
    /// Species that can be captured on this map
    pub capturable_species: Vec<String>,
    /// Base rewards for expeditions
    pub base_rewards: MapBaseRewards,
}

impl TamerMap {
    /// Check if a team meets the element requirements
    pub fn meets_element_requirements(&self, team_elements: &[Element]) -> bool {
        if self.required_elements.is_empty() {
            return true;
        }
        // Team must have at least one monster of each required element
        self.required_elements.iter().all(|required| {
            team_elements.contains(required)
        })
    }
}
