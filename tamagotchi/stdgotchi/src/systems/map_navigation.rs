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
                if let Some(action) = game_manager.map_page.handle_touch(x, y) {
                    use crate::ui::pages::TouchAction;
                    match action {
                        TouchAction::Travel(location_id) => {
                            // Travel to the selected location
                            log::info!("Traveling to location: {}", location_id);
                            if let Err(e) = game_manager.map_page.travel_to(location_id) {
                                log::error!("Failed to travel: {}", e);
                            } else {
                                app_state.needs_redraw = true;
                            }
                        }
                        TouchAction::ViewMapDetails(_map_id) => {
                            // Map details view - just redraw (state changed in handle_touch)
                            log::info!("Viewing map details for map {}", _map_id);
                            app_state.needs_redraw = true;
                        }
                        TouchAction::ViewMonsterList(_map_id) => {
                            // Monster list view - just redraw (state changed in handle_touch)
                            log::info!("Viewing monster list for map {}", _map_id);
                            app_state.needs_redraw = true;
                        }
                        TouchAction::BackToWorldMap => {
                            // Back to world map grid - just redraw (state changed in handle_touch)
                            log::info!("Returning to world map grid");
                            app_state.needs_redraw = true;
                        }
                        TouchAction::BackToMapDetails => {
                            // Back to map details - just redraw (state changed in handle_touch)
                            log::info!("Returning to map details");
                            app_state.needs_redraw = true;
                        }
                        TouchAction::Fight => {
                            // Enter battle on current map
                            let current_location_id = game_manager.map_page.world_map().current_location_id();
                            let location_data = game_manager
                                .map_page
                                .world_map()
                                .get_location(current_location_id)
                                .cloned();

                            if let Some(location) = location_data {
                                if !location.enemies.is_empty() {
                                    log::info!("Entering battle at: {}", location.name);
                                    game_manager.selected_map_id = Some(current_location_id);

                                    // Pick a random enemy from the map
                                    let enemy_index = rand::random::<usize>() % location.enemies.len();
                                    let initial_enemy_id = location.enemies[enemy_index];

                                    // Store battle loading data for deferred creation
                                    game_manager.battle_loading_data =
                                        Some(crate::ecs::resources::BattleLoadingData {
                                            map_id: current_location_id,
                                            enemy_ids: location.enemies.clone(),
                                            initial_enemy_id,
                                        });

                                    // Switch to loading screen first
                                    app_state.current_mode = AppMode::BattleLoading;
                                    app_state.needs_redraw = true;
                                    log::info!(
                                        "Switched to loading screen, battle will be created on next frame"
                                    );
                                }
                            }
                        }
                        TouchAction::Craft => {
                            // Open crafting menu
                            let current_location_id = game_manager.map_page.world_map().current_location_id();
                            let location_data = game_manager
                                .map_page
                                .world_map()
                                .get_location(current_location_id)
                                .cloned();

                            if let Some(location) = location_data {
                                log::info!("Opening crafting menu at: {}", location.name);

                                // Set the current location for crafting (use city name from map data)
                                let city_name = match current_location_id {
                                    1 => "prontera",
                                    2 => "payon",
                                    3 => "geffen",
                                    _ => "prontera", // Default to prontera
                                };
                                game_manager.crafting_page.set_location(city_name.to_string());

                                // Switch to crafting mode
                                app_state.current_mode = AppMode::Crafting;
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
                if let Some(action) = game_manager.handle_hero_overview_touch(x, y) {
                    use crate::ui::pages::hero_overview::ButtonAction;
                    match action {
                        ButtonAction::AllocateStats => {
                            log::info!("Opening stats allocation page");
                            app_state.current_mode = AppMode::StatsAllocation;
                            app_state.needs_redraw = true;
                        }
                        ButtonAction::Close => {
                            log::info!("Closing hero overview - Opening Menu");
                            app_state.current_mode = AppMode::Menu;
                            app_state.needs_redraw = true;
                        }
                    }
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
