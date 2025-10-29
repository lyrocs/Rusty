/// Skills data management
///
/// Loads and provides access to skill data from JSON.
use heapless::Vec as HeaplessVec;
use serde::Deserialize;

use super::common::LazyData;
use crate::combat::skills::{JrpgSkill, SkillEffect, SkillType};

// Embed JSON file at compile time
const SKILLS_JSON: &str = include_str!("../../assets/data/skills.json");

/// Skill data structure (matches skills.json)
#[derive(Debug, Deserialize, Clone)]
pub struct SkillData {
    pub id: u16,
    pub name: &'static str,
    pub sp_cost: u16,
    pub skill_type: &'static str,
    pub power: u16,
    pub job_req: &'static str,
    #[serde(default)]
    pub description: &'static str,
}

// Static storage for parsed skill data
static SKILLS: LazyData<HeaplessVec<SkillData, 32>> = LazyData::new();

/// Parse skills from JSON (done once, cached)
fn parse_skills() -> HeaplessVec<SkillData, 32> {
    esp_println::println!("[GAME_DATA] Parsing skills.json...");

    match serde_json_core::from_str::<HeaplessVec<SkillData, 32>>(SKILLS_JSON) {
        Ok((skills, _)) => {
            esp_println::println!("[GAME_DATA] Successfully parsed {} skills", skills.len());
            for skill in &skills {
                esp_println::println!(
                    "  - {} (ID: {}, Type: {}, Job: {})",
                    skill.name,
                    skill.id,
                    skill.skill_type,
                    skill.job_req
                );
            }
            skills
        }
        Err(e) => {
            esp_println::println!("[ERROR] Failed to parse skills.json: {:?}", e);
            HeaplessVec::new()
        }
    }
}

/// Parse skill type string
fn parse_skill_type(type_str: &str) -> SkillType {
    match type_str {
        "Physical" => SkillType::Physical,
        "Magic" => SkillType::Magic,
        "Buff" => SkillType::Buff,
        "Debuff" => SkillType::Debuff,
        "Healing" => SkillType::Healing,
        "Utility" => SkillType::Utility,
        _ => SkillType::Physical, // Default fallback
    }
}

/// Get skill effect based on ID (hardcoded effects for now)
fn get_skill_effect(id: u16) -> (Option<SkillEffect>, u8) {
    match id {
        1 => (Some(SkillEffect::Stun(1)), 1),                    // Bash
        2 => (Some(SkillEffect::DebuffDef(30, 3)), 3),           // Provoke
        3 => (None, 0),                                          // Magnum Break
        10 => (None, 0),                                         // Fire Bolt
        11 => (Some(SkillEffect::BuffAgi(50, 2)), 2),            // Cold Bolt (slow)
        12 => (Some(SkillEffect::Stun(1)), 1),                   // Lightning Bolt
        20 => (Some(SkillEffect::MultiHit(2)), 0),               // Double Strafe
        21 => (None, 0),                                         // Arrow Shower
        22 => (Some(SkillEffect::BuffAgi(30, 3)), 3),            // Concentration
        30 => (Some(SkillEffect::Steal(10, 50)), 0),             // Steal
        31 => (Some(SkillEffect::DodgeNext), 1),                 // Hiding
        32 => (Some(SkillEffect::Poison(5, 3)), 3),              // Envenom
        40 => (Some(SkillEffect::Heal(0)), 0),                   // Heal
        41 => (Some(SkillEffect::BuffAtk(20, 4)), 4),            // Blessing
        42 => (Some(SkillEffect::BuffDef(40, 2)), 2),            // Divine Protect
        50 => (None, 0),                                         // Mammonite
        51 => (Some(SkillEffect::Steal(20, 100)), 0),            // Discount
        52 => (Some(SkillEffect::BuffAtk(25, 3)), 3),            // Loud Exclaim
        _ => (None, 0),
    }
}

/// Get skill by ID
pub fn get_skill_by_id(id: u16) -> Option<JrpgSkill> {
    let skills = SKILLS.get_or_init(parse_skills);

    skills
        .iter()
        .find(|s| s.id == id)
        .map(|s| {
            let (effect, duration) = get_skill_effect(s.id);
            JrpgSkill {
                id: s.id,
                name: s.name,
                sp_cost: s.sp_cost,
                skill_type: parse_skill_type(s.skill_type),
                power: s.power,
                effect,
                duration,
            }
        })
}

/// Get skills for a specific job
pub fn get_skills_for_job(job: &str) -> HeaplessVec<JrpgSkill, 3> {
    let skills = SKILLS.get_or_init(parse_skills);
    let mut result = HeaplessVec::new();

    for skill_data in skills.iter() {
        if skill_data.job_req.eq_ignore_ascii_case(job) {
            if let Some(skill) = get_skill_by_id(skill_data.id) {
                result.push(skill).ok();
                if result.is_full() {
                    break;
                }
            }
        }
    }

    result
}

/// Get all skills
pub fn get_all_skills() -> HeaplessVec<JrpgSkill, 32> {
    let skills = SKILLS.get_or_init(parse_skills);
    let mut result = HeaplessVec::new();

    for skill_data in skills.iter() {
        if let Some(skill) = get_skill_by_id(skill_data.id) {
            result.push(skill).ok();
            if result.is_full() {
                break;
            }
        }
    }

    result
}
