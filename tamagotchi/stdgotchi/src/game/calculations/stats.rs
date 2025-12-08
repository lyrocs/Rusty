//! Stats Calculations
//!
//! All stat-related calculations: base stats, level scaling, fusion bonuses, power rating.
//! This is the SINGLE SOURCE OF TRUTH for stat formulas.

/// Maximum fusion count
pub const MAX_FUSION: u8 = 9;

/// Fusion bonus per level (5% = 0.05)
pub const FUSION_BONUS_PER_LEVEL: f32 = 0.05;

/// Calculate stat with fusion bonus
/// Formula: stat_final = stat_base * (1 + fusion_count * 0.05)
pub fn apply_fusion_bonus(base_stat: u16, fusion_count: u8) -> u16 {
    let clamped_fusion = fusion_count.min(MAX_FUSION);
    let multiplier = 1.0 + (clamped_fusion as f32 * FUSION_BONUS_PER_LEVEL);
    (base_stat as f32 * multiplier).round() as u16
}

/// Calculate stat at a given level
/// Stats scale linearly with level (simple formula for now)
pub fn calculate_stat_at_level(base_stat: u16, level: u8) -> u16 {
    // Each level adds ~2% to base stat
    let level_multiplier = 1.0 + (level.saturating_sub(1) as f32 * 0.02);
    (base_stat as f32 * level_multiplier).round() as u16
}

/// Calculate final stat with level and fusion
pub fn calculate_final_stat(base_stat: u16, level: u8, fusion_count: u8) -> u16 {
    let level_stat = calculate_stat_at_level(base_stat, level);
    apply_fusion_bonus(level_stat, fusion_count)
}

/// Calculate HP at a given level (HP scales more than other stats)
pub fn calculate_hp_at_level(base_hp: u16, level: u8) -> u16 {
    // HP gains ~3% per level
    let level_multiplier = 1.0 + (level.saturating_sub(1) as f32 * 0.03);
    (base_hp as f32 * level_multiplier).round() as u16
}

/// Calculate final HP with level and fusion
pub fn calculate_final_hp(base_hp: u16, level: u8, fusion_count: u8) -> u16 {
    let level_hp = calculate_hp_at_level(base_hp, level);
    apply_fusion_bonus(level_hp, fusion_count)
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
    fn test_fusion_bonus() {
        assert_eq!(apply_fusion_bonus(100, 0), 100); // No fusion
        assert_eq!(apply_fusion_bonus(100, 1), 105); // +5%
        assert_eq!(apply_fusion_bonus(100, 9), 145); // +45% (max)
        assert_eq!(apply_fusion_bonus(100, 10), 145); // Clamped to max
    }

    #[test]
    fn test_power_calculation() {
        // Example from GDD: ATK=15, DEF=10, SPD=20, HP=80
        // power = 15 + 10 + 20 + (80/5) = 15 + 10 + 20 + 16 = 61
        assert_eq!(calculate_power(15, 10, 20, 80), 61);
    }
}
