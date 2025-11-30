//! Hero State System
//!
//! Monitors and updates hero state (recovery from KO, expedition completion)

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager};
use crate::game::HeroState;

/// System to update hero state based on timers
pub fn hero_state_system(
    _app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check if hero has recovered from KO
    match &game_manager.hero.state {
        HeroState::KO { recovery_time: _ } => {
            if let Some(remaining) = game_manager.hero.state.remaining_time() {
                if remaining == 0 {
                    // Recovery time is up, set hero to Ready
                    log::info!("Hero has recovered from KO! Setting to Ready state");
                    game_manager.hero.state = HeroState::Ready;

                    // Restore some HP (25% of max)
                    let restored_hp = (game_manager.hero.max_health as f32 * 0.25) as i32;
                    game_manager.hero.current_health = restored_hp.max(1);
                    log::info!("Hero HP restored to {}/{}", game_manager.hero.current_health, game_manager.hero.max_health);
                }
            }
        }
        HeroState::OnExpedition { end_time: _ } => {
            // Expedition completion is handled by expedition_in_progress_system
            // This is just here for completeness
        }
        HeroState::Ready => {
            // Nothing to do
        }
    }
}
