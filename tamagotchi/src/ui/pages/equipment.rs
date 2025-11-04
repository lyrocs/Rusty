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

use super::super::colors::*;
use crate::core::GameState;
use crate::tamagotchi::models::{
    BattleState, CircleType, Enemy, FarmState, LocationType, MapHelper, RestState,
};

use super::super::helpers::*;

// Menu background image
const MENU_GIF: &[u8] = include_bytes!("../../../assets/images/ui/menu.gif");

/// Helper function to get tier color based on level requirement
fn get_tier_color(level_req: u16) -> Rgb888 {
    if level_req >= 41 {
        Rgb888::new(255, 165, 0) // Legendary - Orange/Gold
    } else if level_req >= 31 {
        Rgb888::new(163, 53, 238) // Epic - Purple
    } else if level_req >= 21 {
        Rgb888::new(64, 156, 255) // Rare - Blue
    } else if level_req >= 11 {
        Rgb888::new(30, 255, 30) // Uncommon - Green
    } else {
        Rgb888::new(180, 180, 180) // Common - Gray
    }
}

/// Draw the Equipment page
pub fn draw_equipment_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let hero = &game_state.hero;

    // Clear background
    display.clear(Rgb888::new(0, 0, 0))?;

    // Draw background image (single frame GIF)
    let menu_gif = Gif::<Rgb888>::from_slice(MENU_GIF).expect("Failed to parse menu GIF");
    if let Some(frame) = menu_gif.frames().next() {
        Image::new(&frame, Point::new(0, 0)).draw(display)?;
    }

    let title_y = 20;

    // Title with background
    Rectangle::new(Point::new(60, title_y), Size::new(248, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 30, 60)))
        .draw(display)?;
    draw_text(
        display,
        "=== EQUIPMENT ===",
        Point::new(70, title_y + 18),
        &FONT_10X20,
        Rgb888::new(255, 230, 150),
    )?;

    // Equipment display (6 slots in 2x3 grid)
    // Left column (x=20) | Right column (x=200)
    let left_x = 20;
    let right_x = 200;
    let start_y = 70;
    let row_spacing = 115; // Increased to accommodate larger cards

    // Row 1: WEAPON | ARMOR
    draw_equipment_slot_with_tier(
        display,
        &hero.equipped_weapon,
        Point::new(left_x, start_y),
        "WEAPON",
    )?;

    draw_equipment_slot_with_tier(
        display,
        &hero.equipped_armor,
        Point::new(right_x, start_y),
        "ARMOR",
    )?;

    // Row 2: SHOES | GARMENT
    draw_equipment_slot_with_tier(
        display,
        &hero.equipped_shoes,
        Point::new(left_x, start_y + row_spacing),
        "SHOES",
    )?;

    draw_equipment_slot_with_tier(
        display,
        &hero.equipped_garment,
        Point::new(right_x, start_y + row_spacing),
        "GARMENT",
    )?;

    // Row 3: ACCESSORY 1 | ACCESSORY 2
    draw_equipment_slot_with_tier(
        display,
        &hero.equipped_accessory1,
        Point::new(left_x, start_y + (row_spacing * 2)),
        "ACCESS 1",
    )?;

    draw_equipment_slot_with_tier(
        display,
        &hero.equipped_accessory2,
        Point::new(right_x, start_y + (row_spacing * 2)),
        "ACCESS 2",
    )?;

    // Draw refine popup if open
    if game_state.refine_popup_open {
        if let Some(slot) = game_state.refine_slot {
            draw_refine_popup(display, game_state, slot)?;
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

/// Draw equipment slot with tier-based coloring
fn draw_equipment_slot_with_tier<D>(
    display: &mut D,
    equipment: &crate::hero::equipment::Equipment,
    position: Point,
    slot_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let tier_color = get_tier_color(equipment.level_req);
    let card_height = 105; // Increased from 85 to allow one more line

    // Background panel
    Rectangle::new(
        Point::new(position.x - 5, position.y - 5),
        Size::new(170, card_height),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 25, 35)))
    .draw(display)?;

    // Border with tier color
    Rectangle::new(
        Point::new(position.x - 5, position.y - 5),
        Size::new(170, card_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(tier_color, 2))
    .draw(display)?;

    // Slot label (moved up to be clear of border)
    draw_text(
        display,
        slot_name,
        Point::new(position.x, position.y + 6),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Equipment name with refine level (moved down further)
    let mut name_str = String::<48>::new();
    if equipment.refine_level > 0 {
        write!(name_str, "{} [+{}]", equipment.name, equipment.refine_level).ok();
    } else {
        write!(name_str, "{}", equipment.name).ok();
    }
    draw_text(
        display,
        &name_str,
        Point::new(position.x, position.y + 30),
        &FONT_9X18_BOLD,
        tier_color,
    )?;

    // Stats line 1: ATK/DEF
    let mut stats_str = String::<32>::new();
    if equipment.atk_bonus > 0 {
        write!(stats_str, "ATK: {}", equipment.atk_bonus).ok();
    } else if equipment.def_bonus > 0 {
        write!(stats_str, "DEF: {}", equipment.def_bonus).ok();
    }

    if !stats_str.is_empty() {
        draw_text(
            display,
            &stats_str,
            Point::new(position.x, position.y + 55),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Stats line 2: Card slot information
    let mut slots_str = String::<32>::new();
    write!(
        slots_str,
        "Slots: {}/{}",
        equipment.card_slots, equipment.max_card_slots
    )
    .ok();
    draw_text(
        display,
        &slots_str,
        Point::new(position.x, position.y + 75),
        &FONT_9X15,
        Rgb888::new(150, 150, 150),
    )?;

    // Stats line 3: Refinement level (if refined)
    if equipment.refine_level > 0 {
        let mut refine_str = String::<32>::new();
        write!(refine_str, "+{} Refine", equipment.refine_level).ok();
        draw_text(
            display,
            &refine_str,
            Point::new(position.x, position.y + 88),
            &FONT_9X15,
            Rgb888::new(100, 200, 255),
        )?;
    }

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
    write!(
        slots_str,
        "Slots: {}/{}",
        equipment.card_slots, equipment.max_card_slots
    )
    .ok();
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
        write!(
            cost_str,
            "Add Slot (+{} essence, {}z)",
            essence_cost, zeny_cost
        )
        .ok();
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
