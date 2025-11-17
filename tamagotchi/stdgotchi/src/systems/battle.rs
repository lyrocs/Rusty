//! Battle system
//!
//! Handles input during battle mode, including menu access.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};

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

                // Handle touch on battle page (for team switching)
                if let Some(ref mut battle_page) = game_manager.battle_page {
                    if let Some(action) = battle_page.handle_touch(x, y) {
                        use crate::ui::pages::battle::BattleAction;
                        match action {
                            BattleAction::SwitchRustymon(slot) => {
                                log::info!("Switching to team slot {}", slot);
                                if let Err(e) = battle_page.switch_rustymon(slot) {
                                    log::error!("Failed to switch Rustymon: {:?}", e);
                                }
                                app_state.needs_redraw = true;
                            }
                            BattleAction::UseSkill(skill_id) => {
                                log::info!("Using skill {}", skill_id);
                                if let Err(e) = battle_page.use_skill(skill_id) {
                                    log::error!("Failed to use skill: {:?}", e);
                                }
                                app_state.needs_redraw = true;
                            }
                            BattleAction::ToggleAuto => {
                                battle_page.toggle_auto();
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                // Handle swipe to switch Rustymon
                if let Some(ref mut battle_page) = game_manager.battle_page {
                    match direction {
                        SwipeDirection::Right => {
                            log::info!("Swipe right: switching to next Rustymon");
                            if let Err(e) = battle_page.switch_to_next_rustymon() {
                                log::error!("Failed to switch to next Rustymon: {:?}", e);
                            }
                            app_state.needs_redraw = true;
                        }
                        SwipeDirection::Left => {
                            log::info!("Swipe left: switching to previous Rustymon");
                            if let Err(e) = battle_page.switch_to_prev_rustymon() {
                                log::error!("Failed to switch to previous Rustymon: {:?}", e);
                            }
                            app_state.needs_redraw = true;
                        }
                        _ => {
                            // Up/Down swipes not used in battle
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
