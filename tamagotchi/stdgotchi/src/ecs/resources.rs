//! ECS Resources for stdgotchi Monster Tamer
//!
//! Non-send resources for hardware components that cannot be shared between threads.
//!
//! NOTE: This file has been simplified during migration to the new Monster Tamer architecture.
//! Many features are temporarily disabled and will be re-implemented.

use bevy_ecs::prelude::*;
use crossbeam_channel::Receiver;
use esp_idf_svc::hal::gpio::PinDriver;
use std::time::Instant;

use std::collections::HashMap;

use crate::display::{Cst816dDriver, St7789pDriver};
use crate::game::{KillTracker, WorldMap};
use crate::game::core::{Monster, Team, Player};
use crate::game::data::TamerGameData;
use crate::game::systems::expedition::Expedition;
use crate::game::systems::dungeon::DungeonRun;
use crate::input_thread::InputEvent;
use crate::ui::page::Page;
use crate::ui::pages::{BattlePage, MapPage, HomePage, ExpeditionMapPage, ExpeditionTeamSelectPage, ExpeditionResultPage, DungeonCombatPage, BetweenFloorsPage, BonusSelectionPage, DungeonDefeatPage, MonsterUpgradePage, InventoryPage, CollectionPage};

/// Display resource - NonSend because it contains non-thread-safe SPI operations
pub struct DisplayResource<'a> {
    pub display: St7789pDriver<'a>,
}

/// Touch controller resource - NonSend because it contains non-thread-safe I2C operations
pub struct TouchResource {
    pub touch: Cst816dDriver,
    pub last_touch_active: bool,
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
pub struct SharedI2cResource;

impl SharedI2cResource {
    pub fn get(&self) -> Option<&'static mut esp_idf_svc::hal::i2c::I2cDriver<'static>> {
        unsafe { crate::drivers::sd_cs_pin::get_shared_i2c() }
    }
}

/// WiFi resource - NonSend because WiFi operations are not thread-safe
pub struct WifiResource {
    pub wifi: esp_idf_svc::wifi::BlockingWifi<esp_idf_svc::wifi::EspWifi<'static>>,
}

/// Page resource - NonSend because contains Page trait objects with non-Send data
pub struct PageResource {
    pub page: Box<dyn Page>,
}

/// SD card resource for save/load
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

    /// Load a range of bytes from a binary file
    /// Returns the requested bytes or an error
    pub fn load_binary_range(&mut self, filename: &str, offset: usize, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Use the SD card's native range loading (seeks to offset and reads only requested bytes)
        let mut sd_ops = self.sd_ops.borrow_mut();
        sd_ops.load_binary_range(filename, offset as u32, length)
    }
}

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

    fn load_binary_range(&mut self, filename: &str, offset: u32, length: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.sd_ops.borrow_mut().load_binary_range(filename, offset, length)
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
    // UI Pages
    pub home_page: HomePage,
    pub menu_page: crate::ui::pages::MenuPage,
    pub map_page: MapPage,
    pub battle_page: Option<BattlePage>,
    pub battle_result_page: Option<crate::ui::pages::BattleResultPage>,
    pub death_page: Option<crate::ui::pages::DeathPage>,
    pub monster_list_page: Option<crate::ui::pages::MonsterListPage>,
    pub monster_detail_page: Option<crate::ui::pages::MonsterDetailPage>,
    pub monster_upgrade_page: Option<MonsterUpgradePage>,
    pub inventory_page: Option<InventoryPage>,
    pub collection_page: Option<CollectionPage>,

    // Game state
    pub kill_tracker: KillTracker,
    pub game_data: crate::game::GameData,
    pub selected_map_id: Option<u32>,
    pub battle_loading_data: Option<BattleLoadingData>,
    pub play_time_seconds: u64,
    pub session_start: Instant,

    // Monster Tamer data (Phase 2)
    pub tamer_data: TamerGameData,
    pub monsters: Vec<Monster>,
    pub team: Team,
    pub player: Player,
    pub selected_monster_index: Option<usize>,

    // Expedition data (Phase 3)
    pub active_expeditions: [Option<Expedition>; 2],
    pub dungeon_progress: HashMap<String, u16>,

    // Expedition UI pages
    pub expedition_map_page: Option<ExpeditionMapPage>,
    pub expedition_team_page: Option<ExpeditionTeamSelectPage>,
    pub expedition_result_page: Option<ExpeditionResultPage>,
    pub selected_expedition_map_id: Option<String>,

    // Dungeon combat (Phase 4)
    pub dungeon_combat_page: Option<DungeonCombatPage>,
    pub between_floors_page: Option<BetweenFloorsPage>,
    pub bonus_selection_page: Option<BonusSelectionPage>,
    pub dungeon_defeat_page: Option<DungeonDefeatPage>,
    pub active_dungeon_run: Option<DungeonRun>,
    pub selected_dungeon_id: Option<String>,
}

