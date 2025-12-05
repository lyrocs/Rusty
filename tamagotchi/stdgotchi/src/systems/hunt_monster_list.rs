//! Hunt Monster List System
//!
//! Handles hunt monster list page interactions

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{HuntAction, SemiActiveBattlePage};

/// System to handle hunt monster list page interactions
pub fn hunt_monster_list_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in HuntMonsterList mode
    if app_state.current_mode != AppMode::HuntMonsterList {
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
                if let Some(ref mut hunt_page) = game_manager.hunt_monster_list_page {
                    hunt_page.handle_tap(*x as i32, *y as i32);
                    app_state.needs_redraw = true;

                    // Check if action was triggered
                    if let Some(action) = hunt_page.take_action() {
                        match action {
                            HuntAction::Exit => {
                                game_manager.hunt_monster_list_page = None;
                                log::info!("Hunt page closed, returning to map");
                                app_state.current_mode = AppMode::Map;
                                app_state.needs_redraw = true;
                            }
                            HuntAction::Fight(enemy_id) => {
                                // Start semi-active battle with this enemy
                                log::info!("Starting hunt battle with enemy {}", enemy_id);

                                use embedded_graphics::pixelcolor::Rgb888;

                                // Create battle page
                                let mut battle_page = SemiActiveBattlePage::new(
                                    Rgb888::new(20, 25, 35), // Dark background
                                    game_manager.hero.clone(),
                                    enemy_id,
                                    game_manager.kill_tracker.clone(),
                                    game_manager.game_data.clone(),
                                );

                                // Initialize the battle (loads enemy sprites, etc.)
                                if let Err(e) = battle_page.initialize() {
                                    log::error!("Failed to initialize hunt battle: {:?}", e);
                                } else {
                                    game_manager.semi_active_battle_page = Some(battle_page);
                                    game_manager.hunt_enemy_id = Some(enemy_id);

                                    // Keep hunt page for when we return
                                    // game_manager.hunt_monster_list_page remains set

                                    app_state.current_mode = AppMode::SemiActiveBattle;
                                    app_state.needs_redraw = true;

                                    log::info!("Hunt battle started against enemy {}", enemy_id);
                                }
                            }
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                if let Some(ref mut hunt_page) = game_manager.hunt_monster_list_page {
                    match direction {
                        SwipeDirection::Left => {
                            hunt_page.handle_swipe_left();
                            app_state.needs_redraw = true;

                            // Check if exit was triggered
                            if let Some(action) = hunt_page.take_action() {
                                if action == HuntAction::Exit {
                                    game_manager.hunt_monster_list_page = None;
                                    log::info!("Hunt page closed via swipe, returning to map");
                                    app_state.current_mode = AppMode::Map;
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                        SwipeDirection::Up => {
                            hunt_page.handle_swipe_up();
                            app_state.needs_redraw = true;
                        }
                        SwipeDirection::Down => {
                            hunt_page.handle_swipe_down();
                            app_state.needs_redraw = true;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
