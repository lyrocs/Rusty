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

use crate::tamagotchi::models::{
    BattleState, CircleType, Enemy, FarmState, GameState, LocationType, MapHelper, RestState,
};

// Color palette inspired by Ragnarok Online
pub const COLOR_BG: Rgb888 = Rgb888::new(40, 40, 60);
pub const COLOR_PANEL: Rgb888 = Rgb888::new(60, 60, 80);
pub const COLOR_TEXT: Rgb888 = Rgb888::new(255, 255, 255);
pub const COLOR_TEXT_DIM: Rgb888 = Rgb888::new(180, 180, 200);
pub const COLOR_HP: Rgb888 = Rgb888::new(220, 50, 50);
pub const COLOR_SP: Rgb888 = Rgb888::new(50, 120, 220);
pub const COLOR_EXP: Rgb888 = Rgb888::new(255, 200, 50);
pub const COLOR_MENU_BG: Rgb888 = Rgb888::new(30, 30, 50);
pub const COLOR_MENU_SELECT: Rgb888 = Rgb888::new(100, 150, 255);

/// Draw the Overview page showing hero stats
pub fn draw_overview_page<D>(
    display: &mut D,
    game_state: &GameState,
    save_msg: Option<&str>,
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
        "=== HERO STATUS ===",
        Point::new(60, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // LEFT COLUMN: Class, Level, Zeny
    let mut name_str = String::<32>::new();
    write!(name_str, "{}", hero.name).ok();
    draw_text(
        display,
        &name_str,
        Point::new(20, 60),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    let mut job_str = String::<32>::new();
    write!(job_str, "Job: {}", hero.job).ok();
    draw_text(
        display,
        &job_str,
        Point::new(20, 85),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    let mut lvl_str = String::<32>::new();
    write!(lvl_str, "Lv. {}", hero.level).ok();
    draw_text(
        display,
        &lvl_str,
        Point::new(20, 110),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "{}z", hero.zeny).ok();
    draw_text(
        display,
        &zeny_str,
        Point::new(20, 135),
        &FONT_9X18_BOLD,
        Rgb888::YELLOW,
    )?;

    // RIGHT COLUMN: HP, SP, EXP (compact with smaller bars)
    // HP
    draw_text(
        display,
        "HP:",
        Point::new(200, 60),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", hero.hp, hero.max_hp).ok();
    draw_text(display, &hp_str, Point::new(235, 60), &FONT_9X15, COLOR_HP)?;
    draw_bar(
        display,
        Point::new(200, 75),
        150,
        hero.hp_percent(),
        COLOR_HP,
    )?;

    // SP
    draw_text(
        display,
        "SP:",
        Point::new(200, 95),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", hero.sp, hero.max_sp).ok();
    draw_text(display, &sp_str, Point::new(235, 95), &FONT_9X15, COLOR_SP)?;
    draw_bar(
        display,
        Point::new(200, 110),
        150,
        hero.sp_percent(),
        COLOR_SP,
    )?;

    // EXP
    draw_text(
        display,
        "EXP:",
        Point::new(200, 130),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    let mut exp_str = String::<32>::new();
    write!(exp_str, "{}/{}", hero.exp, hero.exp_to_next_level).ok();
    draw_text(
        display,
        &exp_str,
        Point::new(245, 130),
        &FONT_9X15,
        COLOR_EXP,
    )?;
    draw_bar(
        display,
        Point::new(200, 145),
        150,
        hero.exp_percent(),
        COLOR_EXP,
    )?;

    // CENTER: Hero GIF (sitting animation)
    draw_hero_gif(display, game_state, Point::new(184, 280))?;

    // Save status message (if any)
    if let Some(msg) = save_msg {
        draw_text(
            display,
            msg,
            Point::new(110, 310),
            &FONT_9X18_BOLD,
            Rgb888::YELLOW,
        )?;
    }

    // Buttons at bottom (2 rows x 2 buttons)
    // Row 1: Rest, Stats
    // Rest button (top left)
    Rectangle::new(Point::new(14, 350), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 150, 100)))
        .draw(display)?;
    draw_text(
        display,
        "Rest",
        Point::new(75, 368),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Stats button (top right)
    Rectangle::new(Point::new(189, 350), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 100, 150)))
        .draw(display)?;
    draw_text(
        display,
        "Stats",
        Point::new(245, 368),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Row 2: Equipment, Items
    // Equipment button (bottom left)
    Rectangle::new(Point::new(14, 403), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(180, 140, 80)))
        .draw(display)?;
    draw_text(
        display,
        "Equip",
        Point::new(65, 421),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Quests button (bottom right)
    Rectangle::new(Point::new(189, 403), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 100, 150)))
        .draw(display)?;
    draw_text(
        display,
        "Quests",
        Point::new(225, 421),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    Ok(())
}

/// Draw the Stats page for stat allocation
pub fn draw_stats_page<D>(
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
        "=== STAT POINTS ===",
        Point::new(50, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Available stat points
    let mut points_str = String::<32>::new();
    write!(points_str, "Available: {}", hero.stat_points).ok();
    draw_text(
        display,
        &points_str,
        Point::new(90, 50),
        &FONT_9X18_BOLD,
        COLOR_EXP,
    )?;

    // Info text
    draw_text(
        display,
        "Tap to add stat points",
        Point::new(60, 80),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Stat display and increase buttons (2 columns x 3 rows)
    let left_x = 20;
    let right_x = 190;
    let button_width = 150;
    let button_height = 70;
    let y_positions = [110, 185, 260];

    // Left column: STR, AGI, VIT
    let left_stats = [
        ("STR", hero.base_str, Rgb888::new(200, 80, 80)),
        ("AGI", hero.base_agi, Rgb888::new(80, 200, 80)),
        ("VIT", hero.base_vit, Rgb888::new(200, 150, 80)),
    ];

    for (i, (stat_name, stat_value, color)) in left_stats.iter().enumerate() {
        let y = y_positions[i];

        // Button background
        let button_color = if hero.stat_points > 0 {
            *color
        } else {
            Rgb888::new(80, 80, 80) // Grayed out if no points
        };

        Rectangle::new(Point::new(left_x, y), Size::new(button_width as u32, button_height))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        // Stat text (centered vertically in button)
        let mut stat_str = String::<32>::new();
        write!(stat_str, "{}: {}", stat_name, stat_value).ok();
        draw_text(
            display,
            &stat_str,
            Point::new(left_x + 10, y + 38),
            &FONT_9X18_BOLD,
            Rgb888::WHITE,
        )?;
    }

    // Right column: INT, DEX, LUK
    let right_stats = [
        ("INT", hero.base_int, Rgb888::new(80, 80, 200)),
        ("DEX", hero.base_dex, Rgb888::new(200, 80, 200)),
        ("LUK", hero.base_luk, Rgb888::new(200, 200, 80)),
    ];

    for (i, (stat_name, stat_value, color)) in right_stats.iter().enumerate() {
        let y = y_positions[i];

        // Button background
        let button_color = if hero.stat_points > 0 {
            *color
        } else {
            Rgb888::new(80, 80, 80) // Grayed out if no points
        };

        Rectangle::new(Point::new(right_x, y), Size::new(button_width as u32, button_height))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        // Stat text (centered vertically in button)
        let mut stat_str = String::<32>::new();
        write!(stat_str, "{}: {}", stat_name, stat_value).ok();
        draw_text(
            display,
            &stat_str,
            Point::new(right_x + 10, y + 38),
            &FONT_9X18_BOLD,
            Rgb888::WHITE,
        )?;
    }

    // Reset button
    Rectangle::new(Point::new(90, 345), Size::new(180, 45))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(180, 50, 50)))
        .draw(display)?;
    draw_text(
        display,
        "RESET ALL",
        Point::new(110, 363),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Back button
    Rectangle::new(Point::new(100, 400), Size::new(160, 40))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 100, 150)))
        .draw(display)?;
    draw_text(
        display,
        "Back",
        Point::new(155, 418),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    Ok(())
}

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

    // Title
    draw_text(
        display,
        "=== EQUIPMENT ===",
        Point::new(60, 20),
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

    // Equipment display (3 slots, stacked vertically)
    let start_y = 80;
    let spacing = 100;

    // WEAPON
    draw_equipment_slot(
        display,
        &hero.equipped_weapon,
        Point::new(20, start_y),
        "WEAPON",
    )?;

    // ARMOR
    draw_equipment_slot(
        display,
        &hero.equipped_armor,
        Point::new(20, start_y + spacing),
        "ARMOR",
    )?;

    // ACCESSORY
    draw_equipment_slot(
        display,
        &hero.equipped_accessory,
        Point::new(20, start_y + (spacing * 2)),
        "ACCESSORY",
    )?;

    // Back button
    Rectangle::new(Point::new(100, 400), Size::new(160, 40))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 100, 150)))
        .draw(display)?;
    draw_text(
        display,
        "Back",
        Point::new(155, 418),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Draw refine popup if open
    if game_state.refine_popup_open {
        if let Some(slot) = game_state.refine_slot {
            draw_refine_popup(display, game_state, slot)?;
        }
    }

    Ok(())
}

