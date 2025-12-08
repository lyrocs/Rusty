//! Stat Upgrade System
//!
//! Handles upgrading monster stats using crystals and essences.

use crate::game::core::Element;

/// Calculate crystal cost for a +1 stat upgrade
/// Formula: cost = (current_stat / 10) * 5
pub fn upgrade_cost_crystals(current_stat: u16) -> u32 {
    ((current_stat / 10) * 5) as u32
}

/// Calculate cost for major upgrade (+10 stats)
/// Requires crystals + essences of the monster's element
pub fn major_upgrade_cost(current_stat: u16) -> (u32, u8) {
    let crystal_cost = upgrade_cost_crystals(current_stat) * 10; // 10x normal cost
    let essence_cost = 5u8; // 5 essences of the monster's element
    (crystal_cost, essence_cost)
}

/// Upgrade result
#[derive(Debug, Clone)]
pub enum UpgradeResult {
    Success,
    InsufficientCrystals,
    InsufficientEssences,
    MaxStatReached,
}

/// Maximum stat value
pub const MAX_STAT: u16 = 999;

/// Check if upgrade is possible and return the cost
pub fn can_upgrade(current_stat: u16, crystals_available: u32) -> Option<u32> {
    if current_stat >= MAX_STAT {
        return None;
    }
    let cost = upgrade_cost_crystals(current_stat);
    if crystals_available >= cost {
        Some(cost)
    } else {
        None
    }
}

/// Check if major upgrade is possible
pub fn can_major_upgrade(
    current_stat: u16,
    crystals_available: u32,
    essences_available: u16,
) -> Option<(u32, u8)> {
    if current_stat + 10 > MAX_STAT {
        return None;
    }
    let (crystal_cost, essence_cost) = major_upgrade_cost(current_stat);
    if crystals_available >= crystal_cost && essences_available >= essence_cost as u16 {
        Some((crystal_cost, essence_cost))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_cost() {
        // ATK 50 → 51: 25 crystals (50/10 * 5 = 25)
        assert_eq!(upgrade_cost_crystals(50), 25);
        // ATK 100 → 101: 50 crystals
        assert_eq!(upgrade_cost_crystals(100), 50);
    }

    #[test]
    fn test_major_upgrade_cost() {
        let (crystals, essences) = major_upgrade_cost(50);
        assert_eq!(crystals, 250); // 25 * 10
        assert_eq!(essences, 5);
    }
}
