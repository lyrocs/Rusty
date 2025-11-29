//! Battle loading system
//!
//! Creates battle page after showing loading screen to avoid blocking UI.

use bevy_ecs::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;

use crate::ecs::resources::{AppMode, AppState, GameManager};
use crate::ui::pages::BattlePage;

/// System to create battle page after loading screen is shown
pub fn battle_loading_system(
    mut app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
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

    // Get game data from world map
    let game_data = game_manager.map_page.world_map().game_data().clone();

    // Create battle page with solid color background
    let mut battle_page = BattlePage::new(
        Rgb888::new(20, 60, 20),
        game_manager.hero.clone(),
        game_manager.kill_tracker.clone(),
        game_data,
    );

    // Load hero sprites (use poring sprites as placeholder for now)
    log::info!("Loading hero sprites for: {} (Level {})", game_manager.hero.name, game_manager.hero.level);

    use crate::assets::battle::load_enemy_sprites_embedded;
    // Use poring sprites (ID 1002) as hero placeholder
    if let Some((idle, attack, attacked, death)) = load_enemy_sprites_embedded(1002) {
        battle_page
            .add_hero(
                &idle,
                &attack,
                &attacked,
                Some(&death),
                (100, 220), // Hero position on left side
            )
            .ok();
    } else {
        log::error!("Failed to load hero sprites!");
    }

    // Add all enemies from this map to the respawn pool
    for enemy_id in &loading_data.enemy_ids {
        battle_page.add_enemy_id_to_pool(*enemy_id);
    }

    game_manager.battle_page = Some(battle_page);

    log::info!("Battle page created successfully, switching to Battle mode");

    // Switch to battle mode
    app_state.current_mode = AppMode::Battle;
    app_state.needs_redraw = true;
}
