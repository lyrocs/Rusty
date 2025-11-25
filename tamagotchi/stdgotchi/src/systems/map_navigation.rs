//! Map navigation system
//!
//! Handles map navigation, location selection, and transitions to battle.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};

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

                                    // Count alive rustymon in team
                                    let alive_count = game_manager.rustymon_team.active_slots
                                        .iter()
                                        .filter(|slot| {
                                            if let Some(id) = slot {
                                                game_manager.rustymon_collection.iter()
                                                    .find(|r| &r.id == id)
                                                    .map(|r| r.current_hp > 0)
                                                    .unwrap_or(false)
                                            } else {
                                                false
                                            }
                                        })
                                        .count();

                                    let has_enough_enemies = location.enemies.len() >= 3;

                                    // ALWAYS use 3v3 for testing - uncomment condition below to make it conditional
                                    // if has_enough_enemies && alive_count >= 3 {
                                    if true {
                                        log::info!("🎮 Starting 3v3 battle! (enemies: {}, alive rustymon: {})",
                                            location.enemies.len(), alive_count);
                                        app_state.current_mode = AppMode::Battle3v3Loading;
                                    } else {
                                        log::info!("Starting 1v1 battle (enemies: {}, alive: {}, need 3+ each)",
                                            location.enemies.len(), alive_count);
                                        app_state.current_mode = AppMode::BattleLoading;
                                    }

                                    app_state.needs_redraw = true;
                                    log::info!(
                                        "Switched to loading screen, battle will be created on next frame"
                                    );
                                }
                            }
                        }
                        TouchAction::AfkFarm => {
                            // Enter AFK farming mode on current map
                            let current_location_id = game_manager.map_page.world_map().current_location_id();
                            let location_data = game_manager
                                .map_page
                                .world_map()
                                .get_location(current_location_id)
                                .cloned();

                            if let Some(location) = location_data {
                                if !location.enemies.is_empty() {
                                    log::info!("🌾 Starting AFK farming at: {}", location.name);

                                    // Get team rustymon
                                    let mut team_rustymon = Vec::new();
                                    for slot in &game_manager.rustymon_team.active_slots {
                                        if let Some(id) = slot {
                                            if let Some(rustymon) = game_manager.rustymon_collection.iter().find(|r| &r.id == id) {
                                                team_rustymon.push(rustymon.clone());
                                            }
                                        }
                                    }

                                    if team_rustymon.is_empty() {
                                        log::error!("❌ Cannot start AFK farming: no rustymon in team");
                                        return;
                                    }

                                    // Create AFK farm page
                                    match crate::ui::pages::AfkFarmPage::new(
                                        team_rustymon,
                                        &location.enemies,
                                        game_manager.game_data.clone(),
                                    ) {
                                        Ok(afk_page) => {
                                            game_manager.afk_farm_page = Some(afk_page);
                                            app_state.current_mode = AppMode::AfkFarm;
                                            app_state.needs_redraw = true;
                                            log::info!("✅ AFK farming started successfully");
                                        }
                                        Err(e) => {
                                            log::error!("❌ Failed to create AFK farm page: {:?}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe right to go back to menu
                if *direction == SwipeDirection::Right {
                    log::info!("Swipe right: closing Map, returning to menu");
                    app_state.current_mode = AppMode::Menu;
                    app_state.needs_redraw = true;
                }
            }
            _ => {
                // Ignore other events in map mode
            }
        }
    }
}
