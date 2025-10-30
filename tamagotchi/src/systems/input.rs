/// Input handling systems
///
/// Button and touch input processing for game interaction.

use bevy_ecs::prelude::*;
use core::fmt::Write;
use ft3x68_rs::{TouchPoint, TouchState};
use heapless::String;

use crate::ecs::resources::{ButtonResource, TouchResource};
use crate::core::GameState;
use crate::combat::{BattleState, Enemy};
use crate::hero::EquipmentSlot;
use crate::tamagotchi::models::{FarmState, GamePage, MapHelper, RestState};
use crate::quest::system as quest_system;

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
                            3 => GamePage::Quests,
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
            handle_map_touch(game_state, x, y);
        }
        GamePage::Overview => {
            // Row 1: Rest and Stats
            // Rest button: x=14-179, y=350-395
            if x >= 14 && x <= 179 && y >= 350 && y <= 395 {
                game_state.current_page = GamePage::Rest;
                game_state.init_rest_state();
                game_state.needs_redraw = true;
            }
            // Stats button: x=189-354, y=350-395
            else if x >= 189 && x <= 354 && y >= 350 && y <= 395 {
                game_state.current_page = GamePage::Stats;
                game_state.needs_redraw = true;
            }
            // Row 2: Equipment and Items
            // Equipment button: x=14-179, y=403-448
            else if x >= 14 && x <= 179 && y >= 403 && y <= 448 {
                game_state.current_page = GamePage::Equipment;
                game_state.needs_redraw = true;
            }
            // Quests button: x=189-354, y=403-448
            else if x >= 189 && x <= 354 && y >= 403 && y <= 448 {
                game_state.current_page = GamePage::Quests;
                game_state.needs_redraw = true;
            }
        }
        GamePage::Inventory => {
            // Go back to menu on touch
            game_state.current_page = GamePage::Menu;
            game_state.needs_redraw = true;
        }
        GamePage::Stats => {
            handle_stats_touch(game_state, x, y);
        }
        GamePage::Equipment => {
            handle_equipment_touch(game_state, x, y);
        }
        GamePage::Quests => {
            handle_quests_touch(game_state, x, y);
        }
        GamePage::Settings => {
            handle_settings_touch(game_state, x, y);
        }
        GamePage::JrpgBattle => {
            handle_jrpg_battle_touch(game_state, x, y);
        }
        GamePage::Crafting => {
            handle_crafting_touch(game_state, x, y);
        }
    }
}

/// Handle touch input on Map page
fn handle_map_touch(game_state: &mut GameState, x: u16, y: u16) {
    // Handle equipment selection menu if open (intercepts all other touches)
    if game_state.equipment_selection_open {
        handle_equipment_selection_touch(game_state, x, y);
        return;
    }

    // Handle refine popup if open (intercepts all other touches)
    if game_state.refine_popup_open {
        handle_refine_popup_touch(game_state, x, y);
        return;
    }

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
        handle_location_actions(game_state, x, y, location_type, map_id);
    }
}

/// Handle equipment selection menu touches
fn handle_equipment_selection_touch(game_state: &mut GameState, x: u16, y: u16) {
    // Fullscreen equipment selection menu
    // Equipment slots: x=20, width=328, height=50, starting at y=55 with 55px spacing
    // Cancel button: x=110, y=390, width=148, height=36

    let start_y = 55;
    let item_height = 55;
    let slots = [
        (start_y, EquipmentSlot::Weapon, "Weapon"),
        (start_y + item_height, EquipmentSlot::Armor, "Armor"),
        (start_y + item_height * 2, EquipmentSlot::Shoes, "Shoes"),
        (start_y + item_height * 3, EquipmentSlot::Garment, "Garment"),
        (start_y + item_height * 4, EquipmentSlot::Accessory1, "Accessory 1"),
        (start_y + item_height * 5, EquipmentSlot::Accessory2, "Accessory 2"),
    ];

    // Check equipment slot buttons
    for (btn_y, slot, slot_name) in slots.iter() {
        if x >= 20 && x <= 348 && y >= *btn_y as u16 && y <= (*btn_y + 50) as u16 {
            // Check if equipment exists in this slot
            if game_state.hero.get_equipment(*slot).is_some() {
                esp_println::println!("[REFINERY] Selected {} for refinement", slot_name);
                // Close equipment selection and open refine popup
                game_state.equipment_selection_open = false;
                game_state.refine_popup_open = true;
                game_state.refine_slot = Some(*slot);
                game_state.refine_result_message = None;
                game_state.needs_redraw = true;
            } else {
                esp_println::println!("[REFINERY] No equipment in {} slot", slot_name);
                // Could show a message here
            }
            return;
        }
    }

    // Check Cancel button
    if x >= 110 && x <= 258 && y >= 390 && y <= 426 {
        esp_println::println!("[REFINERY] Cancel equipment selection");
        game_state.equipment_selection_open = false;
        game_state.needs_redraw = true;
        return;
    }
}

