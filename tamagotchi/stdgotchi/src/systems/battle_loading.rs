//! Battle loading system (Stub)
//!
//! NOTE: Simplified for Phase 1 migration.
//! Will be replaced with proper battle initialization in Phase 2.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, SdCardWrapper};
use crate::ui::pages::BattlePage;
use crate::game::Enemy;

/// System to create battle page after loading screen is shown
pub fn battle_loading_system(
    mut app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    mut sd_card_res: Option<NonSendMut<SdCardWrapper>>,
) {
    // Only process in BattleLoading mode
    if app_state.current_mode != AppMode::BattleLoading {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check if we have battle loading data
    let Some(loading_data) = game_manager.battle_loading_data.take() else {
        log::error!("No battle loading data available!");
        app_state.current_mode = AppMode::Map;
        app_state.needs_redraw = true;
        return;
    };

    log::info!("Creating battle page for map: {}", loading_data.map_id);

    // Get enemy data
    let enemy_data = match game_manager.game_data.get_enemy(loading_data.initial_enemy_id) {
        Some(data) => data.clone(),
        None => {
            log::error!("Enemy {} not found in game data", loading_data.initial_enemy_id);
            app_state.current_mode = AppMode::Map;
            app_state.needs_redraw = true;
            return;
        }
    };

    // Create Enemy instance
    let enemy = Enemy::from_data(
        enemy_data.id,
        enemy_data.name.clone(),
        enemy_data.level,
        enemy_data.hp,
        enemy_data.attack,
        enemy_data.defense,
        enemy_data.hit,
        enemy_data.flee,
        enemy_data.base_exp,
        enemy_data.get_element(),
    );

    // Create simplified battle page
    let sd_card_mut: Option<&mut SdCardWrapper> = sd_card_res.as_deref_mut();
    match BattlePage::new(enemy, &game_manager.game_data, game_manager.kill_tracker.clone(), sd_card_mut) {
        Ok(battle_page) => {
            game_manager.battle_page = Some(battle_page);
            log::info!("Battle page created successfully");
            app_state.current_mode = AppMode::Battle;
            app_state.needs_redraw = true;
        }
        Err(e) => {
            log::error!("Failed to create battle page: {:?}", e);
            app_state.current_mode = AppMode::Map;
            app_state.needs_redraw = true;
        }
    }
}
