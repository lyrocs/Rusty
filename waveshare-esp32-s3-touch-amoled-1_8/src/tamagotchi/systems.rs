use bevy_ecs::prelude::*;
use ft3x68_rs::{TouchState, TouchPoint};

use crate::ecs::resources::{TouchResource, ButtonResource, DisplayResource, BatteryResource, RtcResource, SdCardResource};
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
            // Updated for new larger menu with 55px spacing
            if x > 40 && x < 328 && y > 130 && y < 350 {
                let item_index = ((y - 130) / 55) as u8;
                if item_index < 4 { // Now 4 items: Overview, Farm, Rest, Save
                    game_state.menu_selection = item_index;

                    // Handle selection
                    if item_index == 3 {
                        // Save Game selected
                        game_state.save_requested = true;
                        game_state.current_page = GamePage::Overview; // Go back to overview after save
                    } else {
                        // Navigate to selected page
                        game_state.current_page = match item_index {
                            0 => GamePage::Overview,
                            1 => GamePage::Farm,
                            2 => GamePage::Rest,
                            _ => GamePage::Overview,
                        };
                    }
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
pub fn tamagotchi_update_system(
    mut rtc_res: NonSendMut<RtcResource>,
    mut game_state: ResMut<GameState>,
) {
    // Get current CPU cycles for precise timing
    let current_cycles = esp_hal::xtensa_lx::timer::get_cycle_count();
    let cycles_elapsed = current_cycles.wrapping_sub(rtc_res.last_cycles);

    // Convert cycles to milliseconds (CPU freq is in MHz, cycles_elapsed is in cycles)
    // delta_ms = (cycles_elapsed / cycles_per_ms) = (cycles_elapsed / (cpu_freq_mhz * 1000))
    let delta_ms = (cycles_elapsed as u64 / (rtc_res.cpu_freq_mhz * 1000)) as u32;

    // Update last cycles for next frame
    rtc_res.last_cycles = current_cycles;

    // Update game time
    game_state.last_update_ms = game_state.last_update_ms.wrapping_add(delta_ms);

    // Update FPS counter
    game_state.frame_count += 1;
    let fps_elapsed = game_state.last_update_ms.wrapping_sub(game_state.last_fps_update_ms);
    if fps_elapsed >= 1000 {
        // Calculate FPS: frames / seconds
        game_state.fps = (game_state.frame_count * 1000) / fps_elapsed;
        game_state.frame_count = 0;
        game_state.last_fps_update_ms = game_state.last_update_ms;
    }

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
    battery_res: Res<BatteryResource>,
) {
    // Get battery info
    let battery_mv = battery_res.voltage_mv;
    let battery_pct = battery_res.percent;
    let fps = game_state.fps;

    // Draw the current page
    match game_state.current_page {
        GamePage::Overview => {
            draw_overview_page(&mut display_res.display, &game_state.hero, battery_mv, battery_pct, fps, game_state.save_status_msg).ok();
        }
        GamePage::Farm => {
            draw_farm_page(&mut display_res.display, &game_state, battery_mv, battery_pct, fps).ok();
        }
        GamePage::Rest => {
            draw_rest_page(&mut display_res.display, &game_state, battery_mv, battery_pct, fps).ok();
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

/// System to handle save requests with SD card persistence
pub fn tamagotchi_save_system(
    mut sd_card_res: NonSendMut<SdCardResource>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.save_requested {
        game_state.save_requested = false;

        // Generate save data
        let save_data = game_state.hero.to_save_string();

        esp_println::println!("[SAVE] Saving hero: Level {} {} with {} EXP and {} Zeny",
            game_state.hero.level,
            game_state.hero.job,
            game_state.hero.exp,
            game_state.hero.zeny
        );

        // Try to write to SD card
        match save_hero_to_sd(&mut sd_card_res, save_data.as_str()) {
            Ok(_) => {
                esp_println::println!("[SAVE] Successfully saved to SD card");
                game_state.save_status_msg = Some("Saved to SD!");
            }
            Err(e) => {
                esp_println::println!("[SAVE] Error saving to SD: {:?}", e);
                game_state.save_status_msg = Some("Save failed!");
            }
        }

        // Show success message for 3 seconds
        game_state.save_status_timeout = game_state.last_update_ms + 3000;
    }

    // Clear save message after timeout
    if game_state.save_status_timeout > 0 && game_state.last_update_ms >= game_state.save_status_timeout {
        game_state.save_status_msg = None;
        game_state.save_status_timeout = 0;
    }
}

/// Helper function to save hero data to SD card
fn save_hero_to_sd(
    sd_card_res: &mut SdCardResource,
    save_data: &str,
) -> Result<(), embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> {
    use embedded_sdmmc::{Mode, VolumeIdx};

    // Open volume
    let mut volume = sd_card_res.volume_mgr.open_volume(VolumeIdx(0))?;

    // Open root directory
    let mut root_dir = volume.open_root_dir()?;

    // Create or truncate save file
    let mut file = root_dir.open_file_in_dir("HERO.SAV", Mode::ReadWriteCreateOrTruncate)?;

    // Write save data
    file.write(save_data.as_bytes())?;

    Ok(())
}
