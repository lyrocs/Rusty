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
            apply_bonus_upgrade(game_manager, app_state, "atk");
        }
        MonsterUpgradeAction::UpgradeDef => {
            apply_bonus_upgrade(game_manager, app_state, "def");
        }
        MonsterUpgradeAction::UpgradeSpd => {
            apply_bonus_upgrade(game_manager, app_state, "spd");
        }
        MonsterUpgradeAction::UpgradeHp => {
            apply_bonus_upgrade(game_manager, app_state, "hp");
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

/// Apply a +1 bonus upgrade (EV-style)
fn apply_bonus_upgrade(game_manager: &mut GameManager, app_state: &mut AppState, stat: &str) {
    use crate::game::systems::progression::upgrade::upgrade_cost_crystals;

    let Some(index) = game_manager.selected_monster_index else { return };

    // Get current bonus value and cost
    let cost = {
        let Some(monster) = game_manager.monsters.get(index) else { return };
        let bonus = match stat {
            "atk" => monster.atk_bonus,
            "def" => monster.def_bonus,
            "spd" => monster.spd_bonus,
            "hp" => monster.hp_bonus,
            _ => return,
        };
        upgrade_cost_crystals(bonus)
    };

    // Check if can afford
    if game_manager.player.crystals < cost {
        log::warn!("Not enough crystals for bonus upgrade");
        return;
    }

    // Apply upgrade
    game_manager.player.crystals -= cost;

    if let Some(monster) = game_manager.monsters.get_mut(index) {
        let success = match stat {
            "atk" => {
                let result = monster.add_atk_bonus(1);
                if result {
                    log::info!("Upgraded ATK bonus to {} for {} crystals", monster.atk_bonus, cost);
                }
                result
            }
            "def" => {
                let result = monster.add_def_bonus(1);
                if result {
                    log::info!("Upgraded DEF bonus to {} for {} crystals", monster.def_bonus, cost);
                }
                result
            }
            "spd" => {
                let result = monster.add_spd_bonus(1);
                if result {
                    log::info!("Upgraded SPD bonus to {} for {} crystals", monster.spd_bonus, cost);
                }
                result
            }
            "hp" => {
                let result = monster.add_hp_bonus(1);
                if result {
                    log::info!("Upgraded HP bonus to {} (+{} HP) for {} crystals",
                        monster.hp_bonus, monster.hp_bonus as u16 * 10, cost);
                }
                result
            }
            _ => false,
        };

        if !success {
            // Refund if upgrade failed (already at max)
            game_manager.player.crystals += cost;
            log::warn!("Bonus already at max!");
            return;
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
