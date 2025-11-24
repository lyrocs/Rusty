//! 3v3 Battle loading system
//!
//! Creates 3v3 battle page after showing loading screen to avoid blocking UI.

use bevy_ecs::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;

use crate::ecs::resources::{AppMode, AppState, GameManager};
use crate::ui::pages::Battle3v3Page;

/// System to create 3v3 battle page after loading screen is shown
pub fn battle_3v3_loading_system(
    mut app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Battle3v3Loading mode
    if app_state.current_mode != AppMode::Battle3v3Loading {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check if we have battle loading data
    let Some(loading_data) = game_manager.battle_loading_data.take() else {
        log::error!("No battle loading data available for 3v3!");
        app_state.current_mode = AppMode::Map;
        app_state.needs_redraw = true;
        return;
    };

    log::info!("Creating 3v3 battle page for map: {}", loading_data.map_id);

    // Get game data
    let game_data = game_manager.map_page.world_map().game_data().clone();

    // Create 3v3 battle page
    let mut battle_page = Battle3v3Page::new(
        Rgb888::new(20, 30, 40), // Dark blue background
        game_manager.kill_tracker.clone(),
        game_data,
        game_manager.rustymon_collection.clone(),
        game_manager.rustymon_team.clone(),
        game_manager.fragment_collection.clone(),
    );

    // Setup heroes (first 3 rustymon from team, or as many as available)
    match battle_page.setup_heroes() {
        Ok(_) => {
            log::info!("✅ Heroes loaded successfully for 3v3 battle");
        }
        Err(e) => {
            log::error!("❌ Failed to setup heroes for 3v3 battle: {:?}", e);
            log::error!("Falling back to map...");
            app_state.current_mode = AppMode::Map;
            app_state.needs_redraw = true;
            return;
        }
    }

    // Add up to 3 enemies (repeat if needed to fill 3 slots)
    let mut enemy_ids_to_load = Vec::new();
    for i in 0..3 {
        let enemy_id = loading_data.enemy_ids[i % loading_data.enemy_ids.len()];
        enemy_ids_to_load.push(enemy_id);
    }

    log::info!("Loading 3 enemies: {:?}", enemy_ids_to_load);
    match battle_page.add_enemies(&enemy_ids_to_load) {
        Ok(_) => {
            log::info!("✅ Enemies loaded successfully for 3v3 battle");
        }
        Err(e) => {
            log::error!("❌ Failed to add enemies to 3v3 battle: {:?}", e);
            log::error!("Falling back to map...");
            app_state.current_mode = AppMode::Map;
            app_state.needs_redraw = true;
            return;
        }
    }

    game_manager.battle_3v3_page = Some(battle_page);

    log::info!("🎮 3v3 Battle page created successfully, switching to Battle3v3 mode");

    // Switch to battle mode
    app_state.current_mode = AppMode::Battle3v3;
    app_state.needs_redraw = true;

    log::info!("✅ Switched to Battle3v3 mode, needs_redraw = true");
}
