//! Menu navigation system
//!
//! Handles menu interactions and navigation to different game modes.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, SharedI2cResource, TouchResource};

/// System to handle menu navigation
pub fn menu_system(
    mut app_state: ResMut<AppState>,
    mut touch_res: NonSendMut<TouchResource>,
    i2c_res: NonSendMut<SharedI2cResource>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Menu mode
    if app_state.current_mode != AppMode::Menu {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Get I2C access from shared resource
    let Some(i2c) = i2c_res.get() else {
        log::error!("Failed to get I2C access in menu_system");
        return;
    };

    // Check for touch (taps)
    if let Ok(count) = touch_res.touch.finger_number(i2c) {
        if count > 0 && !touch_res.last_touch_active {
            // New touch detected
            if let Ok(touches) = touch_res.touch.get_touches(i2c) {
                if let Some(point) = touches.first() {
                    let x = point.x as i32;
                    let y = point.y as i32;
                    log::info!("Menu touch at ({}, {})", x, y);

                    // Handle touch on menu page
                    if let Some(action) = game_manager.menu_page.handle_touch(x, y) {
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
                                // Only switch to battle if there's an active battle
                                if game_manager.battle_page.is_some() {
                                    app_state.current_mode = AppMode::Battle;
                                    app_state.needs_redraw = true;
                                } else {
                                    log::warn!("No active battle");
                                }
                            }
                            MenuAction::Overview => {
                                log::info!("Navigating to Hero Overview");
                                app_state.current_mode = AppMode::HeroOverview;
                                app_state.needs_redraw = true;
                            }
                        }
                    }

                    // Mark touch as active
                    touch_res.last_touch_active = true;
                }
            }
        } else if count == 0 && touch_res.last_touch_active {
            // Touch released
            touch_res.last_touch_active = false;
        }
    }
}
