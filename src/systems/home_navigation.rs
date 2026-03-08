//! Home Navigation System
//!
//! Handles input and navigation for the Home screen (Accueil).
//! Updates expedition progress and team display.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents, SdCardWrapper};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::systems::expedition_navigation::create_expedition_map_page;
use crate::ui::pages::{HomeAction, CollectionPage, ZoneCollectionData, SpeciesCollectionData, ExpeditionDetailPage, ExpeditionMonsterData, DungeonListPage, DungeonDisplayData};

/// System to handle home page navigation and updates
pub fn home_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    mut sd_card_res: Option<NonSendMut<SdCardWrapper>>,
) {
    // Only process in Home mode
    if app_state.current_mode != AppMode::Home {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Update home page data
    update_home_page_data(game_manager);

    // Load icons from SD card if available (only when icons need refresh)
    if let Some(ref mut sd_card) = sd_card_res {
        // Load icons if team changed or icons not loaded yet
        if game_manager.home_page.needs_icon_reload() {
            game_manager.home_page.load_icons(sd_card);
        }
    }

    // Process input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                let action = game_manager.home_page.handle_touch(x, y);
                match action {
                    HomeAction::GoToDungeonList => {
                        log::info!("Home -> DungeonList");
                        let dungeon_list_page = create_dungeon_list_page(game_manager);
                        game_manager.dungeon_list_page = Some(dungeon_list_page);
                        app_state.current_mode = AppMode::DungeonList;
                        app_state.needs_redraw = true;
                    }
                    HomeAction::GoToCollection => {
                        log::info!("Home -> Collection");
                        let collection_page = create_collection_page(game_manager);
                        game_manager.collection_page = Some(collection_page);
                        app_state.current_mode = AppMode::Collection;
                        app_state.needs_redraw = true;
                    }
                    HomeAction::ViewExpedition(slot_idx) => {
                        log::info!("View expedition slot {}", slot_idx);
                        // Check if expedition exists
                        if let Some(ref exp) = game_manager.active_expeditions[slot_idx] {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();

                            if exp.is_complete(now) {
                                // Collect expedition results
                                if let Some((_, result)) = crate::systems::expedition_navigation::check_expedition_completion(game_manager) {
                                    // Show result page
                                    game_manager.expedition_result_page = Some(
                                        crate::ui::pages::ExpeditionResultPage::new(result)
                                    );
                                    app_state.current_mode = AppMode::ExpeditionResult;
                                    app_state.needs_redraw = true;
                                }
                            } else {
                                // Show expedition detail page for in-progress expedition
                                let map_name = game_manager.tamer_data.get_tamer_map(&exp.map_id)
                                    .map(|m| m.name.clone())
                                    .unwrap_or_else(|| exp.map_id.clone());

                                // Get monster data for the expedition
                                let monsters: Vec<ExpeditionMonsterData> = exp.monster_ids.iter()
                                    .filter_map(|id| game_manager.monsters.iter().find(|m| m.id == *id))
                                    .map(|m| ExpeditionMonsterData {
                                        name: m.name.clone(),
                                        species_id: m.species_id.clone(),
                                        level: m.level,
                                        element: m.element,
                                    })
                                    .collect();

                                game_manager.expedition_detail_page = Some(
                                    ExpeditionDetailPage::new(slot_idx, exp, map_name, monsters)
                                );
                                app_state.current_mode = AppMode::ExpeditionDetail;
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                    HomeAction::StartExpedition => {
                        log::info!("Home -> Expedition Map");
                        // Create expedition map page
                        let map_page = create_expedition_map_page(game_manager);
                        game_manager.expedition_map_page = Some(map_page);
                        app_state.current_mode = AppMode::ExpeditionMap;
                        app_state.needs_redraw = true;
                    }
                    HomeAction::ViewMonster(index) => {
                        log::info!("View team monster at index {}", index);
                        // Get monster from team
                        let team_ids = game_manager.team.monster_ids();
                        if index < team_ids.len() {
                            let monster_id = &team_ids[index];
                            if let Some(monster_idx) = game_manager.monsters.iter()
                                .position(|m| m.id == *monster_id)
                            {
                                game_manager.selected_monster_index = Some(monster_idx);
                                let monster = &game_manager.monsters[monster_idx];
                                game_manager.monster_detail_page = Some(
                                    crate::ui::pages::MonsterDetailPage::new(monster, true) // true = in team
                                );
                                app_state.current_mode = AppMode::MonsterDetail;
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                    HomeAction::None => {}
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe gestures on home screen
                match direction {
                    SwipeDirection::Left => {
                        // Swipe left -> DungeonList
                        log::info!("Swipe left: Home -> DungeonList");
                        let dungeon_list_page = create_dungeon_list_page(game_manager);
                        game_manager.dungeon_list_page = Some(dungeon_list_page);
                        app_state.current_mode = AppMode::DungeonList;
                        app_state.needs_redraw = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Update home page with current game data
pub fn update_home_page_data(game_manager: &mut GameManager) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Get team monsters
    let team_ids = game_manager.team.monster_ids();
    let team_monsters: Vec<&crate::game::core::Monster> = team_ids.iter()
        .filter_map(|id| game_manager.monsters.iter().find(|m| m.id == *id))
        .collect();

    // Create a closure to get map names
    let tamer_data = &game_manager.tamer_data;
    let get_map_name = |map_id: &str| -> String {
        tamer_data.get_tamer_map(map_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| map_id.to_string())
    };

    game_manager.home_page.update_data(
        game_manager.player.crystals,
        &game_manager.active_expeditions,
        &team_monsters,
        now,
        get_map_name,
    );
}

/// Create collection page with zone/species data
fn create_collection_page(game_manager: &GameManager) -> CollectionPage {
    use std::collections::HashSet;

    // Get captured species from player's monsters
    let captured: HashSet<String> = game_manager.monsters.iter()
        .map(|m| m.species_id.clone())
        .collect();

    // Build zone collection data from tamer_data
    let mut zones: Vec<ZoneCollectionData> = Vec::new();

    // Iterate over zones HashMap
    for (zone_id, zone) in &game_manager.tamer_data.zones {
        // Find all species that belong to this zone
        let species: Vec<SpeciesCollectionData> = game_manager.tamer_data.species.iter()
            .filter(|(_, sp)| sp.zones.contains(zone_id))
            .map(|(species_id, sp)| {
                SpeciesCollectionData {
                    species_id: species_id.clone(),
                    name: sp.name.clone(),
                    element: sp.element,
                    is_captured: false, // Will be set by CollectionPage::new
                }
            })
            .collect();

        // Zone is unlocked based on dungeon progress or if it's the first zone
        let is_unlocked = zone.is_unlocked(&game_manager.dungeon_progress);

        zones.push(ZoneCollectionData {
            zone_id: zone_id.clone(),
            zone_name: zone.name.clone(),
            is_unlocked,
            species,
            level_min: zone.level_range.0,
        });
    }

    // Sort zones by level (lowest level first)
    zones.sort_by_key(|z| z.level_min);

    CollectionPage::new(zones, &captured)
}

/// Create dungeon list page with all available dungeons
pub fn create_dungeon_list_page(game_manager: &GameManager) -> DungeonListPage {
    use crate::game::core::Element;

    let mut dungeons: Vec<DungeonDisplayData> = Vec::new();

    // Iterate over all dungeons
    for (dungeon_id, dungeon) in &game_manager.tamer_data.dungeons {
        // Get level range from enemy pools
        let level_min = dungeon.enemy_pools.iter()
            .flat_map(|pool| pool.species.iter())
            .filter_map(|species_id| game_manager.tamer_data.species.get(species_id))
            .map(|species| species.base_level)
            .min()
            .unwrap_or(1);

        let level_max = dungeon.enemy_pools.iter()
            .flat_map(|pool| pool.species.iter())
            .filter_map(|species_id| game_manager.tamer_data.species.get(species_id))
            .map(|species| species.base_level)
            .max()
            .unwrap_or(99);

        // Get highest floor reached
        let highest_floor = game_manager.dungeon_progress.get(dungeon_id).copied().unwrap_or(0);

        // Check if dungeon is unlocked (zone must be unlocked)
        let is_unlocked = game_manager.tamer_data.zones.get(&dungeon.zone_id)
            .map(|zone| zone.is_unlocked(&game_manager.dungeon_progress))
            .unwrap_or(false);

        dungeons.push(DungeonDisplayData {
            dungeon_id: dungeon_id.clone(),
            name: dungeon.name.clone(),
            elements: dungeon.dominant_elements.clone(),
            level_min,
            level_max,
            highest_floor,
            is_unlocked,
        });
    }

    DungeonListPage::new(dungeons)
}
