use core::fmt::Write;
use embedded_graphics::{
    image::Image,
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle as EgCircle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;
use tinygif::Gif;

use crate::core::GameState;
use crate::tamagotchi::models::{BattleState, CircleType, Enemy, FarmState, LocationType, MapHelper, RestState};
use super::super::colors::*;

use super::super::helpers::*;

/// Draw the Equipment page
pub fn draw_equipment_page<D>(
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
        "=== EQUIPMENT ===",
        Point::new(60, title_y),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Equipment display (6 slots in 2x3 grid)
    // Left column (x=20) | Right column (x=200)
    let left_x = 20;
    let right_x = 200;
    let start_y = 70;
    let row_spacing = 95;

    // Row 1: WEAPON | ARMOR
    // Draw border for weapon
    Rectangle::new(Point::new(left_x - 5, start_y - 5), Size::new(170, 85))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;
    draw_equipment_slot_clickable(
        display,
        &hero.equipped_weapon,
        Point::new(left_x, start_y),
        "WEAPON",
    )?;

    // Draw border for armor
    Rectangle::new(Point::new(right_x - 5, start_y - 5), Size::new(170, 85))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;
    draw_equipment_slot_clickable(
        display,
        &hero.equipped_armor,
        Point::new(right_x, start_y),
        "ARMOR",
    )?;

    // Row 2: SHOES | GARMENT
    // Draw border for shoes
    Rectangle::new(Point::new(left_x - 5, start_y + row_spacing - 5), Size::new(170, 85))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;
    draw_equipment_slot_clickable(
        display,
        &hero.equipped_shoes,
        Point::new(left_x, start_y + row_spacing),
        "SHOES",
    )?;

    // Draw border for garment
    Rectangle::new(Point::new(right_x - 5, start_y + row_spacing - 5), Size::new(170, 85))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;
    draw_equipment_slot_clickable(
        display,
        &hero.equipped_garment,
        Point::new(right_x, start_y + row_spacing),
        "GARMENT",
    )?;

    // Row 3: ACCESSORY 1 | ACCESSORY 2
    // Draw border for accessory 1
    Rectangle::new(Point::new(left_x - 5, start_y + (row_spacing * 2) - 5), Size::new(170, 85))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;
    draw_equipment_slot_clickable(
        display,
        &hero.equipped_accessory1,
        Point::new(left_x, start_y + (row_spacing * 2)),
        "ACCESS 1",
    )?;

    // Draw border for accessory 2
    Rectangle::new(Point::new(right_x - 5, start_y + (row_spacing * 2) - 5), Size::new(170, 85))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;
    draw_equipment_slot_clickable(
        display,
        &hero.equipped_accessory2,
        Point::new(right_x, start_y + (row_spacing * 2)),
        "ACCESS 2",
    )?;

    // Equipment Presets Section
    let preset_y = 360;
    draw_text(
        display,
        "PRESETS:",
        Point::new(20, preset_y),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Draw 3 preset buttons
    for i in 0..3 {
        let btn_x = 20 + (i * 110);
        let btn_y = preset_y + 10;
        let is_active = hero.active_preset == Some(i as u8);
        let has_preset = hero.equipment_presets[i].is_some();

        // Button background
        let btn_color = if is_active {
            Rgb888::new(80, 120, 80) // Green if active
        } else if has_preset {
            Rgb888::new(60, 80, 100) // Blue if has preset
        } else {
            Rgb888::new(40, 40, 40) // Gray if empty
        };

        Rectangle::new(Point::new(btn_x as i32, btn_y), Size::new(100, 30))
            .into_styled(PrimitiveStyle::with_fill(btn_color))
            .draw(display)?;

        Rectangle::new(Point::new(btn_x as i32, btn_y), Size::new(100, 30))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1))
            .draw(display)?;

        // Button text
        let mut btn_text = String::<16>::new();
        if is_active {
            write!(btn_text, "P{} *", i + 1).ok();
        } else {
            write!(btn_text, "Preset {}", i + 1).ok();
        }

        draw_text(
            display,
            &btn_text,
            Point::new(btn_x as i32 + 8, btn_y + 18),
            &FONT_9X15,
            Rgb888::WHITE,
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

    // Draw refine popup if open
    if game_state.refine_popup_open {
        if let Some(slot) = game_state.refine_slot {
            draw_refine_popup(display, game_state, slot)?;
        }
    }

    // Draw preset menu if open
    if game_state.preset_menu_open {
        if let Some(preset_index) = game_state.preset_selected_index {
            draw_preset_menu(display, game_state, preset_index)?;
        }
    }

    // Draw card socket menu if open
    if game_state.card_socket_menu_open {
        if let Some(slot) = game_state.card_socket_slot {
            draw_card_socket_menu(display, game_state, slot)?;
        }
    }

    // Draw equipment info modal if open
    if game_state.equipment_info_open {
        if let Some(slot) = game_state.equipment_info_slot {
            draw_equipment_info_modal(display, game_state, slot)?;
        }
    }

    // Draw equipment swap menu if open (shown from within equipment info)
    if game_state.equipment_swap_menu_open {
        if let Some(slot) = game_state.equipment_swap_slot {
            draw_equipment_swap_menu(display, game_state, slot)?;
        }
    }

    Ok(())
}

/// Draw equipment slot (clickable - opens equipment info)
fn draw_equipment_slot_clickable<D>(
    display: &mut D,
    equipment: &crate::hero::equipment::Equipment,
    position: Point,
    slot_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Draw the equipment slot (just slot name and equipment name)
    draw_equipment_slot(display, equipment, position, slot_name)?;

    Ok(())
}

/// Draw preset action menu
fn draw_preset_menu<D>(
    display: &mut D,
    game_state: &GameState,
    preset_index: u8,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Semi-transparent overlay
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Menu background
    Rectangle::new(Point::new(50, 150), Size::new(268, 200))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 40, 50)))
        .draw(display)?;

    Rectangle::new(Point::new(50, 150), Size::new(268, 200))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Title
    let mut title_str = String::<32>::new();
    write!(title_str, "Preset {} Actions", preset_index + 1).ok();
    draw_text(
        display,
        &title_str,
        Point::new(100, 175),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    let has_preset = game_state.hero.equipment_presets[preset_index as usize].is_some();

    // Save button
    Rectangle::new(Point::new(75, 200), Size::new(218, 40))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 100)))
        .draw(display)?;
    draw_text(
        display,
        "Save Current",
        Point::new(110, 220),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    // Load button (grayed out if no preset)
    let load_color = if has_preset {
        Rgb888::new(60, 100, 60)
    } else {
        Rgb888::new(40, 40, 40)
    };
    Rectangle::new(Point::new(75, 250), Size::new(218, 40))
        .into_styled(PrimitiveStyle::with_fill(load_color))
        .draw(display)?;
    draw_text(
        display,
        "Load Preset",
        Point::new(115, 270),
        &FONT_9X18_BOLD,
        if has_preset { Rgb888::WHITE } else { Rgb888::new(100, 100, 100) },
    )?;

    // Clear button (grayed out if no preset)
    let clear_color = if has_preset {
        Rgb888::new(100, 40, 40)
    } else {
        Rgb888::new(40, 40, 40)
    };
    Rectangle::new(Point::new(75, 300), Size::new(218, 40))
        .into_styled(PrimitiveStyle::with_fill(clear_color))
        .draw(display)?;
    draw_text(
        display,
        "Clear Preset",
        Point::new(110, 320),
        &FONT_9X18_BOLD,
        if has_preset { Rgb888::WHITE } else { Rgb888::new(100, 100, 100) },
    )?;

    Ok(())
}

