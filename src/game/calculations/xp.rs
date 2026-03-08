//! XP and Leveling Calculations
//!
//! All XP-related calculations: XP needed per level, level up checks.
//! Uses exp_table.json for RO-accurate leveling curve.

use crate::game::data_loader::get_exp_to_next_level;

/// Maximum monster level
pub const MAX_LEVEL: u8 = 99;

/// XP multiplier applied to enemy base_exp rewards
/// Base values from RO database are kept in JSON, this multiplier is applied at runtime
pub const XP_MULTIPLIER: u64 = 10;

/// Calculate actual XP reward from base_exp (applies multiplier)
pub fn calculate_exp_reward(base_exp: u64) -> u64 {
    base_exp * XP_MULTIPLIER
}

/// Calculate XP needed to reach the next level
/// Uses the global exp table loaded from exp_table.json
pub fn xp_for_next_level(current_level: u8) -> u32 {
    get_exp_to_next_level(current_level as u32)
}

/// Check if monster should level up
/// Returns the new level if level up occurred, None otherwise
pub fn check_level_up(current_level: u8, current_xp: u32, xp_to_next: u32) -> Option<u8> {
    if current_level >= MAX_LEVEL {
        return None;
    }
    if current_xp >= xp_to_next {
        Some(current_level + 1)
    } else {
        None
    }
}

/// Calculate remaining XP after level up
pub fn remaining_xp_after_level_up(current_xp: u32, xp_to_next: u32) -> u32 {
    current_xp.saturating_sub(xp_to_next)
}

/// Apply XP gain and handle multiple level ups
/// Returns (new_level, new_xp, new_xp_to_next, levels_gained)
pub fn apply_xp_gain(
    mut level: u8,
    mut xp: u32,
    xp_gained: u32,
) -> (u8, u32, u32, u8) {
    xp += xp_gained;
    let start_level = level;

    while level < MAX_LEVEL {
        let xp_needed = xp_for_next_level(level);
        if xp >= xp_needed {
            xp -= xp_needed;
            level += 1;
        } else {
            break;
        }
    }

    // Cap XP at level 99
    if level >= MAX_LEVEL {
        xp = 0;
    }

    let xp_to_next = if level >= MAX_LEVEL {
        0
    } else {
        xp_for_next_level(level)
    };

    (level, xp, xp_to_next, level - start_level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_up() {
        // Should level up (using exp table values - level 1 needs 548 exp)
        assert_eq!(check_level_up(1, 548, 548), Some(2));
        assert_eq!(check_level_up(1, 600, 548), Some(2));

        // Should not level up
        assert_eq!(check_level_up(1, 500, 548), None);

        // Max level
        assert_eq!(check_level_up(99, 10000, 0), None);
    }
}
