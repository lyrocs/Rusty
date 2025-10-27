use bevy_ecs::prelude::*;
use ft3x68_rs::{TouchPoint, TouchState};

use crate::ecs::resources::{
    BatteryResource, ButtonResource, DisplayResource, RtcResource, SdCardResource, TouchResource,
};
use crate::tamagotchi::models::{
    BattleState, Enemy, FarmState, GamePage, GameState, MapHelper, RestState,
};
use crate::tamagotchi::ui::{
    draw_battle_page, draw_farm_page, draw_inventory, draw_jrpg_battle_page, draw_map_page,
    draw_menu, draw_overview_page, draw_rest_page, draw_settings_page,
};

const DEBOUNCE_THRESHOLD: u8 = 3;

/// System to handle button input for menu toggling
pub fn tamagotchi_button_system(
    mut button_res: NonSendMut<ButtonResource>,
    mut game_state: ResMut<GameState>,
) {
    // BOOT Button (GPIO0) - Active Low
    let boot_pressed = button_res.boot_button.is_low();

    // Debouncing logic for BOOT
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
            let new_page = match game_state.menu_selection {
                0 => GamePage::Overview,
                1 => GamePage::Rest,
                2 => GamePage::Map,
                // 3 is Save - handled in touch system, stays on current page
                _ => GamePage::Overview,
            };

            // Initialize rest state if going to Rest page
            if matches!(new_page, GamePage::Rest) {
                game_state.init_rest_state();
            }

            // Reset map monster animation when entering Map page
            if matches!(new_page, GamePage::Map) {
                game_state.map_monster_animation_frame = 0;
                game_state.map_monster_animation_last_update = game_state.last_update_ms;
            }

            game_state.current_page = new_page;
        } else {
            // Open menu
            game_state.current_page = GamePage::Menu;
        }
        game_state.needs_redraw = true; // Mark for redraw on page change
    }

    // Update last state for BOOT
    button_res.boot_last_state = button_res.boot_debounce_counter >= DEBOUNCE_THRESHOLD;

    // PWR Button (EXIO4 via TCA9554) - Active Low
    let pwr_pin_state = button_res.gpio_expander.read_pin(4).unwrap_or(false);
    let pwr_low = !pwr_pin_state; // Active low: pressed = false (LOW), released = true (HIGH)
    let pwr_pressed = pwr_low;

    // Debouncing logic for PWR
    if pwr_pressed {
        if button_res.pwr_debounce_counter < DEBOUNCE_THRESHOLD {
            button_res.pwr_debounce_counter += 1;
        }
    } else {
        button_res.pwr_debounce_counter = 0;
    }

    // Detect rising edge for PWR (button release after being pressed)
    if button_res.pwr_last_state && !pwr_pressed && button_res.pwr_debounce_counter == 0 {
        // Toggle screen on/off
        game_state.screen_on = !game_state.screen_on;
        game_state.needs_redraw = true;
    }

    // Update last state for PWR
    button_res.pwr_last_state = button_res.pwr_debounce_counter >= DEBOUNCE_THRESHOLD;
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
            // Menu item selection based on button position (2 columns x 3 rows)
            // Now 6 items - Farm and Battle removed (accessed via Map)
            // Button layout:
            // [Overview(0)]  [Rest(1)]      Row 0: y=110-180
            // [Map(2)]       [Inventory(3)] Row 1: y=190-260
            // [Settings(4)]  [Save(5)]      Row 2: y=270-340
            //
            // Col 0: x=24-174, Col 1: x=184-334

            // Check if touch is within button area
            if y >= 110 && y <= 340 {
                let mut clicked_button: Option<u8> = None;

                // Determine row (0, 1, or 2)
                let row = if y >= 110 && y <= 180 {
                    0
                } else if y >= 190 && y <= 260 {
                    1
                } else if y >= 270 && y <= 340 {
                    2
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
                if row < 3 && col < 2 {
                    let button_index = row * 2 + col;
                    if button_index < 6 {
                        // Now 6 buttons exist
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
                    if item_index == 5 {
                        // Save Game selected
                        game_state.save_requested = true;
                        game_state.current_page = GamePage::Overview; // Go back to overview after save
                    } else {
                        // Navigate to selected page
                        let new_page = match item_index {
                            0 => GamePage::Overview,
                            1 => GamePage::Rest,
                            2 => GamePage::Map,
                            3 => GamePage::Inventory,
                            4 => GamePage::Settings,
                            _ => GamePage::Overview,
                        };

                        // Initialize rest state if going to Rest page
                        if matches!(new_page, GamePage::Rest) {
                            game_state.init_rest_state();
                        }

                        // Reset map monster animation when entering Map page
                        if matches!(new_page, GamePage::Map) {
                            game_state.map_monster_animation_frame = 0;
                            game_state.map_monster_animation_last_update =
                                game_state.last_update_ms;
                        }

                        game_state.current_page = new_page;
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
                    game_state.current_page = GamePage::Map;
                }
                BattleState::Playing => {
                    // Record touch position for debug display
                    game_state.battle_last_touch_x = x as i32;
                    game_state.battle_last_touch_y = y as i32;
                    game_state.battle_last_touch_time = game_state.last_update_ms;

                    // Check if touch hit any circle
                    game_state.click_battle_circle(x as i32, y as i32);
                }
                BattleState::Victory | BattleState::Defeat => {
                    // Prevent accidental clicks - require 500ms delay after battle ends
                    let time_since_end = game_state
                        .last_update_ms
                        .saturating_sub(game_state.battle_end_time);
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
                            game_state.start_battle(enemy);
                        }
                    } else if game_state.hero.sp < 30 {
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

            // Update map monster idle animation (10 FPS = 100ms per frame)
            let time_since_last_frame = game_state
                .last_update_ms
                .saturating_sub(game_state.map_monster_animation_last_update);
            if time_since_last_frame >= 100 {
                game_state.map_monster_animation_frame += 1;
                game_state.map_monster_animation_last_update = game_state.last_update_ms;
                game_state.needs_redraw = true;
            }

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
                        // Check Auto Farm button (84, 280, 200x50)
                        if x >= 84 && x <= 284 && y >= 280 && y <= 330 {
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
                        // Check JRPG Battle button (84, 335, 200x50)
                        else if x >= 84 && x <= 284 && y >= 335 && y <= 385 {
                            esp_println::println!("[MAP] JRPG Battle selected");

                            // Check HP first
                            if game_state.hero.hp == 0 {
                                esp_println::println!("[MAP] No HP! Cannot battle");
                                game_state.save_status_msg = Some("No HP! Rest to recover");
                                game_state.save_status_timeout = game_state.last_update_ms + 2000;
                                game_state.needs_redraw = true;
                            } else if game_state.hero.sp < 10 {
                                esp_println::println!("[MAP] Not enough SP for battle");
                                game_state.save_status_msg = Some("Not enough SP! (need 10)");
                                game_state.save_status_timeout = game_state.last_update_ms + 2000;
                                game_state.needs_redraw = true;
                            } else {
                                // Spawn enemy from current map for JRPG battle
                                let enemy_ids = MapHelper::enemies(map_id);
                                if !enemy_ids.is_empty() {
                                    // Pick random enemy from map
                                    let rng_value = (x.wrapping_add(y)) as u8;
                                    let enemy_index = (rng_value as usize) % enemy_ids.len();
                                    let enemy_id = enemy_ids[enemy_index];

                                    if let Some(enemy) = Enemy::from_id(enemy_id) {
                                        esp_println::println!(
                                            "[MAP] Starting JRPG battle with {} from map",
                                            enemy.name
                                        );
                                        game_state.start_jrpg_battle(enemy);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        GamePage::Overview => {
            // Rest button: x=30-180, y=370-420
            if x >= 30 && x <= 180 && y >= 370 && y <= 420 {
                game_state.current_page = GamePage::Rest;
                game_state.init_rest_state();
                game_state.needs_redraw = true;
            }
            // Inventory button: x=188-338, y=370-420
            else if x >= 188 && x <= 338 && y >= 370 && y <= 420 {
                game_state.current_page = GamePage::Inventory;
                game_state.needs_redraw = true;
            }
        }
        GamePage::Inventory => {
            // Go back to menu on touch
            game_state.current_page = GamePage::Menu;
            game_state.needs_redraw = true;
        }
        GamePage::Settings => {
            // Brightness slider area: x=40-320, y=180-200 (horizontal bar)
            // Slider handle position based on brightness: x = 40 + (brightness * 280 / 255)

            if y >= 160 && y <= 220 && x >= 40 && x <= 320 {
                // Calculate new brightness from touch position
                // Left (x=40) should be 0% (dim), Right (x=320) should be 100% (bright)
                let slider_x = (x - 40).max(0).min(280);
                let new_brightness = ((slider_x as u32 * 255) / 280) as u8;

                if game_state.brightness != new_brightness {
                    game_state.brightness = new_brightness;
                    game_state.needs_redraw = true;
                }
            } else if y >= 350 {
                // Bottom area - go back to menu
                game_state.current_page = GamePage::Menu;
                game_state.needs_redraw = true;
            }
        }
        GamePage::JrpgBattle => {
            use crate::tamagotchi::models::{JrpgBattleState, JrpgBattleMenu};

            match game_state.jrpg_battle_state {
                JrpgBattleState::PlayerTurn => {
                    match game_state.jrpg_battle_menu {
                        JrpgBattleMenu::Main => {
                            // 3x2 grid button layout
                            // Button dimensions match UI: 110x50, spacing 12x10, start at (14, 320)
                            let button_width = 110;
                            let button_height = 50;
                            let spacing_x = 12;
                            let spacing_y = 10;
                            let start_x = 14;
                            let start_y = 320;

                            let mut clicked_button: Option<u8> = None;

                            // Check which button was clicked (5 buttons: Attack, Skill, Item, Defend, Run)
                            for i in 0..5 {
                                let row = i / 3;
                                let col = i % 3;
                                let btn_x = start_x + col as i32 * (button_width + spacing_x);
                                let btn_y = start_y + row as i32 * (button_height + spacing_y);

                                if x >= btn_x as u16
                                    && x <= (btn_x + button_width) as u16
                                    && y >= btn_y as u16
                                    && y <= (btn_y + button_height) as u16
                                {
                                    clicked_button = Some(i);
                                    break;
                                }
                            }

                            if let Some(btn) = clicked_button {
                                // Check if this is a selection (clicking on highlighted item)
                                if btn == game_state.jrpg_menu_selection {
                                    // Execute action
                                    match btn {
                                        0 => {
                                            // Attack
                                            game_state.jrpg_player_attack();
                                            game_state.jrpg_battle_state = JrpgBattleState::PlayerAction;
                                            game_state.jrpg_action_animation_timer = 1500;
                                        }
                                        1 => {
                                            // Skill submenu
                                            game_state.jrpg_battle_menu = JrpgBattleMenu::Skills;
                                        }
                                        2 => {
                                            // Item submenu
                                            game_state.jrpg_battle_menu = JrpgBattleMenu::Items;
                                        }
                                        3 => {
                                            // Defend
                                            game_state.jrpg_player_defend();
                                            game_state.jrpg_battle_state = JrpgBattleState::EnemyTurn;
                                            game_state.jrpg_action_animation_timer = 500;
                                        }
                                        4 => {
                                            // Run
                                            game_state.jrpg_battle_state = JrpgBattleState::Fleeing;
                                            game_state.jrpg_action_animation_timer = 1500;
                                            game_state.jrpg_try_run();
                                        }
                                        _ => {}
                                    }
                                } else {
                                    // Just update selection
                                    game_state.jrpg_menu_selection = btn;
                                }
                                game_state.needs_redraw = true;
                            }
                        }
                        JrpgBattleMenu::Skills | JrpgBattleMenu::Items => {
                            // Go back to main menu on any touch
                            game_state.jrpg_battle_menu = JrpgBattleMenu::Main;
                            game_state.needs_redraw = true;
                        }
                    }
                }
                JrpgBattleState::Victory | JrpgBattleState::Defeat | JrpgBattleState::Escaped => {
                    // Tap to exit battle
                    game_state.end_jrpg_battle();
                }
                _ => {
                    // During animations, ignore input
                }
            }
        }
    }
}

/// Helper function to update monster GIF animation
/// Uses global animation clock for synchronized updates - only sets needs_redraw when frame changes
fn update_monster_animation(game_state: &mut GameState, _delta_ms: u32, monster_name: &str) {
    use crate::tamagotchi::models::MonsterAnimation;
    use embedded_graphics::pixelcolor::Rgb888;
    use tinygif::Gif;

    let gif_data = game_state.monster_animation.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");
    let total_frames = gif.frames().count();

    // Use global animation clock for synchronized updates (100ms per frame)
    let elapsed_ms = game_state.gif_animation_clock_ms
        .wrapping_sub(game_state.monster_animation_started_ms);
    let frame_duration_ms = 100;
    let target_frame = ((elapsed_ms / frame_duration_ms) as usize) % total_frames;

    // Only update and redraw if frame actually changed
    if game_state.monster_animation.should_loop() {
        // Loop animations (Idle)
        if game_state.monster_animation_frame != target_frame {
            game_state.monster_animation_frame = target_frame;
            game_state.needs_redraw = true;
        }
    } else {
        // Play-once animations (Attacking, Dying)
        if game_state.monster_animation_frame < total_frames - 1 {
            if game_state.monster_animation_frame != target_frame {
                game_state.monster_animation_frame = target_frame.min(total_frames - 1);
                game_state.needs_redraw = true;
            }
        } else {
            // Animation finished - return to Idle if it was Attacking
            if game_state.monster_animation == MonsterAnimation::Attacking {
                game_state.monster_animation = MonsterAnimation::Idle;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                game_state.needs_redraw = true;
            }
        }
    }
}

/// Helper function to update hero GIF animation
/// Uses global animation clock for synchronized updates - only sets needs_redraw when frame changes
fn update_hero_animation(game_state: &mut GameState, _delta_ms: u32) {
    use crate::tamagotchi::models::HeroAnimation;
    use embedded_graphics::pixelcolor::Rgb888;
    use tinygif::Gif;

    let gif_data = game_state.hero_animation.gif_data();
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse hero GIF");
    let total_frames = gif.frames().count();

    // Use global animation clock for synchronized updates (100ms per frame)
    let elapsed_ms = game_state.gif_animation_clock_ms
        .wrapping_sub(game_state.hero_animation_started_ms);
    let frame_duration_ms = 100;
    let target_frame = ((elapsed_ms / frame_duration_ms) as usize) % total_frames;

    // Only update and redraw if frame actually changed
    if game_state.hero_animation.should_loop() {
        // Loop animations (Resting, Idle)
        if game_state.hero_animation_frame != target_frame {
            game_state.hero_animation_frame = target_frame;
            game_state.needs_redraw = true;
        }
    } else {
        // Play-once animations (Attacking, Attacked)
        if game_state.hero_animation_frame < total_frames - 1 {
            if game_state.hero_animation_frame != target_frame {
                game_state.hero_animation_frame = target_frame.min(total_frames - 1);
                game_state.needs_redraw = true;
            }
        } else {
            // Animation finished - return to Idle
            if game_state.hero_animation == HeroAnimation::Attacking
                || game_state.hero_animation == HeroAnimation::Attacked
            {
                game_state.hero_animation = HeroAnimation::Idle;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
                game_state.needs_redraw = true;
            }
        }
    }
}

/// Helper function to update monster attacked animation (24.gif)
/// Uses global animation clock for synchronized updates - only sets needs_redraw when frame changes
fn update_monster_attacked_animation(
    game_state: &mut GameState,
    _delta_ms: u32,
    monster_name: &str,
) {
    use crate::tamagotchi::models::{MonsterAttackedAnimation, get_monster_attacked_gif};
    use embedded_graphics::pixelcolor::Rgb888;
    use tinygif::Gif;

    if game_state.monster_attacked_animation == MonsterAttackedAnimation::Normal {
        return; // Not being attacked
    }

    let gif_data = get_monster_attacked_gif(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse attacked GIF");
    let total_frames = gif.frames().count();

    // Use global animation clock for synchronized updates (100ms per frame)
    let elapsed_ms = game_state.gif_animation_clock_ms
        .wrapping_sub(game_state.monster_attacked_started_ms);
    let frame_duration_ms = 100;
    let target_frame = ((elapsed_ms / frame_duration_ms) as usize) % total_frames;

    // Play once and return to Normal - only update and redraw if frame actually changed
    if game_state.monster_attacked_frame < total_frames - 1 {
        if game_state.monster_attacked_frame != target_frame {
            game_state.monster_attacked_frame = target_frame.min(total_frames - 1);
            game_state.needs_redraw = true;
        }
    } else {
        // Animation finished - return to Normal
        game_state.monster_attacked_animation = MonsterAttackedAnimation::Normal;
        game_state.monster_attacked_frame = 0;
        game_state.needs_redraw = true;
    }
}

/// Update hero and monster animations for battle based on current animation phase
fn update_battle_animations(game_state: &mut GameState, delta_ms: u32, monster_name: &str) {
    use crate::tamagotchi::models::{BattleAnimationPhase, HeroAnimation, MonsterAnimation};

    // Set animations based on current phase
    match game_state.battle_animation_phase {
        BattleAnimationPhase::BothIdle => {
            // Both on idle animation
            if game_state.hero_animation != HeroAnimation::Idle {
                game_state.hero_animation = HeroAnimation::Idle;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            if game_state.monster_animation != MonsterAnimation::Idle {
                game_state.monster_animation = MonsterAnimation::Idle;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            // Update both idle animations
            update_hero_animation(game_state, delta_ms);
            update_monster_animation(game_state, delta_ms, monster_name);
        }
        BattleAnimationPhase::MonsterAttacking => {
            // Monster attacks (16.gif), hero gets hit (52.gif)
            if game_state.monster_animation != MonsterAnimation::Attacking {
                game_state.monster_animation = MonsterAnimation::Attacking;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            if game_state.hero_animation != HeroAnimation::Attacked {
                game_state.hero_animation = HeroAnimation::Attacked;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            update_hero_animation(game_state, delta_ms);
            update_monster_animation(game_state, delta_ms, monster_name);
        }
        BattleAnimationPhase::HeroAttacking => {
            // Hero attacks (84.gif), monster gets hit (24.gif)
            if game_state.hero_animation != HeroAnimation::Attacking {
                game_state.hero_animation = HeroAnimation::Attacking;
                game_state.hero_animation_frame = 0;
                game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            // Use monster_attacked_animation for the hit animation (24.gif)
            use crate::tamagotchi::models::MonsterAttackedAnimation;
            if game_state.monster_attacked_animation != MonsterAttackedAnimation::Attacked {
                game_state.monster_attacked_animation = MonsterAttackedAnimation::Attacked;
                game_state.monster_attacked_frame = 0;
                game_state.monster_attacked_started_ms = game_state.gif_animation_clock_ms;
            }
            // Set monster to idle so it doesn't override the attacked animation
            if game_state.monster_animation != MonsterAnimation::Idle {
                game_state.monster_animation = MonsterAnimation::Idle;
                game_state.monster_animation_frame = 0;
                game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
            }
            update_hero_animation(game_state, delta_ms);
            update_monster_attacked_animation(game_state, delta_ms, monster_name);
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

    // Only update visual elements (FPS, animations) when screen is on
    if game_state.screen_on {
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

            // Only redraw for FPS updates on pages where FPS changes matter (not during active gameplay)
            // During battle, we redraw based on game events (circles, timer, etc), not FPS counter
            if game_state.current_page != GamePage::Battle
                || game_state.battle_state != BattleState::Playing
            {
                game_state.needs_redraw = true; // Redraw when FPS updates
            }
        }

        // Update global GIF animation clock every 100ms for synchronized animations
        // This ensures all GIF animations update at the same time, reducing redraws
        let gif_clock_elapsed = game_state
            .last_update_ms
            .wrapping_sub(game_state.gif_animation_last_update_ms);
        if gif_clock_elapsed >= 100 {
            game_state.gif_animation_clock_ms = game_state
                .gif_animation_clock_ms
                .wrapping_add(100);
            game_state.gif_animation_last_update_ms = game_state.last_update_ms;

            // Note: We don't set needs_redraw here - individual animation functions will do that
            // only if they actually change frames
        }
    }

    // Handle farm state transitions and animations
    if game_state.current_page == GamePage::Farm {
        match game_state.farm_state {
            FarmState::Idle => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Ensure animation is reset to Idle when on idle page
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Idle {
                        game_state.monster_animation = MonsterAnimation::Idle;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                }
            }
            FarmState::Fighting => {
                // Update farming progress (ALWAYS runs - game logic)
                let old_percent = (game_state.farm_progress * 100) / game_state.farm_duration_ms;
                game_state.update_farm_progress(delta_ms);
                let new_percent = (game_state.farm_progress * 100) / game_state.farm_duration_ms;
                // Only redraw if progress bar changes by at least 1% AND screen is on
                if new_percent != old_percent && game_state.screen_on {
                    game_state.needs_redraw = true;
                }

                // Only update animations when screen is on
                if game_state.screen_on {
                    use crate::tamagotchi::models::{
                        HeroAnimation, MonsterAnimation, MonsterAttackedAnimation,
                    };

                    // Ensure hero is in Idle animation during fighting
                    if game_state.hero_animation != HeroAnimation::Idle
                        && game_state.hero_animation != HeroAnimation::Attacking
                        && game_state.hero_animation != HeroAnimation::Attacked
                    {
                        game_state.hero_animation = HeroAnimation::Idle;
                        game_state.hero_animation_frame = 0;
                        game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }

                    // Hero attacks monster every 4 seconds (trigger both hero attacking + monster attacked)
                    let time_since_last_hero_attack = game_state
                        .last_update_ms
                        .saturating_sub(game_state.last_hero_attack_ms);
                    if time_since_last_hero_attack >= 4000
                        && game_state.hero_animation == HeroAnimation::Idle
                        && game_state.monster_attacked_animation == MonsterAttackedAnimation::Normal
                    {
                        // Hero attacks!
                        game_state.hero_animation = HeroAnimation::Attacking;
                        game_state.hero_animation_frame = 0;
                        game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.last_hero_attack_ms = game_state.last_update_ms;

                        // Monster gets attacked!
                        game_state.monster_attacked_animation = MonsterAttackedAnimation::Attacked;
                        game_state.monster_attacked_frame = 0;
                        game_state.monster_attacked_started_ms = game_state.gif_animation_clock_ms;

                        game_state.needs_redraw = true;
                    }

                    // Monster attacks hero every 6 seconds (trigger both monster attacking + hero attacked)
                    let time_since_last_monster_attack = game_state
                        .last_update_ms
                        .saturating_sub(game_state.last_attack_animation_ms);
                    if time_since_last_monster_attack >= 6000
                        && game_state.monster_animation == MonsterAnimation::Idle
                        && game_state.hero_animation == HeroAnimation::Idle
                    {
                        // Monster attacks!
                        game_state.monster_animation = MonsterAnimation::Attacking;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.last_attack_animation_ms = game_state.last_update_ms;

                        // Hero gets attacked!
                        game_state.hero_animation = HeroAnimation::Attacked;
                        game_state.hero_animation_frame = 0;
                        game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;

                        game_state.needs_redraw = true;
                    }

                    // Update all animations (get monster name from current enemy)
                    if let Some(enemy) = &game_state.current_enemy {
                        let monster_name = enemy.name;
                        update_monster_animation(&mut game_state, delta_ms, monster_name);
                        update_monster_attacked_animation(&mut game_state, delta_ms, monster_name);
                    }
                    update_hero_animation(&mut game_state, delta_ms);
                }
            }
            FarmState::Victory => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Set to dying animation when entering victory
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Dying {
                        game_state.monster_animation = MonsterAnimation::Dying;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                    // Animate dying GIF (get monster name from current enemy)
                    let monster_name = game_state.current_enemy.as_ref().map(|e| e.name);
                    if let Some(name) = monster_name {
                        update_monster_animation(&mut game_state, delta_ms, name);
                    }
                }
            }
            FarmState::Defeat => {
                // No animation for defeat state
            }
        }
    }

    // Update rest progress (only redraw when HP or SP actually changes)
    if game_state.current_page == GamePage::Rest && game_state.rest_state == RestState::Resting {
        let old_sp = game_state.hero.sp;
        let old_hp = game_state.hero.hp;
        game_state.update_rest_progress(delta_ms);
        // Only redraw if HP or SP changed or state changed AND screen is on
        if (game_state.hero.sp != old_sp
            || game_state.hero.hp != old_hp
            || game_state.rest_state != RestState::Resting)
            && game_state.screen_on
        {
            game_state.needs_redraw = true;
        }
    }

    // Update hero animation on Rest page (only when screen is on)
    if game_state.current_page == GamePage::Rest && game_state.screen_on {
        use crate::tamagotchi::models::HeroAnimation;

        // Ensure hero is in Resting animation
        if game_state.hero_animation != HeroAnimation::Resting {
            game_state.hero_animation = HeroAnimation::Resting;
            game_state.hero_animation_frame = 0;
            game_state.hero_animation_started_ms = game_state.gif_animation_clock_ms;
            game_state.needs_redraw = true;
        }

        // Update resting animation
        update_hero_animation(&mut game_state, delta_ms);
    }

    // Update battle progress (spawn circles, check expiration, handle damage)
    if game_state.current_page == GamePage::Battle {
        match game_state.battle_state {
            BattleState::Idle => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Ensure animation is reset to Idle when on idle state
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Idle {
                        game_state.monster_animation = MonsterAnimation::Idle;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                }
            }
            BattleState::Playing => {
                // Update battle mechanics
                let old_score = game_state.battle_score;
                let old_missed = game_state.battle_missed;
                let old_state = game_state.battle_state;
                let old_time_sec = (game_state.battle_duration - game_state.battle_elapsed) / 1000;

                game_state.update_battle(delta_ms);

                let new_time_sec = (game_state.battle_duration - game_state.battle_elapsed) / 1000;

                // Redraw if score/missed/timer/state changed AND screen is on
                if (game_state.battle_score != old_score
                    || game_state.battle_missed != old_missed
                    || game_state.battle_state != old_state
                    || new_time_sec != old_time_sec)
                    && game_state.screen_on
                {
                    game_state.needs_redraw = true;
                }

                // Only update animation phases when screen is on
                if game_state.screen_on {
                    // Battle animation phase cycling
                    // Sequence: BothIdle (2s) -> MonsterAttacking (1s) -> BothIdle (2s) -> HeroAttacking (1s) -> repeat
                    use crate::tamagotchi::models::BattleAnimationPhase;
                    let time_in_phase = game_state
                        .last_update_ms
                        .saturating_sub(game_state.battle_animation_phase_started_ms);

                    let phase_changed = match game_state.battle_animation_phase {
                        BattleAnimationPhase::BothIdle => {
                            if time_in_phase >= 2000 {
                                // Alternate between monster attacking and hero attacking
                                // Use frame count to alternate
                                if (game_state.battle_elapsed / 6000) % 2 == 0 {
                                    game_state.battle_animation_phase =
                                        BattleAnimationPhase::MonsterAttacking;
                                } else {
                                    game_state.battle_animation_phase =
                                        BattleAnimationPhase::HeroAttacking;
                                }
                                game_state.battle_animation_phase_started_ms =
                                    game_state.last_update_ms;
                                true
                            } else {
                                false
                            }
                        }
                        BattleAnimationPhase::MonsterAttacking
                        | BattleAnimationPhase::HeroAttacking => {
                            if time_in_phase >= 1000 {
                                game_state.battle_animation_phase = BattleAnimationPhase::BothIdle;
                                game_state.battle_animation_phase_started_ms =
                                    game_state.last_update_ms;
                                true
                            } else {
                                false
                            }
                        }
                    };

                    // Animation phases and updates disabled during battle for performance
                    // GIFs are not rendered during manual battle gameplay anyway
                    if phase_changed {
                        // Don't set needs_redraw for animation phase changes since we don't render GIFs
                        // game_state.needs_redraw = true;
                    }

                    // Don't update animations during battle - GIFs are not rendered for performance
                    // let monster_name = game_state.battle_enemy.as_ref().map(|e| e.name);
                    // if let Some(name) = monster_name {
                    //     update_battle_animations(&mut game_state, delta_ms, name);
                    // }
                }
            }
            BattleState::Victory => {
                // Only update animations when screen is on
                if game_state.screen_on {
                    // Set to dying animation when entering victory
                    use crate::tamagotchi::models::MonsterAnimation;
                    if game_state.monster_animation != MonsterAnimation::Dying {
                        game_state.monster_animation = MonsterAnimation::Dying;
                        game_state.monster_animation_frame = 0;
                        game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                        game_state.needs_redraw = true;
                    }
                    // Animate dying GIF (get monster name from battle enemy)
                    let monster_name = game_state.battle_enemy.as_ref().map(|e| e.name);
                    if let Some(name) = monster_name {
                        update_monster_animation(&mut game_state, delta_ms, name);
                    }
                }
            }
            BattleState::Defeat => {
                // No animation for defeat state, keep it idle or stopped
            }
        }
    }

    // Handle JRPG battle updates
    if game_state.current_page == GamePage::JrpgBattle {
        use crate::tamagotchi::models::JrpgBattleState;

        // Only update visual timers when screen is on
        if game_state.screen_on {
            // Update battle message timer
            if game_state.jrpg_battle_message_timer > 0 {
                game_state.jrpg_battle_message_timer = game_state.jrpg_battle_message_timer.saturating_sub(delta_ms);
                if game_state.jrpg_battle_message_timer == 0 {
                    game_state.jrpg_battle_message = None;
                    game_state.needs_redraw = true;
                }
            }

            // Update damage animation timer (floats up and fades out over 1 second)
            if game_state.jrpg_damage_animation_timer > 0 {
                game_state.jrpg_damage_animation_timer = game_state.jrpg_damage_animation_timer.saturating_sub(delta_ms);
                if game_state.jrpg_damage_animation_timer == 0 {
                    game_state.jrpg_damage_dealt = 0;
                }
                game_state.needs_redraw = true; // Always redraw while animating
            }
        }

        // Update action animation timer and progress states (ALWAYS runs - game logic)
        if game_state.jrpg_action_animation_timer > 0 {
            game_state.jrpg_action_animation_timer = game_state.jrpg_action_animation_timer.saturating_sub(delta_ms);

            if game_state.jrpg_action_animation_timer == 0 {
                // Animation finished, progress to next state
                match game_state.jrpg_battle_state {
                    JrpgBattleState::PlayerAction => {
                        // Check if enemy defeated
                        if let Some(enemy) = &game_state.jrpg_enemy_combatant {
                            if enemy.hp == 0 {
                                game_state.jrpg_battle_state = JrpgBattleState::Victory;
                                game_state.jrpg_battle_message = Some("Victory!");
                                game_state.jrpg_battle_message_timer = 0; // Don't auto-hide

                                // Set monster dying animation (only when screen is on)
                                if game_state.screen_on {
                                    use crate::tamagotchi::models::MonsterAnimation;
                                    game_state.monster_animation = MonsterAnimation::Dying;
                                    game_state.monster_animation_frame = 0;
                                    game_state.monster_animation_started_ms = game_state.gif_animation_clock_ms;
                                }
                            } else {
                                // Enemy still alive, enemy's turn
                                game_state.jrpg_battle_state = JrpgBattleState::EnemyTurn;
                                game_state.jrpg_action_animation_timer = 500; // Brief pause
                            }
                        }
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    JrpgBattleState::EnemyTurn => {
                        // Execute enemy action
                        game_state.jrpg_enemy_attack();
                        game_state.jrpg_battle_state = JrpgBattleState::EnemyAction;
                        game_state.jrpg_action_animation_timer = 1500;
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    JrpgBattleState::EnemyAction => {
                        // Check if hero defeated
                        if let Some(hero) = &game_state.jrpg_hero_combatant {
                            if hero.hp == 0 {
                                game_state.jrpg_battle_state = JrpgBattleState::Defeat;
                                game_state.jrpg_battle_message = Some("Defeat...");
                                game_state.jrpg_battle_message_timer = 0; // Don't auto-hide
                            } else {
                                // Hero still alive, back to player turn
                                game_state.jrpg_battle_state = JrpgBattleState::PlayerTurn;
                                game_state.jrpg_battle_message = None;
                            }
                        }
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    JrpgBattleState::Fleeing => {
                        // Run attempt finished, check if it was successful (already handled in jrpg_try_run)
                        if game_state.jrpg_battle_state != JrpgBattleState::Escaped {
                            // Failed to escape, enemy's turn
                            game_state.jrpg_battle_state = JrpgBattleState::EnemyTurn;
                            game_state.jrpg_action_animation_timer = 500;
                        }
                        if game_state.screen_on {
                            game_state.needs_redraw = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Only update GIF animations when screen is on
        if game_state.screen_on {
            // Update GIF animations during JRPG battle
            update_hero_animation(&mut game_state, delta_ms);

            // Save enemy name to avoid borrow checker issues
            let enemy_name = game_state.jrpg_enemy_combatant.as_ref().map(|e| e.name);
            if let Some(name) = enemy_name {
                update_monster_animation(&mut game_state, delta_ms, name);

                // Update attacked animation if active
                if game_state.monster_attacked_animation != crate::tamagotchi::models::MonsterAttackedAnimation::Normal {
                    update_monster_attacked_animation(&mut game_state, delta_ms, name);
                }
            }
        }
    }
}

/// System to render the current page
pub fn tamagotchi_render_system(
    mut display_res: NonSendMut<DisplayResource>,
    mut game_state: ResMut<GameState>,
    battery_res: Res<BatteryResource>,
) {
    // Handle screen on/off state changes
    static mut LAST_SCREEN_STATE: bool = true;
    let screen_state_changed = unsafe {
        let changed = LAST_SCREEN_STATE != game_state.screen_on;
        LAST_SCREEN_STATE = game_state.screen_on;
        changed
    };

    if screen_state_changed {
        if game_state.screen_on {
            // Turn display on
            display_res.display.display_on().ok();
        } else {
            // Turn display off
            display_res.display.display_off().ok();
            game_state.needs_redraw = false;
            return;
        }
    }

    // Only render if something changed
    if !game_state.needs_redraw {
        // Don't log skipped frames - too noisy
        return;
    }

    // Skip rendering if screen is off
    if !game_state.screen_on {
        return;
    }

    // Save the redraw state before clearing it
    let should_full_redraw = game_state.needs_redraw;

    // Clear the dirty flag IMMEDIATELY to prevent multiple renders for the same change
    game_state.needs_redraw = false;

    // Get battery info
    let battery_mv = battery_res.voltage_mv;
    let battery_pct = battery_res.percent;
    let fps = game_state.fps;

    // Draw the current page
    match game_state.current_page {
        GamePage::Overview => {
            draw_overview_page(
                &mut display_res.display,
                &game_state,
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
                should_full_redraw,
            )
            .ok();
        }
        GamePage::Map => {
            draw_map_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::Menu => {
            // Draw the previous page first, then overlay menu
            // For simplicity, we'll just draw menu on a dark background
            draw_menu(&mut display_res.display, &game_state).ok();
        }
        GamePage::Inventory => {
            draw_inventory(&mut display_res.display, &game_state).ok();
        }
        GamePage::Settings => {
            draw_settings_page(
                &mut display_res.display,
                &game_state,
                battery_mv,
                battery_pct,
                fps,
            )
            .ok();
        }
        GamePage::JrpgBattle => {
            draw_jrpg_battle_page(&mut display_res.display, &game_state).ok();
        }
    }

    // Apply brightness setting directly
    // Slider 0% (brightness=0) = dim, Slider 100% (brightness=255) = bright
    let brightness_value = game_state.brightness as u16;
    display_res.display.set_brightness(brightness_value).ok();

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

        esp_println::println!(
            "[SAVE] Saving hero: Level {} {} with {} EXP and {} Zeny, {} items",
            game_state.hero.level,
            game_state.hero.job,
            game_state.hero.exp,
            game_state.hero.zeny,
            game_state.hero.inventory.len()
        );

        // Try to write hero data to SD card
        let hero_result = save_hero_to_sd(&mut sd_card_res, save_data.as_str());

        // Try to write inventory to SD card
        let inventory_data = game_state.hero.inventory_to_save_string();
        let inventory_result = save_inventory_to_sd(&mut sd_card_res, inventory_data.as_str());

        // Check results
        match (hero_result, inventory_result) {
            (Ok(_), Ok(_)) => {
                esp_println::println!("[SAVE] Successfully saved hero and inventory to SD card");
                game_state.save_status_msg = Some("Saved to SD!");
            }
            (Ok(_), Err(e)) => {
                esp_println::println!("[SAVE] Hero saved but inventory failed: {:?}", e);
                game_state.save_status_msg = Some("Save partial!");
            }
            (Err(e), _) => {
                esp_println::println!("[SAVE] Error saving hero to SD: {:?}", e);
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

/// Helper function to save inventory data to SD card
fn save_inventory_to_sd(
    sd_card_res: &mut SdCardResource,
    inventory_data: &str,
) -> Result<(), embedded_sdmmc::Error<embedded_sdmmc::SdCardError>> {
    use embedded_sdmmc::{Mode, VolumeIdx};

    // Open volume
    let mut volume = sd_card_res.volume_mgr.open_volume(VolumeIdx(0))?;

    // Open root directory
    let mut root_dir = volume.open_root_dir()?;

    // Create or truncate inventory file
    let mut file = root_dir.open_file_in_dir("ITEMS.SAV", Mode::ReadWriteCreateOrTruncate)?;

    // Write inventory data
    file.write(inventory_data.as_bytes())?;

    Ok(())
}
