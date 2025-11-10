//! Battle loading system
//!
//! Creates battle page after showing loading screen to avoid blocking UI.

use bevy_ecs::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;

use crate::ecs::resources::{AppMode, AppState, GameManager, SdCardWrapper};
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

    log::info!("Creating battle page for map: {}", loading_data.map_id);

    // Use embedded sprites for instant loading (no SD card delay)
    // Common enemies are embedded in the binary
    log::info!("📦 Using embedded sprites for instant battle loading");
    let asset_loader = None;

    // Get game data from world map
    let game_data = game_manager.map_page.world_map().game_data().clone();

    // Create battle page with background
    let battle_background = include_bytes!("../../assets/images/ui/battle.gif");
    let mut battle_page = match BattlePage::new_with_background(
        battle_background,
        (0, 0),
        game_manager.hero.clone(),
        game_manager.kill_tracker.clone(),
        game_data,
        asset_loader.clone(),
    ) {
        Ok(page) => page,
        Err(e) => {
            log::error!("Failed to load battle background: {:?}", e);
            log::info!("Falling back to solid color background");
            let game_data = game_manager.map_page.world_map().game_data().clone();
            BattlePage::new(
                Rgb888::new(20, 60, 20),
                game_manager.hero.clone(),
                game_manager.kill_tracker.clone(),
                game_data,
                asset_loader,
            )
        }
    };

    // Add hero (using job-specific animations)
    let hero_job = game_manager.hero.job;
    log::info!("Loading hero sprites for job: {}", hero_job.name());

    let (hero_idle, hero_attack, hero_attacked, attack_offset): (&[u8], &[u8], &[u8], (i32, i32)) =
        match hero_job {
            crate::game::Job::Novice => (
                include_bytes!("../../assets/images/novice/32.gif"),
                include_bytes!("../../assets/images/novice/80.gif"),
                include_bytes!("../../assets/images/novice/48.gif"),
                (-40, 10), // Novice attack offset
            ),
            crate::game::Job::Swordsman => (
                include_bytes!("../../assets/images/swordman/32.gif"),
                include_bytes!("../../assets/images/swordman/80.gif"),
                include_bytes!("../../assets/images/swordman/48.gif"),
                (-20, -35), // Swordsman attack offset
            ),
            crate::game::Job::Knight => (
                include_bytes!("../../assets/images/knight/32.gif"),
                include_bytes!("../../assets/images/knight/80.gif"),
                include_bytes!("../../assets/images/knight/48.gif"),
                (10, 20), // Knight attack offset
            ),
        };

    battle_page
        .add_hero(
            hero_idle,
            hero_attack,
            hero_attacked,
            (175, 170),
            attack_offset,
        )
        .ok();

    // Add enemy by ID
    if let Err(e) = battle_page.add_enemy(loading_data.initial_enemy_id, (75, 170)) {
        log::error!(
            "Failed to add enemy {}: {:?}",
            loading_data.initial_enemy_id,
            e
        );
    }

    // Add all monsters from this map to the respawn pool
    for enemy_id in &loading_data.enemy_ids {
        battle_page.add_enemy_id_to_pool(*enemy_id);
    }

    game_manager.battle_page = Some(battle_page);

    log::info!("Battle page created successfully, switching to Battle mode");

    // Switch to battle mode
    app_state.current_mode = AppMode::Battle;
    app_state.needs_redraw = true;
}
