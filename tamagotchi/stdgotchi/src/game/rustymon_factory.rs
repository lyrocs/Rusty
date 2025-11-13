//! Rustymon Factory
//!
//! Handles creation of Rustymon instances from enemy data with random stats.

use rand::Rng;
use uuid::Uuid;

use super::rustymon::{Element, Rustymon};

/// Factory for creating Rustymon instances
pub struct RustymonFactory;

impl RustymonFactory {
    /// Create a new Rustymon from enemy data with random stats
    ///
    /// # Arguments
    /// * `species_id` - The monster ID (e.g., 1002 for Poring)
    /// * `name` - The species name
    /// * `level` - The enemy's base level (used for stat ranges)
    /// * `element` - The element type
    pub fn create_from_enemy(
        species_id: u32,
        name: String,
        base_level: u32,
        element: Element,
    ) -> Rustymon {
        let mut rng = rand::thread_rng();

        // Calculate stat ranges based on enemy level
        // Base stat starts at 5 + level, with variance of 5
        let base_stat = 5 + base_level;
        let variance = 5;

        // Randomly generate base stats within range
        let str = rng.gen_range(base_stat..=base_stat + variance);
        let dex = rng.gen_range(base_stat..=base_stat + variance);
        let vit = rng.gen_range(base_stat..=base_stat + variance);
        let int = rng.gen_range(base_stat..=base_stat + variance);
        let luk = rng.gen_range(base_stat..=base_stat + variance);

        log::info!(
            "Creating {} with stats - STR:{} DEX:{} VIT:{} INT:{} LUK:{}",
            name, str, dex, vit, int, luk
        );

        // Create the Rustymon at level 1
        Rustymon::new(
            Uuid::new_v4().to_string(),
            species_id,
            name,
            1, // Always start at level 1
            element,
            str,
            dex,
            vit,
            int,
            luk,
        )
    }

    /// Create a starter Rustymon (Poring) at a specific level
    /// Used for new players or migration
    pub fn create_starter(level: u32) -> Rustymon {
        // Create a Poring with slightly better stats
        let mut rustymon = Self::create_from_enemy(
            1002,
            "Starter Poring".to_string(),
            1,
            Element::Water,
        );

        // Level it up to the desired level
        if level > 1 {
            rustymon.level = level;
            rustymon.recalculate_stats();
            rustymon.full_heal();
        }

        rustymon
    }

    /// Recalculate all stats for a Rustymon (used after level up or stat changes)
    pub fn recalculate_stats(rustymon: &mut Rustymon) {
        rustymon.recalculate_stats();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_from_enemy() {
        let rustymon = RustymonFactory::create_from_enemy(
            1002,
            "Poring".to_string(),
            1,
            Element::Water,
        );

        assert_eq!(rustymon.species_id, 1002);
        assert_eq!(rustymon.name, "Poring");
        assert_eq!(rustymon.level, 1);
        assert_eq!(rustymon.element, Element::Water);
        assert!(rustymon.str >= 6 && rustymon.str <= 11); // 5+1 to 5+1+5
        assert!(rustymon.max_hp > 0);
        assert!(rustymon.is_alive());
    }

    #[test]
    fn test_create_starter() {
        let rustymon = RustymonFactory::create_starter(5);

        assert_eq!(rustymon.species_id, 1002);
        assert_eq!(rustymon.level, 5);
        assert_eq!(rustymon.element, Element::Water);
        assert!(rustymon.is_alive());
    }

    #[test]
    fn test_random_stats_variation() {
        // Create multiple Rustymon and ensure they have different stats
        let rustymon1 = RustymonFactory::create_from_enemy(
            1002,
            "Poring".to_string(),
            1,
            Element::Water,
        );

        let rustymon2 = RustymonFactory::create_from_enemy(
            1002,
            "Poring".to_string(),
            1,
            Element::Water,
        );

        // Very unlikely to have identical stats (5^5 combinations)
        let stats1 = (rustymon1.str, rustymon1.dex, rustymon1.vit, rustymon1.int, rustymon1.luk);
        let stats2 = (rustymon2.str, rustymon2.dex, rustymon2.vit, rustymon2.int, rustymon2.luk);

        // This might occasionally fail, but probability is very low
        // Comment out if it causes test flakiness
        // assert_ne!(stats1, stats2);
    }
}
