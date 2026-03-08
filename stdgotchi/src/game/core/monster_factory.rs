//! Monster Factory
//!
//! Creates Monster instances from Species data.
//! Handles initial stat calculation and skill assignment.

use uuid::Uuid;
use crate::game::core::{Monster, MonsterStatus, Species, Skill, MAX_EQUIPPED_SKILLS};
use crate::game::calculations::{stats, xp};

/// Create a new Monster instance from a Species at its base level
/// `initial_skills` should contain skills the monster has learned at its starting level
pub fn create_monster(species: &Species, initial_skills: Vec<Skill>) -> Monster {
    let id = Uuid::new_v4().to_string();
    let level = species.base_level; // Use species base level from RO database
    let fusion_count = 0;

    // Calculate initial stats at species base level
    let hp_max = stats::calculate_final_hp(species.base_hp, level, fusion_count);
    let atk = stats::calculate_final_stat(species.base_atk, level, fusion_count);
    let def = stats::calculate_final_stat(species.base_def, level, fusion_count);
    let spd = stats::calculate_final_stat(species.base_spd, level, fusion_count);

    // Collect learned skill IDs and equip up to MAX_EQUIPPED_SKILLS
    let learned_skill_ids: Vec<String> = initial_skills.iter().map(|s| s.id.clone()).collect();
    let equipped_skills: Vec<Skill> = initial_skills.into_iter().take(MAX_EQUIPPED_SKILLS).collect();

    Monster {
        id,
        species_id: species.id.clone(),
        name: species.name.clone(),
        level,
        xp: 0,
        xp_to_next: xp::xp_for_next_level(level),
        element: species.element,
        fusion_count,
        hp_current: hp_max,
        hp_max,
        atk,
        def,
        spd,
        // EV-like stat bonuses start at 0
        hp_bonus: 0,
        atk_bonus: 0,
        def_bonus: 0,
        spd_bonus: 0,
        equipped_skills,
        learned_skill_ids,
        skill_cooldowns: [0; MAX_EQUIPPED_SKILLS],
        status: MonsterStatus::Available,
    }
}

/// Create a Monster at a specific level (for testing/debugging)
pub fn create_monster_at_level(species: &Species, initial_skills: Vec<Skill>, level: u8) -> Monster {
    let id = Uuid::new_v4().to_string();
    let fusion_count = 0;

    // Calculate stats at the specified level
    let hp_max = stats::calculate_final_hp(species.base_hp, level, fusion_count);
    let atk = stats::calculate_final_stat(species.base_atk, level, fusion_count);
    let def = stats::calculate_final_stat(species.base_def, level, fusion_count);
    let spd = stats::calculate_final_stat(species.base_spd, level, fusion_count);

    // Collect learned skill IDs and equip up to MAX_EQUIPPED_SKILLS
    let learned_skill_ids: Vec<String> = initial_skills.iter().map(|s| s.id.clone()).collect();
    let equipped_skills: Vec<Skill> = initial_skills.into_iter().take(MAX_EQUIPPED_SKILLS).collect();

    Monster {
        id,
        species_id: species.id.clone(),
        name: species.name.clone(),
        level,
        xp: 0,
        xp_to_next: xp::xp_for_next_level(level),
        element: species.element,
        fusion_count,
        hp_current: hp_max,
        hp_max,
        atk,
        def,
        spd,
        // EV-like stat bonuses start at 0
        hp_bonus: 0,
        atk_bonus: 0,
        def_bonus: 0,
        spd_bonus: 0,
        equipped_skills,
        learned_skill_ids,
        skill_cooldowns: [0; MAX_EQUIPPED_SKILLS],
        status: MonsterStatus::Available,
    }
}

/// Create a starter monster for new players
#[allow(dead_code)]
pub fn create_starter_monster(_species_id: &str, species: &Species, initial_skills: Vec<Skill>) -> Monster {
    // Starters start at level 5 with some bonus stats
    let mut monster = create_monster_at_level(species, initial_skills, 5);
    monster.name = species.name.clone();
    monster
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::core::{SkillEffectType, Element, LearnableSkill};

    fn test_species() -> Species {
        Species {
            id: "poring".to_string(),
            name: "Poring".to_string(),
            element: Element::Water,
            base_level: 1,
            base_hp: 80,
            base_atk: 15,
            base_def: 10,
            base_spd: 20,
            base_exp: 2,
            learnable_skills: vec![
                LearnableSkill { skill_id: "tackle".to_string(), level_required: 1 },
                LearnableSkill { skill_id: "heal".to_string(), level_required: 5 },
            ],
            zones: vec!["prontera".to_string()],
        }
    }

    fn test_skills() -> Vec<Skill> {
        vec![
            Skill {
                id: "tackle".to_string(),
                name: "Tackle".to_string(),
                element: Element::Neutral,
                description: "A basic tackle attack".to_string(),
                effect_type: SkillEffectType::Damage,
                power: 40,
                accuracy: 100,
                crit_chance: 5,
                cooldown: 0,
                effect_value: 1.0,
                buff_stat: None,
                buff_duration: 0,
                dot_damage: 0,
                dot_duration: 0,
            },
            Skill {
                id: "heal".to_string(),
                name: "Heal".to_string(),
                element: Element::Holy,
                description: "Heals the active monster".to_string(),
                effect_type: SkillEffectType::Heal,
                power: 0,
                accuracy: 100,
                crit_chance: 0,
                cooldown: 2,
                effect_value: 0.3,
                buff_stat: None,
                buff_duration: 0,
                dot_damage: 0,
                dot_duration: 0,
            },
        ]
    }

    #[test]
    fn test_create_monster() {
        let species = test_species();
        let skills = test_skills();
        let monster = create_monster(&species, skills);

        assert_eq!(monster.species_id, "poring");
        assert_eq!(monster.level, 1);
        assert_eq!(monster.fusion_count, 0);
        assert_eq!(monster.element, Element::Water);
        assert!(monster.hp_current > 0);
        assert_eq!(monster.equipped_skills.len(), 2);
        assert_eq!(monster.equipped_skills[0].id, "tackle");
        assert_eq!(monster.learned_skill_ids.len(), 2);
    }

    #[test]
    fn test_create_monster_at_level() {
        let species = test_species();
        let skills = test_skills();
        let monster = create_monster_at_level(&species, skills, 10);

        assert_eq!(monster.level, 10);
        // Stats should be higher at level 10
        assert!(monster.hp_max > species.base_hp);
        assert!(monster.atk > species.base_atk);
    }
}
