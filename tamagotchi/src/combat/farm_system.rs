/// Farm efficiency system
///
/// Power calculation and efficiency rating for auto-farming

use crate::hero::Hero;
use crate::combat::{Enemy, EfficiencyRating, FarmDuration};

/// Calculate hero power for farming efficiency
///
/// Improved power formula that better reflects combat capability:
/// - Hero level (baseline power - 5 points per level)
/// - Equipment ATK (weighted 3x - determines damage output)
/// - STR (main damage stat - 2 points per STR)
/// - HP (survivability - 1 point per 20 HP)
/// - AGI/DEX/LUK (minor contributions)
///
/// Example: Level 23 Swordman with 50 STR, Iron Blade (18 ATK), ~400 HP:
/// - Level: 23 × 5 = 115
/// - Equipment: 18 × 3 = 54
/// - STR: 52 × 2 = 104 (50 base + 2 from weapon)
/// - HP: 400 / 20 = 20
/// - Other: ~7
/// - Total: ~300 power
pub fn calculate_hero_power(hero: &Hero) -> f32 {
    // Level is the foundation of power (Level 23 = 115 power)
    let level_power = (hero.level as f32) * 5.0;

    // Equipment ATK is critical (weighted 3x to reflect importance)
    let equipment_atk = hero.equipped_weapon.total_atk() as f32 * 3.0;

    // STR is main damage stat
    let str_power = hero.base_str as f32 * 2.0;

    // HP contributes to survivability
    let hp_power = (hero.max_hp as f32) / 20.0;

    // Other stats have minor contributions
    let agi_power = hero.base_agi as f32 * 0.5;
    let dex_power = hero.base_dex as f32 * 0.5;
    let luk_power = hero.base_luk as f32 * 0.3;

    // Total power
    level_power + equipment_atk + str_power + hp_power + agi_power + dex_power + luk_power
}

/// Calculate enemy power for farming efficiency
///
/// Improved power formula that better balances enemy stats:
/// - Enemy level (baseline power - 5 points per level)
/// - HP (scaled down by 4x to prevent overweighting)
/// - ATK (threat level - 2 points per ATK)
/// - DEF (damage mitigation - 1.5 points per DEF)
///
/// Example: Thief Bug Level 21 (354 HP, 84 ATK, 24 DEF):
/// - Level: 21 × 5 = 105
/// - HP: 354 / 4 = 88.5
/// - ATK: 84 × 2 = 168
/// - DEF: 24 × 1.5 = 36
/// - Total: ~398 power
///
/// Power ratio vs Level 23 Swordman: 300/398 = 0.75 (Risky) ✓
pub fn calculate_enemy_power(enemy: &Enemy) -> f32 {
    // Level is the foundation of power (Level 21 = 105 power)
    let level_power = (enemy.level as f32) * 5.0;

    // HP scaled down to prevent overweighting (354 HP = 88.5 power)
    let hp_power = (enemy.max_hp as f32) / 4.0;

    // ATK represents threat to hero (84 ATK = 168 power)
    let atk_power = (enemy.attack as f32) * 2.0;

    // DEF makes enemy harder to kill (24 DEF = 36 power)
    let def_power = (enemy.defense as f32) * 1.5;

    // Total power
    level_power + hp_power + atk_power + def_power
}

/// Calculate efficiency rating for hero vs enemy
///
/// Returns (EfficiencyRating, power_ratio, hero_power, enemy_power)
pub fn calculate_efficiency(hero: &Hero, enemy: &Enemy) -> (EfficiencyRating, f32, f32, f32) {
    let hero_power = calculate_hero_power(hero);
    let enemy_power = calculate_enemy_power(enemy);

    // Avoid division by zero
    let power_ratio = if enemy_power > 0.0 {
        hero_power / enemy_power
    } else {
        10.0 // If enemy has no power, hero is extremely powerful
    };

    let rating = EfficiencyRating::from_power_ratio(power_ratio);

    (rating, power_ratio, hero_power, enemy_power)
}

/// Calculate expected kills for a farm session
///
/// Base formula:
/// - Baseline: 10 kills per minute (Fair efficiency)
/// - Multiply by efficiency rating multiplier
/// - Multiply by duration multiplier
pub fn calculate_expected_kills(
    rating: EfficiencyRating,
    duration: FarmDuration,
) -> u16 {
    const BASE_KILLS_PER_MINUTE: f32 = 10.0;

    let efficiency_mult = rating.multiplier();
    let duration_mult = duration.total_multiplier();

    let kills = BASE_KILLS_PER_MINUTE * efficiency_mult * duration_mult;

    // Convert to integer, minimum 1 kill (add 0.5 for rounding effect)
    ((kills + 0.5) as u16).max(1)
}

