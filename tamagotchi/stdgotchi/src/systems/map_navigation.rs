//! Map navigation system
//!
//! Handles map navigation, location selection, and transitions to battle.

use bevy_ecs::prelude::*;
use embedded_graphics::pixelcolor::Rgb888;
use esp_idf_svc::hal::i2c::I2cDriver;

use crate::ecs::resources::{AppMode, AppState, GameManager, TouchResource};
use crate::game::EnemyType;
use crate::ui::pages::battle::EnemyType as BattleEnemyType;
use crate::ui::pages::BattlePage;

/// System to handle map navigation
pub fn map_navigation_system(
    mut app_state: ResMut<AppState>,
    mut touch_res: NonSendMut<TouchResource>,
    mut i2c: NonSendMut<I2cDriver>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Map mode
    if app_state.current_mode != AppMode::Map {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check for touch (taps)
    if let Ok(count) = touch_res.touch.finger_number(&mut i2c) {
        if count > 0 && !touch_res.last_touch_active {
            // New touch detected
            if let Ok(touches) = touch_res.touch.get_touches(&mut i2c) {
                if let Some(point) = touches.first() {
                    let x = point.x as i32;
                    let y = point.y as i32;
                    log::info!("Touch at ({}, {})", x, y);

                // Handle touch on map page
                if let Some(selected_location_id) = game_manager.map_page.handle_touch(x, y) {
                    // Check if selected location is a field (battle zone)
                    // Clone location data to avoid borrow conflicts
                    let location = game_manager.map_page.world_map().get_location(&selected_location_id).cloned();

                    if let Some(location) = location {
                        if location.is_field() {
                            // Field selected - enter battle mode
                            log::info!("Entering battle at: {}", location.name);
                            game_manager.selected_field_id = Some(selected_location_id.clone());

                            // Create battle page with monsters from this field
                            if let Some(monsters) = location.monsters() {
                                if !monsters.is_empty() {
                                    // Pick a random monster from the field
                                    let monster_index = rand::random::<usize>() % monsters.len();
                                    let monster_type = monsters[monster_index];

                                    // Convert game EnemyType to battle EnemyType
                                    let battle_enemy_type = match monster_type {
                                        EnemyType::Hornet => BattleEnemyType::Hornet,
                                        EnemyType::Poring => BattleEnemyType::Poring,
                                        EnemyType::Fabre => BattleEnemyType::Fabre,
                                        EnemyType::Lunatic => {
                                            // For now, use Poring as placeholder for Lunatic
                                            log::warn!("Lunatic not implemented in battle, using Poring");
                                            BattleEnemyType::Poring
                                        }
                                    };

                                    // Create battle page with background
                                    let battle_background = include_bytes!("../../assets/images/ui/battle.gif");
                                    let mut battle_page = match BattlePage::new_with_background(battle_background, (0, 0)) {
                                        Ok(page) => page,
                                        Err(e) => {
                                            log::error!("Failed to load battle background: {:?}", e);
                                            log::info!("Falling back to solid color background");
                                            BattlePage::new(Rgb888::new(20, 60, 20))
                                        }
                                    };

                                    // Add hero (using novice animations)
                                    let hero_idle = include_bytes!("../../assets/images/novice/32.gif");
                                    let hero_attack = include_bytes!("../../assets/images/novice/80.gif");
                                    let hero_attacked = include_bytes!("../../assets/images/novice/48.gif");
                                    battle_page
                                        .add_hero(hero_idle, hero_attack, hero_attacked, (175, 170))
                                        .ok();

                                    // Add enemy
                                    battle_page
                                        .add_enemy(battle_enemy_type, (75, 170))
                                        .ok();

                                    // Add all monsters from this field to the respawn pool
                                    for monster_type in monsters {
                                        let battle_monster_type = match monster_type {
                                            EnemyType::Hornet => BattleEnemyType::Hornet,
                                            EnemyType::Poring => BattleEnemyType::Poring,
                                            EnemyType::Fabre => BattleEnemyType::Fabre,
                                            EnemyType::Lunatic => {
                                                log::warn!("Lunatic not implemented in battle pool");
                                                BattleEnemyType::Poring
                                            }
                                        };
                                        battle_page.add_enemy_type_to_pool(battle_monster_type);
                                    }

                                    game_manager.battle_page = Some(battle_page);

                                    // Switch to battle mode
                                    app_state.current_mode = AppMode::Battle;
                                    app_state.needs_redraw = true;
                                }
                            }
                        } else {
                            // City selected - travel there
                            if let Err(e) = game_manager.map_page.travel_to(&selected_location_id) {
                                log::error!("Failed to travel: {}", e);
                            } else {
                                app_state.needs_redraw = true;
                            }
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

/// System to handle hero overview interactions
pub fn hero_overview_system(
    mut app_state: ResMut<AppState>,
    mut touch_res: NonSendMut<TouchResource>,
    mut i2c: NonSendMut<I2cDriver>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in HeroOverview mode
    if app_state.current_mode != AppMode::HeroOverview {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Check for touch (button taps)
    if let Ok(count) = touch_res.touch.finger_number(&mut i2c) {
        if count > 0 && !touch_res.last_touch_active {
            // New touch detected
            if let Ok(touches) = touch_res.touch.get_touches(&mut i2c) {
                if let Some(point) = touches.first() {
                    let x = point.x as i32;
                    let y = point.y as i32;

                    // Handle touch on hero overview page
                    if game_manager.handle_hero_overview_touch(x, y) {
                        app_state.needs_redraw = true;
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
