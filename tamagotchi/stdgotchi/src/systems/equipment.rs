//! Equipment system
//!
//! Handles equipment page interactions and equip/unequip actions.

use bevy_ecs::prelude::*;

use crate::ecs::resources::{AppMode, AppState, GameManager, InputEventChannel};
use crate::input_thread::InputEvent;

/// System to handle equipment interactions
pub fn equipment_system(
    mut app_state: ResMut<AppState>,
    input_channel: Res<InputEventChannel>,
    mut game_manager: Option<NonSendMut<GameManager>>,
) {
    // Only process in Equipment mode
    if app_state.current_mode != AppMode::Equipment {
        return;
    }

    let Some(ref mut game_manager) = game_manager else {
        return;
    };

    // Process all input events from the channel
    while let Ok(event) = input_channel.receiver.try_recv() {
        match event {
            InputEvent::Touch { x, y } => {
                log::info!("Equipment touch at ({}, {})", x, y);

                // Check if dialog is open
                if game_manager.equipment_page.is_dialog_open() {
                    // Handle touch on dialog
                    handle_dialog_touch(x as i32, y as i32, game_manager, &mut app_state);
                } else {
                    // Handle touch on main equipment page
                    if let Some(action) = game_manager.equipment_page.handle_touch(x as i32, y as i32) {
                        use crate::ui::pages::EquipmentAction;
                        match action {
                            EquipmentAction::SelectSlot(slot) => {
                                log::info!("Opening equipment dialog for {:?}", slot);
                                game_manager.equipment_page.open_dialog(slot);
                                app_state.needs_redraw = true;
                            }
                            EquipmentAction::SwitchToInventory => {
                                log::info!("Switching to Inventory");
                                app_state.current_mode = AppMode::Inventory;
                                app_state.needs_redraw = true;
                            }
                            EquipmentAction::Upgrade(unique_id) => {
                                log::info!("Attempting to upgrade equipment with unique_id: {}", unique_id);

                                // Attempt upgrade using helper function to avoid borrow conflicts
                                match upgrade_equipment_helper(game_manager, unique_id) {
                                    Ok(success) => {
                                        if success {
                                            log::info!("Upgrade succeeded!");
                                        } else {
                                            log::warn!("Upgrade failed!");
                                        }
                                        // Close dialog and refresh
                                        game_manager.equipment_page.close_dialog();
                                        app_state.needs_redraw = true;
                                    }
                                    Err(e) => {
                                        log::error!("Upgrade error: {}", e);
                                    }
                                }
                            }
                            EquipmentAction::Close => {
                                log::info!("Closing Equipment page");
                                app_state.current_mode = AppMode::Menu;
                                app_state.needs_redraw = true;
                            }
                        }
                    }
                }
            }
            InputEvent::BootPressed => {
                // Boot button closes dialog or returns to menu
                if game_manager.equipment_page.is_dialog_open() {
                    log::info!("Boot button pressed - closing dialog");
                    game_manager.equipment_page.close_dialog();
                    app_state.needs_redraw = true;
                } else {
                    log::info!("Boot button pressed - returning to Menu");
                    app_state.current_mode = AppMode::Menu;
                    app_state.needs_redraw = true;
                }
            }
            _ => {
                // Ignore other events
            }
        }
    }
}

/// Handle touch on dialog
fn handle_dialog_touch(
    x: i32,
    y: i32,
    game_manager: &mut GameManager,
    app_state: &mut AppState,
) {
    let Some(selected_slot) = game_manager.equipment_page.selected_slot() else {
        return;
    };

    // Dialog dimensions (must match draw_dialog)
    let dialog_x = 20;
    let dialog_y = 50;
    let dialog_width = 328;
    let dialog_height = 348;

    let list_start_y = dialog_y + 50;
    let item_height = 35;
    let visible_items = 7;

    // Get eligible items
    let eligible_items: Vec<_> = game_manager
        .hero
        .inventory
        .items()
        .iter()
        .filter(|item| {
            if let Some(item_data) = game_manager.game_data.get_item(item.item_id) {
                item_data.slot == Some(selected_slot)
            } else {
                false
            }
        })
        .collect();

    // Check if touch is on an item in the list
    for (index, item) in eligible_items
        .iter()
        .skip(game_manager.equipment_page.dialog_scroll_offset)
        .take(visible_items)
        .enumerate()
    {
        let y_pos = list_start_y + (index as i32 * item_height);
        let item_bounds = (dialog_x + 5, y_pos, dialog_width - 10, item_height as u32 - 3);

        if x >= item_bounds.0
            && x < item_bounds.0 + item_bounds.2 as i32
            && y >= item_bounds.1
            && y < item_bounds.1 + item_bounds.3 as i32
        {
            // Item touched - try to equip it
            if let Some(unique_id) = item.unique_id {
                log::info!("Trying to equip item {} (unique_id: {}) to {:?}", item.item_id, unique_id, selected_slot);
                match game_manager
                    .hero
                    .equip_item(unique_id, game_manager.game_data.get_all_items())
                {
                    Ok(()) => {
                        log::info!("Successfully equipped item");
                        game_manager.equipment_page.close_dialog();
                        app_state.needs_redraw = true;
                    }
                    Err(e) => {
                        log::error!("Failed to equip item: {}", e);
                    }
                }
            } else {
                log::error!("Item has no unique_id (not equipment?)");
            }
            return;
        }
    }

    // Check if touch is on unequip button
    if game_manager.hero.equipped_items.get_slot(selected_slot).is_some() {
        let unequip_y = dialog_y + dialog_height - 80;
        let unequip_bounds = (dialog_x + 10, unequip_y, 150, 35);

        if x >= unequip_bounds.0
            && x < unequip_bounds.0 + unequip_bounds.2 as i32
            && y >= unequip_bounds.1
            && y < unequip_bounds.1 + unequip_bounds.3 as i32
        {
            log::info!("Unequipping item from {:?}", selected_slot);
            match game_manager
                .hero
                .unequip_item(selected_slot, game_manager.game_data.get_all_items())
            {
                Ok(()) => {
                    log::info!("Successfully unequipped item");
                    game_manager.equipment_page.close_dialog();
                    app_state.needs_redraw = true;
                }
                Err(e) => {
                    log::error!("Failed to unequip item: {}", e);
                }
            }
            return;
        }
    }

    // Check if touch is on close button
    let close_y = dialog_y + dialog_height - 40;
    let close_bounds = (dialog_x + 10, close_y, 308, 35);

    if x >= close_bounds.0
        && x < close_bounds.0 + close_bounds.2 as i32
        && y >= close_bounds.1
        && y < close_bounds.1 + close_bounds.3 as i32
    {
        log::info!("Closing dialog");
        game_manager.equipment_page.close_dialog();
        app_state.needs_redraw = true;
    }
}

/// Helper function to upgrade equipment (avoids borrow checker issues)
fn upgrade_equipment_helper(
    game_manager: &mut GameManager,
    unique_id: u64,
) -> Result<bool, String> {
    // Get immutable references
    let item_database = game_manager.game_data.get_all_items();
    let upgrade_recipes = game_manager.game_data.get_upgrade_recipes();

    // Perform upgrade
    game_manager.hero.upgrade_equipment(unique_id, item_database, upgrade_recipes)
}
