//! Skill Selection System
//!
//! Handles skill selection page interactions

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, PendingInputEvents};
use crate::input_thread::{InputEvent, SwipeDirection};

/// System to handle skill selection page interactions
pub fn skill_selection_system(
    mut app_state: ResMut<AppState>,
    pending_events: Res<PendingInputEvents>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in SkillSelection mode
    if app_state.current_mode != AppMode::SkillSelection {
        return;
    }

    // Skip if screen is off
    if !app_state.screen_on {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Handle input events
    for event in pending_events.events.iter() {
        match event {
            InputEvent::Touch { x, y } => {
                if let Some(ref mut skill_page) = game_manager.skill_selection_page {
                    skill_page.handle_tap(*x as i32, *y as i32);
                    app_state.needs_redraw = true;

                    // Check if action was triggered
                    if let Some(action) = skill_page.take_action() {
                        use crate::ui::pages::skill_selection::SkillSelectionAction;
                        match action {
                            SkillSelectionAction::Exit => {
                                // Copy updated hero back and return to menu
                                let updated_hero = skill_page.get_hero().clone();
                                game_manager.hero = updated_hero;
                                game_manager.skill_selection_page = None;
                                log::info!("Skill selection completed, returning to menu");
                                app_state.current_mode = AppMode::Menu;
                                app_state.needs_redraw = true;
                            }
                            SkillSelectionAction::CardEquipped | SkillSelectionAction::CardUnequipped => {
                                // Copy updated hero back immediately to persist changes
                                let updated_hero = skill_page.get_hero().clone();
                                game_manager.hero = updated_hero;
                                log::info!("Hero skills updated and saved");
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
            InputEvent::Swipe { direction } => {
                if let Some(ref mut skill_page) = game_manager.skill_selection_page {
                    match direction {
                        SwipeDirection::Left => {
                            skill_page.handle_swipe_left();
                            app_state.needs_redraw = true;

                            // Check if exit was triggered
                            if let Some(action) = skill_page.take_action() {
                                use crate::ui::pages::skill_selection::SkillSelectionAction;
                                if action == SkillSelectionAction::Exit {
                                    let updated_hero = skill_page.get_hero().clone();
                                    game_manager.hero = updated_hero;
                                    game_manager.skill_selection_page = None;
                                    log::info!("Skill selection completed, returning to menu");
                                    app_state.current_mode = AppMode::Menu;
                                    app_state.needs_redraw = true;
                                }
                            }
                        }
                        SwipeDirection::Up => {
                            skill_page.handle_swipe_up();
                            app_state.needs_redraw = true;
                        }
                        SwipeDirection::Down => {
                            skill_page.handle_swipe_down();
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
