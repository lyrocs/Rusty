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

    // Draw farming header if active
    use crate::ui::farming_header::draw_farming_header;
    let has_farming_header = draw_farming_header(display, game_state)?;
    let title_y = if has_farming_header { 40 } else { 20 };

    // Title
    draw_text(
        display,
        "=== BLACKSMITH ===",
        Point::new(50, title_y),
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

    // Draw crafting details modal if open
    if game_state.crafting_details_open {
        if let Some(equip_id) = game_state.crafting_selected_id {
            if let Some(equip_data) = crate::data::get_equipment_data_by_id(equip_id) {
                draw_crafting_details_modal(display, game_state, equip_data)?;
            }
        }
    }

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

    // Materials (show first 2 with names)
    if let Some(materials) = &equip_data.craft_materials {
        let mat_y = position.y + 52;

        for (idx, (mat_id, required_qty)) in materials.iter().enumerate().take(2) {
            let mat_x = if idx == 0 {
                position.x + 5
            } else {
                position.x + 180
            };

            let item_name = crate::data::get_item_name(*mat_id);
            let has_qty = hero.inventory.iter()
                .find(|item| item.id == *mat_id)
                .map(|item| item.quantity)
                .unwrap_or(0);

            let mut mat_str = String::<32>::new();
            write!(mat_str, "{}: {}/{}", item_name, has_qty, required_qty).ok();

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

/// Draw crafting details modal showing equipment stats and materials
pub fn draw_crafting_details_modal<D>(
    display: &mut D,
    game_state: &GameState,
    equip_data: &crate::data::EquipmentData,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let hero = &game_state.hero;

    // Semi-transparent overlay
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Modal panel (fullscreen with margins)
    let panel_x = 10;
    let panel_y = 10;
    let panel_w = 348;
    let panel_h = 428;

    Rectangle::new(Point::new(panel_x, panel_y), Size::new(panel_w, panel_h))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 40)))
        .draw(display)?;

    Rectangle::new(Point::new(panel_x, panel_y), Size::new(panel_w, panel_h))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    let mut y = panel_y + 20;

    // Equipment name
    draw_text(
        display,
        equip_data.name,
        Point::new(panel_x + 10, y),
        &FONT_10X20,
        COLOR_TEXT,
    )?;
    y += 25;

    // Level requirement
    let mut level_str = String::<32>::new();
    write!(level_str, "Level: {}", equip_data.level_req).ok();
    let level_color = if hero.level >= equip_data.level_req {
        COLOR_TEXT
    } else {
        Rgb888::new(200, 100, 100)
    };
    draw_text(display, &level_str, Point::new(panel_x + 10, y), &FONT_9X15, level_color)?;
    y += 20;

    // Stats
    y = draw_equipment_stats_compact(display, equip_data, Point::new(panel_x + 10, y))?;
    y += 10;

    // Materials section
    draw_text(
        display,
        "Materials:",
        Point::new(panel_x + 10, y),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;
    y += 20;

    if let Some(materials) = &equip_data.craft_materials {
        for (mat_id, required_qty) in materials.iter() {
            let item_name = crate::data::get_item_name(*mat_id);
            let has_qty = hero.inventory.iter()
                .find(|item| item.id == *mat_id)
                .map(|item| item.quantity)
                .unwrap_or(0);

            let mut mat_str = String::<48>::new();
            write!(mat_str, "{}: {}/{}", item_name, has_qty, required_qty).ok();

            let mat_color = if has_qty >= *required_qty {
                Rgb888::new(100, 200, 100)
            } else {
                Rgb888::new(200, 100, 100)
            };

            draw_text(display, &mat_str, Point::new(panel_x + 15, y), &FONT_9X15, mat_color)?;
            y += 18;
        }
    }

    y += 5;

    // Cost
    let mut cost_str = String::<32>::new();
    write!(cost_str, "Cost: {}z", equip_data.craft_cost).ok();
    let cost_color = if hero.zeny >= equip_data.craft_cost {
        COLOR_EXP
    } else {
        Rgb888::new(200, 100, 100)
    };
    draw_text(display, &cost_str, Point::new(panel_x + 10, y), &FONT_9X18_BOLD, cost_color)?;

    // Check if can craft
    let can_craft = check_can_craft(hero, equip_data);

    // Buttons on same line at bottom
    let buttons_y = panel_y + panel_h as i32 - 45;
    let button_width = 150;
    let button_gap = 8;
    let craft_btn_x = panel_x + 20;
    let close_btn_x = craft_btn_x + button_width + button_gap;

    // Craft button (green if can craft, gray if not)
    let craft_color = if can_craft {
        Rgb888::new(60, 140, 60)
    } else {
        Rgb888::new(60, 60, 60)
    };

    Rectangle::new(Point::new(craft_btn_x, buttons_y), Size::new(button_width as u32, 35))
        .into_styled(PrimitiveStyle::with_fill(craft_color))
        .draw(display)?;

    Rectangle::new(Point::new(craft_btn_x, buttons_y), Size::new(button_width as u32, 35))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1))
        .draw(display)?;

    let craft_text = if can_craft { "CRAFT" } else { "Cannot" };
    draw_text(
        display,
        craft_text,
        Point::new(craft_btn_x + 40, buttons_y + 22),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Close button (red)
    Rectangle::new(Point::new(close_btn_x, buttons_y), Size::new(button_width as u32, 35))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 60, 60)))
        .draw(display)?;

    Rectangle::new(Point::new(close_btn_x, buttons_y), Size::new(button_width as u32, 35))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1))
        .draw(display)?;

    draw_text(
        display,
        "CLOSE",
        Point::new(close_btn_x + 35, buttons_y + 22),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    Ok(())
}

