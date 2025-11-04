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

// Background image
const BACKGROUND_GIF: &[u8] = include_bytes!("../../../assets/images/ui/background.gif");

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
    // Clear background
    display.clear(Rgb888::new(0, 0, 0))?;

    // Draw background image (single frame GIF)
    let bg_gif = Gif::<Rgb888>::from_slice(BACKGROUND_GIF).expect("Failed to parse background GIF");
    if let Some(frame) = bg_gif.frames().next() {
        Image::new(&frame, Point::new(0, 0)).draw(display)?;
    }

    // Title with background
    Rectangle::new(Point::new(90, 20), Size::new(180, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 30, 60)))
        .draw(display)?;
    draw_text(
        display,
        "=== RESTING ===",
        Point::new(100, 38),
        &FONT_10X20,
        Rgb888::new(255, 230, 150),
    )?;

    // HP Recovery panel with background
    let hp_panel_y = 70;
    Rectangle::new(Point::new(10, hp_panel_y), Size::new(348, 90))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 45)))
        .draw(display)?;
    Rectangle::new(Point::new(10, hp_panel_y), Size::new(348, 90))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(80, 100, 130), 2))
        .draw(display)?;

    draw_text(
        display,
        "HP Recovery",
        Point::new(20, hp_panel_y + 20),
        &FONT_9X18_BOLD,
        COLOR_HP,
    )?;
    let mut hp_str = String::<32>::new();
    write!(hp_str, "{}/{}", game_state.hero.hp, game_state.hero.max_hp).ok();
    draw_text(
        display,
        &hp_str,
        Point::new(20, hp_panel_y + 40),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(20, hp_panel_y + 50),
        328,
        game_state.hero.hp_percent(),
        COLOR_HP,
    )?;

    // HP Regen rate
    draw_text(
        display,
        "+10 HP/sec",
        Point::new(20, hp_panel_y + 70),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // SP Recovery panel with background
    let sp_panel_y = 170;
    Rectangle::new(Point::new(10, sp_panel_y), Size::new(348, 90))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(20, 30, 45)))
        .draw(display)?;
    Rectangle::new(Point::new(10, sp_panel_y), Size::new(348, 90))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(80, 100, 130), 2))
        .draw(display)?;

    draw_text(
        display,
        "SP Recovery",
        Point::new(20, sp_panel_y + 20),
        &FONT_9X18_BOLD,
        COLOR_SP,
    )?;
    let mut sp_str = String::<32>::new();
    write!(sp_str, "{}/{}", game_state.hero.sp, game_state.hero.max_sp).ok();
    draw_text(
        display,
        &sp_str,
        Point::new(20, sp_panel_y + 40),
        &FONT_9X15,
        Rgb888::WHITE,
    )?;
    draw_bar(
        display,
        Point::new(20, sp_panel_y + 50),
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
        Point::new(20, sp_panel_y + 70),
        &FONT_9X15,
        COLOR_TEXT_DIM,
    )?;

    // Status message with background
    let status_y = 280;
    let status_text = match game_state.rest_state {
        RestState::Resting => "Recovering HP & SP...",
        RestState::FullSP => "Fully Recovered!",
        RestState::Complete => "Rest Complete",
    };

    Rectangle::new(Point::new(65, status_y), Size::new(238, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 50, 40)))
        .draw(display)?;
    draw_text(
        display,
        status_text,
        Point::new(75, status_y + 18),
        &FONT_9X18_BOLD,
        Rgb888::new(150, 255, 150),
    )?;

    // Hero resting GIF animation at bottom (like overview)
    draw_hero_gif(display, game_state, Point::new(184, 410))?;

    Ok(())
}

