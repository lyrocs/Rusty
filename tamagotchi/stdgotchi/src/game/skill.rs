//! Skill System
//!
//! Defines skills that can be learned from cards and used in semi-active battles.

use serde::{Deserialize, Serialize};

/// Type of skill effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    /// Deals damage to enemy
    Attack,
    /// Restores HP to self
    Heal,
    /// Temporary stat boost to self
    Buff,
    /// Temporary stat reduction on enemy
    Debuff,
}

/// Who the skill targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillTarget {
    /// Targets the caster (hero)
    #[serde(rename = "self")]
    Self_,
    /// Targets the enemy
    Enemy,
}

/// Skill definition loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillData {
    /// Unique skill ID
    pub id: u32,
    /// Display name
    pub name: String,
    /// Type of skill effect
    pub skill_type: SkillType,
    /// Who this skill targets
    pub target: SkillTarget,
    /// Cooldown in seconds before skill can be used again
    pub cooldown_seconds: f32,
    /// Power value (damage multiplier % or heal amount)
    pub power: u32,
    /// Description for UI
    pub description: String,
    /// Animation name (for future skill-specific animations)
    #[serde(default)]
    pub animation_name: String,
}

/// Active skill instance during battle (tracks cooldown)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSkill {
    /// The skill ID
    pub skill_id: u32,
    /// Remaining cooldown in seconds (0 = ready to use)
    pub remaining_cooldown: f32,
}

impl ActiveSkill {
    /// Create a new active skill (ready to use immediately)
    pub fn new(skill_id: u32) -> Self {
        Self {
            skill_id,
            remaining_cooldown: 0.0,
        }
    }

    /// Check if skill is ready to use
    pub fn is_ready(&self) -> bool {
        self.remaining_cooldown <= 0.0
    }

    /// Put skill on cooldown after use
    pub fn use_skill(&mut self, cooldown_seconds: f32) {
        self.remaining_cooldown = cooldown_seconds;
    }

    /// Update cooldown (call every frame with delta time)
    pub fn update(&mut self, delta_time: f32) {
        if self.remaining_cooldown > 0.0 {
            self.remaining_cooldown -= delta_time;
            if self.remaining_cooldown < 0.0 {
                self.remaining_cooldown = 0.0;
            }
        }
    }

    /// Get cooldown percentage (for UI display)
    pub fn cooldown_percentage(&self, max_cooldown: f32) -> f32 {
        if max_cooldown <= 0.0 {
            return 0.0;
        }
        (self.remaining_cooldown / max_cooldown).clamp(0.0, 1.0)
    }
}

/// Equipped skill card slot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquippedSkillSlot {
    /// The card's monster ID (None if slot is empty)
    pub card_monster_id: Option<u32>,
    /// The skill ID this card provides
    pub skill_id: Option<u32>,
}

impl EquippedSkillSlot {
    pub fn new() -> Self {
        Self {
            card_monster_id: None,
            skill_id: None,
        }
    }

    pub fn equip(&mut self, card_monster_id: u32, skill_id: u32) {
        self.card_monster_id = Some(card_monster_id);
        self.skill_id = Some(skill_id);
    }

    pub fn unequip(&mut self) {
        self.card_monster_id = None;
        self.skill_id = None;
    }

    pub fn is_empty(&self) -> bool {
        self.skill_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_skill_cooldown() {
        let mut skill = ActiveSkill::new(1);
        assert!(skill.is_ready());

        skill.use_skill(5.0);
        assert!(!skill.is_ready());
        assert_eq!(skill.remaining_cooldown, 5.0);

        skill.update(2.0);
        assert!(!skill.is_ready());
        assert_eq!(skill.remaining_cooldown, 3.0);

        skill.update(4.0);
        assert!(skill.is_ready());
        assert_eq!(skill.remaining_cooldown, 0.0);
    }

    #[test]
    fn test_equipped_skill_slot() {
        let mut slot = EquippedSkillSlot::new();
        assert!(slot.is_empty());

        slot.equip(1002, 3); // Poring card with Heal skill
        assert!(!slot.is_empty());
        assert_eq!(slot.card_monster_id, Some(1002));
        assert_eq!(slot.skill_id, Some(3));

        slot.unequip();
        assert!(slot.is_empty());
    }
}
