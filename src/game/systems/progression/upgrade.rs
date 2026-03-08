//! Stat Upgrade System
//!
//! Handles upgrading monster stat bonuses using crystals.
//! Uses Pokemon EV-style bonuses: 0-50 points per stat.

use crate::game::core::MAX_STAT_BONUS;

/// Calculate crystal cost for a +1 bonus upgrade
/// Formula: cost = (current_bonus + 1) * 3
/// Lower bonus = cheaper to upgrade (starts at 3 crystals)
pub fn upgrade_cost_crystals(current_bonus: u8) -> u32 {
    ((current_bonus as u32 + 1) * 3)
}

/// Upgrade result
#[derive(Debug, Clone)]
pub enum UpgradeResult {
    Success,
    InsufficientCrystals,
    MaxBonusReached,
}

/// Check if upgrade is possible and return the cost
pub fn can_upgrade(current_bonus: u8, crystals_available: u32) -> Option<u32> {
    if current_bonus >= MAX_STAT_BONUS {
        return None;
    }
    let cost = upgrade_cost_crystals(current_bonus);
    if crystals_available >= cost {
        Some(cost)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_cost() {
        // Bonus 0 → 1: 3 crystals ((0+1) * 3 = 3)
        assert_eq!(upgrade_cost_crystals(0), 3);
        // Bonus 10 → 11: 33 crystals ((10+1) * 3 = 33)
        assert_eq!(upgrade_cost_crystals(10), 33);
        // Bonus 49 → 50: 150 crystals ((49+1) * 3 = 150)
        assert_eq!(upgrade_cost_crystals(49), 150);
    }

    #[test]
    fn test_can_upgrade() {
        // Can upgrade with enough crystals
        assert!(can_upgrade(0, 10).is_some());
        // Cannot upgrade at max
        assert!(can_upgrade(50, 1000).is_none());
        // Cannot upgrade without enough crystals
        assert!(can_upgrade(10, 10).is_none()); // needs 33
    }
}
