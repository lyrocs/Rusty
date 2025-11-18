//! ECS Resources for stdgotchi
//!
//! Non-send resources for hardware components that cannot be shared between threads.

use bevy_ecs::prelude::*;
use crossbeam_channel::Receiver;
use esp_idf_svc::hal::gpio::PinDriver;
use std::time::Instant;

use crate::display::{Ft3x68Driver, Sh8601Driver};
use crate::game::{FragmentCollection, KillTracker, QuestManager, Rustymon, RustymonTeam, WorldMap};
use crate::input_thread::InputEvent;
use crate::ui::page::Page;
use crate::ui::pages::{BattlePage, MapPage, RustymonListPage, RustymonDetailPage, RustymonSkillsPage, FragmentCollectionPage, RustymonSummonPage, QuestListPage};

/// Display resource - NonSend because it contains non-thread-safe SPI operations
pub struct DisplayResource {
    pub display: Sh8601Driver,
}

/// Touch controller resource - NonSend because it contains non-thread-safe I2C operations
pub struct TouchResource {
    pub touch: Ft3x68Driver,
    pub last_touch_active: bool, // Track if touch was pressed last frame
}

/// GPIO resource for boot button pin
pub struct GpioResource<'d, T>
where
    T: esp_idf_svc::hal::gpio::Pin + esp_idf_svc::hal::gpio::InputPin,
{
    pub boot_pin: PinDriver<'d, T, esp_idf_svc::hal::gpio::Input>,
}

/// Button resource - NonSend because it contains non-thread-safe GPIO operations
pub struct ButtonResource {
    pub boot_last_state: bool,
    pub pwr_last_state: bool,
    pub boot_debounce: u8,
    pub pwr_debounce: u8,
}

/// Shared I2C resource - provides access to the static I2C driver
/// NonSend because I2C operations are not thread-safe
pub struct SharedI2cResource;

impl SharedI2cResource {
    /// Get mutable access to the shared I2C driver
    /// SAFETY: Safe to call in single-threaded ECS context
    pub fn get(&self) -> Option<&'static mut esp_idf_svc::hal::i2c::I2cDriver<'static>> {
        unsafe { crate::drivers::sd_cs_pin::get_shared_i2c() }
    }
}

/// Page resource - NonSend because contains Page trait objects with non-Send data
pub struct PageResource {
    pub page: Box<dyn Page>,
}

/// SD card resource for save/load
/// Generic wrapper to allow any SD card implementation
/// Uses Rc<RefCell<>> for interior mutability and cloning
pub struct SdCardWrapper {
    sd_ops: std::rc::Rc<std::cell::RefCell<Box<dyn crate::sdcard::SdCardOps>>>,
}

impl Clone for SdCardWrapper {
    fn clone(&self) -> Self {
        Self {
            sd_ops: std::rc::Rc::clone(&self.sd_ops),
        }
    }
}

impl SdCardWrapper {
    pub fn new(sd_ops: Box<dyn crate::sdcard::SdCardOps>) -> Self {
        Self {
            sd_ops: std::rc::Rc::new(std::cell::RefCell::new(sd_ops)),
        }
    }

    pub fn is_mounted(&self) -> bool {
        self.sd_ops.borrow().is_mounted()
    }

    pub fn save_to_file(&mut self, filename: &str, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.sd_ops.borrow_mut().save_to_file(filename, data)
    }

    pub fn load_from_file(&mut self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.sd_ops.borrow_mut().load_from_file(filename)
    }

    pub fn file_exists(&mut self, filename: &str) -> bool {
        self.sd_ops.borrow_mut().file_exists(filename)
    }

    pub fn load_binary_file(&mut self, filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.sd_ops.borrow_mut().load_binary_file(filename)
    }
}

// Implement SdCardOps for SdCardWrapper so it can be used with AssetLoader
impl crate::sdcard::SdCardOps for SdCardWrapper {
    fn is_mounted(&self) -> bool {
        self.sd_ops.borrow().is_mounted()
    }

    fn save_to_file(&mut self, filename: &str, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.sd_ops.borrow_mut().save_to_file(filename, data)
    }

    fn load_from_file(&mut self, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.sd_ops.borrow_mut().load_from_file(filename)
    }

