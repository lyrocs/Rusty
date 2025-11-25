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

                        // Get updated rustymon with HP regeneration
                        let updated_rustymon = rest_page.get_updated_rustymon();

                        // Update collection with the regenerated HP
                        for updated in updated_rustymon {
                            if let Some(rustymon) = game_manager.rustymon_collection.iter_mut().find(|r| r.id == updated.id) {
                                let old_hp = rustymon.current_hp;
                                rustymon.current_hp = updated.current_hp;
                                log::info!("💚 {} HP restored: {} → {} / {}",
                                    rustymon.name,
                                    old_hp,
                                    rustymon.current_hp,
                                    rustymon.max_hp);
                            }
                        }

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