/// Draw card socket menu
fn draw_card_socket_menu<D>(
    display: &mut D,
    game_state: &GameState,
    equipment_slot: crate::hero::equipment::EquipmentSlot,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::hero::equipment::EquipmentSlot;

    let hero = &game_state.hero;
    let equipment = match equipment_slot {
        EquipmentSlot::Weapon => &hero.equipped_weapon,
        EquipmentSlot::Armor => &hero.equipped_armor,
        EquipmentSlot::Shoes => &hero.equipped_shoes,
        EquipmentSlot::Garment => &hero.equipped_garment,
        EquipmentSlot::Accessory1 => &hero.equipped_accessory1,
        EquipmentSlot::Accessory2 => &hero.equipped_accessory2,
    };

    // Semi-transparent overlay
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Menu background
    Rectangle::new(Point::new(20, 80), Size::new(328, 340))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 40, 50)))
        .draw(display)?;

    Rectangle::new(Point::new(20, 80), Size::new(328, 340))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Title
    let mut title_str = String::<48>::new();
    write!(title_str, "{} - Card Slots", equipment.name).ok();
    draw_text(
        display,
        &title_str,
        Point::new(35, 105),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Card slots info
    let mut slots_str = String::<32>::new();
    write!(slots_str, "Slots: {}/{}", equipment.card_slots, equipment.max_card_slots).ok();
    draw_text(
        display,
        &slots_str,
        Point::new(35, 125),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Display current socketed cards
    let slot_start_y = 150;
    for i in 0..(equipment.card_slots as usize) {
        let slot_y = slot_start_y + (i as i32 * 50);

        // Card slot background
        let has_card = equipment.socketed_cards[i].is_some();
        let bg_color = if has_card {
            Rgb888::new(50, 70, 90)
        } else {
            Rgb888::new(40, 40, 40)
        };

        Rectangle::new(Point::new(35, slot_y), Size::new(298, 45))
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

        Rectangle::new(Point::new(35, slot_y), Size::new(298, 45))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
            .draw(display)?;

        // Slot label
        let mut slot_label = String::<32>::new();
        write!(slot_label, "Slot {}", i + 1).ok();
        draw_text(
            display,
            &slot_label,
            Point::new(45, slot_y + 15),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;

        // Card info or empty
        if let Some(card_id) = equipment.socketed_cards[i] {
            if let Some(card) = crate::data::get_card_by_id(card_id) {
                // Card name
                draw_text(
                    display,
                    card.name,
                    Point::new(45, slot_y + 30),
                    &FONT_9X15,
                    Rgb888::new(100, 200, 100),
                )?;

                // Remove button
                draw_text(
                    display,
                    "[Remove]",
                    Point::new(250, slot_y + 25),
                    &FONT_9X15,
                    Rgb888::new(200, 100, 100),
                )?;
            }
        } else {
            // Empty slot
            draw_text(
                display,
                "Empty",
                Point::new(45, slot_y + 30),
                &FONT_9X15,
                Rgb888::new(100, 100, 100),
            )?;

            // Socket button (placeholder - needs inventory check)
            draw_text(
                display,
                "[Socket]",
                Point::new(250, slot_y + 25),
                &FONT_9X15,
                Rgb888::new(100, 150, 200),
            )?;
        }
    }

    // Add slot button (if not at max)
    if equipment.card_slots < equipment.max_card_slots {
        let add_slot_y = slot_start_y + (equipment.card_slots as i32 * 50);

        Rectangle::new(Point::new(35, add_slot_y), Size::new(298, 40))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 60)))
            .draw(display)?;

        let mut cost_str = String::<48>::new();
        let (essence_cost, zeny_cost) = match equipment.card_slots {
            1 => (3, 2000),
            2 => (5, 5000),
            3 => (10, 10000),
            _ => (0, 0),
        };
        write!(cost_str, "Add Slot (+{} essence, {}z)", essence_cost, zeny_cost).ok();
        draw_text(
            display,
            &cost_str,
            Point::new(50, add_slot_y + 22),
            &FONT_9X15,
            Rgb888::WHITE,
        )?;
    }

    // Close button
    Rectangle::new(Point::new(124, 385), Size::new(120, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 100, 150)))
        .draw(display)?;
    draw_text(
        display,
        "Close",
        Point::new(155, 403),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    Ok(())
}

