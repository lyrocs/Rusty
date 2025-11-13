//! Rustymon Team Management
//!
//! Manages the active team of up to 4 Rustymon and the bank storage.

use serde::{Deserialize, Serialize};

/// Manages the player's Rustymon team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustymonTeam {
    /// Active team slots (up to 4 Rustymon IDs)
    /// None means empty slot
    pub active_slots: [Option<String>; 4],

    /// Index of currently active Rustymon in battle (0-3)
    pub active_index: usize,

    /// Bank storage for Rustymon not in active team
    pub bank: Vec<String>,
}

impl Default for RustymonTeam {
    fn default() -> Self {
        Self::new()
    }
}

impl RustymonTeam {
    /// Create a new empty team
    pub fn new() -> Self {
        Self {
            active_slots: [None, None, None, None],
            active_index: 0,
            bank: Vec::new(),
        }
    }

    /// Add a Rustymon to the first available team slot
    /// Returns true if added to team, false if added to bank
    pub fn add_rustymon(&mut self, rustymon_id: String) -> bool {
        // Try to add to first empty slot
        for slot in &mut self.active_slots {
            if slot.is_none() {
                *slot = Some(rustymon_id);
                return true;
            }
        }

        // If all slots full, add to bank
        self.bank.push(rustymon_id);
        false
    }

    /// Remove a Rustymon from the team by ID
    /// Returns true if removed from team, false if removed from bank
    pub fn remove_rustymon(&mut self, rustymon_id: &str) -> bool {
        // Check active slots first
        for slot in &mut self.active_slots {
            if let Some(id) = slot {
                if id == rustymon_id {
                    *slot = None;
                    return true;
                }
            }
        }

        // Check bank
        if let Some(pos) = self.bank.iter().position(|id| id == rustymon_id) {
            self.bank.remove(pos);
            return false;
        }

        false
    }

    /// Get the currently active Rustymon ID in battle
    pub fn get_active_rustymon_id(&self) -> Option<&String> {
        self.active_slots[self.active_index].as_ref()
    }

    /// Switch to next available Rustymon in team
    /// Returns the new active index, or None if no more available
    pub fn switch_to_next(&mut self) -> Option<usize> {
        let start_index = self.active_index;
        let mut attempts = 0;

        loop {
            self.active_index = (self.active_index + 1) % 4;
            attempts += 1;

            // If we've looped back to start or tried all slots, no more available
            if attempts >= 4 {
                return None;
            }

            // If this slot has a Rustymon, use it
            if self.active_slots[self.active_index].is_some() {
                return Some(self.active_index);
            }
        }
    }

    /// Move a Rustymon from bank to team
    /// Returns true if successful, false if team is full or Rustymon not in bank
    pub fn move_from_bank_to_team(&mut self, rustymon_id: &str) -> bool {
        // Check if in bank
        let bank_pos = match self.bank.iter().position(|id| id == rustymon_id) {
            Some(pos) => pos,
            None => return false,
        };

        // Find empty slot
        for slot in &mut self.active_slots {
            if slot.is_none() {
                let id = self.bank.remove(bank_pos);
                *slot = Some(id);
                return true;
            }
        }

        false // Team is full
    }

    /// Move a Rustymon from team to bank
    /// Returns true if successful, false if not in team
    pub fn move_from_team_to_bank(&mut self, rustymon_id: &str) -> bool {
        for slot in &mut self.active_slots {
            if let Some(id) = slot {
                if id == rustymon_id {
                    let id = slot.take().unwrap();
                    self.bank.push(id);
                    return true;
                }
            }
        }

        false
    }

    /// Check if a Rustymon is in the active team
    pub fn is_in_team(&self, rustymon_id: &str) -> bool {
        self.active_slots
            .iter()
            .any(|slot| slot.as_ref().map(|id| id == rustymon_id).unwrap_or(false))
    }

    /// Check if a Rustymon is in the bank
    pub fn is_in_bank(&self, rustymon_id: &str) -> bool {
        self.bank.iter().any(|id| id == rustymon_id)
    }

    /// Get the number of Rustymon in active team
    pub fn team_count(&self) -> usize {
        self.active_slots.iter().filter(|slot| slot.is_some()).count()
    }

    /// Get the number of Rustymon in bank
    pub fn bank_count(&self) -> usize {
        self.bank.len()
    }

    /// Get total number of Rustymon
    pub fn total_count(&self) -> usize {
        self.team_count() + self.bank_count()
    }

    /// Check if team is full
    pub fn is_team_full(&self) -> bool {
        self.team_count() == 4
    }

    /// Check if team is empty
    pub fn is_team_empty(&self) -> bool {
        self.team_count() == 0
    }

    /// Get all Rustymon IDs in team (non-None slots)
    pub fn get_team_ids(&self) -> Vec<String> {
        self.active_slots
            .iter()
            .filter_map(|slot| slot.clone())
            .collect()
    }

    /// Get all Rustymon IDs in bank
    pub fn get_bank_ids(&self) -> Vec<String> {
        self.bank.clone()
    }

    /// Set a specific Rustymon as active by ID
    /// Returns true if successful, false if not in team
    pub fn set_active_rustymon(&mut self, rustymon_id: String) -> bool {
        for (index, slot) in self.active_slots.iter().enumerate() {
            if let Some(id) = slot {
                if id == &rustymon_id {
                    self.active_index = index;
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_rustymon() {
        let mut team = RustymonTeam::new();

        // Add to team
        assert!(team.add_rustymon("id1".to_string()));
        assert_eq!(team.team_count(), 1);

        // Add 3 more to fill team
        assert!(team.add_rustymon("id2".to_string()));
        assert!(team.add_rustymon("id3".to_string()));
        assert!(team.add_rustymon("id4".to_string()));
        assert_eq!(team.team_count(), 4);
        assert!(team.is_team_full());

        // Next should go to bank
        assert!(!team.add_rustymon("id5".to_string()));
        assert_eq!(team.bank_count(), 1);
    }

    #[test]
    fn test_switch_to_next() {
        let mut team = RustymonTeam::new();

        team.add_rustymon("id1".to_string());
        team.add_rustymon("id2".to_string());

        assert_eq!(team.active_index, 0);
        team.switch_to_next();
        assert_eq!(team.active_index, 1);
        team.switch_to_next();
        assert_eq!(team.active_index, 0); // Wraps around
    }

    #[test]
    fn test_move_between_team_and_bank() {
        let mut team = RustymonTeam::new();

        // Fill team
        team.add_rustymon("id1".to_string());
        team.add_rustymon("id2".to_string());
        team.add_rustymon("id3".to_string());
        team.add_rustymon("id4".to_string());
        team.add_rustymon("id5".to_string()); // Goes to bank

        assert_eq!(team.team_count(), 4);
        assert_eq!(team.bank_count(), 1);

        // Move from team to bank
        assert!(team.move_from_team_to_bank("id1"));
        assert_eq!(team.team_count(), 3);
        assert_eq!(team.bank_count(), 2);

        // Move from bank to team
        assert!(team.move_from_bank_to_team("id5"));
        assert_eq!(team.team_count(), 4);
        assert_eq!(team.bank_count(), 1);
    }
}
