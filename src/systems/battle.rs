//! Battle system (Stub)
//!
//! NOTE: Simplified for Phase 1 migration.
//! Will be replaced with new real-time combat in Phase 2.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;
use crate::ui::pages::battle::BattleAction;

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
        if let InputEvent::Touch { x, y } = event {
            let x = *x as i32;
            let y = *y as i32;

            // Handle touch on battle page
            let action = if let Some(ref mut battle_page) = game_manager.battle_page {
                battle_page.handle_touch(x, y)
            } else {
                BattleAction::None
            };

            match action {
                BattleAction::Victory => {
                    log::info!("Victory! Switching to result screen");

                    // Get data for result page
                    let exp_gained = game_manager.battle_page.as_ref()
                        .map(|p| p.get_exp_gained())
                        .unwrap_or(0);

                    // Update kill tracker
                    if let Some(ref battle_page) = game_manager.battle_page {
                        game_manager.kill_tracker = battle_page.get_kill_tracker().clone();
                    }

                    let game_data = game_manager.game_data.clone();

                    // Create result page
                    match crate::ui::pages::BattleResultPage::new(exp_gained, true, game_data) {
                        Ok(result_page) => {
                            game_manager.battle_result_page = Some(result_page);
                            game_manager.battle_page = None;
                            app_state.current_mode = AppMode::BattleResult;
                        }
                        Err(e) => {
                            log::error!("Failed to create result page: {:?}", e);
                            game_manager.battle_page = None;
                            app_state.current_mode = AppMode::Home;
                        }
                    }
                    app_state.needs_redraw = true;
                }
                BattleAction::Defeat => {
                    log::info!("Defeat! death_detection_system will handle this");
                }
                BattleAction::Flee => {
                    log::info!("Fleeing battle!");
                    game_manager.battle_page = None;
                    app_state.current_mode = AppMode::Home;
                    app_state.needs_redraw = true;
                }
                BattleAction::None => {
                    app_state.needs_redraw = true;
                }
            }
        }
    }
}
