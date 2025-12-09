//! Home Navigation System
//!
//! Handles input and navigation for the Home screen (Accueil).
//! Updates expedition progress and team display.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::systems::expedition_navigation::create_expedition_map_page;
use crate::ui::pages::{HomeAction, CollectionPage, ZoneCollectionData, SpeciesCollectionData};

/// System to handle home page navigation and updates
pub fn home_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
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

    // Process input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                let action = game_manager.home_page.handle_touch(x, y);
                match action {
                    HomeAction::GoToMap => {
                        log::info!("Home -> Map");
                        app_state.current_mode = AppMode::Map;
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
                        // Check if expedition is complete
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
                                // Just show updated progress
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
                        // Swipe left -> Map
                        log::info!("Swipe left: Home -> Map");
                        app_state.current_mode = AppMode::Map;
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
