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
        game_state.needs_redraw = true; // Mark for redraw on page change
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

    let is_pressed = matches!(touching, TouchState::Pressed(_));

    // Detect touch on release (rising edge) to prevent accidental double-taps
    if touch_res.last_touch_state && !is_pressed {
        // Touch was just released, process it
        if let TouchState::Released = touching {
            // Use the last known touch position - we'll need to store it
            // For now, just mark that a touch happened
        }
    }

    // Also process immediate touch for responsiveness
    if let TouchState::Pressed(TouchPoint { x, y }) = touching {
        // Only process if this is a new touch (wasn't pressed last frame)
        if !touch_res.last_touch_state {
            esp_println::println!("[TOUCH] Detected at ({}, {})", x, y);
            handle_touch_input(&mut game_state, x, y);
        }
    }

    // Update last touch state
    touch_res.last_touch_state = is_pressed;
}

/// Handle touch input based on current page
fn handle_touch_input(game_state: &mut GameState, x: u16, y: u16) {
    game_state.needs_redraw = true; // Mark for redraw on any touch
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
                    // Check cooldown first
                    if game_state.farm_touch_cooldown > 0 {
                        esp_println::println!("[FARM] Touch cooldown active: {}ms", game_state.farm_touch_cooldown);
                        return; // Ignore touch during cooldown
                    }

                    // Start farming if hero has enough SP
                    if game_state.hero.sp >= 20 {
                        esp_println::println!("[FARM] Starting farm with enemy");
                        // Generate a random enemy (using touch position as random seed)
                        let rng_value = (x.wrapping_add(y)) as u8;
                        let enemy = Enemy::random_for_level(game_state.hero.level, rng_value);
                        game_state.start_farming(enemy);
                    } else {
                        esp_println::println!("[FARM] Not enough SP: {}/20", game_state.hero.sp);
                    }
                }
                FarmState::Victory | FarmState::Defeat => {
                    esp_println::println!("[FARM] Resetting farming state from {:?}", game_state.farm_state);
                    // Reset farming state
                    game_state.reset_farming();
                }
                _ => {
                    esp_println::println!("[FARM] Touch ignored, state: {:?}", game_state.farm_state);
                }
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

    // Update farm touch cooldown
    if game_state.farm_touch_cooldown > 0 {
        game_state.farm_touch_cooldown = game_state.farm_touch_cooldown.saturating_sub(delta_ms);
    }

    // Update FPS counter every 2 seconds for less frequent updates
    game_state.frame_count += 1;
    let fps_elapsed = game_state.last_update_ms.wrapping_sub(game_state.last_fps_update_ms);
    if fps_elapsed >= 2000 {
        // Calculate FPS: frames / seconds
        game_state.fps = (game_state.frame_count * 1000) / fps_elapsed;
        game_state.frame_count = 0;
        game_state.last_fps_update_ms = game_state.last_update_ms;
        game_state.needs_redraw = true; // Redraw when FPS updates
    }

    // Update farming progress (only redraw every ~200ms for smoother animation without too much overhead)
    if game_state.current_page == GamePage::Farm && game_state.farm_state == FarmState::Fighting {
        let old_percent = (game_state.farm_progress * 100) / game_state.farm_duration_ms;
        game_state.update_farm_progress(delta_ms);
        let new_percent = (game_state.farm_progress * 100) / game_state.farm_duration_ms;
        // Only redraw if progress bar changes by at least 1%
        if new_percent != old_percent {
            game_state.needs_redraw = true;
        }
    }

    // Update rest progress (only redraw when SP actually changes)
    if game_state.current_page == GamePage::Rest && game_state.rest_state == RestState::Resting {
        let old_sp = game_state.hero.sp;
        game_state.update_rest_progress(delta_ms);
        // Only redraw if SP changed or state changed
        if game_state.hero.sp != old_sp || game_state.rest_state != RestState::Resting {
            game_state.needs_redraw = true;
        }
    }
}

/// System to render the current page
pub fn tamagotchi_render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut game_state: ResMut<GameState>,
    battery_res: Res<BatteryResource>,
) {
    // Only render if something changed
    if !game_state.needs_redraw {
        return;
    }

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

    // Clear the dirty flag
    game_state.needs_redraw = false;
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
        game_state.needs_redraw = true; // Redraw to show save message
    }

    // Clear save message after timeout
    if game_state.save_status_timeout > 0 && game_state.last_update_ms >= game_state.save_status_timeout {
        game_state.save_status_msg = None;
        game_state.save_status_timeout = 0;
        game_state.needs_redraw = true; // Redraw to clear message
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
