//! Semi-Active Battle System
//!
//! Handles semi-active battle page interactions (MVP fights)

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};

/// System to handle semi-active battle page interactions
pub fn semi_active_battle_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in SemiActiveBattle mode
    if app_state.current_mode != AppMode::SemiActiveBattle {
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
                if let Some(ref mut battle_page) = game_manager.semi_active_battle_page {
                    battle_page.handle_touch(*x as i32, *y as i32);
                    app_state.needs_redraw = true;
                }
            }
            InputEvent::Swipe { direction } => {
                if let Some(ref mut battle_page) = game_manager.semi_active_battle_page {
                    match direction {
                        SwipeDirection::Right => {
                            // Swipe right to flee
                            battle_page.attempt_flee();
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
