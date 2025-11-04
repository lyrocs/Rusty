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

// Menu background image
const MENU_GIF: &[u8] = include_bytes!("../../../assets/images/ui/menu.gif");

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
    display.clear(Rgb888::new(0, 0, 0))?;

    // Draw background image (single frame GIF)
    let menu_gif = Gif::<Rgb888>::from_slice(MENU_GIF).expect("Failed to parse menu GIF");
    if let Some(frame) = menu_gif.frames().next() {
        Image::new(&frame, Point::new(0, 0)).draw(display)?;
    }

    let title_y = 20;

    // Title with background
    let title_x = 90;
    Rectangle::new(Point::new(title_x, title_y), Size::new(230, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 30, 60)))
        .draw(display)?;
    draw_text(
        display,
        "=== STAT POINTS ===",
        Point::new(title_x + 10, title_y + 18),
        &FONT_10X20,
        Rgb888::new(255, 230, 150),
    )?;

    // Available stat points with background
    let mut points_str = String::<32>::new();
    write!(points_str, "Available: {}", hero.stat_points).ok();
    Rectangle::new(Point::new(90, 60), Size::new(180, 30))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(30, 50, 40)))
        .draw(display)?;
    draw_text(
        display,
        &points_str,
        Point::new(100, 78),
        &FONT_9X18_BOLD,
        Rgb888::new(150, 255, 150),
    )?;

    // Stat display and increase buttons (2 columns x 3 rows)
    let left_x = 15;
    let right_x = 195;
    let button_width = 165;
    let button_height = 70;
    let y_positions = [110, 185, 260];

    // Uniform button colors - no multicolor
    let active_button_color = Rgb888::new(60, 80, 120);
    let disabled_button_color = Rgb888::new(60, 60, 60);

    // Left column: STR, AGI, VIT
    let left_stats = [
        ("STR", hero.base_str),
        ("AGI", hero.base_agi),
        ("VIT", hero.base_vit),
    ];

    for (i, (stat_name, stat_value)) in left_stats.iter().enumerate() {
        let y = y_positions[i];

        // Button background
        let button_color = if hero.stat_points > 0 {
            active_button_color
        } else {
            disabled_button_color
        };

        Rectangle::new(Point::new(left_x, y), Size::new(button_width as u32, button_height))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        Rectangle::new(Point::new(left_x, y), Size::new(button_width as u32, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 120, 160), 2))
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
        ("INT", hero.base_int),
        ("DEX", hero.base_dex),
        ("LUK", hero.base_luk),
    ];

    for (i, (stat_name, stat_value)) in right_stats.iter().enumerate() {
        let y = y_positions[i];

        // Button background
        let button_color = if hero.stat_points > 0 {
            active_button_color
        } else {
            disabled_button_color
        };

        Rectangle::new(Point::new(right_x, y), Size::new(button_width as u32, button_height))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        Rectangle::new(Point::new(right_x, y), Size::new(button_width as u32, button_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 120, 160), 2))
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

    // Bottom buttons on same line (half width each)
    let bottom_y = 350;
    let bottom_button_width = 165;
    let bottom_button_height = 50;

    // Reset button (left half)
    Rectangle::new(Point::new(15, bottom_y), Size::new(bottom_button_width as u32, bottom_button_height))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(100, 60, 60)))
        .draw(display)?;
    Rectangle::new(Point::new(15, bottom_y), Size::new(bottom_button_width as u32, bottom_button_height))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(150, 80, 80), 2))
        .draw(display)?;
    draw_text(
        display,
        "Reset All",
        Point::new(40, bottom_y + 28),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    // Back button (right half)
    Rectangle::new(Point::new(195, bottom_y), Size::new(bottom_button_width as u32, bottom_button_height))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 120)))
        .draw(display)?;
    Rectangle::new(Point::new(195, bottom_y), Size::new(bottom_button_width as u32, bottom_button_height))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 120, 160), 2))
        .draw(display)?;
    draw_text(
        display,
        "Back",
        Point::new(250, bottom_y + 28),
        &FONT_9X18_BOLD,
        Rgb888::WHITE,
    )?;

    // Reset confirmation modal (if showing)
    if game_state.show_reset_confirm {
        // Semi-transparent overlay
        Rectangle::new(Point::new(0, 0), Size::new(360, 480))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 0, 0)))
            .draw(display)?;

        // Modal panel
        let modal_x = 40;
        let modal_y = 150;
        let modal_width = 280;
        let modal_height = 180;

        Rectangle::new(Point::new(modal_x, modal_y), Size::new(modal_width, modal_height))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(40, 40, 50)))
            .draw(display)?;
        Rectangle::new(Point::new(modal_x, modal_y), Size::new(modal_width, modal_height))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(150, 150, 170), 3))
            .draw(display)?;

        // Modal text
        draw_text(
            display,
            "Reset All Stats?",
            Point::new(modal_x + 50, modal_y + 35),
            &FONT_10X20,
            Rgb888::WHITE,
        )?;

        draw_text(
            display,
            "This will refund all",
            Point::new(modal_x + 45, modal_y + 70),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;
        draw_text(
            display,
            "spent stat points",
            Point::new(modal_x + 55, modal_y + 90),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;

        // Confirm button
        Rectangle::new(Point::new(modal_x + 20, modal_y + 120), Size::new(110, 40))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(150, 60, 60)))
            .draw(display)?;
        draw_text(
            display,
            "Confirm",
            Point::new(modal_x + 40, modal_y + 140),
            &FONT_9X18_BOLD,
            Rgb888::WHITE,
        )?;

        // Cancel button
        Rectangle::new(Point::new(modal_x + 150, modal_y + 120), Size::new(110, 40))
            .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 120)))
            .draw(display)?;
        draw_text(
            display,
            "Cancel",
            Point::new(modal_x + 175, modal_y + 140),
            &FONT_9X18_BOLD,
            Rgb888::WHITE,
        )?;
    }

    Ok(())
}

