//! Pokemon Info display system
//!
//! Handles the Pokemon API info display and navigation.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle Pokemon info screen interactions
pub fn pokemon_info_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
) {
    // Only process in PokemonInfo mode
    if app_state.current_mode != AppMode::PokemonInfo {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    // Process all input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { .. } => {
                // Any touch returns to menu
                log::info!("Returning to menu from Pokemon info");
                app_state.current_mode = AppMode::Menu;
                app_state.needs_redraw = true;
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
