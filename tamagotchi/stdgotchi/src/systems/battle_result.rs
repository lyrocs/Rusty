//! Battle Result System
//!
//! Handles battle result screen interactions and transitions

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

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

                // Check if user tapped continue button
                if let Some(ref result_page) = game_manager.battle_result_page {
                    if result_page.handle_touch(x, y) {
                        log::info!("✅ User tapped continue button - applying EXP rewards");

                        // Get updated rustymon with EXP gains and HP regen
                        let updated_rustymon = result_page.get_updated_rustymon();

                        // Update collection with the updated rustymon
                        for updated in updated_rustymon {
                            if let Some(rustymon) = game_manager.rustymon_collection.iter_mut().find(|r| r.id == updated.id) {
                                let old_exp = rustymon.exp;
                                let old_level = rustymon.level;
                                rustymon.current_hp = updated.current_hp;
                                rustymon.exp = updated.exp;
                                rustymon.level = updated.level;
                                log::info!("📈 Updated {}: HP={}/{}, EXP {} → {} (+{}), Level {} → {}",
                                    rustymon.name,
                                    rustymon.current_hp, rustymon.max_hp,
                                    old_exp, rustymon.exp, rustymon.exp - old_exp,
                                    old_level, rustymon.level);
                            }
                        }

                        // Clear result page
                        game_manager.battle_result_page = None;

                        // Return to map
                        app_state.current_mode = AppMode::Map;
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
