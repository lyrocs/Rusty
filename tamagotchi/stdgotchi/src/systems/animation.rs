//! Animation management system
//!
//! Handles initialization and cleanup of page-based animation.

use bevy_ecs::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;
use log::info;

use crate::ecs::resources::{AppMode, AppState, PageResource};
use crate::ui::pages::BattlePage;
use crate::ui::pages::battle::EnemyType;

/// Embedded GIF animation data

// Map background
const MAP_GIF_DATA: &[u8] = include_bytes!("../../assets/images/ui/battle.gif");

// Hero animations (Novice) - load upfront since hero is always present
const HERO_IDLE: &[u8] = include_bytes!("../../assets/images/novice/32.gif");
const HERO_ATTACK: &[u8] = include_bytes!("../../assets/images/novice/80.gif");
const HERO_ATTACKED: &[u8] = include_bytes!("../../assets/images/novice/48.gif");

/// System to initialize page when entering GifPlaying mode
pub fn animation_init_system(world: &mut World) {
    // Query app state
    let app_state = world.resource::<AppState>();
    let current_mode = app_state.current_mode;

    // Check if page already exists
    let has_page = world.get_non_send_resource::<PageResource>().is_some();

    // Check if we need to initialize page
    if current_mode == AppMode::GifPlaying && !has_page {
        info!("Initializing battle page...");

        // Create battle page with map background (384x384 centered on 368x448 display)
        let mut battle_page = match BattlePage::new_with_background(MAP_GIF_DATA, (0, 0)) {
            Ok(page) => page,
            Err(e) => {
                log::error!("Failed to load map background: {:?}", e);
                log::info!("Falling back to solid color background");
                BattlePage::new(Rgb888::new(20, 60, 20))
            }
        };

        // Calculate positions
        const DISPLAY_WIDTH: i32 = 368;
        const DISPLAY_HEIGHT: i32 = 448;
        const HALF_WIDTH: i32 = DISPLAY_WIDTH / 2;

        // Add hero on the right side
        let hero_x = HALF_WIDTH + HALF_WIDTH / 2;
        let hero_y = DISPLAY_HEIGHT / 2;

        if let Err(e) = battle_page.add_hero(HERO_IDLE, HERO_ATTACK, HERO_ATTACKED, (175, 170)) {
            log::error!("Failed to add hero: {:?}", e);
            return;
        }

        // Add first enemy (Hornet) on the left side
        let enemy_x = HALF_WIDTH / 2;
        let enemy_y = DISPLAY_HEIGHT / 2;

        if let Err(e) = battle_page.add_enemy(EnemyType::Hornet, (75, 170)) {
            log::error!("Failed to add hornet enemy: {:?}", e);
            return;
        }

        // Add more enemy types to the respawn pool (not loaded until needed)
        battle_page.add_enemy_type_to_pool(EnemyType::Poring);
        battle_page.add_enemy_type_to_pool(EnemyType::Fabre);

        log::info!("Battle system initialized with hero and enemy types");

        // Create page resource
        let page_resource = PageResource {
            page: Box::new(battle_page),
        };

        // Insert as NonSend resource
        world.insert_non_send_resource(page_resource);

        info!("Battle page initialized");
    }
}

/// System to cleanup page when exiting GifPlaying mode
pub fn animation_cleanup_system(world: &mut World) {
    // Query app state
    let app_state = world.resource::<AppState>();
    let current_mode = app_state.current_mode;

    // Check if page exists
    let has_page = world.get_non_send_resource::<PageResource>().is_some();

    // If we have page but are no longer in GifPlaying mode, clean up
    if current_mode != AppMode::GifPlaying && has_page {
        info!("Cleaning up battle page...");
        world.remove_non_send_resource::<PageResource>();
    }
}
