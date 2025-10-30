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
use super::super::helpers::*;

use super::super::colors::*;

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
    // Uniform button color - no multicolor
    let button_color = Rgb888::new(60, 80, 120);
    let button_border = Rgb888::new(100, 120, 160);

    // Row 1: Rest, Stats
    // Rest button (top left)
    Rectangle::new(Point::new(14, 350), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_fill(button_color))
        .draw(display)?;
    Rectangle::new(Point::new(14, 350), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_stroke(button_border, 2))
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
        .into_styled(PrimitiveStyle::with_fill(button_color))
        .draw(display)?;
    Rectangle::new(Point::new(189, 350), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_stroke(button_border, 2))
        .draw(display)?;
    draw_text(
        display,
        "Stats",
        Point::new(245, 368),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    // Row 2: Equipment, Quests
    // Equipment button (bottom left)
    Rectangle::new(Point::new(14, 403), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_fill(button_color))
        .draw(display)?;
    Rectangle::new(Point::new(14, 403), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_stroke(button_border, 2))
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
        .into_styled(PrimitiveStyle::with_fill(button_color))
        .draw(display)?;
    Rectangle::new(Point::new(189, 403), Size::new(165, 45))
        .into_styled(PrimitiveStyle::with_stroke(button_border, 2))
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

