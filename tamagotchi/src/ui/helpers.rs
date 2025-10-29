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

use crate::core::GameState;
use super::colors::*;

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
    let gif_data = game_state.hero_animation.gif_data();
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
    draw_text(
        display,
        slot_name,
        position,
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Equipment name with refine level
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

    // Stats display (compact, 2 lines)
    let mut stats_str1 = String::<64>::new();
    let mut stats_str2 = String::<64>::new();

    // Build stat string based on equipment bonuses
    if equipment.atk_bonus > 0 {
        let total_atk = equipment.total_atk();
        if equipment.refine_level > 0 {
            write!(stats_str1, "ATK:{}({}+{}) ", total_atk, equipment.atk_bonus, equipment.get_refine_bonus()).ok();
        } else {
            write!(stats_str1, "ATK:{} ", total_atk).ok();
        }
    }
    if equipment.def_bonus > 0 {
        let total_def = equipment.total_def();
        if equipment.refine_level > 0 {
            write!(stats_str1, "DEF:{}({}+{}) ", total_def, equipment.def_bonus, equipment.get_refine_bonus()).ok();
        } else {
            write!(stats_str1, "DEF:{} ", total_def).ok();
        }
    }
    if equipment.hp_bonus > 0 {
        write!(stats_str1, "HP+{} ", equipment.hp_bonus).ok();
    }
    if equipment.sp_bonus > 0 {
        write!(stats_str1, "SP+{} ", equipment.sp_bonus).ok();
    }

    // Secondary stats
    if equipment.str_bonus != 0 {
        write!(stats_str2, "STR{:+} ", equipment.str_bonus).ok();
    }
    if equipment.agi_bonus != 0 {
        write!(stats_str2, "AGI{:+} ", equipment.agi_bonus).ok();
    }
    if equipment.vit_bonus != 0 {
        write!(stats_str2, "VIT{:+} ", equipment.vit_bonus).ok();
    }
    if equipment.int_bonus != 0 {
        write!(stats_str2, "INT{:+} ", equipment.int_bonus).ok();
    }
    if equipment.dex_bonus != 0 {
        write!(stats_str2, "DEX{:+} ", equipment.dex_bonus).ok();
    }
    if equipment.luk_bonus != 0 {
        write!(stats_str2, "LUK{:+} ", equipment.luk_bonus).ok();
    }

    // Display stats
    if !stats_str1.is_empty() {
        draw_text(
            display,
            &stats_str1,
            Point::new(position.x + 5, position.y + 42),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }
    if !stats_str2.is_empty() {
        draw_text(
            display,
            &stats_str2,
            Point::new(position.x + 5, position.y + 58),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    Ok(())
}

/// Draw the equipment selection menu overlay
pub fn draw_equipment_selection<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Semi-transparent overlay
    Rectangle::new(Point::new(0, 0), Size::new(368, 448))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
        .draw(display)?;

    // Popup panel (centered, slightly larger for 3 equipment slots)
    let panel_x = 30;
    let panel_y = 80;
    let panel_width = 308;
    let panel_height = 280;

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
        "SELECT EQUIPMENT",
        Point::new(70, 100),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Draw 3 equipment slot buttons (Weapon, Armor, Accessory)
    let slots = [
        (crate::tamagotchi::models::EquipmentSlot::Weapon, "WEAPON"),
        (crate::tamagotchi::models::EquipmentSlot::Armor, "ARMOR"),
        (crate::tamagotchi::models::EquipmentSlot::Accessory, "ACCESSORY"),
    ];

    for (i, (slot, label)) in slots.iter().enumerate() {
        let btn_y = 130 + i as i32 * 60;
        let btn_x = 50;
        let btn_width = 268u32;
        let btn_height = 50u32;

        // Get equipment for this slot
        let equipment = game_state.hero.get_equipment(*slot);

        // Button background
        Rectangle::new(Point::new(btn_x, btn_y), Size::new(btn_width, btn_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 60, 80)))
            .draw(display)?;

        Rectangle::new(Point::new(btn_x, btn_y), Size::new(btn_width, btn_height))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
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
                Point::new(btn_x + 10, btn_y + 20),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            draw_text(
                display,
                &name_str,
                Point::new(btn_x + 10, btn_y + 38),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
        } else {
            // No equipment in this slot
            draw_text(
                display,
                label,
                Point::new(btn_x + 10, btn_y + 20),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            draw_text(
                display,
                "(Empty)",
                Point::new(btn_x + 10, btn_y + 38),
                &FONT_9X15,
                Rgb888::RED,
            )?;
        }
    }

    // Cancel button at bottom
    let cancel_btn_y = 310;
    Rectangle::new(Point::new(120, cancel_btn_y), Size::new(128, 40))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
        .draw(display)?;

    Rectangle::new(Point::new(120, cancel_btn_y), Size::new(128, 40))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 2))
        .draw(display)?;

    draw_text(
        display,
        "CANCEL",
        Point::new(145, cancel_btn_y + 22),
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
        EquipmentSlot::Accessory => &hero.equipped_accessory,
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
        draw_text(
            display,
            msg,
            Point::new(80, 195),
            &FONT_9X18_BOLD,
            color,
        )?;
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
                EquipmentSlot::Weapon => 2,  // +2 ATK
                EquipmentSlot::Armor => 1,   // +1 DEF
                EquipmentSlot::Accessory => 1, // +1 stat
            };
            let stat_name = match slot {
                EquipmentSlot::Weapon => "ATK",
                EquipmentSlot::Armor => "DEF",
                EquipmentSlot::Accessory => "Stat",
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