/// Draw equipment stats in compact form
fn draw_equipment_stats_compact<D>(
    display: &mut D,
    equip_data: &crate::data::EquipmentData,
    start_pos: Point,
) -> Result<i32, D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let mut y = start_pos.y;
    let x = start_pos.x;

    // Primary stats
    if equip_data.atk_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "ATK: +{}", equip_data.atk_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_HP)?;
        y += 17;
    }

    if equip_data.def_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "DEF: +{}", equip_data.def_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, Rgb888::new(150, 150, 200))?;
        y += 17;
    }

    if equip_data.hp_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "HP: +{}", equip_data.hp_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_HP)?;
        y += 17;
    }

    if equip_data.sp_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "SP: +{}", equip_data.sp_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_SP)?;
        y += 17;
    }

    // Stat bonuses (combine in one line if possible)
    let mut stat_bonuses = String::<64>::new();
    let mut has_stats = false;

    if equip_data.str_bonus != 0 {
        write!(stat_bonuses, "STR+{} ", equip_data.str_bonus).ok();
        has_stats = true;
    }
    if equip_data.agi_bonus != 0 {
        write!(stat_bonuses, "AGI+{} ", equip_data.agi_bonus).ok();
        has_stats = true;
    }
    if equip_data.vit_bonus != 0 {
        write!(stat_bonuses, "VIT+{} ", equip_data.vit_bonus).ok();
        has_stats = true;
    }
    if equip_data.int_bonus != 0 {
        write!(stat_bonuses, "INT+{} ", equip_data.int_bonus).ok();
        has_stats = true;
    }
    if equip_data.dex_bonus != 0 {
        write!(stat_bonuses, "DEX+{} ", equip_data.dex_bonus).ok();
        has_stats = true;
    }
    if equip_data.luk_bonus != 0 {
        write!(stat_bonuses, "LUK+{} ", equip_data.luk_bonus).ok();
        has_stats = true;
    }

    if has_stats {
        draw_text(display, &stat_bonuses, Point::new(x, y), &FONT_9X15, COLOR_TEXT_DIM)?;
        y += 17;
    }

    // Special bonuses
    if equip_data.crit_rate_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "Crit Rate: +{}", equip_data.crit_rate_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_TEXT_DIM)?;
        y += 17;
    }

    if equip_data.aspd_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "ASPD: +{}", equip_data.aspd_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_TEXT_DIM)?;
        y += 17;
    }

    if equip_data.flee_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "Flee: +{}", equip_data.flee_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_TEXT_DIM)?;
        y += 17;
    }

    if equip_data.hit_bonus > 0 {
        let mut s = String::<32>::new();
        write!(s, "Hit: +{}", equip_data.hit_bonus).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_TEXT_DIM)?;
        y += 17;
    }

    if equip_data.damage_reduction > 0 {
        let mut s = String::<32>::new();
        write!(s, "DMG Reduction: +{}%", equip_data.damage_reduction).ok();
        draw_text(display, &s, Point::new(x, y), &FONT_9X15, COLOR_TEXT_DIM)?;
        y += 17;
    }

    Ok(y)
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
