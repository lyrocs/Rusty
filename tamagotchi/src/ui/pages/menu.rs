use embedded_graphics::{
    image::Image,
    mono_font::ascii::{FONT_9X15, FONT_9X18_BOLD},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use tinygif::Gif;

use crate::core::GameState;
use super::super::helpers::*;

use super::super::colors::*;

// Menu background image
const MENU_GIF: &[u8] = include_bytes!("../../../assets/images/ui/menu.gif");

/// Draw the Menu overlay
pub fn draw_menu<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear background
    display.clear(Rgb888::new(0, 0, 0))?;

    // Draw menu background image (single frame GIF)
    let menu_bg_gif = Gif::<Rgb888>::from_slice(MENU_GIF).expect("Failed to parse menu GIF");
    if let Some(frame) = menu_bg_gif.frames().next() {
        Image::new(&frame, Point::new(0, 0)).draw(display)?;
    }

    // Menu items in 2 columns x 5 rows (10 items)
    // Farm and Battle are now accessed via Map page
    // Button size: 150x70 with 10px spacing
    let menu_items = ["Overview", "Stats", "Rest", "Equip", "Map", "Invent", "Quests", "Settings", "Save", "Debug"];

    for (i, item) in menu_items.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;

        // Calculate button position
        let x = 24 + col as i32 * 160; // 24px left margin, 160px spacing (150 button + 10 gap)
        let y = 40 + row as i32 * 80; // 40px top, 80px spacing (70 button + 10 gap)

        let is_selected = i as u8 == game_state.menu_selection;

        // Draw button with semi-transparent fill and bright border
        let button_color = if is_selected {
            Rgb888::new(60, 100, 140) // Brighter when selected
        } else {
            Rgb888::new(20, 40, 60) // Darker when not selected
        };

        Rectangle::new(Point::new(x, y), Size::new(150, 70))
            .into_styled(PrimitiveStyle::with_fill(button_color))
            .draw(display)?;

        // Draw button border (thicker if selected)
        let border_color = if is_selected {
            Rgb888::new(150, 200, 255) // Bright blue when selected
        } else {
            Rgb888::new(100, 140, 180) // Normal blue when not selected
        };
        let border_width = if is_selected { 3 } else { 2 };
        Rectangle::new(Point::new(x, y), Size::new(150, 70))
            .into_styled(PrimitiveStyle::with_stroke(border_color, border_width))
            .draw(display)?;

        // Draw text centered in button
        let text_color = if is_selected {
            Rgb888::WHITE
        } else {
            Rgb888::new(200, 200, 200)
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

    Ok(())
}

