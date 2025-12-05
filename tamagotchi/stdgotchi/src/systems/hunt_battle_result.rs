//! Hunt Battle Result System
//!
//! Handles hunt battle result page interactions

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{HuntResultAction, SemiActiveBattlePage};

/// System to handle hunt battle result page interactions
pub fn hunt_battle_result_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in HuntBattleResult mode
    if app_state.current_mode != AppMode::HuntBattleResult {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Handle input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                if let Some(ref mut result_page) = game_manager.hunt_battle_result_page {
                    result_page.handle_tap(*x as i32, *y as i32);
                    app_state.needs_redraw = true;

                    // Check if action was triggered
                    if let Some(action) = result_page.take_action() {
                        match action {
                            HuntResultAction::Next => {
                                // Start another battle with the same enemy
                                let enemy_id = result_page.get_enemy_id();
                                let hero = result_page.get_hero().clone();

                                log::info!("Next hunt battle requested for enemy {}", enemy_id);

                                use embedded_graphics::pixelcolor::Rgb888;

                                // Update game manager's hero with current state (includes EXP/level)
                                game_manager.hero = hero.clone();

                                // Create new battle page
                                let mut battle_page = SemiActiveBattlePage::new(
                                    Rgb888::new(20, 25, 35), // Dark background
                                    hero,
                                    enemy_id,
                                    game_manager.kill_tracker.clone(),
                                    game_manager.game_data.clone(),
                                );

                                // Initialize the battle (loads enemy sprites, etc.)
                                if let Err(e) = battle_page.initialize() {
                                    log::error!("Failed to initialize next hunt battle: {:?}", e);
                                    // Fall back to map
                                    game_manager.hunt_battle_result_page = None;
                                    game_manager.hunt_monster_list_page = None;
                                    game_manager.hunt_enemy_id = None;
                                    game_manager.hunt_map_id = None;
                                    app_state.current_mode = AppMode::Map;
                                    app_state.needs_redraw = true;
                                } else {
                                    game_manager.semi_active_battle_page = Some(battle_page);
                                    game_manager.hunt_battle_result_page = None;

                                    app_state.current_mode = AppMode::SemiActiveBattle;
                                    app_state.needs_redraw = true;

                                    log::info!("Next hunt battle started against enemy {}", enemy_id);
                                }
                            }
                            HuntResultAction::Stop => {
                                // Save hero state and return to map
                                let hero = result_page.get_hero().clone();
                                game_manager.hero = hero;

                                log::info!("Hunt stopped, returning to map");

                                game_manager.hunt_battle_result_page = None;
                                game_manager.hunt_monster_list_page = None;
                                game_manager.hunt_enemy_id = None;
                                game_manager.hunt_map_id = None;

                                app_state.current_mode = AppMode::Map;
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                if let Some(ref mut result_page) = game_manager.hunt_battle_result_page {
                    match direction {
                        SwipeDirection::Left => {
                            result_page.handle_swipe_left();
                            app_state.needs_redraw = true;

                            // Check if stop was triggered
                            if let Some(action) = result_page.take_action() {
                                if action == HuntResultAction::Stop {
                                    let hero = result_page.get_hero().clone();
                                    game_manager.hero = hero;

                                    log::info!("Hunt stopped via swipe, returning to map");

                                    game_manager.hunt_battle_result_page = None;
                                    game_manager.hunt_monster_list_page = None;
                                    game_manager.hunt_enemy_id = None;
                                    game_manager.hunt_map_id = None;

                                    app_state.current_mode = AppMode::Map;
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