impl GameManager {
    pub fn new(world_map: WorldMap, game_data: crate::game::GameData) -> Self {
        log::info!("Monster Tamer - Starting new game");

        // Load tamer data
        let tamer_data = TamerGameData::load().unwrap_or_default();

        // Create starter monster (Poring at level 5)
        let mut monsters = Vec::new();
        let mut team = Team::new();

        if let Some(starter) = tamer_data.create_monster_at_level("poring", 5) {
            let monster_id = starter.id.clone();
            monsters.push(starter);
            team.add(monster_id);
            log::info!("Created starter monster: Poring Lv.5");
        }

        Self {
            home_page: HomePage::new(),
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::new(world_map, None),
            battle_page: None,
            battle_result_page: None,
            death_page: None,
            monster_list_page: None,
            monster_detail_page: None,
            monster_upgrade_page: None,
            inventory_page: None,
            collection_page: None,
            kill_tracker: KillTracker::new(),
            game_data,
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: 0,
            session_start: Instant::now(),
            tamer_data,
            monsters,
            team,
            player: Player::new(),
            selected_monster_index: None,
            active_expeditions: [None, None],
            dungeon_progress: HashMap::new(),
            expedition_map_page: None,
            expedition_team_page: None,
            expedition_result_page: None,
            selected_expedition_map_id: None,
            dungeon_combat_page: None,
            between_floors_page: None,
            bonus_selection_page: None,
            dungeon_defeat_page: None,
            active_dungeon_run: None,
            selected_dungeon_id: None,
        }
    }

    /// Create GameManager from save data
    pub fn from_save_data(save_data: crate::game::SaveData, world_map: WorldMap, game_data: crate::game::GameData) -> Self {
        let tamer_data = TamerGameData::load().unwrap_or_default();

        // Load monsters from save data, or create starter if none
        let (mut monsters, team, player) = if save_data.monsters.is_empty() {
            log::info!("No saved monsters, creating starter Poring");
            let mut monsters = Vec::new();
            let mut team = Team::new();

            if let Some(starter) = tamer_data.create_monster_at_level("poring", 5) {
                let monster_id = starter.id.clone();
                monsters.push(starter);
                team.add(monster_id);
            }

            (monsters, team, Player::new())
        } else {
            log::info!("Loaded {} monsters from save", save_data.monsters.len());
            (save_data.monsters, save_data.team, save_data.player)
        };

        // Recalculate all monster stats to ensure they use current formula
        // This handles formula changes between versions
        use crate::game::systems::progression::leveling::recalculate_stats_with_base;
        log::info!("Recalculating stats for {} monsters with current formula", monsters.len());
        for monster in &mut monsters {
            if let Some(species) = tamer_data.get_species(&monster.species_id) {
                recalculate_stats_with_base(
                    monster,
                    species.base_hp,
                    species.base_atk,
                    species.base_def,
                    species.base_spd,
                );
                log::info!("  {} Lv{}: HP={}, ATK={}, DEF={}, SPD={}",
                    monster.name, monster.level, monster.hp_max, monster.atk, monster.def, monster.spd);
            }
        }

        Self {
            home_page: HomePage::new(),
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::from_save(world_map, save_data.current_location_id, None),
            battle_page: None,
            battle_result_page: None,
            death_page: None,
            monster_list_page: None,
            monster_detail_page: None,
            monster_upgrade_page: None,
            inventory_page: None,
            collection_page: None,
            kill_tracker: save_data.kill_tracker,
            game_data,
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: save_data.play_time_seconds,
            session_start: Instant::now(),
            tamer_data,
            monsters,
            team,
            player,
            selected_monster_index: None,
            active_expeditions: save_data.active_expeditions,
            dungeon_progress: save_data.dungeon_progress,
            expedition_map_page: None,
            expedition_team_page: None,
            expedition_result_page: None,
            selected_expedition_map_id: None,
            dungeon_combat_page: None,
            between_floors_page: None,
            bonus_selection_page: None,
            dungeon_defeat_page: None,
            active_dungeon_run: None,
            selected_dungeon_id: None,
        }
    }

    /// Get a monster by ID
    pub fn get_monster(&self, monster_id: &str) -> Option<&Monster> {
        self.monsters.iter().find(|m| m.id == monster_id)
    }

    /// Get a mutable monster by ID
    pub fn get_monster_mut(&mut self, monster_id: &str) -> Option<&mut Monster> {
        self.monsters.iter_mut().find(|m| m.id == monster_id)
    }

    /// Add a new monster (from capture or starter)
    pub fn add_monster(&mut self, monster: Monster) -> bool {
        if self.monsters.len() >= 6 {
            log::warn!("Cannot add monster: inventory full (max 6)");
            return false;
        }
        log::info!("Added monster: {} ({})", monster.name, monster.species_id);
        self.monsters.push(monster);
        true
    }

