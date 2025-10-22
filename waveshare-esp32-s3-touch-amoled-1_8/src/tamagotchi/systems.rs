use bevy_ecs::prelude::*;
use ft3x68_rs::{TouchState, TouchPoint};

use crate::ecs::resources::{TouchResource, ButtonResource, DisplayResource};
use crate::tamagotchi::models::{GameState, GamePage, FarmState, RestState, Enemy};
use crate::tamagotchi::ui::{draw_overview_page, draw_farm_page, draw_rest_page, draw_menu};

const DEBOUNCE_THRESHOLD: u8 = 3;

/// System to handle button input for menu toggling
pub fn tamagotchi_button_system(
    mut button_res: NonSendMut<ButtonResource>,
    mut game_state: ResMut<GameState>,
) {
    // BOOT Button (GPIO0) - Active Low
    let boot_pressed = button_res.boot_button.is_low();

    // Debouncing logic
    if boot_pressed {
        if button_res.boot_debounce_counter < DEBOUNCE_THRESHOLD {
            button_res.boot_debounce_counter += 1;
        }
    } else {
        button_res.boot_debounce_counter = 0;
    }

    // Detect rising edge (button release after being pressed)
    if button_res.boot_last_state && !boot_pressed && button_res.boot_debounce_counter == 0 {
        // Toggle menu
        if game_state.current_page == GamePage::Menu {
            // Close menu and go to selected page
            game_state.current_page = match game_state.menu_selection {
                0 => GamePage::Overview,
                1 => GamePage::Farm,
                2 => GamePage::Rest,
                _ => GamePage::Overview,
            };
        } else {
            // Open menu
            game_state.current_page = GamePage::Menu;
        }
    }

    // Update last state
    button_res.boot_last_state = button_res.boot_debounce_counter >= DEBOUNCE_THRESHOLD;
}

/// System to handle touch input
pub fn tamagotchi_touch_system(
    mut touch_res: NonSendMut<TouchResource>,
    mut game_state: ResMut<GameState>,
) {
    let touching = touch_res
        .touch
        .touch1()
        .unwrap_or_else(|_e| TouchState::Released);

    if let TouchState::Pressed(TouchPoint { x, y }) = touching {
        handle_touch_input(&mut game_state, x, y);
    }
}

/// Handle touch input based on current page
fn handle_touch_input(game_state: &mut GameState, x: u16, y: u16) {
    match game_state.current_page {
        GamePage::Menu => {
            // Menu item selection based on touch Y position
            if x > 60 && x < 308 && y > 170 && y < 275 {
                let item_index = ((y - 170) / 35) as u8;
                if item_index < 3 {
                    game_state.menu_selection = item_index;
                    // Close menu and navigate to selected page
                    game_state.current_page = match item_index {
                        0 => GamePage::Overview,
                        1 => GamePage::Farm,
                        2 => GamePage::Rest,
                        _ => GamePage::Overview,
                    };
                }
            }
        }
        GamePage::Farm => {
            match game_state.farm_state {
                FarmState::Idle => {
                    // Start farming if hero has enough SP
                    if game_state.hero.sp >= 20 {
                        // Generate a random enemy (using touch position as random seed)
                        let rng_value = (x.wrapping_add(y)) as u8;
                        let enemy = Enemy::random_for_level(game_state.hero.level, rng_value);
                        game_state.start_farming(enemy);
                    }
                }
                FarmState::Victory | FarmState::Defeat => {
                    // Reset farming state
                    game_state.reset_farming();
                }
                _ => {}
            }
        }
        GamePage::Rest => {
            if game_state.rest_state == RestState::FullSP {
                // Return to overview when SP is full
                game_state.current_page = GamePage::Overview;
                game_state.rest_state = RestState::Resting;
                game_state.rest_progress = 0;
            }
        }
        GamePage::Overview => {
            // No touch actions on overview page
        }
    }
}

/// System to update game logic (farming progress, SP regen, etc.)
pub fn tamagotchi_update_system(mut game_state: ResMut<GameState>) {
    // Get current time (simplified - uses generation counter as time proxy)
    // In a real implementation, you'd use actual millisecond timing
    let current_ms = game_state.last_update_ms + 16; // Assume ~60 FPS = 16ms per frame
    let delta_ms = current_ms - game_state.last_update_ms;
    game_state.last_update_ms = current_ms;

    // Update farming progress
    if game_state.current_page == GamePage::Farm && game_state.farm_state == FarmState::Fighting {
        game_state.update_farm_progress(delta_ms);
    }

    // Update rest progress
    if game_state.current_page == GamePage::Rest && game_state.rest_state == RestState::Resting {
        game_state.update_rest_progress(delta_ms);
    }
}

/// System to render the current page
pub fn tamagotchi_render_system(
    mut display_res: NonSendMut<DisplayResource>,
    game_state: Res<GameState>,
) {
    // Draw the current page
    match game_state.current_page {
        GamePage::Overview => {
            draw_overview_page(&mut display_res.display, &game_state.hero).ok();
        }
        GamePage::Farm => {
            draw_farm_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::Rest => {
            draw_rest_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::Menu => {
            // Draw the previous page first, then overlay menu
            // For simplicity, we'll just draw menu on a dark background
            draw_menu(&mut display_res.display, &game_state).ok();
        }
    }

    // Flush the display
    display_res.display.flush().ok();
}
