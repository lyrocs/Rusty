//! Battle system
//!
//! Handles damage calculations, hit/miss, critical hits for hero vs enemy combat

use super::{Enemy, Hero};
use super::element_system::get_element_advantage;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Damage result from an attack
#[derive(Debug, Clone, Copy)]
pub struct DamageResult {
    pub damage: u32,
    pub is_critical: bool,
    pub is_miss: bool,
}

/// Battle state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    pub hero_last_attack: f64,
    pub enemy_last_attack: f64,
}

impl Default for BattleState {
    fn default() -> Self {
        Self {
            hero_last_attack: 0.0,
            enemy_last_attack: 0.0,
        }
    }
}

impl BattleState {
    /// Start a new battle
    pub fn start_battle(&mut self) {
        self.hero_last_attack = 0.0;
        self.enemy_last_attack = 0.0;
    }
}

/// Calculate damage with hit/miss and critical hit checks
///
/// # Arguments
/// * `attacker_atk` - Attacker's attack stat
/// * `attacker_hit` - Attacker's hit stat
/// * `attacker_crit_rate` - Attacker's critical rate (percentage)
/// * `defender_def` - Defender's defense stat
/// * `defender_flee` - Defender's flee/evasion stat
/// * `attacker_element` - Attacker's element type
/// * `defender_element` - Defender's element type
///
/// # Returns
/// DamageResult with damage amount, critical flag, and miss flag
pub fn calculate_damage(
    attacker_atk: u32,
    attacker_hit: u32,
    attacker_crit_rate: f32,
    defender_def: u32,
    defender_flee: u32,
    attacker_element: super::element_system::Element,
    defender_element: super::element_system::Element,
) -> DamageResult {
    let mut rng = rand::thread_rng();

    // Hit/Miss calculation
    // Hit rate = (attacker_hit / (attacker_hit + defender_flee)) * 100
    let hit_chance = if attacker_hit + defender_flee == 0 {
        95.0 // Default 95% if both stats are 0
    } else {
        ((attacker_hit as f32 / (attacker_hit + defender_flee) as f32) * 100.0)
            .min(95.0) // Cap at 95%
            .max(30.0) // Minimum 30% hit chance
    };

    let roll = rng.gen_range(0.0..100.0);
    if roll > hit_chance {
        // Miss!
        return DamageResult {
            damage: 0,
            is_critical: false,
            is_miss: true,
        };
    }

    // Critical hit check
    let is_critical = rng.gen_range(0.0..100.0) < attacker_crit_rate;

    // Base damage calculation
    // Damage = ATK * 2 - DEF (with variance)
    let base_damage = if attacker_atk * 2 > defender_def {
        attacker_atk * 2 - defender_def
    } else {
        1 // Minimum 1 damage
    };

    // Apply damage variance (85% to 100% of base damage)
    let variance = rng.gen_range(0.85..=1.0);
    let mut final_damage = (base_damage as f32 * variance) as u32;

    // Apply critical multiplier (1.5x damage)
    if is_critical {
        final_damage = (final_damage as f32 * 1.5) as u32;
    }

    // Apply element advantage
    let element_multiplier = get_element_advantage(attacker_element, defender_element);
    final_damage = (final_damage as f32 * element_multiplier) as u32;

    // Ensure minimum 1 damage
    final_damage = final_damage.max(1);

    DamageResult {
        damage: final_damage,
        is_critical,
        is_miss: false,
    }
}

/// Hero attacks enemy (basic attack)
pub fn hero_attack_enemy(hero: &Hero, enemy: &mut Enemy) -> DamageResult {
    let result = calculate_damage(
        hero.attack as u32,
        hero.hit as u32,
        hero.critical as f32,
        enemy.def,
        enemy.flee,
        hero.element,
        enemy.element,
    );

    if !result.is_miss {
        enemy.take_damage(result.damage);

        if result.is_critical {
            log::info!("💥 {} landed a CRITICAL HIT on {} for {} damage!",
                hero.name, enemy.name, result.damage);
        } else {
            log::info!("⚔️ {} attacked {} for {} damage",
                hero.name, enemy.name, result.damage);
        }
    } else {
        log::info!("❌ {} missed {}!", hero.name, enemy.name);
    }

    result
}