/// Handle equipment page touches
fn handle_equipment_touch(game_state: &mut GameState, x: u16, y: u16) {
    // Check if equipment swap menu is open first
    if game_state.equipment_swap_menu_open {
        handle_equipment_swap_menu_touch(game_state, x, y);
        return;
    }

    // Check if card socket menu is open
    if game_state.card_socket_menu_open {
        handle_card_socket_menu_touch(game_state, x, y);
        return;
    }

    // Check if equipment info modal is open
    if game_state.equipment_info_open {
        handle_equipment_info_touch(game_state, x, y);
        return;
    }

    // Check if preset menu is open
    if game_state.preset_menu_open {
        handle_preset_menu_touch(game_state, x, y);
        return;
    }

    // Equipment slots - 2x3 grid
    // Left column x=20-170, Right column x=200-350
    // Row 1 y=70-160, Row 2 y=165-255, Row 3 y=260-350
    let left_x = 20;
    let right_x = 200;
    let start_y = 70;
    let row_spacing = 95;

    // Check equipment slot clicks - clicking anywhere opens equipment info
    let slots = [
        (left_x, start_y, EquipmentSlot::Weapon),
        (right_x, start_y, EquipmentSlot::Armor),
        (left_x, start_y + row_spacing, EquipmentSlot::Shoes),
        (right_x, start_y + row_spacing, EquipmentSlot::Garment),
        (left_x, start_y + (row_spacing * 2), EquipmentSlot::Accessory1),
        (right_x, start_y + (row_spacing * 2), EquipmentSlot::Accessory2),
    ];

    for (slot_x, slot_y, slot) in slots.iter() {
        // Check if equipment slot was clicked - open equipment info
        if x >= (*slot_x - 5) && x <= (*slot_x + 165) && y >= (*slot_y - 5) && y <= (*slot_y + 80) {
            esp_println::println!("[EQUIPMENT] Equipment slot clicked - opening info");
            game_state.equipment_info_slot = Some(*slot);
            game_state.equipment_info_open = true;
            game_state.needs_redraw = true;
            return;
        }
    }

    // Preset buttons: y=370-400, 3 buttons from x=20
    let preset_y = 370;
    for i in 0..3 {
        let btn_x = 20 + (i * 110);
        if x >= btn_x && x <= (btn_x + 100) && y >= preset_y && y <= (preset_y + 30) {
            esp_println::println!("[EQUIPMENT] Preset {} button clicked", i + 1);
            game_state.preset_selected_index = Some(i as u8);
            game_state.preset_menu_open = true;
            game_state.needs_redraw = true;
            return;
        }
    }

    // Back button: x=100-260, y=410-440
    if x >= 100 && x <= 260 && y >= 410 && y <= 440 {
        game_state.current_page = GamePage::Overview;
        game_state.needs_redraw = true;
    }
}

/// Handle card socket menu touches
fn handle_card_socket_menu_touch(game_state: &mut GameState, x: u16, y: u16) {
    use crate::hero::equipment::EquipmentSlot;

    if let Some(equipment_slot) = game_state.card_socket_slot {
        // Get equipment
        let equipment = match equipment_slot {
            EquipmentSlot::Weapon => &game_state.hero.equipped_weapon,
            EquipmentSlot::Armor => &game_state.hero.equipped_armor,
            EquipmentSlot::Shoes => &game_state.hero.equipped_shoes,
            EquipmentSlot::Garment => &game_state.hero.equipped_garment,
            EquipmentSlot::Accessory1 => &game_state.hero.equipped_accessory1,
            EquipmentSlot::Accessory2 => &game_state.hero.equipped_accessory2,
        };

        let slot_start_y = 150;

        // Check card slot actions (Remove/Socket buttons)
        for i in 0..(equipment.card_slots as usize) {
            let slot_y = slot_start_y + (i as i32 * 50);

            // Remove button: x=250-320, y=slot_y to slot_y+45
            if x >= 250 && x <= 320 && y >= slot_y as u16 && y <= (slot_y + 45) as u16 {
                if equipment.socketed_cards[i].is_some() {
                    // Remove card
                    match game_state.hero.remove_card(equipment_slot, i) {
                        Ok(card_id) => {
                            esp_println::println!("[CARD] Removed card {} from slot {}", card_id, i);
                            // TODO: Add card back to inventory
                            game_state.needs_redraw = true;
                        }
                        Err(e) => {
                            esp_println::println!("[CARD] Failed to remove card: {}", e);
                        }
                    }
                } else {
                    // Socket card (placeholder - need card selection)
                    esp_println::println!("[CARD] Socket button clicked - need card selection UI");
                    // TODO: Show card selection from inventory
                }
                return;
            }
        }

        // Add slot button (if showing)
        if equipment.card_slots < equipment.max_card_slots {
            let add_slot_y = slot_start_y + (equipment.card_slots as i32 * 50);
            if x >= 35 && x <= 333 && y >= add_slot_y as u16 && y <= (add_slot_y + 40) as u16 {
                esp_println::println!("[CARD] Add slot button clicked");
                // TODO: Implement add slot logic (check essences in inventory)
                // For now, just show message
                esp_println::println!("[CARD] Add slot not yet implemented - need essence check");
                return;
            }
        }

        // Close button: x=124-244, y=385-415
        if x >= 124 && x <= 244 && y >= 385 && y <= 415 {
            game_state.card_socket_menu_open = false;
            game_state.card_socket_slot = None;
            game_state.needs_redraw = true;
            return;
        }

        // Click outside to close
        if x < 20 || x > 348 || y < 80 || y > 420 {
            game_state.card_socket_menu_open = false;
            game_state.card_socket_slot = None;
            game_state.needs_redraw = true;
        }
    }
}

