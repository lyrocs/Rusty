//! Utility Navigation System
//!
//! Handles input and navigation for utility pages (Inventory, Collection).

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::game::core::Element;
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{InventoryAction, CollectionAction, MonsterListPage};

/// System to handle inventory and collection page navigation
pub fn utility_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Inventory or Collection mode
    if app_state.current_mode != AppMode::Inventory
        && app_state.current_mode != AppMode::Collection
    {
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

                match app_state.current_mode {
                    AppMode::Inventory => {
                        if let Some(ref inventory_page) = game_manager.inventory_page {
                            let action = inventory_page.handle_touch(x, y);
                            match action {
                                InventoryAction::Back => {
                                    log::info!("Inventory -> Home");
                                    game_manager.inventory_page = None;
                                    app_state.current_mode = AppMode::Home;
                                    app_state.needs_redraw = true;
                                }
                                InventoryAction::None => {
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                    }
                    AppMode::Collection => {
                        if let Some(ref mut collection_page) = game_manager.collection_page {
                            let action = collection_page.handle_touch(x, y);
                            match action {
                                CollectionAction::Back => {
                                    log::info!("Collection -> Home");
                                    game_manager.collection_page = None;
                                    app_state.current_mode = AppMode::Home;
                                    app_state.needs_redraw = true;
                                }
                                CollectionAction::SelectZone(zone_id) => {
                                    log::info!("Collection -> MonsterList (zone: {})", zone_id);
                                    // Create monster list filtered by zone
                                    if let Some(monster_list) = create_zone_monster_list(game_manager, &zone_id) {
                                        game_manager.monster_list_page = Some(monster_list);
                                        app_state.current_mode = AppMode::MonsterList;
                                        app_state.needs_redraw = true;
                                    }
                                }
                                CollectionAction::None => {
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            InputEvent::Swipe { direction } => {
                match app_state.current_mode {
                    AppMode::Inventory => {
                        // Swipe right to go back
                        if *direction == SwipeDirection::Right {
                            log::info!("Swipe right: Inventory -> Home");
                            game_manager.inventory_page = None;
                            app_state.current_mode = AppMode::Home;
                            app_state.needs_redraw = true;
                        }
                    }
                    AppMode::Collection => {
                        match direction {
                            SwipeDirection::Right => {
                                log::info!("Swipe right: Collection -> Home");
                                game_manager.collection_page = None;
                                app_state.current_mode = AppMode::Home;
                                app_state.needs_redraw = true;
                            }
                            SwipeDirection::Up | SwipeDirection::Down => {
                                // Swipe up/down to scroll
                                if let Some(ref mut collection_page) = game_manager.collection_page {
                                    collection_page.handle_swipe(*direction == SwipeDirection::Up);
                                    app_state.needs_redraw = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Create monster list filtered by zone
fn create_zone_monster_list(game_manager: &GameManager, zone_id: &str) -> Option<MonsterListPage> {
    // Get zone data
    let zone = game_manager.tamer_data.zones.get(zone_id)?;

    // Build species list for this zone: (species_id, name, element)
    let zone_species: Vec<(String, String, Element)> = game_manager.tamer_data.species.iter()
        .filter(|(_, sp)| sp.zones.contains(&zone_id.to_string()))
        .map(|(species_id, sp)| (species_id.clone(), sp.name.clone(), sp.element))
        .collect();

    // Get team IDs
    let team_ids = game_manager.team.monster_ids();

    Some(MonsterListPage::from_zone(
        &zone.name,
        &zone_species,
        &game_manager.monsters,
        &team_ids,
    ))
}