    fn load_binary_file(&mut self, filename: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.sd_ops.borrow_mut().load_binary_file(filename)
    }

    fn file_exists(&mut self, filename: &str) -> bool {
        self.sd_ops.borrow_mut().file_exists(filename)
    }
}

/// Battle loading data - stores information needed to create battle page
#[derive(Clone)]
pub struct BattleLoadingData {
    pub map_id: u32,
    pub enemy_ids: Vec<u32>,
    pub initial_enemy_id: u32,
}

/// Game manager - Manages pages and game state
pub struct GameManager {
    pub menu_page: crate::ui::pages::MenuPage,
    pub map_page: MapPage,
    pub battle_page: Option<BattlePage>,
    pub death_page: Option<crate::ui::pages::DeathPage>,
    pub rustymon_list_page: RustymonListPage,
    pub rustymon_detail_page: RustymonDetailPage,
    pub rustymon_skills_page: RustymonSkillsPage,
    pub fragment_collection_page: FragmentCollectionPage,
    pub rustymon_summon_page: RustymonSummonPage,
    pub kill_tracker: KillTracker,
    pub game_data: crate::game::GameData, // Game data for items, recipes, etc.
    pub selected_map_id: Option<u32>, // Map selected for battle
    pub battle_loading_data: Option<BattleLoadingData>, // Data for deferred battle creation
    pub play_time_seconds: u64,             // Total play time
    pub session_start: Instant,             // Session start time for tracking play time
    // Rustymon system fields
    pub rustymon_collection: Vec<Rustymon>, // All owned Rustymon
    pub rustymon_team: RustymonTeam,        // Active team and bank
    pub fragment_collection: FragmentCollection, // Monster fragments
    pub selected_rustymon_index: Option<usize>, // Index of currently selected Rustymon for detail view
    pub pending_summon_rustymon: Option<Rustymon>, // Rustymon pending summon confirmation
    pub quest_manager: QuestManager,        // Quest system manager
    pub quest_list_page: QuestListPage,     // Quest list UI page
}

