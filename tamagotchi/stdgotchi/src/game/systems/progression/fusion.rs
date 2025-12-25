//! Fusion System
//!
//! Handles monster fusion when capturing duplicates.

use crate::game::core::Monster;
use crate::game::calculations::stats::MAX_FUSION;

/// Check if two monsters can be fused (same species)
pub fn can_fuse(monster1: &Monster, monster2: &Monster) -> bool {
    monster1.species_id == monster2.species_id && monster1.fusion_count < MAX_FUSION
}

/// Apply fusion bonus to a monster
/// Returns true if fusion was successful
pub fn apply_fusion(monster: &mut Monster) -> bool {
    if monster.fusion_count >= MAX_FUSION {
        return false;
    }

    monster.fusion_count += 1;

    // Stats will be recalculated when needed
    // The fusion bonus is applied through the stat calculation functions

    true
}

/// Get the fusion bonus percentage (for display)
/// Each fusion gives 10% bonus
pub fn fusion_bonus_percent(fusion_count: u8) -> u8 {
    (fusion_count.min(MAX_FUSION) * 10) as u8
}

/// Format fusion display string (e.g., "+3")
pub fn format_fusion(fusion_count: u8) -> String {
    if fusion_count > 0 {
        format!("+{}", fusion_count)
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_bonus_percent() {
        assert_eq!(fusion_bonus_percent(0), 0);
        assert_eq!(fusion_bonus_percent(1), 10);
        assert_eq!(fusion_bonus_percent(9), 90);
        assert_eq!(fusion_bonus_percent(10), 90); // Capped
    }

    #[test]
    fn test_format_fusion() {
        assert_eq!(format_fusion(0), "");
        assert_eq!(format_fusion(1), "+1");
        assert_eq!(format_fusion(9), "+9");
    }
}
