//! ECS Resources for stdgotchi
//!
//! Non-send resources for hardware components that cannot be shared between threads.

use bevy_ecs::prelude::*;
use esp_idf_svc::hal::gpio::PinDriver;
use std::time::Instant;

use crate::display::{Ft3x68Driver, Sh8601Driver};
use crate::game::{Hero, KillTracker, WorldMap};
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

/// Page resource - NonSend because contains Page trait objects with non-Send data
pub struct PageResource {
    pub page: Box<dyn Page>,
}

/// SD card resource for save/load
pub struct SdCardResource {
    pub sd_card: crate::sdcard::SdCard,
    pub save_path: String,
}

/// Game manager - Manages pages and game state
pub struct GameManager {
    pub menu_page: crate::ui::pages::MenuPage,
    pub map_page: MapPage,
    pub battle_page: Option<BattlePage>,
    pub hero_overview_page: HeroOverviewPage,
    pub hero: Hero,
    pub kill_tracker: KillTracker,
    pub selected_field_id: Option<String>, // Field selected for battle
    pub play_time_seconds: u64,             // Total play time
    pub session_start: Instant,             // Session start time for tracking play time
}

impl GameManager {
    pub fn new(world_map: WorldMap) -> Self {
        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::new(world_map),
            battle_page: None,
            hero_overview_page: HeroOverviewPage::new(),
            hero: Hero::new(),
            kill_tracker: KillTracker::new(),
            selected_field_id: None,
            play_time_seconds: 0,
            session_start: Instant::now(),
        }
    }

    /// Create GameManager from save data
    pub fn from_save_data(save_data: crate::game::SaveData, world_map: WorldMap) -> Self {
        Self {
            menu_page: crate::ui::pages::MenuPage::new(),
            map_page: MapPage::from_save(world_map, save_data.current_location_id.clone()),
            battle_page: None,
            hero_overview_page: HeroOverviewPage::new(),
            hero: save_data.hero,
            kill_tracker: save_data.kill_tracker,
            selected_field_id: None,
            play_time_seconds: save_data.play_time_seconds,
            session_start: Instant::now(),
        }
    }

    /// Get current page based on mode (for standard Page trait operations)
    pub fn get_current_page(&mut self, mode: AppMode) -> Option<&mut dyn Page> {
        match mode {
            AppMode::Menu => Some(&mut self.menu_page as &mut dyn Page),
            AppMode::Map => Some(&mut self.map_page as &mut dyn Page),
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
    pub fn save_to_sd(&mut self, sd_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Update total play time
        let session_duration = self.session_start.elapsed().as_secs();
        self.play_time_seconds += session_duration;
        self.session_start = Instant::now();

        // Create save data
        let current_location_id = self.map_page.world_map().current_location_id().to_string();
        let save_data = crate::game::SaveData::new(
            self.hero.clone(),
            self.kill_tracker.clone(),
            current_location_id,
            self.play_time_seconds,
        );

        // Save to file
        save_data.save_to_file(sd_path)?;
        log::info!("Game saved to {}", sd_path);
        Ok(())
    }

    /// Auto-save game state (called after important events)
    pub fn auto_save(&mut self, sd_mounted: bool, sd_path: &str) {
        if !sd_mounted {
            return;
        }

        if let Err(e) = self.save_to_sd(sd_path) {
            log::error!("Auto-save failed: {:?}", e);
        }
    }
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
