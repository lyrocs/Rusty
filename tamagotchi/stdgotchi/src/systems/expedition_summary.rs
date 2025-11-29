//! Expedition Summary System
//!
//! Handles user interactions on the expedition summary page

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle expedition summary interactions
pub fn expedition_summary_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in ExpeditionSummary mode
    if app_state.current_mode != AppMode::ExpeditionSummary {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on expedition summary page
                if let Some(ref mut summary_page) = game_manager.expedition_summary_page {
                    // Handle touch (loot reveal or continue)
                    let should_continue = summary_page.handle_touch(x, y);

                    if should_continue {
                        log::info!("📋 Expedition summary completed, returning to map");

                        // Get collected cards and update hero
                        let collected_cards = summary_page.get_collected_cards();
                        log::info!("💎 Collected {} cards total", collected_cards.len());

                        // Update hero with final state from summary
                        game_manager.hero = summary_page.get_updated_hero();

                        // Clean up
                        game_manager.expedition_summary_page = None;
                        game_manager.expedition_data = None;

                        // Return to map
                        app_state.current_mode = AppMode::Map;
                        app_state.needs_redraw = true;
                    } else {
                        // Loot was revealed, trigger redraw
                        app_state.needs_redraw = true;
                    }
                }
            }
            _ => {}
        }
    }
}
