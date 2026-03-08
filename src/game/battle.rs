//! Battle system (Legacy)
//!
//! Basic damage calculations for the existing battle system.
//! NOTE: This is a simplified version for Phase 1 migration.
//! The new real-time combat system will be in game::systems::combat.

use super::Enemy;
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

/// Battle state tracking (simplified)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BattleState {
    pub hero_last_attack: f64,
    pub enemy_last_attack: f64,
    /// Current turn number
    #[serde(default)]
    pub turn_number: u32,
}

impl BattleState {
    /// Start a new battle
    pub fn start_battle(&mut self) {
        self.hero_last_attack = 0.0;
        self.enemy_last_attack = 0.0;
        self.turn_number = 0;
    }

    /// Increment turn number
    pub fn next_turn(&mut self) {
        self.turn_number += 1;
    }
}

/// Calculate damage from attacker to defender
pub fn calculate_damage(
    attacker_atk: u32,
    attacker_hit: u32,
    attacker_crit_rate: f32,
    defender_def: u32,
    defender_flee: u32,
) -> DamageResult {
    let mut rng = rand::thread_rng();

    // Check for miss (hit vs flee)
    let hit_roll = rng.gen_range(0..100);
    let hit_chance = calculate_hit_chance(attacker_hit, defender_flee);

    if hit_roll > hit_chance {
        return DamageResult {
            damage: 0,
            is_critical: false,
            is_miss: true,
        };
    }

    // Check for critical hit
    let crit_roll: f32 = rng.gen_range(0.0..100.0);
    let is_critical = crit_roll < attacker_crit_rate;

    // Calculate base damage
    let raw_damage = if attacker_atk > defender_def {
        attacker_atk - defender_def
    } else {
        1 // Minimum 1 damage
    };

    // Add variance (80% to 120% of base damage)
    let variance: f32 = rng.gen_range(0.8..1.2);
    let mut final_damage = (raw_damage as f32 * variance) as u32;

    // Apply critical multiplier
    if is_critical {
        final_damage = (final_damage as f32 * 2.0) as u32;
    }

    // Ensure at least 1 damage on hit
    final_damage = final_damage.max(1);

    DamageResult {
        damage: final_damage,
        is_critical,
        is_miss: false,
    }
}

/// Calculate hit chance (0-100)
fn calculate_hit_chance(attacker_hit: u32, defender_flee: u32) -> u32 {
    let base_hit = 80; // 80% base hit rate
    let hit_bonus = (attacker_hit as i32 - defender_flee as i32) / 2;
    let final_hit = (base_hit + hit_bonus).clamp(20, 95); // Clamp between 20% and 95%
    final_hit as u32
}

/// Generic attack function using stats directly
pub fn generic_attack(
    attacker_atk: u32,
    attacker_hit: u32,
    attacker_crit_rate: f32,
    attacker_element: super::element_system::Element,
    defender: &mut Enemy,
) -> DamageResult {
    let mut result = calculate_damage(
        attacker_atk,
        attacker_hit,
        attacker_crit_rate,
        defender.def,
        defender.flee,
    );

    // Apply element advantage
    let element_multiplier = get_element_advantage(attacker_element, defender.element);
    if element_multiplier != 1.0 && !result.is_miss {
        result.damage = (result.damage as f32 * element_multiplier) as u32;
        result.damage = result.damage.max(1); // Ensure at least 1 damage
    }

    if !result.is_miss {
        defender.take_damage(result.damage);
    }

    result
}

/// Enemy attacks (simplified - returns damage to be applied to player)
pub fn enemy_attack(enemy: &Enemy, defender_def: u32, defender_flee: u32) -> DamageResult {
    calculate_damage(
        enemy.atk,
        enemy.hit,
        5.0, // Enemies have 5% base crit rate
        defender_def,
        defender_flee,
    )
}
