//! Menu navigation system
//!
//! Handles menu navigation and action dispatching.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::InputEvent;
use crate::systems::expedition_navigation::create_expedition_map_page;
use crate::ui::pages::MonsterListPage;

/// System to handle menu navigation
pub fn menu_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Menu mode
    if app_state.current_mode != AppMode::Menu {
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
                log::info!("Menu touch at ({}, {})", x, y);

                // Handle touch on menu page
                if let Some(action) = game_manager.menu_page.handle_touch(*x as i32, *y as i32) {
                    // Navigate based on selected action
                    use crate::ui::pages::menu::MenuAction;
                    match action {
                        MenuAction::Map => {
                            log::info!("Navigating to Map");
                            app_state.current_mode = AppMode::Map;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Battle => {
                            log::info!("Navigating to Battle");
                            if game_manager.battle_page.is_some() {
                                app_state.current_mode = AppMode::Battle;
                                app_state.needs_redraw = true;
                            } else {
                                log::warn!("No active battle");
                            }
                        }
                        MenuAction::Monsters => {
                            log::info!("Navigating to Monster List");
                            // Create monster list page
                            let team_ids: Vec<String> = game_manager.team.monster_ids().to_vec();
                            let list_page = MonsterListPage::new(&game_manager.monsters, &team_ids);
                            game_manager.monster_list_page = Some(list_page);
                            app_state.current_mode = AppMode::MonsterList;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Expedition => {
                            log::info!("Navigating to Expedition Map");
                            // Create expedition map page
                            let map_page = create_expedition_map_page(game_manager);
                            game_manager.expedition_map_page = Some(map_page);
                            app_state.current_mode = AppMode::ExpeditionMap;
                            app_state.needs_redraw = true;
                        }
                        MenuAction::Dungeon => {
                            // Dungeon system coming in Phase 2
                            log::info!("Dungeon system coming in Phase 2");
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
