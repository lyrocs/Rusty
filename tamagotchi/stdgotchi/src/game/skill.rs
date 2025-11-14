//! Skill System for Rustymon
//!
//! Handles skill definitions, effects, and battle integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::rustymon::Element;

/// Type of skill (active or passive)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillType {
    Active,
    Passive,
}

/// Type of effect a skill has
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    /// Direct damage to enemy
    Damage,
    /// Damage over time
    Dot,
    /// Buff self stats
    BuffSelf,
    /// Debuff enemy stats
    DebuffEnemy,
    /// Passive team-wide buff
    PassiveTeam,
}

/// Target of a skill effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillTarget {
    Enemy,
    #[serde(rename = "self")]
    SelfTarget,
    Team,
}

/// Stat that can be modified by skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillStat {
    AtkPercent,
    DefPercent,
    HitPercent,
    FleePercent,
    CritPercent,
    HpPercent,
    RegenFlat,
    RegenPercent,
    DamageBonus,
}

/// A skill that can be learned and used by Rustymon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub skill_type: SkillType,
    #[serde(default)]
    pub element: Option<String>,
    pub cooldown: u32,
    pub duration: u32,
    pub effect_type: EffectType,
    pub effect_value: f32,
    pub effect_target: SkillTarget,
    #[serde(default)]
    pub stat: Option<SkillStat>,
    pub description: String,
    pub icon: String,
}

impl Skill {
    /// Get the element of this skill as an Element enum
    pub fn get_element(&self) -> Option<Element> {
        self.element.as_ref().and_then(|e| Element::from_str(e))
    }

    /// Check if this is an active skill
    pub fn is_active(&self) -> bool {
        self.skill_type == SkillType::Active
    }

    /// Check if this is a passive skill
    pub fn is_passive(&self) -> bool {
        self.skill_type == SkillType::Passive
    }

    /// Check if this skill deals damage
    pub fn is_damage_skill(&self) -> bool {
        matches!(self.effect_type, EffectType::Damage | EffectType::Dot)
    }
}

/// Skill learning configuration for enemy data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnableSkill {
    pub skill_id: u32,
    pub learn_level: u32,
}

/// Active effect currently applied in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub skill_id: u32,
    pub skill_name: String,
    pub effect_type: EffectType,
    pub stat: Option<SkillStat>,
    pub value: f32,
    pub remaining_turns: u32,
    pub is_buff: bool, // true for buff, false for debuff
}

impl ActiveEffect {
    /// Create a new active effect from a skill
    pub fn from_skill(skill: &Skill, is_buff: bool) -> Self {
        Self {
            skill_id: skill.id,
            skill_name: skill.name.clone(),
            effect_type: skill.effect_type,
            stat: skill.stat,
            value: skill.effect_value,
            remaining_turns: skill.duration,
            is_buff,
        }
    }

    /// Update the effect, decrementing turns. Returns true if still active
    pub fn tick(&mut self) -> bool {
        if self.remaining_turns > 0 {
            self.remaining_turns -= 1;
        }
        self.remaining_turns > 0
    }

    /// Check if this effect is expired
    pub fn is_expired(&self) -> bool {
        self.remaining_turns == 0
    }
}

/// Team passive bonuses collected from all team members
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamPassives {
    pub damage_bonus: f32,      // Percentage bonus to all damage
    pub atk_percent: f32,        // ATK percentage bonus
    pub def_percent: f32,        // DEF percentage bonus
    pub hit_percent: f32,        // HIT percentage bonus
    pub flee_percent: f32,       // FLEE percentage bonus
    pub crit_percent: f32,       // CRIT percentage bonus
    pub hp_percent: f32,         // Max HP percentage bonus
    pub regen_flat: f32,         // Flat HP regen per turn
    pub regen_percent: f32,      // Percentage HP regen per turn
}

impl TeamPassives {
    /// Create new empty team passives
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a passive skill bonus to the team
    pub fn add_passive(&mut self, stat: SkillStat, value: f32) {
        match stat {
            SkillStat::DamageBonus => self.damage_bonus += value,
            SkillStat::AtkPercent => self.atk_percent += value,
            SkillStat::DefPercent => self.def_percent += value,
            SkillStat::HitPercent => self.hit_percent += value,
            SkillStat::FleePercent => self.flee_percent += value,
            SkillStat::CritPercent => self.crit_percent += value,
            SkillStat::HpPercent => self.hp_percent += value,
            SkillStat::RegenFlat => self.regen_flat += value,
            SkillStat::RegenPercent => self.regen_percent += value,
        }
    }

    /// Apply team passive bonuses to a stat value
    pub fn apply_to_atk(&self, base_atk: u32) -> u32 {
        (base_atk as f32 * (1.0 + self.atk_percent / 100.0)) as u32
    }

    pub fn apply_to_def(&self, base_def: u32) -> u32 {
        (base_def as f32 * (1.0 + self.def_percent / 100.0)) as u32
    }

    pub fn apply_to_hit(&self, base_hit: u32) -> u32 {
        (base_hit as f32 * (1.0 + self.hit_percent / 100.0)) as u32
    }

    pub fn apply_to_flee(&self, base_flee: u32) -> u32 {
        (base_flee as f32 * (1.0 + self.flee_percent / 100.0)) as u32
    }

    pub fn apply_to_crit(&self, base_crit: f32) -> f32 {
        base_crit + self.crit_percent
    }

    pub fn apply_to_max_hp(&self, base_hp: u32) -> u32 {
        (base_hp as f32 * (1.0 + self.hp_percent / 100.0)) as u32
    }

