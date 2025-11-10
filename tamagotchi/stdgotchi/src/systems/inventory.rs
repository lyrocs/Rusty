//! Inventory system
//!
//! Handles inventory page interactions

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, InputEventChannel};
use crate::input_thread::InputEvent;

/// System to handle inventory interactions
pub fn inventory_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Inventory mode
    if app_state.current_mode != AppMode::Inventory {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                log::info!("Inventory touch at ({}, {})", x, y);

                // Handle touch on inventory page
                if let Some(action) = game_manager.inventory_page.handle_touch(x as i32, y as i32) {
                    use crate::ui::pages::InventoryAction;
                    match action {
                        InventoryAction::SwitchToEquipment => {
                            log::info!("Switching to Equipment");
                            app_state.current_mode = AppMode::Equipment;
                            app_state.needs_redraw = true;
                        }
                        InventoryAction::Close => {
                            log::info!("Closing Inventory page");
                            app_state.current_mode = AppMode::Menu;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            InputEvent::BootPressed => {
                // Boot button closes inventory
                log::info!("Boot button pressed - returning to Menu");
                app_state.current_mode = AppMode::Menu;
                app_state.needs_redraw = true;
            }
            _ => {
                // Ignore other events
            }
        }
    }
}
