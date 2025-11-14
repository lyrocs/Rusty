//! Rustymon Factory
//!
//! Handles creation of Rustymon instances from enemy data with random stats.

use rand::Rng;
use uuid::Uuid;

use super::rustymon::{Element, Rustymon};

/// Factory for creating Rustymon instances
pub struct RustymonFactory;

impl RustymonFactory {
    /// Create a new Rustymon from enemy data with base stats from the enemy
    ///
    /// # Arguments
    /// * `species_id` - The monster ID (e.g., 1002 for Poring)
    /// * `name` - The species name
    /// * `base_level` - The enemy's base level
    /// * `element` - The element type
    /// * `str` - Base STR stat from enemy
    /// * `dex` - Base DEX stat from enemy
    /// * `vit` - Base VIT stat from enemy
    /// * `int` - Base INT stat from enemy
    /// * `luk` - Base LUK stat from enemy
    pub fn create_from_enemy(
        species_id: u32,
        name: String,
        base_level: u32,
        element: Element,
        str: u32,
        dex: u32,
        vit: u32,
        int: u32,
        luk: u32,
    ) -> Rustymon {
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
    /// Note: This uses hardcoded Poring stats - ideally should get from game data
    pub fn create_starter(level: u32) -> Rustymon {
        // Create a Poring with base stats (hardcoded for simplicity)
        // In production, should get these from game data
        let mut rustymon = Self::create_from_enemy(
            1002,
            "Starter Poring".to_string(),
            1,
            Element::Water,
            1,  // str
            1,  // dex
            1,  // vit
            1,  // int
            1,  // luk
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
            5,  // str
            5,  // dex
            5,  // vit
            5,  // int
            5,  // luk
        );

        assert_eq!(rustymon.species_id, 1002);
        assert_eq!(rustymon.name, "Poring");
        assert_eq!(rustymon.level, 1);
        assert_eq!(rustymon.element, Element::Water);
        assert_eq!(rustymon.str, 5);
        assert_eq!(rustymon.dex, 5);
        assert_eq!(rustymon.vit, 5);
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
    fn test_stats_from_enemy_data() {
        // Test that stats are correctly copied from enemy data
        let rustymon1 = RustymonFactory::create_from_enemy(
            1002,
            "Poring".to_string(),
            1,
            Element::Water,
            10,  // str
            8,   // dex
            12,  // vit
            5,   // int
            7,   // luk
        );

        let rustymon2 = RustymonFactory::create_from_enemy(
            1007,
            "Fabre".to_string(),
            1,
            Element::Earth,
            15,  // str
            10,  // dex
            8,   // vit
            6,   // int
            9,   // luk
        );

        // Verify stats are correctly set
        assert_eq!(rustymon1.str, 10);
        assert_eq!(rustymon1.dex, 8);
        assert_eq!(rustymon1.vit, 12);

        assert_eq!(rustymon2.str, 15);
        assert_eq!(rustymon2.dex, 10);
        assert_eq!(rustymon2.vit, 8);
    }
}
