//! Job Class System
//!
//! Defines job classes, evolution paths, and stat bonuses.

use serde::{Deserialize, Serialize};
use super::element_system::Element;

/// All available job classes in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobClass {
    // Tier 1: Starting class
    Novice,

    // Tier 2: First job advancement (Level 10+)
    Swordsman,
    Mage,
    Archer,
    Thief,
    Merchant,
    Acolyte,

    // Tier 3: Second job advancement (Level 40+)
    Knight,
    Crusader,
    Wizard,
    Sage,
    Hunter,
    Bard,
    Assassin,
    Rogue,
    Blacksmith,
    Alchemist,
    Priest,
    Monk,
}

/// Stat bonuses provided by a job
#[derive(Debug, Clone)]
pub struct StatBonus {
    pub strength: u16,
    pub dexterity: u16,
    pub intelligence: u16,
    pub vitality: u16,
    pub agility: u16,
    pub critical_bonus: u16,
}

/// Stat growth per level for a job
#[derive(Debug, Clone)]
pub struct StatGrowth {
    pub strength: u16,
    pub dexterity: u16,
    pub intelligence: u16,
    pub vitality: u16,
    pub agility: u16,
}

impl JobClass {
    /// Get the stat bonuses for this job class
    pub fn get_stat_bonus(&self) -> StatBonus {
        match self {
            // Tier 1
            JobClass::Novice => StatBonus {
                strength: 0,
                dexterity: 0,
                intelligence: 0,
                vitality: 0,
                agility: 0,
                critical_bonus: 0,
            },

            // Tier 2 - Physical Classes
            JobClass::Swordsman => StatBonus {
                strength: 5,
                dexterity: 2,
                intelligence: 0,
                vitality: 5,
                agility: 2,
                critical_bonus: 5,
            },
            JobClass::Archer => StatBonus {
                strength: 2,
                dexterity: 7,
                intelligence: 0,
                vitality: 2,
                agility: 5,
                critical_bonus: 10,
            },
            JobClass::Thief => StatBonus {
                strength: 3,
                dexterity: 5,
                intelligence: 0,
                vitality: 2,
                agility: 7,
                critical_bonus: 15,
            },
            JobClass::Merchant => StatBonus {
                strength: 4,
                dexterity: 3,
                intelligence: 2,
                vitality: 5,
                agility: 2,
                critical_bonus: 3,
            },

            // Tier 2 - Magic Classes
            JobClass::Mage => StatBonus {
                strength: 0,
                dexterity: 3,
                intelligence: 8,
                vitality: 2,
                agility: 2,
                critical_bonus: 2,
            },
            JobClass::Acolyte => StatBonus {
                strength: 1,
                dexterity: 2,
                intelligence: 6,
                vitality: 4,
                agility: 2,
                critical_bonus: 3,
            },

            // Tier 3 - Swordsman Path
            JobClass::Knight => StatBonus {
                strength: 10,
                dexterity: 5,
                intelligence: 0,
                vitality: 10,
                agility: 5,
                critical_bonus: 10,
            },
            JobClass::Crusader => StatBonus {
                strength: 8,
                dexterity: 4,
                intelligence: 3,
                vitality: 12,
                agility: 3,
                critical_bonus: 8,
            },

            // Tier 3 - Mage Path
            JobClass::Wizard => StatBonus {
                strength: 0,
                dexterity: 5,
                intelligence: 15,
                vitality: 3,
                agility: 4,
                critical_bonus: 5,
            },
            JobClass::Sage => StatBonus {
                strength: 1,
                dexterity: 6,
                intelligence: 12,
                vitality: 5,
                agility: 5,
                critical_bonus: 7,
            },

            // Tier 3 - Archer Path
            JobClass::Hunter => StatBonus {
                strength: 4,
                dexterity: 14,
                intelligence: 2,
                vitality: 4,
                agility: 10,
                critical_bonus: 20,
            },
            JobClass::Bard => StatBonus {
                strength: 3,
                dexterity: 10,
                intelligence: 5,
                vitality: 4,
                agility: 8,
                critical_bonus: 12,
            },

            // Tier 3 - Thief Path
            JobClass::Assassin => StatBonus {
                strength: 6,
                dexterity: 10,
                intelligence: 1,
                vitality: 4,
                agility: 14,
                critical_bonus: 25,
            },
            JobClass::Rogue => StatBonus {
                strength: 5,
                dexterity: 12,
                intelligence: 3,
                vitality: 5,
                agility: 12,
                critical_bonus: 18,
            },

            // Tier 3 - Merchant Path
            JobClass::Blacksmith => StatBonus {
                strength: 9,
                dexterity: 6,
                intelligence: 3,
                vitality: 10,
                agility: 4,
                critical_bonus: 8,
            },
            JobClass::Alchemist => StatBonus {
                strength: 4,
                dexterity: 7,
                intelligence: 8,
                vitality: 6,
                agility: 5,
                critical_bonus: 10,
            },

            // Tier 3 - Acolyte Path
            JobClass::Priest => StatBonus {
                strength: 2,
                dexterity: 4,
                intelligence: 12,
                vitality: 8,
                agility: 4,
                critical_bonus: 6,
            },
            JobClass::Monk => StatBonus {
                strength: 8,
                dexterity: 6,
                intelligence: 4,
                vitality: 10,
                agility: 8,
                critical_bonus: 12,
            },
        }
    }

