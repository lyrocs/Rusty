//! Skill System
//!
//! Each monster species has ONE unique skill.
//! Skills are loaded from skills.json.

use serde::{Deserialize, Serialize};
use super::Element;

/// Skill effect type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillEffectType {
    /// Deal damage (multiplier of ATK)
    Damage,
    /// Deal damage and apply DoT
    DamageDot,
    /// Heal active monster (percentage of max HP)
    Heal,
    /// Deal damage ignoring some DEF
    DamageIgnoreDef,
    /// Apply a buff/debuff
    Buff,
}

/// A monster skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub element: Element,
    pub description: String,
    pub effect_type: SkillEffectType,
    /// Effect value (damage multiplier, heal percentage, etc.)
    pub effect_value: f32,
    /// DoT duration in seconds (if applicable)
    #[serde(default)]
    pub dot_duration: f32,
    /// Buff/debuff duration in seconds (if applicable)
    #[serde(default)]
    pub buff_duration: f32,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            id: "unknown".to_string(),
            name: "Unknown".to_string(),
            element: Element::Water,
            description: "Unknown skill".to_string(),
            effect_type: SkillEffectType::Damage,
            effect_value: 1.0,
            dot_duration: 0.0,
            buff_duration: 0.0,
        }
    }
}
