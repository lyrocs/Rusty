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

                        // Get updated hero with EXP gains
                        let updated_hero = afk_farm_page.get_updated_hero();

                        let old_level = game_manager.hero.level;
                        let old_exp = game_manager.hero.experience;

                        // Update hero with EXP gains
                        game_manager.hero = updated_hero;

                        // Log results
                        if game_manager.hero.level > old_level {
                            log::info!("⬆️ {} leveled up: {} → {} (EXP: {} → {})",
                                game_manager.hero.name,
                                old_level,
                                game_manager.hero.level,
                                old_exp,
                                game_manager.hero.experience);
                        } else {
                            log::info!("💰 {} gained EXP: {} → {}",
                                game_manager.hero.name,
                                old_exp,
                                game_manager.hero.experience);
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