    /// Get the stat growth per level for this job
    pub fn get_stat_growth(&self) -> StatGrowth {
        match self {
            JobClass::Novice => StatGrowth {
                strength: 1,
                dexterity: 1,
                intelligence: 1,
                vitality: 1,
                agility: 1,
            },

            // Physical focused classes
            JobClass::Swordsman | JobClass::Knight | JobClass::Crusader => StatGrowth {
                strength: 3,
                dexterity: 1,
                intelligence: 0,
                vitality: 3,
                agility: 1,
            },

            JobClass::Thief | JobClass::Assassin | JobClass::Rogue => StatGrowth {
                strength: 2,
                dexterity: 2,
                intelligence: 0,
                vitality: 1,
                agility: 3,
            },

            JobClass::Archer | JobClass::Hunter | JobClass::Bard => StatGrowth {
                strength: 1,
                dexterity: 3,
                intelligence: 0,
                vitality: 1,
                agility: 2,
            },

            // Magic focused classes
            JobClass::Mage | JobClass::Wizard | JobClass::Sage => StatGrowth {
                strength: 0,
                dexterity: 1,
                intelligence: 4,
                vitality: 1,
                agility: 1,
            },

            JobClass::Acolyte | JobClass::Priest => StatGrowth {
                strength: 0,
                dexterity: 1,
                intelligence: 3,
                vitality: 2,
                agility: 1,
            },

            // Hybrid classes
            JobClass::Merchant | JobClass::Blacksmith | JobClass::Alchemist => StatGrowth {
                strength: 2,
                dexterity: 1,
                intelligence: 1,
                vitality: 2,
                agility: 1,
            },

            JobClass::Monk => StatGrowth {
                strength: 2,
                dexterity: 1,
                intelligence: 1,
                vitality: 2,
                agility: 2,
            },
        }
    }

    /// Get the element affinity for this job
    pub fn get_element(&self) -> Element {
        match self {
            JobClass::Novice => Element::Neutral,
            JobClass::Swordsman | JobClass::Knight | JobClass::Crusader => Element::Neutral,
            JobClass::Mage | JobClass::Wizard => Element::Fire, // Can be changed based on specialization
            JobClass::Sage => Element::Neutral,
            JobClass::Archer | JobClass::Hunter | JobClass::Bard => Element::Wind,
            JobClass::Thief | JobClass::Assassin | JobClass::Rogue => Element::Dark,
            JobClass::Merchant | JobClass::Blacksmith | JobClass::Alchemist => Element::Earth,
            JobClass::Acolyte | JobClass::Priest => Element::Holy,
            JobClass::Monk => Element::Neutral,
        }
    }

    /// Get job tier (1 = Novice, 2 = First Job, 3 = Second Job)
    pub fn get_tier(&self) -> u8 {
        match self {
            JobClass::Novice => 1,
            JobClass::Swordsman
            | JobClass::Mage
            | JobClass::Archer
            | JobClass::Thief
            | JobClass::Merchant
            | JobClass::Acolyte => 2,
            _ => 3,
        }
    }

    /// Check if can evolve at current level
    pub fn can_evolve(&self, level: u32) -> bool {
        match self {
            JobClass::Novice => level >= 10,
            JobClass::Swordsman
            | JobClass::Mage
            | JobClass::Archer
            | JobClass::Thief
            | JobClass::Merchant
            | JobClass::Acolyte => level >= 40,
            _ => false, // Tier 3 classes cannot evolve further (yet)
        }
    }