/// Handle preset menu touches
fn handle_preset_menu_touch(game_state: &mut GameState, x: u16, y: u16) {
    if let Some(preset_index) = game_state.preset_selected_index {
        let has_preset = game_state.hero.equipment_presets[preset_index as usize].is_some();

        // Save button: x=75-293, y=200-240
        if x >= 75 && x <= 293 && y >= 200 && y <= 240 {
            match game_state.hero.save_equipment_preset(preset_index) {
                Ok(_) => {
                    esp_println::println!("[PRESET] Successfully saved preset {}", preset_index + 1);
                }
                Err(e) => {
                    esp_println::println!("[PRESET] Failed to save: {}", e);
                }
            }
            game_state.preset_menu_open = false;
            game_state.preset_selected_index = None;
            game_state.needs_redraw = true;
            return;
        }

        // Load button: x=75-293, y=250-290 (only if preset exists)
        if has_preset && x >= 75 && x <= 293 && y >= 250 && y <= 290 {
            match game_state.hero.load_equipment_preset(preset_index) {
                Ok(_) => {
                    esp_println::println!("[PRESET] Successfully loaded preset {}", preset_index + 1);
                }
                Err(e) => {
                    esp_println::println!("[PRESET] Failed to load: {}", e);
                }
            }
            game_state.preset_menu_open = false;
            game_state.preset_selected_index = None;
            game_state.needs_redraw = true;
            return;
        }

        // Clear button: x=75-293, y=300-340 (only if preset exists)
        if has_preset && x >= 75 && x <= 293 && y >= 300 && y <= 340 {
            match game_state.hero.clear_equipment_preset(preset_index) {
                Ok(_) => {
                    esp_println::println!("[PRESET] Successfully cleared preset {}", preset_index + 1);
                }
                Err(e) => {
                    esp_println::println!("[PRESET] Failed to clear: {}", e);
                }
            }
            game_state.preset_menu_open = false;
            game_state.preset_selected_index = None;
            game_state.needs_redraw = true;
            return;
        }

        // Click outside menu to close
        if x < 50 || x > 318 || y < 150 || y > 350 {
            game_state.preset_menu_open = false;
            game_state.preset_selected_index = None;
            game_state.needs_redraw = true;
        }
    }
}

/// Handle equipment info modal touches
fn handle_equipment_info_touch(game_state: &mut GameState, x: u16, y: u16) {
    use crate::hero::equipment::EquipmentSlot;

    if let Some(slot) = game_state.equipment_info_slot {
        let equipment = game_state.hero.get_equipment(slot);
        let has_card_slots = equipment.is_some() && equipment.unwrap().card_slots > 0;

        // Switch button: x=20-170, y=350-385
        if x >= 20 && x <= 170 && y >= 350 && y <= 385 {
            esp_println::println!("[EQUIPMENT] Switch button clicked in info modal");
            // Open swap menu without closing info modal
            game_state.equipment_swap_slot = Some(slot);
            game_state.equipment_swap_menu_open = true;
            game_state.equipment_swap_scroll = 0;
            game_state.needs_redraw = true;
            return;
        }

        // Cards button: x=180-330, y=350-385 (only if has card slots)
        if has_card_slots && x >= 180 && x <= 330 && y >= 350 && y <= 385 {
            esp_println::println!("[EQUIPMENT] Cards button clicked in info modal");
            // Close info modal and open card socket menu
            game_state.equipment_info_open = false;
            game_state.card_socket_slot = Some(slot);
            game_state.card_socket_menu_open = true;
            game_state.needs_redraw = true;
            return;
        }

        // Close button: x=110-258, y=395-431
        if x >= 110 && x <= 258 && y >= 395 && y <= 431 {
            esp_println::println!("[EQUIPMENT] Close button clicked in info modal");
            game_state.equipment_info_open = false;
            game_state.equipment_info_slot = None;
            game_state.needs_redraw = true;
            return;
        }
    }
}

