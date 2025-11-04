use core::fmt::Write;
use embedded_graphics::{
    image::Image,
    mono_font::ascii::{FONT_9X15, FONT_9X18_BOLD, FONT_10X20},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;
use tinygif::Gif;

use super::super::helpers::*;
use crate::core::GameState;

use super::super::colors::*;

// Background image
const BACKGROUND_GIF: &[u8] = include_bytes!("../../../assets/images/ui/background.gif");

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
    display.clear(Rgb888::new(0, 0, 0))?;

    // Draw background image (single frame GIF)
    let bg_gif = Gif::<Rgb888>::from_slice(BACKGROUND_GIF).expect("Failed to parse background GIF");
    if let Some(frame) = bg_gif.frames().next() {
        Image::new(&frame, Point::new(0, 0)).draw(display)?;
    }

    let base_y = 20;

    // === LEFT HALF: STATS PANEL (HP/SP/EXP) ===
    let left_panel_x = 10;
    let left_panel_y = base_y;
    let panel_width = 165;

    // Stats panel background
    Rectangle::new(Point::new(left_panel_x, left_panel_y), Size::new(panel_width, 200))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 45)))
        .draw(display)?;
    Rectangle::new(Point::new(left_panel_x, left_panel_y), Size::new(panel_width, 200))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(80, 100, 130), 2))
        .draw(display)?;

    // HP Bar
    draw_text(
        display,
        "HP",
        Point::new(left_panel_x + 10, left_panel_y + 20),
        &FONT_9X15,
        COLOR_HP,
    )?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", hero.hp, hero.max_hp).ok();
    draw_text(
        display,
        &hp_str,
        Point::new(left_panel_x + 10, left_panel_y + 40),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(left_panel_x + 10, left_panel_y + 50),
        panel_width - 20,
        hero.hp_percent(),
        COLOR_HP,
    )?;

    // SP Bar
    draw_text(
        display,
        "SP",
        Point::new(left_panel_x + 10, left_panel_y + 80),
        &FONT_9X15,
        COLOR_SP,
    )?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", hero.sp, hero.max_sp).ok();
    draw_text(
        display,
        &sp_str,
        Point::new(left_panel_x + 10, left_panel_y + 100),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(left_panel_x + 10, left_panel_y + 110),
        panel_width - 20,
        hero.sp_percent(),
        COLOR_SP,
    )?;

    // EXP Bar
    draw_text(
        display,
        "EXP",
        Point::new(left_panel_x + 10, left_panel_y + 140),
        &FONT_9X15,
        COLOR_EXP,
    )?;
    let mut exp_str = String::<32>::new();
    write!(exp_str, "{}/{}", hero.exp, hero.exp_to_next_level).ok();
    draw_text(
        display,
        &exp_str,
        Point::new(left_panel_x + 10, left_panel_y + 160),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(left_panel_x + 10, left_panel_y + 170),
        panel_width - 20,
        hero.exp_percent(),
        COLOR_EXP,
    )?;

    // === RIGHT HALF: HERO INFO (Name, Job, Level, Zeny) ===
    let right_panel_x = 193;
    let right_panel_y = base_y;

    // Hero name with background
    let mut name_str = String::<32>::new();
    write!(name_str, "{}", hero.name).ok();
    Rectangle::new(Point::new(right_panel_x, right_panel_y), Size::new(panel_width, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 30, 60)))
        .draw(display)?;
    draw_text(
        display,
        &name_str,
        Point::new(right_panel_x + 10, right_panel_y + 18),
        &FONT_9X18_BOLD,
        Rgb888::new(255, 230, 150),
    )?;

    // Job with background
    let mut job_str = String::<32>::new();
    write!(job_str, "{}", hero.job).ok();
    Rectangle::new(Point::new(right_panel_x, right_panel_y + 40), Size::new(panel_width, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 40, 50)))
        .draw(display)?;
    draw_text(
        display,
        &job_str,
        Point::new(right_panel_x + 10, right_panel_y + 58),
        &FONT_9X18_BOLD,
        Rgb888::new(180, 200, 220),
    )?;

    // Level with background
    let mut lvl_str = String::<32>::new();
    write!(lvl_str, "Level {}", hero.level).ok();
    Rectangle::new(Point::new(right_panel_x, right_panel_y + 80), Size::new(panel_width, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 50, 40)))
        .draw(display)?;
    draw_text(
        display,
        &lvl_str,
        Point::new(right_panel_x + 10, right_panel_y + 98),
        &FONT_9X18_BOLD,
        Rgb888::new(150, 255, 150),
    )?;

    // Zeny with background
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "{} z", hero.zeny).ok();
    Rectangle::new(Point::new(right_panel_x, right_panel_y + 120), Size::new(panel_width, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 50, 20)))
        .draw(display)?;
    draw_text(
        display,
        &zeny_str,
        Point::new(right_panel_x + 10, right_panel_y + 138),
        &FONT_9X18_BOLD,
        Rgb888::new(255, 215, 0),
    )?;

    // === BOTTOM: HERO IMAGE WITH EQUIPMENT ===
    let hero_bottom_y = 410;

    // Hero GIF (centered at bottom)
    draw_hero_gif(display, game_state, Point::new(184, hero_bottom_y))?;

    // Helper function to get tier color based on level requirement
    let get_tier_color = |level_req: u16| -> Rgb888 {
        if level_req >= 41 {
            Rgb888::new(255, 165, 0)  // Legendary - Orange/Gold
        } else if level_req >= 31 {
            Rgb888::new(163, 53, 238)  // Epic - Purple
        } else if level_req >= 21 {
            Rgb888::new(64, 156, 255)  // Rare - Blue
        } else if level_req >= 11 {
            Rgb888::new(30, 255, 30)   // Uncommon - Green
        } else {
            Rgb888::new(180, 180, 180) // Common - Gray
        }
    };

    // LEFT SIDE EQUIPMENT (top to bottom)
    let left_x = 5;
    let equip_bg_color = Rgb888::new(20, 25, 35);
    let equip_height = 40; // Height for two lines
    let equip_spacing = 45; // Spacing between equipment slots

    // Weapon (top left)
    if hero.equipped_weapon.id != 0 {
        let y_pos = hero_bottom_y - 150;
        Rectangle::new(Point::new(left_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_fill(equip_bg_color))
            .draw(display)?;
        Rectangle::new(Point::new(left_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_stroke(get_tier_color(hero.equipped_weapon.level_req), 1))
            .draw(display)?;
        draw_text(
            display,
            hero.equipped_weapon.name,
            Point::new(left_x + 5, y_pos + 12),
            &FONT_9X15,
            get_tier_color(hero.equipped_weapon.level_req),
        )?;
        // Second line: refinement and ATK
        let mut stats_str = String::<16>::new();
        write!(stats_str, "+{} ATK:{}", hero.equipped_weapon.refine_level, hero.equipped_weapon.atk_bonus).ok();
        draw_text(
            display,
            &stats_str,
            Point::new(left_x + 5, y_pos + 27),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Shoes (middle left)
    if hero.equipped_shoes.id != 0 {
        let y_pos = hero_bottom_y - 105;
        Rectangle::new(Point::new(left_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_fill(equip_bg_color))
            .draw(display)?;
        Rectangle::new(Point::new(left_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_stroke(get_tier_color(hero.equipped_shoes.level_req), 1))
            .draw(display)?;
        draw_text(
            display,
            hero.equipped_shoes.name,
            Point::new(left_x + 5, y_pos + 12),
            &FONT_9X15,
            get_tier_color(hero.equipped_shoes.level_req),
        )?;
        // Second line: refinement and DEF
        let mut stats_str = String::<16>::new();
        write!(stats_str, "+{} DEF:{}", hero.equipped_shoes.refine_level, hero.equipped_shoes.def_bonus).ok();
        draw_text(
            display,
            &stats_str,
            Point::new(left_x + 5, y_pos + 27),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Accessory 1 (bottom left)
    if hero.equipped_accessory1.id != 0 {
        let y_pos = hero_bottom_y - 60;
        Rectangle::new(Point::new(left_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_fill(equip_bg_color))
            .draw(display)?;
        Rectangle::new(Point::new(left_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_stroke(get_tier_color(hero.equipped_accessory1.level_req), 1))
            .draw(display)?;
        draw_text(
            display,
            hero.equipped_accessory1.name,
            Point::new(left_x + 5, y_pos + 12),
            &FONT_9X15,
            get_tier_color(hero.equipped_accessory1.level_req),
        )?;
        // Second line: refinement
        let mut stats_str = String::<16>::new();
        write!(stats_str, "+{}", hero.equipped_accessory1.refine_level).ok();
        draw_text(
            display,
            &stats_str,
            Point::new(left_x + 5, y_pos + 27),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // RIGHT SIDE EQUIPMENT (top to bottom)
    let right_x = 248;

    // Armor (top right)
    if hero.equipped_armor.id != 0 {
        let y_pos = hero_bottom_y - 150;
        Rectangle::new(Point::new(right_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_fill(equip_bg_color))
            .draw(display)?;
        Rectangle::new(Point::new(right_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_stroke(get_tier_color(hero.equipped_armor.level_req), 1))
            .draw(display)?;
        draw_text(
            display,
            hero.equipped_armor.name,
            Point::new(right_x + 5, y_pos + 12),
            &FONT_9X15,
            get_tier_color(hero.equipped_armor.level_req),
        )?;
        // Second line: refinement and DEF
        let mut stats_str = String::<16>::new();
        write!(stats_str, "+{} DEF:{}", hero.equipped_armor.refine_level, hero.equipped_armor.def_bonus).ok();
        draw_text(
            display,
            &stats_str,
            Point::new(right_x + 5, y_pos + 27),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Garment (middle right)
    if hero.equipped_garment.id != 0 {
        let y_pos = hero_bottom_y - 105;
        Rectangle::new(Point::new(right_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_fill(equip_bg_color))
            .draw(display)?;
        Rectangle::new(Point::new(right_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_stroke(get_tier_color(hero.equipped_garment.level_req), 1))
            .draw(display)?;
        draw_text(
            display,
            hero.equipped_garment.name,
            Point::new(right_x + 5, y_pos + 12),
            &FONT_9X15,
            get_tier_color(hero.equipped_garment.level_req),
        )?;
        // Second line: refinement and DEF
        let mut stats_str = String::<16>::new();
        write!(stats_str, "+{} DEF:{}", hero.equipped_garment.refine_level, hero.equipped_garment.def_bonus).ok();
        draw_text(
            display,
            &stats_str,
            Point::new(right_x + 5, y_pos + 27),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Accessory 2 (bottom right)
    if hero.equipped_accessory2.id != 0 {
        let y_pos = hero_bottom_y - 60;
        Rectangle::new(Point::new(right_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_fill(equip_bg_color))
            .draw(display)?;
        Rectangle::new(Point::new(right_x, y_pos), Size::new(115, equip_height))
            .into_styled(PrimitiveStyle::with_stroke(get_tier_color(hero.equipped_accessory2.level_req), 1))
            .draw(display)?;
        draw_text(
            display,
            hero.equipped_accessory2.name,
            Point::new(right_x + 5, y_pos + 12),
            &FONT_9X15,
            get_tier_color(hero.equipped_accessory2.level_req),
        )?;
        // Second line: refinement
        let mut stats_str = String::<16>::new();
        write!(stats_str, "+{}", hero.equipped_accessory2.refine_level).ok();
        draw_text(
            display,
            &stats_str,
            Point::new(right_x + 5, y_pos + 27),
            &FONT_9X15,
            Rgb888::new(150, 150, 150),
        )?;
    }

    // Save status message (if any)
    if let Some(msg) = save_msg {
        draw_text(
            display,
            msg,
            Point::new(184 - (msg.len() as i32 * 5), 270),
            &FONT_9X18_BOLD,
            Rgb888::new(100, 255, 100),
        )?;
    }

    Ok(())
}
