//! Auto-save system
//!
//! Handles automatic saving of game state after important events.

use bevy_ecs::prelude::*;
use std::time::{Duration, Instant};

use crate::ecs::resources::{GameManager, SdCardResource};

/// Auto-save resource - tracks when to trigger auto-save
#[derive(Resource)]
pub struct AutoSaveState {
    pub last_save_time: Instant,
    pub save_interval: Duration,
    pub save_requested: bool,
}

impl Default for AutoSaveState {
    fn default() -> Self {
        Self {
            last_save_time: Instant::now(),
            save_interval: Duration::from_secs(60), // Auto-save every 60 seconds
            save_requested: false,
        }
    }
}

impl AutoSaveState {
    /// Request an immediate auto-save (e.g., after stat allocation or battle victory)
    pub fn request_save(&mut self) {
        self.save_requested = true;
    }
}

/// Auto-save system - saves game state periodically or on request
pub fn autosave_system(
    mut autosave_state: ResMut<AutoSaveState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    sd_card_res: Option<NonSendMut<SdCardResource>>,
) {
    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    let Some(ref sd_card_res) = sd_card_res else {
        return;
    };

    // Check if it's time for periodic save
    let should_periodic_save = autosave_state.last_save_time.elapsed() >= autosave_state.save_interval;

    // Save if requested or periodic interval reached
    if autosave_state.save_requested || should_periodic_save {
        let sd_mounted = sd_card_res.sd_card.is_mounted();
        game_manager.auto_save(sd_mounted, &sd_card_res.save_path);

        // Reset save state
        autosave_state.save_requested = false;
        autosave_state.last_save_time = Instant::now();

        if should_periodic_save {
            log::info!("Periodic auto-save completed");
        }
    }
}
