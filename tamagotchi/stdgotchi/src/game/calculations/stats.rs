//! Stats Calculations
//!
//! All stat-related calculations: base stats, level scaling, fusion bonuses, power rating.
//! This is the SINGLE SOURCE OF TRUTH for stat formulas.
//!
//! Formula:
//!   Level_Multi = 1 + (Level - 1) × 0.04
//!   Tier_Multi = 1 + Tier × 0.10 (where Tier = fusion_count)
//!   Stat = Base × Level_Multi × Tier_Multi

/// Maximum fusion count (tier)
pub const MAX_FUSION: u8 = 9;

/// Level bonus per level (4% = 0.04)
pub const LEVEL_BONUS_PER_LEVEL: f32 = 0.04;

/// Tier/Fusion bonus per level (10% = 0.10)
pub const TIER_BONUS_PER_FUSION: f32 = 0.10;

/// Calculate level multiplier
/// Formula: Level_Multi = 1 + (Level - 1) × 0.04
pub fn calculate_level_multiplier(level: u8) -> f32 {
    1.0 + (level.saturating_sub(1) as f32 * LEVEL_BONUS_PER_LEVEL)
}

/// Calculate tier/fusion multiplier
/// Formula: Tier_Multi = 1 + Tier × 0.10
pub fn calculate_tier_multiplier(fusion_count: u8) -> f32 {
    let clamped_fusion = fusion_count.min(MAX_FUSION);
    1.0 + (clamped_fusion as f32 * TIER_BONUS_PER_FUSION)
}

/// Calculate final stat with level and fusion
/// Formula: Stat = Base × Level_Multi × Tier_Multi
pub fn calculate_final_stat(base_stat: u16, level: u8, fusion_count: u8) -> u16 {
    let level_multi = calculate_level_multiplier(level);
    let tier_multi = calculate_tier_multiplier(fusion_count);
    (base_stat as f32 * level_multi * tier_multi).round() as u16
}

/// Calculate final HP with level and fusion (same formula as other stats)
/// Formula: HP = Base_HP × Level_Multi × Tier_Multi
pub fn calculate_final_hp(base_hp: u16, level: u8, fusion_count: u8) -> u16 {
    calculate_final_stat(base_hp, level, fusion_count)
}

/// Calculate power rating for display
/// Formula: power = ATK + DEF + SPD + (HP / 5)
pub fn calculate_power(atk: u16, def: u16, spd: u16, hp_max: u16) -> u16 {
    atk + def + spd + (hp_max / 5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_multiplier() {
        // Level_Multi = 1 + (Level - 1) × 0.04
        assert_eq!(calculate_level_multiplier(1), 1.0);   // Level 1: 1 + 0 × 0.04 = 1.0
        assert_eq!(calculate_level_multiplier(10), 1.36); // Level 10: 1 + 9 × 0.04 = 1.36
        assert_eq!(calculate_level_multiplier(26), 2.0);  // Level 26: 1 + 25 × 0.04 = 2.0
    }

    #[test]
    fn test_tier_multiplier() {
        // Tier_Multi = 1 + Tier × 0.10
        assert_eq!(calculate_tier_multiplier(0), 1.0);  // No fusion
        assert_eq!(calculate_tier_multiplier(1), 1.1);  // +10%
        assert_eq!(calculate_tier_multiplier(9), 1.9);  // +90% (max)
        assert_eq!(calculate_tier_multiplier(10), 1.9); // Clamped to max
    }

    #[test]
    fn test_final_stat() {
        // Stat = Base × Level_Multi × Tier_Multi
        // Level 1, Fusion 0: 100 × 1.0 × 1.0 = 100
        assert_eq!(calculate_final_stat(100, 1, 0), 100);
        // Level 1, Fusion 1: 100 × 1.0 × 1.10 = 110
        assert_eq!(calculate_final_stat(100, 1, 1), 110);
        // Level 1, Fusion 9: 100 × 1.0 × 1.90 = 190
        assert_eq!(calculate_final_stat(100, 1, 9), 190);
        // Level 10, Fusion 0: 100 × 1.36 × 1.0 = 136
        assert_eq!(calculate_final_stat(100, 10, 0), 136);
        // Level 10, Fusion 1: 100 × 1.36 × 1.10 = 149.6 ≈ 150
        assert_eq!(calculate_final_stat(100, 10, 1), 150);
    }

    #[test]
    fn test_power_calculation() {
        // Example from GDD: ATK=15, DEF=10, SPD=20, HP=80
        // power = 15 + 10 + 20 + (80/5) = 15 + 10 + 20 + 16 = 61
        assert_eq!(calculate_power(15, 10, 20, 80), 61);
    }
}
