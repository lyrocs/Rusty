//! Expedition Rewards
//!
//! Calculates rewards from completed expeditions.

use super::ExpeditionDuration;
use crate::game::core::Element;

/// Expedition reward data
#[derive(Debug, Clone)]
pub struct ExpeditionRewards {
    /// XP gained per monster
    pub xp_per_monster: u32,
    /// Crystals earned
    pub crystals: u16,
    /// Essences earned (element, amount)
    pub essences: Vec<(Element, u8)>,
    /// Capture chance (0.0 to 1.0)
    pub capture_chance: f32,
}

/// Get base rewards for a duration
/// Based on GDD table 2.2.3
/// NOTE: Dev capture_chance is 1.0 (100%). Production values: 0.15/0.25/0.40/0.50
pub fn get_base_rewards(duration: ExpeditionDuration) -> ExpeditionRewards {
    match duration {
        ExpeditionDuration::Short => ExpeditionRewards {
            xp_per_monster: 50,
            crystals: 15,
            essences: vec![],
            capture_chance: 1.0, // Dev: 100%, Prod: 0.15
        },
        ExpeditionDuration::Medium => ExpeditionRewards {
            xp_per_monster: 120,
            crystals: 35,
            essences: vec![],
            capture_chance: 1.0, // Dev: 100%, Prod: 0.25
        },
        ExpeditionDuration::Long => ExpeditionRewards {
            xp_per_monster: 350,
            crystals: 90,
            essences: vec![],
            capture_chance: 1.0, // Dev: 100%, Prod: 0.40
        },
        ExpeditionDuration::Overnight => ExpeditionRewards {
            xp_per_monster: 600,
            crystals: 150,
            essences: vec![],
            capture_chance: 1.0, // Dev: 100%, Prod: 0.50
        },
    }
}

/// Calculate final rewards for an expedition
pub fn calculate_expedition_rewards(
    duration: ExpeditionDuration,
    map_essences: &[(Element, u8)],
    monster_count: usize,
) -> ExpeditionRewards {
    let mut rewards = get_base_rewards(duration);

    // Add map-specific essences
    rewards.essences = map_essences.to_vec();

    // Bonus for more monsters? (optional, not in GDD)
    // For now, same rewards regardless of monster count

    rewards
}
