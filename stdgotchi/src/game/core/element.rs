//! Element System
//!
//! Defines the 9 elements based on Ragnarok Online and their relationships.
//! Element advantages and reactions are loaded from element_config.json.

use serde::{Deserialize, Serialize};

/// The 9 elements (8 from RO + Neutral)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    #[default]
    Neutral,  // No elemental affinity, no advantages/disadvantages
    Fire,
    Water,
    Earth,
    Wind,
    Thunder,
    Shadow,
    Holy,
    Ghost,
}

impl Element {
    /// Get element display icon
    pub fn icon(&self) -> &'static str {
        match self {
            Element::Neutral => "Neutral",
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Earth => "Earth",
            Element::Wind => "Wind",
            Element::Thunder => "Thunder",
            Element::Shadow => "Shadow",
            Element::Holy => "Holy",
            Element::Ghost => "Ghost",
        }
    }

    /// Get all elements
    pub fn all() -> &'static [Element] {
        &[
            Element::Neutral,
            Element::Fire,
            Element::Water,
            Element::Earth,
            Element::Wind,
            Element::Thunder,
            Element::Shadow,
            Element::Holy,
            Element::Ghost,
        ]
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.icon())
    }
}