impl GameManager {
    pub fn new(world_map: WorldMap, game_data: crate::game::GameData) -> Self {
        // Create starter Rustymon (Poring - ID 1002, level 1) with skills
        use crate::game::RustymonFactory;

        // Get Poring data from game_data to use its stats
        let poring_data = game_data.get_enemy(1002).expect("Poring data not found");
        let starter = RustymonFactory::create_from_enemy_with_skills(
            poring_data.id,
            poring_data.name.clone(),
            poring_data.level,
            poring_data.get_element(),
            poring_data.str,
            poring_data.dex,
            poring_data.vit,
            poring_data.int,
            poring_data.luk,
            &game_data,
        );
        let starter_id = starter.id.clone();

        let mut rustymon_collection = Vec::new();
        rustymon_collection.push(starter);

        let mut rustymon_team = RustymonTeam::new();
        rustymon_team.add_rustymon(starter_id); // Add to team and set as active

        log::info!("🎮 New game started with starter Rustymon: Poring");

        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::new(world_map, None), // Use embedded map backgrounds
            battle_page: None,
            death_page: None,
            rustymon_list_page: RustymonListPage::new(),
            rustymon_detail_page: RustymonDetailPage::new(),
            rustymon_skills_page: RustymonSkillsPage::new(),
            fragment_collection_page: FragmentCollectionPage::new(),
            rustymon_summon_page: RustymonSummonPage::new(),
            kill_tracker: KillTracker::new(),
            game_data,
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: 0,
            session_start: Instant::now(),
            rustymon_collection,
            rustymon_team,
            fragment_collection: FragmentCollection::new(),
            selected_rustymon_index: None,
            pending_summon_rustymon: None,
            quest_manager: QuestManager::new(),
            quest_list_page: QuestListPage::new(),
        }
    }

    /// Create GameManager from save data
    pub fn from_save_data(save_data: crate::game::SaveData, world_map: WorldMap, game_data: crate::game::GameData) -> Self {
        // Learn skills for all existing Rustymon (migration for old saves)
        let mut rustymon_collection = save_data.rustymon_collection;
        for rustymon in &mut rustymon_collection {
            if let Some(enemy_data) = game_data.get_enemy(rustymon.species_id) {
                let newly_learned = rustymon.check_and_learn_skills(&enemy_data.learnable_skills);
                if !newly_learned.is_empty() {
                    log::info!("✨ {} learned {} skills (migration)", rustymon.name, newly_learned.len());
                }
            }
        }

        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::from_save(world_map, save_data.current_location_id, None), // Use embedded map backgrounds
            battle_page: None,
            death_page: None,
            rustymon_list_page: RustymonListPage::new(),
            rustymon_detail_page: RustymonDetailPage::new(),
            rustymon_skills_page: RustymonSkillsPage::new(),
            fragment_collection_page: FragmentCollectionPage::new(),
            rustymon_summon_page: RustymonSummonPage::new(),
            kill_tracker: save_data.kill_tracker,
            game_data,
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: save_data.play_time_seconds,
            session_start: Instant::now(),
            rustymon_collection,
            rustymon_team: save_data.rustymon_team,
            fragment_collection: save_data.fragment_collection,
            selected_rustymon_index: None,
            pending_summon_rustymon: None,
            quest_manager: save_data.quest_manager,
            quest_list_page: QuestListPage::new(),
        }
    }

    /// Get current page based on mode (for standard Page trait operations)
    pub fn get_current_page(&mut self, mode: AppMode) -> Option<&mut dyn Page> {
        match mode {
            AppMode::Menu => Some(&mut self.menu_page as &mut dyn Page),
            AppMode::Map => Some(&mut self.map_page as &mut dyn Page),
            AppMode::BattleLoading => None, // Loading screen has no page
            AppMode::Battle => {
                if let Some(ref mut battle_page) = self.battle_page {
                    Some(battle_page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::Death => {
                if let Some(ref mut death_page) = self.death_page {
                    Some(death_page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::RustymonList => Some(&mut self.rustymon_list_page as &mut dyn Page),
            AppMode::RustymonDetail => Some(&mut self.rustymon_detail_page as &mut dyn Page),
            AppMode::RustymonSkills => Some(&mut self.rustymon_skills_page as &mut dyn Page),
            AppMode::FragmentCollection => Some(&mut self.fragment_collection_page as &mut dyn Page),
            AppMode::RustymonSummon => Some(&mut self.rustymon_summon_page as &mut dyn Page),
            AppMode::QuestList => Some(&mut self.quest_list_page as &mut dyn Page),
        }
    }

    /// Draw rustymon list page with collection and team data
    pub fn draw_rustymon_list(&mut self, display: &mut crate::display::Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.rustymon_list_page.draw_rustymon_list(display, &self.rustymon_collection, &self.rustymon_team, full_redraw)
    }

    /// Draw rustymon detail page
    pub fn draw_rustymon_detail(&mut self, display: &mut crate::display::Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Get the selected rustymon
        if let Some(index) = self.selected_rustymon_index {
            if let Some(rustymon) = self.rustymon_collection.get(index) {
                // Load idle animation (will always load when entering since we clear on transition)
                if !self.rustymon_detail_page.has_idle_animation() {
                    if let Err(e) = self.rustymon_detail_page.load_idle_animation(rustymon.species_id) {
                        log::warn!("Failed to load idle animation for species {}: {:?}", rustymon.species_id, e);
                    }
                }
                return self.rustymon_detail_page.draw_rustymon_detail(display, rustymon, &self.rustymon_team, &self.game_data, full_redraw);
            }
        }
        // If no rustymon selected, just clear the screen
        Ok(())
    }

    /// Draw rustymon skills page
    pub fn draw_rustymon_skills(&mut self, display: &mut crate::display::Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Get the selected rustymon
        if let Some(index) = self.selected_rustymon_index {
            if let Some(rustymon) = self.rustymon_collection.get(index) {
                return self.rustymon_skills_page.draw_skills(display, rustymon, &self.game_data, full_redraw);
            }
        }
        // If no rustymon selected, just clear the screen
        Ok(())
    }

    /// Draw fragment collection page
    pub fn draw_fragment_collection(&mut self, display: &mut crate::display::Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.fragment_collection_page.draw_fragment_collection(display, &self.fragment_collection, &self.game_data, full_redraw)
    }

    /// Draw rustymon summon preview page
    pub fn draw_rustymon_summon(&mut self, display: &mut crate::display::Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Get the pending summon rustymon
        if let Some(ref rustymon) = self.pending_summon_rustymon {
            return self.rustymon_summon_page.draw_summon_preview(display, rustymon, full_redraw);
        }
        // If no pending summon, just clear the screen
        Ok(())
    }

    /// Draw quest list page
    pub fn draw_quest_list(&mut self, display: &mut crate::display::Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.quest_list_page.draw_quest_list(display, &self.quest_manager, &self.game_data, full_redraw)
    }

    /// Save game state to SD card
    pub fn save_to_sd(&mut self, sd_card: &mut SdCardWrapper, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Update total play time
        let session_duration = self.session_start.elapsed().as_secs();
        self.play_time_seconds += session_duration;
        self.session_start = Instant::now();

        // Create save data
        let current_location_id = self.map_page.world_map().current_location_id();
        let save_data = crate::game::SaveData::new(
            self.kill_tracker.clone(),
            current_location_id,
            self.play_time_seconds,
            self.rustymon_collection.clone(),
            self.rustymon_team.clone(),
            self.fragment_collection.clone(),
            self.quest_manager.clone(),
        );

        // Serialize to JSON
        let json = save_data.to_json()?;

        // Save to SD card
        sd_card.save_to_file(filename, &json)?;
        log::info!("Game saved to {}", filename);
        Ok(())
    }

    /// Auto-save game state (called after important events)
    pub fn auto_save(&mut self, sd_card: &mut Option<&mut SdCardWrapper>, filename: &str) {
        // Sync battle state before saving
        self.sync_battle_state();

        let Some(sd_card) = sd_card else {
            return;
        };

        if !sd_card.is_mounted() {
            return;
        }

        if let Err(e) = self.save_to_sd(sd_card, filename) {
            log::error!("Auto-save failed: {:?}", e);
        }
    }

    /// Sync kill tracker, fragments, and Rustymon from battle page back to GameManager
    /// This ensures battle progress is saved
    pub fn sync_battle_state(&mut self) {
        // Extract data from battle page first, then process
        let battle_data = if let Some(ref mut battle_page) = self.battle_page {
            let new_kill_tracker = battle_page.get_kill_tracker().clone();
            let new_rustymon_collection = battle_page.get_rustymon_collection().clone();
            let new_rustymon_team = battle_page.get_rustymon_team().clone();
            let fragment_drops = battle_page.take_fragment_drops();
            Some((
                new_kill_tracker,
                new_rustymon_collection,
                new_rustymon_team,
                fragment_drops,
            ))
        } else {
            None
        };

        if let Some((new_kill_tracker, new_rustymon_collection, new_rustymon_team, fragment_drops)) =
            battle_data
        {
            // Get kill diff before syncing (for quest events)
            let old_kills = self.kill_tracker.clone();
            self.kill_tracker = new_kill_tracker;

            // Calculate kills made in this sync
            let new_kills = self.calculate_kill_diff(&old_kills, &self.kill_tracker);

            // Sync Rustymon collection (EXP, levels, HP changes)
            let old_levels: std::collections::HashMap<String, u32> = self
                .rustymon_collection
                .iter()
                .map(|r| (r.id.clone(), r.level))
                .collect();
            self.rustymon_collection = new_rustymon_collection;
            self.rustymon_team = new_rustymon_team;

            // Sync fragment drops and track for quests
            for (enemy_id, _enemy_name) in &fragment_drops {
                self.fragment_collection.add_fragment(*enemy_id, 1);

                // Process quest event for fragment collection
                let event = crate::game::QuestEvent::FragmentCollected {
                    monster_id: *enemy_id,
                    amount: 1,
                };
                self.quest_manager
                    .process_event(&event, self.game_data.get_all_quests());
            }

            // Process quest events for kills
            for (monster_id, count) in new_kills {
                for _ in 0..count {
                    let event = crate::game::QuestEvent::MonsterKilled { monster_id };
                    self.quest_manager
                        .process_event(&event, self.game_data.get_all_quests());
                }
            }

            // Process quest event for level ups
            for rustymon in &self.rustymon_collection {
                if let Some(&old_level) = old_levels.get(&rustymon.id) {
                    if rustymon.level > old_level {
                        let event = crate::game::QuestEvent::LevelReached {
                            level: rustymon.level,
                        };
                        self.quest_manager
                            .process_event(&event, self.game_data.get_all_quests());
                    }
                }
            }

            // Log Rustymon sync for debugging
            if let Some(rustymon_id) = self.rustymon_team.get_active_rustymon_id() {
                if let Some(rustymon) = self
                    .rustymon_collection
                    .iter()
                    .find(|r| &r.id == rustymon_id)
                {
                    log::debug!(
                        "Synced Rustymon: {} Lv{}, {}/{} EXP",
                        rustymon.name,
                        rustymon.level,
                        rustymon.exp,
                        rustymon.exp_to_next
                    );
                }
            }
        }
    }

    /// Calculate kill differences between two kill trackers
    fn calculate_kill_diff(
        &self,
        old: &KillTracker,
        new: &KillTracker,
    ) -> Vec<(u32, u32)> {
        let mut diff = Vec::new();

        // For each enemy in new tracker, check if count increased
        for (enemy_id, new_count) in new.get_all_kills() {
            let old_count = old.get_kills(*enemy_id);
            if *new_count > old_count {
                diff.push((*enemy_id, *new_count - old_count));
            }
        }

        diff
    }

    /// Process battle won event for quests
    pub fn process_battle_won(&mut self) {
        let event = crate::game::QuestEvent::BattleWon;
        self.quest_manager
            .process_event(&event, self.game_data.get_all_quests());
        log::debug!("Quest event: BattleWon processed");
    }

    /// Check and reset daily quests if needed
    pub fn check_quest_resets(&mut self) {
        if self.quest_manager.should_reset_daily() {
            self.quest_manager
                .reset_daily_quests(self.game_data.get_all_quests());
            log::info!("Daily quests have been reset");
        }

        if self.quest_manager.should_reset_weekly() {
            self.quest_manager
                .reset_weekly_quests(self.game_data.get_all_quests());
            log::info!("Weekly quests have been reset");
        }
    }

    /// Auto-start available daily quests
    pub fn auto_start_daily_quests(&mut self) {
        let player_level = self.get_player_level();
        let available = self
            .quest_manager
            .get_available_quests(self.game_data.get_all_quests(), player_level);

        // Start all available daily quests
        for quest in available {
            if quest.is_daily() && !self.quest_manager.is_quest_active(quest.id) {
                self.quest_manager.start_quest(quest);
            }
        }
    }

    /// Get player level (highest level Rustymon in team)
    fn get_player_level(&self) -> u32 {
        self.rustymon_collection
            .iter()
            .filter(|r| self.rustymon_team.get_team_ids().contains(&r.id))
            .map(|r| r.level)
            .max()
            .unwrap_or(1)
    }
}

/// Input event channel resource - receives events from the input thread on Core 0
/// This is a regular Resource (not NonSend) because the Receiver is Send
#[derive(Resource)]
pub struct InputEventChannel {
    pub receiver: Receiver<InputEvent>,
}

/// Pending input events resource - stores events that button_system didn't consume
/// This allows touch events to be passed through to other systems
#[derive(Resource, Default)]
pub struct PendingInputEvents {
    pub events: Vec<InputEvent>,
}

/// App state resource
#[derive(Resource)]
pub struct AppState {
    pub needs_redraw: bool,
    pub current_mode: AppMode,
    pub fps: f32,
    pub frame_count: u32,
    pub last_fps_update: Instant,
    pub screen_on: bool, // Screen power state (controlled by PWR button)
}

/// Application modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Menu screen
    Menu,
    /// Map navigation
    Map,
    /// Loading screen before battle
    BattleLoading,
    /// Battle mode
    Battle,
    /// Death screen (hero died)
    Death,
    /// Rustymon list screen
    RustymonList,
    /// Rustymon detail screen
    RustymonDetail,
    /// Rustymon skills screen
    RustymonSkills,
    /// Fragment collection screen
    FragmentCollection,
    /// Rustymon summon preview screen
    RustymonSummon,
    /// Quest list screen
    QuestList,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            needs_redraw: true,
            current_mode: AppMode::Menu,
            fps: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            screen_on: true, // Screen starts on
        }
    }
}
