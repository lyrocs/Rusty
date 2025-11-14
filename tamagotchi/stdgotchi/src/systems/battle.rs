//! Battle system
//!
//! Handles input during battle mode, including menu access.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, InputEventChannel};
use crate::input_thread::InputEvent;

/// System to handle battle mode input
pub fn battle_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Battle mode
    if app_state.current_mode != AppMode::Battle {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                let x = x as i32;
                let y = y as i32;

                // Handle touch on battle page (for team switching)
                if let Some(ref mut battle_page) = game_manager.battle_page {
                    if let Some(action) = battle_page.handle_touch(x, y) {
                        use crate::ui::pages::battle::BattleAction;
                        match action {
                            BattleAction::SwitchRustymon(slot) => {
                                log::info!("Switching to team slot {}", slot);
                                if let Err(e) = battle_page.switch_rustymon(slot) {
                                    log::error!("Failed to switch Rustymon: {:?}", e);
                                }
                                app_state.needs_redraw = true;
                            }
                            BattleAction::UseSkill(skill_id) => {
                                log::info!("Using skill {}", skill_id);
                                if let Err(e) = battle_page.use_skill(skill_id) {
                                    log::error!("Failed to use skill: {:?}", e);
                                }
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
            InputEvent::BootPressed => {
                // Boot button opens menu from battle
                log::info!("Boot button pressed - Opening Menu from Battle");

                // Sync battle state before switching modes
                game_manager.sync_battle_state();

                app_state.current_mode = AppMode::Menu;
                app_state.needs_redraw = true;
            }
            _ => {
                // Other events are not needed in battle mode
            }
        }
    }
}
