/// Damage calculation system
///
/// Contains formulas for calculating damage in JRPG battles with
/// variance, criticals, lucky strikes, and miss chances.

use super::models::CombatResult;

/// Calculate damage for JRPG battles with variance, crits, lucky strikes, and miss chance
pub fn calculate_jrpg_damage(
    attacker_atk: u16,
    attacker_luck: u16,
    attacker_dex: u16,
    defender_def: u16,
    defender_agi: u16,
    rng_value: u8, // 0-255 random value
) -> (u16, CombatResult) {
    // Calculate hit chance based on DEX vs AGI
    // Base hit rate: 80%
    // +1% hit per 5 DEX difference
    // -1% hit per 5 AGI difference
    let dex_bonus = (attacker_dex as i32) / 5;
    let agi_penalty = (defender_agi as i32) / 5;
    let hit_rate = 80 + dex_bonus - agi_penalty;
    let hit_rate = hit_rate.clamp(20, 95) as u16; // Min 20%, Max 95%

    // Check if attack hits
    let hit_roll = (rng_value as u16 * 100) / 255;
    if hit_roll >= hit_rate {
        // Miss!
        return (0, CombatResult::Miss);
    }

    // Base damage calculation
    let base_damage = if attacker_atk > defender_def {
        attacker_atk - (defender_def / 2)
    } else {
        1 // Minimum 1 damage
    };

    // Check for critical hit (LUK-based)
    // Base crit rate: 1%
    // +1% per 10 LUK
    let crit_rate = 1 + (attacker_luck / 10);
    let crit_roll = rng_value % 100;
    if crit_roll < crit_rate as u8 {
        // Critical hit! (2x damage)
        return ((base_damage * 2).max(1), CombatResult::Critical);
    }

    // Check for lucky strike (LUK-based, separate from crit)
    // Base lucky rate: 0.5%
    // +0.5% per 10 LUK
    let lucky_rate = 1 + (attacker_luck / 20); // 0.5% per 10 LUK (scaled for 0-255)
    let lucky_roll = rng_value % 200; // 0-199 for finer granularity
    if lucky_roll < lucky_rate as u8 {
        // Lucky strike! (3x damage)
        return ((base_damage * 3).max(1), CombatResult::Lucky);
    }

    // Variance: ±10% damage
    let variance = ((rng_value as i32 % 21) - 10) as i16; // -10 to +10
    let damage_with_variance = (base_damage as i32 + (base_damage as i32 * variance as i32 / 100)).max(1) as u16;

    (damage_with_variance, CombatResult::Normal)
}
