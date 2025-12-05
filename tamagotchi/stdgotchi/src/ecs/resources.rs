//! ECS Resources for stdgotchi
//!
//! Non-send resources for hardware components that cannot be shared between threads.

use bevy_ecs::prelude::*;
use crossbeam_channel::Receiver;
use esp_idf_svc::hal::gpio::PinDriver;
use std::time::Instant;

use crate::display::{Ft3x68Driver, Sh8601Driver};
use crate::game::{Hero, KillTracker, QuestManager, WorldMap, MvpSpawnManager};
use crate::input_thread::InputEvent;
use crate::ui::page::Page;
use crate::ui::pages::{AfkFarmPage, BattlePage, MapPage, QuestListPage};

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

/// WiFi resource - NonSend because WiFi operations are not thread-safe
/// Keeps the WiFi connection alive for the duration of the program
pub struct WifiResource {
    pub wifi: esp_idf_svc::wifi::BlockingWifi<esp_idf_svc::wifi::EspWifi<'static>>,
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

/// Expedition data - stores information about an ongoing expedition
#[derive(Clone)]
pub struct ExpeditionData {
    pub enemy_id: u32,
    pub enemy_name: String,
    pub target_kills: u32,
    pub expedition_size: crate::game::ExpeditionSize,
    pub start_time: Instant,
    pub end_time: Instant, // When expedition will complete
    pub result: crate::game::ExpeditionResult,
}

/// Game manager - Manages pages and game state
pub struct GameManager {
    pub menu_page: crate::ui::pages::MenuPage,
    pub map_page: MapPage,
    pub battle_page: Option<BattlePage>,
    pub battle_result_page: Option<crate::ui::pages::BattleResultPage>,
    pub death_page: Option<crate::ui::pages::DeathPage>,
    pub rest_page: Option<crate::ui::pages::RestPage>,
    pub afk_farm_page: Option<crate::ui::pages::AfkFarmPage>,
    pub kill_tracker: KillTracker,
    pub game_data: crate::game::GameData, // Game data for items, recipes, etc.
    pub selected_map_id: Option<u32>, // Map selected for battle
    pub battle_loading_data: Option<BattleLoadingData>, // Data for deferred battle creation
    pub play_time_seconds: u64,             // Total play time
    pub session_start: Instant,             // Session start time for tracking play time
    // Hero system fields
    pub hero: Hero,                         // The player's hero
    pub quest_manager: QuestManager,        // Quest system manager
    pub quest_list_page: QuestListPage,     // Quest list UI page
    pub hero_info_page: Option<crate::ui::pages::HeroInfoPage>, // Hero info UI page
    pub cards_page: Option<crate::ui::pages::CardsPage>, // Card collection UI page
    pub pokemon_api_response: Option<String>, // Pokemon API response data
    // Expedition system fields
    pub expedition_setup_page: Option<crate::ui::pages::ExpeditionSetupPage>,
    pub expedition_in_progress_page: Option<crate::ui::pages::ExpeditionInProgressPage>,
    pub expedition_summary_page: Option<crate::ui::pages::ExpeditionSummaryPage>,
    pub expedition_data: Option<ExpeditionData>, // Ongoing expedition state
    // MVP spawn management
    pub mvp_spawn_manager: MvpSpawnManager,
    // Semi-active battle and skill selection pages
    pub skill_selection_page: Option<crate::ui::pages::SkillSelectionPage>,
    pub semi_active_battle_page: Option<crate::ui::pages::SemiActiveBattlePage>,
    // Hunt system pages
    pub hunt_monster_list_page: Option<crate::ui::pages::HuntMonsterListPage>,
    pub hunt_battle_result_page: Option<crate::ui::pages::HuntBattleResultPage>,
    pub hunt_map_id: Option<u32>,  // Current map for hunting
    pub hunt_enemy_id: Option<u32>,  // Current enemy being hunted
}

impl GameManager {
    pub fn new(world_map: WorldMap, game_data: crate::game::GameData) -> Self {
        // Create a new hero with Novice job class
        let hero = Hero::new("Hero".to_string());

        log::info!("🎮 New game started with hero: {}, Job: Novice", hero.name);

        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::new(world_map, None), // Use embedded map backgrounds
            battle_page: None,
            battle_result_page: None,
            death_page: None,
            rest_page: None,
            afk_farm_page: None,
            kill_tracker: KillTracker::new(),
            game_data,
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: 0,
            session_start: Instant::now(),
            hero,
            quest_manager: QuestManager::new(),
            quest_list_page: QuestListPage::new(),
            hero_info_page: None,
            cards_page: None,
            pokemon_api_response: None,
            expedition_setup_page: None,
            expedition_in_progress_page: None,
            expedition_summary_page: None,
            expedition_data: None,
            mvp_spawn_manager: MvpSpawnManager::new(),
            skill_selection_page: None,
            semi_active_battle_page: None,
            hunt_monster_list_page: None,
            hunt_battle_result_page: None,
            hunt_map_id: None,
            hunt_enemy_id: None,
        }
    }

