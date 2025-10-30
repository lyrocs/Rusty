/// Farm efficiency system
///
/// Power calculation and efficiency rating for auto-farming

use crate::hero::Hero;
use crate::combat::{Enemy, EfficiencyRating, FarmDuration};

/// Calculate hero power for farming efficiency
///
/// Power formula:
/// - Base ATK from equipment (most important - determines kill speed)
/// - STR (main stat for physical damage)
/// - AGI (affects attack speed)
/// - DEX (affects hit rate)
/// - LUK (affects critical rate)
pub fn calculate_hero_power(hero: &Hero) -> f32 {
    // Calculate ATK from equipment
    let equipment_atk = hero.equipped_weapon.total_atk() as f32;

    // Base stats contribute to power
    let str_contribution = hero.base_str as f32 * 2.0;  // STR is main stat for ATK
    let agi_contribution = hero.base_agi as f32 * 0.5; // AGI affects speed
    let dex_contribution = hero.base_dex as f32 * 0.5; // DEX affects accuracy
    let luk_contribution = hero.base_luk as f32 * 0.3; // LUK affects crit

    // Total power
    equipment_atk + str_contribution + agi_contribution + dex_contribution + luk_contribution
}

/// Calculate enemy power for farming efficiency
///
/// Power formula:
/// - Enemy HP (main factor - how long it takes to kill)
/// - Enemy DEF (reduces effective damage)
/// - Enemy ATK (threat level, minor factor)
pub fn calculate_enemy_power(enemy: &Enemy) -> f32 {
    let hp_factor = enemy.max_hp as f32;

    // Defense reduces effective damage (1 DEF = ~1% damage reduction, capped)
    let def_factor = 1.0 + (enemy.defense as f32 * 0.01).min(0.5);

    // ATK represents threat (minor contribution to power rating)
    let threat_factor = (enemy.attack as f32) * 0.1;

    // Total power: HP is main component, scaled by defense and threat
    (hp_factor * def_factor) + threat_factor
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

/// Calculate total rewards for a farm session
///
/// Returns (total_exp, total_zeny)
pub fn calculate_farm_rewards(
    enemy: &Enemy,
    actual_kills: u16,
) -> (u32, u32) {
    let total_exp = (enemy.base_exp as u32) * (actual_kills as u32);
    let total_zeny = (enemy.zeny_reward as u32) * (actual_kills as u32);

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
