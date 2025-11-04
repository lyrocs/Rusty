/// UI Helper Functions
///
/// Common drawing utilities shared across multiple pages.
use core::fmt::Write;
use embedded_graphics::{
    image::Image,
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;
use tinygif::Gif;

use super::colors::*;
use crate::core::GameState;

pub fn draw_monster_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = game_state.monster_animation.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.monster_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw monster idle GIF (0.gif) on map page
pub fn draw_map_monster_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Get idle animation GIF (0.gif) for the monster
    let gif_data = crate::tamagotchi::models::MonsterAnimation::Idle.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame from map animation frame counter
    // Count total frames first to wrap properly
    let total_frames = gif.frames().count();
    if total_frames == 0 {
        return Ok(());
    }

    let frame_index = game_state.map_monster_animation_frame % total_frames;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw hero GIF animation
pub fn draw_hero_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = game_state.hero_animation.gif_data(&game_state.hero.job);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse hero GIF");

    // Get GIF dimensions
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to align by bottom
    // center_position is treated as the bottom-center anchor point
    // This ensures smooth transitions between different-sized animations (36.gif vs 84.gif)
    let top_left = Point::new(
        center_position.x - (gif_width / 2), // Center horizontally
        center_position.y - gif_height,      // Align by bottom
    );

    // Get current frame
    let frame_index = game_state.hero_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw hero GIF animation with specific animation state
/// This function clears only the GIF zone and draws the frame within it
pub fn draw_hero_gif_with_animation<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    animation: crate::combat::HeroAnimation,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = animation.gif_data(&game_state.hero.job);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse hero GIF");

    // Get GIF dimensions
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to align by bottom
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - gif_height,
    );

    // // Clear ONLY the GIF zone with background color
    // Rectangle::new(
    //     top_left,
    //     Size::new(gif_width as u32, gif_height as u32),
    // )
    // .into_styled(PrimitiveStyle::with_fill(super::colors::COLOR_BG))
    // .draw(display)?;

    // Get current frame
    let frame_index = game_state.hero_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw monster GIF animation with specific animation state
pub fn draw_monster_gif_with_animation<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
    animation: crate::combat::MonsterAnimation,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let gif_data = animation.gif_data(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.monster_animation_frame;
    let mut current_index = 0;

    for frame in gif.frames() {
        if current_index == frame_index {
            Image::new(&frame, top_left).draw(display)?;
            break;
        }
        current_index += 1;
    }

    Ok(())
}

/// Helper: Draw FPS information
pub fn draw_fps_info<D>(display: &mut D, position: Point, fps: u32) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // FPS label
    draw_text(display, "FPS:", position, &FONT_9X18_BOLD, COLOR_TEXT_DIM)?;

    // FPS value
    let mut fps_str = String::<16>::new();
    write!(fps_str, "{}", fps).ok();

    // Color based on FPS (green if 30+, yellow if 20-29, red if <20)
    let fps_color = if fps >= 30 {
        Rgb888::GREEN
    } else if fps >= 20 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    // Position value to the right of the label (FPS: is 4 chars * 9px = 36px + 5px spacing)
    draw_text(
        display,
        &fps_str,
        position + Point::new(45, 0),
        &FONT_9X18_BOLD,
        fps_color,
    )?;

    Ok(())
}

/// Helper: Draw text
pub fn draw_text<D>(
    display: &mut D,
    text: &str,
    position: Point,
    font: &embedded_graphics::mono_font::MonoFont,
    color: Rgb888,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    Text::new(text, position, MonoTextStyle::new(font, color)).draw(display)?;
    Ok(())
}

/// Helper: Apply alpha fade to a color
/// alpha_factor: 0.0 (fully transparent/faded) to 1.0 (fully opaque)
/// Since Rgb888 doesn't support alpha, we simulate it by blending with background color
pub fn apply_alpha_fade(color: Rgb888, alpha_factor: f32) -> Rgb888 {
    let alpha = alpha_factor.max(0.0).min(1.0);

    // Blend with dark background (approximating transparency)
    let bg = COLOR_BG;
    let r = (color.r() as f32 * alpha + bg.r() as f32 * (1.0 - alpha)) as u8;
    let g = (color.g() as f32 * alpha + bg.g() as f32 * (1.0 - alpha)) as u8;
    let b = (color.b() as f32 * alpha + bg.b() as f32 * (1.0 - alpha)) as u8;

    Rgb888::new(r, g, b)
}