/// Handle equipment swap menu touches
fn handle_equipment_swap_menu_touch(game_state: &mut GameState, x: u16, y: u16) {
    use crate::hero::equipment::EquipmentSlot;

    if let Some(slot) = game_state.equipment_swap_slot {
        // Get equipment items from inventory that match this slot
        let slot_str = match slot {
            EquipmentSlot::Weapon => "Weapon",
            EquipmentSlot::Armor => "Armor",
            EquipmentSlot::Shoes => "Shoes",
            EquipmentSlot::Garment => "Garment",
            EquipmentSlot::Accessory1 | EquipmentSlot::Accessory2 => "Accessory",
        };

        // Collect equipment items from inventory
        let mut equipment_items: heapless::Vec<u16, 16> = heapless::Vec::new();
        for item in game_state.hero.inventory.iter() {
            if item.id >= 1000 && item.id < 6000 {
                if let Some(equip_data) = crate::data::get_equipment_data_by_id(item.id as u16) {
                    if equip_data.slot == slot_str {
                        equipment_items.push(item.id as u16).ok();
                    }
                }
            }
        }

        // Check equipment item clicks (y=60 + i*60, height=55)
        let start_y = 60;
        let item_height = 60;
        let scroll_offset = game_state.equipment_swap_scroll as usize;

        for (i, equip_id) in equipment_items.iter()
            .skip(scroll_offset)
            .take(5)
            .enumerate()
        {
            let btn_y = start_y + i as i32 * item_height;
            if x >= 20 && x <= 348 && y >= btn_y as u16 && y <= (btn_y + 55) as u16 {
                esp_println::println!("[EQUIPMENT] Selected equipment ID {} to swap", equip_id);
                // Swap equipment
                if let Err(e) = game_state.hero.swap_equipment(slot, *equip_id) {
                    esp_println::println!("[EQUIPMENT] Failed to swap: {}", e);
                }
                // Close both swap menu and info modal
                game_state.equipment_swap_menu_open = false;
                game_state.equipment_swap_slot = None;
                game_state.equipment_info_open = false;
                game_state.equipment_info_slot = None;
                game_state.needs_redraw = true;
                return;
            }
        }

        // Cancel button: x=110-258, y=380-416
        if x >= 110 && x <= 258 && y >= 380 && y <= 416 {
            esp_println::println!("[EQUIPMENT] Swap menu cancelled");
            game_state.equipment_swap_menu_open = false;
            game_state.equipment_swap_slot = None;
            game_state.needs_redraw = true;
            return;
        }

        // Scroll indicators (if applicable)
        // Scroll up: y < 55
        if y < 55 && game_state.equipment_swap_scroll > 0 {
            game_state.equipment_swap_scroll -= 1;
            game_state.needs_redraw = true;
            return;
        }

        // Scroll down: y > 365
        if y > 365 && equipment_items.len() > (scroll_offset + 5) {
            game_state.equipment_swap_scroll += 1;
            game_state.needs_redraw = true;
            return;
        }
    }
}

