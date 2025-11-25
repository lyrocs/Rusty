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

                if let Some(ref result_page) = game_manager.battle_result_page {
                    // Check if user tapped "Battle Again" button (skips regen)
                    if result_page.handle_battle_again_touch(x, y) {
                        log::info!("⚔️ User tapped Battle Again - applying EXP and starting new battle");

                        // Get updated rustymon with EXP gains (HP stays as-is, no regen)
                        let updated_rustymon = result_page.get_updated_rustymon();

                        // Update collection with EXP gains only
                        for updated in updated_rustymon {
                            if let Some(rustymon) = game_manager.rustymon_collection.iter_mut().find(|r| r.id == updated.id) {
                                let old_exp = rustymon.exp;
                                let old_level = rustymon.level;
                                // Don't update HP - keep battle damage
                                rustymon.exp = updated.exp;
                                rustymon.level = updated.level;
                                if rustymon.level > old_level {
                                    rustymon.recalculate_stats();
                                }
                                log::info!("📈 Updated {}: EXP {} → {} (+{}), Level {} → {} (HP: {}/{})",
                                    rustymon.name,
                                    old_exp, rustymon.exp, rustymon.exp - old_exp,
                                    old_level, rustymon.level,
                                    rustymon.current_hp, rustymon.max_hp);
                            }
                        }

                        // Clear result page
                        game_manager.battle_result_page = None;

                        // Start a new battle using the same battle loading data
                        if let Some(ref battle_loading_data) = game_manager.battle_loading_data {
                            log::info!("🔄 Starting new battle at map {}", battle_loading_data.map_id);

                            // Count alive rustymon in team
                            let alive_count = game_manager.rustymon_team.active_slots
                                .iter()
                                .filter(|slot| {
                                    if let Some(id) = slot {
                                        game_manager.rustymon_collection.iter()
                                            .find(|r| &r.id == id)
                                            .map(|r| r.current_hp > 0)
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    }
                                })
                                .count();

                            if alive_count == 0 {
                                log::warn!("⚠️ No alive rustymon, cannot start new battle. Returning to map.");
                                app_state.current_mode = AppMode::Map;
                            } else {
                                // Always use 3v3 mode
                                log::info!("🎮 Starting new 3v3 battle! (alive rustymon: {})", alive_count);
                                app_state.current_mode = AppMode::Battle3v3Loading;
                            }
                        } else {
                            log::error!("❌ No battle loading data available, returning to map");
                            app_state.current_mode = AppMode::Map;
                        }

                        app_state.needs_redraw = true;
                        return; // Exit after handling battle again
                    }

                    // Check if user tapped continue button (waits for HP full)
                    if result_page.handle_touch(x, y) {
                        log::info!("✅ User tapped continue button - applying EXP and HP, starting new battle");

                        // Get updated rustymon with EXP gains and HP regen
                        let updated_rustymon = result_page.get_updated_rustymon();

                        // Update collection with the updated rustymon (EXP + HP)
                        for updated in updated_rustymon {
                            if let Some(rustymon) = game_manager.rustymon_collection.iter_mut().find(|r| r.id == updated.id) {
                                let old_exp = rustymon.exp;
                                let old_level = rustymon.level;
                                rustymon.current_hp = updated.current_hp;
                                rustymon.exp = updated.exp;
                                rustymon.level = updated.level;
                                if rustymon.level > old_level {
                                    rustymon.recalculate_stats();
                                }
                                log::info!("📈 Updated {}: HP={}/{}, EXP {} → {} (+{}), Level {} → {}",
                                    rustymon.name,
                                    rustymon.current_hp, rustymon.max_hp,
                                    old_exp, rustymon.exp, rustymon.exp - old_exp,
                                    old_level, rustymon.level);
                            }
                        }

                        // Clear result page
                        game_manager.battle_result_page = None;

                        // Start a new battle using the same battle loading data
                        if let Some(ref battle_loading_data) = game_manager.battle_loading_data {
                            log::info!("🔄 Starting new battle at map {} (after full HP regen)", battle_loading_data.map_id);

                            // Count alive rustymon in team
                            let alive_count = game_manager.rustymon_team.active_slots
                                .iter()
                                .filter(|slot| {
                                    if let Some(id) = slot {
                                        game_manager.rustymon_collection.iter()
                                            .find(|r| &r.id == id)
                                            .map(|r| r.current_hp > 0)
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    }
                                })
                                .count();

                            if alive_count == 0 {
                                log::warn!("⚠️ No alive rustymon, cannot start new battle. Returning to map.");
                                app_state.current_mode = AppMode::Map;
                            } else {
                                // Always use 3v3 mode
                                log::info!("🎮 Starting new 3v3 battle! (alive rustymon: {})", alive_count);
                                app_state.current_mode = AppMode::Battle3v3Loading;
                            }
                        } else {
                            log::error!("❌ No battle loading data available, returning to map");
                            app_state.current_mode = AppMode::Map;
                        }

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
