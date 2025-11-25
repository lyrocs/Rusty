//! Fragment Collection System
//!
//! Manages monster fragments that can be used to evolve Rustymon.
//! Evolution uses Fibonacci-based fragment requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Calculate Fibonacci number at position n (1-indexed)
/// Used for evolution fragment requirements
/// Returns: fib(0)=0, fib(1)=1, fib(2)=1, fib(3)=2, fib(4)=3, fib(5)=5, fib(6)=8, etc.
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

/// Calculate fragments needed for evolution
/// Evolution 0 (initial summon): base_fragments
/// Evolution 1: base_fragments * fib(2) = base * 2
/// Evolution 2: base_fragments * fib(3) = base * 3
/// Evolution 3: base_fragments * fib(4) = base * 5
/// Evolution 4: base_fragments * fib(5) = base * 8
/// Example: If base is 10, then: 10, 20, 30, 50, 80, 130, 210, ...
pub fn calculate_evolution_fragments(base_fragments: u32, evolution_level: u32) -> u32 {
    if evolution_level == 0 {
        base_fragments
    } else {
        base_fragments * fibonacci(evolution_level + 1)
    }
}

/// Collection of monster fragments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentCollection {
    /// Map of monster ID to fragment count
    pub fragments: HashMap<u32, u32>,
}

impl Default for FragmentCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl FragmentCollection {
    /// Create a new empty fragment collection
    pub fn new() -> Self {
        Self {
            fragments: HashMap::new(),
        }
    }

    /// Add fragments for a specific monster
    pub fn add_fragment(&mut self, monster_id: u32, count: u32) {
        *self.fragments.entry(monster_id).or_insert(0) += count;
        log::info!("Added {} fragment(s) for monster ID {}", count, monster_id);
    }

    /// Remove fragments for a specific monster (used when summoning)
    /// Returns true if successful, false if not enough fragments
    pub fn remove_fragments(&mut self, monster_id: u32, count: u32) -> bool {
        if let Some(current_count) = self.fragments.get_mut(&monster_id) {
            if *current_count >= count {
                *current_count -= count;

                // Remove entry if count reaches zero
                if *current_count == 0 {
                    self.fragments.remove(&monster_id);
                }

                log::info!("Removed {} fragment(s) for monster ID {}", count, monster_id);
                return true;
            }
        }
        false
    }

    /// Get the number of fragments for a specific monster
    pub fn get_fragment_count(&self, monster_id: u32) -> u32 {
        self.fragments.get(&monster_id).copied().unwrap_or(0)
    }

    /// Check if player has enough fragments to summon a monster
    pub fn can_summon(&self, monster_id: u32, required_count: u32) -> bool {
        self.get_fragment_count(monster_id) >= required_count
    }

    /// Get all monster IDs that have fragments
    pub fn get_monsters_with_fragments(&self) -> Vec<u32> {
        let mut monster_ids: Vec<u32> = self.fragments.keys().copied().collect();
        monster_ids.sort();
        monster_ids
    }

    /// Get total number of different monster types with fragments
    pub fn get_unique_monster_count(&self) -> usize {
        self.fragments.len()
    }

    /// Get total number of all fragments
    pub fn get_total_fragment_count(&self) -> u32 {
        self.fragments.values().sum()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// Clear all fragments (used for testing or reset)
    pub fn clear(&mut self) {
        self.fragments.clear();
    }

    /// Get a list of monsters with fragments and their counts
    pub fn get_fragment_list(&self) -> Vec<(u32, u32)> {
        let mut list: Vec<(u32, u32)> = self.fragments.iter().map(|(&id, &count)| (id, count)).collect();
        list.sort_by_key(|(id, _)| *id);
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_fragments() {
        let mut collection = FragmentCollection::new();

        collection.add_fragment(1002, 5);
        assert_eq!(collection.get_fragment_count(1002), 5);

        collection.add_fragment(1002, 3);
        assert_eq!(collection.get_fragment_count(1002), 8);
    }

    #[test]
    fn test_remove_fragments() {
        let mut collection = FragmentCollection::new();

        collection.add_fragment(1002, 10);
        assert!(collection.remove_fragments(1002, 5));
        assert_eq!(collection.get_fragment_count(1002), 5);

        // Try to remove more than available
        assert!(!collection.remove_fragments(1002, 10));
        assert_eq!(collection.get_fragment_count(1002), 5);

        // Remove all remaining
        assert!(collection.remove_fragments(1002, 5));
        assert_eq!(collection.get_fragment_count(1002), 0);
    }

    #[test]
    fn test_can_summon() {
        let mut collection = FragmentCollection::new();

        collection.add_fragment(1002, 5);
        assert!(collection.can_summon(1002, 5));
        assert!(!collection.can_summon(1002, 6));
        assert!(!collection.can_summon(1007, 1)); // Different monster
    }

    #[test]
    fn test_get_monsters_with_fragments() {
        let mut collection = FragmentCollection::new();

        collection.add_fragment(1002, 5);
        collection.add_fragment(1007, 3);
        collection.add_fragment(1004, 2);

        let monsters = collection.get_monsters_with_fragments();
        assert_eq!(monsters.len(), 3);
        assert!(monsters.contains(&1002));
        assert!(monsters.contains(&1007));
        assert!(monsters.contains(&1004));
    }

    #[test]
    fn test_totals() {
        let mut collection = FragmentCollection::new();

        collection.add_fragment(1002, 5);
        collection.add_fragment(1007, 3);

        assert_eq!(collection.get_unique_monster_count(), 2);
        assert_eq!(collection.get_total_fragment_count(), 8);
    }
}
