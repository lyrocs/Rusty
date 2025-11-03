/// Zone-based rendering functions for Battle Overview page
/// Each function clears and redraws only its specific zone
use core::fmt::Write;
use embedded_graphics::{
    mono_font::ascii::{FONT_9X15, FONT_9X18_BOLD},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::String;

use crate::combat::{Enemy, IdleFarmSession};
use crate::ui::colors::*;
use crate::ui::helpers::*;

/// Draw enemy info zone (name, level, HP bar)
/// Zone: x=20-350, y=45-95
pub fn draw_enemy_info_zone<D>(
    display: &mut D,
    enemy: &Enemy,
    current_enemy_hp: u16,
    enemy_max_hp: u16,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let zone_x = 20;
    let zone_y = 25;
    let zone_width = 330;
    let zone_height = 60;

    // Clear zone
    Rectangle::new(
        Point::new(zone_x, zone_y),
        Size::new(zone_width, zone_height),
    )
    .into_styled(PrimitiveStyle::with_fill(COLOR_BG))
    .draw(display)?;

    // Enemy name with level
    let mut enemy_label = String::<32>::new();
    write!(enemy_label, "{} Lv{}", enemy.name, enemy.level).ok();
    draw_text(
        display,
        &enemy_label,
        Point::new(zone_x, zone_y + 10),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Enemy HP bar
    draw_text(
        display,
        "HP:",
        Point::new(zone_x, zone_y + 25),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    let enemy_hp_percent = if enemy_max_hp > 0 {
        (current_enemy_hp as u32 * 100) / enemy_max_hp as u32
    } else {
        0
    };

    let enemy_hp_color = if enemy_hp_percent > 50 {
        Rgb888::GREEN
    } else if enemy_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    draw_bar(
        display,
        Point::new(60, zone_y + 25),
        150,
        enemy_hp_percent as u8,
        enemy_hp_color,
    )?;

    // Enemy HP value
    let mut enemy_hp_str = String::<32>::new();
    write!(enemy_hp_str, "{}/{}", current_enemy_hp, enemy_max_hp).ok();
    draw_text(
        display,
        &enemy_hp_str,
        Point::new(220, zone_y + 25),
        &FONT_9X15,
        enemy_hp_color,
    )?;

    Ok(())
}

/// Draw hero info zone (name, level, HP bar)
/// Zone: x=20-200, y=280-330
pub fn draw_hero_info_zone<D>(
    display: &mut D,
    hero_name: &str,
    hero_level: u16,
    current_hp: u16,
    max_hp: u16,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let zone_x = 20;
    let zone_y = 280;
    let zone_width = 180;
    let zone_height = 75;

    // Clear zone
    Rectangle::new(
        Point::new(zone_x, zone_y),
        Size::new(zone_width, zone_height),
    )
    .into_styled(PrimitiveStyle::with_fill(COLOR_BG))
    .draw(display)?;

    // Hero name and level
    let mut hero_label = String::<32>::new();
    write!(hero_label, "{} Lv{}", hero_name, hero_level).ok();
    draw_text(
        display,
        &hero_label,
        Point::new(zone_x, zone_y),
        &FONT_9X18_BOLD,
        COLOR_TEXT,
    )?;

    // Hero HP
    draw_text(
        display,
        "HP:",
        Point::new(zone_x, zone_y + 25),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    let hero_hp_percent = if max_hp > 0 {
        (current_hp as u32 * 100) / max_hp as u32
    } else {
        0
    };

    let hero_hp_color = if hero_hp_percent > 50 {
        Rgb888::GREEN
    } else if hero_hp_percent > 25 {
        Rgb888::YELLOW
    } else {
        Rgb888::RED
    };

    draw_bar(
        display,
        Point::new(60, zone_y + 25),
        130,
        hero_hp_percent as u8,
        hero_hp_color,
    )?;

    let mut hero_hp_str = String::<32>::new();
    write!(hero_hp_str, "{}/{}", current_hp, max_hp).ok();
    draw_text(
        display,
        &hero_hp_str,
        Point::new(60, zone_y + 45),
        &FONT_9X15,
        hero_hp_color,
    )?;

    Ok(())
}

/// Draw session stats panel zone
/// Zone: x=200-358, y=280-400
pub fn draw_stats_panel_zone<D>(display: &mut D, session: &IdleFarmSession) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    let stats_panel_x = 200;
    let stats_panel_y = 280;
    let stats_panel_width = 158;
    let stats_panel_height = 120;

    // Clear and draw background panel
    Rectangle::new(
        Point::new(stats_panel_x, stats_panel_y),
        Size::new(stats_panel_width, stats_panel_height),
    )
    .into_styled(PrimitiveStyle::with_fill(COLOR_PANEL))
    .draw(display)?;

    Rectangle::new(
        Point::new(stats_panel_x, stats_panel_y),
        Size::new(stats_panel_width, stats_panel_height),
    )
    .into_styled(PrimitiveStyle::with_stroke(COLOR_TEXT_DIM, 2))
    .draw(display)?;

    // Session stats header
    draw_text(
        display,
        "Session Stats",
        Point::new(stats_panel_x + 10, stats_panel_y + 20),
        &FONT_9X15,
        COLOR_TEXT,
    )?;

    // Kills
    let mut kills_str = String::<32>::new();
    write!(kills_str, "Kills: {}", session.monsters_killed).ok();
    draw_text(
        display,
        &kills_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 40),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Zeny
    let mut zeny_str = String::<32>::new();
    write!(zeny_str, "Zeny: {}", session.zeny_earned).ok();
    draw_text(
        display,
        &zeny_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 60),
        &FONT_9X15,
        Rgb888::YELLOW,
    )?;

    // Exp
    let mut exp_str = String::<32>::new();
    write!(exp_str, "Exp: {}", session.exp_gained).ok();
    draw_text(
        display,
        &exp_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 80),
        &FONT_9X15,
        Rgb888::CYAN,
    )?;

    // Items
    let mut items_str = String::<32>::new();
    write!(items_str, "Items: {}", session.items_collected).ok();
    draw_text(
        display,
        &items_str,
        Point::new(stats_panel_x + 10, stats_panel_y + 100),
        &FONT_9X15,
        Rgb888::new(200, 150, 255),
    )?;

    Ok(())
}