/// Helper: Draw a horizontal progress bar
pub fn draw_bar<D>(
    display: &mut D,
    position: Point,
    width: u32,
    percent: u8,
    color: Rgb888,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let percent = percent.min(100);
    let height = 10;

    // Background
    Rectangle::new(position, Size::new(width, height))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
        .draw(display)?;

    // Fill
    let fill_width = (width as u32 * percent as u32) / 100;
    if fill_width > 0 {
        Rectangle::new(
            position + Point::new(1, 1),
            Size::new(fill_width, height - 2),
        )
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(display)?;
    }

    Ok(())
}

/// Helper: Draw battery information (voltage and percentage)
pub fn draw_battery_info<D>(
    display: &mut D,
    position: Point,
    voltage_mv: u16,
    percent: u8,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Battery label
    draw_text(
        display,
        "Battery:",
        position,
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;

    // Battery percentage and voltage
    let mut bat_str = String::<32>::new();
    write!(bat_str, "{}% | {}mV", percent, voltage_mv).ok();

    // Color based on battery level
    let bat_color = if percent >= 50 {
        Rgb888::GREEN
    } else if percent >= 20 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    draw_text(
        display,
        &bat_str,
        position + Point::new(0, 20),
        &FONT_9X15,
        bat_color,
    )?;

    // Battery bar
    draw_bar(
        display,
        position + Point::new(0, 35),
        200,
        percent,
        bat_color,
    )?;

    Ok(())
}

// Equipment helper functions
pub fn draw_equipment_slot<D>(
    display: &mut D,
    equipment: &crate::tamagotchi::models::Equipment,
    position: Point,
    slot_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Slot label
    draw_text(display, slot_name, position, &FONT_9X15, COLOR_TEXT_DIM)?;

    // Equipment name with refine level (simplified - only name)
    let mut name_str = String::<48>::new();
    if equipment.refine_level > 0 {
        write!(name_str, "{} [+{}]", equipment.name, equipment.refine_level).ok();
    } else {
        write!(name_str, "{}", equipment.name).ok();
    }
    draw_text(
        display,
        &name_str,
        Point::new(position.x, position.y + 20),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    Ok(())
}

/// Draw the equipment selection menu overlay
pub fn draw_equipment_selection<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Fullscreen dark overlay
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(10, 10, 20)))
        .draw(display)?;

    // Fullscreen panel
    let panel_x = 10;
    let panel_y = 10;
    let panel_width = 348;
    let panel_height = 428;

    Rectangle::new(
        Point::new(panel_x, panel_y),
        Size::new(panel_width, panel_height),
    )
    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
    .draw(display)?;

    Rectangle::new(
        Point::new(panel_x, panel_y),
        Size::new(panel_width, panel_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
    .draw(display)?;

    // Title
    draw_text(
        display,
        "SELECT EQUIPMENT TO REFINE",
        Point::new(30, 30),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Draw 6 equipment slot buttons in single column
    let slots = [
        (crate::tamagotchi::models::EquipmentSlot::Weapon, "WEAPON"),
        (crate::tamagotchi::models::EquipmentSlot::Armor, "ARMOR"),
        (crate::tamagotchi::models::EquipmentSlot::Shoes, "SHOES"),
        (crate::tamagotchi::models::EquipmentSlot::Garment, "GARMENT"),
        (
            crate::tamagotchi::models::EquipmentSlot::Accessory1,
            "ACCESSORY 1",
        ),
        (
            crate::tamagotchi::models::EquipmentSlot::Accessory2,
            "ACCESSORY 2",
        ),
    ];

    let start_y = 55;
    let item_height = 55;

    for (i, (slot, label)) in slots.iter().enumerate() {
        let btn_y = start_y + i as i32 * item_height;
        let btn_x = 20;
        let btn_width = 328u32;
        let btn_height = 50u32;

        // Get equipment for this slot
        let equipment = game_state.hero.get_equipment(*slot);

        // Button background - different color for equipped items
        let bg_color = if equipment.is_some() {
            Rgb888::new(40, 60, 80)
        } else {
            Rgb888::new(30, 30, 40)
        };

        Rectangle::new(Point::new(btn_x, btn_y), Size::new(btn_width, btn_height))
            .into_styled(PrimitiveStyle::with_fill(bg_color))
            .draw(display)?;

        Rectangle::new(Point::new(btn_x, btn_y), Size::new(btn_width, btn_height))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 1))
            .draw(display)?;

        // Equipment info
        if let Some(equip) = equipment {
            // Equipment name with refine level
            let mut name_str = String::<48>::new();
            if equip.refine_level > 0 {
                write!(name_str, "{} [+{}]", equip.name, equip.refine_level).ok();
            } else {
                write!(name_str, "{}", equip.name).ok();
            }

            draw_text(
                display,
                label,
                Point::new(btn_x + 10, btn_y + 18),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            draw_text(
                display,
                &name_str,
                Point::new(btn_x + 10, btn_y + 36),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
        } else {
            // No equipment in this slot
            draw_text(
                display,
                label,
                Point::new(btn_x + 10, btn_y + 18),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            draw_text(
                display,
                "(Empty)",
                Point::new(btn_x + 10, btn_y + 36),
                &FONT_9X15,
                Rgb888::new(150, 50, 50),
            )?;
        }
    }

    // Cancel button at bottom
    let cancel_btn_y = 390;
    Rectangle::new(Point::new(110, cancel_btn_y), Size::new(148, 36))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
        .draw(display)?;

    Rectangle::new(Point::new(110, cancel_btn_y), Size::new(148, 36))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 2))
        .draw(display)?;

    draw_text(
        display,
        "CANCEL",
        Point::new(140, cancel_btn_y + 22),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    Ok(())
}

/// Draw the refine popup overlay
pub fn draw_refine_popup<D>(
    display: &mut D,
    game_state: &GameState,
    slot: crate::tamagotchi::models::EquipmentSlot,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::tamagotchi::models::EquipmentSlot;

    let hero = &game_state.hero;
    let equipment = match slot {
        EquipmentSlot::Weapon => &hero.equipped_weapon,
        EquipmentSlot::Armor => &hero.equipped_armor,
        EquipmentSlot::Shoes => &hero.equipped_shoes,
        EquipmentSlot::Garment => &hero.equipped_garment,
        EquipmentSlot::Accessory1 => &hero.equipped_accessory1,
        EquipmentSlot::Accessory2 => &hero.equipped_accessory2,
    };

    // Semi-transparent overlay background
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Popup panel
    Rectangle::new(Point::new(30, 120), Size::new(308, 220))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 50, 70)))
        .draw(display)?;

    // Title
    draw_text(
        display,
        "=== REFINE ===",
        Point::new(100, 140),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Equipment name with current refine
    let mut name_str = String::<48>::new();
    if equipment.refine_level > 0 {
        write!(name_str, "{} [+{}]", equipment.name, equipment.refine_level).ok();
    } else {
        write!(name_str, "{}", equipment.name).ok();
    }
    draw_text(
        display,
        &name_str,
        Point::new(45, 170),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Show result message if present
    if let Some(msg) = game_state.refine_result_message {
        let color = if msg.contains("Success") {
            Rgb888::new(100, 255, 100)
        } else {
            Rgb888::new(255, 100, 100)
        };
        draw_text(display, msg, Point::new(80, 195), &FONT_9X18_BOLD, color)?;
    } else {
        // Show next level preview
        if equipment.can_refine() {
            let mut next_str = String::<32>::new();
            write!(next_str, "Next: [+{}]", equipment.refine_level + 1).ok();
            draw_text(
                display,
                &next_str,
                Point::new(45, 195),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            // Show stat change
            let bonus_change = match slot {
                EquipmentSlot::Weapon => 2,     // +2 ATK
                EquipmentSlot::Armor => 1,      // +1 DEF
                EquipmentSlot::Shoes => 1,      // +1 AGI
                EquipmentSlot::Garment => 1,    // +1 DEF
                EquipmentSlot::Accessory1 => 1, // +1 stat
                EquipmentSlot::Accessory2 => 1, // +1 stat
            };
            let stat_name = match slot {
                EquipmentSlot::Weapon => "ATK",
                EquipmentSlot::Armor => "DEF",
                EquipmentSlot::Shoes => "AGI",
                EquipmentSlot::Garment => "DEF",
                EquipmentSlot::Accessory1 => "Stat",
                EquipmentSlot::Accessory2 => "Stat",
            };
            let mut stat_str = String::<32>::new();
            write!(stat_str, "{} +{}", stat_name, bonus_change).ok();
            draw_text(
                display,
                &stat_str,
                Point::new(45, 215),
                &FONT_9X15,
                Rgb888::new(100, 255, 100),
            )?;
        } else {
            draw_text(
                display,
                "Max refine level!",
                Point::new(80, 195),
                &FONT_9X15,
                Rgb888::new(255, 100, 100),
            )?;
        }

        // Cost and success rate
        if equipment.can_refine() {
            let cost = equipment.refine_cost();
            let mut cost_str = String::<32>::new();
            write!(cost_str, "Cost: {} Zeny", cost).ok();
            let cost_color = if hero.zeny >= cost {
                COLOR_TEXT
            } else {
                Rgb888::new(255, 100, 100)
            };
            draw_text(
                display,
                &cost_str,
                Point::new(45, 240),
                &FONT_9X15,
                cost_color,
            )?;

            let rate = equipment.refine_success_rate();
            let mut rate_str = String::<32>::new();
            write!(rate_str, "Success: {}%", rate).ok();
            draw_text(
                display,
                &rate_str,
                Point::new(45, 260),
                &FONT_9X15,
                COLOR_TEXT,
            )?;

            // Warning for risky refines
            if equipment.is_risky_refine() {
                draw_text(
                    display,
                    "Failure drops to",
                    Point::new(45, 280),
                    &FONT_9X15,
                    Rgb888::new(255, 180, 0),
                )?;
                let mut warn_str = String::<32>::new();
                write!(warn_str, "[+{}]", equipment.refine_level.saturating_sub(1)).ok();
                draw_text(
                    display,
                    &warn_str,
                    Point::new(200, 280),
                    &FONT_9X15,
                    Rgb888::new(255, 100, 100),
                )?;
            }
        }
    }

    // Buttons
    if game_state.refine_result_message.is_some() {
        // Just Close button after refine attempt
        Rectangle::new(Point::new(120, 300), Size::new(128, 30))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 100, 150)))
            .draw(display)?;
        draw_text(
            display,
            "Close",
            Point::new(160, 316),
            &FONT_9X18_BOLD,
            Rgb888::WHITE,
        )?;
    } else {
        // Refine and Cancel buttons
        if equipment.can_refine() && hero.zeny >= equipment.refine_cost() {
            Rectangle::new(Point::new(50, 300), Size::new(110, 30))
                .into_styled(PrimitiveStyle::with_fill(Rgb888::new(180, 140, 60)))
                .draw(display)?;
            draw_text(
                display,
                "REFINE",
                Point::new(70, 316),
                &FONT_9X18_BOLD,
                Rgb888::WHITE,
            )?;
        }

        Rectangle::new(Point::new(208, 300), Size::new(110, 30))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 50, 50)))
            .draw(display)?;
        draw_text(
            display,
            "Cancel",
            Point::new(230, 316),
            &FONT_9X18_BOLD,
            Rgb888::WHITE,
        )?;
    }

    Ok(())
}

