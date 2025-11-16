//! Rustymon navigation system
//!
//! Handles navigation for Rustymon-related pages (list, detail, fragments, summon).

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;

/// System to handle Rustymon list navigation
pub fn rustymon_list_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in RustymonList mode
    if app_state.current_mode != AppMode::RustymonList {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on rustymon list page
                if let Some(action) = game_manager.rustymon_list_page.handle_touch(x, y) {
                    use crate::ui::pages::rustymon_list::RustymonListAction;
                    match action {
                        RustymonListAction::SelectRustymon(index) => {
                            // Navigate to rustymon detail page
                            log::info!("Viewing Rustymon at index {}", index);

                            // Set the selected rustymon index for detail view
                            if index < game_manager.rustymon_collection.len() {
                                game_manager.selected_rustymon_index = Some(index);
                                app_state.current_mode = AppMode::RustymonDetail;
                                app_state.needs_redraw = true;
                            }
                        }
                        RustymonListAction::ScrollUp => {
                            game_manager.rustymon_list_page.scroll_up();
                            app_state.needs_redraw = true;
                        }
                        RustymonListAction::ScrollDown => {
                            let total = game_manager.rustymon_collection.len();
                            game_manager.rustymon_list_page.scroll_down(total);
                            app_state.needs_redraw = true;
                        }
                        RustymonListAction::Close => {
                            log::info!("Closing Rustymon list - returning to Menu");
                            app_state.current_mode = AppMode::Menu;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// System to handle Rustymon detail navigation
pub fn rustymon_detail_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in RustymonDetail mode
    if app_state.current_mode != AppMode::RustymonDetail {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on rustymon detail page
                if let Some(action) = game_manager.rustymon_detail_page.handle_touch(x, y) {
                    use crate::ui::pages::rustymon_detail::RustymonDetailAction;
                    match action {
                        RustymonDetailAction::AddToTeam => {
                            // Add Rustymon to team
                            if let Some(index) = game_manager.selected_rustymon_index {
                                if index < game_manager.rustymon_collection.len() {
                                    let rustymon_id = game_manager.rustymon_collection[index].id.clone();
                                    if game_manager.rustymon_team.add_rustymon(rustymon_id) {
                                        log::info!("Added Rustymon to team");
                                        app_state.needs_redraw = true;
                                    } else {
                                        log::warn!("Failed to add to team (team may be full)");
                                    }
                                }
                            }
                        }
                        RustymonDetailAction::RemoveFromTeam => {
                            // Remove Rustymon from team
                            if let Some(index) = game_manager.selected_rustymon_index {
                                if index < game_manager.rustymon_collection.len() {
                                    let rustymon_id = game_manager.rustymon_collection[index].id.clone();
                                    if game_manager.rustymon_team.remove_rustymon(&rustymon_id) {
                                        log::info!("Removed Rustymon from team");
                                        app_state.needs_redraw = true;
                                    }
                                }
                            }
                        }
                        RustymonDetailAction::OpenSkills => {
                            // Open skills page
                            log::info!("Opening Rustymon skills page");
                            app_state.current_mode = AppMode::RustymonSkills;
                            app_state.needs_redraw = true;
                        }
                        RustymonDetailAction::Close => {
                            log::info!("Closing Rustymon detail - returning to list");
                            app_state.current_mode = AppMode::RustymonList;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// System to handle Fragment Collection navigation
pub fn fragment_collection_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in FragmentCollection mode
    if app_state.current_mode != AppMode::FragmentCollection {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on fragment collection page
                if let Some(action) = game_manager.fragment_collection_page.handle_touch(x, y) {
                    use crate::ui::pages::FragmentCollectionAction;
                    match action {
                        FragmentCollectionAction::Summon(enemy_id) => {
                            // Check if we can summon this Rustymon
                            log::info!("Selected enemy {} for summon", enemy_id);
                            let fragment_count = game_manager.fragment_collection.get_fragment_count(enemy_id);

                            // Get enemy data to check fragments_required
                            let enemy_data_opt = game_manager.map_page.world_map().game_data().get_enemy(enemy_id).cloned();

                            if let Some(enemy_data) = enemy_data_opt {
                                let required_fragments = enemy_data.fragments_required;

                                if fragment_count >= required_fragments {
                                    // Can summon! Create pending summon and switch to summon preview
                                    // Create the Rustymon from enemy data with skills (using RustymonFactory)
                                    let game_data = &game_manager.map_page.world_map().game_data();
                                    let rustymon = crate::game::RustymonFactory::create_from_enemy_with_skills(
                                        enemy_data.id,
                                        enemy_data.name.clone(),
                                        enemy_data.level,
                                        enemy_data.get_element(),
                                        enemy_data.str,
                                        enemy_data.dex,
                                        enemy_data.vit,
                                        enemy_data.int,
                                        enemy_data.luk,
                                        game_data,
                                    );
                                    game_manager.pending_summon_rustymon = Some(rustymon);

                                    // Switch to summon preview mode
                                    app_state.current_mode = AppMode::RustymonSummon;
                                    app_state.needs_redraw = true;
                                    log::info!("Opening summon preview for {}", enemy_data.name);
                                } else {
                                    log::info!("Not enough fragments: {}/{}", fragment_count, required_fragments);
                                }
                            } else {
                                log::warn!("Enemy data not found for ID {}", enemy_id);
                            }
                        }
                        FragmentCollectionAction::ScrollUp => {
                            game_manager.fragment_collection_page.scroll_up();
                            app_state.needs_redraw = true;
                        }
                        FragmentCollectionAction::ScrollDown => {
                            // Get unique enemy count
                            let unique_count = game_manager.fragment_collection.get_unique_monster_count();
                            game_manager.fragment_collection_page.scroll_down(unique_count);
                            app_state.needs_redraw = true;
                        }
                        FragmentCollectionAction::Close => {
                            log::info!("Closing Fragment Collection - returning to Menu");
                            app_state.current_mode = AppMode::Menu;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// System to handle Rustymon Summon preview navigation
pub fn rustymon_summon_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in RustymonSummon mode
    if app_state.current_mode != AppMode::RustymonSummon {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on rustymon summon page
                if let Some(action) = game_manager.rustymon_summon_page.handle_touch(x, y) {
                    use crate::ui::pages::rustymon_summon::RustymonSummonAction;
                    match action {
                        RustymonSummonAction::Confirm => {
                            // Confirm summon - add to collection and deduct fragments
                            if let Some(rustymon) = game_manager.pending_summon_rustymon.take() {
                                let species_id = rustymon.species_id;

                                // Get enemy data to know how many fragments to deduct
                                let enemy_data_opt = game_manager.map_page.world_map().game_data().get_enemy(species_id).cloned();
                                let fragments_to_deduct = enemy_data_opt
                                    .map(|data| data.fragments_required)
                                    .unwrap_or(50); // Fallback to 50 if enemy data not found

                                // Add to collection
                                game_manager.rustymon_collection.push(rustymon.clone());
                                log::info!("✨ Summoned {}! Added to collection", rustymon.name);

                                // Deduct fragments based on enemy's fragments_required
                                game_manager.fragment_collection.remove_fragments(species_id, fragments_to_deduct);
                                log::info!("Deducted {} fragments for {}", fragments_to_deduct, rustymon.name);

                                // Return to fragment collection
                                app_state.current_mode = AppMode::FragmentCollection;
                                app_state.needs_redraw = true;
                            }
                        }
                        RustymonSummonAction::Cancel => {
                            // Cancel summon
                            game_manager.pending_summon_rustymon = None;
                            log::info!("Summon cancelled");
                            app_state.current_mode = AppMode::FragmentCollection;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// System to handle Rustymon Skills page navigation
pub fn rustymon_skills_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in RustymonSkills mode
    if app_state.current_mode != AppMode::RustymonSkills {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from pending events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = *x as i32;
                let y = *y as i32;

                // Handle touch on rustymon skills page
                if let Some(action) = game_manager.rustymon_skills_page.handle_touch(x, y) {
                    use crate::ui::pages::rustymon_skills::RustymonSkillsAction;
                    match action {
                        RustymonSkillsAction::ToggleSkill(skill_index) => {
                            // Toggle skill on/off
                            if let Some(index) = game_manager.selected_rustymon_index {
                                if index < game_manager.rustymon_collection.len() {
                                    let rustymon = &mut game_manager.rustymon_collection[index];
                                    use crate::ui::pages::rustymon_skills::RustymonSkillsPage;
                                    if RustymonSkillsPage::toggle_skill(rustymon, skill_index) {
                                        log::info!("Toggled skill at index {}", skill_index);
                                        app_state.needs_redraw = true;
                                    } else {
                                        log::warn!("Failed to toggle skill (all slots full or invalid index)");
                                    }
                                }
                            }
                        }
                        RustymonSkillsAction::Close => {
                            log::info!("Closing Rustymon skills - returning to detail");
                            app_state.current_mode = AppMode::RustymonDetail;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
