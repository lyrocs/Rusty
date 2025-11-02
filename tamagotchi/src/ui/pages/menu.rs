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

    // Menu items in 2 columns x 4 rows (7 items - last row has 1 item)
    // Farm and Battle are now accessed via Map page
    // Button size: 150x70 with 10px spacing
    let menu_items = ["Overview", "Rest", "Map", "Quests", "Settings", "Save", "Debug"];

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

