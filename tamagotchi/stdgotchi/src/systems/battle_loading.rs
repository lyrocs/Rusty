//! Battle loading system
//!
//! Creates battle page after showing loading screen to avoid blocking UI.

use bevy_ecs::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;

use crate::assets::AssetLoader;
use crate::ecs::resources::{AppMode, AppState, GameManager, SdCardWrapper};
use crate::game::EnemyType;
use crate::ui::pages::battle::EnemyType as BattleEnemyType;
use crate::ui::pages::BattlePage;

/// System to create battle page after loading screen is shown
pub fn battle_loading_system(
    mut app_state: ResMut<AppState>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    sd_card_res: Option<NonSendMut<SdCardWrapper>>,
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

    log::info!("Creating battle page for field: {}", loading_data.field_id);

    // Convert initial enemy type
    let battle_enemy_type = match loading_data.initial_enemy {
        EnemyType::Hornet => BattleEnemyType::Hornet,
        EnemyType::Poring => BattleEnemyType::Poring,
        EnemyType::Fabre => BattleEnemyType::Fabre,
        EnemyType::Lunatic => {
            log::warn!("Lunatic not implemented in battle, using Poring");
            BattleEnemyType::Poring
        }
    };

    // Create asset loader if SD card is available
    let asset_loader = if let Some(sd_card) = sd_card_res.as_ref() {
        log::info!("📁 SD card available - will try loading sprites from SD");
        Some(AssetLoader::new(Some((**sd_card).clone()), true))
    } else {
        log::info!("📦 No SD card - using embedded sprites");
        None
    };

    // Create battle page with background
    let battle_background = include_bytes!("../../assets/images/ui/battle.gif");
    let mut battle_page = match BattlePage::new_with_background(
        battle_background,
        (0, 0),
        game_manager.hero.clone(),
        game_manager.kill_tracker.clone(),
        asset_loader.clone(),
    ) {
        Ok(page) => page,
        Err(e) => {
            log::error!("Failed to load battle background: {:?}", e);
            log::info!("Falling back to solid color background");
            BattlePage::new(
                Rgb888::new(20, 60, 20),
                game_manager.hero.clone(),
                game_manager.kill_tracker.clone(),
                asset_loader,
            )
        }
    };

    // Add hero (using novice animations)
    let hero_idle = include_bytes!("../../assets/images/novice/32.gif");
    let hero_attack = include_bytes!("../../assets/images/novice/80.gif");
    let hero_attacked = include_bytes!("../../assets/images/novice/48.gif");
    battle_page
        .add_hero(hero_idle, hero_attack, hero_attacked, (175, 170))
        .ok();

    // Add enemy
    battle_page
        .add_enemy(battle_enemy_type, (75, 170))
        .ok();

    // Add all monsters from this field to the respawn pool
    for monster_type in &loading_data.monster_types {
        let battle_monster_type = match monster_type {
            EnemyType::Hornet => BattleEnemyType::Hornet,
            EnemyType::Poring => BattleEnemyType::Poring,
            EnemyType::Fabre => BattleEnemyType::Fabre,
            EnemyType::Lunatic => {
                log::warn!("Lunatic not implemented in battle pool");
                BattleEnemyType::Poring
            }
        };
        battle_page.add_enemy_type_to_pool(battle_monster_type);
    }

    game_manager.battle_page = Some(battle_page);

    log::info!("Battle page created successfully, switching to Battle mode");

    // Switch to battle mode
    app_state.current_mode = AppMode::Battle;
    app_state.needs_redraw = true;
}