    /// Get available evolutions for this job at current level
    pub fn get_evolutions(&self, level: u32) -> Vec<JobClass> {
        if !self.can_evolve(level) {
            return vec![];
        }

        match self {
            JobClass::Novice => vec![
                JobClass::Swordsman,
                JobClass::Mage,
                JobClass::Archer,
                JobClass::Thief,
                JobClass::Merchant,
                JobClass::Acolyte,
            ],
            JobClass::Swordsman => vec![JobClass::Knight, JobClass::Crusader],
            JobClass::Mage => vec![JobClass::Wizard, JobClass::Sage],
            JobClass::Archer => vec![JobClass::Hunter, JobClass::Bard],
            JobClass::Thief => vec![JobClass::Assassin, JobClass::Rogue],
            JobClass::Merchant => vec![JobClass::Blacksmith, JobClass::Alchemist],
            JobClass::Acolyte => vec![JobClass::Priest, JobClass::Monk],
            _ => vec![],
        }
    }

    /// Get the display name for this job
    pub fn get_name(&self) -> &'static str {
        match self {
            JobClass::Novice => "Novice",
            JobClass::Swordsman => "Swordsman",
            JobClass::Mage => "Mage",
            JobClass::Archer => "Archer",
            JobClass::Thief => "Thief",
            JobClass::Merchant => "Merchant",
            JobClass::Acolyte => "Acolyte",
            JobClass::Knight => "Knight",
            JobClass::Crusader => "Crusader",
            JobClass::Wizard => "Wizard",
            JobClass::Sage => "Sage",
            JobClass::Hunter => "Hunter",
            JobClass::Bard => "Bard",
            JobClass::Assassin => "Assassin",
            JobClass::Rogue => "Rogue",
            JobClass::Blacksmith => "Blacksmith",
            JobClass::Alchemist => "Alchemist",
            JobClass::Priest => "Priest",
            JobClass::Monk => "Monk",
        }
    }

    /// Get description for this job
    pub fn get_description(&self) -> &'static str {
        match self {
            JobClass::Novice => "A beginner with balanced stats. Can evolve into any first job.",
            JobClass::Swordsman => "A warrior who excels in melee combat with high STR and VIT.",
            JobClass::Mage => "A magic user with powerful spells, high INT and MATK.",
            JobClass::Archer => "A ranged attacker with high DEX and critical hits.",
            JobClass::Thief => "A swift attacker with high AGI and critical rate.",
            JobClass::Merchant => "A versatile class with balanced stats and durability.",
            JobClass::Acolyte => "A support class with healing abilities and INT.",
            JobClass::Knight => "Master of swords with devastating physical attacks.",
            JobClass::Crusader => "Holy knight with high defense and balanced attacks.",
            JobClass::Wizard => "Master of elements with the highest magic damage.",
            JobClass::Sage => "Scholar with balanced magic and support capabilities.",
            JobClass::Hunter => "Expert marksman with the highest critical rate.",
            JobClass::Bard => "Musical archer with support and ranged capabilities.",
            JobClass::Assassin => "Silent killer with extreme speed and critical damage.",
            JobClass::Rogue => "Versatile thief with tricks and high evasion.",
            JobClass::Blacksmith => "Craftsman warrior with powerful strikes.",
            JobClass::Alchemist => "Potion master with versatile abilities.",
            JobClass::Priest => "Holy servant with powerful support magic.",
            JobClass::Monk => "Martial artist combining STR and AGI in combat.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novice_evolution() {
        let novice = JobClass::Novice;
        assert!(novice.can_evolve(10));
        assert!(!novice.can_evolve(9));

        let evolutions = novice.get_evolutions(10);
        assert_eq!(evolutions.len(), 6);
        assert!(evolutions.contains(&JobClass::Swordsman));
    }

    #[test]
    fn test_job_tier() {
        assert_eq!(JobClass::Novice.get_tier(), 1);
        assert_eq!(JobClass::Swordsman.get_tier(), 2);
        assert_eq!(JobClass::Knight.get_tier(), 3);
    }

    #[test]
    fn test_stat_bonuses() {
        let knight_bonus = JobClass::Knight.get_stat_bonus();
        assert!(knight_bonus.strength > 0);
        assert!(knight_bonus.vitality > 0);
    }
}
