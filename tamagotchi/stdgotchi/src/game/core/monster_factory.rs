//! Monster Factory
//!
//! Creates Monster instances from Species data.
//! Handles initial stat calculation and skill assignment.

use uuid::Uuid;
use crate::game::core::{Monster, MonsterStatus, Species, Skill, Element};
use crate::game::calculations::{stats, xp};

/// Create a new Monster instance from a Species
pub fn create_monster(species: &Species, skill: &Skill) -> Monster {
    let id = Uuid::new_v4().to_string();
    let level = 1;
    let fusion_count = 0;

    // Calculate initial stats at level 1
    let hp_max = stats::calculate_final_hp(species.base_hp, level, fusion_count);
    let atk = stats::calculate_final_stat(species.base_atk, level, fusion_count);
    let def = stats::calculate_final_stat(species.base_def, level, fusion_count);
    let spd = stats::calculate_final_stat(species.base_spd, level, fusion_count);

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
        skill: skill.clone(),
        status: MonsterStatus::Available,
    }
}

/// Create a Monster at a specific level (for testing/debugging)
pub fn create_monster_at_level(species: &Species, skill: &Skill, level: u8) -> Monster {
    let id = Uuid::new_v4().to_string();
    let fusion_count = 0;

    // Calculate stats at the specified level
    let hp_max = stats::calculate_final_hp(species.base_hp, level, fusion_count);
    let atk = stats::calculate_final_stat(species.base_atk, level, fusion_count);
    let def = stats::calculate_final_stat(species.base_def, level, fusion_count);
    let spd = stats::calculate_final_stat(species.base_spd, level, fusion_count);

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
        skill: skill.clone(),
        status: MonsterStatus::Available,
    }
}

/// Create a starter monster for new players
pub fn create_starter_monster(species_id: &str, species: &Species, skill: &Skill) -> Monster {
    // Starters start at level 5 with some bonus stats
    let mut monster = create_monster_at_level(species, skill, 5);
    monster.name = format!("{}", species.name); // Could add "Your " prefix
    monster
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::core::{SkillEffectType};

    fn test_species() -> Species {
        Species {
            id: "poring".to_string(),
            name: "Poring".to_string(),
            element: Element::Water,
            base_hp: 80,
            base_atk: 15,
            base_def: 10,
            base_spd: 20,
            skill_id: "heal".to_string(),
            zones: vec!["prontera".to_string()],
        }
    }

    fn test_skill() -> Skill {
        Skill {
            id: "heal".to_string(),
            name: "Heal".to_string(),
            element: Element::Holy,
            description: "Heals the active monster".to_string(),
            effect_type: SkillEffectType::Heal,
            effect_value: 0.3,
            dot_duration: 0.0,
            buff_duration: 0.0,
        }
    }

    #[test]
    fn test_create_monster() {
        let species = test_species();
        let skill = test_skill();
        let monster = create_monster(&species, &skill);

        assert_eq!(monster.species_id, "poring");
        assert_eq!(monster.level, 1);
        assert_eq!(monster.fusion_count, 0);
        assert_eq!(monster.element, Element::Water);
        assert!(monster.hp_current > 0);
        assert_eq!(monster.skill.id, "heal");
    }

    #[test]
    fn test_create_monster_at_level() {
        let species = test_species();
        let skill = test_skill();
        let monster = create_monster_at_level(&species, &skill, 10);

        assert_eq!(monster.level, 10);
        // Stats should be higher at level 10
        assert!(monster.hp_max > species.base_hp);
        assert!(monster.atk > species.base_atk);
    }
}
