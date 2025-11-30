//! Card Collection System
//!
//! Handles card collection page interactions

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle card collection page interactions
pub fn cards_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in CardCollection mode
    if app_state.current_mode != AppMode::CardCollection {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Handle touch events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                if let Some(ref mut cards_page) = game_manager.cards_page {
                    // Handle touch - returns true if user wants to return to menu
                    if cards_page.handle_touch(*x as i32, *y as i32) {
                        log::info!("Returning to menu from card collection");
                        app_state.current_mode = AppMode::Menu;
                        app_state.needs_redraw = true;
                    } else {
                        // Touch handled (scrolling), redraw
                        app_state.needs_redraw = true;
                    }
                }
            }
            _ => {}
        }
    }
}
