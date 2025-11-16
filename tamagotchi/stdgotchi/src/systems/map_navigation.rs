//! Map navigation system
//!
//! Handles map navigation, location selection, and transitions to battle.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle map navigation
pub fn map_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Map mode
    if app_state.current_mode != AppMode::Map {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;
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
                    }
                }
            }
            _ => {
                // Ignore other events in map mode
            }
        }
    }
}
