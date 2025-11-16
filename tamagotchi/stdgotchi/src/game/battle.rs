//! Battle system
//!
//! Handles damage calculations, hit/miss, critical hits, and fragment drops

use super::{Enemy, Rustymon};
use super::fragment_collection::FragmentCollection;
use super::element_system::get_element_advantage;
use super::skill::{Skill, ActiveEffect, TeamPassives, EffectType};
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
    /// Active effects on the player's Rustymon (buffs)
    #[serde(default)]
    pub rustymon_effects: Vec<ActiveEffect>,
    /// Active effects on the enemy (debuffs/DOTs)
    #[serde(default)]
    pub enemy_effects: Vec<ActiveEffect>,
    /// Team passive bonuses
    #[serde(default)]
    pub team_passives: TeamPassives,
    /// Current turn number
    #[serde(default)]
    pub turn_number: u32,
}

impl Default for BattleState {
    fn default() -> Self {
        Self {
            hero_last_attack: 0.0,
            enemy_last_attack: 0.0,
            rustymon_effects: Vec::new(),
            enemy_effects: Vec::new(),
            team_passives: TeamPassives::new(),
            turn_number: 0,
        }
    }
}

impl BattleState {
    /// Start a new battle, resetting effects and collecting team passives
    pub fn start_battle(&mut self, team_skills: &[&Skill]) {
        self.rustymon_effects.clear();
        self.enemy_effects.clear();
        self.team_passives = TeamPassives::new();
        self.turn_number = 0;

        // Collect passive skills from the team
        for skill in team_skills {
            if skill.is_passive() {
                if let (Some(stat), effect_value) = (skill.stat, skill.effect_value) {
                    self.team_passives.add_passive(stat, effect_value);
                    log::info!("Team passive active: {} (+{} {})", skill.name, effect_value, stat_name(stat));
                }
            }
        }
    }

    /// Process turn-based effects (DOT, buff/debuff ticks)
    pub fn process_turn_effects(&mut self, rustymon: &mut Rustymon, enemy: &mut Enemy) {
        self.turn_number += 1;

        // Process rustymon effects (buffs mostly, but could have regen)
        self.rustymon_effects.retain_mut(|effect| {
            if effect.effect_type == EffectType::Dot {
                // DOT on rustymon (rare but possible)
                let dot_damage = (rustymon.max_hp as f32 * effect.value / 100.0) as u32;
                rustymon.take_damage(dot_damage.max(1));
                log::info!("{} takes {} DOT damage from {}", rustymon.name, dot_damage, effect.skill_name);
            }
            effect.tick()
        });

        // Process enemy effects (debuffs and DOTs)
        self.enemy_effects.retain_mut(|effect| {
            if effect.effect_type == EffectType::Dot {
                let dot_damage = (enemy.max_hp as f32 * effect.value / 100.0) as u32;
                enemy.take_damage(dot_damage.max(1));
                log::info!("{} takes {} DOT damage from {}", enemy.name, dot_damage, effect.skill_name);
            }
            effect.tick()
        });

        // Tick down cooldowns on rustymon
        rustymon.skills.tick_cooldowns();
    }

    /// Add an effect to rustymon (buff)
    pub fn add_rustymon_effect(&mut self, effect: ActiveEffect) {
        log::info!("Applied {} to rustymon for {} turns", effect.skill_name, effect.remaining_turns);
        self.rustymon_effects.push(effect);
    }

    /// Add an effect to enemy (debuff/DOT)
    pub fn add_enemy_effect(&mut self, effect: ActiveEffect) {
        log::info!("Applied {} to enemy for {} turns", effect.skill_name, effect.remaining_turns);
        self.enemy_effects.push(effect);
    }

    /// Get total stat modifier from active effects
    fn get_stat_modifier(&self, effects: &[ActiveEffect], stat: super::skill::SkillStat) -> f32 {
        effects.iter()
            .filter(|e| e.stat == Some(stat))
            .map(|e| if e.is_buff { e.value } else { -e.value })
            .sum()
    }

