//! Expedition Navigation System
//!
//! Handles input and navigation for expedition screens.

use bevy_ecs::prelude::*;
use uuid::Uuid;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::game::core::Element;
use crate::systems::home_navigation::update_home_page_data;
use crate::ui::page::Page;
use crate::game::systems::expedition::{Expedition, ExpeditionDuration, ExpeditionRewards, get_base_rewards, roll_capture, select_capture_species};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{
    ExpeditionMapPage, ExpeditionMapAction, ZoneDisplayData, MapDisplayData,
    ExpeditionTeamSelectPage, ExpeditionTeamAction, MonsterSelectData,
    ExpeditionResultPage, ExpeditionResultAction, ExpeditionResultData,
};

/// System to handle expedition navigation
pub fn expedition_navigation_system(
    mut app_state: ResMut<AppState>,
    mut pending_events: ResMut<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in expedition modes
    if !matches!(
        app_state.current_mode,
        AppMode::ExpeditionMap | AppMode::ExpeditionTeamSelect | AppMode::ExpeditionResult
    ) {
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
                    AppMode::ExpeditionMap => {
                        if let Some(ref mut map_page) = game_manager.expedition_map_page {
                            let action = map_page.handle_touch(x, y);
                            match action {
                                ExpeditionMapAction::SelectMap(map_id) => {
                                    log::info!("Selected map: {}", map_id);
                                    game_manager.selected_expedition_map_id = Some(map_id.clone());

                                    // Create team selection page
                                    if let Some(tamer_map) = game_manager.tamer_data.get_tamer_map(&map_id) {
                                        let monster_data: Vec<MonsterSelectData> = game_manager.monsters.iter()
                                            .map(|m| MonsterSelectData {
                                                id: m.id.clone(),
                                                name: m.name.clone(),
                                                level: m.level,
                                                element: m.element,
                                                is_available: m.status == crate::game::core::MonsterStatus::Available,
                                                is_selected: false,
                                            })
                                            .collect();

                                        let team_page = ExpeditionTeamSelectPage::new(
                                            tamer_map.id.clone(),
                                            tamer_map.name.clone(),
                                            tamer_map.required_elements.clone(),
                                            monster_data,
                                        );
                                        game_manager.expedition_team_page = Some(team_page);
                                        app_state.current_mode = AppMode::ExpeditionTeamSelect;
                                        app_state.needs_redraw = true;
                                    }
                                }
                                ExpeditionMapAction::Back => {
                                    log::info!("Back to home from expedition map");
                                    game_manager.expedition_map_page = None;
                                    app_state.current_mode = AppMode::Home;
                                    app_state.needs_redraw = true;
                                }
                                ExpeditionMapAction::None => {
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                    }
                    AppMode::ExpeditionTeamSelect => {
                        if let Some(ref mut team_page) = game_manager.expedition_team_page {
                            let action = team_page.handle_touch(x, y);
                            match action {
                                ExpeditionTeamAction::ToggleMonster(index) => {
                                    team_page.toggle_monster(index);
                                    app_state.needs_redraw = true;
                                }
                                ExpeditionTeamAction::SelectDuration(_duration) => {
                                    app_state.needs_redraw = true;
                                }
                                ExpeditionTeamAction::StartExpedition => {
                                    if team_page.can_start() {
                                        log::info!("Starting expedition!");
                                        let map_id = team_page.map_id().to_string();
                                        let monster_ids = team_page.selected_monster_ids();
                                        let duration = team_page.selected_duration();

                                        // Find free expedition slot
                                        let slot = game_manager.active_expeditions.iter()
                                            .position(|e| e.is_none());

                                        if let Some(slot_idx) = slot {
                                            // Get current timestamp
                                            let now = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs();

                                            // Create expedition
                                            let expedition = Expedition::new(
                                                Uuid::new_v4().to_string(),
                                                map_id,
                                                monster_ids.clone(),
                                                duration,
                                                now,
                                            );

                                            // Mark monsters as in expedition
                                            for monster_id in &monster_ids {
                                                if let Some(monster) = game_manager.monsters.iter_mut()
                                                    .find(|m| m.id == *monster_id)
                                                {
                                                    monster.status = crate::game::core::MonsterStatus::InExpedition;
                                                }
                                            }

                                            game_manager.active_expeditions[slot_idx] = Some(expedition);
                                            log::info!("Expedition started in slot {}", slot_idx);
                                        } else {
                                            log::warn!("No free expedition slot!");
                                        }

                                        // Return to home
                                        game_manager.expedition_team_page = None;
                                        game_manager.expedition_map_page = None;
                                        game_manager.selected_expedition_map_id = None;
                                        app_state.current_mode = AppMode::Home;
                                        app_state.needs_redraw = true;
                                    }
                                }
                                ExpeditionTeamAction::Back => {
                                    log::info!("Back to map selection");
                                    game_manager.expedition_team_page = None;
                                    game_manager.selected_expedition_map_id = None;
                                    app_state.current_mode = AppMode::ExpeditionMap;
                                    app_state.needs_redraw = true;
                                }
                                ExpeditionTeamAction::None => {
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                    }
                    AppMode::ExpeditionResult => {
                        if let Some(ref mut result_page) = game_manager.expedition_result_page {
                            let action = result_page.handle_touch(x, y);
                            match action {
                                ExpeditionResultAction::Continue => {
                                    log::info!("Expedition result dismissed");
                                    game_manager.expedition_result_page = None;
                                    update_home_page_data(game_manager);
                                    game_manager.home_page.mark_dirty();
                                    app_state.current_mode = AppMode::Home;
                                    app_state.needs_redraw = true;
                                    // Clear events to prevent home_navigation from processing same touch
                                    pending_events.events.clear();
                                    return;
                                }
                                ExpeditionResultAction::Rerun => {
                                    // Get rerun data before consuming the page
                                    let (map_id, monster_ids, duration_secs) = result_page.rerun_data();
                                    let map_id = map_id.to_string();
                                    let monster_ids: Vec<String> = monster_ids.to_vec();

                                    // Find free expedition slot
                                    let free_slot = game_manager.active_expeditions.iter()
                                        .position(|e| e.is_none());

                                    if let Some(slot_idx) = free_slot {
                                        // Check all monsters are available
                                        let all_available = monster_ids.iter().all(|id| {
                                            game_manager.monsters.iter()
                                                .find(|m| m.id == *id)
                                                .map(|m| m.status == crate::game::core::MonsterStatus::Available)
                                                .unwrap_or(false)
                                        });

                                        if all_available {
                                            // Create duration from seconds
                                            let duration = ExpeditionDuration::from_seconds(duration_secs);

                                            // Start expedition
                                            let now = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs();

                                            let expedition = Expedition::new(
                                                Uuid::new_v4().to_string(),
                                                map_id.clone(),
                                                monster_ids.clone(),
                                                duration,
                                                now,
                                            );

                                            game_manager.active_expeditions[slot_idx] = Some(expedition);

                                            // Mark monsters as in expedition
                                            for monster_id in &monster_ids {
                                                if let Some(monster) = game_manager.monsters.iter_mut()
                                                    .find(|m| m.id == *monster_id)
                                                {
                                                    monster.status = crate::game::core::MonsterStatus::InExpedition;
                                                }
                                            }

                                            log::info!("Rerun expedition started: {} with {} monsters",
                                                map_id, monster_ids.len());
                                        } else {
                                            log::warn!("Cannot rerun: some monsters not available");
                                        }
                                    } else {
                                        log::warn!("Cannot rerun: no free expedition slot");
                                    }

                                    // Go back to home
                                    game_manager.expedition_result_page = None;
                                    update_home_page_data(game_manager);
                                    game_manager.home_page.mark_dirty();
                                    app_state.current_mode = AppMode::Home;
                                    app_state.needs_redraw = true;
                                    // Clear events to prevent home_navigation from processing same touch
                                    pending_events.events.clear();
                                    return;
                                }
                                ExpeditionResultAction::None => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            InputEvent::Swipe { direction } => {
                // Swipe right to go back
                if *direction == SwipeDirection::Right {
                    match app_state.current_mode {
                        AppMode::ExpeditionMap => {
                            log::info!("Swipe right: back to home");
                            game_manager.expedition_map_page = None;
                            app_state.current_mode = AppMode::Home;
                            app_state.needs_redraw = true;
                        }
                        AppMode::ExpeditionTeamSelect => {
                            log::info!("Swipe right: back to map selection");
                            game_manager.expedition_team_page = None;
                            game_manager.selected_expedition_map_id = None;
                            app_state.current_mode = AppMode::ExpeditionMap;
                            app_state.needs_redraw = true;
                        }
                        AppMode::ExpeditionResult => {
                            log::info!("Swipe right: dismiss result");
                            game_manager.expedition_result_page = None;
                            update_home_page_data(game_manager);
                            game_manager.home_page.mark_dirty();
                            app_state.current_mode = AppMode::Home;
                            app_state.needs_redraw = true;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

/// Helper function to create expedition map page from game data
pub fn create_expedition_map_page(game_manager: &GameManager) -> ExpeditionMapPage {
    let mut zones: Vec<ZoneDisplayData> = game_manager.tamer_data.all_zones()
        .map(|z| ZoneDisplayData {
            id: z.id.clone(),
            name: z.name.clone(),
            level_range: z.level_range,
            is_unlocked: z.is_unlocked(&game_manager.dungeon_progress),
        })
        .collect();

    // Sort zones by level range so Prontera (lowest level) comes first
    zones.sort_by_key(|z| z.level_range.0);

    let mut maps: Vec<MapDisplayData> = game_manager.tamer_data.all_tamer_maps()
        .map(|m| MapDisplayData {
            id: m.id.clone(),
            name: m.name.clone(),
            zone_id: m.zone_id.clone(),
            level_range: m.level_range,
            required_elements: m.required_elements.clone(),
            capturable_count: m.capturable_species.len(),
        })
        .collect();

    // Sort maps by level range
    maps.sort_by_key(|m| m.level_range.0);

    log::info!("Created ExpeditionMapPage with {} zones and {} maps", zones.len(), maps.len());

    ExpeditionMapPage::new(zones, maps)
}

/// Check and complete expeditions
pub fn check_expedition_completion(game_manager: &mut GameManager) -> Option<(usize, ExpeditionResultData)> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // First pass: find completed expedition and extract data (immutable borrow)
    let completed_data: Option<(usize, String, Vec<String>, ExpeditionDuration)> = {
        game_manager.active_expeditions.iter().enumerate()
            .find_map(|(slot_idx, expedition_opt)| {
                if let Some(ref expedition) = expedition_opt {
                    if expedition.is_complete(now) && !expedition.completed {
                        return Some((
                            slot_idx,
                            expedition.map_id.clone(),
                            expedition.monster_ids.clone(),
                            expedition.duration,
                        ));
                    }
                }
                None
            })
    };

    // If no completed expedition found, return None
    let (slot_idx, map_id, monster_ids, duration) = completed_data?;

    // Calculate rewards
    let base_rewards = get_base_rewards(duration);

    // Get map essences
    let map_essences: Vec<(Element, u8)> = game_manager.tamer_data
        .get_tamer_map(&map_id)
        .map(|m| m.base_rewards.essences.iter()
            .map(|e| (e.element, e.amount))
            .collect())
        .unwrap_or_default();

    // Get map name for result
    let map_name = game_manager.tamer_data.get_tamer_map(&map_id)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| map_id.clone());

    // Get capturable species for this map
    let capturable_species: Vec<String> = game_manager.tamer_data
        .get_tamer_map(&map_id)
        .map(|m| m.capturable_species.clone())
        .unwrap_or_default();

    // Get monster names for display
    let monster_names: Vec<String> = monster_ids.iter()
        .filter_map(|id| game_manager.monsters.iter().find(|m| m.id == *id))
        .map(|m| m.name.clone())
        .collect();

    // Apply XP to monsters and mark them available
    for monster_id in &monster_ids {
        if let Some(monster) = game_manager.monsters.iter_mut()
            .find(|m| m.id == *monster_id)
        {
            monster.xp += base_rewards.xp_per_monster;
            monster.status = crate::game::core::MonsterStatus::Available;

            // Check level up
            while monster.xp >= monster.xp_to_next && monster.level < 99 {
                monster.xp -= monster.xp_to_next;
                monster.level += 1;
                monster.xp_to_next = crate::game::calculations::xp::xp_for_next_level(monster.level);
                log::info!("{} leveled up to {}!", monster.name, monster.level);
            }
        }
    }

    // Add crystals
    game_manager.player.add_crystals(base_rewards.crystals as u32);

    // Add essences
    for (element, amount) in &map_essences {
        game_manager.player.add_essence(*element, *amount as u16);
    }

    // Roll for capture
    let captured_species_result: Option<String>;
    let was_fusion: bool;

    if roll_capture(base_rewards.capture_chance) {
        if let Some(species_id) = select_capture_species(&capturable_species) {
            // Check if already owned (fusion) or new capture
            let already_owned = game_manager.monsters.iter()
                .any(|m| m.species_id == *species_id);

            if already_owned {
                // Fusion: increase fusion count of existing monster
                if let Some(existing) = game_manager.monsters.iter_mut()
                    .find(|m| m.species_id == *species_id)
                {
                    if existing.fusion_count < 9 {
                        existing.fusion_count += 1;
                        log::info!("Fused {}! Now +{}", existing.name, existing.fusion_count);
                    }
                }
                captured_species_result = Some(species_id.clone());
                was_fusion = true;
            } else if game_manager.monsters.len() < 6 {
                // New capture
                if let Some(new_monster) = game_manager.tamer_data.create_monster(species_id) {
                    captured_species_result = Some(new_monster.name.clone());
                    game_manager.monsters.push(new_monster);
                    was_fusion = false;
                    log::info!("Captured new monster: {}", species_id);
                } else {
                    captured_species_result = None;
                    was_fusion = false;
                }
            } else {
                log::warn!("Monster inventory full, cannot capture");
                captured_species_result = None;
                was_fusion = false;
            }
        } else {
            captured_species_result = None;
            was_fusion = false;
        }
    } else {
        captured_species_result = None;
        was_fusion = false;
    }

    // Clear the expedition slot
    game_manager.active_expeditions[slot_idx] = None;

    // Create result data
    let result = ExpeditionResultData {
        map_name,
        map_id: map_id.clone(),
        duration_minutes: duration.minutes(),
        duration_seconds: duration.seconds() as u64,
        xp_per_monster: base_rewards.xp_per_monster,
        monster_names,
        monster_ids: monster_ids.clone(),
        crystals: base_rewards.crystals,
        essences: map_essences,
        captured_species: captured_species_result,
        was_fusion,
    };

    Some((slot_idx, result))
}