/// Handle refine popup touches
fn handle_refine_popup_touch(game_state: &mut GameState, x: u16, y: u16) {
    if let Some(slot) = game_state.refine_slot {
        // Check if there's a result message (showing success/failure)
        if game_state.refine_result_message.is_some() {
            // Only show Close button during result display
            // Close button: x=120, y=300, size 128x40
            if x >= 120 && x <= 248 && y >= 300 && y <= 340 {
                esp_println::println!("[REFINE] Close button clicked");
                game_state.refine_popup_open = false;
                game_state.refine_slot = None;
                game_state.refine_result_message = None;
                game_state.refine_result_timer = 0;
                game_state.needs_redraw = true;
            }
        } else {
            // Normal refine popup buttons
            // REFINE button: x=50, y=300, size 128x40
            if x >= 50 && x <= 178 && y >= 300 && y <= 340 {
                esp_println::println!("[REFINE] Attempting to refine {:?}", slot);

                // Use touch coords as RNG seed
                let rng_value = x.wrapping_add(y) as u8;

                match game_state.hero.refine_equipment(slot, rng_value) {
                    Ok((success, new_level)) => {
                        if success {
                            let mut msg = String::<64>::new();
                            write!(msg, "Success! Now +{}", new_level).ok();
                            game_state.refine_result_message = Some("Refinement Success!");
                            esp_println::println!("[REFINE] Success! New level: +{}", new_level);

                            // Update quest progress - equipment refined
                            quest_system::update_quest_progress(
                                game_state,
                                crate::quest::QuestAction::EquipmentRefined,
                            );
                        } else {
                            game_state.refine_result_message = Some("Refinement Failed!");
                            esp_println::println!("[REFINE] Failed! Level now: +{}", new_level);
                        }
                        game_state.refine_result_timer = game_state.last_update_ms;
                        game_state.needs_redraw = true;
                    }
                    Err(err) => {
                        esp_println::println!("[REFINE] Error: {}", err);
                        game_state.refine_result_message = Some(err);
                        game_state.refine_result_timer = game_state.last_update_ms;
                        game_state.needs_redraw = true;
                    }
                }
            }
            // Cancel button: x=208, y=300, size 128x40
            else if x >= 208 && x <= 336 && y >= 300 && y <= 340 {
                esp_println::println!("[REFINE] Cancel button clicked");
                game_state.refine_popup_open = false;
                game_state.refine_slot = None;
                game_state.refine_result_message = None;
                game_state.refine_result_timer = 0;
                game_state.needs_redraw = true;
            }
        }
    }
}

/// Handle location action buttons (city NPCs or field battles)
fn handle_location_actions(
    game_state: &mut GameState,
    x: u16,
    y: u16,
    location_type: crate::tamagotchi::models::LocationType,
    map_id: u32,
) {
    use crate::tamagotchi::models::LocationType;

    match location_type {
        LocationType::City => {
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
                        // Handle Refinery NPC
                        if *npc == "Refinery" {
                            // Open equipment selection menu
                            game_state.equipment_selection_open = true;
                            game_state.needs_redraw = true;
                        }
                        // Handle Blacksmith NPC
                        else if *npc == "Blacksmith" {
                            // Open crafting page
                            game_state.current_page = GamePage::Crafting;
                            game_state.crafting_scroll = 0;
                            game_state.crafting_filter = "All";
                            game_state.needs_redraw = true;
                            esp_println::println!("[MAP] Opening Blacksmith crafting");
                        }
                        // TODO: Implement other NPC interactions (Quest Giver, etc.)
                    }
                }
            }
        }
        LocationType::Field => {
            handle_field_actions(game_state, x, y, map_id);
        }
    }
}