    /// Get modified rustymon stats with buffs applied
    pub fn get_modified_rustymon_stats(&self, rustymon: &Rustymon) -> ModifiedStats {
        use super::skill::SkillStat;

        let atk_mod = self.get_stat_modifier(&self.rustymon_effects, SkillStat::AtkPercent);
        let def_mod = self.get_stat_modifier(&self.rustymon_effects, SkillStat::DefPercent);
        let hit_mod = self.get_stat_modifier(&self.rustymon_effects, SkillStat::HitPercent);
        let flee_mod = self.get_stat_modifier(&self.rustymon_effects, SkillStat::FleePercent);
        let crit_mod = self.get_stat_modifier(&self.rustymon_effects, SkillStat::CritPercent);

        // Apply team passives and effect modifiers
        let atk = self.team_passives.apply_to_atk(rustymon.atk);
        let atk = (atk as f32 * (1.0 + atk_mod / 100.0)) as u32;

        let def = self.team_passives.apply_to_def(rustymon.def);
        let def = (def as f32 * (1.0 + def_mod / 100.0)) as u32;

        let hit = self.team_passives.apply_to_hit(rustymon.hit);
        let hit = (hit as f32 * (1.0 + hit_mod / 100.0)) as u32;

        let flee = self.team_passives.apply_to_flee(rustymon.flee);
        let flee = (flee as f32 * (1.0 + flee_mod / 100.0)) as u32;

        let crit_rate = self.team_passives.apply_to_crit(rustymon.crit_rate) + crit_mod;

        ModifiedStats {
            atk,
            def,
            hit,
            flee,
            crit_rate,
        }
    }

    /// Get modified enemy stats with debuffs applied
    pub fn get_modified_enemy_stats(&self, enemy: &Enemy) -> ModifiedStats {
        use super::skill::SkillStat;

        let atk_mod = self.get_stat_modifier(&self.enemy_effects, SkillStat::AtkPercent);
        let def_mod = self.get_stat_modifier(&self.enemy_effects, SkillStat::DefPercent);
        let hit_mod = self.get_stat_modifier(&self.enemy_effects, SkillStat::HitPercent);
        let flee_mod = self.get_stat_modifier(&self.enemy_effects, SkillStat::FleePercent);

        let atk = (enemy.atk as f32 * (1.0 - atk_mod / 100.0).max(0.1)) as u32;
        let def = (enemy.def as f32 * (1.0 - def_mod / 100.0).max(0.0)) as u32;
        let hit = (enemy.hit as f32 * (1.0 - hit_mod / 100.0).max(0.1)) as u32;
        let flee = (enemy.flee as f32 * (1.0 - flee_mod / 100.0).max(0.0)) as u32;

        ModifiedStats {
            atk,
            def,
            hit,
            flee,
            crit_rate: 5.0, // Enemies have base 5% crit
        }
    }
}

/// Modified stats after applying buffs/debuffs
#[derive(Debug, Clone, Copy)]
pub struct ModifiedStats {
    pub atk: u32,
    pub def: u32,
    pub hit: u32,
    pub flee: u32,
    pub crit_rate: f32,
}

