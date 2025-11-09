//! Map navigation system
//!
//! Handles map navigation, location selection, and transitions to battle.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, InputEventChannel};
use crate::input_thread::InputEvent;

/// System to handle map navigation
pub fn map_navigation_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Map mode
    if app_state.current_mode != AppMode::Map {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = x as i32;
                let y = y as i32;
                log::info!("Touch at ({}, {})", x, y);

                // Handle touch on map page
                if let Some(selected_location_id) = game_manager.map_page.handle_touch(x, y) {
                    // Get location data and clone it to avoid borrow issues
                    let location_data = game_manager
                        .map_page
                        .world_map()
                        .get_location(selected_location_id)
                        .cloned();

                    if let Some(location) = location_data {
                        // Check if location has enemies (battle zone)
                        if !location.enemies.is_empty() {
                            // Field with enemies - prepare for battle
                            log::info!("Entering battle at: {}", location.name);
                            game_manager.selected_map_id = Some(selected_location_id);

                            // Pick a random enemy from the map
                            let enemy_index = rand::random::<usize>() % location.enemies.len();
                            let initial_enemy_id = location.enemies[enemy_index];

                            // Store battle loading data for deferred creation
                            game_manager.battle_loading_data =
                                Some(crate::ecs::resources::BattleLoadingData {
                                    map_id: selected_location_id,
                                    enemy_ids: location.enemies.clone(),
                                    initial_enemy_id,
                                });

                            // Switch to loading screen first
                            // The battle_loading_system will create the actual battle page
                            app_state.current_mode = AppMode::BattleLoading;
                            app_state.needs_redraw = true;
                            log::info!(
                                "Switched to loading screen, battle will be created on next frame"
                            );
                        } else {
                            // Safe zone - travel there
                            log::info!("Traveling to safe zone: {}", location.name);
                            if let Err(e) = game_manager.map_page.travel_to(selected_location_id) {
                                log::error!("Failed to travel: {}", e);
                            } else {
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
            InputEvent::BootPressed => {
                // Boot button opens menu
                log::info!("Boot button pressed - Opening Menu");
                app_state.current_mode = AppMode::Menu;
                app_state.needs_redraw = true;
            }
            _ => {
                // Ignore other events in map mode
            }
        }
    }
}

/// System to handle hero overview interactions
pub fn hero_overview_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in HeroOverview mode
    if app_state.current_mode != AppMode::HeroOverview {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = x as i32;
                let y = y as i32;

                // Handle touch on hero overview page
                if game_manager.handle_hero_overview_touch(x, y) {
                    app_state.needs_redraw = true;
                }
            }
            InputEvent::BootPressed => {
                // Boot button returns to menu
                log::info!("Boot button pressed - Opening Menu");
                app_state.current_mode = AppMode::Menu;
                app_state.needs_redraw = true;
            }
            _ => {
                // Ignore other events in hero overview mode
            }
        }
    }
}
