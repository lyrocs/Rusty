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

