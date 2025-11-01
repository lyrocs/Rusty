/// IDLE farming calculator
///
/// Calculates farming rates (kills/min, zeny/min, damage/min, regen/min)
/// based on hero and enemy stats.

use crate::combat::Enemy;
use crate::hero::Hero;

/// Simple ceiling function for f32 (no_std compatible)
fn ceil(x: f32) -> f32 {
    let truncated = x as i32 as f32;
    if x > truncated {
        truncated + 1.0
    } else {
        truncated
    }
}

/// Farming rates calculated for IDLE farming
pub struct FarmingRates {
    pub kills_per_minute: f32,
    pub zeny_per_minute: f32,
    pub damage_per_minute: f32,
    pub regen_per_minute: f32,
    pub net_hp_per_minute: f32,  // regen - damage
}

/// Calculate farming rates based on hero and enemy stats
pub fn calculate_farming_rates(hero: &Hero, enemy: &Enemy) -> FarmingRates {
    // Calculate hero's effective attack (base stat + equipment)
    let hero_atk = calculate_hero_attack(hero);

    // Calculate hero's attack speed (attacks per second)
    // Base: 1 attack per second, AGI increases this
    let hero_agi = calculate_hero_agi(hero);
    let attack_speed = 1.0 + (hero_agi as f32 * 0.01); // +1% per AGI

    // Calculate time to kill one enemy
    let damage_per_hit = if hero_atk > enemy.defense {
        (hero_atk - enemy.defense) as f32
    } else {
        1.0 // Minimum damage
    };

    let hits_to_kill = ceil(enemy.max_hp as f32 / damage_per_hit);
    let seconds_to_kill = hits_to_kill / attack_speed;

    // Calculate kills per minute
    let kills_per_minute = if seconds_to_kill > 0.0 {
        60.0 / seconds_to_kill
    } else {
        60.0 // Default if calculation fails
    };

    // Calculate zeny per minute
    let zeny_per_minute = kills_per_minute * enemy.zeny_reward as f32;

    // Calculate damage taken per minute
    // Enemy attacks once per kill cycle (simplification)
    let hero_def = calculate_hero_defense(hero);
    let enemy_damage_per_hit = if enemy.attack > hero_def {
        (enemy.attack - hero_def) as f32
    } else {
        1.0 // Minimum damage
    };
    let damage_per_minute = kills_per_minute * enemy_damage_per_hit * 0.5; // 50% chance to take hit per kill

    // Calculate HP regen per minute
    // Base: 1 HP/min, VIT increases this
    let hero_vit = calculate_hero_vit(hero);
    let regen_per_minute = 1.0 + (hero_vit as f32 * 0.5); // +0.5 HP/min per VIT

    // Calculate net HP change
    let net_hp_per_minute = regen_per_minute - damage_per_minute;

    FarmingRates {
        kills_per_minute,
        zeny_per_minute,
        damage_per_minute,
        regen_per_minute,
        net_hp_per_minute,
    }
}

/// Calculate hero's total attack (base STR + equipment ATK)
fn calculate_hero_attack(hero: &Hero) -> u16 {
    let base_atk = hero.base_str * 2; // STR contributes to ATK
    let weapon_atk = hero.equipped_weapon.atk_bonus;
    let equip_atk = hero.equipped_armor.atk_bonus
        + hero.equipped_shoes.atk_bonus
        + hero.equipped_garment.atk_bonus
        + hero.equipped_accessory1.atk_bonus
        + hero.equipped_accessory2.atk_bonus;

    base_atk + weapon_atk + equip_atk
}

/// Calculate hero's total defense (equipment DEF + VIT bonus)
fn calculate_hero_defense(hero: &Hero) -> u16 {
    let base_def = hero.base_vit / 2; // VIT contributes to DEF
    let equip_def = hero.equipped_weapon.def_bonus
        + hero.equipped_armor.def_bonus
        + hero.equipped_shoes.def_bonus
        + hero.equipped_garment.def_bonus
        + hero.equipped_accessory1.def_bonus
        + hero.equipped_accessory2.def_bonus;

    base_def + equip_def
}

/// Calculate hero's total AGI (base + equipment bonuses)
fn calculate_hero_agi(hero: &Hero) -> u16 {
    let equip_agi = hero.equipped_weapon.agi_bonus as i16
        + hero.equipped_armor.agi_bonus as i16
        + hero.equipped_shoes.agi_bonus as i16
        + hero.equipped_garment.agi_bonus as i16
        + hero.equipped_accessory1.agi_bonus as i16
        + hero.equipped_accessory2.agi_bonus as i16;

    ((hero.base_agi as i16) + equip_agi).max(1) as u16
}

/// Calculate hero's total VIT (base + equipment bonuses)
fn calculate_hero_vit(hero: &Hero) -> u16 {
    let equip_vit = hero.equipped_weapon.vit_bonus as i16
        + hero.equipped_armor.vit_bonus as i16
        + hero.equipped_shoes.vit_bonus as i16
        + hero.equipped_garment.vit_bonus as i16
        + hero.equipped_accessory1.vit_bonus as i16
        + hero.equipped_accessory2.vit_bonus as i16;

    ((hero.base_vit as i16) + equip_vit).max(1) as u16
}

/// Estimate time until death based on current HP and net HP change rate
pub fn estimate_time_until_death(current_hp: u16, net_hp_per_minute: f32) -> Option<u32> {
    if net_hp_per_minute >= 0.0 {
        // Hero will not die (regen >= damage)
        None
    } else {
        // Calculate time until HP reaches 0
        let minutes_until_death = current_hp as f32 / net_hp_per_minute.abs();
        Some((minutes_until_death * 60_000.0) as u32) // Convert to milliseconds
    }
}

/// Calculate recommended level for a map based on average enemy level
pub fn calculate_recommended_level(enemy_level: u16) -> &'static str {
    let min_level = if enemy_level > 5 { enemy_level - 5 } else { 1 };
    let max_level = enemy_level + 5;

    // Return static string for display
    match (min_level, max_level) {
        (1, max) if max <= 10 => "1-10",
        (min, max) if max <= 20 => if min <= 5 { "5-20" } else { "10-20" },
        (min, max) if max <= 30 => if min <= 15 { "15-30" } else { "20-30" },
        (min, _) if min >= 25 => "25+",
        _ => "10-25",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_farming_rates_calculation() {
        // Test that farming rates are calculated correctly
        // This is a placeholder - actual tests would use real hero/enemy data
    }
}
