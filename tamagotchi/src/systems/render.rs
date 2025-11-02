/// Render system - handles drawing the current page to the display

use bevy_ecs::prelude::*;

use crate::ecs::resources::{BatteryResource, DisplayResource};
use crate::core::GameState;
use crate::tamagotchi::models::GamePage;
use crate::tamagotchi::ui::{
    draw_battle_overview_page, draw_battle_page, draw_crafting_page, draw_debug_page,
    draw_equipment_page, draw_farm_page, draw_idle_farm_result_page, draw_inventory,
    draw_item_detail_page, draw_jrpg_battle_page, draw_map_page, draw_menu,
    draw_mvp_battle_page, draw_overview_page, draw_quests_page, draw_rest_page,
    draw_settings_page, draw_stats_page, draw_zelda_battle_page,
};

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
        GamePage::Quests => {
            draw_quests_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::Stats => {
            draw_stats_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::Equipment => {
            draw_equipment_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::Crafting => {
            draw_crafting_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::IdleFarmResult => {
            draw_idle_farm_result_page(&mut display_res.display, &game_state).ok();
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
        GamePage::ZeldaBattle => {
            draw_zelda_battle_page(
                &mut display_res.display,
                &game_state,
                battery_mv,
                battery_pct,
                fps,
                should_full_redraw,
            )
            .ok();
        }
        GamePage::MvpBattle => {
            draw_mvp_battle_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::ItemDetail => {
            draw_item_detail_page(&mut display_res.display, &game_state).ok();
        }
        GamePage::BattleOverview => {
            draw_battle_overview_page(&mut display_res.display, &mut game_state).ok();
        }
        GamePage::Debug => {
            draw_debug_page(&mut display_res.display, &game_state).ok();
        }
    }

    // Apply brightness setting directly
    // Slider 0% (brightness=0) = dim, Slider 100% (brightness=255) = bright
    let brightness_value = game_state.brightness as u16;
    display_res.display.set_brightness(brightness_value).ok();

    // Flush the display
    display_res.display.flush().ok();
}