    /// Recalculate all monster stats using current formula
    /// Call this after loading from save to ensure stats match current formula
    pub fn recalculate_all_monster_stats(&mut self) {
        use crate::game::systems::progression::leveling::recalculate_stats_with_base;

        log::info!("Recalculating stats for {} monsters with updated formula", self.monsters.len());

        for monster in &mut self.monsters {
            // Look up species base stats
            if let Some(species) = self.tamer_data.get_species(&monster.species_id) {
                let old_atk = monster.atk;
                let old_hp = monster.hp_max;

                recalculate_stats_with_base(
                    monster,
                    species.base_hp,
                    species.base_atk,
                    species.base_def,
                    species.base_spd,
                );

                log::info!("  {} Lv{} +{}: ATK {} -> {}, HP {} -> {}",
                    monster.name, monster.level, monster.fusion_count,
                    old_atk, monster.atk, old_hp, monster.hp_max);
            } else {
                log::warn!("  Species {} not found, cannot recalculate stats", monster.species_id);
            }
        }
    }

    /// Get current page based on mode
    pub fn get_current_page(&mut self, mode: AppMode) -> Option<&mut dyn Page> {
        match mode {
            AppMode::Home => Some(&mut self.home_page as &mut dyn Page),
            AppMode::Menu => Some(&mut self.menu_page as &mut dyn Page),
            AppMode::Map => Some(&mut self.map_page as &mut dyn Page),
            AppMode::BattleLoading => None,
            AppMode::Battle => {
                self.battle_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::BattleResult => {
                self.battle_result_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::Death => {
                self.death_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::MonsterList => {
                self.monster_list_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::MonsterDetail => {
                self.monster_detail_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::MonsterUpgrade => {
                self.monster_upgrade_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::ExpeditionMap => {
                self.expedition_map_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::ExpeditionTeamSelect => {
                self.expedition_team_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::ExpeditionResult => {
                self.expedition_result_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::Inventory => {
                self.inventory_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::Collection => {
                self.collection_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::DungeonCombat => {
                self.dungeon_combat_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::BetweenFloors => {
                self.between_floors_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::BonusSelection => {
                self.bonus_selection_page.as_mut().map(|p| p as &mut dyn Page)
            }
            AppMode::DungeonDefeat => {
                self.dungeon_defeat_page.as_mut().map(|p| p as &mut dyn Page)
            }
        }
    }

    /// Save game state to SD card
    pub fn save_to_sd(&mut self, sd_card: &mut SdCardWrapper, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let session_duration = self.session_start.elapsed().as_secs();
        self.play_time_seconds += session_duration;
        self.session_start = Instant::now();

        let current_location_id = self.map_page.world_map().current_location_id();

        // Save full game state including monsters, team, player, and expeditions
        let save_data = crate::game::SaveData::new(
            self.kill_tracker.clone(),
            current_location_id,
            self.play_time_seconds,
            self.monsters.clone(),
            self.team.clone(),
            self.player.clone(),
            self.active_expeditions.clone(),
            self.dungeon_progress.clone(),
        );

        let json = save_data.to_json()?;
        sd_card.save_to_file(filename, &json)?;
        log::info!("Game saved to {} ({} monsters, {} expeditions)",
            filename, self.monsters.len(),
            self.active_expeditions.iter().filter(|e| e.is_some()).count());
        Ok(())
    }

    /// Auto-save game state
    pub fn auto_save(&mut self, sd_card: &mut Option<&mut SdCardWrapper>, filename: &str) {
        let Some(sd_card) = sd_card else { return };
        if !sd_card.is_mounted() { return }
        if let Err(e) = self.save_to_sd(sd_card, filename) {
            log::error!("Auto-save failed: {:?}", e);
        }
    }

    /// Sync battle state (simplified - no Rustymon)
    pub fn sync_battle_state(&mut self) {
        if let Some(ref mut battle_page) = self.battle_page {
            self.kill_tracker = battle_page.get_kill_tracker().clone();
        }
    }
}

/// Input event channel resource
#[derive(Resource)]
pub struct InputEventChannel {
    pub receiver: Receiver<InputEvent>,
}

/// Pending input events resource
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
    pub screen_on: bool,
}

/// Application modes for Monster Tamer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Home screen (main dashboard)
    Home,
    /// Menu screen (legacy, may be removed)
    Menu,
    /// Map navigation
    Map,
    /// Loading screen before battle
    BattleLoading,
    /// Battle mode
    Battle,
    /// Battle result screen
    BattleResult,
    /// Death screen
    Death,
    /// Monster list screen
    MonsterList,
    /// Monster detail screen
    MonsterDetail,
    /// Monster upgrade screen
    MonsterUpgrade,
    /// Expedition map selection
    ExpeditionMap,
    /// Expedition team selection
    ExpeditionTeamSelect,
    /// Expedition result screen
    ExpeditionResult,
    /// Inventory screen
    Inventory,
    /// Dungeon combat (real-time)
    DungeonCombat,
    /// Between dungeon floors
    BetweenFloors,
    /// Bonus selection after clearing a floor
    BonusSelection,
    /// Dungeon defeat screen
    DungeonDefeat,
    /// Collection tracker screen
    Collection,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            needs_redraw: true,
            current_mode: AppMode::Home, // Start at Home screen
            fps: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            screen_on: true,
        }
    }
}
