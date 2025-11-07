//! Animation management system
//!
//! Handles initialization and cleanup of page-based animation.

use bevy_ecs::prelude::*;
use log::info;
use std::time::Duration;

use crate::ecs::resources::{AppMode, AppState, PageResource};
use crate::ui::pages::BattlePage;

/// Embedded GIF animation data
const MAP_GIF_DATA: &[u8] = include_bytes!("../../assets/images/map/1.gif");
const HORNET_GIF_DATA: &[u8] = include_bytes!("../../assets/images/hornet/38.gif");
const HERO_GIF_DATA: &[u8] = include_bytes!("../../assets/images/novice/80.gif");

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

        // Create battle page with map background
        let mut battle_page = match BattlePage::new(MAP_GIF_DATA, (-8, 0)) {
            Ok(page) => page,
            Err(e) => {
                log::error!("Failed to create battle page: {:?}", e);
                return;
            }
        };

        // Add left-centered hornet monster with 20ms frame delay (50 FPS), infinite loops
        if let Err(e) = battle_page.add_left_centered_monster(
            HORNET_GIF_DATA,
            Duration::from_millis(20),
            None, // Infinite loops
        ) {
            log::error!("Failed to add hornet monster: {:?}", e);
            return;
        }

        // Add right-centered hero with 20ms frame delay (50 FPS), infinite loops
        if let Err(e) = battle_page.add_right_centered_monster(
            HERO_GIF_DATA,
            Duration::from_millis(20),
            None, // Infinite loops
        ) {
            log::error!("Failed to add hero: {:?}", e);
            return;
        }

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
