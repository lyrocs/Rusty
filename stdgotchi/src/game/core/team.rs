//! Team Management
//!
//! The player's active team of up to 3 monsters for dungeon runs.

use serde::{Deserialize, Serialize};

/// Maximum monsters in a team
pub const MAX_TEAM_SIZE: usize = 3;

/// The player's active team for dungeon runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    /// Monster IDs in the team (up to 3)
    monster_ids: Vec<String>,
    /// Index of the currently active monster (0-2)
    active_index: u8,
}

impl Team {
    /// Create a new empty team
    pub fn new() -> Self {
        Self {
            monster_ids: Vec::new(),
            active_index: 0,
        }
    }

    /// Add a monster to the team
    pub fn add(&mut self, monster_id: String) -> bool {
        if self.monster_ids.len() >= MAX_TEAM_SIZE {
            return false;
        }
        if self.monster_ids.contains(&monster_id) {
            return false;
        }
        self.monster_ids.push(monster_id);
        true
    }

    /// Remove a monster from the team
    pub fn remove(&mut self, monster_id: &str) -> bool {
        if let Some(pos) = self.monster_ids.iter().position(|id| id == monster_id) {
            self.monster_ids.remove(pos);
            // Adjust active index if needed
            if self.active_index as usize >= self.monster_ids.len() && !self.monster_ids.is_empty() {
                self.active_index = (self.monster_ids.len() - 1) as u8;
            }
            true
        } else {
            false
        }
    }

    /// Get the active monster ID
    pub fn active_monster_id(&self) -> Option<&String> {
        self.monster_ids.get(self.active_index as usize)
    }

    /// Set active monster by index
    pub fn set_active(&mut self, index: u8) -> bool {
        if (index as usize) < self.monster_ids.len() {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    /// Swap to next monster (for combat)
    pub fn swap_to(&mut self, index: u8) -> bool {
        if (index as usize) < self.monster_ids.len() && index != self.active_index {
            self.active_index = index;
            true
        } else {
            false
        }
    }

    /// Get all monster IDs in the team
    pub fn monster_ids(&self) -> &[String] {
        &self.monster_ids
    }

    /// Get team size
    pub fn len(&self) -> usize {
        self.monster_ids.len()
    }

    /// Check if team is empty
    pub fn is_empty(&self) -> bool {
        self.monster_ids.is_empty()
    }

    /// Check if team is full
    pub fn is_full(&self) -> bool {
        self.monster_ids.len() >= MAX_TEAM_SIZE
    }

    /// Get active index
    pub fn active_index(&self) -> u8 {
        self.active_index
    }

    /// Check if monster is in team
    pub fn contains(&self, monster_id: &str) -> bool {
        self.monster_ids.iter().any(|id| id == monster_id)
    }
}

impl Default for Team {
    fn default() -> Self {
        Self::new()
    }
}