/// Draw a single equipment slot
fn draw_equipment_slot<D>(
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
fn draw_equipment_selection<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
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
fn draw_refine_popup<D>(
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

/// Draw the Farm page with enemy and progress
pub fn draw_farm_page<D>(
    display: &mut D,
    game_state: &GameState,
    _battery_mv: u16,
    _battery_pct: u8,
    fps: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    match game_state.farm_state {
        FarmState::Idle => {
            draw_text(
                display,
                "=== AUTO FARM ===",
                Point::new(70, 20),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            // SP display with color coding
            let mut sp_str = String::<32>::new();
            write!(
                sp_str,
                "SP: {}/{}",
                game_state.hero.sp, game_state.hero.max_sp
            )
            .ok();
            let sp_color = if game_state.hero.sp >= 20 {
                COLOR_SP
            } else {
                COLOR_HP
            };
            draw_text(
                display,
                &sp_str,
                Point::new(20, 60),
                &FONT_9X18_BOLD,
                sp_color,
            )?;

            // SP bar
            draw_bar(
                display,
                Point::new(20, 78),
                328,
                game_state.hero.sp_percent(),
                sp_color,
            )?;

            // Check if user has enough SP
            if game_state.hero.sp >= 20 {
                // Enough SP - show normal instructions
                draw_text(
                    display,
                    "Touch screen to",
                    Point::new(90, 200),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;
                draw_text(
                    display,
                    "start farming",
                    Point::new(95, 225),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                draw_text(
                    display,
                    "Cost: 20 SP",
                    Point::new(110, 280),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "Duration: 1 minute",
                    Point::new(90, 300),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
            } else {
                // Not enough SP - show warning
                draw_text(
                    display,
                    "NOT ENOUGH SP!",
                    Point::new(75, 180),
                    &FONT_10X20,
                    COLOR_HP,
                )?;

                let mut needed_str = String::<32>::new();
                write!(needed_str, "Need {} more SP", 20 - game_state.hero.sp).ok();
                draw_text(
                    display,
                    &needed_str,
                    Point::new(90, 215),
                    &FONT_9X18_BOLD,
                    COLOR_HP,
                )?;

                draw_text(
                    display,
                    "Go to Rest page to",
                    Point::new(75, 265),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "recover SP",
                    Point::new(115, 288),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
            }

            draw_text(
                display,
                "Press BOOT for Menu",
                Point::new(90, 440),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        FarmState::Fighting => {
            if let Some(enemy) = &game_state.current_enemy {
                draw_text(
                    display,
                    "=== FIGHTING ===",
                    Point::new(80, 20),
                    &FONT_10X20,
                    COLOR_TEXT,
                )?;

                // Enemy name
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "{} Lv.{}", enemy.name, enemy.level).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(100, 60),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Draw hero GIF animation (middle-left, lower)
                draw_hero_gif(display, game_state, Point::new(120, 280))?;

                // Draw monster GIF animation with attacked state (middle-right, same level)
                draw_monster_attacked_gif(display, game_state, Point::new(260, 280), enemy.name)?;

                // Progress bar
                draw_text(
                    display,
                    "Progress",
                    Point::new(135, 330),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;
                draw_bar(
                    display,
                    Point::new(20, 355),
                    328,
                    game_state.farm_progress_percent(),
                    COLOR_EXP,
                )?;

                let mut time_str = String::<32>::new();
                let remaining_sec = (game_state.farm_duration_ms - game_state.farm_progress) / 1000;
                write!(time_str, "{}s remaining", remaining_sec).ok();
                draw_text(
                    display,
                    &time_str,
                    Point::new(120, 375),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // Potential rewards
                let mut reward_str = String::<32>::new();
                write!(
                    reward_str,
                    "Rewards: EXP {} | Zeny {}",
                    enemy.base_exp, enemy.zeny_reward
                )
                .ok();
                draw_text(
                    display,
                    &reward_str,
                    Point::new(30, 405),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;

                // FPS display at bottom
                draw_fps_info(display, Point::new(10, 425), fps)?;
            }
        }
        FarmState::Victory => {
            draw_text(
                display,
                "=== VICTORY! ===",
                Point::new(80, 20),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            if let Some(enemy) = &game_state.current_enemy {
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "Defeated {}", enemy.name).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(85, 60),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Draw dying monster GIF animation (centered)
                draw_monster_gif(display, game_state, Point::new(120, 110), enemy.name)?;

                draw_text(
                    display,
                    "Rewards:",
                    Point::new(130, 280),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                let mut exp_str = String::<32>::new();
                write!(exp_str, "+{} EXP", enemy.base_exp).ok();
                draw_text(
                    display,
                    &exp_str,
                    Point::new(115, 310),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                let mut zeny_str = String::<32>::new();
                write!(zeny_str, "+{} Zeny", enemy.zeny_reward).ok();
                draw_text(
                    display,
                    &zeny_str,
                    Point::new(115, 340),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                // Display loot if any
                if !game_state.last_drops.is_empty() {
                    draw_text(
                        display,
                        "Items:",
                        Point::new(140, 380),
                        &FONT_9X18_BOLD,
                        Rgb888::YELLOW,
                    )?;

                    let mut y = 410;
                    for (_, item_name, quantity) in &game_state.last_drops {
                        let mut item_str = String::<48>::new();
                        write!(item_str, "{} x{}", item_name, quantity).ok();
                        draw_text(
                            display,
                            &item_str,
                            Point::new(100, y),
                            &FONT_9X15,
                            Rgb888::YELLOW,
                        )?;
                        y += 20;
                        if y > 450 {
                            break; // Don't overflow screen
                        }
                    }
                } else {
                    draw_text(
                        display,
                        "No items dropped",
                        Point::new(90, 400),
                        &FONT_9X15,
                        COLOR_TEXT_DIM,
                    )?;
                }
            }

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 440),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        FarmState::Defeat => {
            draw_text(
                display,
                "=== DEFEATED ===",
                Point::new(80, 100),
                &FONT_10X20,
                COLOR_HP,
            )?;

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 420),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    Ok(())
}

/// Draw the Rest/Sit page for HP and SP regeneration
pub fn draw_rest_page<D>(
    display: &mut D,
    game_state: &GameState,
    battery_mv: u16,
    battery_pct: u8,
    fps: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    draw_text(
        display,
        "=== RESTING ===",
        Point::new(90, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Hero resting GIF animation (16.gif)
    draw_hero_gif(display, game_state, Point::new(120, 100))?;

    // HP bar
    draw_text(
        display,
        "HP Recovery",
        Point::new(105, 160),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", game_state.hero.hp, game_state.hero.max_hp).ok();
    draw_text(
        display,
        &hp_str,
        Point::new(125, 180),
        &FONT_9X18_BOLD,
        COLOR_HP,
    )?;
    draw_bar(
        display,
        Point::new(20, 195),
        328,
        game_state.hero.hp_percent(),
        COLOR_HP,
    )?;

    // HP Regen rate
    draw_text(
        display,
        "+10 HP/sec",
        Point::new(120, 215),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // SP bar
    draw_text(
        display,
        "SP Recovery",
        Point::new(105, 245),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", game_state.hero.sp, game_state.hero.max_sp).ok();
    draw_text(
        display,
        &sp_str,
        Point::new(125, 265),
        &FONT_9X18_BOLD,
        COLOR_SP,
    )?;
    draw_bar(
        display,
        Point::new(20, 280),
        328,
        game_state.hero.sp_percent(),
        COLOR_SP,
    )?;

    // SP Regen rate
    let mut sp_regen_str = String::<32>::new();
    write!(sp_regen_str, "+{} SP/sec", game_state.sp_regen_rate).ok();
    draw_text(
        display,
        &sp_regen_str,
        Point::new(120, 300),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    match game_state.rest_state {
        RestState::Resting => {
            draw_text(
                display,
                "Recovering HP & SP...",
                Point::new(65, 330),
                &FONT_9X18_BOLD,
                COLOR_TEXT_DIM,
            )?;
        }
        RestState::FullSP => {
            draw_text(
                display,
                "Fully Recovered!",
                Point::new(75, 330),
                &FONT_9X18_BOLD,
                COLOR_TEXT,
            )?;
            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 355),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    // Battery info
    draw_battery_info(display, Point::new(20, 360), battery_mv, battery_pct)?;

    // FPS info
    draw_fps_info(display, Point::new(230, 360), fps)?;

    draw_text(
        display,
        "Press BOOT for Menu",
        Point::new(90, 420),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw the Battle (Whac-A-Mole) page
pub fn draw_battle_page<D>(
    display: &mut D,
    game_state: &GameState,
    _battery_mv: u16,
    _battery_pct: u8,
    fps: u32,
    should_clear: bool,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Only clear screen when needed (entering battle or state change)
    // During active gameplay, only clear if should_clear is true
    if should_clear || game_state.battle_state != BattleState::Playing {
        display.clear(COLOR_BG)?;
    }

    match game_state.battle_state {
        BattleState::Idle => {
            draw_text(
                display,
                "=== BATTLE ===",
                Point::new(85, 20),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            // SP display
            let mut sp_str = String::<32>::new();
            write!(
                sp_str,
                "SP: {}/{}",
                game_state.hero.sp, game_state.hero.max_sp
            )
            .ok();
            let sp_color = if game_state.hero.sp >= 20 {
                COLOR_SP
            } else {
                COLOR_HP
            };
            draw_text(
                display,
                &sp_str,
                Point::new(20, 60),
                &FONT_9X18_BOLD,
                sp_color,
            )?;
            draw_bar(
                display,
                Point::new(20, 78),
                328,
                game_state.hero.sp_percent(),
                sp_color,
            )?;

            if game_state.hero.sp >= 20 {
                draw_text(
                    display,
                    "Touch screen to",
                    Point::new(90, 160),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;
                draw_text(
                    display,
                    "start battle!",
                    Point::new(100, 185),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                draw_text(
                    display,
                    "Cost: 20 SP",
                    Point::new(110, 230),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "Duration: 30 seconds",
                    Point::new(75, 250),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;
            } else {
                draw_text(
                    display,
                    "NOT ENOUGH SP!",
                    Point::new(75, 150),
                    &FONT_10X20,
                    COLOR_HP,
                )?;
                let mut needed_str = String::<32>::new();
                write!(needed_str, "Need {} more SP", 20 - game_state.hero.sp).ok();
                draw_text(
                    display,
                    &needed_str,
                    Point::new(90, 185),
                    &FONT_9X18_BOLD,
                    COLOR_HP,
                )?;
                draw_text(
                    display,
                    "Go to Rest page to",
                    Point::new(75, 225),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
                draw_text(
                    display,
                    "recover SP",
                    Point::new(115, 248),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT_DIM,
                )?;
            }

            draw_text(
                display,
                "Press BOOT for Menu",
                Point::new(90, 420),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        BattleState::Playing => {
            if let Some(enemy) = &game_state.battle_enemy {
                // Enemy name and level at top
                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "{} Lv.{}", enemy.name, enemy.level).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(100, 60),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Enemy HP bar
                draw_bar(
                    display,
                    Point::new(60, 100),
                    250,
                    enemy.hp_percent(),
                    COLOR_HP,
                )?;

                // No GIF animations during manual battle for better gameplay focus

                // Timer (top right)
                let remaining_sec = (game_state.battle_duration - game_state.battle_elapsed) / 1000;
                let mut time_str = String::<16>::new();
                write!(time_str, "{}s", remaining_sec).ok();
                draw_text(
                    display,
                    &time_str,
                    Point::new(315, 20),
                    &FONT_10X20,
                    Rgb888::YELLOW,
                )?;

                // Score and Combo (top area - no GIF during gameplay for performance)
                let mut score_str = String::<48>::new();
                write!(
                    score_str,
                    "Hits:{} Miss:{} x{}",
                    game_state.battle_score, game_state.battle_missed, game_state.battle_combo
                )
                .ok();
                draw_text(
                    display,
                    &score_str,
                    Point::new(45, 140),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // Draw all active circles
                for circle in &game_state.battle_circles {
                    if let Some(c) = circle {
                        let color = match c.circle_type {
                            CircleType::GoodTarget => Rgb888::GREEN,
                            CircleType::BadTarget => Rgb888::RED,
                        };

                        // Draw only colored border (no fill)
                        EgCircle::new(
                            Point::new(c.x - c.radius as i32, c.y - c.radius as i32),
                            c.radius * 2,
                        )
                        .into_styled(PrimitiveStyle::with_stroke(color, 3))
                        .draw(display)?;
                    }
                }

                // Draw touch indicator cross (shows for 500ms after touch)
                if game_state.battle_last_touch_time > 0 {
                    let time_since_touch = game_state
                        .last_update_ms
                        .saturating_sub(game_state.battle_last_touch_time);
                    if time_since_touch < 500 {
                        let tx = game_state.battle_last_touch_x;
                        let ty = game_state.battle_last_touch_y;
                        let cross_size = 10;

                        // Draw white cross at touch position
                        Line::new(
                            Point::new(tx - cross_size, ty),
                            Point::new(tx + cross_size, ty),
                        )
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 3))
                        .draw(display)?;

                        Line::new(
                            Point::new(tx, ty - cross_size),
                            Point::new(tx, ty + cross_size),
                        )
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 3))
                        .draw(display)?;
                    }
                }

                // Instructions at bottom
                draw_text(
                    display,
                    "Green: Hit  Red: Block",
                    Point::new(60, 395),
                    &FONT_9X15,
                    COLOR_TEXT_DIM,
                )?;

                // FPS display at bottom
                draw_fps_info(display, Point::new(10, 415), fps)?;
            }
        }
        BattleState::Victory => {
            draw_text(
                display,
                "=== VICTORY! ===",
                Point::new(75, 60),
                &FONT_10X20,
                COLOR_TEXT,
            )?;

            if let Some(enemy) = &game_state.battle_enemy {
                // Draw dying monster GIF animation (centered)
                draw_monster_gif(display, game_state, Point::new(120, 110), enemy.name)?;

                let mut enemy_str = String::<32>::new();
                write!(enemy_str, "Defeated {}", enemy.name).ok();
                draw_text(
                    display,
                    &enemy_str,
                    Point::new(85, 220),
                    &FONT_9X18_BOLD,
                    COLOR_TEXT,
                )?;

                // Score
                let mut score_str = String::<32>::new();
                write!(score_str, "Hits: {}", game_state.battle_score).ok();
                draw_text(
                    display,
                    &score_str,
                    Point::new(110, 250),
                    &FONT_9X15,
                    COLOR_TEXT,
                )?;

                draw_text(
                    display,
                    "Rewards:",
                    Point::new(120, 285),
                    &FONT_9X18_BOLD,
                    COLOR_EXP,
                )?;

                let mut exp_str = String::<32>::new();
                write!(exp_str, "+{} EXP", enemy.base_exp).ok();
                draw_text(
                    display,
                    &exp_str,
                    Point::new(105, 310),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;

                let mut zeny_str = String::<32>::new();
                write!(zeny_str, "+{} Zeny", enemy.zeny_reward).ok();
                draw_text(
                    display,
                    &zeny_str,
                    Point::new(105, 330),
                    &FONT_9X15,
                    COLOR_EXP,
                )?;

                // Display loot if any
                if !game_state.last_drops.is_empty() {
                    draw_text(
                        display,
                        "Items:",
                        Point::new(140, 360),
                        &FONT_9X15,
                        Rgb888::YELLOW,
                    )?;

                    let mut y = 380;
                    for (_, item_name, quantity) in &game_state.last_drops {
                        let mut item_str = String::<40>::new();
                        write!(item_str, "{} x{}", item_name, quantity).ok();
                        draw_text(
                            display,
                            &item_str,
                            Point::new(80, y),
                            &FONT_9X15,
                            Rgb888::YELLOW,
                        )?;
                        y += 18;
                        if y > 410 {
                            break; // Don't overflow screen
                        }
                    }
                }
            }

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 420),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
        BattleState::Defeat => {
            draw_text(
                display,
                "=== DEFEATED ===",
                Point::new(75, 150),
                &FONT_10X20,
                COLOR_HP,
            )?;

            // Score
            let mut score_str = String::<32>::new();
            write!(score_str, "Hits: {}", game_state.battle_score).ok();
            draw_text(
                display,
                &score_str,
                Point::new(110, 220),
                &FONT_9X18_BOLD,
                COLOR_TEXT,
            )?;

            let mut missed_str = String::<32>::new();
            write!(missed_str, "Missed: {}", game_state.battle_missed).ok();
            draw_text(
                display,
                &missed_str,
                Point::new(95, 250),
                &FONT_9X18_BOLD,
                COLOR_HP,
            )?;

            draw_text(
                display,
                "You were defeated!",
                Point::new(70, 300),
                &FONT_9X18_BOLD,
                COLOR_HP,
            )?;
            draw_text(
                display,
                "No rewards",
                Point::new(110, 330),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;

            draw_text(
                display,
                "Touch to continue",
                Point::new(90, 420),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    Ok(())
}

/// Draw the Map/Navigation page
pub fn draw_map_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    let map_id = game_state.current_location;

    // Draw map background image if available
    if let Some(map_bg_data) = crate::tamagotchi::models::get_map_background(map_id) {
        let gif = Gif::<Rgb888>::from_slice(map_bg_data).expect("Failed to parse map GIF");

        // Get GIF dimensions
        let gif_width = gif.width() as i32;
        let gif_height = gif.height() as i32;

        // Center the background on screen (368x448 display)
        let top_left = Point::new((368 - gif_width) / 2, (448 - gif_height) / 2);

        // Render first (and only) frame of the map GIF
        if let Some(frame) = gif.frames().next() {
            Image::new(&frame, top_left).draw(display)?;
        }
    }

    let location_type = MapHelper::location_type(map_id);

    // Title with location name
    let mut title = String::<32>::new();
    write!(title, "=== {} ===", MapHelper::name(map_id)).ok();
    draw_text(display, &title, Point::new(60, 20), &FONT_10X20, COLOR_TEXT)?;

    // Draw directional navigation indicators (blue circles at borders)
    let exits = MapHelper::exits(map_id);
    for exit in exits {
        match exit.direction {
            "North" => {
                // Top circle
                EgCircle::new(Point::new(164, 5), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            "South" => {
                // Bottom circle
                EgCircle::new(Point::new(164, 413), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            "West" => {
                // Left circle
                EgCircle::new(Point::new(10, 209), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            "East" => {
                // Right circle
                EgCircle::new(Point::new(328, 209), 30)
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::BLUE))
                    .draw(display)?;
            }
            _ => {}
        }
    }

    // Center area for info and actions
    match location_type {
        LocationType::City => {
            // Show NPC actions as buttons (similar to menu)
            let npcs = MapHelper::npcs(map_id);
            if !npcs.is_empty() {
                for (i, npc) in npcs.iter().enumerate() {
                    let row = i / 2;
                    let col = i % 2;
                    let x = 59 + col as i32 * 130; // Centered buttons
                    let y = 100 + row as i32 * 75;

                    // Draw action button
                    Rectangle::new(Point::new(x, y), Size::new(120, 60))
                        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                        .draw(display)?;
                    Rectangle::new(Point::new(x, y), Size::new(120, 60))
                        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
                        .draw(display)?;

                    // Text (word wrap for long names)
                    if npc.len() > 10 {
                        // Split long text
                        if let Some(space_idx) = npc.find(' ') {
                            let (first, second) = npc.split_at(space_idx);
                            draw_text(
                                display,
                                first,
                                Point::new(x + 10, y + 20),
                                &FONT_9X15,
                                COLOR_TEXT,
                            )?;
                            draw_text(
                                display,
                                second.trim(),
                                Point::new(x + 10, y + 38),
                                &FONT_9X15,
                                COLOR_TEXT_DIM,
                            )?;
                        } else {
                            draw_text(
                                display,
                                npc,
                                Point::new(x + 10, y + 25),
                                &FONT_9X15,
                                COLOR_TEXT,
                            )?;
                        }
                    } else {
                        draw_text(
                            display,
                            npc,
                            Point::new(x + 10, y + 25),
                            &FONT_9X15,
                            COLOR_TEXT,
                        )?;
                    }
                }
            }
        }
        LocationType::Field => {
            // Show monster GIF animations on the map
            let enemy_ids = MapHelper::enemies(map_id);
            if !enemy_ids.is_empty() {
                // Display up to 4 monsters with their GIF animations
                for (i, &enemy_id) in enemy_ids.iter().enumerate().take(4) {
                    if let Some(enemy) = Enemy::from_id(enemy_id) {
                        // Calculate position for monsters (2x2 grid in center)
                        let col = i % 2;
                        let row = i / 2;
                        let x = 90 + col as i32 * 100;
                        let y = 140 + row as i32 * 100;
                        let center = Point::new(x, y);

                        // Draw monster name in black with white background above GIF
                        let name_x = center.x - (enemy.name.len() as i32 * 9) / 2;
                        let name_y = center.y - 40;

                        // Draw white background rectangle for name
                        // Note: text y position is at baseline, so background must start higher
                        let name_width = enemy.name.len() as i32 * 9;
                        let bg_padding = 3;
                        let font_height = 18; // FONT_9X18_BOLD height
                        Rectangle::new(
                            Point::new(name_x - bg_padding, name_y - font_height - bg_padding + 2),
                            Size::new(
                                (name_width + bg_padding * 2) as u32,
                                (font_height + bg_padding * 2) as u32,
                            ),
                        )
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
                        .draw(display)?;

                        // Draw black text on top
                        draw_text(
                            display,
                            enemy.name,
                            Point::new(name_x, name_y),
                            &FONT_9X18_BOLD,
                            Rgb888::BLACK,
                        )?;

                        // Draw monster idle GIF (0.gif)
                        draw_map_monster_gif(display, game_state, center, enemy.name)?;
                    }
                }

                // Action buttons (centered, higher to leave space for bottom navigation)
                // Auto Farm button
                Rectangle::new(Point::new(84, 280), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
                    .draw(display)?;
                Rectangle::new(Point::new(84, 280), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "AUTO FARM",
                    Point::new(115, 300),
                    &FONT_9X18_BOLD,
                    Rgb888::WHITE,
                )?;

                // Battle button
                Rectangle::new(Point::new(84, 335), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 50, 50)))
                    .draw(display)?;
                Rectangle::new(Point::new(84, 335), Size::new(200, 50))
                    .into_styled(PrimitiveStyle::with_stroke(Rgb888::RED, 3))
                    .draw(display)?;
                draw_text(
                    display,
                    "BATTLE",
                    Point::new(140, 355),
                    &FONT_9X18_BOLD,
                    Rgb888::WHITE,
                )?;
            }
        }
    }

    // Status message (for SP/HP warnings)
    if let Some(msg) = game_state.save_status_msg {
        draw_text(display, msg, Point::new(60, 390), &FONT_10X20, Rgb888::RED)?;
    }

    // Equipment selection menu overlay (if open)
    if game_state.equipment_selection_open {
        draw_equipment_selection(display, game_state)?;
    }
    // Refine popup overlay (if open)
    else if game_state.refine_popup_open {
        if let Some(slot) = game_state.refine_slot {
            draw_refine_popup(display, game_state, slot)?;
        }
    }

    Ok(())
}

/// Draw the Menu overlay
pub fn draw_menu<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Full-width background panel with padding
    Rectangle::new(Point::new(10, 40), Size::new(348, 368))
        .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_BG))
        .draw(display)?;

    Rectangle::new(Point::new(10, 40), Size::new(348, 368))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 3))
        .draw(display)?;

    draw_text(
        display,
        "=== MENU ===",
        Point::new(115, 70),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Menu items in 2 columns x 3 rows (6 items)
    // Farm and Battle are now accessed via Map page
    // Button size: 150x70 with 10px spacing
    let menu_items = ["Overview", "Rest", "Map", "Quests", "Settings", "Save"];

    for (i, item) in menu_items.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;

        // Calculate button position
        let x = 24 + col as i32 * 160; // 24px left margin, 160px spacing (150 button + 10 gap)
        let y = 110 + row as i32 * 80; // 110px top, 80px spacing (70 button + 10 gap)

        let is_selected = i as u8 == game_state.menu_selection;

        // Draw button background
        let button_color = if is_selected {
            COLOR_MENU_SELECT
        } else {
            COLOR_PANEL
        };

        Rectangle::new(Point::new(x, y), Size::new(150, 70))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        // Draw button border (thicker if selected)
        let border_width = if is_selected { 3 } else { 2 };
        Rectangle::new(Point::new(x, y), Size::new(150, 70))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, border_width))
            .draw(display)?;

        // Draw text centered in button
        let text_color = if is_selected {
            COLOR_TEXT
        } else {
            COLOR_TEXT_DIM
        };

        // Calculate text centering (rough approximation)
        let text_len = item.len() as i32;
        let text_x = x + (150 - text_len * 9) / 2; // 9px per char for FONT_9X18_BOLD
        let text_y = y + 30; // Center vertically in 70px button

        draw_text(
            display,
            item,
            Point::new(text_x, text_y),
            &FONT_9X18_BOLD,
            text_color,
        )?;
    }

    draw_text(
        display,
        "Touch button to select",
        Point::new(75, 360),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;
    draw_text(
        display,
        "BOOT to close menu",
        Point::new(80, 385),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw inventory page showing all collected items
pub fn draw_inventory<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear display
    display.clear(COLOR_BG)?;

    // Header
    draw_text(
        display,
        "=== INVENTORY ===",
        Point::new(85, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Draw item list
    let inventory = &game_state.hero.inventory;

    if inventory.is_empty() {
        draw_text(
            display,
            "No items yet!",
            Point::new(110, 200),
            &FONT_10X20,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "Defeat enemies to earn items",
            Point::new(40, 230),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    } else {
        // Draw items in a scrollable list (show first 15 items)
        let mut y = 60;
        for (i, item) in inventory.iter().take(15).enumerate() {
            let mut item_str = String::<64>::new();
            write!(item_str, "{} x{}", item.name, item.quantity).ok();

            let text_color = if i % 2 == 0 {
                COLOR_TEXT
            } else {
                Rgb888::new(200, 200, 200)
            };

            draw_text(
                display,
                &item_str,
                Point::new(20, y),
                &FONT_9X15,
                text_color,
            )?;

            y += 20;
        }

        // Show count if there are more items
        if inventory.len() > 15 {
            let mut count_str = String::<32>::new();
            write!(count_str, "...and {} more", inventory.len() - 15).ok();
            draw_text(
                display,
                &count_str,
                Point::new(90, y + 10),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    // Footer
    draw_text(
        display,
        "Touch to go back",
        Point::new(90, 440),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Draw the quest page with quest list
pub fn draw_quests_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    display.clear(COLOR_BG)?;

    // Header
    draw_text(
        display,
        "=== QUESTS ===",
        Point::new(100, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Filter active quests (not claimed)
    let active_quests: heapless::Vec<&crate::tamagotchi::models::ActiveQuest, 16> = game_state
        .active_quests
        .iter()
        .filter(|q| !q.claimed)
        .collect();

    if active_quests.is_empty() {
        draw_text(
            display,
            "No active quests!",
            Point::new(90, 200),
            &FONT_10X20,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "Visit the Guild Master",
            Point::new(60, 230),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "in Prontera for quests",
            Point::new(60, 250),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    } else {
        // Draw quest cards (show up to 4 quests)
        let start_index = game_state.quest_page_scroll as usize;
        let visible_quests = active_quests.iter().skip(start_index).take(4);

        let mut card_y = 60;
        for active_quest in visible_quests {
            // Get quest data
            if let Some(quest_data) = crate::tamagotchi::quest_system::get_quest_data(active_quest.quest_id) {
                // Quest card background
                let card_height = 80u32;
                Rectangle::new(Point::new(10, card_y), Size::new(348, card_height))
                    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
                    .draw(display)?;

                Rectangle::new(Point::new(10, card_y), Size::new(348, card_height))
                    .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
                    .draw(display)?;

                // Quest type indicator
                let type_color = match quest_data.quest_type {
                    crate::tamagotchi::models::QuestType::Daily => Rgb888::new(100, 150, 255),
                    crate::tamagotchi::models::QuestType::Story => Rgb888::new(255, 200, 100),
                    crate::tamagotchi::models::QuestType::Achievement => Rgb888::new(255, 100, 255),
                };

                let type_label = match quest_data.quest_type {
                    crate::tamagotchi::models::QuestType::Daily => "DAILY",
                    crate::tamagotchi::models::QuestType::Story => "STORY",
                    crate::tamagotchi::models::QuestType::Achievement => "ACHIEVEMENT",
                };

                draw_text(
                    display,
                    type_label,
                    Point::new(20, card_y + 18),
                    &FONT_9X15,
                    type_color,
                )?;

                // Quest name
                draw_text(
                    display,
                    quest_data.name,
                    Point::new(20, card_y + 35),
                    &FONT_9X15,
                    COLOR_TEXT,
                )?;

                // Progress for first objective
                if !quest_data.objectives.is_empty() && !active_quest.progress.is_empty() {
                    let objective = &quest_data.objectives[0];
                    let progress = active_quest.progress[0];

                    let target = match objective.objective_type {
                        "KillMonster" => objective.count,
                        "CollectItem" => objective.count,
                        "ReachLevel" => objective.level,
                        "EarnZeny" => objective.amount as u16,
                        "RefineEquipment" => objective.count,
                        "CompleteBattles" => objective.count,
                        _ => 0,
                    };

                    // Progress text
                    let mut progress_str = String::<32>::new();
                    write!(progress_str, "{}/{}", progress, target).ok();
                    draw_text(
                        display,
                        &progress_str,
                        Point::new(20, card_y + 52),
                        &FONT_9X15,
                        COLOR_TEXT_DIM,
                    )?;

                    // Progress bar
                    let bar_width = 308;
                    let bar_x = 20;
                    let bar_y = card_y + 58;

                    // Background bar
                    Rectangle::new(Point::new(bar_x, bar_y), Size::new(bar_width, 8))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 40)))
                        .draw(display)?;

                    // Progress bar fill
                    let progress_percent = if target > 0 {
                        (progress as u32 * 100) / target as u32
                    } else {
                        0
                    };
                    let fill_width = (bar_width as u32 * progress_percent / 100) as u32;

                    let bar_color = if active_quest.completed {
                        Rgb888::GREEN
                    } else {
                        COLOR_EXP
                    };

                    Rectangle::new(Point::new(bar_x, bar_y), Size::new(fill_width, 8))
                        .into_styled(PrimitiveStyle::with_fill(bar_color))
                        .draw(display)?;
                }

                // Status/Claim button
                if active_quest.completed {
                    // Claim button
                    Rectangle::new(Point::new(250, card_y + 10), Size::new(98, 60))
                        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(50, 100, 50)))
                        .draw(display)?;

                    Rectangle::new(Point::new(250, card_y + 10), Size::new(98, 60))
                        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 2))
                        .draw(display)?;

                    draw_text(
                        display,
                        "CLAIM",
                        Point::new(262, card_y + 45),
                        &FONT_9X18_BOLD,
                        Rgb888::WHITE,
                    )?;
                } else {
                    // In progress indicator
                    draw_text(
                        display,
                        "In Progress",
                        Point::new(260, card_y + 35),
                        &FONT_9X15,
                        COLOR_TEXT_DIM,
                    )?;
                }

                card_y += card_height as i32 + 8;
            }
        }

        // Scroll indicator if needed
        if active_quests.len() > 4 {
            let mut scroll_str = String::<32>::new();
            write!(
                scroll_str,
                "{}-{} of {}",
                start_index + 1,
                (start_index + 4).min(active_quests.len()),
                active_quests.len()
            )
            .ok();
            draw_text(
                display,
                &scroll_str,
                Point::new(140, 390),
                &FONT_9X15,
                COLOR_TEXT_DIM,
            )?;
        }
    }

    // Navigation buttons at bottom
    // Up arrow button (left) - only show if not at start
    if game_state.quest_page_scroll > 0 {
        Rectangle::new(Point::new(10, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
            .draw(display)?;

        Rectangle::new(Point::new(10, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
            .draw(display)?;

        // Draw up arrow (^)
        draw_text(
            display,
            "^",
            Point::new(35, 422),
            &FONT_10X20,
            COLOR_TEXT,
        )?;
        draw_text(
            display,
            "UP",
            Point::new(28, 434),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    // Back button (center)
    Rectangle::new(Point::new(134, 400), Size::new(100, 40))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;

    Rectangle::new(Point::new(134, 400), Size::new(100, 40))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    draw_text(
        display,
        "BACK",
        Point::new(153, 422),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Down arrow button (right) - only show if more quests below
    if !active_quests.is_empty() && (game_state.quest_page_scroll as usize + 4) < active_quests.len() {
        Rectangle::new(Point::new(288, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
            .draw(display)?;

        Rectangle::new(Point::new(288, 400), Size::new(70, 40))
            .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
            .draw(display)?;

        // Draw down arrow (v)
        draw_text(
            display,
            "v",
            Point::new(313, 422),
            &FONT_10X20,
            COLOR_TEXT,
        )?;
        draw_text(
            display,
            "DOWN",
            Point::new(297, 434),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
    }

    Ok(())
}

/// Draw the settings page with brightness slider
pub fn draw_settings_page<D>(
    display: &mut D,
    game_state: &GameState,
    battery_mv: u16,
    battery_pct: u8,
    fps: u32,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear display
    display.clear(COLOR_BG)?;

    draw_text(
        display,
        "=== SETTINGS ===",
        Point::new(85, 20),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Brightness section
    draw_text(
        display,
        "Brightness",
        Point::new(130, 100),
        &FONT_10X20,
        COLOR_TEXT,
    )?;

    // Brightness value display
    let mut brightness_str = String::<16>::new();
    write!(
        brightness_str,
        "{}%",
        (game_state.brightness as u32 * 100) / 255
    )
    .ok();
    draw_text(
        display,
        &brightness_str,
        Point::new(155, 130),
        &FONT_9X18_BOLD,
        Rgb888::YELLOW,
    )?;

    // Slider track (background bar)
    Rectangle::new(Point::new(40, 180), Size::new(280, 20))
        .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
        .draw(display)?;

    Rectangle::new(Point::new(40, 180), Size::new(280, 20))
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 2))
        .draw(display)?;

    // Slider filled portion (represents current brightness)
    let filled_width = ((game_state.brightness as u32 * 280) / 255) as u32;
    if filled_width > 0 {
        Rectangle::new(Point::new(40, 180), Size::new(filled_width, 20))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::YELLOW))
            .draw(display)?;
    }

    // Slider handle (indicator)
    let handle_x = 40 + ((game_state.brightness as i32 * 280) / 255);
    EgCircle::new(Point::new(handle_x - 8, 172), 16)
        .into_styled(PrimitiveStyle::with_fill(COLOR_MENU_SELECT))
        .draw(display)?;

    EgCircle::new(Point::new(handle_x - 8, 172), 16)
        .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT, 3))
        .draw(display)?;

    // Instructions
    draw_text(
        display,
        "Touch slider to adjust",
        Point::new(70, 250),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Brightness range labels
    draw_text(
        display,
        "0%",
        Point::new(35, 210),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    draw_text(
        display,
        "100%",
        Point::new(290, 210),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // System Info section at bottom (Battery and FPS)
    draw_text(
        display,
        "System Info",
        Point::new(125, 360),
        &FONT_9X18_BOLD,
        COLOR_TEXT_DIM,
    )?;

    // Battery info
    draw_battery_info(display, Point::new(20, 380), battery_mv, battery_pct)?;

    // FPS info (right side)
    draw_fps_info(display, Point::new(230, 380), fps)?;

    // Footer
    draw_text(
        display,
        "Touch bottom to go back",
        Point::new(65, 420),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    Ok(())
}

/// Helper: Draw monster GIF animation
fn draw_monster_gif<D>(
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
fn draw_map_monster_gif<D>(
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
fn draw_hero_gif<D>(
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

/// Helper: Draw monster attacked GIF animation (24.gif when hero attacks)
fn draw_monster_attacked_gif<D>(
    display: &mut D,
    game_state: &GameState,
    center_position: Point,
    monster_name: &str,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::tamagotchi::models::{MonsterAttackedAnimation, get_monster_attacked_gif};

    if game_state.monster_attacked_animation == MonsterAttackedAnimation::Normal {
        // No attacked animation, draw normal monster
        return draw_monster_gif(display, game_state, center_position, monster_name);
    }

    // Draw attacked animation (24.gif) for specific monster
    let gif_data = get_monster_attacked_gif(monster_name);
    let gif = Gif::<Rgb888>::from_slice(gif_data).expect("Failed to parse monster attacked GIF");

    // Get GIF dimensions to calculate centered position
    let gif_width = gif.width() as i32;
    let gif_height = gif.height() as i32;

    // Calculate top-left position to center the GIF at center_position
    let top_left = Point::new(
        center_position.x - (gif_width / 2),
        center_position.y - (gif_height / 2),
    );

    // Get current frame
    let frame_index = game_state.monster_attacked_frame;
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
fn draw_fps_info<D>(display: &mut D, position: Point, fps: u32) -> Result<(), D::Error>
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
fn draw_text<D>(
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
fn draw_bar<D>(
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
fn draw_battery_info<D>(
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

/// Draw JRPG turn-based battle page
pub fn draw_jrpg_battle_page<D>(
    display: &mut D,
    game_state: &GameState,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    use crate::tamagotchi::models::{JrpgBattleState, JrpgBattleMenu, CombatResult};

    display.clear(COLOR_BG)?;

    // Get combatants
    let hero = game_state.jrpg_hero_combatant.as_ref();
    let enemy = game_state.jrpg_enemy_combatant.as_ref();

    if hero.is_none() || enemy.is_none() {
        draw_text(display, "Battle Error!", Point::new(100, 224), &FONT_10X20, Rgb888::RED)?;
        return Ok(());
    }

    let hero = hero.unwrap();
    let enemy = enemy.unwrap();

    // === TOP: Enemy Info ===
    // Enemy name
    draw_text(display, enemy.name, Point::new(20, 20), &FONT_10X20, COLOR_TEXT)?;

    // Enemy level
    let mut enemy_level_str = String::<16>::new();
    write!(enemy_level_str, "Lv.{}", enemy.level).ok();
    draw_text(display, &enemy_level_str, Point::new(20, 45), &FONT_9X15, COLOR_TEXT_DIM)?;

    // Enemy HP bar
    draw_text(display, "HP:", Point::new(140, 45), &FONT_9X15, COLOR_TEXT_DIM)?;
    let enemy_hp_percent = (enemy.hp as u32 * 100) / enemy.max_hp as u32;
    let enemy_hp_color = if enemy_hp_percent > 50 {
        Rgb888::GREEN
    } else if enemy_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };
    draw_bar(display, Point::new(180, 45), 150, enemy_hp_percent as u8, enemy_hp_color)?;

    // Enemy HP value
    let mut enemy_hp_str = String::<32>::new();
    write!(enemy_hp_str, "{}/{}", enemy.hp, enemy.max_hp).ok();
    draw_text(display, &enemy_hp_str, Point::new(180, 65), &FONT_9X15, enemy_hp_color)?;

    // === CENTER: Battle GIFs ===
    // Draw enemy GIF (left side)
    draw_monster_gif(display, game_state, Point::new(80, 150), enemy.name)?;

    // Draw hero GIF (right side)
    draw_hero_gif(display, game_state, Point::new(240, 150))?;

    // Draw monster attacked overlay if active
    if game_state.monster_attacked_animation != crate::tamagotchi::models::MonsterAttackedAnimation::Normal {
        draw_monster_attacked_gif(display, game_state, Point::new(80, 150), enemy.name)?;
    }

    // === BOTTOM: Hero Info ===
    // Hero name and level
    let mut hero_info = String::<32>::new();
    write!(hero_info, "{} Lv.{}", hero.name, hero.level).ok();
    draw_text(display, &hero_info, Point::new(20, 250), &FONT_9X18_BOLD, COLOR_TEXT)?;

    // Hero HP
    draw_text(display, "HP:", Point::new(20, 275), &FONT_9X15, COLOR_TEXT_DIM)?;
    let hero_hp_percent = (hero.hp as u32 * 100) / hero.max_hp as u32;
    let hero_hp_color = if hero_hp_percent > 50 {
        Rgb888::GREEN
    } else if hero_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };
    draw_bar(display, Point::new(60, 275), 130, hero_hp_percent as u8, hero_hp_color)?;

    let mut hero_hp_str = String::<32>::new();
    write!(hero_hp_str, "{}/{}", hero.hp, hero.max_hp).ok();
    draw_text(display, &hero_hp_str, Point::new(60, 295), &FONT_9X15, hero_hp_color)?;

    // Hero SP
    draw_text(display, "SP:", Point::new(200, 275), &FONT_9X15, COLOR_TEXT_DIM)?;
    let hero_sp_percent = (hero.sp as u32 * 100) / hero.max_sp as u32;
    draw_bar(display, Point::new(240, 275), 110, hero_sp_percent as u8, Rgb888::CYAN)?;

    let mut hero_sp_str = String::<32>::new();
    write!(hero_sp_str, "{}/{}", hero.sp, hero.max_sp).ok();
    draw_text(display, &hero_sp_str, Point::new(240, 295), &FONT_9X15, Rgb888::CYAN)?;

    // === Battle Message (if any) ===
    if let Some(msg) = game_state.jrpg_battle_message {
        // Message box background
        Rectangle::new(Point::new(60, 105), Size::new(248, 35))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 60)))
            .draw(display)?;
        Rectangle::new(Point::new(60, 105), Size::new(248, 35))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 2))
            .draw(display)?;

        // Message text (centered)
        let text_x = 184 - ((msg.len() as i32 * 9) / 2); // Center text
        draw_text(display, msg, Point::new(text_x, 115), &FONT_9X18_BOLD, Rgb888::WHITE)?;
    }

    // === Floating Damage Text Animation ===
    if game_state.jrpg_damage_dealt > 0 && game_state.jrpg_damage_animation_timer > 0 {
        // Calculate animation progress (0.0 to 1.0)
        let progress = 1.0 - (game_state.jrpg_damage_animation_timer as f32 / 1000.0);

        // Float up by 40 pixels over the animation
        let float_offset = (progress * 40.0) as i32;
        let damage_y = game_state.jrpg_damage_y - float_offset;

        // Fade out alpha (simulate with color brightness)
        let alpha_factor = 1.0 - progress;

        // Color and text based on combat result
        let (damage_color, prefix) = match game_state.jrpg_last_combat_result {
            CombatResult::Critical => {
                let red_value = (255.0 * alpha_factor) as u8;
                let yellow_value = (100.0 * alpha_factor) as u8;
                (Rgb888::new(red_value, yellow_value, 0), "CRIT! ")
            },
            CombatResult::Lucky => {
                let gold_value = (255.0 * alpha_factor) as u8;
                let yellow_value = (215.0 * alpha_factor) as u8;
                (Rgb888::new(gold_value, yellow_value, 0), "LUCKY! ")
            },
            CombatResult::Miss => {
                let gray_value = (150.0 * alpha_factor) as u8;
                (Rgb888::new(gray_value, gray_value, gray_value), "MISS!")
            },
            CombatResult::Normal => {
                let red_value = (255.0 * alpha_factor) as u8;
                (Rgb888::new(red_value, 0, 0), "")
            },
        };

        // Draw damage text
        let mut dmg_str = String::<24>::new();
        if game_state.jrpg_last_combat_result == CombatResult::Miss {
            write!(dmg_str, "{}", prefix).ok();
        } else {
            write!(dmg_str, "{}-{}", prefix, game_state.jrpg_damage_dealt).ok();
        }

        // Draw text centered on damage position
        let text_width = dmg_str.len() as i32 * 10; // FONT_10X20 width
        let text_x = game_state.jrpg_damage_x - (text_width / 2);
        draw_text(display, &dmg_str, Point::new(text_x, damage_y), &FONT_10X20, damage_color)?;
    }

    // === Combo Counter Display ===
    if game_state.jrpg_combo_count > 0 {
        let mut combo_str = String::<16>::new();
        if game_state.jrpg_combo_ready {
            write!(combo_str, "COMBO x{} READY!", game_state.jrpg_combo_count).ok();
            // Draw in bright orange
            draw_text(display, &combo_str, Point::new(80, 180), &FONT_10X20, Rgb888::new(255, 140, 0))?;
        } else {
            write!(combo_str, "COMBO x{}", game_state.jrpg_combo_count).ok();
            // Draw in yellow
            draw_text(display, &combo_str, Point::new(100, 180), &FONT_10X20, Rgb888::new(255, 255, 0))?;
        }
    }

    // === Action Menu (during player turn) ===
    if game_state.jrpg_battle_state == JrpgBattleState::PlayerTurn {
        match game_state.jrpg_battle_menu {
            JrpgBattleMenu::Main => {
                // Main menu: 3 buttons in a row (Attack, Skill, Run)
                let options = ["Attack", "Skill", "Run"];
                let button_width = 110;
                let button_height = 60;
                let spacing_x = 12;
                let start_x = 14;
                let start_y = 360;

                for (i, option) in options.iter().enumerate() {
                    let x = start_x + i as i32 * (button_width + spacing_x);
                    let y = start_y;

                    let is_selected = game_state.jrpg_menu_selection == i as u8;

                    // Button background
                    let bg_color = if is_selected {
                        Rgb888::new(80, 80, 120) // Highlighted
                    } else {
                        Rgb888::new(50, 50, 80) // Normal
                    };

                    Rectangle::new(Point::new(x, y), Size::new(button_width as u32, button_height as u32))
                        .into_styled(PrimitiveStyle::with_fill(bg_color))
                        .draw(display)?;

                    // Button border
                    let border_color = if is_selected {
                        Rgb888::YELLOW
                    } else {
                        COLOR_TEXT
                    };
                    let border_width = if is_selected { 3 } else { 2 };

                    Rectangle::new(Point::new(x, y), Size::new(button_width as u32, button_height as u32))
                        .into_styled(PrimitiveStyle::with_stroke(border_color, border_width))
                        .draw(display)?;

                    // Button text (centered)
                    let text_color = if is_selected { Rgb888::YELLOW } else { Rgb888::WHITE };
                    let text_x = x + (button_width / 2) - ((option.len() as i32 * 9) / 2);
                    let text_y = y + (button_height / 2) - 9;
                    draw_text(display, option, Point::new(text_x, text_y), &FONT_9X18_BOLD, text_color)?;
                }
            }
            JrpgBattleMenu::Skills => {
                // Skills submenu - display available skills
                if let Some(hero) = &game_state.jrpg_hero_combatant {
                    let button_width = 340;
                    let button_height = 45;  // Reduced from 50
                    let spacing_y = 6;       // Reduced from 8
                    let start_x = 14;
                    let start_y = 220;       // Moved up from 250

                    // Draw skill buttons
                    for (i, skill) in hero.available_skills.iter().enumerate() {
                        let y = start_y + i as i32 * (button_height + spacing_y);
                        let is_selected = game_state.jrpg_skill_menu_selection == i as u8;
                        let has_enough_sp = hero.sp >= skill.sp_cost;

                        // Button background
                        let bg_color = if !has_enough_sp {
                            Rgb888::new(40, 40, 40) // Disabled (not enough SP)
                        } else if is_selected {
                            Rgb888::new(80, 80, 120) // Highlighted
                        } else {
                            Rgb888::new(50, 50, 80) // Normal
                        };

                        Rectangle::new(Point::new(start_x, y), Size::new(button_width as u32, button_height as u32))
                            .into_styled(PrimitiveStyle::with_fill(bg_color))
                            .draw(display)?;

                        // Button border
                        let border_color = if !has_enough_sp {
                            Rgb888::new(100, 100, 100) // Gray for disabled
                        } else if is_selected {
                            Rgb888::YELLOW
                        } else {
                            COLOR_TEXT
                        };
                        let border_width = if is_selected { 3 } else { 2 };

                        Rectangle::new(Point::new(start_x, y), Size::new(button_width as u32, button_height as u32))
                            .into_styled(PrimitiveStyle::with_stroke(border_color, border_width))
                            .draw(display)?;

                        // Skill name (left side)
                        let text_color = if !has_enough_sp {
                            Rgb888::new(120, 120, 120) // Dim gray for disabled
                        } else if is_selected {
                            Rgb888::YELLOW
                        } else {
                            Rgb888::WHITE
                        };
                        let text_x = start_x + 10;
                        let text_y = y + (button_height / 2) - 9;
                        draw_text(display, skill.name, Point::new(text_x, text_y), &FONT_9X18_BOLD, text_color)?;

                        // SP cost (right side)
                        let mut sp_str = String::<16>::new();
                        write!(sp_str, "SP: {}", skill.sp_cost).ok();
                        let sp_x = start_x + button_width - 80;
                        let sp_color = if !has_enough_sp {
                            Rgb888::RED // Red if not enough SP
                        } else {
                            Rgb888::new(100, 200, 255) // Cyan
                        };
                        draw_text(display, &sp_str, Point::new(sp_x, text_y), &FONT_9X18_BOLD, sp_color)?;
                    }

                    // Draw "Back" button
                    let back_y = start_y + (hero.available_skills.len() as i32) * (button_height + spacing_y);
                    let is_back_selected = game_state.jrpg_skill_menu_selection == hero.available_skills.len() as u8;

                    let back_bg_color = if is_back_selected {
                        Rgb888::new(80, 80, 120)
                    } else {
                        Rgb888::new(50, 50, 80)
                    };

                    Rectangle::new(Point::new(start_x, back_y), Size::new(button_width as u32, button_height as u32))
                        .into_styled(PrimitiveStyle::with_fill(back_bg_color))
                        .draw(display)?;

                    let back_border_color = if is_back_selected {
                        Rgb888::YELLOW
                    } else {
                        COLOR_TEXT
                    };
                    let back_border_width = if is_back_selected { 3 } else { 2 };

                    Rectangle::new(Point::new(start_x, back_y), Size::new(button_width as u32, button_height as u32))
                        .into_styled(PrimitiveStyle::with_stroke(back_border_color, back_border_width))
                        .draw(display)?;

                    let back_text_color = if is_back_selected { Rgb888::YELLOW } else { Rgb888::WHITE };
                    let back_text_x = start_x + (button_width / 2) - 27; // Center "Back"
                    let back_text_y = back_y + (button_height / 2) - 9;
                    draw_text(display, "Back", Point::new(back_text_x, back_text_y), &FONT_9X18_BOLD, back_text_color)?;

                    // Display current SP at top
                    let mut sp_display = String::<32>::new();
                    write!(sp_display, "SP: {}/{}", hero.sp, hero.max_sp).ok();
                    draw_text(display, &sp_display, Point::new(130, 190), &FONT_9X18_BOLD, Rgb888::new(100, 200, 255))?;
                }
            }
        }
    }

    // Battle end states (Victory/Defeat/Escaped) are handled by automatic transition
    // No modal messages needed - user can see result through animations and returning to map

    Ok(())
}
