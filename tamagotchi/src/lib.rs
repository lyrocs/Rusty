#![no_std]

extern crate alloc;

// Clean architecture modules (organized by domain)
pub mod core;       // Core game state and types
pub mod hero;       // Hero domain (character, stats, inventory, equipment)
pub mod combat;     // Combat domain (enemies, battles, skills, animations)
pub mod quest;      // Quest domain (quests, objectives, rewards)
pub mod world;      // World domain (maps, navigation, locations)
pub mod systems;    // ECS systems (organized by responsibility)
pub mod data;       // Game data (enemies, maps, items, NPCs, drops)
pub mod ui;         // UI rendering (pages, components, helpers)

// Hardware and infrastructure
pub mod drivers;
pub mod display;
pub mod ecs;
pub mod utils;

// ==============================================================================
// Backward Compatibility Layer
// ==============================================================================
// The `tamagotchi` module provides backward compatibility by re-exporting all
// types from their new clean architecture locations. This allows existing code
// using `crate::tamagotchi::*` imports to continue working without changes.

/// Backward compatibility module - re-exports types from clean architecture modules
pub mod tamagotchi {
    // Re-export core types
    pub use crate::core::{GamePage, GameState, MapId};

    // Re-export hero types
    pub use crate::hero::{
        Equipment, EquipmentSlot, EquipmentType, Hero, Inventory, InventoryExt, Item,
    };

    // Re-export quest types
    pub use crate::quest::{ActiveQuest, QuestAction, QuestData, QuestObjective, QuestReward, QuestType};

    // Re-export combat types
    pub use crate::combat::{
        ActiveStatusEffect, BattleAnimationPhase, BattleState, Circle, CircleType, CombatResult,
        Enemy, FarmState, HeroAnimation, JrpgBattleAction, JrpgBattleMenu, JrpgBattleState,
        JrpgCombatant, JrpgSkill, MonsterAnimation, MonsterAttackedAnimation, RestState,
        SkillEffect, SkillType, StatusEffectType, calculate_jrpg_damage, get_map_background,
        get_monster_attacked_gif,
    };

    // Re-export world types
    pub use crate::world::{LocationType, MapHelper, MapExit};

    // Re-export modules under old names
    pub use crate::systems as systems;
    pub use crate::quest::system as quest_system;
    pub use crate::ui as ui;
    pub use crate::data as game_data;

    // Re-export everything from these modules
    pub use systems::*;
    pub use quest_system::*;
    pub use ui::*;
    pub use game_data::*;

    /// Legacy `models` submodule for `crate::tamagotchi::models::*` imports
    pub mod models {
        pub use super::*;
    }
}
