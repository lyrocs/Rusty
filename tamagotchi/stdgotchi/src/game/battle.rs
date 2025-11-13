//! Battle system
//!
//! Handles damage calculations, hit/miss, critical hits, and fragment drops

use super::{Enemy, Hero, Rustymon};
use super::fragment_collection::FragmentCollection;
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

/// Result of fragment drop attempt
#[derive(Debug, Clone)]
pub enum FragmentDropResult {
    /// Fragment was dropped (enemy_id, enemy_name)
    Dropped(u32, String),
    /// No fragment dropped
    None,
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

/// Calculate damage from attacker to defender
pub fn calculate_damage(attacker_atk: u32, attacker_hit: u32, attacker_crit_rate: f32, 
                       defender_def: u32, defender_flee: u32) -> DamageResult {
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
        1  // Minimum 1 damage
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
    let base_hit = 80;  // 80% base hit rate
    let hit_bonus = (attacker_hit as i32 - defender_flee as i32) / 2;
    let final_hit = (base_hit + hit_bonus).clamp(20, 95);  // Clamp between 20% and 95%
    final_hit as u32
}

/// Hero attacks enemy
pub fn hero_attack(hero: &Hero, enemy: &mut Enemy) -> DamageResult {
    let result = calculate_damage(
        hero.atk,
        hero.hit,
        hero.crit_rate,
        enemy.def,
        enemy.flee,
    );
    
    if !result.is_miss {
        enemy.take_damage(result.damage);
        log::info!("Hero attacks for {} damage{}", 
                   result.damage, 
                   if result.is_critical { " (CRITICAL!)" } else { "" });
    } else {
        log::info!("Hero's attack missed!");
    }
    
    result
}

/// Enemy attacks hero
pub fn enemy_attack(enemy: &Enemy, hero: &mut Hero) -> DamageResult {
    let result = calculate_damage(
        enemy.atk,
        enemy.hit,
        5.0,  // Enemies have 5% base crit rate
        hero.def,
        hero.flee,
    );

    if !result.is_miss {
        hero.take_damage(result.damage);
        log::info!(
            "{} attacks for {} damage{}",
            enemy.name,
            result.damage,
            if result.is_critical {
                " (CRITICAL!)"
            } else {
                ""
            }
        );
    } else {
        log::info!("{}'s attack missed!", enemy.name);
    }

    result
}

/// Check for fragment drop when enemy is defeated
/// Returns FragmentDropResult indicating if a fragment was dropped
pub fn check_fragment_drop(
    enemy_id: u32,
    enemy_name: &str,
    drop_rate: f32,
    fragment_collection: &mut FragmentCollection,
) -> FragmentDropResult {
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();

    if roll < drop_rate {
        // Fragment dropped!
        fragment_collection.add_fragment(enemy_id, 1);
        log::info!("Fragment dropped from {}!", enemy_name);
        FragmentDropResult::Dropped(enemy_id, enemy_name.to_string())
    } else {
        FragmentDropResult::None
    }
}

/// Rustymon attacks enemy with element advantage
pub fn rustymon_attack_enemy(rustymon: &Rustymon, enemy: &mut Enemy) -> DamageResult {
    let mut result = calculate_damage(
        rustymon.atk,
        rustymon.hit,
        rustymon.crit_rate,
        enemy.def,
        enemy.flee,
    );

    // Apply element advantage
    let element_multiplier = get_element_advantage(rustymon.element, enemy.element);
    if element_multiplier != 1.0 && !result.is_miss {
        result.damage = (result.damage as f32 * element_multiplier) as u32;
        result.damage = result.damage.max(1); // Ensure at least 1 damage
    }

    if !result.is_miss {
        enemy.take_damage(result.damage);
        let advantage_text = if element_multiplier > 1.0 {
            " (Super Effective!)"
        } else if element_multiplier < 1.0 {
            " (Not Very Effective...)"
        } else {
            ""
        };

        log::info!("{} attacks for {} damage{}{}",
                   rustymon.name,
                   result.damage,
                   if result.is_critical { " (CRITICAL!)" } else { "" },
                   advantage_text);
    } else {
        log::info!("{}'s attack missed!", rustymon.name);
    }

    result
}

/// Enemy attacks Rustymon with element advantage
pub fn enemy_attack_rustymon(enemy: &Enemy, rustymon: &mut Rustymon) -> DamageResult {
    let mut result = calculate_damage(
        enemy.atk,
        enemy.hit,
        5.0,  // Enemies have 5% base crit rate
        rustymon.def,
        rustymon.flee,
    );

    // Apply element advantage
    let element_multiplier = get_element_advantage(enemy.element, rustymon.element);
    if element_multiplier != 1.0 && !result.is_miss {
        result.damage = (result.damage as f32 * element_multiplier) as u32;
        result.damage = result.damage.max(1); // Ensure at least 1 damage
    }

    if !result.is_miss {
        rustymon.take_damage(result.damage);
        let advantage_text = if element_multiplier > 1.0 {
            " (Super Effective!)"
        } else if element_multiplier < 1.0 {
            " (Not Very Effective...)"
        } else {
            ""
        };

        log::info!(
            "{} attacks {} for {} damage{}{}",
            enemy.name,
            rustymon.name,
            result.damage,
            if result.is_critical { " (CRITICAL!)" } else { "" },
            advantage_text
        );
    } else {
        log::info!("{}'s attack on {} missed!", enemy.name, rustymon.name);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_drop() {
        let mut collection = FragmentCollection::new();

        // Test with 100% drop rate
        let result = check_fragment_drop(1002, "Poring", 1.0, &mut collection);
        match result {
            FragmentDropResult::Dropped(id, name) => {
                assert_eq!(id, 1002);
                assert_eq!(name, "Poring");
                assert_eq!(collection.get_fragment_count(1002), 1);
            }
            FragmentDropResult::None => panic!("Expected fragment drop with 100% rate"),
        }

        // Test with 0% drop rate
        let result = check_fragment_drop(1007, "Fabre", 0.0, &mut collection);
        match result {
            FragmentDropResult::Dropped(_, _) => panic!("Unexpected fragment drop with 0% rate"),
            FragmentDropResult::None => {
                assert_eq!(collection.get_fragment_count(1007), 0);
            }
        }
    }
}
