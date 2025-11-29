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

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Check if user tapped continue button
                if let Some(ref rest_page) = game_manager.rest_page {
                    if rest_page.handle_touch(x, y) {
                        log::info!("✅ User tapped continue button - applying HP regeneration");

                        // Get updated hero with HP regeneration
                        let updated_hero = rest_page.get_updated_hero();

                        let old_hp = game_manager.hero.current_health;

                        // Update hero
                        game_manager.hero = updated_hero;

                        log::info!("💚 {} HP restored: {} → {} / {}",
                            game_manager.hero.name,
                            old_hp,
                            game_manager.hero.current_health,
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
