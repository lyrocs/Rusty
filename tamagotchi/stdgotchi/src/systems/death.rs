//! Death System
//!
//! Handles hero death detection, death screen, and respawn

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, InputEventChannel};
use crate::input_thread::InputEvent;

/// System to detect hero death in battle and switch to death screen
pub fn death_detection_system(
    mut app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only check in Battle mode
    if app_state.current_mode != AppMode::Battle {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check if hero died in battle
    if let Some(ref battle_page) = game_manager.battle_page {
        if battle_page.hero_died() {
            log::info!("💀 Hero died! Switching to death screen...");

            // Sync battle state before switching
            game_manager.sync_battle_state();

            // Create death page
            match crate::ui::pages::DeathPage::new() {
                Ok(death_page) => {
                    game_manager.death_page = Some(death_page);
                    app_state.current_mode = AppMode::Death;
                    app_state.needs_redraw = true;
                }
                Err(e) => {
                    log::error!("Failed to create death page: {:?}", e);
                    // Fallback: just reset HP and continue
                    game_manager.hero.current_hp = game_manager.hero.max_hp / 2;
                }
            }
        }
    }
}

/// System to handle death screen interactions
pub fn death_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Death mode
    if app_state.current_mode != AppMode::Death {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = x as i32;
                let y = y as i32;

                // Check if we can respawn
                if let Some(ref death_page) = game_manager.death_page {
                    if death_page.handle_touch(x, y) {
                        log::info!("✨ Respawning hero!");

                        // Restore hero HP to full
                        game_manager.hero.current_hp = game_manager.hero.max_hp;

                        // Clear battle and death pages
                        game_manager.battle_page = None;
                        game_manager.death_page = None;

                        // Return to map
                        app_state.current_mode = AppMode::Map;
                        app_state.needs_redraw = true;
                    }
                }
            }
            InputEvent::BootPressed => {
                // Boot button does nothing on death screen (must wait for respawn)
                log::info!("Cannot use menu while dead - wait for respawn timer");
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
