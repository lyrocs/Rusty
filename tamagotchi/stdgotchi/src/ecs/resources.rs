//! ECS Resources for stdgotchi
//!
//! Non-send resources for hardware components that cannot be shared between threads.

use bevy_ecs::prelude::*;
use crossbeam_channel::Receiver;
use esp_idf_svc::hal::gpio::PinDriver;
use std::time::Instant;

use crate::display::{Ft3x68Driver, Sh8601Driver};
use crate::game::{Hero, KillTracker, WorldMap};
use crate::input_thread::InputEvent;
use crate::ui::page::Page;
use crate::ui::pages::{BattlePage, HeroOverviewPage, MapPage};

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
    pub hero_overview_page: HeroOverviewPage,
    pub hero: Hero,
    pub kill_tracker: KillTracker,
    pub selected_map_id: Option<u32>, // Map selected for battle
    pub battle_loading_data: Option<BattleLoadingData>, // Data for deferred battle creation
    pub play_time_seconds: u64,             // Total play time
    pub session_start: Instant,             // Session start time for tracking play time
}

impl GameManager {
    pub fn new(world_map: WorldMap) -> Self {
        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::new(world_map, None), // Use embedded map backgrounds
            battle_page: None,
            hero_overview_page: HeroOverviewPage::new(),
            hero: Hero::new(),
            kill_tracker: KillTracker::new(),
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: 0,
            session_start: Instant::now(),
        }
    }

    /// Create GameManager from save data
    pub fn from_save_data(save_data: crate::game::SaveData, world_map: WorldMap) -> Self {
        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::from_save(world_map, save_data.current_location_id, None), // Use embedded map backgrounds
            battle_page: None,
            hero_overview_page: HeroOverviewPage::new(),
            hero: save_data.hero,
            kill_tracker: save_data.kill_tracker,
            selected_map_id: None,
            battle_loading_data: None,
            play_time_seconds: save_data.play_time_seconds,
            session_start: Instant::now(),
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
            AppMode::HeroOverview => Some(&mut self.hero_overview_page as &mut dyn Page),
        }
    }

    /// Handle hero overview page touch input
    /// This method borrows both page and hero internally to satisfy the borrow checker
    pub fn handle_hero_overview_touch(&mut self, x: i32, y: i32) -> bool {
        self.hero_overview_page.handle_touch(x, y, &mut self.hero)
    }

    /// Draw hero overview page
    /// This method borrows both page and hero internally to satisfy the borrow checker
    pub fn draw_hero_overview(&mut self, display: &mut crate::display::Sh8601Driver, full_redraw: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.hero_overview_page.draw_with_hero(display, &self.hero, full_redraw)
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
            self.hero.clone(),
            self.kill_tracker.clone(),
            current_location_id,
            self.play_time_seconds,
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

    /// Sync hero and kill tracker from battle page back to GameManager
    /// This ensures battle progress is saved
    pub fn sync_battle_state(&mut self) {
        if let Some(ref battle_page) = self.battle_page {
            self.hero = battle_page.get_hero().clone();
            self.kill_tracker = battle_page.get_kill_tracker().clone();
            log::debug!("Synced battle state: Hero Lv{}, {} EXP", self.hero.level, self.hero.exp);
        }
    }
}

/// Input event channel resource - receives events from the input thread on Core 0
/// This is a regular Resource (not NonSend) because the Receiver is Send
#[derive(Resource)]
pub struct InputEventChannel {
    pub receiver: Receiver<InputEvent>,
}

/// App state resource
#[derive(Resource)]
pub struct AppState {
    pub needs_redraw: bool,
    pub current_mode: AppMode,
    pub fps: f32,
    pub frame_count: u32,
    pub last_fps_update: Instant,
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
    /// Hero overview and stats
    HeroOverview,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            needs_redraw: true,
            current_mode: AppMode::Menu,
            fps: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
        }
    }
}
