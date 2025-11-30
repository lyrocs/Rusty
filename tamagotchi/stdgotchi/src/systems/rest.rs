//! Rest System
//!
//! Handles rest screen interactions and HP regeneration

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle rest screen
pub fn rest_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Rest mode
    if app_state.current_mode != AppMode::Rest {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process HP regeneration even if screen is off
    if let Some(ref mut rest_page) = game_manager.rest_page {
        rest_page.process_regen();

        // Update hero in game_manager with regenerated HP
        game_manager.hero = rest_page.get_updated_hero();
    }

    // Only process input if screen is on
    if !app_state.screen_on {
        return;
    }

    // Process input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Check if user tapped continue button
                if let Some(ref rest_page) = game_manager.rest_page {
                    if rest_page.handle_touch(x, y) {
                        log::info!("✅ User tapped continue button - rest complete");

                        let final_hp = game_manager.hero.current_health;

                        log::info!("💚 {} rested successfully: {} / {}",
                            game_manager.hero.name,
                            final_hp,
                            game_manager.hero.max_health);

                        // Clear rest page
                        game_manager.rest_page = None;

                        // Return to menu
                        app_state.current_mode = AppMode::Menu;
                        app_state.needs_redraw = true;
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
