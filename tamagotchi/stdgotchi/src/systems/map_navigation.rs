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
                        // Check if selected location is a field (battle zone)
                        // Clone location data to avoid borrow conflicts
                        let location = game_manager.map_page.world_map().get_location(&selected_location_id).cloned();

                        if let Some(location) = location {
                            if location.is_field() {
                                // Field selected - prepare for battle
                                log::info!("Entering battle at: {}", location.name);
                                game_manager.selected_field_id = Some(selected_location_id.clone());

                                // Get monsters from this field
                                if let Some(monsters) = location.monsters() {
                                    if !monsters.is_empty() {
                                        // Pick a random monster from the field
                                        let monster_index = rand::random::<usize>() % monsters.len();
                                        let initial_enemy = monsters[monster_index];

                                        // Store battle loading data for deferred creation
                                        game_manager.battle_loading_data = Some(crate::ecs::resources::BattleLoadingData {
                                            field_id: selected_location_id.clone(),
                                            monster_types: monsters.to_vec(),
                                            initial_enemy,
                                        });

                                        // Switch to loading screen first
                                        // The battle_loading_system will create the actual battle page
                                        app_state.current_mode = AppMode::BattleLoading;
                                        app_state.needs_redraw = true;
                                        log::info!("Switched to loading screen, battle will be created on next frame");
                                    }
                                }
                            } else {
                                // City selected - travel there
                                if let Err(e) = game_manager.map_page.travel_to(&selected_location_id) {
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
