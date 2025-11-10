//! Crafting system
//!
//! Handles crafting page interactions and item crafting

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, InputEventChannel, SdCardWrapper};
use crate::input_thread::InputEvent;

/// System to handle crafting interactions
pub fn crafting_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
    mut sd_card_res: Option<NonSendMut<SdCardWrapper>>,
) {
    // Only process in Crafting mode
    if app_state.current_mode != AppMode::Crafting {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                log::info!("Crafting touch at ({}, {})", x, y);

                // Check if success dialog is open - handle touch to close it
                if game_manager.crafting_page.is_success_dialog_open() {
                    // Any touch closes the success dialog
                    log::info!("Closing craft success dialog");
                    game_manager.crafting_page.clear_craft_success();
                    app_state.needs_redraw = true;
                } else if let Some(action) = game_manager.crafting_page.handle_touch(x as i32, y as i32) {
                    use crate::ui::pages::CraftingAction;
                    match action {
                        CraftingAction::SelectRecipe(index) => {
                            log::info!("Selected recipe index: {}", index);
                            game_manager.crafting_page.select_recipe(index);
                            app_state.needs_redraw = true;
                        }
                        CraftingAction::Craft => {
                            // Attempt to craft the selected recipe
                            if let Some(index) = game_manager.crafting_page.selected_recipe_index() {
                                let location = game_manager.crafting_page.current_location.clone();

                                // Clone recipe to avoid borrow conflicts
                                let recipe_opt = game_manager.game_data.get_recipes_for_city(&location)
                                    .and_then(|recipes| recipes.get(index).cloned());

                                if let Some(recipe) = recipe_opt {
                                    log::info!("Attempting to craft: {}", recipe.result_item_name);

                                    // Try to craft the item
                                    match craft_item(game_manager, recipe.clone()) {
                                        Ok(unique_id) => {
                                            log::info!("Successfully crafted {} (unique_id: {:?})", recipe.result_item_name, unique_id);

                                            // Show success dialog
                                            game_manager.crafting_page.show_craft_success(
                                                recipe.result_item_name.clone(),
                                                recipe.result_item_id
                                            );

                                            // Auto-save after crafting
                                            if let Some(ref mut sd_card) = sd_card_res.as_mut() {
                                                game_manager.auto_save(&mut Some(sd_card), crate::sdcard::get_save_path());
                                            }

                                            app_state.needs_redraw = true;
                                        }
                                        Err(e) => {
                                            log::error!("Failed to craft: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        CraftingAction::Close => {
                            log::info!("Closing crafting page");
                            app_state.current_mode = AppMode::Map;
                            app_state.needs_redraw = true;
                        }
                    }
                }
            }
            InputEvent::BootPressed => {
                // Boot button closes crafting
                log::info!("Boot button pressed - returning to Map");
                app_state.current_mode = AppMode::Map;
                app_state.needs_redraw = true;
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// Craft an item from a recipe
fn craft_item(
    game_manager: &mut GameManager,
    recipe: crate::game::Recipe,
) -> Result<Option<u64>, String> {
    let hero = &mut game_manager.hero;

    // Check level requirement
    if hero.level < recipe.required_level {
        return Err(format!("Requires level {}", recipe.required_level));
    }

    // Check gold
    if hero.gold < recipe.gold_cost {
        return Err(format!("Not enough gold (need: {}, have: {})", recipe.gold_cost, hero.gold));
    }

    // Check materials
    for material in &recipe.materials {
        let has = hero.inventory.get_material_quantity(material.item_id);
        if has < material.quantity {
            return Err(format!(
                "Not enough {} (need: {}, have: {})",
                material.name,
                material.quantity,
                has
            ));
        }
    }

    // Deduct gold
    hero.gold -= recipe.gold_cost;

    // Consume materials
    for material in &recipe.materials {
        hero.inventory.remove_material(material.item_id, material.quantity)?;
    }

    // Get item data to check if it's equipment
    let item_data = game_manager.game_data.get_item(recipe.result_item_id)
        .ok_or_else(|| format!("Item {} not found in database", recipe.result_item_id))?;

    // Add the crafted item to inventory
    let unique_id = if item_data.slot.is_some() {
        // Equipment item - add as equipment
        let unique_id = hero.inventory.add_equipment(recipe.result_item_id)?;
        Some(unique_id)
    } else {
        // Material item - add as material (stackable)
        let item_database = game_manager.game_data.get_all_items();
        hero.inventory.add_material(recipe.result_item_id, 1, item_database)?;
        None
    };

    log::info!("Crafted: {} (ID: {})", recipe.result_item_name, recipe.result_item_id);
    Ok(unique_id)
}
