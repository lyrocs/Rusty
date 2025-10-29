
// Re-export core types for backward compatibility
pub use crate::core::{GamePage, GameState, MapId};

// Re-export hero types for backward compatibility
pub use crate::hero::{
    Equipment, EquipmentSlot, EquipmentType, Hero, Inventory, InventoryExt, Item,
};

// Re-export quest types for backward compatibility
pub use crate::quest::{ActiveQuest, QuestAction, QuestData, QuestObjective, QuestReward, QuestType};

// Re-export combat types for backward compatibility
pub use crate::combat::{
    ActiveStatusEffect, BattleAnimationPhase, BattleState, Circle, CircleType, CombatResult,
    Enemy, FarmState, HeroAnimation, JrpgBattleAction, JrpgBattleMenu, JrpgBattleState,
    JrpgCombatant, JrpgSkill, MonsterAnimation, MonsterAttackedAnimation, RestState,
    SkillEffect, SkillType, StatusEffectType, calculate_jrpg_damage, get_map_background,
    get_monster_attacked_gif,
};

// Re-export world types for backward compatibility
pub use crate::world::{LocationType, MapHelper, MapExit};
