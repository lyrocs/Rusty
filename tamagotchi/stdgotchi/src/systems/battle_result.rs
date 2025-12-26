//! Battle Result System (Stub)
//!
//! NOTE: Simplified for Phase 1 migration.
//! Will be replaced with proper result handling in Phase 2.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;
use crate::ui::pages::BattleResultAction;

/// System to handle battle result screen
pub fn battle_result_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in BattleResult mode
    if app_state.current_mode != AppMode::BattleResult {
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

                if let Some(ref result_page) = game_manager.battle_result_page {
                    let action = result_page.handle_touch(x, y);

                    if action == BattleResultAction::Continue {
                        log::info!("Continuing after battle result");

                        // Clear result page
                        game_manager.battle_result_page = None;

                        // Return to home
                        app_state.current_mode = AppMode::Home;
                        app_state.needs_redraw = true;
                    }
                }
            }
            _ => {}
        }
    }
}
