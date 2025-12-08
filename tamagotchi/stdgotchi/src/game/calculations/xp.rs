//! XP and Leveling Calculations
//!
//! All XP-related calculations: XP needed per level, level up checks.
//! This is the SINGLE SOURCE OF TRUTH for XP formulas.

/// Maximum monster level
pub const MAX_LEVEL: u8 = 99;

/// Calculate XP needed to reach the next level
/// Formula: xp_needed = level * 100
pub fn xp_for_next_level(current_level: u8) -> u32 {
    current_level as u32 * 100
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
    fn test_xp_formula() {
        assert_eq!(xp_for_next_level(1), 100);   // Level 1 → 2: 100 XP
        assert_eq!(xp_for_next_level(10), 1000); // Level 10 → 11: 1000 XP
        assert_eq!(xp_for_next_level(50), 5000); // Level 50 → 51: 5000 XP
    }

    #[test]
    fn test_level_up() {
        // Should level up
        assert_eq!(check_level_up(1, 100, 100), Some(2));
        assert_eq!(check_level_up(1, 150, 100), Some(2));

        // Should not level up
        assert_eq!(check_level_up(1, 50, 100), None);

        // Max level
        assert_eq!(check_level_up(99, 10000, 9900), None);
    }

    #[test]
    fn test_apply_xp_gain() {
        // Single level up
        let (level, xp, xp_to_next, gained) = apply_xp_gain(1, 0, 150);
        assert_eq!(level, 2);
        assert_eq!(xp, 50); // 150 - 100
        assert_eq!(xp_to_next, 200); // 2 * 100
        assert_eq!(gained, 1);

        // Multiple level ups
        let (level, xp, xp_to_next, gained) = apply_xp_gain(1, 0, 350);
        assert_eq!(level, 3); // 100 + 200 = 300, leftover 50
        assert_eq!(xp, 50);
        assert_eq!(xp_to_next, 300);
        assert_eq!(gained, 2);
    }
}
