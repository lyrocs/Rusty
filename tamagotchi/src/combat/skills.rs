/// Skill system for JRPG battles
///
/// Contains skill definitions, effects, and status effect management.

/// Skill type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillType {
    Physical,
    Magic,
    Buff,
    Debuff,
    Healing,
    Utility,
}

/// Skill effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillEffect {
    Damage(u16),           // Base damage
    Heal(u16),             // Heal amount
    Stun(u8),              // Stun for X turns
    Poison(u16, u8),       // Damage per turn, duration
    BuffAtk(u16, u8),      // Increase ATK by X%, duration
    BuffDef(u16, u8),      // Increase DEF by X%, duration
    BuffAgi(u16, u8),      // Increase AGI by X%, duration
    DebuffAtk(u16, u8),    // Decrease ATK by X%, duration
    DebuffDef(u16, u8),    // Decrease DEF by X%, duration
    Steal(u16, u16),       // Min/max zeny to steal
    MultiHit(u8),          // Number of hits
    DodgeNext,             // Dodge next attack
}

/// JRPG Skill definition
#[derive(Debug, Clone, Copy)]
pub struct JrpgSkill {
    pub id: u16,
    pub name: &'static str,
    pub sp_cost: u16,
    pub skill_type: SkillType,
    pub power: u16,              // Damage multiplier (150 = 150%)
    pub effect: Option<SkillEffect>,
    pub duration: u8,            // For buffs/debuffs
}

/// Status effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectType {
    Poison,
    Stun,
    Slow,
    Burn,
    Freeze,
    Blind,
    AtkBuff,
    DefBuff,
    AgiBuff,
    AtkDebuff,
    DefDebuff,
    AgiDebuff,
    Blessing,
    DodgeNext,
}

/// Active status effect on a combatant
#[derive(Debug, Clone, Copy)]
pub struct ActiveStatusEffect {
    pub effect_type: StatusEffectType,
    pub duration: u8,     // Turns remaining
    pub power: u16,       // Effect strength (%)
}

/// Skill database - skills available per job
impl JrpgSkill {
    /// Get skills for Swordsman job
    pub const fn get_swordsman_skills() -> [JrpgSkill; 3] {
        [
            // Bash - High damage single target
            JrpgSkill {
                id: 1,
                name: "Bash",
                sp_cost: 8,
                skill_type: SkillType::Physical,
                power: 150, // 150% ATK damage
                effect: Some(SkillEffect::Stun(1)), // 10% stun chance handled in code
                duration: 1,
            },
            // Provoke - Debuff enemy DEF, buff own ATK
            JrpgSkill {
                id: 2,
                name: "Provoke",
                sp_cost: 5,
                skill_type: SkillType::Debuff,
                power: 0, // No damage
                effect: Some(SkillEffect::DebuffDef(30, 3)), // -30% DEF for 3 turns
                duration: 3,
            },
            // Magnum Break - Medium damage
            JrpgSkill {
                id: 3,
                name: "Magnum Break",
                sp_cost: 15,
                skill_type: SkillType::Physical,
                power: 120, // 120% ATK damage
                effect: None, // Just damage
                duration: 0,
            },
        ]
    }

    /// Get skills for Mage job
    pub const fn get_mage_skills() -> [JrpgSkill; 3] {
        [
            // Fire Bolt - High INT-based magic damage
            JrpgSkill {
                id: 10,
                name: "Fire Bolt",
                sp_cost: 12,
                skill_type: SkillType::Magic,
                power: 200, // INT × 2
                effect: None,
                duration: 0,
            },
            // Cold Bolt - INT-based magic damage with slow
            JrpgSkill {
                id: 11,
                name: "Cold Bolt",
                sp_cost: 12,
                skill_type: SkillType::Magic,
                power: 180, // INT × 1.8
                effect: Some(SkillEffect::BuffAgi(50, 2)), // Implemented as slow (reduce AGI)
                duration: 2,
            },
            // Lightning Bolt - Highest INT-based magic damage with stun
            JrpgSkill {
                id: 12,
                name: "Lightning Bolt",
                sp_cost: 12,
                skill_type: SkillType::Magic,
                power: 220, // INT × 2.2
                effect: Some(SkillEffect::Stun(1)), // 10% stun chance
                duration: 1,
            },
        ]
    }

