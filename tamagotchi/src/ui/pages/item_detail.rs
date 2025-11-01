/// Item Detail Page
///
/// Shows detailed information about a selected inventory item.

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

use crate::core::GameState;
use super::super::helpers::*;
use super::super::colors::*;

/// Draw the item detail page
pub fn draw_item_detail_page<D>(display: &mut D, game_state: &GameState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Rgb888>,
{
    // Clear display
    display.clear(COLOR_BG)?;

    // Draw farming header if active
    use crate::ui::farming_header::draw_farming_header;
    let has_farming_header = draw_farming_header(display, game_state)?;
    let title_y = if has_farming_header { 40 } else { 20 };

    // Get the selected item
    let item_id = match game_state.selected_item_id {
        Some(id) => id,
        None => {
            // No item selected - shouldn't happen
            draw_text(
                display,
                "No item selected",
                Point::new(90, 200),
                &FONT_10X20,
                COLOR_TEXT_DIM,
            )?;
            return Ok(());
        }
    };

    // Find the item in inventory
    let item = game_state.hero.inventory.iter().find(|i| i.id == item_id);

    if let Some(item) = item {
        // Title
        draw_text(
            display,
            "=== ITEM DETAILS ===",
            Point::new(60, title_y),
            &FONT_10X20,
            COLOR_TEXT,
        )?;

        // Item icon (large, centered)
        let icon_x = 150;
        let icon_y = title_y + 40;

        let icon_loaded = (|| -> Result<(), ()> {
            use crate::data::items::get_item_icon;

            // Get compiled-in icon data
            let gif_data = get_item_icon(item.id).ok_or(())?;
            let gif: Gif<Rgb888> = Gif::from_slice(gif_data).map_err(|_| ())?;

            // Draw the first frame of the GIF (scaled 2x by drawing larger)
            if let Some(frame) = gif.frames().next() {
                Image::new(&frame, Point::new(icon_x, icon_y)).draw(display).map_err(|_| ())?;
            }
            Ok(())
        })().is_ok();

        // Item name (below icon)
        draw_text(
            display,
            item.name,
            Point::new(180 - (item.name.len() as i32 * 5), icon_y + 45),
            &FONT_9X18_BOLD,
            COLOR_TEXT,
        )?;

        // Quantity
        let mut qty_text = String::<32>::new();
        write!(qty_text, "Quantity: {}", item.quantity).ok();
        draw_text(
            display,
            &qty_text,
            Point::new(120, icon_y + 75),
            &FONT_9X15,
            Rgb888::YELLOW,
        )?;

        // Item ID (for reference)
        let mut id_text = String::<32>::new();
        write!(id_text, "ID: {}", item.id).ok();
        draw_text(
            display,
            &id_text,
            Point::new(140, icon_y + 95),
            &FONT_9X15,
            COLOR_TEXT_DIM,
        )?;

        // Description section
        draw_text(
            display,
            "Description:",
            Point::new(20, icon_y + 130),
            &FONT_9X18_BOLD,
            COLOR_TEXT,
        )?;

        // Get item description based on ID
        let description = get_item_description(item.id);

        // Draw description (word-wrapped)
        let mut y = icon_y + 155;
        for line in description {
            draw_text(
                display,
                line,
                Point::new(20, y),
                &FONT_9X15,
                COLOR_TEXT,
            )?;
            y += 20;
        }

    } else {
        // Item not found in inventory
        draw_text(
            display,
            "Item not found",
            Point::new(100, 200),
            &FONT_10X20,
            COLOR_TEXT_DIM,
        )?;
    }

    // BACK button at bottom
    Rectangle::new(Point::new(100, 450), Size::new(160, 50))
        .into_styled(PrimitiveStyle::with_fill(Rgb888::new(60, 80, 120)))
        .draw(display)?;
    Rectangle::new(Point::new(100, 450), Size::new(160, 50))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(100, 120, 160), 2))
        .draw(display)?;
    draw_text(
        display,
        "BACK",
        Point::new(160, 480),
        &FONT_10X20,
        Rgb888::WHITE,
    )?;

    Ok(())
}

/// Get item description lines by ID
fn get_item_description(item_id: u32) -> &'static [&'static str] {
    match item_id {
        909 => &[
            "A sticky, transparent jelly",
            "dropped by Porings.",
            "",
            "Useless but sells well.",
        ],
        914 => &[
            "Soft, fluffy cotton-like",
            "substance from monsters.",
            "",
            "Used in crafting.",
        ],
        938 => &[
            "Sticky mucus that clings",
            "to everything.",
            "",
            "Crafting material.",
        ],
        939 => &[
            "A sharp stinger from a",
            "giant hornet.",
            "",
            "Used for poison items.",
        ],
        512 => &[
            "A fresh, crispy apple.",
            "",
            "Restores 15 HP.",
        ],
        _ => &[
            "A mysterious item.",
            "",
            "Its purpose is unknown.",
        ],
    }
}
