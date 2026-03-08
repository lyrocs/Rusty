//! Player Resources
//!
//! Tracks the player's resources: crystals, essences, and owned monsters.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::Element;

/// Maximum monsters the player can own
pub const MAX_MONSTERS: usize = 6;

/// Player resources and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Crystals (main currency)
    pub crystals: u32,
    /// Elemental essences (for major upgrades)
    pub essences: HashMap<Element, u16>,
}

impl Player {
    /// Create a new player with starting resources
    pub fn new() -> Self {
        let mut essences = HashMap::new();
        for element in Element::all() {
            essences.insert(*element, 0);
        }

        Self {
            crystals: 0,
            essences,
        }
    }

    /// Add crystals
    pub fn add_crystals(&mut self, amount: u32) {
        self.crystals = self.crystals.saturating_add(amount);
    }

    /// Spend crystals, returns true if successful
    pub fn spend_crystals(&mut self, amount: u32) -> bool {
        if self.crystals >= amount {
            self.crystals -= amount;
            true
        } else {
            false
        }
    }

    /// Add essence of an element
    pub fn add_essence(&mut self, element: Element, amount: u16) {
        let current = self.essences.entry(element).or_insert(0);
        *current = current.saturating_add(amount);
    }

    /// Spend essence, returns true if successful
    pub fn spend_essence(&mut self, element: Element, amount: u16) -> bool {
        if let Some(current) = self.essences.get_mut(&element) {
            if *current >= amount {
                *current -= amount;
                return true;
            }
        }
        false
    }

    /// Get essence count for an element
    pub fn get_essence(&self, element: Element) -> u16 {
        *self.essences.get(&element).unwrap_or(&0)
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}