/// Calculate time between kill ticks in milliseconds
///
/// This determines how often the "kill tick" happens during farming
pub fn calculate_kill_tick_interval(
    expected_kills: u16,
    duration_ms: u32,
) -> u32 {
    if expected_kills == 0 {
        return duration_ms; // If no kills expected, never tick
    }

    // Interval = total duration / number of kills
    let interval = duration_ms / (expected_kills as u32);

    // Minimum interval of 1 second to avoid too frequent updates
    interval.max(1000)
}

/// Calculate level difference EXP penalty/bonus multiplier
///
/// Based on iRO wiki formula (simplified version)
/// Returns a multiplier between 0.10 and 1.40
///
/// Level difference = enemy_level - hero_level
pub fn calculate_level_penalty(hero_level: u16, enemy_level: u16) -> f32 {
    let level_diff = (enemy_level as i32) - (hero_level as i32);

    match level_diff {
        // Enemy much lower level (severe penalty)
        i32::MIN..=-31 => 0.10,
        -30..=-26 => 0.25,
        -25..=-21 => 0.60,
        -20..=-16 => 0.70,
        -15..=-11 => 0.85,
        -10..=-6 => 0.95,
        -5..=-1 => 1.00,

        // Same level
        0 => 1.00,

        // Enemy higher level (bonus)
        1..=5 => 1.15,
        6..=10 => 1.40,  // Maximum bonus at +10
        11..=15 => 1.15,

        // Enemy much higher level (reduced bonus)
        16..=i32::MAX => 0.40,
    }
}

/// Calculate total rewards for a farm session (AUTO FARM)
///
/// AUTO FARM penalties:
/// - 1/10 reward rate (10% of manual kills)
/// - Level difference penalty applied
///
/// Returns (total_exp, total_zeny)
pub fn calculate_farm_rewards(
    enemy: &Enemy,
    actual_kills: u16,
    hero_level: u16,
) -> (u32, u32) {
    const AUTO_FARM_RATE: f32 = 0.10; // 1/10 of manual kill rewards

    // Calculate level penalty
    let level_mult = calculate_level_penalty(hero_level, enemy.level);

    // Calculate base rewards per kill
    let exp_per_kill = (enemy.base_exp as f32) * level_mult * AUTO_FARM_RATE;
    let zeny_per_kill = (enemy.zeny_reward as f32) * AUTO_FARM_RATE;

    // Total rewards
    let total_exp = (exp_per_kill * (actual_kills as f32) + 0.5) as u32;
    let total_zeny = (zeny_per_kill * (actual_kills as f32) + 0.5) as u32;

    (total_exp, total_zeny)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hero_power_calculation() {
        let hero = Hero::new();
        let power = calculate_hero_power(&hero);
        assert!(power > 0.0, "Hero power should be positive");
    }

    #[test]
    fn test_enemy_power_calculation() {
        let enemy = Enemy {
            id: 1002,
            name: "Poring",
            level: 5,
            hp: 100,
            max_hp: 100,
            attack: 10,
            defense: 5,
            base_exp: 50,
            zeny_reward: 5,
        };
        let power = calculate_enemy_power(&enemy);
        assert!(power > 0.0, "Enemy power should be positive");
    }

    #[test]
    fn test_efficiency_rating() {
        let hero = Hero::new();
        let weak_enemy = Enemy {
            id: 1002,
            name: "Poring",
            level: 1,
            hp: 10,
            max_hp: 10,
            attack: 1,
            defense: 0,
            base_exp: 10,
            zeny_reward: 1,
        };

        let (rating, ratio, _, _) = calculate_efficiency(&hero, &weak_enemy);
        assert!(ratio > 1.0, "Hero should be stronger than weak enemy");
        assert!(rating != EfficiencyRating::Impossible, "Should be possible to farm weak enemy");
    }

    #[test]
    fn test_expected_kills() {
        let kills = calculate_expected_kills(
            EfficiencyRating::Excellent,
            FarmDuration::OneMinute
        );
        assert!(kills > 0, "Should expect at least 1 kill");
        assert!(kills >= 20, "Excellent rating should give many kills per minute");
    }
}