/// Helper to get stat name for logging
fn stat_name(stat: super::skill::SkillStat) -> &'static str {
    use super::skill::SkillStat;
    match stat {
        SkillStat::AtkPercent => "ATK%",
        SkillStat::DefPercent => "DEF%",
        SkillStat::HitPercent => "HIT%",
        SkillStat::FleePercent => "FLEE%",
        SkillStat::CritPercent => "CRIT%",
        SkillStat::HpPercent => "HP%",
        SkillStat::RegenFlat => "HP Regen",
        SkillStat::RegenPercent => "HP Regen%",
        SkillStat::DamageBonus => "Damage",
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

/// Rustymon uses a skill on an enemy
pub fn rustymon_use_skill(
    rustymon: &mut Rustymon,
    enemy: &mut Enemy,
    skill: &Skill,
    battle_state: &mut BattleState,
) -> DamageResult {
    // Put skill on cooldown
    rustymon.skills.apply_cooldown(skill.id, skill.cooldown);

    match skill.effect_type {
        EffectType::Damage => {
            // Direct damage skill
            let skill_element = skill.get_element().unwrap_or(rustymon.element);
            let mut result = calculate_damage(
                rustymon.atk,
                rustymon.hit,
                rustymon.crit_rate,
                enemy.def,
                enemy.flee,
            );

            if !result.is_miss {
                // Apply skill damage multiplier
                result.damage = (result.damage as f32 * (skill.effect_value / 100.0)) as u32;

                // Apply element advantage
                let element_multiplier = get_element_advantage(skill_element, enemy.element);
                result.damage = (result.damage as f32 * element_multiplier) as u32;

                // Apply team passive damage bonus
                result.damage = battle_state.team_passives.apply_to_damage(result.damage);

                result.damage = result.damage.max(1);
                enemy.take_damage(result.damage);

                let advantage_text = if element_multiplier > 1.0 {
                    " (Super Effective!)"
                } else if element_multiplier < 1.0 {
                    " (Not Very Effective...)"
                } else {
                    ""
                };

                log::info!(
                    "{} uses {} for {} damage{}{}",
                    rustymon.name,
                    skill.name,
                    result.damage,
                    if result.is_critical { " (CRITICAL!)" } else { "" },
                    advantage_text
                );
            } else {
                log::info!("{}'s {} missed!", rustymon.name, skill.name);
            }

            result
        }
        EffectType::Dot => {
            // Damage over time skill
            let effect = ActiveEffect::from_skill(skill, false);
            battle_state.add_enemy_effect(effect);
            log::info!("{} uses {}!", rustymon.name, skill.name);

            DamageResult {
                damage: 0,
                is_critical: false,
                is_miss: false,
            }
        }
        EffectType::BuffSelf => {
            // Self buff
            let effect = ActiveEffect::from_skill(skill, true);
            battle_state.add_rustymon_effect(effect);
            log::info!("{} uses {}!", rustymon.name, skill.name);

            DamageResult {
                damage: 0,
                is_critical: false,
                is_miss: false,
            }
        }
        EffectType::DebuffEnemy => {
            // Enemy debuff
            let effect = ActiveEffect::from_skill(skill, false);
            battle_state.add_enemy_effect(effect);
            log::info!("{} uses {}!", rustymon.name, skill.name);

            DamageResult {
                damage: 0,
                is_critical: false,
                is_miss: false,
            }
        }
        EffectType::PassiveTeam => {
            // Passive skills shouldn't be "used" in battle
            log::warn!("Attempted to use passive skill {} in battle", skill.name);
            DamageResult {
                damage: 0,
                is_critical: false,
                is_miss: false,
            }
        }
    }
}

/// Rustymon attacks enemy with modified stats from battle state
pub fn rustymon_attack_with_battle_state(
    rustymon: &Rustymon,
    enemy: &mut Enemy,
    battle_state: &BattleState,
) -> DamageResult {
    let rustymon_stats = battle_state.get_modified_rustymon_stats(rustymon);
    let enemy_stats = battle_state.get_modified_enemy_stats(enemy);

    let mut result = calculate_damage(
        rustymon_stats.atk,
        rustymon_stats.hit,
        rustymon_stats.crit_rate,
        enemy_stats.def,
        enemy_stats.flee,
    );

    // Apply element advantage
    let element_multiplier = get_element_advantage(rustymon.element, enemy.element);
    if element_multiplier != 1.0 && !result.is_miss {
        result.damage = (result.damage as f32 * element_multiplier) as u32;
    }

    // Apply team passive damage bonus
    if !result.is_miss {
        result.damage = battle_state.team_passives.apply_to_damage(result.damage);
        result.damage = result.damage.max(1);
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

        log::info!(
            "{} attacks for {} damage{}{}",
            rustymon.name,
            result.damage,
            if result.is_critical { " (CRITICAL!)" } else { "" },
            advantage_text
        );
    } else {
        log::info!("{}'s attack missed!", rustymon.name);
    }

    result
}

/// Enemy attacks Rustymon with modified stats from battle state
pub fn enemy_attack_with_battle_state(
    enemy: &Enemy,
    rustymon: &mut Rustymon,
    battle_state: &BattleState,
) -> DamageResult {
    let rustymon_stats = battle_state.get_modified_rustymon_stats(rustymon);
    let enemy_stats = battle_state.get_modified_enemy_stats(enemy);

    let mut result = calculate_damage(
        enemy_stats.atk,
        enemy_stats.hit,
        enemy_stats.crit_rate,
        rustymon_stats.def,
        rustymon_stats.flee,
    );

    // Apply element advantage
    let element_multiplier = get_element_advantage(enemy.element, rustymon.element);
    if element_multiplier != 1.0 && !result.is_miss {
        result.damage = (result.damage as f32 * element_multiplier) as u32;
        result.damage = result.damage.max(1);
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
