/// JRPG battle system
///
/// Turn-based combat system with combatants, actions, and menus.

use heapless::Vec as HeaplessVec;
use super::skills::{ActiveStatusEffect, JrpgSkill};

/// JRPG Battle State - Turn-based combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JrpgBattleState {
    Start,           // Battle start - show enemy encounter
    PlayerTurn,      // Player choosing action
    PlayerAction,    // Player action animation
    EnemyTurn,       // Enemy choosing action (auto)
    EnemyAction,     // Enemy action animation
    Victory,         // Battle won
    Defeat,          // Battle lost
}

/// JRPG Battle Action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JrpgBattleAction {
    Attack,    // Basic attack
    Skill,     // Use skill (costs SP)
    Item,      // Use item
    Defend,    // Reduce damage taken
}

/// JRPG Battle Menu Selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JrpgBattleMenu {
    Main,      // Main menu: Attack + Skills (direct execution)
}

/// JRPG Battle Combatant (for both hero and enemy)
#[derive(Debug, Clone)]
pub struct JrpgCombatant {
    pub name: &'static str,
    pub level: u16,
    pub hp: u16,
    pub max_hp: u16,
    pub sp: u16,
    pub max_sp: u16,
    pub attack: u16,
    pub defense: u16,

    // New stats for improved combat
    pub agility: u16,      // For double attack chance
    pub luck: u16,         // For critical/lucky hits
    pub intelligence: u16, // For magic damage
    pub dexterity: u16,    // For accuracy (future)

    // Active status effects (max 8 active effects)
    pub active_effects: HeaplessVec<ActiveStatusEffect, 8>,

    // Available skills (max 3 skills)
    pub available_skills: HeaplessVec<JrpgSkill, 3>,
}
