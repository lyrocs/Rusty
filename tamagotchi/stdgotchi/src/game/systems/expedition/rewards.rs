//! Expedition Rewards
//!
//! Calculates rewards from completed expeditions.

use super::ExpeditionDuration;
use crate::game::core::Element;
use crate::game::calculations::xp::XP_MULTIPLIER;

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

/// Base XP values before multiplier (kept low, multiplier applied at runtime)
const BASE_XP_SHORT: u32 = 5;
const BASE_XP_MEDIUM: u32 = 12;
const BASE_XP_LONG: u32 = 35;
const BASE_XP_OVERNIGHT: u32 = 60;

/// Get base rewards for a duration
/// Based on GDD table 2.2.3
/// XP values are multiplied by XP_MULTIPLIER (same as enemy kills)
/// NOTE: Dev capture_chance is 1.0 (100%). Production values: 0.15/0.25/0.40/0.50
pub fn get_base_rewards(duration: ExpeditionDuration) -> ExpeditionRewards {
    match duration {
        ExpeditionDuration::Short => ExpeditionRewards {
            xp_per_monster: BASE_XP_SHORT * XP_MULTIPLIER as u32,
            crystals: 15,
            essences: vec![],
            capture_chance: 1.0, // Dev: 100%, Prod: 0.15
        },
        ExpeditionDuration::Medium => ExpeditionRewards {
            xp_per_monster: BASE_XP_MEDIUM * XP_MULTIPLIER as u32,
            crystals: 35,
            essences: vec![],
            capture_chance: 1.0, // Dev: 100%, Prod: 0.25
        },
        ExpeditionDuration::Long => ExpeditionRewards {
            xp_per_monster: BASE_XP_LONG * XP_MULTIPLIER as u32,
            crystals: 90,
            essences: vec![],
            capture_chance: 1.0, // Dev: 100%, Prod: 0.40
        },
        ExpeditionDuration::Overnight => ExpeditionRewards {
            xp_per_monster: BASE_XP_OVERNIGHT * XP_MULTIPLIER as u32,
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
