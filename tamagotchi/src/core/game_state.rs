/// Core game state management
///
/// This module contains the main GameState struct that holds all game state.
/// In future refactoring phases, this will be split into domain-specific state components.

use bevy_ecs::prelude::*;
use heapless::Vec as HeaplessVec;

use crate::core::{GamePage, MapId, MAP_PRONTERA_ID};
use crate::core::constants::*;
use crate::hero::{EquipmentSlot, Hero};
use crate::quest::ActiveQuest;
use crate::combat::{
    BattleAnimationPhase, BattleState, Circle, CombatResult, Enemy,
    FarmState, HeroAnimation, JrpgBattleMenu, JrpgBattleState,
    JrpgCombatant, MonsterAnimation, RestState,
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
    pub selected_quest_id: Option<u32>,         // Quest ID for details view (None = list view)
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
            selected_quest_id: None,
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

impl GameState {
    /// Serialize quest data to string for SD card persistence
    /// Format: daily_refresh_time|completed_ids|active_quests
    /// Where active_quests is semicolon-separated: quest_id,p0,p1,p2,p3,completed,claimed
    pub fn quests_to_save_string(&self) -> heapless::String<1024> {
        use core::fmt::Write;

        let mut save_str = heapless::String::<1024>::new();

        // Write daily refresh time
        write!(save_str, "{}|", self.daily_quest_refresh_time).ok();

        // Write completed quest IDs (comma-separated)
        for (i, quest_id) in self.completed_quest_ids.iter().enumerate() {
            if i > 0 {
                write!(save_str, ",{}", quest_id).ok();
            } else {
                write!(save_str, "{}", quest_id).ok();
            }
        }

        write!(save_str, "|").ok();

        // Write active quests (semicolon-separated)
        for (i, quest) in self.active_quests.iter().enumerate() {
            if i > 0 {
                write!(save_str, ";").ok();
            }
            write!(
                save_str,
                "{},{},{},{},{},{},{}",
                quest.quest_id,
                quest.progress[0],
                quest.progress[1],
                quest.progress[2],
                quest.progress[3],
                if quest.completed { 1 } else { 0 },
                if quest.claimed { 1 } else { 0 }
            ).ok();
        }

        save_str
    }

    /// Deserialize quest data from string
    pub fn quests_from_save_string(&mut self, save_str: &str) {
        let parts: heapless::Vec<&str, 3> = save_str.split('|').collect();

        if parts.len() != 3 {
            esp_println::println!("[LOAD] Invalid quest save format");
            return;
        }

        // Parse daily refresh time
        if let Ok(refresh_time) = parts[0].parse::<u32>() {
            self.daily_quest_refresh_time = refresh_time;
        }

        // Parse completed quest IDs
        self.completed_quest_ids.clear();
        if !parts[1].is_empty() {
            for id_str in parts[1].split(',') {
                if let Ok(quest_id) = id_str.parse::<u32>() {
                    self.completed_quest_ids.push(quest_id).ok();
                }
            }
        }

        // Parse active quests
        self.active_quests.clear();
        if !parts[2].is_empty() {
            for quest_str in parts[2].split(';') {
                let quest_parts: heapless::Vec<&str, 7> = quest_str.split(',').collect();
                if quest_parts.len() == 7 {
                    if let (Ok(quest_id), Ok(p0), Ok(p1), Ok(p2), Ok(p3), Ok(completed), Ok(claimed)) = (
                        quest_parts[0].parse::<u32>(),
                        quest_parts[1].parse::<u16>(),
                        quest_parts[2].parse::<u16>(),
                        quest_parts[3].parse::<u16>(),
                        quest_parts[4].parse::<u16>(),
                        quest_parts[5].parse::<u8>(),
                        quest_parts[6].parse::<u8>(),
                    ) {
                        let active_quest = ActiveQuest {
                            quest_id,
                            progress: [p0, p1, p2, p3],
                            completed: completed != 0,
                            claimed: claimed != 0,
                        };
                        self.active_quests.push(active_quest).ok();
                    }
                }
            }
        }

        esp_println::println!(
            "[LOAD] Loaded {} active quests, {} completed",
            self.active_quests.len(),
            self.completed_quest_ids.len()
        );
    }
}
