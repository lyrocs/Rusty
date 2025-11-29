//! Death System
//!
//! Handles hero death detection, death screen, and respawn

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to detect hero death in battle and switch to death screen
pub fn death_detection_system(
    mut app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Check in Battle mode only
    if app_state.current_mode != AppMode::Battle {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check for death in battle
    if let Some(ref battle_page) = game_manager.battle_page {
        if battle_page.hero_died() {
            log::info!("💀 Hero fainted! Switching to death screen...");

            // Sync battle state before switching
            game_manager.sync_battle_state();

            // Only create death page if it doesn't already exist (to preserve timer)
            if game_manager.death_page.is_none() {
                // Create death page
                match crate::ui::pages::DeathPage::new() {
                    Ok(death_page) => {
                        log::info!("Created new death page with 2-minute timer");
                        game_manager.death_page = Some(death_page);
                    }
                    Err(e) => {
                        log::error!("Failed to create death page: {:?}", e);
                        // Fallback: just reset hero HP and continue
                        game_manager.hero.heal(game_manager.hero.max_health / 2);
                    }
                }
            } else {
                log::info!("Death page already exists, keeping existing timer");
            }

            // Switch to death mode
            app_state.current_mode = AppMode::Death;
            app_state.needs_redraw = true;
        }
    }
}

/// System to handle death screen interactions
pub fn death_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Death mode
    if app_state.current_mode != AppMode::Death {
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

                // Check if we can respawn
                if let Some(ref death_page) = game_manager.death_page {
                    if death_page.handle_touch(x, y) {
                        log::info!("✨ Respawning hero!");

                        // Restore hero HP to full
                        game_manager.hero.current_health = game_manager.hero.max_health;
                        log::info!("Hero HP restored to full ({}/{})",
                            game_manager.hero.current_health, game_manager.hero.max_health);

                        // Clear battle and death pages
                        game_manager.battle_page = None;
                        game_manager.death_page = None;

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
