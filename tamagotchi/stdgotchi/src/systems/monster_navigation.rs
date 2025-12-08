//! Monster Navigation System
//!
//! Handles input and navigation for monster list, detail, and upgrade pages.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};
use crate::ui::pages::{MonsterListAction, MonsterDetailPage, MonsterDetailAction, MonsterUpgradePage, MonsterUpgradeAction};

/// System to handle monster list navigation
pub fn monster_navigation_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in MonsterList, MonsterDetail, or MonsterUpgrade mode
    if app_state.current_mode != AppMode::MonsterList
        && app_state.current_mode != AppMode::MonsterDetail
        && app_state.current_mode != AppMode::MonsterUpgrade
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
                    AppMode::MonsterList => {
                        if let Some(ref mut list_page) = game_manager.monster_list_page {
                            let action = list_page.handle_touch(x, y);
                            match action {
                                MonsterListAction::Select(index) => {
                                    log::info!("Selected monster at index {}", index);
                                    game_manager.selected_monster_index = Some(index);

                                    // Create detail page for selected monster
                                    if let Some(monster) = game_manager.monsters.get(index) {
                                        let is_in_team = game_manager.team.contains(&monster.id);
                                        let detail_page = MonsterDetailPage::new(monster, is_in_team);
                                        game_manager.monster_detail_page = Some(detail_page);
                                        app_state.current_mode = AppMode::MonsterDetail;
                                        app_state.needs_redraw = true;
                                    }
                                }
                                MonsterListAction::Back => {
                                    log::info!("Back to collection from monster list");
                                    game_manager.monster_list_page = None;
                                    app_state.current_mode = AppMode::Collection;
                                    app_state.needs_redraw = true;
                                }
                                MonsterListAction::None => {
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                    }
                    AppMode::MonsterDetail => {
                        if let Some(ref mut detail_page) = game_manager.monster_detail_page {
                            let action = detail_page.handle_touch(x, y);
                            match action {
                                MonsterDetailAction::Back => {
                                    log::info!("Back to monster list");
                                    game_manager.monster_detail_page = None;
                                    game_manager.selected_monster_index = None;
                                    // Keep existing monster_list_page (zone-filtered)
                                    app_state.current_mode = AppMode::MonsterList;
                                    app_state.needs_redraw = true;
                                }
                                MonsterDetailAction::AddToTeam => {
                                    if let Some(index) = game_manager.selected_monster_index {
                                        // Extract data before mutating to avoid borrow conflicts
                                        let monster_data = game_manager.monsters.get(index)
                                            .map(|m| (m.id.clone(), m.name.clone()));

                                        if let Some((monster_id, monster_name)) = monster_data {
                                            if game_manager.team.add(monster_id) {
                                                log::info!("Added {} to team", monster_name);
                                                // Refresh detail page
                                                if let Some(monster) = game_manager.monsters.get(index) {
                                                    let detail_page = MonsterDetailPage::new(monster, true);
                                                    game_manager.monster_detail_page = Some(detail_page);
                                                }
                                            } else {
                                                log::warn!("Could not add to team (full or already in team)");
                                            }
                                        }
                                    }
                                    app_state.needs_redraw = true;
                                }
                                MonsterDetailAction::RemoveFromTeam => {
                                    if let Some(index) = game_manager.selected_monster_index {
                                        // Extract data before mutating to avoid borrow conflicts
                                        let monster_data = game_manager.monsters.get(index)
                                            .map(|m| (m.id.clone(), m.name.clone()));

                                        if let Some((monster_id, monster_name)) = monster_data {
                                            if game_manager.team.remove(&monster_id) {
                                                log::info!("Removed {} from team", monster_name);
                                                // Refresh detail page
                                                if let Some(monster) = game_manager.monsters.get(index) {
                                                    let detail_page = MonsterDetailPage::new(monster, false);
                                                    game_manager.monster_detail_page = Some(detail_page);
                                                }
                                            }
                                        }
                                    }
                                    app_state.needs_redraw = true;
                                }
                                MonsterDetailAction::Upgrade => {
                                    if let Some(index) = game_manager.selected_monster_index {
                                        // Extract data before mutating
                                        let upgrade_data = game_manager.monsters.get(index).map(|monster| {
                                            let crystals = game_manager.player.crystals;
                                            let essence_count = game_manager.player.get_essence(monster.element);
                                            let page = MonsterUpgradePage::new(monster, crystals, essence_count);
                                            let name = monster.name.clone();
                                            (page, name)
                                        });

                                        if let Some((upgrade_page, name)) = upgrade_data {
                                            game_manager.monster_upgrade_page = Some(upgrade_page);
                                            app_state.current_mode = AppMode::MonsterUpgrade;
                                            app_state.needs_redraw = true;
                                            log::info!("Opening upgrade page for {}", name);
                                        }
                                    }
                                }
                                MonsterDetailAction::None => {
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                    }
                    AppMode::MonsterUpgrade => {
                        if let Some(ref mut upgrade_page) = game_manager.monster_upgrade_page {
                            let action = upgrade_page.handle_touch(x, y);
                            handle_upgrade_action(action, game_manager, &mut app_state);
                        }
                    }
                    _ => {}
                }
            }
            InputEvent::Swipe { direction } => {
                match app_state.current_mode {
                    AppMode::MonsterList => {
                        match direction {
                            SwipeDirection::Right => {
                                log::info!("Swipe right: back to collection");
                                game_manager.monster_list_page = None;
                                app_state.current_mode = AppMode::Collection;
                                app_state.needs_redraw = true;
                            }
                            SwipeDirection::Up | SwipeDirection::Down => {
                                // Swipe up/down to scroll
                                if let Some(ref mut list_page) = game_manager.monster_list_page {
                                    list_page.handle_swipe(*direction == SwipeDirection::Up);
                                    app_state.needs_redraw = true;
                                }
                            }
                            _ => {}
                        }
                    }
                    AppMode::MonsterDetail => {
                        if *direction == SwipeDirection::Right {
                            log::info!("Swipe right: back to monster list");
                            game_manager.monster_detail_page = None;
                            game_manager.selected_monster_index = None;
                            // Keep existing monster_list_page (zone-filtered)
                            app_state.current_mode = AppMode::MonsterList;
                            app_state.needs_redraw = true;
                        }
                    }
                    AppMode::MonsterUpgrade => {
                        if *direction == SwipeDirection::Right {
                            log::info!("Swipe right: back to monster detail");
                            go_back_to_detail(game_manager, &mut app_state);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Handle upgrade actions
fn handle_upgrade_action(
    action: MonsterUpgradeAction,
    game_manager: &mut GameManager,
    app_state: &mut AppState,
) {
    match action {
        MonsterUpgradeAction::Back => {
            go_back_to_detail(game_manager, app_state);
        }
        MonsterUpgradeAction::UpgradeAtk => {
            apply_stat_upgrade(game_manager, app_state, "atk");
        }
        MonsterUpgradeAction::UpgradeDef => {
            apply_stat_upgrade(game_manager, app_state, "def");
        }
        MonsterUpgradeAction::UpgradeSpd => {
            apply_stat_upgrade(game_manager, app_state, "spd");
        }
        MonsterUpgradeAction::UpgradeHp => {
            apply_stat_upgrade(game_manager, app_state, "hp");
        }
        MonsterUpgradeAction::MajorUpgradeAtk => {
            apply_major_upgrade(game_manager, app_state);
        }
        MonsterUpgradeAction::None => {}
    }
}

/// Go back from upgrade to detail page
fn go_back_to_detail(game_manager: &mut GameManager, app_state: &mut AppState) {
    game_manager.monster_upgrade_page = None;

    // Refresh detail page with updated monster data
    if let Some(index) = game_manager.selected_monster_index {
        if let Some(monster) = game_manager.monsters.get(index) {
            let is_in_team = game_manager.team.contains(&monster.id);
            let detail_page = MonsterDetailPage::new(monster, is_in_team);
            game_manager.monster_detail_page = Some(detail_page);
        }
    }

    app_state.current_mode = AppMode::MonsterDetail;
    app_state.needs_redraw = true;
}

/// Apply a +1 stat upgrade
fn apply_stat_upgrade(game_manager: &mut GameManager, app_state: &mut AppState, stat: &str) {
    use crate::game::systems::progression::upgrade::upgrade_cost_crystals;

    let Some(index) = game_manager.selected_monster_index else { return };

    // Get current stat value and cost
    let (_current_stat, cost) = {
        let Some(monster) = game_manager.monsters.get(index) else { return };
        let stat_val = match stat {
            "atk" => monster.atk,
            "def" => monster.def,
            "spd" => monster.spd,
            "hp" => monster.hp_max,
            _ => return,
        };
        (stat_val, upgrade_cost_crystals(stat_val))
    };

    // Check if can afford
    if game_manager.player.crystals < cost {
        log::warn!("Not enough crystals for upgrade");
        return;
    }

    // Apply upgrade
    game_manager.player.crystals -= cost;

    if let Some(monster) = game_manager.monsters.get_mut(index) {
        match stat {
            "atk" => {
                monster.atk += 1;
                log::info!("Upgraded ATK to {} for {} crystals", monster.atk, cost);
            }
            "def" => {
                monster.def += 1;
                log::info!("Upgraded DEF to {} for {} crystals", monster.def, cost);
            }
            "spd" => {
                monster.spd += 1;
                log::info!("Upgraded SPD to {} for {} crystals", monster.spd, cost);
            }
            "hp" => {
                monster.hp_max += 10;
                monster.hp_current = monster.hp_current.min(monster.hp_max);
                log::info!("Upgraded HP to {} for {} crystals", monster.hp_max, cost);
            }
            _ => {}
        }

        // Refresh upgrade page
        let crystals = game_manager.player.crystals;
        let essence_count = game_manager.player.get_essence(monster.element);
        if let Some(ref mut page) = game_manager.monster_upgrade_page {
            page.refresh(monster, crystals, essence_count);
        }
    }

    app_state.needs_redraw = true;
}

/// Apply a major (+10) stat upgrade
fn apply_major_upgrade(game_manager: &mut GameManager, app_state: &mut AppState) {
    use crate::game::systems::progression::upgrade::major_upgrade_cost;

    let Some(index) = game_manager.selected_monster_index else { return };

    // Get cost and element
    let (crystal_cost, essence_cost, element) = {
        let Some(monster) = game_manager.monsters.get(index) else { return };
        let (c, e) = major_upgrade_cost(monster.atk);
        (c, e, monster.element)
    };

    // Check if can afford
    if game_manager.player.crystals < crystal_cost {
        log::warn!("Not enough crystals for major upgrade");
        return;
    }
    if game_manager.player.get_essence(element) < essence_cost as u16 {
        log::warn!("Not enough essences for major upgrade");
        return;
    }

    // Apply costs
    game_manager.player.crystals -= crystal_cost;
    game_manager.player.spend_essence(element, essence_cost as u16);

    // Apply upgrade
    if let Some(monster) = game_manager.monsters.get_mut(index) {
        monster.atk += 10;
        log::info!("Major upgrade: ATK to {} for {} crystals + {} essences",
            monster.atk, crystal_cost, essence_cost);

        // Refresh upgrade page
        let crystals = game_manager.player.crystals;
        let essence_count = game_manager.player.get_essence(monster.element);
        if let Some(ref mut page) = game_manager.monster_upgrade_page {
            page.refresh(monster, crystals, essence_count);
        }
    }

    app_state.needs_redraw = true;
}
