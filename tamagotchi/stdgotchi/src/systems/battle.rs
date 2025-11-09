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
            InputEvent::BootPressed => {
                // Boot button opens menu from battle
                log::info!("Boot button pressed - Opening Menu from Battle");

                // Sync battle state before switching modes
                game_manager.sync_battle_state();

                app_state.current_mode = AppMode::Menu;
                app_state.needs_redraw = true;
            }
            _ => {
                // Battle pages handle their own touch input internally via Page::update()
                // Other events are not needed in battle mode
            }
        }
    }
}