/// Draw the equipment swap menu overlay
pub fn draw_equipment_swap_menu<D>(
    display: &mut D,
    game_state: &GameState,
    slot: crate::tamagotchi::models::EquipmentSlot,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::tamagotchi::models::EquipmentSlot;

    // Fullscreen background with solid color
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 45)))
        .draw(display)?;

    // Title
    let slot_name = match slot {
        EquipmentSlot::Weapon => "WEAPON",
        EquipmentSlot::Armor => "ARMOR",
        EquipmentSlot::Shoes => "SHOES",
        EquipmentSlot::Garment => "GARMENT",
        EquipmentSlot::Accessory1 | EquipmentSlot::Accessory2 => "ACCESSORY",
    };

    let mut title_str = String::<48>::new();
    write!(title_str, "SELECT {} TO EQUIP", slot_name).ok();

    Rectangle::new(Point::new(10, 20), Size::new(348, 35))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 30, 60)))
        .draw(display)?;

    draw_text(
        display,
        &title_str,
        Point::new(30, 40),
        &FONT_10X20,
        Rgb888::new(255, 230, 150),
    )?;

    // Get equipment items from inventory that match this slot
    let slot_str = match slot {
        EquipmentSlot::Weapon => "Weapon",
        EquipmentSlot::Armor => "Armor",
        EquipmentSlot::Shoes => "Shoes",
        EquipmentSlot::Garment => "Garment",
        EquipmentSlot::Accessory1 | EquipmentSlot::Accessory2 => "Accessory",
    };

    // Collect equipment items from inventory
    let mut equipment_items: heapless::Vec<crate::hero::equipment::Equipment, 16> =
        heapless::Vec::new();
    for item in game_state.hero.inventory.iter() {
        // Equipment IDs: 1000-1999 (Weapons), 2000-2999 (Armor), 3000-3999 (Shoes), 4000-4999 (Garment), 5000-5999 (Accessory)
        if item.id >= 1000 && item.id < 6000 {
            // Try to get equipment data - either from JSON or use get_equipment_by_id which handles both
            if let Some(equip) = crate::data::get_equipment_by_id(item.id as u16) {
                // Check if the slot matches
                if equip.slot == slot {
                    equipment_items.push(equip).ok();
                }
            }
        }
    }

    // Draw equipment list (max 5 visible items with scrolling)
    let start_y = 70;
    let item_height = 70;
    let max_visible = 5;

    let scroll_offset = game_state.equipment_swap_scroll as usize;

    // Helper function to get tier color
    let get_tier_color = |level_req: u16| -> Rgb888 {
        if level_req >= 41 {
            Rgb888::new(255, 165, 0)
        } else if level_req >= 31 {
            Rgb888::new(163, 53, 238)
        } else if level_req >= 21 {
            Rgb888::new(64, 156, 255)
        } else if level_req >= 11 {
            Rgb888::new(30, 255, 30)
        } else {
            Rgb888::new(180, 180, 180)
        }
    };

    for (i, equip) in equipment_items
        .iter()
        .skip(scroll_offset)
        .take(max_visible)
        .enumerate()
    {
        let btn_y = start_y + i as i32 * item_height;
        let btn_x = 20;
        let btn_width = 328u32;
        let btn_height = 65u32;

        let tier_color = get_tier_color(equip.level_req);

        // Button background
        Rectangle::new(Point::new(btn_x, btn_y), Size::new(btn_width, btn_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 25, 35)))
            .draw(display)?;

        Rectangle::new(Point::new(btn_x, btn_y), Size::new(btn_width, btn_height))
            .into_styled(PrimitiveStyle::with_stroke(tier_color, 2))
            .draw(display)?;

        // Equipment name with refine level
        let mut name_str = String::<48>::new();
        if equip.refine_level > 0 {
            write!(name_str, "{} [+{}]", equip.name, equip.refine_level).ok();
        } else {
            write!(name_str, "{}", equip.name).ok();
        }
        draw_text(
            display,
            &name_str,
            Point::new(btn_x + 10, btn_y + 20),
            &FONT_9X18_BOLD,
            tier_color,
        )?;

        // Equipment stats
        let mut stats_str = String::<64>::new();
        if equip.atk_bonus > 0 {
            write!(stats_str, "ATK:{} | Lv:{} | Slots:{}/{}",
                   equip.atk_bonus, equip.level_req, equip.card_slots, equip.max_card_slots).ok();
        } else if equip.def_bonus > 0 {
            write!(stats_str, "DEF:{} | Lv:{} | Slots:{}/{}",
                   equip.def_bonus, equip.level_req, equip.card_slots, equip.max_card_slots).ok();
        } else {
            write!(stats_str, "Lv:{} | Slots:{}/{}",
                   equip.level_req, equip.card_slots, equip.max_card_slots).ok();
        }
        draw_text(
            display,
            &stats_str,
            Point::new(btn_x + 10, btn_y + 45),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Show scroll indicators
    if scroll_offset > 0 {
        draw_text(
            display,
            "^ More",
            Point::new(155, 60),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    if equipment_items.len() > scroll_offset + max_visible {
        draw_text(
            display,
            "v More",
            Point::new(155, 425),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Show message if no equipment
    if equipment_items.is_empty() {
        draw_text(
            display,
            "No equipment in inventory",
            Point::new(50, 220),
            &FONT_9X18_BOLD,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Cancel button at bottom (larger)
    let cancel_btn_y = 390;
    Rectangle::new(Point::new(110, cancel_btn_y), Size::new(148, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
        .draw(display)?;

    draw_text(
        display,
        "CANCEL",
        Point::new(135, cancel_btn_y + 28),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    Ok(())
}

/// Draw the equipment info modal with full details
pub fn draw_equipment_info_modal<D>(
    display: &mut D,
    game_state: &GameState,
    slot: crate::tamagotchi::models::EquipmentSlot,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::tamagotchi::models::EquipmentSlot;

    let hero = &game_state.hero;
    let equipment = match slot {
        EquipmentSlot::Weapon => &hero.equipped_weapon,
        EquipmentSlot::Armor => &hero.equipped_armor,
        EquipmentSlot::Shoes => &hero.equipped_shoes,
        EquipmentSlot::Garment => &hero.equipped_garment,
        EquipmentSlot::Accessory1 => &hero.equipped_accessory1,
        EquipmentSlot::Accessory2 => &hero.equipped_accessory2,
    };

    // Fullscreen background with solid color
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 45)))
        .draw(display)?;

    // Equipment name with refine level (with background)
    let mut name_str = String::<48>::new();
    if equipment.refine_level > 0 {
        write!(name_str, "{} [+{}]", equipment.name, equipment.refine_level).ok();
    } else {
        write!(name_str, "{}", equipment.name).ok();
    }

    Rectangle::new(Point::new(10, 20), Size::new(348, 35))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 30, 60)))
        .draw(display)?;

    draw_text(
        display,
        &name_str,
        Point::new(20, 40),
        &FONT_10X20,
        Rgb888::new(255, 230, 150),
    )?;

    let mut y = 70;

    // Main stats
    if equipment.atk_bonus > 0 {
        let total_atk = equipment.total_atk();
        let mut stat_str = String::<64>::new();
        if equipment.refine_level > 0 {
            write!(
                stat_str,
                "ATK: {} ({}+{})",
                total_atk,
                equipment.atk_bonus,
                equipment.get_refine_bonus()
            )
            .ok();
        } else {
            write!(stat_str, "ATK: {}", total_atk).ok();
        }
        draw_text(
            display,
            &stat_str,
            Point::new(20, y),
            &FONT_9X18_BOLD,
            COLOR_TEXT,
        )?;
        y += 20;
    }

    if equipment.def_bonus > 0 {
        let total_def = equipment.total_def();
        let mut stat_str = String::<64>::new();
        if equipment.refine_level > 0 {
            write!(
                stat_str,
                "DEF: {} ({}+{})",
                total_def,
                equipment.def_bonus,
                equipment.get_refine_bonus()
            )
            .ok();
        } else {
            write!(stat_str, "DEF: {}", total_def).ok();
        }
        draw_text(
            display,
            &stat_str,
            Point::new(20, y),
            &FONT_9X18_BOLD,
            COLOR_TEXT,
        )?;
        y += 20;
    }

    if equipment.hp_bonus > 0 {
        let mut stat_str = String::<32>::new();
        write!(stat_str, "HP: +{}", equipment.hp_bonus).ok();
        draw_text(
            display,
            &stat_str,
            Point::new(20, y),
            &FONT_9X15,
            COLOR_TEXT,
        )?;
        y += 18;
    }

    if equipment.sp_bonus > 0 {
        let mut stat_str = String::<32>::new();
        write!(stat_str, "SP: +{}", equipment.sp_bonus).ok();
        draw_text(
            display,
            &stat_str,
            Point::new(20, y),
            &FONT_9X15,
            COLOR_TEXT,
        )?;
        y += 18;
    }

    // Stat bonuses
    y += 10;
    draw_text(
        display,
        "Stats:",
        Point::new(20, y),
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;
    y += 20;

    let stats = [
        ("STR", equipment.str_bonus),
        ("AGI", equipment.agi_bonus),
        ("VIT", equipment.vit_bonus),
        ("INT", equipment.int_bonus),
        ("DEX", equipment.dex_bonus),
        ("LUK", equipment.luk_bonus),
    ];

    for (stat_name, bonus) in &stats {
        if *bonus != 0 {
            let mut stat_str = String::<32>::new();
            write!(stat_str, "{}: {:+}", stat_name, bonus).ok();
            draw_text(
                display,
                &stat_str,
                Point::new(20, y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
            y += 18;
        }
    }

    // Special bonuses
    let has_special = equipment.crit_rate_bonus > 0
        || equipment.aspd_bonus > 0
        || equipment.flee_bonus > 0
        || equipment.hit_bonus > 0
        || equipment.damage_reduction > 0;

    if has_special {
        y += 5;
        draw_text(
            display,
            "Special:",
            Point::new(20, y),
            &FONT_9X18_BOLD,
            COLOR_TEXT_DIM,
        )?;
        y += 20;

        if equipment.crit_rate_bonus > 0 {
            let mut stat_str = String::<32>::new();
            write!(stat_str, "Crit Rate: +{}%", equipment.crit_rate_bonus).ok();
            draw_text(
                display,
                &stat_str,
                Point::new(20, y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
            y += 18;
        }

        if equipment.aspd_bonus > 0 {
            let mut stat_str = String::<32>::new();
            write!(stat_str, "ASPD: +{}%", equipment.aspd_bonus).ok();
            draw_text(
                display,
                &stat_str,
                Point::new(20, y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
            y += 18;
        }

        if equipment.flee_bonus > 0 {
            let mut stat_str = String::<32>::new();
            write!(stat_str, "Flee: +{}", equipment.flee_bonus).ok();
            draw_text(
                display,
                &stat_str,
                Point::new(20, y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
            y += 18;
        }

        if equipment.hit_bonus > 0 {
            let mut stat_str = String::<32>::new();
            write!(stat_str, "Hit: +{}", equipment.hit_bonus).ok();
            draw_text(
                display,
                &stat_str,
                Point::new(20, y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
            y += 18;
        }

        if equipment.damage_reduction > 0 {
            let mut stat_str = String::<32>::new();
            write!(stat_str, "DMG Reduction: {}%", equipment.damage_reduction).ok();
            draw_text(
                display,
                &stat_str,
                Point::new(20, y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
            y += 18;
        }
    }

    // Action buttons at bottom (larger size)
    let btn_y = 340;

    // Switch button
    Rectangle::new(Point::new(20, btn_y), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 120, 60)))
        .draw(display)?;
    draw_text(
        display,
        "Switch",
        Point::new(60, btn_y + 28),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Cards button (if has card slots)
    if equipment.card_slots > 0 {
        Rectangle::new(Point::new(195, btn_y), Size::new(165, 45))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 120)))
            .draw(display)?;
        draw_text(
            display,
            "Cards",
            Point::new(240, btn_y + 28),
            &FONT_10X20,
            Rgb888::WHITE,
        )?;
    }

    // Close button
    let close_btn_y = 395;
    Rectangle::new(Point::new(110, close_btn_y), Size::new(148, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
        .draw(display)?;
    draw_text(
        display,
        "Close",
        Point::new(150, close_btn_y + 28),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    Ok(())
}

/// Draw farm duration selection modal with efficiency preview
pub fn draw_farm_duration_selection<D>(
    display: &mut D,
    game_state: &GameState,
    enemy: &crate::combat::Enemy,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::combat::{
        FarmDuration, calculate_efficiency, calculate_expected_kills, calculate_farm_rewards,
    };

    // Semi-transparent overlay
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Modal panel
    let panel_x = 10;
    let panel_y = 20;
    let panel_w = 348;
    let panel_h = 408;

    Rectangle::new(Point::new(panel_x, panel_y), Size::new(panel_w, panel_h))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 40)))
        .draw(display)?;

    Rectangle::new(Point::new(panel_x, panel_y), Size::new(panel_w, panel_h))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    let mut y = panel_y + 20;

    // Title
    draw_text(
        display,
        "AUTO FARM",
        Point::new(panel_x + 110, y),
        &FONT_10X20,
        COLOR_TEXT,
    )?;
    y += 30;

    // Enemy name
    let mut enemy_str = String::<32>::new();
    write!(enemy_str, "vs {} Lv.{}", enemy.name, enemy.level).ok();
    draw_text(
        display,
        &enemy_str,
        Point::new(panel_x + 90, y),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;
    y += 25;

    // Calculate efficiency
    let (rating, _power_ratio, hero_power, enemy_power) =
        calculate_efficiency(&game_state.hero, enemy);

    // Power comparison
    let mut power_str = String::<48>::new();
    write!(power_str, "Power: {:.0} vs {:.0}", hero_power, enemy_power).ok();
    draw_text(
        display,
        &power_str,
        Point::new(panel_x + 70, y),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    y += 20;

    // Efficiency rating
    let mut rating_str = String::<48>::new();
    write!(
        rating_str,
        "Efficiency: {} {}",
        rating.icon(),
        rating.display_name()
    )
    .ok();
    let rating_color = match rating {
        crate::combat::EfficiencyRating::Excellent => Rgb888::new(100, 255, 100),
        crate::combat::EfficiencyRating::Good => Rgb888::new(150, 255, 150),
        crate::combat::EfficiencyRating::Fair => Rgb888::new(200, 200, 100),
        crate::combat::EfficiencyRating::Risky => Rgb888::new(255, 150, 50),
        crate::combat::EfficiencyRating::Impossible => Rgb888::new(255, 50, 50),
    };
    draw_text(
        display,
        &rating_str,
        Point::new(panel_x + 50, y),
        &FONT_9X18_BOLD,
        rating_color,
    )?;
    y += 30;

    // Check if farming is allowed
    if !rating.is_allowed() {
        draw_text(
            display,
            "Too dangerous!",
            Point::new(panel_x + 85, y),
            &FONT_9X18_BOLD,
            Rgb888::RED,
        )?;
        y += 20;
        draw_text(
            display,
            "Get stronger first",
            Point::new(panel_x + 75, y),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    } else {
        // Duration options
        draw_text(
            display,
            "Select Duration:",
            Point::new(panel_x + 90, y),
            &FONT_9X15,
            COLOR_TEXT,
        )?;
        y += 25;

        let durations = [
            FarmDuration::OneMinute,
            FarmDuration::FiveMinutes,
            FarmDuration::TenMinutes,
        ];

        for (i, duration) in durations.iter().enumerate() {
            let btn_y = y + (i as i32 * 85);
            let expected_kills = calculate_expected_kills(rating, *duration);
            let (exp_reward, zeny_reward) =
                calculate_farm_rewards(enemy, expected_kills, game_state.hero.level);

            // Check if player has enough SP
            let sp_cost = duration.sp_cost();
            let can_afford = game_state.hero.sp >= sp_cost;
            let btn_color = if can_afford {
                Rgb888::new(40, 70, 40)
            } else {
                Rgb888::new(50, 50, 50)
            };

            // Button background
            Rectangle::new(Point::new(panel_x + 15, btn_y), Size::new(318, 75))
                .into_styled(PrimitiveStyle::with_fill(btn_color))
                .draw(display)?;

            Rectangle::new(Point::new(panel_x + 15, btn_y), Size::new(318, 75))
                .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 1))
                .draw(display)?;

            // Duration name
            draw_text(
                display,
                duration.display_name(),
                Point::new(panel_x + 25, btn_y + 18),
                &FONT_9X18_BOLD,
                if can_afford {
                    COLOR_TEXT
                } else {
                    COLOR_TEXT_DIM
                },
            )?;

            // SP cost
            let mut sp_str = String::<16>::new();
            write!(sp_str, "{}SP", sp_cost).ok();
            draw_text(
                display,
                &sp_str,
                Point::new(panel_x + 285, btn_y + 18),
                &FONT_9X15,
                if can_afford { COLOR_SP } else { COLOR_TEXT_DIM },
            )?;

            // Expected kills
            let mut kills_str = String::<32>::new();
            write!(kills_str, "~{} kills", expected_kills).ok();
            draw_text(
                display,
                &kills_str,
                Point::new(panel_x + 25, btn_y + 40),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            // Rewards
            let mut reward_str = String::<32>::new();
            write!(reward_str, "{}exp, {}z", exp_reward, zeny_reward).ok();
            draw_text(
                display,
                &reward_str,
                Point::new(panel_x + 25, btn_y + 58),
                &FONT_9X15,
                COLOR_EXP,
            )?;
        }
    }

    // Close button at bottom
    let close_y = panel_y + panel_h as i32 - 45;
    Rectangle::new(Point::new(panel_x + 100, close_y), Size::new(148, 35))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
        .draw(display)?;
    draw_text(
        display,
        "Cancel",
        Point::new(panel_x + 135, close_y + 22),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    Ok(())
}
