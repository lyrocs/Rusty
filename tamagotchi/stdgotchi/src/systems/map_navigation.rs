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
                            // Enter expedition setup on current map
                            let current_location_id = game_manager.map_page.world_map().current_location_id();
                            let location_data = game_manager
                                .map_page
                                .world_map()
                                .get_location(current_location_id)
                                .cloned();

                            if let Some(location) = location_data {
                                if !location.enemies.is_empty() {
                                    // Check if hero is ready for expedition
                                    use crate::game::HeroState;
                                    match &game_manager.hero.state {
                                        HeroState::Ready => {
                                            // Hero is ready, proceed with expedition setup
                                            log::info!("Setting up expedition at: {}", location.name);
                                            game_manager.selected_map_id = Some(current_location_id);

                                            // Pick a random enemy from the map
                                            let enemy_index = rand::random::<usize>() % location.enemies.len();
                                            let enemy_id = location.enemies[enemy_index];

                                            // Get enemy data
                                            if let Some(enemy_data) = game_manager.game_data.get_enemy(enemy_id) {
                                                // Create Enemy instance for expedition
                                                use crate::game::Enemy;
                                                let enemy = Enemy::from_data(
                                                    enemy_data.id,
                                                    enemy_data.name.clone(),
                                                    enemy_data.level,
                                                    enemy_data.hp,
                                                    enemy_data.attack,
                                                    enemy_data.defense,
                                                    enemy_data.hit,
                                                    enemy_data.flee,
                                                    enemy_data.base_exp,
                                                    enemy_data.get_element(),
                                                );

                                                // Create expedition setup page
                                                match crate::ui::pages::ExpeditionSetupPage::new(
                                                    game_manager.hero.clone(),
                                                    enemy,
                                                ) {
                                                    Ok(setup_page) => {
                                                        game_manager.expedition_setup_page = Some(setup_page);
                                                        app_state.current_mode = AppMode::ExpeditionSetup;
                                                        app_state.needs_redraw = true;
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to create expedition setup: {:?}", e);
                                                    }
                                                }
                                            }
                                        }
                                        HeroState::KO { recovery_time: _ } => {
                                            // Hero is KO, show remaining recovery time
                                            if let Some(remaining) = game_manager.hero.state.remaining_time() {
                                                let minutes = remaining / 60;
                                                let seconds = remaining % 60;
                                                log::warn!("Hero is KO! Recovery in {}:{:02}", minutes, seconds);
                                            } else {
                                                log::warn!("Hero is KO!");
                                            }
                                        }
                                        HeroState::OnExpedition { end_time: _ } => {
                                            // Hero is already on expedition
                                            if let Some(remaining) = game_manager.hero.state.remaining_time() {
                                                log::warn!("Hero is already on an expedition! ({} seconds remaining)", remaining);
                                            } else {
                                                log::warn!("Hero is already on an expedition!");
                                            }
                                        }
                                    }
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
                                    log::info!("Starting AFK farming at: {}", location.name);

                                    // Create AFK farm page with hero
                                    match crate::ui::pages::AfkFarmPage::new(
                                        game_manager.hero.clone(),
                                        &location.enemies,
                                        game_manager.game_data.clone(),
                                    ) {
                                        Ok(afk_page) => {
                                            game_manager.afk_farm_page = Some(afk_page);
                                            app_state.current_mode = AppMode::AfkFarm;
                                            app_state.needs_redraw = true;
                                            log::info!("AFK farming started for {}", game_manager.hero.name);
                                        }
                                        Err(e) => {
                                            log::error!("Failed to create AFK farm page: {:?}", e);
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
