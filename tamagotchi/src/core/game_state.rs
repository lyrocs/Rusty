/// Core game state management
///
/// This module contains the main GameState struct that holds all game state.
/// In future refactoring phases, this will be split into domain-specific state components.

use bevy_ecs::prelude::*;
use heapless::Vec as HeaplessVec;

use crate::core::{GamePage, MapId, MAP_PRONTERA_ID};
use crate::core::constants::*;
use crate::tamagotchi::{
    ActiveQuest, BattleAnimationPhase, BattleState, Circle, CombatResult, Enemy,
    EquipmentSlot, FarmState, Hero, HeroAnimation, JrpgBattleMenu, JrpgBattleState,
    JrpgCombatant, MonsterAnimation, MonsterAttackedAnimation, RestState,
};

/// Main game state containing all game data and UI state
#[derive(Resource)]
pub struct GameState {
    pub current_page: GamePage,
    pub hero: Hero,
    pub current_location: MapId, // Current map location
    pub current_enemy: Option<Enemy>,
    pub farm_state: FarmState,
    pub farm_progress: u32,       // 0-60000 (60 seconds in milliseconds)
    pub farm_duration_ms: u32,    // 60000 ms = 1 minute
    pub farm_touch_cooldown: u32, // Cooldown in ms to prevent immediate re-touch
    pub rest_state: RestState,
    pub rest_progress: u32,                  // Progress in milliseconds
    pub sp_regen_rate: u16,                  // SP per second while resting
    pub menu_selection: u8, // 0 = Overview, 1 = Farm, 2 = Rest, 3 = Battle, 4 = Save
    pub battle_state: BattleState, // Current battle state
    pub battle_enemy: Option<Enemy>, // Enemy being fought
    pub battle_circles: [Option<Circle>; 4], // Up to 4 active circles
    pub battle_score: u16,  // Hits made in current battle
    pub battle_missed: u16, // Circles missed or bad targets hit
    pub battle_combo: u16,  // Current combo count (consecutive green hits)
    pub battle_next_spawn: u32, // When next circle spawns
    pub battle_spawn_interval: u32, // Time between spawns (800ms)
    pub battle_duration: u32, // Total battle time (30 seconds)
    pub battle_elapsed: u32, // Time elapsed in battle
    pub battle_last_touch_x: i32, // Last touch X position for debug display
    pub battle_last_touch_y: i32, // Last touch Y position for debug display
    pub battle_last_touch_time: u32, // When last touch occurred (for fade out)
    pub battle_end_time: u32, // When battle ended (for preventing accidental clicks)
    pub battle_animation_phase: BattleAnimationPhase, // Current animation phase
    pub battle_animation_phase_started_ms: u32, // When current phase started
    // JRPG Battle state
    pub jrpg_battle_state: JrpgBattleState,    // Current JRPG battle state
    pub jrpg_battle_menu: JrpgBattleMenu,      // Current menu
    pub jrpg_menu_selection: u8,               // Current menu item selected (0-4)
    pub jrpg_hero_combatant: Option<JrpgCombatant>, // Hero battle stats
    pub jrpg_enemy_combatant: Option<JrpgCombatant>, // Enemy battle stats
    pub jrpg_battle_message: Option<&'static str>, // Battle message (e.g., "Hero attacks!")
    pub jrpg_battle_message_timer: u32,        // How long to show message
    pub jrpg_damage_dealt: u16,                // Last damage dealt (for display)
    pub jrpg_damage_animation_timer: u32,      // Timer for damage text animation (0-1000ms)
    pub jrpg_damage_x: i32,                    // X position for damage text
    pub jrpg_damage_y: i32,                    // Y position for damage text
    pub jrpg_action_animation_timer: u32,      // Timer for action animations
    pub jrpg_combo_count: u8,                  // Current combo count (hits in a row)
    pub jrpg_combo_ready: bool,                // Combo attack available (3 hits)
    pub jrpg_last_combat_result: CombatResult, // Last attack result (normal/crit/lucky)
    pub jrpg_skill_menu_selection: u8,         // Selected skill in skill menu (0-2)
    pub jrpg_selected_skill_index: Option<usize>, // Index of skill being used
    // Equipment refinement UI state
    pub equipment_selection_open: bool,         // Whether equipment selection menu is shown
    pub refine_popup_open: bool,                // Whether refine popup is shown
    pub refine_slot: Option<EquipmentSlot>,     // Which slot is being refined
    pub refine_result_message: Option<&'static str>, // Result message (success/failure)
    pub refine_result_timer: u32,               // How long to show result (0-2000ms)
    // Quest system state
    pub active_quests: HeaplessVec<ActiveQuest, 16>, // Currently active quests
    pub completed_quest_ids: HeaplessVec<u32, 64>, // IDs of all completed quests
    pub daily_quest_refresh_time: u32,          // When daily quests last refreshed (ms)
    pub quest_page_scroll: u8,                  // Scroll position in quest list (0-255)
    pub last_update_ms: u32, // Last update time for progress tracking
    pub save_requested: bool, // Flag to trigger save
    pub save_status_msg: Option<&'static str>, // Status message after save
    pub save_status_timeout: u32, // Time when save message should clear (0 = no message)
    pub fps: u32,           // Current FPS
    pub frame_count: u32,   // Total frames rendered
    pub last_fps_update_ms: u32, // Last time FPS was calculated
    pub needs_redraw: bool, // Flag to indicate screen needs redrawing
    pub screen_on: bool,    // Screen power state (controlled by PWR button)
    pub last_drops: HeaplessVec<(u32, &'static str, u16), 4>, // Last items that dropped
    pub brightness: u8,     // Screen brightness (0-255)
    pub monster_animation: MonsterAnimation, // Current monster animation
    pub monster_animation_frame: usize, // Current frame in animation
    pub monster_animation_started_ms: u32, // When current animation started
    pub monster_attacked_animation: MonsterAttackedAnimation, // Monster attacked state
    pub monster_attacked_frame: usize, // Current frame in attacked animation
    pub monster_attacked_started_ms: u32, // When monster attacked animation started
    pub hero_animation: HeroAnimation, // Current hero animation
    pub hero_animation_frame: usize, // Current frame in hero animation
    pub hero_animation_started_ms: u32, // When current hero animation started
    pub last_attack_animation_ms: u32, // When last attack animation was triggered (for timing)
    pub last_hero_attack_ms: u32, // When hero last attacked (for triggering hero attack anim)
    pub map_monster_animation_frame: usize, // Current frame for monster idle animations on map page
    pub map_monster_animation_last_update: u32, // Last time map monster animation was updated
    pub gif_animation_clock_ms: u32, // Global clock for synchronized GIF animations (increments every 100ms)
    pub gif_animation_last_update_ms: u32, // Last time GIF animation clock was updated
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_page: GamePage::Overview,
            hero: Hero::new(),
            current_location: MAP_PRONTERA_ID, // Start in Prontera (ID from maps.json)
            current_enemy: None,
            farm_state: FarmState::Idle,
            farm_progress: 0,
            farm_duration_ms: DEFAULT_FARM_DURATION_MS, // 1 minute
            farm_touch_cooldown: 0,
            rest_state: RestState::Resting,
            rest_progress: 0,
            sp_regen_rate: DEFAULT_SP_REGEN_RATE, // 5 SP per second
            menu_selection: 0,
            battle_state: BattleState::Idle,
            battle_enemy: None,
            battle_circles: [None, None, None, None],
            battle_score: 0,
            battle_missed: 0,
            battle_combo: 0,
            battle_next_spawn: 0,
            battle_spawn_interval: DEFAULT_BATTLE_SPAWN_INTERVAL_MS, // 800ms between spawns
            battle_duration: DEFAULT_BATTLE_DURATION_MS,     // 30 seconds
            battle_elapsed: 0,
            battle_last_touch_x: 0,
            battle_last_touch_y: 0,
            battle_last_touch_time: 0,
            battle_end_time: 0,
            battle_animation_phase: BattleAnimationPhase::BothIdle,
            battle_animation_phase_started_ms: 0,
            jrpg_battle_state: JrpgBattleState::Start,
            jrpg_battle_menu: JrpgBattleMenu::Main,
            jrpg_menu_selection: 0,
            jrpg_hero_combatant: None,
            jrpg_enemy_combatant: None,
            jrpg_battle_message: None,
            jrpg_battle_message_timer: 0,
            jrpg_damage_dealt: 0,
            jrpg_damage_animation_timer: 0,
            jrpg_damage_x: 0,
            jrpg_damage_y: 0,
            jrpg_action_animation_timer: 0,
            jrpg_combo_count: 0,
            jrpg_combo_ready: false,
            jrpg_last_combat_result: CombatResult::Normal,
            jrpg_skill_menu_selection: 0,
            jrpg_selected_skill_index: None,
            // Equipment refinement UI state
            equipment_selection_open: false,
            refine_popup_open: false,
            refine_slot: None,
            refine_result_message: None,
            refine_result_timer: 0,
            // Quest system state
            active_quests: HeaplessVec::new(),
            completed_quest_ids: HeaplessVec::new(),
            daily_quest_refresh_time: 0,
            quest_page_scroll: 0,
            last_update_ms: 0,
            save_requested: false,
            save_status_msg: None,
            save_status_timeout: 0,
            fps: 0,
            frame_count: 0,
            last_fps_update_ms: 0,
            needs_redraw: true, // Start with needing a redraw
            screen_on: true,    // Screen starts on
            last_drops: HeaplessVec::new(),
            brightness: DEFAULT_BRIGHTNESS, // 80% brightness by default (204/255 = 0.8)
            monster_animation: MonsterAnimation::Idle,
            monster_animation_frame: 0,
            monster_animation_started_ms: 0,
            monster_attacked_animation: MonsterAttackedAnimation::Normal,
            monster_attacked_frame: 0,
            monster_attacked_started_ms: 0,
            hero_animation: HeroAnimation::Idle,
            hero_animation_frame: 0,
            hero_animation_started_ms: 0,
            last_attack_animation_ms: 0,
            last_hero_attack_ms: 0,
            map_monster_animation_frame: 0,
            map_monster_animation_last_update: 0,
            gif_animation_clock_ms: 0,
            gif_animation_last_update_ms: 0,
        }
    }
}
