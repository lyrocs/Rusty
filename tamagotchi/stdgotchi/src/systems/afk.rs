//! AFK Farm System
//!
//! Handles AFK farming screen interactions and EXP gain processing

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle AFK farming screen
pub fn afk_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in AfkFarm mode
    if app_state.current_mode != AppMode::AfkFarm {
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

                // Check if user tapped stop farming button
                if let Some(ref afk_farm_page) = game_manager.afk_farm_page {
                    if afk_farm_page.handle_touch(x, y) {
                        log::info!("🛑 User stopped AFK farming");

                        // Get statistics before clearing page
                        let (total_exp, elapsed_secs) = afk_farm_page.get_stats();
                        let exp_per_min = if elapsed_secs > 0 {
                            (total_exp as f32 / elapsed_secs as f32 * 60.0) as u32
                        } else {
                            0
                        };

                        log::info!("📊 AFK Farming Results: {} total EXP in {}s ({}/min)",
                            total_exp, elapsed_secs, exp_per_min);

                        // Get updated rustymon with EXP gains
                        let updated_rustymon = afk_farm_page.get_updated_rustymon();

                        // Update collection with the gained EXP and levels
                        for updated in updated_rustymon {
                            if let Some(rustymon) = game_manager.rustymon_collection.iter_mut().find(|r| r.id == updated.id) {
                                let old_level = rustymon.level;
                                let old_exp = rustymon.exp;
                                rustymon.exp = updated.exp;
                                rustymon.level = updated.level;

                                // Recalculate stats if level changed
                                if rustymon.level != old_level {
                                    rustymon.recalculate_stats();
                                }

                                if rustymon.level > old_level {
                                    log::info!("⬆️ {} leveled up: {} → {} (EXP: {} → {})",
                                        rustymon.name,
                                        old_level,
                                        rustymon.level,
                                        old_exp,
                                        rustymon.exp);
                                } else {
                                    log::info!("💰 {} gained EXP: {} → {}",
                                        rustymon.name,
                                        old_exp,
                                        rustymon.exp);
                                }
                            }
                        }

                        // Clear AFK farm page
                        game_manager.afk_farm_page = None;

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
