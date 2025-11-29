//! Battle system
//!
//! Handles input during battle mode (auto-toggle button)

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle battle mode input
pub fn battle_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Battle mode
    if app_state.current_mode != AppMode::Battle {
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
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on battle page (for auto-toggle button)
                if let Some(ref mut battle_page) = game_manager.battle_page {
                    if let Some(action) = battle_page.handle_touch(x, y) {
                        use crate::ui::pages::battle::BattleAction;
                        match action {
                            BattleAction::ToggleAuto => {
                                battle_page.toggle_auto();
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
            _ => {
                // Other events are not needed in battle mode
            }
        }
    }
}
