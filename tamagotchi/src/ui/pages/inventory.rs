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