    /// Create GameManager from save data
    pub fn from_save_data(save_data: crate::game::SaveData, world_map: WorldMap, game_data: crate::game::GameData) -> Self {
        // Migrate cards to add missing unlocks_skill from game data
        let mut hero = save_data.hero;
        let mut cards_migrated = 0;
        for card in &mut hero.cards {
            if card.unlocks_skill.is_none() {
                // Look up the enemy data to get the correct unlocks_skill
                if let Some(enemy_data) = game_data.get_enemy(card.monster_id) {
                    if enemy_data.card.unlocks_skill.is_some() {
                        card.unlocks_skill = enemy_data.card.unlocks_skill;
                        cards_migrated += 1;
                    }
                }
            }
        }
        if cards_migrated > 0 {
            log::info!("Migrated {} cards with unlocks_skill data", cards_migrated);
        }

        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::from_save(world_map, save_data.current_location_id, None), // Use embedded map backgrounds
            battle_page: None,
            battle_result_page: None,
            death_page: None,
            rest_page: None,
            afk_farm_page: None,
            expedition_setup_page: None,
            expedition_in_progress_page: None,
            expedition_summary_page: None,
            expedition_data: None,
            kill_tracker: save_data.kill_tracker,
            game_data,
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: save_data.play_time_seconds,
            session_start: Instant::now(),
            hero,
            quest_manager: save_data.quest_manager,
            quest_list_page: QuestListPage::new(),
            hero_info_page: None,
            cards_page: None,
            pokemon_api_response: None,
            mvp_spawn_manager: save_data.mvp_spawn_manager,
            skill_selection_page: None,
            semi_active_battle_page: None,
            hunt_monster_list_page: None,
            hunt_battle_result_page: None,
            hunt_map_id: None,
            hunt_enemy_id: None,
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
            AppMode::BattleResult => {
                if let Some(ref mut battle_result_page) = self.battle_result_page {
                    Some(battle_result_page as &mut dyn Page)
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
            AppMode::Rest => {
                if let Some(ref mut rest_page) = self.rest_page {
                    Some(rest_page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::AfkFarm => {
                if let Some(ref mut afk_farm_page) = self.afk_farm_page {
                    Some(afk_farm_page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::QuestList => Some(&mut self.quest_list_page as &mut dyn Page),
            AppMode::PokemonInfo => None, // Pokemon info has no page, handled directly in render system
            AppMode::HeroInfo => {
                if let Some(ref mut page) = self.hero_info_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::ExpeditionSetup => {
                if let Some(ref mut page) = self.expedition_setup_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::ExpeditionInProgress => {
                if let Some(ref mut page) = self.expedition_in_progress_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::ExpeditionSummary => {
                if let Some(ref mut page) = self.expedition_summary_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::CardCollection => {
                if let Some(ref mut page) = self.cards_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::SkillSelection => {
                if let Some(ref mut page) = self.skill_selection_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::SemiActiveBattle => {
                if let Some(ref mut page) = self.semi_active_battle_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::HuntMonsterList => {
                if let Some(ref mut page) = self.hunt_monster_list_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
            AppMode::HuntBattleResult => {
                if let Some(ref mut page) = self.hunt_battle_result_page {
                    Some(page as &mut dyn Page)
                } else {
                    None
                }
            }
        }
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
            self.hero.clone(),
            self.quest_manager.clone(),
            self.mvp_spawn_manager.clone(),
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

    /// Sync kill tracker and hero from battle page back to GameManager
    /// This ensures battle progress is saved
    pub fn sync_battle_state(&mut self) {
        // Extract data from battle page first, then process
        let battle_data = if let Some(ref mut battle_page) = self.battle_page {
            let new_kill_tracker = battle_page.get_kill_tracker().clone();
            let new_hero = battle_page.get_hero().clone();
            Some((new_kill_tracker, new_hero))
        } else {
            None
        };

        if let Some((new_kill_tracker, new_hero)) = battle_data {
            // Get kill diff before syncing (for quest events)
            let old_kills = self.kill_tracker.clone();
            let old_level = self.hero.level;

            self.kill_tracker = new_kill_tracker;

            // Calculate kills made in this sync
            let new_kills = self.calculate_kill_diff(&old_kills, &self.kill_tracker);

            // Sync hero (EXP, levels, HP changes)
            self.hero = new_hero;

            // Process quest events for kills
            for (monster_id, count) in new_kills {
                for _ in 0..count {
                    let event = crate::game::QuestEvent::MonsterKilled { monster_id };
                    self.quest_manager
                        .process_event(&event, self.game_data.get_all_quests());
                }
            }

            // Process quest event for level ups
            if self.hero.level > old_level {
                let event = crate::game::QuestEvent::LevelReached {
                    level: self.hero.level,
                };
                self.quest_manager
                    .process_event(&event, self.game_data.get_all_quests());
            }

            // Log hero sync for debugging
            log::debug!(
                "Synced Hero: {} Lv{}, {}/{} HP",
                self.hero.name,
                self.hero.level,
                self.hero.current_health,
                self.hero.max_health
            );
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

    /// Get player level (hero level)
    fn get_player_level(&self) -> u32 {
        self.hero.level
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
    /// Battle mode (1v1)
    Battle,
    /// Battle result screen (after victory)
    BattleResult,
    /// Death screen (hero died)
    Death,
    /// Rest screen (hero HP regeneration)
    Rest,
    /// AFK Farm mode (passive EXP farming)
    AfkFarm,
    /// Quest list screen
    QuestList,
    /// Pokemon API info screen
    PokemonInfo,
    /// Hero info screen
    HeroInfo,
    /// Expedition setup (select size and see risk)
    ExpeditionSetup,
    /// Expedition in progress (animation-only battle)
    ExpeditionInProgress,
    /// Expedition summary (loot reveal)
    ExpeditionSummary,
    /// Card collection screen
    CardCollection,
    /// Skill selection screen (equip cards with skills)
    SkillSelection,
    /// Semi-active battle mode (turn-based with skills)
    SemiActiveBattle,
    /// Hunt monster selection screen
    HuntMonsterList,
    /// Hunt battle result screen
    HuntBattleResult,
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
