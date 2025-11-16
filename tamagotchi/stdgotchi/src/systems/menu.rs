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
                        MenuAction::Rustymon => {
                            log::info!("Navigating to Rustymon List");
                            app_state.current_mode = AppMode::RustymonList;
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
