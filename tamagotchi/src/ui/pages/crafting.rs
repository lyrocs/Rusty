use core::fmt::Write;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;

use crate::core::GameState;
use crate::hero::inventory::InventoryExt;
use super::super::colors::*;
use super::super::helpers::*;

/// Draw the Crafting/Blacksmith page
pub fn draw_crafting_page<D>(
    display: &mut D,
    game_state: &GameState,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let hero = &game_state.hero;

    // Clear background
    display.clear(COLOR_BG)?;

    // Title
    draw_text(
        display,
        "=== BLACKSMITH ===",
        Point::new(50, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Zeny display
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "Zeny: {}", hero.zeny).ok();
    draw_text(
        display,
        &zeny_str,
        Point::new(220, 50),
        &FONT_9X15,
        COLOR_EXP,
    )?;

    // Filter buttons
    draw_filter_buttons(display, game_state)?;

    // Get craftable equipment based on current location
    let current_map = crate::data::get_map_data(game_state.current_location);
    let city_name = if let Some(map) = current_map {
        if crate::data::is_city(map.id) {
            map.name
        } else {
            "Prontera" // Default to Prontera if not in a city
        }
    } else {
        "Prontera"
    };

    // Get craftable items
    let craftable_items_all = crate::data::get_craftable_equipment_for_city(city_name);

    // Filter by slot if needed
    let craftable_items: heapless::Vec<&crate::data::EquipmentData, 16> = if game_state.crafting_filter == "All" {
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

    // Draw recipe list (max 4 visible)
    let start_y = 120;
    let item_height = 70;
    let max_visible = 4;

    for (i, equip_data) in craftable_items.iter()
        .skip(game_state.crafting_scroll as usize)
        .take(max_visible)
        .enumerate()
    {
        let y = start_y + (i as i32 * item_height);
        draw_recipe_entry(display, game_state, equip_data, Point::new(10, y))?;
    }

    // Scroll indicators
    if game_state.crafting_scroll > 0 {
        draw_text(
            display,
            "^ More",
            Point::new(155, 110),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    if craftable_items.len() > (game_state.crafting_scroll as usize + max_visible) {
        draw_text(
            display,
            "v More",
            Point::new(155, 400),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    // Back button
    Rectangle::new(Point::new(100, 410), Size::new(160, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 100, 150)))
        .draw(display)?;
    draw_text(
        display,
        "Back",
        Point::new(155, 428),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Draw craft result message if active
    if let Some(msg) = game_state.craft_result_message {
        draw_craft_result_popup(display, msg)?;
    }

    Ok(())
}

/// Draw filter buttons for equipment slots
fn draw_filter_buttons<D>(
    display: &mut D,
    game_state: &GameState,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let filters = ["All", "Weapon", "Armor", "Shoes", "Garment", "Accessory"];
    let btn_width = 58;
    let btn_height = 28;
    let start_x = 5;
    let start_y = 70;

    for (i, filter) in filters.iter().enumerate() {
        let x = start_x + (i as i32 * (btn_width + 3));
        let is_active = game_state.crafting_filter == *filter;

        let bg_color = if is_active {
            Rgb888::new(80, 120, 80)
        } else {
            Rgb888::new(50, 50, 70)
        };

        Rectangle::new(Point::new(x, start_y), Size::new(btn_width as u32, btn_height as u32))
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

        Rectangle::new(Point::new(x, start_y), Size::new(btn_width as u32, btn_height as u32))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1))
            .draw(display)?;

        draw_text(
            display,
            filter,
            Point::new(x + 5, start_y + 18),
            &FONT_9X15,
            Rgb888::WHITE,
        )?;
    }

    Ok(())
}

/// Draw a single recipe entry
fn draw_recipe_entry<D>(
    display: &mut D,
    game_state: &GameState,
    equip_data: &crate::data::EquipmentData,
    position: Point,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let hero = &game_state.hero;

    // Check if craftable
    let can_craft = check_can_craft(hero, equip_data);

    let bg_color = if can_craft {
        Rgb888::new(40, 60, 50)
    } else {
        Rgb888::new(40, 40, 40)
    };

    // Background
    Rectangle::new(position, Size::new(348, 65))
        .into_styled(PrimitiveStyle::with_fill(bg_color))
        .draw(display)?;

    Rectangle::new(position, Size::new(348, 65))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;

    // Equipment name
    let name_color = if can_craft { Rgb888::new(100, 200, 100) } else { COLOR_TEXT_DIM };
    draw_text(
        display,
        equip_data.name,
        Point::new(position.x + 5, position.y + 15),
        &FONT_9X18_BOLD,
        name_color,
    )?;

    // Cost
    let mut cost_str = String::<32>::new();
    write!(cost_str, "Cost: {}z", equip_data.craft_cost).ok();
    let cost_color = if hero.zeny >= equip_data.craft_cost {
        COLOR_EXP
    } else {
        Rgb888::new(200, 100, 100)
    };
    draw_text(
        display,
        &cost_str,
        Point::new(position.x + 5, position.y + 35),
        &FONT_9X15,
        cost_color,
    )?;

    // Materials (show first 2)
    if let Some(materials) = &equip_data.craft_materials {
        let mut mat_x = position.x + 5;
        let mat_y = position.y + 52;

        for (idx, (mat_id, required_qty)) in materials.iter().enumerate().take(2) {
            if idx > 0 {
                mat_x += 120;
            }

            let item_name = crate::data::get_item_name(*mat_id);
            let has_qty = hero.inventory.iter()
                .find(|item| item.id == *mat_id)
                .map(|item| item.quantity)
                .unwrap_or(0);

            let mut mat_str = String::<32>::new();
            write!(mat_str, "{}/{}", has_qty, required_qty).ok();

            let mat_color = if has_qty >= *required_qty {
                Rgb888::new(100, 200, 100)
                } else {
                Rgb888::new(200, 100, 100)
            };

            draw_text(
                display,
                &mat_str,
                Point::new(mat_x, mat_y),
                &FONT_9X15,
                mat_color,
            )?;
        }
    }

    // Craft button
    if can_craft {
        draw_text(
            display,
            "[Craft]",
            Point::new(position.x + 285, position.y + 35),
            &FONT_9X18_BOLD,
            Rgb888::new(100, 200, 100),
        )?;
    }

    Ok(())
}

/// Check if hero can craft an equipment
fn check_can_craft(hero: &crate::hero::Hero, equip_data: &crate::data::EquipmentData) -> bool {
    // Check level
    if hero.level < equip_data.level_req {
        return false;
    }

    // Check zeny
    if hero.zeny < equip_data.craft_cost {
        return false;
    }

    // Check materials
    if let Some(materials) = &equip_data.craft_materials {
        for (mat_id, required_qty) in materials.iter() {
            if !hero.inventory.has_item(*mat_id, *required_qty) {
                return false;
            }
        }
    }

    true
}

/// Draw craft result popup
fn draw_craft_result_popup<D>(
    display: &mut D,
    message: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Semi-transparent overlay
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Message box
    Rectangle::new(Point::new(50, 180), Size::new(268, 88))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 50, 40)))
        .draw(display)?;

    Rectangle::new(Point::new(50, 180), Size::new(268, 88))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Message text
    draw_text(
        display,
        message,
        Point::new(80, 220),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    Ok(())
}
