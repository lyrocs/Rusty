//! Skill System
//!
//! Monsters learn skills by leveling up (Pokemon-style).
//! Each monster can equip up to 3 skills for battle.
//! Skills have power, accuracy, and cooldown.

use serde::{Deserialize, Serialize};
use super::Element;

/// Skill effect type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillEffectType {
    /// Deal damage based on power and ATK
    Damage,
    /// Deal damage and apply DoT
    DamageDot,
    /// Heal active monster (percentage of max HP)
    Heal,
    /// Deal damage ignoring some DEF
    DamageIgnoreDef,
    /// Apply a buff (stat increase)
    Buff,
    /// Apply a debuff (stat decrease to enemy)
    Debuff,
}

/// Stat type for buff/debuff effects
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatType {
    Atk,
    Def,
    Spd,
}

/// A monster skill with Pokemon-style mechanics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub element: Element,
    pub description: String,
    pub effect_type: SkillEffectType,

    /// Base power for damage calculation (0 for non-damage skills)
    #[serde(default)]
    pub power: u16,

    /// Accuracy percentage (0-100, 100 = always hits)
    #[serde(default = "default_accuracy")]
    pub accuracy: u8,

    /// Cooldown in turns after use (0 = no cooldown)
    #[serde(default)]
    pub cooldown: u8,

    /// Effect value (heal percentage, buff/debuff multiplier, etc.)
    #[serde(default = "default_effect_value")]
    pub effect_value: f32,

    /// Stat affected by buff/debuff (if applicable)
    #[serde(default)]
    pub buff_stat: Option<StatType>,

    /// Buff/debuff duration in turns (if applicable)
    #[serde(default)]
    pub buff_duration: u8,

    /// DoT damage per turn (if applicable)
    #[serde(default)]
    pub dot_damage: u16,

    /// DoT duration in turns (if applicable)
    #[serde(default)]
    pub dot_duration: u8,
}

fn default_accuracy() -> u8 { 100 }
fn default_effect_value() -> f32 { 1.0 }

impl Default for Skill {
    fn default() -> Self {
        Self {
            id: "unknown".to_string(),
            name: "Unknown".to_string(),
            element: Element::Water,
            description: "Unknown skill".to_string(),
            effect_type: SkillEffectType::Damage,
            power: 40,
            accuracy: 100,
            cooldown: 0,
            effect_value: 1.0,
            buff_stat: None,
            buff_duration: 0,
            dot_damage: 0,
            dot_duration: 0,
        }
    }
}

impl Skill {
    /// Check if this skill deals damage
    pub fn is_damage_skill(&self) -> bool {
        matches!(
            self.effect_type,
            SkillEffectType::Damage | SkillEffectType::DamageDot | SkillEffectType::DamageIgnoreDef
        )
    }

    /// Check if this skill heals
    pub fn is_heal_skill(&self) -> bool {
        matches!(self.effect_type, SkillEffectType::Heal)
    }

    /// Get short name for button display (max 8 chars)
    pub fn short_name(&self) -> &str {
        if self.name.len() > 8 {
            &self.name[..8]
        } else {
            &self.name
        }
    }
}