/// Enemy attacks hero (basic attack)
pub fn enemy_attack_hero(enemy: &Enemy, hero: &mut Hero) -> DamageResult {
    let result = calculate_damage(
        enemy.atk,
        enemy.hit,
        5.0, // Enemies have 5% base crit rate
        hero.defense as u32,
        hero.flee as u32,
        enemy.element,
        hero.element,
    );

    if !result.is_miss {
        hero.take_damage(result.damage as i32);

        if result.is_critical {
            log::info!("💥 {} landed a CRITICAL HIT on {} for {} damage!",
                enemy.name, hero.name, result.damage);
        } else {
            log::info!("⚔️ {} attacked {} for {} damage",
                enemy.name, hero.name, result.damage);
        }
    } else {
        log::info!("❌ {} missed {}!", enemy.name, hero.name);
    }

    result
}

/// Hero attacks enemy with battle state tracking (for auto-attack timing)
pub fn hero_attack_with_battle_state(
    hero: &Hero,
    enemy: &mut Enemy,
    battle_state: &BattleState,
) -> DamageResult {
    let result = calculate_damage(
        hero.attack as u32,
        hero.hit as u32,
        hero.critical as f32,
        enemy.def,
        enemy.flee,
        hero.element,
        enemy.element,
    );

    if !result.is_miss {
        enemy.take_damage(result.damage);

        if result.is_critical {
            log::debug!("💥 {} CRIT {} for {} damage (HP: {}/{})",
                hero.name, enemy.name, result.damage, enemy.current_hp, enemy.max_hp);
        } else {
            log::debug!("⚔️ {} → {} for {} damage (HP: {}/{})",
                hero.name, enemy.name, result.damage, enemy.current_hp, enemy.max_hp);
        }
    } else {
        log::debug!("❌ {} missed {}", hero.name, enemy.name);
    }

    result
}

/// Enemy attacks hero with battle state tracking (for auto-attack timing)
pub fn enemy_attack_with_battle_state(
    enemy: &Enemy,
    hero: &mut Hero,
    battle_state: &BattleState,
) -> DamageResult {
    let result = calculate_damage(
        enemy.atk,
        enemy.hit,
        5.0, // Enemies have 5% base crit rate
        hero.defense as u32,
        hero.flee as u32,
        enemy.element,
        hero.element,
    );

    if !result.is_miss {
        hero.take_damage(result.damage as i32);

        if result.is_critical {
            log::debug!("💥 {} CRIT {} for {} damage (HP: {}/{})",
                enemy.name, hero.name, result.damage, hero.current_health, hero.max_health);
        } else {
            log::debug!("⚔️ {} → {} for {} damage (HP: {}/{})",
                enemy.name, hero.name, result.damage, hero.current_health, hero.max_health);
        }
    } else {
        log::debug!("❌ {} missed {}", enemy.name, hero.name);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::element_system::Element;

    #[test]
    fn test_damage_calculation() {
        // Test basic damage
        let result = calculate_damage(
            100, // atk
            80,  // hit
            10.0, // crit rate
            30,  // def
            20,  // flee
            Element::Neutral,
            Element::Neutral,
        );

        // Should not miss with these stats
        assert!(!result.is_miss);
        // Damage should be positive
        assert!(result.damage > 0);
    }

    #[test]
    fn test_minimum_damage() {
        // Even with high defense, should deal at least 1 damage
        let result = calculate_damage(
            10,  // low atk
            50,  // hit
            0.0, // no crit
            1000, // very high def
            10,  // flee
            Element::Neutral,
            Element::Neutral,
        );

        if !result.is_miss {
            assert!(result.damage >= 1);
        }
    }
}
