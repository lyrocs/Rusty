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
                    // Check if user tapped continue button (waits for HP full)
                    if result_page.handle_touch(x, y) {
                        log::info!("✅ User tapped continue button - applying EXP and HP");

                        // Get updated hero with EXP gains and HP regen
                        let updated_hero = result_page.get_updated_hero();

                        let old_exp = game_manager.hero.experience;
                        let old_level = game_manager.hero.level;
                        let old_hp = game_manager.hero.current_health;

                        // Update hero
                        game_manager.hero = updated_hero;

                        log::info!("📈 Updated {}: HP={}/{}, EXP {} → {} (+{}), Level {} → {}",
                            game_manager.hero.name,
                            game_manager.hero.current_health, game_manager.hero.max_health,
                            old_exp, game_manager.hero.experience, game_manager.hero.experience - old_exp,
                            old_level, game_manager.hero.level);

                        // Clear result page
                        game_manager.battle_result_page = None;

                        // Start a new battle using the same battle loading data
                        if game_manager.battle_loading_data.is_some() {
                            log::info!("🔄 Starting new battle (after full HP regen)");

                            // Check if hero is alive
                            if game_manager.hero.current_health > 0 {
                                app_state.current_mode = AppMode::BattleLoading;
                            } else {
                                log::warn!("⚠️ Hero is dead, cannot start new battle. Returning to map.");
                                app_state.current_mode = AppMode::Map;
                            }
                        } else {
                            log::info!("✅ Battle victory complete, returning to map");
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
