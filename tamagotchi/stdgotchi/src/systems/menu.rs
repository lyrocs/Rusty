//! Menu navigation system
//!
//! Handles menu interactions and navigation to different game modes.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle menu navigation
pub fn menu_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Menu mode
    if app_state.current_mode != AppMode::Menu {
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
                log::info!("Menu touch at ({}, {})", x, y);

                // Handle touch on menu page
                if let Some(action) = game_manager.menu_page.handle_touch(*x as i32, *y as i32) {
                    // Navigate based on selected action
                    use crate::ui::pages::menu::MenuAction;
                    match action {
                        MenuAction::Map => {
                            log::info!("Navigating to Map");
                            app_state.current_mode = AppMode::Map;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Battle => {
                            log::info!("Navigating to Battle");
                            // Only switch to battle if there's an active battle
                            if game_manager.battle_page.is_some() {
                                app_state.current_mode = AppMode::Battle;
                                app_state.needs_redraw = true;
                            } else {
                                log::warn!("No active battle");
                            }
                        }
                        MenuAction::Rest => {
                            log::info!("Navigating to Rest screen");
                            // Get team rustymon (first 3 from active slots)
                            let mut team_rustymon = Vec::new();
                            for slot in game_manager.rustymon_team.active_slots.iter().take(3) {
                                if let Some(rustymon_id) = slot {
                                    if let Some(rustymon) = game_manager.rustymon_collection.iter().find(|r| &r.id == rustymon_id) {
                                        team_rustymon.push(rustymon.clone());
                                    }
                                }
                            }

                            if !team_rustymon.is_empty() {
                                // Create rest page with team rustymon
                                match crate::ui::pages::RestPage::new(team_rustymon) {
                                    Ok(rest_page) => {
                                        game_manager.rest_page = Some(rest_page);
                                        app_state.current_mode = AppMode::Rest;
                                        app_state.needs_redraw = true;
                                        log::info!("✅ Rest page created");
                                    }
                                    Err(e) => {
                                        log::error!("Failed to create rest page: {:?}", e);
                                    }
                                }
                            } else {
                                log::warn!("No rustymon in team to rest");
                            }
                        }
                        MenuAction::Rustymon => {
                            log::info!("Navigating to Rustymon List");
                            app_state.current_mode = AppMode::RustymonList;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Quests => {
                            log::info!("Navigating to Quest List");
                            // Auto-start daily quests when opening quest page
                            game_manager.check_quest_resets();
                            game_manager.auto_start_daily_quests();
                            app_state.current_mode = AppMode::QuestList;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Fragments => {
                            log::info!("Navigating to Fragment Collection");
                            app_state.current_mode = AppMode::FragmentCollection;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            _ => {
                // Ignore other events in menu mode
            }
        }
    }
}