    /// Get skills for Archer job
    pub const fn get_archer_skills() -> [JrpgSkill; 3] {
        [
            // Double Strafe - Attack twice
            JrpgSkill {
                id: 20,
                name: "Double Strafe",
                sp_cost: 10,
                skill_type: SkillType::Physical,
                power: 100, // 100% ATK × 2 hits
                effect: Some(SkillEffect::MultiHit(2)),
                duration: 0,
            },
            // Arrow Shower - Area damage
            JrpgSkill {
                id: 21,
                name: "Arrow Shower",
                sp_cost: 15,
                skill_type: SkillType::Physical,
                power: 80, // 80% ATK
                effect: None,
                duration: 0,
            },
            // Improve Concentration - Buff AGI and DEX
            JrpgSkill {
                id: 22,
                name: "Concentration",
                sp_cost: 8,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffAgi(30, 3)), // +30% AGI for 3 turns
                duration: 3,
            },
        ]
    }

    /// Get skills for Thief job
    pub const fn get_thief_skills() -> [JrpgSkill; 3] {
        [
            // Steal - Steal Zeny from enemy
            JrpgSkill {
                id: 30,
                name: "Steal",
                sp_cost: 10,
                skill_type: SkillType::Utility,
                power: 0,
                effect: Some(SkillEffect::Steal(10, 50)), // 10-50z
                duration: 0,
            },
            // Hiding - Dodge next attack and counter
            JrpgSkill {
                id: 31,
                name: "Hiding",
                sp_cost: 12,
                skill_type: SkillType::Utility,
                power: 80, // Counter for 80% ATK
                effect: Some(SkillEffect::DodgeNext),
                duration: 1,
            },
            // Envenom - Poison damage over time
            JrpgSkill {
                id: 32,
                name: "Envenom",
                sp_cost: 15,
                skill_type: SkillType::Physical,
                power: 120, // 120% ATK
                effect: Some(SkillEffect::Poison(5, 3)), // 5 dmg/turn for 3 turns
                duration: 3,
            },
        ]
    }

    /// Get skills for Acolyte job
    pub const fn get_acolyte_skills() -> [JrpgSkill; 3] {
        [
            // Heal - Restore HP
            JrpgSkill {
                id: 40,
                name: "Heal",
                sp_cost: 13,
                skill_type: SkillType::Healing,
                power: 300, // INT × 3
                effect: Some(SkillEffect::Heal(0)), // Amount calculated in code
                duration: 0,
            },
            // Blessing - Buff all stats
            JrpgSkill {
                id: 41,
                name: "Blessing",
                sp_cost: 10,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffAtk(20, 4)), // +20% ATK/DEF for 4 turns
                duration: 4,
            },
            // Divine Protection - Reduce damage taken
            JrpgSkill {
                id: 42,
                name: "Divine Protect",
                sp_cost: 12,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffDef(40, 2)), // +40% DEF for 2 turns
                duration: 2,
            },
        ]
    }

    /// Get skills for Merchant job
    pub const fn get_merchant_skills() -> [JrpgSkill; 3] {
        [
            // Mammonite - Spend Zeny for high damage
            JrpgSkill {
                id: 50,
                name: "Mammonite",
                sp_cost: 8,
                skill_type: SkillType::Physical,
                power: 180, // 180% ATK (costs 50z)
                effect: None,
                duration: 0,
            },
            // Discount - Steal item (implemented as Zeny)
            JrpgSkill {
                id: 51,
                name: "Discount",
                sp_cost: 5,
                skill_type: SkillType::Utility,
                power: 0,
                effect: Some(SkillEffect::Steal(20, 100)), // 20-100z
                duration: 0,
            },
            // Loud Exclamation - Buff ATK
            JrpgSkill {
                id: 52,
                name: "Loud Exclaim",
                sp_cost: 8,
                skill_type: SkillType::Buff,
                power: 0,
                effect: Some(SkillEffect::BuffAtk(25, 3)), // +25% ATK for 3 turns
                duration: 3,
            },
        ]
    }
}