/// Handle field action buttons (Auto Farm and JRPG Battle)
fn handle_field_actions(game_state: &mut GameState, x: u16, y: u16, map_id: u32) {
    // Check Auto Farm button (54, 295, 130x55)
    if x >= 54 && x <= 184 && y >= 295 && y <= 350 {
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
    // Check JRPG Battle button (194, 295, 130x55)
    else if x >= 194 && x <= 324 && y >= 295 && y <= 350 {
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

/// Handle Stats page touches
fn handle_stats_touch(game_state: &mut GameState, x: u16, y: u16) {
    // Handle reset confirmation modal if showing
    if game_state.show_reset_confirm {
        let modal_x = 40;
        let modal_y = 150;

        // Confirm button: modal_x+20 to modal_x+130, modal_y+120 to modal_y+160
        if x >= (modal_x + 20) as u16
            && x <= (modal_x + 130) as u16
            && y >= (modal_y + 120) as u16
            && y <= (modal_y + 160) as u16
        {
            // Confirm reset
            game_state.hero.reset_stats();
            game_state.show_reset_confirm = false;
            game_state.needs_redraw = true;
            esp_println::println!("[STATS] Stats reset confirmed");
            return;
        }

        // Cancel button: modal_x+150 to modal_x+260, modal_y+120 to modal_y+160
        if x >= (modal_x + 150) as u16
            && x <= (modal_x + 260) as u16
            && y >= (modal_y + 120) as u16
            && y <= (modal_y + 160) as u16
        {
            // Cancel reset
            game_state.show_reset_confirm = false;
            game_state.needs_redraw = true;
            esp_println::println!("[STATS] Stats reset cancelled");
            return;
        }

        // Click outside modal to cancel
        game_state.show_reset_confirm = false;
        game_state.needs_redraw = true;
        return;
    }

    // 6 stat increase buttons (2 columns x 3 rows): x=20-170 (left), x=190-340 (right)
    // STR button (top left): x=20-170, y=110-180
    if x >= 20 && x <= 170 && y >= 110 && y <= 180 {
        if game_state.hero.increase_stat("STR") {
            game_state.needs_redraw = true;
        }
    }
    // AGI button (middle left): x=20-170, y=185-255
    else if x >= 20 && x <= 170 && y >= 185 && y <= 255 {
        if game_state.hero.increase_stat("AGI") {
            game_state.needs_redraw = true;
        }
    }
    // VIT button (bottom left): x=20-170, y=260-330
    else if x >= 20 && x <= 170 && y >= 260 && y <= 330 {
        if game_state.hero.increase_stat("VIT") {
            game_state.needs_redraw = true;
        }
    }
    // INT button (top right): x=190-340, y=110-180
    else if x >= 190 && x <= 340 && y >= 110 && y <= 180 {
        if game_state.hero.increase_stat("INT") {
            game_state.needs_redraw = true;
        }
    }
    // DEX button (middle right): x=190-340, y=185-255
    else if x >= 190 && x <= 340 && y >= 185 && y <= 255 {
        if game_state.hero.increase_stat("DEX") {
            game_state.needs_redraw = true;
        }
    }
    // LUK button (bottom right): x=190-340, y=260-330
    else if x >= 190 && x <= 340 && y >= 260 && y <= 330 {
        if game_state.hero.increase_stat("LUK") {
            game_state.needs_redraw = true;
        }
    }
    // Reset button (left half of bottom): x=15-180, y=350-400
    else if x >= 15 && x <= 180 && y >= 350 && y <= 400 {
        game_state.show_reset_confirm = true;
        game_state.needs_redraw = true;
        esp_println::println!("[STATS] Reset confirmation requested");
    }
    // Back button (right half of bottom): x=195-360, y=350-400
    else if x >= 195 && x <= 360 && y >= 350 && y <= 400 {
        game_state.current_page = GamePage::Overview;
        game_state.needs_redraw = true;
    }
}

/// Handle Quests page touches
fn handle_quests_touch(game_state: &mut GameState, x: u16, y: u16) {
    // Check if we're in details view
    if game_state.selected_quest_id.is_some() {
        // Details view - handle Back and Claim buttons

        // Back button: x=20-170, y=360-420
        if x >= 20 && x <= 170 && y >= 360 && y <= 420 {
            game_state.selected_quest_id = None;
            game_state.needs_redraw = true;
            esp_println::println!("[QUEST] Back to quest list");
            return;
        }

        // Claim button: x=190-340, y=360-420
        if x >= 190 && x <= 340 && y >= 360 && y <= 420 {
            if let Some(quest_id) = game_state.selected_quest_id {
                // Check if quest is completed and not claimed
                if let Some(active_quest) = game_state.active_quests.iter().find(|q| q.quest_id == quest_id) {
                    if active_quest.completed && !active_quest.claimed {
                        esp_println::println!("[QUEST] Claiming quest ID: {}", quest_id);
                        quest_system::claim_quest_reward(game_state, quest_id);
                        game_state.selected_quest_id = None; // Return to list after claiming
                        game_state.needs_redraw = true;
                    }
                }
            }
            return;
        }
    } else {
        // List view - handle scrolling, back button, and quest card clicks

        // UP arrow button: x=15-125, y=365-420
        if x >= 15 && x <= 125 && y >= 365 && y <= 420 {
            if game_state.quest_page_scroll > 0 {
                game_state.quest_page_scroll -= 1;
                game_state.needs_redraw = true;
                esp_println::println!("[QUEST] Scrolled up to {}", game_state.quest_page_scroll);
            }
            return;
        }

        // Back button (center): x=135-245, y=365-420
        if x >= 135 && x <= 245 && y >= 365 && y <= 420 {
            game_state.current_page = GamePage::Overview;
            game_state.needs_redraw = true;
            return;
        }

        // DOWN arrow button: x=255-365, y=365-420
        if x >= 255 && x <= 365 && y >= 365 && y <= 420 {
            // Count total unclaimed quests
            let total_quests = game_state
                .active_quests
                .iter()
                .filter(|q| !q.claimed)
                .count();

            // Check if we can scroll down (more than 4 quests and not at bottom)
            if total_quests > 4 && (game_state.quest_page_scroll as usize + 4) < total_quests {
                game_state.quest_page_scroll += 1;
                game_state.needs_redraw = true;
                esp_println::println!("[QUEST] Scrolled down to {}", game_state.quest_page_scroll);
            }
            return;
        }

        // Check for quest card clicks to show details
        // Quest cards: x=10-358, y=60-140, 148-228, 236-316, 324-404 (height 80, spacing 8)

        // Filter and sort quests the same way as the UI does
        let mut sorted_quests: heapless::Vec<&crate::tamagotchi::models::ActiveQuest, 16> = game_state
            .active_quests
            .iter()
            .filter(|q| !q.claimed)
            .collect();

        // Sort by priority (lower priority value = higher priority)
        sorted_quests.sort_by(|a, b| {
            let a_data = crate::tamagotchi::quest_system::get_quest_data(a.quest_id);
            let b_data = crate::tamagotchi::quest_system::get_quest_data(b.quest_id);

            match (a_data, b_data) {
                (Some(a_quest), Some(b_quest)) => a_quest.priority.cmp(&b_quest.priority),
                (Some(_), None) => core::cmp::Ordering::Less,
                (None, Some(_)) => core::cmp::Ordering::Greater,
                (None, None) => core::cmp::Ordering::Equal,
            }
        });

        let start_index = game_state.quest_page_scroll as usize;
        let mut card_y = 60i32;

        for (card_index, active_quest) in sorted_quests.iter().enumerate() {
            if card_index < start_index {
                continue;
            }

            if card_index >= start_index + 4 {
                break;
            }

            // Check if clicking anywhere on the quest card
            if x >= 10 && x <= 358 && y >= card_y as u16 && y <= (card_y + 80) as u16 {
                game_state.selected_quest_id = Some(active_quest.quest_id);
                game_state.needs_redraw = true;
                esp_println::println!("[QUEST] Selected quest ID: {}", active_quest.quest_id);
                return;
            }

            card_y += 88; // card_height (80) + spacing (8)
        }
    }
}

/// Handle Settings page touches
fn handle_settings_touch(game_state: &mut GameState, x: u16, y: u16) {
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

/// Handle JRPG Battle page touches
fn handle_jrpg_battle_touch(game_state: &mut GameState, x: u16, y: u16) {
    use crate::combat::{JrpgBattleState};

    match game_state.jrpg_battle_state {
        JrpgBattleState::PlayerTurn => {
            handle_jrpg_action_buttons_touch(game_state, x, y);
        }
        JrpgBattleState::Victory | JrpgBattleState::Defeat => {
            // Tap to exit battle
            game_state.end_jrpg_battle();
        }
        _ => {
            // During animations, ignore input
        }
    }
}

/// Handle JRPG action button touches (Attack + 3 Skills)
fn handle_jrpg_action_buttons_touch(game_state: &mut GameState, x: u16, y: u16) {
    use crate::combat::JrpgBattleState;

    // Get number of available skills
    let num_skills = if let Some(hero) = &game_state.jrpg_hero_combatant {
        hero.available_skills.len()
    } else {
        return;
    };

    // Button layout: 1 row with Attack + 3 Skills (4 buttons total)
    // Button dimensions: width varies, height 60, spacing 12
    let button_height = 60;
    let spacing_x = 12;
    let start_x = 14;
    let start_y = 360;

    // Attack button is wider (110px), skill buttons are narrower (66px each)
    let attack_width = 110;
    let skill_width = 66;

    let mut clicked_button: Option<u8> = None;

    // Check Attack button (button 0)
    if x >= start_x as u16
        && x <= (start_x + attack_width) as u16
        && y >= start_y as u16
        && y <= (start_y + button_height) as u16
    {
        clicked_button = Some(0);
    }

    // Check skill buttons (buttons 1-3)
    for i in 0..num_skills.min(3) {
        let btn_x = start_x + attack_width + spacing_x + i as i32 * (skill_width + spacing_x);
        if x >= btn_x as u16
            && x <= (btn_x + skill_width) as u16
            && y >= start_y as u16
            && y <= (start_y + button_height) as u16
        {
            clicked_button = Some((i + 1) as u8);
            break;
        }
    }

    if let Some(btn) = clicked_button {
        // Execute action directly (no selection step)
        match btn {
            0 => {
                // Attack
                game_state.jrpg_player_attack();
                game_state.jrpg_battle_state = JrpgBattleState::PlayerAction;
                game_state.jrpg_action_animation_timer = 1500;
            }
            1..=3 => {
                // Skill (button 1 = skill 0, button 2 = skill 1, etc.)
                let skill_index = (btn - 1) as usize;

                // Check if hero has enough SP
                if let Some(h) = &game_state.jrpg_hero_combatant {
                    if skill_index < h.available_skills.len() {
                        let skill = h.available_skills[skill_index];
                        if h.sp >= skill.sp_cost {
                            // Use skill
                            game_state.jrpg_player_use_skill(skill_index);
                            game_state.jrpg_battle_state = JrpgBattleState::PlayerAction;
                            game_state.jrpg_action_animation_timer = 1500;
                        }
                        // Note: if not enough SP, jrpg_player_use_skill shows error message
                    }
                }
            }
            _ => {}
        }
        game_state.needs_redraw = true;
    }
}

/// Handle touch input on Crafting page
fn handle_crafting_touch(game_state: &mut GameState, x: u16, y: u16) {
    use crate::hero::inventory::InventoryExt;

    // Clear craft result message on any touch (if timeout expired)
    if game_state.craft_result_timer > 0 {
        if game_state.last_update_ms >= game_state.craft_result_timer {
            game_state.craft_result_message = None;
            game_state.craft_result_timer = 0;
        }
        // If message is still showing, any touch clears it
        game_state.craft_result_message = None;
        game_state.craft_result_timer = 0;
        game_state.needs_redraw = true;
        return;
    }

    // Back button: x=100-260, y=410-440
    if x >= 100 && x <= 260 && y >= 410 && y <= 440 {
        game_state.current_page = GamePage::Map;
        game_state.needs_redraw = true;
        return;
    }

    // Filter buttons: y=70-98
    if y >= 70 && y <= 98 {
        let filters = ["All", "Weapon", "Armor", "Shoes", "Garment", "Accessory"];
        let btn_width = 58;
        let start_x = 5;

        for (i, filter) in filters.iter().enumerate() {
            let btn_x = start_x + (i as i32 * (btn_width + 3));
            if x >= btn_x as u16 && x <= (btn_x + btn_width) as u16 {
                game_state.crafting_filter = filter;
                game_state.crafting_scroll = 0; // Reset scroll when changing filter
                game_state.needs_redraw = true;
                return;
            }
        }
    }

    // Scroll up (if showing "^ More"): y=100-120
    if y >= 100 && y <= 120 && game_state.crafting_scroll > 0 {
        game_state.crafting_scroll = game_state.crafting_scroll.saturating_sub(1);
        game_state.needs_redraw = true;
        return;
    }

    // Get craftable equipment for current city
    let current_map = crate::data::get_map_data(game_state.current_location);
    let city_name = if let Some(map) = current_map {
        if crate::data::is_city(map.id) {
            map.name
        } else {
            "Prontera"
        }
    } else {
        "Prontera"
    };

    let craftable_items_all = crate::data::get_craftable_equipment_for_city(city_name);
    let craftable_items: heapless::Vec<&crate::data::EquipmentData, 16> =
        if game_state.crafting_filter == "All" {
            craftable_items_all
        } else {
            let mut filtered = heapless::Vec::new();
            for item in craftable_items_all.iter() {
                if item.slot == game_state.crafting_filter {
                    filtered.push(*item).ok();
                }
            }
            filtered
        };

    // Scroll down (if showing "v More"): y=390-410
    let max_visible = 4;
    if y >= 390
        && y <= 410
        && craftable_items.len() > (game_state.crafting_scroll as usize + max_visible)
    {
        game_state.crafting_scroll = game_state.crafting_scroll.saturating_add(1);
        game_state.needs_redraw = true;
        return;
    }

    // Recipe list: y=120-400
    if y >= 120 && y <= 400 {
        let start_y = 120;
        let item_height = 70;

        for (i, equip_data) in craftable_items
            .iter()
            .skip(game_state.crafting_scroll as usize)
            .take(max_visible)
            .enumerate()
        {
            let item_y = start_y + (i as i32 * item_height);
            if y >= item_y as u16 && y <= (item_y + 65) as u16 {
                // Check if this item can be crafted
                let hero = &game_state.hero;
                let can_craft = hero.level >= equip_data.level_req
                    && hero.zeny >= equip_data.craft_cost
                    && equip_data
                        .craft_materials
                        .as_ref()
                        .map(|materials| {
                            materials
                                .iter()
                                .all(|(mat_id, qty)| hero.inventory.has_item(*mat_id, *qty))
                        })
                        .unwrap_or(false);

                if can_craft {
                    // Craft the item
                    match game_state.hero.craft_equipment(equip_data.id) {
                        Ok(equipment) => {
                            // Successfully crafted - add to inventory
                            game_state.hero.add_item(equipment.id as u32, equipment.name, 1);
                            game_state.craft_result_message = Some("Crafted successfully!");
                            game_state.craft_result_timer = game_state.last_update_ms + 2000;
                            esp_println::println!("[CRAFT] Crafted {}", equipment.name);
                        }
                        Err(err) => {
                            game_state.craft_result_message = Some(err);
                            game_state.craft_result_timer = game_state.last_update_ms + 2000;
                            esp_println::println!("[CRAFT] Failed: {}", err);
                        }
                    }
                    game_state.needs_redraw = true;
                    return;
                }
            }
        }
    }

    game_state.needs_redraw = true;
}
