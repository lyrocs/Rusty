//! Hero Info System
//!
//! Handles navigation from hero info page

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};

/// System to handle hero info page navigation
pub fn hero_info_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in HeroInfo mode
    if app_state.current_mode != AppMode::HeroInfo {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(_game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Swipe { direction } => {
                // Swipe right to return to menu
                if *direction == SwipeDirection::Right {
                    log::info!("Swipe right: closing Hero Info, returning to menu");
                    app_state.current_mode = AppMode::Menu;
                    app_state.needs_redraw = true;
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
