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

    // Hero resting GIF animation (16.gif) - centered and lowered
    draw_hero_gif(display, game_state, Point::new(180, 120))?;

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
        RestState::Complete => {
            draw_text(
                display,
                "Rest Complete",
                Point::new(90, 330),
                &FONT_9X18_BOLD,
                COLOR_TEXT,
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