    pub fn apply_to_damage(&self, base_damage: u32) -> u32 {
        (base_damage as f32 * (1.0 + self.damage_bonus / 100.0)) as u32
    }

    /// Calculate regen amount for a given max HP
    pub fn calculate_regen(&self, max_hp: u32) -> u32 {
        let flat_regen = self.regen_flat;
        let percent_regen = max_hp as f32 * self.regen_percent / 100.0;
        (flat_regen + percent_regen) as u32
    }
}

/// Manages skills for a Rustymon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustymonSkills {
    /// Skills that have been learned
    pub learned_skills: Vec<u32>,
    /// Up to 3 enabled skill IDs
    pub enabled_skills: [Option<u32>; 3],
    /// Cooldown tracking: skill_id -> turns remaining
    #[serde(default)]
    pub cooldowns: HashMap<u32, u32>,
}

impl Default for RustymonSkills {
    fn default() -> Self {
        Self::new()
    }
}

impl RustymonSkills {
    /// Create new empty skill set
    pub fn new() -> Self {
        Self {
            learned_skills: Vec::new(),
            enabled_skills: [None, None, None],
            cooldowns: HashMap::new(),
        }
    }

    /// Learn a new skill
    pub fn learn_skill(&mut self, skill_id: u32) -> bool {
        if self.learned_skills.contains(&skill_id) {
            return false; // Already learned
        }
        if self.learned_skills.len() >= 6 {
            return false; // Max skills reached
        }
        self.learned_skills.push(skill_id);
        log::info!("Learned skill ID: {}", skill_id);
        true
    }

    /// Enable a skill in a slot (0-2)
    pub fn enable_skill(&mut self, skill_id: u32, slot: usize) -> bool {
        if slot >= 3 {
            return false; // Invalid slot
        }
        if !self.learned_skills.contains(&skill_id) {
            return false; // Skill not learned
        }

        // Check if skill is already enabled in another slot
        for i in 0..3 {
            if i != slot && self.enabled_skills[i] == Some(skill_id) {
                return false; // Already enabled elsewhere
            }
        }

        self.enabled_skills[slot] = Some(skill_id);
        true
    }

    /// Disable a skill slot
    pub fn disable_skill(&mut self, slot: usize) -> bool {
        if slot >= 3 {
            return false;
        }
        self.enabled_skills[slot] = None;
        true
    }

    /// Get enabled active skills (those not on cooldown)
    pub fn get_available_skills(&self) -> Vec<u32> {
        self.enabled_skills
            .iter()
            .filter_map(|&skill_id| {
                skill_id.filter(|&id| {
                    !self.cooldowns.contains_key(&id) || self.cooldowns[&id] == 0
                })
            })
            .collect()
    }

    /// Get all enabled skills regardless of cooldown
    pub fn get_enabled_skills(&self) -> Vec<u32> {
        self.enabled_skills
            .iter()
            .filter_map(|&skill_id| skill_id)
            .collect()
    }

    /// Check if a skill is on cooldown
    pub fn is_on_cooldown(&self, skill_id: u32) -> bool {
        self.cooldowns.get(&skill_id).map_or(false, |&cd| cd > 0)
    }

    /// Get cooldown remaining for a skill
    pub fn get_cooldown(&self, skill_id: u32) -> u32 {
        self.cooldowns.get(&skill_id).copied().unwrap_or(0)
    }

    /// Put a skill on cooldown
    pub fn apply_cooldown(&mut self, skill_id: u32, turns: u32) {
        self.cooldowns.insert(skill_id, turns);
    }

    /// Reduce all cooldowns by 1 turn
    pub fn tick_cooldowns(&mut self) {
        let keys: Vec<u32> = self.cooldowns.keys().copied().collect();
        for skill_id in keys {
            if let Some(cooldown) = self.cooldowns.get_mut(&skill_id) {
                if *cooldown > 0 {
                    *cooldown -= 1;
                }
                if *cooldown == 0 {
                    self.cooldowns.remove(&skill_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_learning() {
        let mut skills = RustymonSkills::new();
        assert!(skills.learn_skill(1));
        assert!(skills.learn_skill(2));
        assert!(!skills.learn_skill(1)); // Can't learn twice
    }

    #[test]
    fn test_skill_enabling() {
        let mut skills = RustymonSkills::new();
        skills.learn_skill(1);
        skills.learn_skill(2);

        assert!(skills.enable_skill(1, 0));
        assert!(skills.enable_skill(2, 1));
        assert!(!skills.enable_skill(1, 1)); // Already enabled
        assert!(!skills.enable_skill(3, 0)); // Not learned
    }

    #[test]
    fn test_cooldowns() {
        let mut skills = RustymonSkills::new();
        skills.apply_cooldown(1, 3);

        assert_eq!(skills.get_cooldown(1), 3);
        assert!(skills.is_on_cooldown(1));

        skills.tick_cooldowns();
        assert_eq!(skills.get_cooldown(1), 2);

        skills.tick_cooldowns();
        skills.tick_cooldowns();
        assert_eq!(skills.get_cooldown(1), 0);
        assert!(!skills.is_on_cooldown(1));
    }

    #[test]
    fn test_team_passives() {
        let mut passives = TeamPassives::new();
        passives.add_passive(SkillStat::AtkPercent, 10.0);
        passives.add_passive(SkillStat::DamageBonus, 5.0);

        assert_eq!(passives.apply_to_atk(100), 110);
        assert_eq!(passives.apply_to_damage(100), 105);
    }
}
