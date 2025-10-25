use bevy_ecs::prelude::*;
use ft3x68_rs::{TouchPoint, TouchState};

use crate::ecs::resources::{
    BatteryResource, ButtonResource, DisplayResource, RtcResource, SdCardResource, TouchResource,
};
use crate::tamagotchi::models::{
    BattleState, Enemy, FarmState, GamePage, GameState, MapHelper, RestState,
};
use crate::tamagotchi::ui::{
    draw_battle_page, draw_farm_page, draw_map_page, draw_menu, draw_overview_page, draw_rest_page,
};

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
            // Menu now has 4 items: Overview, Rest, Map, Save
            game_state.current_page = match game_state.menu_selection {
                0 => GamePage::Overview,
                1 => GamePage::Rest,
                2 => GamePage::Map,
                // 3 is Save - handled in touch system, stays on current page
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
            // Menu item selection based on button position (2 columns x 2 rows)
            // Now only 4 items - Farm and Battle removed (accessed via Map)
            // Button layout:
            // [Overview(0)] [Rest(1)]      Row 0: y=110-180
            // [Map(2)]      [Save(3)]      Row 1: y=190-260
            //
            // Col 0: x=24-174, Col 1: x=184-334

            // Check if touch is within button area
            if y >= 110 && y <= 260 {
                let mut clicked_button: Option<u8> = None;

                // Determine row (0 or 1)
                let row = if y >= 110 && y <= 180 {
                    0
                } else if y >= 190 && y <= 260 {
                    1
                } else {
                    255 // Invalid
                };

                // Determine column (0 or 1)
                let col = if x >= 24 && x <= 174 {
                    0
                } else if x >= 184 && x <= 334 {
                    1
                } else {
                    255 // Invalid
                };

                // Calculate button index (row * 2 + col)
                if row < 2 && col < 2 {
                    let button_index = row * 2 + col;
                    if button_index < 4 {
                        // Now 4 buttons exist
                        clicked_button = Some(button_index);
                    }
                }

                if let Some(item_index) = clicked_button {
                    game_state.menu_selection = item_index;

                    esp_println::println!(
                        "[MENU] Selected button {} at ({}, {})",
                        item_index,
                        x,
                        y
                    );

                    // Handle selection
                    if item_index == 3 {
                        // Save Game selected
                        game_state.save_requested = true;
                        game_state.current_page = GamePage::Overview; // Go back to overview after save
                    } else {
                        // Navigate to selected page
                        game_state.current_page = match item_index {
                            0 => GamePage::Overview,
                            1 => GamePage::Rest,
                            2 => GamePage::Map,
                            _ => GamePage::Overview,
                        };
                    }
                }
            }
        }
        GamePage::Farm => {
            match game_state.farm_state {
                FarmState::Idle => {
                    // Farm should be started from Map page
                    // If we're here with Idle state, go back to map
                    esp_println::println!("[FARM] No active farm, returning to map");
                    game_state.current_page = GamePage::Map;
                }
                FarmState::Victory | FarmState::Defeat => {
                    esp_println::println!(
                        "[FARM] Restarting auto farm from {:?}",
                        game_state.farm_state
                    );
                    // Reset farming state first
                    game_state.reset_farming();

                    // Restart farming with a new enemy from current map
                    let map_id = game_state.current_location;
                    let enemy_ids = MapHelper::enemies(map_id);
                    if !enemy_ids.is_empty() && game_state.hero.sp >= 20 {
                        // Pick random enemy from map using touch coordinates as seed
                        let rng_value = (x.wrapping_add(y)) as u8;
                        let enemy_index = (rng_value as usize) % enemy_ids.len();
                        let enemy_id = enemy_ids[enemy_index];

                        if let Some(enemy) = Enemy::from_id(enemy_id) {
                            esp_println::println!("[FARM] Starting new farm with {}", enemy.name);
                            game_state.start_farming(enemy);
                        }
                    } else if game_state.hero.sp < 20 {
                        esp_println::println!("[FARM] Not enough SP to restart farming");
                        game_state.current_page = GamePage::Map;
                    }
                }
                _ => {
                    esp_println::println!(
                        "[FARM] Touch ignored, state: {:?}",
                        game_state.farm_state
                    );
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
        GamePage::Battle => {
            match game_state.battle_state {
                BattleState::Idle => {
                    // Battle should be started from Map page
                    // If we're here with Idle state, go back to map
                    esp_println::println!("[BATTLE] No active battle, returning to map");
                    game_state.current_page = GamePage::Map;
                }
                BattleState::Playing => {
                    // Record touch position for debug display
                    game_state.battle_last_touch_x = x as i32;
                    game_state.battle_last_touch_y = y as i32;
                    game_state.battle_last_touch_time = game_state.last_update_ms;

                    // Check if touch hit any circle
                    let hit = game_state.click_battle_circle(x as i32, y as i32);
                    if hit {
                        esp_println::println!("[BATTLE] Circle hit at ({}, {})", x, y);
                    } else {
                        esp_println::println!("[BATTLE] Touch miss at ({}, {})", x, y);
                    }
                }
                BattleState::Victory | BattleState::Defeat => {
                    // Prevent accidental clicks - require 500ms delay after battle ends
                    let time_since_end = game_state.last_update_ms.saturating_sub(game_state.battle_end_time);
                    if time_since_end < 500 {
                        esp_println::println!(
                            "[BATTLE] Ignoring click too soon after battle end ({}ms)",
                            time_since_end
                        );
                        return;
                    }

                    esp_println::println!(
                        "[BATTLE] Restarting manual battle from {:?}",
                        game_state.battle_state
                    );
                    // Reset battle state first
                    game_state.reset_battle();

                    // Restart battle with a new enemy from current map
                    let map_id = game_state.current_location;
                    let enemy_ids = MapHelper::enemies(map_id);
                    if !enemy_ids.is_empty() && game_state.hero.sp >= 30 {
                        // Pick random enemy from map using touch coordinates as seed
                        let rng_value = (x.wrapping_add(y)) as u8;
                        let enemy_index = (rng_value as usize) % enemy_ids.len();
                        let enemy_id = enemy_ids[enemy_index];

                        if let Some(enemy) = Enemy::from_id(enemy_id) {
                            esp_println::println!("[BATTLE] Starting new battle with {}", enemy.name);
                            game_state.start_battle(enemy);
                        }
                    } else if game_state.hero.sp < 30 {
                        esp_println::println!("[BATTLE] Not enough SP to restart battle");
                        game_state.current_page = GamePage::Map;
                    }
                }
            }
        }
        GamePage::Map => {
            // Map navigation with border buttons and center actions
            let map_id = game_state.current_location;
            let exits = MapHelper::exits(map_id);
            let location_type = MapHelper::location_type(map_id);

            // Check directional navigation buttons (large border buttons)
            let mut traveled = false;
            for exit in exits.iter() {
                let hit = match exit.direction {
                    "North" => x >= 10 && x <= 358 && y <= 40,
                    "South" => x >= 10 && x <= 358 && y >= 408,
                    "West" => x <= 50 && y >= 45 && y <= 403,
                    "East" => x >= 318 && y >= 45 && y <= 403,
                    _ => false,
                };

                if hit {
                    esp_println::println!(
                        "[MAP] Traveling {} to {}",
                        exit.direction,
                        MapHelper::name(exit.destination)
                    );
                    game_state.current_location = exit.destination;
                    traveled = true;
                    break;
                }
            }

            // Check center action buttons (only if didn't travel)
            if !traveled {
                match location_type {
                    crate::tamagotchi::models::LocationType::City => {
                        // NPC action buttons (2x2 grid in center)
                        let npcs = MapHelper::npcs(map_id);
                        if !npcs.is_empty() {
                            for (i, npc) in npcs.iter().enumerate() {
                                let row = i / 2;
                                let col = i % 2;
                                let btn_x = 59 + col as i32 * 130;
                                let btn_y = 100 + row as i32 * 75;

                                if x >= btn_x as u16
                                    && x <= (btn_x + 120) as u16
                                    && y >= btn_y as u16
                                    && y <= (btn_y + 60) as u16
                                {
                                    esp_println::println!("[MAP] Selected NPC: {}", npc);
                                    // TODO: Implement NPC interactions
                                }
                            }
                        }
                    }
                    crate::tamagotchi::models::LocationType::Field => {
                        // Check Auto Farm button (84, 210, 200x60)
                        if x >= 84 && x <= 284 && y >= 210 && y <= 270 {
                            esp_println::println!("[MAP] Auto Farm selected");
                            // Spawn enemy from current map
                            let enemy_ids = MapHelper::enemies(map_id);
                            if !enemy_ids.is_empty() && game_state.hero.sp >= 20 {
                                // Pick random enemy from map
                                let rng_value = (x.wrapping_add(y)) as u8;
                                let enemy_index = (rng_value as usize) % enemy_ids.len();
                                let enemy_id = enemy_ids[enemy_index];

                                if let Some(enemy) = Enemy::from_id(enemy_id) {
                                    esp_println::println!(
                                        "[MAP] Starting farm with {} from map",
                                        enemy.name
                                    );
                                    game_state.start_farming(enemy);
                                }
                            } else if game_state.hero.sp < 20 {
                                esp_println::println!("[MAP] Not enough SP for farming");
                                game_state.save_status_msg = Some("Not enough SP! (need 20)");
                                game_state.save_status_timeout = game_state.last_update_ms + 2000;
                                game_state.needs_redraw = true;
                            }
                        }
                        // Check Battle button (84, 280, 200x60)
                        else if x >= 84 && x <= 284 && y >= 280 && y <= 340 {
                            esp_println::println!("[MAP] Battle selected");

                            // Check HP first
                            if game_state.hero.hp == 0 {
                                esp_println::println!("[MAP] No HP! Cannot battle");
                                game_state.save_status_msg = Some("No HP! Rest to recover");
                                game_state.save_status_timeout = game_state.last_update_ms + 2000;
                                game_state.needs_redraw = true;
                            } else if game_state.hero.sp < 30 {
                                esp_println::println!("[MAP] Not enough SP for battle");
                                game_state.save_status_msg = Some("Not enough SP! (need 30)");
                                game_state.save_status_timeout = game_state.last_update_ms + 2000;
                                game_state.needs_redraw = true;
                            } else {
                                // Spawn enemy from current map
                                let enemy_ids = MapHelper::enemies(map_id);
                                if !enemy_ids.is_empty() {
                                    // Pick random enemy from map
                                    let rng_value = (x.wrapping_add(y)) as u8;
                                    let enemy_index = (rng_value as usize) % enemy_ids.len();
                                    let enemy_id = enemy_ids[enemy_index];

                                    if let Some(enemy) = Enemy::from_id(enemy_id) {
                                        esp_println::println!(
                                            "[MAP] Starting battle with {} from map",
                                            enemy.name
                                        );
                                        game_state.start_battle(enemy);
                                    }
                                }
                            }
                        }
                    }
                }
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
    let fps_elapsed = game_state
        .last_update_ms
        .wrapping_sub(game_state.last_fps_update_ms);
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

    // Update rest progress (only redraw when HP or SP actually changes)
    if game_state.current_page == GamePage::Rest && game_state.rest_state == RestState::Resting {
        let old_sp = game_state.hero.sp;
        let old_hp = game_state.hero.hp;
        game_state.update_rest_progress(delta_ms);
        // Only redraw if HP or SP changed or state changed
        if game_state.hero.sp != old_sp
            || game_state.hero.hp != old_hp
            || game_state.rest_state != RestState::Resting
        {
            game_state.needs_redraw = true;
        }
    }

    // Update battle progress (spawn circles, check expiration, handle damage)
    if game_state.current_page == GamePage::Battle
        && game_state.battle_state == BattleState::Playing
    {
        let old_score = game_state.battle_score;
        let old_missed = game_state.battle_missed;
        let old_state = game_state.battle_state;

        game_state.update_battle(delta_ms);

        // Redraw if score/missed changed or state changed
        if game_state.battle_score != old_score
            || game_state.battle_missed != old_missed
            || game_state.battle_state != old_state
        {
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
            draw_overview_page(
                &mut display_res.display,
                &game_state.hero,
                battery_mv,
                battery_pct,
                fps,
                game_state.save_status_msg,
            )
            .ok();
        }
        GamePage::Farm => {
            draw_farm_page(
                &mut display_res.display,
                &game_state,
                battery_mv,
                battery_pct,
                fps,
            )
            .ok();
        }
        GamePage::Rest => {
            draw_rest_page(
                &mut display_res.display,
                &game_state,
                battery_mv,
                battery_pct,
                fps,
            )
            .ok();
        }
        GamePage::Battle => {
            draw_battle_page(
                &mut display_res.display,
                &game_state,
                battery_mv,
                battery_pct,
                fps,
            )
            .ok();
        }
        GamePage::Map => {
            draw_map_page(
                &mut display_res.display,
                &game_state,
                battery_mv,
                battery_pct,
                fps,
            )
            .ok();
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

        esp_println::println!(
            "[SAVE] Saving hero: Level {} {} with {} EXP and {} Zeny",
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
    if game_state.save_status_timeout > 0
        && game_state.last_update_ms >= game_state.save_status_timeout
    {
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
